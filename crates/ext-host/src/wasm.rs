
use crate::api::{AppApi, Code, Grants, Invocation, OpError, authorize};
use crate::jobs::{JobBoard, JobState, Ticket};
use crate::kernels;
use serde_json::Value;
use std::sync::Arc;
use wasmi::{Caller, Config, Engine, Instance, Linker, Memory, Module, Store, StoreLimits, StoreLimitsBuilder, TypedFunc, TypedResumableCall};

pub const ABI_VERSION: i32 = 1;
pub const MAX_MODULE_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_TRANSFORM_MEMORY: usize = 512 * 1024 * 1024;
pub const MAX_SESSION_MEMORY: usize = 64 * 1024 * 1024;
pub const MAX_CALL_BYTES: usize = 1024 * 1024;
pub const MAX_PENDING_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_APP_PENDING_BYTES: usize = 64 * 1024 * 1024;
static APP_PENDING: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
const FUEL_SLICE: u64 = 20_000_000;
const FUEL_BASE: u64 = 50_000_000;
const FUEL_PER_PIXEL: u64 = 4_000;
const FUEL_PER_EVENT: u64 = 2_000_000_000;
const FUEL_PER_JOB: u64 = 2_000_000_000_000;
pub const MAX_JOB_BYTES: usize = MAX_PENDING_BYTES;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginError {
    Load(String),
    Trap(String),
    TooSlow,
    TooMuchMemory,
    Cancelled,
    Stopped(String),
    InvalidOutput(String),
    Failed(i32, String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(e) => write!(f, "could not load the plugin: {e}"),
            Self::Trap(e) => write!(f, "the plugin crashed: {e}"),
            Self::TooSlow => f.write_str("the plugin took too long"),
            Self::TooMuchMemory => f.write_str("the plugin used too much memory"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::Stopped(e) => write!(f, "stopped: {e}"),
            Self::InvalidOutput(e) => write!(f, "the plugin returned an invalid result: {e}"),
            Self::Failed(c, m) => write!(f, "the plugin reported an error ({c}): {m}"),
        }
    }
}

impl std::error::Error for PluginError {}

pub type LogFn = Arc<dyn Fn(&str, i32, &str) + Send + Sync>;
pub type ProgressFn = Arc<dyn Fn(Option<f32>, &str) + Send + Sync>;

fn engine() -> Engine {
    let mut c = Config::default();
    c.consume_fuel(true)
        .set_max_recursion_depth(4096)
        .wasm_multi_memory(false)
        .wasm_reference_types(true)
        .wasm_tail_call(false)
        .wasm_custom_page_sizes(false)
        .ignore_custom_sections(true);
    Engine::new(&c)
}

#[derive(Clone)]
pub struct Plugin {
    engine: Engine,
    module: Module,
    pub id: String,
}

impl Plugin {
    pub fn load(id: &str, bytes: &[u8]) -> Result<Self, PluginError> {
        if bytes.len() > MAX_MODULE_BYTES {
            return Err(PluginError::Load("module too large".into()));
        }
        let engine = engine();
        let module = Module::new(&engine, bytes).map_err(|e| PluginError::Load(e.to_string()))?;
        for import in module.imports() {
            if import.module() != "craft" || !matches!(import.name(), "log" | "call" | "take" | "result") {
                return Err(PluginError::Load(format!("unknown import {}::{}", import.module(), import.name())));
            }
        }
        let has = |n: &str| module.exports().any(|e| e.name() == n);
        for need in ["memory", "craft_alloc", "craft_reset", "craft_abi"] {
            if !has(need) {
                return Err(PluginError::Load(format!("missing export {need}")));
            }
        }
        Ok(Self { engine, module, id: id.to_string() })
    }

    pub fn exports(&self, name: &str) -> bool {
        self.module.exports().any(|e| e.name() == name)
    }
}

impl Plugin {
    pub fn describe(&self, log: LogFn, check: &dyn Fn() -> Result<(), PluginError>) -> Result<Option<craft_ext_core::form::Form>, PluginError> {
        if !self.exports("craft_describe") {
            return Ok(None);
        }
        let bytes = self.form_call("craft_describe", None, log, check)?;
        let form: craft_ext_core::form::Form = serde_json::from_slice(&bytes).map_err(|e| PluginError::InvalidOutput(e.to_string()))?;
        form.validate().map_err(PluginError::InvalidOutput)?;
        Ok(Some(form))
    }
    pub fn update(
        &self,
        values: &craft_ext_core::form::Values,
        rev: u64,
        log: LogFn,
        check: &dyn Fn() -> Result<(), PluginError>,
    ) -> Result<Option<craft_ext_core::form::Patch>, PluginError> {
        if !self.exports("craft_update") {
            return Ok(None);
        }
        let input = serde_json::to_vec(&serde_json::json!({"values":values,"rev":rev})).map_err(|e| PluginError::InvalidOutput(e.to_string()))?;
        let bytes = self.form_call("craft_update", Some(&input), log, check)?;
        let patch: craft_ext_core::form::Patch = serde_json::from_slice(&bytes).map_err(|e| PluginError::InvalidOutput(e.to_string()))?;
        if patch.schema != craft_ext_core::form::FORM_SCHEMA || patch.base_rev != rev || patch.rev <= rev || patch.ops.len() > 512 {
            return Err(PluginError::InvalidOutput("invalid update revision or size".into()));
        }
        Ok(Some(patch))
    }
    fn form_call(&self, name: &str, input: Option<&[u8]>, log: LogFn, check: &dyn Fn() -> Result<(), PluginError>) -> Result<Vec<u8>, PluginError> {
        check()?;
        let mut inst = instantiate(self, MAX_SESSION_MEMORY, log, None)?;
        inst.reset.call(&mut inst.store, ()).map_err(|e| map_err(e, &inst.store))?;
        let code = if let Some(bytes) = input {
            if bytes.len() > MAX_CALL_BYTES {
                return Err(PluginError::TooMuchMemory);
            }
            let ptr = write_scratch(&mut inst, bytes)?;
            let f = inst.func::<(i32, i32), i32>(name)?;
            run_sliced(&mut inst, &f, (ptr, bytes.len() as i32), FUEL_PER_EVENT, &|_| check())?
        } else {
            let f = inst.func::<(), i32>(name)?;
            run_sliced(&mut inst, &f, (), FUEL_PER_EVENT, &|_| check())?
        };
        check()?;
        if code != 0 {
            return Err(PluginError::Failed(code, String::new()));
        }
        inst.store.data_mut().pending.take().map(|p| p.0.clone()).ok_or_else(|| PluginError::InvalidOutput("missing craft.result".into()))
    }
}

struct HostData {
    limits: StoreLimits,
    plugin: String,
    log: LogFn,
    pending: Option<Pending>,
    session: Option<SessionCtx>,
    stopped: Option<String>,
    progress: Option<ProgressFn>,
    debt: u64,
    budget: u64,
}

struct Pending(Vec<u8>);

impl Pending {
    fn reserve(bytes: Vec<u8>) -> Option<Self> {
        use std::sync::atomic::Ordering;
        let n = bytes.len();
        if n > MAX_PENDING_BYTES {
            return None;
        }
        let prev = APP_PENDING.fetch_add(n, Ordering::AcqRel);
        if prev.saturating_add(n) > MAX_APP_PENDING_BYTES {
            APP_PENDING.fetch_sub(n, Ordering::AcqRel);
            return None;
        }
        Some(Self(bytes))
    }
}

impl Drop for Pending {
    fn drop(&mut self) {
        APP_PENDING.fetch_sub(self.0.len(), std::sync::atomic::Ordering::AcqRel);
    }
}

struct SessionCtx {
    api: Arc<dyn AppApi>,
    grants: Arc<dyn Grants>,
    inv: Option<Invocation>,
    jobs: Option<(Arc<JobBoard>, Plugin)>,
}

fn read_guest(caller: &Caller<'_, HostData>, mem: &Memory, ptr: i32, len: i32) -> Result<Vec<u8>, Code> {
    let (ptr, len) = (usize::try_from(ptr).map_err(|_| Code::Invalid)?, usize::try_from(len).map_err(|_| Code::Invalid)?);
    if len > MAX_CALL_BYTES {
        return Err(Code::Budget);
    }
    let end = ptr.checked_add(len).ok_or(Code::Invalid)?;
    let data = mem.data(caller);
    data.get(ptr..end).map(<[u8]>::to_vec).ok_or(Code::Invalid)
}

const HOST_CALL_FUEL: u64 = 100_000;
const HOST_LOG_FUEL: u64 = 10_000;
const HOST_BYTE_FUEL: u64 = 100;

fn charge(caller: &mut Caller<'_, HostData>, cost: u64) {
    if let Ok(left) = caller.get_fuel() {
        if cost > left {
            let d = caller.data_mut();
            d.debt = d.debt.saturating_add(cost - left);
        }
        let _ = caller.set_fuel(left.saturating_sub(cost));
    }
}

fn kernel_call(caller: &mut Caller<'_, HostData>, mem: &Memory, op: &str, args: &Value) -> Result<Value, OpError> {
    if !kernels::OPS.contains(&op) {
        return Err(OpError::new(Code::NotFound, format!("this app has no kernel {op}")));
    }
    let names = kernels::inputs(op);
    let refs = names.iter().map(|n| kernels::ImageRef::parse(args.get(*n), n)).collect::<Result<Vec<_>, _>>().map_err(OpError::invalid)?;
    let pixels = refs.iter().map(|r| (r.width * r.height) as u64).max().unwrap_or(0);
    let cost = kernels::cost(op, args, pixels);
    if cost > caller.data().budget {
        return Err(OpError::new(Code::Budget, "this kernel call is larger than the plugin's remaining budget"));
    }
    charge(caller, cost);
    let data = mem.data(&*caller);
    let region = |ptr: usize, bytes: Option<usize>| bytes.and_then(|b| ptr.checked_add(b).and_then(|end| data.get(ptr..end)));
    let floats = |raw: &[u8]| raw.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect::<Vec<f32>>();
    let mut images = Vec::with_capacity(refs.len());
    for r in &refs {
        let raw = region(r.ptr, r.bytes()).ok_or_else(|| OpError::invalid("an image lies outside the plugin's memory"))?;
        images.push(kernels::Image { width: r.width, height: r.height, px: floats(raw) });
    }
    let splats = match op {
        "kernel.splat" => {
            let pts = args.get("points").ok_or_else(|| OpError::invalid("points is required"))?;
            let ptr = pts.get("ptr").and_then(Value::as_u64).and_then(|p| usize::try_from(p).ok());
            let count = pts.get("count").and_then(Value::as_u64).and_then(|c| usize::try_from(c).ok()).filter(|c| *c <= kernels::MAX_SPLATS);
            let (Some(ptr), Some(count)) = (ptr, count) else { return Err(OpError::invalid("points needs ptr and count (at most 1000000)")) };
            floats(region(ptr, count.checked_mul(32)).ok_or_else(|| OpError::invalid("points lie outside the plugin's memory"))?)
        }
        _ => Vec::new(),
    };
    let out = kernels::run(op, args, &mut images, &splats).map_err(|why| OpError::invalid(format!("{op}: {why}")))?;
    if kernels::writes(op) {
        let at = names.iter().position(|n| *n == "image").unwrap_or(0);
        let (Some(r), Some(img)) = (refs.get(at), images.get(at)) else { return Err(OpError::new(Code::Internal, "kernel image missing")) };
        let mut bytes = Vec::with_capacity(img.px.len() * 4);
        for v in &img.px {
            bytes.extend_from_slice(&(if v.is_finite() { *v } else { 0.0 }).to_le_bytes());
        }
        mem.write(&mut *caller, r.ptr, &bytes).map_err(|_| OpError::invalid("an image lies outside the plugin's memory"))?;
    }
    Ok(out)
}

fn job_call(caller: &mut Caller<'_, HostData>, op: &str, args: &Value) -> Result<Value, OpError> {
    if op == "job.progress" {
        let value = args.get("value").and_then(Value::as_f64).map(|v| v as f32);
        let note = args.get("note").and_then(Value::as_str).unwrap_or("");
        if let Some(p) = caller.data().progress.clone() {
            p(value, note);
        }
        return Ok(Value::Null);
    }
    let data = caller.data();
    let Some(session) = data.session.as_ref() else { return Err(OpError::denied("jobs can only be started from a session event")) };
    let Some(inv) = session.inv.as_ref() else { return Err(OpError::new(Code::Stale, "no event is running")) };
    let Some((board, module)) = session.jobs.as_ref() else {
        return Err(OpError::new(Code::NotFound, "background jobs need a craft_job export and an app with a jobs list"));
    };
    match op {
        "job.start" => {
            if session.grants.stopped(&inv.plugin, inv.version) || session.grants.generation(&inv.plugin) != inv.grant_generation {
                return Err(OpError::denied("permissions changed while this was running"));
            }
            let label = args.get("label").and_then(Value::as_str).filter(|l| !l.trim().is_empty()).ok_or_else(|| OpError::invalid("job.start: label is required"))?;
            let input = serde_json::to_vec(args.get("input").unwrap_or(&Value::Null)).map_err(|e| OpError::invalid(e.to_string()))?;
            if input.len() > MAX_JOB_BYTES {
                return Err(OpError::new(Code::Budget, "job.start: input is larger than 8 MiB"));
            }
            let cancellable = args.get("cancellable").and_then(Value::as_bool).unwrap_or(true);
            let ticket = Arc::new(board.admit(&inv.plugin, &inv.contribution, label, cancellable).map_err(|e| OpError::new(Code::Budget, e))?);
            let id = ticket.id;
            let (module, log, grants, inv) = (module.clone(), data.log.clone(), session.grants.clone(), inv.clone());
            let worker = ticket.clone();
            let spawned = std::thread::Builder::new().name(format!("plugin-job-{id}")).spawn(move || {
                let state = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_job(&module, log, &worker, &input, grants.as_ref(), &inv)))
                    .unwrap_or_else(|_| JobState::Failed("the app failed while running this job".into()));
                worker.finish(state);
            });
            if spawned.is_err() {
                ticket.finish(JobState::Failed("the app could not start a worker thread".into()));
            }
            Ok(serde_json::json!({ "job": id }))
        }
        "job.cancel" => {
            let id = args.get("job").and_then(Value::as_u64).ok_or_else(|| OpError::invalid("job.cancel: job is required"))?;
            let mine = board.list().into_iter().any(|j| j.id == id && j.plugin == inv.plugin);
            Ok(serde_json::json!({ "cancelled": mine && board.cancel(id, true) }))
        }
        _ => Err(OpError::new(Code::NotFound, format!("this app has no operation {op}"))),
    }
}

fn run_job(p: &Plugin, log: LogFn, ticket: &Arc<Ticket>, input: &[u8], grants: &dyn Grants, inv: &Invocation) -> JobState {
    let go = || -> Result<Value, PluginError> {
        let mut inst = instantiate(p, MAX_TRANSFORM_MEMORY, log, None)?;
        let t = ticket.clone();
        inst.store.data_mut().progress = Some(Arc::new(move |v, note| t.progress(v, note)));
        inst.reset.call(&mut inst.store, ()).map_err(|e| map_err(e, &inst.store))?;
        let ptr = write_scratch(&mut inst, input)?;
        let f = inst.func::<(i32, i32), i32>("craft_job")?;
        let cancel = ticket.cancel.clone();
        let code = run_sliced(&mut inst, &f, (ptr, input.len() as i32), FUEL_PER_JOB, &|_| {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return Err(PluginError::Cancelled);
            }
            if grants.stopped(&inv.plugin, inv.version) || grants.generation(&inv.plugin) != inv.grant_generation {
                return Err(PluginError::Stopped("permissions changed while this was running".into()));
            }
            Ok(())
        })?;
        if code != 0 {
            return Err(PluginError::Failed(code, String::new()));
        }
        match inst.store.data_mut().pending.take() {
            Some(bytes) => serde_json::from_slice(&bytes.0).map_err(|e| PluginError::InvalidOutput(e.to_string())),
            None => Ok(Value::Null),
        }
    };
    match go() {
        Ok(v) => JobState::Done(v),
        Err(PluginError::Cancelled) => JobState::Cancelled,
        Err(e) => JobState::Failed(e.to_string()),
    }
}

fn byte_fuel(len: i32) -> u64 {
    HOST_BYTE_FUEL.saturating_mul(u64::try_from(len).unwrap_or(0))
}

fn linker(engine: &Engine) -> Linker<HostData> {
    let mut l = Linker::<HostData>::new(engine);
    let _ = l.func_wrap("craft", "log", |mut caller: Caller<'_, HostData>, level: i32, ptr: i32, len: i32| {
        charge(&mut caller, HOST_LOG_FUEL.saturating_add(byte_fuel(len.min(4096))));
        let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) else { return };
        if let Ok(bytes) = read_guest(&caller, &mem, ptr, len.min(4096)) {
            let text = String::from_utf8_lossy(&bytes);
            (caller.data().log)(&caller.data().plugin, level.clamp(0, 3), &text);
        }
    });
    let _ = l.func_wrap("craft", "call", |mut caller: Caller<'_, HostData>, op: i32, op_len: i32, args: i32, args_len: i32| -> i64 {
        charge(&mut caller, HOST_CALL_FUEL.saturating_add(byte_fuel(args_len)));
        let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) else { return Code::Internal as i64 };
        let op = match read_guest(&caller, &mem, op, op_len.min(128)).map(String::from_utf8) {
            Ok(Ok(s)) => s,
            _ => return Code::Invalid as i64,
        };
        let args: Value = match read_guest(&caller, &mem, args, args_len) {
            Ok(b) if b.is_empty() => Value::Null,
            Ok(b) => match serde_json::from_slice(&b) {
                Ok(v) => v,
                Err(_) => return Code::Invalid as i64,
            },
            Err(c) => return c as i64,
        };
        let host = if op.starts_with("kernel.") {
            Some(kernel_call(&mut caller, &mem, &op, &args))
        } else if op.starts_with("job.") {
            Some(job_call(&mut caller, &op, &args))
        } else {
            None
        };
        if let Some(outcome) = host {
            let (bytes, ret) = match outcome {
                Ok(v) => {
                    let b = serde_json::to_vec(&v).unwrap_or_default();
                    let n = b.len() as i64;
                    (b, n)
                }
                Err(e) => (serde_json::to_vec(&e.to_json()).unwrap_or_default(), e.code as i64),
            };
            let data = caller.data_mut();
            data.pending = None;
            return match Pending::reserve(bytes) {
                Some(p) => {
                    data.pending = Some(p);
                    ret
                }
                None => Code::Budget as i64,
            };
        }
        let data = caller.data_mut();
        let Some(session) = data.session.as_ref() else { return Code::Denied as i64 };
        let Some(inv) = session.inv.as_ref() else { return Code::Stale as i64 };
        let authorization = crate::api::check_args(session.api.contract(), &op, &args).and_then(|()| {
            if session.api.authorizes_on_owner_thread() {
                crate::api::authorize_preflight(session.api.contract(), session.grants.as_ref(), inv, &op)
            } else {
                authorize(session.api.contract(), session.grants.as_ref(), inv, &op, &args)
            }
        });
        let outcome = authorization.and_then(|()| crate::api::contain(&op, || session.api.call(inv, &op, args)));
        let (bytes, ret) = match outcome {
            Ok(v) => {
                let b = serde_json::to_vec(&v).unwrap_or_default();
                let n = b.len() as i64;
                (b, n)
            }
            Err(e) => {
                if session.grants.stopped(&inv.plugin, inv.version) || session.grants.generation(&inv.plugin) != inv.grant_generation {
                    data.stopped = Some(e.message.clone());
                }
                (serde_json::to_vec(&e.to_json()).unwrap_or_default(), e.code as i64)
            }
        };
        data.pending = None;
        match Pending::reserve(bytes) {
            Some(p) => {
                data.pending = Some(p);
                ret
            }
            None => Code::Budget as i64,
        }
    });
    let _ = l.func_wrap("craft", "take", |mut caller: Caller<'_, HostData>, dst: i32, len: i32| -> i32 {
        let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) else { return Code::Internal as i32 };
        let Some(pending) = caller.data_mut().pending.take() else { return Code::NotFound as i32 };
        let (Ok(dst), Ok(len)) = (usize::try_from(dst), usize::try_from(len)) else { return Code::Invalid as i32 };
        if len < pending.0.len() {
            return Code::Invalid as i32;
        }
        match mem.write(&mut caller, dst, &pending.0) {
            Ok(()) => pending.0.len() as i32,
            Err(_) => Code::Invalid as i32,
        }
    });
    let _ = l.func_wrap("craft", "result", |mut caller: Caller<'_, HostData>, ptr: i32, len: i32| -> i32 {
        let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) else { return Code::Invalid as i32 };
        let bytes = match read_guest(&caller, &mem, ptr, len) {
            Ok(b) => b,
            Err(c) => return c as i32,
        };
        if caller.data().pending.is_some() {
            return Code::Invalid as i32;
        }
        match Pending::reserve(bytes) {
            Some(p) => {
                caller.data_mut().pending = Some(p);
                0
            }
            None => Code::Budget as i32,
        }
    });
    l
}

struct Inst {
    instance: Instance,
    store: Store<HostData>,
    memory: Memory,
    alloc: TypedFunc<i32, i32>,
    reset: TypedFunc<(), ()>,
}

fn instantiate(p: &Plugin, max_memory: usize, log: LogFn, session: Option<SessionCtx>) -> Result<Inst, PluginError> {
    let limits = StoreLimitsBuilder::new().memory_size(max_memory).memories(1).tables(4).table_elements(100_000).instances(1).build();
    let mut store = Store::new(&p.engine, HostData { limits, plugin: p.id.clone(), log, pending: None, session, stopped: None, progress: None, debt: 0, budget: 0 });
    store.limiter(|d| &mut d.limits);
    store.set_fuel(FUEL_BASE).map_err(|e| PluginError::Load(e.to_string()))?;
    let instance: Instance = linker(&p.engine).instantiate_and_start(&mut store, &p.module).map_err(|e| PluginError::Load(e.to_string()))?;
    let memory = instance.get_memory(&store, "memory").ok_or_else(|| PluginError::Load("no memory".into()))?;
    let abi = instance.get_typed_func::<(), i32>(&store, "craft_abi").map_err(|e| PluginError::Load(e.to_string()))?;
    let alloc = instance.get_typed_func::<i32, i32>(&store, "craft_alloc").map_err(|e| PluginError::Load(e.to_string()))?;
    let reset = instance.get_typed_func::<(), ()>(&store, "craft_reset").map_err(|e| PluginError::Load(e.to_string()))?;
    let v = abi.call(&mut store, ()).map_err(|e| PluginError::Load(e.to_string()))?;
    if v != ABI_VERSION {
        return Err(PluginError::Load(format!("built for plugin ABI {v}; this app supports {ABI_VERSION}")));
    }
    Ok(Inst { instance, store, memory, alloc, reset })
}

fn map_err(e: wasmi::Error, store: &Store<HostData>) -> PluginError {
    if let Some(reason) = store.data().stopped.clone() {
        return PluginError::Stopped(reason);
    }
    let text = e.to_string();
    if text.contains("memory") && (text.contains("limit") || text.contains("grow")) { PluginError::TooMuchMemory } else { PluginError::Trap(text) }
}

fn run_sliced<P: wasmi::WasmParams, R: wasmi::WasmResults>(
    inst: &mut Inst,
    func: &TypedFunc<P, R>,
    params: P,
    total_fuel: u64,
    keep_going: &dyn Fn(&HostData) -> Result<(), PluginError>,
) -> Result<R, PluginError> {
    let mut left = total_fuel;
    let first = left.min(FUEL_SLICE);
    left -= first;
    inst.store.data_mut().debt = 0;
    inst.store.data_mut().budget = total_fuel;
    inst.store.set_fuel(first).map_err(|e| PluginError::Trap(e.to_string()))?;
    let mut call = func.call_resumable(&mut inst.store, params).map_err(|e| map_err(e, &inst.store))?;
    loop {
        match call {
            TypedResumableCall::Finished(r) => return Ok(r),
            TypedResumableCall::HostTrap(_) => return Err(PluginError::Trap("host call failed".into())),
            TypedResumableCall::OutOfFuel(more) => {
                keep_going(inst.store.data())?;
                let debt = std::mem::take(&mut inst.store.data_mut().debt);
                left = left.checked_sub(debt).ok_or(PluginError::TooSlow)?;
                if left == 0 {
                    return Err(PluginError::TooSlow);
                }
                let slice = left.min(FUEL_SLICE);
                left -= slice;
                inst.store.data_mut().budget = left.saturating_add(slice);
                inst.store.set_fuel(slice).map_err(|e| PluginError::Trap(e.to_string()))?;
                call = more.resume(&mut inst.store).map_err(|e| map_err(e, &inst.store))?;
            }
        }
    }
}

fn write_scratch(inst: &mut Inst, bytes: &[u8]) -> Result<i32, PluginError> {
    let len = i32::try_from(bytes.len()).map_err(|_| PluginError::TooMuchMemory)?;
    inst.store.set_fuel(FUEL_BASE).map_err(|e| PluginError::Trap(e.to_string()))?;
    let ptr = inst.alloc.call(&mut inst.store, len).map_err(|e| map_err(e, &inst.store))?;
    if ptr <= 0 {
        return Err(PluginError::TooMuchMemory);
    }
    let start = usize::try_from(ptr).map_err(|_| PluginError::InvalidOutput("bad pointer".into()))?;
    inst.memory.write(&mut inst.store, start, bytes).map_err(|_| PluginError::InvalidOutput("craft_alloc returned a buffer out of bounds".into()))?;
    Ok(ptr)
}

pub struct Filter {
    inst: Option<Inst>,
    plugin: Plugin,
    log: LogFn,
    progress: Option<ProgressFn>,
}

impl Filter {
    pub fn new(p: &Plugin, log: LogFn) -> Result<Self, PluginError> {
        if !p.exports("craft_filter") {
            return Err(PluginError::Load("missing export craft_filter".into()));
        }
        let inst = instantiate(p, MAX_TRANSFORM_MEMORY, log.clone(), None)?;
        Ok(Self { inst: Some(inst), plugin: p.clone(), log, progress: None })
    }

    pub fn set_progress(&mut self, progress: ProgressFn) {
        self.progress = Some(progress);
    }

    pub fn run(
        &mut self,
        ctx: &Value,
        pixels: &mut [f32],
        width: u32,
        height: u32,
        check: &dyn Fn() -> Result<(), PluginError>,
    ) -> Result<(), PluginError> {
        check()?;
        let mut inst = match self.inst.take() {
            Some(i) => i,
            None => instantiate(&self.plugin, MAX_TRANSFORM_MEMORY, self.log.clone(), None)?,
        };
        inst.store.data_mut().progress = self.progress.clone();
        let out = Self::run_inner(&mut inst, ctx, pixels.len(), &*pixels, width, height, check);
        match out {
            Ok(result) => {
                check()?;
                pixels.copy_from_slice(&result);
                self.inst = Some(inst);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn run_inner(
        inst: &mut Inst,
        ctx: &Value,
        len: usize,
        pixels: &[f32],
        width: u32,
        height: u32,
        check: &dyn Fn() -> Result<(), PluginError>,
    ) -> Result<Vec<f32>, PluginError> {
        let n = (width as usize).checked_mul(height as usize).and_then(|n| n.checked_mul(4)).ok_or(PluginError::TooMuchMemory)?;
        if len != n {
            return Err(PluginError::InvalidOutput("pixel buffer size does not match".into()));
        }
        inst.store.set_fuel(FUEL_BASE).map_err(|e| PluginError::Trap(e.to_string()))?;
        inst.reset.call(&mut inst.store, ()).map_err(|e| map_err(e, &inst.store))?;
        let ctx_bytes = serde_json::to_vec(ctx).map_err(|e| PluginError::InvalidOutput(e.to_string()))?;
        let ctx_ptr = write_scratch(inst, &ctx_bytes)?;
        let mut raw = Vec::with_capacity(n * 4);
        for v in pixels.iter() {
            raw.extend_from_slice(&v.to_le_bytes());
        }
        let px_ptr = write_scratch(inst, &raw)?;
        let func = inst.func::<(i32, i32, i32, i32, i32), i32>("craft_filter")?;
        let fuel = FUEL_BASE.saturating_add(FUEL_PER_PIXEL.saturating_mul((width as u64) * (height as u64)));
        let code = run_sliced(inst, &func, (ctx_ptr, ctx_bytes.len() as i32, px_ptr, width as i32, height as i32), fuel, &|_| check())?;
        if code != 0 {
            return Err(PluginError::Failed(code, String::new()));
        }
        let start = usize::try_from(px_ptr).map_err(|_| PluginError::InvalidOutput("bad pointer".into()))?;
        inst.memory.read(&inst.store, start, &mut raw).map_err(|_| PluginError::InvalidOutput("pixels out of bounds".into()))?;
        let mut out = Vec::with_capacity(n);
        for chunk in raw.chunks_exact(4) {
            let v = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            if !v.is_finite() {
                return Err(PluginError::InvalidOutput("a pixel is not a number".into()));
            }
            out.push(v);
        }
        for px in out.chunks_exact_mut(4) {
            px[3] = px[3].clamp(0.0, 1.0);
            if px[3] == 0.0 {
                px[0] = 0.0;
                px[1] = 0.0;
                px[2] = 0.0;
            }
        }
        Ok(out)
    }
}

impl Inst {
    fn func<P: wasmi::WasmParams, R: wasmi::WasmResults>(&self, name: &str) -> Result<TypedFunc<P, R>, PluginError> {
        self.instance.get_typed_func::<P, R>(&self.store, name).map_err(|_| PluginError::Load(format!("missing or mistyped export {name}")))
    }
}

pub struct Session {
    inst: Inst,
    event: TypedFunc<(i32, i32), i32>,
    failures: u32,
    pub paused: bool,
    plugin: Plugin,
    log: LogFn,
    jobs: Option<Arc<JobBoard>>,
}

impl Session {
    pub fn new(p: &Plugin, api: Arc<dyn AppApi>, grants: Arc<dyn Grants>, log: LogFn) -> Result<Self, PluginError> {
        let inst = instantiate(p, MAX_SESSION_MEMORY, log.clone(), Some(SessionCtx { api, grants, inv: None, jobs: None }))?;
        let event = inst.func::<(i32, i32), i32>("craft_event")?;
        Ok(Self { inst, event, failures: 0, paused: false, plugin: p.clone(), log, jobs: None })
    }

    pub fn set_jobs(&mut self, board: Arc<JobBoard>) {
        if !self.plugin.exports("craft_job") {
            return;
        }
        self.jobs = Some(board.clone());
        if let Some(s) = self.inst.store.data_mut().session.as_mut() {
            s.jobs = Some((board, self.plugin.clone()));
        }
    }

    fn reinstantiate(&mut self, api: Arc<dyn AppApi>, grants: Arc<dyn Grants>) -> Result<(), PluginError> {
        let jobs = self.jobs.clone().map(|b| (b, self.plugin.clone()));
        let inst = instantiate(&self.plugin, MAX_SESSION_MEMORY, self.log.clone(), Some(SessionCtx { api, grants, inv: None, jobs }))?;
        self.event = inst.func::<(i32, i32), i32>("craft_event")?;
        self.inst = inst;
        Ok(())
    }

    pub fn dispatch(&mut self, inv: Invocation, event: &Value) -> Result<(), PluginError> {
        if self.paused {
            return Err(PluginError::Stopped("paused after repeated failures".into()));
        }
        let (api, grants) = match self.inst.store.data().session.as_ref() {
            Some(s) => (s.api.clone(), s.grants.clone()),
            None => return Err(PluginError::Load("not a session".into())),
        };
        if grants.stopped(&inv.plugin, inv.version) {
            return Err(PluginError::Stopped("this plugin has been turned off".into()));
        }
        crate::api::contain("begin", || api.begin(&inv)).map_err(|e| PluginError::Failed(e.code as i32, e.message))?;
        let result = self.run_event(&inv, event, grants.as_ref());
        let still_ok = !grants.stopped(&inv.plugin, inv.version) && grants.generation(&inv.plugin) == inv.grant_generation && !inv.cancelled();
        let commit = result.is_ok() && still_ok;
        let _ = crate::api::contain("finish", || {
            api.finish(&inv, commit);
            Ok(())
        });
        if let Some(s) = self.inst.store.data_mut().session.as_mut() {
            s.inv = None;
        }
        self.inst.store.data_mut().pending = None;
        match result {
            Ok(()) if !still_ok => Err(PluginError::Stopped("permissions changed or cancelled before commit".into())),
            Ok(()) => {
                self.failures = 0;
                Ok(())
            }
            Err(e) => {
                if matches!(
                    e,
                    PluginError::Trap(_) | PluginError::TooSlow | PluginError::TooMuchMemory | PluginError::Cancelled | PluginError::Stopped(_)
                ) {
                    if matches!(e, PluginError::Trap(_) | PluginError::TooSlow | PluginError::TooMuchMemory) {
                        self.failures += 1;
                        if self.failures >= 3 {
                            self.paused = true;
                        }
                    }
                    if let Err(load) = self.reinstantiate(api, grants) {
                        self.paused = true;
                        return Err(load);
                    }
                }
                Err(e)
            }
        }
    }

    fn run_event(&mut self, inv: &Invocation, event: &Value, grants: &dyn Grants) -> Result<(), PluginError> {
        let inst = &mut self.inst;
        inst.store.data_mut().stopped = None;
        if let Some(s) = inst.store.data_mut().session.as_mut() {
            s.inv = Some(inv.clone());
        }
        inst.store.set_fuel(FUEL_BASE).map_err(|e| PluginError::Trap(e.to_string()))?;
        inst.reset.call(&mut inst.store, ()).map_err(|e| map_err(e, &inst.store))?;
        let bytes = serde_json::to_vec(event).map_err(|e| PluginError::InvalidOutput(e.to_string()))?;
        let ptr = write_scratch(inst, &bytes)?;
        let inv2 = inv.clone();
        let code = run_sliced(inst, &self.event, (ptr, bytes.len() as i32), FUEL_PER_EVENT, &move |d: &HostData| {
            if let Some(r) = &d.stopped {
                return Err(PluginError::Stopped(r.clone()));
            }
            if inv2.cancelled() {
                return Err(PluginError::Cancelled);
            }
            if grants.stopped(&inv2.plugin, inv2.version) || grants.generation(&inv2.plugin) != inv2.grant_generation {
                return Err(PluginError::Stopped("permissions changed while this was running".into()));
            }
            Ok(())
        })?;
        if let Some(r) = inst.store.data().stopped.clone() {
            return Err(PluginError::Stopped(r));
        }
        if code != 0 {
            return Err(PluginError::Failed(code, String::new()));
        }
        Ok(())
    }
}

pub fn code_of(e: &PluginError) -> i32 {
    match e {
        PluginError::Cancelled => Code::Cancelled as i32,
        PluginError::Stopped(_) => Code::Denied as i32,
        PluginError::TooSlow | PluginError::TooMuchMemory => Code::Budget as i32,
        PluginError::InvalidOutput(_) | PluginError::Load(_) => Code::Invalid as i32,
        PluginError::Trap(_) | PluginError::Failed(..) => Code::Internal as i32,
    }
}

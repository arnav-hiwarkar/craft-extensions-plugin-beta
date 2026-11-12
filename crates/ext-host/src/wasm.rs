//! The sandbox: wasmi with one locked profile, guest ABI v1 (see beta-design B4).
//!
//! Invariants:
//! - One `Config` for every app; apps cannot loosen it. Unknown imports fail instantiation.
//! - Fuel runs in slices; between slices the host checks cancel, deadline and (sessions) the grant.
//! - Every guest pointer range is bounds-checked with checked arithmetic before use.
//! - Host-held bytes (pending results) are charged to a per-instance budget.
//! - `craft_reset` runs before every invocation; scratch from `craft_alloc` is the host's until then.
//! - A trap, budget hit or invalid output never panics the host; the instance is discarded.

use crate::api::{AppApi, Code, Grants, Invocation, authorize};
use serde_json::Value;
use std::sync::Arc;
use wasmi::{Caller, Config, Engine, Instance, Linker, Memory, Module, Store, StoreLimits, StoreLimitsBuilder, TypedFunc, TypedResumableCall};

pub const ABI_VERSION: i32 = 1;
pub const MAX_MODULE_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_TRANSFORM_MEMORY: usize = 512 * 1024 * 1024;
pub const MAX_SESSION_MEMORY: usize = 64 * 1024 * 1024;
/// Largest JSON a guest may pass to the host in one call (op name + args, logs).
pub const MAX_CALL_BYTES: usize = 1024 * 1024;
/// Host-held pending result budget per instance.
pub const MAX_PENDING_BYTES: usize = 8 * 1024 * 1024;
const FUEL_SLICE: u64 = 20_000_000;
const FUEL_BASE: u64 = 50_000_000;
const FUEL_PER_PIXEL: u64 = 4_000;
const FUEL_PER_EVENT: u64 = 2_000_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginError {
    /// The module is malformed, too large, or does not match ABI v1.
    Load(String),
    Trap(String),
    TooSlow,
    TooMuchMemory,
    Cancelled,
    /// Permission revoked or plugin stopped while running.
    Stopped(String),
    InvalidOutput(String),
    /// The guest returned a non-zero code.
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

/// Log sink: (plugin id, level 0=debug..3=error, message). Plugin text is untrusted.
pub type LogFn = Arc<dyn Fn(&str, i32, &str) + Send + Sync>;

fn engine() -> Engine {
    let mut c = Config::default();
    c.consume_fuel(true)
        .set_max_recursion_depth(4096)
        .wasm_multi_memory(false)
        .wasm_reference_types(false)
        .wasm_tail_call(false)
        .wasm_custom_page_sizes(false)
        .ignore_custom_sections(true);
    Engine::new(&c)
}

/// A validated module, shareable across threads; instantiate per render thread or per session.
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
            if import.module() != "craft" || !matches!(import.name(), "log" | "call" | "take") {
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

/// Per-instance host data.
struct HostData {
    limits: StoreLimits,
    plugin: String,
    log: LogFn,
    pending: Option<Vec<u8>>,
    /// Present only for sessions: transforms get no `call`.
    session: Option<SessionCtx>,
    /// Revocation observed during this invocation.
    stopped: Option<String>,
}

struct SessionCtx {
    api: Arc<dyn AppApi>,
    grants: Arc<dyn Grants>,
    inv: Option<Invocation>,
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

fn linker(engine: &Engine) -> Linker<HostData> {
    let mut l = Linker::<HostData>::new(engine);
    let _ = l.func_wrap("craft", "log", |caller: Caller<'_, HostData>, level: i32, ptr: i32, len: i32| {
        let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) else { return };
        if let Ok(bytes) = read_guest(&caller, &mem, ptr, len.min(4096)) {
            let text = String::from_utf8_lossy(&bytes);
            (caller.data().log)(&caller.data().plugin, level.clamp(0, 3), &text);
        }
    });
    let _ = l.func_wrap("craft", "call", |mut caller: Caller<'_, HostData>, op: i32, op_len: i32, args: i32, args_len: i32| -> i64 {
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
        let data = caller.data_mut();
        let Some(session) = data.session.as_ref() else { return Code::Denied as i64 };
        let Some(inv) = session.inv.as_ref() else { return Code::Stale as i64 };
        let outcome = authorize(session.api.contract(), session.grants.as_ref(), inv, &op, &args).and_then(|()| session.api.call(inv, &op, args));
        let (bytes, ret) = match outcome {
            Ok(v) => {
                let b = serde_json::to_vec(&v).unwrap_or_default();
                let n = b.len() as i64;
                (b, n)
            }
            Err(e) => {
                if e.code == Code::Denied && (e.message.starts_with("permissions changed") || e.message.starts_with("this plugin has been turned off")) {
                    data.stopped = Some(e.message.clone());
                }
                (serde_json::to_vec(&e.to_json()).unwrap_or_default(), e.code as i64)
            }
        };
        if bytes.len() > MAX_PENDING_BYTES {
            data.pending = None;
            return Code::Budget as i64;
        }
        data.pending = Some(bytes);
        ret
    });
    let _ = l.func_wrap("craft", "take", |mut caller: Caller<'_, HostData>, dst: i32, len: i32| -> i32 {
        let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) else { return Code::Internal as i32 };
        let Some(bytes) = caller.data_mut().pending.take() else { return Code::NotFound as i32 };
        let (Ok(dst), Ok(len)) = (usize::try_from(dst), usize::try_from(len)) else { return Code::Invalid as i32 };
        if len < bytes.len() {
            return Code::Invalid as i32;
        }
        match mem.write(&mut caller, dst, &bytes) {
            Ok(()) => bytes.len() as i32,
            Err(_) => Code::Invalid as i32,
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
    let mut store = Store::new(&p.engine, HostData { limits, plugin: p.id.clone(), log, pending: None, session, stopped: None });
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
    if text.contains("memory") && (text.contains("limit") || text.contains("grow")) {
        PluginError::TooMuchMemory
    } else {
        PluginError::Trap(text)
    }
}

/// Runs `func` with fuel slices, checking `keep_going` between slices.
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
    inst.store.set_fuel(first).map_err(|e| PluginError::Trap(e.to_string()))?;
    let mut call = func.call_resumable(&mut inst.store, params).map_err(|e| map_err(e, &inst.store))?;
    loop {
        match call {
            TypedResumableCall::Finished(r) => return Ok(r),
            TypedResumableCall::HostTrap(_) => return Err(PluginError::Trap("host call failed".into())),
            TypedResumableCall::OutOfFuel(more) => {
                keep_going(inst.store.data())?;
                if left == 0 {
                    return Err(PluginError::TooSlow);
                }
                let slice = left.min(FUEL_SLICE);
                left -= slice;
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

/// A `raster.filter@1` transform instance. One per render thread; never shared.
pub struct Filter {
    inst: Inst,
}

impl Filter {
    pub fn new(p: &Plugin, log: LogFn) -> Result<Self, PluginError> {
        if !p.exports("craft_filter") {
            return Err(PluginError::Load("missing export craft_filter".into()));
        }
        Ok(Self { inst: instantiate(p, MAX_TRANSFORM_MEMORY, log, None)? })
    }

    /// Runs the filter in place on premultiplied RGBA f32 pixels. `ctx` is the B4 JSON object.
    /// `check` is called between fuel slices (cancel, deadline, kill list).
    pub fn run(&mut self, ctx: &Value, pixels: &mut [f32], width: u32, height: u32, check: &dyn Fn() -> Result<(), PluginError>) -> Result<(), PluginError> {
        let n = (width as usize).checked_mul(height as usize).and_then(|n| n.checked_mul(4)).ok_or(PluginError::TooMuchMemory)?;
        if pixels.len() != n {
            return Err(PluginError::InvalidOutput("pixel buffer size does not match".into()));
        }
        let inst = &mut self.inst;
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
        for (i, chunk) in raw.chunks_exact(4).enumerate() {
            let v = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            if !v.is_finite() {
                return Err(PluginError::InvalidOutput("a pixel is not a number".into()));
            }
            pixels[i] = v;
        }
        for px in pixels.chunks_exact_mut(4) {
            // Alpha clamps to 0..=1; colour is clamped to the premultiplied range when alpha is zero.
            px[3] = px[3].clamp(0.0, 1.0);
            if px[3] == 0.0 {
                px[0] = 0.0;
                px[1] = 0.0;
                px[2] = 0.0;
            }
        }
        Ok(())
    }
}

impl Inst {
    fn func<P: wasmi::WasmParams, R: wasmi::WasmResults>(&self, name: &str) -> Result<TypedFunc<P, R>, PluginError> {
        self.instance.get_typed_func::<P, R>(&self.store, name).map_err(|_| PluginError::Load(format!("missing or mistyped export {name}")))
    }
}

/// A session plugin instance (commands, panels, tools, event hooks). Driven from one worker thread.
pub struct Session {
    inst: Inst,
    event: TypedFunc<(i32, i32), i32>,
    failures: u32,
    pub paused: bool,
}

impl Session {
    pub fn new(p: &Plugin, api: Arc<dyn AppApi>, grants: Arc<dyn Grants>, log: LogFn) -> Result<Self, PluginError> {
        let inst = instantiate(p, MAX_SESSION_MEMORY, log, Some(SessionCtx { api, grants, inv: None }))?;
        let event = inst.func::<(i32, i32), i32>("craft_event")?;
        Ok(Self { inst, event, failures: 0, paused: false })
    }

    /// Delivers one event as one transaction. Commits only if the guest returned 0, nothing was
    /// revoked meanwhile, and the grant is still current at commit time.
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
        api.begin(&inv).map_err(|e| PluginError::Failed(e.code as i32, e.message))?;
        let result = self.run_event(&inv, event);
        let still_ok = !grants.stopped(&inv.plugin, inv.version) && grants.generation(&inv.plugin) == inv.grant_generation && !inv.cancelled();
        let commit = result.is_ok() && still_ok;
        api.finish(&inv, commit);
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
                if matches!(e, PluginError::Trap(_) | PluginError::TooSlow | PluginError::TooMuchMemory) {
                    self.failures += 1;
                    if self.failures >= 3 {
                        self.paused = true;
                    }
                }
                Err(e)
            }
        }
    }

    fn run_event(&mut self, inv: &Invocation, event: &Value) -> Result<(), PluginError> {
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

/// Convenience used by tests and `craft-ext test`: an error-to-code mapping for CLI exit codes.
pub fn code_of(e: &PluginError) -> i32 {
    match e {
        PluginError::Cancelled => Code::Cancelled as i32,
        PluginError::Stopped(_) => Code::Denied as i32,
        PluginError::TooSlow | PluginError::TooMuchMemory => Code::Budget as i32,
        PluginError::InvalidOutput(_) | PluginError::Load(_) => Code::Invalid as i32,
        PluginError::Trap(_) | PluginError::Failed(..) => Code::Internal as i32,
    }
}

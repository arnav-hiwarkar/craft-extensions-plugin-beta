use serde_json::{Value, json};
use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

mod comp {
    wasmtime::component::bindgen!({
        path: "../wit",
        world: "plugin",
        with: { "craft:bench/doc.layer": super::LayerRep },
    });
}

pub struct LayerRep {
    index: u32,
}

#[derive(Clone)]
struct Layer {
    id: u64,
    name: String,
    visible: bool,
    opacity: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum Perm {
    Read,
    Edit,
}

struct OpMeta {
    perm: Perm,
}

struct Host {
    layers: Vec<Layer>,
    pending: Vec<u8>,
    grants: [bool; 2],
    ops: HashMap<&'static str, OpMeta>,
    table: wasmtime::component::ResourceTable,
}

impl Host {
    fn new() -> Self {
        let layers = (0..1000).map(|i| Layer { id: 1000 + i as u64, name: format!("Layer {i}"), visible: true, opacity: 1.0 }).collect();
        let mut ops = HashMap::new();
        ops.insert("layer.get", OpMeta { perm: Perm::Read });
        ops.insert("layer.set_opacity", OpMeta { perm: Perm::Edit });
        ops.insert("ops.batch", OpMeta { perm: Perm::Read });
        Host { layers, pending: Vec::new(), grants: [true, true], ops, table: wasmtime::component::ResourceTable::new() }
    }

    fn authorize(&self, p: Perm) -> Result<(), String> {
        if black_box(self.grants[p as usize]) { Ok(()) } else { Err("permission".into()) }
    }

    fn layer(&self, i: u64) -> Result<&Layer, String> {
        self.layers.get(i as usize).ok_or_else(|| "not found".to_string())
    }

    fn json_op(&mut self, op: &str, args: &Value) -> Result<Value, String> {
        let meta = self.ops.get(op).ok_or("unknown op")?;
        self.authorize(meta.perm)?;
        let idx = args.get("layer").and_then(Value::as_u64).ok_or("layer: expected integer")?;
        match op {
            "layer.get" => {
                let l = self.layer(idx)?;
                Ok(json!({"id": l.id, "name": l.name, "visible": l.visible, "opacity": l.opacity}))
            }
            "layer.set_opacity" => {
                let v = args.get("opacity").and_then(Value::as_f64).ok_or("opacity: expected number")?;
                if !(0.0..=1.0).contains(&v) {
                    return Err("opacity: out of range".into());
                }
                let l = self.layers.get_mut(idx as usize).ok_or("not found")?;
                l.opacity = v as f32;
                Ok(Value::Null)
            }
            _ => Err("unknown op".into()),
        }
    }

    fn json_call(&mut self, op: &str, args: &[u8]) -> Result<Vec<u8>, String> {
        if op == "layer.get.typed" {
            #[derive(serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            struct A { layer: u32 }
            #[derive(serde::Serialize)]
            struct R<'a> { id: u64, name: &'a str, visible: bool, opacity: f32 }
            self.authorize(self.ops.get("layer.get").ok_or("unknown op")?.perm)?;
            let a: A = serde_json::from_slice(args).map_err(|e| e.to_string())?;
            let l = self.layer(a.layer as u64)?;
            return serde_json::to_vec(&R { id: l.id, name: &l.name, visible: l.visible, opacity: l.opacity }).map_err(|e| e.to_string());
        }
        let args: Value = serde_json::from_slice(args).map_err(|e| e.to_string())?;
        let out = if op == "ops.batch" {
            let ops = args.get("ops").and_then(Value::as_array).ok_or("ops")?;
            let mut res = Vec::with_capacity(ops.len());
            for o in ops {
                let name = o.get("op").and_then(Value::as_str).ok_or("op")?;
                res.push(self.json_op(name, o.get("args").ok_or("args")?)?);
            }
            Value::Array(res)
        } else {
            self.json_op(op, &args)?
        };
        serde_json::to_vec(&out).map_err(|e| e.to_string())
    }

    fn set_opacity(&mut self, i: u32, v: f32) -> Result<(), String> {
        self.authorize(Perm::Edit)?;
        if !(0.0..=1.0).contains(&v) {
            return Err("opacity: out of range".into());
        }
        self.layers.get_mut(i as usize).ok_or("not found")?.opacity = v;
        Ok(())
    }
}

const LAYER_OUT: usize = 72;

fn write_layer(mem: &mut [u8], at: usize, l: &Layer) -> bool {
    let Some(dst) = mem.get_mut(at..at + LAYER_OUT) else { return false };
    dst[0..8].copy_from_slice(&l.id.to_le_bytes());
    dst[8..12].copy_from_slice(&l.opacity.to_le_bytes());
    dst[12..16].copy_from_slice(&(l.visible as u32).to_le_bytes());
    let n = l.name.len().min(52);
    dst[16..20].copy_from_slice(&(n as u32).to_le_bytes());
    dst[20..20 + n].copy_from_slice(&l.name.as_bytes()[..n]);
    true
}

fn core_call(mem: &[u8], h: &mut Host, op: i32, op_len: i32, a: i32, a_len: i32) -> i64 {
    let (op, a) = (op as usize, a as usize);
    let Some(opb) = mem.get(op..op + op_len as usize) else { return -1 };
    let Some(ab) = mem.get(a..a + a_len as usize) else { return -1 };
    let Ok(op) = std::str::from_utf8(opb) else { return -1 };
    let op = op.to_owned();
    match h.json_call(&op, ab) {
        Ok(v) => {
            let n = v.len() as i64;
            h.pending = v;
            n
        }
        Err(_) => -1,
    }
}

fn core_take(mem: &mut [u8], h: &mut Host, dst: i32, len: i32) -> i32 {
    let n = h.pending.len().min(len as usize);
    let Some(d) = mem.get_mut(dst as usize..dst as usize + n) else { return -1 };
    d.copy_from_slice(&h.pending[..n]);
    h.pending.clear();
    n as i32
}

fn core_get_layer(mem: &mut [u8], h: &mut Host, i: i32, out: i32) -> i32 {
    if h.authorize(Perm::Read).is_err() {
        return -2;
    }
    let Some(l) = h.layers.get(i as usize) else { return -3 };
    if write_layer(mem, out as usize, l) { 0 } else { -1 }
}

fn core_batch(mem: &mut [u8], h: &mut Host, ops: i32, n: i32, out: i32) -> i32 {
    if h.authorize(Perm::Read).is_err() {
        return -2;
    }
    for k in 0..n as usize {
        let at = ops as usize + k * 12;
        let Some(rec) = mem.get(at..at + 12) else { return -1 };
        let layer = u32::from_le_bytes(rec[4..8].try_into().unwrap());
        let Some(l) = h.layers.get(layer as usize) else { return -3 };
        let l = l.clone();
        if !write_layer(mem, out as usize + k * LAYER_OUT, &l) {
            return -1;
        }
    }
    0
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn time(mut f: impl FnMut() -> u64) -> f64 {
    black_box(f());
    let mut ts = Vec::new();
    for _ in 0..9 {
        let t = Instant::now();
        black_box(f());
        ts.push(t.elapsed().as_secs_f64());
    }
    median(ts)
}

fn report(label: &str, secs: f64, ops: u32) {
    println!("{label:<62} {:>10.1} ns/op  {:>9.3} ms total  ({ops} ops)", secs * 1e9 / ops as f64, secs * 1e3);
}

fn wasmtime_core(engine: &wasmtime::Engine, wasm: &[u8]) -> wasmtime::Result<()> {
    use wasmtime::{Caller, Linker, Module, Store};
    let module = Module::new(engine, wasm)?;
    let mut linker: Linker<Host> = Linker::new(engine);
    fn mem<'a>(c: &'a mut Caller<'_, Host>) -> (&'a mut [u8], &'a mut Host) {
        let m = c.get_export("memory").unwrap().into_memory().unwrap();
        m.data_and_store_mut(c)
    }
    linker.func_wrap("craft", "call", |mut c: Caller<'_, Host>, a: i32, b: i32, x: i32, y: i32| -> i64 {
        let (m, h) = mem(&mut c);
        core_call(m, h, a, b, x, y)
    })?;
    linker.func_wrap("craft", "take", |mut c: Caller<'_, Host>, d: i32, l: i32| -> i32 {
        let (m, h) = mem(&mut c);
        core_take(m, h, d, l)
    })?;
    linker.func_wrap("bench", "noop", |_c: Caller<'_, Host>| {})?;
    linker.func_wrap("bench", "set_opacity", |mut c: Caller<'_, Host>, i: i32, v: f32| -> i32 {
        match c.data_mut().set_opacity(i as u32, v) { Ok(()) => 0, Err(_) => -1 }
    })?;
    linker.func_wrap("bench", "get_layer", |mut c: Caller<'_, Host>, i: i32, o: i32| -> i32 {
        let (m, h) = mem(&mut c);
        core_get_layer(m, h, i, o)
    })?;
    linker.func_wrap("bench", "batch", |mut c: Caller<'_, Host>, ops: i32, n: i32, o: i32| -> i32 {
        let (m, h) = mem(&mut c);
        core_batch(m, h, ops, n, o)
    })?;
    let mut store = Store::new(engine, Host::new());
    store.set_epoch_deadline(u64::MAX / 2);
    let inst = linker.instantiate(&mut store, &module)?;
    let run = inst.get_typed_func::<(u32, u32), u64>(&mut store, "run")?;
    let n = 10_000;
    let mut go = |label: &str, mode: u32, n: u32| {
        let s = time(|| run.call(&mut store, (mode, n)).unwrap());
        report(label, s, n);
    };
    go("wasmtime core: export call, no imports (n=1 fixed cost)", 99, 1);
    go("wasmtime core: noop import", 3, n);
    go("wasmtime core: typed scalar set_opacity(i32,f32)->i32", 4, n);
    go("wasmtime core: typed get_layer -> fixed 72-byte struct", 5, n);
    go("wasmtime core: JSON craft.call+take layer.get", 0, n);
    go("wasmtime core: JSON craft.call+take layer.set_opacity", 1, n);
    go("wasmtime core: JSON typed serde structs layer.get", 8, n);
    go("wasmtime core: JSON ops.batch 1000 x layer.get (1 crossing)", 2, 1000);
    go("wasmtime core: JSON ops.batch 1000 x set_opacity (1 crossing)", 7, 1000);
    go("wasmtime core: binary batch 1000 x get_layer (1 crossing)", 6, 1000);
    Ok(())
}

fn wasmi_core(wasm: &[u8]) -> wasmtime::Result<()> {
    use wasmi::{Caller, Engine, Linker, Module, Store};
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).map_err(|e| wasmtime::format_err!("{e}"))?;
    let mut linker: Linker<Host> = Linker::new(&engine);
    fn mem<'a>(c: &'a mut Caller<'_, Host>) -> (&'a mut [u8], &'a mut Host) {
        let m = c.get_export("memory").unwrap().into_memory().unwrap();
        m.data_and_store_mut(c)
    }
    linker.func_wrap("craft", "call", |mut c: Caller<'_, Host>, a: i32, b: i32, x: i32, y: i32| -> i64 {
        let (m, h) = mem(&mut c);
        core_call(m, h, a, b, x, y)
    })?;
    linker.func_wrap("craft", "take", |mut c: Caller<'_, Host>, d: i32, l: i32| -> i32 {
        let (m, h) = mem(&mut c);
        core_take(m, h, d, l)
    })?;
    linker.func_wrap("bench", "noop", |_c: Caller<'_, Host>| {})?;
    linker.func_wrap("bench", "set_opacity", |mut c: Caller<'_, Host>, i: i32, v: f32| -> i32 {
        match c.data_mut().set_opacity(i as u32, v) { Ok(()) => 0, Err(_) => -1 }
    })?;
    linker.func_wrap("bench", "get_layer", |mut c: Caller<'_, Host>, i: i32, o: i32| -> i32 {
        let (m, h) = mem(&mut c);
        core_get_layer(m, h, i, o)
    })?;
    linker.func_wrap("bench", "batch", |mut c: Caller<'_, Host>, ops: i32, n: i32, o: i32| -> i32 {
        let (m, h) = mem(&mut c);
        core_batch(m, h, ops, n, o)
    })?;
    let mut store = Store::new(&engine, Host::new());
    let inst = linker.instantiate_and_start(&mut store, &module)?;
    let run = inst.get_typed_func::<(u32, u32), u64>(&store, "run")?;
    let n = 10_000;
    let mut go = |label: &str, mode: u32, n: u32| {
        let s = time(|| run.call(&mut store, (mode, n)).unwrap());
        report(label, s, n);
    };
    go("wasmi 2.0.1 tail-call: export call, no imports", 99, 1);
    go("wasmi 2.0.1 tail-call: noop import", 3, n);
    go("wasmi 2.0.1 tail-call: typed scalar set_opacity", 4, n);
    go("wasmi 2.0.1 tail-call: typed get_layer -> 72-byte struct", 5, n);
    go("wasmi 2.0.1 tail-call: JSON craft.call+take layer.get", 0, n);
    go("wasmi 2.0.1 tail-call: JSON craft.call+take set_opacity", 1, n);
    go("wasmi 2.0.1 tail-call: JSON typed serde structs layer.get", 8, n);
    go("wasmi 2.0.1 tail-call: JSON ops.batch 1000 x layer.get", 2, 1000);
    go("wasmi 2.0.1 tail-call: JSON ops.batch 1000 x set_opacity", 7, 1000);
    go("wasmi 2.0.1 tail-call: binary batch 1000 x get_layer", 6, 1000);
    Ok(())
}

impl comp::craft::bench::doc::Host for Host {
    fn get_layer(&mut self, index: u32) -> Result<comp::craft::bench::doc::LayerInfo, String> {
        self.authorize(Perm::Read)?;
        let l = self.layer(index as u64)?;
        Ok(comp::craft::bench::doc::LayerInfo { id: l.id, name: l.name.clone(), visible: l.visible, opacity: l.opacity })
    }
    fn set_opacity(&mut self, index: u32, v: f32) -> Result<(), String> {
        Host::set_opacity(self, index, v)
    }
    fn open_layer(&mut self, index: u32) -> Result<wasmtime::component::Resource<LayerRep>, String> {
        self.authorize(Perm::Read)?;
        self.layer(index as u64)?;
        self.table.push(LayerRep { index }).map_err(|e| e.to_string())
    }
    fn batch(&mut self, ops: Vec<comp::craft::bench::doc::Op>) -> Vec<comp::craft::bench::doc::Reply> {
        use comp::craft::bench::doc::{LayerInfo, OpKind, Reply};
        ops.into_iter()
            .map(|o| match o.kind {
                OpKind::GetLayer => match self.authorize(Perm::Read).and_then(|_| self.layer(o.layer as u64).cloned()) {
                    Ok(l) => Reply::Layer(LayerInfo { id: l.id, name: l.name, visible: l.visible, opacity: l.opacity }),
                    Err(e) => Reply::Failed(e),
                },
                OpKind::SetOpacity => match Host::set_opacity(self, o.layer, o.value) {
                    Ok(()) => Reply::Done,
                    Err(e) => Reply::Failed(e),
                },
            })
            .collect()
    }
    fn call(&mut self, op: String, args: String) -> Result<String, String> {
        let v = self.json_call(&op, args.as_bytes())?;
        String::from_utf8(v).map_err(|e| e.to_string())
    }
    fn noop(&mut self) {}
}

impl comp::craft::bench::doc::HostLayer for Host {
    fn opacity(&mut self, h: wasmtime::component::Resource<LayerRep>) -> f32 {
        let i = self.table.get(&h).unwrap().index;
        self.layers[i as usize].opacity
    }
    fn set_opacity(&mut self, h: wasmtime::component::Resource<LayerRep>, v: f32) -> Result<(), String> {
        let i = self.table.get(&h).map_err(|e| e.to_string())?.index;
        Host::set_opacity(self, i, v)
    }
    fn drop(&mut self, h: wasmtime::component::Resource<LayerRep>) -> wasmtime::Result<()> {
        self.table.delete(h)?;
        Ok(())
    }
}

struct HostView;
impl wasmtime::component::HasData for HostView {
    type Data<'a> = &'a mut Host;
}

fn wasmtime_component(engine: &wasmtime::Engine, core: &[u8]) -> wasmtime::Result<()> {
    use wasmtime::Store;
    use wasmtime::component::{Component, Linker};
    let t = Instant::now();
    let bytes = wit_component::ComponentEncoder::default().module(core).and_then(|e| e.validate(true).encode()).map_err(|e| wasmtime::format_err!("{e}"))?;
    let enc = t.elapsed();
    let t = Instant::now();
    let component = Component::new(engine, &bytes)?;
    println!("component: encode {:.1} ms, compile {:.1} ms, {} bytes", enc.as_secs_f64() * 1e3, t.elapsed().as_secs_f64() * 1e3, bytes.len());
    let mut linker: Linker<Host> = Linker::new(engine);
    comp::craft::bench::doc::add_to_linker::<Host, HostView>(&mut linker, |h| h)?;
    let mut store = Store::new(engine, Host::new());
    store.set_epoch_deadline(u64::MAX / 2);
    let p = comp::Plugin::instantiate(&mut store, &component, &linker)?;
    let n = 10_000;
    let mut go = |label: &str, mode: u32, n: u32| {
        let s = time(|| p.call_run(&mut store, mode, n).unwrap());
        report(label, s, n);
    };
    go("wasmtime component: export call, no imports", 99, 1);
    go("wasmtime component: noop import", 5, n);
    go("wasmtime component: typed set-opacity(u32,f32)->result", 1, n);
    go("wasmtime component: typed get-layer -> record{string}", 0, n);
    go("wasmtime component: resource method layer.set-opacity", 2, n);
    go("wasmtime component: open-layer + opacity + drop (handle churn)", 6, n);
    go("wasmtime component: JSON call(string,string) layer.get", 4, n);
    go("wasmtime component: batch list<op> 1000 x get-layer", 3, 1000);
    go("wasmtime component: batch list<op> 1000 x set-opacity", 7, 1000);
    Ok(())
}

fn native_json() {
    let mut h = Host::new();
    let n = 10_000u32;
    let s = time(|| {
        let mut acc = 0u64;
        for i in 0..n {
            let a = serde_json::to_vec(&json!({"layer": i % 1000, "name": "Layer"})).unwrap();
            let r = h.json_call("layer.get", &a).unwrap();
            let v: Value = serde_json::from_slice(&r).unwrap();
            acc += v["id"].as_u64().unwrap();
        }
        acc
    });
    report("native Rust: same JSON encode+host parse+reply+decode", s, n);
    let s = time(|| {
        let mut acc = 0u64;
        for i in 0..n {
            if h.set_opacity(i % 1000, 0.5).is_ok() {
                acc += 1;
            }
        }
        acc
    });
    report("native Rust: direct typed set_opacity", s, n);
}

fn main() -> wasmtime::Result<()> {
    let dir = std::env::args().nth(1).expect("wasm dir");
    let core = std::fs::read(format!("{dir}/guest_core.wasm"))?;
    let compg = std::fs::read(format!("{dir}/guest_comp.wasm"))?;
    native_json();
    let mut cfg = wasmtime::Config::new();
    cfg.epoch_interruption(true);
    let engine = wasmtime::Engine::new(&cfg)?;
    wasmtime_core(&engine, &core)?;
    wasmtime_component(&engine, &compg)?;
    wasmi_core(&core)?;
    if std::env::var("PULLEY").is_ok() {
        let mut cfg = wasmtime::Config::new();
        cfg.epoch_interruption(true);
        cfg.target("pulley64")?;
        let engine = wasmtime::Engine::new(&cfg)?;
        println!("--- pulley64 interpreter ---");
        wasmtime_core(&engine, &core)?;
        wasmtime_component(&engine, &compg)?;
    }
    Ok(())
}

use craft_ext_core::contract::Provided;
use craft_ext_core::id::{AppId, ExtensionId};
use craft_ext_core::permission::Permission;
use craft_ext_core::version::Version;
use craft_ext_host::api::*;
use craft_ext_host::wasm::*;
use craft_ext_kernels as k;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

const W: usize = 1920;
const H: usize = 1080;

fn med(mut v: Vec<f64>) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }
fn time<F: FnMut()>(n: usize, mut f: F) -> f64 {
    f();
    med((0..n).map(|_| { let t = Instant::now(); f(); t.elapsed().as_secs_f64() * 1e3 }).collect())
}
fn img(w: usize, h: usize) -> Vec<f32> {
    (0..w * h * 4).map(|i| if i % 4 == 3 { 1.0 } else { ((i * 7919) % 1000) as f32 / 1000.0 }).collect()
}
fn ctx(params: Value, w: usize, h: usize) -> Value {
    json!({"params": params, "time": 0.0, "scale": 1.0, "origin": [0.0, 0.0], "full": [w, h], "quality": "final", "seed": 1, "color": "linear-srgb"})
}
fn log() -> LogFn { Arc::new(|_, _, m| eprintln!("plugin log: {m}")) }

struct App(AppContract);
impl AppApi for App {
    fn contract(&self) -> &AppContract { &self.0 }
    fn begin(&self, _: &Invocation) -> Result<(), OpError> { Ok(()) }
    fn call(&self, _: &Invocation, _op: &str, args: Value) -> Result<Value, OpError> {
        Ok(json!({"id": args.get("layer").cloned().unwrap_or(Value::Null), "name": "Layer 1", "visible": true, "opacity": 1.0}))
    }
    fn finish(&self, _: &Invocation, _: bool) {}
}
struct G;
impl Grants for G {
    fn generation(&self, _: &ExtensionId) -> u64 { 1 }
    fn allowed(&self, _: &ExtensionId, _: &Permission) -> bool { true }
    fn stopped(&self, _: &ExtensionId, _: Version) -> bool { false }
    fn full_access(&self, _: &ExtensionId) -> bool { false }
}
fn inv() -> Invocation {
    Invocation { app: AppId::new("testcraft").unwrap(), plugin: ExtensionId::new("org.example.bench").unwrap(), version: Version::new(1, 0, 0),
        contribution: "c".into(), grant_generation: 1, cancel: Arc::new(AtomicBool::new(false)), deadline: None, label: "x".into(), document: None, declared: Vec::<Permission>::new().into() }
}

fn raw_wasmi(bytes: &[u8], fuel: bool, w: usize, h: usize) -> f64 {
    use wasmi::*;
    let mut c = Config::default();
    c.consume_fuel(fuel);
    let e = Engine::new(&c);
    let m = Module::new(&e, bytes).unwrap();
    let mut s = Store::new(&e, ());
    if fuel { s.set_fuel(u64::MAX / 2).unwrap(); }
    let mut l = Linker::<()>::new(&e);
    l.func_wrap("craft", "log", |_: i32, _: i32, _: i32| {}).unwrap();
    l.func_wrap("craft", "call", |_: i32, _: i32, _: i32, _: i32| -> i64 { -3 }).unwrap();
    l.func_wrap("craft", "take", |_: i32, _: i32| -> i32 { -3 }).unwrap();
    l.func_wrap("craft", "result", |_: i32, _: i32| -> i32 { 0 }).unwrap();
    let i = l.instantiate_and_start(&mut s, &m).unwrap();
    let mem = i.get_memory(&s, "memory").unwrap();
    let alloc = i.get_typed_func::<i32, i32>(&s, "craft_alloc").unwrap();
    let f = i.get_typed_func::<(i32, i32, i32, i32, i32), i32>(&s, "craft_filter").unwrap();
    let c = serde_json::to_vec(&ctx(json!({"gain": 1.1}), w, h)).unwrap();
    let cp = alloc.call(&mut s, c.len() as i32).unwrap();
    mem.write(&mut s, cp as usize, &c).unwrap();
    let px: Vec<u8> = img(w, h).iter().flat_map(|v| v.to_le_bytes()).collect();
    let pp = alloc.call(&mut s, px.len() as i32).unwrap();
    mem.write(&mut s, pp as usize, &px).unwrap();
    time(5, || { assert_eq!(f.call(&mut s, (cp, c.len() as i32, pp, w as i32, h as i32)).unwrap(), 0); })
}

fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let wasm = |n: &str| std::fs::read(format!("{dir}/{n}.wasm")).unwrap();
    let bright = wasm("guest_bright");
    let px0 = img(W, H);

    let native = time(9, || {
        let mut p = px0.clone();
        let t = Instant::now();
        let gain = std::hint::black_box(1.1f32);
        for q in p.chunks_exact_mut(4) { q[0] *= gain; q[1] *= gain; q[2] *= gain; }
        std::hint::black_box(&p);
        let _ = t;
    });
    let clone_only = time(9, || { std::hint::black_box(px0.clone()); });
    println!("native_bright_1080p_ms(incl clone) {native:.2} clone_only {clone_only:.2}");

    let t = Instant::now();
    let p = Plugin::load("bench.bright", &bright).unwrap();
    println!("plugin_load_compile_ms {:.2}", t.elapsed().as_secs_f64() * 1e3);
    let t = Instant::now();
    let mut f = Filter::new(&p, log()).unwrap();
    println!("filter_instantiate_ms {:.2}", t.elapsed().as_secs_f64() * 1e3);
    let c = ctx(json!({"gain": 1.1}), W, H);
    let mut px = px0.clone();
    let e2e = time(5, || { f.run(&c, &mut px, W as u32, H as u32, &|| Ok(())).unwrap(); });
    println!("wasmi_platform_filter_run_e2e_1080p_ms {e2e:.2}");
    println!("raw_wasmi_portable_fuel_1080p_ms {:.2}", raw_wasmi(&bright, true, W, H));
    println!("raw_wasmi_portable_nofuel_1080p_ms {:.2}", raw_wasmi(&bright, false, W, H));
    if let Ok(simd) = std::fs::read(format!("{dir}/../../../target-simd/wasm32-unknown-unknown/release/guest_bright.wasm")) {
        println!("simd_module_load_platform: {:?}", Plugin::load("x.y", &simd).err());
    }

    let contract = AppContract { app: AppId::new("testcraft").unwrap(), display_name: "T", provided: Provided::new(&[("testcraft.session", 1, 0)]),
        ops: vec![OpSpec { id: "layer.get", kind: OpKind::Query, summary: "", args: "{}", result: "{}", requires: none, permission_rule: "" }], events: vec![], slots: vec![], notes: "" };
    let ses = Plugin::load("bench.session", &wasm("guest_session")).unwrap();
    let mut s = Session::new(&ses, Arc::new(App(contract)), Arc::new(G), log()).unwrap();
    for n in [1u32, 1000, 10000] {
        let ev = json!({"event": "command", "id": format!("layer.get:{n}")});
        let ms = time(5, || { s.dispatch(inv(), &ev).unwrap(); });
        println!("session_calls n={n} event_ms {ms:.3} per_call_us {:.2} calls_per_sec {:.0}", ms * 1e3 / n as f64, n as f64 / (ms / 1e3));
    }
    let ev = json!({"event": "command", "id": "layer.get:30000"});
    println!("session_30000_calls_one_event: {:?}", s.dispatch(inv(), &ev).err());
    let empty = json!({"event": "command", "id": "layer.get:0"});
    println!("session_empty_event_ms {:.3}", time(21, || { s.dispatch(inv(), &empty).unwrap(); }));
    let v = json!({"layer": 3, "name": "Layer"});
    let n = 200_000;
    let t = Instant::now();
    for _ in 0..n {
        let b = serde_json::to_vec(&v).unwrap();
        let x: Value = serde_json::from_slice(&b).unwrap();
        let r = serde_json::to_vec(&json!({"id": x.get("layer").cloned(), "name": "Layer 1", "visible": true, "opacity": 1.0})).unwrap();
        let y: Value = serde_json::from_slice(&r).unwrap();
        std::hint::black_box(y);
    }
    println!("native_json_roundtrip_us {:.3}", t.elapsed().as_secs_f64() * 1e6 / n as f64);

    for (w, h, tag) in [(1920usize, 1080usize, "1080p"), (3840, 2160, "4K")] {
        let base = k::Image { width: w, height: h, px: img(w, h) };
        let blur = time(5, || { let mut i = [base.clone()]; k::run("kernel.blur", &json!({"radius": 20.0}), &mut i, &[]).unwrap(); });
        let blur5 = time(5, || { let mut i = [base.clone()]; k::run("kernel.blur", &json!({"radius": 5.0}), &mut i, &[]).unwrap(); });
        let blend = time(5, || { let mut i = [base.clone(), base.clone()]; k::run("kernel.blend", &json!({"mode": "overlay", "opacity": 0.7}), &mut i, &[]).unwrap(); });
        let blendn = time(5, || { let mut i = [base.clone(), base.clone()]; k::run("kernel.blend", &json!({"mode": "normal", "opacity": 0.7}), &mut i, &[]).unwrap(); });
        let cl = time(5, || { std::hint::black_box(base.clone()); });
        println!("kernel {tag} blur_r20_ms {blur:.2} blur_r5_ms {blur5:.2} blend_overlay_ms {blend:.2} blend_normal_ms {blendn:.2} (clone of one image {cl:.2})");
        let bp = Plugin::load("bench.blur", &wasm("guest_blur")).unwrap();
        let mut f = Filter::new(&bp, log()).unwrap();
        let c = ctx(json!({"radius": 20.0}), w, h);
        let mut px = base.px.clone();
        let ms = time(3, || { f.run(&c, &mut px, w as u32, h as u32, &|| Ok(())).unwrap(); });
        println!("wasm_filter_calling_kernel_blur_r20 {tag} e2e_ms {ms:.2}");
        let bf = Plugin::load("bench.bright", &bright).unwrap();
        let mut f = Filter::new(&bf, log()).unwrap();
        let c = ctx(json!({"gain": 1.1}), w, h);
        let ms = time(3, || { f.run(&c, &mut px, w as u32, h as u32, &|| Ok(())).unwrap(); });
        println!("wasm_filter_bright {tag} e2e_ms {ms:.2}");
    }
    println!("threads {:?}", std::thread::available_parallelism());
}

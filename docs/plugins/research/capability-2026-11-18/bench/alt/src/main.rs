use serde_json::json;
use std::time::Instant;
fn med(mut v: Vec<f64>) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }
fn time<F: FnMut()>(n: usize, mut f: F) -> f64 { f(); med((0..n).map(|_| { let t = Instant::now(); f(); t.elapsed().as_secs_f64() * 1e3 }).collect()) }
fn inputs(w: usize, h: usize) -> (Vec<u8>, Vec<u8>) {
    let c = serde_json::to_vec(&json!({"params": {"gain": 1.1}, "time": 0.0, "scale": 1.0, "origin": [0.0, 0.0], "full": [w, h], "quality": "final", "seed": 1, "color": "linear-srgb"})).unwrap();
    let px = (0..w * h * 4).map(|i| if i % 4 == 3 { 1.0f32 } else { ((i * 7919) % 1000) as f32 / 1000.0 }).flat_map(|v| v.to_le_bytes()).collect();
    (c, px)
}
fn wasmi_run(bytes: &[u8], fuel: bool, w: usize, h: usize) -> (f64, f64) {
    use wasmi::*;
    let mut cfg = Config::default();
    cfg.consume_fuel(fuel);
    let e = Engine::new(&cfg);
    let t = Instant::now();
    let m = Module::new(&e, bytes).unwrap();
    let compile = t.elapsed().as_secs_f64() * 1e3;
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
    let (c, px) = inputs(w, h);
    let cp = alloc.call(&mut s, c.len() as i32).unwrap();
    mem.write(&mut s, cp as usize, &c).unwrap();
    let pp = alloc.call(&mut s, px.len() as i32).unwrap();
    mem.write(&mut s, pp as usize, &px).unwrap();
    (compile, time(5, || { assert_eq!(f.call(&mut s, (cp, c.len() as i32, pp, w as i32, h as i32)).unwrap(), 0); }))
}
fn wt_run(bytes: &[u8], strat: wasmtime::Strategy, fuel: bool, epoch: bool, w: usize, h: usize) -> (f64, f64) {
    use wasmtime::*;
    let mut cfg = Config::new();
    cfg.strategy(strat).consume_fuel(fuel).epoch_interruption(epoch);
    let e = Engine::new(&cfg).unwrap();
    let t = Instant::now();
    let m = Module::new(&e, bytes).unwrap();
    let compile = t.elapsed().as_secs_f64() * 1e3;
    let mut s = Store::new(&e, ());
    if fuel { s.set_fuel(u64::MAX / 2).unwrap(); }
    if epoch { s.set_epoch_deadline(u64::MAX / 2); }
    let mut l = Linker::<()>::new(&e);
    l.func_wrap("craft", "log", |_: i32, _: i32, _: i32| {}).unwrap();
    l.func_wrap("craft", "call", |_: i32, _: i32, _: i32, _: i32| -> i64 { -3 }).unwrap();
    l.func_wrap("craft", "take", |_: i32, _: i32| -> i32 { -3 }).unwrap();
    l.func_wrap("craft", "result", |_: i32, _: i32| -> i32 { 0 }).unwrap();
    let i = l.instantiate(&mut s, &m).unwrap();
    let mem = i.get_memory(&mut s, "memory").unwrap();
    let alloc = i.get_typed_func::<i32, i32>(&mut s, "craft_alloc").unwrap();
    let f = i.get_typed_func::<(i32, i32, i32, i32, i32), i32>(&mut s, "craft_filter").unwrap();
    let (c, px) = inputs(w, h);
    let cp = alloc.call(&mut s, c.len() as i32).unwrap();
    mem.write(&mut s, cp as usize, &c).unwrap();
    let pp = alloc.call(&mut s, px.len() as i32).unwrap();
    mem.write(&mut s, pp as usize, &px).unwrap();
    (compile, time(5, || { assert_eq!(f.call(&mut s, (cp, c.len() as i32, pp, w as i32, h as i32)).unwrap(), 0); }))
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let scalar = std::fs::read(&a[1]).unwrap();
    let simd = std::fs::read(&a[2]).unwrap();
    for (w, h, tag) in [(1920, 1080, "1080p"), (3840, 2160, "4K")] {
        for (name, m) in [("scalar", &scalar), ("simd128", &simd)] {
            for fuel in [true, false] {
                let (c, r) = wasmi_run(m, fuel, w, h);
                println!("{tag} {name} wasmi2-tailcall fuel={fuel} compile_ms {c:.2} run_ms {r:.2}");
            }
            for (sn, st) in [("cranelift", wasmtime::Strategy::Cranelift), ("winch", wasmtime::Strategy::Winch)] {
                for (fuel, epoch) in [(false, false), (true, false), (false, true)] {
                    let (c, r) = wt_run(m, st, fuel, epoch, w, h);
                    println!("{tag} {name} wasmtime50-{sn} fuel={fuel} epoch={epoch} compile_ms {c:.2} run_ms {r:.2}");
                }
            }
        }
    }
}

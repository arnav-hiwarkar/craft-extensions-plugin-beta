use std::time::Instant;
use wasmtime::*;

const W: usize = 1920;
const H: usize = 1080;
const N: usize = W * H * 4;

fn med(mut v: Vec<f64>) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }
fn time<F: FnMut()>(n: usize, mut f: F) -> f64 { f(); med((0..n).map(|_| { let t = Instant::now(); f(); t.elapsed().as_secs_f64() * 1e3 }).collect()) }
fn rss_mb() -> f64 {
    unsafe {
        let mut info: libc::proc_taskinfo = std::mem::zeroed();
        let sz = std::mem::size_of::<libc::proc_taskinfo>() as i32;
        libc::proc_pidinfo(libc::getpid(), libc::PROC_PIDTASKINFO, 0, &mut info as *mut _ as *mut libc::c_void, sz);
        info.pti_resident_size as f64 / 1048576.0
    }
}
fn as_bytes(v: &[f32]) -> &[u8] { unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) } }
fn as_bytes_mut(v: &mut [f32]) -> &mut [u8] { unsafe { std::slice::from_raw_parts_mut(v.as_mut_ptr() as *mut u8, v.len() * 4) } }

struct Frame { src: Vec<f32>, tile_reads: u64 }

fn tiles_linker(e: &Engine) -> Linker<Frame> {
    let mut l = Linker::<Frame>::new(e);
    l.func_wrap("host", "read_tile", |mut c: Caller<'_, Frame>, x: u32, y: u32, w: u32, h: u32, dst: u32| -> i32 {
        let mem = c.get_export("memory").unwrap().into_memory().unwrap();
        let (data, st) = mem.data_and_store_mut(&mut c);
        let (x, y, w, h, dst) = (x as usize, y as usize, w as usize, h as usize, dst as usize);
        let row = w * 16;
        if dst + row * h > data.len() { return -1; }
        for r in 0..h {
            let s = ((y + r) * W + x) * 4;
            data[dst + r * row..dst + (r + 1) * row].copy_from_slice(as_bytes(&st.src[s..s + w * 4]));
        }
        st.tile_reads += 1;
        0
    }).unwrap();
    l.func_wrap("host", "write_tile", |mut c: Caller<'_, Frame>, x: u32, y: u32, w: u32, h: u32, src: u32| -> i32 {
        let mem = c.get_export("memory").unwrap().into_memory().unwrap();
        let (data, st) = mem.data_and_store_mut(&mut c);
        let (x, y, w, h, src) = (x as usize, y as usize, w as usize, h as usize, src as usize);
        let row = w * 16;
        if src + row * h > data.len() { return -1; }
        for r in 0..h {
            let d = ((y + r) * W + x) * 4;
            as_bytes_mut(&mut st.src[d..d + w * 4]).copy_from_slice(&data[src + r * row..src + (r + 1) * row]);
        }
        0
    }).unwrap();
    l.func_wrap("host", "tick", || -> i32 { 1 }).unwrap();
    l
}

fn frame() -> Vec<f32> { (0..N).map(|i| if i % 4 == 3 { 1.0 } else { ((i * 7919) % 1000) as f32 / 1000.0 }).collect() }

fn data_plane(tiles_wasm: &[u8]) {
    println!("## data plane, 1080p RGBA f32 = {:.1} MB", (N * 4) as f64 / 1e6);
    let mut native = frame();
    println!("native_bright_inplace_ms {:.3}", time(9, || { for p in native.chunks_exact_mut(4) { p[0] *= 1.0001; p[1] *= 1.0001; p[2] *= 1.0001; } }));
    let a = frame(); let mut b = vec![0f32; N];
    println!("native_memcpy_33MB_ms {:.3}", time(9, || b.copy_from_slice(&a)));

    let e = Engine::default();
    let m = Module::new(&e, tiles_wasm).unwrap();
    let mut s = Store::new(&e, Frame { src: frame(), tile_reads: 0 });
    let l = tiles_linker(&e);
    let i = l.instantiate(&mut s, &m).unwrap();
    let mem = i.get_memory(&mut s, "memory").unwrap();
    let alloc = i.get_typed_func::<u32, u32>(&mut s, "alloc").unwrap();
    let bright = i.get_typed_func::<(u32, u32, f32), ()>(&mut s, "bright").unwrap();
    let p = alloc.call(&mut s, (N * 4) as u32).unwrap();
    let host = frame(); let mut out = vec![0f32; N];
    let copy_in = time(9, || mem.write(&mut s, p as usize, as_bytes(&host)).unwrap());
    let run = time(9, || bright.call(&mut s, (p, N as u32, 1.0001)).unwrap());
    let copy_out = time(9, || mem.read(&s, p as usize, as_bytes_mut(&mut out)).unwrap());
    let e2e = time(9, || {
        mem.write(&mut s, p as usize, as_bytes(&host)).unwrap();
        bright.call(&mut s, (p, N as u32, 1.0001)).unwrap();
        mem.read(&s, p as usize, as_bytes_mut(&mut out)).unwrap();
    });
    println!("A copy: copy_in_ms {copy_in:.3} run_ms {run:.3} copy_out_ms {copy_out:.3} e2e_ms {e2e:.3}");
    println!("B view (host produced into guest memory, guest runs in place): run_ms {run:.3}");

    let src8: Vec<u8> = (0..N).map(|i| (i % 251) as u8).collect();
    let mut staging = vec![0f32; N];
    let conv_then_copy = time(9, || {
        for (d, s8) in staging.iter_mut().zip(&src8) { *d = *s8 as f32 * (1.0 / 255.0); }
        mem.write(&mut s, p as usize, as_bytes(&staging)).unwrap();
    });
    let conv_direct = time(9, || {
        let g = &mut mem.data_mut(&mut s)[p as usize..p as usize + N * 4];
        let g: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(g.as_mut_ptr() as *mut f32, N) };
        for (d, s8) in g.iter_mut().zip(&src8) { *d = *s8 as f32 * (1.0 / 255.0); }
    });
    println!("C rgba8->f32: convert_to_vec_then_copy_ms {conv_then_copy:.3} convert_directly_into_guest_ms {conv_direct:.3}");
    let src16: Vec<half::f16> = (0..N).map(|i| half::f16::from_f32((i % 1000) as f32 / 1000.0)).collect();
    let conv16 = time(9, || {
        let g = &mut mem.data_mut(&mut s)[p as usize..p as usize + N * 4];
        let g: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(g.as_mut_ptr() as *mut f32, N) };
        for (d, s16) in g.iter_mut().zip(&src16) { *d = s16.to_f32(); }
    });
    println!("C rgba16f->f32 directly into guest_ms {conv16:.3}");
    let validate = time(9, || {
        let g = &mut mem.data_mut(&mut s)[p as usize..p as usize + N * 4];
        let g: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(g.as_mut_ptr() as *mut f32, N) };
        let mut bad = 0u32;
        for px in g.chunks_exact_mut(4) {
            let a = px[3].clamp(0.0, 1.0); px[3] = a;
            for c in &mut px[..3] { if !c.is_finite() { *c = 0.0; bad += 1; } }
            if a == 0.0 { px[0] = 0.0; px[1] = 0.0; px[2] = 0.0; }
        }
        std::hint::black_box(bad);
    });
    println!("fused_validate_in_place_ms {validate:.3}");

    unsafe {
        let base = mem.data_ptr(&s);
        let pg = 16384usize;
        let off = ((p as usize) + pg - 1) / pg * pg;
        let len = (N * 4 + pg - 1) / pg * pg;
        let mut path = *b"/tmp/craftshmXXXXXX\0";
        let fd = libc::mkstemp(path.as_mut_ptr() as *mut libc::c_char);
        libc::unlink(path.as_ptr() as *const libc::c_char);
        libc::ftruncate(fd, len as i64);
        let hostmap = libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0) as *mut f32;
        let hv = std::slice::from_raw_parts_mut(hostmap, N);
        hv.copy_from_slice(&host);
        let target = base.add(off) as *mut libc::c_void;
        let gptr = (off) as u32;
        let mut remap = vec![]; let mut first = vec![]; let mut warm = vec![]; let mut unmap = vec![];
        for _ in 0..7 {
            let t = Instant::now();
            let r = libc::mmap(target, len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED | libc::MAP_FIXED, fd, 0);
            assert_eq!(r, target);
            remap.push(t.elapsed().as_secs_f64() * 1e3);
            let t = Instant::now();
            bright.call(&mut s, (gptr, N as u32, 1.0001)).unwrap();
            first.push(t.elapsed().as_secs_f64() * 1e3);
            let t = Instant::now();
            bright.call(&mut s, (gptr, N as u32, 1.0001)).unwrap();
            warm.push(t.elapsed().as_secs_f64() * 1e3);
            let t = Instant::now();
            let r = libc::mmap(target, len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_FIXED, -1, 0);
            assert_eq!(r, target);
            unmap.push(t.elapsed().as_secs_f64() * 1e3);
        }
        println!("D mmap-alias (shm frame MAP_FIXED into guest memory): remap_ms {:.3} first_run_ms(page faults) {:.3} second_run_ms {:.3} restore_ms {:.3} host_sees_result {}",
            med(remap), med(first), med(warm), med(unmap), hv[1] != host[1]);
        libc::munmap(hostmap as *mut libc::c_void, len);
        libc::close(fd);
    }

    let tiles = i.get_typed_func::<(u32, u32, u32, f32, u32), i32>(&mut s, "bright_tiles").unwrap();
    for t in [64u32, 128, 256, 512, 1024] {
        let scratch = alloc.call(&mut s, t * t * 16).unwrap();
        s.data_mut().tile_reads = 0;
        let ms = time(7, || assert_eq!(tiles.call(&mut s, (W as u32, H as u32, t, 1.0001, scratch)).unwrap(), 0));
        println!("E host-resident handle, read_tile/write_tile tile={t}: e2e_ms {ms:.3} calls_per_frame {}", s.data().tile_reads / 8 * 2);
    }
    let calls = i.get_typed_func::<u32, i32>(&mut s, "calls").unwrap();
    let ms = time(5, || { calls.call(&mut s, 1_000_000).unwrap(); });
    println!("F host import call (typed, no args): {:.1} ns/call", ms * 1e6 / 1e6);
    let noop = i.get_typed_func::<(), i32>(&mut s, "noop").unwrap();
    let ms = time(5, || { for _ in 0..100_000 { noop.call(&mut s, ()).unwrap(); } });
    println!("F host->guest typed call: {:.1} ns/call", ms * 1e6 / 1e5);

    for threads in [1usize, 2, 4, 8] {
        let band = H / threads;
        let mut stores: Vec<_> = (0..threads).map(|_| {
            let mut s = Store::new(&e, Frame { src: vec![], tile_reads: 0 });
            let i = l.instantiate(&mut s, &m).unwrap();
            let mem = i.get_memory(&mut s, "memory").unwrap();
            let alloc = i.get_typed_func::<u32, u32>(&mut s, "alloc").unwrap();
            let f = i.get_typed_func::<(u32, u32, f32), ()>(&mut s, "bright").unwrap();
            let p = alloc.call(&mut s, (band * W * 16) as u32).unwrap();
            (s, mem, f, p)
        }).collect();
        let mut img = frame();
        let ms = time(7, || {
            std::thread::scope(|sc| {
                for ((s, mem, f, p), chunk) in stores.iter_mut().zip(img.chunks_mut(band * W * 4)) {
                    sc.spawn(move || {
                        mem.write(&mut *s, *p as usize, as_bytes(chunk)).unwrap();
                        f.call(&mut *s, (*p, chunk.len() as u32, 1.0001)).unwrap();
                        mem.read(&*s, *p as usize, as_bytes_mut(chunk)).unwrap();
                    });
                }
            });
        });
        println!("G parallel bands threads={threads} (copy in + run + copy out per band): e2e_ms {ms:.3}");
    }
    let mut stores: Vec<_> = (0..8).map(|_| {
        let mut s = Store::new(&e, Frame { src: vec![], tile_reads: 0 });
        let i = l.instantiate(&mut s, &m).unwrap();
        let mem = i.get_memory(&mut s, "memory").unwrap();
        let alloc = i.get_typed_func::<u32, u32>(&mut s, "alloc").unwrap();
        let f = i.get_typed_func::<(u32, u32, f32), ()>(&mut s, "bright").unwrap();
        let p = alloc.call(&mut s, (H / 8 * W * 16) as u32).unwrap();
        (s, mem, f, p)
    }).collect();
    let ms = time(7, || {
        std::thread::scope(|sc| {
            for (s, _mem, f, p) in stores.iter_mut() {
                sc.spawn(move || { f.call(&mut *s, (*p, (H / 8 * W * 4) as u32, 1.0001)).unwrap(); });
            }
        });
    });
    println!("G parallel bands threads=8 view only (no copies): e2e_ms {ms:.3}");
}

fn craft_linker(e: &Engine) -> Linker<()> {
    let mut l = Linker::<()>::new(e);
    l.func_wrap("craft", "log", |_: i32, _: i32, _: i32| {}).unwrap();
    l.func_wrap("craft", "call", |_: i32, _: i32, _: i32, _: i32| -> i64 { -3 }).unwrap();
    l.func_wrap("craft", "take", |_: i32, _: i32| -> i32 { -3 }).unwrap();
    l.func_wrap("craft", "result", |_: i32, _: i32| -> i32 { 0 }).unwrap();
    l
}

fn engine(pool: bool, strat: Strategy, epoch: bool) -> Engine {
    let mut c = Config::new();
    c.strategy(strat).epoch_interruption(epoch);
    if pool {
        let mut p = PoolingAllocationConfig::new();
        p.total_core_instances(1000).total_memories(1000).total_tables(1000).max_memory_size(512 << 20);
        c.allocation_strategy(InstanceAllocationStrategy::Pooling(p));
    }
    Engine::new(&c).unwrap()
}

fn lifecycle(big: &[u8], tag: &str, dir: &std::path::Path) {
    println!("## lifecycle, module {tag} {} KB", big.len() / 1024);
    for (sn, st) in [("cranelift", Strategy::Cranelift), ("winch", Strategy::Winch)] {
        let e = engine(false, st, true);
        let mut v = vec![];
        for _ in 0..5 { let t = Instant::now(); let _ = Module::new(&e, big).unwrap(); v.push(t.elapsed().as_secs_f64() * 1e3); }
        let m = Module::new(&e, big).unwrap();
        let ser = m.serialize().unwrap();
        let path = dir.join(format!("{tag}-{sn}.cwasm"));
        std::fs::write(&path, &ser).unwrap();
        let de = time(21, || { let _ = unsafe { Module::deserialize(&e, &ser) }.unwrap(); });
        let def = time(21, || { let _ = unsafe { Module::deserialize_file(&e, &path) }.unwrap(); });
        println!("{sn}: compile_ms {:.2} artifact_kb {} deserialize_bytes_ms {de:.3} deserialize_file_mmap_ms {def:.3}", med(v), ser.len() / 1024);
    }
    let t = Instant::now(); let e_def = engine(false, Strategy::Cranelift, true); let e_def_ms = t.elapsed().as_secs_f64() * 1e3;
    let t = Instant::now(); let e_pool = engine(true, Strategy::Cranelift, true); let e_pool_ms = t.elapsed().as_secs_f64() * 1e3;
    println!("engine_new_ms on_demand {e_def_ms:.3} pooling(1000 slots, 512MiB max mem) {e_pool_ms:.3}");
    for (name, e) in [("on_demand", &e_def), ("pooling", &e_pool)] {
        let m = Module::new(e, big).unwrap();
        let l = craft_linker(e);
        let pre = l.instantiate_pre(&m).unwrap();
        let t = Instant::now();
        { let mut s = Store::new(e, ()); s.set_epoch_deadline(1 << 40); let _ = pre.instantiate(&mut s).unwrap(); }
        let cold = t.elapsed().as_secs_f64() * 1e3;
        let mut v = vec![];
        for _ in 0..2000 {
            let t = Instant::now();
            let mut s = Store::new(e, ()); s.set_epoch_deadline(1 << 40);
            let i = pre.instantiate(&mut s).unwrap();
            std::hint::black_box(&i);
            drop(s);
            v.push(t.elapsed().as_secs_f64() * 1e6);
        }
        let mut v2 = vec![];
        for _ in 0..500 {
            let t = Instant::now();
            let mut s = Store::new(e, ()); s.set_epoch_deadline(1 << 40);
            let i = pre.instantiate(&mut s).unwrap();
            let mem = i.get_memory(&mut s, "memory").unwrap();
            mem.grow(&mut s, 16).unwrap();
            mem.data_mut(&mut s)[..1 << 20].fill(1);
            drop(s);
            v2.push(t.elapsed().as_secs_f64() * 1e6);
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("instantiate {name}: cold_first_ms {cold:.3} warm_p50_us {:.1} warm_p99_us {:.1} with_1MiB_touched_p50_us {:.1}", v[v.len() / 2], v[v.len() * 99 / 100], med(v2));
    }
}

fn footprint(big: &[u8], dir: &std::path::Path) {
    println!("## footprint, 100 installed plugins (distinct copies of the same module)");
    let e = engine(false, Strategy::Cranelift, true);
    let m = Module::new(&e, big).unwrap();
    let ser = m.serialize().unwrap();
    drop(m);
    let paths: Vec<_> = (0..100).map(|k| { let p = dir.join(format!("p{k}.cwasm")); std::fs::write(&p, &ser).unwrap(); p }).collect();
    let r0 = rss_mb();
    let t = Instant::now();
    let mods: Vec<Module> = paths.iter().map(|p| unsafe { Module::deserialize_file(&e, p) }.unwrap()).collect();
    let load = t.elapsed().as_secs_f64() * 1e3;
    let r1 = rss_mb();
    let l = craft_linker(&e);
    let mut stores = vec![];
    let t = Instant::now();
    for m in &mods {
        let mut s = Store::new(&e, ()); s.set_epoch_deadline(1 << 40);
        let i = l.instantiate(&mut s, m).unwrap();
        stores.push((s, i));
    }
    let inst = t.elapsed().as_secs_f64() * 1e3;
    let r2 = rss_mb();
    println!("load_100_deserialize_file_ms {load:.2} rss_delta_mb {:.1} | instantiate_100_ms {inst:.2} rss_delta_mb {:.1} (initial memory {} KiB each)", r1 - r0, r2 - r1,
        { let (s, i) = &mut stores[0]; i.get_memory(&mut *s, "memory").unwrap().data_size(&*s) / 1024 });
    let wm = wasmi::Engine::default();
    let r3 = rss_mb();
    let t = Instant::now();
    let wmods: Vec<_> = (0..100).map(|_| wasmi::Module::new(&wm, big).unwrap()).collect();
    let wl = t.elapsed().as_secs_f64() * 1e3;
    let r4 = rss_mb();
    println!("wasmi_load_100_ms {wl:.2} rss_delta_mb {:.1}", r4 - r3);
    std::hint::black_box((&wmods, &stores));
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let tiles = std::fs::read(&a[1]).unwrap();
    let dir = std::path::PathBuf::from(&a[4]);
    std::fs::create_dir_all(&dir).unwrap();
    data_plane(&tiles);
    lifecycle(&std::fs::read(&a[2]).unwrap(), "lto", &dir);
    lifecycle(&std::fs::read(&a[3]).unwrap(), "nolto", &dir);
    footprint(&std::fs::read(&a[2]).unwrap(), &dir);
}

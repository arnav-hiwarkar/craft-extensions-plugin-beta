use std::time::Instant;
const W: u32 = 1920;
const H: u32 = 1080;

fn med(mut v: Vec<f64>) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }
fn time<F: FnMut()>(n: usize, mut f: F) -> f64 { f(); med((0..n).map(|_| { let t = Instant::now(); f(); t.elapsed().as_secs_f64() * 1e3 }).collect()) }

fn shader(fmt: &str) -> String {
    format!(r#"
struct P {{ gain: f32, _a: f32, _b: f32, _c: f32 }}
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var dst: texture_storage_2d<{fmt}, write>;
@group(0) @binding(2) var<uniform> p: P;
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let d = textureDimensions(src);
    if (id.x >= d.x || id.y >= d.y) {{ return; }}
    let c = textureLoad(src, vec2<i32>(id.xy), 0);
    textureStore(dst, vec2<i32>(id.xy), vec4<f32>(c.rgb * p.gain, c.a));
}}
"#)
}

const HEAVY: &str = r#"
struct P { gain: f32, radius: f32, _b: f32, _c: f32 }
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var dst: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var<uniform> p: P;
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let d = vec2<i32>(textureDimensions(src));
    if (i32(id.x) >= d.x || i32(id.y) >= d.y) { return; }
    var acc = vec4<f32>(0.0);
    let r = i32(p.radius);
    for (var dy = -r; dy <= r; dy++) {
        for (var dx = -r; dx <= r; dx++) {
            acc += textureLoad(src, clamp(vec2<i32>(id.xy) + vec2<i32>(dx, dy), vec2<i32>(0), d - 1), 0);
        }
    }
    let n = f32((2 * r + 1) * (2 * r + 1));
    textureStore(dst, vec2<i32>(id.xy), acc / n);
}
"#;

struct Ctx { device: wgpu::Device, queue: wgpu::Queue, ts: bool, period: f32 }

fn wait(c: &Ctx) { c.device.poll(wgpu::PollType::wait_indefinitely()).unwrap(); }

fn tex(c: &Ctx, fmt: wgpu::TextureFormat, usage: wgpu::TextureUsages) -> wgpu::Texture {
    c.device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format: fmt, usage, view_formats: &[],
    })
}

fn run_format(c: &Ctx, fmt: wgpu::TextureFormat, wgsl_fmt: &str, bpp: u32, data: &[u8], heavy: bool) {
    let t = Instant::now();
    let src_code = if heavy { HEAVY.to_string() } else { shader(wgsl_fmt) };
    let module = c.device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(src_code.clone().into()) });
    let pipe = c.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None, layout: None, module: &module, entry_point: Some("main"), compilation_options: Default::default(), cache: None,
    });
    wait(c);
    let pipe_ms = t.elapsed().as_secs_f64() * 1e3;
    let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC;
    let a = tex(c, fmt, usage);
    let b = tex(c, fmt, usage);
    let ubuf = c.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    c.queue.write_buffer(&ubuf, 0, bytemuck_f32(&[1.0001, 3.0, 0.0, 0.0]));
    let bytes = (W * H * bpp) as u64;
    let rb = c.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: bytes, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    let bg = |s: &wgpu::Texture, d: &wgpu::Texture| c.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None, layout: &pipe.get_bind_group_layout(0), entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&s.create_view(&Default::default())) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&d.create_view(&Default::default())) },
            wgpu::BindGroupEntry { binding: 2, resource: ubuf.as_entire_binding() },
        ],
    });
    let ab = bg(&a, &b);
    let ba = bg(&b, &a);
    let qs = if c.ts { Some(c.device.create_query_set(&wgpu::QuerySetDescriptor { label: None, ty: wgpu::QueryType::Timestamp, count: 64 })) } else { None };
    let qbuf = c.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 512, usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
    let qrb = c.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 512, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    let upload = || {
        c.queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &a, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            data, wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(W * bpp), rows_per_image: Some(H) },
            wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 });
    };
    let dispatch = |enc: &mut wgpu::CommandEncoder, n: usize, timed: bool| {
        for k in 0..n {
            let tw = if timed && qs.is_some() {
                Some(wgpu::ComputePassTimestampWrites { query_set: qs.as_ref().unwrap(), beginning_of_pass_write_index: Some(2 * k as u32), end_of_pass_write_index: Some(2 * k as u32 + 1) })
            } else { None };
            let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: tw });
            p.set_pipeline(&pipe);
            p.set_bind_group(0, if k % 2 == 0 { &ab } else { &ba }, &[]);
            p.dispatch_workgroups(W.div_ceil(16), H.div_ceil(16), 1);
        }
    };
    let gpu_ms = |n: usize| -> Option<f64> {
        let qs = qs.as_ref()?;
        let mut enc = c.device.create_command_encoder(&Default::default());
        dispatch(&mut enc, n, true);
        enc.resolve_query_set(qs, 0..(2 * n as u32), &qbuf, 0);
        enc.copy_buffer_to_buffer(&qbuf, 0, &qrb, 0, 16 * n as u64);
        c.queue.submit([enc.finish()]);
        qrb.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        wait(c);
        let v = qrb.slice(..).get_mapped_range().unwrap();
        let mut sum = 0u64;
        for k in 0..n { let t0 = u64::from_le_bytes(v[16 * k..16 * k + 8].try_into().unwrap()); let t1 = u64::from_le_bytes(v[16 * k + 8..16 * k + 16].try_into().unwrap()); sum += t1.saturating_sub(t0); }
        drop(v); qrb.unmap();
        Some(sum as f64 * c.period as f64 / 1e6)
    };
    let readback = |out: &mut Vec<u8>, src: &wgpu::Texture, enc: &mut wgpu::CommandEncoder| {
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: src, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo { buffer: &rb, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(W * bpp), rows_per_image: Some(H) } },
            wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 });
        let _ = out;
    };
    let mut out = vec![0u8; bytes as usize];
    let up_ms = time(9, || { upload(); c.queue.submit([]); wait(c); });
    let resident1 = time(15, || { let mut e = c.device.create_command_encoder(&Default::default()); dispatch(&mut e, 1, false); c.queue.submit([e.finish()]); wait(c); });
    let resident20 = time(9, || { let mut e = c.device.create_command_encoder(&Default::default()); dispatch(&mut e, 20, false); c.queue.submit([e.finish()]); wait(c); });
    let g1 = (0..9).filter_map(|_| gpu_ms(1)).collect::<Vec<_>>();
    let g20 = (0..9).filter_map(|_| gpu_ms(20)).collect::<Vec<_>>();
    let rb_ms = time(9, || {
        let mut e = c.device.create_command_encoder(&Default::default());
        readback(&mut out, &a, &mut e);
        c.queue.submit([e.finish()]);
        rb.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        wait(c);
        out.copy_from_slice(&rb.slice(..).get_mapped_range().unwrap());
        rb.unmap();
    });
    let rt = time(9, || {
        upload();
        let mut e = c.device.create_command_encoder(&Default::default());
        dispatch(&mut e, 1, false);
        readback(&mut out, &b, &mut e);
        c.queue.submit([e.finish()]);
        rb.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        wait(c);
        out.copy_from_slice(&rb.slice(..).get_mapped_range().unwrap());
        rb.unmap();
    });
    let name = if heavy { format!("{wgsl_fmt}->rgba16float box7x7(49 taps)") } else { wgsl_fmt.to_string() };
    println!("{name} {:.1}MB: pipeline_create_ms {pipe_ms:.2} upload_ms {up_ms:.3} dispatch_submit_wait_ms(1 pass) {resident1:.3} (20 passes) {resident20:.3} gpu_ts_ms(1 pass) {} (20 passes) {} readback_ms {rb_ms:.3} roundtrip(upload+1 pass+readback)_ms {rt:.3}",
        bytes as f64 / 1e6,
        if g1.is_empty() { "n/a".into() } else { format!("{:.3}", med(g1)) },
        if g20.is_empty() { "n/a".into() } else { format!("{:.3}", med(g20)) });
}

fn bytemuck_f32(v: &[f32]) -> &[u8] { unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) } }

fn naga_validate() {
    let code = HEAVY;
    let t = Instant::now();
    let mut v = vec![];
    for _ in 0..50 {
        let t = Instant::now();
        let m = naga::front::wgsl::parse_str(code).unwrap();
        let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty()).validate(&m).unwrap();
        std::hint::black_box(&info);
        v.push(t.elapsed().as_secs_f64() * 1e3);
    }
    let _ = t;
    let m = naga::front::wgsl::parse_str(code).unwrap();
    let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty()).validate(&m).unwrap();
    let mut msl = vec![];
    for _ in 0..50 {
        let t = Instant::now();
        let opts = naga::back::msl::Options { bounds_check_policies: naga::proc::BoundsCheckPolicies { index: naga::proc::BoundsCheckPolicy::ReadZeroSkipWrite, buffer: naga::proc::BoundsCheckPolicy::ReadZeroSkipWrite, image_load: naga::proc::BoundsCheckPolicy::ReadZeroSkipWrite, binding_array: naga::proc::BoundsCheckPolicy::ReadZeroSkipWrite }, force_loop_bounding: true, ..Default::default() };
        let pipe = naga::back::msl::PipelineOptions::default();
        let s = naga::back::msl::write_string(&m, &info, &opts, &pipe).unwrap();
        std::hint::black_box(&s);
        msl.push(t.elapsed().as_secs_f64() * 1e3);
    }
    println!("naga: parse+validate_ms {:.3} msl_out_ms {:.3} (box blur WGSL, {} bytes)", med(v), med(msl), code.len());
}

fn compile_cost(c: &Ctx) {
    for (name, base) in [("simple", shader("rgba16float")), ("box7x7", HEAVY.to_string())] {
        let mut v = vec![];
        for k in 0..6 {
            let code = base.replace("c.rgb * p.gain", &format!("c.rgb * p.gain * {}.0", k + 2)).replace("acc / n", &format!("acc / n * {}.0", k + 2));
            let t = Instant::now();
            let m = c.device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(code.into()) });
            let _p = c.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: None, layout: None, module: &m, entry_point: Some("main"), compilation_options: Default::default(), cache: None });
            v.push(t.elapsed().as_secs_f64() * 1e3);
        }
        println!("pipeline_create_unique_{name}_ms first {:.2} median_of_rest {:.2}", v[0], med(v[1..].to_vec()));
    }
}

fn main() {
    naga_validate();
    let t = Instant::now();
    let inst = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(inst.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() })).unwrap();
    let ts = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
    let feats = if ts { wgpu::Features::TIMESTAMP_QUERY } else { wgpu::Features::empty() };
    let limits = wgpu::Limits { max_buffer_size: adapter.limits().max_buffer_size, max_texture_dimension_2d: 16384, ..Default::default() };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { required_features: feats, required_limits: limits, ..Default::default() })).unwrap();
    println!("adapter {:?} backend {:?} timestamp_query {ts} device_create_ms {:.1}", adapter.get_info().name, adapter.get_info().backend, t.elapsed().as_secs_f64() * 1e3);
    let period = queue.get_timestamp_period();
    let c = Ctx { device, queue, ts, period };
    compile_cost(&c);
    let n = (W * H * 4) as usize;
    let f32data: Vec<u8> = (0..n).flat_map(|i| (((i * 7919) % 1000) as f32 / 1000.0).to_le_bytes()).collect();
    let f16data: Vec<u8> = (0..n).flat_map(|i| half::f16::from_f32(((i * 7919) % 1000) as f32 / 1000.0).to_le_bytes()).collect();
    let u8data: Vec<u8> = (0..n).map(|i| (i % 251) as u8).collect();
    run_format(&c, wgpu::TextureFormat::Rgba32Float, "rgba32float", 16, &f32data, false);
    run_format(&c, wgpu::TextureFormat::Rgba16Float, "rgba16float", 8, &f16data, false);
    run_format(&c, wgpu::TextureFormat::Rgba8Unorm, "rgba8unorm", 4, &u8data, false);
    run_format(&c, wgpu::TextureFormat::Rgba16Float, "rgba16float", 8, &f16data, true);
}

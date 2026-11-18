# 03 — Performance and GPU architecture for Craft plugins

Date: 2026-11-18. Status: research and design only. No repository file was changed, nothing was
committed, and no app repository was built. Throwaway benchmarks were built and run in the session
scratchpad (`perf-bench/`, see §7). Machine: Apple M5, 10 cores, 24 GB, 16 KiB pages, macOS 25.5,
Rust 1.97.1, release builds. *Measured* marks numbers from this pass or the 2026-11-18 capability
benchmarks; everything else is an estimate or a cited source.

Inputs read: `craft-extensions/docs/plugins/research/capability-program-2026-11-18.md` §1–4,
`capability-2026-11-18/{platform,3d,audio}.md`, the prior bench crates in
`capability-2026-11-18/bench/`, and the GPU, ML and plugin code of all eleven apps, `craft-libs`
and `craft-extensions` (survey in §1).

---

## Summary

1. **Every app already runs one GPU stack: wgpu 30.0.1 and naga 30.0.1, with eframe/egui-wgpu
   0.37** (AniCraft is on 0.36.2). That means one plugin GPU host crate can serve the whole suite
   with no version matrix. There is no shared GPU crate today. Apps open extra devices for SAM2,
   the denoiser, face models, baking and export, and wgpu cannot share textures between devices.
2. **Plugins are CPU-only, so they break GPU chains.** EffectCraft's GPU compositor handles any
   step without a GPU kernel by reading the canvas back, running the step on the CPU and uploading
   the result (`effectcraft/crates/gpu/src/walk.rs:1-6`). That round trip alone costs
   **5.2 ms at 1080p RGBA32F and 2.4 ms at RGBA16F** (*measured*). The same brightness pass done
   on the GPU, where the image already lives, costs **0.39 ms / 0.18 ms of GPU time** (*measured*).
3. **The data plane should be views, not messages.** With Wasmtime, a host `memcpy` of a 1080p
   RGBA32F frame into and out of guest memory costs 0.56 + 0.56 ms around a 0.87 ms guest loop,
   2.1 ms in all. Today's `Filter::run` takes 52 ms. If the host writes its conversion straight into
   guest memory, the extra copy disappears: RGBA8 to f32 drops from 1.02 ms to 0.40 ms. Aliasing a
   shared-memory frame into guest memory with `mmap` maps in 3 µs, but page faults cost about
   1.3 ms on first touch, so it is no faster than a copy. **Recommendation:** host-materialised
   views in guest memory for dense data, and host-resident handles with region reads for big or
   random-access data. Do not use mmap aliasing.
4. **Run all desktop plugin code on Wasmtime with Cranelift, compiled ahead of time; never ship an
   interpreter on the hot path.** With parallel compilation, Cranelift compiles a 122–160 KB module
   in **8.9–11.5 ms**. A cached artifact deserializes in **0.03–0.05 ms**. Instantiation takes
   **6–7 µs pooled or 12 µs on demand** (*measured*). With 100 plugins installed, loading all of
   them costs **3.8 ms and 26 MB RSS** if done eagerly. Lazy loading makes the startup cost
   essentially zero.
5. **Keep wasmi only where a JIT is forbidden or unwanted:** iOS, deterministic reference tests,
   and as an emergency fallback. **On the web, use the browser's own WebAssembly engine**, not wasmi
   compiled to Wasm. The browser already JIT-compiles plugin modules at near-native speed.
6. **The GPU tier has four layers, in this order:**
   - **L1 `gpu.kernel`:** a WGSL function body. The host writes the entry point and bindings and
     fuses chains of kernels.
   - **L2 `gpu.pass`:** full WGSL compute or fragment modules with a multi-pass DAG declared in the
     manifest, in the style of ISF and BlinkScript.
   - **L3 `gpu.*`:** an imperative, handle-based WebGPU subset driven from Wasm, validated in the
     style of Dawn's wire protocol.
   - **L4 `viewport.*`:** overlays and custom render passes in app viewports.

   Every layer runs on the device that owns the data, using host-owned bindings and read-only views
   of app textures, so sharing needs no copy.
7. **Validation is cheap; compilation is not.** naga parses and validates a small shader in
   **0.08 ms** and emits MSL in **0.03 ms**. The first Metal pipeline compile of a new shader takes
   **≈64 ms**, and **0.15–0.25 ms** once the OS shader cache holds it (*measured*). So compile and
   warm shaders at install or enable time, off the UI thread. wgpu's `PipelineCache` only works on
   Vulkan.
8. **wgpu's safe `create_shader_module` already applies bounds checks and `force_loop_bounding`**
   (`wgpu-30.0.1/src/api/device.rs:139-142`). Loop bounding prevents undefined behaviour, but it
   does not bound running time. Fairness and hang safety have to come from the host:
   - tiled dispatches sized from timestamp-query estimates;
   - a per-frame GPU budget;
   - a degrade ladder;
   - a watchdog plus device-loss quarantine keyed by shader digest.
9. **GPU memory quotas must be counted by the host from resource descriptors.** wgpu has no
   allocator report on Metal or WebGPU. EffectCraft's `memcap.rs` already documents this gap.
10. **AI inference should become one host service, `craft-nn`, in craft-libs.** It would provide:
    - a wgpu backend built from LightCraft's WGSL convolutions, running on the app device, so
      tensors are buffers with no copy;
    - a candle backend for transformer-class models;
    - a content-addressed model store shared by all apps.

    Today there are five separate inference devices or backends and three downloaders.
11. **Budgets and SLOs are enforced per plugin class** by deadlines (epoch ticks for CPU, timestamp
    estimates for GPU), generation-token cancellation, and a ladder that degrades to proxy, then
    last-good, then bypass. Export never degrades; it only fails with a named error. Interactive
    slider preview targets **≤ 16 ms p95 at preview scale**. An audio block gets **≤ 50% of its
    period** as a hard deadline, falling back to the dry signal.

---

## 1. Current state per app

Source: a read-only survey of every repository under `plugin-beta`, with paths
relative to each repository. All repositories resolve `wgpu`, `wgpu-core`, `wgpu-hal` and `naga` to
**30.0.1**. No repository uses `wgpu::PipelineCache`.

| App | Device owner | Extra devices | Working format | Per-frame readback | WGSL files | ML |
|---|---|---|---|---|---|---|
| EffectCraft | eframe device shared into `Gpu::new` (`crates/ui-egui/src/app/gpu.rs:173-192`), built on a worker (≈235 pipelines) | headless `GpuContext::request` for CLI and web workers (`crates/gpu/src/context.rs:1421-1462`); SAM2 device (`crates/sam2/src/gpu.rs:144-171`) | `Rgba32Float` (`context.rs:113`); viewer `Rgba8Unorm` with mips | No; CPU steps round-trip (`walk.rs:1-6`) | 35 | SAM2 (candle + WGSL), MobileSAM, TFLite |
| PhotoCraft | eframe (`crates/ui-egui/src/app/gpu.rs:6`); requests `TIMESTAMP_QUERY(_INSIDE_ENCODERS)` (`canvas/gpu_canvas/setup.rs:55-118`) | headless (`crates/gpu/src/context.rs:110-130`) | `Rgba32Float` accumulator; `Rgba8`/`Rgba16Float` 8192² canvas tiles | No (export only) | 4 render, 0 compute | none |
| LightCraft | adopts the window device or creates its own (`crates/gpu/src/lib.rs:571`, `ctx.rs:470-500`); refuses software adapters | denoiser (`crates/gpu/src/nn.rs:85-122`), faces (`faces.rs:109-123`) | f32 storage buffers, `Rgba8Unorm` out | Yes when on its own device (`ctx.rs:862`) | 21 | pure-Rust ONNX subset + WGSL conv; SAM3 (candle Metal) |
| TextureCraft | eframe | baker (`crates/render/src/gpu_bake.rs:69-86`), headless | RGBA8, 16f HDR, 32f path trace | No | 11 | none |
| FilmCraft | eframe | export renderer (`crates/gpu/src/export_renderer.rs:118-134`) | 16f accumulator, 32f effects (`fx.rs:21`) | No | 5 | Whisper, Kokoro (candle CPU) |
| AniCraft | window or own headless (`crates/gpu/src/lib.rs:343-370`) | — | `Rgba8Unorm` storage texture | Headless path only | 4 | none |
| MaterialCraft, FigCraft, VectorCraft, DesignCraft, SoundCraft | eframe (UI only) | — | CPU raster (vello_cpu, tiny-skia) | — | 0 | none |
| craft-libs | no GPU crate; wgpu only in `egui_kittest` | — | — | — | 0 | `craft-ai` GGUF on candle CPU (no app uses it) |
| craft-extensions | none | — | f32 RGBA on CPU | — | 0 | none |

Facts that drive the design:

- **One wgpu version suite-wide.** A `craft-gpu` / `ext-gpu` crate pinned to `wgpu = "=30.0.1"`
  can be linked by every app without a version split. It must move in lockstep with the apps' wgpu
  upgrades, as egui-wgpu already does.
- **Devices are fragmented.** EffectCraft has three devices and LightCraft three or four. A wgpu
  `Texture` belongs to one `Device`. Cross-device use needs CPU copies or unsafe HAL interop. The
  plugin GPU host must therefore run on **whichever device holds the data**. In practice that is
  the eframe device for interactive work and the export or bake device for those jobs. It must
  never create a "plugin device" for interactive work.
- **Timestamp queries are almost unused.** Only PhotoCraft requests them (`crates/gpu/src/timing.rs:58`).
  The scheduler in §5.6 needs them wherever the adapter supports them. On the M5 they are
  supported and cost nothing measurable.
- **Device-loss handling exists everywhere** (EffectCraft `apps/effectcraft/src/main.rs:310+`,
  `crates/gpu/src/readback.rs:394-395`; PhotoCraft `crates/gpu/src/health.rs:107-118`; LightCraft
  `ctx.rs:535-545`; VectorCraft `main.rs:631`). Plugin quarantine can hook into these.
- **EffectCraft already has GPU back-pressure** (`crates/gpu/src/memcap.rs`), including the
  comment that "wgpu has no safe query for the adapter's memory size" and that Metal, WebGPU and
  GL report no allocator data. Plugin GPU quotas should plug into this, not duplicate it.
- **EffectCraft already time-slices waits.** Its `poll_wait` uses 20 ms `POLL_SLICE` waits under
  a poll lock (`crates/gpu/src/context.rs:1082-1125`). The plugin scheduler should use the same
  primitive.
- **The plugin runtime** is wasmi 2.0 with `portable-dispatch` (`crates/ext-host/Cargo.toml:18`)
  and fuel on, and a new `Engine` per plugin (`crates/ext-host/src/wasm.rs:83-93`). It has three
  full copies per `Filter::run` (`wasm.rs:611-680`), `MAX_SESSION_MEMORY` of 64 MiB, and
  `MAX_TRANSFORM_MEMORY` of 512 MiB (`wasm.rs:10-18`). There is no wasm32-specific code in
  `ext-host`, so on web builds wasmi would run as an interpreter inside the app's own Wasm.
- **The web split.** EffectCraft, PhotoCraft, FilmCraft, TextureCraft, MaterialCraft, FigCraft,
  VectorCraft, DesignCraft and SoundCraft use wgpu on WebGPU with a WebGL2 fallback. LightCraft and
  AniCraft use glow (WebGL) only, so they have no compute shaders on the web.

---

## 2. Design principles

1. **Two planes.** The *control plane* (ops, events, parameters, edits) stays as simple as it is
   today: JSON ops, later typed bindings or WIT. The *data plane* (pixels, texels, samples,
   vertices, tensors) never goes through serialization. It goes through **views** (host
   materialises a region in guest memory) or **handles** (data stays host- or GPU-resident and the
   guest names it).
2. **Data does not move unless the computation needs it somewhere else.** A GPU-resident image is
   processed by GPU plugin tiers. A CPU-resident image is processed in a view. Transfers between
   the two happen only at tier boundaries the host chose, and are charged to the plugin that caused
   them.
3. **The host owns scheduling, memory, residency and caching.** Plugins declare what they need:
   ROI, apron, formats, temporal frames, GPU bindings and cost hints. The host decides where and
   when it runs.
4. **Everything has a budget and a degrade path.** No plugin can stall the UI thread, the audio
   thread, or the GPU queue beyond its class deadline.

---

## 3. Data plane design

### 3.1 Options measured (1080p RGBA32F, 33.2 MB, Wasmtime 50.0.2 Cranelift, *measured*)

| Option | What happens | Cost | Verdict |
|---|---|---|---|
| Today, `Filter::run` on wasmi | serialize to LE bytes, `memory.write`, run, `memory.read`, validate, `copy_from_slice` | **52.0 ms** e2e (≈10 ms of it is copies) | replace |
| A. host `memcpy` in/out | `Memory::write` from host `Vec<f32>`, guest loop in place, `Memory::read` back | copy-in 0.56 + run 0.87 + copy-out 0.56 = **2.10 ms** | acceptable baseline |
| B. host produces directly into guest memory | host decode, convert or render step writes into `Memory::data_mut()[view]`; the consumer reads from the same slice | run only, **0.87 ms**, plus whatever producer work was needed anyway | **default** |
| B′. RGBA8 source → f32 view | convert into a `Vec` then copy, versus convert straight into guest memory | 1.02 ms → **0.40 ms** | the conversion *is* the copy |
| B″. RGBA16F source → f32 view | `half` scalar conversion straight into guest memory | 1.87 ms (NEON `fcvt` batch would be lower; not measured) | prefer f16 views when the plugin accepts them |
| C. fused output validation | clamp alpha, zero non-finite, premultiplied-zero fixup, in place | 1.17 ms | run only at commit or export, or fold into the GPU upload shader |
| D. mmap aliasing | shm-backed host frame mapped `MAP_FIXED` over part of the guest's linear memory | remap 3 µs, but first guest run **2.12 ms** (page faults on 2,026 × 16 KiB pages) against 0.84 ms warm; restore 75 µs | no gain over memcpy; unsafe outside `MemoryCreator`; **reject** for frames |
| E. host-resident handle, tile reads | guest calls `read_tile`/`write_tile` imports, host copies rows into a guest scratch tile | 64–128 px tiles: **3.4 ms** (1,020 / 270 calls); 256–1024 px: 3.8–4.3 ms | use for ROI and random access on big images, not for dense full-frame passes |
| F. host↔guest call cost | typed import with no args; typed export call | **1.8 ns** per import, **11.1 ns** per export call | host calls are free; JSON (`craft.call` 10.5 µs) is the cost |
| G. host-side bands across N instances | 1/2/4/8 instances on threads, each with copy-in, run, copy-out of its band | 2.11 / 1.62 / 1.53 / **1.37 ms**; view-only with 8 threads **0.31 ms** | parallel tiles pay off once copies are gone (memory-bandwidth bound) |

Reading the table:

- Once a JIT runs the loop, **copies and conversions dominate**. Two `memcpy`s cost 1.1 ms against
  0.87 ms of work.
- With 8 bands and no copies, the work drops to 0.31 ms. With copies it stays at 1.37 ms, because
  the copies saturate memory bandwidth.
- The design target is **one producer write into the view, run in place, one consumer read from
  the view**. Copies happen only where a format change or device transfer is needed anyway.

### 3.2 Three kinds of data access

| Kind | Used for | Mechanism | Guest sees |
|---|---|---|---|
| **View** | dense per-call data: a filter's ROI plus apron, an audio block, a mesh's position array, a tensor's input | at call time the host writes (decodes or converts) straight into a reusable **arena** in guest memory, then passes a `#[repr(C)]` descriptor table | `&[f32]` / `&mut [f32]` slices with stride and format |
| **Handle** | big or long-lived data the guest touches sparsely: other layers and frames, 4K/8K material maps, sample libraries, model weights, GPU textures | host-resident, refcounted, with residency (CPU, GPU or both); region ops copy only what is asked for | `u32` handle plus `image.read_region`, `image.write_region`, `image.info` |
| **Ring** | streams: audio events, parameter changes, meter and scope feeds, progress | SPSC lock-free rings at fixed offsets in guest memory, allocated at activation | `RingReader` / `RingWriter` in the SDK |

**Arena.** At instantiation the guest exports `craft_arena(bytes) -> ptr`, or the SDK reserves
one. The host keeps the arena across calls and grows it only when the ROI grows, so no allocation
happens per frame. Views are valid only for the duration of the call. The host never keeps guest
pointers, and the guest never sees host pointers.

**ABI sketch** (core-Wasm level; the SDK wraps it):

```rust
#[repr(C)] pub struct View {
    pub ptr: u32, pub len: u32,
    pub width: u32, pub height: u32, pub stride: u32,
    pub format: u32,
    pub origin: [i32; 2],
    pub flags: u32,
}

#[repr(C)] pub struct FilterCall {
    pub ctx_json: View,
    pub n_in: u32, pub inputs: u32,
    pub out: View,
    pub roi: [i32; 4], pub scale: f32, pub time: f64, pub quality: u32,
}

craft_filter2(call: *const FilterCall) -> i32;
```

- Formats are `RGBA32F`, `RGBA16F`, `RGBA8`, `GRAY32F`, `GRAY16`, `RGBA16`, ….
- `origin` is in full-image pixels.
- `flags` carries `READONLY` and `PREMULTIPLIED`.
- `inputs` points to `View[n_in]`. Input 0 is the layer with its apron; the rest are declared
  extras such as other layers or `frame.at` results.

The plugin declares the formats it accepts in the manifest (`accepts = ["rgba16f", "rgba32f"]`).
The host picks the cheapest format it can produce. An app that works in 16F hands over 16F
(16.6 MB) instead of converting to 32F (33.2 MB): half the bandwidth, and no conversion. For
**geometry** the same `View` carries SoA arrays (`positions: f32x3`, `normals`, `uvs`, `indices:
u32`). Index arrays written back are bounds-checked on commit. For **audio** it is planar `f32`
per channel at `max_block`.

**Output path.** The consumer reads the output view directly. A CPU compositor reads the slice. A
GPU consumer calls `queue.write_texture(&mem[out.ptr..])` straight from guest memory, which is
the only copy and is unavoidable for a CPU→GPU transfer. Validation of finite values and alpha
runs:

- **on commit and export** on the CPU (1.17 ms per 1080p, fused);
- **for free** in the GPU upload or first-consumer shader (`select(0.0, x, x == x && abs(x) < INF)`);
- **never** per preview frame on the CPU.

### 3.3 Control plane stays simple

- Ops stay `craft.call` with JSON (10.5 µs). They are fine for control. They are not for data:
  `MAX_CALL_BYTES` stays at 1 MiB and base64 pixels are removed.
- Hot control-plane values get **typed imports** (1.8 ns): `param_f32(id)`, `time()`,
  `cancelled() -> bool`, `progress(f32)`.
- For bulk structured data (node lists, layer trees), `craft.call_buf` writes a compact binary
  encoding into an arena view. The proposal is postcard or FlatBuffers-style; JSON is kept for
  agents and debugging.
- **Component model caveat.** Once WIT (`craft:plugin@2`) arrives, the canonical ABI *copies*
  every `list<u8>` argument into the callee's memory. The data plane must therefore stay
  **resources + core-memory views** (`resource image { read-region: func(...) }`), never
  `list<f32>` parameters.

### 3.4 Large data and memory caps

- Guest linear memory per instance is capped by contribution class: session 64 MiB, filter 512 MiB
  today. With views and handles, a 4K RGBA32F frame (133 MB) never needs to be resident in full.
  The host tiles the request (ROI plus apron) to fit the arena, for example 1024² tiles of 16 MB
  each.
- The 512 MiB cap stops being a limit on document size. That is item R5 in the capability
  program, made concrete.
- Handles are accounted to the plugin, in bytes by residency, against the per-app budget. They are
  released when the call or session ends, unless they were promoted with `cache.put`.

---

## 4. Execution engine design

### 4.1 Engine matrix

| Target | Engine | Why |
|---|---|---|
| macOS, Windows, Linux desktop (all contribution kinds) | **Wasmtime 50 + Cranelift**, AOT at install, artifact cache, epoch interruption, pooling allocator | 0.87–0.96 ms vs native ≈0.76–1.06 ms on the loop (*measured*); µs instantiation |
| Web builds | **Browser WebAssembly** (`WebAssembly.Module` / `Instance` via `js-sys`) in a Worker per plugin class | V8, SpiderMonkey and JSC JIT-compile; wasmi-inside-Wasm would be an interpreter run in Wasm, roughly 10–40× slower than the browser JIT (estimate from the 11× / 40× wasmi figures) |
| iOS / iPadOS (JIT forbidden to apps) | wasmi 2 with **tail-call dispatch** (or Wasmtime **Pulley**; not measured) | 9.4–11.3 ms loop, 4× better than today |
| CI reference and determinism tests | wasmi with fuel | deterministic instruction counting for golden "fuel" regression tests |

**Decision:** desktop uses **one engine for all plugin code**. A second interpreter for "small UI
sessions" is not worth it: the only advantage it had was compile latency, and AOT plus the
artifact cache remove that (§4.2). `Engine` stays a trait in `ext-host` with three backends
(`wasmtime`, `wasmi`, `web`) behind one `Instance` API, so contracts and the SDK do not change.

### 4.2 Compilation, caching, startup

*Measured* on `guest_big.wasm` (the capability bench's `guest-bright` filter with `craft-guest` +
serde_json, 122 KB LTO and 160 KB non-LTO):

| Step | 122 KB | 160 KB |
|---|---|---|
| Cranelift compile (parallel-compilation on, 10 cores) | 8.9 ms | 11.5 ms |
| Winch compile | 2.2 ms | 2.7 ms |
| Cranelift artifact size | 335 KB | 439 KB |
| `Module::deserialize` from bytes | 0.037 ms | 0.054 ms |
| `Module::deserialize_file` (mmap) | 0.033 ms | 0.039 ms |
| 100 modules via `deserialize_file` | 3.75 ms, **+25.7 MB RSS** | — |
| 100 modules in wasmi (for comparison) | 30.3 ms, +12.1 MB RSS | — |

The earlier capability benchmark saw 34–42 ms Cranelift compiles on the same module because its
Wasmtime build lacked the `parallel-compilation` feature. Enable it.

Design:

- **Compile at install and update** on a background thread, at low priority. Store the artifact at
  `<store>/artifacts/<sha256(module)>-<engine.precompile_compatibility_hash()>.cwasm`. If the hash
  does not match after a Wasmtime upgrade, recompile lazily and keep the old file until the new one
  succeeds.
- **Never accept `.cwasm` from a catalogue or package.** Artifacts are native code and
  `Module::deserialize` is `unsafe` for that reason. Only host-compiled files in a host-owned
  directory are loaded. Their sha256 is recorded in the store database and checked against a
  keyed hash before mmap.
- **Lazy instantiation.** At app start, read only the manifest index. A plugin's module is
  deserialized when one of its contributions is first used: a menu command, an effect in the
  document, a panel opened, or an event subscription that fires. Idle modules (no instance for
  10 minutes) are dropped, and re-mapping costs 0.04 ms. **Startup overhead with 100 plugins
  installed: ≤ 5 ms** (index read), regardless of plugin count.
- **First use after a cold install with no artifact yet:** wait for Cranelift if the compile
  finishes within 50 ms, which covers modules up to about 500 KB. Larger modules show a "preparing
  plugin" status. A Winch tier (2–3 ms compile, about 3.5× slower code) is optional; §11 D3
  recommends against it for now.

### 4.3 Instances, pooling, reuse

*Measured*, per instantiation of the 122 KB module (`InstancePre`, a new `Store` each time):

| Allocator | Cold first | Warm p50 | Warm p99 | p50 with 1 MiB touched |
|---|---|---|---|---|
| On demand | 0.035 ms | 11.8 µs | 17.2 µs | 55.9 µs |
| Pooling (1000 slots, 512 MiB max) | 0.023 ms | **6.2 µs** | 10.7 µs | 50.0 µs |

- Creating a pooling engine takes 2.8–3.1 ms, which reserves address space. Per-slot
  reservation × slots must fit the process's VA. 1,000 slots at 512 MiB worked on macOS. Use
  about 256 slots and size `max_memory_size` per contribution class.
- **Page faults dominate** once memory is touched: about 44 µs per MiB with 16 KiB pages. Use
  `linear_memory_keep_resident` and `max_unused_warm_slots` for the hot classes (filters, audio).
  Keep **instances alive across calls** rather than re-instantiating.
- **Instance pools** follow EffectCraft's per-render-thread pool (`crates/plugins/src/effects.rs`),
  moved into the platform filter service:
  - one instance per (plugin contribution, worker thread);
  - reused while the document and parameter schema stay the same;
  - reset through an exported `craft_reset` or re-instantiated (6 µs) on a document switch,
    revocation or error;
  - poisoned instances (trap, deadline) are always discarded.
- **Audio** pre-instantiates at `activate()` off the audio thread. The real-time thread never
  compiles, instantiates or grows memory. A `ResourceLimiter` refuses `memory.grow` while the
  store is flagged real-time.

### 4.4 Interruption and metering

| Mechanism | Overhead on the tight loop (*measured*, prior bench) | Use |
|---|---|---|
| none | 0.96 ms | never for untrusted code |
| epoch interruption | 1.29 ms (+34%) | **all Wasmtime execution**. One process-wide ticker thread at 1 ms (0.25 ms while audio is active). Deadlines in ticks per call class |
| fuel | 1.68 ms (+75%) | wasmi only (deterministic tests and iOS) |

- An expired deadline traps the call. The host treats the instance as poisoned, applies the class
  fallback (§8), and records a strike.
- `policy::Strikes` already gives three-strike quarantine. Keep it, but count strikes per
  contribution and class, so a slow export does not quarantine the plugin's UI.
- Host calls are billed by **bytes and wall time**, not the flat 100 k fuel per call (capability
  item R7). The 19 k-calls-per-event ceiling disappears.

### 4.5 Parallelism and SIMD

- **Host-side parallelism only.** The filter service splits the ROI into bands or tiles (apron
  included) and runs one pooled instance per worker. *Measured* with 8 workers: 0.31 ms without
  copies against 0.87 ms single-threaded. Tiles are also the unit for cancellation, progress and
  caching.
- **Wasm threads** (shared memory, `wasi-threads`) remain off. Shared mutable memory between
  untrusted threads breaks per-instance poisoning and determinism. The shared-everything-threads
  proposal is not finished. Revisit only if a plugin class appears that cannot be split by the
  host (for example a monolithic solver). A `job.map` fan-out across sandboxes covers most of
  those.
- **SIMD128** is on in every engine (wasmi needs the `simd` feature; it is refused today). SDK
  templates build with `+simd128`. Memory-bound loops gain nothing (*measured*). Compute-bound
  kernels such as convolutions, colour matrices and FFTs typically gain 2–4× (published V8 and
  Wasmtime figures).
- **Relaxed SIMD** is off by default. Its results are platform-dependent, which breaks render-cache
  and export determinism. A plugin can opt in with `nondeterministic = true` for preview-only
  contributions.
- Turn on **NaN canonicalization** (`cranelift_nan_canonicalization`) for determinism. Its cost is
  unmeasured; benchmark it on the brightness and blur guests before enabling it by default.

### 4.6 Memory footprint with 100 plugins

| Component | Per plugin | 100 plugins | Notes |
|---|---|---|---|
| Manifest and index | ≈2–10 KB | ≤ 1 MB | always resident |
| Deserialized module (mmap'd artifact) | ≈257 KB RSS (*measured*: 25.7 MB / 100) | 26 MB if all loaded | lazy plus idle eviction keeps 5–20 resident in practice |
| Idle instance | ≈6 KB RSS untouched (*measured*: +0.6 MB / 100; 1,088 KiB initial memory is CoW) | < 1 MB | grows with touched memory |
| Active filter instance at 1080p f32 | 33–66 MB arena | bounded by pool size × arena | arena shrinks after 30 s idle |
| Wasmtime + Cranelift in the binary | — | **+≈12 MB stripped** for the bench binary with Cranelift, Winch and wasmi (*measured* 12.5 MB) | measure the real delta in one app before rollout; dropping Winch saves part of it |

---

## 5. GPU API for plugins

### 5.1 How other hosts do it

| Host | What the plugin supplies | Who owns device and resources | Validation and safety | Scheduling | CPU fallback |
|---|---|---|---|---|---|
| Browsers (WebGPU) | full API: buffers, textures, WGSL pipelines | page owns a `GPUDevice`; Chrome proxies every call to the GPU process (Dawn wire) | every command validated before reaching the driver; Tint/naga translation with robustness (bounds clamping); zero-initialised resources; `device.lost`, error scopes; non-normative watchdog; shader compilers fuzzed ([Chromium WebGPU technical report](https://chromium.googlesource.com/chromium/src/+/main/docs/security/research/graphics/webgpu_technical_report.md), [W3C WebGPU security](https://w3.org/TR/2025/CRD-webgpu-20250320), [DarthShader](https://arxiv.org/pdf/2409.01824)) | page-level; OS TDR | none |
| OFX 1.4/1.5 | CUDA, OpenCL or Metal code in a native plugin, given the host's queue or stream | host owns the device and passes a command queue (`kOfxImageEffectPropMetalCommandQueue`) | none: native trust | host per render call | plugin's own CPU path |
| After Effects SDK | `PF_Cmd_SMART_RENDER_GPU` with the GPU suite's device and world | host | none: native | host | `SMART_RENDER` CPU |
| FxPlug 4 | Metal on tiles, out of process | host passes `MTLDevice` and tile textures over XPC | process isolation | host | none |
| Resolve DCTL | a per-pixel `transform(...)` function in C-like DCTL, with `DEFINE_UI_PARAMS` | host compiles to CUDA, OpenCL or Metal and owns everything | language subset | host per frame | none (GPU only) |
| Fusion Fuses | Lua tool plus optional DCTL kernels | host | Lua sandbox | host | Lua CPU |
| Nuke BlinkScript | `kernel : ImageComputationKernel<ePixelWise>` with `Image<eRead, eAccessRandom>` declarations and `process()` | host compiles for CPU SIMD and GPU | restricted C++ subset | host tiles | the same kernel on CPU |
| ISF | GLSL fragment plus JSON header (`INPUTS`, `PASSES` with `TARGET`, `PERSISTENT`) | host | GLSL compiler | host per frame | none |
| Shadertoy | `mainImage`, `iChannel0-3`, Buffers A–D | host | WebGL | per frame | none |
| TouchDesigner | GLSL TOP / GLSL Multi TOP (fragment or compute), Script TOP (numpy) | host | none | cook graph | Script TOP |
| Bevy | `Material` WGSL, render-graph nodes, compute in the render world | engine | wgpu validation | render graph | none |
| Godot 4 | `RenderingDevice` compute and `CompositorEffect` hooks into the render pipeline | engine (local devices for compute) | Godot's validation | engine | none |
| Blender | `gpu` module: shaders, batches, offscreen, `draw_handler_add` | Blender | Python trust | draw callbacks | none |

The lessons:

1. **Hosts that accept untrusted code** (browsers, Resolve, BlinkScript, ISF) either give the
   author a **function or kernel with declared I/O**, or put a **full validating proxy** between
   the author and the driver (browsers).
2. **Hosts that give device access** (OFX, AE, FxPlug, Blender) trust the plugin as native code.
3. We need both a "kernel with declared I/O" tier for most authors and a "validated WebGPU subset"
   tier for advanced ones, with the browser model as the security baseline.
4. **The host owns scheduling everywhere.** Only browsers ever expose device loss to the author.

### 5.2 Layers

| Layer | Author writes | Host does | Covers | Web (WebGPU) | WebGL2/headless fallback |
|---|---|---|---|---|---|
| **L1 `gpu.kernel`** | a WGSL *function body*: `fn cook(p: Pixel) -> vec4f` with generated helpers `load_<input>(xy)`, `sample_<input>(uv)` and `params.<name>` | generates the entry point, bindings, params uniform and dispatch; **fuses chains** into one shader (Material Maker style); picks compute or fragment | per-pixel effects, colour and LUT work, generators, material and texture nodes (most of the catalogue) | yes | `kernel.program` CPU twin or the package's Wasm fallback |
| **L2 `gpu.pass`** | full WGSL modules plus a manifest DAG of passes: declared inputs, outputs, persistent and scratch targets, sizes relative to the ROI, iteration counts | creates resources within quota and binds by name; validates reflection against the manifest; schedules and tiles | multi-pass blurs and glows, pyramids, temporal feedback, reaction-diffusion, simulation steps, flow | yes | Wasm fallback required for headless targets |
| **L3 `gpu.*`** (from Wasm) | imperative calls: create buffer or texture, create pipeline from WGSL, bind groups, encode dispatches and copies, `read_async` | records into a **command list in host memory**, validates each command (handles, usages, ranges, quota), then replays into the app encoder; readbacks arrive as events or handles | data-dependent control (iterative solvers, reductions, stream compaction), custom particle systems, ML-ish ops | yes | Wasm fallback |
| **L4 `viewport.*`** | (a) declarative draw lists; (b) WGSL vertex and fragment with handle-backed vertex buffers; (c) a panel canvas texture | (a) host renders; (b) host runs the plugin pipeline inside an overlay pass with host camera uniforms, read-only depth and its own colour target, composited by the host; (c) a plugin-owned offscreen texture registered with egui | gizmos, guides, 3D overlays, custom scopes, node previews, custom canvases | (a)–(c) yes | (a) CPU path; (b) disabled; (c) CPU canvas |

Ship the layers in order of reach and safety: **L1 → L2 → L4a → L3 → L4b → L4c**.

### 5.3 Manifest and API sketch

```toml
[[contributes.effect]]
id = "glow"
contract = "raster.filter@2"
exec = ["gpu.pass", "wasm"]
accepts = ["rgba16f", "rgba32f"]
apron = "params.radius * 3"

[contributes.effect.gpu]
entry = "shaders/glow.wgsl"
quota_mb = 128
cost_hint = "per_pixel * params.radius"

[[contributes.effect.gpu.pass]]
name = "threshold"
entry = "threshold"
in = ["src"]
out = { bright = { size = "roi/2", format = "rgba16f" } }

[[contributes.effect.gpu.pass]]
name = "blur_h"
entry = "blur_h"
in = ["bright"]
out = { tmp = { size = "bright" } }
repeat = "params.passes"

[[contributes.effect.gpu.pass]]
name = "blur_v"
entry = "blur_v"
in = ["tmp"]
out = { bright = {} }
repeat = "params.passes"

[[contributes.effect.gpu.pass]]
name = "composite"
entry = "composite"
in = ["src", "bright"]
out = "dst"
```

The L1 kernel the author writes, and the code the host generates around it:

```wgsl
fn cook(p: Pixel) -> vec4f {
    let c = load_src(p.xy);
    let l = dot(c.rgb, vec3f(0.2126, 0.7152, 0.0722));
    return vec4f(mix(vec3f(l), c.rgb, params.saturation), c.a);
}
```

```wgsl
struct Params { saturation: f32, _pad: vec3f }
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var t_src: texture_2d<f32>;
@group(0) @binding(2) var t_dst: texture_storage_2d<rgba16float, write>;
struct Pixel { xy: vec2i, uv: vec2f, roi: vec4i }
fn load_src(xy: vec2i) -> vec4f { return textureLoad(t_src, clamp(xy, vec2i(0), vec2i(textureDimensions(t_src)) - 1), 0); }
@compute @workgroup_size(16, 16) fn main(@builtin(global_invocation_id) id: vec3u) { /* bounds, Pixel, textureStore(t_dst, …, cook(p)) */ }
```

L3, the imperative surface, as seen from the guest SDK (Rust; other languages get generated
bindings):

```rust
let tex = gpu::texture(TextureDesc { size: roi.size(), format: Rgba16F, usage: STORAGE | SAMPLED })?;
let pipe = gpu::compute_pipeline(include_str!("reduce.wgsl"), "main")?;
let bg = gpu::bind(&pipe, 0, &[Bind::Texture(input), Bind::StorageTexture(tex), Bind::Uniform(&params)])?;
let mut enc = gpu::encoder();
enc.dispatch(&pipe, &bg, roi.size().div_ceil(16), 1);
enc.copy(tex, output);
let sums = enc.read_buffer_async(sum_buf, 0..64);
enc.submit()?;
```

Resource rules:

- `input` and `output` are host handles to the frame's input and output.
- `submit` hands the encoder to the host, which validates it and splices it into the frame graph.
- Results such as `sums` arrive at a later event or `job` step. There is no blocking readback
  inside a render call, because a wait would stall the frame.
- Handles (`Texture`, `Buffer`, `Pipeline`, `BindGroup`) are `u32`s in a per-plugin table.

### 5.4 Validation pipeline for plugin WGSL

All steps run at **install** (refuse the package on failure) and again at **load** (in case naga
or wgpu changed):

1. **Parse and validate with naga** (`ValidationFlags::all()`, capabilities limited to the
   plugin's declared feature set). *Measured*: 0.08 ms for a 766-byte shader.
2. **IR inspection, refusing:**
   - bindings not declared in the manifest, and the reserved `@group(0)` slots the host injects;
   - `var<workgroup>` above the declared or host limit (L1 allows none);
   - atomics or storage writes in L1;
   - subgroup, ray-query, f16 and push-constant use unless declared and the adapter supports them;
   - entry points whose reflection disagrees with the manifest pass declarations.
3. **Runtime checks.** Always use `Device::create_shader_module`, which applies
   `ShaderRuntimeChecks::checked()`: bounds checks plus `force_loop_bounding`
   (`wgpu-30.0.1/src/api/device.rs:139-142`). Plugin code never reaches
   `create_shader_module_trusted` or `create_shader_module_passthrough`. A clippy `disallowed_methods`
   rule in `ext-gpu` enforces this.
4. **Static cost estimate**: loop nesting × declared bounds × taps from the IR, combined with the
   manifest `cost_hint`. It seeds the scheduler before any timestamps exist.
5. **Pipeline warm-up** off the UI thread at enable time. *Measured* on Metal: a first-time unique
   pipeline compile takes **≈64 ms** (up to 99 ms for the very first in the process). Once the OS
   shader cache has it, it takes **0.15–0.25 ms**. wgpu's `PipelineCache` helps only on Vulkan
   (`wgpu-30.0.1/src/api/pipeline_cache.rs`). Use it there and persist it in the store.
6. **Digest kill list.** The sha256 of the WGSL plus entry is checked against a host or catalogue
   deny list. A local quarantine adds digests that hung or lost the device (§5.6).

Residual risk is the same one every WebGPU page carries: driver compiler bugs reachable through
valid WGSL. Mitigations:

- naga is fuzzed upstream;
- the catalogue can pre-validate;
- the most likely outcome of such a bug is device loss, which §9 recovers from.

### 5.5 Sharing app textures without copies

- **Same device, read-only bindings.** The host passes app textures (the current frame, a layer,
  a depth or ID channel, a material map) as `texture_2d<f32>` bindings. Plugins never get
  `STORAGE_BINDING` on app-owned textures. Outputs come from the host's texture pool (EffectCraft
  already pools textures) and are swapped into the chain. This also avoids wgpu's rule against
  using one texture as both sampled input and storage output in a single dispatch.
- **Usage flags are an app-side prerequisite.** Working textures must be created with
  `TEXTURE_BINDING`. Pool textures need `STORAGE_BINDING | TEXTURE_BINDING | COPY_SRC`. Each
  adapter lists which of its textures it can expose (`CraftApp::gpu_view(target, region)`, the
  GPU twin of `pixels()`).
- **Formats.** Plugins declare the formats they accept. If none matches, the host inserts one
  conversion pass, and the cost is charged to the plugin.
  - `Rgba32Float` is not filterable without `FLOAT32_FILTERABLE`. That feature is optional in
    wgpu and WebGPU, and no app requests it. L1 `sample_*` helpers therefore sample a 16F view or
    do manual bilinear `textureLoad`. Plugins must not assume f32 filtering.
  - `rgba16float` storage writes are core WebGPU, so 16F is the preferred plugin interchange
    format: half the bandwidth (*measured*: 0.18 ms against 0.39 ms per brightness pass), and
    filterable.
- **Cross-device data** (an export device, a bake device, SAM2 on its own device) is copied by
  the host through CPU staging. Prefer moving the work to the data's device. Unsafe HAL interop,
  such as wrapping the same `MTLTexture` on two devices, is a later optimisation for EffectCraft's
  SAM2 path only, not a plugin API.
- **CPU-tier plugins inside GPU chains.** The host batches adjacent CPU-tier effects so one
  readback and one upload cover the run: 5.2 ms at 32F or 2.4 ms at 16F per 1080p round trip
  (*measured*). The host also shows the cost in the plugin console, so authors see why the GPU tier
  matters.

### 5.6 Scheduling and fairness with the app's own rendering

GPUs on wgpu backends offer no fine-grained preemption controllable from the API. Metal and
Vulkan can switch between command buffers, and OS watchdogs (Windows TDR defaults to 2 s; macOS
and Linux drivers have similar resets) kill the context of anything longer. The host therefore
bounds plugin work **before submission**:

1. **Frame-graph integration.** Interactive plugin passes are nodes in the app's per-frame
   encoder, so they need no extra submit. *Measured*: a trivial 1080p pass costs 0.39 ms of GPU
   time but 1.3 ms of wall time when submitted alone. Batching 20 passes amortises submit and wait
   to about 0.42 ms per pass.
2. **Cost model.** Every plugin pass writes timestamp queries where `TIMESTAMP_QUERY` (and, for
   in-encoder queries, `TIMESTAMP_QUERY_INSIDE_ENCODERS`) is available. The host keeps an EMA per
   (digest, pass, pixel count) and falls back to the static estimate when queries are unavailable
   (some WebGPU contexts). **Every app should request these features when the adapter offers
   them**, as PhotoCraft already does.
3. **Budget per frame class** (§8): interactive preview gives plugins about 50% of the frame's GPU
   budget. If the estimate exceeds it, the host steps through:
   - preview scale ½;
   - then ¼;
   - then last-good, with a "rendering" badge;
   - then a deferred background render that is tiled and resumes across frames.
4. **Tiling for long work** (export, bake, a large ROI): split dispatches into tiles of
   **≤ 4 ms estimated GPU time**, one submit per tile group, and use EffectCraft's time-sliced
   `poll_wait` between groups so app frames interleave. A tile that takes more than 4× its estimate
   halves the tile size for the next one.
5. **Watchdog.** If a submission containing plugin work is still pending after 1 s interactive or
   10 s export, the host:
   - marks the digest suspect;
   - does not submit more of its work;
   - lets the work finish if it can.

   If the device is lost, the host recovers the device (apps already handle this), adds the digest
   to the local kill list, re-renders without the plugin, and shows a badge naming it.
6. **Fairness among plugins**: weighted round-robin per frame over plugin GPU time. Background job
   GPU work gets at most 25% of frame time while the app is interactive and 100% when idle. Audio
   plugins never use the GPU on the audio thread.

### 5.7 GPU memory quotas

- **Accounting by descriptor**, in the host, for every plugin-created resource: bytes = size ×
  bytes per pixel × mips × layers, rounded to 64 KiB. The host also charges pooled outputs and
  conversion temporaries to the plugin that caused them. wgpu's `generate_allocator_report` returns
  nothing on Metal or WebGPU (`memcap.rs` header), so counting is the only portable source.
- **Defaults** (manifest may lower, consent dialog may raise):

  | Contribution | Default quota | Maximum |
  |---|---|---|
  | interactive effect or node | 256 MiB | 1 GiB |
  | job or export | 1 GiB | 4 GiB |
  | viewport overlay or canvas | 64 MiB | 256 MiB |

  The quotas also fit within the app's memcap and a suite budget (for example 50% of
  `max_buffer_size`-derived device memory on unified-memory Macs).
- **Errors.** Plugin resource creation and encoding run inside `push_error_scope(OutOfMemory)`
  and `push_error_scope(Validation)`. A failure fails the plugin call with a named error and
  frees that plugin's transient resources. The app is never affected.
- **Eviction order under pressure:**
  1. plugin result caches;
  2. idle plugin persistent targets (recomputed on demand, so the plugin must tolerate a reset);
  3. model weights on the GPU;
  4. as a last resort, the app's own caches, through its existing policy.
- **Limits** besides bytes:
  - max texture dimension 16384 (EffectCraft already caps it), max 64 live resources per instance;
  - max dispatch of 65535 workgroups per dimension per pass (the WebGPU default);
  - bind groups within WebGPU defaults, so the same plugin runs on web.

### 5.8 CPU fallback (headless, web without WebGPU, CI)

| Situation | Behaviour |
|---|---|
| Desktop with a GPU | GPU tiers run normally |
| Desktop headless CLI (`craft-ext run`) | headless device on the best adapter (EffectCraft and PhotoCraft already have one); same code path |
| CI without a GPU | wgpu on a **software adapter**: lavapipe (Vulkan) or llvmpipe on Linux, WARP on Windows. Plugins run correctly, just slowly. LightCraft refuses software adapters for the app; the plugin test runner must allow them (`allow_software = true`). macOS CI runners have Metal |
| Web with WebGPU | GPU tiers run; no timestamp queries unless the feature is granted, so the static cost model is used |
| Web with WebGL2 only (LightCraft and AniCraft on the web, or fallback browsers) | no compute. L1 runs through its `kernel.program` CPU twin when the kernel was written in that IR; otherwise the package's declared `wasm` fallback runs. Without either, the contribution is shown as "needs WebGPU", never as a silent pass-through |

**Fallback policy:**

- Packages that target `web` or `headless` must declare a CPU path for L2 and L3 contributions.
  `exec = ["gpu.pass", "wasm"]` lists them in order of preference.
- `craft-ext test` runs both paths and compares them within a tolerance of ΔE ≤ 1, or 1e-3 for
  linear values. It uses golden images per backend, because GPU results differ in the last few
  bits across vendors.
- A naga-IR CPU interpreter is not recommended. No maintained crate exists, and it would be a
  second compiler to secure.

---

## 6. AI inference

### 6.1 Today

| App | Model | Runtime | Device | Weights |
|---|---|---|---|---|
| EffectCraft | SAM 2.1 Hiera | candle CPU + custom WGSL kernels (`crates/sam2/src/gpu/kernels*.wgsl`) | own wgpu device (`sam2/src/gpu.rs:144-171`) | safetensors, downloaded with `curl`, SHA-256 verified, `<config>/models` (`engine/src/analysis/models.rs`) |
| EffectCraft | MobileSAM, MediaPipe (TFLite interpreter), PyTorch checkpoint reader | pure Rust, rayon | CPU | same store |
| LightCraft | denoiser U-Net | pure-Rust ONNX subset + WGSL `nn_conv`/`nn_pool` | own device (`gpu/src/nn.rs:85-122`) | packaged |
| LightCraft | YuNet, SFace, AuraFace | CPU ONNX interpreter + GPU conv | own device (`faces.rs`) | packaged |
| LightCraft | SAM 3 | candle (Metal on macOS, CPU elsewhere) | candle Metal device | mirrors, on request |
| FilmCraft | Whisper, Kokoro TTS | candle CPU | CPU | pinned catalogue, SHA-256 |
| craft-libs | GGUF LLM (`craft-ai`, `local` feature) | candle CPU | CPU | unused |

That is three runtime families (candle, a custom ONNX subset, a TFLite interpreter), at least
four private GPU devices, and three download and verification paths.

### 6.2 Design: `craft-nn` host service

- **Crate:** `craft-libs/crates/nn`, shared by the apps and by `ext-host`'s `nn.*` ops. Its
  backends:
  - **wgpu**: generalise LightCraft's ONNX-subset reader and WGSL conv and pool kernels, and add
    matmul and attention kernels. It runs on **the app's device**, so tensors are `wgpu::Buffer`s
    and image↔tensor conversions are GPU kernels with no CPU copy. This is the default for
    convolutional vision models: denoise, upscale, segmentation decoders, depth, matting.
  - **candle** (CPU, Metal, later CUDA): transformers, Whisper-class audio models, LLMs, and SAM
    image encoders until the wgpu backend covers them. candle's Metal device is not the wgpu
    device, so the boundary costs one copy, staged through unified memory on Apple silicon.
  - **ort (ONNX Runtime)** with CoreML, DirectML or CUDA providers: **deferred**. It is the widest
    model coverage, but it is a large native C++ dependency per platform. Add it only as an
    optional "runtime pack" if third-party model demand outgrows the two pure-Rust backends.
- **Plugin API**, shaped like wasi-nn:
  - `nn.load {model}` returns a model handle. `model` is a sha256-pinned data contribution or a
    host-blessed id (`sam2.1-hiera-s`, `depth-anything-v2-s`).
  - `nn.run {model, inputs: {name: image|tensor handle}, outputs}` returns tensor or image handles
    as a job, with progress and cancel.
  - Pre- and post-processing ops (`nn.preprocess {resize, normalize, layout}`) run as host GPU
    kernels.
  - Output handles bind directly as L1 and L2 inputs, for example a matte straight into a
    compositing kernel, so nothing reaches the CPU.
  - Permission `compute.nn`. The consent dialog shows model size and source.
- **Tensor handles** live in the same handle table as images (§3.2), with dtype (f32, f16, i8),
  shape, layout and residency. A Wasm guest can `tensor.read_region` small outputs such as
  embeddings or boxes. Big ones stay host- or GPU-resident.
- **Model cache.** There is one **suite-wide, content-addressed store** at
  `~/Library/Application Support/Craft/models/<sha256>` (platform equivalents elsewhere), shared
  by all apps and plugins:
  - It replaces the per-app `<config>/models` stores.
  - Downloads go through the platform `net` service with consent, resume and SHA-256 checks.
  - Weights are safetensors loaded **by mmap**, so they cost no copy and the OS can page them out.
  - Loaded models are shared across plugins in a process, refcounted, with LRU eviction under a
    RAM and GPU budget. Model pipelines and kernels are warmed at first load, and on Vulkan cached
    in the `PipelineCache`.
- **Scheduling.** Inference is a *job* class by default: it gets the GPU tile scheduler (§5.6),
  with tiles at layer or batch granularity and ≤ 4 ms each while the app is interactive.
  Interactive inference, such as point-prompt SAM decoding at about 10–30 ms per click, uses the
  "interactive tool" budget, with the image encoder cached per (image hash, model).
- **Real-time audio ML** (Clarity-class denoise) gets an `ml.stream` service with fixed-shape
  small models on the CPU, pre-allocated, inside the audio class's deadline. Never the GPU: a
  trivial GPU submit and wait costs 0.23–1.3 ms here (*measured*), with an unbounded tail under
  load.

---

## 7. Benchmarks

### 7.1 Method

- Sources are in the session scratchpad,
  `bench/data-gpu/`:
  - `guest-tiles/`: a `no_std` Wasm guest with `alloc`, `bright` in place, `bright_tiles` (calls
    host `read_tile`/`write_tile`), `calls(n)` and `noop`; 1.4 KB.
  - `guest-big/`: a copy of the prior `guest-bright` filter (`craft-guest` + serde_json), built
    with LTO (122 KB) and without (160 KB, `target-nolto`).
  - `wt/`: the host. Wasmtime `=50.0.2` with `cranelift`, `winch`, `runtime`, `std`,
    `pooling-allocator` and `parallel-compilation`, plus wasmi 2 for comparison; libc
    `proc_pidinfo` for RSS.
  - `gpu/`: wgpu `=30.0.1` (metal, wgsl) and naga `=30.0.1` (wgsl-in, msl-out). Compute pass:
    `textureLoad` → `textureStore`, 16×16 workgroups, ping-pong between two textures, timestamp
    queries per pass.
- Raw output is in `perf-bench/wt-results.txt` and `perf-bench/gpu-results.txt`.
- Image: 1920×1080 RGBA. The f32 frame is 33.2 MB, f16 16.6 MB, RGBA8 8.3 MB.
- Timing: median of 7–21 runs after one warm-up. Instantiation is the median and p99 of 2,000
  runs. GPU "wall" includes submit and `poll(wait_indefinitely)`. GPU "ts" is the sum of pass
  begin and end timestamps.
- Built with `nice` and `CARGO_BUILD_JOBS=4`, own target directories, `--offline`. No app repo
  was built.

### 7.2 (i) Copy vs view (Wasmtime Cranelift, default config, no metering)

| Measure | ms |
|---|---|
| native in-place brightness | 0.65–1.34 (varies run to run; prior bench 1.06) |
| native 33 MB `memcpy` | 0.58 |
| A: copy in / run / copy out / e2e | 0.56 / 0.87 / 0.56 / **2.10** |
| B: view (run only) | **0.87** |
| RGBA8 → f32: to `Vec` then copy / straight into guest | 1.02 / **0.40** |
| RGBA16F → f32 straight into guest (scalar `half`) | 1.87 |
| fused validate in place | 1.17 |
| D: mmap alias remap / first run / second run / restore | 0.003 / 2.12 / 0.84 / 0.075 |
| E: handle tiles 64 / 128 / 256 / 512 / 1024 px | 3.46 / 3.43 / 4.18 / 4.27 / 3.79 |
| F: host import call / export call | 1.8 ns / 11.1 ns |
| G: bands with copies, 1 / 2 / 4 / 8 threads | 2.11 / 1.62 / 1.53 / 1.37 |
| G: 8 threads, views only | **0.31** |

For context, the prior *measured* wasmi figures on the same loop were:

- platform today, `Filter::run` end to end: 52.0 ms;
- tail-call dispatch with fuel: 11.3 ms;
- Wasmtime with epoch interruption: 1.29 ms.

### 7.3 (ii) Instantiation and AOT

The data is in the tables in §4.2 and §4.3. In brief:

- Cranelift compiles in 8.9 / 11.5 ms (122 / 160 KB) and Winch in 2.2 / 2.7 ms.
- Deserialization takes **0.033–0.054 ms**.
- Pooled instantiation is **6.2 µs** p50 and 10.7 µs p99; on demand it is 11.8 / 17.2 µs.
- Both rise to about 50–56 µs when 1 MiB of memory is touched.
- A pooling engine takes 2.8–3.1 ms to create.
- 100 modules: 3.75 ms and 25.7 MB RSS on Wasmtime (mmap), against 30.3 ms and 12.1 MB on wasmi.
- 100 idle instances add 0.6 MB RSS.
- The stripped bench binary (Cranelift, Winch, wasmi) is 12.5 MB.

### 7.4 (iii) wgpu compute on Metal (Apple M5)

| Format (1080p) | Upload (`write_texture` + wait) | 1 pass: wall / GPU ts | 20 passes: wall / GPU ts | Readback (copy + map + memcpy) | Round trip: upload + pass + readback |
|---|---|---|---|---|---|
| rgba32float (33.2 MB) | 4.73 ms | 1.33 / **0.39** ms | 8.43 / 8.15 ms | 1.33 ms | **5.21 ms** |
| rgba16float (16.6 MB) | 1.79 ms | 0.36 / **0.18** ms | 4.11 / 3.85 ms | 0.69 ms | **2.42 ms** |
| rgba8unorm (8.3 MB) | 0.95 ms | 0.24 / **0.034** ms | 0.95 / 0.69 ms | 0.35 ms | **1.19 ms** |
| rgba16f 7×7 box (49 taps) | 1.84 ms | 1.65 / 1.40 ms | 28.4 / 28.0 ms | 0.57 ms | 3.62 ms |

| Other GPU measures | Value |
|---|---|
| Device creation (high-performance adapter) | 18–79 ms (first launch higher) |
| naga parse + validate (766 B shader) | 0.080 ms |
| naga MSL output | 0.027 ms |
| Pipeline create, unique new shader (Metal compile) | first 64–99 ms, then ≈64 ms each |
| Pipeline create, shader in the OS Metal cache | 0.13–0.25 ms |

Interpretation:

- **Keeping data on the GPU is worth 13× for a trivial pass.** Resident 32F costs 0.39 ms of GPU
  time against a 5.21 ms round trip. A CPU plugin in a GPU chain pays the round trip (plus its own
  compute) on every frame. For 16F the figure is 0.18 against 2.42 ms.
- **The upload is the most expensive leg** (4.7 ms for 32F), so CPU→GPU traffic should be 16F
  wherever the plugin accepts it.
- A 49-tap kernel costs 1.4 ms of GPU time at 1080p. The same work on CPU-side Wasm would be
  about 49× the 0.87 ms brightness loop, unvectorised (estimate). This is the class where the GPU
  tier is decisive.

### 7.5 Not run, cited instead

- **Wasm SIMD on compute-bound kernels:** 2–4× in published V8 and Wasmtime material. Benchmark it
  on the blur guest when the SIMD feature lands.
- **WebCLAP and Wasm audio:** about 1.5× native on a JIT (`audio.md`).
- **Browser Wasm vs Wasmtime:** typically within 1.2–1.5× of native on SPEC-like code (`audio.md`
  cites 1.45–1.55×).

---

## 8. Budgets, SLOs and enforcement

### 8.1 Per-class budgets

Budgets are per plugin invocation unless noted. "Hard" means the host enforces it; "target" is the
SLO tracked in CI benchmarks and telemetry.

| Class | Target (p95) | Hard deadline | On overrun | Mechanism |
|---|---|---|---|---|
| **Interactive slider preview** (live param drag) | ≤ 16 ms per update at preview scale; first pixels ≤ 50 ms | 250 ms per update | proxy ½ → ¼ → last good + spinner; stale generations cancelled | generation token per drag; epoch deadline per tile; GPU EMA; result cache by (digest, params hash, ROI, scale) |
| **Viewport playback** | ≤ 25% of the frame budget per plugin (≈4 ms at 60 fps, ≈10 ms at 24 fps); total plugin share ≤ 50% | 2× the frame budget | drop to proxy; then bypass *in preview only*, with a badge and the plugin named | same, plus a per-frame plugin GPU budget (§5.6) |
| **Export / final render** | none (throughput) | progress heartbeat every 10 s; 10 s per GPU submission; job wall limit from the manifest (default 30 min per frame) | never degrade; fail with a named error, frame and plugin | tiling ≤ 4 ms GPU per tile group; epoch with long deadlines; deterministic engine settings |
| **Audio block** (`audio.effect@1`) | p99.9 ≤ 2× native and ≤ 25% of the block period | 50% of the block period (128 frames at 48 kHz = 2.67 ms, so 1.33 ms) | output dry for that block; 3 overruns in 10 s → bypass + badge; never silence | epoch ticker at 0.25 ms while audio is active; pre-instantiated, no grow, no host calls that allocate |
| **Panel UI frame** (plugin UI tree, events) | event handler ≤ 4 ms; tree diff ≤ 1 ms | 100 ms → "busy" overlay; 5 s → instance killed and restarted | UI thread never waits; the last tree is kept | events on the plugin worker; `pump()` with a per-frame budget (platform.md Txn design) |
| **Edit transaction / command** | ≤ 100 ms | 5 s hold timeout (existing) | roll back with one message | existing Txn timeout |
| **On-canvas tool / input filter** | ≤ 2 ms per pen sample batch | 8 ms | pass input through unfiltered | epoch |
| **Background job** (analysis, AI, batch) | throughput | manifest limit; cancellable | cancel and report | job service; GPU share ≤ 25% while interactive |
| **Plugin load** | activation ≤ 1 ms (deserialize + instantiate); first compile ≤ 50 ms per 500 KB, off the UI thread | — | "preparing" status | AOT at install |
| **App startup with 100 plugins** | ≤ 5 ms added | — | — | index only; lazy modules |

### 8.2 How the host enforces them

1. **Deadlines.**
   - CPU: epoch deadlines set per call from the class table. Ticks come from one ticker thread.
     When the deadline trips, the store traps and the instance is discarded.
   - GPU: estimates come before submission (tiling and the degrade ladder). Elapsed submission
     time is checked by the watchdog after submission.
2. **Cancellation.**
   - Every render request carries a generation token. A newer token for the same (instance, ROI)
     cancels the older one. Typed import `cancelled()` (1.8 ns) lets cooperative plugins stop
     early. The epoch deadline stops the rest.
   - GPU work already submitted cannot be cancelled. Its result is discarded, and the tile
     scheduler stops issuing tiles for it.
3. **Degrade ladder.** It runs from full quality, to proxy ½, to proxy ¼, to last good, to bypass
   with a badge. It applies to preview and playback only. Export and commit always run full
   quality or fail.
4. **Strikes and quarantine.** Strikes are counted per contribution and class:
   - 3 hard-deadline trips in 60 s disable the contribution for the session;
   - a device loss attributed to a digest adds it to the local kill list until the plugin
     updates;
   - all of this shows in the manager with the measured numbers.
5. **Telemetry for authors.** The plugin console (capability item D4) shows per-call CPU ms, GPU
   ms (timestamps), bytes moved, cache hit rate and degrade events. `craft-ext test --bench` runs
   the same budgets offline. CI fails on a regression of more than 20% (capability item R1).

---

## 9. Edge cases

| Case | Handling |
|---|---|
| **Device loss** (driver reset, TDR, eGPU unplug, plugin-induced hang) | The app's existing lost callback rebuilds the device. Plugin GPU state (pipelines, persistent targets, model weights) is recreated lazily from the manifest and cached data. If the loss followed a plugin submission inside the watchdog window, that digest is quarantined. Persistent temporal targets are reset, and plugins must accept `reset` (contract rule). Interactive renders retry once without the suspect plugin. |
| **Multiple GPUs** (discrete + integrated, Windows hybrids, Linux multi-GPU) | Plugin work runs on the device that holds its inputs; the host never picks a different adapter for a plugin. Jobs may target the export or bake device when the app routes them there. Power preference stays app-controlled (`WGPU_POWER_PREF`, `LIGHTCRAFT_GPU_ADAPTER`). GPU estimates are kept per adapter name, because a cost model learned on a discrete card would mislead on an iGPU. |
| **Cross-device data** (SAM2 device, LightCraft nn device, export device) | Host-staged copies, charged to the plugin. Long term, collapse private devices onto the app device through `craft-gpu` so model outputs can be shared without copies. |
| **Low memory (RAM)** | Arenas shrink after 30 s idle. Idle modules are unmapped (0.04 ms to remap). Instance pools are capped by the memory budget. Handles go through LRU eviction. Model weights are mmap'd, so the OS can drop them. On a macOS memory-pressure warning: clear result caches first, then idle instances. |
| **Low memory (GPU)** | Quotas (§5.7); OOM error scopes fail the plugin call only; the eviction order is in §5.7. EffectCraft's `memcap` gate counts plugin allocations as app allocations. |
| **Big images** beyond `max_texture_dimension_2d` (16384) or guest memory | The host tiles: both views and GPU passes are tile-based, with the apron declared in the manifest. Plugins that need global data (histograms, sums) use a reduction pass (L2/L3) or a host kernel. |
| **f16 shader support** (`SHADER_F16` optional) | f16 *storage* formats are core and always allowed. f16 shader arithmetic is a declared capability, with a f32 fallback compiled when it is absent. |
| **Rgba32Float filtering** | `FLOAT32_FILTERABLE` is optional. The L1 sampling helpers fall back to 16F views or manual bilinear interpolation. |
| **Web / WebGPU** | Browser WebAssembly in Workers. A runaway plugin is stopped with `worker.terminate()`, the only reliable kill in browsers, followed by a new Worker; JIT warmup is cached by the browser. Data crosses between the app's Wasm memory and the plugin's memory with `Uint8Array.set` (a memcpy) or through `SharedArrayBuffer` views when cross-origin isolated. GPU tiers use the app's WebGPU device on the app's thread: the plugin Worker submits an encoded L1–L3 command list, and the host replays it, because WebGPU objects belong to one thread. WebGPU limits are the defaults (256 MiB buffers, 128 MiB storage bindings) unless the adapter grants more. There are no timestamps without the feature. |
| **WebGL2-only web** (LightCraft-web, AniCraft-web, old browsers) | GPU tiers are unavailable; CPU fallbacks are used, and the manager shows "needs WebGPU" for GPU-only contributions. |
| **iOS / iPadOS** | No JIT: wasmi tail-call (about 10× native on loops), with GPU tiers via Metal. Steer authors toward L1 and L2 so heavy work never touches the interpreter. |
| **Headless CI** | lavapipe, SwiftShader or WARP software adapters, plus Metal on macOS runners. Golden images per backend with tolerances. Wasm runs on Wasmtime with deterministic settings (no relaxed SIMD, NaN canonicalization); wasmi with fuel is the instruction-count reference. Full app builds stay in Actions per the workspace CI rule; scoped `ext-gpu` tests run locally. |
| **Wasmtime or wgpu upgrade** | Artifacts are invalidated by the compatibility hash and recompiled in the background. WGSL is revalidated at load. The Vulkan `PipelineCache` is keyed by the driver UUID. |
| **Plugin update during a render** | The old module stays mapped until in-flight calls finish (refcount). New calls use the new artifact. The result cache key includes the module sha256. |
| **Determinism across machines** | Same-backend results are bit-stable for Wasm with canonical NaNs. GPU results are stable per adapter and driver only. Export caches and render farms key on (digest, adapter id), and tests compare with tolerances. |
| **Rosetta or x86_64 Macs** | Cranelift supports x86_64; artifacts are per architecture and are not copied between machines. |

---

## 10. Implementation sequence (performance and GPU slice only)

| Step | Work | Depends on | Size |
|---|---|---|---|
| P1 | Drop `portable-dispatch`, enable wasmi `simd`; CI benchmark gate | — | S |
| P2 | Views + arena + `craft_filter2` ABI; fused validation at commit; formats negotiated (16F first-class) | — | S–M |
| P3 | `Engine` trait; Wasmtime backend (Cranelift, `parallel-compilation`, pooling, epoch ticker); AOT artifact cache; lazy load and idle unmap | P2 | M |
| P4 | Filter service: per-thread instance pools, bands and tiles with apron, generation tokens, result cache, degrade ladder | P3 | M |
| P5 | `ext-gpu` crate (wgpu `=30.0.1`): naga validation pipeline, L1 kernels with fusion, host bindings, texture pool integration, timestamp cost model, quotas, error scopes, kill list | P2; app `gpu_view` adapters | L |
| P6 | Adapters expose GPU views: EffectCraft (`walk.rs` node for plugin passes), then PhotoCraft, FilmCraft, TextureCraft and LightCraft; request `TIMESTAMP_QUERY` features | P5 | M per app |
| P7 | L2 `gpu.pass` DAG, persistent targets, Wasm fallback runner, golden tests on lavapipe and Metal | P5 | M |
| P8 | `craft-nn` with wgpu (from LightCraft) and candle backends; shared model store; `nn.*` ops | P5 | L |
| P9 | L4a overlay draw lists; L3 imperative command lists; L4b custom overlay passes; L4c canvases | P5, P7 | L |
| P10 | Web engine backend (browser WebAssembly in Workers) and GPU command-list replay | P3, P5 | M |

---

## 11. Open decisions with recommendations

| # | Decision | Options | Recommendation | Evidence |
|---|---|---|---|---|
| D1 | Desktop engine | dual (wasmi for sessions, Wasmtime for compute) / **Wasmtime only** | **Wasmtime only on desktop**; wasmi stays for iOS and as the CI reference | 0.87–1.29 ms vs 11.3 ms; 6 µs instantiation; 0.04 ms deserialize remove wasmi's only advantage |
| D2 | Web engine | wasmi inside the app's Wasm / **browser WebAssembly** | **Browser WebAssembly in Workers** | interpreter-in-Wasm is slower again than wasmi native; browsers JIT Wasm; Worker termination gives a hard kill |
| D3 | Winch fast tier | yes / **no** | **No** for now: AOT at install hides Cranelift's 9–12 ms; revisit if first-use latency complaints appear (2.2 ms compile, about 3.5× slower code) | §4.2 |
| D4 | Zero-copy mechanism | **views (host writes into guest memory)** / mmap aliasing / host-resident handles / WIT lists | **Views + handles**; reject mmap aliasing and WIT `list<…>` on the data plane | §3.1 table (D gains nothing; the canonical ABI copies lists) |
| D5 | Output validation | per call / **at commit, export and GPU upload** | **At commit, export and GPU upload** | 1.17 ms per 1080p per call is about the cost of the loop itself |
| D6 | Plugin GPU device | dedicated plugin device / **app's own device** | **The app's device** for interactive work; the existing export and bake devices for jobs. A dedicated device would force copies (wgpu textures are device-bound) | §1, §5.5 |
| D7 | GPU tier order | — | **L1 → L2 → L4a → L3 → L4b → L4c** | reach versus attack surface (§5.2) |
| D8 | GPU CPU fallback | naga-IR interpreter / **declared Wasm or `kernel.program` fallback + software adapters in CI** | **Declared fallback + lavapipe/WARP in CI** | no maintained naga CPU backend; software adapters run the same code |
| D9 | Timestamp queries | optional / **requested by every GPU app when available** | **Request them everywhere**, as PhotoCraft does | needed for fair scheduling; zero measurable cost |
| D10 | Shared GPU crate | per-app / **`craft-gpu` in craft-libs** (device context, texture pool, memcap, scheduler, pipeline warmup) + **`ext-gpu`** in craft-extensions (plugin tiers) | **Both**: `ext-gpu` first, scoped to plugins; apps move shared pieces into `craft-gpu` as they are touched | five device-creation paths and two memcaps today |
| D11 | NN runtime | ort / candle / **wgpu-conv + candle** | **wgpu-conv (LightCraft) + candle**; ort later as an optional runtime pack | pure Rust, already in the suite; wgpu backend shares the app device |
| D12 | Model storage | per app / **suite-wide content-addressed store** | **Suite-wide**, mmap'd safetensors, consented downloads via `net` | three downloaders today |
| D13 | Wasm threads | wasi-threads / **host tiles** | **Host tiles** | 8 bands give 0.31 ms without copies; no shared-memory risk |
| D14 | Relaxed SIMD and NaN canonicalization | — | **Relaxed SIMD off** (opt-in for preview-only); **NaN canonicalization on**, once benchmarked | determinism of caches and exports |
| D15 | Pooling allocator | on demand / **pooling for filters, audio and nodes** | **Pooling**, about 256 slots, keep-resident for the filter and audio classes | 6.2 vs 11.8 µs; no mmap on the audio thread |
| D16 | Default plugin interchange format | f32 / **f16** | **RGBA16F by default**, f32 on request | half the upload, readback and bandwidth (§7.4); filterable in WebGPU core |
| D17 | Binary size budget for Wasmtime in every app | — | **Measure in one app** (the bench binary with Cranelift, Winch and wasmi was 12.5 MB stripped); drop Winch; consider `wasmtime` without `cranelift` in app binaries plus an out-of-process compile helper only if the size is a problem | §4.6 |

---

## Sources

App and platform source (read-only):
- `effectcraft/crates/gpu/src/{context.rs,walk.rs,memcap.rs,readback.rs}`,
  `effectcraft/crates/effects/src/external.rs`, `effectcraft/crates/sam2/src/gpu.rs`,
  `effectcraft/crates/engine/src/analysis/models.rs`;
- `photocraft/crates/ui-egui/src/canvas/gpu_canvas/setup.rs`, `photocraft/crates/gpu/src/{timing.rs,health.rs,context.rs}`;
- `lightcraft/crates/gpu/src/{ctx.rs,nn.rs,faces.rs}`, `lightcraft/crates/denoise`;
- `texturecraft/crates/render/src/gpu_bake.rs`; `filmcraft/crates/gpu/src/export_renderer.rs`;
  `anicraft/crates/gpu/src/lib.rs`;
- `craft-extensions/crates/ext-host/{Cargo.toml,src/wasm.rs}`;
- `craft-extensions/docs/plugins/research/capability-program-2026-11-18.md`,
  `capability-2026-11-18/{platform,3d,audio}.md`, `capability-2026-11-18/bench/*-results.txt`.

Crate source in the cargo registry:
- `wgpu-30.0.1/src/api/device.rs:139-197` (shader runtime checks);
- `wgpu-30.0.1/src/api/pipeline_cache.rs` (Vulkan only);
- `wgpu-types-30.0.1/src/{shader.rs,features.rs}`;
- `wasmtime-50.0.2/src/config.rs` (pooling, `with_host_memory`), `src/runtime/module.rs`
  (`deserialize*`, `serialize`).

Web:
- [Chromium WebGPU technical report](https://chromium.googlesource.com/chromium/src/+/main/docs/security/research/graphics/webgpu_technical_report.md)
- [W3C WebGPU (security considerations, watchdog note)](https://w3.org/TR/2025/CRD-webgpu-20250320)
- [DarthShader: fuzzing WebGPU shader translators](https://arxiv.org/pdf/2409.01824)
- [wgpu v30.0.0 release mirror](https://sourceforge.net/projects/wgpu.mirror/files/v30.0.0/)
- [wgpu releases](https://github.com/gfx-rs/wgpu/releases)
- wgpu loop-bounding PRs [#6929](https://github.com/gfx-rs/wgpu/pull/6929) and [#7080](https://github.com/gfx-rs/wgpu/pull/7080) (via `3d.md`)
- [Wasmtime `PoolingAllocationConfig`](https://docs.rs/wasmtime/latest/wasmtime/struct.PoolingAllocationConfig.html)
- [OpenFX GPU rendering](https://openfx.readthedocs.io/)
- [Nuke BlinkScript](https://learn.foundry.com/nuke/developers/latest/BlinkKernels/)
- [ISF spec](https://docs.isf.video/)
- [Godot `RenderingDevice`](https://docs.godotengine.org/en/stable/classes/class_renderingdevice.html)
- [Blender `gpu` module](https://docs.blender.org/api/current/gpu.html)

# Plugin capability program: what the suite needs so plugins can do anything

Date: 2026-11-18. Status: gap catalogue. Its architecture and plan are superseded by
[../architecture-2026-11-18/](../architecture-2026-11-18/README.md).

The owner's goal: a plugin author can build any plugin they want for any creative ArtCraft app.
Sandboxed plugins (Wasm, and the GPU and data tiers proposed below) should cover nearly everything,
so native plugins are rarely needed. Everything must be fast, centralized in `craft-extensions`,
simple for authors, and simple for agents to maintain.

This document merges five research passes and one benchmark run. Each pass read the app source,
the shared platform and the public plugin APIs and catalogues of the incumbent tools. The detailed
reports sit beside this file in [`capability-2026-11-18/`](capability-2026-11-18/):

| Report | Apps | Compared with |
|---|---|---|
| [motion.md](capability-2026-11-18/motion.md) | EffectCraft, FilmCraft, AniCraft | After Effects SDK, OFX, FxPlug, Premiere UXP, Resolve, Toon Boom, Moho |
| [image.md](capability-2026-11-18/image.md) | PhotoCraft, FigCraft, VectorCraft, DesignCraft, LightCraft | Photoshop UXP, Lightroom SDK, Illustrator, Figma, InDesign, Affinity |
| [3d.md](capability-2026-11-18/3d.md) | MaterialCraft, TextureCraft, SolidCraft, CadCraft | Substance Designer and Painter, Blender, Houdini, Material Maker, MaterialX |
| [audio.md](capability-2026-11-18/audio.md) | SoundCraft, audio in EffectCraft and FilmCraft | CLAP, VST3, AUv3, LV2, WAM2, WebCLAP, REAPER, Bitwig |
| [platform.md](capability-2026-11-18/platform.md) | craft-extensions and the three adapters | Wasmtime, wasmi, the component model; measured on this Mac |

Conventions used throughout:

- **Layers.** *App* means the engine itself. *Adapter* means the per-app glue between the engine
  and the platform. *Platform* means `craft-extensions`.
- **Priority.**
  - **P0** blocks a whole class of famous plugins.
  - **P1** blocks quality or speed.
  - **P2** is parity.
  - **P3** is later.
- **Effort.** **S** is up to a week, **M** one to four weeks, **L** more than four weeks. Effort
  figures are estimates. Measured numbers are marked *measured*; everything else is an estimate or
  comes from a cited source.

## 1. The ten findings that matter most

1. **One line of configuration makes every plugin almost 4× faster.** `ext-host` enables wasmi's
   `portable-dispatch` feature. That forces wasmi's slow loop interpreter. Without it, the same
   per-pixel loop at 1080p falls from 42.1 ms to 11.3 ms (*measured*, Apple M5).
2. **A JIT runs plugin code at native speed.** Wasmtime 50 with Cranelift ran the same loop in
   0.96 ms, against about 1.06 ms for native Rust (*measured*). With a JIT, authors can write their
   own pixel loops instead of being limited to the 16 host kernels. That is the core of "do
   anything".
3. **Copying now costs as much as computing.** `Filter::run` adds about 10 ms of copies and
   conversions at 1080p. A Wasm filter that calls the host blur spends half its 28 ms converting
   buffers (*measured*). Zero-copy pixel views must come with, or before, the JIT.
4. **The apps can already do far more than plugins can reach.** Every app has a JSON command
   registry that its own MCP server drives:

   | App | Engine commands | Plugin ops |
   |---|---|---|
   | EffectCraft | 435+ | about 37 |
   | FigCraft | about 630 | about 74 |
   | PhotoCraft | registry-wide | about 40 |
   | VectorCraft | about 888 | 0 |
   | DesignCraft | about 635 | 0 |
   | LightCraft | about 325 | 0 |
   | MaterialCraft | about 50 | 0 |
   | TextureCraft | about 80 | 0 |

   One generated, permission-classified bridge over these registries (a safe version of
   Photoshop's `batchPlay`) is the single largest capability win.
5. **EffectCraft's built-in effects get far more than plugin effects do.** Built-ins get other
   frames, other layers, audio, depth and ID channels, and the comp camera and lights
   (`EffectHost`). Plugins get one image and a few numbers. Opening this up, plus region-of-interest
   and tiled rendering, unlocks Twixtor-, Neat Video-, Particular-, Saber- and Optical Flares-class
   plugins.
6. **Every pro plugin API has a GPU path, and we have none.** Plugin effects are CPU-only and break
   EffectCraft's GPU chains. The proposed fix is a WGSL shader tier: the host writes the entry
   point, naga validates the code, loops are bounded, work is tiled with a watchdog, and it runs on
   the app's own wgpu device. WGSL nodes are also the fast path for 4K material and texture work.
7. **The basics every commercial plugin assumes are missing:**
   - network with an allowed-domain list and consent;
   - secrets and OAuth;
   - private settings;
   - change events with change lists;
   - clipboard;
   - drag and drop;
   - binary pixel transport;
   - AI inference;
   - fonts.

   Most are small or medium jobs, and together they unlock most of the Figma, Lightroom and
   UXP-panel catalogue.
8. **The adapters have drifted apart.** About 1,500 lines are the same logic written three times,
   and the copies behave differently. Only 6 ops behave identically in all three apps, so a plugin
   can't be written once for the suite. One `CraftApp` trait, with the platform owning the
   lifecycle and one conformance suite every adapter must pass, cuts adapter code by about 65%.
9. **Eight of the creative apps have no plugin support at all:** FilmCraft, VectorCraft,
   DesignCraft, LightCraft, MaterialCraft, TextureCraft, SoundCraft and AniCraft. SoundCraft is a
   partial exception: it hosts native CLAP, VST3 and AU plugins, but inside its own process, so a
   crash takes the app down. VectorCraft runs a second, competing Wasm plugin system that the suite
   replacement rule says must be migrated and removed. MaterialCraft and TextureCraft also need app
   changes first, because their node and effect lists are closed Rust enums.
10. **Real-time audio can be sandboxed, but only on a JIT.** CLAP semantics over Wasm, run on
    Wasmtime/Cranelift ahead of time, costs about 1.5× native (WebCLAP's figure). An interpreter
    costs 5–20×, which is too much for a mixer. Faults fall back to the dry signal, never to
    silence.

## 2. Target architecture

### 2.1 Execution tiers

A plugin picks the cheapest tier that can express its work. The host owns scheduling, budgets and
caching in every tier.

| Tier | Runs | For | Isolation |
|---|---|---|---|
| **Data** | nothing | presets, LUTs, brushes, smart materials, fonts, themes, graph-as-node packages | no code |
| **Host graph** | built-in nodes and kernels composed by the plugin (`kernel.graph`, `graph` node tier) | most custom effects and material nodes | no plugin code at render time |
| **Kernel program** | a small typed per-pixel IR the host runs on CPU threads or translates to WGSL (`kernel.program`) | custom per-pixel maths without a guest loop | host-interpreted, bounded |
| **WGSL** | plugin function bodies inside a host-written shader, validated by naga, bounded loops, tiled with a watchdog | per-pixel effects, generators, material nodes, LUT and colour work at GPU speed | WebGPU-grade validation plus a digest kill list |
| **Wasm (interpreter)** | wasmi with tail-call dispatch | UI, sessions, events, small logic, web and iOS targets | full sandbox, fuel |
| **Wasm (JIT)** | Wasmtime with Cranelift, compiled ahead of time and cached by sha256; epoch interruption | heavy algorithms, importers, simulations, real-time audio | full sandbox, epoch deadlines |
| **Native, restricted** | out-of-process helper under Seatbelt or bubblewrap | closed vendor engines that need no network or files | OS sandbox |
| **Native, full** | out-of-process helper with consent | hardware, DRM, OS-wide input hooks, apps outside the suite | consent only |

The same manifest selects the tier per contribution (`Manifest::runtime` already does this for
Data, Wasm and Native).

### 2.2 One adapter seam

Today's seam (`AppApi`: `contract`, `begin`, `call`, `finish`) is too shallow. Every adapter rebuilt
the threads, timeouts, grants, workers, events, jobs and references around it. The target:

```rust
pub trait CraftApp: Send + Sync + 'static {
    type Doc: 'static;
    type Snapshot: Send;
    fn app(&self) -> AppId;
    fn provided(&self) -> Provided;
    fn ops(&self) -> &'static OpTable<Self>;
    fn registry(&self) -> &dyn CommandRegistry;
    fn active_document(&self) -> Option<DocToken>;
    fn with_document(&self, doc: DocToken, f: Box<dyn FnOnce(&mut Self::Doc) + Send>);
    fn snapshot(&self, doc: &mut Self::Doc) -> Self::Snapshot;
    fn restore(&self, doc: &mut Self::Doc, s: Self::Snapshot);
    fn commit(&self, doc: &mut Self::Doc, label: &str);
    fn busy(&self, doc: &Self::Doc) -> bool;
    fn plugin_data<'d>(&self, doc: &'d mut Self::Doc, at: &Target) -> Result<&'d mut PluginDataMap, OpError>;
    fn references<'d>(&self, doc: &'d mut Self::Doc) -> &'d mut Vec<PluginRef>;
    fn pixels(&self, doc: &mut Self::Doc, at: &Target, region: Rect) -> Result<PixelView<'_>, OpError>;
    fn host(&self) -> &dyn HostUi;
}
```

`craft_ext_runtime::Runtime<A: CraftApp>` owns the following:

- loading, reload, workers, quarantine and safe mode;
- one grant cache;
- the transaction state machine, with one timeout and one message;
- op dispatch from the `OpTable`, with schema, permission, panic containment and timing;
- every built-in op family (`ui.*`, `project.data.*`, `job.*`, `kernel.*`, `settings.*`, `net.*`,
  `clipboard.*`, `timer.*`, `service.*`, `files.*`);
- the event bus with canonical payloads;
- job delivery;
- reference tracking;
- the filter service (instance pool, module cache, tiles, result cache);
- the catalogue refresh scheduler;
- generated contract docs and typed guest bindings.

The app supplies only document primitives, its own ops, and its command registry. A shared
conformance suite (`conformance::run::<MyApp>()`) checks that every adapter behaves the same way.
It covers rollback, revocation mid-event, quarantine, data limits, events, jobs and missing plugins.

Migration order: FigCraft (smallest and already command-based), then EffectCraft, then PhotoCraft.
New apps start on `Runtime<A>` directly.

### 2.3 The registry bridge (a safe `batchPlay`)

Each app's `CommandSpec` registry is classified once, by a generated, reviewed table checked into
the adapter:

- `query`, with permission `document.read` at selection or document scope;
- `edit`, with permission `document.edit` at selection or document scope;
- `refused`: file system, network, preferences, plugin management, collaboration, library, undo,
  window management.

Plugins call `app.run {command, params}` and discover the allowed set with `app.commands`. Each
spec's schema validates the parameters, and unknown parameters are refused. The bridge sits beside
the typed shared ops, not instead of them. Shared ops (`selection.get`, `layer.list`, `node.get`,
`pixels.*`) get one canonical shape across apps; `app.run` gives breadth.

A deterministic check keeps it honest: CI fails when a registry command has no classification, or
when a `refused` class command appears in the `edit` list.

### 2.4 Contract families

Existing: `raster.filter@1`, `<app>.session@1.x`, `resource.{preset,lut,font}`, `craft-theme`.

| Contract | Purpose | Apps | P |
|---|---|---|---|
| `raster.filter@2` | ROI and tiles with padding, preview scale, output bounds and size (glows, upscalers), declared extra inputs (layer parameters), temporal frames (`frame.at`), optional `craft_frames` prefetch, `quality` and `thread_safe` | EC, PC, FC (film), VC raster effects, FigCraft image fills | P0 |
| `graph.node@1` | typed ports and parameters; `graph`, `wgsl` or `wasm` cook; deterministic with a seed; host-drawn parameter UI | MaterialCraft, TextureCraft, EC, PC | P0 |
| `gpu.shader@1` | WGSL passes with declared bindings; `gpu.dispatch` from Wasm for multi-pass effects; required Wasm or kernel fallback for headless runs | EC, FC (film), PC, LC, MC, TC | P1 |
| `video.transition@1`, `video.generator@1`, `video.retimer@1` | OFX-style contexts | EC, FilmCraft | P1 |
| `audio.effect@1`, `audio.instrument@1`, `audio.note-fx@1` | CLAP semantics: sample-accurate events, modulation separate from automation, gestures, note expressions, latency, sidechain, state, meter rings; WebCLAP import | SoundCraft, FilmCraft, EC | P0 for audio |
| `audio.clip-editor@1` | ARA-style clip access, analysis jobs, render on demand | SoundCraft, FilmCraft | P2 |
| `format.import@1`, `format.export@1` | bytes in, neutral document draft out (raster, vector, story, mesh, material), and back | all | P1 |
| `codegen@1` | Dev Mode code panels | FigCraft | P1 |
| `export.service@1`, `publish.service@1`, `metadata.provider@1` | Lightroom-class export, upload and sync; custom searchable fields | LightCraft | P0 for LC |
| `vector.effect@1` | live effect in the appearance stack, geometry or raster output | VectorCraft, FigCraft | P0 for VC |
| `preflight.rule@1`, `story.import@1` | layout automation | DesignCraft | P1 |
| `input.filter@1` | transform the pen stream before built-in tools (stabilisers, rulers) | PC, VC, TC, AniCraft | P1 |
| `brush.dab@1` and `resource.brush@1` | custom brush engines and ABR-class presets | PC, TC, AniCraft | P1 |
| `texture.bake-map@1` plus `bake.rays` | custom bakers on a host ray service | TextureCraft | P2 |
| `viewport.overlay@1` | world-space draw lists and 3D gestures | TC, MC, SolidCraft, EC 3D | P2 |
| `resource.{smart-material, smart-mask, brush-alpha, environment, material, export-template, profile, model}` | data packages | TC, MC, LC, PC | P1 |
| `widget@1` | on-canvas plugin node with synced state | FigCraft | P3 |

## 3. Benchmarks

*Measured* on an Apple M5 with 10 cores, Rust 1.97.1, release builds, median of 5 runs after
warm-up. Sources and raw output are in [`capability-2026-11-18/bench/`](capability-2026-11-18/bench/).

### Per-pixel brightness loop

The loop is 1920×1080 RGBA f32, 33 MB.

| Engine | 1080p | 4K | vs native | Compile |
|---|---|---|---|---|
| Native Rust | 1.06 ms | – | 1× | – |
| Platform today: `Filter::run` end to end | **52.0 ms** | 213.8 ms | 49× | 0.9 ms |
| wasmi, portable dispatch, fuel | 42.1 ms | – | 40× | – |
| wasmi, tail-call dispatch, fuel | **11.3 ms** | 35.9 ms | 11× | 0.4–1.9 ms |
| Wasmtime 50, Winch | 3.6 ms | 14.7 ms | 3.4× | 6.6 ms |
| Wasmtime 50, Cranelift, epoch interruption | 1.29 ms | 5.1 ms | 1.2× | 40 ms |
| Wasmtime 50, Cranelift, no metering | **0.96 ms** | 3.6 ms | 0.9× | 34–42 ms |

SIMD128 made no difference here, because the loop is memory-bound. Today's platform refuses SIMD
modules at load: "SIMD support is not enabled".

### Host calls and kernels

| Measure | Result |
|---|---|
| `craft.call` with a small JSON payload | 10.5 µs per call, about 95,000 per second |
| 30,000 calls in one event | refused as too slow: the fuel cap allows about 19,000 |
| Native serde_json for the same work | 0.57 µs |
| `kernel.blur`, radius 20 | 13.7 ms at 1080p, 67.4 ms at 4K |
| `kernel.blend`, overlay | 4.9 ms at 1080p, 19.6 ms at 4K |
| Wasm filter calling `kernel.blur` r=20, end to end | 28.4 ms at 1080p, 127.9 ms at 4K |

### Earlier figures through the full sandbox (program doc)

Each is per 1080p frame:

- Lightwell: 478 ms.
- Softskin: 602 ms.
- Emberfield: 176 ms.
- A pure per-pixel Wasm effect: about 17 s, in a debug app build.

### Performance targets for the plan

These are proposed acceptance numbers. Each phase must measure against them in CI on a fixed
runner, with a ±15% tolerance.

| Case | Today | Target |
|---|---|---|
| Per-pixel loop, 1080p, end to end | 52 ms | ≤ 15 ms on wasmi tail-call with zero-copy; ≤ 3 ms on Cranelift |
| Lightwell, 1080p | 478 ms | ≤ 100 ms (zero-copy, fused graph); ≤ 10 ms (WGSL) |
| Softskin, 24 MP RGB16 | not measured | ≤ 1.5 s (kernels and tiles) |
| Nik-style local contrast preview | – | ≤ 100 ms per preview frame (WGSL, proxy scale) |
| 4K material node, per-pixel generator | about 2 min (estimate, wasmi) | ≤ 10 ms (WGSL) |
| `craft.call` small op | 10.5 µs | ≤ 3 µs (tail-call or JIT guest serde), ≤ 1 µs (typed binary ABI) |
| Real-time audio EQ, 128-frame blocks | – | p99.9 block time ≤ 2× native, zero dropouts across 10 min |
| AI 2× upscale, 24 MP | – | ≤ 20 s on GPU via `nn.run` |

## 4. Platform work (craft-extensions)

Merged and deduplicated across all five reports.

### 4.1 Runtime and speed

| # | Item | P | E |
|---|---|---|---|
| R1 | Drop `portable-dispatch` from wasmi; keep `deterministic`; add a CI benchmark that fails on a 20% regression | P0 | S |
| R2 | Enable the wasmi `simd` feature; build SDK templates with `+simd128` | P1 | S |
| R3 | Zero-copy pixels: one `memcpy` into linear memory; kernels run on `&mut [f32]` views of guest memory; one fused validation pass | P0 | S |
| R4 | `Engine` trait with wasmi (default, web, iOS, reference) and Wasmtime with Cranelift (desktop). Ahead-of-time compile at install, artifact cache keyed by module sha256 plus engine version; epoch interruption for deadlines; pooling allocator; NaN canonicalization for determinism | P0 | M |
| R5 | Host-resident image handles (`image.alloc`, `image.read_region`, `image.write_region`), plus Gray, f16 and u16 storage, so 4K maps never enter guest memory and the 512 MiB cap stops mattering | P0 | M |
| R6 | Binary transport (`craft.call_buf`): ops read from and write to a guest buffer, with a `-3` fallback. Replaces base64 pixels, which today are capped at 1 MiB per write | P0 | M |
| R7 | Bill host calls by bytes and wall time, not a flat 100k fuel; batch ops (`node.get {ids}`, `ops.batch`) | P1 | S |
| R8 | Session memory per contribution in the manifest (up to 512 MiB), with app-level accounting | P1 | S |
| R9 | Parallel tiles: the host runs one instance per tile across pools; `job.map` fans inputs out to N sandboxes | P1 | M |
| R10 | Wasm threads: not planned. Host-side parallelism instead | – | – |

### 4.2 Compute

| # | Item | P | E |
|---|---|---|---|
| K1 | `kernel.graph`: fuse a chain into one tiled, multi-threaded call, with one fuel charge, and map to WGSL twins on GPU | P1 | M |
| K2 | `kernel.program`: a validated per-pixel IR on host threads or translated to WGSL. Seed it from MaterialCraft's function VM | P1 | M |
| K3 | Video kernels: radial and zoom blur, pyramid, displace, mesh warp, perspective, lut3d (tetrahedral), FFT and FFT convolve, median, dense flow, warp by flow, NCC match, raster lines, paths, meshes, text shaping, distance transform, connected components | P1 | L |
| K4 | Photo kernels: guided filter, surface blur, high pass and frequency split, unsharp, colour convert (via craft-cms), tone map, Lanczos resize, stack reduce, flood fill | P1 | M |
| K5 | Material kernels: Perlin, gradient, Worley and 3D tileable noise; height to normal; RNM normal blend; slope blur; tile sampler; gradient map; histogram scan; levels. Move them from MaterialCraft's `nodes/src/ops.rs` into a shared crate | P0 for MC/TC | M |
| K6 | Audio kernels (allocation-free, run on the calling thread): FFT/STFT, FIR and partitioned convolution, resample, oversample, true peak, loudness. Share them with SoundCraft and FilmCraft through a new `craft-audio-dsp` crate in craft-libs | P1 | M |
| K7 | `ext-gpu`: WGSL function nodes. The host writes the entry point and bindings, and IR inspection refuses storage, workgroup, atomics and barriers in plugin code. naga validates at install and on load. Untrusted loop bounding (`force_loop_bounding`). Tiled submits with timestamp budgets, device-loss recovery, a digest kill list, and a separate device or queue for cooks | P0 | L |
| K8 | Shared `craft-nn` (from LightCraft's pure-Rust ONNX interpreter plus WGSL convolutions, with candle as a second backend): `nn.load` and `nn.run`, shaped like wasi-nn. Models are data contributions pinned by sha256; jobs only at first; permission `compute.nn` with download size shown at consent. `ml.stream` for small real-time audio models later | P1 | L |

Kernel quality fixes from the benchmark pass:
- Hoist the per-pixel blend-mode string match in `ext-kernels` to a function pointer.
- `separable` clones the image twice.

### 4.3 Host services

| # | Item | P | E |
|---|---|---|---|
| H1 | `settings.get/set`: private per-plugin storage, up to 1 MiB, survives updates, removed on uninstall | P0 | S |
| H2 | `net.fetch`: asynchronous, with completion as an event. Declared `network:<host>` domains only, consent on first use with the manifest's reasoning, HTTPS only, no redirects off the list, size, time and rate limits, no cookies, the shared HTTPS agent, and a per-plugin network log in the manager | P0 | M |
| H3 | `secret.get/set/delete` in the OS keychain; `auth.oauth` with PKCE through the system browser and a loopback redirect | P0 | M |
| H4 | Event bus: `events.subscribe {names, filter}`; canonical payloads with change lists (`document.changed {revision, changes:[…]}`); `document.activated/saved`, `history.changed`, `color.changed`, `tool.changed`, `panel.opened/closed`, `dispose`; coalescing; edits from a change event run as a deferred follow-up event with loop detection | P0 | M |
| H5 | `files.*` portals for every app (PhotoCraft's handle model moved into the platform); persistent folder grants for exporters; `asset.read` for files in the plugin's own package | P0 | S |
| H6 | `clipboard.read/write`: text, image handle, SVG, app-private JSON; writes only during user-initiated events, reads need a grant | P1 | S |
| H7 | Drag and drop: `drag_source` on list, table and image nodes; dropped payloads placed by the plugin; OS file drops onto panels as handles | P1 | M |
| H8 | `timer.set` (16 ms minimum, coalesced, paused when hidden) and host-driven `ui.animate` | P1 | S |
| H9 | `service.call`: inter-plugin services declared in the manifest, run in the provider's sandbox with the provider's grants, versioned like contracts | P2 | M |
| H10 | Fonts: activate `resource.font` through the shared font library; `fonts.list`, `fonts.glyphs`; `font.install` with consent (Fontself class) | P1 | M |
| H11 | Localisation: `locales/<lang>.ftl` in packages, resolved for forms and UI trees | P2 | S |
| H12 | `host.capabilities`: ops, kernels, UI level and limits, so plugins degrade explicitly | P1 | S |
| H13 | Cross-app handoff: `handoff.open {app, pixels or file, return}` brokered by the launcher (LightCraft to PhotoCraft "Edit in", FilmCraft to PhotoCraft) | P2 | M |
| H14 | Per-instance cache and state: `cache.get/put` keyed by plugin, instance, parameter hash and time; project state on the effect instance (`project.data` target `{effect}`) | P1 | M |
| H15 | Native helper shared-memory transport (pixel and audio rings) for the few native plugins that must process media | P2 | M |

### 4.4 Plugin UI

| # | Item | P | E |
|---|---|---|---|
| U1 | Live controls: `live:true` sliders stream throttled values; continuous gestures (begin, change, end) for automation recording | P0 | S |
| U2 | Overlay `image` op, hit regions with hover and drag, and `canvas.preview.set` for unapplied previews over the document | P0 | M |
| U3 | Effect on-canvas handles: a `craft_overlay` export, with drags turned into parameter edits in one undo step | P1 | M |
| U4 | Parameter kinds: Point3, Layer, Mask, Path, Curve, Gradient, File (resolved to a handle, never a path), Data (hidden, animatable), Button, Group, Angle with handle | P0 | M |
| U5 | New widgets: curve editor, gradient editor, colour wheel, image preview from a handle, `viewer` of a host-rendered frame, animated thumbnails, meter and scope bindings fed from lock-free rings, 60 fps canvas | P1 | M |
| U6 | Pen fidelity in tool events: tilt, rotation, pointer type, hover, and coalesced full-rate samples | P1 | S |
| U7 | One manager stack (ext-manager as the model, with egui and framework views) and one jobs list | P1 | M |
| U8 | Command palette, shortcuts and Actions visibility for plugin commands; recorded parameters so batch and droplets can replay them | P1 | S |

### 4.5 Authoring and agents

| # | Item | P | E |
|---|---|---|---|
| D1 | Generated typed guest bindings (Rust first, then C, Zig, AssemblyScript headers and `.d.ts`) from `OpTable` schemas for every op, not just today's 6 | P1 | M |
| D2 | One checked-in `ops.json` (machine-readable contracts for every app) that CI checks against `OpTable`; Markdown contract pages generated from it | P1 | S |
| D3 | `MockHost` that loads the real app contract and rejects invalid calls; `craft-ext test` runs session scripts (events in, expected calls out) and golden-image tests | P0 | M |
| D4 | `craft-ext dev --watch` with hot reload that keeps panel state; an in-app plugin console (log, op trace, fuel, memory, host calls) | P1 | M |
| D5 | Keep the guest's own error text through `craft.result`; `-1` messages name the op, argument path and schema rule | P0 | S |
| D6 | Templates for command, tool, hook, job, shader, node, audio effect, importer and multi-app plugins, each with a passing test; non-Rust minimal templates; Faust and Cmajor to Wasm for audio | P1 | M |
| D7 | One headless runner: `craft-ext run --app <app> --doc <file> --script <jsonl>`, replacing three CLI syntaxes | P1 | M |
| D8 | WIT world `craft:plugin@2` on the component model, once Wasmtime is the desktop engine; JSON ops remain for app-specific ops | P2 | L |
| D9 | Source-level debugging (DWARF under Wasmtime) and `--trace-ops` replay | P2 | M |
| D10 | An `AGENT-START` page listing the five commands an agent needs; fuel, memory and host-call usage in test output | P1 | S |

## 5. Centralization: what moves into craft-extensions

Measured sizes: shared crates about 26,200 lines. Adapters:

- EffectCraft about 4,350 lines;
- PhotoCraft about 5,900 lines;
- FigCraft about 1,320 lines.

| Duplicated concern | Approx. lines | How the three copies differ today |
|---|---|---|
| Grants | 150 | Caching, failure modes and safe-mode switches. PhotoCraft opens the grant store about 8 times per op |
| Transaction loop | 300 | Nested begin, rollback replies and timeout messages all differ; only EffectCraft tells the user about the timeout |
| UI-thread waits | 120 | 8 s, 10 s and 9 s stalls |
| Workers and quarantine | 200 | Same three-strike policy, written three times |
| Plugin data ops | 120 | FigCraft writes past the edit API |
| `ui.*` limits | 200 | 4 windows per plugin, against 32 surfaces per app; FigCraft `ui.notify` only logs |
| Job delivery | 110 | Different document keys and busy rules |
| App events | 100 | Different payloads for the same event; PhotoCraft drops hook errors |
| References and missing plugins | 100 | PhotoCraft cannot detect a state schema mismatch |
| Catalogue refresh | 90 | Three schedules; PhotoCraft refreshes on every launch, even in safe mode |
| Filter running | – | Only EffectCraft pools instances. PhotoCraft compiles each filter from disk twice per run, copies the full buffer about 6 times, and starts a thread per slider change |
| UI glue | 200 | Two manager stacks; PhotoCraft duplicates the jobs list |

Bugs found by reading the code (to confirm with a test before fixing):

- FigCraft's read-only guard misses `selection.set` and `viewport.set`.
- `layer.select` needs edit permission in EffectCraft but read permission in PhotoCraft.
- FigCraft's `ui.notify` is invisible.
- EffectCraft repaints every 100 ms when idle.
- PhotoCraft re-serializes every UI tree on each `ui.*` call.
- `figcraft/crates/plugins/src/lib.rs:17-104` looks like dead code.
- Filter errors drop the guest's message (`wasm.rs:173,383,663,804`).
- PhotoCraft tool events drop tilt and rotation (`plugin_sessions.rs:368`).
- The author guide and `capabilities.md` disagree about PhotoCraft project references.

Rules to add to AGENTS.md when this work starts:

- **Behaviour goes in the runtime, never in an adapter.** An adapter may only map an op onto its
  engine.
- **A shared op has one schema and one payload for every app**, checked by the conformance suite.
- **New op families start in `OpTable` with a schema**, so bindings and docs are generated, never
  hand-written.

## 6. Per-app plans

Each app section lists what the app needs, what its adapter needs, and which famous plugin classes
that unlocks.

### EffectCraft (After Effects class)

- **App:**
  - a GPU path for external effects (`render_gpu`, `gpu = true` when a validated shader is shipped);
  - map the new parameter kinds onto the existing `ParamUi` variants;
  - effect overlays in the viewer;
  - a bounded "render layer or comp at time t" API for panels and exporters;
  - import and export registries;
  - stateful streaming audio mixdown, so audio plugins don't restart from preroll every block.
- **Adapter:**
  - pass `EffectHost` through to filters (`self_at`, `layer_at`, `layer`, `audio`, `aux`,
    `comp_scene`, `params_at`, `param_at`);
  - richer filter context (fps, shutter, span, masks, adjustment flag, depth);
  - typed setters for text, path and gradient values;
  - `prop.setExpression`;
  - structural ops through the registry bridge (parent, precompose, masks, track matte, cameras,
    lights, time remap);
  - full keyframe ops (`keys.list/delete/move/setTangents/batch`);
  - tracking, roto and render-queue ops as observable jobs;
  - `audio.read`;
  - finer events (property paths, key times).
- **Unlocks:**
  - Trapcode Particular, Form, Shine, Mir and 3D Stroke;
  - Plexus, Saber, Optical Flares and Deep Glow;
  - Sapphire and Continuum;
  - Mocha-style tracking panels;
  - Twixtor and RSMB;
  - Neat Video-class denoise;
  - Magic Bullet Looks;
  - Duik, Rubberhose, Motion, Flow, Newton, Explode Shape Layers, BeatEdit and Sound Keys;
  - a Frame.io-style panel.
- **Still native:** Element 3D-class custom renderers (most uses could move to a `scene.render` op
  on EffectCraft's advanced 3D renderer), and vendor CUDA paths.

### FilmCraft (Premiere class)

- **App:**
  - an external effect registry (effect ids are already strings, and `FxEnv` already gives other
    frames and tracks);
  - an external transition registry;
  - an external audio effect slot (its own `AudioEffect` with per-sample smoothing is the model the
    audio contract follows);
  - an import and export registry.
- **Adapter (new):**
  - `filmcraft.session@1`: sequences, clips, insert, overwrite, trim and razor, markers, captions,
    transcript, effects and export, all through the registry bridge;
  - `raster.filter@2` on clip effects;
  - `video.transition@1`;
  - `audio.effect@1`.
- **Unlocks:**
  - Red Giant Universe transitions;
  - Magic Bullet and FilmConvert looks;
  - auto-captioning and transcript tools (Whisper is already in the app);
  - scene detection and auto-reframe panels;
  - Frame.io-style review panels;
  - BeatEdit;
  - audio repair and loudness plugins.

### AniCraft (2D animation)

- There is no source on this Mac (the clone is empty), so this section is planning only.
- **App:** a command registry and `Session::execute` from day one, the same pattern as EffectCraft
  and FilmCraft.
- **Adapter:**
  - stroke and vector drawing ops;
  - a bone and deformer model;
  - exposure-sheet ops;
  - switch layers for lip sync;
  - `audio.read`;
  - pen-fidelity tool events;
  - `input.filter`;
  - `brush.dab`;
  - exporters.
- **Unlocks:**
  - rigging scripts in the Moho and Harmony style;
  - Rhubarb- and Papagayo-style lip sync;
  - inbetweening and onion-skin tools;
  - Spine, DragonBones, Lottie and sprite-sheet exporters;
  - brush engines.

### PhotoCraft (Photoshop class)

- **App:**
  - a wgpu compute queue beside the fragment compositor;
  - smart filters that dispatch to plugin filters through the deterministic path, with a cached
    raster when the plugin is missing;
  - a dynamic command overlay so plugin commands reach menus, the palette, Actions and MCP.
- **Adapter:**
  - mask and channel pixels;
  - channel to selection;
  - clipping, locks and Blend If;
  - editing adjustment layers in place;
  - layer styles, text, paths and vector masks;
  - smart object place and replace;
  - foreground and background colour and swatches;
  - ICC and mode conversion, with Gray, CMYK and Lab pixel access;
  - `paint.stroke` and brush presets;
  - content-aware fill, Liquify with a plugin mesh, and Camera Raw;
  - resize, canvas size, crop and new document;
  - journaling plugin commands into Actions;
  - metadata;
  - tilt and rotation in tool events;
  - change events.
- **Unlocks:**
  - Topaz Photo AI, Sharpen and Gigapixel (with `nn.run`);
  - Nik Collection;
  - Portraiture and Retouch4me;
  - Infinite Color;
  - TK8 and Lumenzia;
  - Coolorus;
  - brush packs and custom brush engines;
  - Fontself (with fonts);
  - generative fill bridges (with net or nn);
  - UXP-style panels such as Retouching Toolkit and ActionsXL;
  - exporters with format options.

### FigCraft (Figma class)

- **App:** a widget node kind (P3).
- **Adapter:**
  - the registry bridge, which takes it from 74 to about 500 commands and covers effects, text
    features, prototyping, align and distribute, vector offset, shape builder, `data.fill` and
    variable import and export;
  - `image.get` and `raster.filter` on image fills;
  - `export.png/pdf` to a buffer;
  - `svg.import` as nodes;
  - a `codegen@1` contribution (3 s deadline, building on the engine's own codegen);
  - selection-scoped grants;
  - `document.changed` with node ids;
  - pressure in tool events.
- **Unlocks:**
  - Unsplash, Pexels and Iconify (with net plus drag and drop);
  - Content Reel;
  - Autoflow;
  - Stark;
  - Tokens Studio, with GitHub sync;
  - Anima, Locofy and Builder.io codegen;
  - remove.bg and AI generators;
  - chart generators;
  - widgets.

### VectorCraft (Illustrator class)

- **App:** generalise the appearance stack to registry-backed effect ids, with plugin geometry or
  raster output.
- **Adapter (new):**
  - `vectorcraft.session@1` through the registry bridge (about 700 of its 888 commands);
  - geometry queries (intersect, distance, hit test);
  - overlay hit regions and hover for live handles;
  - `vector.effect@1`;
  - `format.import/export`;
  - object attributes through the existing `plugin_data`.
- **Migration:** move the legacy Wasm plugin kinds (object filter to command, live effect to
  `vector.effect@1`, exporter to `format.export@1`), migrate installed plugins, then delete
  `crates/plugins` under the suite replacement rule. `scriptcheck` helps port Illustrator scripts.
- **Unlocks:**
  - Astute VectorScribe, ColliderScribe, Phantasm and Texturino;
  - CADtools;
  - MAPublisher-style GIS import;
  - Illustrator script ports;
  - Fontself.

### DesignCraft (InDesign class)

- **App:** per-item change sets from the command layer (there is no event bus today, only a
  revision counter).
- **Adapter (new):**
  - `designcraft.session@1` built on the existing step script runner;
  - `story.get/set` with styled runs;
  - styles, tables and data merge;
  - `preflight.rule@1`;
  - `story.import@1`;
  - PDF post-processing hooks.
- **Unlocks:**
  - EasyCatalog- and InData-style data publishing (with net for live sources);
  - Typefi-style automated layout;
  - Xtags import;
  - custom preflight;
  - imposition.

### LightCraft (Lightroom class)

- **App:**
  - export and publish service hooks around the export pipeline, with per-photo remote ids and a
    re-publish queue;
  - a GPU LUT stage;
  - a profile registry for plugin DCP and LUT profiles;
  - namespaced, searchable custom metadata fields.
- **Adapter (new):**
  - `lightcraft.session@1`: `catalog.query` using the smart-album rules;
  - photo metadata;
  - `develop.get/set` with partial settings patches;
  - `photo.render` to a buffer;
  - `export.service@1`, `publish.service@1` and `metadata.provider@1`;
  - LUT, profile and preset resources.
- **Shared asset:** LightCraft's pure-Rust ONNX interpreter and WGSL convolutions become `craft-nn`
  for the whole suite.
- **Unlocks:**
  - Flickr, SmugMug and LR/Mogrify-style publish services;
  - Any Filter- and Data Explorer-style metadata tools;
  - Negative Lab Pro;
  - LRTimelapse;
  - preset and profile packs;
  - "Edit in" round trips through the launcher.

### MaterialCraft (Substance Designer class)

- **App:**
  - an external node source in the static node catalogue (`NodeSource::External` with package,
    node, version and digest);
  - a cook that goes through a trait object;
  - the cache key includes the plugin digest;
  - more port kinds (values, vectors, mesh, material bundle);
  - unknown plugin nodes preserved on save, shown as placeholders;
  - a wgpu compute backend shared with the WGSL tier;
  - mesh import and bakers shared with TextureCraft through a common crate.
- **Adapter (new):** `materialcraft.session@1`, generated from its about 50 commands; node
  contributions in the library under Plugins.
- **Unlocks:**
  - Designer-style custom nodes, through graph-as-node packages;
  - Material Maker and MaterialX `genglsl`-class shader nodes, through WGSL;
  - SAT/pysbs-style batch cooking;
  - Node Wrangler-style graph helpers.

### TextureCraft (Substance Painter class)

- **App:**
  - `External` variants in the closed effect, filter, source and generator enums;
  - whole-map plugin cooks cached by key;
  - unknown entries preserved on save;
  - an f32 conversion now, and GPU-resident texture handles later;
  - a viewport overlay pass with 3D picking.
- **Adapter (new):**
  - `texturecraft.session@1` (layers, masks, effects, bake as a job, export with plugin templates,
    brush presets);
  - node contributions under Add effect, Filters and Generators;
  - resource contracts for smart materials, smart masks, brush alphas, presets, export templates,
    HDRIs and materials;
  - `texture.bake-map@1` with `bake.rays`;
  - `viewport.overlay@1`;
  - format hooks.
- **Unlocks:**
  - smart material and mask packs;
  - custom generators such as edge wear from curvature;
  - Quixel, BlenderKit and Poliigon-style libraries (with net and secrets);
  - Marmoset-style custom bakers;
  - Extreme PBR-style libraries;
  - AI texture tools (with net or nn).
- **Still native:** UVPackmaster-class closed engines.

### SoundCraft (DAW class)

- **App:**
  - sample-accurate parameter events (today values apply once per 512-sample block, at
    `mix/src/strip.rs:529`);
  - sidechain and key-input routing;
  - MIDI devices, MIDI learn, MPE and MIDI 2.0;
  - hosted plugins usable as AudioSuite and render-in-place;
  - embedded editors on every platform;
  - the VST3 entitlement for signed macOS builds;
  - ARA2 later;
  - host modulators later;
  - smaller mix blocks for live monitoring.
- **Adapter:**
  - a `WasmPlugin` implementing `soundcraft_dsp::Plugin` on the real-time engine tier;
  - extend the plugin trait with event queues, modulation, multi-bus ports, note expressions,
    restart requests and meter rings;
  - session persistence with missing-plugin placeholders;
  - an AudioWorklet host for the web build.
- **Safety:** move native CLAP, VST3 and AU into a `craft-plugin-bridge` process with
  shared-memory rings and Bitwig-style isolation modes. A crash bypasses that slot and keeps the
  session alive; a panic must never silence the master again.
- **Unlocks:**
  - Valhalla, Soundtoys and FabFilter-class effects;
  - Vital- and Serum-class synths with MPE;
  - Auto-Tune live;
  - JSFX-style quick scripts via Faust or Cmajor to Wasm;
  - ReaScript-style automation through the registry bridge;
  - Demucs-style stem separation and RX-style repair (with `nn.run` jobs);
  - LANDR-style cloud services (with net).
- **Still native:** Kontakt-class multi-GB streaming until `sample.stream` exists; iLok and
  PACE-protected plugins; the commercial catalogue in general. All of these are hosted out of
  process.

### SolidCraft and CadCraft

These are 3D CAD and drafting, not material tools.

- They share the `mesh@1` data shape, the registry-bridge adapter pattern, `viewport.overlay@1`
  and the WGSL sandbox.
- CadCraft's extension story is AutoLISP-compatible scripting, which maps onto a session contract
  plus the registry bridge.

## 7. What genuinely stays native

After the plan, native is needed only for these:

- **Hardware and OS integration:** tethering, camera SDKs, colorimeters, print drivers, OS-wide
  input hooks (Lazy Nezumi across all apps), MIDI hardware outside the app's own device layer.
- **Vendor-locked code:**
  - closed engines sold under an EULA (UVPackmaster);
  - CUDA- or Metal-only kernels that WGSL cannot express;
  - proprietary AI runtimes that are not ONNX or candle (TensorRT engines, CoreML-only models);
  - Element 3D-class custom renderers;
  - iLok, PACE and licence daemons.
- **Other applications:** round trips with apps outside the suite (Topaz, Luminar, ON1 as external
  editors, Houdini Engine, Overlord to Illustrator). Loopback IPC could later be a `net.local`
  permission.
- **Commercial audio catalogues:** VST3, AU and CLAP binaries, hosted out of process.
- **Binary compatibility layers:** Photoshop 8bf, which is out of scope.

## 8. Phased plan

Each phase ends with its deterministic checks green in CI and the benchmark targets measured.

**Phase 0: quick wins, one to two weeks.**

| Work | Items |
|---|---|
| Speed | R1 (drop `portable-dispatch`), R2 (SIMD), R3 (zero-copy) |
| Small platform additions | H1 (settings), H5 (`files.*` portals for all apps, `asset.read`), D5 (error text), U1 (live sliders), U6 (pen fidelity) |
| Correctness | fix the read-out bugs in section 5, each with a test |
| Measurement | a benchmark CI job with today's numbers as the baseline |

Expected effect: existing plugins about 3–4× faster; Lightwell and Softskin noticeably quicker. No
contract changes.

**Phase 1: one runtime and breadth, four to eight weeks.**

| Work | Items |
|---|---|
| Runtime and centralization | `CraftApp` and `Runtime<A>` (section 2.2), with the conformance suite; port FigCraft, then EffectCraft, then PhotoCraft; delete the duplicated adapter code |
| Breadth | the registry bridge (section 2.3) in all three adapters |
| Host services | H2 (net), H3 (secrets and OAuth), H4 (event bus), H12 (capabilities) |
| Pixels and parameters | R5 (image handles), R6 (binary transport), U2 (overlay image, hit regions, preview), U4 (parameter kinds) |
| Contracts | `raster.filter@2` (ROI, tiles, temporal, inputs, bounds); EffectCraft `EffectHost` passthrough |
| Authoring | D1 to D3 (bindings, `ops.json`, MockHost) |

**Phase 2: speed and new kinds, eight to twelve weeks.**

| Work | Items |
|---|---|
| Engines | R4 (Wasmtime with Cranelift on desktop, ahead-of-time cache, epoch deadlines) |
| GPU | K7 (`ext-gpu` WGSL tier); `gpu.shader@1` |
| Kernels | K1 (`kernel.graph`), K2 (`kernel.program`), K3 to K5 (kernels) |
| Contracts | `graph.node@1`; `video.transition/generator/retimer` |
| New adapters | MaterialCraft and TextureCraft (after their App changes); FilmCraft |
| Rest of the services | H6 to H8 (clipboard, drag and drop, timers), H10 (fonts), H14 (cache and state); U3 (effect handles), U5 (widgets) |

**Phase 3: whole suite, twelve weeks or more.**

| Work | Items |
|---|---|
| Real-time audio | engine tier, `audio.*` contracts, SoundCraft `WasmPlugin`, native bridge process |
| AI | K8 (`craft-nn`, `nn.run`) |
| Remaining adapters | VectorCraft (with legacy removal), DesignCraft, LightCraft (with export, publish and metadata services) |
| Remaining contracts | `format.import/export`, `codegen@1`, `input.filter`, `brush.dab`, `texture.bake-map`, `viewport.overlay` |

**Phase 4: depth.**

- `audio.clip-editor@1` (ARA-like) and `sample.stream`;
- `service.call`, widgets and `ml.stream`;
- the component-model WIT world;
- source-level debugging;
- AniCraft from its first commit.

## 9. Decisions for the owner

1. **Wasmtime on desktop.** It adds a larger trusted code base (Cranelift) and 35–50 ms compile
   per module, cached once per install. It is required for real-time audio and for "write any
   algorithm". The recommendation is yes, with wasmi kept for web, iOS and as the reference engine.
2. **WGSL plugins.** They are novel: no wgpu plugin precedent was found. The risk is GPU driver
   bugs reachable through valid WGSL, the same exposure every WebGPU page has. The recommendation is
   yes, gated to Verified and above at first, with a digest kill list.
3. **Registry bridge scope.** It is broad by design. The classification table is the security
   boundary and needs review whenever an app adds commands, so CI enforces it.
4. **Order of new adapters.** The recommendation is FilmCraft, then MaterialCraft and TextureCraft,
   then VectorCraft, LightCraft and DesignCraft, then SoundCraft's audio tier. The owner may prefer
   a different order based on product priority.
5. **Native audio isolation.** Moving SoundCraft's in-process hosts into a bridge process changes
   existing behaviour, so it needs a decision on the default isolation mode.
6. **FigCraft protocol 3** (from the current beta) already refuses older peers. Keep that in mind
   if collaboration features expand to widgets.

## 10. Confidence

- **Measured on this Mac:** every number in section 3, the wasmi feature flag, and the SIMD
  refusal.
- **Read in source but not run:** app inventories, command counts, adapter duplication and the
  bugs in section 5. The duplication map was produced by a sub-agent and spot-checked.
- **From cited sources:** competitor plugin APIs and capabilities; WebCLAP 1.5×; Wasm vs native
  ranges; Winch vs Cranelift ratios.
- **Not verified:** product details marked unverified in the detailed reports (some audio
  catalogue entries, Affinity, LRTimelapse, Typefi, Em Software, Hot Door, MAPublisher). All
  projected speedups that are not in the benchmark table.
- **No source on disk:** AniCraft.

# Plugin capability gaps: motion graphics, video and animation

Date: 2026-11-18. Scope: EffectCraft (`effectcraft`, branch
`plugin-beta`), FilmCraft (`filmcraft`, branch `main`, no plugin adapter),
AniCraft (competitive analysis only: `AniCraft` is an empty partial clone with no
checked-out files), and the shared platform (`craft-extensions`).

Method: source and doc reading plus web research. Nothing was built, run or committed. Line numbers
are approximate. Vendor feature claims are marketing claims, not inspected algorithms.

Abbreviations: **EC** = `effectcraft/crates`, **FC** = `filmcraft/crates`, **CE** =
`craft-extensions/crates`, **docs** = `craft-extensions/docs/plugins`.

---

## 1. Summary

1. **The apps are far ahead of the plugin surface.** EffectCraft's engine already offers built-in
   effects nearly everything the famous AE plugins ask a host for: other layers, other times,
   own-layer-at-time, audio samples, aux/depth channels, comp camera and light, parameters sampled at
   other times, a GPU particle backend (`EffectHost`, `EC/effects/src/lib.rs:520-585`), a wgpu
   compositor with 32 WGSL families (`EC/gpu/src/lib.rs`), point/planar/3D-camera tracking, SAM 2
   roto, puppet, AE-compatible expressions and an AE-compatible scripting object model. Plugins see
   **one premultiplied f32 buffer plus scalars** (`EC/plugins/src/effects.rs:82-125`) and **~37 session
   ops out of 435+ engine commands** (`docs/contracts/effectcraft.md` vs the `cmd!`/`query!` ids in
   `EC/engine/src`). The single largest lever is **adapter work, not app work**: wire `EffectHost`
   through to `raster.filter` and expose more engine commands.
2. **The platform lacks five primitives every pro video plugin API has:** (a) temporal/multi-input
   image requests, (b) ROI/arbitrary output bounds, (c) a GPU path (WGSL on the app's wgpu),
   (d) rich parameter types (layer, 3D point, path, curve, gradient, file, opaque data) with on-canvas
   handles for effects, and (e) persistent per-instance/simulation state or cache. OFX, the AE SDK
   and FxPlug all have (a)-(d); without them Twixtor, Neat Video, Particular, Saber, Optical Flares,
   Plexus and Sapphire transitions cannot exist as Wasm plugins at all.
3. **Speed is the second blocker.** Wasm runs on `wasmi` (interpreter, no SIMD, no threads,
   `CE/ext-host/Cargo.toml:18`). Kernel-heavy plugins are acceptable (Lightwell 478 ms, Emberfield
   176 ms per 1080p frame in the sandbox, `docs/program-2026-11-17.md:57`) but a per-pixel Wasm loop
   costs ~17 s per 1080p frame (halftone, `effectcraft/docs/plugin-testing.md:156`), and plugin effects
   are `gpu: false` (`EC/effects/src/external.rs:~105`) so they break EffectCraft's GPU chains with a
   readback/upload. Real-time needs **user WGSL compute shaders run by the host** plus a
   **kernel graph**, not just more single kernels; a JIT (Wasmtime/Winch/Cranelift) is the third leg.
4. **FilmCraft has no adapter at all** (`filmcraft/docs/gaps.md` G11: "No plugin hosting", 200-400 h),
   although its engine has the same command façade and an `FxEnv` with other frames and other
   tracks (`FC/render/src/vfx.rs:40-65`). An `filmcraft.session` + `raster.filter` + new
   `video.transition` and `audio.effect` contracts are the NLE minimum.
5. **AniCraft:** no source. The 2D-animation plugin market (Toon Boom/Moho scripts, Animate JSFL,
   Blender Grease Pencil add-ons) is almost entirely **document automation** (rigs, lip sync, onion
   skin, inbetweening, exporters), which the session model already suits; it needs stroke/vector
   drawing ops, a bone/deformer model, timeline exposure-sheet ops and audio analysis.
6. **Genuinely native-only after these changes:** vendor GPU code that cannot be expressed in WGSL
   (CUDA/Metal-specific kernels, Element 3D-class engines with their own renderers), large AI models
   (Topaz, Neat Video's profiles are fine but its CUDA path is not), and hardware/dongle licensing.
   A host `model.run` (ONNX/candle on the app's runtime) would move most AI plugins into Wasm too.

Top 10 actions (detail in section 4):

| # | Action | Layer | Pri | Effort |
|---|---|---|---|---|
| 1 | `raster.filter@1.1`: `frame.at`, `layer.at`, declared inputs, ROI/padding, output bounds | Platform + EC adapter | P0 | M |
| 2 | Pass `EffectHost` services through the EC adapter (audio, aux, comp scene, params_at) | EC adapter | P0 | S-M |
| 3 | `gpu.shader@1`: author WGSL compute passes run by the host on wgpu, with a CPU fallback contract | Platform + EC/FC | P0 | L |
| 4 | Parameter types: layer, point3, path/mask, curve, gradient, file/resource, angle-handle, data | Platform form + adapters | P0 | M |
| 5 | Kernel graph (`kernel.graph`) and kernels: flow, displacement, radial blur, FFT, LUT3D, raster lines/paths, text | Platform kernels | P1 | M-L |
| 6 | Per-instance cache and simulation state (`state.get/set`, `cache` keyed by params+time) | Platform + adapter | P1 | M |
| 7 | Expose more engine commands: expressions, mask/shape path data, text documents, precompose, parenting, import, render queue, tracker results | EC adapter | P0-P1 | M |
| 8 | Effect on-canvas handles (`effect.overlay` draw + drag events on params) | Platform UI + EC | P1 | M |
| 9 | FilmCraft adapter (session, filter, transition, audio effect, import/export) | FC adapter | P1 | L |
| 10 | Wasm JIT option (Wasmtime with fuel/epoch, SIMD128, host-side tiling) | Platform | P1 | L |

---

## 2. App inventory (what matters to plugins)

### 2.1 EffectCraft (After Effects counterpart)

| Area | What exists | Where | Exposed to plugins? |
|---|---|---|---|
| Command façade | Every action is a command id + JSON through `Session::execute`; 435+ ids (layer 78, view 39, file 36, keys 23, comp 20, camera 15, mask 14, track 13, prop 12, effect 12, renderQueue 11, roto 6, puppet 3, warp 3...) | `EC/engine/src/lib.rs`, `EC/engine/src/cmd/*.rs` | ~37 ops via `EC/plugins/src/ops.rs:179-300` |
| Keyframes | Bezier/hold/linear/auto/continuous, speed/influence, roving, labels, smooth, time-reverse, audio-to-keyframes | `EC/keyframe`, `cmd/keys_more.rs`, `cmd/key_transform.rs` | `prop.get/set/addKey/reset/ease`; no delete/move/batch keys, no labels, no temporal/spatial tangents |
| Expressions | AE-compatible JavaScript on boa: array maths, `wiggle`, `loopOut`, `valueAtTime`, noise, layer/effect references | `EC/expr` (5.9k LOC) | Not exposed: `prop.setExpression` exists (`cmd/prop.rs:807`) but is not an op; `prop.get` returns the text |
| Scripting | AE-style object model (`app.project`, `CompItem`, `Property`, `RenderQueue`, `File`, ScriptUI, sockets, `scheduleTask`) | `EC/script` (9.3k LOC) | Separate system; not a sandboxed plugin path. Old public EC v0.24.1 had docked ScriptUI (`docs/continuation-2026-11-17.md:135`) |
| Effects library | Hundreds of `EffectSpec`s incl. particles (CC Particle World-like), Card Dance/Shatter, keying (Keylight-like), OCIO, time effects, transitions, VR, warp stabilizer, Mocha shape | `EC/effects` (62k LOC) | `effect.apply/remove/reorder` with numeric params |
| Effect host services | `original`, `layer(id)`, `layer_masks`, `self_at(t, n_effects)`, `layer_at(id, t)`, `audio(id, start, frames, rate)`, `aux()` (depth, IDs, normals, Cryptomatte), `comp_scene()` (camera matrix, first light), `params_at(t)`, `param_at(i, id, t)`, `param_is_static`, `particles()` (GPU sim) | `EC/effects/src/lib.rs:520-585` | **None.** Plugin context is `{params,time,scale,origin,full,quality,seed,color}` (`EC/plugins/src/effects.rs:82-125`) |
| Effect env | masks, comp time, fps, working space/linear/float depth, shutter, `layer_span` | `EC/effects/src/lib.rs:589-617` | Only time, colour flag |
| Parameter UI | Number, Slider, Percent, Angle, Pixels, Point, **Point3, Layer, Mask, Path, File, Gradient, Hidden** | `EC/project/src/props.rs:12-48` | Platform form has Range/Int/Number/Bool/Choice/Radio/Angle/Color/Point/Text/Section only (`CE/ext-core/src/form.rs:84+`) |
| GPU | wgpu 30 compositor (Metal/Vulkan/D3D12/WebGPU), 38 blend modes, motion blur, classic 3D, advanced 3D render pipeline (PBR, IBL, shadows, DOF), GPU effect families (`fx_*.rs` + 32 WGSL files), GPU particle sim, texture pool | `EC/gpu` (26k LOC) | No. Plugin effects are `gpu: false`, forcing CPU fallback inside GPU chains |
| CPU render | Comp renderer: sources → masks → effects → transform (2D/3D) → matte → blend; adjustment layers; layer cache, disk cache | `EC/render` (30k LOC) | No render-request op (no "render comp/layer at t into my buffer") |
| 3D | Camera layers, lights, classic and advanced 3D, model layers (glTF/OBJ/STL/FBX), extruded text/shapes, materials, environment layers, Blender camera-solve bridge | `EC/render/src/three_d`, `EC/engine/src/cmd/model3d.rs`, `cmd/blender.rs` | Only `layer.setSwitch` 3D flag; no camera/light ops |
| Tracking | Point tracker (NCC + Lucas-Kanade), planar (`plane.rs`), 3D camera tracker (SfM + bundle adjustment), face tracking, stabilize, warp stabilizer, mask tracking | `EC/track` (14.8k LOC), `cmd/track.rs`, `camera.*` | No |
| Roto / AI | Roto Brush with SAM 2.1 (Hiera, wgpu), MobileSAM/MediaPipe/TFLite face, content-aware fill, inpaint, object matte | `EC/sam2`, `EC/segment`, `cmd/content_fill.rs` | No; no `model.run` |
| Masks / shapes / paths | Bezier masks with feather points, shape layers with trim/offset/booleans/varstroke, path tracing, mask interpolation | `EC/path`, `cmd/mask.rs`, `cmd/mask_interp.rs` | `layer.newShape` (rect/ellipse/star/polygon); no path vertices, no mask ops |
| Text | Shaping (harfrust), bidi, variable fonts, per-glyph outlines with char/word/line indices, animators/selectors, path text | `EC/text` | `layer.newText` only; no animators, no text-doc edits, no glyph outlines |
| Audio | Comp mixdown, 13 audio effects, `keys.audioToKeyframes`, waveform | `EC/render/src/audio.rs`, `EC/effects/src/audio_fx.rs` | No audio read, no audio effect contract |
| Puppet / warp | Puppet pins, liquify, mesh/perspective warps | `EC/effects/src/puppet`, `cmd/puppet.rs`, `cmd/liquify.rs` | No |
| Import/export | AEP read, PSD, PDF, SVG, Lottie in/out, interchange; export H.264/ProRes/HEVC/AV1/WebM/EXR/uncompressed | `EC/aep`, `EC/lottie`, `EC/export` | No import/export contract; `resource.lut`/`resource.preset` only |
| Automation | MCP server, control channel, long jobs | `EC/automation` | Parallel to plugins; plugins cannot call it |
| Jobs | `jobs.list/cancel/wait` engine jobs; platform `job.*` | `EC/engine/src/background*` | Plugin jobs yes; app jobs (tracking, roto) not observable |

### 2.2 FilmCraft (Premiere Pro counterpart)

| Area | What exists | Where |
|---|---|---|
| Command façade | Same pattern as EC: `Session::execute` with ids (`sequence.*`, `trim.*`, `edit.*`, `graphics.*`, `markers.*`, `clip.*`, `effects.*`) | `FC/engine/src/lib.rs`, `FC/engine/src/commands.rs` |
| Edit algebra | Overwrite/insert/lift/extract/ripple/roll/slip/slide/rate stretch, sync locks, multicam | `FC/edit` |
| Effects | Premiere 26 catalogue incl. Lumetri, Ultra Key, Warp Stabilizer, Auto Reframe, Echo, Posterize Time, Track Matte; effect ids are **strings** in `EffectInstance` (`FC/project/src/effect.rs:105`), so external ids fit the model | `FC/render/src/effects.rs`, `FC/render/src/vfx.rs` |
| Effect env | `FxEnv`: `frame(dt, upto)`, `source_frame(dt, scale)`, `track(index)`, `layer_to_output`, clip duration/offset, fps, timecode, file/sequence names, sizes, `clip_key` (analysis cache key) | `FC/render/src/vfx.rs:40-65` |
| Transitions | Video/audio transitions with outgoing/incoming layers | `FC/render/src/transitions*`, `FC/project/src/vtransition.rs` |
| Retime | Optical-flow interpolation (dense LK, forward splatting) | `FC/render/src/flow.rs` |
| Tracking | Mask tracking (Shi-Tomasi + pyramidal LK + RANSAC) | `FC/render/src/track.rs` |
| GPU | wgpu compositor, YUV planes in shader, 27 blend modes, fx/lut/mask/transition WGSL | `FC/gpu` (5.2k LOC) |
| Audio | Planar f32 DSP, EBU R128 loudness, biquads, FFT, GCC-PHAT sync, ducking, 5.1 | `FC/audio-dsp` (13.7k LOC), `FC/render/src/audio_fx.rs` |
| Speech / text | Whisper (candle) transcription, diarization, Kokoro TTS, captions (SRT/VTT/SCC/MCC/STL/TTML) | `FC/speech`, `FC/tts`, `FC/captions` |
| Scopes | Waveform, parade, vectorscope, histogram with numeric summaries | `FC/scopes` |
| Interchange | EDL, FCP7 XML, FCPXML, OTIO, AAF, OMF | `FC/interchange` |
| Export | Parallel batch render, H.264/HEVC/ProRes/DNx/AV1/MXF/EXR/DPX…, progress/cancel atomics | `FC/export` |
| Automation | MCP server (rmcp), bridge mode | `FC/automation` |
| Plugins | **None** (`README.md:303`, `docs/gaps.md` G11) | — |

### 2.3 AniCraft (2D animation; no source)

Treated as a Toon Boom Harmony / Moho / Adobe Animate / Blender Grease Pencil class app. Expected
engine surface a plugin ecosystem needs: vector/bitmap strokes, drawings/cels with exposure sheet
(X-sheet), symbols/nested timelines, bones and deformers (bone, curve, envelope), onion skin, camera,
lip-sync (phoneme switch layers), audio scrubbing, export to video/GIF/Lottie/sprite sheets.

---

## 3. Competitive plugin catalogue

Legend: **App** = the engine already does this internally; **Exp** = exposed to plugins today;
**Gap** = where the missing piece lives (A app, D adapter, P platform); **Wasm** = can be a Wasm
plugin after the proposed changes.

### 3.0 What the incumbent plugin APIs guarantee a plugin

| Host capability | AE SDK | OFX 1.5 (Resolve/Fusion/Nuke) | FxPlug 4 (FCP/Motion) | Craft today |
|---|---|---|---|---|
| Two-phase render, ROI/ROD | SmartFX `PF_Cmd_SMART_PRE_RENDER` + `checkout_layer`, `RETURNS_EXTRA_PIXELS` | `GetRegionOfDefinition`, `GetRegionsOfInterest`, tiles | `FxTileableEffect`, host tiles | No (one whole-layer buffer) |
| Other times / sub-frame | `checkout_layer(..., what_time)`, `PF_CHECKOUT_PARAM` at any time (re-entrant) | `TemporalClipAccess`, `GetFramesNeeded`, continuous samples | `FxTimingAPI_v4` | No (regression from public EC ABI 2) |
| Multiple inputs | `PF_Param_LAYER` | General context, any clips; Transition `SourceFrom/To` | Image params | No |
| Contexts | effect, transition (Premiere) | generator, filter, transition, paint, retimer, general | generator, filter, transition | filter only |
| Params | float, angle, color, point, point3D, popup, layer, path (mask), arbitrary data, button, groups | Double2D/3D, RGB/RGBA, Custom, Parametric (curves), StrChoice | standard + custom | range/int/number/bool/choice/angle/color/point/text |
| On-canvas UI | `PF_Cmd_EVENT` (click/drag/draw/key) on comp window | Interact / `OfxDrawSuiteV1` | `FxOnScreenControl_v4` | Tools only (`canvas.overlay.set`), not effects |
| Camera / lights | `AEGP_GetEffectCamera(Matrix)`, light suites | host-specific | — | No (EC has `comp_scene`) |
| GPU | DirectX/Metal/CUDA (`PF_Cmd_SMART_RENDER_GPU`) | Metal/CUDA/OpenCL render suites | Metal via IOSurface | No |
| Threading / cache | Multi-Frame Rendering thread-safety, Compute Cache API | `MultiThreadSuite` | stateless XPC service | per-thread pool, no cache |
| Isolation | in-process | in-process | **out-of-process XPC** | Wasm sandbox (stronger than all three) |
| Automation | AEGP suites (keyframes, masks, text, camera creation), ExtendScript, UXP | Resolve Lua/Python API | — | ~37 ops |

The Craft model is already more isolated than every incumbent; FxPlug 4 shows a host can demand
isolation and still keep a premium ecosystem, provided the host gives tiling, timing and GPU.

### 3.1 Pixel effects and generators

| Plugin | Needs from host | App | Exp | Gap | Wasm? |
|---|---|---|---|---|---|
| Trapcode Particular / Form | Comp camera + lights, layer emitters (sample another layer's pixels/alpha), text/OBJ emitters, sprites from layers, persistent sim (pre-run), animated params sampled per sub-step, GPU, motion blur, Designer window, 3D points, own render outside layer bounds | Yes (`comp_scene`, `param_at`, `particles()`, `layer`) | No (closed-form particles only, see Emberfield `examples/emberfield/src/lib.rs:37`) | D (pass services) + P (state/cache, point3, layer param, output bounds, GPU) | Yes: sim in Wasm, splat/raster in kernel or WGSL |
| Trapcode Shine / Starglow, Deep Glow | Large multi-scale blur, radial/zoom blur, 32 bpc, output beyond layer bounds, GPU | Yes | Partial (`kernel.blur`, `directional_blur`) | P (radial blur, pyramid, output bounds) | Yes (Lightwell proves it, 478 ms) |
| Trapcode Mir / 3D Stroke, Plexus | Geometry generation, 3D camera projection, anti-aliased line/triangle raster, mask/path input, layer sampling | Yes (paths, 3D) | No | D (mask/path params, camera) + P (`kernel.raster_lines`, `kernel.raster_mesh`) | Yes |
| Video Copilot Saber | Path/mask/text outline as core, glow, jitter | Yes (masks, glyph outlines) | No | D (path param, glyph outlines) + P (stroke raster kernel) | Yes |
| Video Copilot Optical Flares | 3D light positions, flare designer window, preset files, occlusion from layer luminance, on-canvas handle | Yes (lights) | No | D (lights, layer) + P (effect handles, rich window canvas) | Yes |
| Element 3D | Own OpenGL/DX renderer, model/texture file IO, AE camera/lights, scene setup window | Partly (advanced 3D, glTF/OBJ/FBX import) | No | P (GPU render pipeline, files) | Partly: a `scene.render` op on EC's adv3D renderer could replace most uses; a custom engine needs native |
| Boris FX Sapphire / Continuum | ~270 effects incl. transitions (two inputs), temporal (S_Retime, motion blur), lens flares, GPU on all hosts, preset browser, Mocha inside | Mostly | No | P (transition contract, temporal, GPU), D | Yes for most; Mocha-in-plugin needs tracking access |
| Mocha Pro / Silhouette | Planar tracking over many frames, own full UI, export tracking data to host (corner pin, shape data), roto shapes | Yes (`EC/track/plane.rs`, `effects/mocha_shape.rs`) | No | D (`track.*`, mask write) + P (frame access in jobs) | Yes as job + panel; heavy work via kernels (pyramid LK, NCC) |
| RE:Vision Twixtor / RSMB | Arbitrary-time source frames, optical flow, motion vectors, retimer context | Yes (`EC/raster/src/flow.rs`, `FC/render/src/flow.rs`) | No | P (`frame.at`, `kernel.flow`) + D | Yes with host flow kernel |
| Neat Video / DE:Noise / Red Giant Denoiser | ±N neighbour frames, noise profile analysis (job), GPU, large memory | Yes (`self_at`) | No | P (temporal, GPU, state) | Yes (temporal window + WGSL); Neat-quality CUDA tuning is native |
| Magic Bullet Looks / FilmConvert / Dehancer | LUT3D, OCIO, curves, grain, full-window look designer with live preview | Yes (OCIO, LUTs) | Partial (`resource.lut`, `kernel.curves`) | P (`kernel.lut3d`, full-window preview canvas showing host-rendered frame) | Yes |
| Heat Distortion / Displacement plugins | Displacement map from another layer, noise | Yes | No | P (`kernel.displace`, layer input) | Yes |
| Red Giant Universe transitions | Two-input transition with progress | FC yes, EC via comp | No | P (`video.transition@1`) | Yes |

### 3.2 Animation and rigging tools (panels and scripts)

| Plugin | Needs from host | App | Exp | Gap | Wasm? |
|---|---|---|---|---|---|
| Duik Angela / Rubberhose / Joysticks'n Sliders | Create nulls/shapes, parenting, expressions (write), effect controls (sliders, checkbox, point), keyframes, bone layers, undo groups | Yes (script object model, `prop.setExpression`, `layer.setParent`) | Partial (nulls, prop set/key, effects) | D (`prop.setExpression`, `layer.setParent`, shape paths, layer.copy, precompose) | Yes |
| Motion v4 / Flow / Ease Copy / Animation Composer, Motion Bro | Keyframe read/write with tangents and velocities, preset application (.ffx / preset bundles), preset browser with animated thumbnails, anchor point ops | Yes | Partial (`prop.ease`, `resource.preset`) | D (`keys.*` batch, anchor ops, `effect.applyPreset` with timing) + P (video/GIF thumbnails in UI) | Yes |
| Newton (2D physics) | Read shape geometry, simulate, bake many keyframes fast | Yes | Partial (no shape vertices) | D (path/shape read, bulk key write op) | Yes (sim in Wasm/job) |
| Explode Shape Layers / Overlord | Shape layer path data read/write, IPC with Illustrator | Yes (shape model) | No | D (shape path ops) + P (local IPC/network permission op) | Mostly; Illustrator IPC needs a native helper or `net.local` |
| BeatEdit / Sound Keys | Audio analysis (FFT, onsets), markers/keys bulk write | Yes (`keys.audioToKeyframes`, `audio()`) | No | D (`audio.read`) + P (`kernel.fft`, `kernel.onsets`) | Yes |
| Lockdown (Plugin Everything) | Surface tracking on non-rigid surfaces, mesh warp result effect | Partly (tracking, mesh warp) | No | P (temporal frames in jobs, mesh warp kernel) | Yes |
| Sure Target, Camera tools | Camera ops, 3D layer transforms | Yes | No | D (camera ops) | Yes |
| Frame.io panel | Network, OAuth, markers with comments, render upload | Partly (markers, render queue) | Partial (markers) | P (`net.fetch`, secrets) + D (render queue op) | Yes |

### 3.3 AI and analysis

| Plugin | Needs | App | Exp | Gap | Wasm? |
|---|---|---|---|---|---|
| Rotobrush 3 / Resolve Magic Mask | Segmentation model on GPU, temporal propagation, mask output | Yes (SAM 2) | No | D (`roto.*` ops) or P (`model.run`) | Via host model runtime |
| Topaz Video AI (upscale, interpolation) | Large models, GPU/NPU, output size change, frame interpolation (time), long jobs | No (no upscaler) | No | P (`model.run`, output size, temporal) | Model run in host; else native |
| Auto-captioning, transcript editing | Speech model, word timestamps, caption track write | FC yes (Whisper) | No | D (FC adapter) | Yes via host op |
| Scene detection, auto-reframe | Frame analysis jobs, sequence edits | FC yes | No | D | Yes |

### 3.4 2D animation (AniCraft class)

| Tool class (examples) | Needs |
|---|---|
| Rigging (Moho Lua rigs, Harmony rigging scripts, Mixamo-like auto rig) | Bone/deformer model read/write, drawing/stroke data, constraints |
| Lip sync (Papagayo, Rhubarb, Harmony auto lip-sync) | Audio read + phoneme detection (model or DSP), switch-layer exposure write |
| Inbetweening / onion-skin tools | Stroke correspondence, vector path read/write, X-sheet exposure ops, viewer overlays |
| Exporters (sprite sheets, Spine/DragonBones, Lottie) | Render frames, read full scene graph, write files (permissioned) |
| Brush engines | Stroke input events with pressure/tilt, raster or vector stroke write |

---

## 4. Gaps by layer

Priorities: **P0** blocks a famous plugin class entirely; **P1** blocks quality or speed; **P2**
needed for parity; **P3** nice to have. Effort: **S** < 1 week, **M** 1-3 weeks, **L** > 3 weeks.

### 4.1 App gaps

| ID | Gap | Pri | Effort | Proposal | Wasm |
|---|---|---|---|---|---|
| A1 | EC: external effects cannot declare GPU support or run in a GPU chain; a plugin effect forces readback/upload | P1 | M | Add `ExternalRender::render_gpu(ctx, &wgpu::Texture)` path and `EffectSpec.gpu = true` when the plugin ships a validated WGSL shader (see P3) | n/a |
| A2 | EC: effect parameter-ui for plugin effects cannot express Layer/Point3/Path/Mask/Gradient/File | P0 | S | Map new platform `ParamKind`s onto existing `ParamUi` variants (`EC/project/src/props.rs:12-48`) | n/a |
| A3 | EC: no on-canvas handle protocol for effects beyond Point | P1 | M | Effect overlay callback in viewer (`viewer.rs`): plugin draws ops, host routes drags to params | n/a |
| A4 | EC: no render-request API for "render layer/comp X at time t to buffer" outside the effect stack (used by panels, exporters, look designers) | P1 | M | `Renderer::render_layer_at` behind a bounded op | n/a |
| A5 | FC: no external-effect registry like `EC/effects/src/external.rs` | P0 | M | Port the `ExternalRender` pattern; effect ids are already strings | n/a |
| A6 | FC: transitions are a closed set | P1 | M | External transition registry with `(outgoing, incoming, progress)` | n/a |
| A7 | FC/EC: no external audio-effect slot | P2 | M | `AudioEffect` trait impl that forwards blocks to a plugin (planar f32, pre-roll aware: `FC/render/src/audio_fx.rs`) | n/a |
| A8 | EC: no import/export plug-in points (format registry is closed) | P2 | M | Importer/exporter registries in `EC/media`, `EC/export` | n/a |
| A9 | AniCraft: needs a command façade like EC/FC from day one | P1 | L | Same `Session::execute` + adapter pattern | n/a |

### 4.2 Adapter gaps (EffectCraft adapter `EC/plugins`, future FilmCraft adapter)

| ID | Gap | Pri | Effort | Proposal | Wasm |
|---|---|---|---|---|---|
| D1 | `EffectHost` services not passed to filters | P0 | M | Implement platform `frame.*` ops (P1) in `EC/plugins/src/effects.rs` by calling `ctx.env.host`: `self_at`, `layer_at`, `layer`, `audio`, `aux`, `comp_scene`, `params_at`, `param_at` | Yes |
| D2 | Context lacks masks, fps, shutter, layer span, working space name, float depth, adjustment flag | P0 | S | Add `{fps, compTime, shutter:{angle,phase,samples}, span:[a,b], masks:[{path,mode,feather}], adjustment, depth}` to the filter context | Yes |
| D3 | `prop.set` refuses strings and >4-vectors, so text, path, gradient, file props are unreachable (`EC/plugins/src/ops.rs:204`) | P0 | S-M | Typed setters: `text.setDocument`, `path.get/set` (vertices, in/out tangents, closed), `gradient.set`, `resource` handles for files | Yes |
| D4 | No expression ops | P0 | S | `prop.setExpression {layer,path,expression,enabled}`, `expr.errors` (engine has both) | Yes |
| D5 | Missing structural ops: parenting, precompose, duplicate, delete layer, in/out/start, time remap, blend mode, track matte, camera/light creation, masks, shape groups, effect copy | P0 | M | Map directly to engine ids `layer.setParent`, `layer.trackMatte`, `layer.setBlendMode`, `mask.new/setVertex`, `layer.create`, `layer.enableTimeRemap`... with captured-scope checks | Yes |
| D6 | Keyframe ops are minimal | P1 | S | `keys.list/delete/move/setTangents/setLabel/batch` with spatial tangents and roving | Yes |
| D7 | Tracking and roto results unreachable | P1 | M | `track.motion/status/apply`, `camera.analyze/points`, `roto.span`, as jobs observable by plugins | Yes |
| D8 | No render/queue ops | P2 | S | `renderQueue.add/list/render` with output path permission | Yes |
| D9 | No audio read | P1 | S | `audio.read {layer|comp, start, frames, rate}` → planar f32 in guest memory (bounded) | Yes |
| D10 | `layer.tree` hides string values (redaction) and depth limit | P2 | S | Keep redaction for file paths; return text, font, path data | Yes |
| D11 | Events lack keyframe/selection granularity (`document.changed` is coalesced) | P2 | S | Payload with changed property paths and key times; `prop.selection.changed` | Yes |
| D12 | FilmCraft has no adapter | P0 (for NLE) | L | `filmcraft.session@1`: `sequence.info`, `clips.list`, `clip.insert/overwrite/trim/razor`, `markers.*`, `captions.*`, `transcript.get`, `effects.apply`, `export.start`; `raster.filter` on clip effects using `FxEnv` | Yes |
| D13 | Panel docking in EC: `docs/program-2026-11-17.md` P4 reports dock panels and OS windows as source-complete, but nothing has been clicked through; the generated contract still says "beta: a floating panel" | P2 | S | Verify in a click-through and regenerate `contracts/effectcraft.md` | n/a |

### 4.3 Platform gaps (`craft-extensions`)

| ID | Gap | Pri | Effort | Proposal (contract, ops, shapes) | Wasm |
|---|---|---|---|---|---|
| P1 | No temporal/multi-input image access in filters | P0 | M | `raster.filter@1.1`. Manifest `inputs = [{id:"src2", kind:"layer-param"}]`, `temporal = {max_frames: 16}`. Ops callable from `craft_filter`: `frame.at {time, upto?, scale?} → {ptr,width,height,origin}`, `layer.at {param, time?, source:"source"|"masks"|"effects"}`, sub-frame times allowed (AE `checkout_layer` and OFX continuous samples do this; the host must handle re-entrant renders and refuse recursion), both write into guest memory, bounded by `MAX_TRANSFORM_MEMORY`, charged fuel per pixel. Optional pre-declared `craft_frames(ctx) → [times]` export so the host can prefetch in parallel (OFX `getFramesNeeded`) | Yes |
| P2 | No ROI / output-bounds negotiation | P0 | M | Optional export `craft_bounds(ctx) → {out:[x0,y0,x1,y1], in:{src:[...]} }` (AE SmartFX pre-render / OFX `getRegionOfDefinition`/`getRegionsOfInterest`); host allocates the larger output (glows, drop shadows, particles off-layer) and passes only the requested input rect; `craft_filter_tile` for host-side tiling | Yes |
| P3 | No GPU path | P0 | L | `gpu.shader@1`: package ships `.wgsl` compute entry points, declares bindings (`input`, `output`, extra textures, uniform block from params). Host validates with naga, enforces limits (dispatch size, loop bound via timeout/watchdog, no storage outside bindings), runs on the app's wgpu device; the Wasm part computes uniforms and chooses passes (`gpu.dispatch {shader, entry, groups, uniforms, textures}` inside a `gpu.graph`). CPU fallback: naga→ host interpreter is not viable; require a Wasm fallback or kernel-graph equivalent for headless/no-GPU | Yes (WGSL is sandboxable: no pointers, bounds-checked by naga/wgpu robustness) |
| P4 | Kernel set too narrow for video | P1 | M-L | Add: `radial_blur`/`zoom_blur`, `pyramid` (down/up), `displace`, `mesh_warp`, `perspective`, `lut3d` (tetrahedral), `fft`/`fft_convolve`, `median`, `flow` (dense LK, as FC/EC already have), `warp_by_flow`, `raster_lines`, `raster_path` (fill/stroke with joins/dash), `raster_mesh` (triangles with UV, z-buffer), `text_shape` (host fonts → glyph outlines), `distance_transform`, `connected_components`, `ncc_match` (tracker), `onsets`/`spectrum` (audio) | Yes |
| P5 | One kernel = one host call; no fusion | P1 | M | `kernel.graph {nodes:[{id,op,args,inputs}], output}`: host executes tiled and multithreaded, or on GPU where every node has a WGSL twin; one fuel charge per graph | Yes |
| P6 | No per-instance state, sim cache or analysis cache | P1 | M | `cache.get/put {key, bytes}` scoped to plugin+effect instance+param hash, LRU in host RAM/disk (EC has `analysis_cache.rs`, `file_cache.rs`); `state` saved in project for analysis results (Neat profile, Mocha track) via `project.data` on the effect instance (`target: {effect: uid}`) | Yes |
| P7 | Form types missing | P0 | M | `ParamKind` additions: `Point3`, `Layer {sources}`, `Mask`, `Path`, `Curve {channels}` (parametric curve, OFX `ParametricParam`), `Gradient`, `File {resource kinds}` (resolved by host to a read handle, never a path), `Data` (opaque animatable blob, hidden), `Button` (fires `effect.action` event), `Group`/twirl, `Angle` with handle | Yes |
| P8 | No effect overlay/handles | P1 | M | `effect.overlay` export `craft_overlay(ctx) → [draw ops]` in comp space plus `overlay` events (`down/drag/up` on handle ids) that set params in one undo step (OFX Interact, AE `PF_Cmd_EVENT`, FxPlug on-screen controls) | Yes |
| P9 | No transition / generator / retimer contexts | P1 | S-M | Contract kinds: `video.transition@1` (`from`, `to`, `progress`), `video.generator@1` (no input, own bounds), `video.retimer@1` (source time mapping + `frame.at`) mirroring OFX contexts | Yes |
| P10 | No audio contract | P2 | M | `audio.effect@1`: `craft_audio(ctx, planar f32 ptrs, frames)`, declared latency and tail, block-invariant pre-roll semantics as in EC/FC mixers; `audio.analyze` kernels | Yes (DSP is cheap; Wasm interpreter adequate for many effects, JIT for reverbs) |
| P11 | No model runtime | P2 | L | `model.run@1 {model: resource id, inputs: [tensor handles]}` on host candle/ONNX (EC sam2 and FC Whisper already run candle/wgpu); models are signed resources with size caps; permission `compute.model` | Yes (weights as data, inference in host) |
| P12 | Network missing (`net.fetch` not implemented: `docs/program-2026-11-17.md:79`) | P1 | M | `net.fetch {url, method, headers, body}` gated by `network:<host>`; secrets store | Yes |
| P13 | Engine speed | P1 | L | Option A: Wasmtime (Cranelift/Winch) with fuel or epoch interruption and SIMD128; keep wasmi for tiny event handlers. Option B: wasmi + host tiling (`craft_filter_tile`) across 8+ threads | Yes |
| P14 | UI: no full-window canvas with a host-rendered frame, no video/GIF thumbnails, no curve/gradient widgets | P2 | M | `viewer` node showing a host frame (`{source:"effect-preview", instance}`), `curve-editor`, `gradient-editor`, `image-sequence` for animated thumbnails | Yes |
| P15 | Calls between plugins absent | P3 | M | Needed for preset browsers driving other vendors' effects; keep deferred | — |
| P16 | Native pixel transport (JSON lines, 1 MiB) | P2 | M | Shared-memory blob handles for helpers (ecosystem doc §2) so the few genuinely native plugins can still be effects | Native |

---

## 5. Performance

### 5.1 Current numbers

| Measurement | Value | Source |
|---|---|---|
| Halftone (per-pixel Wasm loop), 1080p | ~17 s/frame, single thread | `effectcraft/docs/plugin-testing.md:156` |
| Lightwell (kernels: blur, directional blur, blend), 1080p | 478 ms | `docs/program-2026-11-17.md:57` |
| Emberfield (closed-form particles + `kernel.splat` + blur), 1080p | 176 ms | same |
| Softskin (PhotoCraft) 1080p | 602 ms | same |
| Fuel | 4000/pixel filter budget, 2e9/event, 2e12/job; 20M slice | `CE/ext-host/src/wasm.rs:44-48` |
| Memory | 512 MiB filter, 64 MiB session, 1 GiB app kernel budget | `CE/ext-host/src/wasm.rs:11-17` |

A 1080p f32 RGBA frame is 33 MB. At 24 fps real-time preview needs < 40 ms/frame for the whole
stack, so even Lightwell is ~12x too slow for interactive playback, though fine for RAM preview
and final render.

### 5.2 What makes sandboxed plugins fast

1. **WGSL shaders on the app's wgpu device (P3).** Biggest win: EC already runs its own effects as
   WGSL compute (`EC/gpu/src/fx_*.rs`) and keeps textures resident across a chain. A plugin shader
   in the same chain avoids two 33 MB transfers per frame and runs at GPU speed. WGSL is a safe
   target: naga validates it, wgpu applies robust buffer access; the remaining risk (long-running
   shaders hanging the GPU) is handled by dispatch-size caps and a watchdog. This is the Resolve
   DCTL / Fusion Fuse / ISF model.
2. **Kernel graphs (P5).** Fuse chains like `threshold → pyramid blur → screen` into one host call,
   tiled and SIMD/rayon on CPU, or mapped to the same WGSL twins on GPU. Removes per-call copies and
   fuel overhead.
3. **Temporal and analysis caches (P6).** Twixtor, Neat, RSMB and trackers re-read the same frames;
   a host frame cache keyed by `(layer, time, upto, scale)` plus the plugin `cache.put` for flow
   fields or noise profiles turns O(frames x window) into O(frames).
4. **ROI and tiling (P2).** Render only the visible viewer region at preview scale; `craft_filter_tile`
   lets the host run one instance per tile across threads (wasmi is single-threaded per instance;
   the per-render-thread pool already exists in `EC/plugins/src/effects.rs` `POOL`).
5. **JIT (P13).** Wasmtime/Cranelift typically runs scalar code close to native, versus an
   interpreter roughly an order of magnitude slower (ecosystem research §4.2 flags this as an
   inference to benchmark). Fuel metering costs some throughput; epoch interruption is cheaper.
   SIMD128 in the module gives another 2-4x on pixel loops. Determinism holds if NaN
   canonicalisation is on and relaxed SIMD is off.
6. **Preview quality signals.** Pass `quality: draft`, scale and ROI so plugins can drop sample
   counts; keep `final` for render queue.
7. **Multi-frame rendering.** Filters are already pure per frame; declare `thread_safe = true` and
   let EC/FC export render frames in parallel instances (FC export already renders one frame per
   core: `FC/export/src/lib.rs`).

Projected costs (inference, to benchmark): a WGSL glow at 1080p on Apple M-class GPU is a few ms;
kernel-graph Lightwell on CPU could drop from 478 ms to ~100 ms by removing copies and fusing
blend passes; a JIT halftone from 17 s to ~1 s, SIMD to ~0.3 s.

---

### 5.3 Shader-format lesson

ISF (GLSL + JSON header with typed `INPUTS` and passes; VDMX, Millumin, ISF4AE) and Resolve DCTL
(one C-like source compiled to Metal/CUDA/OpenCL) are the closest precedents for `gpu.shader@1`.
AE's removal of Pixel Bender in CS6 left projects with missing effects: the host-owned shader format
must be versioned like any contract, with the Wasm/kernel fallback kept so projects still render
if a shader feature is retired. No wgpu/WGSL plugin precedent was found, so this would be novel.

### 5.4 What still needs native after all of the above

| Case | Why | Mitigation |
|---|---|---|
| Vendor CUDA/Metal code (Neat Video CUDA path, Element 3D engine) | Not expressible in WGSL limits or relies on vendor APIs | Native helper with shared-memory pixel transport (P16), effects allowed for restricted native |
| Large AI models (Topaz, custom roto) | Weights + GPU inference | `model.run` host runtime (P11); native only for proprietary runtimes |
| Licensing/dongles, cross-app IPC (Overlord ↔ Illustrator) | OS access | Native helper, or `net.local` permission for loopback IPC |

---

## 6. Sources

Local (read, not executed): `craft-extensions/docs/plugins/contracts/effectcraft.md`,
`compute-and-jobs.md`, `ui.md`, `research/gap-audit-2026-11-17.md`,
`research/ecosystem-research-2026-11-17.md`, `program-2026-11-17.md`, `continuation-2026-11-17.md`;
`effectcraft/docs/plugin-testing.md`; `filmcraft/docs/gaps.md`, `filmcraft/README.md`,
`filmcraft/ROADMAP.md`; source files cited inline.

Web (fetched 2026-11-18 by a research sub-agent; items it marked as from memory are excluded):

- AE SDK SmartFX: https://ae-plugins.docsforadobe.dev/smartfx/smartfx/
- AE interaction callbacks (`PF_CHECKOUT_PARAM`): https://ae-plugins.docsforadobe.dev/effect-details/interaction-callback-functions/
- AE parameters: https://ae-plugins.docsforadobe.dev/effect-basics/parameters/
- AE custom UI events: https://ae-plugins.docsforadobe.dev/effect-ui-events/effect-ui-events/
- AE camera/light access: https://ae-plugins.docsforadobe.dev/effect-details/accessing-camera-light-information/
- AE GPU build: https://ae-plugins.docsforadobe.dev/intro/gpu-build-instructions/
- AE Multi-Frame Rendering: https://ae-plugins.docsforadobe.dev/effect-details/multi-frame-rendering-in-ae/
- AEGP and suites: https://ae-plugins.docsforadobe.dev/aegps/aegps/ , https://ae-plugins.docsforadobe.dev/aegps/aegp-suites/
- Artisans: https://ae-plugins.docsforadobe.dev/artisans/artisans/
- Premiere UXP changelog: https://developer.adobe.com/premiere-pro/uxp/changelog ; https://blog.developer.adobe.com/en/publish/2025/12/uxp-arrives-in-premiere-a-new-era-for-plugin-development
- Premiere GPU effects: https://premiere-plugin-sdk-guide.readthedocs.io/gpu-effects-transitions/PrGPUFilter-function-table.html ; https://ppro-plugins.docsforadobe.dev/gpu-effects-transitions/cuda-opencl-metal-opengl/
- OFX contexts: https://openfx.readthedocs.io/en/main/Reference/ofxImageEffectContexts.html
- OFX clips/temporal: https://openfx.readthedocs.io/en/main/Reference/ofxImageClip.html ; https://openfx.readthedocs.io/en/main/Reference/ofxClipPreferences.html
- OFX parameters: https://openfx.readthedocs.io/en/main/Reference/ofxParameter.html
- OFX 1.5 release notes: https://openfx.readthedocs.io/en/main/ReleaseNotes/relnotes-1.5.html
- Natron OFX guide: https://github.com/NatronGitHub/Natron/wiki/OpenFX-plugin-programming-guide-(Advanced-issues)
- Fusion Fuse SDK: https://documents.blackmagicdesign.com/UserManuals/Fusion_Fuse_SDK.pdf ; DCTL: https://wiki.dvresolve.com/developer-docs/dctl ; Resolve scripting: https://resolvedevdoc.readthedocs.io/en/latest/API_intro.html
- FxPlug 4 out-of-process: https://developer.apple.com/documentation/professional-video-applications/using-out-of-process-fxplug-plug-ins.md ; tiling/Metal: https://developer.apple.com/documentation/professional-video-applications/rendering-in-fxplug.md ; timing: https://developer.apple.com/documentation/professional_video_applications/fxtimingapi_v4.md ; on-screen controls: https://developer.apple.com/documentation/professional_video_applications/fxonscreencontrol_v4.md
- Toon Boom Harmony scripting: https://docs.toonboom.com/help/harmony-24/premium/scripting/about-scripting.html
- Moho scripting: https://www.lostmarble.com/moho/manual/scriptsmenu.html ; https://mohoscripting.com/script_structure
- Blender Grease Pencil v3 migration: https://developer.blender.org/docs/release_notes/4.3/grease_pencil_migration/
- Rhubarb Lip Sync: https://github.com/wzpan/rhubarb-lip-sync
- Trapcode: https://maxon.net/red-giant/trapcode
- Element 3D: https://www.creativebloq.com/features/element-3d-what-it-is-and-how-to-use-it
- Mocha: https://borisfx.com/blog/mocha-ae-vs-mocha-pro-what-are-the-differences/ ; https://borisfx.com/documentation/mocha/
- Sapphire retime: https://borisfx.com/blog/best-twixtor-alternative-in-2024/
- RE:Vision (Twixtor, RSMB): https://www.toolfarm.com/?p=31901
- Neat Video: https://www.neatvideo.com/features ; https://www.neatvideo.com/support/known-issues-old
- Deep Glow: https://aescripts.com/deep-glow ; Plexus: https://aescripts.com/plexus/
- Newton: https://www.provideocoalition.com/newton_a_2d_physics_engine_for_after_effects/
- Duik: https://rxlaboratorio.org/rx-tool/duik/ ; Overlord: https://battleaxe.co/overlord-v1
- Explode Shape Layers: https://aescripts.com/explode-shape-layers/ ; Lockdown: https://aescripts.com/lockdown/
- BeatEdit: https://aescripts.com/beatedit-for-premiere-pro/
- Frame.io V4 panel: https://help.frame.io/articles/12833113-adobe-premiere-frame-io-v4-panel-overview-25-6-and-later
- Object Matte / Roto Brush: https://helpx.adobe.com/after-effects/desktop/roto-brush-and-refine-matte/roto-brush/object-matte.html
- Topaz plugins: https://docs.topazlabs.com/topaz-video/plugins
- Resolve 19 (Magic Mask, Speed Warp): https://documents.blackmagicdesign.com/SupportNotes/DaVinci_Resolve_19_New_Features_Guide.pdf
- Dehancer: https://dehancer.com/shop/davinci_resolve/pro
- Pricing (reseller, weak): https://www.toolfarm.com/?p=110925
- Web Audio Modules: https://www.github.com/webaudiomodules/api
- Figma plugin sandbox: https://www.figma.com/blog/an-update-on-plugin-security/
- ISF spec: https://docs.isf.video/ref_json
- Natron Shadertoy node: https://natron.readthedocs.io/en/rb-2.5/plugins/net.sf.openfx.Shadertoy.html

Not found: a reliable best-seller ranking for AE plugins (aescripts blocked automated fetches); any
Wasm-hosted OFX precedent; any wgpu/WGSL plugin precedent.

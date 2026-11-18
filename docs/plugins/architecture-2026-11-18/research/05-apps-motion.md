# 05: Motion, video and animation apps: final plugin architecture inputs

Date: 2026-11-18. Researcher 5 of 8. Read-only pass: source and docs reading in the three app
repos and `craft-extensions`, plus web research. Nothing was built, run, edited or pushed. No
storytold repository was contacted (the AniCraft clone was already on disk).

Scope and revisions read:

| Repo | Path | Branch | Head |
|---|---|---|---|
| EffectCraft | `effectcraft` | `plugin-beta` | `02064025` (2026-11-18, "Adapter on the platform's new APIs") |
| FilmCraft | `filmcraft` | `main` (upstream) | `0b3f30ed` (2026-11-18) |
| AniCraft | `anicraft` | `main` (upstream) | `ab48665` (2026-11-17), v0.3.0 |
| Platform | `craft-extensions` | `plugin-beta` | `24cd2ff` |

Abbreviations: **EC** = `effectcraft/crates`, **FC** = `filmcraft/crates`, **AC** = `anicraft`
(root), **CE** = `craft-extensions/crates`. Command counts are regex counts of registry
constructors over non-test sources (method stated per app); they are measurements of source text,
not of a running registry. Line numbers are approximate. Vendor feature descriptions are product
claims, not inspected algorithms. This builds on
[`capability-program-2026-11-18.md`](../../research/capability-program-2026-11-18.md)
and [`capability-2026-11-18/motion.md`](../../research/capability-2026-11-18/motion.md)
and does not repeat their AE SDK/OFX/FxPlug comparison or performance benchmarks.

---

## 1. Summary

### 1.1 Corrections to the earlier pass

1. **AniCraft has source, and already has a plugin system.** The earlier pass treated AniCraft as
   an empty clone with no command façade. Upstream main is a 358-file Rust workspace (about 117k
   lines in crates plus 55k in the egui shell) with an Animate-class document model, about
   **365 registry commands**, an MCP server, a live control port, and its **own sandboxed Wasm
   plugin system** (`AC/crates/plugins`, ABI v1, four kinds: command, frame-script, exporter,
   ease; `AC/docs/plugins.md`, ADR 0017 and 0088). Under the
   [suite replacement rule](../../suite-replacement-rule.md)
   that is a competing per-app plugin system that must be migrated into the shared platform and
   then removed. It is also the best-designed small plugin host in the suite and carries four
   lessons the platform lacks (section 2.3.4).
2. **EffectCraft's registry is about 691 commands, not 435+.** 616 `cmd!` (journaled, i.e. may
   edit) and 75 `query!` ids across 54 namespaces (`EC/engine/src/cmd.rs` macros at ~124-158).
   The adapter exposes **35 op names** (`EC/plugins/src/ops.rs:177-300`), i.e. about 5% of the
   surface.
3. **FilmCraft's registry is about 619 commands** (`cmd!`/`query!` in `FC/engine/src`, 39 tagged
   `query!`), and its project format **silently drops unknown data**: no `plugin_data` maps, no
   `#[serde(flatten)]` extras in `FC/project/src`, and an unknown effect id renders as a silent
   pass-through (`FC/render/src/effects.rs:508`, `_ => {}`). Both violate the replacement rule's
   "silent pass-through rendering … discarded state" clause the moment FilmCraft gains plugins.
4. **EffectCraft plugin panels still float.** `EC/ui-egui/src/plugins_ui.rs:118-140` draws every
   plugin surface through `renderer.window(...)`, although EC's dock (craft-ui's shared docking
   model, `EC/ui-egui/src/dock.rs:1-45`) already has a precedent variant `PanelKind::ScriptPanel(u32)`
   for docked ScriptUI panels. A `PanelKind::Plugin(SurfaceKey)` is a small, concrete change.
5. **There are now three different "plugin-like" systems in EffectCraft alone**: the platform
   adapter, the AE-compatible `script` crate with ScriptUI panels and its own file/network
   preference gates (`EC/script/src/lib.rs:33,48`), and the MCP automation with its own file
   confinement and a hand-written "commands that run code" list (`EC/automation/src/confine.rs:1-15`).
   Their permission vocabularies differ. One registry metadata scheme should serve all three.

### 1.2 What matters most

1. **Registry metadata is the keystone.** All three apps already route every user action through
   one `CommandSpec` registry with a free-text params doc. None of them records *side effects* in
   the spec: EC/FC have `journal: bool`, AniCraft has `mutates: bool`. AniCraft fills the gap with a
   namespace allowlist and a test that fails on any new namespace (`AC/crates/engine/src/cmds_plugin.rs:33-72`);
   EC's MCP fills it with a path-confinement layer. Adding a typed `effects` bitset, a scope and a
   machine schema to `CommandSpec` in each app makes the program's registry bridge (`app.run`)
   generable and checkable in CI. This is app work, P0, S-M per app.
2. **Every pixel-producing extension point in all three apps is a closed `match`.** EC GPU effect
   dispatch (`EC/gpu/src/effects.rs:210-240`), FC's `FxOp` GPU enum and `effects::apply` string
   match (`FC/render/src/gpufx.rs:34`, `effects.rs:118-508`), FC transitions (`FC/render/src/transitions/*`),
   AniCraft's seven Animate filters (`AC/crates/doc/src/filter.rs:149`) and its closed `Tool` enum
   (`AC/src/app.rs:14-40`). EC alone has an external effect registry (`EC/effects/src/external.rs`);
   FC and AC need the same pattern for effects, transitions, filters, tools, importers and
   exporters.
3. **The apps already have the hard host services plugins need.** EC's `EffectHost`
   (`EC/effects/src/lib.rs:520-585`) and FC's `FxEnv` (`FC/render/src/vfx.rs:42-75`) both offer
   other frames, other layers/tracks and analysis cache keys; AniCraft's `evaluate(doc, frame)` is
   pure and its exporter ABI already pulls evaluated display lists per frame with cancel. The
   adapter must pass these through; the platform must define `raster.filter@2` (temporal, multi
   input, ROI/output bounds), `video.transition@1`, `video.generator@1`, `gpu.shader@1`,
   `audio.effect@1` and `format.export@1` once, for all three.
4. **GPU texture sharing is feasible in all three** because all three use wgpu 30 on the window's
   device (EC `wgpu 30.0.1`, FC `wgpu 30`, AC `wgpu 30` from eframe 0.36). EC keeps effect chains
   resident as `GBuf` textures; AC's stage renders to `Raster::Native` GPU images
   (`AC/crates/engine/src/render_service.rs:167-176`); FC evaluates `FxOp`s with a CPU reference.
   A host-owned WGSL wrapper (naga-validated) slots into each as one more arm.
5. **Determinism and portability are already first-class in AniCraft** (pure `evaluate`, CPU
   oracle, eases sampled into the document so a movie plays the same without the plugin) and
   partially in EC (missing-effect refusal plus explicit `RenderWithout`). The platform should
   adopt AniCraft's "bake on apply" rule as a general pattern for any plugin function the render
   path depends on, and EC's missing-effect contract as the suite-wide rule.

Top actions (detail in section 4):

| # | Action | Layer | Pri | Effort | Depends on |
|---|---|---|---|---|---|
| 1 | Typed `CommandSpec` metadata (`effects`, `scope`, schema) in EC, FC, AC + generated classification table + CI check | App ×3, Platform check | P0 | S-M each | none |
| 2 | AniCraft adapter on `Runtime<A>`; ABI v1 bridge with removal criteria; migrate command/exporter/ease/frame-script kinds | Adapter (new), Platform | P0 | L | 1 |
| 3 | FilmCraft adapter (`filmcraft.session@1`, `raster.filter@2`, `video.transition@1`) + `plugin_data` + unknown-data preservation + missing-effect refusal | App + Adapter (new) | P0 | L | 1, 6 |
| 4 | EC: pass `EffectHost` through `raster.filter@2`; dock plugin panels; typed setters; `app.run` bridge | Adapter + App | P0 | M | 1, 6 |
| 5 | `gpu.shader@1` with arms in EC `gpu/effects.rs`, FC `FxOp::External`, AC filter passes | Platform + App ×3 | P1 | L | 6 |
| 6 | `raster.filter@2`, `video.transition/generator/retimer@1`, rich `ParamKind`s, `effect.overlay` | Platform | P0 | M-L | none |
| 7 | External registries: FC effects/transitions/audio, AC filters/tools/importers, EC importers/exporters/transitions | App | P1 | M each | 6 |
| 8 | Project plugin lock (id, version, sha256, contracts) + farm install-from-lock + headless grant policy | Platform | P1 | M | 2-4 |
| 9 | `audio.read`, `audio.effect@1` on FC `AudioEffect` and EC audio fx | Platform + App | P1 | M | 1 |
| 10 | Animation contracts from AniCraft: `timeline.ease@1` (baked), `runtime.script@1` (player), `rig.controller@1`, `input.filter@1`, `brush.dab@1` | Platform + AC | P1-P2 | M each | 2 |

---

## 2. Per-app inventory

### 2.1 EffectCraft (After Effects class)

| Area | What exists (cited) | Plugin reach today |
|---|---|---|
| **Command registry** | `CommandSpec { id, label, menu, shortcut, alt_shortcuts, params: &str, enabled: fn, run: fn, journal: bool }` (`EC/engine/src/cmd.rs`). Defined by `cmd!` (journal true) and `query!` (journal false, always enabled) macros. **~691 ids** (616 `cmd!`, 75 `query!`); largest namespaces `layer` 121, `file` 66, `view` 48, `keys` 37, `comp` 31, `edit` 27, `essential` 22, `track` 22, `camera` 20, `renderQueue` 20. 86 modules under `EC/engine/src/cmd/`. | 35 adapter op names (`EC/plugins/src/ops.rs:177-300`), of which 19 pass straight to `Session::execute` after scope checks (lines 189-215) |
| **Schema** | Free text params doc parsed at run time for accepted keys (`EC/engine/src/cmd/schema.rs`, `accepted_params`). Unknown-key refusal is heuristic, not a JSON Schema. | Adapter caps args at 32 fields / 64 KiB |
| **Undo** | Journaled commands record history; `edit.*` history ops; replay journal (`EC/engine/src/replay.rs`, `history.rs`) | Adapter wraps events in one undo group |
| **Side effects outside the document** | `file` (66: open/save/import/export/render queue outputs, "Execute File"), `prefs`, `shortcuts`, `storage`, `mediaBrowser`, `help` (opens URLs), `script`/`scriptui` (runs JS with prefs-gated file/network), `jobs`, `cache`, `learn`. **No flag records this**; MCP keeps a separate hand list (`EC/automation/src/confine.rs`) | Adapter refuses by not exposing them |
| **MCP / CLI / scripting** | MCP server (`EC/automation`: `command_list`, `command_run`, `command_batch`, `doc_inspect`, `get_comp`, `get_property`, `add_effect`, `add_keyframe`, `history`, `screenshot`, `control`…), live control port 9877, `effectcraft-cli` (`apps/effectcraft-cli`, `render --without-missing-effects`), AE-compatible JS scripting with ScriptUI, sockets and File/Folder (`EC/script`, 9.3k lines, boa 0.22) | Plugins cannot call any of these |
| **Document model** | AE model: project items, comps, layers (AV, text, shape, camera, light, null, 3D model), property groups with keyframes and expressions (`EC/project`, `EC/keyframe`). `Project.plugin_data` and `Layer.plugin_data` maps added on this branch (commit `9d63cb7f`) | `project.data.get/set` |
| **Render pipeline** | CPU comp renderer (`EC/render`, 30k lines): source → masks → effects → transform → matte → blend; layer/input caches keyed by hashed effect stacks (`EC/render/src/cache.rs:684-695`); disk cache with file identities (`disk_cache.rs`) | Plugin effects run inside it as `ExternalRender` |
| **GPU** | wgpu 30.0.1 compositor and effect families (`EC/gpu`, 26k lines; `fx_*.rs`, `shaders/`), adv3D renderer, particle sim. GPU effect dispatch is a closed `match id` (`EC/gpu/src/effects.rs:210-240`) gated by `catalog::gpu_supported` (`EC/effects/src/catalog.rs:97`). Effects pass `GBuf` textures through a chain | External specs are `gpu: false` (`EC/effects/src/external.rs:~105`), so a plugin in a GPU chain forces readback and upload |
| **Effects registry** | ~308 `ec.*` effect ids (17 transitions, 44 generate/sim, 13 audio). `EffectSpec { id, name, category, params, render, gpu, float }` (`EC/effects/src/lib.rs:656`). External registry with leak cap 4096, `register`/`unregister`, pass-through + status badge, global generation bump on change (`EC/effects/src/external.rs`) | `raster.filter@1` contributions |
| **Effect host services** | `EffectHost`: `original`, `layer`, `layer_masks`, `self_at`, `layer_at`, `audio`, `aux`, `comp_scene`, `params_at`, `param_at`, `param_is_static`, `particles` (`EC/effects/src/lib.rs:520-585`); `EffectEnv`: masks, comp time, fps, effect index, working space, linear, float depth, shutter, layer span (589-617) | Plugin context JSON is `{params, time, scale, origin, full, quality, seed, color}` (`EC/plugins/src/effects.rs:82-127`) |
| **Timeline/keys/expressions** | Bezier/hold/roving keys, labels, keys tools; AE expressions on boa (`EC/expr`, 5.9k lines); `expr.*` (2), `anim.*` (8) | `prop.get/set/addKey/reset/ease`; numeric only (`ops.rs:~203` refuses text/file values) |
| **Audio** | Comp mixdown, 13 audio effects (`EC/effects/src/audio_fx.rs`), audio-to-keyframes | None |
| **Tracking / AI** | Point/planar/3D camera tracker, face tracking, warp stabiliser (`EC/track`, 14.8k), SAM 2 roto (`EC/sam2`), segmentation (`EC/segment`), content-aware fill, object matte (`roto` 14, `track` 22, `face` 5, `objectMatte` 5 commands) | None |
| **Import/export** | AEP (`EC/aep`, 37k), PSD, PDF, SVG, Lottie in/out, interchange; export pipeline with rayon frame parallelism (`EC/export/src/pipeline.rs:211-228`) | `resource.lut`, `resource.preset` only |
| **UI** | egui 0.37 (storytold fork patch), craft-ui shared docking (`EC/ui-egui/src/dock.rs`), `PanelKind` enum with `ScriptPanel(u32)`; platform renderer via `craft_ext_ui` (`plugins_ui.rs`), tool overlays (`panels/viewer.rs:310`) | Panels and windows float; tool overlays only, no effect overlays |

### 2.2 FilmCraft (Premiere Pro class)

| Area | What exists (cited) | Plugin reach |
|---|---|---|
| **Command registry** | `CommandSpec { id, label, menu, shortcut, params, enabled, run, journal, named: NamedKeys }` (`FC/engine/src/commands.rs:23-42`); `named` = keys that let a caller target items instead of the selection, derived from the params doc and pinned by a test. `cmd!`/`query!` macros (80-88). **~619 ids**: `graphics` 61, `file` 57, `clip` 57, `sequence` 52, `timeline` 41, `project` 33, `markers` 33, `trim` 24, `captions` 23, `mixer` 20, `export` 19, `multicam` 17, `effects` 16, `transcript` 14 | **No adapter** (`FC/README.md:305` "No plugins"; `docs/gaps.md` G11, 200-400 h, "needs an owner decision (FFI under the unsafe rule)") |
| **MCP / CLI** | MCP server (`FC/automation`: `command_list`, `command_run`, `command_batch`, `doc_inspect`, `project_inspect`, `render_frame`, `render_preview`, `media_import`, `ui_click`), bridge mode, long jobs; `filmcraft-cli render` (`apps/filmcraft-cli/src/main.rs:335`) | n/a |
| **Document** | Project → sequences → tracks → `TrackItem`s; `EffectInstance { effect: String, enabled, params: BTreeMap, masks, post_fader, essential, layer }` (`FC/project/src/effect.rs:105`); transitions are `Transition { id, effect: EffectInstance, start, duration, from, to, align }` (`FC/project/src/track.rs:268`), so **external effect and transition ids already fit the data model** | n/a |
| **Effect environment** | `FxEnv`: `frame(dt, upto)`, `source_frame(dt, scale)`, `track(index)`, `layer_to_output`, clip seconds/offset, fps, timecodes, names, sizes, `clip_key` (`FC/render/src/vfx.rs:42-75`); temporal effects skip each other (`is_temporal`) | n/a |
| **Render** | Pure `render_sequence(project, seq, t, opts, sources)` (`FC/render/src/lib.rs:116`); effects: `vfx::apply` → `gpufx::FxOp::eval` → string `match` (`effects.rs:118-508`); export renders frames in parallel (`FC/export/src/run.rs:190`, `job.rs:121`) | n/a |
| **GPU** | wgpu 30 (`FC/gpu`, 5.9k lines): YUV planes, blend modes, fx/LUT/mask/transition WGSL; a closed `FxOp` enum with a CPU reference (`FC/render/src/gpufx.rs:34`, `GPU_EFFECTS` at 225) | n/a |
| **Audio** | `AudioEffect` trait with per-sample param smoothing and reset (`FC/audio-dsp/src/lib.rs:123`), 14.9k lines DSP, loudness, mixer with track/clip inserts and post-fader flag | n/a |
| **AI** | Whisper transcription, diarisation (`FC/speech`), Kokoro TTS (`FC/tts`), scene detection, colour match | n/a |
| **Interchange** | EDL, FCP7 XML, FCPXML, OTIO, AAF/OMF (`FC/interchange`, 15.7k) | n/a |
| **Side effects** | `file` 57, `export` 19, `media` (import, relink), `prefs`, `mediaCache`, `fonts`, `tts`/`speech` (model download over network: `FC/speech/Cargo.toml` pulls an HTTP client), native file dialogs (`FC/ui-egui/src/native_dialogs.rs`). No side-effect flag in `CommandSpec` | n/a |
| **UI** | egui 0.37 (same storytold fork rev as EC), craft-ui docking (`FC/ui-egui/src/dock.rs`) | n/a |

### 2.3 AniCraft (Adobe Animate class)

#### 2.3.1 Shape of the codebase

| Crate (layer) | Lines | Role |
|---|---|---|
| `geom` (L0) | 8.8k | Vec2/Affine, paths, flattening, planar merge (merge drawing), brush sweeps |
| `plugins` (L0) | 2.0k | wasmi sandbox, ABI v1, manifest, registry |
| `audio` (L0) | 1.3k | WAV/Ogg/MP3 decode, pure mix/resample, peaks |
| `actions` (L0) | 5.2k | Timeline action IR, ActionScript 3 subset compiler, `FrameScript` (`actions/src/lib.rs:632`) |
| `doc` (L1) | 23.6k | Document, `evaluate`, easing, armature/IK/springs, skins/weights, warp, camera, filters, player |
| `render` (L2) | 9.5k | CPU oracle rasteriser (analytic coverage, band-parallel), filters, HTML canvas, Lottie, SVG |
| `engine` (L3) | 49.7k | `Session`, history, command registry, render service, publish |
| `gpu` (L3) | 4.7k | wgpu stage renderer: binning, fine raster, filters (`bin.wgsl`, `fine.wgsl`, `filters.wgsl`) |
| `cli` (L4) | 2.2k | `anicraft-cli`: info/render/gif/export/commands/run/exec/serve/mcp; control port |
| `xfl`, `svg`, `text`, `xml`, `guide` | ~10k | FLA/XFL import, SVG, text shaping, bundled guide |
| shell `src/` (L5) | 54.6k | egui 0.36 shell, own dock (ADR 0075), tools, panels, plugin dialogs |

Principles that matter to plugins (`AC/docs/architecture.md` section 1): the document is data and
time is a function (`evaluate(doc, frame) -> DisplayList`); one deterministic CPU oracle that
GPU output must match; everything is a command; `unsafe_code = "forbid"` workspace-wide.

#### 2.3.2 Surfaces

| Area | What exists (cited) |
|---|---|
| **Command registry** | `CommandSpec { id, label, menu, shortcut, params: &str, mutates: bool, run }` (`AC/crates/engine/src/lib.rs:~107-125`), assembled from 49 `specs()` functions (`commands()`, ~128-178). **~365 ids** by regex over `c("…")` and `id: "…"` constructors: `file` 32, `layer` 26, `timeline` 23, `modify` 18, `motion` 16, `element` 15, `ease` 14, `view` 14, `bone` 14, `draw` 14, `filter` 13, `raw` 11 (raw path points), `swatch` 11, `edit` 10, `text` 10, `library` 10, `scene` 9, `frames` 9, `macro` 8, `warp` 6, `script` 5, `sound` 5, `plugin` 5, `camera` 4. In the subset whose `mutates` flag a regex could read (288), 178 mutate. Params are prose docs with `key?` optional markers, which ADR 0097 already treats as a contract |
| **Undo** | Persistent `Arc`-shared snapshots with a memory budget (ADR 0001); gestures coalesce; `execute_nested` runs a plugin's or macro's commands atomically with no history of their own; `revert_gesture` for live preview |
| **Confirmation** | Irreversible actions require `"confirm": true` (ADR 0004); plugins cannot confirm |
| **MCP / CLI / control** | `anicraft-cli mcp` with five tools (`anicraft_commands`, `anicraft_run`, `anicraft_run_script`, `anicraft_document`, `anicraft_render`; `AC/docs/mcp.md`); live control port with a token file and gesture-aware queuing (ADR 0104); JSON-lines `serve`; saved command macros (ADR 0097) |
| **Document model** | Scenes → layers (Normal, Guide, Mask, Folder, Camera, Armature) → keyframes (start, duration, elements, tween, sound, script) → elements (Shape, Symbol instance with blend/filters/events, Text, Camera); library symbols (Graphic, MovieClip, Button), bitmaps, sounds. Armatures with IK, springs, shape skins and weights; asset warp; motion tweens with property keys and Motion Editor curves; shape tweens with hints |
| **Render** | CPU oracle (analytic signed-area coverage, f32 premultiplied, 32-row bands in rayon, bit-identical to serial). `RenderService` worker with LRU `(state, frame, view)` cache; pluggable `Rasterizer` returning bytes or `Raster::Native` (`AC/crates/engine/src/render_service.rs:167-176`) |
| **GPU** | wgpu 30 via eframe 0.36's egui-wgpu (one wgpu build), compute binning and fine raster on the window's device, no readback for the stage; filters as compute passes mirroring the oracle (`AC/crates/gpu/src/filters.wgsl`); headless device fallback; glow (OpenGL) and CPU fallbacks |
| **Filters** | Closed `Filter` enum: Blur, DropShadow, Glow, Bevel, GradientGlow, GradientBevel, AdjustColor (`AC/crates/doc/src/filter.rs:149`); filter presets in prefs; plugins may call `filter.*` except copy/paste/preset files |
| **Timeline / easing** | Classic, shape and motion tweens; `Easing` incl. `CubicBezier`, `Custom`, and `Sampled(Arc<SampledEase>)` for plugin eases (`AC/crates/doc/src/easing.rs:188`) |
| **Interactivity** | Pure deterministic player (`player::step`), action IR, ActionScript 3 subset (tiers 1-2.5), Test Movie, HTML canvas export with a JS port of the player |
| **Audio** | Event/Start/Stop/Stream sounds laid out by `doc::sound::voices`, pure `audio::mix`; envelopes; no audio effect slots, no analysis beyond waveform peaks; no lip sync (the word appears only for Stream sync, `AC/crates/doc/src/sound.rs:161`) |
| **Import/export** | FLA/XFL, SVG import; PNG, PNG sequence, GIF, APNG, WebP, SVG frames, sprite sheet, Lottie (`craft-lottie`), HTML canvas, audio, video via the user's ffmpeg (`file.export_*`, 32 `file` commands); plugin exporters (`file.export_plugin`) |
| **Tools** | Closed `Tool` enum: Select, Subselect, Transform, Warp, GradientTransform, Width, Bone, Bind, Pen, Text, Pencil, Brush, PaintBrush, Eraser, Rect, Oval, Star, Line, PaintBucket, InkBottle, Eyedropper, Camera, Hand, Zoom (`AC/src/app.rs:14-40`) |
| **UI** | egui/eframe **0.36** (platform `ext-ui` is on 0.37), own dock model (`AC/src/dock.rs`, ADR 0075) with a closed `Panel` enum of 13 kinds (lines 19-34); command palette from the registry; web build (wasm32, WebGL2, CPU stage) |

#### 2.3.3 The existing plugin system (`AC/crates/plugins`, `AC/docs/plugins.md`)

| Aspect | AniCraft ABI v1 |
|---|---|
| Runtime | wasmi 2.0 with `validate`, `stable`, `portable-dispatch` (`AC/crates/plugins/Cargo.toml`); same engine choice as CE (`CE/ext-host/Cargo.toml:18`) |
| Imports | Only `anicraft.call(ptr,len)` and `anicraft.response(ptr,cap)`; JSON requests `{"op": "command"|"document"|"elements"|"log"}`; host calls implemented as resumable traps |
| Kinds | `command` (registry calls, one undo step, live preview dialog from manifest params), `frame-script` (player actions only, 20M instructions per frame shared, no document access), `exporter` (pulls evaluated display list per frame, returns bytes, host writes the file, 64 MiB cap, cancel), `ease` (sampled 257 times at load, table stored in the document) |
| Policy | `plugin_may_call` (`AC/crates/engine/src/cmds_plugin.rs:33-72`): document namespaces allowed; `file.*`, `plugin.*`, `prefs.*`, `macro.*`, `desktop.*`, `control.*`, undo/redo, clipboard, brush/filter preset files, publish settings writes and most `view.*` refused; unknown namespaces refused and a test fails when one appears; confirmations refused |
| Limits | 16 MiB module, 256 MiB memory, 1e9 fuel and 5 s per run, 100k requests, 1 MiB/8 MiB request/response, 64 MiB total, fresh instance per run (`runtime.rs:47`) |
| Lifecycle | Session registry, folder `<settings>/plugins` or `ANICRAFT_PLUGINS`; `plugin.install/list/run/remove/reload`; CLI loads plugins only with `--plugins` |
| Missing (self-declared "Not in v1") | Panels/UI plugins, background runs with progress for commands, persistent state, binary document view, direct path-point edits by data, signing, catalogue, per-plugin permissions |

#### 2.3.4 Lessons the platform should adopt from AniCraft

1. **Namespace allowlist with a failing test for new namespaces** is the cheapest honest version of
   the program's classified registry bridge. Generalise it to per-command classification from
   typed metadata (section 4.1, A-ALL-1).
2. **Nested execution as the transaction**: a plugin's registry calls run through
   `execute_nested`, each checked and atomic, the whole run one undo step labelled with the
   plugin name, any failure restoring the pre-run document; live preview via a reverted gesture.
   This is exactly what `Runtime<A>`'s transaction state machine must guarantee for every app.
3. **Bake on apply**: eases are sampled at load and stored in the document, so documents open and
   play the same without the plugin and a plugin update never changes a finished movie. Apply this
   to any plugin output the render path depends on that can be tabulated (eases, curves, LUTs,
   gradients, deformation tables, phoneme tracks).
4. **Exporters see evaluated frames, not the document**, pulled one frame at a time with cancel
   checks at each request. That is the right default for `format.export@1` in timeline apps: the
   plugin needs no document-read permission and memory stays bounded.
5. **Interactive runtime scripts are separate from editing plugins**: frame scripts steer a pure,
   budgeted player and can never touch the document; budgets are shared per frame. A future
   `runtime.script@1` contract (Rive/Lottie-interactivity class) should keep this separation.
6. **Headless repeatability by explicit load**: the CLI loads plugins only when asked.

### 2.4 Cross-app comparison

| Capability | EffectCraft | FilmCraft | AniCraft | Platform today (CE) |
|---|---|---|---|---|
| Registry size | ~691 | ~619 | ~365 | n/a |
| Edit flag | `journal` | `journal` | `mutates` | n/a |
| Side-effect metadata | none (MCP hand list) | none | none (namespace policy) | `Permission` per op |
| Machine schema | params prose, heuristic key parse | params prose, `named` keys | params prose, `key?` markers | form `ParamKind` (11 variants, `CE/ext-core/src/form.rs:84-127`) |
| Plugin adapter | yes, 35 ops | none | own ABI v1 (not the platform) | `AppApi` |
| External effect registry | yes (`external.rs`) | no (string ids fit) | no (closed filters) | `raster.filter@1` |
| Transitions | effect-style (17 ids) | `Transition` with `EffectInstance` | n/a (tweens) | none |
| Audio effect slot | built-in only | `AudioEffect` trait | none | none |
| GPU effect dispatch | closed match | closed `FxOp` | closed filter passes | none |
| Frame/layer access | `EffectHost` | `FxEnv` | `evaluate` per frame | none |
| Plugin data in document | `plugin_data` (project, layer) | none, unknown fields dropped | eases baked, frame-script refs kept | `project.data.*` |
| Missing plugin behaviour | pass-through + badge; final render refuses unless `RenderWithout` | silent pass-through | ease keeps table; frame script skipped and reported in HTML export (`AC/crates/render/src/html.rs:102`) | `ProjectPrompt` |
| egui | 0.37 fork | 0.37 fork | 0.36 | 0.37 |
| Docking | craft-ui | craft-ui | own (ADR 0075) | none |
| Plugin panels | floating windows | n/a | none | UI 1.0 renderer |

---

## 3. Plugin catalogue

Legend. **Host capability** is the precise thing a host must offer. **Status** gives the app engine
(EC/FC/AC: ✓ has the capability internally, ~ partial, ✗ none) and the plugin reach today (R: ✓
reachable through the adapter, ✗ not). **Gap** names the layer: **A** app, **D** adapter, **P**
platform. **Proposal** is the op, contract or hook that closes it. "Native" means the item stays on
a native helper after the proposal. Contract names follow the capability program (section 2.4
there); new names are introduced in section 4.4 here.

### 3.1 Generators, particles and 3D

| # | Plugin (host) | What it does | Host capability required | Status | Gap | Proposal |
|---|---|---|---|---|---|---|
| 1 | Trapcode Particular (AE) [maxon.net/red-giant/trapcode](https://www.maxon.net/en/red-giant/trapcode) | 3D particle sim with layer/text emitters, sprites, physics, designer | comp camera + lights; sample other layers' pixels at time t; persistent sim with pre-run; params sampled per sub-step; output beyond layer bounds; GPU | EC ✓ (`comp_scene`, `layer_at`, `param_at`, `particles()`); R ✗ | D, P | `raster.filter@2` `frame.at/layer.at/params.at`, `scene.camera`, `cache.*` sim state, output bounds; splat via `gpu.shader@1` |
| 2 | Trapcode Form | particle grids/meshes displaced by layers and audio | as 1 + audio spectrum | EC ✓; R ✗ | D, P | as 1 + `audio.read`, `kernel.fft` |
| 3 | Stardust (Superluminal) [superluminal.tv](https://superluminal.tv/stardust) | node-based particles and 3D models | as 1 + model files + node editor window | EC ~; R ✗ | P | as 1 + `resource.model`, full-window canvas surface |
| 4 | Plexus 3 [aescripts.com/plexus](https://aescripts.com/plexus/) | lines/triangles between points from layers, masks, OBJ | read mask paths, 3D camera, anti-aliased line/triangle raster | EC ✓; R ✗ | D, P | `path.get`, `kernel.raster_lines/raster_mesh` or WGSL |
| 5 | Element 3D (Video Copilot) | real-time 3D object renderer inside AE | own GPU render pipeline, model/texture IO, camera/lights | EC ~ (adv3D renderer, glTF/OBJ/FBX) | P, A | `scene.render` op over EC adv3D; custom engines Native |
| 6 | Trapcode Mir / 3D Stroke | 3D surfaces, path strokes in 3D | mask paths, 3D camera, mesh raster | EC ✓; R ✗ | D, P | `path.get`, `scene.camera`, `kernel.raster_mesh` |
| 7 | Saber (Video Copilot) [videocopilot.net/blog/2016/03/new-plug-in-saber-now-available-free](https://www.videocopilot.net/blog/2016/03/new-plug-in-saber-now-available-free/) | energy beams along masks/text | mask/text outlines as input, glow, jitter | EC ✓ (glyph outlines); R ✗ | D, P | `path.get`, `text.outlines`, stroke kernel |
| 8 | Optical Flares (Video Copilot) | lens flares with designer window | light positions, occlusion from layer luminance, on-canvas handle, preset files | EC ✓; R ✗ | D, P | `scene.lights`, `layer.at`, `effect.overlay`, `resource.preset` |
| 9 | Red Giant Universe generators [maxon.net/red-giant/universe](https://www.maxon.net/en/red-giant/universe) | generators, text FX, transitions across AE/Premiere/FCP/Resolve | generator context (no input, own bounds), transition context | EC ~, FC ~; R ✗ | P, A | `video.generator@1`, `video.transition@1` |
| 10 | Boris Continuum / Particle Illusion [borisfx.com/products/continuum](https://borisfx.com/products/continuum/) | 250+ effects incl. particle emitter library | multi-host, GPU, preset browser, emitters | EC ✓; R ✗ | P | `raster.filter@2`, `gpu.shader@1`, preset browser panel |
| 11 | Krokodove (Fusion) [nmbr73.github.io/Kernfusion/…/Krokodove](https://nmbr73.github.io/Kernfusion/Plugins/Krokodove/Krokodove/) | 100+ Fusion tools: generators, distortions, shape tools | node-graph tool contract with typed ports | EC/FC ✗ (layer stacks, no node graph) | P | `graph.node@1` (EC flowchart is view-only) or map to effect stack |
| 12 | Fusion Fuses (Lua), e.g. ShaderToy Fuses [github.com/nmbr73/Shadertoys](https://github.com/nmbr73/Shadertoys) | scripted tools, GPU kernels via DCTL-like code | script-defined tool with OpenCL/Metal kernels | n/a | P | `gpu.shader@1` (WGSL) is the analogue; port by hand |
| 13 | DCTL (Resolve) [wiki.dvresolve.com/developer-docs/dctl](https://wiki.dvresolve.com/developer-docs/dctl) | per-pixel colour/effect kernels compiled for CUDA/Metal/OpenCL | single-source kernel, host compile, params from header | FC ✗ | P, A | `gpu.shader@1` with per-pixel `fn` body; FC `FxOp::External` |
| 14 | ISF shaders / Natron Shadertoy [docs.isf.video](https://docs.isf.video/ref_json) | GLSL fragment shaders with JSON inputs | shader + typed inputs + multi-pass | none | P | `gpu.shader@1` importer for ISF metadata (shader code ported to WGSL) |
| 15 | Nuke BlinkScript [learn.foundry.com/nuke/…/blinkscript.html](https://learn.foundry.com/nuke/content/reference_guide/other_nodes/blinkscript.html) | user kernels run on CPU or GPU, publishable as gizmos | one source for CPU and GPU, kernel params, random access to inputs | none | P | `kernel.program` (host IR to CPU/WGSL) + `gpu.shader@1`; "publish as preset" = data package |

### 3.2 Glow, light, distortion and stylise

| # | Plugin | What it does | Host capability required | Status | Gap | Proposal |
|---|---|---|---|---|---|---|
| 16 | Deep Glow 2 [aescripts.com/deep-glow](https://aescripts.com/deep-glow) | physically based multi-scale glow | output beyond layer bounds, pyramid blur, 32 bpc, GPU | EC ✓ (glow on GPU); R ~ (`kernel.blur`) | P | output bounds in `raster.filter@2`; `kernel.pyramid`; WGSL |
| 17 | Trapcode Shine / Starglow | volumetric light rays | radial/zoom blur from a point, on-canvas point | EC ✓; R ✗ | P | `kernel.radial_blur`, `effect.overlay` |
| 18 | Sapphire S_Glow / S_LensFlare [borisfx.com/products/sapphire](https://borisfx.com/products/sapphire/) | glows, flares with flare designer | as 16 + preset files + designer window | EC ✓; R ✗ | P | as 16 + full-window canvas showing host frame |
| 19 | Video Copilot Twitch / Heat Distortion | glitch, displacement | displacement map from a layer, time-varying noise, other-time frames | EC ✓; R ✗ | D, P | `layer.at`, `kernel.displace`, `frame.at` |
| 20 | Pixel Sorter (aescripts) | sorts pixel runs by luminance | arbitrary per-pixel loops (not a kernel) | EC n/a; R ~ (17 s/frame in wasmi) | P | Wasm JIT tier, or WGSL compute with workgroup sort |
| 21 | BCC Displacement / Lens Distortion | lens and mesh distortion | mesh warp, ST map input | EC ✓ | P | `kernel.mesh_warp`, ST-map input |
| 22 | Glitchify / Datamosh tools (aescripts) | codec-style smearing | motion vectors / optical flow, previous output frame | EC ✓ (flow) | P | `kernel.flow`, `frame.at(prev, own output)` with recursion guard |

### 3.3 Keying, tracking, roto, retiming, restoration

| # | Plugin | What it does | Host capability required | Status | Gap | Proposal |
|---|---|---|---|---|---|---|
| 23 | Keylight / Primatte / IBK | chroma and difference keying with clean plates | second input layer (clean plate), screen colour picker on viewer | EC ✓ (Keylight-like), FC ✓ (Ultra Key) | P | layer param input; `viewer.pick` event |
| 24 | Mocha Pro [borisfx.com/products/mocha-pro](https://borisfx.com/products/mocha-pro/) | planar tracking, roto, remove, stabilise; own UI | many-frame access in a job, write corner pins/masks to host, full window | EC ✓ (`EC/track/plane.rs`, `mocha_shape.rs`); R ✗ | D, P | `job` with `frame.at` streaming, `mask.set`, `track.apply` ops via `app.run` |
| 25 | Silhouette (Boris FX) | roto and paint | spline masks read/write with feather, per-frame paint | EC ✓ (masks, paint) | D | `mask.*` and `paint.*` through `app.run` with typed path setters |
| 26 | Lockdown (aescripts) [aescripts.com/lockdown](https://aescripts.com/lockdown/) | surface tracking with mesh warp | dense tracking over frames, mesh warp render | EC ~ | P | job frame access; `kernel.mesh_warp` |
| 27 | CoreMelt SliceX (FxPlug) [coremelt.com](https://coremelt.com/) | planar-tracked masks in FCP | tracking in plugin, on-screen controls | FC ~ (mask tracking) | A, P | `effect.overlay`, FC adapter |
| 28 | mTracker 3D (MotionVFX, FxPlug) [motionvfx.com](https://www.motionvfx.com/) | 3D camera tracking in FCP | camera solve, write camera to host | EC ✓ (3D camera tracker); FC ✗ | A, D | EC `camera.*` via `app.run`; FC has no 3D camera |
| 29 | Twixtor (RE:Vision) | optical-flow retiming | arbitrary-time source frames, retimer context | EC ✓, FC ✓ (`FC/render/src/flow.rs`) | P | `video.retimer@1` + `frame.at` + `kernel.flow` |
| 30 | RSMB (RE:Vision) | motion-vector blur | flow between neighbours, shutter info | EC ✓ | P, D | `frame.at`, shutter in context (EC `EffectEnv.shutter`) |
| 31 | Sapphire S_Retime / Resolve Speed Warp | retime with flow or AI | as 29, plus model inference | FC ✓ flow | P | as 29 + `model.run` |
| 32 | Neat Video [neatvideo.com](https://www.neatvideo.com/features) | temporal denoise with noise profiles | ±N frames, profile analysis stored per instance, GPU | EC ✓ (`self_at`); R ✗ | P | `frame.at` window, `state` per effect instance (project data on effect), WGSL; CUDA path Native |
| 33 | DE:Noise / DE:Flicker (RE:Vision) | denoise, deflicker | temporal neighbours, analysis over clip | EC ✓ | P | as 32; analysis job + `cache.put` |
| 34 | Topaz Video AI plugin [docs.topazlabs.com/topaz-video/plugins](https://docs.topazlabs.com/topaz-video/plugins) | upscale, interpolate, enhance | large models on GPU/NPU, output size change, frame interpolation | ✗ (no upscaler) | P | `model.run@1`, output size in `raster.filter@2`, `video.retimer@1`; proprietary runtime Native |
| 35 | Kognat Rotobot [kognat.com](https://kognat.com/) | AI segmentation mattes (OFX/AE) | segmentation model, mask output | EC ✓ (SAM 2) | P, D | `model.run` or expose `roto.*`/`objectMatte.*` jobs |
| 36 | Depth Scanner (aescripts) | AI depth maps | depth model, depth channel output to other effects | EC ~ (aux depth channels) | P, D | `model.run`; `aux.depth` read in `raster.filter@2` |
| 37 | Cryptomatte (Psyop) [github.com/Psyop/Cryptomatte](https://github.com/Psyop/Cryptomatte) | ID mattes from EXR | multichannel EXR read, ID channel access, viewer picking | EC ✓ (aux Cryptomatte, `crypto_select.rs`) | D, P | `aux.ids` in filter context; `viewer.pick` |

### 3.4 Colour, looks and transitions

| # | Plugin | What it does | Host capability required | Status | Gap | Proposal |
|---|---|---|---|---|---|---|
| 38 | Magic Bullet Looks [maxon.net/red-giant/magic-bullet](https://www.maxon.net/en/red-giant/magic-bullet-suite) | look designer window | full-window editor showing host-rendered frame, LUT3D, curves, grain | EC ✓ (OCIO/LUT), FC ✓ (Lumetri) | P, A | `viewer` node with host frame, `kernel.lut3d`, `render.frame` op |
| 39 | FilmConvert Nitrate [filmconvert.com](https://www.filmconvert.com/) | film emulation per camera profile | camera metadata, LUT, grain plates | FC ✓ metadata | D, P | `clip.metadata` read; `resource.lut` |
| 40 | Dehancer [dehancer.com](https://dehancer.com/shop/davinci_resolve/pro) | film print emulation, halation | multi-scale blur, LUT, GPU | ✓ | P | WGSL + kernels |
| 41 | Colourlab AI [colourlab.ai](https://colourlab.ai/) | AI shot matching, look transfer to NLE | read many clips' frames, write grade params on many clips | FC ✓ colour match | D | `clip.frame`, `effects.apply` on many clips through `app.run`; job |
| 42 | Color Finale 2 (FxPlug) [colorfinale.com](https://colorfinale.com/) | layer-based grading in FCP | scopes data, curves widgets, LUT export | FC ✓ scopes | P | `scopes.read`, curve/gradient widgets in UI 1.0 |
| 43 | FilmImpact transitions (Premiere) [filmimpact.net](https://www.filmimpact.net/) | GPU transitions | outgoing/incoming frames, progress, handles | FC ✓ data model; R ✗ | A, P | `video.transition@1`; FC external transition registry |
| 44 | Sapphire / Universe transitions | effect-based transitions | as 43, multi-frame access | FC ✓, EC ~ (17 `ec.transition.*` as layer effects) | A, P | as 43; EC maps transition to effect with `completion` param |
| 45 | MotionVFX mTransition packs (FxPlug/Motion templates) | template transitions | template (preset) with exposed params | FC ✓ (graphic templates) | P | data-tier `resource.template` (MOGRT-like) |

### 3.5 Text, titling, motion presets and animation tools

| # | Plugin | What it does | Host capability required | Status | Gap | Proposal |
|---|---|---|---|---|---|---|
| 46 | Animation Composer (Mister Horse) [misterhorse.com](https://misterhorse.com/animation-composer) | preset browser with animated thumbnails, one-click apply | apply presets with timing to selection, video/GIF thumbnails in panel | EC ✓ (`anim_presets.rs`); R ~ (`resource.preset`) | D, P | `effect.applyPreset {at, duration}`; animated thumbnails in UI 1.0 |
| 47 | Motion Bro [motionbro.com](https://motionbro.com/) | asset/preset manager, imports AEP packs | import project files, place items, network downloads | EC ✓ (AEP import) | D, P | `format.import@1`, `net.fetch`, `files.read` via picker handle |
| 48 | Flow / Ease and Wizz [aescripts.com/flow](https://aescripts.com/flow/) | curve editor panel for keyframe ease | read/write tangents and velocities on selected keys, docked panel | EC ✓ (`keys` 37); R ~ (`prop.ease`) | D, P | `keys.list/setTangents/batch` via `app.run`; curve widget; docked panel |
| 49 | Motion 4 (Mt. Mograph) | align, anchor, motion tools | anchor point ops, parenting, nulls, keyframe batch | EC ✓; R ✗ | D | `app.run` for `layer.*`, `keys.*` |
| 50 | Typemonkey / TextEvo (aescripts) | kinetic typography | text documents, per-character animators, camera moves | EC ✓ (`EC/text`, `text_anim.rs`); R ✗ (strings refused) | D | `text.setDocument`, `text.animator.*` typed ops |
| 51 | Essential Graphics / MOGRT (Premiere/AE) | templates with exposed controls | template package, editable params in NLE | FC ✓ (`graphics` 61, `graphic_templates`); EC ✓ (`essential` 22) | P | data-tier `resource.template` shared EC→FC |
| 52 | MotionVFX mTitle (FCP) | title packs | generator/title context with text params | FC ✓ graphics | P | `video.generator@1` with text param kinds |
| 53 | KBar (aescripts) [aescripts.com/kbar](https://aescripts.com/kbar/) | toolbar running menu commands, scripts, expressions | invoke any menu command by id; dockable bar | EC ✓ registry | D, P | `app.run` + `command.invoke` of UI commands with class `view`; docked panel |
| 54 | FX Console (Video Copilot) | search-and-apply effects, frame snapshots | list effects incl. third party, apply, grab viewer frame | EC ✓ | D, P | `effect.list`, `effect.apply`, `render.frame` |
| 55 | Ajar "Convert animation to frame interval" (Animate JSFL) [ajarproductions.com](https://ajarproductions.com/blog/flash-extensions/) | converts tweens/frames to 2s/3s | read keyframes and tweens, insert/clear keyframes, convert tween to frames | AC ✓ (`timeline.*`, `frames.*`); R ✓ via ABI v1 | D (migration) | AC adapter `app.run` |
| 56 | Ajar "Roughen Edges" (Animate JSFL) | roughens strokes/fills | read/write raw path points | AC ✓ (`raw.*` 11, `path.*`); R ~ (paths read, writes by command) | D, P | `path.get/set` typed vector op (AC "not in v1") |
| 57 | Animate HTML (CEP) extension panels [helpx.adobe.com/animate/using/creating-html-extensions.html](https://helpx.adobe.com/animate/using/creating-html-extensions.html) | panels that run JSFL | docked panel, document events, command calls | AC ✗ panels | A, D, P | AC `Panel::Plugin(key)` + UI 1.0 + events |

### 3.6 Rigging, lip sync, physics and 2D animation

| # | Plugin | What it does | Host capability required | Status | Gap | Proposal |
|---|---|---|---|---|---|---|
| 58 | Duik Angela [rxlaboratorio.org/rx-tool/duik](https://rxlaboratorio.org/rx-tool/duik/) | IK rigs, controllers, automation | create nulls/shapes, parent, write expressions, effect controls, undo groups | EC ✓ (`prop.setExpression`, `layer.setParent`); R ~ | D | `app.run` for `layer.*`, `expr.*`; `prop.setExpression` typed op |
| 59 | RubberHose 3 / Limber (Battle Axe, aescripts) [battleaxe.co](https://battleaxe.co/) | bendy limbs from shape paths | write shape path data per frame via expressions, controllers | EC ✓; R ✗ | D | typed `path.set`, expressions |
| 60 | Joysticks 'n Sliders (aescripts) | pose interpolation driven by a joystick controller | on-canvas controller, keyframe pose library | EC ✓ | D, P | `effect.overlay` or `rig.controller@1`; `keys.batch` |
| 61 | Newton 4 (Motion Boutique) [motionboutique.com](https://www.motionboutique.com/newton/) | 2D rigid body physics baked to keys | read shape geometry, simulate, write thousands of keys quickly | EC ✓; R ~ (no shape vertices) | D | `path.get`, `keys.batch` (bulk) |
| 62 | Explode Shape Layers / Overlord [battleaxe.co/overlord-v1](https://battleaxe.co/overlord-v1) | split shapes; Illustrator↔AE transfer | shape path read/write; IPC with another app | EC ✓ | D, P | `path.*`; cross-app via suite clipboard/transfer, else Native |
| 63 | Toon Boom Harmony scripts / OpenHarmony [cfourney.github.io/OpenHarmony](https://cfourney.github.io/OpenHarmony/), [github.com/mchaptel/TBScripts](https://github.com/mchaptel/TBScripts) | DOM over nodes, drawings, timing; batch rig tools | node/drawing/exposure read/write, packages folder | AC ✓ (layers, symbols, bones); R ✓ ABI v1 commands | D | AC adapter + `app.run` |
| 64 | Harmony Master Controller (scripted widgets) [docs.toonboom.com/…/scripting](https://docs.toonboom.com/help/harmony-24/premium/scripting/about-scripting.html) | on-stage sliders/points that drive poses | scripted on-canvas widgets bound to poses/params, evaluated live | AC ✗ | A, P | `rig.controller@1`: overlay handles + baked pose mapping |
| 65 | Moho Lua tools (e.g. Make Bones) [github.com/SimplSam/moho-make-bones](https://github.com/SimplSam/moho-make-bones) | create/mirror bones, joint circles | bone add/parent/limits, tool events with pointer input | AC ✓ (`bone.*` 14); R ✓ commands, ✗ tools | A, P | tool contributions on AC stage (`tool@1` events), `bone.*` via `app.run` |
| 66 | Moho lip-sync scripts / Papagayo [lostmarble.com](https://www.lostmarble.com/forum/viewtopic.php?p=202486) | switch layers driven by phoneme data | audio read, phoneme timing import, switch-layer exposure write | AC ~ (symbols with first frame) | A, D, P | `audio.read`, job; `instance.set_first_frame` over frames; `resource.phonemes` |
| 67 | Rhubarb Lip Sync [github.com/DanielSWolf/rhubarb-lip-sync](https://github.com/DanielSWolf/rhubarb-lip-sync) | automatic mouth shapes from audio | audio samples, long CPU job, write timing | AC ✓ sounds; ✗ access | D, P | `audio.read` + `job` (Wasm JIT for the recogniser), bake to keyframes |
| 68 | Live2D Cubism (SDK, AE plugin) [live2d.com/en/sdk](https://www.live2d.com/en/sdk/about/) | deformer-based 2.5D puppets | mesh deformers with parameter blending, physics, runtime export | AC ~ (skins/weights, warp, springs) | A, P | `rig.deformer`: baked deformation tables; exporter via `format.export@1` |
| 69 | Spine CLI and export scripts [esotericsoftware.com/spine-command-line-interface](https://esotericsoftware.com/spine-command-line-interface) | skeletal export to JSON/binary, atlas packing | read bones, slots, skins, weights, keys; pack images | AC ✓ armatures | D, P | AC exporter `format.export@1` with document (not display list) access + `rig.read` |
| 70 | Blender Grease Pencil Tools [extensions.blender.org/add-ons/grease-pencil-tools](https://extensions.blender.org/add-ons/grease-pencil-tools/) | box deform, straighten stroke, timeline scrub | stroke point read/write, viewport gestures | AC ✓ (`raw.*`, brush strokes) | D, P | `path.get/set`, tool events |
| 71 | GP Toolbox (Autour de Minuit) [git.autourdeminuit.com/autour_de_minuit/gp_toolbox](https://git.autourdeminuit.com/autour_de_minuit/gp_toolbox) | production helpers: layer nav, onion, realign | layers/frames ops, onion settings | AC ✓ (`onion` in `view`, refused to plugins) | D | classify onion-skin view commands as `view` class allowed with `ui.view` permission |
| 72 | GP Tween / Storytools (Blender extensions) [extensions.blender.org/tag/grease-pencil](https://extensions.blender.org/tag/grease-pencil/) | inbetweening, storyboarding | stroke correspondence, frame insert, camera | AC ✓ shape tweens + hints | D | `app.run` + `path.*` |
| 73 | TVPaint George scripts / PyTVPaint [doc.tvpaint.com/docs/george/george-overview](https://doc.tvpaint.com/docs/george/george-overview), [brunchstudio.github.io/pytvpaint](https://brunchstudio.github.io/pytvpaint) | automation of bitmap animation; Python bridge over WebSocket to a C++ plugin | full command access, external process control | AC ✓ registry; EC/FC ✓ | P | `app.run`; external control stays on MCP/control port, not plugins |
| 74 | OpenToonz DWANGO plugins (OpenCV effects) [github.com/opentoonz/dwango_opentoonz_plugins](https://github.com/opentoonz/dwango_opentoonz_plugins) | raster effect plugins via an effect SDK | pixels in/out, params, native OpenCV | AC ✗ raster filters on artwork | A, P | AC external filter registry on the filtered-layer path + `raster.filter@2` |

### 3.7 Audio-reactive and audio

| # | Plugin | What it does | Host capability required | Status | Gap | Proposal |
|---|---|---|---|---|---|---|
| 75 | Trapcode Sound Keys | audio spectrum bands to keyframes | audio samples of a layer/comp, FFT, bulk key write | EC ✓ (`keys.audioToKeyframes`); R ✗ | D, P | `audio.read`, `kernel.fft`, `keys.batch` |
| 76 | BeatEdit (aescripts, Premiere) [aescripts.com/beatedit-for-premiere-pro](https://aescripts.com/beatedit-for-premiere-pro/) | beat detection to markers/cuts | audio read, onset detection, markers, sequence edits | FC ✓; EC ✓ | D, P | `audio.read`, `kernel.onsets`, `markers.*`, FC `clip.*` |
| 77 | iZotope RX / VST3 repair in Premiere [izotope.com/en/products/rx.html](https://www.izotope.com/en/products/rx.html) | spectral repair, de-noise, de-reverb | real-time audio effect slot, latency, state | FC ✓ `AudioEffect`; R ✗ | A, P | `audio.effect@1` on `AudioEffect`; closed binaries Native (VST3 via out-of-process helper) |
| 78 | FireCut / AutoCut (Premiere panels) [firecut.ai](https://firecut.ai/blog/top-10-davinci-resolve-plugins-for-2026/), [autocut.com](https://www.autocut.com/) | silence removal, auto captions, zoom cuts | transcript, audio levels, ripple edits, captions write, network AI | FC ✓ (Whisper, loudness, trim) | D, P | FC adapter: `transcript.*`, `clip.razor/ripple`, `captions.*`; `net.fetch` |

### 3.8 Workflow, collaboration, render and assets

| # | Plugin | What it does | Host capability required | Status | Gap | Proposal |
|---|---|---|---|---|---|---|
| 79 | Frame.io panel [help.frame.io/…/adobe-premiere-frame-io-v4-panel](https://help.frame.io/articles/12833113-adobe-premiere-frame-io-v4-panel-overview-25-6-and-later) | upload renders, import comments as markers | OAuth, network, render/export job, markers write, docked panel | EC/FC ✓ markers, export | P, D | `net.fetch` + secrets, `export.start` job, `markers.*` |
| 80 | Resolve Workflow Integration plugins (Electron) [CommandPost/ResolveCafe docs](https://github.com/CommandPost/ResolveCafe/blob/main/docs/developers/workflowintegrations.md) | panels driving Resolve's JS API | full project/timeline API, panel UI, network | FC ✓ registry | D, P | FC adapter + docked panel; web-UI panels stay out (UI 1.0 only) |
| 81 | ftrack / ShotGrid integrations (AE, Nuke, Harmony) | publish versions, load assets, task context | project file paths, render outputs, network, versioned publish | partial | P | `net.fetch`, `files.write` via save-picker handle, `project.path` read with permission |
| 82 | Deadline submitters (AE, Harmony, TVPaint) [docs.thinkboxsoftware.com/…/app-tvpaint](https://docs.thinkboxsoftware.com/products/deadline/10.2/1_User%20Manual/manual/app-tvpaint.html) | submit renders to a farm | saved project path, render settings, plugin lock, network/job submission | EC/FC/AC CLIs ✓ | P | `render.submit` service + project plugin lock (section 5.1) |
| 83 | RenderGarden / BG Renderer (aescripts) | multi-process local rendering | spawn headless renderers on project copies | EC CLI ✓ | P | host-provided "render in background processes" (app feature, not plugin); plugins only request |
| 84 | Reactor (Fusion package manager) [steakunderwater.com/wesuckless](https://www.steakunderwater.com/wesuckless/) | installs community Fuses, macros, scripts | store/catalogue, dependencies | n/a | P | already the platform store/manager role |
| 85 | Bodymovin / LottieFiles for AE [github.com/airbnb/lottie-web](https://github.com/airbnb/lottie-web) | exports AE comps to Lottie JSON | read full comp model (layers, shapes, keys with tangents, expressions), write file | EC ✓ (built-in Lottie export), AC ✓ | D, P | `format.export@1` with document-model read (EC `layer.tree` with paths and keys) |
| 86 | Rive scripting (Luau) [rive.app/blog/scripting-is-live-in-rive](https://rive.app/blog/scripting-is-live-in-rive) | scripted layouts, behaviours, data binding at runtime | sandboxed runtime script run identically in editor and player | AC ✓ (frame scripts, player) | P | `runtime.script@1` (AC frame-script model) exported with the movie |
| 87 | Nuke Python panels (KnobScripter, W_hotbox) | script editors, radial menus | panel UI, run arbitrary script, node graph ops | EC ✓ scripting | D | out of plugin scope: AE-compat scripting stays automation; plugin analogue is `app.run` + panel |
| 88 | CommandPost (FCP/Resolve automation) [commandpost.io](https://commandpost.io/) | control surfaces, batch actions | external control of app commands | EC/FC/AC control ports ✓ | n/a | stays on MCP/control port (outside plugins) |

That is 88 items across 12 categories. Patterns visible across them:

- About a third (rows 46-73, 85) are **document automation**. They need `app.run` with typed
  setters for paths, text and keys, plus docked panels. They need no pixel access.
- About a third (rows 1-45) are **pixel effects**. They need `raster.filter@2` (frames, layers,
  bounds, aux channels, camera), GPU and per-instance state.
- The rest need **services**: `audio.read`, jobs, `net.fetch` with secrets, `model.run`, and a
  render/export service. Of all 88 rows, only a few need native code: vendor CUDA paths, closed
  VST3/OFX binaries, and Element 3D-class engines.

---

## 4. Per-app change lists

Priorities: **P0** blocks a plugin class; **P1** quality or speed; **P2** parity; **P3** later. Effort:
**S** ≤ 1 week, **M** 1-4 weeks, **L** > 4 weeks.

### 4.1 Changes common to all three apps

| ID | Change | Layer | Pri | Effort | Deps |
|---|---|---|---|---|---|
| A-ALL-1 | **Typed command metadata.** Extend each `CommandSpec` with `class: Query | Edit | View | Session | App`, `effects: EffectSet` (bitflags: `DOC_EDIT`, `FILE_READ`, `FILE_WRITE`, `NETWORK`, `PREFS`, `CLIPBOARD`, `PROCESS` (opens apps, runs scripts/helpers), `MODAL_UI` (dialogs), `LONG_JOB`, `IRREVERSIBLE` (needs confirm), `PLAYBACK`), `scope: Selection | Layer | Comp/Sequence/Scene | Document | App`, and `schema: Option<&'static str>` (JSON Schema). Fill from the existing params docs and namespaces; AniCraft's `plugin_may_call` (`AC/crates/engine/src/cmds_plugin.rs:33-72`), EC's MCP confine list and FC's `named` keys are the seed data | App ×3 | P0 | S-M each | none |
| A-ALL-2 | **One classification check per app**: a test that fails when a command lacks metadata, or when an `effects` bit is not covered by the adapter's permission map. Replaces AniCraft's namespace test and EC's hand list | App + Platform (shared test helper) | P0 | S | A-ALL-1 |
| A-ALL-3 | **Unknown data preservation**: every serialised struct a plugin can annotate keeps `plugin_data: BTreeMap<String, Value>` and, on effect/filter/transition instances, the unknown contribution id and params verbatim | App ×3 (FC and AC new; EC done for project/layer, add for effect instances) | P0 | S-M | none |
| A-ALL-4 | **Missing-plugin rule** as EC already implements it: preview pass-through with a badge, final render refuses unless the caller opts into "render without", project prompt on open | FC (new), AC (filters/tools once external), EC (done) | P0 | S | A-ALL-3 |
| A-ALL-5 | **Render-frame service** for panels and exporters: "render comp/sequence/scene X at time t, scale s, region r into a bounded buffer or GPU texture handle", as a query command (`render.frame`) usable by `app.run` and jobs | App ×3 (FC has `render_sequence`, AC has `RenderService`, EC needs `Renderer::render_layer_at`) | P1 | S-M | A-ALL-1 |
| A-ALL-6 | **Change events with change lists** (property paths, key times, element ids, selection) emitted by `Session::execute` | App ×3 | P1 | M | none |

### 4.2 EffectCraft

| ID | Change | Layer | Pri | Effort | Deps |
|---|---|---|---|---|---|
| EC-A1 | `ExternalRender::render_gpu(&self, enc: &mut dyn GpuEncoder, ctx, input: GpuImage) -> Option<GpuImage>`; arm `_ if external::gpu(id)` in `EC/gpu/src/effects.rs::apply`; `EffectSpec.gpu = true` when the package ships a validated shader | App | P1 | M | P-GPU |
| EC-A2 | Map new platform `ParamKind`s onto existing `ParamUi` variants (Point3, Layer, Mask, Path, File, Gradient, Hidden; `EC/project/src/props.rs`) | App | P0 | S | P-FORM |
| EC-A3 | Effect overlays in the viewer: plugin draw ops in layer space and drag events to params as one undo step (tool overlays already draw through `paint_ops`, `EC/ui-egui/src/panels/viewer.rs:310`) | App | P1 | M | P-OVL |
| EC-A4 | `PanelKind::Plugin(SurfaceKey)` in `EC/ui-egui/src/dock.rs`, drawn through `craft_ext_ui` in the dock instead of `renderer.window` (`plugins_ui.rs:118-140`); persist plugin panel placement in workspaces | App | P0 | S | none |
| EC-A5 | Importer/exporter registries in `EC/media` and `EC/export` that accept external contributions | App | P2 | M | P-FMT |
| EC-A6 | External transition kind: an external effect flagged `transition` gets a `completion` param and a second layer input (EC models transitions as layer effects) | App | P2 | S | P-TRN |
| EC-A7 | Persistent cache key includes plugin identity: hash `(plugin id, version, module sha256, contract version)` into layer/input cache keys (`EC/render/src/cache.rs:684-695`) instead of relying on the global generation bump (`external.rs` → `file_cache::bump_generation`) which drops every viewer frame | App | P1 | S | none |
| EC-A8 | Decide disposition of the `script` crate and ScriptUI panels: keep as AE-compatible automation (not a plugin system) but put its file/network prefs under the same `effects` metadata and permission words; dock ScriptUI and plugin panels through one code path | App | P1 | M | A-ALL-1 |
| EC-D1 | Implement `raster.filter@2` ops over `EffectHost`: `frame.at` (`self_at`), `layer.at`, `layer.masks`, `audio.read`, `aux.*`, `scene.camera/lights` (`comp_scene`), `params.at` | Adapter | P0 | M | P-RF2 |
| EC-D2 | Richer context: fps, comp time, shutter, layer span, masks, adjustment, float depth, working space name, proxy flag, px scale (`EffectEnv` fields 589-617) | Adapter | P0 | S | P-RF2 |
| EC-D3 | `app.run` bridge generated from A-ALL-1 metadata, replacing the 19-op pass-through list (`ops.rs:189-215`); typed setters for text documents, path vertices, gradients and expressions | Adapter | P0 | M | A-ALL-1, P-BRIDGE |
| EC-D4 | Tracking, roto, object matte, content fill and render queue as observable jobs | Adapter | P1 | M | P-JOB |
| EC-D5 | Per-instance state: `state.get/set` stored as effect-instance `plugin_data` (needs A-ALL-3 on effects) | Adapter | P1 | S | A-ALL-3 |

### 4.3 FilmCraft

| ID | Change | Layer | Pri | Effort | Deps |
|---|---|---|---|---|---|
| FC-A1 | External effect registry modelled on EC `external.rs`: lookup before the string `match` in `FC/render/src/effects.rs:118`; status badge; generation or identity in cache keys | App | P0 | M | A-ALL-3 |
| FC-A2 | External transition registry: `Transition.effect.effect` id dispatched to a plugin with `(from, to, progress)` | App | P1 | M | FC-A1 |
| FC-A3 | External audio effects: an `AudioEffect` impl (`FC/audio-dsp/src/lib.rs:123`) forwarding blocks to `audio.effect@1`, with latency and tail reported to the mixer; parameter smoothing stays host-side | App | P1 | M | P-AUD |
| FC-A4 | `FxOp::External { shader, uniforms }` in `FC/render/src/gpufx.rs` with the plugin's CPU fallback as reference | App | P1 | M | P-GPU |
| FC-A5 | `plugin_data` on project, sequence, track item, effect instance; keep unknown fields on round trip | App | P0 | S-M | A-ALL-3 |
| FC-A6 | Missing-effect refusal for export (`FC/export`) with an explicit "render without" option, as in EC commit `9d63cb7f` | App | P0 | S | FC-A1 |
| FC-A7 | Interchange: carry plugin effect ids through FCPXML/OTIO metadata so round trips keep them | App | P2 | S | FC-A5 |
| FC-D1 | New adapter crate on `Runtime<A>`: `filmcraft.session@1` = shared ops + `app.run` over ~619 commands; `raster.filter@2` over `FxEnv` (`frame`, `source_frame`, `track`, `clip_key` as cache key); `video.transition@1`; `audio.effect@1`; jobs for transcription, scene detection, export | Adapter (new) | P0 | L | P-RUNTIME, A-ALL-1 |
| FC-D2 | Docked plugin panels in FC's craft-ui dock | Adapter + App | P1 | S | FC-D1 |
| FC-OWN | Owner decision recorded in `FC/docs/gaps.md` G11: FFI forbidden in-process, so OFX/VST3 hosting is a native helper only | Decision | P1 | S | none |

### 4.4 AniCraft

| ID | Change | Layer | Pri | Effort | Deps |
|---|---|---|---|---|---|
| AC-A1 | Adopt the platform: new adapter crate on `Runtime<A>` implementing session ops over `Session::execute_nested` (atomic one-step runs exist), `document`/`elements` views as shared `layer.list`/`selection.get` shapes, and `app.run` from A-ALL-1 metadata | Adapter (new) | P0 | L | P-RUNTIME, A-ALL-1 |
| AC-A2 | **Migrate ABI v1** (`anicraft.call/response`, `ac_*` exports) through a bounded compatibility bridge: the bridge loads v1 modules as platform packages with a generated manifest and the fixed v1 policy as declared permissions. Owner: AniCraft adapter. Removal: once the catalogue holds platform builds of the four samples and one release has warned about v1. Then delete `AC/crates/plugins`, `plugin.*` commands, `<settings>/plugins` loader and `ANICRAFT_PLUGINS` | Adapter + App | P0 | M | AC-A1 |
| AC-A3 | Keep the four kinds as platform contracts: `anicraft.command` → session contract command contributions; `exporter` → `format.export@1` (display-list variant); `ease` → `timeline.ease@1` (baked); `frame-script` → `runtime.script@1` | Platform + Adapter | P0 | M | AC-A2 |
| AC-A4 | Upgrade egui/eframe 0.36 → 0.37 to embed `craft_ext_ui` (platform UI renderer is 0.37) and decide between AC's own dock (ADR 0075) and craft-ui docking; add `Panel::Plugin(key)` (closed 13-variant enum at `AC/src/dock.rs:19-34`) | App | P0 | M | framework decision |
| AC-A5 | External filters: `Filter::Plugin { id, params, data }` variant (plus unknown-variant preservation) applied in the oracle's filtered-layer path and as a GPU pass in `AC/crates/gpu/src/filters.wgsl` when a WGSL shader ships | App | P1 | M | P-RF2, P-GPU |
| AC-A6 | Tool contributions: `Tool::Plugin(id)` with pointer/pressure/tilt events and overlay draw ops on the stage; `input.filter@1` hook before built-in Pencil/Brush; `brush.dab@1` for custom vector brush sweeps | App | P1 | M | P-OVL |
| AC-A7 | Typed vector data ops: `path.get/set` by data (points, handles, closed) for shapes, motion paths and raw subpaths (AC's "not in v1" item) | App + Adapter | P0 | S | A-ALL-1 |
| AC-A8 | Audio access: `audio.read {sound|layer|scene, start, frames, rate}` over `doc::sound::voices` + `audio::mix`; phoneme/lip-sync job writes switch-symbol first frames over a span | App + Adapter | P1 | S | P-JOB |
| AC-A9 | Rig hooks: `rig.controller@1` (on-stage controller widgets mapped to poses, baked), and a decision on plugin deformers inside `evaluate` (section 6, D2) | App + Platform | P2 | M | AC-A6 |
| AC-A10 | Exporters with document access: a second exporter variant (opt-in permission `document.read`) for Spine/DragonBones/Live2D-class formats that need bones and keys rather than flattened frames | Platform + Adapter | P2 | S | AC-A3 |
| AC-A11 | Background command runs with progress and cancel (exports already run on a worker), and binary document views for large reads | Adapter + Platform | P2 | M | P-JOB |
| AC-A12 | HTML canvas export of `runtime.script@1` modules: embed the Wasm module and run it in the page's player instead of reporting "not exported" (`AC/crates/render/src/html.rs:102`), license permitting | App | P3 | M | AC-A3 |

### 4.5 Platform needs from these apps (`craft-extensions`)

| ID | Need | Pri | Effort | Notes |
|---|---|---|---|---|
| P-RUNTIME | `Runtime<A: CraftApp>` with the transaction state machine AniCraft already proves (nested atomic calls, one labelled step, revert on failure, preview via reverted gesture) | P0 | L | program 2.2 |
| P-BRIDGE | `app.run`/`app.commands` generated from A-ALL-1 metadata; permission mapping `effects` → permissions (`DOC_EDIT` → `document.edit`, `FILE_*` → picker handles only, `NETWORK` refused, `PROCESS` refused for Wasm, `PREFS` refused, `MODAL_UI` refused while headless) | P0 | M | |
| P-RF2 | `raster.filter@2`: `frame.at`, `layer.at`, `aux.*`, `scene.*`, `params.at`, ROI and output bounds, tiles, `thread_safe`, `quality`, `px_scale`, `proxy` | P0 | M | |
| P-TRN | `video.transition@1`, `video.generator@1`, `video.retimer@1` | P1 | S-M | |
| P-FORM | `ParamKind` additions: Point3, Layer, Mask, Path, Curve, Gradient, File (handle), Data, Button, Group, Text multiline (CE form has 11 variants today) | P0 | M | |
| P-OVL | `effect.overlay` and stage tool events shared by EC viewer, FC program monitor and AC stage | P1 | M | |
| P-GPU | `gpu.shader@1` with host-written WGSL wrapper, naga validation, bounded dispatch, watchdog, mandatory CPU fallback; a `GpuHost` trait the three apps implement over their wgpu 30 device | P1 | L | |
| P-AUD | `audio.read`, `audio.effect@1` (latency, tail, state, block-size invariance), `audio.analyze` kernels | P1 | M | |
| P-FMT | `format.import@1`, `format.export@1` with a display-list variant and a document variant | P1 | M | |
| P-JOB | Jobs with frame streaming, progress, cancel, and results as document edits in one step | P1 | M | |
| P-LOCK | Project plugin lock and farm install (section 5.1) | P1 | M | |
| P-ANIM | `timeline.ease@1` (baked tables), `runtime.script@1`, `rig.controller@1`, `input.filter@1`, `brush.dab@1`, `expression.function@1` (section 5.6) | P1-P2 | M each | |

---

## 5. Edge cases

### 5.1 Render farm and headless rendering

- **Where it stands.** All three apps have headless renderers:
  - `effectcraft-cli render` takes `--without-missing-effects`;
  - `filmcraft-cli render` and FC's parallel export exist;
  - `anicraft-cli` loads plugins only with `--plugins` or `ANICRAFT_PLUGINS`.

  None of them records which plugin build a project needs.
- **Proposal: a project lock.** Each project carries a lock entry per plugin effect it uses:
  plugin id, version, module sha256, contract version and grants needed. EC's `plugin_data` or a
  dedicated block can hold it. A farm worker then:
  - runs `craft-ext-cli install --from-lock <project>` against a signed catalogue or a mirrored
    store;
  - refuses to render on a hash mismatch, unless the job opts into "render without missing".
- **Grants on farm nodes.** Nothing can be granted interactively on a farm node. A node gets only
  the grants an admin set in a farm policy file. Network and native helpers are off by default.
  Native helpers still need consent per version, so an admin pre-consents for the farm. They stay
  out of process and supervised.
- **GPU.** A farm node may have no GPU, or a different vendor. `gpu.shader@1` must ship a CPU
  fallback (Wasm or `kernel.graph`). The final render should record which path ran for each
  effect. Review mode can force the CPU path, following AniCraft's oracle rule.
- **Dialogs.** Commands flagged `MODAL_UI` (file dialogs, confirmations) are refused headless.
  AniCraft already refuses confirmations inside a plugin run.

### 5.2 Multi-frame rendering and thread safety

- **EffectCraft.** EC keeps a thread-local pool of up to 16 filter instances per render thread
  (`EC/plugins/src/effects.rs:132-134`), and export renders frames in parallel with rayon. At up
  to 512 MiB of memory per filter instance, the worst case is threads × 16 × 512 MiB. The platform
  needs:
  - one global memory budget across instances;
  - instance eviction under pressure;
  - a manifest flag, `thread_safe` or `stateless`, that allows parallel instances.
- **FilmCraft.** FC's export renders frames with rayon from inside a scope (`FC/export/src/job.rs:121`,
  `run.rs:190`), so the same rules apply.
- **Simulations.** Particular-class plugins depend on earlier frames. Under MFR (AE's model) a
  frame may only read shared state, so a sim needs a host-run pre-roll:
  - a sim-cache job steps frames in order and stores state with `cache.put`, keyed by plugin
    identity, parameter hash and frame;
  - frame renders read that state and never write it.

  EC's built-in particle sim and its `analysis_cache.rs` show the pattern.
- **Re-entrancy.** A `frame.at` call inside a filter renders another frame of the same layer. The
  host must:
  - refuse recursion through the same effect (FC already skips temporal effects for each other,
    `is_temporal`);
  - bound the frame window;
  - count nested renders against the outer job's budget.
- **AniCraft.** The oracle is deterministic across threads because each output row has exactly
  one writer. Plugin passes must keep that property: a band-tiled plugin filter may write only its
  own band.

### 5.3 Cache invalidation when a plugin updates

- **EffectCraft today.** Registering or unregistering an external effect bumps one global
  generation (`EC/effects/src/external.rs`, which calls `file_cache::bump_generation`). That drops
  every cached viewer frame, even frames that have nothing to do with the plugin. The layer cache
  hashes effect stacks (`EC/render/src/cache.rs:684-695`). This pass did not verify whether the
  disk cache key carries plugin identity, so it should be checked.
- **Fix.** Hash `(plugin id, version, module sha256, contract version, declared determinism)` into
  every cache key that covers a plugin effect. Then an update invalidates only the frames that
  depend on that plugin, and persistent disk caches cannot serve stale pixels after an update.
- **AniCraft** avoids the problem for eases by baking them. The same rule should apply to any
  tabulable plugin output.
- **FilmCraft** needs the identity in its media/preview cache keys (`FC/engine/src/previews.rs`)
  as soon as FC-A1 lands.
- **Pinning.** A project should pin a plugin version, and updating it should be an explicit,
  undoable project change. Render queue jobs lock versions when they start, so a long render never
  mixes frames from two plugin versions.

### 5.4 Project portability with missing plugins

| App | Today | Needed |
|---|---|---|
| EC | `plugin_data` survives; missing effects keep their property groups, pass through with a badge, and the final render refuses unless `RenderWithout` is chosen (commit `9d63cb7f`); a project prompt lists missing plugins | Lock entries; effect-instance `plugin_data` |
| FC | Unknown effect id renders as a silent pass-through (`FC/render/src/effects.rs:508`); `EffectInstance.params` is a `BTreeMap`, so params survive, but project structs have no unknown-field preservation | FC-A1, FC-A5, FC-A6 |
| AC | Documents never contain plugins. Ease tables are baked and play anywhere. Frame-script references survive. HTML export reports plugin frame scripts as not exported (`AC/crates/render/src/html.rs:102`). The `Filter` enum is closed, so an unknown filter variant from a newer build is a parse error, not preserved data | AC-A5 with an unknown-variant fallback; AC-A12 |

Rule for all three: open-without, save and reopen must keep the plugin's instance data
byte-for-byte. Preview may pass through only with a visible badge. A final render or export needs
explicit consent to render without the missing plugin.

### 5.5 Deterministic output for collaboration and review

- **Engine settings.** The platform's wasmi already runs with the `deterministic` feature
  (`CE/ext-host/Cargo.toml:18`). A JIT tier must turn on NaN canonicalisation and turn off relaxed
  SIMD. Plugins get no clock or randomness; the platform passes a seed in the context (EC already
  does).
- **Review renders.** GPU shader output can differ by vendor through FMA, transcendental precision
  and subgroup behaviour. For review, the reference render uses the CPU fallback (AniCraft's oracle
  rule), and render metadata records each plugin's identity and the path it ran on.
- **Content hashes.** A per-frame hash needs plugin outputs that are a pure function of
  `(inputs, params, seed, time)`. A plugin that reads `state`, `cache` or `net` must declare
  itself non-deterministic. Review tools then show its frames as "not reproducible".
- **Collaboration.** None of these three apps has live co-editing. Shared review still depends on
  identical renders on two machines, which is the lock plus determinism rules above.

### 5.6 Expressions calling plugins

- **Today.** EC's expressions (boa, `EC/expr`) can read a plugin effect's parameters, because
  external effects are ordinary property groups (`external.rs` header). They cannot call plugin
  code.
- **Proposal: `expression.function@1`.** A plugin declares pure functions, such as
  `noise.curl(p, t)` or `ease.house(t)`. Expressions call them through a namespaced global. The
  host:
  - runs them with a small fuel budget per call;
  - memoises them by arguments;
  - refuses calls into the document;
  - detects cycles (a plugin reading a property whose expression calls the plugin).
- **Bake option.** For one-input curves, AniCraft's baking is the better default: sample into the
  document and evaluate without the plugin.
- **AniCraft** has no expressions. Its equivalent is the ActionScript subset, which runs only in
  the player.
- **FilmCraft** has no expressions.

### 5.7 Plugins in proxies and full resolution

- **What the apps pass today.**
  - EC passes `scale`, `full` and `origin`, plus `quality: draft|final` inferred from whether an
    error sink exists (`EC/plugins/src/effects.rs:118-126`).
  - EC proxies change the source pixels, and the cache key hashes the proxy (`EC/render/src/cache.rs:803-807`).
  - FC passes `px_scale` in `FxCtx`.
  - AC renders at a view scale.
- **What plugins need.**
  - an explicit `proxy: bool` and the source's full-resolution size, so spatial parameters stay in
    full-resolution units (EC's `PointSpace::LayerPixels` already converts);
  - a rule that analysis results (noise profiles, tracks, flow) carry the resolution they were
    computed at and are recomputed or rescaled for final render;
  - cache keys that include scale and proxy state.
- **Draft-only shortcuts** must not leak into final renders. The platform should derive
  `quality: final` from the render queue or export context, not infer it from an error sink.

### 5.8 Other edge cases

- **Undo of external effects.** The rule is that external effects (network uploads, exported
  files) are not document Undo. AC exporters write files only through the host, which keeps this
  honest. EC and FC export jobs must report files as external results, not as undoable edits.
- **Plugin update during an open live preview.** AniCraft's live preview reverts the gesture
  before each re-run. On reload, the platform must cancel the preview gesture so it never commits
  output from a mix of versions.
- **Audio block invariance.** FC's `AudioEffect` smooths parameters per sample. A plugin effect
  must produce identical output whatever the block size, and must declare its tail for renders
  that stop early.
- **The web builds** (EC, FC and AC all have them) cannot run native helpers. They need the Wasm
  interpreter and WebGPU for `gpu.shader@1`, and no farm features.

---

## 6. Open decisions

| # | Decision | Options | Recommendation |
|---|---|---|---|
| D1 | AniCraft ABI v1 | (a) bridge v1 modules into the platform with removal criteria; (b) hard cut | (a): AniCraft is on upstream main and has public samples and docs; a bridge with an owner and an end date satisfies the replacement rule |
| D2 | Plugin code inside the pure render path (`evaluate`, expressions, deformers, eases) | (a) never: bake only; (b) allowed for pure, deterministic, declared functions with a baked fallback stored in the document | (b) with the bake stored as the missing-plugin fallback, so documents stay portable (generalises ADR 0088) |
| D3 | Where command metadata lives | (a) in each app's `CommandSpec` (upstream change); (b) adapter-side tables | (a): the MCP, control port and plugins all need it, and drift is caught by one test per app |
| D4 | GPU tier scope | (a) WGSL fragments inside a host wrapper; (b) also full compute pipelines from Wasm (`gpu.dispatch`) | (a) first, (b) behind a P2 flag after watchdog evidence on Metal/Vulkan/D3D12 |
| D5 | AniCraft UI stack | (a) move to craft-ui docking and egui 0.37 now; (b) keep ADR 0075 dock and add a plugin panel variant | Coordinate with the craft-framework/UI work; minimum is (b) plus the egui bump |
| D6 | FilmCraft foreign plugins (OFX, VST3, AU) | (a) native helpers only; (b) none | (a) for audio first (largest user demand), with shared-memory transport; FC's `unsafe` rule keeps FFI out of process |
| D7 | EC `script`/ScriptUI | (a) keep as automation outside the plugin platform; (b) fold into the platform as a script tier | (a), with shared permission vocabulary and docking path (EC-A8); it is general automation with non-plugin users |
| D8 | Farm policy | admin-signed farm policy file vs per-node consent | admin policy file, per version, auditable |
| D9 | Exporter document access in AniCraft | display list only vs opt-in document read | both, with `document.read` declared for the second |
| D10 | Determinism class in manifests | none vs `deterministic | seeded | stateful | networked` | require it; review tooling and caches depend on it |

---

## 7. Sources

Local source and docs (read, not executed):

- AniCraft: `README.md`, `Cargo.toml`, `docs/plugins.md`, `docs/architecture.md`, `docs/mcp.md`,
  `docs/roadmap.md`, `docs/requirements.md`, `docs/adr/0017-wasm-plugins.md`, `0088-exporter-and-easing-plug-ins.md`,
  `0097-command-macros.md`, `0104-live-control-port.md`; `crates/engine/src/lib.rs`, `cmds_plugin.rs`,
  `cmds_bone.rs`, `onion.rs`, `multi.rs`, `cmds_retime.rs`, `render_service.rs`; `crates/plugins/src/*`;
  `crates/doc/src/filter.rs`, `easing.rs`, `sound.rs`; `crates/actions/src/lib.rs`; `crates/render/src/html.rs`;
  `src/dock.rs`, `src/app.rs`, `src/plugins_ui.rs`.
- EffectCraft: `Cargo.toml`; `crates/engine/src/cmd.rs`, `cmd/schema.rs`; `crates/plugins/src/ops.rs`,
  `effects.rs`; `crates/effects/src/lib.rs`, `external.rs`, `catalog.rs`, `file_cache.rs`;
  `crates/gpu/src/effects.rs`; `crates/render/src/cache.rs`; `crates/export/src/pipeline.rs`;
  `crates/ui-egui/src/dock.rs`, `plugins_ui.rs`; `crates/automation/src/confine.rs`; `crates/script/src/lib.rs`;
  commit `9d63cb7f`.
- FilmCraft: `README.md`, `docs/gaps.md`, `Cargo.toml`; `crates/engine/src/commands.rs`;
  `crates/render/src/vfx.rs`, `effects.rs`, `gpufx.rs`, `lib.rs`; `crates/project/src/effect.rs`, `track.rs`;
  `crates/audio-dsp/src/lib.rs`; `crates/export/src/run.rs`, `job.rs`; `crates/automation/src/server.rs`.
- Platform: `docs/plugins/suite-replacement-rule.md`, `continuation-2026-11-17.md`,
  `research/capability-program-2026-11-18.md`, `research/capability-2026-11-18/motion.md`;
  `crates/ext-core/src/form.rs`, `crates/ext-host/Cargo.toml`, `crates/ext-kernels/src/lib.rs`,
  `crates/ext-ui/Cargo.toml`.

Web (searched 2026-11-18; vendor pages are product claims):

- Toon Boom scripting: https://docs.toonboom.com/help/harmony-24/premium/scripting/about-scripting.html ;
  https://docs.toonboom.com/help/harmony-20/go/scripting/extended/index.html ; OpenHarmony
  https://cfourney.github.io/OpenHarmony/ ; https://github.com/mchaptel/TBScripts ; https://github.com/dpotekhin/ToonBoomHarmonyScripts
- Moho: https://www.lostmarble.com/forum/viewtopic.php?p=202486 ; https://github.com/SimplSam/moho-make-bones ; https://redspark.itch.io/moho-scripts
- TVPaint George: https://doc.tvpaint.com/docs/george/george-overview ; PyTVPaint https://brunchstudio.github.io/pytvpaint ;
  Deadline TVPaint https://docs.thinkboxsoftware.com/products/deadline/10.2/1_User%20Manual/manual/app-tvpaint.html
- OpenToonz: https://opentoonz.readthedocs.io/en/latest/ ; https://github.com/opentoonz/dwango_opentoonz_plugins
- Rive scripting: https://rive.app/blog/scripting-is-live-in-rive ; https://rive.app/blog/why-scripting-runs-on-luau ; https://rive.app/docs/scripting/script-inputs
- Nuke BlinkScript: https://learn.foundry.com/nuke/content/reference_guide/other_nodes/blinkscript.html ;
  https://learn.foundry.com/NUKE/content/comp_environment/blinkscript/image_processing_blink.html
- Resolve Workflow Integrations: https://github.com/CommandPost/ResolveCafe/blob/main/docs/developers/workflowintegrations.md ;
  Krokodove https://nmbr73.github.io/Kernfusion/Plugins/Krokodove/Krokodove/ ; DCTL https://wiki.dvresolve.com/developer-docs/dctl
- Adobe Animate extensions: https://ajarproductions.com/blog/flash-extensions/ ; https://helpx.adobe.com/animate/using/creating-html-extensions.html
- Blender Grease Pencil: https://extensions.blender.org/add-ons/grease-pencil-tools/ ; https://extensions.blender.org/tag/grease-pencil/ ;
  https://git.autourdeminuit.com/autour_de_minuit/gp_toolbox
- Spine CLI: https://esotericsoftware.com/spine-command-line-interface ; https://esotericsoftware.com/spine-export
- FxPlug ecosystem: https://fxfactory.com/help/finalcutpro/ ; https://www.apple.com/final-cut-pro/resources/ecosystem ; https://fcp.cafe/ecosystem/plugins/
- Resolve plugin round-ups (secondary): https://firecut.ai/blog/top-10-davinci-resolve-plugins-for-2026/
- Carried from the earlier pass (AE SDK, OFX, FxPlug, Trapcode, Mocha, Neat, Topaz, Duik, Overlord,
  Plexus, Deep Glow, Newton, BeatEdit, Frame.io, Rhubarb, ISF): see section 6 of
  `research/capability-2026-11-18/motion.md`.
- Named from general knowledge, not fetched this pass (treat as pointers): Stardust, Element 3D,
  Saber, Optical Flares, Universe, Continuum, Sapphire, Silhouette, Lockdown, SliceX,
  MotionVFX, Twixtor/RSMB, DE:Noise, Rotobot, Depth Scanner, Cryptomatte, Magic Bullet Looks,
  FilmConvert, Dehancer, Colourlab AI, Color Finale, FilmImpact, Animation Composer, Motion Bro,
  Flow, Motion 4, Typemonkey, KBar, FX Console, RubberHose, Joysticks 'n Sliders, Live2D SDK,
  iZotope RX, FireCut, AutoCut, ftrack/ShotGrid, RenderGarden, Reactor, Bodymovin, KnobScripter,
  CommandPost. Their host-capability rows are inferred from documented behaviour, not inspected code.

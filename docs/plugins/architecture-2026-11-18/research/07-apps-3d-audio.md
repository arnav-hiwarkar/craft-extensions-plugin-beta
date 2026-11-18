# 07: MaterialCraft, TextureCraft, SoundCraft and craft-libs plugin architecture

Researcher 7 of 8. Date 2026-11-18. Read-only. Nothing was built or run.

Revisions read:

- MaterialCraft `49ebd50` (752 `.rs` files)
- TextureCraft `97f6b81` (523)
- SoundCraft `8ee63dc` (397)
- craft-libs `baa619f01` (3,383)
- craft-extensions `24cd2ff`

"Observed" means read in source at those revisions. Counts were made with grep or perl over non-test source; each count says how it was made. Everything else is an estimate or comes from a cited URL. URLs marked *(unverified)* were not fetched in this pass.

This report builds on, and corrects, `craft-extensions/docs/plugins/research/capability-2026-11-18/3d.md` and `audio.md` and the program document `capability-program-2026-11-18.md`. All three were written against much older trees.

---

## 1. Summary

1. **The apps are bigger than the earlier passes assumed, and they already reach far beyond plugins.**

   | | MaterialCraft | TextureCraft | SoundCraft |
   |---|---|---|---|
   | Engine command ids | **133** (earlier pass: about 50) | **183** (earlier: about 80) | **389** |
   | Command definitions | `crates/engine/src/commands.rs:12-23` | `crates/engine/src/commands.rs:14-25` | `crates/engine/src/cmd.rs:74-88` |
   | Plugin ops today | 0 | 0 | 0 |

   - All three use the same `CommandSpec` shape, and each defines it separately.
   - Parameters are a free-text `params: &'static str` in all three. None has a JSON schema.
   - Only SoundCraft classifies its commands: undoable, `noundo`, `query`, `view`, and a `journal` flag.
   - MaterialCraft and TextureCraft give no query/edit flag at all. An edit is whatever calls `Session::edit`.

   **A shared, schema-bearing command type in craft-libs is the cheapest P0 win.** It would feed the registry bridge, MCP and plugin bindings from one source.

2. **MaterialCraft's node library is already partly open, but through data, not code.**
   - The native catalogue is closed:
     - It is a `static OnceLock<Vec<NodeDefinition>>` (`crates/nodes/src/catalog.rs:5-8`).
     - Cooks are `pub type Cook = fn(&CookContext) -> CookResult` (`crates/nodes/src/lib.rs:37`).
     - Dispatch is a string match in `crates/eval/src/node_execution.rs:166-223`.
     - It holds only 29 atomic nodes, 85 function operations and 4 pattern nodes.
   - The visible library comes from SBS files instead. There are 549 bundled `.sbs` packages, which contain 1,025 graphs and 954 functions. They load from `assets/sbs` or `MATERIALCRAFT_SBS_DIR` (`crates/formats/src/sbs/library.rs:62`; `crates/engine/src/node_library.rs:1-7`).
   - Unknown nodes are kept as `missing_node` placeholders, with their raw parameters (`crates/core/src/missing_node.rs:4`; `crates/formats/src/sbs/placeholder.rs`).
   - So the "graph tier" of `graph.node@1` already works as a folder of SBS files. Packaging it as a plugin data contribution is small work. Code tiers (WGSL and Wasm) need the registry rewrite.

3. **TextureCraft is the riskier app for plugin data.**
   - Seventeen closed enums carry document semantics, among them:
     - `EffectKind` (10 variants) and `Filter` (26), in `crates/core/src/layer.rs:1008,642`;
     - `GeneratorKind` (7), `ProceduralKind` (23) and `BlendMode` (32);
     - `BakedMap` (10), `BrushAlpha` (7), and the export `FileFormat` and `VirtualMap`.
   - serde is strict on enum variants, and core has no `#[serde(other)]` anywhere. **One unknown effect therefore makes the whole `.tcraft` fail to load.**
   - Unknown struct fields are silently dropped.
   - Tolerant decoding is a P0 precondition before any plugin can write into a TextureCraft project.

4. **GPU.**
   - TextureCraft runs wgpu 30 with 11 WGSL files. Its pipelines are 6 compute entry points (earlier pass: 10) and 24 render pipelines. They cover the PBR viewport, a path tracer with an à-trous denoiser, and GPU bake rays over a CPU-built BVH.
   - Its compositor is CPU-only, on 64×64 u16 tiles. Only composited display textures live on the GPU.
   - MaterialCraft cooks entirely on the CPU and renders its 3D view with a CPU ray caster. Neither app's crates call wgpu or naga directly; wgpu (30.0.1) is present only through eframe and egui-wgpu.
   - craft-libs has **no GPU crate**.
   - A shared GPU layer (a device broker, a texture pool with a memory budget, a WGSL prelude and a tiled watchdog dispatch) is the missing foundation. It underlies GPU-resident textures and the WGSL plugin tier.

5. **Mesh.** The suite has two mesh types:
   - craft-libs `craft-mesh::Mesh` (`crates/mesh/src/lib.rs:229`), which MaterialCraft uses with a 4M-triangle cap;
   - TextureCraft's own `geom::Mesh` (`crates/geom/src/mesh.rs:37-52`), with tangents, colours, submeshes and a 20M-triangle cap.

   A shared `mesh@1` should be one type in craft-libs that both apps adopt.

6. **SoundCraft hosts native CLAP, VST3 and AUv2 in-process.** The earlier pass got several engine facts wrong:
   - **Blocks are not a fixed 512.** Realtime blocks are the callback size times the speed ratio, clamped to 1..512 (`crates/playback/src/callback.rs:292`). Bounces use 1024 (`crates/engine/src/io.rs:390`).
   - **Strips do not render in parallel in realtime.** They render in sequence on the audio thread, because `run_strips` only goes parallel off the audio thread (`crates/mix/src/threads.rs:38`).
   - **Plugin parameters** are read once per block at the block start (`crates/mix/src/strip.rs:480-516`).
   - **Automation resolution differs by target:**
     - volume, trim and VCA: per sample;
     - pan: on a 64-sample grid;
     - sends and plugin parameters: per block.

   Consequence: **a bounce does not equal playback for any automated plugin parameter.** That is the first audio fix, before any Wasm tier.

7. **SoundCraft fault handling.** A panic still silences the whole output and stops the transport (`crates/playback/src/callback.rs:55-62`). A NaN from a plugin already takes just that slot out (`crates/mix/src/strip.rs:441-451`). The per-slot fault policy that a Wasm or bridge tier needs extends that NaN path.

8. **The plugin entry seam in SoundCraft already exists.** `create_plugin` tries each host in turn by id prefix (`crates/mix/src/instances.rs:114-126`): `soundcraft_dsp`, then `clap:`, then `vst3:`, then `au:`. A `craft:` prefix (the Wasm real-time tier) and a `bridge:` prefix (out-of-process native) slot in there. The `Plugin` trait (`crates/dsp/src/lib.rs:195-265`) needs:
   - an event-list process call;
   - buses;
   - modulation;
   - restart requests.

9. **craft-libs has the format and UI breadth, but none of the plugin-critical runtime pieces:**
   - no GPU crate;
   - no DSP crate;
   - no MIDI device I/O;
   - no shared command registry;
   - no node-graph widget;
   - no ONNX inference (only candle LLMs in `crates/ai`, unused by these apps).

   Three MCP stacks also coexist: `craft-mcp`, `craft-mcp-server`, and the apps' own rmcp or hand-rolled servers.

10. **Catalogue verdict.** Section 4 has 92 named items.
    - About 70% need only these: the registry bridge, data packages, the `graph.node@1` tiers, `mesh@1`, net and secrets, and the audio Wasm tier.
    - About 15% need host services that do not exist yet: `nn.run`, `sample.stream`, ARA-like clip access, and controller I/O.
    - About 15% stay native out of process:
      - the commercial audio catalogue;
      - UVPackmaster-class closed engines;
      - Houdini Engine;
      - iLok/PACE-licensed and DSP-hardware products.

---

## 2. Per-app inventory

### 2.1 MaterialCraft (Substance Designer class)

| Area | Observed | Evidence |
|---|---|---|
| Commands | 133 ids, no duplicates (perl over `cmd!(\s*"id"`, tests excluded). By prefix: `graph.*` 65, `view.*` 20 (17 are `view.3d.*`), `file.*` 13, `edit.*` 13, `library.*` 8, `folder.*` 5, `session.*` 3, `package.*` 3, `explorer.*` 3. The UI adds 46 `ui.*` ids | `crates/engine/src/commands.rs:12-23,43-48,50-96`; `crates/ui-egui/src/menus.rs` |
| Schema | Free text, for example `"{dir, format? (png|…)}"`. Only the MCP wrapper tools have schemars structs | `crates/engine/src/file_cmds.rs:41`; `crates/automation/src/server/parameter/` |
| Query vs edit | No flag. An edit is whatever calls `Session::edit`, `edit_merged` or `group`: 73 call sites in 26 files | `crates/engine/src/session.rs:219-306` |
| Undo | Whole-`Package` snapshot. `edit_merged` coalesces edits, `begin_group`/`end_group` group them, the limit is 200 (1..10,000), and each automation call is one step unless `merge:true` | `snapshot.rs:5-16`; `history_prefs.rs:12-19`; `commands.rs:197-208` |
| Side-effect commands | Writes: `file.save*`, `file.export`, `file.exportMaterial`, `view.3d.saveRender`/`exportGeometry`/`saveState`. `file.sendToTextureCraft` writes to `~/Documents/TextureCraft/library`. Reads: `file.open`, `importBitmap`/`importVector`, `view.3d.loadMesh`, `graph.input.bind {path}`. Preferences: four commands. No network, no process spawning | `material_cmds.rs:34-38,181-185` |
| MCP, CLI, RPC | rmcp server with 20 tools, among them `command_list`, `command_run` and `command_batch`. Two backends, Headless and Bridge. JSON-lines RPC with 12 methods over stdio or loopback TCP with a token. The CLI covers info, commands, nodes, run, batch, cook, export, render, mcp and serve | `crates/automation/src/server/material_craft_mcp.rs:137-150`; `rpc.rs:15-28`; `apps/materialcraft-cli/src/main.rs:12-44` |
| Sandbox | cap-std `AuthorizedWorkspace`. `authorize_command` refuses `file.*` and any `path`/`dir` parameter. The app control channel, which needs a token, skips that filter | `crates/automation/src/workspace.rs:1-24,166-180`; `crates/ui-egui/src/control.rs:261-269` |
| Events | None; clients poll `modified`, `dirty` and `cook_epoch` | `session.rs:83-102` |
| Document | `Package` → `Graph` → `Node`. `Node.def` is a `String`, with optional sub-graph instances and per-parameter functions. `GraphKind` is `Material` or `Function` | `crates/core/src/{package.rs:8-27,graph.rs:11-60,node.rs:10-35}` |
| Files | SBS XML is the document. `.mcraft` is a zip holding `package.sbs` and `resources/`. `.mcmat` comes through craft-mcmat. No `.sbsar` and no MaterialX; the MaterialX export tab is a stub | `crates/formats/src/project.rs:1-15`; `ui-egui/src/export/material_tab.rs:1` |
| Ports and values | `PortKind {Gray, Color, Any, Value}`. `Value` has 16 variants, among them Curve, Gradient, Function and PatternGraph. Parameter widgets are inferred from the default value | `crates/nodes/src/port_kind.rs:3-9`; `core/src/value.rs:7-27`; `parameter_definition.rs:5-19` |
| Node registry | Closed native set: 29 atomic, 85 function and 4 pattern nodes. The library is SBS data: 549 packages, 1,025 graphs, 954 functions (grep counts of `<graph>`/`<function>`). The docs disagree (227 nodes in `docs/gaps.md:45`) | `crates/nodes/src/catalog.rs:5-8`; `nodes/src/atomic/catalog.rs:4-35`; `engine/src/node_library.rs` |
| Cook | Topological order on one thread, with rayon inside nodes, on a worker on desktop and the UI thread on the web. Cache key is a `DefaultHasher` over the def, parameters, size, seed, tiling, format, inputs, resources, sub-graph hash and variables. LRU of 256 MiB and 8,192 entries. Graphs are 16..8192 px. Preview cap 512 (64..8192). Instance depth up to 8 | `eval/src/graph_execution.rs:45-55`; `node_execution.rs:148-160`; `cache.rs:5-9`; `core/src/graph.rs:63-65` |
| Determinism | Integer-hash and LCG random numbers; fixed float order through `mul_add`; SHA-256 preview fingerprints | `nodes/src/random.rs:1-23`; `node_execution.rs:181-183` |
| GPU | No direct wgpu or naga in any crate. No WGSL. The 3D view is a CPU ray caster with prefiltered IBL, uploaded as an egui texture. Under `AGENTS.md`, only `render`, `ui-egui` and the apps may use wgpu, and any GPU path must match the CPU cook | `render/src/preview.rs:1-4`; `ibl.rs:1-3`; `ui-egui/src/view3d/frame.rs:435` |
| Mesh | OBJ, glTF and GLB through craft-mesh, capped at 4M triangles, with auto-unwrap | `crates/formats/src/mesh.rs:1-15`; `geom/src/unwrap.rs` |
| Baking | None. A `UvGBuffer` exists but nothing uses it. Mesh maps arrive from TextureCraft through `graph.input.bind` | `geom/src/gbuffer.rs:1-3`; `docs/gaps.md:35,47` |
| UI | egui 0.37 fork with `craft_ui::docking`, 8 panels and a custom node editor (`ui-egui/src/graph/`). No plugin hooks | `ui-egui/src/docking.rs:22-31` |
| craft-libs (git rev `453b0da`) | mime, mesh, xml, svg, mcmat, ui, keymap, dialogs, quit, menu, window, update, atomic, fonts-build, texture | root `Cargo.toml:42-106` |

### 2.2 TextureCraft (Substance Painter class)

| Area | Observed | Evidence |
|---|---|---|
| Commands | 183 ids (perl over `cmd!(`). By prefix: layer 26, tool 21, camera 17, textureSet 16, effect 16, file 14, bake 14, viewport 11, mask 10, material 8, brush 7, project 5, export 5, resource 4, edit 3, path 2, and one each for text, filter, display and alpha. `ROADMAP.md:37` says 178, which is stale | `crates/engine/src/commands.rs:14-25,44-49,51-90` |
| Query vs edit | No flag. 68 `edit`/`edit_merging` call sites; 22 ids are queries by name. `tool_cmds.rs`, `view_cmds.rs` and `file_cmds.rs` change state outside undo | `session.rs:518-541` |
| Undo | Whole-`Document` snapshot; `MAX_HISTORY = 200`; `merge` folds edits with the same label | `session.rs:22,528-575` |
| Side effects | `file.*`; `material.save`/`load` (`.tcmat`); `export.preset.*`; `brush.preset.*`; `project.template.save`; `material.link.cook`, which **spawns `materialcraft-cli`**; `resource.import`; `bake.skew.set {path}`; `textureSet.meshMap.import` | `material_lib.rs:147-157`; `link_cmds.rs:232,254-258` |
| MCP and RPC | rmcp 3.5.1 with 22 tools. Headless runs through `spawn_blocking`; Bridge goes over JSON-lines TCP. Limits: 8 MiB requests, 16 connections, 256 batch steps. `authorize_command` refuses `file.*` and path parameters | `automation/src/server.rs:315,335-338`; `security.rs:14-25`; `workspace.rs:133-150` |
| Events | None; clients poll `Session.revision`, `SetCache.revision` and `take_uploads` | `session.rs:368,974` |
| Document | `Project` → `TextureSet`: resolution 128..8192, `channels: Vec<(Channel, ChannelFormat)>`, UDIM 1001..=2000, with per-tile resolutions. `Layer` holds kind, per-channel blend, mask, `content_effects` and `geometry_mask`. `Fill` holds sources, a projection and a `MaterialLink` | `core/src/project.rs:40-83,393-412`; `layer.rs:42-63,386` |
| Closed enums | `LayerKind` 4, `Projection` 6, `ProceduralKind` 23, `Source` 4, `GeneratorKind` 7, `Filter` 26, `EffectKind` 10, `Channel` 21, `ChannelFormat` 9, `BlendMode` 32, `BakedMap` 10, `Tool` 9, `BrushAlpha` 7, `DisplayKind` 8, `FileFormat` 8, `VirtualMap` 8 | `core/src/layer.rs:95,162,218,348,499,642,1008`; `channel.rs:7,235`; `blend.rs:9`; `bake/baked_map.rs:4` |
| Unknown data | A newer `format_version` is rejected. An unknown enum variant fails the load, because there is no `#[serde(other)]` in core; `recovery.rs` only salvages bake settings. Unknown fields are dropped. Unknown zip entries are ignored | `formats/src/project_file/mod.rs:365,434,466`; `recovery.rs:5-41` |
| Built-in libraries | 31 filter-catalogue entries, 93 smart materials, 24 smart masks, 18 brush presets, 10 export presets | `engine/src/filter_lib.rs:40-78`; `materials.rs:49-151`; `smart_masks.rs:28`; `paint/src/presets.rs:62`; `formats/src/export/presets.rs:85-200` |
| Compositor | CPU with rayon, on 64×64 u16 copy-on-write tiles in 16×16-tile blocks, with an f32 accumulator. **Neighbourhood filters declare a radius** and are precomputed over a region. The UI budget is 30 ms per frame. The open limit is 8 GiB. Up to 2^27 texels per set | `texel/src/tile.rs:6-24`; `compose/src/eval.rs:31,260-298,1973-1982`; `ui-egui/src/gpu.rs:109,189`; `project_open_limit.rs:4` |
| GPU | wgpu 30 (lock 30.0.1). The viewport and path tracer share eframe's device; the baker and headless mode create their own. There are 11 WGSL files (mesh, post, fx, util, shadow, displace, bake_view, rays, trace, trace_post, bake), 6 `@compute` entry points and 24 render pipelines. The BVH is built on the CPU with SAH and stored in storage buffers; there is no hardware ray query. The denoiser is a custom à-trous filter. The tracer allows at most 255 texture sets | `render/src/shaders/`; `gpu_trace.rs:26,305-308,756`; `gpu_bake.rs:71-84,113`; `renderer.rs:700,1101,1436-1474` |
| Baking | 10 `BakedMap`s. The `Tracer` trait (`bake/src/tracing/tracer.rs:4-7`) has CPU and GPU implementations; the GPU one is installed through a global `OnceLock`. `BakeRequest` takes high-poly, cage, skew and cage paint. Up to 4,096 rays. `docs/features.md:107` claims GPU bakes are about 12× the CPU | `bake/src/lib.rs:43,151-161`; `engine/src/bake/mesh_map_command.rs:67-72` |
| Painting | `BrushSettings`, mirror and radial symmetry, a particle brush, a CPU BVH for picking. **Pressure only scales size; tilt is unsupported** | `paint/src/brush.rs:138,263,281-321`; `stroke.rs:128-131`; `ui-egui/src/viewport.rs:1711` |
| Mesh | Its own `geom::Mesh` (positions, normals, Vec4 tangents, uvs, colours, indices, submeshes, materials), capped at 20M triangles. Import of 12 extensions: glTF via the `gltf` crate; FBX, OBJ, PLY and STL via craft-mesh; USD via craft-usd plus local code; COLLADA and Alembic are local. Export to OBJ+MTL, GLB and USD | `geom/src/mesh.rs:37-55`; `formats/src/mesh_import/mod.rs:44`; `mesh_export.rs:1-3` |
| Export | 8 formats; PNG/TIFF at 8/16 bit, EXR at 16/32; a `$udim` token; ICC v2 profiles through craft-cms | `formats/src/export/mod.rs:45,69,105-112,880`; `icc.rs:1-20` |
| UI | egui 0.37 with `craft_ui::docking` and 13 panels. Overlays cover the axis gizmo, symmetry, stencil, decal and path | `ui-egui/src/docking.rs:24`; `viewport.rs:495,789-849` |
| Plugins or scripts | None. The closest seams are the `Tracer` trait, `MaterialLink` and command ids | grep |

### 2.3 SoundCraft (Pro Tools class)

| Area | Observed | Evidence |
|---|---|---|
| Commands | **389** `cmd!` ids plus the hand-built `mix.insert_state`. By kind: 237 undoable, 122 `noundo`, 20 `query`, 9 `view`. By prefix: edit 94, track 66, event 35, clip 33, view 30, mix 29, setup 20, file 17, transport 13, session 11, automation 10, options 8, midi 6, markers 6, engine 6, video 3, audiosuite 1, app 1. The UI adds 71 | `crates/engine/src/cmd.rs:74-88,117-127,129-169`; `cmd/plugin_state.rs:11-22`; `ui-egui/src/menus.rs:12-84` |
| Undo and journal | `Arc<Session>` with copy-on-write. `execute` wraps the call in `catch_unwind` and rolls back on error or panic. `execute_merged`/`hold_merge` merge gestures. A journal of up to 10,000 (id, params) pairs is "for macro recording / agents", but nothing replays it | `crates/engine/src/lib.rs:34-37,182-213,435-532` |
| Side effects | `session.*` files, `file.import_*`, bounces (`file.bounce_mix`/`stems`), `audiosuite.process`, plugin rescans (`engine.clap_plugins`/`vst3_plugins {rescan}`), device setup. No network | `cmd/file.rs:14-194,315`; `cmd/clap.rs:11`; `cmd/vst3.rs:11`; `cmd/setup_more.rs:46-118` |
| MCP and control | Hand-rolled MCP (protocol 2025-06-18) with 17 tools and 2 resources. Loopback JSON-lines control channel, bounded, with `engine.execute`, `session.inspect` and `ui.*`. Request/response only; no events | `crates/automation/src/mcp.rs:7,25-106,214-228`; `docs/control-protocol.md:29,40-58` |
| Model | Lives in craft-libs `craft-scraft-format`. `Session` holds a tempo map, tracks (8 kinds: Audio, Aux, Master, Midi, Instrument, Vca, Folder, Video), busses, sends (10 per strip) and inserts (10). An `Insert` has `{plugin id, params map, bypass, preset, base64 state, sample}`. MIDI is at 960 PPQ. Files are `.scraft` (JSON) and `.scraftz`, up to 256 MiB | scraft-format `session.rs:348-385`, `track.rs:8-19`, `mixer.rs:6-7,425-526` |
| Realtime engine | Rendering happens **inside the cpal callback** (`guarded_block` → `AudioState::fill`). Block = callback × speed ratio, clamped to ≤512. The device rate is converted by linear interpolation. Commands arrive through `mpsc`; meters go out through `try_lock`. Strips render **in sequence on the audio thread**; they go parallel only offline with 4 or more strips | `crates/playback/src/{stream.rs:62-84,callback.rs:55-62,133-199,276-318,lib.rs:39}`; `crates/mix/src/threads.rs:34-42` |
| PDC | Per strip: the sum of insert latencies. Mix level: `compensate_delay` aligns to the slowest path, gated by `edit.delay_compensation`. Offline drops the latency head. `crates/playback` never mentions latency (grep) | `mix/src/strip.rs:62-64,337-356`; `mix/src/lib.rs:429-494`; `mix/src/offline.rs:87-94` |
| Allocation discipline | By convention only: no `assert_no_alloc` and no counting allocator. The audio thread allocates when the session structure syncs (`ensure_synced` → `sync`) and when a slot is resized | `mix/src/lib.rs:276-363`; `strip.rs:521` |
| Faults | A panic sets `failed`, stops the transport and zero-fills **all later blocks**. A NaN or Inf from a plugin mutes it for the block and takes it out of service. A 2-second stall counts as device loss. Xruns are only logged | `playback/src/callback.rs:36-62`; `mix/src/strip.rs:441-451`; `instances.rs` `PluginFailures` |
| Plugin trait | `prepare(sr, max_block, ch)`, `set_offline`, `reset`, `set_param(&str, f32)`, `process(&mut [Vec<f32>], frames)`, `latency`, `tail_samples`, `gain_reduction_db`, `set_transport`, `note_on`/`note_off`/`midi(offset, [u8;3])`, state, and editor. **One planar bus, no events list, no modulation** | `crates/dsp/src/lib.rs:195-294` |
| CLAP | clap-sys 0.5.0. Plugin extensions used: params, audio-ports, latency, note-ports, state, render, gui. Host extensions provided: log, params, latency, state, gui; thread-check, timer and fd are unsupported. **Parameter events are queued at `time: 0`**. Editors are floating; embedded only on Windows | `crates/clap-host/src/ffi/bundle.rs:215-221`; `ffi/host.rs:192-203`; `plugin.rs:288-299` |
| VST3 | `vst3` crate 0.3.0. `IParameterChanges`, `IEventList`, `IMidiMapping`, `IMessage`. Aux buses are kept inactive. No 64-bit processing. Editors on macOS and Windows | `crates/vst3-host/src/lib.rs:27-39`; `ffi/instance.rs:78-467` |
| AU | AUv2 only: no AUv3, no editor views, no sidechain | `crates/au-host/src/lib.rs:1-28` |
| Scanning | In-process and synchronous, from the UI thread | `vst3-host/src/registry.rs:33`; `clap-host/src/registry.rs:34-57`; `ui-egui/src/mix_window/plugin_menu.rs:131-155` |
| Instance path | `create_plugin` tries dsp, then clap, then vst3, then au. Instances are created on the control thread, handed to the audio thread with `adopt`, and dropped back on the UI thread. A refused state is reported | `crates/mix/src/instances.rs:60-126`; `mix/src/lib.rs:23-38` |
| Built-in DSP | 27 processors and 3 instruments, 26 of them usable as AudioSuite. Analysis: FFT, YIN pitch, LUFS and true peak, transients. Offline AudioSuite **refuses hosted plugins** | `crates/dsp/src/plugins.rs:20-49`; `cmd/audiosuite.rs:60`; `dsp/src/offline.rs:79-797` |
| Automation | 7 modes: Off, Read, Touch, Latch, TouchLatch, **Write** and **Trim**. Targets: volume, pan, mute, send level/pan/mute, and `Plugin{slot, param}`. Lanes are linear breakpoints | scraft-format `automation.rs:7-65,138-175`; `mix/src/automation.rs:179-193` |
| MIDI | Files and editing only. No device I/O, MIDI learn, MPE, MIDI 2.0, clock, Mackie, HUI or OSC (gaps G-02, G-10, G-11, G-20) | `crates/midi/src/lib.rs:1-20`; `docs/gaps.md` |
| macOS signing | `entitlements.plist` grants only `device.audio-input`; there is no `disable-library-validation`, so third-party VST3s are refused (G-05) | `packaging/macos/entitlements.plist:10` |
| Sources | Decoded whole into RAM (G-06) | `docs/gaps.md:23` |
| UI | egui 0.37 with `craft_ui::docking` and Mix/Edit windows. Plugin editors are separate native windows; built-ins get generic egui editors | `ui-egui/src/docking.rs:4-7`; `panels/plugins.rs:4,50` |

### 2.4 craft-libs (shared crates relevant to these apps)

| Crate | What it gives plugins | Used by | Gap |
|---|---|---|---|
| `mesh` (9.4k lines) | `Scene`, `Mesh`, `Material` and `TextureRef`; STL, OBJ, PLY, glTF, FBX and 3MF readers (`src/lib.rs:229,432,680`) | MaterialCraft, TextureCraft | No tangents/submesh parity with TextureCraft's `geom::Mesh`; no writer for every format |
| `texture` (2.4k) | Linear f32 `Image` planes, sampling, normal/ORM packing (`src/lib.rs:148`) | MaterialCraft | CPU only |
| `materialx` (619) / `mcmat` (216) / `usd` (448) | `.mtlx` parse and write with an OpenPBR writer; the `.mcmat` zip; USD crate primitives | TextureCraft (and MaterialCraft for mcmat) | No MaterialX nodedef model, no USD stage |
| `dds` (2.2k), `exr`, `hdr`, `tga`, `image-codecs` | Texture I/O; BC1–BC5 encode | various | No KTX2, BC6H or BC7 |
| `audio` (68k) | Codecs, `AudioBuffer`, peaks | SoundCraft | No DSP, no device I/O, no resampler for general use |
| `midi` (3.5k) / `sf2` (510) / `time` (3.5k) | SMF and note model; SoundFont reader; ticks and `TempoMap` | SoundCraft (midi, time) | No live MIDI, no UMP or MIDI 2.0 |
| `ui` (53k) | `DockArea`, the `DockContent` trait (`src/docking/surface.rs:837,1099`), widgets, a 97-line theme palette | all three | No node-graph widget, curve or gradient editors, knob or meter widgets |
| `keymap` | `CommandRef {id, params}` (`src/command.rs:14`) | MaterialCraft, TextureCraft | Not a registry |
| `control`, `control-protocol`, `jsonrpc`, `mcp`, `mcp-server`, `ipc` | Loopback control hub; bounded JSON-lines; an MCP `Tools` trait; Craft Link IPC with bulk frame transfer (`ipc/src/bulk.rs:174,305`) | none of the three | Duplicated MCP stacks; `ipc` bulk transfer could carry the native-bridge pixel and audio rings |
| `ai` / `ai-core` | OpenAI-compatible cloud provider; local GGUF LLMs on candle 0.9 on the CPU | none | No ONNX, no image or audio models, no GPU |
| `wasm` | Module inspection only, "never executes" | none | — |
| `cms` | ICC transforms; `GpuMatrixShaper` and `Lut3d` as data | TextureCraft | — |

**Missing shared pieces:**

- GPU (device, pool, WGSL);
- realtime audio (events, rings, RT thread pool);
- DSP;
- MIDI I/O;
- a command registry with schemas;
- a node registry or graph core;
- a node-editor widget;
- ML inference beyond LLMs.

The `AGENTS.md` scope rule is "useful to more than one app, no app code". Every item here qualifies.

---

## 3. What plugins need from these apps: the shared contracts, refined

### 3.1 `graph.node@1` against the two real document models

MaterialCraft's graph and TextureCraft's stack differ. The contract therefore defines **node types**, and each app decides where they appear.

```text
NodeType {
  id: "pkg.id/node"                 # stable, namespaced
  version: semver
  digest: sha256(package)
  domains: [material, texture-fill, texture-mask, texture-filter, texture-generator, effect, photo]
  inputs/outputs: [{id, kind, optional, colorspace?}]
  params: [ParamDef]
  exec: graph | program | wgsl | wasm
  neighbourhood: {radius_px?: f32, whole_map?: bool}   # TextureCraft tiles need it
  determinism: exact | tolerant(ulp|abs)              # see §6.2
  size: from_parent | fixed | from_input(id)
}
```

**Port kinds.** v1 port kinds follow MaterialCraft's actual `PortKind` and `Value`:

- `gray`, `color`;
- `value` (typed: float, int, vec2–4, bool, string, curve, gradient, enum).

v1.1 adds:

- `mesh` (a `mesh@1` handle);
- `points` (instance lists, the FX-map output);
- `material` (a named channel bundle keyed by TextureCraft's `Channel` names, `core/src/channel.rs:7`).

**Parameters.** `ParamDef` reuses the shape of MaterialCraft's `ParameterDefinition` plus `Value`, so the host draws parameter UI. Plugins never draw parameter panels for nodes.

**Cook context.** It maps 1:1 to MaterialCraft's `CookContext {format, w, h, seed, tile, inputs, params, resource, vars}` (`crates/nodes/src/cook_context.rs`). For TextureCraft it adds:

- the texture set's resolution and UDIM tile;
- implicit inputs `mesh_maps.{ao, curvature, position, normal, id, thickness, height, bent_normal}`, which are TextureCraft's `BakedMap` set.

**Execution tiers:**

| Tier | MaterialCraft today | TextureCraft today | Runs where |
|---|---|---|---|
| `graph` | **Already works:** SBS graphs in the library folder | none (fills can link `.mcmat`) | host |
| `program` | MaterialCraft function VM (85 ops); seed `kernel.program` from it | — | host CPU threads, or translated to WGSL |
| `wgsl` | none | none | shared GPU layer (§5.4 CL-2) |
| `wasm` | none | none | Wasmtime on desktop; for scatter and analysis, it emits `kernel.*` and `points` |

**Cache key.** `(digest, id, version, params, size, seed, tiling, input keys, backend-class)`. It slots into MaterialCraft's existing hasher (`eval/src/node_execution.rs:148-160`). `backend-class` is `cpu-ref` or `gpu:<vendor-family>`, and is needed only for nodes marked `tolerant`.

### 3.2 `mesh@1` (one shared type)

The suite has two mesh types today. craft-libs `Mesh` is used by MaterialCraft. TextureCraft's `geom::Mesh` has tangents, colours, submeshes and materials.

**Proposal.** Grow `craft-mesh` with a render-ready `MeshData`, and have both apps adopt it:

- positions `f32x3[]`;
- normals `f32x3[]?`;
- tangents `f32x4[]?` (w = handedness);
- `uv: [f32x2[]; ≤4]`, where UDIM is the integer part of UV0;
- colours `f32x4[]?`;
- indices `u32[]`;
- `submeshes [{name, material, start, count}]`;
- optional `face_sizes u8[]` for retopology tools that need polygons.

**Plugin-side mapping.** In plugins, `mesh@1` is a **host handle** (`mesh.read_region{attr, range}`, `mesh.create` in jobs), never a JSON blob. Triangle caps come from the host, so a plugin sees one limit and the app's cap (4M or 20M) stops being an inconsistency.

### 3.3 GPU-resident textures

```text
TextureHandle {
  id, generation,
  size [w,h], udim?,
  format: R16Unorm | R16F | R32F | RGBA8 | RGBA16F | RGBA32F,
  colorspace,
  residency: Gpu | Cpu | Both
}
```

Ops: `image.alloc`, `image.read_region`, `image.write_region`, `image.view` (a WGSL binding), `image.release`.

The host owns a **budgeted pool**:

- 70% of `adapter.limits` and the reported VRAM;
- LRU eviction to CPU, as u16 or f16, matching TextureCraft's storage;
- tiled views when a texture exceeds `max_texture_dimension_2d` or `max_storage_buffer_binding_size`.

The WebGPU default limits are 8192 for 2D textures and 128 MiB for storage-buffer bindings (https://www.w3.org/TR/webgpu/#limits). TextureCraft already reads `max_texture_dimension_2d` (`render/src/renderer.rs:1440`).

### 3.4 Audio contracts (summary; detail in `audio.md`)

The contracts are `audio.effect@1`, `audio.instrument@1` and `audio.note-fx@1`, with CLAP semantics over Wasm:

- sorted sample-offset events;
- `param_value` separate from `param_mod`;
- gestures;
- note IDs and expressions;
- MIDI 1, MIDI 2 (UMP) and sysex;
- buses, including sidechain;
- latency through `request_restart`;
- state;
- meter and scope rings.

One correction to `audio.md`: SoundCraft's host already sends `vst3:` events with `IParameterChanges` support, but CLAP events go at `time: 0` (`clap-host/src/plugin.rs:288-299`). The engine must produce event lists first (§5.3 SC-2); the hosts are then a small change.

---

## 4. Plugin catalogue

How to read the status column:

- **A** is the app engine, **D** the adapter, **P** the platform (craft-extensions and craft-libs).
- **Y** means present, **~** partial, **N** missing.
- "Gap" names the first blocking layer and its location.
- MC is MaterialCraft, TC TextureCraft, SC SoundCraft.

### 4.1 Materials, texturing and 3D (52 items)

| # | Item | Function | Host capability needed | Status | Gap location | Proposed contract / op / hook |
|---|---|---|---|---|---|---|
| 1 | Substance Designer Python plugins (Node Snippets, graph-layout tools) [1][2] | Persistent tools: read and edit graphs, menus, panels | Graph query/edit, selection, menus, dockable panel, change events | A:Y D:N P:~ | MC commands have no schema or class (`commands.rs:12-23`); no events | `materialcraft.session@1` generated from the registry; `app.run`; `document.changed` |
| 2 | Designer custom nodes as `.sbs` graphs | New node = sub-graph plus exposed parameters | Graph-as-node packaging, install, versioning | A:Y D:N P:N | Packaging only; library loads from a folder (`node_library.rs`) | `resource.node-library@1` (data tier): SBS files in the package, mounted into the library |
| 3 | Designer Pixel/Value Processor | Per-pixel and per-value function graphs | Function graph IR | A:Y (85 ops) D:N P:N | — | `kernel.program` seeded from the MC function VM |
| 4 | Designer FX-Map | Iterated scatter or stamp of patterns | Instance list → stamping kernel | A:Y (`fx_map`) P:N | Plugin code cannot emit instances | `graph.node exec=wasm` → `points` → `kernel.tile_sample` |
| 5 | Designer MDL graphs | Shading-model graphs | Shader-model codegen | A:N | No material shading graph | `material.shader@1` (P3) |
| 6 | SAT / pysbs batch cook [3] | Headless cook, parameter variation, export | Headless cook and export with overrides | A:Y (CLI `cook`/`export`) D:N | No plugin job path | `job` plus `app.run graph.cook` and `file.exportMaterial` through a picker grant |
| 7 | `.sbsar` (Substance Engine) | Run Adobe compiled materials | Closed engine | A:N | Licensing | Stays native or out of scope |
| 8 | Sampler plugins (Python + QML) [4] | Panels and automation over layer stacks | Panel UI, stack ops | A:Y (TC) D:N | — | `ui` tree and `texturecraft.session@1` |
| 9 | Sampler custom filters (Designer graphs) [5] | Graph-defined image-to-material filters | Graph node in a fill or filter slot | A:~ (TC `MaterialLink` spawns the MC CLI) | `link_cmds.rs:232` | `graph.node` domain `texture-filter`, cooked in-process |
| 10 | Sampler and Firefly image/text to material | Photo or prompt → PBR set | ML inference or a cloud API | A:N P:N | No `nn.*`, no `net.fetch` | `nn.run` (craft-nn) or `net.fetch` plus `secret.*` |
| 11 | Painter Python plugins (export automation) [6] | Layer and export automation, menus, events | Layer/mask/effect/export ops, events | A:Y (183 commands) D:N | No schema or class; no events | `texturecraft.session@1`, `app.run`, events |
| 12 | Painter live links (Send to Unreal, Unity, Blender, Marmoset) | Push textures and mesh to another app | Local IPC or handoff | A:~ D:N P:N | No `net.local` | `handoff.open`, or `net.local` with consent |
| 13 | Painter smart materials | Reusable layer stacks | Data import into the stack | A:Y (`.tcmat`, 93 built in) | Install path | `resource.smart-material@1` |
| 14 | Painter smart masks | Reusable masks | Same | A:Y (24) | — | `resource.smart-mask@1` |
| 15 | Painter generators (mesh-map driven) | Edge wear, dirt from curvature and AO | Per-texel node with mesh-map inputs | A:~ (`GeneratorKind` 7, closed) | `layer.rs:499` | `GeneratorKind::Plugin` + `graph.node` with `mesh_maps.*` |
| 16 | Painter filters | Channel filters | Filter slot, neighbourhood radius | A:~ (`Filter` 26, closed) | `layer.rs:642` | `Filter::Plugin` with declared `neighbourhood` |
| 17 | Painter particle brushes (PopcornFX) | Physics particles that deposit paint | Particle simulation, mesh ray casts, dab output | A:~ (particle brush `brush.rs:263`) | No sim hook | `brush.particles@1` (Wasm sim + `mesh.raycast` + `brush.dab` output) |
| 18 | Painter custom GLSL viewport shaders | Custom viewport shading of channels | Shader function inside the mesh pass | A:N | `mesh.wgsl` is fixed | `viewport.shader@1`: a WGSL function `fn shade(ch: Channels, g: Geometry) -> vec4f` |
| 19 | Painter export presets | Channel packing and naming | Export templates, new formats | A:Y (10 presets, user JSON) A:~ formats closed | `export/mod.rs:69,139` | `resource.export-template@1`; `format.export@1` |
| 20 | Painter brush alphas, stencils, presets | Brush resources | Data | A:~ (`BrushAlpha` 7 closed, 18 presets) | `paint/src/brush.rs:22` | `resource.brush-alpha@1`, `resource.brush-preset@1` |
| 21 | Substance add-ons in other DCCs | Run our materials elsewhere | Portable material | A:Y (`.mcmat`, CLI) | — | Keep CLI and `.mcmat`; no plugin needed |
| 22 | Blender Node Wrangler [7] | Graph editing helpers, PBR setup from a folder | Graph edits, file pick, preview | A:Y D:N | — | session + `files.pick` + `graph.node.add`/`connect` |
| 23 | BlenderKit [8] | Asset browser, download, auth | Network, secrets, import material, mesh, HDRI | A:Y import P:N | `net.fetch`, `secret.*` missing | `net.fetch` (`network:<host>`), `secret.*`, `resource.import` |
| 24 | Poliigon add-on [9] | Same, with a subscription | Same plus OAuth | P:N | Same | Plus `auth.oauth` |
| 25 | Fab / Quixel Bridge plugin [10] | Megascans browse and import | Same | P:N | Same | Same |
| 26 | Poly Haven / Extreme PBR packs | Material and HDRI packs | Data | A:Y | Install | `resource.material@1`, `resource.environment@1` |
| 27 | UVPackmaster 3 [11] | GPU UV packing (closed engine) | Read/write mesh UVs, heavy compute | A:~ (TC auto-unwrap `geom/src/unwrap.rs:74`) | No mesh write op | Native restricted helper + `mesh.uv.write`; or a WGSL compute job |
| 28 | Zen UV, TexTools, Magic UV [12][13] | UV editing, seams, packing helpers | UV ops, overlays, selection | A:~ | No UV edit commands, no overlay | `mesh@1` write in jobs + `viewport.overlay@1` |
| 29 | Texel Density Checker [14] | Texel-density analysis and paint | Mesh read, colour overlay | A:N | Overlay | `mesh.read_region` + `viewport.overlay@1` (per-face colours) |
| 30 | Material Combiner [15] | Atlas multiple materials | Mesh UV rewrite, texture repack | A:N | Mesh write | `mesh.create` + image handles |
| 31 | DECALmachine / MESHmachine [16] | Trim and info decals, bevels | Projected decals, mesh ops | A:~ (TC decals, `Projection` 6) | Closed `Projection` | `resource.decal@1`; `Projection::Plugin` (P3) |
| 32 | Hard Ops / BoxCutter [17] | Interactive boolean modelling | Mesh editing, gizmos | A:N (not in TC/MC) | Out of scope (SolidCraft) | `viewport.tool@1` (SolidCraft) |
| 33 | Ucupaint [18] | Layered painting in Blender | (shows the layer API a painter needs) | A:Y | — | Reference for `texturecraft.session` coverage |
| 34 | SimpleBake / Principled Baker [19] | Bake automation and channel packing | Bake trigger, settings, export | A:Y (14 `bake.*`) D:N | — | session + `job` + export templates |
| 35 | Blender custom bake passes / Bake Wrangler | Custom bake maps | Per-texel ray queries | A:~ (`Tracer` trait, `BakedMap` closed) | `bake/baked_map.rs:4` | `texture.bake-map@1` + `bake.rays` over the existing `Tracer` |
| 36 | xNormal map-renderer plugin SDK [20] | Plugin bakers (historic precedent) | Same as 35 | — | — | Same |
| 37 | Dream Textures, AI-Render [21][22] | Stable Diffusion textures, project onto mesh | `nn.run` or cloud; camera projection fill | A:~ (TC camera projection) P:N | No inference | `nn.run` + `layer.fill.fromImage{projection:camera}` |
| 38 | Quad Remesher, RetopoFlow, Instant Meshes [23][24] | Retopology for a bake low-poly | `mesh.create`, heavy compute | A:N | Mesh write, job memory | Wasm JIT job + `mesh@1` handles |
| 39 | Geometry Nodes assets, Geo-Scatter, Sverchok [25][26] | Procedural geometry graphs | Mesh and points ports | A:N | MC `PortKind` has no mesh | `graph.node` v1.1 `mesh`/`points` ports (P2) |
| 40 | Cycles OSL Script node [27] | User shader code in a node | Shader-code node | A:N | No GPU cook in MC | `graph.node exec=wgsl` |
| 41 | fSpy importer [28] | Camera match from a photo | Camera set | A:Y (TC 17 `camera.*`) | — | session |
| 42 | Houdini HDAs [29] | Node = subnet + parameters | Graph-as-node | A:Y (MC SBS instance) | Packaging | `resource.node-library@1` |
| 43 | Copernicus OpenCL COP [30] | Custom GPU kernels on image layers | Sandboxed GPU kernel | A:N P:N | No shared GPU layer | `graph.node exec=wgsl` on CL-2/K7 |
| 44 | SideFX Labs Maps Baker [31] | Bake inside a node graph | Bake as a node | A:N (MC) A:Y (TC) | MC has no baker | Shared `craft-bake` (CL-4) + `texture.bake-map@1` |
| 45 | Labs Trim Texture, Texture Sheet | Trim sheets, flipbooks | Mesh plus graph outputs | A:N | `mesh` ports | `mesh@1` + `graph.node` |
| 46 | Labs Auto UV, Edge Damage, Detail Mesh | Mesh processing | Mesh jobs | A:~ | Mesh write | `mesh.create` jobs |
| 47 | Houdini Engine [32] | Run HDAs inside other hosts | Licensed native engine | A:N | Vendor | Native full helper + `format.import@1` (mesh) |
| 48 | Material Maker shader nodes [33] | GLSL snippets fused into one shader | WGSL function nodes, fusion | A:N | No GPU cook | `graph.node exec=wgsl`; the host fuses chains |
| 49 | ArmorPaint plugins and custom nodes [34] | Scripted panels and project ops; node extensions | Session, UI tree | A:Y D:N | — | session + `graph.node` |
| 50 | Quixel Mixer (final 2023.1 release, discontinued Feb 2026) [35] | Scan blending, smart materials | Migration of user libraries | A:N | Closed format | `format.import@1` for exported texture sets; a migration story to win users |
| 51 | Marmoset Toolbag `mset` Python (incl. `bakeAll`, MCP bridge) [36] | Bake and render automation | Bake ops, render | A:Y (TC bake, `camera.*`) | — | session; MCP already exists |
| 52 | Marmoset custom shaders, Unreal Custom HLSL, Unity Custom Function, MaterialX `<implementation>` genglsl [37][38][39] | Inline shader code nodes | WGSL function tier, typed nodedefs | A:N | MC has no MaterialX (stub tab); no GPU cook | `graph.node exec=wgsl`; `format.import@1` that maps MaterialX nodegraphs to the graph tier; `format.export@1` for OpenPBR from MC |

Items not in a numbered row (reasons in brackets):

- OIDN/OptiX denoisers (native or `nn.run`; TC has à-trous only);
- NVIDIA Texture Tools / KTX2 / BC7 (`format.export@1`; craft-libs `dds` encodes BC1–BC5 only);
- RizomUV, ZBrush GoZ and 3D-Coat AppLink bridges (native full or `handoff.open`);
- Unreal Material Functions and Layers (graph tier);
- NVIDIA MDL (`material.shader@1`, P3).

These are also tracked in §5.

### 4.2 Audio products (25 items)

| # | Item | Function | Host capability needed | Status | Gap location | Proposed contract / op / hook |
|---|---|---|---|---|---|---|
| 53 | FabFilter Pro-Q 4 | Dynamic and linear-phase EQ, spectrum, instance list | RT, latency, sidechain, analyzer ring, cross-instance messages, 60 fps canvas | A:~ native hosting; sidechain N | No sidechain (`vst3-host/src/lib.rs:37-39`) | Native via the bridge; Wasm class: `audio.effect@1` + `scope.write` + `service.call` |
| 54 | FabFilter Pro-L 2 / Pro-C 3 | Limiter and compressor, oversampling, true peak | RT, latency, sidechain, gain-reduction meter | A:~ (`gain_reduction_db` exists) | Sidechain | Same; host `audio.oversample` kernel |
| 55 | FabFilter Saturn 2, Timeless 3, Volcano 3 | Modulation-matrix effects | Tempo sync, `param_mod` | A:~ (transport in `set_transport`) | No modulation | `param_mod` events |
| 56 | iZotope RX 11 | Spectral repair, ML, ARA | Clip random access, offline ML, spectral canvas | A:N | No ARA, no `nn` | `audio.clip-editor@1` + `nn.run` jobs |
| 57 | iZotope Ozone 12 / Neutron 5 | Assistants, inter-plugin unmasking | Offline analysis, cross-instance messages | A:~ | — | `service.call`, analysis jobs |
| 58 | Waves catalogue (SSL, CLA) incl. Clarity Vx Pro | Character and channel processing; neural denoise | Native licence daemon; RT ML | A:~ native | Licensing | Native via the bridge; `ml.stream` later |
| 59 | Soundtoys (EchoBoy, Decapitator, Little AlterBoy) | Character delay and saturation | RT, tempo sync | A:Y native | — | Native; Wasm-class ideal |
| 60 | Valhalla VintageVerb, Supermassive, Delay | Algorithmic reverb and delay | RT | A:Y native | — | Same |
| 61 | oeksound soothe2 [40] | STFT resonance suppression | RT, latency, sidechain | A:~ | Sidechain | `audio.effect@1` + `audio.fft` kernel |
| 62 | Xfer Serum 2 | Wavetable synth, MPE | Instrument, note expressions, file import | A:~ (notes, no MPE) | No MPE (grep) | `audio.instrument@1` + `note_expression` + `files.pick` |
| 63 | Vital | Wavetable synth (GPL), GPU UI | Same, plus a GPU canvas | A:~ | Same | Same; plugin UI via a canvas |
| 64 | NI Kontakt 8 + KSP, NKS | Multi-GB streaming sampler, scripting | Disk streaming, large memory, NKS metadata | A:~ native | G-06 (sources whole in RAM) | Native via the bridge; `sample.stream`; NKS-style `resource.preset` metadata |
| 65 | NI Reaktor 6 / Blocks | Modular instruments | Plugin graph, `param_mod` | A:~ native | — | Native; a Wasm modular host is a later tier |
| 66 | NI Guitar Rig 7, Neural DSP Archetype | Amp simulation, ML captures | RT, low latency | A:Y native | Small mix block for monitoring (AP-9) | Native or `ml.stream` |
| 67 | Spitfire BBC SO / LABS, Spectrasonics Omnisphere, Toontrack Superior Drummer 3 | Huge libraries; drag MIDI grooves to the host | Streaming, drag of MIDI to the timeline | A:~ | G-06; no plugin→host MIDI drag | `sample.stream`; `clip.drop{midi}` host op |
| 68 | Celemony Melodyne 5 (ARA2) [41] | Note-level audio editing | ARA2 | A:N | G-13 | Native ARA2 in the bridge + `audio.clip-editor@1` |
| 69 | Antares Auto-Tune Pro 11 | Live pitch correction; ARA Graph mode | RT low latency; ARA | A:~ | ARA | Wasm-feasible live mode; ARA via the bridge |
| 70 | Arturia V Collection / Pigments | Analogue modelling, heavy CPU | RT, multicore | A:Y native | **Strips render in sequence on the audio thread** (`threads.rs:38`) | RT worker pool (SC-6) |
| 71 | u-he Diva, Zebra, Repro (CLAP polyphonic modulation) | Synths | CLAP `param_mod` per note, voice info | A:~ | Events at `time:0` (`clap-host/src/plugin.rs:288-299`) | Event lists (SC-1/SC-2) |
| 72 | UAD Native / UAD-2 hardware | Hardware DSP and licensing | Native full | A:~ | — | Native via the bridge |
| 73 | Kilohearts Phase Plant + Snapins | Plugin hosting inside a plugin | Nested hosting | A:Y native | — | Native; Wasm-side `service.call` to host built-ins |
| 74 | Cableguys ShaperBox 3 | Drawn LFO shapes, sidechain-triggered | `param_mod`, sidechain, curve UI | A:~ | Sidechain, curve widget | `audio.effect@1` + `ui` curve |
| 75 | Output Arcade | Online loop library, streaming | Network, sample streaming | A:N | `net.fetch`, `sample.stream` | Both |
| 76 | Steinberg SpectraLayers, Demucs/HTDemucs, RipX | Stem separation, spectral editing | Offline ML, clip access | A:N | No `nn` | `nn.run` job + `audio.clip-editor@1` |
| 77 | Synthesizer V / ACE Studio | AI vocal synthesis, lyrics, ARA | Clip access, MIDI read, `nn` | A:N | — | `audio.clip-editor@1` + `midi.read` |

### 4.3 DAW extension systems, scripting and controllers (15 items)

| # | Item | Function | Host capability needed | Status | Gap location | Proposed contract / op / hook |
|---|---|---|---|---|---|---|
| 78 | REAPER JSFX [42] | EEL2 DSP scripts, live edit, `@gfx` UI | Compile at load, RT, canvas | A:N P:N | No Wasm RT tier | Faust/Cmajor/"jsfx-lite" → Wasm + `audio.effect@1`; hot reload |
| 79 | REAPER ReaScript + ReaImGui [43] | Lua, EEL or Python automation over the whole API; immediate-mode GUIs | Full command surface, UI | A:Y (389 commands) D:N | No schema or permission class | `soundcraft.session@1` + `app.run` + `ui` tree |
| 80 | REAPER extensions (SWS, ReaPack, js_ReaScriptAPI) [44] | Native API extensions; package manager | Native API and repositories | A:N | — | Platform store and catalogue; native helpers only for OS hooks |
| 81 | Ableton Max for Live + Live Object Model [45] | Devices with access to the session object model | Session query/observe, device DSP | A:~ D:N | No observers (request/reply only) | events + `audio.*` + `soundcraft.session@1` observers |
| 82 | Ableton Live 12 MIDI Tools | MIDI transformations and generators on clips | MIDI clip read and edit | A:Y (`event.*` 35, `midi.*` 6) | — | `midi.transform@1` (main thread, undoable) |
| 83 | Ableton MIDI Remote Scripts (Python) | Controller mapping | MIDI device I/O | A:N | G-02 | `control.surface@1` (below) |
| 84 | Cycling '74 RNBO [46] | Exports patches to VST3, Web Audio (Wasm) and C++ | Load Wasm DSP | P:N | RT tier | Accept RNBO Wasm through a template adapter |
| 85 | Bitwig Grid and modulators [47] | Modular device graph; modulators | `param_mod`, per-voice | A:N | No modulation | Host modulators + `param_mod` (P3) |
| 86 | Bitwig Controller API + DrivenByMoss [48][49] | JS/Java controller extensions | MIDI I/O, observers, transport | A:N | G-02, no events | `control.surface@1` |
| 87 | Logic Pro Scripter (JS MIDI FX) [50] | Per-event MIDI scripting, RT | Note-fx at sample offsets | A:~ (MIDI at sample offset `mix/src/midi.rs:69,87`) | No note-fx contract | `audio.note-fx@1` |
| 88 | Cubase MIDI Remote API (JS), Logical Editor [51] | Controller scripting; rule-based edits | MIDI I/O; batch edits | A:~ | G-02 | `control.surface@1`; `app.run` batches |
| 89 | FL Studio Python MIDI scripting, VFX Script [52][53] | Controller scripts; Python MIDI generators | MIDI I/O; note-fx | A:N | G-02 | `control.surface@1`, `audio.note-fx@1` |
| 90 | Audacity Nyquist + OpenVINO AI plugins [54][55] | Lisp effects and analyzers; local ML separation, denoise, transcription | Offline clip processing, `nn` | A:~ (AudioSuite exists; refuses hosted plugins `audiosuite.rs:60`) | No offline plugin path | `audio.offline@1` job + `nn.run` |
| 91 | Pro Tools AudioSuite/AAX + PTSL (gRPC scripting) | Offline processing; external automation | Render-in-place; RPC | A:~ (MCP and control) | — | Already near PTSL via MCP; add events |
| 92 | Reason Rack Extensions, VCV Rack, plugdata, WebCLAP/WAM2, Faust/Cmajor [56]–[61] | Sandboxed and portable DSP formats; modular patches | Wasm RT tier, CLAP-over-Wasm | P:N | Platform RT engine | `.wclap` import adapter; Faust/Cmajor templates. **Rack Extensions are the precedent for sandboxed RT plugins in a commercial DAW** (vendor-compiled bytecode run in a host sandbox) |

**Controllers.** These sit with row 86: Mackie Control, HUI, OSC (TouchOSC), NI NKS / Komplete Kontrol, Ableton Push, and MPE or MIDI 2.0 devices (ROLI, LinnStrument) [62][63]. They need:

- MIDI and OSC device I/O;
- a controller-script tier that observes the session;
- MPE and UMP in the event model.

None exists today (`docs/gaps.md` G-02, G-10, G-11, G-20). Proposal `control.surface@1`:

- **Process and timing:** a Wasm session plugin that is granted named MIDI and OSC ports, receives `midi.in` events at 1 kHz, and calls `app.run` and the observers.
- **Feedback:** it writes feedback (LEDs, motor faders) through `midi.out`.
- **Device I/O:** the host owns the device I/O (CL-7).

Total: 92 rows, several of which group 2–5 named products (over 120 products named).

---

## 5. Per-app change lists

Priorities and effort follow the program document:

- **Priority.** P0 blocks a plugin class, P1 blocks quality or speed, P2 is parity, P3 is later.
- **Effort.** S is up to a week, M one to four weeks, L more than four weeks.

"Layer" is App, Adapter or Platform; platform items are in §5.4 and §5.5.

### 5.1 MaterialCraft

| ID | Change | Layer | P | E | Depends on |
|---|---|---|---|---|---|
| MC-1 | **Open the node registry.** Replace `catalog()`'s `OnceLock<Vec<NodeDefinition>>` and `Cook = fn(...)` with a `NodeRegistry` snapshot, an `Arc` swapped on install or uninstall. Entries are `{id: String, version, source: Builtin \| SbsLibrary{package} \| Plugin{package, digest}, ports, params, cook: Arc<dyn NodeCook>}`. `node_execution.rs:166-223` dispatches by registry lookup; built-ins wrap their `fn` pointers. The cache key adds `source.digest` | App | P0 | M | CL-3 |
| MC-2 | **Encode plugin nodes in SBS.** Write plugin nodes as `compInstance` of a dependency URL `pkg://craft-plugin/<package>/<node>?v=<semver>&d=<digest>`, with raw parameters kept. Unknown entries reuse the existing `missing_node` placeholder with `raw_params` (`formats/src/sbs/placeholder.rs`). Substance Designer then sees a missing dependency, not corrupt XML | App | P0 | S | MC-1 |
| MC-3 | **Library folder → data contribution.** The SBS library folder (`MATERIALCRAFT_SBS_DIR`, `library.reload`) becomes `resource.node-library@1`, with SBS files mounted read-only from the plugin package. This is the first working MaterialCraft plugin type, with no code execution | App + Adapter | P0 | S | adapter skeleton |
| MC-4 | **Classify commands.** Add `kind: Query \| Edit \| View \| SideEffect`, a `permission` class and a JSON-schema `params` to `CommandSpec` (`commands.rs:12-23`) through CL-1. Generate the classification table for all 133 ids; refuse `file.*` and `*.loadMesh`/`input.bind {path}` as plugin ops (path portals instead) | App | P0 | S | CL-1 |
| MC-5 | **Change events.** Emit `document.changed {revision, changes:[{graph, node?, kind}]}` from `Session::edit*` (`session.rs:219-306`), plus `cook.finished {graph, node, ms}` from the worker (`ui-egui/src/cook.rs:148`) | App | P0 | S | — |
| MC-6 | **`materialcraft.session@1` adapter.** On `Runtime<A>` (program §2.2): ops `package.info`, `graph.inspect`, `node.preview{handle}`, `app.run`; node contributions under Library ▸ Plugins | Adapter | P0 | M | MC-1, MC-4, MC-5, platform runtime |
| MC-7 | **Typed ports.** Extend `PortKind` with `Mesh`, `Points` and `MaterialBundle`; `Value` is already typed | App | P1 | M | CL-2 (mesh) |
| MC-8 | **GPU cook backend.** wgpu compute for image ops, with `NodeOutput::Texture(TextureHandle)` and lazy readback. CPU stays the reference (an `AGENTS.md` rule). Needs an `AGENTS.md` layering exception so `eval` can use a craft-libs GPU crate | App | P1 | L | CL-2/CL-3, an owner decision |
| MC-9 | **WGSL and program tiers.** Expose the function VM as `kernel.program`; WGSL nodes on the shared GPU layer | Adapter + Platform | P1 | M | MC-8, K7 |
| MC-10 | **Shared mesh.** Adopt `craft-mesh::MeshData`; lift the 4M-triangle cap to the shared host cap | App | P1 | S | CL-2 |
| MC-11 | **Bakers.** Adopt the shared `craft-bake` (TextureCraft's `bake` crate moved to craft-libs) for mesh-based generators; this closes `docs/gaps.md` #3 | App | P2 | L | CL-4 |
| MC-12 | **GPU 3D view** (`docs/gaps.md` #2) with `viewport.overlay@1` and `viewport.shader@1` hooks | App | P2 | L | CL-2 |
| MC-13 | **Format hooks.** `format.export@1` (MaterialX/OpenPBR fills the stub tab) and `format.import@1` (MaterialX nodegraph → SBS graph) | Adapter | P2 | M | MC-6 |

### 5.2 TextureCraft

| ID | Change | Layer | P | E | Depends on |
|---|---|---|---|---|---|
| TC-1 | **Tolerant decoding.** Wrap every document enum in `Known(T) \| Unknown(serde_json::Value)`, using a custom `Deserialize` or `#[serde(untagged)]` with a raw fallback. Add `#[serde(flatten)] extra: Map<String, Value>` to `Layer`, `Effect`, `Fill` and `TextureSet`, so newer data survives open and save. Today one unknown `EffectKind` fails the load (`project_file/mod.rs`; no `serde(other)` in core) | App | P0 | S | — |
| TC-2 | **`Plugin` variants.** Add a `Plugin(PluginRef{id, version, digest, params})` variant to `EffectKind`, `Filter`, `ProceduralKind`, `GeneratorKind` and `Source`; later to `BakedMap`, `BrushAlpha`, `BlendMode` and the export `FileFormat`/`VirtualMap`. Missing plugins render pass-through **with a visible badge** and stay in the document | App | P0 | M | TC-1, CL-3 |
| TC-3 | **Compositor plugin path.** Pointwise nodes run per 16×16-tile block (1024² texels, `compose/src/eval.rs:1978`). Nodes with `neighbourhood.radius` join the existing neighbourhood-filter precompute (`eval.rs:260-298`). `whole_map` nodes cook once per set and are cached by key. u16→f32 conversion happens at the boundary, and once per block, not per tile | App | P0 | M | TC-2 |
| TC-4 | **Classify commands.** Schema and class for 183 ids. Mark `tool_cmds`, `view_cmds` and `file_cmds` as `View` or `SideEffect`. Refuse `material.link.cook` (process spawn) and path-taking commands as plugin ops | App | P0 | S | CL-1 |
| TC-5 | **Events:** `document.changed`, `bake.progress`/`finished` (it is polled today, `docs/control-protocol.md:164`), `selection.changed`, `tool.changed`, `stroke.end` | App | P0 | S | — |
| TC-6 | **`texturecraft.session@1` adapter.** Node contributions under Add effect, Filters, Generators and Fill source; resource contracts for smart materials and masks, brush alphas and presets, export templates, HDRIs and materials | Adapter | P0 | M | TC-2, TC-4, TC-5 |
| TC-7 | **Bake maps.** `texture.bake-map@1`: the `Tracer` trait (`bake/src/tracing/tracer.rs:4-7`) becomes the `bake.rays` service. The plugin gets a per-texel G-buffer and batched ray queries (up to 1M rays per call) and returns a map. `BakedMap::Plugin` | App + Adapter | P2 | M | TC-2 |
| TC-8 | **Brush hooks.** `brush.dab@1`, `brush.particles@1`, `input.filter@1`; add tilt, rotation and coalesced samples (`ui-egui/src/viewport.rs:1711`, `stroke.rs:128-131`) | App | P1 | M | — |
| TC-9 | **Viewport hooks.** `viewport.overlay@1` (world-space lines, points, per-face colours, text) and `viewport.shader@1` (a WGSL shading function inside `mesh.wgsl`'s fragment stage, behind the WGSL validator). Picking is exposed as `{pos, normal, uv, tri, set}` from the CPU BVH | App | P2 | M | K7 |
| TC-10 | **GPU-resident layers.** Move layer storage and compositing onto the GPU pool (CL-3), with the CPU compositor kept as the reference and for export. Today only display textures are resident (`renderer.rs:1436-1474`) | App | P2 | L | CL-2/CL-3 |
| TC-11 | **Device policy.** Plugin cooks run on a dedicated cook device, as the baker already does (`gpu_bake.rs:71-84`), so a hostile or slow WGSL node cannot stall the viewport. Results cross devices by CPU copy, or by an export/import of shared handles where the backend supports it | App | P1 | S | CL-2 |
| TC-12 | **Shared mesh.** Replace `geom::Mesh` with `craft-mesh::MeshData`, or provide a lossless conversion; expose `mesh@1` read handles | App | P1 | M | CL-2 |
| TC-13 | **In-process `MaterialLink`.** Cook linked `.mcmat` graphs in-process through the shared graph runtime, not by spawning `materialcraft-cli` (`link_cmds.rs:232`) | App | P3 | M | MC-1, CL-3 |

### 5.3 SoundCraft

| ID | Change | Layer | P | E | Depends on |
|---|---|---|---|---|---|
| SC-1 | **Event-list process.** Extend `Plugin` (`crates/dsp/src/lib.rs:195`) with `process_ctx(&mut ProcessCtx)`. It carries: <br>- buses (main, sidechain, aux) <br>- sorted events (`param_value`, `param_mod`, gestures, notes with ids, expressions, MIDI 1, UMP, sysex) <br>- transport <br>- `steady_time` <br>The default implementation splits the block at events and calls the legacy `set_param` + `process`, so built-ins keep working | App | P0 | M | CL-5 |
| SC-2 | **Sample-accurate automation.** `apply_params` (`mix/src/strip.rs:480-516`) emits automation breakpoints within the block as events, and sends get the same treatment. **Hosts:** CLAP stamps event offsets (it uses `time:0` today), VST3 uses `addPoint(offset)`, and AU uses `AudioUnitScheduleParameters`. **Built-ins:** they split at events with a floor of 16 samples, plus per-sample smoothing (FilmCraft's `AudioEffect` convention) | App | P0 | M | SC-1 |
| SC-3 | **Render-quantum invariance test.** Bounce one session at blocks of 64, 128, 512 and 1024 and require bit-identical output for deterministic built-ins. **This test fails today** for automated plugin parameters and send levels. It is the acceptance gate for SC-2 | App | P0 | S | — |
| SC-4 | **Per-slot fault policy.** Contain a panic, trap or deadline miss to one slot: keep a dry copy, crossfade over 64 samples, and auto-bypass after N faults in M seconds. Replace the whole-stream silence in `guarded_block` (`playback/src/callback.rs:55-62`) with "fail the slot, keep the mix". Never silence the master | App | P0 | M | — |
| SC-5 | **`craft:` Wasm real-time tier.** Add a `craft:` prefix to `create_plugin` (`mix/src/instances.rs:114-126`) that creates a `WasmPlugin` on Wasmtime/Cranelift, compiled ahead of time at install. Instantiation runs on the control thread, as now, using a pooling allocator with memory touched in advance. `process_ctx` runs on the audio thread under an epoch deadline. State goes into `Insert.state` | App + Platform | P0 | L | SC-1, SC-4, PL-RT |
| SC-6 | **Realtime worker pool.** Strips run in sequence on the audio thread today (`mix/src/threads.rs:38`). Add a fixed pool of realtime-priority threads that join the device's audio workgroup on macOS (`os_workgroup`, Apple "Adding parallel real-time threads to audio workgroups") and use MMCSS on Windows. Fork and join per block, with no locks | App | P1 | L | CL-5 |
| SC-7 | **No-alloc guard.** A debug global allocator that panics on audio-thread allocation, plus a CI test that plays a session. Move `sync` allocations (`mix/src/lib.rs:276-363`, `strip.rs:521`) to the control thread with pointer swap | App | P1 | S | — |
| SC-8 | **Sidechain and buses.** Key-input routing in the model (`Insert.sidechain: Option<Route>`); enable VST3 aux buses and CLAP and AU sidechains | App | P1 | M | SC-1 |
| SC-9 | **Out-of-process native bridge** (`bridge:` prefix). Wraps the clap, vst3 and au hosts in `soundcraft-plugin-bridge` processes, using shared-memory rings (craft-libs `ipc` bulk is a candidate) and one futex or semaphore wake per block. Isolation modes follow Bitwig: Within, Together, Per vendor, Per plugin, Individually. Scanning moves out of process; it is synchronous on the UI thread today (`plugin_menu.rs:152-155`). The `disable-library-validation` entitlement goes on the bridge only | App + Platform | P1 | L | SC-1, SC-4 |
| SC-10 | **PDC.** Make PDC visible in realtime: playhead display, record alignment and plugin-latency-change restarts. `crates/playback` has no latency handling (grep) | App | P1 | S | — |
| SC-11 | **MIDI and controllers.** MIDI devices (G-02), MIDI learn (G-20), MPE, MIDI 2.0 UMP, clock (G-11); `control.surface@1` | App + Platform | P1 | L | CL-7 |
| SC-12 | **Hosted plugins offline.** Allow hosted and Wasm plugins in `audiosuite.process` (it refuses them at `cmd/audiosuite.rs:60`) and in render-in-place, as `audio.offline@1` jobs | App | P2 | S | SC-5 |
| SC-13 | **`sample.stream`.** A host disk streamer (preloaded heads plus background reads into lock-free rings) used by the built-in sampler, Wasm instruments and the source pool (G-06) | App + Platform | P2 | L | CL-5 |
| SC-14 | **Classify commands and expose the journal.** Schema and permission class over the 389 ids; the existing `query`/`view`/`noundo` flags map directly. Expose the journal as `history.journal` for macro recording, and add a `macro.replay` command | App | P0 | S | CL-1 |
| SC-15 | **Events.** `session.changed`, transport, selection, `meter` (throttled) and `plugin.fault`, for M4L/LOM-style observers | App | P0 | S | — |
| SC-16 | **Plugin UI.** Continuous gestures to automation Touch/Latch; meter and scope rings; native editors stay windows owned by the bridge; embedded CLAP editors on macOS and Linux (G-05) | App + Platform | P1 | M | U1, U5 |
| SC-17 | **More formats.** AUv3 hosting (out of process by design) and ARA2 in the bridge (`audio.clip-editor@1`) | App | P2 | L | SC-9 |
| SC-18 | **Web.** An AudioWorklet host for `craft:` plugins in `soundcraft-web` | App | P2 | M | SC-5 |
| SC-19 | **Host modulators.** LFO and envelope devices that write `param_mod` | App | P3 | L | SC-1 |

### 5.4 craft-libs

| ID | New or changed crate | Contents | Consumers | P | E |
|---|---|---|---|---|---|
| CL-1 | `craft-command` | `CommandSpec<S>` with `id`, label, menu, shortcut, `schema` (JSON Schema), `kind`, `permission`, `undo`, `run`. A registry trait; MCP tool generation; a classification check for CI. It unifies the three copies (`materialcraft/.../commands.rs:12`, `texturecraft/.../commands.rs:14`, `soundcraft/.../cmd.rs:74`) and should be folded into `craft-mcp-server` | MC, TC, SC, platform bridge | P0 | M |
| CL-2 | `craft-mesh` `MeshData` | The §3.2 type plus conversions from `Scene` and from TextureCraft's `geom::Mesh` | MC, TC, SolidCraft | P1 | M |
| CL-3 | `craft-gpu` | Device broker (shared or dedicated cook device); `TexturePool` with a byte budget and eviction; `TextureHandle`; a WGSL prelude (integer-hash RNG, UDIM-aware sampling, colour conversions); tiled dispatch with timestamp queries and a per-submit budget; device-loss recovery; a digest kill list. **Needs the app `AGENTS.md` rule ("only render/ui-egui/apps may use wgpu") amended** to allow the cook crates | MC, TC, platform `ext-gpu` | P0 | L |
| CL-4 | `craft-graph` | Node type registry trait, port and parameter types, cache key, missing-node placeholder semantics; the `graph.node@1` host runtime shared by MC and TC (and EffectCraft and PhotoCraft) | MC, TC, ext | P0 | M |
| CL-5 | `craft-audio-rt` | CLAP-shaped event types, SPSC rings, realtime pool with workgroup and MMCSS joins, a no-alloc guard, a dry-bypass crossfader | SC, FilmCraft, ext audio tier | P0 | M |
| CL-6 | `craft-audio-dsp` | Biquads, FFT and STFT, partitioned convolution, resamplers, oversampling, loudness and true peak, taken from SoundCraft `dsp` and FilmCraft `audio-dsp`; the realtime-safe kernels behind `audio.*` | SC, FC, ext | P1 | M |
| CL-7 | `craft-midi-io` | Device I/O (midir/CoreMIDI), UMP and MIDI 2.0, MPE zones, clock and MTC; OSC | SC, FC | P1 | M |
| CL-8 | `craft-bake` | TextureCraft's `bake` crate plus `Tracer`, moved out so MaterialCraft can bake | TC, MC | P2 | M |
| CL-9 | `craft-nn` | ONNX (pure Rust, seeded from LightCraft's interpreter) plus candle, with GPU via CL-3; `nn.run` jobs; models as sha256-pinned data | all | P2 | L |
| CL-10 | `craft-ui` widgets | Node-graph widget, curve and gradient editors, knob, meter and scope bound to rings; panels hosted as `DockContent` | all | P1 | M |
| CL-11 | `craft-texture` codecs | KTX2, BC6H/BC7 encode (next to `dds`) | TC, MC | P3 | M |

### 5.5 Platform items this report depends on (cross-references)

The program document's numbering is used in parentheses.

- **PL-RT, the realtime Wasm engine** (R4, PL-2). Wasmtime with Cranelift, compiled ahead of time and cached by sha256 plus engine version; pooling allocator; memory touched in advance; epoch watchdog; a no-JSON struct ABI; SIMD128.
- **`ext-gpu`** (K7) on CL-3.
- **Host-resident `image.*` handles** (R5) bound to `TextureHandle`.
- **`graph.node@1`** in `ext-core`, with the runtime in CL-4.
- **`audio.*` contracts**, the `.wclap` import adapter, and **`control.surface@1`**.
- **Host services:** `net.fetch`, `secret.*`, `auth.oauth`, `service.call` and `handoff.open`.
- **UI:** continuous gestures (U1) and meter rings (U5).

---

## 6. Edge cases

### 6.1 8K and UDIM texture sets, and GPU memory

**Memory per layer at 8K:**

| Storage | Bytes per layer |
|---|---|
| RGBA f32 | 1 GiB |
| RGBA f16 | 512 MiB |
| One u16 channel, TextureCraft's storage | 128 MiB |

**A realistic hero asset.** A metal/rough TextureCraft set has about 6 active channels. At 8K in u16 that is 768 MiB per *flattened* set, before layers, masks and the cache.

Ten UDIM tiles at 4K come to the same total. TextureCraft already caps a set at 2^27 texels (`bake/src/lib.rs:44`) and the project-open limit at 8 GiB (`project_open_limit.rs:4`).

**Device limits.**

- WebGPU defaults: `max_texture_dimension_2d` 8192 and `max_storage_buffer_binding_size` 128 MiB.
- Native adapters usually allow 16384 and larger bindings.
- One 8K RGBA f32 storage buffer exceeds the default binding size.

**Design:**

- **Plugins never allocate.** Nodes declare outputs and the host allocates them in CL-3's pool. The pool's budget is 70% of the adapter's reported memory, or a user setting.
- **Tiled cooks over budget.** A pointwise node cooks per tile region (1024² or 2048²), declaring its neighbourhood radius for an apron. A `whole_map` node gets the full map only if it fits; otherwise it is refused with a clear error ("node X needs 1.0 GiB, budget 0.6 GiB").
- **UDIM.** Cooks run per tile with a shared parameter set, plus `udim` in the context. Seams are handled by the host padding (dilation) pass, which already exists for curvature (`compose/src/curvature_sizes.rs:68`). Plugins do not see adjacent tiles unless they request `whole_set`, which is expensive and budget-checked.
- **Eviction.** GPU textures fall back to CPU u16 or f16. A plugin's cache entry stores its key, not its pixels, so re-cooking is always possible.
- **Wasm guests never hold maps.** They get handles, and the 512 MiB transform cap (`ext-host/src/wasm.rs:11`) stops mattering.

### 6.2 Deterministic cooks across GPUs

**The problem.**

- WGSL allows implementation-defined precision for transcendentals and for FMA contraction (https://www.w3.org/TR/WGSL/#floating-point-accuracy). The same shader can differ in the last bits between Apple, NVIDIA, AMD and Intel.
- Both apps' `AGENTS.md` require GPU results to match the CPU reference.
- MaterialCraft already fixes float order with `mul_add` (`node_execution.rs:181-183`) and uses integer-hash RNG (`nodes/src/random.rs`).

**Design:**

1. **Determinism class per node.**
   - `exact`: integer or fixed-point, or the CPU `program` tier.
   - `tolerant{abs, ulp}`.
2. **RNG.** The WGSL prelude provides the same integer-hash RNG as MaterialCraft's CPU code (PCG/xxhash, u32 arithmetic, bit-exact everywhere). It refuses to promise anything for `fract(sin(x))`-style hashes; a lint warns on them.
3. **Cache keys include `backend-class`** for tolerant nodes, so a cached GPU result is never reused as a CPU-reference export.
4. **Export modes.** "Export (fast)" uses the GPU. "Export (reference)" re-cooks tolerant nodes on the CPU interpreter of the same naga IR or `kernel.program`. Batch and CLI default to reference.
5. **Conformance.** Every WGSL node package ships golden images. Install-time verification runs them on the user's device and compares with the declared tolerance (CIEDE2000 via craft-libs `render-parity`). Failing nodes are marked "GPU differs" and run on the CPU.

### 6.3 Graphs with missing plugin nodes

**MaterialCraft is already well placed.** SBS placeholders keep their ports and raw parameters (`formats/src/sbs/placeholder.rs`), and cooking a missing node errors with "unsupported node" (`node_execution.rs:202-205`). What it adds:

- MC-2 dependency URLs, so the missing package id, version and digest are known;
- a "Find plugin" action that looks the package up in the catalogue;
- **downstream nodes cook from a magenta placeholder** of the declared output kind, not an error, so the rest of the graph stays editable;
- export refuses a graph with missing nodes unless forced, and records it in the export log.

**TextureCraft needs TC-1 first.** After TC-1 and TC-2, a missing `Filter::Plugin` or effect works like this:

- it is pass-through in the compositor, with a badge on the layer and in the layer stack;
- it is preserved verbatim, including unknown fields and unknown variants, on save, reinstall and downgrade (suite replacement rule);
- a smart material that references a missing node imports with the same placeholders.

**SoundCraft:**

- `create_plugin` returns `None` for an unknown id, and the `Insert` (id, params, base64 state) stays in the model.
- What it adds: a visible "missing plugin" slot state (pass-through, or silence for instruments), the version that was last saved, and "replace with…" mapping the parameter map by name.
- `refused_state` is already reported (`mix/src/instances.rs:63-64`) and should surface in the UI.

**Version skew.** A newer plugin that changed parameter ids needs `migrate(old_version, params) -> params`, as a pure main-thread function in the plugin. CLAP's state versioning and MaterialCraft's `ParameterDefinition` both support this pattern.

### 6.4 Real-time audio deadline misses

**Today:**

- No per-plugin deadline exists.
- An xrun is only logged (`playback/src/stream.rs:79-83`).
- A panic kills the whole stream (`callback.rs:55-62`).

**Design (SC-4, SC-5, SC-6):**

| Event | Handling |
|---|---|
| Wasm slot exceeds its budget (epoch deadline at k × the block's share; the ticker runs at block period / 4) | Trap → output dry for this block (64-sample crossfade) → `plugin.fault{kind: deadline}` |
| Trap or NaN | Same as a deadline miss; NaN handling already exists per slot (`strip.rs:441-451`) |
| N faults in M seconds | Auto-bypass with a UI badge; manual re-enable |
| Bridge process stalls | Wait on the semaphore with a timeout (block period minus a margin), then dry, and the bridge slot is marked late. A bridge crash means dry until reload. The session continues |
| Whole-engine overrun | Count xruns per strip with cycle timing, and show a per-strip CPU meter (Pro Tools style). Adaptive buffer suggestions |

Budgets are measured, not guessed. At activate, the host runs the plugin for 200 blocks of silence and of noise, and records p99 cost. The default deadline is 4× p99, capped at 50% of the block period.

### 6.5 Sample-accurate automation

**Current resolution:**

| Target | Resolution | Source |
|---|---|---|
| Plugin parameters | per block | `strip.rs:480-516` |
| Sends | per block | — |
| Pan | 64-sample grid | `mix/src/automation.rs:179-193` |
| Volume, trim, VCA | per sample | — |

Because the realtime block varies (≤512) and bounces use 1024, **an automated plugin parameter changes at different sample positions in playback and in a bounce.**

**Design.** Lane evaluation yields `param_value` events at:

- every breakpoint inside the block;
- for ramps, every *k* samples, where *k* defaults to 32 and is a per-plugin hint (CLAP plugins smooth internally, so breakpoints alone suffice when the plugin declares it).

Events reach:

- CLAP as stamped events (fixing `time:0`);
- VST3 through `IParameterChanges` points;
- AU through scheduled parameters;
- Wasm through the event ring;
- built-ins through split processing.

**Touch/Latch recording.** Plugin UIs emit gesture begin and end (CLAP `param_gesture`; ext-ui U1). These map to the existing Touch/Latch/TouchLatch/Write/Trim modes.

### 6.6 Plugin latency compensation

**Today:**

- Per-strip latency is summed and the mix aligns to the slowest path (`mix/src/lib.rs:429-494`), gated by `edit.delay_compensation`.
- Offline drops the head (`offline.rs:87-94`).

**Missing:**

- **Latency changes.** CLAP `latency.changed` and VST3 `kLatencyChanged` require a deactivate/activate cycle. SoundCraft has no restart-request path, so a plugin that changes latency (a linear-phase toggle) silently misaligns. Fix: a main-thread `request_restart` that re-prepares the slot off-thread, swaps it in with `adopt`, and recomputes PDC.
- **Bridge latency.** An out-of-process bridge in synchronous mode adds no samples, but costs wall time. An asynchronous ("anticipative", one block ahead) mode adds one block of latency, which must be reported into PDC.
- **Sidechains** must be delay-compensated to the key target (SC-8).
- **Display and recording.** The realtime playhead and recording offset should include output latency and PDC (SC-10).
- **Wasm latency** is declared at activate and is immutable until a restart, as in CLAP.

### 6.7 Offline bounce equality

**Sources of difference today:**

1. Block size: ≤512 variable in realtime against 1024 in bounces. This interacts with per-block parameters (fixed by SC-2).
2. `set_offline(true)` lets plugins change quality: realtime and offline render paths, as in VST3 `kOffline` and CLAP render mode.
3. Metronome excluded.
4. Parallel strips offline against sequential realtime. This is harmless when summing is ordered.
5. Linear-interpolated device-rate conversion in realtime only.

**Design:**

- **Invariant:** for a session with only deterministic plugins and no offline-quality switch, `bounce(block=N)` is bit-identical for every N in {64, 128, 512, 1024}. The SC-3 test enforces it.
- **Playback against bounce.** They are equal up to the device resampler, so tests compare the mix bus *before* device conversion.
- **Plugin determinism.** Wasm plugins get NaN canonicalization and no wall clock or randomness without a seed. Their `deterministic` flag is verified at install by bouncing twice.
- **Faults are recorded in bounces.** A plugin that faulted during a bounce marks the render "contains bypassed plugin" (program fail-safe policy). The CLI returns a non-zero status unless `--allow-faults` is passed.
- **Mixdown sum order.** The sum order must be fixed (per bus, by strip index) regardless of the parallel schedule. `run_strips` currently writes per strip and sums afterwards, which keeps it that way. A test should guard it.

### 6.8 Very large sample libraries

**The problem.**

- Sources are decoded whole into RAM (G-06).
- The built-in sampler gets `set_sample(Arc<dyn SampleSource>)`.
- Kontakt, Spitfire, Omnisphere and Superior Drummer–class libraries run to 50–500 GB.
- Wasm guests cannot do file I/O on the audio thread and must not hold gigabytes in linear memory.

**Design (`sample.stream`, SC-13):**

- **Resident heads.** A plugin registers sample regions at load, from a package or user-picked folder grant. The host keeps a preloaded head per region (for example 64 KiB per zone, configurable) in host memory outside the guest.
- **Voice start and streaming.** On note-on the guest calls `stream.open(region, offset) -> stream_id`, which is realtime-safe: it takes a slot from a preallocated pool. The host's I/O threads fill an SPSC ring per voice, and the guest reads with `stream.read(id, dst_ptr, frames)`, a zero-copy copy into its buffer.
- **Underruns.** An underrun returns fewer frames plus a flag, never a blocking call. Metrics surface in the plugin manager.
- **Memory accounting.** Purge and RAM budgets are host-owned (Kontakt's "purge unused samples" becomes a host op).
- **Formats.** Decoding uses craft-audio codecs. Compressed lossless libraries are decoded by I/O threads ahead of the ring.
- **Native instruments** (Kontakt) keep their own streaming inside the bridge process. The host only budgets the bridge's RAM and warns.
- **SoundCraft's own sources** move to the same streamer, which closes G-06 for long sessions too.

---

## 7. Open decisions

| # | Decision | Recommendation | Why it is open |
|---|---|---|---|
| 1 | **Allow wgpu outside `render`/`ui-egui`.** Both texture apps' `AGENTS.md` restrict it | Yes, through a craft-libs `craft-gpu` crate only, with the CPU reference rule kept | It changes app architecture rules; the owner must approve |
| 2 | **One cook device or two in TextureCraft.** A shared device is cheaper; a dedicated one isolates plugin WGSL from the viewport | A dedicated cook device for plugin nodes, following the baker precedent | Memory duplication on 8 GB GPUs |
| 3 | **SBS as the plugin-node encoding in MaterialCraft** (dependency URLs), against a private extension block | Dependency URLs: Designer-compatible and diffable | Interop with Substance Designer when plugins are missing |
| 4 | **Default native isolation in SoundCraft:** in-process (today), Together, or Per vendor | Together by default, Per plugin for any plugin that ever crashed | It changes behaviour and adds CPU cost (about 20–100 µs per round trip, estimated) |
| 5 | **Realtime Wasm engine.** Wasmtime/Cranelift adds a large trusted base and 35–50 ms compile per module (measured in the program document) | Yes on desktop; AudioWorklet on web; no realtime Wasm on iOS | Security review of Cranelift on the audio thread (signal-based traps) |
| 6 | **Which CLAP-over-Wasm ABI: WCLAP as is, or a craft ABI with a WCLAP adapter** | Craft ABI (simpler, no wasi-threads) plus a WCLAP import adapter | WCLAP is still young |
| 7 | **Command schema source:** derive from Rust types (schemars), or hand-write JSON Schema in specs | schemars on typed parameter structs, generated into `ops.json` | It touches 705 command definitions across three apps |
| 8 | **Bounce equality:** bit-exact for every block size, or a tolerance | Bit-exact for built-ins and deterministic Wasm; tolerance for native plugins | Native plugins are outside our control |
| 9 | **The `.tcraft` schema rework** (`ROADMAP.md:22`): do TC-1 now or wait | Do TC-1 now; it is small and protects users today | Upstream roadmap churn |
| 10 | **`sample.stream` ownership:** SoundCraft first, or the platform | Write it in craft-libs (`craft-audio-rt`) and use it in SoundCraft first | Shared-crate cadence is slower |
| 11 | **MIDI and controller scripting tier:** Wasm sessions at 1 kHz or a dedicated controller runtime | Wasm sessions with a `midi.in` event fast path | Latency for motor-fader feedback |
| 12 | **Mixer-migration and Quixel-style marketing importers** (closed formats) | Import only exported texture sets and MaterialX; do not reverse closed project formats | Licensing and clean-room policy (suite replacement rule) |

---

## Sources

**Code (all read at the revisions in the header):**

- MaterialCraft: `crates/{engine,nodes,eval,core,formats,automation,render,ui-egui}`, `docs/{gaps,roadmap,architecture,node-catalog-parity}.md`, `AGENTS.md`
- TextureCraft: `crates/{engine,core,compose,texel,render,bake,paint,geom,formats,automation,ui-egui}`, `ROADMAP.md`, `docs/{architecture,features,control-protocol}.md`
- SoundCraft: `crates/{engine,mix,playback,dsp,clap-host,vst3-host,au-host,automation,midi}`, `packaging/macos/entitlements.plist`, `docs/{gaps,plugin-parity,hardware-parity,mcp,cli,control-protocol}.md`
- craft-libs: `crates/{mesh,texture,materialx,mcmat,usd,dds,audio,midi,sf2,time,ui,keymap,control,mcp,mcp-server,ipc,ai,ai-core,wasm,cms}`, `AGENTS.md`
- craft-extensions: `crates/ext-host/src/wasm.rs`, `crates/ext-host/Cargo.toml`, `docs/plugins/research/{capability-program-2026-11-18.md, capability-2026-11-18/3d.md, audio.md}`

**Web:**

[1] Substance 3D Designer scripting API: https://experienceleague.adobe.com/en/docs/substance-3d-designer/using/scripting/scripting-api-reference · Designer Python API: https://adobedocs.github.io/designer-python-api · https://magazine.substance3d.com/plugins-substance-designer-new-api-new-opportunities/
[2] Designer node and property scripting: https://helpx.adobe.com/substance-3d-designer/scripting/nodes-and-properties.html
[3] Pysbs: https://helpx.adobe.com/substance-3d-sat/pysbs-python-api.html
[4] Sampler plugins (Python + QML): https://helpx.adobe.com/substance-3d-sampler/scripting-and-development/create-a-plugin-with-python-and-qml.html
[5] Sampler custom filters: https://helpx.adobe.com/substance-3d-sampler/filters/custom-filters.html
[6] Painter Python API: https://experienceleague.adobe.com/en/docs/substance-3d-dev/painter-python/api/api-overview · Plugins menu: https://experienceleague.adobe.com/en/docs/substance-3d-painter/using/interface/main-menu/plugins-menu
[7] Blender Extensions platform: https://extensions.blender.org/ *(unverified for the Node Wrangler page)*
[8] BlenderKit: https://github.com/BlenderKit/blenderkit/wiki/BlenderKit-add-on-documentation
[9] Poliigon: https://help.poliigon.com/en/articles/6342599-poliigon-blender-addon
[10] Quixel to Fab FAQ: https://forums.unrealengine.com/t/quixel-to-fab-transition-faqs/2033839
[11] UVPackmaster: https://superhivemarket.com/products/uvpackmaster/
[12] TexTools: https://github.com/franMarz/TexTools-Blender
[13] Zen UV: https://zen-masters.github.io/Zen-UV/ *(unverified)*
[14] Texel Density Checker: https://github.com/mrven/Blender-Texel-Density-Checker
[15] Material Combiner: https://github.com/Grim-es/material-combiner-addon
[16] MACHIN3 tools: https://machin3.io/ *(unverified)*
[17] Hard Ops manual: https://hardops-manual.readthedocs.io/ *(unverified)*
[18] Ucupaint: https://github.com/ucupumar/ucupaint
[19] Principled Baker: https://github.com/danielenger/Principled-Baker *(unverified)*
[20] xNormal: https://xnormal.net/ *(unverified)*
[21] Dream Textures: https://github.com/carson-katri/dream-textures
[22] AI-Render: https://github.com/benrugg/AI-Render
[23] Quad Remesher: https://exoside.com/ *(unverified)*
[24] RetopoFlow: https://github.com/CGCookie/retopoflow
[25] Geo-Scatter: https://www.geoscatter.com/ *(unverified)*
[26] Sverchok: https://github.com/nortikin/sverchok
[27] Blender OSL script node: https://docs.blender.org/manual/en/latest/render/shader_nodes/osl.html
[28] fSpy for Blender: https://github.com/stuffmatic/fSpy-Blender
[29] Houdini `hou.HDADefinition`: https://www.sidefx.com/docs/houdini/hom/hou/HDADefinition.html
[30] Copernicus OpenCL COP: https://www.sidefx.com/docs/houdini/nodes/cop/opencl.html · https://www.sidefx.com/docs/houdini/copernicus/index.html
[31] SideFX Labs: https://github.com/sideeffects/SideFXLabs
[32] Houdini Engine for Unreal: https://sidefx.com/docs/houdini/unreal/intro.html
[33] Material Maker shader nodes: https://rodzill4.github.io/material-maker/doc/shader_nodes.html
[34] ArmorPaint plugin API (third-party mirror, language claims unverified): https://mintlify.wiki/armory3d/armorpaint/dev/plugin-api · https://github.com/armory3d/armorpaint
[35] Quixel Mixer discontinued: https://www.cgchannel.com/2026/02/quixel-mixer-has-been-discontinued-get-the-final-version-for-free/
[36] Marmoset `mset` Python: https://marmoset.co/toolbag/python · Add-on library: https://marmoset.co/toolbag-add-on-library/
[37] Unreal Custom material expression: https://dev.epicgames.com/documentation/en-us/unreal-engine/custom-material-expressions-in-unreal-engine *(unverified)*
[38] Unity Shader Graph Custom Function node: https://docs.unity3d.com/Packages/com.unity.shadergraph@17.0/manual/Custom-Function-Node.html *(unverified)*
[39] MaterialX specification: https://github.com/AcademySoftwareFoundation/MaterialX/blob/main/documents/Specification/MaterialX.Specification.md · OpenPBR: https://github.com/AcademySoftwareFoundation/OpenPBR
[40] soothe2: https://oeksound.com/plugins/soothe2/ *(unverified)*
[41] ARA SDK: https://github.com/Celemony/ARA_SDK
[42] JSFX: https://www.reaper.fm/sdk/js/js.php
[43] ReaScript: https://www.reaper.fm/sdk/reascript/reascript.php · ReaImGui: https://github.com/cfillion/reaimgui
[44] SWS: https://www.sws-extension.org/ · ReaPack: https://reapack.com/
[45] Max for Live: https://www.ableton.com/en/live/max-for-live/
[46] RNBO: https://rnbo.cycling74.com/ *(unverified)*
[47] Bitwig Grid: https://www.bitwig.com/the-grid/ · plug-in sandboxing: https://bitwig.com/userguide/latest/vst_plug-in_handling_and_options
[48] Bitwig extensions: https://github.com/bitwig/bitwig-extensions *(unverified)*
[49] DrivenByMoss: https://www.mossgrabers.de/Software/Bitwig/Bitwig.html *(unverified)*
[50] Logic Pro User Guide (Scripter chapter): https://support.apple.com/guide/logicpro/welcome/mac *(unverified deep link)*
[51] Cubase MIDI Remote API: https://steinbergmedia.github.io/midiremote_api_doc/ *(unverified)*
[52] FL Studio MIDI scripting: https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/midi_scripting.htm · API stubs: https://il-group.github.io/FL-Studio-API-Stubs
[53] FL Studio VFX Script: https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/VFX%20Script.htm
[54] Audacity plugins (Nyquist): https://plugins.audacityteam.org/
[55] OpenVINO AI plugins for Audacity: https://github.com/intel/openvino-plugins-ai-audacity *(unverified)*
[56] Reason Rack Extension SDK: https://developer.reasonstudios.com/ *(unverified)*
[57] VCV Rack plugin development: https://vcvrack.com/manual/PluginDevelopmentTutorial *(unverified)*
[58] plugdata: https://plugdata.org/ *(unverified)*
[59] WebCLAP: https://github.com/WebCLAP · WAM2: https://www.github.com/webaudiomodules/api
[60] Faust Wasm: https://github.com/grame-cncm/faustwasm · Cmajor: https://cmajor.dev/
[61] CLAP: https://github.com/free-audio/clap (versions 1.2.6–1.2.7 reported by Wikipedia and a mirror; unconfirmed against the changelog) · VST3 3.8 MIT: https://steinbergmedia.github.io/vst3_dev_portal/pages/Versions/Version+3.8.0.html · AUv3 hosting: https://developer.apple.com/documentation/audiotoolbox/migrating-your-audio-unit-host-to-the-auv3-api
[62] MIDI 2.0: https://midi.org/midi-2-0 *(unverified)*
[63] TouchOSC: https://hexler.net/touchosc *(unverified)*

**Platform references:**

- WebGPU limits: https://www.w3.org/TR/webgpu/#limits
- WGSL floating-point accuracy: https://www.w3.org/TR/WGSL/#floating-point-accuracy
- Wasmtime Config: https://docs.wasmtime.dev/api/wasmtime/struct.Config.html
- Apple audio workgroups: https://developer.apple.com/documentation/audiotoolbox/workgroup_management *(unverified)*

Product facts for FabFilter, iZotope, Waves, Soundtoys, Valhalla, Serum, Vital, Kontakt, Melodyne, Auto-Tune, Arturia, u-he, NI and Spitfire come from the sources list in `capability-2026-11-18/audio.md` and were not re-fetched in this pass.

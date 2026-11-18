# Image and design apps: surfaces, plugin catalogue and the work to reach incumbent parity

Date: 2026-11-18. Researcher 06 of 8. Read-only: source reading, grep counts and public documentation. No builds,
no app runs, no edits outside this file. Repos (paths below are relative to each repo root unless prefixed):

| Short | Repo | Branch | Head |
|---|---|---|---|
| PC | `photocraft` | plugin-beta | `e45144c2a` |
| FC | `figcraft` | plugin-beta | `48f0eb6c` |
| VC | `vectorcraft` | upstream main | `ef698d76f` |
| DC | `designcraft` | upstream main | `3930cff8` |
| LC | `lightcraft` | upstream main | `27b755a9` |
| CE | `craft-extensions` | plugin-beta | `24cd2ff` |

This builds on `CE/docs/plugins/research/capability-program-2026-11-18.md` (the program) and
`CE/docs/plugins/research/capability-2026-11-18/image.md` (the earlier image pass). Corrections to those are
marked **[corr]**. Status legend in tables: ✔ possible today, ◐ partial, ✖ blocked; the blocking layer is
App (engine), Ad (adapter) or Pl (platform, craft-extensions). Priorities and efforts follow the program:
P0 blocks a whole class of famous plugins, P3 is later; S ≤ 1 week, M 1–4 weeks, L > 4 weeks (estimates).

## Summary

1. **The five registries are bigger than recorded, and none of their flags can be trusted as a permission
   class.** Static counts: PC about 809 commands (777 literal ids plus 32 generated adjustment ids), FC 630,
   VC about 1,034 engine plus about 290 UI, DC 567 engine plus 194 UI, LC 391 engine plus 213 UI
   **[corr: program said ~700/630/888/635/325]**. Each app marks queries differently (`journal`,
   `history`, `query`/`noundo`), and the flags lie about side effects: DC's `file.exportIdml` is a `query`
   that writes a file; LC writes an XMP sidecar after *every* command (`LC/crates/engine/src/lib.rs:630-633`).
   A safe `batchPlay` therefore needs a **reviewed classification table per app**, not a derived one.
2. **VectorCraft already built that table.** Its legacy `vc_call` host import is gated by a reviewed list of
   **705 allowed and 13 denied** commands plus a prefix denylist, with a test that fails if any registered
   command is unclassified (`VC/crates/engine/src/cmd/plugin_call.rs:34-130,681-694`,
   `plugin_call/reviewed.rs`). This is the template for the suite registry bridge and should move into
   the platform, not be deleted with the legacy system.
3. **VectorCraft's legacy system is larger than recorded** **[corr]**: four kinds (filter, live effect,
   exporter, importer), API levels 1 and 2 with **11 permission-gated host imports** (progress, settings,
   cache, commands), 10 permissions, per-plugin settings and cache storage, Safe Mode with a launch-crash
   marker, deferred live-effect budgets, about 12.3k lines and 422 tests. Its migration is an L-sized
   project with a document-format disposition for every `Effect{id:"plugin.<id>"}` record.
4. **Only VectorCraft has a JavaScript engine.** `VC/crates/js` (34k lines, ES5.1 plus ExtendScript
   preprocessor, fuel and heap limits) runs scripts against the registry through `vc.run`. It is general
   automation with non-plugin callers, so the suite rule keeps it; it must not become a plugin bypass.
5. **GPU reality.** LC is the only app with compute shaders (21 WGSL files, 67 `@compute` entry points,
   0 fragment). PC has 48 `@fragment` and 0 `@compute`, on eframe's device. FC (tiny-skia), VC and DC
   (vello_cpu) render on the CPU and touch wgpu 30 only through eframe. A `gpu.shader` tier must bring
   its own scheduler on the eframe device in four of five apps.
6. **Every app polls; none has an event bus.** But each already has a change source a bus can be built
   from without engine rewrites: FC's collab property diffs (`FC/crates/collab/src/change.rs`), LC's
   catalogue `Op` log (36 variants), and the PC, VC and DC command journals plus PC damage rects.
7. **Non-destructive plugin results are close in PhotoCraft and silent elsewhere.** PC plugin filters are
   already `CommandSpec`s (`plugin.<plugin>/<id>`, journaled) and smart filters re-run by command id, so
   plugin smart filters are an M-sized change. But an unknown smart filter id leaves pixels unchanged
   (`PC/crates/engine/src/cmd/smart_cmds/rendering.rs:158-174`), and a missing VC plugin effect draws the
   geometry unchanged with no warning (`VC/crates/effects/src/lib.rs:25-27`). Both are the "silent
   pass-through" the suite rule forbids.
8. **Pixel access is narrower than UXP's.** PC filters take the whole canvas rectangle as premultiplied
   f32 sRGB, refuse non-RGB documents, masks, channels and smart objects (`PC/crates/engine/src/plugin_filters.rs:49-70,279-392`).
   Wide-gamut 16-bit documents are converted to sRGB for plugins, which clips gamut. Photoshop's Imaging
   API returns layers, masks, selections and the composite in RGB, Gray or Lab at 8/16/32 bits, with
   `targetSize` and pyramid `level` ([Imaging API](https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/imaging)).
9. **The adapters are uneven.** PC docks plugin panels in the framework dock; FC uses its own per-sidebar
   dock with craft-ui at three call sites; FC has 84 ops through a closed allowlist (not a generic bridge
   **[corr]**), but `node.update` reaches effects, OpenType features, variable axes and prototype
   interactions **[corr]**. FC's read-only guard still lets a `document.changed` handler move selection,
   view and page; `ui.notify` still only logs.
10. **LightCraft's ONNX interpreter cannot be the suite inference runtime.** It supports six op types
    (Conv, ConvTranspose, MaxPool, Concat, DepthToSpace, LeakyRelu; `LC/crates/denoise/src/onnx.rs:1-6`)
    **[corr: program K8 proposed lifting it as `craft-nn`]**. candle 0.9 (already in LC for SAM 3) is
    the realistic base, with the WGSL convolutions as a fast path.
11. **Catalogue:** 110 named incumbent plugins and plugin classes below. With the program's platform work
    (network, secrets, settings, events, binary pixels, ROI, GPU, `nn.run`) plus the per-app items here,
    99 are reachable in the sandbox; 9 need native helpers (hardware, OS input hooks, other-app round
    trips, 8bf, a TeX engine, a vendor workflow server); 2 depend on a loopback-network decision.
12. **Top app-side P0s:** PC tile-native pixel ops in all modes; PC/VC missing-plugin cached results;
    a reviewed registry bridge in all five apps; new adapters for VC, DC and LC; LC export, publish and
    metadata provider hooks; FC server-side plugin-data ownership; DC dynamic preflight issue kinds.

## 1. Per-app inventory

### 1.1 Cross-app comparison

| | PhotoCraft | FigCraft | VectorCraft | DesignCraft | LightCraft |
|---|---|---|---|---|---|
| Registry | `CommandSpec` `PC/crates/engine/src/cmd.rs:15-28`, `OnceLock` | `FC/crates/engine/src/cmd.rs:36-44` | `VC/crates/engine/src/cmd.rs:128-144` | `DC/crates/engine/src/cmd.rs:88` | `LC/crates/engine/src/cmd.rs:57` |
| Count (engine + UI) | ~809 | 630 | ~1,034 + ~290 | 567 + 194 | 391 + 213 |
| Query/edit flag | `journal` | `history` (343 edit, 287 not) | `journal` | `query` / `noundo` / default | `query` / default; undo only if `run` calls `commit` |
| Param schema | free text; 2 strict ids (`param_check.rs:25`) | names only (`refuse_unknown_params`, `cmd.rs:173-183`) | JSON-Schema for 32 ids (`crates/apischema`) | top-level keys, external paths only (`params.rs:224`) | free text; 125 typed develop controls (`develop/src/controls.rs:150`) |
| Undo | snapshot + coalesce (`session/execute.rs:248-290`) | rollback guard; collab undo in live sessions | snapshot `History` (`engine/src/history.rs`) | whole-doc snapshot, limit 1000 | op-inverse `UndoEntry`, limit 1000 |
| MCP | 26 tools, rmcp 3.5 (`automation/src/server.rs`) | 19 tools (`mcp/src/tools.rs:96-114`) | ~33 tools | 26 tools | 24 + one `cmd_<id>` per command |
| CLI | `plugin convert info run batch droplet commands mcp serve` | `Info Export Convert Render Run Commands Mcp Connect Perf Plugin` | `run`, `mcp`, `--safe-mode` | `run script describe app mcp validate-pdf thumbnail` | `run mcp render snapshot merge denoise-serve commands controls` |
| Scripting | Actions/ATN, batch, droplets | none beyond MCP | JS engine `crates/js`, `scriptcheck` | `script.run` step runner (`engine/src/script.rs`) | none |
| Render | wgpu 30, 48 `@fragment`, 0 `@compute` | tiny-skia 0.12 CPU | vello_cpu 0.2 CPU | vello_cpu 0.2 CPU | wgpu 30 compute, 67 `@compute` |
| Colour | craft-cms; 8 modes × U8/U16/F32; proofing | sRGB or Display P3 docs; moxcms read | moxcms; CMYK, spot, overprint | craft-cms; CMYK, Lab ΔE, separations | linear Rec.2020; HDR export; monitor profiles |
| AI | YuNet faces compiled in; PatchMatch fill; heuristics | none | none (potrace-style trace) | none | ONNX subset, candle SAM 3, model fetch |
| Format registry | closed `Format` enum (craft-libs) | hard-coded `match` (`cmd/file/load.rs:105-155`) | `FORMATS` + runtime `plugin_formats()` | `file/registry.rs` | 39 import extensions; export enum |
| Docking | framework `DockArea`; plugin panels dock | custom sidebar tab dock (`plugin_dock.rs`) | craft-ui docking, 88 panels | craft-ui, 43 panels | craft-ui, 4 dockable panels |
| Events | revision counters, polled | `EventLog` holds only `comment.*` | revision counters | revision + public journal | journal + op log |
| Plugin adapter | `photocraft.session@1.2`, 37 ops, 57 filters, 17 adjustments | `figcraft.session@1.2`, 84 ops | legacy `crates/plugins` only | none | none |
| Pen input | tablet: pressure, tilt, rotation, eraser | egui touch force only | pressure + `PenPose` tilt/barrel | none | egui force for mask brushes |

### 1.2 PhotoCraft (Photoshop class)

**Commands.** `Registry` builds about 110 module `specs()` lists (`PC/crates/engine/src/cmd.rs:304,1388+`);
`find()` checks built-ins, then plugin filters, then plugin session commands (`:243-246`). Largest prefixes:
layer 168, filter 71, file 60, edit 59, type 55, image 50. Execution: `Session::execute`
(`session/execute.rs:17`) → `dispatch` (`jobs.rs:1729`), with an authorization gate, param check, admission
(preconditions, job conflicts) and `catch_unwind`. External effects: `file.*` (open, save, export, print,
batch, droplet), `edit.preferences.*` (20 ids, including `plugIns`), `edit.colorSettings`,
`actions.play/conditional`, linked smart objects. The automation gate (`automation/src/workspace.rs:625-699`)
already denies `file.*` except 13 safe ids, linked smart-object commands and, on the desktop,
`image.mode.*`. That gate is a second, independent classification next to the plugin allowlist; the bridge
should share one table with it.

**Document.** `LayerContent` = Raster, Group, Adjustment, Fill, Text, Shape, Smart (`doc/src/lib.rs:606-614`).
Layers carry mask, vector mask, effects, `clipped`, `blend_if`, advanced blending, excluded channels,
video/frame, XMP and `plugin_data` (`:639`). The document has alpha channels, paths, work and clipping
path, Quick Mask, layer comps, notes, slices, variables, timeline, `plugins: Vec<PluginRef>` (`:899`).
Smart objects hold embedded or linked sources, smart filters, filter mask, warp, perspective and stack
mode (`:518`); `SmartFilter{command,params,blend,opacity,visible}` (`:584`). Modes: Bitmap, Gray, Indexed,
RGB, CMYK, Lab, Multichannel, Duotone; samples U8, U16, F32 (`crates/color/src/lib.rs:28,51`). Storage
is 256-pixel tiles (`geom/src/lib.rs:13`) with an LZ4 scratch-disk spill and clock eviction
(`raster/src/spill.rs`), which matters for 100 MP documents (section 5).

**Input and paint.** `tablet::Sample{pressure,tilt_x,tilt_y,rotation,eraser}` (`tablet/src/lib.rs:51`) with
AppKit, Windows Ink, Wintab, X11 and XInput backends. Brush controls include pen pressure, tilt, stylus
wheel and rotation (`paint/src/brush.rs:28`); ABR import (`io/src/abr_map.rs`). Plugin tool events still
send only `at, start, modifiers, button, pressure, zoom` (`engine/src/plugin_sessions.rs:368-375`), and the
UI forwards only pressure (`ui-egui/src/plugins_ui.rs:362`). No hover.

**GPU, colour, AI.** wgpu 30; `gpu/src/compose.wgsl` (24 fragment), three Camera Raw WGSL files (21), an
inline canvas shader (3); no compute pipeline anywhere. The window device belongs to eframe's
`RenderState` (`ui-egui/src/app/gpu.rs:6-32`); headless creates its own (`gpu/src/context.rs:98-128`).
craft-cms (rev `a6b9d9c`) handles assign/convert profile, installed profiles and 10 proof setups. No ML
runtime: YuNet face weights compiled in (`algo/src/faces.rs`), PatchMatch content-aware fill and inpaint
(`algo/src/content_aware.rs`, `inpaint.rs`), heuristic Select Subject/Sky.

**Formats.** `io/src/lib.rs:554-928,1082-1200` picks formats by hard-coded extension checks (pdn, affinity,
kra, xcf, ora, raw), else craft-libs codecs via the closed `Format` enum. No importer/exporter contribution
kind exists (`CE/crates/ext-core/src/manifest.rs:31-48`).

**UI.** The `craft` facade (runtime 287 refs, dialog 201, contract 161). Plugin panels are framework dock
entries `plugin.<plugin>.<panel>` (`ui-egui/src/panels/registry.rs:260-307`, `panel_docking.rs:249`)
**[corr: the continuation record says EC/PC beta panels float]**; plugin windows are `ViewportWindow`s
(`plugins_ui.rs:237`). The contract text still says "A floating plugin panel" (`crates/plugins/src/contract.rs:397`).

**Adapter.** 37 ops (14 query, 15 edit, 4 effect, 4 UI; `contract.rs`, implemented in
`engine/src/plugin_ops.rs:599-935`); `raster.filter@1.0`, `photocraft.session@1.2`, preset, LUT and theme
resources. Edit ops already call registry commands (`plugin_ops.rs:727-1020`) but plugins cannot name one.
57 allowlisted filters (`plugins/src/session_filters.rs:15`; Displace, Lens Blur, Lighting, Camera Raw,
Liquify, Vanishing Point and Filter Gallery excluded), 17 adjustments (`plugin_ops.rs:38-56`, no
`colorLookup`). Plugin filters are journaled (`plugin_filters.rs:135`), session commands are not and their
inner commands are stripped (`plugin_sessions.rs:611,676`). Missing plugins prompt per document
(`plugin_sessions.rs:80-102`). App events are polled: only `document.opened` and `selection.changed`
(`:125,128`).

### 1.3 FigCraft (Figma class)

**Commands and automation.** 630 specs from 46 modules (`FC/crates/engine/src/cmd.rs:73-133`).
`Session::execute` (`session/exec.rs:70-140`) checks parameter names for every command, rolls back on
failure, caps the journal at 5,000. External effects: `file.*` ×24, `collab.*` ×22, `library.*` ×19
(including `publishToServer`), `font.download`, invite links, `branch.*` ×7, clipboard, `pref.*` ×18.
MCP has 19 tools including `run_command`, `batch`, `pointer` with pressure and `wait_for_changes`.

**Document.** 16 node kinds (`doc/src/node.rs:86-149`); fills, strokes, 7 effect kinds (including glass,
noise, texture; `props.rs:847`), auto layout, grids, variables with modes, overrides, `plugin_data`
per node and per document keyed `plugin:<id>` (`lib.rs:255-257`). Design and Board (FigJam-like) file kinds.
Dev Mode: codegen for CSS, HTML, Swift, Compose, Flutter, ArkUI, Tailwind, `inspect.*`, `codeConnect.*`
(`engine/src/dev/`). Prototyping with an expression engine (`proto_expr*.rs`). `data.fill` (`content/datafill.rs:18`).

**Collaboration.** Property-level diffs ordered by an embedded server; last writer wins in server order;
fractional child keys; offline batches resend; branches three-way merge (`collab/src/lib.rs:1-21`). Protocol 3
syncs plugin data per owner key (`change.rs:260-261`) and plugin references (`:232-233`). Plugin edits are
grouped and synced on commit (`engine/src/ext.rs:38,73`). **The server does not check plugin ownership**;
only the local adapter does (`crates/plugins/src/runtime.rs:418-461`, commit `48f0eb6c`).

**Render, colour, formats, UI.** CPU tiny-skia with rayon; no WGSL. Documents are sRGB or Display P3;
ICC is written (`color/src/icc.rs`) and read via moxcms; CMYK/Lab images fall back to sRGB. Export:
PNG, JPG, SVG, PDF (krilla), WebP, VectorDrawable (`doc/src/props.rs:1310-1319`) **[corr: prior listed
three]**. Import is a hard-coded match with probing for `.fig`, `.sketch`, REST JSON, Penpot, SVG.
UI is egui 0.37; craft-ui at 3 sites; plugin panels in a custom sidebar dock saved in the `plugin_panels`
preference (`ui-egui/src/plugin_dock.rs`); plugin windows are in-viewport `egui::Window`s.

**Adapter.** 84 ops: 14 contract ops, 61 `EDIT_COMMANDS`, 6 `QUERY_COMMANDS`, 3 extra (`contract.rs:24-93`).
Only `selection.get` is selection-scoped; selection-scoped grants deferred (`contract.rs:47,57,120`).
`document.changed` payload is `{"event":"app","type":X,"name":X}` with no ids (`runtime.rs:511`). Read-only
guard refuses only `OpKind::Edit` (`runtime.rs:237,278`), so `selection.set`, `viewport.set`, `page.select`
run during change handlers. `ui.notify` logs only (`runtime.rs:397,407`). `export.svg` only, ≤ 4 MiB.
Tool events carry `{phase, at, modifiers}`, no pressure, no hover. The fifth-window bug is fixed (refused
with `Code::Budget`, `runtime.rs:391`). `crates/plugins/src/lib.rs:17-108` is still dead code.
`FIGCRAFT_SAFE_MODE=1` blocks dispatch (`lib.rs:13-15`).

### 1.4 VectorCraft (Illustrator class)

**Commands.** About 1,034 engine ids from 108 modules (`VC/crates/engine/src/cmd.rs:289-404`) plus about 290
`UI_COMMANDS` (`ui-egui/src/menus/ui_commands.rs:2`). `CommandSpec.schema` uses `crates/apischema`
(JSON-Schema 2020-12 shaped, validator and hash; `API_VERSION = 4`, `engine/src/schemas.rs:29`), but only 32
commands have schemas. External side effects are enumerated by the `vc_call` prefix denylist: `file.`
`document.` `clipboard.` `prefs.` `print.` `pdf.` `webExport.` `library.` `links.` `recovery.` `script.`
`action.` `help.` `view.` and copy/paste/undo/redo (`cmd/plugin_call.rs:34-85`). MCP: about 33 tools,
including `pointer_gesture` and `apply_effect`; a loopback JSON-lines control port.

**Document.** `NodeKind` Layer, Group, Path, Compound, Text, Image, SymbolInstance, Blend, Envelope, Mesh,
Repeat, PlacedDocument (`doc/src/node/kind.rs:32`); artboards, swatches, graphic styles, character and
paragraph styles, composite fonts, text threads, symbols (nine-slice), graph designs, variables
(datasets), patterns, slices, colour profiles and an `unknown` map (`doc/src/lib.rs:174`); perspective,
dimensions, inks, overprint, live paint, puppet warp. Appearance `Effect` records can sit on objects,
fills, strokes and graphic styles (`doc/src/appearance/layers.rs:7,34,86`; `graphic_style.rs:5-7`).

**Input, render, colour.** `PointerEvent.pressure` and `pen: Option<PenPose>` (tilt and barrel rotation;
`tools/src/input.rs:48-53`), tablet readers for XInput2, macOS and Windows. Rendering is vello_cpu 0.2;
no WGSL. moxcms ICC with CMYK profiles and soft proofing (`color/src/cms.rs`), spot inks and separations.
22 built-in formats including ai, eps, pdf, dxf, emf, wmf, hpgl, psd, svg (`fileio/formats.rs:365`), more
open-only (Affinity, JXL, HEIC, PICT), and runtime plugin formats (`:883-900`). craft-ui docking with 88
panel files. No event bus; revision counters (`menus/state.rs:166-169`).

**Legacy plugin system** (`crates/plugins`, about 12.3k lines, wasmi 2.0 with `portable-dispatch`):

| Aspect | Current behaviour (cite) | Migration disposition |
|---|---|---|
| ABI | exports `memory`, `vc_abi_version`, `vc_manifest`, `vc_alloc`, `vc_run` (`docs/plugins.md:345-351`) | bounded shim in the VC adapter (section 4.3) |
| Host imports | 11 in `env` at api 2: `vc_progress`, `vc_cancelled`, `vc_log`, `vc_settings_*`, `vc_cache_*`, `vc_call`, `vc_call_result` (`host.rs:217-238`) | map to `job.progress`, cancel flag, `log`, `settings.*` (H1), `cache.*` (H14), `app.run` |
| Kinds | filter, effect, exporter, importer (`manifest.rs:37-50`) | `command` + `vector.object` op set; `vector.effect@1`; `format.export@1`; `format.import@1` |
| Permissions | 10: `doc.read/write`, `state`, `effect.render`, `export.write`, `import.read`, `progress`, `settings`, `cache`, `commands` (`permissions.rs:80-111`); grants keyed `(id, sha256, perms)` | map to `document.read/edit`, contribution kinds; re-consent on first run under the platform |
| Limits | module 32 MiB, memory 512 MiB, fuel 50M + 4k/byte, wall 60 s; heavy 1e9/300 s; effects 5 s (`runtime.rs:77-91`, `effect.rs:35`) | platform budgets; keep the light/heavy split as `budget` manifest field |
| Live-effect cache | 512 entries / 1M anchors, keyed on install, params, geometry and paints (`effect.rs:37-42`) | platform `cache` keyed by plugin version and params hash |
| Deferred open | 2 s first-view budget then defer (`deferred.rs:27`) | keep in app as generic effect scheduling |
| Missing plugin | record kept, geometry drawn unchanged, no warning (`effects/src/lib.rs:25-27`) | **fix**: stored baked output + project prompt |
| Storage | `<prefs>/Plug-ins/<id>.wasm`, `grants.json`, `disabled.json`, `disabled.lock`, `storage/id-<id>/settings.json`, `cache/` (`engine/src/store.rs:15-71`) | import into CE store once; then delete |
| Install | `plugin.install` (path or base64), File › Open or drop of `.wasm`, delayed-OK dialog, CLI `--allow-plugin-grants` | retire; platform package install only |
| UI | Object › Plug-ins (after 17 built-in native add-on submenus, `ui-egui/src/addons.rs:24-45`), Effect › Plug-ins, Export As, Open With, appearance panel editing, palette, shortcuts `plugin:<id>` and `effect:plugin.<id>` | platform menus, palette and manager; rewrite shortcut keys |
| Commands | 10 `plugin.*` engine commands (`cmd/plugin.rs:1255-1345`) plus UI `plugin.dialog`, `plugin.cancelRuns`, `ui.installPlugin`, `help.safeMode`, `file.openWith` | replace with platform manager commands; keep `help.safeMode` routed to platform safe mode |
| Document data | `Document::unknown["pluginData"][id]` (`doc/src/plugin_data.rs`); exporters read all plugins' data | read forever, write as `plugin:<id>` owner keys; exporters see only their own |
| Tests | 177 crate, 190 engine, 55 UI | port the behavioural ones to the conformance suite |
| Non-plugin callers | scripts use `permissions::sha256_hex` and `display_text`; effect catalogue and format registry use `registry::intern`; export progress uses `host::PROGRESS_STEP` | move to `engine` utilities before deletion |

**Scripting.** `crates/js` is a bespoke ES5.1 interpreter with ExtendScript preprocessor support and limits
(fuel 1e9, heap 256 MiB). Scripts get `vc.run(id, params)`, `$`, dialogs and confined `File`/`Folder`, run as
one transaction and undo step, and throw on `plugin.*` and `prefs.*` (`engine/src/script/mod.rs:78-88,1539-1564`).
`crates/scriptcheck` statically assesses Illustrator `.jsx`/`.jsxbin`.

### 1.5 DesignCraft (InDesign class)

**Commands.** 567 engine ids (383 undoable, 109 `noundo`, 75 `query`) plus 194 UI; namespaces object 60,
type 44, table 42, file 42, layout 32, style 28. `query` and `noundo` include file writes, printing via `lp`
(`cmd/print.rs:120`), `prefs.set`, libraries and `color.loadProfile`. Param keys are checked only on the
control channel and MCP (`params.rs:224`, `ui-egui/src/control.rs:244`), so journals replay. The step runner
(`engine/src/script.rs`) accepts JSON, JSON lines or `id {json}` lines, `$N.path` and `${N.path}`
references, 8-deep nesting and 100k steps. **[corr]** It is one undo step only through `script.run`
(`cmd/scripts.rs:10-37`); MCP `batch` and the CLI `script` record one step each and do not roll back.

**Document.** Spreads and parents, layers, sections, articles, stories, conditions, hyperlinks, footnotes,
cross-reference formats, index, TOC, 9 text-variable kinds, paragraph/character/object/table/cell styles,
GREP and nested styles, composite fonts and mojikumi, tables (42 commands), data merge with text, image and
QR fields from CSV, TSV, XLSX (`doc/src/datamerge.rs:16-17,255`), books (`.dcbook`), track changes. Preflight is
one command with hard-coded checks and `Issue.kind: &'static str` (`cmd/preflight.rs:19,295`); the live part
is a UI worker (`ui-egui/src/preflight_worker.rs`). Fonts: bundled, installed, document-folder and added
sources; OS/2 fsType embedding flags are read and **enforced** in PDF, packaging and PPTX
(`fonts/fontdb/face.rs:296-310`, `cmd/export/formats.rs:252`, `cmd/package.rs:403`).

**Other.** No pen pressure; tools emit actions and never mutate (`tools/src/lib.rs:115,500`). vello_cpu
rendering. craft-cms with sRGB/Adobe RGB/P3/ProPhoto, FOGRA39 and Generic CMYK, Lab ΔE76/ΔE2000, soft proof,
separations (craft-prepress); display is always sRGB. PDF/X-4, X-1a, A-2b, tagged; EPUB reflowable and fixed,
HTML, IDML, PPTX, text, XML, package. Opens IDML and INDD (via craft-indd). No AI. craft-ui docking with
43 panels. Change tracking: `DocState.revision` and a public `Session.journal` (`engine/src/lib.rs:97-98,413`).

### 1.6 LightCraft (Lightroom class)

**Commands.** 391 engine ids (283 journaled, 108 query) plus 213 UI; namespaces library 48, photo 44,
mask 31, develop 29, keyword 23, faces 21, print 20. `run_command` (`engine/src/lib.rs:608`) writes an XMP
sidecar after each command (`:630-633`) and persists the catalogue. External effects: deletes, renames,
folder moves, metadata to file, three model downloads (all `query`), GeoNames download, `export.addToPhotos`,
20 print commands. MCP generates one tool per command (`mcp/src/tools.rs:341-416`); `command_batch` has no
references and no undo grouping.

**Catalogue and develop.** Op-log catalogue with 36 `Op` variants including `Batch`, snapshot plus JSON-lines
log and a writer thread (`catalog/src/journal.rs`, `log_writer.rs`), replaceable `Store` (OPFS on the web).
`Photo` has rating, flag, label, develop settings, versions, history with cursor, virtual copies, analysis
and content hash (`catalog/src/model.rs:259`). Smart albums use `RuleSet` with nine rule kinds
(`rules.rs:22-31`). `DevelopSettings` has 29 sections (`develop/src/settings.rs:128`); 125 typed slider
controls plus 10 indexed templates. Masks: brush, linear, radial, colour, luminance, depth, subject, sky,
background, object and prompt (SAM 3), people, landscape **[corr: subject, sky and people use embedded
mattes or saliency, not a model; `pipeline/src/masks.rs:95-130`]**. Presets: XMP, lrtemplate, DNG,
Luminar; profiles: `.cube` LUTs and DCP. LUTs run on the GPU too (`gpu/src/wgsl/finish.wgsl:383`)
**[corr]**; the LUT registry is mutable at runtime (`pipeline/src/lut.rs:50-51`).

**GPU, colour, AI.** wgpu 30.0.1 compute twin of the CPU pipeline (linear Rec.2020). Monitor profiles,
soft proofing, ICC print, HDR gain-map JPEG and PQ AVIF/PNG/JXL; HDR display reports SDR because eframe
picks an 8-bit surface. Denoise (RawNIND UtNet2), faces (YuNet, SFace, AuraFace; user `.onnx` accepted with
licence prompt), SAM 3 via candle; `crates/fetch` verifies SHA-256 only where pinned; SAM 3 tokenizer
files are unpinned (`segment/src/fetch.rs:20-21`). Denoise and SAM 3 can run on a remote worker over
unencrypted TCP (`denoise/src/remote.rs`).

**Export.** JPEG, PNG, TIFF, PSD, WebP, AVIF, JXL, Original+XMP, DNG, RenderedDng; resize, sharpen, size
limit, naming, metadata filters, location removal, colour space, depth, HDR, text or image watermark
(`export/options.rs`, `export/watermark.rs:23`). `AfterExport` exists but only adds to Apple Photos
(`export/prepare.rs:316-410`); Edit In launches external editors (`apps/lightcraft/src/editors.rs`). No
publish services. craft-ui docking manages only Canvas, Library, Tools and Presets.

### 1.7 Platform facts that bound the design (CE)

- wasmi 2 with `std, validate, deterministic, portable-dispatch` (`crates/ext-host/Cargo.toml:18`); no SIMD.
- Contribution kinds: Effect, Command, Panel, Tool, EventHook, Preset, Lut, Font, Theme, Data
  (`crates/ext-core/src/manifest.rs:31-48`). No importer, exporter, codegen, preflight, brush or
  service kinds.
- Permissions parsed: `document.read/edit:<scope>`, `files.read/write:picked`, `network:<host>`,
  `clipboard.read/write`, `ui.window`, `app.events` (`crates/ext-core/src/permission.rs:83-91`). No
  `net.*`, `settings.*`, `clipboard.*`, `secret.*`, `nn.*` or `gpu.*` op exists (grep over `crates/`).
- 16 kernels: bilateral, blend, blur, color_matrix, convolve, curves, directional_blur, histogram,
  hsl_range, mix, mix_masked, morphology, noise, resample, splat, warp_affine.
- UI level 1.2 (`crates/ext-core/src/ui/tree.rs:29`). `project.data`: 64 keys, 256 KiB per place, 4 MiB
  per plugin per document (`docs/plugins/capabilities.md:212`).

## 2. What the inventory changes in the target design

These refine the program's sections 2.3, 2.4 and 4 for the image and design apps.

1. **Registry bridge = one reviewed table, shared with automation gates.** Lift VC's mechanism into the
   platform as `craft_ext_runtime::bridge`: per-app `bridge.toml` rows `{id, class, scope, schema?}`
   where class is `query`, `edit`, `view` (selection, viewport, page, tool: UI state, not document),
   `file` (allowed only with a picked handle substituted for the path), `external` (network, print,
   other apps), `prefs`, `collab`, `plugin-admin` or `refused`. A test per app, copied from
   `VC/crates/engine/src/cmd/plugin_call.rs:681-694`, fails on any unclassified id. PC's automation gate
   (`automation/src/workspace.rs:625-699`) reads the same table so MCP and plugins cannot drift. `view`
   commands need `app.events`-safe treatment: allowed in user-initiated events, refused in change
   handlers (fixes FC's guard gap).
2. **Schemas come from VC's `apischema`.** Only VC has JSON-Schema-shaped params (32 commands). Move
   `apischema` to craft-libs and backfill schemas for every `edit`-class command the bridge exposes,
   starting with the ones the catalogue needs. Unknown keys are refused at the bridge regardless of the
   app's own leniency (DC deliberately accepts unknown keys for journal replay).
3. **Change events from existing sources.** FC: collab `Change` entries already name node ids and
   properties; publish them as `document.changed{revision, changes}` after each commit. LC: map catalogue
   `Op` variants (SetRating, SetDevelop, SetMeta, album ops) to `catalog.changed`. PC, VC, DC: emit the
   journal entries committed since the last pump plus PC's `last_damage` rect for pixel plugins.
   No engine-wide event bus is needed first.
4. **Pixel model follows UXP Imaging, not today's filter buffer.** `pixels.read{target: layer|mask|
   vector-mask|channel|selection|composite, bounds, level, space: native|rgb|gray|lab|cmyk, depth: 8|16|32,
   profile?}` returning a host image handle; `pixels.write` into the same targets with history labels.
   Handles map onto PC's 256-pixel tiles so the spill system (`raster/src/spill.rs`) keeps working.
5. **GPU is a platform service on the eframe device.** Four apps have no own compute pipeline; the
   service owns WGSL validation (naga), pipeline cache, submission budgets and readback. Headless runs
   create a device; no GPU means a CPU kernel twin or refusal declared in the manifest.
6. **Non-destructive by stored recipe plus stored result.** Every plugin result that the app re-renders
   (PC smart filter, VC/FC live effect, LC develop stage, DC preflight annotation) stores
   `{plugin, version, contribution, params, deterministic, cached_output}`. Missing plugin: draw the cached
   output, badge it, prompt once per document; never draw the unmodified source silently.
7. **Contract additions for this family** beyond the program's list: `document.check@1` (one contract
   for preflight rules, Sketch Assistants and lint), `export.postprocess@1` (LR export filters),
   `datamerge.source@1` (EasyCatalog-class live data), `pdf.post@1`, `text.review@1` (Figma text
   review, spell and grammar providers), `asset.source@1` (host-drawn asset browser for stock, icon and
   DAM providers), and a `network:loopback` permission decision for local AI servers.

## 3. Plugin catalogue

110 named plugins and plugin classes. "Needs" is the precise host capability; "Status" is today's state in our nearest app;
the last column names the blocking layer and location, then the proposed op or contract. Commercial
behaviour is vendor-described, not inspected. API references: Photoshop UXP
([Imaging](https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/imaging),
[batchPlay](https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/batchplay)), Lightroom SDK
([mirror](https://lrc.mcor.dev/)), Figma ([plugins](https://developers.figma.com/docs/plugins/),
[codegen](https://developers.figma.com/docs/plugins/codegen-plugins),
[figma global](https://developers.figma.com/docs/plugins/api/figma/)), InDesign UXP
([network](https://developer.adobe.com/indesign/uxp/resources/recipes/network/)), Capture One
([developer](https://www.captureone.com/partnerships/developer)), GIMP
([filters from plug-ins](https://developer.gimp.org/resource/writing-a-plug-in/tutorial-gegl-ops)), Krita
([Python](https://docs.krita.org/user_manual/python_scripting/krita_python_plugin_howto.html)), Photopea
([plugins](https://www.photopea.com/api/plugins)), Inkscape
([extensions](https://inkscape.gitlab.io/extensions/documentation/)), Sketch
([JS API](https://developer.sketch.com/plugins/javascript-api)), Penpot
([plugins](https://help.penpot.app/plugins/getting-started/)), Canva
([apps](https://www.canva.dev/docs/apps/)).

### 3.1 Photoshop, Affinity, GIMP, Krita, Photopea (target: PhotoCraft)

| # | Plugin | Does | Needs | Status | Gap → proposal |
|---|---|---|---|---|---|
| 1 | Topaz Photo AI | AI denoise, sharpen, face recovery | full-res pixels, 0.3–2 GB models on GPU, background job | ◐ | Pl no inference, base64 pixels → `nn.run`, `pixels.read` handles |
| 2 | Topaz Gigapixel | 2–6× upscale | output larger than input, tiled with overlap, new document | ✖ | Ad no resize/new doc (`plugin_ops.rs`) → `raster.filter@2{output_scale}`, `document.new` |
| 3 | Magnific (plugin) | cloud upscale back onto layer | upload, poll, download, API key | ✖ | Pl no `net.fetch`/secrets → H2, H3 |
| 4 | remove.bg | cloud cutout to mask | write layer mask | ✖ | Ad filters refuse masks (`plugin_filters.rs:49-70`) → `pixels.write{target:mask}` |
| 5 | Nik Color Efex | stacked GPU looks | proxy live preview, smart filter, local masks | ◐ | Pl no GPU or live sliders; App no plugin smart filters → `gpu.shader@1`, U1, X-5 |
| 6 | Nik Viveza | U Point local edits | draggable on-canvas control points | ◐ | Pl overlay lacks hit regions → U2/U3 overlay regions |
| 7 | Nik Silver Efex | B&W and film looks | Gray output, 16-bit | ◐ | Ad non-RGB refused (`plugin_filters.rs:49`) → `space:gray` targets |
| 8 | Nik Dfine | profile-based denoise | per-camera profiles kept between runs | ◐ | Pl no settings or cache → H1, H14 |
| 9 | Imagenomic Portraiture | skin smoothing with auto mask | face detection, skin mask, new layer | ◐ | Ad YuNet in app (`algo/src/faces.rs`) unexposed → `analysis.faces`, `layer.from_pixels` |
| 10 | Retouch4me Heal | AI blemish heal | nn + content-aware heal | ◐ | Ad PatchMatch (`algo/src/content_aware.rs`) unexposed → bridge `edit.fill{contentAware}` |
| 11 | Retouch4me Dodge & Burn | AI D&B as curves layers | create and edit adjustment layers | ◐ | Ad adjustments create-only → `adjustment.update` |
| 12 | Boris FX Optics | 160 lens and film filters | GPU, own window, smart filter | ◐ | Pl GPU; App smart filters → `gpu.shader@1`, X-5 |
| 13 | Filter Forge | node-graph filter builder | user graphs, seamless tiling | ◐ | Pl no graph tier → `kernel.graph`, `graph.node@1` |
| 14 | Knoll Light Factory | lens flares | positioned handles, out-of-bounds output | ◐ | Pl handles; ROI output bounds → U3, `raster.filter@2` |
| 15 | G'MIC-Qt | 500+ scripted filters (PS, GIMP, Krita) | interpreter, preview, multi-layer input | ◐ | Pl speed, one input → Wasmtime tier, `needs:["layers"]` |
| 16 | Infinite Color / Texture | randomised looks, textures | adjustment edits, bundled images as layers | ◐ | Ad, Pl → `adjustment.update`, `asset.read` |
| 17 | Lumenzia | luminosity masks | channel math, overlay preview, channel→selection | ◐ | Ad no channel pixels → `pixels.read{target:channel}`, `selection.from_channel`, `canvas.preview.set` |
| 18 | TK8 | luminosity panel plus actions | run recorded actions | ◐ | Ad → bridge `actions.play` as user-confirmed `edit` |
| 19 | Retouching Toolkit (UXP) | panel over batchPlay | breadth, modal edits, settings | ◐ | Ad 37 ops; Pl settings → `app.run`, H1 |
| 20 | Delicious Retouch | frequency separation, D&B | groups, filters, blend modes | ◐ | Ad → `app.run` |
| 21 | Coolorus | colour wheel | fg/bg get/set, colour events | ◐ | Ad no colour ops → `color.fg/bg`, `color.changed` |
| 22 | Lazy Nezumi Pro | pen smoothing for any app | OS input hook | ✖ native | In-app: Ad drops tilt/rotation (`plugin_sessions.rs:368-375`) → `input.filter@1` |
| 23 | Kyle T. Webster brushes | ABR packs | brush resources | ◐ | App imports ABR (`io/src/abr_map.rs`); Pl kind → `resource.brush@1` |
| 24 | Fontself Maker | colour fonts from layers | layer pixels/paths, OTF write, install | ✖ | Ad path read; Pl → `path.get`, `font.install` |
| 25 | SD/ComfyUI bridges | local generative fill | loopback HTTP/WebSocket, selection, result layer | ✖ | Pl refuses `network:localhost` (`permission.rs:203`) → decide `network:loopback` |
| 26 | Luminar Neo / ON1 Effects | round trip to standalone | write TIFF, launch app, re-import | ✖ native | → native helper + `handoff.open` |
| 27 | Perfectly Clear | face-aware auto correct | faces, batch | ◐ | as 9 plus A-9 batch |
| 28 | Export Kit | PSD to HTML/React | text runs, styles, per-layer renders | ◐ | Ad text/style opaque → `text.get`, `style.get` |
| 29 | Image Processor Pro | batch export | format options, folder grant | ◐ | Ad no options; Pl one-shot handles → `document.export{options}`, H5 folder grant |
| 30 | NVIDIA Texture Tools Exporter | DDS with mipmaps | layered draft → bytes | ✖ | App closed `Format` enum; Pl no kind → `format.export@1` |
| 31 | Envato / Adobe Stock panels | search, license, place | net, OAuth, drag-drop, linked place | ◐ | Pl H2, H3, H7; Ad `smart.place{bytes}` |
| 32 | PixelSquid | 3D object renders by angle | replace smart-object contents | ✖ | Ad → `smart.replace_contents` |
| 33 | Affinity macros | record and replay | plugin steps in recordings | ◐ | Ad session commands `journal:false` (`plugin_sessions.rs:676`) → U8 recorded params |
| 34 | Affinity/PS 8bf hosting | load Photoshop filter binaries | native ABI | ✖ | out of scope; decision 6 |
| 35 | Resynthesizer (GIMP) | heal by texture synthesis | random reads outside ROI | ◐ | Pl ROI only → `needs:["context"]` |
| 36 | BIMP (GIMP) | batch manipulation | iterate folders, save | ◐ | Ad droplets skip plugin commands → A-9 |
| 37 | Krita AI Diffusion | ComfyUI generation, live painting | loopback socket, stroke events, layers | ✖ | Pl loopback, events with damage → H4, decision |
| 38 | Krita Shortcut Composer | pie menus | tool and brush switching | ✖ | Ad → bridge `view` class `tool.set` |
| 39 | Photopea plugins | iframe panels sending scripts | script surface, binary in/out | ◐ | Ad → `app.run`, R6 binary |

### 3.2 Lightroom Classic and Capture One (target: LightCraft)

| # | Plugin | Does | Needs | Status | Gap → proposal |
|---|---|---|---|---|---|
| 40 | Flickr (Friedl) | publish service | rendered files, OAuth, remote id/URL per photo, republish on edit, comments | ✖ | App no publish model; Ad none → X-1, `publish.service@1` |
| 41 | SmugMug | publish service | as 40 plus galleries from collections | ✖ | as 40 |
| 42 | Glass / eye.photo (Capture One) | publish | as 40 | ✖ | as 40 |
| 43 | LR/Mogrify 2 | borders, watermarks, overlays | pixel transform inside export chain | ✖ | App `AfterExport` only Apple Photos (`export/prepare.rs:316-410`) → `export.postprocess@1` |
| 44 | Metadata Wrangler | strip/alter metadata on export | edit rendered file metadata | ◐ | App has metadata filters → `export.postprocess@1` |
| 45 | Any Filter | custom collection criteria | catalogue query with expressions | ✖ | Ad → `catalog.query` over `RuleSet` (`rules.rs:22`) |
| 46 | Data Explorer | statistics over catalogue | paged bulk metadata reads | ✖ | Ad → `catalog.query{fields,page}` |
| 47 | LR/Transporter | metadata to/from text | batch metadata writes, files | ✖ | Ad → `metadata.set_many` (one `Op::Batch`) |
| 48 | Negative Lab Pro | negative conversion | analyse pixels, write curves, install profile, sync roll | ✖ | Ad, App profile slot → `photo.render`, `develop.set`, `resource.profile` |
| 49 | LRTimelapse | keyframed develop across sequences | develop writes on hundreds of photos | ✖ | Ad → `develop.set_many` |
| 50 | Imagen | cloud AI editing profiles | upload previews, write develop | ✖ | Pl net; Ad → H2, `develop.set` |
| 51 | Excire Foto/Search | AI keywords and search | nn on previews, keyword writes, search panel | ✖ | Pl nn; Ad → `nn.run`, `metadata.provider@1` |
| 52 | Teekesselchen | duplicate finder | content hashes, collection create | ✖ | Ad → `catalog.query{content_hash}`, bridge `album.*` |
| 53 | MIDI2LR / Loupedeck | hardware sliders | relative develop deltas, low latency | ✖ | Ad delta op; device is native → `develop.adjust`, native helper |
| 54 | Jeffrey's Geoencoding | GPS lookup | net, metadata writes | ◐ | App offline geocode; Pl net → H2 |
| 55 | Preset/profile packs | data | XMP, DCP, LUT install | ◐ | App import exists (`engine/src/preset_import.rs`) → `resource.preset/profile/lut` for LC |
| 56 | ON1 / Topaz "Edit In" | external editor round trip | TIFF out and back, stack | ◐ | App `editors.rs`; suite → `handoff.open` to PC |

### 3.3 Illustrator, Inkscape (target: VectorCraft)

| # | Plugin | Does | Needs | Status | Gap → proposal |
|---|---|---|---|---|---|
| 57 | Astute VectorScribe | dynamic shapes, corners, measure | custom tools, live handles, hover, anchor edits | ✖ | Ad none; Pl overlay regions → VC adapter, U2 |
| 58 | ColliderScribe | collision-aware drawing | intersect/distance during drag | ✖ | Ad → `geom.intersect/distance/hit_test` |
| 59 | Phantasm | live halftones, recolour | raster output in appearance, image pixels | ✖ | App appearance takes geometry/painted output only → `vector.effect@1{output:raster}` |
| 60 | Texturino | textures on vectors | raster blend in appearance, bundled assets | ✖ | as 59 + `asset.read` |
| 61 | Astute Stylism / WidthScribe | live styles, width profiles | appearance and width edits | ✖ | Ad → `app.run` (705 commands already reviewed) |
| 62 | Astute MirrorMe | live symmetry drawing | input transform, live copies | ✖ | → `input.filter@1` + `vector.effect@1` |
| 63 | Hot Door CADtools | dimensions, scaled drawing | dimension objects, tools | ✖ | App has `doc/src/dimension.rs` → bridge + tool |
| 64 | Avenza MAPublisher | GIS import, attributes, styling | Shapefile/GeoJSON importer, attribute tables | ◐ | legacy importer kind; Ad → `format.import@1`, `node.attributes` |
| 65 | Fontself (Illustrator) | vector glyphs → OTF | path reads, font install | ✖ | Ad, Pl → `font.install` |
| 66 | Esko DeskPack | packaging prepress | inks, overprint, traps, barcodes | ◐ | App trapping/inks exist (`pathops/`, `doc/src/inks.rs`) → bridge + `document.check@1` |
| 67 | Illustrator scripts (Osokin, Sato) | utility scripts | DOM-like access | ◐ | VC JS engine with `vc.run`; DOM absent by design → generated typed bindings |
| 68 | Cutting Master plotters | send paths to cutter | device | ✖ native | → native helper consuming `format.export` bytes |
| 69 | AxiDraw (Inkscape) | pen plotter | serial device | ✖ native | as 68 |
| 70 | TexText (Inkscape) | LaTeX as editable vectors | TeX engine, source kept in object | ✖ | native restricted helper; `plugin_data` source |
| 71 | Boxes.py (Inkscape) | laser-cut box generator | create paths in units | ✖ | Ad → command + `vector.create` |
| 72 | Hershey Text (Inkscape) | single-stroke text | text to strokes | ✖ | Ad → command, `fonts.glyphs` |

### 3.4 Figma, Sketch, Penpot, Canva (target: FigCraft)

| # | Plugin | Does | Needs | Status | Gap → proposal |
|---|---|---|---|---|---|
| 73 | Iconify | insert icons | net, SVG as nodes, drag-drop | ◐ | Pl net, drag; Ad no SVG import op → H2, H7, `svg.import` |
| 74 | Unsplash | stock photos | net, image fills | ◐ | Pl → H2 |
| 75 | Content Reel | realistic content | text and image writes | ✔ | — |
| 76 | Lorem ipsum | filler text | text writes | ✔ | — |
| 77 | html.to.design | web page to layers | net, thousands of node creates | ◐ | Pl → H2, `ops.batch` (10.5 µs per call) |
| 78 | Remove BG | background removal on fills | fill bytes read, net or nn | ✖ | Ad no `image.get` → A-12 |
| 79 | Autoflow | flow connectors | change events with ids, deferred edits | ◐ | Ad payload has no ids (`runtime.rs:511`) → H4 from collab diffs |
| 80 | Stark | contrast, vision sim, annotations | render nodes to PNG | ◐ | Ad `export.svg` only → `export.png` buffer |
| 81 | Tokens Studio | tokens, Git sync | variables import/export, OAuth | ◐ | Pl, Ad → H2, H3, `variable.import/export` |
| 82 | Anima | code export | tree, assets, codegen | ◐ | Ad → `codegen@1`, `export.png` |
| 83 | Locofy | code export | as 82 | ◐ | as 82 |
| 84 | Builder.io Visual Copilot | AI code export | as 82 + net | ◐ | as 82 + H2 |
| 85 | Dev Mode codegen (Tailwind, Relay) | code in inspect panel | codegen hook, 3 s deadline | ✖ | Ad; engine has `dev/codegen` → `codegen@1` |
| 86 | Design Lint | find detached styles | tree reads, live re-check | ◐ | Ad events → `document.check@1` |
| 87 | Rename It / Similayer | batch rename, select similar | names, `selection.set` | ✔ | — |
| 88 | Figma Chart | charts from data | CSV, clipboard | ◐ | Pl no clipboard/files in FC → H6, H5 |
| 89 | LottieFiles | Lottie import/export | prototype read, JSON out/in | ◐ | Pl → `format.import/export@1` |
| 90 | Magician | AI copy, icons, text review | net, text-review mode | ◐ | Pl → H2, `text.review@1` |
| 91 | Crowdin / Localazy | localisation sync | bulk text, string ids in plugin data | ◐ | Pl → H2, H3 |
| 92 | Mockup plugins | screens into devices | render frames, warp into fill | ✖ | Ad → `export.png`, `raster.filter` on fills |
| 93 | Voting/Jira widgets | on-canvas interactive nodes | widget node, synced state | ✖ | App → `widget@1` (P3) |
| 94 | Sketch Runner | palette, insert symbols | commands, instances | ◐ | Pl → U8 palette |
| 95 | Sketch Assistants | lint rule packages | rules returning node issues | ✖ | → `document.check@1` |
| 96 | Penpot plugins | scoped content/library/comment API | `library:*`, `comment:*` scopes | ◐ | Pl permission vocabulary lacks library/comment → add scopes |

### 3.5 InDesign, QuarkXPress, Affinity Publisher, Canva (target: DesignCraft)

| # | Plugin | Does | Needs | Status | Gap → proposal |
|---|---|---|---|---|---|
| 97 | EasyCatalog (65bit) | live database publishing | data panel, field links, re-sync, HTTP/ODBC | ✖ | Ad none; App merge is CSV/TSV/XLSX → `datamerge.source@1` |
| 98 | Em Software InData | template-driven import | records → frames | ✖ | Ad → `app.run` + `script.run` |
| 99 | Em Software Xtags | tagged text import | story import filter | ✖ | Pl → `story.import@1` |
| 100 | Em Software WordsFlow | live-linked Word stories | DOCX link refresh, export back | ◐ | App DOCX import → `format.export@1` (story) |
| 101 | Typefi | XML/Word to automated layout | headless templates, server | ◐ | MCP/CLI ✔; → `craft-ext run --app designcraft` |
| 102 | Blatner Tools | style automation, paging | find/change, styles, pages | ✖ | Ad → bridge |
| 103 | Peter Kahrel scripts | index, footnotes, GREP | styled runs, GREP | ✖ | Ad → `story.get/set` runs |
| 104 | Imposer Pro / Quite Imposing | imposition | page order, signatures, PDF post | ✖ | App → `pdf.post@1` |
| 105 | Enfocus PitStop | PDF preflight and fixups | rule profiles, fixes | ✖ | App `Issue.kind: &'static str` (`cmd/preflight.rs:19`) → `document.check@1`, `pdf.post@1` |
| 106 | WoodWing Studio | editorial check-in/out | network locks, story sync | ✖ | Pl, App story locks → L, possibly native |
| 107 | Redokun / Smartling | translation round trip | text extract, XLIFF, write runs | ✖ | Ad → `story.get/set` |
| 108 | Markzware Q2ID | QuarkXPress import | QXP → layout draft | ✖ | Pl → `format.import@1` layout drafts |
| 109 | Quark Xpert Tools Pro | XTension productivity tools | layout commands | ✖ | Ad → bridge |
| 110 | Canva Bynder / DAM apps | asset source | OAuth, browser, insert | ◐ | Pl → `asset.source@1` |

**Sandbox reachability:** with the program's
P0/P1 platform work and the per-app items in section 4, all rows are reachable in Wasm, WGSL or data tiers
except nine native rows (22 OS hook; 26 and 56 external apps; 34 8bf; 53 device part; 68 and 69
devices; 70 TeX; 106 vendor server, likely) and two rows (25, 37) pending the loopback decision.

## 4. Per-app change lists

Platform items referenced by id (R, K, H, U, D) are the program's section 4. Only items owned by, or
specific to, these apps are listed; "Dep" names prerequisites.

### 4.1 Shared platform items this family needs first

| # | Item | P | E | Dep |
|---|---|---|---|---|
| S1 | `craft_ext_runtime::bridge`: reviewed `bridge.toml` per app, classes from section 2.1, picked-handle substitution for `file`, CI test that every registry id is classified (port of `VC/.../plugin_call.rs:681-694`) | P0 | M | — |
| S2 | `apischema` moved to craft-libs; bridge validates against it; schema backfill tool that drafts schemas from the free-text `params` strings for review | P0 | M | S1 |
| S3 | `pixels.read/write` v2 with targets, spaces, depths, `level`, host image handles over app tiles | P0 | M | R5, R6 |
| S4 | `document.changed` built from app change sources; `view`-class ops refused in change handlers | P0 | M | H4 |
| S5 | Contribution kinds `importer`, `exporter`, `check`, `codegen`, `brush`, `input-filter`, `export-postprocess`, `publish-service`, `metadata-provider`, `datamerge-source`, `asset-source` in `ext-core/src/manifest.rs` | P0 | S | — |
| S6 | Stored-result rule for non-destructive contributions: `{plugin, version, params, deterministic, cached_output}`, missing badge, one prompt | P0 | M | — |
| S7 | Permission vocabulary: `document.edit:selection` live, `library.read/write`, `comment.write`, `catalog.read/edit`, decision on `network:loopback` | P1 | S | — |
| S8 | GPU compute service on the eframe device (naga validation, budgets, readback) shared by PC, FC, VC, DC, LC | P1 | L | K7 |
| S9 | `nn.run` on candle with WGSL conv fast path; model packages pinned by sha256; download consent | P1 | L | K8 corrected |

### 4.2 PhotoCraft

| # | Layer | Change | P | E | Dep |
|---|---|---|---|---|---|
| PC1 | Ad | `bridge.toml` over ~809 commands; merge with automation gate (`automation/src/workspace.rs:625-699`) | P0 | M | S1 |
| PC2 | Ad | Pixel targets: layer mask, vector mask (read), channels, Quick Mask, selection, composite; Gray, CMYK, Lab, Multichannel at U8/U16/F32; drop the sRGB-only conversion in `plugin_filters.rs:279-392` | P0 | M | S3 |
| PC3 | Ad | Tile-streamed filters (`raster.filter@2`): ROI with padding, proxy level, output bounds; stop compiling twice and copying ~6× | P0 | M | R3, R5 |
| PC4 | App | Smart filters dispatch `plugin.<plugin>/<id>`; allow plugin filters on smart objects; store cached raster; unknown id shows cached raster plus warning instead of untouched pixels (`smart_cmds/rendering.rs:158-174`) | P0 | M | S6 |
| PC5 | Ad | Tool events: tilt, rotation, eraser, pointer kind, hover, coalesced samples (`plugin_sessions.rs:368-375`, `plugins_ui.rs:362`) | P0 | S | — |
| PC6 | Ad | Ops: `adjustment.update`, `style.get/set`, `text.get/set`, `path.get/set`, `smart.place/replace_contents`, `color.fg/bg`, `swatches.*`, `document.new/resize/canvas_size/crop`, `layer.from_pixels`, `metadata.get/set` (most via bridge) | P0 | M | PC1 |
| PC7 | Ad | `analysis.faces` (YuNet), content-aware fill, Liquify with plugin mesh, Camera Raw settings on RGB layers | P1 | M | PC1 |
| PC8 | Ad | Journal session commands with recorded params; `actions.play` user-confirmed; droplets and batch run plugin commands | P1 | M | U8 |
| PC9 | App | wgpu compute pipeline beside the fragment compositor; canvas preview layer for `canvas.preview.set` | P1 | M | S8 |
| PC10 | App/Ad | `format.import/export@1` behind the closed `Format` enum (`io/src/lib.rs:554-928`): plugin formats after built-ins | P1 | M | S5 |
| PC11 | App/Ad | `brush.dab@1` and `resource.brush@1` in `paint/`; `paint.stroke` op | P1 | L | K2, S8 |
| PC12 | Ad | Fix: contract slot text says "floating" (`contract.rs:397`); grant store opened ~8× per op; UI tree re-serialised per call | P2 | S | — |

### 4.3 VectorCraft (new adapter, legacy migration and removal)

| # | Layer | Change | P | E | Dep |
|---|---|---|---|---|---|
| VC1 | Ad | New `crates/craft-plugins` adapter on `Runtime<A>`: `vectorcraft.session@1`, commands, panels, tools, events, `project.data` | P0 | L | program 2.2 |
| VC2 | Ad | Bridge: convert `reviewed.rs` (705 allowed, 13 denied) into `bridge.toml`; reuse the prefix and file-word rules | P0 | S | S1 |
| VC3 | App | Appearance `Effect` ids resolve through a registry that includes `ext:` contributions; outputs geometry, painted pieces or raster; stored cached output | P0 | M | S6 |
| VC4 | Ad | `vector.effect@1`, `format.import@1`, `format.export@1`, `geom.*` queries, overlay hit regions and hover in tool events, `PenPose` tilt and barrel | P0 | M | VC1 |
| VC5 | Ad | Temporary legacy shim (below) | P0 | M | VC1 |
| VC6 | App | Remove `crates/plugins`, 10 `plugin.*` commands, plugin dialogs, Effect/Object › Plug-ins wasm entries, `pluginsFolder` pref, store files; keep 17 native add-on submenus under a renamed menu | P1 | M | migration complete |
| VC7 | App | Keep `crates/js` and scripts; route `vc.run` through the same bridge table so scripts and plugins share one classification; move `sha256_hex`, `display_text`, `intern`, `PROGRESS_STEP` to engine utilities | P1 | S | VC2 |

**Migration plan, in order.**

1. *Inventory and freeze.* Freeze new legacy features. Record each legacy capability and stored reference
   with its disposition (table in 1.4). Owner: VC adapter maintainer.
2. *Platform parity.* Land H1 settings and H14 cache, `job.progress`/cancel, the bridge (VC2), VC3 and VC4.
   Parity gate: each of the 4 kinds and 11 host imports has a platform equivalent covered by conformance
   tests, including deferred open budgets and the effect cache keyed by geometry and paints.
3. *Shim with removal criteria.* The VC adapter loads a legacy module (`vc_abi_version() == 1`) inside the
   platform Wasm runtime and provides `env.vc_*` imports implemented with platform ops; its manifest is
   synthesised from `vc_manifest` and requires fresh platform consent. Purpose: open old documents and run
   installed plugins while authors port. Owner: VC adapter maintainer. Removal: two public releases
   after parity, or earlier when telemetry-free catalogue review shows no unported legacy plugin in the
   suite catalogue; the shim is then deleted, not disabled.
4. *Data migration on first launch.* Import `<prefs>/Plug-ins/*.wasm` into the platform store as
   shim-backed packages; convert `grants.json` to *pending* platform grants (never silently granted),
   `disabled.json` to platform disabled state, settings JSON to H1 storage; drop cache. Rewrite shortcut
   keys `plugin:<id>` and `effect:plugin.<id>` to platform command ids, keeping the old keys readable.
5. *Document migration.* On open, read `Effect{id:"plugin.<id>"}` and `unknown["pluginData"][id]` forever.
   On save, write `ext:<package>/<contribution>` with version, params and cached output, and plugin data
   under `plugin:<id>`. Graphic styles and style libraries convert the same way. Action journals replay
   through an id alias table. Exporters lose the right to read other plugins' data.
6. *Remove* (VC6) once the shim criteria are met; delete docs/plugins.md sections, WAT templates and
   tests, keeping behavioural tests ported to the conformance suite.

### 4.4 FigCraft

| # | Layer | Change | P | E | Dep |
|---|---|---|---|---|---|
| FC1 | Ad | Replace the closed 67-command allowlist with the bridge over 630 commands; `selection.set`, `viewport.set`, `page.select` become `view` class | P0 | S | S1 |
| FC2 | Ad | `document.changed{changes}` from collab `Change` entries; deferred follow-up edits with loop detection | P0 | M | S4 |
| FC3 | App | Server-side plugin-data ownership in `collab` (`server.rs` currently trusts clients); changes carry plugin id | P0 | M | — |
| FC4 | Ad | `image.get` and `raster.filter` on image fills; `export.png/pdf/webp` to buffers; `svg.import` as nodes | P0 | M | S3 |
| FC5 | Ad | Selection-scoped grants; `ui.notify` toast; pressure in tool events; delete dead `lib.rs:17-108` | P0 | S | S7 |
| FC6 | Ad | `codegen@1` on `engine/src/dev/codegen`; `text.review@1` | P1 | M | S5 |
| FC7 | UI | Move plugin panels from `plugin_dock.rs` to the framework dock used by PC | P1 | M | framework |
| FC8 | App/Ad | Importer/exporter registry replacing the hard-coded match (`cmd/file/load.rs:105-155`) | P1 | M | S5 |
| FC9 | App | `vector.effect@1` for live effects on nodes (shared with VC) | P2 | M | VC3 |
| FC10 | App | Widget node kind with synced state | P3 | L | FC3 |

### 4.5 DesignCraft

| # | Layer | Change | P | E | Dep |
|---|---|---|---|---|---|
| DC1 | Ad | New adapter `designcraft.session@1`; plugin events run as `script.run`-style transactions (one undo step, rollback) | P0 | L | program 2.2 |
| DC2 | Ad | Bridge over 567 engine ids; reclassify `query` ids that write files (`file.exportIdml`) | P0 | S | S1 |
| DC3 | Ad | `story.get/set` with styled runs, `style.*`, `table.*`, `datamerge.*` | P0 | M | DC1 |
| DC4 | App | Preflight issue kinds as `String` with source; rule registry for `document.check@1`; worker runs plugin rules with budgets | P1 | M | S5 |
| DC5 | App | `datamerge.source@1` (records from a plugin), `story.import@1`, `pdf.post@1` after PDF export | P1 | M | S5 |
| DC6 | App | Per-item change sets from the journal for events | P1 | M | S4 |
| DC7 | App/Pl | Plugin fonts obey the fsType enforcement already in PDF, package and PPTX export | P1 | S | H10 |

### 4.6 LightCraft

| # | Layer | Change | P | E | Dep |
|---|---|---|---|---|---|
| LC1 | Ad | New adapter `lightcraft.session@1`; `catalog.query`, `photo.get`, `metadata.get/set(_many)`, `develop.get/set(_many)` with partial patches over the 125 typed controls, `photo.render` to buffer | P0 | L | program 2.2 |
| LC2 | App | `export.service@1`, `export.postprocess@1`, `publish.service@1` around `export/prepare.rs`; per-photo `{remote_id, url, published_rev}`; republish queue on develop changes | P0 | L | H2, H3 |
| LC3 | App | Namespaced searchable plugin metadata fields in catalogue and `RuleSet` (`metadata.provider@1`) | P1 | M | LC1 |
| LC4 | App | Profile registry for plugin DCP/LUT profiles over the mutable LUT registry | P1 | S | — |
| LC5 | App | Batch-aware sidecar writes: one XMP write per photo per plugin event, not per command (`engine/src/lib.rs:630-633`) | P1 | S | LC1 |
| LC6 | App | Expand docking beyond 4 panels so plugin panels sit in module side panels | P1 | M | framework |
| LC7 | App | Pin SAM 3 tokenizer hashes; encrypt or retire remote inference TCP before plugins can reach it | P1 | S | — |
| LC8 | Pl/App | `handoff.open` "Edit in PhotoCraft" replacing external-editor launch for suite apps | P2 | M | H13 |

## 5. Edge cases

**100 MP documents.** A 100 MP layer is 1.6 GB as premultiplied f32 RGBA, more than the 512 MiB job cap;
today's whole-canvas filter (`plugin_filters.rs`, `doc.bounds()`) fails outright. PC already stores
256-pixel tiles with LZ4 spill and clock eviction. Plugins must therefore see tiles through host handles,
with reservations against the app-wide budget *before* materialising a tile, and filters must declare
padding so the host can stream tile rows. Previews run at pyramid `level` on the visible ROI. Upscalers
(row 2) write output tiles directly into a new document. LC is unaffected for originals (develop is
parametric) but `photo.render` must cap size per call.

**CMYK, Lab, 16 and 32 bit.** PC converts every plugin buffer to sRGB (linear at 32 bit), so ProPhoto
16-bit documents lose gamut and CMYK/Lab documents are refused. Proposal: `space:"native"` delivers the
document's own channels with a profile handle; `space:"rgb"` uses an unclipped linear working space at f32;
plugins declare `spaces` and `depths`, and the host refuses rather than silently converting. Writes are
validated against range: 32-bit allows values above 1.0; 16-bit uses Photoshop's 0..32768 or full range
explicitly. Spot and Multichannel documents expose channel names.

**Non-destructive results with missing plugins.** PC smart filters with unknown ids leave pixels unchanged;
VC effects with missing plugins draw unmodified geometry; both silent. Under S6, the cached output is
stored in the document (PC: compressed tiles of the smart-filter result; VC: output geometry or painted
pieces; FC: same), drawn when the plugin is missing, version-mismatched or in Safe Mode, and flagged in the
layers/appearance panel. Parameters remain editable only when the plugin is present. Export and print use
the cached output and say so in the export report. Determinism: plugins that declare `deterministic=false`
(AI, network) store results only; re-render is an explicit user action.

**Collaborative FigCraft with plugins on some peers.** Plugin edits sync as ordinary property diffs, so
peers without the plugin see normal nodes. Risks: (a) the server does not check plugin-data ownership,
so a modified client can overwrite another plugin's data (FC3); (b) live effects or widgets would render
differently per peer unless their output is materialised (S6 cached output synced as a normal property);
(c) change-event handlers on several peers can fight over the same nodes under last-writer-wins; deferred
edits must be attributed and rate-limited, and a plugin running on two peers should elect one runner per
document (lowest session id) for automatic reactions; (d) undo in live sessions is collab undo, so one
plugin event must equal one collab undo unit (`engine/src/ext.rs:62`).

**Batch processing thousands of files.** PC droplets and batch cannot replay plugin session commands
(`journal:false`); LC writes an XMP sidecar per command; PC compiles each filter twice per run. Needs:
journaled parameters (PC8), a module and instance cache keyed by sha256, `job.map` fan-out, resumable batch
state with per-file results (publication is per file, not batch-atomic), LC5 batched sidecars, and catalogue
`Op::Batch` for metadata writes. Network plugins in batches need rate limits and backoff in H2.

**Colour-managed previews.** Plugin UI images and `canvas.preview.set` must be tagged (space plus profile)
and converted by the host to the display profile; LC already has monitor profiles, PC has craft-cms
display transforms, DC and FC display sRGB only. Proof mode must apply to plugin previews too. HDR: LC
reports SDR surfaces; plugin previews of HDR edits must be tone-mapped by the host, never by the plugin.

**Font licensing.** DC enforces fsType for embedding; `resource.font` packages must carry licence and
fsType, catalogue admission must check redistribution rights, and DC/VC exports apply the same rules to
plugin fonts. Session-activated fonts must be recorded in documents so peers and later opens can prompt;
FC collab peers lacking a plugin font fall back visibly. Fontself-class `font.install` writes to the user
font folder only with consent and never into the app bundle.

**Tablet latency for plugin brushes.** A per-dab `craft.call` costs about 10.5 µs plus JSON; a 240 Hz pen
with 20 dabs per sample would spend milliseconds per frame in transport. Plugin brush engines should be a
`kernel.program` or WGSL dab shader the host runs inside its own stroke loop, with Wasm only for
per-stroke setup. `input.filter@1` runs before built-in tools with a 1 ms budget per coalesced sample
batch; on overrun the host passes raw input through and flags the plugin. PC tablet backends deliver
tilt and rotation that plugins must receive (PC5). FC and DC have no pen data; FC needs tilt from the
egui fork before pen plugins make sense there.

## 6. Open decisions

1. **Bridge table location.** One `bridge.toml` per app repo (engine owners review) or in craft-extensions
   (platform owners review)? Recommendation: in the app repo, checked by a platform-provided test.
2. **Loopback network.** Allow `network:loopback:<port>` for local AI servers (ComfyUI, Krita AI
   Diffusion class) with explicit consent, or require native helpers? Loopback services can be anything.
3. **Inference backend.** candle (already in LC) versus burn (wgpu backend reuses our device) versus ONNX
   Runtime via native helper. LC's six-op interpreter is insufficient.
4. **Legacy VC shim lifetime** and whether old grants carry over (recommend: never; re-consent).
5. **VC JavaScript.** Keep `crates/js` as automation only, or offer JS as a plugin authoring language
   compiled to Wasm (QuickJS-in-Wasm) so ExtendScript authors can port? The suite rule forbids a second
   plugin system, so JS plugins would have to run inside the platform runtime.
6. **8bf hosting** through a native helper: big catalogue (Nik, Topaz, Filter Forge) but Photoshop SDK
   licensing and binary compatibility risk.
7. **Cached outputs in documents** increase file size (100 MP smart filters). Store at full resolution,
   at proxy resolution with re-render required, or user choice?
8. **Collab runner election** for automatic plugin reactions in FigCraft.
9. **Asset-source contract** (host-drawn browser) versus plugin panels with drag-drop only.
10. **FigCraft docking**: adopt the framework dock now (FC7) or wait for the framework migration.

## Sources

Local: paths cited inline; prior reports `CE/docs/plugins/research/capability-program-2026-11-18.md`,
`CE/docs/plugins/research/capability-2026-11-18/image.md`, `CE/docs/plugins/suite-replacement-rule.md`,
`CE/docs/plugins/capabilities.md`, `VC/docs/plugins.md`.

Web (accessed 2026-11-18): Photoshop Imaging API
https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/imaging; batchPlay
https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/batchplay; InDesign UXP network
https://developer.adobe.com/indesign/uxp/resources/recipes/network/; Illustrator UXP status (secondary)
https://mapsoft.com/posts/illustrator-uxp-status.html; Lightroom SDK mirror https://lrc.mcor.dev/ and
tagset provider https://lrc.mcor.dev/modules/SDK%20-%20Metadata%20tagset%20provider.html; Capture One
developer https://www.captureone.com/partnerships/developer; Affinity plugin scope (secondary)
https://gregbenzphotography.com/photoshop/can-i-migrate-my-plugins-from-photoshop-to-affinity and Magnific
Affinity docs https://www.magnific.com/ai/docs/affinity-plugin; GIMP
https://developer.gimp.org/resource/writing-a-plug-in/tutorial-gegl-ops; Krita
https://docs.krita.org/user_manual/python_scripting/krita_python_plugin_howto.html; Photopea
https://www.photopea.com/api/plugins and https://www.photopea.com/api/live; Inkscape
https://inkscape.gitlab.io/extensions/documentation/; Figma https://developers.figma.com/docs/plugins/api/figma/,
https://developers.figma.com/docs/plugins/codegen-plugins,
https://developers.figma.com/docs/plugins/api/properties/figma-createimageasync,
https://www.figma.com/blog/how-magician-uses-figmas-text-review-api/; Figma install figures (secondary)
https://hike.one/insights/the-top-50-figma-plug-ins-reviewed; Sketch
https://developer.sketch.com/plugins/javascript-api; Penpot https://help.penpot.app/plugins/getting-started/;
Canva https://www.canva.dev/docs/apps/api/design-add-element-at-point; QuarkXPress XTensions
https://en.wikipedia.com/wiki/XTension; InDesign plugin overview https://redokun.com/blog/indesign-plugins.
Vendor pages (product claims only): topazlabs.com, magnific.ai, remove.bg, nikcollection.dxo.com,
imagenomic.com, retouch4.me, borisfx.com/products/optics, filterforge.com, gmic.eu, infinitecolorpanel.com,
lumenzia.com, coolorus.com, lazynezumi.com, fontself.com, github.com/Acly/krita-ai-diffusion,
astutegraphics.com, hotdoor.com, avenza.com/mapublisher, esko.com, negativelabpro.com, lrtimelapse.com,
regex.info/blog/lightroom-goodies, johnrellis.com, excire.com, iconify.design, getstark.co, tokens.studio,
anima.app, locofy.ai, builder.io, lottiefiles.com, 65bit.com, emsoftware.com, typefi.com, enfocus.com,
woodwing.com, markzware.com.

**Not verified this pass:** Affinity by Canva's current 8bf support after the 2025 unification; current
Figma install rankings (no official sorted list was retrievable; rows 73–93 follow widely reported
popularity); Capture One SDK maintenance status (forum reports of stagnation); Typefi and Quark XDK
details. These are product-level facts and do not change the host-capability requirements above.

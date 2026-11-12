# Suite-wide plugins: independent functional study (Opus)

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Written on 2026-11-08 as the independent investigation listed in [the study decisions](plugin-functional-decisions.md). This is a design study. It does not authorize implementation. Facts come from reading the source at the revisions below; nothing was built or run. Each **Proposal** is a suggestion, not a decision. Background: [resource register](plugin-resources.md), [initial ideas](plugin-architecture-ideas.md).

During synthesis, the primary researcher corrected broad absence claims, checked ArtCraft's current native target, and qualified unmeasured audio/GPU assumptions below. The independent investigation was completed before those edits.

## Sources

| Repository | Revision | Read |
|---|---|---|
| [storytold/effectcraft](https://github.com/storytold/effectcraft/tree/ce9eaec65015a5508c72af0e2351cd02612c9450) | `ce9eaec` | `docs/plugins.md`, `docs/external-helpers.md`, CLI, host, render pipeline |
| [storytold/figcraft](https://github.com/storytold/figcraft/tree/0e86f3597a7443b15c93ec3ab2091202f8802c95) | `0e86f35` | `docs/plugins.md`, `crates/plugin`, `crates/engine/src/extensions` |
| [storytold/vectorcraft](https://github.com/storytold/vectorcraft/tree/065a646799f1f20f5454845d6648e09a5252c452) | `065a646` | `docs/plugins.md`, `crates/doc/src/plugin_data.rs` |
| [storytold/photocraft](https://github.com/storytold/photocraft/tree/655295b2a2c400bded0640f5d97367000a057b2e) | `655295b` | `docs/plugins.md`, `crates/plugins` |
| [storytold/soundcraft](https://github.com/storytold/soundcraft/tree/da02c19fcca8fbcc49d66c957e5efcc67a63befe) | `da02c19` | `docs/plugin-parity.md` |
| [storytold/gridcraft](https://github.com/storytold/gridcraft/tree/178f3eb745a589c63b61ef97dd7b6b2d6fd48515) | `178f3eb` | `docs/macros.md` |
| [storytold/designcraft](https://github.com/storytold/designcraft/tree/1256decbeb4a49c6fbe94400efc1e2a838de23ce), [storytold/wordcraft](https://github.com/storytold/wordcraft/tree/ffb155a0321bba08da495efda66495104c1c8b9a), [storytold/cadcraft](https://github.com/storytold/cadcraft/tree/b04273e8974245e1c59ec57eb5ad86cec7a200eb), [storytold/lightcraft](https://github.com/storytold/lightcraft/tree/ac1799db38661e07a855653f86feb3e66081102e), [storytold/texturecraft](https://github.com/storytold/texturecraft/tree/9dc33d894e9cd8e25ce84f6074fd1b476dad5698) | `1256dec`, `ffb155a`, `b04273e`, `ac1799d`, `9dc33d8` | Scripts, macros, workers, presets, automation entry points (module docs only) |

FilmCraft, PDFCraft, DeckCraft, SolidCraft and ArtCraft were only scanned for plugin hosts; that scan did not establish executable plugin support. SolidCraft does have manifest-listing preferences and a proposed runtime design. The primary researcher's subsequent check found a current pure-Rust `artcraft-native` target in ArtCraft's workspace, so this study does not classify that app from its older web stack. These apps need deeper integration discovery.

## 1. What "plugin" means today

At least four apps have their own wasmi-based WebAssembly host. Each defines a different contract:

| App | Contribution | Shape | Where it acts | Kept with the document |
|---|---|---|---|---|
| EffectCraft | Pixel effect ([ABI v1](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/docs/plugins.md#webassembly-abi-v1)) | No imports. Manifest inside the module. In-place RGBA `f32` buffer, flattened params and time | Render threads, one instance per thread | Effect group by id, with parameters as properties. No version |
| FigCraft | Command, panel, widget ([ABI v6](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/docs/plugins.md#abi-version-6)) | Imports `figcraft.call` and others. Manifest beside the module | Editor thread. One undo step per run or callback | Widget state and `pluginData` on layers. Ordinary command results are native edits rather than live plugin dependencies |
| VectorCraft | Object filter, live effect, exporter ([ABI v1](https://github.com/storytold/vectorcraft/blob/065a646799f1f20f5454845d6648e09a5252c452/docs/plugins.md)) | No imports. JSON objects in, JSON objects out | Filter: one undo step. Live effect: at draw time, cached. Exporter: on the export worker | `plugin.<id>` effect record and document-level [plugin data](https://github.com/storytold/vectorcraft/blob/065a646799f1f20f5454845d6648e09a5252c452/crates/doc/src/plugin_data.rs) |
| PhotoCraft | Pixel filter ([crate docs](https://github.com/storytold/photocraft/blob/655295b2a2c400bded0640f5d97367000a057b2e/crates/plugins/src/lib.rs)) | No imports. Bands in the document's own colour model | Destructive edit, or a smart filter re-run on render | Smart filter records `plugin.run {id, params}`. Skipped when the plugin is missing |

Other mechanisms exist outside these hosts:

- **Scripts that drive the command bus.** EffectCraft has After Effects-style JavaScript and ScriptUI panels. These are not WebAssembly-sandboxed; file and network access follow a setting ([docs](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/docs/plugins.md#scripts)). GridCraft has typed declarative [macros](https://github.com/storytold/gridcraft/blob/178f3eb745a589c63b61ef97dd7b6b2d6fd48515/docs/macros.md) with one undo step, a dry-run footprint check, and approval for file steps. DesignCraft has [command scripts](https://github.com/storytold/designcraft/blob/1256decbeb4a49c6fbe94400efc1e2a838de23ce/crates/engine/src/script.rs) with `$N.path` references, and WordCraft has [`run --script`](https://github.com/storytold/wordcraft/blob/ffb155a0321bba08da495efda66495104c1c8b9a/apps/wordcraft-cli/src/script.rs). WordCraft keeps VBA projects as opaque bytes ([test](https://github.com/storytold/wordcraft/blob/ffb155a0321bba08da495efda66495104c1c8b9a/crates/engine/tests/it/macro_packages.rs)).
- **Native code outside the sandbox.** SoundCraft hosts CLAP, VST3 and Audio Units [in-process](https://github.com/storytold/soundcraft/blob/da02c19fcca8fbcc49d66c957e5efcc67a63befe/docs/plugin-parity.md#third-party-plugin-hosting) on the real-time path. EffectCraft runs OpenFX through [external helper processes](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/docs/external-helpers.md). CadCraft decodes DWG in a [worker process](https://github.com/storytold/cadcraft/blob/b04273e8974245e1c59ec57eb5ad86cec7a200eb/crates/dwg/src/worker.rs), which is internal crash isolation, not an extension point.
- **Data-only content.** Examples are LightCraft's [preset import](https://github.com/storytold/lightcraft/blob/ac1799db38661e07a855653f86feb3e66081102e/crates/engine/src/preset_import.rs) and the brush and geometry presets in TextureCraft and DeckCraft. None of these is packaged or distributed as a plugin today.

The inspected editor apps commonly reuse their own command IDs across GUI, CLI, control-channel and MCP entry points. IDs differ between apps, and ArtCraft's native target needs separate investigation. **Reuse of each app's command dispatcher, rather than a universal command vocabulary, is a common integration pattern.**

## 2. Journey A: an EffectCraft effect

1. **Create.** The author writes a `cdylib` with no dependencies and builds for `wasm32-unknown-unknown`. The module exports `ec_api_version`, `ec_manifest_ptr`/`len`, `ec_alloc` and `ec_render` ([example](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/examples/plugins/posterize-bands/src/lib.rs)). The manifest, with its id, category and typed parameters, is compiled into the module. No SDK crate exists. FigCraft does have one (`plugins/sdk`).
2. **Install.** There are three routes: Effect ▸ Install, `effect.plugins.install {path|folder}`, and the reviewed [Plug-in Library](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/docs/plugins.md#plug-in-library), which checks a SHA-256 digest. A folder bundle installs each `.wasm` and each `.jsx` panel. Installation also loads the effect into the session, so it appears in the menus at once. Updating or removing a loaded effect needs a restart, because effects cannot be unregistered.
3. **Apply and animate.** `effect.apply {"effect": "<id>"}` adds it. Its parameters become ordinary properties, so keyframes, expressions and Effect Controls come from the host for free. The plugin itself never sees animation. It receives values already evaluated for the current time, plus the time.
4. **Render.** For each frame and thread, the host passes a padded, possibly downscaled buffer and a scale factor. A trap or an exhausted budget leaves the frame unchanged. Plugin effects are registered with [`gpu: false`](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/effects/src/plugin.rs#L485), so they always run on the CPU, even inside GPU compositions.
5. **Save and reopen.** The project stores `GroupKind::Effect { effect: id }` with its property values. It does not store the package version or digest. At launch the desktop app loads `Plug-ins` and then the Library, *before* it opens a project ([main.rs](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/apps/effectcraft/src/main.rs#L273-L281)). If an effect is missing, the render pipeline [filters it out](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/render/src/pipeline.rs#L627-L628). Its data is kept, and `file.findMissing {what: "effects"}` or `library.missing` report it only when someone asks.
6. **Headless export: a functional gap.** The CLI's [`session()`](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/apps/effectcraft-cli/src/main.rs#L470-L489) and the shared [`host::session()`](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/host/src/lib.rs#L88-L104) set `plugins_dir`, but neither loads it. On this reading, `effectcraft-cli render P.ecproj --out x.mov` silently renders without an installed plugin effect, and the MCP server behaves the same. This comes from reading the source and has not been run. `exec effect.plugins.load` exists, but `render` is a separate subcommand, so a one-shot render cannot load a plugin first.

## 3. Journey B: a FigCraft command and panel

1. **Create.** The author writes `figcraft-plugin.json`, which lists commands with typed parameters, `permissions.access` (read or write), an optional command allowlist, and an optional `assets/` folder of up to 32 files. The module exports `figcraft_run` and imports host calls such as `call`, `selection`, `ui_show`, `setting_*` and `save_file`.
2. **Install.** Importing from a manifest is meant for development: the module is re-read on every run, so a rebuild takes effect immediately. A team's `figcraft-server` directory installs after confirmation and a SHA-256 check. Installed plugins live in the preferences and in a `Plugins/` folder.
3. **Run.** The Plugins menu opens a form generated from the parameters, then calls `plugin.run`. The input carries the selection. The plugin edits only through `call(command, params)`, and each call is checked against its permissions. The whole run is wrapped in [`grouped(label, …)`](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/crates/engine/src/extensions/plugins.rs#L501-L553), so it is one undo step named after the plugin. If it fails, the document, selection and page are restored.
4. **Panel.** `ui_show` returns a declarative spec, and the host draws it. The instance stays alive while the panel is open. Each `plugin.event` callback is its own undo step. A plugin cannot call `edit.undo`, `plugin.*` or `file.*` ([refused list](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/docs/plugins.md#what-a-plugin-cant-do)).
5. **Headless.** `figcraft-cli run script.jsonl --in a --out b` starts [with no plugins](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/docs/plugins.md#using-plugins), so the script must call `plugin.import` first. `plugin.event` and `plugin.close` drive panels. A file offered through `save_file` is [dropped for programmatic callers](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/crates/engine/src/extensions/plugins.rs#L331), so a headless run of an exporter plugin produces nothing. Directory installs last only for the session.

FigCraft differs from EffectCraft in useful ways. It is session-oriented rather than frame-oriented, it acts through imports rather than a pure function, the document generally does not depend on the plugin, and it already works in the browser build. EffectCraft's WebAssembly effects do not run in its web build.

## 4. Differences the shared design must keep

- **Live versus destructive.** EffectCraft effects, VectorCraft live effects and PhotoCraft smart filters re-run during rendering, so the document depends on the plugin. Filters and FigCraft commands change the document once, and the result no longer needs the plugin.
- **State.** EffectCraft offers `instance_key` for native API 2 only. FigCraft keeps a live instance per panel, and widget state lives in the document. VectorCraft and PhotoCraft use a fresh instance on every call.
- **Concurrency and jobs.** EffectCraft runs one instance per render thread. PhotoCraft runs parallel bands under a 2 GiB total. VectorCraft runs [heavy filters](https://github.com/storytold/vectorcraft/blob/065a646799f1f20f5454845d6648e09a5252c452/docs/plugins.md#heavy-filters) in the background, and their result applies as one undo step only if the document has not changed meanwhile. Library and directory downloads run asynchronously, with `wait: true` for scripts.
- **Output.** VectorCraft exporters are document formats: the host writes the bytes, and `document.export {format: "plugin.<id>"}` works headless. FigCraft's `save_file` depends on a dialog.
- **Real-time audio.** SoundCraft plugins run on the audio thread under latency compensation, with editor windows and state saved in the session. Real-time suitability of a shared sandboxed backend has not been established by this source review and needs separate latency and scheduling evidence.
- **Limits differ for the same purpose.** Linear memory is 1 GiB in EffectCraft, 64 MB in FigCraft and 512 MiB in VectorCraft. Only VectorCraft and PhotoCraft enforce a wall-clock limit.

## 5. Proposal: shared contracts and app adapters

**Shared in craft-libs:**

- **Package and identity.** One package id across the suite. The package holds one or more *contributions*, and each names `{app, kind, kind_api}`. A package can carry an EffectCraft effect and a FigCraft command side by side, but nothing claims that one module runs in every app.
- **Store, install and catalog.** One per-user store, with versioned directories and digest checks. It handles official and third-party sources, and grants are kept per app. This generalizes the EffectCraft Library and FigCraft's directory.
- **Runtime host library.** One wasmi wrapper covering fuel slicing, the wall clock, memory limits, a process-wide memory budget, the portable dispatcher, and checks on host-call bounds. The four separate copies would converge on it. ABIs stay with each app; only the sandbox mechanics are shared.
- **Plugin reference record.** `{package, contribution, version_at_save, state_schema}` embedded in each app's own file format, plus a shared check that lists missing or incompatible references.
- **Plugin environment resolver.** A single function that GUI, CLI and MCP all call. It decides which plugins load (installed, none, or an explicit folder) and in what order relative to opening a document.
- **Declarative UI vocabulary.** FigCraft's parameter and panel controls, with EffectCraft's and VectorCraft's parameter schemas mapped onto them. Each app renders the controls with its own widgets.

**App-owned, through an adapter:** contribution ABIs and buffer formats, command ids and permission classes, how references are stored in the document, missing-plugin behaviour, menu and panel placement, and how render or export scheduling calls into the runtime.

**Integration duties for each app adapter:** register contributions and declare unsupported kinds explicitly; call the environment resolver from every entry point; check references before rendering or exporting; wrap a session callback in one undo step and roll it back on failure; provide a headless equivalent of every GUI interaction (FigCraft's `plugin.event` and `widget.click` are the pattern); report plugin errors in command results, not only in toasts.

**What an author ships:** a package manifest (id, version, publisher, contributions, permissions, assets with digests), one or more modules mapped to contributions, optional assets, and a parameter schema. Today EffectCraft, VectorCraft and PhotoCraft embed metadata in the module. The package manifest should sit outside the module so a catalog can read it without running code, and the embedded copy is then checked against it. Large assets such as LUTs and models are declared resources that the host downloads and verifies. A sandboxed module reads them through a host call limited to its own package. Dependencies between plugins are left out at first.

**Lifecycle events, minimum set:** install, load, activate for an app, and unload at restart (EffectCraft's registry cannot unload). For session actors the events are run, callback and close. For processors there is only a stateless call, plus an optional cache key. Startup scripts stay app-owned.

## 6. Resources and GPU within the sandbox

The four inspected WASM contracts do not expose a direct plugin GPU interface. This does not establish whether native plugins use GPUs internally. A proposed progression is CPU WASM where supported, host-provided kernels requested through typed operations, then plugin shaders only after a separate decision. CPU fallback is optional and must be declared, rather than assumed for every workload. A GPU path with an equivalent CPU implementation needs defined numerical tolerances. Cache only contributions whose declared inputs and state make caching valid. Stateful temporal or audio processors require their own evaluation and reset rules. Native helpers remain outside the first sandboxed scope until an appropriate execution backend is designed.

## 7. Key functional decisions to make

1. **Package across apps, contract per app.** The shared unit is the package and its contributions. No universal ABI, command vocabulary or media format.
2. **Two initial execution shapes.** *Processors* are host-scheduled and off the editor thread where their contract allows it. *Session actors* request editor commands, with one undo transaction per run or callback. Purity, caching, persistent state, and threading are contract properties, not assumptions about all plugins. Audio, tools, and temporal processing need additional lifecycle rules.
3. **Headless parity is required.** Every entry point uses the same environment resolver. A document with unresolved live references fails or warns before rendering, never silently.
4. **Outputs go through host formats.** A plugin never receives a path. Exporters become formats, so `--out` works. Any offered file must map to a declared headless destination.
5. **Documents record a reference, not code.** Record version and state schema, preserve data when the plugin is missing, and choose explicitly between failing, skipping and placeholder output for each app.
6. **Native and real-time hosts stay out of the first scope.** SoundCraft's audio plugins and EffectCraft's helpers stay app-owned until a separate trust class is designed.

## 8. Gaps

- The EffectCraft headless omission comes from source inspection only and needs to be reproduced at runtime.
- This pass did not establish executable plugin hosts in FilmCraft, PDFCraft, DeckCraft, SolidCraft, CADCraft, TextureCraft or LightCraft. It did not prove their absence. What contributions they would want (codecs, transitions, CAD constraints, materials, presets) needs further study.
- The ArtCraft web stack and AI backends were not studied, and may not fit this model at all.
- Whether EffectCraft's JavaScript, GridCraft macros and the command scripts become package contribution kinds is undecided.
- Cross-app contributions, where one package acts in several apps, have no observed demand yet.
- Migration of the existing ids, catalogs and the four hosts' differing limits is not planned here.
- No performance or isolation properties are claimed.

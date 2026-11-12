# Plugin architecture merge-hold list

Prepared 2026-11-09 for maintainers; updated 2026-11-10 to the settled [architecture](architecture.md). It answers the question "which plugin-related changes should not be merged while the basic plugin/extension architecture is being finished?"

## Summary

We ask maintainers to hold merges of changes that create or reshape **plugin and extension infrastructure**: plugin loaders and runtimes, ABIs and manifests, plugin registries and install/replace commands, the extension store and catalogue, theme packaging, new plugin SDK or host crates, and new ways for outside callers to run app commands. Each of these would collide with an accepted decision. Examples: one shared store in craft-extensions, grants per package per app, all plugin-platform crates in craft-extensions, one operation path with plugin, management and core markings, and the replacement of each app's current plugin system by the new adapter. A merge now would also add a seventh install mechanism, which the design says we must not do. Ordinary app work is **not** held: features, bug fixes, file formats, translations, UI polish, themes as app styling, and SoundCraft's CLAP/VST3/AU host work all continue. Security and crash fixes always merge, coordinated with us. The hold is temporary. It lifts area by area as the basic architecture pull requests land (see [When the hold lifts](#when-the-hold-lifts)). It is not a permanent freeze.

Three levels are used below:

- **Hold**: please do not merge until our pull request for that area lands, or we agree otherwise. Open pull requests stay open. Nothing is closed.
- **Review with us**: may merge, but please ping the plugin architecture owners first so we can check it against the design.
- **Not affected**: merge as normal.

## Why these areas

Each entry names the decision or planned change it would collide with. These are the references:

| Ref | Source | What it says |
|---|---|---|
| D-store | [Decisions](draft/decisions.md): Common manager, Store, Enablement | One management system and one transactional store, reusing craft-extensions. Install does not enable. Updates preserve choices |
| D-grant | Decisions, T01 | Grants attach to a package within one app. Each contribution is enabled separately |
| D-contract | Decisions, T02 | Three contract layers. Existing ABIs stay as legacy app profiles. Shared domain contracts (raster, temporal, audio, resources) are designed centrally |
| D-sdk | Decisions, T03 | Every plugin-platform crate lives in **craft-extensions**: core, store, contracts, generator, guest SDK, host support, conformance, and the author CLI `craft-ext`. Contributions are declared in Rust code; one `Craft.toml` per package |
| P-dispatch | [T04 proposal](design-sessions/app-families-and-sdk.md#t04-session-proposal-one-operation-path-for-gui-cli-automation-and-plugins), **proposed, not yet accepted** | Each app keeps its existing command registry. One dispatcher takes a mandatory `Invocation` (caller, scope, cancel, transaction). Plugin exposure is opt-in per operation |
| W1–W12 | [Roadmap](readiness/system-gaps-and-roadmap.md) | W1/W2 store schema 2 and eligibility; W3 truthful output; W4 safe replacement and registration; W6/W7 authenticated catalogue; W8 SDK; W11 PhotoCraft first-party proof |
| B8 | Roadmap | Six install mechanisms already exist. Do not add a seventh |

[The assessment's root corrections](readiness/assessment-and-first-work.md#root-corrections-to-the-coordinator-findings) apply throughout this list. In particular: several apps already have real caller controls; PhotoCraft's registry can remove plugins; VectorCraft resolves built-in formats first; GridCraft's test helper has a real integration-test consumer. Entries below do not treat these as defects.

## Hold list by repository

Paths and line numbers are at the audited committed revisions in the [evidence index](readiness/evidence-index.md). Each path was confirmed to exist at those revisions on 2026-11-09.

### craft-extensions

The whole repository is plugin infrastructure. It is the agreed home of the store and the SDK (D-store, D-sdk).

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | `crates/ext-core/` (`manifest.rs`, `id.rs`, `tier.rs`, `index.rs`, `palette.rs`, `version.rs`) | Package manifest, ID rules, trust tiers, catalogue index format and validator | One validator is shared by SDK, catalogue and installer (D-sdk). Manifest schema 1, dotted IDs and the unsigned index are all being replaced (W1, W6, UUID decision) |
| Hold | `crates/ext-store/` (`store.rs`, `state.rs`, `reader.rs`, `unpack.rs`, `location.rs`) | Installed-package store, state file, per-app reader | W1 changes the state schema: install stops enabling every app (`store.rs` 215–219), old releases are retained (229–231), grants are added. W2 replaces `Reader::installed_for` |
| Hold | `docs/THREAT-MODEL.md` | Draft threat model | Being reconciled with the security design |
| Hold | Branch `wip/extensions-handover` (`ffd8512e`) and `chore/repo-url-storytold` | Jeremy's unmerged store work and URL fix | Preserved as the base we extend. Please do not merge or rebase it separately |
| Hold | Any new crate here (contracts, guest SDK, host, conformance, `craft-ext` CLI) | Planned SDK crates | These are our pull requests (D-sdk) |

No open pull requests were found in this repository.

### craft-libs

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | Any new crate for plugins or extensions, for example `craft-plugin-*`, `craft-ext-*`, plugin ABI, host, SDK, guest macros, plugin UI, or plugin developer CLI | Plugin platform code | D-sdk places all of it in craft-extensions. A parallel ABI, manifest or permission vocabulary here would be a second source of truth (D-contract, B8). See the open pull requests below |
| Hold | `crates/keymap/src/keymap.rs`: `Policy` (default `Steal`, 333–341), `Keymap::new` first-wins (497–534), `ConflictKind` (436–441) | Shared shortcut conflict policy | W4 needs contributed bindings to use `Refuse` and never steal built-in chords. A change to the default or to conflict semantics changes what every app does with contributed shortcuts. Adding bindings in an app is not affected |
| Hold | `crates/menu/` (adding a registration or runtime-insertion API) | Shared menu model, currently built by apps with no registration API | Plugin menu placement is undecided; contributions must not take built-in placement (W4). Ordinary menu changes are not affected |
| Review with us | `crates/ui/src/theme/` (theme file format, palette loading) | Shared theme drawing and `ThemeFile`/`WidgetPalette` | The themes decision keeps data-only themes, explicit activation and app-aware mappings, and keeps `ThemeFile` and `WidgetPalette` separate. Styling changes are fine; changes to how theme files are found, loaded or activated need a check |
| Review with us | New shared crates for theme storage or selection (for example a suite theme store) | Machine-wide theme library | A second theme store would overlap craft-ext-store's theme reader and activation (D-store, B8). This needs a short conversation, not necessarily a hold |
| Review with us | New shared command-registry or IPC crates | Shared command dispatch or app-to-app channels | Would be a new entry point to app operations (P-dispatch). Caller identity and scope must be designed in |
| Not affected | `crates/update/` (app updater), fonts, i18n, print, PDF, docking and other UI work | Unrelated to plugins | — |

### craft-launcher

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | Any Extensions, Plug-ins, Themes or Resources page, browser or local plug-in manager | Launcher extension UI | d0 (Jeremy) puts the shared browser in the launcher. The approved UI is on unmerged branches `feat/extensions` (`6ede77f9`: `apps/artcraft-launcher/src/extensions.rs`, `crates/ui-egui/src/features/extensions/`) and `test/ext-plus-ai` (`be3bff3e`, reference only, never a release source). A different manager would conflict with D-store |
| Hold | Deep links such as `--open extensions?kind=…` and single-instance forwarding for extensions (`crates/native/src/deeplink.rs`, `single_instance.rs` on `feat/extensions`) | Opening the browser from an app | Part of the same branch. Not on `main` |
| Hold | Anything that installs packages or plugins through the launcher | Install path | Must go through craft-ext-store (D-store). The branch installs only as `Tier::FromFile` (`extensions.rs:79`) |
| Review with us | `crates/native/src/install.rs` (621–659), `http.rs` (227–260), `crates/core/src/release.rs` (245), `crates/core/src/feed.rs`, `crates/native/src/backend.rs` (680–830), `feed_throttle.rs` | GitHub download, resume, digest check, feed back-off | We plan to reuse these mechanics for catalogue downloads (W6/W7). For plugins, only the catalogue digest is authoritative, not the publisher's `SHA256SUMS.txt` or GitHub `digest`. App-update fixes are fine; we only need to know about API changes |
| Not affected | App catalogue, app installs and updates, translations, AI features | — | — |

### EffectCraft (`ce9eaec`)

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | `crates/plugin/` (`wasm.rs`, `lib.rs`): WASM ABI v1, `ec_*` exports, fuel and memory limits | Effect plugin runtime and ABI | Kept as the legacy app profile (D-contract). An ABI bump (for example a new export) adds a contract we must describe and carry |
| Hold | `crates/effects/src/plugin.rs`: `PluginManifest` (168–183), `register_plugin` (467), slots (393, 474–478), duplicate refusal (471–472), failure fallback (680–690) | Effect plugin manifest and registration | Registration, manifest fields and unload behaviour are W2/W4 work |
| Hold | `crates/engine/src/system/library.rs` | Plug-in Library: reviewed `index.json`, GitHub download, SHA-256, installed records | Its fields and behaviour are being merged into the shared catalogue format (W6/W7). New library features, sources or install paths would deepen a separate install mechanism (B8). Security fixes to download checks are exceptions |
| Hold | `crates/engine/src/cmd/effect.rs`: `effect.plugins.load`, `effect.plugins.install`, `effect.helpers.scan`/`load` | Plugin install and helper start commands | Install and helper start need caller rules before new uses (W2, P-dispatch) |
| Hold | `crates/helper-host/`, `crates/host/src/lib.rs` (plugin loader wiring, 97–100, 115–118) | Native helper processes and loader wiring | Native execution needs backend-specific proof (decisions: native backend) |
| Hold | `crates/project/src/props.rs` `GroupKind::Effect` (183–186), `crates/engine/src/session/edit.rs` `upgrade_effects` (277–317) | How projects save effect references | Exact release references and migration (project experience decision). Changing the saved format now creates a migration we must carry |
| Hold | `docs/plugins.md` | Plugin documentation | Will be rewritten against the SDK |
| Review with us | `crates/render/src/pipeline.rs` (622–632), `crates/engine/src/render_queue.rs` (471), `apps/effectcraft-cli/src/main.rs` (801–811, 919–951) | Render and export path; missing effects are dropped | W3 truthful output. Fixes here are welcome; we want to agree the status shape |
| Review with us | `crates/engine/src/session/dispatch.rs` `Session::execute` (9), `crates/automation/src/confine.rs`, `crates/automation/src/tools.rs` | Dispatch, MCP confinement | Hold if a change adds a new entry point; otherwise review (P-dispatch) |
| Exception (merge, tell us) | `apps/effectcraft/src/control_server.rs` | Unauthenticated loopback control channel | Adding authentication is a security fix |

### PhotoCraft (`655295b`)

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | `crates/plugins/` (`runtime.rs`, `registry.rs`, `manifest.rs`, `apply.rs`) | WASM filter runtime (ABI 1, `pc_*`), process-wide registry, manifest | Legacy app profile and the W11 pilot. A new ABI version (for example panel plug-ins, native plug-ins) adds contracts outside the T02 model |
| Hold | `crates/engine/src/cmd/plugin_cmds.rs`: `plugin.install` (`replace` defaults to `true`, 200–208), `plugin.run`, `plugin.remove`, `plugin.reload`, smart-filter record (133–141, 150–159) | Install/replace commands and saved plugin references | W4 changes the replace default; saved references need versions (project decision) |
| Hold | `crates/engine/src/prefs.rs` (889–896) `useAdditionalPluginsFolder`, folder loading | Folder-based plugin loading | Becomes a developer option once the store is the install path (Jeremy pl-tasks, D-store) |
| Hold | `crates/ui-egui/src/menus/plugin_ui.rs` | Filter ▸ Plug-ins menu | Menu placement and enablement follow T01 |
| Hold | `examples/plugins/invert-rs`, `crates/plugins/tests/fixtures/invert.wasm` | Pilot plugin | The W11 first-party proof uses it |
| Hold | Branch `feat/extensions-theme-adapter` (`ecb056cc`: `crates/ui-egui/src/ext_theme.rs`) | Jeremy's theme adapter, unmerged | Must update with store schema 2 (W1). Merge it only with us |
| Review with us | `crates/automation/src/workspace.rs` (plugin gates 446–486), `crates/automation/src/headless.rs` (`Filesystem` modes, 30–60) | Caller authority for automation | Existing real controls we build on. Tightening is welcome; loosening or new entry points need a check |
| Exception | `crates/automation/src/security.rs` | Token-authenticated transport | Security fixes merge |
| Not affected | Filters, adjustments, formats, UI themes as app styling | — | — |

### FigCraft (`0e86f35`)

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | `crates/plugin/` (`sandbox.rs` ABI 4 with 14 host imports, `manifest.rs` API 6) | Plugin sandbox and manifest | Legacy app profile; permissions `commands` allowlist accepts `"*"` (`manifest.rs` 104–109, 354–358), hardening is W4 |
| Hold | `crates/engine/src/extensions/` (`plugins.rs`, `plugin_directory.rs`, `widgets.rs`) | Plugin host, policy, directory install, widgets | Reinstall replaces and deletes settings (`plugins.rs` 460–478); directory install trusts a caller `confirm: true`. W4 and D-grant |
| Hold | `apps/figcraft-server/src/directory.rs` | Plugin directory server publish | A second catalogue (W6/W7, B8) |
| Hold | `crates/doc/src/node.rs` `plugin_data` (543–545) | Plugin data saved in documents | Saved plugin references (project decision) |
| Review with us | `crates/ui-egui/src/control.rs` (1422–1432) `engine.execute` → `execute_as`; `crates/engine/src/session/exec.rs` | Control channel reaches commands without the plugin denylist | P-dispatch. Adding authentication to `apps/figcraft/src/control_server.rs` is a security exception |

### VectorCraft (`065a646`)

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | `crates/plugins/` (`runtime.rs`, `registry.rs`, `effect.rs`, `export.rs`, `manifest.rs`) | WASM filters, live effects and exporters (ABI 1, no imports) | Legacy app profile. New kinds (tools, panels) add contracts outside T02 |
| Hold | `crates/engine/src/cmd/plugin.rs` (`replace` default 292, `allowHeavy` 296–300, "for this session" 287–307, 444) | Install, replace and heavy-budget consent | W4 and D-grant; consent is a caller-supplied boolean today |
| Hold | `crates/engine/src/store.rs` `attach_plugin_store` (221–254), `apps/vectorcraft-cli/src/main.rs` `with_stores` (205–213), `apps/vectorcraft/src/prefs_dir.rs` | Plug-ins folder store and CLI loading | Fresh CLI eligibility must come from the shared store (CLI decision, W2) |
| Review with us | `crates/engine/src/cmd/fileio/formats.rs` (703–740), `crates/engine/src/cmd/params.rs` (254) | Built-in-first format resolution, `plugin.` prefix routing | Keep built-in precedence; exporter selection by unique ID is the plan |
| Review with us | `crates/mcp/src/headless.rs` (342–349) | MCP reaches every command including `plugin.install` | P-dispatch |

### FilmCraft (`32749e3`)

No plugin runtime today.

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | Any pull request adding a plugin runtime, SDK, plugin UI or plug-in host | New plugin system | Would be an app-specific runtime and SDK outside D-sdk and D-contract |
| Hold | `crates/engine/src/settings/prefs.rs` `PluginPrefs` (427–428), `settings/schema.rs` (534–538) | Plugin settings placeholder | Will follow the shared enablement model |
| Review with us | `crates/export/src/registry.rs` (94–99, 156) | Encoder factories; a later one pre-empts built-ins | A plugin export seam needs a precedence rule first. In-process encoder work is fine |
| Review with us | `crates/engine/src/export_tools.rs`, `crates/automation/src/long_job.rs`, `apps/filmcraft-cli/src/main.rs` (197–208) | Export jobs and CLI wait | W3 truthful output |

### LightCraft (`ac1799d`)

No code-plugin mechanism.

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | Any new code-plugin runtime or resource-pack installer | New extension path | Resources are typed packages in the shared store (resources decision) |
| Review with us | Resource importers: `preset.import` (`cmd/preset_files.rs:117`), `profile.import` (`cmd/lut_profiles.rs:143`), `curve.importPresets`, `mask.importEffects` | Data imports | Likely first resource-pack seam. Ordinary importer fixes are fine |
| Exception | `apps/lightcraft/src/control_server.rs` | Unauthenticated control channel | Security fix |

### ArtCraft (`1f969a9`)

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | Making `Provider` (`artcraft-native/src/backend/providers/mod.rs` 22–27) pluggable or loadable | Closed provider enum | A provider adapter would be new plugin design work |
| Not affected | Everything else, including the Tauri app | — | — |

### PDFCraft, DesignCraft, DeckCraft (`14ae839`, `1256dec`, `0c00f22`)

No plugin runtime. Commands and export formats are fixed in code.

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | Any plugin host, ABI, extensions registry, or runtime command/format registration | New plugin system | D-sdk, D-contract, W4 |
| Hold | Theme packages with install or CLI management (rather than app styling) | Theme distribution | Themes decision, D-store |
| Review with us | PDFCraft `crates/automation/src/tools.rs`, `file_access.rs`; DesignCraft and DeckCraft `crates/engine/src/cmd.rs` lookup (`find_command`) | Automation table and command lookup | P-dispatch, duplicate-ID handling |
| Not affected | Embedded Acrobat JavaScript in PDFCraft `crates/js` (document scripting, not extensions), formats, editing | — | Sandbox fixes there are normal security work |

### SoundCraft (`da02c19`)

Third-party CLAP, VST3 and AU hosting is a separate, existing system for industry plug-ins. It is **not held**.

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | A Craft package loader or `wasm:` insert effect | New plugin system | D-contract `audio` domain contract |
| Review with us | `crates/model/src/mixer.rs` `Insert.plugin` (425–440) | Saved plugin reference (string ID, opaque state) | Saved references (project decision) |
| Review with us | Changes that move hosted plug-ins into a shared install location or store | Install path | B8 |
| Exception | `apps/soundcraft/src/control_server.rs` | Unauthenticated control channel | Security fix |
| Not affected | `crates/clap-host/`, VST3/AU hosts, scanning, editors, parameters, blocklists | — | — |

### CADCraft (`b04273e`)

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | Installable AutoLISP packages, extension systems, .NET or other plugin hosts | New extension systems | AutoLISP has no confinement today (`crates/engine/src/lisp/builtins/files.rs` 76–108). Packages must not run there before a capability gate (W4, D-grant) |
| Review with us | Startup `acad.lsp`/`acaddoc.lsp` loading, `crates/engine/src/session/cmdline.rs` (54, 189), MCP `command_line`/`script` (`crates/mcp/src/tools.rs` 11–13) | Paths that evaluate LISP | P-dispatch |
| Exception | LISP file I/O confinement | Security fix | Merge, tell us |

### SolidCraft (`4bdffef`)

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | `docs/plugins.md`, `crates/ui-egui/src/plugins.rs`, any WASM runtime or `FeatureKind::Plugin` in `crates/doc/src/document.rs` | Plugin design and manifest listing | Must follow D-contract and the saved-reference design |
| Review with us | `crates/engine/src/render_jobs.rs`, render-executor registration | Render jobs | Possible contribution seam |

### TextureCraft, WordCraft, GridCraft (`9dc33d8`, `ffb155a`, `178f3eb`)

| Level | Path or symbol | What it is | Why it matters |
|---|---|---|---|
| Hold | Any plugin runtime, function-package or resource-pack installer | New extension systems | D-sdk, D-store |
| Review with us | TextureCraft `crates/automation/src/headless.rs` (14–24), `workspace.rs` (136–150) | Caller file-authority modes | Existing controls we build on |
| Review with us | WordCraft `Registry::new` (`crates/engine/src/lib.rs` 145–153, last-wins on duplicate IDs); customisable ribbon | Command registry | W4 wants fail-closed registration. A fix is welcome; we want to agree the shape |
| Review with us | GridCraft `register_test_command` (`crates/engine/src/cmd.rs` 202–205), `crates/engine/src/macros/policy.rs`, `crates/functions/src/lib.rs` registration | Runtime command registrar, macro policy, function table | Do **not** gate the helper behind plain `cfg(test)`: an integration test in `crates/ui-egui/tests/keytips.rs` uses it. No hostile path was shown |
| Review with us | GridCraft sandboxed helper process work (Python cells) | Process sandbox | Not plugins, but its confinement design may be reused |

## Across all apps

| Level | What to look for | Why |
|---|---|---|
| Hold | A new outside entry point to app commands: web `window.*` APIs, new MCP tools that run any command, new control-channel methods, app-to-app links that run commands | P-dispatch (proposed): every entry point should carry a caller. Adding entry points now adds migration work |
| Hold | New commands named `plugin.*`, `extension.*`, `library.install`, or anything that installs, enables, grants or removes code | D-grant: management operations are never plugin-exposed and need explicit user choice |
| Review with us | Changes to the command spec structure: `cmd!`/`query!`/`CommandSpec`/`spec!` fields, lookup or duplicate handling, `execute(id, params)` signatures | P-dispatch extends these registries rather than replacing them |
| Review with us | New commands that write files, use the network or start processes | Need a `requires` entry under P-dispatch |
| Not affected | New ordinary commands added to an existing table with the existing macros | — |

## Exceptions

**Security fixes and crash fixes always merge.** Please tell us when they touch a listed path, so we can rebase. This includes adding authentication to unauthenticated control channels (EffectCraft, LightCraft, SoundCraft, FigCraft), tightening plugin sandbox limits, download or path checks, and fixing panics in plugin parsers. If a security fix also changes a plugin contract (for example a new ABI export), please ask first; we will turn it around quickly.

## How to recognise a plugin pull request

A pull request probably needs a hold or a review if any of these is true:

- [ ] Title or branch mentions plugin, plug-in, extension(s) (meaning add-ons, not file extensions), add-on, SDK, ABI, manifest, sandbox host, catalogue/library/directory/store (for packages), theme package, or Craft Link.
- [ ] It adds a crate named `plugin*`, `ext*`, `*-abi`, `*-sdk`, `*-host`, or a developer CLI for plugins.
- [ ] It touches `crates/plugin/`, `crates/plugins/`, `crates/effects/src/plugin.rs`, `extensions/`, `helper-host`, `system/library.rs`, `plugin_cmds.rs`, `cmd/plugin.rs`, `store.rs` (plugin store), or anything in craft-extensions.
- [ ] It changes `craft-keymap` `Policy` or adds menu registration to `craft-menu`.
- [ ] It adds a `.wasm` loader (`wasmi`, `wasmtime`), `libloading`/`dlopen`, or a new process launcher for third-party code.
- [ ] It changes how a project file stores a plugin, effect or filter reference.
- [ ] It adds a new way for an outside program or web page to run app commands.
- [ ] It installs, enables or removes code, or adds a permission or consent prompt.

Not plugin work, despite the words: file-name extensions, OOXML `extLst`, CAD extension dictionaries, OmaStore/Flatpak/winget manifests, font manifests, and GStreamer plugins in packaging.

## Open pull requests found

Read-only search on 2026-11-09 with `gh search prs --owner storytold --state open` for plugin, plug-in, plug-ins, plugins, extension(s), craft-ext, manifest, sandbox, wasm, ABI, theme and keymap (up to 100 results per term). Classification is from titles and the first part of descriptions only; we have not reviewed the code. This list is not exhaustive.

**Hold**

| Repository | Pull request | Why |
|---|---|---|
| craft-libs | #432 craft-plugin-abi, #454 craft-plugin-host engines, #455 craft-plugin-sdk, #613 craft-plugin-host host layer, #615 craft-plugin-ui, #616 craft-cli for plugins (all part of RFC issue #414) | A complete parallel plugin platform in craft-libs: ABI, manifest, permissions, host, SDK and CLI. Directly overlaps D-sdk (craft-extensions placement), D-contract and D-grant. This is the most important item on the list; it needs a conversation with its author, not just a hold |
| craft-libs | #431 craft-command, #433 craft-ipc (base of the same stack) | Shared command registry and local IPC. P-dispatch; would be a new entry point |
| effectcraft | #3138 Craft Link and the plug-in host; filmcraft #4351 Edit in EffectCraft with a plug-in host | Depend on the craft-libs stack above |
| craft-launcher | #225 local plug-in management page and Photoshop filter import | Competing manager (D-store, d0) |
| photocraft | #6479 native plugin support and SDK; #7907 Photoshop plugin support; #7995 host-neutral adapters; #4301 ABI v2 panel plug-ins; #6733 panel plug-ins running engine commands; #7915, #7916 plug-in controls and suites; #4094 folder hot-reload | ABI, runtime and install-path changes |
| vectorcraft | #5654 installable WASM tool plugins; #5287 panel plug-ins; #3807 live-effect overlays | New contribution kinds and ABI surface |
| figcraft | #2019 MCP harness, built-in plugin store; #1737 plugin directories on a server | Second store/catalogue and new entry point |
| filmcraft | #1572 WASM plugin system with SDK | New runtime and SDK |
| pdfcraft | #2291 sandboxed WASM plug-in host (ABI v1) | New runtime |
| cadcraft | #1446 installable AutoLISP packages; #1384 extension system reference; #2009 includes a .NET plugin host | New extension systems on an unconfined interpreter |
| craft-contrib | #12 RFC-0001 command catalogue in the web build | New outside entry point (P-dispatch) |
| effectcraft | #3265 `ec_pad` (ABI 2) | ABI change |

**Review with us**

| Repository | Pull request | Why |
|---|---|---|
| craft-libs | #397 craft-theme, #401 craft-theme-store, #402 SuiteWatch, #515 shared palette (draft) | Theme storage and selection overlap craft-ext-store theme activation |
| pdfcraft | #1090 extensible theme palettes and CLI management | Theme distribution |
| effectcraft | #3147, #3148 Plug-in Library URL and asset checks; #2995 WASM memory/table limits; #3002 plug-in loading under script permission | Look like security tightening: exceptions, please tell us |
| effectcraft | #1996 plug-in effects re-render per frame; #330 effect plug-in context API (design) | Temporal contract (D-contract `temporal`) |
| photocraft | #11307 filter concurrency; #11090 manifest name validation; #4827 invert buffer validation; #12156, #9914 registry in tests; #10446 Skin Smooth plug-in; #3772 G'MIC via external process | Touch the plugin crate or pilot; most look like fixes |
| gridcraft | #3476–#3478 Python cell sandbox; #2744 capability port host transport | Confinement and transport designs |
| wordcraft | #4603 capability-bound port hosting | Transport design |
| soundcraft | #1066, #1071, #1072, #1076, #1078 plugin parameters by command | New commands over hosted plug-ins; only the command exposure matters to us |

Other matches, such as most SoundCraft CLAP/VST3 host work, app theme styling, keyboard customisation and the Craft AI assistant, look **not affected**.

## When the hold lifts

Each group lifts when its delivery unit from the [architecture's delivery plan](architecture.md#22-delivery-plan) lands, not all at once:

| Area | Lifts when |
|---|---|
| craft-extensions store, launcher install, app folder loaders | P1 store foundation and manifest schema 2 merge |
| Outside entry points and command-spec structure | P2 operations kit merges; each app's entry points then move with that app's pilot |
| New plugin crates, runtimes, ABIs, SDKs, CLIs anywhere | P3 host runtime and P4 first contract and SDK merge |
| Install/replace commands, registries, keymap, menus | P5 manager and `plugin` CLI merge |
| EffectCraft plugin crate, Plug-in Library, helpers, control server | P6 EffectCraft pilot, which replaces them |
| Catalogue and Plug-in Library indexes | P7 catalogue merges |
| FigCraft plugin APIs | P8 FigCraft adapter, which replaces them |
| PhotoCraft plugin crate and theme adapter | P9 PhotoCraft adapter, which replaces them |
| Other apps' plugin surfaces | That app's P11 adapter |

The apps' current plugin systems are not extended in the meantime: each is removed in the same change set that lands its new adapter. We will update this list as areas lift. If something waits more than two weeks because of the hold, please ping us.

## Limits

- Paths, symbols and line numbers come from committed local snapshots at the revisions in the [evidence index](readiness/evidence-index.md). The 16 apps were not fetched; upstream `main` has moved since, and line numbers may have shifted. Each listed path was re-checked to exist at those revisions on 2026-11-09.
- craft-extensions paths are from branch `wip/extensions-handover` (`ffd8512e`); craft-launcher from `main` (`c9c3aca`) and `feat/extensions` (`6ede77f9`).
- The audits did not trace several areas (for example GUI wiring in PDFCraft, DesignCraft and DeckCraft; EffectCraft's script engine; LightCraft export; SoundCraft host internals). Plugin-relevant code may exist there that this list does not name.
- The open pull request list comes from keyword search and short descriptions. It may miss pull requests and may misclassify some.
- All eighteen design topics are settled (2026-11-10); see the [architecture](architecture.md). SoundCraft's CLAP/VST3/AU hosting and CADCraft AutoLISP are kept (2026-11-10) as native full-access mechanisms, so ordinary work on them continues; changes to how they are managed or labelled are Review with us.

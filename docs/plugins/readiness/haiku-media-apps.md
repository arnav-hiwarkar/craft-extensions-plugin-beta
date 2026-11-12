# Media app readiness audit

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: 2026-11-08, Haiku 5.5 worker A. Source-only: committed revisions read with `git show`, `git ls-tree` and `git grep` at the revision named in each section. No build, test, clippy, `cargo run`, app launch, fetch, branch change, worktree, or network access. Working trees were not read. One temporary tally file was written under `TMPDIR` during an EffectCraft count and then removed; no app or repository file was changed. Line numbers are at the named revision. "Observed" means source reading, not runtime behaviour. "Not found in inspected files" is not proof that a mechanism is absent elsewhere.

Collision classes used below: ENFORCED-IN-CODE, NOT-ENFORCED (replaces or shadows), NO-SEAM (no registration path), NOT-FOUND-IN-INSPECTED-FILES. Items marked PROPOSED are design proposals, not approved decisions and not implemented.

## Revisions

| App | Clone | HEAD (`git rev-parse HEAD`) | Baseline | Match | Branch | Working tree at start |
|---|---|---|---|---|---|---|
| EffectCraft | `/Users/ash/Projects/effectcraft` | `ce9eaec65015a5508c72af0e2351cd02612c9450` | same | yes | main | dirty (`ATTRIBUTION.md`, `crates/ui-egui/examples/snapshot.rs` modified; `scripts/` untracked). Not read. |
| PhotoCraft | `/Users/ash/Projects/photocraft` | `655295b2a2c400bded0640f5d97367000a057b2e` | same | yes | main | clean |
| FilmCraft | `/Users/ash/Projects/filmcraft` | `32749e34f611735460e9773335345db026b7b037` | same | yes | main | clean |
| LightCraft | `/Users/ash/Projects/lightcraft` | `ac1799db38661e07a855653f86feb3e66081102e` | same | yes | main | clean |
| ArtCraft | `/Users/ash/Projects/artcraft` | `1f969a9224934a55417daf10359a6fb1458b63f4` | same | yes | main | clean |

The older inventory (`docs/plugin-functional-inventory.json`) has a LightCraft entry at this revision. Its two recorded SHA-256 values (`crates/engine/src/lib.rs`, `apps/lightcraft-cli/src/main.rs`) recompute to the recorded values from committed content. It has no ArtCraft entry.

## Summary

| App | Revision | Runtime / plugin mechanism | Operation registry | CLI loads installed plugins? | Collision rule status | Smallest slice (future, PROPOSED) |
|---|---|---|---|---|---|---|
| EffectCraft | `ce9eaec` | WASM effect module, ABI v1 (wasmi, `ec_*` exports). Native in-process `EffectPlugin` trait. Native helper child processes (protocol v1). Plug-in Library (GitHub downloads, SHA-256 checked). | Static `cmd!`/`query!` table, at least 439 distinct literal IDs in engine (`crates/engine/src/cmd.rs:173-200`). 36 MCP tools. | **No at start.** The CLI never calls `effect.plugins.load` or `library.loadInstalled`. Explicit `exec` of either works. | Duplicate ID against built-in or loaded plugin: ENFORCED-IN-CODE (refused). Display-name shadowing: NOT-ENFORCED. Shortcut, menu, file type, default handler: NO-SEAM. | Refuse, or report, unresolved effect IDs at render and export (`crates/render/src/pipeline.rs:627-630`), with a fixture. |
| PhotoCraft | `655295b` | WASM filter module, ABI 1 (wasmi, `pc_*` exports). No host imports. Process-wide registry. CPU only. | Static `CommandSpec` table (229 literal `id:` entries; 111 spec modules; `crates/engine/src/cmd.rs:177-178`, `:1285-1400`). 18 automation RPC methods. | **No.** The trusted CLI session never applies preferences, so `plugin.run` returns NotFound until `plugin.install` or `plugin.reload` runs in the same process. | Same-ID install **replaces silently** by default (NOT-ENFORCED). Built-in command IDs: NO-SEAM. Menu placement: fixed by the app. Shortcut: none. | Invert pilot through `plugin.install` (data form) and `plugin.run`, with a test that `replace:false` refuses a same-ID install. |
| FilmCraft | `32749e3` | **No processing plugin runtime.** In-process encoder, decoder and frame-renderer factories. System GPU libraries loaded by name (`libloading`). | Static `cmd!`/`query!` table, at least 194 literal IDs in engine. 17 MCP tools. | N/A (no plugin runtime). | Export factories: a later registration is tried first, so it can preempt a built-in encoder (NOT-ENFORCED, in-process only). Plugin path: NO-SEAM. | None for plugins. Fixture for `file.exportMedia` with explicit `wait:false` from a one-shot CLI. |
| LightCraft | `ac1799d` | **No code-plugin mechanism found.** Resource importers: presets, profiles (LUT), curves, mask effects, keywords, print templates, slideshow templates. | Static `cmd!`/`query!`/`CommandSpec` table: 373 registration sites, at least 169 distinct literal IDs (`crates/engine/src/cmd.rs:140-160`). One MCP tool per command (`cmd_<id>`). | N/A (no plugin runtime). | Code: NO-SEAM. Resource import IDs: collisions not traced. | None for code. Possible first candidate: a namespaced resource pack through `preset.import` (future). |
| ArtCraft | `1f969a9` | **None.** Provider set is a closed Rust enum (`Artcraft`, `Midjourney`, `Fal`). Tauri shell in the same repo. | egui: no registry; UI actions and a provider `match` (`artcraft-native/src/backend/providers/submit.rs:55-60`). Tauri: `generate_handler!` with 63 entries (`crates/desktop/artcraft/src/lib.rs:228-290`). | N/A (no CLI, no automation listener found). | NO-SEAM. | None. A provider adapter trait would be PROPOSED design work. |

## Cross-app findings: blockers for this group

1. **Code can enter a process through commands that carry no caller identity (EffectCraft).** `Session::execute(&mut self, id, params)` has no caller parameter (`crates/engine/src/session/dispatch.rs:9`). Unconfined MCP `execute_command` passes any ID to `Backend::exec` (`crates/automation/src/tools.rs:202-205`; `crates/automation/src/backend.rs:69-81`). The command set includes `library.install`, which downloads and loads code (`crates/engine/src/system/library.rs:755-760`, `:1048-1054`). The library commands are blocked only on wasm32 (`library.rs:597-603`). Confinement, which refuses them (`crates/automation/src/confine.rs:70-75`), is opt-in through `--automation-read-root` or `--automation-write-root` (`apps/effectcraft-cli/src/main.rs:768-772`).

2. **Local control channels differ in authentication, and EffectCraft has none.** EffectCraft's desktop control server is loopback-only and JSON-lines, with no token or caller check found (`apps/effectcraft/src/control_server.rs:1-7`, `:25-37`). A local process that can connect can send `engine.execute` with any command (`crates/ui-egui/src/control.rs:180-183`, `:211-222`). LightCraft's control server also has no token (`apps/lightcraft/src/control_server.rs:28-35`). FilmCraft requires `FILMCRAFT_CONTROL_TOKEN` (`apps/filmcraft/src/control_server.rs:7-8`, `:66-68`). PhotoCraft requires a 256-bit bearer token as the first frame (`crates/automation/src/security.rs:14-30`, `:112-127`). ArtCraft has no listener.

3. **Missing or failed contributions are dropped at output, with no status (EffectCraft and PhotoCraft).** EffectCraft filters out unresolved effect IDs during render (`crates/render/src/pipeline.rs:627-630`). A failing plugin logs a warning, stores the error for the UI, and returns the input frame unchanged (`crates/effects/src/plugin.rs:684-688`). The CLI failure check reads only `Failed` render status (`apps/effectcraft-cli/src/main.rs:801-811`), and the render queue checks only the composition (`crates/engine/src/render_queue.rs:471`). PhotoCraft's smart-filter re-render returns `None` on a missing plugin or any error, through `.ok()`, so the filter is skipped (`crates/engine/src/cmd/plugin_cmds.rs:152-158`).

4. **Same-ID replacement and process-wide registries (PhotoCraft replaces, EffectCraft refuses but cannot unregister).** PhotoCraft's registry is a process-wide static (`crates/plugins/src/registry.rs:14`). `install` replaces by ID (`registry.rs:21-27`). `plugin.install` defaults to `replace: true` (`crates/engine/src/cmd/plugin_cmds.rs:205`). Folder loads replace too (`crates/plugins/src/registry.rs:112`). EffectCraft refuses duplicates (`crates/effects/src/plugin.rs:471-472`). Its 1024 slots are `OnceLock`s that are never cleared (`plugin.rs:393`, `:474-478`), so uninstall takes effect only at restart (`crates/engine/src/system/library.rs:957-975`, `:1057`).

5. **Saved references carry no version or digest.** EffectCraft stores only the effect ID (`crates/project/src/props.rs:183-186`). Its upgrade pass migrates by ID (`crates/engine/src/session/edit.rs:306`). PhotoCraft's smart filter stores `{id, params: resolved}` with no version or digest (`crates/engine/src/cmd/plugin_cmds.rs:133-141`). A plugin update can therefore run new code under old parameter values.

6. **Code loads from folders with no digest, as well as from the verified library (EffectCraft and PhotoCraft).** EffectCraft's desktop start loads every `.wasm` in `<config>/Plug-ins` (`apps/effectcraft/src/main.rs:274-276`; `crates/engine/src/cmd/effect.rs:37-72`). Library packages are checked against their SHA-256 at load (`crates/engine/src/system/library.rs:470-471`). PhotoCraft loads `*.wasm` from a user-chosen folder when the preference is on (`crates/engine/src/prefs.rs:889-896`; `crates/engine/src/cmd/plugin_cmds.rs:227-237`; `crates/plugins/src/registry.rs:102-117`), with no digest check.

7. **A native helper trust boundary exists in EffectCraft and is reachable from every caller.** Helpers are spawned as native children with no sandbox and no `env_clear` (`crates/helper-host/src/client.rs:390-401`). `effect.helpers.scan` starts them from fixed default folders (`crates/host/src/lib.rs:115-118`; `crates/helper-host/src/lib.rs:50-71`). The command takes only `retry` (`crates/engine/src/cmd/effect.rs:249-251`). The `EFFECTCRAFT_HELPERS=1` opt-in gates only the automatic start at launch (`apps/effectcraft/src/main.rs:151-167`). The command itself has no such gate. Helper crashes block an effect after 3 crashes (`crates/helper-host/src/blocklist.rs:16-17`).

8. **Pixel and temporal contracts are app-specific and partial.** EffectCraft WASM v1 gets pixels, `time` and `scale` only (`crates/plugin/src/lib.rs:14`). Temporal neighbours need native API 2 or a helper (`crates/effects/src/plugin.rs:275-300`). Plugin effects are registered with `gpu: false` (`plugin.rs:485`). PhotoCraft bands carry an overlap margin of at most 256 px (`crates/plugins/src/manifest.rs:13`; `crates/plugins/src/apply.rs:167-170`). Kernels wider than the declared overlap would produce band seams. This is a design reading, not a runtime result. FilmCraft and LightCraft have no plugin pixel or time contract.

**Conflicts for the coordinator, not counted as blockers.** ArtCraft's native crate says "no Tauri, no webview" (`artcraft-native/Cargo.toml:3`; `artcraft-native/src/main.rs:3`), yet depends on `wry` (`artcraft-native/Cargo.toml:50`) for a Midjourney sign-in WebView (`artcraft-native/src/overlays/providers.rs:2`, `:13-14`). The repo's default workspace member is the Tauri app (`Cargo.toml:46`). House rule 4 covers native Craft apps. Whether the WebView is an accepted exception is not established here. PhotoCraft's example plugin uses `unsafe extern "C"` (`examples/plugins/invert-rs/src/lib.rs:52`). It is guest code, not host code, and whether the no-unsafe rule covers example crates is not decided here.

---

## EffectCraft (`ce9eaec65015a5508c72af0e2351cd02612c9450`)

Paths are relative to the app repo at this revision.

### 1. Repo, branch, revision, workspace

- Clone `/Users/ash/Projects/effectcraft`, branch `main`, HEAD `ce9eaec65015a5508c72af0e2351cd02612c9450`, matching the baseline.
- Workspace: `Cargo.toml:1-4` (`crates/*`, `apps/*`, `xtask`). About 40 crates under `crates/`, including `plugin`, `effects`, `engine`, `host`, `helper-host`, `automation`, `render`, `project`, `export`, `script` and `ui-egui`. Apps: `apps/effectcraft` (desktop), `apps/effectcraft-cli`, `apps/effectcraft-web`.
- Plugin-related crates named in the workspace: `Cargo.toml:54-55`.

### 2. Operation surface and a traced operation

**Registry.** `crates/engine/src/cmd.rs:101-120` (`Run`, `CommandSpec`), `:122-141` (`cmd!`, `query!` macros), `:173-200` (`command_specs()` aggregates module specs), `:278-286` (`find`, first registration wins: `or_insert`). The table holds at least 439 distinct literal IDs from `cmd!`/`query!` in non-test engine code (the count is a lower bound; `cmd.rs:279` says "~2,000 ids", so some IDs are generated). The top families are `layer` (79), `view` (39), `file` (36), `keys` (23), `comp` (20), `essential` (17), `edit` (17), `time` (15), `camera` (15). Categories: queries (`query!`, 57 sites), edits (`cmd!`), file, import and export, render, and UI-only (`view`, `keys`, `help`). The MCP catalogue has 36 tools (`crates/automation/src/tools.rs:557`).

**Traced operation: `effect.apply`.**
1. Definition: `crates/engine/src/cmd/effect.rs:1278`, `cmd!("effect.apply", "Apply Effect", [], None, "{effect: id|name…, layers?}", has_layers, apply)`.
2. Dispatch: `Session::execute` (`crates/engine/src/session/dispatch.rs:9-31`) finds the spec (`cmd.rs:278-286`), checks enablement (`dispatch.rs:11-20`), runs under `guard::guarded` (`:22`) and journals it (`:24-29`). Agent paths use `execute_checked`, which rejects unknown keys (`dispatch.rs:35-39`).
3. Validation: the `effect` parameter is required (`cmd/effect.rs:274`). It is resolved by `effectcraft_effects::lookup` (`crates/effects/src/lib.rs:792-800`): ID first, then case-insensitive display name over `all()`, then aliases. Layers are resolved (`cmd/effect.rs:276`); an empty set is refused (`:277-279`).
4. Edit: `s.edit("Apply <name>", …)` (`cmd/effect.rs:284`) goes to `Session::edit` (`crates/engine/src/session/edit.rs:10-15`). `edit_inner` clones the project (`:33-36`), runs the closure, syncs derived data, skips a no-op (`:60`), and records one undo step with an optional merge key (`:64-72`).
5. Model change: `effectcraft_effects::instantiate` sets `GroupKind::Effect { effect: spec.id }` (`crates/effects/src/lib.rs:822-824`). The group is pushed into the layer's `effects` group (`cmd/effect.rs:289-294`).
6. Result: `{effects, paths, effect}` (`cmd/effect.rs:311`).
7. Render: the stored ID is resolved per frame, and unresolved IDs are dropped (`crates/render/src/pipeline.rs:627-630`; see item 6).

**Command-line and control paths.** The CLI `exec` subcommand (`apps/effectcraft-cli/src/main.rs:617-631`) and `mcp` (`:763-790`) both reach `Backend::exec`. Desktop control `engine.execute` reaches `execute_engine` for engine IDs (`crates/ui-egui/src/control.rs:211-222`).

### 3. Plugin, extension, script, resource mechanisms

- **WASM effect, ABI v1.** Runtime: wasmi (`crates/plugin/src/wasm.rs:5`; `crates/plugin/Cargo.toml:13-15`, feature `wasm`). `WASM_API_VERSION = 1`, `PLUGIN_API_VERSION = 2` (`crates/effects/src/plugin.rs:41-44`). Required exports: `ec_api_version`, `ec_manifest_ptr`, `ec_manifest_len`, `ec_alloc`, `ec_render` (`wasm.rs:50-51`, `:75-81`). The `ec_render` signature is `(pixels, width, height, params, nparams, time: f64, scale: f64) -> i32` (`crates/plugin/src/lib.rs:14`).
- **Host surface.** None. A module with any import is refused (`wasm.rs:71-72`).
- **Limits.** Fuel: 50,000,000 base plus 20,000 per pixel per render (`wasm.rs:11-12`, `:46`, `:125-126`). Linear memory: 1 GiB, one instance (`wasm.rs:14`, `:44`). Manifest: 1 MiB (`wasm.rs:16`). No wall-clock limit was found in `wasm.rs`; fuel is the only bound there.
- **Pooling.** Instances are pooled and returned only after a successful render (`wasm.rs:62`, `:95-103`, `:157-159`). Linear memory and globals persist between renders, and no reset was found.
- **Manifest.** `PluginManifest` fields: `api`, `id`, `name`, `category`, `version`, `author`, `description`, `params` (`crates/effects/src/plugin.rs:168-183`). There is no permission, menu, shortcut or file-type field.
- **Validation.** Reverse-domain-style ID (letters, digits, `.`, `_`, `-`); the `ec.` prefix is reserved (`plugin.rs:196-199`). The API version must be 1 or 2 (`plugin.rs:194`). Parameter count, popup and slider ranges are checked (`plugin.rs:188-236`).
- **Native in-process.** `EffectPlugin` trait (`crates/effects/src/plugin.rs:370-390`) and `register_plugin` (`:467`). These are for Rust code linked at build time. No dynamic-library loader was found in the plugin, effects, helper, host or engine files inspected (`git grep` for `libloading|dlopen|Library::new` returned zero matches).
- **Native helper, protocol v1.** Constants: `MAX_LINE` 256 MiB (`crates/helper-host/src/protocol.rs:24`), `MAX_PIXELS` 2^28 (`:27`), `PIXEL_BYTES` 16 (`:30`), so the largest exchange file is 4 GiB. Timeouts: heartbeat after 30 s, heartbeat reply 10 s (`crates/helper-host/src/client.rs:29-30`); render 120 s by default (`:49`); load 600 s (`:57`). Spawn: `Command::new(program)` with a helper argument, stdio piped, an env variable for the state directory, and no sandbox or `env_clear` (`client.rs:390-401`). Crash limit 3 (`crates/helper-host/src/blocklist.rs:16-17`). Restarts are capped at a set rate per minute (`client.rs:380-388`).
- **Helper discovery.** `helpers/` next to the executable, `helpers/` in the settings folder, and `EFFECTCRAFT_HELPER_PATH` (`crates/helper-host/src/lib.rs:50-71`). A file whose name starts with `effectcraft-helper-` is a helper (`lib.rs:73-84`). An explicit file path is accepted under any name (`lib.rs:89-97`).
- **Plug-in Library.** Downloads only from GitHub hosts (`crates/engine/src/system/library.rs:352-356`). The catalogue SHA-256 is checked on download (`library.rs:494-498`) and at load (`library.rs:470-471`). Installed records keep the SHA (`library.rs:119-131`). Layout: `<Plug-ins>/Library/<id>/<version>/` (`library.rs:11-12`).
- **Loose files.** `effect.plugins.install` copies a `.wasm` into the Plug-ins folder, capped at 64 MiB and checked by loading first (`crates/engine/src/cmd/effect.rs:78-81`, `:83-96`). It needs `s.config` (`cmd/effect.rs:92-93`).
- **Grants.** None found. `EffectSpec` has no permission field (`crates/effects/src/lib.rs:645-655`).
- **Other resources** (LUT, brush, font, template): no plugin importer seam found.

### 4. Contribution families

| Family | Seam today | Evidence |
|---|---|---|
| Effects (WASM, helper, native Rust) | Yes: `register_plugin` | `crates/effects/src/plugin.rs:467` |
| Commands, tools, panels | NO-SEAM. The table is static. | `cmd.rs:173-200` |
| Menus and shortcuts | NO-SEAM. Manifests have no such fields. | `plugin.rs:168-183`; `crates/effects/src/lib.rs:645-655` |
| Import and export formats | NO-SEAM. `OutputFormat` is an enum. | `crates/project/src/render_queue.rs:510` |
| Resources (LUT, brush, font, template) | NO-SEAM found | not found in inspected files |
| Themes | not traced | out of focus |
| Jobs and renders | NO-SEAM | `cmd.rs:173-200` |

### 5. Transactions, undo, jobs, persistence, output

- **Undo.** Whole-project snapshot steps (`crates/engine/src/session/edit.rs:32-72`). Effect output is a render-time result, not a document edit, so a plugin's render output is never an undo step.
- **Render failure.** A plugin error returns the frame unchanged and stores the error (`crates/effects/src/plugin.rs:684-688`). The error is shown in the UI (`crates/effects/src/plugin.rs:414-419`; `crates/ui-egui/src/panels/effect_controls/panel.rs:133`).
- **Jobs.** The CLI starts `renderQueue.render` with `wait: false`, then polls `is_rendering()` every 100 ms and prints progress (`apps/effectcraft-cli/src/main.rs:919-951`). Cancellation was not traced.
- **Project format.** The file has a numeric `schema` (`crates/project/src/lib.rs:56`). Effect references are IDs (`props.rs:183-186`). `upgrade_effects` (`crates/engine/src/session/edit.rs:277-317`) migrates by ID (`:306`) and sets the schema (`:316`).
- **Output.** `render --out` writes through the render queue. The file-writing path was not traced in full. `failed_render` checks only `Failed` status (`apps/effectcraft-cli/src/main.rs:801-811`).

### 6. Collision and core-override behaviour

| Rule | Status | Evidence |
|---|---|---|
| Plugin ID equal to a built-in effect ID | ENFORCED-IN-CODE (refused) | `crates/effects/src/plugin.rs:471-472` (`crate::registry()` checked) |
| Plugin ID equal to a loaded plugin ID | ENFORCED-IN-CODE (refused, not replaced), whatever the source | `plugin.rs:471-472`; tests `crates/plugin/src/tests.rs:154`, `crates/host/tests/it/plugins.rs:302` |
| `ec.` prefix | ENFORCED-IN-CODE | `plugin.rs:197` |
| Charset and reverse-domain shape | ENFORCED-IN-CODE for charset. Reverse-domain shape is not required. | `plugin.rs:196` |
| Display name equal to a built-in name | NOT-ENFORCED. Name lookup may return the plugin. | `register_plugin` checks IDs only (`plugin.rs:471`). `lookup` falls back to names (`crates/effects/src/lib.rs:794`) over `all()`, which is sorted by category then name once any plugin is registered (`lib.rs:757-765`). A plugin in an earlier-sorting category can win (inference, not run). |
| Helper effect with a built-in ID | ENFORCED-IN-CODE (error reported) | `crates/helper-host/src/lib.rs:131-136` |
| Helper effect with a loaded plugin's ID | NOT-ENFORCED in the sense of a silent skip: no error, the existing plugin is kept | `crates/helper-host/src/lib.rs:127-128` |
| Shortcut, menu placement, file type, default handler | NO-SEAM | `plugin.rs:168-183`; `lib.rs:645-655` |
| Duplicate built-in command IDs | NOT-FOUND-IN-INSPECTED-FILES (no uniqueness check; first wins) | `cmd.rs:278-286` |
| Namespace | Only `ec.` is reserved | `plugin.rs:197` |
| Slot exhaustion | ENFORCED-IN-CODE (1024 slots; message says restart) | `plugin.rs:47`, `:474-477` |
| Update or uninstall while loaded | NO-SEAM for hot replace. Changes apply at restart. | `plugin.rs:393`, `:474-478`; `library.rs:957-975`, `:1057` |

### 7. GUI and CLI parity

- **Shared registry.** Desktop control, CLI `exec` and MCP all run `Session::execute_checked`-family calls on the same `command_specs()` table (`cmd.rs:173-200`; `dispatch.rs:35-39`). UI-only IDs go through the menu dispatcher (`crates/ui-egui/src/control.rs:218-221`).
- **Fresh CLI start.** The loader is wired in the host session (`crates/host/src/lib.rs:97-100`), but the only startup load is in the desktop (`apps/effectcraft/src/main.rs:272-283`). `apps/effectcraft-cli/src/main.rs` calls neither `effect.plugins.load` nor `library.loadInstalled` (`git grep`). An explicit `exec effect.plugins.load {folder}` works. The command is not refused under `desktop_only`, which applies only on wasm32 (`library.rs:597-603`), and is not in the confinement refusal list (`crates/automation/src/confine.rs:98`).
- **Installing from the CLI.** `library.install` runs headless (`library.rs:755-760`) and uses the host session's `plugins_dir` (`crates/host/src/lib.rs:100`; `library.rs:185-191`). `effect.plugins.install` fails there. The CLI session never sets `s.config`; it reads only auto-save preferences (`apps/effectcraft-cli/src/main.rs:471-482`), and `plugins_install` requires `s.config` (`cmd/effect.rs:92-93`).
- **Hidden dialogs.** Not traced for headless paths. A `Services` injection point exists (`crates/automation/src/confine.rs:21`, `ConfinedServices`).

### 8. Caller authority

- **No caller identity at dispatch.** `Session::execute` takes no caller (`dispatch.rs:9`). The only caller-kind state is a script counter (`dispatch.rs:41-55`), used to stop scripts changing script-limit settings (`:41-44`).
- **Unconfined MCP and CLI.** Any registered command runs (`tools.rs:202-205`; `backend.rs:69-81`). Confinement, if given, refuses `library.*`, `effect.plugins.install`, `effect.helpers.scan`/`load`, `file.executeFile`, and scripts (`confine.rs:45-76`). It also refuses UI-driving tools (`tools.rs:108-118`). Confinement is opt-in (`cli main.rs:768-772`).
- **Desktop control.** Loopback only (`control_server.rs:25-27`) with no token or caller check found. Bridge clients are restricted to loopback (`crates/automation/src/bridge.rs:22-27`).
- **Effect helper start.** `effect.helpers.scan` has no gate apart from `has_helpers` (`cmd/effect.rs:249-251`, `:1370-1377`). The launch-time opt-in applies only to the automatic call (`apps/effectcraft/src/main.rs:151-167`).
- **Consequence.** No narrower authority can be given to a plugin, because there is no caller or authority object. The only narrowing is the confinement roots, and they are opt-in.

### 9. Reusable APIs and missing hooks

- **Reusable.** `register_plugin` (`crates/effects/src/plugin.rs:467`); `EffectPlugin` and `PluginHost` (`plugin.rs:275-300`, `:370-390`); `plugin_error` (`plugin.rs:414`); `Session::execute_checked` (`dispatch.rs:35`); `command_specs()` and `find` (`cmd.rs:173`, `:278`); `Session::edit` (`session/edit.rs:10`); the library install record and SHA check (`library.rs:110-131`, `:456-472`); and the WASM loader hook (`crates/plugin/src/lib.rs:41-63`).
- **Missing hooks (PROPOSED).** (a) A shared resolver that runs before the session opens. None exists today. (b) A missing-contribution status consumed by render and export. (c) A caller or authority parameter on dispatch. (d) A grant check before registration. (e) A way to clear `SLOTS` on uninstall (`plugin.rs:393`). (f) A version or digest in `GroupKind::Effect` (`props.rs:183-186`). (g) A render-cache key that includes the digest. No cache keyed by plugin output was found.

### 10. App-specific data constraints

- **WASM pixel contract.** Premultiplied RGBA `f32`, row-major, row 0 at the top (`crates/plugin/src/lib.rs:14-18`). Pixels are written as little-endian `f32` (`wasm.rs:133-137`).
- **Parameters.** Flat `f64`. Points are scaled to frame pixels (`crates/plugin/src/lib.rs:79-89`). Popups are indexes. Colours are four `f64`.
- **Frame context.** `PluginFrame` carries `width`, `height`, `scale`, `origin` and `layer_size` (`crates/effects/src/plugin.rs:243-257`). WASM v1 receives only `time` and `scale`. Native API 2 adds `input_at`, `layer_input`, `params_at`, `frame_rate`, `time` and `instance_key` (`plugin.rs:275-300`).
- **Limits.** `MAX_PLUGINS` 1024 (`plugin.rs:47`). `MAX_PADDING` 4096 (`:50`). Helper pixel budget as in item 3.
- **Audio.** Audio effects are excluded from the video path (`crates/render/src/pipeline.rs:631`). No plugin audio contract was found.
- **Colour model.** Not traced.

### 11. Smallest useful slice and behavioural proof (PROPOSED, future work)

- **Slice A: refuse or report unresolved effects.** Collect unresolved effect IDs in the render path (`crates/render/src/pipeline.rs:627-630`), return them to the render queue (`crates/engine/src/render_queue.rs:471`), and extend the CLI failure check (`apps/effectcraft-cli/src/main.rs:801-811`). Default: refuse output while any ID is unresolved, unless the caller passes a skip flag that lists what was skipped. This needs no new install path.
  - Proof (test to write): save a project with effect ID `org.test.gain`, render it through the CLI with the plugin absent, and expect a non-zero status naming the effect. With the skip flag, expect output plus a report. A plugin render that fails (`plugin.rs:684-688`) should also refuse output.
- **Slice B: CLI eligibility for installed packages.** A shared bootstrap that calls `library.loadInstalled` for the read-only render commands, with no install path.
  - Proof (test to write): `effectcraft-cli render` of a project whose effect comes from an installed package resolves that effect and matches the desktop render within tolerance.
- **Not proposed now:** helper enablement from the CLI or MCP. Caller authority for `library.install` and `effect.helpers.scan` should be decided first.

### 12. Evidence limits

Not traced: `crates/engine/src/cmd/params.rs` (`layers_p`); `crates/engine/src/lib.rs` plugin-loader assignment; `library.rs` `install_job` in full (`:504-560`); export output writers in `crates/export`; the script engine (`crates/script`), which is reachable unconfined; WASM pool sizing and module size; helper exchange-file lifecycle; GPU path; web build; the effects category UI. The `scripts/` directory and `ATTRIBUTION.md` in the dirty working tree were not read.

---

## PhotoCraft (`655295b2a2c400bded0640f5d97367000a057b2e`)

Paths are relative to the app repo at this revision.

### 1. Repo, branch, revision, workspace

- Clone `/Users/ash/Projects/photocraft`, branch `main`, HEAD `655295b2a2c400bded0640f5d97367000a057b2e`, matching the baseline. Working tree clean at start.
- Workspace: `Cargo.toml:1-4`. Default members: `crates/*`, `apps/photocraft`, `apps/photocraft-cli`, `xtask` (`Cargo.toml:4`). About 27 library crates under `crates/` (`plugins`, `engine`, `automation`, `doc`, `ops`, `raster`, `paint`, `compose`, `format`, `codecs`, `psd`, `raw`, `affinity`, `text`, `ui-egui`, `gpu`, `cms`, `tablet`, `ipp`, `testkit`, `vector`, and others). Apps: `photocraft`, `photocraft-cli`, `photocraft-web`, `photocraft-web-boot`.
- The `plugins` crate is named in the workspace at `Cargo.toml:34`. It denies `unwrap`, `expect`, `panic`, `unimplemented`, `todo` and `unreachable`, and forbids `unsafe` (`crates/plugins/src/lib.rs:20-21`).

### 2. Operation surface and a traced operation

**Registry.** `crates/engine/src/cmd.rs:173-178` (`command_specs()` returns the registry), `:216-218` (`find`), `:226-234` (`Registry::new`; duplicate IDs keep the first, `:230-231`). Module specs are added in one build function (111 `v.extend(...)` lines, `cmd.rs:1285-1400`). There are 229 literal `id:` entries in non-test engine code. Image adjustments are generated from a table (`cmd.rs:1196`, `:1213`). The total was not counted exactly. Categories include selection, layer, adjustment, filter, file, channel, and plugin commands.

**Automation.** RPC methods: 18 names (`crates/automation/src/rpc.rs:42-62`), including `engine.execute` and `engine.commands` (`rpc.rs:87-98`). CLI subcommands: `convert`, `info`, `run`, `batch`, `droplet`, `commands`, `mcp`, `serve` (`apps/photocraft-cli/src/args.rs:31-39`).

**Traced operation: `plugin.run`.**
1. Definition: `crates/engine/src/cmd/plugin_cmds.rs:318-327`. Enabled by `crate::filters::has_filterable_layer` (`:324`). Journaled (`:326`).
2. Dispatch: `Session::dispatch` (`crates/engine/src/jobs.rs:824-841`). It runs the authorization gate for untrusted sessions (`:838-840`) and sets an authorization scope (`:841`). Automation calls enter through `command_start` (`crates/automation/src/headless.rs:377-392`).
3. Validation: `plugin_by_id` looks up the installed plugin (`plugin_cmds.rs:100-103`). `resolve_params` rejects bad parameters before anything is touched (`:109`). The layer is resolved (`:110`).
4. Edit: `s.edit(&label, …)` (`:113`) with the manifest name as the undo label. Session edits run under an authorization scope and a document-path scope (`crates/engine/src/session/execute.rs:206-232`).
5. Pixel layer: `channel_surface_for_filter` may redirect to a channel (`:116-119`). Otherwise the surface is replaced after `plugin.apply` succeeds, and Lock Transparency re-applies alpha (`:121-130`).
6. Smart object: a `SmartFilter { command: "plugin.run", params: {id, params: resolved} }` is added (`:133-141`).
7. Result: `{layer, plugin}` (`:146`).

### 3. Plugin mechanisms

- **Runtime.** wasmi (`crates/plugins/src/runtime.rs:5`). `ABI_VERSION = 1` (`runtime.rs:14`). Exports checked: `pc_abi_version` (`runtime.rs:194`), `pc_manifest` (`:199`), `pc_alloc` and `pc_filter` (`:208-209`, `:281-282`). The example module exports the same four (`examples/plugins/invert-rs/src/lib.rs:27-52`).
- **Host imports.** None. A module that imports anything is refused (`runtime.rs:184-190`).
- **Manifest.** JSON, at most 64 KiB (`crates/plugins/src/manifest.rs:9`), at most 64 parameters (`:11`), overlap at most 256 px (`:13`). Kind: `filter` only (`:18-20`). Area: `content` or `canvas` (`:25-31`). ID: 1 to 64 characters from A–Z, a–z, 0–9, `.`, `_`, `-` (`:143-145`). Name: at most 64 printable characters (`:146-150`). Parameter names: ASCII alphanumerics and `_`, no leading `_`, and not one of `id`, `layer`, `target`, `channel` (`:407-413`).
- **Limits.** Module 32 MiB (`runtime.rs:47`), memory 512 MiB per instance (`:48`), 2 GiB per run (`:49`), fuel 50,000,000 plus 4,000 per sample (`:51-52`), recursion depth 1024 (`:53`), wall-clock 60 s (`:54`; native only, per the comment at `:38-40`). Fuel is handed out in 20,000,000 slices (`:17`) with deadline checks between them. Instantiation allows one memory, four tables, one instance (`:237`).
- **Parallel bands.** `parallel_bands = min(threads, total / instance)` (`runtime.rs:63-66`). At defaults that is 4.
- **Loading.** `registry::load_file` opens without blocking on FIFOs, requires a regular file, and enforces the module size (`crates/plugins/src/registry.rs:71-89`). `plugin.install` takes a `path` (native only) or base64 `data` up to 32 MiB encoded (`crates/engine/src/cmd/plugin_cmds.rs:160-161`, `:192-204`). Folder load: up to 1000 `.wasm` files, in file-name order (`registry.rs:102-117`). Preference folder: when `useAdditionalPluginsFolder` is on, loaded at preference apply (`crates/engine/src/prefs.rs:889-896`; `plugin_cmds.rs:227-237`).
- **Digest, signature, grants.** None found in `crates/plugins` (`git grep` for `sha|digest|hash|permission|grant|signature` in the crate returned no matches, apart from `hash` in unrelated comments). No permission field appears in `Manifest` (`manifest.rs:108-127`).
- **Pixel and band contract.** Normalised `f32` in the document colour model, with a format code: depth in bits 0–7, straight alpha in bit 8, colour mode in bits 16–23 (`crates/plugins/src/apply.rs:12-33`). Each band is inflated by the overlap, and edges repeat outside the canvas (`apply.rs:48-71`, `:167-170`). A JSON `_image` context is added to the parameters (`apply.rs:173-182`). Bands are capped at 65,536 (`apply.rs:74`). A panic in the run is caught and reported as a trap (`apply.rs:184-187`). Non-finite output reverts to the original (`apply.rs:207-209`). Integer depths are clamped (`apply.rs:210-212`). Selection blending (`apply.rs:213-215`).

### 4. Contribution families

| Family | Seam today | Evidence |
|---|---|---|
| Filters | Yes: `Kind::Filter` through `plugin.run` | `manifest.rs:18-20`; `plugin_cmds.rs:318-327` |
| Menu items | App-generated: `plugin.filter.<id>` under Filter ▸ Plug-ins, labelled with the manifest name, no shortcut | `crates/ui-egui/src/menus/plugin_ui.rs:9-43` |
| Smart filters | Through the recorded `plugin.run` command | `plugin_cmds.rs:133-141` |
| Commands | NO-SEAM. Plugins do not register commands. | `cmd.rs:173-178` |
| Shortcuts | NO-SEAM. Plugin items have `shortcut: None`. | `plugin_ui.rs:31` |
| Import or export formats, resources, themes, jobs | NO-SEAM found | not found in inspected files |

### 5. Transactions, undo, jobs, persistence, output

- **Undo.** One undo step per run, labelled with the manifest name (`plugin_cmds.rs:111-113`, `:145`). The surface is replaced only after success (`:127`).
- **Smart objects.** The recorded filter is stored in the document (`:133-141`). Re-render calls `apply_to_surface` (`:152-158`), which resolves the plugin from the process-wide registry, not from the document. A missing plugin or any error gives `None`, and the filter is skipped (`:149-151`, `:156-157`).
- **Registry scope.** The registry is per process, "as in Photoshop", so smart filters and previews find the same plugins (`crates/plugins/src/registry.rs:3-4`, `:14`).
- **Jobs.** `plugin.run` runs inline inside the edit. Whether it is eligible for background jobs was not traced. `Session::start` dispatches background jobs (`crates/engine/src/jobs.rs:554-555`).
- **CLI.** `run` executes each step through `command_run`, which waits (`apps/photocraft-cli/src/document.rs:75-112`; `crates/automation/src/headless.rs:370-372`).
- **Installation is not journaled.** `plugin.install` is `journal: false` (`plugin_cmds.rs:293`).

### 6. Collision and core-override behaviour

| Rule | Status | Evidence |
|---|---|---|
| Same plugin ID, different source | NOT-ENFORCED. Replaces silently by default. | `crates/plugins/src/registry.rs:21-27`; `plugin_cmds.rs:205` (`replace` defaults to `true`) |
| Same ID, `replace: false` | ENFORCED-IN-CODE (refused) | `plugin_cmds.rs:206-208`; test `plugin_cmds/tests.rs:130` |
| Folder load with a duplicate ID | NOT-ENFORCED. Later file replaces earlier. | `registry.rs:112` (calls `install`) |
| Plugin ID equal to a built-in command ID | NO-SEAM. Plugin IDs live in the `plugin.run` argument, not the command table. | `plugin_cmds.rs:105-109`; `cmd.rs:173-178` |
| Plugin ID with reserved prefix | NOT-FOUND-IN-INSPECTED-FILES. Only charset and length are checked. | `manifest.rs:143-145` |
| Display name equal to a built-in name | NOT-ENFORCED. Label is not checked. Built-in Invert is `image.adjustments.invert` (`cmd.rs:1196`, `:1213`), in Image ▸ Adjustments with `Cmd+I` (`cmd.rs:1235`, `:1241`). A plugin named "Invert" would show under Filter ▸ Plug-ins. | `manifest.rs:146-150`; `plugin_ui.rs:29` |
| Reserved parameter names | ENFORCED-IN-CODE | `manifest.rs:407-413` |
| Menu placement chosen by a plugin | NO-SEAM | `plugin_ui.rs:12-20` (placed above `plugin.install`) |
| Smart-filter plugin missing at render | Silent skip (see item 5) | `plugin_cmds.rs:152-158` |

### 7. GUI and CLI parity; fresh CLI

- **Shared registry.** The desktop menu lists installed plugins from the registry (`crates/ui-egui/src/menus/plugin_ui.rs:14`). CLI and automation use the same `command_start` and `execute` (`headless.rs:377-392`). Hidden dialogs: the `plugin.install` label carries "…", which suggests a dialog in the GUI (`plugin_cmds.rs:287-288`). The headless command takes `path` or `data` and requests no dialog (`plugin_cmds.rs:190-210`).
- **Fresh CLI.** `run_cmds` creates `Headless::trusted_local()` (`apps/photocraft-cli/src/document.rs:77`). Its `Session::new()` (`crates/engine/src/session/documents.rs:19-21`) has no preferences applied, and the CLI source has no preference handling (`git grep`). The registry is process-wide, so a `plugin.run` in a new process returns NotFound (`plugin_cmds.rs:100-103`) until `plugin.install` or `plugin.reload` runs in that same process. A `run` invocation executes its commands in order in one process (`apps/photocraft-cli/src/document.rs:94-101`), so a `plugin.install` step can precede a `plugin.run` step.
- **Desktop.** `apply_prefs` runs at startup (`crates/ui-egui/src/prefs/prefs_ui/startup.rs:44`), which calls `sync_prefs` (`crates/engine/src/prefs.rs:1377`).

### 8. Caller authority

PhotoCraft has the clearest caller model in this group. Authority is set by how the session is built:

- **Constructors.** `Headless::new()` is Denied (`crates/automation/src/headless.rs:37-45`). `trusted_local()` is for CLI invocations, and the source says the CLI caller "supplies these host paths" (`headless.rs:47-51`). `with_workspace()` is a capability workspace (`headless.rs:56-57`). Untrusted sessions install `authorize_engine_step` (`headless.rs:15-19`). Automation in the CLI uses `new()` and `with_workspace()` (`apps/photocraft-cli/src/automation.rs:15`, `:37`).
- **The gate.** `command_start` skips the gate only for `TrustedLocal` (`headless.rs:379-383`). The gate refuses non-safe `file.*` commands (`crates/automation/src/workspace.rs:327-346`), ambient-path parameters (`workspace.rs:369-375`), `plugin.reload` (`workspace.rs:484`), and `plugin.install` with `path` but no `data` (`workspace.rs:471-477`). It also refuses preference writes to `plugIns` (`workspace.rs:446-461`).
- **Per-step checks.** Nested `actions.play` steps and the desktop path are checked too (`crates/engine/src/jobs.rs:835-840`; `apps/photocraft/src/services.rs:680-711`).
- **Token transports.** A 256-bit random token, compared without an early exit (`crates/automation/src/security.rs:78-109`). The first frame must be `auth` within 10 s (`security.rs:14-27`, `:112-127`). The token file must be owner-only (`security.rs:141-166`). A new file is created with mode 0600 (`security.rs:168-181`).
- **What automation can still do.** `plugin.install` with `data`, `plugin.remove` and `plugin.run` are not on any gate list (`workspace.rs:465-486`). An automation caller can therefore install a module from base64 and replace an installed plugin by ID, since `replace` defaults to `true` (`plugin_cmds.rs:205-209`). This is a source reading, not run.

### 9. Reusable APIs and missing hooks

- **Reusable.** `Plugin::load`, `registry::{install, get, list, remove, load_file, load_folder}` (`crates/plugins/src/lib.rs:28-30`; `registry.rs`); `Manifest::resolve_params`, `preview_factor`, `scale_for_preview` (`plugin_cmds.rs:61-98`); `Session::edit` (`session/execute.rs:206`); `Session::authorize` (`jobs.rs:838`).
- **Missing hooks (PROPOSED).** (a) A reserved plugin ID namespace and an owner or source check on replacement. (b) A version or digest in `SmartFilter` (`plugin_cmds.rs:133-141`). (c) A status type for a missing plugin in smart-filter rendering. (d) A document-scoped or session-scoped registry, instead of the process-wide static (`registry.rs:14`). (e) Grants for plugin use by automation callers. (f) Caller-specific rules for `plugin.install` and `plugin.remove`.

### 10. App-specific data constraints

- **Colour.** Samples are normalised `f32` in the document colour model (`crates/plugins/src/apply.rs:12-33`). The format code carries bit depth, alpha and colour mode.
- **Geometry.** Bands carry an overlap margin of at most 256 px (`manifest.rs:13`; `apply.rs:167-170`). Edges repeat outside the canvas (`apply.rs:48-71`). Area is `content` (the layer's non-empty pixels) or `canvas` (`manifest.rs:25-31`).
- **Units.** Parameters with `"unit": "px"` are divided by the preview factor for live previews (`plugin_cmds.rs:61-98`).
- **Selection and locks.** Selection mixes the result (`apply.rs:213-215`). Lock Transparency keeps the original alpha (`plugin_cmds.rs:121-130`).
- **Parameters.** Resolved values include defaults (`plugin_cmds.rs:109`), so a version change changes what is stored.

### 11. Smallest useful slice and behavioural proof (PROPOSED, future work)

- **Slice A: invert pilot.** The module already exists in the tree: `examples/plugins/invert-rs` (ID `org.photocraft.example.invert`, `src/lib.rs:14`), and `crates/plugins/tests/fixtures/invert.wasm`. The pilot installs it with `plugin.install` (data or path form), then runs it through `plugin.run` on a pixel layer.
  - Proof (test to write): in `crates/engine/src/cmd/plugin_cmds/tests.rs`, install the example with `replace: false`, run `plugin.run`, and check the inversion at 8, 16 and 32 bits. Check that `image.adjustments.invert` output is unchanged. Existing crate-level coverage: `crates/plugins/tests/abi.rs:68-79` (the example loads and inverts at every depth).
  - Constraint: pick a display name that does not collide with the built-in Invert, or accept the menu overlap.
- **Slice B: same-ID collision.** Change the default of `plugin.install` to refuse a same-ID install, or require the same source (PROPOSED; `plugin_cmds.rs:205`).
  - Proof (test to write): a second install with the same ID and different bytes is refused by default. The existing test covers only an explicit `replace: false` (`plugin_cmds/tests.rs:130`).
- **Slice C: automation authority for installs.** Gate `plugin.install` with `data`, and `plugin.remove`, for automation callers (PROPOSED; `workspace.rs:465-486`).
  - Proof (test to write): an automation `engine.execute` of `plugin.install` returns an error, and the registry is unchanged.

### 12. Evidence limits

Not traced: the job eligibility of `plugin.run`; the full desktop control transport (`crates/automation/src/server.rs`, `server/transport.rs`); the document format's versioning for plugin smart filters; the plugin dialog flow in the UI; the rayon band scheduler beyond `runtime.rs:63-66`; the authorization path for stored smart filters (`crates/engine/src/cmd/smart_cmds/filters.rs:133-150`, `rendering.rs:314`); the `photocraft-web` build. `ATTRIBUTION.md` and `docs/` were not read.

---

## FilmCraft (`32749e34f611735460e9773335345db026b7b037`)

Paths are relative to the app repo at this revision.

### 1. Repo, branch, revision, workspace

- Clone `/Users/ash/Projects/filmcraft`, branch `main`, HEAD `32749e34f611735460e9773335345db026b7b037`, matching the baseline. Working tree clean at start.
- Workspace: `Cargo.toml:1-4`. About 52 crates under `crates/` (for example `engine`, `export`, `codecs`, `platform`, `automation`, `render`, `media`, `edit`, `project`, `format`, `ui-egui`, `gpu`). Apps: `filmcraft`, `filmcraft-cli`, `filmcraft-web`.

### 2. Operation surface and a traced operation

**Registry.** `crates/engine/src/commands.rs:35-46` (`cmd!` and `query!` macros), `:49-52` (`command_specs()`), `:54-56` (`find`, a linear first match). Literal IDs: at least 194 from `cmd!`/`query!` in non-test engine code, plus 86 `id:` literals. Registration is spread across modules; `file.exportMedia` is at `commands.rs:1959`.

**MCP.** 17 `#[tool(` attributes in `crates/automation/src/server.rs`: `serve_stdio`, `run`, `document`, `ui`, `command_list`, `command_run`, `command_batch`, `doc_inspect`, `render_preview`, `project_inspect`, `sequence_inspect`, `media_import`, `render_frame`, `ui_inspect`, `ui_elements`, `ui_click`, `ui_drag`, `ui_key`, `ui_type`, `ui_screenshot`, `ui_control`, `screenshot`, and resource methods.

**Traced operation: `file.exportMedia`.**
1. Definition: `crates/engine/src/commands.rs:1959`. Handler: `export_media` (`crates/engine/src/export_tools.rs:597-624`).
2. Validation: sequence (`:600`), export settings (`:601`), required `path` (`:602-604`), range with warnings (`:606`), output path (`:608`).
3. Wait default: the engine's own default is `wait = false` (`:609`). The CLI sets `wait: true` for long commands unless the caller passes an explicit `false` (`apps/filmcraft-cli/src/main.rs:199-208`).
4. Job: `spawn_export` (`export_tools.rs:500-525`) checks that an encoder exists (`:502-503`), wires the sink for in-memory exports (`:505-508`), creates the directory (`:509-511`), writes the sidecar (`:512`), and creates a job in `s.jobs` (`:513-514`). The provider uses the full-resolution media (`:517`). Stepped WASM exports run between frames (`:520-525`).
5. Result: `{job, warnings, path, files, …}` (`export_tools.rs:613-614`). With `wait: false`, it includes `poll: jobs.list` and `cancel: jobs.cancel` (`:618-623`).

**Command-line wait.** `Backend::Local` injects `wait: true` for `LONG_COMMANDS` (`apps/filmcraft-cli/src/main.rs:199-208`), and `masks.track` is added explicitly (`:200-201`). The comment says a background export would never be written once the process exits (`:197-198`). Whether the CLI warns when an explicit `wait: false` reaches it was not traced.

### 3. Plugin, extension, script, resource mechanisms

- **Processing plugins: none found.** A whole-tree `git grep` for `wasmi|wasmtime|libloading|dlopen|plugin|extension_host` in `.rs` files returned only `crates/engine/src/settings/prefs.rs`, `crates/engine/src/settings/schema.rs`, and three `crates/platform` files that load system libraries.
- **Developer mode.** `PluginPrefs { developer_mode: bool }` (`crates/engine/src/settings/prefs.rs:427-428`). The settings panel says "No plugins are installed. Changes will take effect the next time you start FilmCraft." (`crates/engine/src/settings/schema.rs:534-538`).
- **System libraries.** `libloading::Library::new("libnvcuvid.so.1")` (`crates/platform/src/nvdec/cuvid.rs:41`) and `"libcuda.so.1"` (`crates/platform/src/nvenc/device.rs:102`). These are GPU driver libraries loaded by name. They are not user plugins.
- **In-process factories.** Encoder, audio encoder, frame renderer, decoder and reader registries (`crates/export/src/registry.rs:78-160`; `crates/codecs/src/lib.rs:100`, `:174`). The `platform` crate registers hardware encoders and decoders (`crates/platform/src/lib.rs:72-101`); the timing of those calls was not traced.

### 4. Contribution families

| Family | Seam today | Evidence |
|---|---|---|
| Processing (effects, filters) | NO-SEAM | not found in inspected files |
| Export formats (encoders) | In-process only | `crates/export/src/registry.rs:94-99` |
| Audio encoders | In-process only | `registry.rs:159` |
| Frame renderers (GPU) | In-process only | `registry.rs:106-134`; `apps/filmcraft/src/main.rs:445`; `apps/filmcraft-cli/src/main.rs:278` |
| Decoders and readers | In-process only | `crates/codecs/src/lib.rs:100`, `:174` |
| Commands, panels, menus | NO-SEAM. The table is static. | `commands.rs:49-52` |

### 5. Transactions, undo, jobs, persistence, output

- **Export is a job.** `spawn_export` puts the export in `s.jobs` (`export_tools.rs:513-525`). `jobs.list` and `jobs.cancel` follow it (`:618-623`).
- **MCP long calls.** `LONG_COMMANDS` is `["file.exportMedia"]` (`crates/automation/src/long_job.rs:18-19`). With `wait: true`, the server reports progress and, on cancel, deletes the partial output (`long_job.rs:1-5`).
- **Sidecar.** `write_sidecar` runs before the job starts (`export_tools.rs:512`).
- **Undo.** Export is not an edit. Its undo behaviour was not traced.

### 6. Collision and core-override behaviour

| Rule | Status | Evidence |
|---|---|---|
| Export factory registered later | NOT-ENFORCED. The newest factory is tried first, and the first `Some` wins, so it can preempt a built-in encoder. | `crates/export/src/registry.rs:94-99` (inserts at index 0); `:156` (`filter_map(...).next()`) |
| Same factory registered twice | ENFORCED-IN-CODE (ignored) | `registry.rs:96` |
| Command ID duplicates | NOT-FOUND-IN-INSPECTED-FILES. `find` is first match. | `commands.rs:54-56` |
| Plugin registration path | NO-SEAM | not found in inspected files |

Not reachable by user plugins in the inspected code. A plugin bridge to these registries would need a precedence rule first.

### 7. GUI and CLI parity

- **Shared table.** The CLI and the app use the same `command_specs()` (`commands.rs:49-52`), and MCP `command_run` calls `run` (`crates/automation/src/server.rs:353-355`).
- **Fresh CLI.** There is no plugin runtime to load. The CLI session opens a demo project in some modes (`apps/filmcraft-cli/src/main.rs:190`).
- **Hidden dialogs.** Not traced for headless paths.

### 8. Caller authority

- **MCP.** `command_run` runs any registered command (`crates/automation/src/server.rs:353-355`). No confinement, roots, allowlist or permission check was found in `crates/automation/src` or `apps/filmcraft-cli/src` (`git grep`).
- **Desktop control.** Requires `FILMCRAFT_CONTROL_TOKEN`, a 64-hex credential. Without one, the listener stays closed (`apps/filmcraft/src/control_server.rs:7-8`, `:33-34`, `:66-68`). Every request must carry the session credential (`:7`).

### 9. Reusable APIs and missing hooks

- **Reusable.** `command_specs()`, `find` (`commands.rs:49-56`); `spawn_export` (`export_tools.rs:500`); `long_job::is_long` (`crates/automation/src/long_job.rs:25-27`); the export registries (`crates/export/src/registry.rs`).
- **Missing hooks (PROPOSED).** A processing contract (frame, time, audio, job). A plugin-export seam with a namespace and a precedence rule. A caller or authority object for MCP. None of these exists.

### 10. App-specific data constraints

- **Frame renderer contract.** Premultiplied, linear-light RGBA, the same size and format as the CPU reference (`crates/export/src/registry.rs:106-110`).
- **Audio encoder factory.** Takes format, sample rate, channel count and settings (`registry.rs:80`).
- **Time.** Timeline ticks through `filmcraft-time` (`Cargo.toml:15`). Frame rate through `FrameRate` (`registry.rs:78`). Details not traced.

### 11. Smallest useful slice and behavioural proof (PROPOSED, future work)

No slice for plugins. The FilmCraft design work should start from the export and job path. The smallest change in this area that does not add a plugin is a test for the CLI's `wait: false` case.
- **Proof (test to write):** a one-shot CLI `file.exportMedia` with an explicit `wait: false` must either refuse, or warn that the output may never be written (`apps/filmcraft-cli/src/main.rs:197-208`). Today it passes through unchanged (`:205`).

### 12. Evidence limits

Not traced: whether the CLI refuses or warns on `wait: false` beyond `main.rs:199-208`; the full MCP tool bodies (`server.rs`); the export writer and encoder selection (`export_tools.rs` beyond `:525`); undo behaviour of export; the web build (`apps/filmcraft-web`); the desktop control protocol in full; `masks.track`.

---

## LightCraft (`ac1799db38661e07a855653f86feb3e66081102e`)

Paths are relative to the app repo at this revision.

### 1. Repo, branch, revision, workspace

- Clone `/Users/ash/Projects/lightcraft`, branch `main`, HEAD `ac1799db38661e07a855653f86feb3e66081102e`, matching the baseline. Working tree clean at start.
- Workspace: `Cargo.toml:1-3` (`crates/*`, `apps/*`, `xtask`). 29 crates under `crates/` (`catalog`, `codecs`, `color`, `denoise-core`, `denoise`, `develop`, `engine`, `faces`, `fetch`, `geocode`, `geom`, `gpu`, `heif`, `inpaint`, `lensdb`, `mcp`, `merge`, `meta`, `pipeline`, `preview`, `raster`, `raw`, `scenes`, `segment`, `slideshow`, `sysmem`, `testkit`, `tiff`, `ui-egui`). Apps: `lightcraft`, `lightcraft-cli`, `lightcraft-web`.

### 2. Operation surface and a traced operation

**Registry.** `crates/engine/src/cmd.rs:54-71` (`CommandSpec`: ID, label, menu, shortcut, params, `enabled`, `run`, `journal`, `explicit_targets`), `:130-137` (`cmd!` macro with a `query` variant), `:140-160` (`command_specs()` extends from each module, for example `edit`, `library`, `develop`, `color`, `curves`, `masks`, `organize`, `keywords`, `manage`). Registration sites: 373 in non-test `cmd.rs` and `cmd/` files. Distinct literal IDs on the same line: at least 169. Both counts are lower bounds, because some registrations span lines.

**MCP.** One tool per command, named `cmd_<id>`, plus hand-written helpers (`crates/mcp/src/tools.rs:1-3`). The server forwards each call to a `Backend`: `Remote` (the desktop control channel) or `Headless` (an in-process session) (`crates/mcp/src/lib.rs:6-12`).

**CLI.** `run`, `mcp`, `render`, `snapshot`, `commands`, `merge`, `synth-merge`, `controls`, `denoise-serve`, `calibrate`, `version` (`apps/lightcraft-cli/src/main.rs:168-182`).

**Traced operation: `develop.adjust`.**
1. Definition: `crates/engine/src/cmd/develop.rs:308` (`cmd!("develop.adjust", "Nudge Develop Value", [], None, "{control, delta}", has_active, …)`).
2. Validation: the control name must be known (`controls::find`, `develop.rs:309-310`). `delta` is required and numeric (`:311`). The active photo is required (`:312`).
3. Edit: the closure reads the photo's source info and crop dimensions (`:313`), then calls `edit(s, "develop.adjust", spec.label, …)` (`:314`). The closure normalises white balance when needed (`:316-323`), sets the control to the old value plus `delta` (`:324-325`), and refits the crop angle if needed (`:326`).
4. Helper: `edit` (`develop.rs:16-22`) copies the photo's current settings, runs the closure, and calls `s.set_develop(id, d, label)` (`:20`).
5. Session: `Session::set_develop` (`crates/engine/src/lib.rs:929-944`) constrains the crop (`fit_constrained_crop`, `:933`, `:952-963`), builds a `develop` op, adds auto-sync ops, and calls `commit` (`:941-944`).
6. Commit: `Session::commit` (`lib.rs:634-647`) applies the op through `apply_through` (`:636`, `:657-680`), which forgets cached data for affected photos. It records the forward op in `pending_log` (`:637`) and the inverse op on the undo stack (`:638`). The undo stack holds at most 1,000 entries (`:640-643`).
7. Result: `ok()` (`develop.rs:21`).

### 3. Plugin, extension, script, resource mechanisms

- **Code plugins: none found.** A targeted `git grep` for `wasmi|wasmtime|libloading|dlopen|plugin|PluginHost|register_command|register_plugin` in non-test `.rs` files returned no matches.
- **Resource importers.** These are real and go through commands:
  - `preset.import` (`crates/engine/src/cmd/preset_files.rs:117`), with `paths` (`:16`).
  - `profile.import` (`crates/engine/src/cmd/lut_profiles.rs:143`) and `profile.deleteImported` (`crates/engine/src/cmd/lut_profiles.rs:151`).
  - `curve.importPresets` (`crates/engine/src/cmd/curves.rs:100`).
  - `mask.importEffects` (`crates/engine/src/cmd/mask_effects.rs:660`).
  - `library.importLightroom` (`crates/engine/src/cmd/library/specs.rs:1308`).
  - `keyword.import`, `print.template.import`, `slideshow.templates.import` (names from a `git grep` over `crates/engine/src`).
- **Slideshow templates** ship as JSON in the engine (`crates/engine/src/slideshow_templates/*.json`).

### 4. Contribution families

| Family | Seam today | Evidence |
|---|---|---|
| Code (commands, effects, panels) | NO-SEAM | `cmd.rs:140-160` (static lists) |
| Presets, profiles, curves, mask effects, keywords, print and slideshow templates | Import commands, data files | `preset_files.rs:117`; `lut_profiles.rs:143`; `curves.rs:100`; `mask_effects.rs:660` |
| Export formats | NO-SEAM found | not found in inspected files |

### 5. Transactions, undo, jobs, persistence, output

- **Transactions.** Each command becomes a catalog `Op`, with an inverse on the undo stack (`crates/engine/src/lib.rs:634-647`). Undo depth 1,000 (`:640-643`).
- **Persistence.** The catalog journals operations (`crates/catalog/src/journal.rs:153` describes flushing, `sync_all` and an atomic rename of `catalog.snap`). `safe_file.rs` writes through a temporary file with `sync_all` and then `rename` (`crates/catalog/src/safe_file.rs:49`, `:176`, `:231-241`). `drain_log` hands the pending ops to persistence (`lib.rs:912`).
- **Sidecars.** Automatic XMP sidecar writes (`lib.rs:273-280`); failed writes are retried later (`unsaved_sidecars`, `lib.rs:278-280`).
- **Output and jobs.** The CLI `render` dispatch is at `apps/lightcraft-cli/src/main.rs:171`. Its wait behaviour and the export pipeline (`crates/engine/src/export/*.rs`) were not traced.

### 6. Collision and core-override behaviour

- **Code.** NO-SEAM: no registration function exists in inspected files. Built-in command IDs are static.
- **Resource IDs.** Collisions between imported presets or profiles and built-ins were not traced.
- **Duplicate command IDs.** Not found in inspected files: no uniqueness check was seen in `command_specs()` (`cmd.rs:140-160`).

### 7. GUI and CLI parity

- **Shared table.** The MCP server and the desktop control channel use the same command names (`crates/mcp/src/lib.rs:6-12`). The `Headless` backend answers the control-channel names itself (`lib.rs:9-11`).
- **Fresh CLI.** Nothing to load. The CLI has no plugin step.
- **Hidden dialogs.** Not traced.

### 8. Caller authority

- **Control channel.** Loopback (`apps/lightcraft/src/control_server.rs:28-35`), with no token or authentication in the non-test code (a `git grep` for `token|auth|secret|password` matched only an unrelated identifier).
- **MCP.** No confinement, allowlist or permission check was found in `crates/mcp/src` (`git grep`). Remote and headless backends both forward every command.
- **Consequence.** A local process that can connect to the control port has the desktop's authority.

### 9. Reusable APIs and missing hooks

- **Reusable.** `Session::commit` and `Op` (`crates/engine/src/lib.rs:634`; `crates/catalog/src/lib.rs:78`); `set_develop` (`lib.rs:929`); the develop controls (`controls::find`, `controls::set`, `cmd/develop.rs:310`, `:324-325`); the command table (`cmd.rs:54-71`); the MCP backend abstraction (`crates/mcp/src/lib.rs:6-12`).
- **Missing hooks (PROPOSED).** A code-plugin seam, which does not exist. A provenance record for imported resources, so an import can be traced and removed. A caller identity on the control channel.

### 10. App-specific data constraints

- **Pixels.** `Image<T>` is interleaved, row-major, with `width`, `height` and `data: Vec<T>` (`crates/raster/src/lib.rs:50-56`). The precision used in each pipeline stage was not traced.
- **Crop.** "Constrain Crop" keeps the crop inside the warped photo (`crates/engine/src/lib.rs:947-963`).
- **Undo.** 1,000 steps (`lib.rs:640`).

### 11. Smallest useful slice and behavioural proof (PROPOSED, future work)

No code-plugin slice. If LightCraft ever hosts a contribution, the first candidate is a namespaced resource pack through `preset.import` (`crates/engine/src/cmd/preset_files.rs:117`).
- **Proof (test to write):** importing the same pack twice must not duplicate or replace an existing preset silently. Importing a pack whose preset name matches a built-in must be reported.

### 12. Evidence limits

Not traced: the CLI `render` implementation (`apps/lightcraft-cli/src/main.rs:171` is dispatch only); the export pipeline (`crates/engine/src/export/*.rs`); `DevelopSettings` and the sidecar format; `preset_import.rs` and `preset_luminar.rs` parsing; the `lightcraft-mcp` headless engine; `model_download.rs` (a network download path in the engine, not examined); the denoise models; the `lightcraft-web` build. Only the `develop` family and resource-import IDs were traced; the remaining registry families were not.

---

## ArtCraft (`1f969a9224934a55417daf10359a6fb1458b63f4`)

Paths are relative to the app repo at this revision.

### 1. Repo, branch, revision, workspace

- Clone `/Users/ash/Projects/artcraft`, branch `main`, HEAD `1f969a9224934a55417daf10359a6fb1458b63f4`, matching the baseline. Working tree clean at start.
- Workspace: `Cargo.toml:5-44` lists 39 members, including `artcraft-native` (`:6`), API client crates (`crates/api_clients/*`), `crates/desktop/artcraft` (the Tauri app), `crates/lib/*`, `crates/schema/*`, and a vendored `crates/vendor/tauri-plugin-http`. Default member: `crates/desktop/artcraft` (`Cargo.toml:46`).
- `artcraft-native` describes itself as a pure-Rust egui app with "no Tauri, no webview" (`artcraft-native/Cargo.toml:3`; `artcraft-native/src/main.rs:3`). Its dependencies include `wry = "0.55"` (`artcraft-native/Cargo.toml:50`), used for the Midjourney sign-in window (`artcraft-native/src/overlays/providers.rs:2`, `:13-14`).

### 2. Operation surface and a traced operation

**egui app.** No command registry. Operations are UI actions (pages: `create_image`, `create_video`, `create_audio`, others; `artcraft-native/src/pages/*`) and the provider submission path. The module list is in the `ls-tree` of `artcraft-native/src`.

**Tauri app.** `tauri::generate_handler!` with 63 entries (`crates/desktop/artcraft/src/lib.rs:227-291`), including `enqueue_image_bg_removal_command`, `enqueue_image_to_3d_object_command`, `enqueue_image_to_gaussian_command`, `download_media_file_command`, and `estimate_image_cost_command` (`lib.rs:228-236`). Command bodies were not traced.

**Traced operation: image generation through the native app.**
1. Entry: `submit(host, creds, accounts, provider, fields, image_urls)` (`artcraft-native/src/backend/providers/submit.rs:55-60`), which dispatches on `Provider`. `Provider` is a closed enum: `Artcraft`, `Midjourney`, `Fal` (`artcraft-native/src/backend/providers/mod.rs:22-27`). `Provider::for_model` maps model IDs to providers (`mod.rs:51`).
2. Fal path: `submit_fal` (`submit.rs:93-112`). It checks the stored Fal key (`:94`) and builds the router request (`:95`). A prompt record is created when credentials exist (`:97-100`). The builder is validated by `build2()` (`:101-105`). The client is `RouterFalClient::new_polling_only_from_raw_key` (`:106`), and the request is sent (`:107`).
3. Request builder: `request_builder` (`submit.rs:117-126`) parses the model (unknown models are refused, `:118-123`), and the prompt, resolution, aspect ratio, quality and batch count from the page fields (`:125`). Midjourney aborts on a mode mismatch; the other providers generate anyway (`:124`).
4. Job: `new_job` (`submit.rs:128-130`) creates a `ProviderJob` with a local ID, a batch token and the provider's job ID.
5. Persistence: `JobStore::put` (`artcraft-native/src/backend/providers/jobs.rs:71-76`) calls `mutate`, which saves the whole list (`:82-90`). `save` writes atomically (`:93-96`).
6. Worker: a tick loop (`artcraft-native/src/backend/providers/worker.rs:94-98`) polls Fal (`:124`) and Midjourney (`:151`), downloads results, and uploads them.

### 3. Plugin, extension, script, resource mechanisms

- **None found.** `git grep` for `wasmi|wasmtime|libloading|dlopen|plugin_host|extension|lua|javascript|boa_engine|rhai` in `artcraft-native` and `crates/desktop/artcraft` matched file-name and file-extension wording. Checked: `artcraft-native/src/app.rs:947`; `worker.rs:212`, `:310-311`. Other matches (`artcraft-native/README.md`, `pages/common.rs`, `prompt_box/refs.rs`, and several Tauri `commands/` and `state/` files) were listed but their lines were not read.
- **Model metadata.** Defaults and constants in `artcraft-native/src/models.rs:9-19`: default image, video and audio model IDs, prompt limit of 1,000 characters (`:13`), audio references for `suno_remix` and `suno_sample` (`:17`). Whether the catalogue is fetched or built in was not traced.
- **Provider and router crates.** `artcraft_router` and the provider client crates (`Cargo.toml:5-44`).

### 4. Contribution families

| Family | Seam today | Evidence |
|---|---|---|
| Generation providers | NO-SEAM. The provider set is a closed enum, and dispatch is a `match`. | `providers/mod.rs:22-27`; `submit.rs:55-60` |
| Models | Data (server or built-in; not traced) | `models.rs:9-23` |
| Themes, panels, tools | NO-SEAM found | not found in inspected files |
| Downloads and media | Fixed paths | `artcraft-native/src/backend/media_cache.rs:78-109`; `crates/desktop/artcraft/src/core/commands/download/*` |

### 5. Transactions, undo, jobs, persistence, output

- **Jobs.** `ProviderJob` records are kept in `<data_root>/state/<file>` (`artcraft-native/src/backend/providers/jobs.rs:60-63`). A job is added on submit and updated by the worker (`:71-76`).
- **Unreadable state.** A state file that fails to parse is ignored and the in-memory list starts empty (`jobs.rs:62`). The next save writes that list (`:82-90`). This means a corrupt file is overwritten on the next change. This is an inference from the code, not a run.
- **Atomic writes.** `save` writes atomically, "so a crash can't truncate the list" (`jobs.rs:93-96`).
- **Undo.** None. Generation is a remote job.
- **Output.** Results are downloaded into the media cache and saved with an image extension taken from the URL (`worker.rs:310-311`; `app.rs:947`). The cache location was not traced.
- **Storage.** The native app has no SQL dependency (`artcraft-native/Cargo.toml`, no `sqlite`, `sqlx` or `rusqlite`). The Tauri crate has `crates/schema/database/sqlite_tasks`; its use was not traced.

### 6. Collision and core-override behaviour

NO-SEAM for all contributions. The provider enum is closed (`providers/mod.rs:22-27`), and no registration function was found.

### 7. GUI and CLI parity

- **No CLI.** No `clap` or CLI entry point was found in `artcraft-native/src` or `crates/desktop/artcraft/src` (`git grep`).
- **No automation.** The only TCP listeners found in these crates are in test code (`crates/desktop/artcraft/src/core/utils/auto_download.rs:138`, `:152`, `:235`, `:293`).
- **Tauri shell.** The 63 Tauri handlers are registered with `generate_handler!` (`crates/desktop/artcraft/src/lib.rs:227-291`). They are normally called from the Tauri webview, which is inferred and not traced here. This is not a plugin seam.

### 8. Caller authority

No caller identity model exists: there is no listener and no CLI.

Credentials: the native app reads the Tauri app's session cookie and keeps its own copy in `~/Artcraft/credentials/native_session.json` (`artcraft-native/README.md`; `artcraft-native/src/backend/session.rs:17`, `:34`). No file-permission setting was found (`git grep` for `set_permissions` and `0o600` in `artcraft-native/src` returned nothing). The file mode was not checked at runtime.

### 9. Reusable APIs and missing hooks

- **Reusable.** `submit` and `request_builder` (`submit.rs:55-130`); `JobStore` (`jobs.rs:54-96`); `Provider` (`providers/mod.rs:22-68`); the router builder (`artcraft_router`, used at `submit.rs:95`); `MediaCache` (`media_cache.rs:78-109`).
- **Missing hooks (PROPOSED).** A provider adapter trait with declared capabilities per model, replacing the closed enum and `match`. A credential-scoping rule for provider keys. A way to register a model or provider from outside the crate. None exists.

### 10. App-specific data constraints

- **Inputs.** Images are URLs (`image_urls: Vec<String>`, `submit.rs:55`). The default prompt limit is 1,000 characters (`models.rs:13`, `DEFAULT_PROMPT_MAX`); per-model limits were not traced.
- **Batch.** Midjourney direct jobs produce four images and take no references (`providers/mod.rs:15-17`).
- **Audio.** Some audio models require an audio reference (`models.rs:17`).
- **No pixel pipeline.** Images are fetched as bytes and shown through an egui texture cache (`media_cache.rs:78-109`).

### 11. Smallest useful slice and behavioural proof (PROPOSED, future work)

No plugin slice. A provider adapter trait is design work that should follow a decision on credentials and consent.
- **Proof (test to write, after a design decision):** a second provider adapter, registered through the trait, must not change the `Artcraft`, `Midjourney` or `Fal` results for the same fields. This needs the trait first.

### 12. Evidence limits

Not traced: the body of each Tauri command (63 entries); the Midjourney sign-in and session flow; the media-cache location; `auto_download`; whether the model catalogue is fetched; the Tauri app's `sqlite_tasks` use; the `frontend/` tree (not read); the `services/storyteller` command modules (only `AGENTS.md` line 9 was checked); the credential file's actual mode at runtime.

---

## Evidence limits: files read per app

Every path is relative to the app repository at the revision named in its section. "grep" means a `git grep` or `git show … | grep` with the cited lines only. "awk" means a line range was printed.

**EffectCraft @ `ce9eaec`:** `Cargo.toml` (1–60); `crates/plugin/src/lib.rs` (grep; 14–63); `crates/plugin/src/wasm.rs` (1–20; 43–50; 118–165; grep); `crates/plugin/Cargo.toml` (grep 12–20); `crates/plugin/src/tests.rs` (grep); `crates/effects/src/plugin.rs` (grep; 37–50; 190–240; 370–400; 390–432; 465–492; 674–693; 275–300 grep); `crates/effects/src/lib.rs` (grep; 645–655; 745–815; 822–826); `crates/engine/src/cmd.rs` (grep; 52–75; 101–200; 278–286); `crates/engine/src/cmd/effect.rs` (1276–1280); `crates/engine/src/session/dispatch.rs` (1–70); `crates/engine/src/session/edit.rs` (1–60; 277–320); `crates/engine/src/cmd/effect.rs` (30–80; 249–272; 273–312; 1366–1380); `crates/engine/src/system/library.rs` (grep; 1–45 header; 185–196; 456–475; 596–612; 754–760; 957–980; 1040–1066); `crates/engine/src/render_queue.rs` (460–482); `crates/engine/src/menus.rs` (grep); `crates/engine/src/lib.rs` (grep); `crates/render/src/pipeline.rs` (615–645); `crates/project/src/props.rs` (175–192); `crates/project/src/lib.rs` (grep); `crates/host/src/lib.rs` (grep; 88–130); `crates/helper-host/src/lib.rs` (44–160; 170–200); `crates/helper-host/src/client.rs` (grep; 380–402); `crates/helper-host/src/protocol.rs` (22–31); `crates/helper-host/src/blocklist.rs` (14–18); `crates/automation/src/confine.rs` (1–130); `crates/automation/src/backend.rs` (60–100); `crates/automation/src/tools.rs` (grep; 104–135; 202–206; 557–604); `crates/automation/src/bridge.rs` (grep); `crates/ui-egui/src/control.rs` (180–240); `apps/effectcraft/src/main.rs` (148–170; 268–292); `apps/effectcraft/src/control_server.rs` (1–100; 175–195); `apps/effectcraft-cli/src/main.rs` (grep; 469–482; 569–632; 763–790; 801–811; 905–968); `crates/host/tests/it/plugins.rs` and `library.rs` (grep). Plus `docs/plugin-functional-inventory.json` and `docs/plugins/research/haiku-app-cli-state.md` (context only).

**PhotoCraft @ `655295b`:** `Cargo.toml` (1–60); `crates/plugins/src/lib.rs` (1–80); `crates/plugins/src/registry.rs` (1–140); `crates/plugins/src/runtime.rs` (grep; 1–60; 63–66; 86–120; 126–200; 235–246; 250–280); `crates/plugins/src/manifest.rs` (grep; 9–31; 108–170; 407–413); `crates/plugins/src/apply.rs` (1–80; 160–215); `crates/plugins/tests/abi.rs` (grep for function names; 68–79; 310; 325); `crates/plugins/examples` (ls-tree only); `crates/engine/src/cmd/plugin_cmds.rs` (1–165; 186–333); `crates/engine/src/cmd/plugin_cmds/tests.rs` (grep; 130); `crates/engine/src/cmd.rs` (173–240; 1190–1245; 1285–1300; grep 1213, 1235); `crates/engine/src/prefs.rs` (887–906; 1357–1380; grep callers); `crates/engine/src/session/documents.rs` (14–26); `crates/engine/src/session/execute.rs` (200–232); `crates/engine/src/jobs.rs` (543–580; 824–860); `crates/engine/src/cmd/adjust_cmds.rs` (grep 335); `crates/engine/src/adjust_params.rs` (grep); `crates/ui-egui/src/menus/plugin_ui.rs` (1–50); `crates/ui-egui/src/prefs/prefs_ui/startup.rs` (grep 44); `apps/photocraft/src/services.rs` (grep 680–711); `apps/photocraft/src/main.rs` (grep 400); `crates/automation/src/headless.rs` (1–60; 370–392); `crates/automation/src/workspace.rs` (320–500); `crates/automation/src/security.rs` (1–200); `crates/automation/src/rpc.rs` (36–100); `crates/automation/tests/it/smart_filter_authorization.rs` (1–60); `apps/photocraft-cli/src/args.rs` (30–75); `apps/photocraft-cli/src/document.rs` (75–160 grep); `apps/photocraft-cli/src/automation.rs` (grep 15, 37); `apps/photocraft-cli/src/main.rs` (1–12); `examples/plugins/invert-rs/src/lib.rs` (1–55; grep); `examples/plugins/invert-rs` (ls-tree).

**FilmCraft @ `32749e3`:** `Cargo.toml` (grep 1–60); `crates/engine/src/settings/prefs.rs` (grep 424–430); `crates/engine/src/settings/schema.rs` (grep 530–540); `crates/engine/src/commands.rs` (35–75; grep 1959); `crates/engine/src/export_tools.rs` (500–525; 590–625; grep 599); `crates/export/src/registry.rs` (78–160; grep); `crates/codecs/src/lib.rs` (grep 100, 174); `crates/codecs/src/gop.rs` (grep 374); `crates/platform/src/lib.rs` (grep 72–101); `crates/platform/src/nvdec/cuvid.rs` (grep 37–41); `crates/platform/src/nvenc/device.rs` (grep 93–102); `apps/filmcraft-cli/src/main.rs` (190–215; grep 278); `apps/filmcraft/src/main.rs` (grep 445); `apps/filmcraft-web/src/webcodecs.rs` (grep 189); `crates/automation/src/long_job.rs` (1–30); `crates/automation/src/server.rs` (grep tool names; 353–355); `apps/filmcraft/src/control_server.rs` (grep 1–70); `apps/filmcraft/src/args.rs` (grep 12); whole-tree `git grep` for `wasmi|wasmtime|libloading|dlopen|plugin|extension_host|developer_mode`.

**LightCraft @ `ac1799d`:** `Cargo.toml` (grep 1–60); `crates/engine/src/lib.rs` (grep; 630–682; 905–945; 952–963); `crates/engine/src/cmd.rs` (grep; 50–75; 128–160); `crates/engine/src/cmd/develop.rs` (16–30; 300–330; grep); `crates/engine/src/cmd/preset_files.rs` (grep 16, 117); `crates/engine/src/cmd/lut_profiles.rs` (grep 91, 143); `crates/engine/src/cmd/curves.rs` (grep 100, 304); `crates/engine/src/cmd/library/specs.rs` (grep 1308); `crates/engine/src/cmd/mask_effects.rs` (grep 660); `crates/engine/src/cmd/mask_effect_files.rs` (grep 447); `crates/mcp/src/lib.rs` (1–30); `crates/mcp/src/tools.rs` (1–40); `crates/mcp/src/server.rs` (grep); `apps/lightcraft-cli/src/main.rs` (grep; 160–182); `apps/lightcraft/src/control_server.rs` (grep); `crates/catalog/src/journal.rs` (grep 153); `crates/catalog/src/safe_file.rs` (grep 49, 176, 231–241); `crates/catalog/Cargo.toml` (grep); `crates/raster/src/lib.rs` (44–56); `docs/plugin-functional-inventory.json` (280–303; SHA-256 recomputation of the two recorded LightCraft files).

**ArtCraft @ `1f969a9`:** `Cargo.toml` (5–46; grep); `artcraft-native/Cargo.toml` (grep 1–50); `artcraft-native/README.md` (1–25); `artcraft-native/src/main.rs` (grep 3); `artcraft-native/src/app.rs` (grep 947); `artcraft-native/src/models.rs` (9–23); `artcraft-native/src/backend/providers/mod.rs` (1–30; 51–70); `artcraft-native/src/backend/providers/submit.rs` (55–60; 93–131; 134–138); `artcraft-native/src/backend/providers/jobs.rs` (18–96); `artcraft-native/src/backend/providers/worker.rs` (grep; 39–44; 94–124; 310–311); `artcraft-native/src/backend/media_cache.rs` (grep; 78–109); `artcraft-native/src/backend/session.rs` (grep; 1–50); `artcraft-native/src/overlays/providers.rs` (grep; 1–14); `crates/desktop/artcraft/src/lib.rs` (grep 157; 227–291); `crates/desktop/artcraft/src/core/utils/auto_download.rs` (grep line numbers); `crates/desktop/artcraft/src/services/storyteller/commands/AGENTS.md` (grep 9); `artcraft-native/src` and `crates/desktop/artcraft/src` (ls-tree).

**Shared context (not app source):** `docs/plugins/readiness/brief.md` (read); `docs/plugins/research/haiku-app-cli-state.md` (read); `docs/plugin-functional-inventory.json` (grep and lines 280–303).

**Not read by design:** every working tree; `ATTRIBUTION.md`; `scripts/` in EffectCraft; `frontend/` in ArtCraft; `docs/` in every app; test bodies, except where a name or line was cited.

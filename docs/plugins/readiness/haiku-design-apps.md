# Design app readiness audit

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: 2026-11-08 · Haiku 5.5 worker B · source-only. Committed revisions were read with `git -C <clone> show|ls-tree|grep <REV>`; no working tree was read, and nothing was built, tested, linted, fetched, launched, or contacted over the network. No agents were started. This file is the only file written.

Scope: FigCraft, VectorCraft, PDFCraft, DesignCraft, DeckCraft. Citations are `path:line` at the revision of that section. Classifications: ENFORCED-IN-CODE, NOT-ENFORCED (replaces or shadows), NO-SEAM (no registration path), NOT-FOUND-IN-INSPECTED-FILES (this means only that the files read did not show it, not that it is absent). PROPOSED marks anything not present in source. Counts from `grep -c` are approximate and not de-duplicated unless stated.

## Revisions

| App | Clone | Branch | HEAD (`git rev-parse HEAD`) | Baseline | Match |
|---|---|---|---|---|---|
| FigCraft | /Users/ash/Projects/figcraft | main | `0e86f3597a7443b15c93ec3ab2091202f8802c95` | same | yes |
| VectorCraft | /Users/ash/Projects/vectorcraft | main | `065a646799f1f20f5454845d6648e09a5252c452` | same | yes |
| PDFCraft | /Users/ash/Projects/pdfcraft | main | `14ae839a28902a17b9c6adba68d820eb922544c4` | same | yes |
| DesignCraft | /Users/ash/Projects/designcraft | main | `1256decbeb4a49c6fbe94400efc1e2a838de23ce` | same | yes |
| DeckCraft | /Users/ash/Projects/deckcraft | main | `0c00f2240352b17cba571e93573af48e017a058d` | same | yes |

Shared libraries (`storytold/craft-libs`, pinned per app): VectorCraft and DesignCraft at `92d392cff132eaf53e13afff77f38b5c11b73aa0`; PDFCraft at `787a7a0f972fd1d7a7618fdbeca3a5267a0407ff`; FigCraft at `e11f9d9aebea5f6b8e74643c1ea1ed172ba5f545` (most crates) and `c5a1cbfb5f9049f9270955caffb5bee5997aebf7` (`craft-update`); DeckCraft at `c5a1cbfb5f9049f9270955caffb5bee5997aebf7`. Pinned trees were read from the doc repo's object store with `git show` and `ls-tree`, only where cited. None of the pinned trees contains a plugin, extension or command-registry crate.

## Summary

| App | Runtime / plugin mechanism | Operation registry | CLI loads installed plugins? | Collision rule status | Smallest slice (PROPOSED unless noted) |
|---|---|---|---|---|---|
| FigCraft | WebAssembly on wasmi, sandbox ABI 4 (14 host imports), manifest `api` 6, metered (fuel, memory, call and log caps) | `spec!` table, 336 literal ids (`crates/engine/src/cmd.rs:73-135`); plugin commands are not registry entries | No, by source: the CLI session starts from `Prefs::default()` (`crates/engine/src/session/new.rs:17`); only the GUI loads saved prefs (`apps/figcraft/src/main.rs:209`) | Built-ins cannot be shadowed by plugin commands (ENFORCED, by construction). Re-installing a plugin id silently replaces it (NOT-ENFORCED). Duplicate built-in ids only log (`cmd.rs:129`). No plugin namespace check | Re-check the module digest at run time; refuse replacing an id across sources without confirmation |
| VectorCraft | WebAssembly on wasmi, ABI 1, **no host imports** (`crates/plugins/src/runtime.rs:170-172`); kinds: filter, live effect, exporter | `cmd!` lists, 888 `cmd!(` lines (`crates/engine/src/cmd.rs:224-240`); process-wide plugin registry (`crates/plugins/src/registry.rs:14`) | Yes: `with_stores` attaches the Plug-ins folder for headless constructors (`apps/vectorcraft-cli/src/main.rs:208-212`, used at 255, 262, 540) | Built-in formats resolve first by id, then by extension (ENFORCED, `crates/engine/src/cmd/fileio/formats.rs:707-717`). Effect ids are `plugin.`-prefixed (ENFORCED by routing, `crates/engine/src/cmd/params.rs:254`). Same-id install replaces by default (NOT-ENFORCED, `crates/engine/src/cmd/plugin.rs:292`). Two exporters on one extension: the first by plugin id wins (NOT-ENFORCED) | Refuse a second exporter claiming a claimed extension; make `replace` opt-in; move heavy-budget consent out of a parameter |
| PDFCraft | None for extensions. Embedded Acrobat JavaScript runs in boa (`crates/js/src/lib.rs:1-4`), which is document scripting, not an extension runtime | `COMMANDS` const table, 210 `c(`/`ct(` entries (`crates/engine/src/commands.rs:165`); automation `tools()`, 155 `t("…")` entries (`crates/automation/src/tools.rs:120`) | N/A. CLI `run` and `mcp` drive the same automation tool table (`apps/pdfcraft-cli/src/main.rs:5`) | NO-SEAM for contributions. Built-in command ids are unique by test (`crates/engine/src/commands.rs:467-473`) | None needed for plugins now. Add an id-namespace test (PROPOSED) |
| DesignCraft | None found in inspected files | `cmd!` lists, about 547 lines (`crates/engine/src/cmd.rs:157-170`); `find_command` is a first-match scan (`cmd.rs:241-243`) | N/A. CLI reads the engine registry and `Session::new()` (`apps/designcraft-cli/src/main.rs:110-112`) | NO-SEAM. First registered entry wins a duplicate id. No id-uniqueness test found | Runtime key check plus a duplicate-id test (PROPOSED) |
| DeckCraft | None found in inspected files | `cmd!` lists, about 329 lines (`crates/engine/src/cmd.rs:148-161`); `find_command` first-match (`cmd.rs:197-199`) | N/A. CLI lists `command_specs()` (`apps/deckcraft-cli/src/main.rs:76`) | NO-SEAM. Export formats are a closed match (`crates/engine/src/cmd/file.rs:343-348`). No id-uniqueness test found | Same as DesignCraft |

## Cross-app blockers for this group

1. **Only FigCraft and VectorCraft have any runtime extension mechanism.** PDFCraft, DesignCraft and DeckCraft register commands at compile time (PDFCraft `commands.rs:165`; DeckCraft `cmd.rs:161`; DesignCraft `cmd.rs:170`) and fix their formats in code: DeckCraft's export is a closed `match` (`cmd/file.rs:343-348`) with a constant writable list (`cmd/file.rs:432`); DesignCraft's export commands are one static list (`cmd/export/registry.rs:5-7`); PDFCraft's image export is an enum (`crates/engine/src/export.rs:361-369`). Impact: an SDK needs a new contribution seam per app. Installation alone cannot add a menu, command, or format to these three apps. FigCraft's menus come from `craft-menu`, whose model the app builds with no registration API (`crates/menu/src/model.rs:31-281` at `e11f9d9`; `crates/menu/README.md` line 3), so plugin menu placement is NO-SEAM at the shared layer as well.

2. **No caller identity at operation dispatch in any of the five apps. Policy lives only in plugin hosts.** FigCraft applies its denylist only inside `EngineHost::call` (`crates/engine/src/extensions/plugins.rs:235-245`). The control channel's `engine.execute` calls `execute_as` directly (`crates/ui-egui/src/control.rs:1425-1430`), so it bypasses that denylist, and the socket has no handshake (`apps/figcraft/src/control_server.rs:96-115`). VectorCraft's MCP `engine.execute`, `ui.menu.invoke` and `command` all reach `Session::execute` for any id (`crates/mcp/src/headless.rs:342-349`), so any MCP or CLI caller can reach `plugin.install` and `plugin.remove`. FigCraft's `agent` is the MCP client's self-declared name (`crates/mcp/src/server.rs:250`), used only for comment authorship (`crates/engine/src/session/exec.rs:58-68`). PDFCraft is the only app with a transport credential: a random token that each control connection must present (`crates/ui-egui/src/control.rs:3-5`), plus separate read and write roots for automation (`crates/automation/src/file_access.rs:1-6`). DesignCraft and DeckCraft: no caller identity found.

3. **Install replaces silently, and an id is not bound to a publisher.** FigCraft `register` matches on the manifest id only, returns `replaced: true`, and deletes that plugin's settings (`plugins.rs:462-476`, settings removed at 475; `remove` also at 491). The directory server's publish checks only that the version is newer for that id (`apps/figcraft-server/src/directory.rs:215-226`). A plugin from a second server with the same id replaces an installed one (`crates/engine/src/extensions/plugin_directory.rs:141`) because nothing compares `source`. VectorCraft's `install` defaults `replace` to true (`cmd/plugin.rs:292`), and `attach_plugin_store` discards the return value of `registry::install` (`crates/engine/src/store.rs:235`), so duplicate files in the folder collapse silently.

4. **FigCraft checks a module's digest once, at install. Disk modules are then re-read unchecked on every run.** The directory SHA-256 is checked at install (`plugin_directory.rs:118-124`). The bundle is written to the app-owned install folder under a hash-named directory (`plugin_directory.rs:153-192`), and `register` drops the cached module when a path exists (`plugins.rs:466-467`). `module_of` then reads the file again on each run (`plugins.rs:377-386`; `module_from_disk` at 388-397). The folder is app-owned, but nothing re-verifies the bytes. VectorCraft holds the loaded module in the registry; no digest or hash appears in `crates/plugins/src` or `cmd/plugin.rs` (grep).

5. **Headless callers can have a plugin's file accepted and then dropped (FigCraft).** `save_file` returns success (0, "accepted") to the plugin after queuing a `dialog.pluginSave` request (`plugins.rs:315-333`; the request is at 332). The headless backend clears `ui_requests` after each call (`crates/mcp/src/headless.rs:68`, `:125`), and the CLI clears them per batch step (`apps/figcraft-cli/src/files.rs:433`). Only the GUI consumes the request (`crates/ui-egui/src/app/commands.rs:275` → `dialogs/plugins_ui.rs:118`). Source-only; nothing was run. VectorCraft handles this differently: headless reports dialogs as data (`crates/mcp/src/headless.rs:170`). DesignCraft (`cmd/file/place.rs:25`) and DeckCraft (`cmd/design/colors.rs:10,138`) push dialog requests, and no headless handling was found in the MCP files inspected.

6. **Consent gates are caller-supplied booleans.** VectorCraft installs a heavy-budget filter only with `allowHeavy: true` (`cmd/plugin.rs:296-300`), a parameter any caller can pass. FigCraft's directory install needs `confirm: true` (`plugin_directory.rs`, confirm check near 101-106; module doc at 5-8 says the app passes it only from its confirmation step), which the control channel can also pass. The `plugin.install` spec says the plugin is installed "for this session" (`cmd/plugin.rs:444`), yet the code writes it to the shared Plug-ins folder (`cmd/plugin.rs:307`, `store.rs:244-248`) and the function's own comment says it is kept for the next start (`cmd/plugin.rs:287-288`). The spec and the behaviour disagree.

7. **Manifest permissions are declarations, with gaps (FigCraft).** `permissions.commands` accepts `"*"`: the bare entry is the empty string, which passes the check at `manifest.rs:354-358`, and `allows` treats an empty prefix as a match for every command (`manifest.rs:104-109`). `access` defaults to read (`manifest.rs:116-121`), and read-only is enforced twice: by the history flag (`plugins.rs:176-178`) and by a document-pointer check after the call (`plugins.rs:250-252`). `PluginCommand` has no shortcut or menu fields (`manifest.rs:124-131`), which is good. Unknown manifest keys are not rejected (no `deny_unknown_fields` in `crates/plugin/src` or `crates/engine/src/extensions`), so a `shortcut` key would be dropped silently.

8. **The shared keymap defaults to stealing chords (craft-libs at `92d392cf`, used by VectorCraft and DesignCraft; PDFCraft pins `787a7a0f`, which also depends on craft-keymap).** `Policy` defaults to `Steal` (`crates/keymap/src/keymap.rs:334-337`), which removes the chord from other commands when one is assigned. A contributed command bound through this default would take a built-in shortcut. Any extension keymap path must use `Refuse`, or lock the built-in set (PROPOSED).

---

## FigCraft (`0e86f3597a7443b15c93ec3ab2091202f8802c95`)

### 1. Repo, branch, revision, workspace
- Path `/Users/ash/Projects/figcraft`; branch `main`; HEAD matches the baseline. 1459 tracked files.
- Root `Cargo.toml` lists 17 crates under `crates/`: collab-net, collab, color, doc, engine, format, geom, layout, mcp, pathops, pdf, plugin, render, svg, text, ui-egui, vectordrawable. Apps: figcraft, figcraft-cli, figcraft-server, figcraft-web. Also `plugins/sdk`, four examples under `plugins/examples/`, and `xtask`.
- craft-libs pins (root `Cargo.toml`, lines 39-57): `craft-docpath`, `craft-window`, `craft-font-store`, `craft-font-catalog`, `craft-ui`, `craft-menu`, `craft-settings` at `e11f9d9`; `craft-update` at `c5a1cbf`. These are shared UI, font, menu and document-path crates. None is a plugin or command crate.

### 2. Operation surface
- Registry: `crates/engine/src/cmd.rs:73-135` builds one static list from about 50 `specs()` modules (`list.extend(...)` per module). `CommandSpec` (`cmd.rs:36-44`) has id, label, params doc, a `history` flag and a run fn. A duplicate id logs a warning (`cmd.rs:129`); the index keeps the later entry.
- Size: 336 literal `spec!("…")` ids in `crates/engine/src` (grep, unique). Top prefixes by count: view 42, text 40, tool 25, vector 24, object 21, select 19, edit 12, pref 10, variable 8, page 8, arrange 8, proto 7, file 7, component 7; plugin 3, plugins 2, widget 1. Categories: query (`*.list`, `*.get`, `inspect`), edit (most of the rest), file (`file.*`, `doc.*`, `version.*`), render and export (`export`, `file.export`), UI-only (`view.*`, `window.*`, `app.*`).
- Traced `node.set` end to end: `crates/engine/src/cmd/edit.rs:307` registers it with `history = true`. Entry is `Session::execute` (`crates/engine/src/session/exec.rs:71-76`: `find_command`, then `refuse_unknown_params` at 73, then `board::refuse_on_board`, then `guard::run`). The handler `set` (`crates/engine/src/cmd/edit/props.rs:95-111`) resolves targets, rejects empty props, and checks each key with `variable_prop` (`crates/engine/src/cmd/props.rs:392`). It applies per layer with `props::apply` (`cmd/props.rs:1131`), which clamps values (opacity to 0..1 as f32, `cmd/props.rs:569-571`). Results go through instance-structure, preview, live-access and finite-geometry checks (`exec.rs`, `check_*` calls around 97-110), then `normalize_edit`, `after_command`, and the journal (exec.rs ~111-140). A failure restores the document (`exec.rs` error branch).

### 3. Existing plugin mechanism
- Runtime: wasmi interpreter, no WASI (`crates/plugin/src/sandbox.rs:1-4`; `crates/plugin/Cargo.toml`). Sandbox ABI 4 (`sandbox.rs:5-6`): 14 imports from module `figcraft` (`sandbox.rs:49-63`): `call`, `selection`, `reply_len`, `reply_read`, `log`, `notify`, `set_output`, `ui_show`, `ui_close`, `setting_get`, `setting_set`, `setting_delete`, `widget_render`, `save_file`. The manifest API is a separate number, `API_VERSION = 6` (`crates/plugin/src/manifest.rs:11`); the two must not be conflated.
- Module contract: exports `memory` and `figcraft_run(i32)->i32`; any import outside `figcraft` is rejected at check time (`sandbox.rs:278-300`).
- Limits (`sandbox.rs:68-100`): fuel 1,000,000,000 per run (`sandbox.rs:92`), 10,000 fuel per host call (`:93`), 64 MiB linear memory (`:94`), 16 MiB module, 100,000 host calls, 16 MiB per message, 1000 log lines of 8 KiB and 1 MiB total. `plugin.run {fuel}` can only lower the budget (`plugins.rs:403-409`).
- Manifest (`figcraft-plugin.json`; `manifest.rs:33-59`): id (letters, digits, `.`, `-`, `_`; `manifest.rs:453-456`), name, version, description, author, `main`, `api`, `permissions` (`access` read|write, `commands` allowlist, `saveFile`), `commands` (each id, label, params; `manifest.rs:124-131`), `widget`.
- Loading: desktop reads the manifest from disk (`plugins.rs:351-362`); browser takes manifest plus base64 module (`plugins.rs:430-454`); directory installs a SHA-256-checked bundle (`plugin_directory.rs:97-146`) with `confirm: true`.
- Contribution families present: menu commands (`plugin.run`, Plugins menu from prefs, `catalog.rs:245-247`); panels (`ui_show`; `open_panel` at `plugins.rs:88`, `ui` at `:566`); per-plugin settings, JSON, 64 KB cap (`plugins.rs:22-24`, `set_setting` at 283-313); widgets, plugin-owned frames with a `figcraft.widget` data prefix (`crates/engine/src/extensions/widgets.rs:1-12`, `:30`); property menus and editable fields (API 4); `save_file` (API 4, `plugins.rs:315-333`).
- Not found, or closed: import and export formats (`ExportFormat` is a closed enum, `crates/doc/src/props.rs:1256`; `file.export` hard-codes `fig` and `penpot`, `crates/engine/src/cmd/file/export.rs:78-84`); shortcuts (keyed by built-in id, `catalog.rs:314`); file types; resources such as brushes, LUTs, swatches or themes (NOT-FOUND-IN-INSPECTED-FILES).

### 4. Transactions, undo, jobs, persistence, output
- One undo step per plugin run: `call_plugin` wraps the run in `Session::grouped` (`plugins.rs:529-552`; `crates/engine/src/session/undo.rs:256-258`). Commands the plugin runs inside join the same group (`plugins.rs:242-244`). A failed run restores the group-start state (`undo.rs` checkpoint, ~260-300). Panel callbacks are also one step (`plugins.rs:576`).
- Document references: a widget frame keeps its plugin id and state as layer `plugin_data` (`widgets.rs:1-12`; `crates/doc/src/node.rs:543-545`). Native files are zip archives of JSON (`crates/format/src/lib.rs:536-571`). No plugin version or digest is saved in the document (NOT-FOUND-IN-INSPECTED-FILES). The `.fig` export drops `pluginData` with a warning (`crates/format/src/fig/export/layer.rs:309-310`).
- Jobs: no progress or cancel model for plugin runs was found; runs are synchronous and bounded by fuel and memory. CLI output: `file.export` writes synchronously and returns the path or bytes (`cmd/file/export.rs`, body not traced in full). The CLI waits because the call returns.

### 5. Collision and core override
| Rule | Status | Evidence |
|---|---|---|
| Plugin command id equal to a built-in command id | ENFORCED-IN-CODE (by construction): plugin commands are not registered; they travel as text to the plugin via `plugin.run` (`plugins.rs:502-523`) | `plugins.rs:235`, `plugins.rs:502` |
| Plugin id equal to a built-in id | Separate namespace: plugins live in `prefs.plugins` (`plugins.rs:424-428`), not in the command registry | `plugins.rs:424` |
| Plugin id equal to an installed plugin's id | NOT-ENFORCED (replaces): `register` matches on id, sets `replaced`, and deletes settings | `plugins.rs:462-476` |
| Duplicate built-in command id | NOT-ENFORCED: warns, then the index keeps the later entry | `cmd.rs:126-130` |
| Shortcut or menu placement by a plugin | NO-SEAM: no such manifest field; built-in shortcuts come from `catalog.rs:314` | `manifest.rs:124-131` |
| Default handler, file type, default exporter | NO-SEAM | not found in inspected files |
| Plugin namespace rule | NOT-FOUND-IN-INSPECTED-FILES beyond the id character set | `manifest.rs:453-456` |
| Plugin calling `file.*`, `doc.*`, `app.*`, `plugin.*`, `edit.undo`, `command.batch`, `collab.*`, `window.*` | ENFORCED for plugin callers (denylist), NOT applied to `engine.execute` | `plugins.rs:116-137` vs `control.rs:1425-1430` |

Also: unknown manifest keys are ignored, and the `*` allowlist entry is accepted (cross-app item 7).

### 6. GUI and CLI parity
- Same registry: the CLI lists `all_commands()` (`apps/figcraft-cli/src/main.rs:107`); MCP and the control channel call `Session::execute` or `execute_as` (`crates/mcp/src/headless.rs:57-70`, `crates/ui-egui/src/control.rs:1425-1430`).
- A fresh CLI process does not load installed plugins, by source: `Session::new` sets `Prefs::default()` and an empty plugin runtime (`session/new.rs:17`, `:29`); only the GUI assigns saved prefs (`apps/figcraft/src/main.rs:209`) and the plugin folder (`:219`). No prefs load was found in `apps/figcraft-cli/src`. Runtime not checked.
- Hidden dialogs: `request_ui` queues (`undo.rs:184-186`). Headless clears the queue (`headless.rs:68`, `:125`). For plugin runs, a command that opens a dialog fails the run (`plugins.rs:245-249`, "it opens a dialog or panel, which plugins can't").

### 7. Caller authority
- `Session::execute(id, params)` has no caller parameter (`exec.rs:71`). `execute_as(agent: Option<&str>, …)` only tags comment authorship (`exec.rs:58-68`). The agent name is the MCP client's name (`crates/mcp/src/server.rs:250`), self-declared.
- The only authority check is the plugin host's policy (`plugins.rs:148-181`: denylist, allowlist, read-only). It does not apply to the control channel's `engine.execute` (`ui-egui/src/control.rs:1425-1430`), which takes any command.
- Control socket: binds 127.0.0.1 and queues each JSON line for the UI thread (`apps/figcraft/src/control_server.rs:1-2`, `:96-115`). No handshake, token, or caller check was found (NOT-FOUND-IN-INSPECTED-FILES for any auth).
- Directory install needs `confirm: true` (`plugin_directory.rs`, ~101-106), a caller-supplied boolean.

### 8. Reusable APIs and missing hooks
- Reusable now: `find_command` and `all_commands` (`cmd.rs:138-146`), `Session::execute` (`exec.rs:71`), `Session::grouped` (`undo.rs:256`), `plugins::run` and `call_plugin` (`plugins.rs:502`, `:529`), `figcraft_plugin::{check_module, run, Instance, Limits}` (`sandbox.rs:278-352`), the manifest validator (`manifest.rs:333`).
- Missing (PROPOSED): a caller-identity argument on dispatch (for example `Caller::{User, Cli, Automation(name), Plugin(id, grants)}`); a digest field in `InstalledPlugin` checked in `module_of`; a source-aware replacement policy in `register`; a headless save-destination rule so `save_file` fails when no UI consumes it; a namespaced contribution registry for menus and shortcuts (PROPOSED, not in source).

### 9. App-specific data constraints
- Opacity is f32 clamped to 0..1 (`cmd/props.rs:569-571`). Numeric props clamp to ranges (for example `miterLimit` 1–100 at `cmd/props.rs:620`; corner smoothing 0–1 at `:670`). Non-finite geometry is refused (`exec.rs`, `check_finite_geometry`).
- Plugin settings 64 KB total (`plugins.rs:22-24`); module 16 MiB; log and message caps (`sandbox.rs:68-100`).
- Geometry units and text positions were not traced.

### 10. Smallest useful slice and proof (PROPOSED, future work)
- Slice A (integrity): store the expected SHA-256 in `InstalledPlugin`, re-hash the bytes in `module_of` before instantiation, and refuse a run on mismatch. Proof: install from a directory, change the module file in the install folder, run the plugin, and assert refusal. An engine test file `plugin_directory_tests.rs` already exists in the tree (`ls-tree`), so it is the natural place.
- Slice B (replacement): refuse an install whose id is held by a different `source` unless the caller passes an explicit replace flag. Proof: two directory sources, same id, second install refused; same source updates succeed.
- Slice C (caller boundary): refuse `plugin.install`, `plugin.remove`, `plugin.directory.install` and `plugin.directory.publish` on the control channel unless the caller is the user session. Proof: a control-channel `engine.execute` of `plugin.install` returns an error.
- Slice D (headless save): make `save_file` return an error when no UI consumer is attached. Proof: headless `plugin.run` of a `saveFile` plugin reports refusal, not acceptance.

### 11. Evidence limits (FigCraft)
- Not traced: the widget callback path beyond `widgets.rs` head; `ui-egui` plugin dialogs (`dialogs/plugins_ui.rs` only at lines 118, 150, 329); the directory server beyond publish; `cmd/file/export.rs` body; the native load path; sharing and background jobs (`crates/engine/src/sharing/background`); `plugins/examples` contents beyond listing.
- Not run: nothing built or executed. The older inventory (`docs/plugin-functional-inventory.json`) listed these entry sources, and the citations above confirm them at this revision.

---

## VectorCraft (`065a646799f1f20f5454845d6648e09a5252c452`)

### 1. Repo, branch, revision, workspace
- Path `/Users/ash/Projects/vectorcraft`; branch `main`; HEAD matches the baseline.
- Root `Cargo.toml` depends on `craft-*` crates from `storytold/craft-libs` at `92d392cf` (`Cargo.toml:14-21`). Crates under `crates/` (24 plus `plugins/example`): affinity, brush, cad, color, doc, effects, engine, eps, format, geom, mcp, metafile, pathops, pdf, plugins, render, scriptcheck, svg, testkit, text, three-d, tools, trace, ui-egui. Apps: vectorcraft, vectorcraft-cli, vectorcraft-web. Also `xtask`.

### 2. Operation surface
- Registry: `command_specs()` (`crates/engine/src/cmd.rs:234-240`) assembles about 110 `specs()` modules. The `cmd!` macro (`cmd.rs:224-232`) has a `journal` flag (undoable) and a `query` variant (not journalled). 888 `cmd!(` lines in `crates/engine/src` (grep count; not de-duplicated).
- Alias table: `ALIASES` (`cmd.rs:676-678`) has one entry (`object.convertDocumentColorMode`). Aliases do not reach plugins.
- Traced `plugin.run` (filter) end to end: spec at `crates/engine/src/cmd/plugin.rs:431`. `FilterRun::prepare` (`cmd/plugin.rs:104-130`) gets the plugin, checks it is a filter (`:107-109`), resolves parameters against the manifest, selects objects (`objects::filter_targets`), and builds `_context` (bounds, artboard, `documentData`). `objects::encode_input` (`crates/plugins/src/objects.rs:154-157`) serialises them. `FilterRun::run` (`cmd/plugin.rs:166-168`) runs the module with no document access and can run off the main thread. `finish` (`cmd/plugin.rs:179-196`) decodes the output and applies it inside `s.edit` (`:183`). `objects::apply_output` (`objects.rs:677-697`) accepts only ids of its own inputs (or new objects with a path) and refuses other ids. It stores `documentData` under the plugin id (`cmd/plugin.rs:185`). The whole run is one undo step.
- Size: about 888 `cmd!` invocations; not categorised in full.

### 3. Existing plugin mechanism
- Runtime: wasmi (`crates/plugins/src/lib.rs:4`), ABI version 1 (`crates/plugins/src/runtime.rs:12`). Modules get no host imports; any import is rejected (`runtime.rs:170-172`). Each call uses a fresh instance (`lib.rs:1-20` doc). Fuel is metered in slices with a wall-clock deadline on native builds (`lib.rs:1-20` doc; `runtime.rs:14-19`).
- Limits (`runtime.rs:65-75`): module 32 MiB (`:67`), memory 512 MiB (`:68`), output 64 MiB (`:69`), base fuel 50,000,000 (`:70`), fuel per input byte 4,000 (`:71`). Heavy filters only (`manifest.rs:261-268`): 1,000,000,000 fuel and 300 s wall time (`runtime.rs:28-31`).
- Manifest (`crates/plugins/src/manifest.rs:194-215`): id `[A-Za-z0-9._-]{1,64}` (`:197-198`, `:250`, `valid_id` at `:430`); `kind` filter | effect | exporter (`:203`); `budget` (light | heavy; heavy only for filters, `:261-268`); exporter `extensions` (1–MAX, lower-case, unique; `:285-300`); parameters with validation and dialog groups (`crates/plugins/src/dialog.rs`).
- Loading: desktop path with `file_access::check_read` (`cmd/plugin.rs:318-323`); the plugin folder (`crates/engine/src/store.rs:221-239`); base64 via `dataBase64` (`cmd/plugin.rs:267-287`); web builds take base64 only (`:326-328`).
- Digest: none found in `crates/plugins/src` or `cmd/plugin.rs` (grep for sha, digest, hash returns only file-name and effect-cache hashes).
- Grants: none beyond the heavy-budget consent (see cross-app item 6). Plugins receive no host functions.
- Contribution families present: object filters (`plugin.run`); live effects with effect ids `plugin.<id>` (`crates/plugins/src/effect.rs:25`, `:38-48`; `cmd/params.rs:254`); exporters with format ids `plugin.<id>` (`crates/plugins/src/export.rs:38-55`; `crates/engine/src/cmd/fileio/formats.rs:719-740`); parameter dialogs.
- Not found: commands, tools, panels, themes, resource importers, jobs (NOT-FOUND-IN-INSPECTED-FILES).

### 4. Transactions, undo, jobs, persistence, output
- Filter output is one `s.edit` step (`cmd/plugin.rs:179-196`). Live effects are per-object records admitted by `admit_effects` (`cmd/plugin.rs:222-250`), which checks parameters and the single-3D-effect rule.
- Cancel: runs inside a `vectorcraft_doc::cancel` scope stop between slices (`cmd/plugin.rs:166-168` doc); `Error::Cancelled` maps to `EngineError::Cancelled` (`cmd/plugin.rs:26-28`).
- Persistence: `documentData` lives in the document under the plugin id (`cmd/plugin.rs:184-185`). Effects reference `plugin.<id>` strings only; no version or digest is recorded (NOT-FOUND-IN-INSPECTED-FILES). Installed modules are files in the Plug-ins folder, named by a stem of the id (`store.rs:244-248`, `:251-254`).
- Output: `document.export` writes the file synchronously or returns bytes (`cmd/fileio/encode.rs:446-450` dispatches to `plugin_file`). CLI and MCP calls return when the write is done.

### 5. Collision and core override
| Rule | Status | Evidence |
|---|---|---|
| Plugin exporter claims a built-in extension (for example `svg`) | ENFORCED-IN-CODE for extension lookup: built-in id, then built-in extension, then plugin id, then plugin extension. An exporter reaches its format only as `plugin.<id>` | `crates/engine/src/cmd/fileio/formats.rs:707-717`; doc `crates/plugins/src/export.rs:1-12` |
| Two exporters claim the same extension | NOT-ENFORCED: `plugin_formats` iterates installed exporters sorted by id (`export.rs:48-50`; registry is a `BTreeMap`, `registry.rs:14`), and `format()` takes the first match (`formats.rs:716-717`). No check at install (`cmd/plugin.rs:289-308`) | as shown |
| Effect id equal to a built-in effect id | ENFORCED by routing: plugin effects must carry the `plugin.` prefix (`params.rs:254`; `effect.rs:38-45`) | as shown |
| Same plugin id installed again | NOT-ENFORCED (replaces): `registry::install` overwrites (`registry.rs:18-24`); `install` defaults `replace: true` (`cmd/plugin.rs:292`) | as shown |
| Folder duplicates on attach | NOT-ENFORCED: the return of `registry::install` is ignored (`store.rs:235`) | as shown |
| Duplicate built-in command id | ENFORCED in tests only: `every_command_has_unique_id_and_doc` (`crates/engine/src/tests.rs:3112`) | as shown |
| Shortcut or menu placement by a plugin | NO-SEAM: manifest has no such fields (`manifest.rs:197-235`); `cmd!` carries them for built-ins (`cmd.rs:224-232`) | as shown |
| Namespacing | ENFORCED by prefix (`crates/plugins/src/lib.rs:42`, `PREFIX = "plugin."`) for effects and formats only | as shown |

### 6. GUI and CLI parity
- Same engine path: headless `exec` runs `check_keys` or `check_params` (`crates/mcp/src/headless.rs:129-134`), then `Session::execute` (`:140-143`). `Session::execute` itself calls `check_params` (`crates/engine/src/session/execute.rs:41`), so the unknown-key rule is enforced for every caller that reaches the engine.
- Fresh CLI process: loads installed plugins (`with_stores` in `apps/vectorcraft-cli/src/main.rs:208-212`, called from the headless constructors at 255, 262, 540; call sites not traced one by one). `prefs_dir::attach_plugin_store` is the same file as the GUI's (`apps/vectorcraft/src/prefs_dir.rs:46-50`, included by `#[path]` at `main.rs:46-47`).
- Hidden dialogs: `UiRequest::Dialog` is returned to the caller as data (`headless.rs:164-171`), not dropped. Plugins cannot open dialogs because they have no host calls.

### 7. Caller authority
- `Session::execute` takes no caller identity; MCP and CLI callers all reach it (`headless.rs:140-143`, `:342-349`). No allowlist for `plugin.install` or `plugin.remove` was found.
- Only consent gate for heavy filters: `allowHeavy` (`cmd/plugin.rs:296-300`), a caller-supplied boolean.
- Plugin path loads are confined when file-access roots are set (`cmd/plugin.rs:319` calls `check_read`); the root configuration was not traced. MCP transport authentication: not traced.
- Plugins themselves hold no caller authority, because they cannot call anything (no host imports, `runtime.rs:170-172`). This is the strongest sandbox boundary among the five apps.

### 8. Reusable APIs and missing hooks
- Reusable now: `registry::{install, get, list, remove, revision}` (`registry.rs:18-47`); `export::{installed, by_format, format_id, plugin_id}` (`export.rs:38-55`); `effect::{installed, effect_id, plugin_id}` (`effect.rs:38-48`); `objects::{filter_targets, encode_input, decode_output, apply_output}` (`objects.rs:154-157`, `:677`); `Session::edit` (`cmd/plugin.rs:183`); `formats::{format, writable_format}` (`formats.rs:707-815`); `Plugin::load` (`runtime.rs:155`); `Manifest::resolve_params` (`manifest.rs:337`).
- Missing (PROPOSED): an install-policy function in `registry.rs` (extension and id collision, source comparison); caller identity on `Session::execute`; a UI consent request replacing `allowHeavy`; a digest recorded alongside `documentData`; any contribution kind for commands, panels or themes (none in source).

### 9. App-specific data constraints
- Points: `check_point` rejects non-finite values (`objects.rs:165`). Gradient midpoint clamped to 0.13–0.87 (`objects.rs:228`); gradient angle normalised to 0–360 (`objects.rs:233`). Paint is a solid colour with a tint (`objects.rs:216`). `documentData` and effect-id sizes are capped (`objects.rs:31`, `:33`). Exporters receive the native JSON with images of `INLINE_MAX` bytes or more stripped of their data (`crates/plugins/src/export.rs:5-7`).

### 10. Smallest useful slice and proof (PROPOSED, future work)
- Slice: at install, refuse a second exporter that claims an extension already claimed by an installed exporter; refuse same-id replacement from a different `source` unless `replace: true`; replace `allowHeavy` with a UI consent request, refused in headless mode unless the user session approves. Proof: engine test that installs two exporters with extension `xyz` and asserts the second is refused; installs the same id from a different `source` and asserts refusal without `replace`; asserts a heavy filter install without consent fails. The test should live beside the existing plugin tests.

### 11. Evidence limits (VectorCraft)
- Not traced: `runtime.rs` call loop and slice accounting; `crates/plugins/src/dialog.rs`; `effect.rs` render path; `objects.rs` decode details; `cmd/fileio/encode.rs` `plugin_file` body; the GUI plugin UI; `plugin.reload`; MCP transport authentication; the plugin tests (grep only).
- Not run: nothing built or executed.

---

## PDFCraft (`14ae839a28902a17b9c6adba68d820eb922544c4`)

### 1. Repo, branch, revision, workspace
- Path `/Users/ash/Projects/pdfcraft`; branch `main`; HEAD matches the baseline.
- Root `Cargo.toml` depends on craft-libs at `787a7a0f` (`craft-ui`, `craft-quit`, `craft-menu`, `craft-keymap`, `craft-print`, `craft-docpath`; `Cargo.toml:16-25`). Crates under `crates/` (33): a11y, annot, audit, automation, compare, content, cos, create, crypt, edit, engine, export, filters, fonts, forms, geom, js, measure, model, ocr, optimize, oracle, organize, platform, preflight, print, redact, render, sign, testkit, ui-egui, xfa, xfdf. Vendored under `vendor/` (11): egui-wgpu, hayro (and its six sub-crates), lopdf, vello_common, vello_cpu, winit. Apps: pdfcraft, pdfcraft-cli, pdfcraft-web.

### 2. Operation surface
- Two surfaces. GUI commands: `COMMANDS` (`crates/engine/src/commands.rs:165`), 210 `c(`/`ct(` entries, each with a label, menu, shortcut, `needs` predicate and icon (`commands.rs:120-130`). Automation tools: `tools()` (`crates/automation/src/tools.rs:120`), 155 `t("…")` entries, each with a JSON schema, `read_only`, `destructive`, and an optional registry `command` link (`tools.rs:1-6` doc; `ToolDef` at `tools.rs:11`).
- Traced `page_rotate` end to end: `Automation::call` (`crates/automation/src/dispatch.rs:46-53`) finds the tool, runs `check_args` (`tools.rs:1516-1530`: refuses unknown arguments, checks required ones), and matches the name (`dispatch.rs:55`). The `page_rotate` arm (`dispatch.rs:75-78`) requires degrees to be a multiple of 90, resolves the document and pages, then calls `self.apply(&a, Edit::RotatePages {…})` (`dispatch.rs:102`). `documents.rs:443-452` forwards to `Session::apply` (`crates/engine/src/session/editing.rs:22`). That checks PDF encryption permissions (`check_permission`, `editing.rs:43`), applies the edit, and records the undo step; `undo` and `redo` are at `editing.rs:234` and `:263`.
- Edit enum: `crates/engine/src/edit.rs:20`, 99 top-level variants (awk count).
- Categories (from tool names): query (`text_*`, `doc_info`, `page_render`), edit (`page_rotate`, `doc_set_info`, `attachment_add`), file and batch (`doc_open`, `doc_save`, `doc_batch`), render and print (`printing`, `page_render`), signing (`sign_list`), forms and comments. Not fully counted.

### 3. Existing plugin or extension mechanism
- None. `crates/ui-egui/src/control/plugin.rs` is an egui input hook named after egui's plugin concept (`crates/ui-egui/src/app/frame.rs:74-81`), not an extension. `crates/js/src/lib.rs:1-4` runs the PDF's own Acrobat JavaScript in boa; the session has a `js_off` switch (`session/editing.rs`, read as a field). No loader, manifest, ABI, grants, or digest was found (NOT-FOUND-IN-INSPECTED-FILES).
- Automation file access: read and write roots set by CLI flags (`crates/automation/src/file_access.rs:1-6`).

### 4. Transactions, undo, jobs, persistence, output
- `Session::apply` makes one undo step per `Edit`; history scope comes from `scope_of` and `keys_after` (`session.rs:11` imports from `history.rs`; `MAX_UNDO`). Undo and redo at `editing.rs:234`, `:263`.
- Save: atomic writes through `write_atomic` and `write_atomic_with` (`crates/automation/src/paths.rs:151-160`); save and revert in `crates/engine/src/session/save.rs`.
- Jobs: `doc_batch` processes files one at a time with bounded discovery, as its schema text says (`tools.rs` doc for `doc_batch`). No progress or cancel model was found in automation (grep for "cancel" finds comments and status enums only).
- Output: the CLI and tools write to `out` or `out_dir` (tool schema), synchronously.

### 5. Collision and core override
- No contribution path exists, so there is nothing to collide (NO-SEAM). Built-in command ids are unique by test (`crates/engine/src/commands.rs:467-473`, which asserts unique ids and unique shortcuts). No uniqueness test for `TOOLS` names was found in the inspected files.
- `command` links on tools (`tools.rs:1-6`) are table-driven; a tool cannot point at an id that is not in the table.

### 6. GUI and CLI parity
- The `COMMANDS` doc says menus, shortcuts, palette and automation all go through the registry (`commands.rs:1-14`). Per-call-site use was not traced.
- The CLI's `run` and `mcp` drive the same automation tool table (`apps/pdfcraft-cli/src/main.rs:5`). Parity is partial: 155 tools against 210 commands, with the `command` field as the bridge.
- Headless dialogs: not traced.

### 7. Caller authority
- Automation confines reads and writes by root (`file_access.rs:1-6`; `Automation::resolve`). `ToolDef.read_only` and `destructive` (`tools.rs:11` onward) are descriptive metadata; no gate on them was found.
- Control channel: opt-in only, loopback, random port, and every connection must present the random token written to the control file (`crates/ui-egui/src/control.rs:3-5`). This is the only authenticated local transport among the five apps.

### 8. Reusable APIs and missing hooks
- Reusable now: `Session::apply` and `Edit` (`session/editing.rs:22`, `edit.rs:20`), `commands::COMMANDS` and `command()` (`commands.rs:165`, `:410`), `tools()`, `check_args` and `Automation::call` (`tools.rs:120`, `:1516`; `dispatch.rs:46`), `AutomationRoots` (`file_access.rs`), `write_atomic` (`paths.rs:151`).
- Missing (PROPOSED): a contribution registry for read-only tools only; caller identity on `Session::apply` and on automation; a gate that uses `destructive`.

### 9. App-specific data constraints
- PDF user space: `view_transform_to_user` converts view-space transforms to user space and rejects non-finite or area-zero matrices (`dispatch.rs:17-30`).
- Page rotation must be a multiple of 90 (`dispatch.rs:76-78`). PDF encryption permissions are checked on every edit (`editing.rs:43`).
- Image export formats: PNG, JPEG (quality 1–100), TIFF (`crates/engine/src/export.rs:361-369`).
- Fonts and OCR: embedded fonts (`crates/fonts`), OCR with installed models (`doc_batch` description).

### 10. Smallest useful slice and proof (PROPOSED, future work)
- No extension slice is needed for this group. If a read-only contribution seam is wanted later, start with a namespaced tool registry that refuses ids outside the table. Proof: a test that a contributed id fails `command()` and `tools::find`, and that duplicate tool names are refused (a `TOOLS` uniqueness test, not found today).

### 11. Evidence limits (PDFCraft)
- Not traced: `session/save.rs` and `history.rs` internals; `crates/ui-egui/src/control/plugin.rs` beyond its header; `pdfcraft-js` sandbox limits; whether GUI command handlers refuse unknown keys (tool-side check confirmed, GUI-side not traced); signing and OCR paths; the `ui-egui` menu wiring.
- Not run: nothing built or executed.

---

## DesignCraft (`1256decbeb4a49c6fbe94400efc1e2a838de23ce`)

### 1. Repo, branch, revision, workspace
- Path `/Users/ash/Projects/designcraft`; branch `main`; HEAD matches the baseline.
- Root `Cargo.toml` depends on craft-libs at `92d392cf` (`craft-pdf`, `craft-keymap`, `craft-prepress`, `craft-update`; `Cargo.toml:15-21`) and on its own crates (`designcraft-geom`, `designcraft-color`; `:23-24`). Crates under `crates/` (17): color, compose, doc, engine, epub, fonts, format, geom, idml, images, mcp, pdf, pptx, render, textimport, tools, ui-egui. Apps: designcraft, designcraft-cli, designcraft-web. Also `xtask`.

### 2. Operation surface
- Registry: `command_specs()` (`crates/engine/src/cmd.rs:170`) aggregates per-module `specs()`. `CommandSpec` (`cmd.rs:82-95`): id, label, menu path, shortcut string, params doc, `enabled`, run fn, `journal`, `undoable`. Macro variants (`cmd.rs:157-168`): default (journal and undoable), `query` (neither), `noundo` (journal only). About 547 `cmd!(` lines (grep count).
- Traced `object.group`: spec at `crates/engine/src/cmd/object.rs:155` (`cmd!("object.group", …, can_group, group)`). Entry is `Session::execute` (`crates/engine/src/lib.rs:597-606`: ends IME composition first, then `guarded`). `execute_unguarded` (`lib.rs:607`) finds the spec, runs `check_params` only under `#[cfg(test)]` (`lib.rs:611-612`), checks `enabled`, stops data preview, sets the undo nesting flag and calls `(spec.run)`. `group` (`crates/engine/src/cmd/object.rs:2060`) calls `targets`, then `s.edit(|d, sel| …)`: `group_plan`, remove children in reverse, allocate a group item, insert it in place. `Session::edit` (`lib.rs:799-810`) clones the document, runs the closure, runs `refit_groups` and `reroute`, swaps the `Arc`, bumps the revision. `record_undo_step` (`lib.rs:716`) records the step.
- Export is one static list: `crates/engine/src/cmd/export/registry.rs:5-7` (for example `file.exportPdf`, with its param doc at line 7). Other export and import commands are listed by `cmd!` in the same tree.
- Categories: edit, object, text, style, layer, page, book, file, export, design, UI.

### 3. Existing plugin or extension mechanism
- None found. Case-insensitive grep for plugin, wasmi, wasmtime, dylib, libloading, addon and extension registry matched nothing in `apps/` or `crates/`.
- Data-driven features that look extensible but are not: presets and templates (`file.savePreset`, `file.presets`, `file.deletePreset` in the command set), the theme gallery (`crates/engine/src/cmd/design.rs:46`, `design.themes`), theme colour and font dialogs (`crates/engine/src/cmd/design/colors.rs:10`, `:138`).
- Native format: zip with `mimetype` (`crates/format/src/lib.rs:1-3`, `:23`, `application/vnd.designcraft+zip`).

### 4. Transactions, undo, jobs, persistence, output
- Undo: `Session::edit` and `record_undo_step` (`lib.rs:799`, `:716`); interactions and compositions are settled in `execute` (`lib.rs:597-606`).
- Persistence: `.designcraft` zip (`format/src/lib.rs`). Export returns `{path, bytes, pages, warnings}` or `{base64, …}` (`cmd/export/registry.rs:7-9` doc).
- Jobs: none found. Output: synchronous.

### 5. Collision and core override
- `find_command` is a first-match scan (`cmd.rs:241-243`). A later duplicate id is unreachable and nothing warns (NOT-ENFORCED). No id-uniqueness test was found in the inspected files; the VectorCraft test pattern (`tests.rs:3112`) is absent here.
- Plugin contributions: NO-SEAM.
- Shortcut strings are literals inside each `cmd!` (`cmd.rs:157-168`). Collision checks on shortcuts were not traced.
- Export formats: one command per format (`cmd/export/registry.rs`); no registration seam.

### 6. GUI and CLI parity
- CLI: `Session::new()` and `s.commands()` list the engine registry (`apps/designcraft-cli/src/main.rs:110-112`, `:496-497`). The CLI pre-checks parameter keys (`apps/designcraft-cli/src/main.rs:409`, `check_params`).
- GUI: not traced (ui-egui not read beyond names).
- Headless UI requests: `UiRequest::Pick` (`cmd/file/place.rs:25`) and `UiRequest::View` (`cmd/layout/registry.rs:254`) are pushed on `ui_requests`. No headless clearing or handling was found in `crates/mcp/src` (inspected files only).

### 7. Caller authority
- None found. `Session::execute(id, params)` takes no caller (`lib.rs:597`). `agent` appears in no DesignCraft engine or backend file inspected (grep returned none).
- MCP `tools/call` reaches the engine through `engine.execute` (`crates/mcp/src/backend.rs:193`); resources are served from `server.rs:144-164`. Tool dispatch and transport authentication were not traced.

### 8. Reusable APIs and missing hooks
- Reusable now: `command_specs`, `find_command` (`cmd.rs:170`, `:241`), `Session::execute` and `edit` (`lib.rs:597`, `:799`), `record_undo_step` (`lib.rs:716`), the format crate (`crates/format/src/lib.rs`), `check_params` (`crates/engine/src/cmd/params.rs:224`, with `UNCHECKED` at `:215`).
- Missing (PROPOSED): a runtime key check in `execute_unguarded` (currently test-only at `lib.rs:611-612`); a duplicate-id guard over `command_specs()`; caller identity; any contribution registry.

### 9. App-specific data constraints
- Paths are anchor-based with absolute handle positions (`crates/geom/src/lib.rs:1-5`). The unit convention was not established in inspected lines.
- Colour, text layout and PDF/IDML/EPUB import internals were not traced.

### 10. Smallest useful slice and proof (PROPOSED, future work)
- Slice A: run the key check at runtime (remove the `#[cfg(test)]` gate at `lib.rs:611`), then check the doc-described keys of each command. Proof: `execute("object.group", {"bogus": 1})` returns an error outside tests. Risk: existing callers that pass undocumented keys would start failing; this needs a survey first.
- Slice B: a duplicate-id test over `command_specs()`, following VectorCraft's `every_command_has_unique_id_and_doc` pattern. Proof: the test fails when a duplicate is introduced.

### 11. Evidence limits (DesignCraft)
- Not traced: ui-egui (GUI command wiring); the MCP tool dispatch; `crates/compose` and text layout; IDML, EPUB, PPTX import internals; `designcraft-geom` units; the test suite (grep only).
- Not run: nothing built or executed.

---

## DeckCraft (`0c00f2240352b17cba571e93573af48e017a058d`)

### 1. Repo, branch, revision, workspace
- Path `/Users/ash/Projects/deckcraft`; branch `main`; HEAD matches the baseline.
- Root `Cargo.toml` depends on `deckcraft-*` path crates (`Cargo.toml:15-20`) and on craft-libs at `c5a1cbf` (`craft-media`, `craft-pdf`, `craft-update`, `craft-ui`; `Cargo.toml:62-66`). Crates under `crates/` (24): anim, av1, bitstream, color, engine, fonts, format, geom, h264, hevc, isobmff, matroska, mcp, media, metafile, model, ogg, opus, pdf, pptx, render, text, ui-egui, vp9. Apps: deckcraft, deckcraft-cli, deckcraft-web. Also `xtask`.

### 2. Operation surface
- Registry: `command_specs()` (`crates/engine/src/cmd.rs:161`) aggregates per-module specs (animation, arrange, artistic, background, compress, design, edit, file, file_info, find, interchange and more). `CommandSpec` (`cmd.rs:44-58`) carries a params doc parsed by `parse_params` (doc at `:50-54`), `enabled`, run fn, `journal`, `undoable`. About 329 `cmd!(` lines (grep count).
- Traced `file.export`: spec at `crates/engine/src/cmd/file.rs:40`. `export` (`cmd/file.rs:581-592`) requires `path` (`:582`), infers the format from it, then matches the format (`:590`). Formats: pptx, potx, ppsx (`:343`), outline and txt (`:344`), rtf (`:345`), pdf (`:346`), gif (`:347`), png (rejected as a single image, `:348`), and deckcraft, which is the native format. Writable extensions are a constant (`cmd/file.rs:432`) and checked at `:439`.
- Edit path: `Session::execute` (`crates/engine/src/lib.rs:623-628`) → `execute_unguarded` (`lib.rs:629`: find spec, end composition, check `enabled`) → `Session::edit` (`lib.rs:714-724`: clones the whole `Presentation`, runs the closure, `refit_groups`, `reroute`, swaps the `Arc`, bumps the revision).
- Categories: edit, arrange, animation, background, design, find, file and interchange, insert, UI.

### 3. Existing plugin or extension mechanism
- None found (case-insensitive grep). The only hit was `crates/ui-egui/src/ime_tests.rs`, which is an input-method test.
- Resource-like features are user data, not extensions: templates (`file.saveTemplate` at `cmd/file.rs:38`; `file.templates`), theme gallery (`design.themes` at `cmd/design.rs:46`), shape presets (`shape.presets` at `cmd/insert.rs:53`), theme colour and font dialogs (`cmd/design/colors.rs:10`, `:138`).

### 4. Transactions, undo, jobs, persistence, output
- Undo: `Session::edit` clone-and-swap (`lib.rs:714-724`) plus the `undoable` flag (`cmd.rs:58`). The undo-stack implementation was not traced.
- Persistence: native `.deckcraft` is writable (`cmd/file.rs:432`). Export writes to `path` and returns `{path, bytes}` (`cmd/file.rs:581-592`); there is no base64 return for `file.export` (path is required).
- Animation timing model (`crates/anim`), audio and video encode (`opus`, `ogg`, `h264`, `hevc`, `av1`, `vp9`, `matroska`): present in the workspace; timing and deadlines were not traced.
- Jobs: none found. Output: synchronous.

### 5. Collision and core override
- `find_command` is a first-match scan (`cmd.rs:197-199`). A later duplicate is unreachable, and nothing warns (NOT-ENFORCED). No id-uniqueness test was found in the inspected files.
- Export and import formats: closed `match` (`cmd/file.rs:343-348`) with a constant writable list (`:432`). NO-SEAM.
- Plugin contributions: NO-SEAM.

### 6. GUI and CLI parity
- CLI lists `deckcraft_engine::command_specs()` (`apps/deckcraft-cli/src/main.rs:76`) and creates `Session::new()` for each command path (`main.rs:203`, `:272`, `:316`, `:396`, `:452`). Same registry as the GUI, inferred from shared code; GUI wiring not traced.
- Dialog requests (`cmd/design/colors.rs:10`, `:138`) are pushed on `ui_requests`. No headless handling was found in the MCP files inspected.
- Runtime key checks: no call to `check_params` or `check_keys` was found in non-test engine or MCP code (grep).

### 7. Caller authority
- None found. `agent` appears only in an MCP test name (`crates/mcp/src/tests.rs:100`, `agent_builds_a_deck`).

### 8. Reusable APIs and missing hooks
- Reusable now: `command_specs`, `find_command` (`cmd.rs:161`, `:197`), `Session::execute` and `edit` (`lib.rs:623`, `:714`), export functions in `cmd/file.rs`, `deckcraft-geom` (`crates/geom/src/lib.rs:3-5`, `:18`), `deckcraft-format` and `deckcraft-pptx` (export).
- Missing (PROPOSED): the same as DesignCraft: runtime key checking, a duplicate-id guard, caller identity, and a contribution registry.

### 9. App-specific data constraints
- Positions are points as `f64`; Office Open XML EMU conversions are exact for integer values, 12,700 EMU per point (`crates/geom/src/lib.rs:3-5`, `:18`).
- Export option limits: `perPage` 1, 2, 3, 4, 6 or 9 (`cmd/file.rs:364-366`); GIF `secondsPerSlide` 0.1–600 and width presets (`cmd/file.rs:40` doc).
- Timeline and audio deadlines: not traced.

### 10. Smallest useful slice and proof (PROPOSED, future work)
- As for DesignCraft: a duplicate-id test over `command_specs()` (proof: fails when a duplicate is introduced), and a runtime key check (proof: an unknown key to `file.export` fails outside tests). Hold off on any contribution seam until the registry tests exist.

### 11. Evidence limits (DeckCraft)
- Not traced: `crates/anim`, `crates/media`, `crates/render`, the undo stack, ui-egui, the MCP tool dispatch, the test suite (grep only), audio and video encode paths, and the `deckcraft-cli` command bodies beyond `command_specs()`.
- Not run: nothing built or executed.

---

## Evidence limits (all apps)

Files read per app, all at the revision named in its section. Git commands used: `rev-parse`, `show`, `ls-tree`, `grep`. The working trees were not read. For ranges printed by `sed`, the line numbers are as printed, and the citations above use only those lines or lines reached by `grep -n`.

- **FigCraft**: `Cargo.toml` (grep); `crates/plugin/Cargo.toml`, `crates/plugin/src/lib.rs`, `sandbox.rs` (parts, grep), `manifest.rs` (parts, grep), `bundle.rs` (grep only); `crates/engine/src/extensions/plugins.rs` (parts), `plugin_directory.rs` (parts), `widgets.rs` (head, grep); `crates/engine/src/cmd.rs` (parts), `session/exec.rs` (parts), `session/undo.rs` (parts, grep), `session/new.rs` (grep), `prefs.rs` (grep), `catalog.rs` (grep), `cmd/edit.rs` (grep), `cmd/edit/props.rs` (parts), `cmd/props.rs` (grep and parts), `cmd/file/export.rs` (grep), `cmd/file/save.rs` (parts), `crates/doc/src/props.rs` (parts, grep), `crates/doc/src/node.rs` (grep), `crates/format/src/lib.rs` (grep), `crates/format/src/fig/export/layer.rs` (grep); `crates/mcp/src/headless.rs` (parts), `crates/mcp/src/server.rs` (grep), `crates/mcp/src/backend.rs` (grep); `crates/ui-egui/src/control.rs` (header, grep), `crates/ui-egui/src/app/commands.rs` (parts), `crates/ui-egui/src/dialogs/plugins_ui.rs` (grep); `apps/figcraft/src/control_server.rs` (header, grep), `apps/figcraft/src/main.rs` (parts), `apps/figcraft/src/services.rs` (grep); `apps/figcraft-cli/src/main.rs` (parts), `apps/figcraft-cli/src/files.rs` (parts); `apps/figcraft-server/src/directory.rs` (parts); `plugins/sdk/src/lib.rs` (head); `docs/plugin-functional-inventory.json` (FigCraft entry).
- **VectorCraft**: `Cargo.toml` (grep); `crates/plugins/src/lib.rs`, `runtime.rs` (grep and parts), `registry.rs` (parts), `export.rs` (parts), `effect.rs` (grep), `manifest.rs` (grep and parts), `objects.rs` (grep and parts); `crates/engine/src/cmd.rs` (parts), `cmd/plugin.rs` (parts), `cmd/params.rs` (grep and parts), `session/execute.rs` (grep), `cmd/fileio/formats.rs` (parts), `cmd/fileio/encode.rs` (grep and parts), `store.rs` (parts), `tests.rs` (grep); `crates/mcp/src/headless.rs` (parts), `crates/mcp/src/tools` (grep); `apps/vectorcraft-cli/src/main.rs` (parts), `apps/vectorcraft/src/prefs_dir.rs` (parts); `docs/plugin-functional-inventory.json` (VectorCraft entry).
- **PDFCraft**: `Cargo.toml` (grep); `crates/engine/src/commands.rs` (parts, grep), `edit.rs` (grep), `export.rs` (grep and parts), `session/editing.rs` (parts, grep), `session.rs` (grep); `crates/automation/src/dispatch.rs` (parts), `tools.rs` (parts, grep), `documents.rs` (parts), `file_access.rs` (header), `paths.rs` (grep), `mcp.rs` (grep); `crates/ui-egui/src/control.rs` (header), `crates/ui-egui/src/control/plugin.rs` (file name), `crates/ui-egui/src/app/frame.rs` (grep); `crates/js/src/lib.rs` (header); `apps/pdfcraft-cli/src/main.rs` (grep); `docs/plugin-functional-inventory.json` (PDFCraft entry).
- **DesignCraft**: `Cargo.toml` (grep); `crates/engine/src/cmd.rs` (parts), `lib.rs` (parts), `cmd/params.rs` (parts), `cmd/object.rs` (parts), `cmd/export/registry.rs` (parts), `cmd/file/place.rs` (grep), `cmd/design.rs` (grep), `cmd/design/colors.rs` (grep), `cmd/buttons.rs` (grep), `cmd/layout/parents.rs` (grep); `crates/format/src/lib.rs` (head, grep); `crates/geom/src/lib.rs` (head); `crates/mcp/src/server.rs` (grep), `backend.rs` (grep), `tests.rs` (grep); `apps/designcraft-cli/src/main.rs` (grep and parts); `docs/plugin-functional-inventory.json` (DesignCraft entry).
- **DeckCraft**: `Cargo.toml` (grep); `crates/engine/src/cmd.rs` (parts), `lib.rs` (parts), `cmd/file.rs` (parts, grep), `cmd/design.rs` (grep), `cmd/design/colors.rs` (grep), `cmd/insert.rs` (grep); `crates/geom/src/lib.rs` (head); `crates/mcp/src/tests.rs` (grep); `crates/ui-egui/src/ime_tests.rs` (name only); `apps/deckcraft-cli/src/main.rs` (grep); `docs/plugin-functional-inventory.json` (DeckCraft entry).
- **Pinned craft-libs** (doc repo object store, read-only): `git ls-tree` at `92d392cf`, `787a7a0f`, `c5a1cbfb`, `e11f9d9` and `HEAD`; `crates/menu/src/model.rs` and `README.md` at `e11f9d9` (grep); `crates/keymap/src/keymap.rs` at `92d392cf` (lines 326-346 and 700-716); `crates/keymap/README.md` at `92d392cf` (grep only).
- **Doc context**: `docs/plugins/readiness/brief.md` (read, working tree of the doc repo); `docs/plugin-functional-inventory.json` (read for entry sources only, verified not trusted).
- Not covered by this worker: runtime behaviour, any GUI wiring, most test bodies (grep only), CI, and any app outside the five assigned.

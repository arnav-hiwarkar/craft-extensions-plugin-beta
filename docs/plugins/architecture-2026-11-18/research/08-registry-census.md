# 08 — Command registry census across the Craft suite

Researcher 8 of 8. The workspace was read-only. Nothing was built and nothing was run except extraction scripts. Date: 2026-11-18.

Source revisions:

| Repository | Branch and commit |
|---|---|
| effectcraft | `plugin-beta@02064025` |
| photocraft | `plugin-beta@e45144c2a` |
| figcraft | `plugin-beta@48f0eb6c` |
| filmcraft | `main@0b3f30ed` |
| anicraft | `main@ab48665` |
| lightcraft | `main@27b755a9` |
| vectorcraft | `main@ef698d76f` |
| designcraft | `main@3930cff8` |
| materialcraft | `main@49ebd50` |
| texturecraft | `main@97f6b81` |
| soundcraft | `main@8ee63dc` |

Machine-readable output is in `census/`:

- **Per app:** `<app>.csv` and `<app>.json`, one row per command, 11 apps.
- **All apps:** `all_apps.csv` and `all_apps.json`.
- **Tables:** `classification_matrix.tsv` and `metadata_quality.tsv`.
- **Cross-app matches:** `vocab_matches.json`.
- **Raw extraction:** `raw_<app>.json` and `stage_<app>.json`.
- **Scripts:** `scripts/` contains `rustlex.py`, `extract.py`, `macroarms.py`, `finalize.py`, `classify.py` and `vocab.py`.

---

## 1. Summary

- **The suite has 6,193 engine registry commands (5,806 distinct ids).** That is VectorCraft 1,044, PhotoCraft 954, EffectCraft 757, FilmCraft 686, FigCraft 630, DesignCraft 567, SoundCraft 483, LightCraft 391, AniCraft 365, TextureCraft 183 and MaterialCraft 133. Most apps have grown well past the earlier estimates:
  - EffectCraft: 435+ before, 757 now.
  - VectorCraft: ~888 before, 1,044 now.
  - MaterialCraft: ~50 before, 133 now.
  - TextureCraft: ~80 before, 183 now.

  About 1,000 more UI or shell commands sit outside the engine registries. Examples: PhotoCraft's 146 `SHELL_COMMANDS` and LightCraft's and DesignCraft's `UI_COMMANDS` lists. A headless MCP session never sees them.
- **All 11 apps share one shape.** Each has a `CommandSpec` struct (id, label, menu, shortcut, a free-text `params` string, `enabled`, `run`, plus 0–3 flags) and a static `command_specs()` list. Every transport looks commands up by id: UI menus, CLI, control port and MCP. The apps differ only in macro sugar and in which flags exist. So a generated registry bridge is mechanically feasible.
- **The metadata is too weak for a default-deny bridge.**
  - No app declares side effects.
  - No app has a description field. The `params` text is the only prose.
  - Real typed parameter schemas exist only in VectorCraft, for 8 of 1,044 commands.
  - 95% of commands document their parameters, but as a hand-written mini-language. Each app parses it differently: EffectCraft `cmd/schema.rs`, VectorCraft and DesignCraft `cmd/params.rs`, FigCraft `doc_param_names`, PhotoCraft `param_check.rs`, where only 2 commands are strict.
  - The "query" bit means different things in different apps: journal, history, mutates, or undoable plus view. MaterialCraft and TextureCraft have no such bit at all.
  - Most apps do not declare undo behaviour. It happens inside `run`.
- **What the commands do (heuristic classification):**
  - document edit: 3,367
  - selection or view state: 686
  - query: 608
  - file system as the primary effect: 722. 1,140 commands touch the file system as their primary effect or through a path parameter.
  - app state or preferences: 232
  - clipboard: 153
  - long-running job: 130
  - process/OS: 78
  - undo/history: 57
  - collaboration: 52
  - window/UI: 45
  - AI/model download: 27
  - plugin management: 25
  - network, detected by name: 11. This is an undercount; see §4.

  On top of the primary class:
  - 107 commands are irreversible or outside the document's undo.
  - 455 more are destructive document edits that undo can reverse.
- **About 79% of the surface is plausibly plugin-safe** once metadata exists: document edits, queries and selection/view. The other 21% must be denied by default or mediated by the host. That 21% is file paths, network, process, clipboard, preferences, plugin management, history, UI and jobs.
- **Cross-app vocabulary converges only on clipboard, undo and file operations.** Only 15 ids appear in five or more apps, for example `edit.undo`, `edit.copy` and `file.save`. Selection, node and layer creation, listing, transform, zoom and playhead are named differently in every app. Examples:
  - Listing the document: `layer.tree`, `node.get`, `document.inspect`, `doc.info`, `layer.list`.
  - Setting the selection: `layer.select`, `select.set`, `selection.set`.
  - Moving the playhead: `time.set`, `playhead.set`, `timeline.goto`, `transport.locate`.

  AniCraft and SoundCraft use `snake_case`; the other nine apps use `lowerCamelCase`. §6 proposes 84 canonical operations.
- **The beta adapters reach only a hand-picked slice of each registry:**

  | Adapter | Registry ids reached | Share of registry |
  |---|---|---|
  | EffectCraft | 28 of 757 | 3.7% |
  | PhotoCraft | about 106 of 954 | 11% |
  | FigCraft | 72 of 630 | 11% |

  The upstream legacy systems reach far more: VectorCraft 705 of 1,044 (68%), with every id classified and enforced by a test, and AniCraft 274 of 365 (75%), allowed by namespace.
- **MCP is effectively full-power local automation.** Every server has a generic execute-by-id tool. Three apps have no file sandbox: FilmCraft, LightCraft and AniCraft. Five more have one only when the operator opts in. Six control ports accept any local process without a token. A plugin must get none of these defaults.
- **Recommendation:** add a typed `effects`/`scope`/`undo`/`run_mode`/`exposure` declaration to `CommandSpec` in every app, as one shared struct from craft-libs. Then generate the bridge allowlist, permission classes and typed bindings from it. Fail CI on any command with no declaration, as VectorCraft's `every_command_is_reviewed` test already does. Enforce effects at runtime through FigCraft-style choke points. Expose canonical operations to plugins and keep raw registry ids as an app-scoped, explicitly permissioned extra.

---

## 2. Method

### 2.1 Extraction

The registries are Rust source, so the counts are static and nothing was built. `scripts/rustlex.py` is a small Rust lexer. It handles comments, raw strings, escapes, char-versus-lifetime, balanced delimiters and `#[cfg(test)]` module skipping. `extract.py` finds three kinds of construct:

1. **Invocations whose first argument is an id-shaped string literal** (`^[a-z]\w*(\.\w+)+$`), as a macro (`cmd!`, `spec!`, `query!`, `fe!`, `flag!`…) or a call (`c(…)`, `spec(…)`, `q(…)`), with at least one more argument. A leading variant keyword (`cmd!(query "id", …)` or `cmd!(noundo …)`) is recorded as `variant`.
2. **`CommandSpec { id: "…", … }` struct literals.**
3. **`CommandSpec { journal: false, ..cmd!(…) }` overrides.**

`macroarms.py` then parses every `macro_rules!` in each app. For each invocation it matches the arm by variant and arity and reads which argument feeds `journal`, `history`, `mutates`, `undoable`, `view`, `params`, `menu` and `shortcut`. Most local helper macros resolve to the right flags this way. This matters most in PhotoCraft, which has about 40 file-local `spec!` macros, some of which set `journal: false` (for example `timeline_cmds.rs:138`).

Some false positives were removed by constructor name. These were settings tables (EffectCraft `settings/prefs/dialog.rs` `s(…)`, FilmCraft `settings/schema.rs` `f/b/sub`) and param helpers (`req_num(p,"x")`).

Some id families are generated with `concat!` or `Box::leak` and were added back by hand-written expanders in `finalize.py`:

- **PhotoCraft:**
  - `gallery_specs!`: 39 `filter.gallery.*` ids
  - `stack_spec!`: 12
  - `style_specs!`: 18
  - `stylistic_sets!`: 20
  - `style_cmd!`: 10
  - the `ADJ` loop: 32 `image.adjustments.*` and `layer.newAdjustmentLayer.*` ids, `cmd.rs:1251-1313`
- **FilmCraft:** `camera_keys!`, 9 ids (`multicam.rs:53`).
- **VectorCraft:** `textstyles.rs` `charStyle.*`/`paraStyle.*`, 20 ids.

Commands registered at runtime are excluded:

- PhotoCraft `plugin_filters` (`plugin.<id>/<contribution>`) and `plugin_sessions`
- VectorCraft plugin commands (`cmd/plugin.rs:460-483`, `Box::leak`)

### 2.2 Validation

| Check | Result |
|---|---|
| AniCraft `docs/guide/reference/commands.md` ("the same 365 commands") | 365/365 identical, no extras |
| VectorCraft `cmd/plugin_call/reviewed.rs` (ALLOWED + DENIED, 718 ids) | 718/718 present in the census |
| FigCraft `engine/src/catalog.rs` menu ids | 216/221. The 5 misses are UI-side `edit.copyAs*` in `ui-egui/src/menus.rs` |
| EffectCraft `engine/src/menus.rs` | 39/40 (`ui.menu.list` is UI-side) |
| PhotoCraft `ui-egui/src/menu_catalog.rs` | 506/664. The 158 misses are shell/UI commands in `engine/src/shell_commands.rs` (146 entries, including `file.open`, `file.save`, `edit.freeTransform`), which the UI implements |
| Macro variant counts checked by grep | DesignCraft 75 query + 109 noundo + 383 plain = 567. SoundCraft 20 + 123 + 9 + 237 = 389 `cmd!`, plus 94 helper macros. LightCraft 108 + 283. VectorCraft 226 + 795 |

The counts are accurate to within a few commands per app. PhotoCraft has the most uncertainty, about ±20, because of its macro sprawl.

### 2.3 Classification

`classify.py` assigns one primary class and secondary flags with ordered regex rules. The rules look at the id and, for file and network detection, at the label and the param doc. The `query` flag and query-shaped verbs (`list`, `get`, `info`, `inspect`…) promote a command to `query`. Destructiveness is an overlay with two levels:

- `dangerous/destructive`: irreversible or outside the document, such as quit, revert, purge, prefs reset, deleting presets or models, or clearing history.
- `doc-destructive (undoable)`: delete, remove, flatten, crop and similar operations on the document.

I hand-checked a random sample of 70 rows; about 85–90% had the right primary class. Errors cluster at the boundaries between network, process and long job, and in app-specific namespaces. One example is LightCraft `print.cell.*`, which is a document layout, not printing. Treat per-class numbers as ±10%. The per-row CSVs make it easy to correct any row by hand.

---

## 3. Per-app declaration mechanism and counts

### 3.1 The common pattern

Every app defines `pub struct CommandSpec { id, label, menu, shortcut, params: &'static str, enabled: fn(&Session)->Result<(),String>, run: fn(&mut Session,&Value)->Result<Value>, … }`. Each app builds a `command_specs()` list from per-module `specs()` functions. `Session::execute(id, params)` dispatches by id. UI menus, the CLI (`apps/<app>-cli`, or `crates/cli` for AniCraft), the control port and the MCP server all end up in that `execute`. Commands return untyped `serde_json::Value`.

### 3.2 Per app

| App | Struct and registry | Declaration sugar | Flags beyond id/label/menu/shortcut/params/enabled/run | Engine cmds | Namespaces (top) | Ids |
|---|---|---|---|---|---|---|
| EffectCraft | `crates/engine/src/cmd.rs:106` and `command_specs()` `:175`; dispatch `session/dispatch.rs:9` | `cmd!` (616), `query!` (75), `fe!` (frontend), `chan!`/`fast!`/`res!`, journal `spec()` const fn, 3 literals | `alt_shortcuts`, `journal` (false = pure query) | **757** | 59 (layer 124, view 78, file 66) | camelCase |
| PhotoCraft | `crates/engine/src/cmd.rs:15`, `build()` `:297`; plus runtime `plugin_filters`/`plugin_sessions` | ~40 file-local `spec!` macros, `cmd!`, `filter_cmd!`, `section!`, `key_cmd!`, 225 struct literals, `concat!` generators, `ADJ` loop | `journal` | **954** (+146 shell commands in `shell_commands.rs`) | 35 (layer 213, filter 110, type 98) | camelCase |
| FigCraft | `crates/engine/src/cmd.rs:36`, `registry()` `:73` | `spec!(id,label,params,history,run)` (630) | `history` (edit = one undo step). No `enabled`; menu and shortcut live in `catalog.rs` | **630** | 58 (text 63, view 49, vector 42) | camelCase |
| FilmCraft | `crates/engine/src/commands.rs:23`, `command_specs()` `:91` | `spec()`/`query()`/`c()`/`q()` fns, `cmd!`, `query!`, `align_cmd!`, `label_cmd!`, `sc!`, `tt!`, `camera_keys!` | `journal`, `named: NamedKeys` (targets derived from the param doc, `:96`) | **686** | 41 (graphics 83, clip 57, file 57) | camelCase |
| AniCraft | `crates/engine/src/lib.rs:115`, `commands()` `:136`, `execute` `:594` | closure `c(id,label,menu,shortcut,params,mutates,run)` and literals | `mutates` (undo step); `confirm.rs` gate for lossy ids. No `enabled` | **365** | 41 (file 32, layer 26, timeline 23) | snake_case |
| LightCraft | `crates/engine/src/cmd.rs:57`, `command_specs()` `:143` | `cmd!` and `cmd!(query …)` | `journal`, `explicit_targets` (5 commands) | **391** (+~210 `UI_COMMANDS`, `ui-egui/src/menus/ui_commands.rs`) | 41 (library 48, photo 44, mask 31) | camelCase |
| VectorCraft | `crates/engine/src/cmd.rs:128`, `command_specs()` `:289`, `session/execute.rs:61` | `cmd!` and `cmd!(query …)`, optional `schema: &S` | `journal`, `schema: Option<&ParamSchema>` (typed, `crates/apischema`) | **1,044** | 69 (object 158, addon 92, select 59) | camelCase |
| DesignCraft | `crates/engine/src/cmd.rs:88`, `command_specs()` `:176`, `lib.rs:632` | `cmd!`, `cmd!(query …)`, `cmd!(noundo …)` | `journal`, `undoable` | **567** (+~190 `UI_COMMANDS`) | 64 (object 60, type 44, file 42) | camelCase |
| MaterialCraft | `crates/engine/src/commands.rs:12`, `:50`, `execute` `:103` | `cmd!` | **none** | **133** | 9 (graph 65, view 20, edit 13) | camelCase |
| TextureCraft | `crates/engine/src/commands.rs:14`, `:51`, `execute` `:97` | `cmd!` | **none** | **183** | 20 (layer 26, tool 21, camera 17) | camelCase |
| SoundCraft | `crates/engine/src/cmd.rs:74`, `command_specs()` `:129`, `lib.rs:435` | `cmd!`, `cmd!(query|noundo|view …)`, `flag!`/`toggle!`/`option!`/`radio!` | `journal`, `undoable`, `view` | **483** (+~70 UI ids) | 18 (view 97, edit 94, track 66) | snake_case |

UI-only registries hold another ~1,000 commands; the ~ counts above come from id-literal counts. The surface a plugin sees therefore depends on whether it reaches the engine or the shell. In PhotoCraft, `file.open` and `file.save` are shell and job operations, not engine commands. Its MCP server has bespoke `doc_open`/`doc_save` tools to cover them.

### 3.3 How MCP and the CLI expose the registry

Every MCP server offers a generic list tool plus a run-by-id tool, and each app names them differently:

| App | List tool | Run tool |
|---|---|---|
| EffectCraft | `list_commands` | `execute_command`/`command_run` |
| PhotoCraft | `command_list` | `command_run` |
| FigCraft | `list_commands` | `run_command` |
| FilmCraft | `command_list` | `command_run` |
| AniCraft | `anicraft_commands` | `anicraft_run` |
| LightCraft | `command_list` | `command_run` |
| VectorCraft | `command_list` | `command_run` |
| DesignCraft | `list_commands` | `execute` |
| MaterialCraft | `command_list` | `command_run` |
| TextureCraft | `command_list` | `command_run` |
| SoundCraft | `list_commands` | `execute` |

LightCraft also generates one `cmd_<id>` MCP tool per command, with an untyped `{"type":"object","additionalProperties":true}` schema (`crates/mcp/src/tools.rs:15,341-370`). EffectCraft derives a key-only JSON Schema from the param doc (`engine/src/cmd/schema.rs:77` `params_schema`). VectorCraft emits a real JSON Schema 2020-12 for the 8 commands that have `schema`. Section 8 has the transport and sandbox details.

---

## 4. Classification

### 4.1 App × class (primary class; heuristic, about ±10%)

| App | Total | Query | Doc edit | Sel/view | App state/prefs | File system | Network | Clipboard | Process/OS | Collab | Plugin mgmt | Undo/history | Window/UI | Long job | AI/model dl | +Hard-destructive | +Doc-destructive | +Touches FS (any) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| EffectCraft | 757 | 54 | 354 | 108 | 8 | 97 | 1 | 16 | 27 | 0 | 6 | 6 | 27 | 47 | 6 | 20 | 38 | 150 |
| PhotoCraft | 954 | 79 | 544 | 101 | 61 | 122 | 0 | 17 | 8 | 0 | 1 | 14 | 2 | 5 | 0 | 13 | 64 | 163 |
| FigCraft | 630 | 103 | 293 | 84 | 29 | 44 | 3 | 15 | 1 | 45 | 0 | 2 | 9 | 2 | 0 | 8 | 42 | 83 |
| FilmCraft | 686 | 49 | 392 | 65 | 14 | 113 | 1 | 9 | 2 | 0 | 0 | 8 | 1 | 30 | 2 | 16 | 61 | 141 |
| AniCraft | 365 | 42 | 207 | 25 | 9 | 41 | 0 | 14 | 17 | 3 | 5 | 2 | 0 | 0 | 0 | 8 | 34 | 67 |
| LightCraft | 391 | 57 | 215 | 9 | 4 | 42 | 4 | 7 | 2 | 0 | 0 | 8 | 3 | 21 | 19 | 10 | 42 | 89 |
| VectorCraft | 1,044 | 131 | 624 | 95 | 20 | 110 | 1 | 27 | 15 | 0 | 10 | 5 | 2 | 4 | 0 | 17 | 73 | 242 |
| DesignCraft | 567 | 62 | 367 | 41 | 4 | 73 | 0 | 9 | 4 | 0 | 0 | 3 | 1 | 3 | 0 | 6 | 50 | 102 |
| MaterialCraft | 133 | 5 | 71 | 19 | 0 | 25 | 0 | 7 | 0 | 3 | 0 | 3 | 0 | 0 | 0 | 1 | 11 | 34 |
| TextureCraft | 183 | 13 | 77 | 15 | 26 | 28 | 0 | 7 | 0 | 0 | 0 | 3 | 0 | 14 | 0 | 5 | 10 | 33 |
| SoundCraft | 483 | 13 | 223 | 124 | 57 | 27 | 1 | 25 | 2 | 1 | 3 | 3 | 0 | 4 | 0 | 3 | 30 | 36 |
| **All** | **6,193** | **608** | **3,367** | **686** | **232** | **722** | **11** | **153** | **78** | **52** | **25** | **57** | **45** | **130** | **27** | **107** | **455** | **1,140** |

"+Touches FS (any)" counts commands whose primary class is file system or whose param doc has a `path`/`file`/`dir`/`folder` key. That is 1,140, 18% of the suite. Many are otherwise ordinary edits that also accept a path, such as `layer.newFillLayer` with `profile: path to .icc`, `image.adjustments.colorLookup {file}` and VectorCraft `swatch.*` library loads. A bridge must strip or refuse those keys even on commands it otherwise allows, as the PhotoCraft adapter already does with its reserved-key list in `plugin_ops.rs:59`.

### 4.2 Examples per class

| Class | Examples (app:id) |
|---|---|
| Query | `effectcraft:prop.get`, `effectcraft:layer.tree`, `photocraft:document.inspect`, `figcraft:node.get`, `vectorcraft:document.info`, `lightcraft:photo.inspect`, `soundcraft:session.inspect`, `texturecraft:layer.list` |
| Document edit | `effectcraft:layer.newSolid`, `photocraft:filter.blur.gaussianBlur`, `figcraft:component.makeSlot`, `filmcraft:clip.automateToSequence`, `designcraft:table.convertRowsToFooter`, `soundcraft:mix.volume` |
| Selection/view state | `vectorcraft:select.same.strokeWeight`, `effectcraft:view.channel.red`, `effectcraft:view.zoomIn`, `figcraft:view.zoomAt`, `soundcraft:view.zoom_fit`, `effectcraft:time.set`, `filmcraft:playhead.set` |
| App state/prefs | `effectcraft:prefs.set`, `effectcraft:shortcuts.set`, `figcraft:tool.ellipse`, `photocraft:brush.presets.delete`, `soundcraft:options.click`, `texturecraft:tool.brush` |
| File system | `effectcraft:file.saveAs`, `filmcraft:file.exportFrame`, `photocraft:file.import.variableDataSets`, `lightcraft:curve.savePreset`, `vectorcraft:document.exportPdf`, `designcraft:snippet.export`, `texturecraft:file.exportMesh` |
| Network | `figcraft:font.download`, `figcraft:pref.publishServer`, `lightcraft:metadata.reverseGeocode`, `lightcraft:lens.downloadDatabase`, `filmcraft:graphics.template.install` |
| Clipboard | `*:edit.copy/cut/paste`, `vectorcraft:clipboard.exportPng`, `soundcraft:edit.copy_pan_automation`, `effectcraft:edit.pasteSvg` |
| Process/OS | `effectcraft:file.executeFile`, `effectcraft:file.runScript`, `effectcraft:file.revealInFinder`, `effectcraft:edit.editOriginal`, `anicraft:desktop.integrate`, `anicraft:macro.*`, `photocraft:actions.play`, `designcraft:script.run`, `vectorcraft:help.website`, `*:app.quit` |
| Collaboration | `figcraft:collab.cursorChat`, `figcraft:comment.reply`, `figcraft:annotation.add`, `anicraft:file.publish`, `materialcraft:graph.comment.add` |
| Plugin mgmt | `vectorcraft:plugin.install/remove/setEnabled/safeMode`, `anicraft:plugin.install/remove/run`, `effectcraft:plugin.list/uiEvent`, `soundcraft:engine.vst3_plugins` (scans and loads native plug-ins) |
| Undo/history | `*:edit.undo/redo`, `vectorcraft:edit.undoGroup.begin/end`, `vectorcraft:edit.clearHistory`, `photocraft:history.newSnapshot`, `filmcraft:history.deleteRedoable` |
| Window/UI | `effectcraft:app.commandPalette`, `effectcraft:window.saveWorkspace`, `figcraft:window.new`, `figcraft:view.fullscreen` |
| Long-running job | `effectcraft:renderQueue.render`, `effectcraft:jobs.wait`, `filmcraft:sequence.renderInToOut`, `filmcraft:masks.track`, `lightcraft:faces.detect`, `lightcraft:denoise.queue`, `texturecraft:bake.run`, `soundcraft:audiosuite.process` |
| AI/model download | `effectcraft:face.model.download`, `effectcraft:roto.model.download`, `filmcraft:transcript.downloadModel`, `filmcraft:tts.downloadVoices`, `lightcraft:denoise.models.download`, `lightcraft:faces.models.download` |
| Hard-destructive | `*:file.revert`, `*:prefs.reset`, `effectcraft:edit.purgeUndo`, `filmcraft:file.discardRecovery`, `filmcraft:project.delete`, `lightcraft:library.emptyRecentlyDeleted`, `vectorcraft:edit.clearHistory`, `anicraft:plugin.remove`, `anicraft:desktop.remove`, `soundcraft:session.revert` |

### 4.3 What the counts mean for a bridge

- **Default-allowable (≈79%).** These are query, document edit and selection/view: 4,661 commands. Path-bearing parameters must be stripped. Selection and view state should be a separate permission, not `document.edit`.
- **Host-mediated only (≈14%).** File system (722), clipboard (153), long jobs (130) and undo/history (57). The host performs the operation for the plugin, for example with picked file handles, a host clipboard API, a job handle with progress and cancel, or an automatic undo group. The plugin never calls the raw command.
- **Never exposed (≈7%).** App state and prefs (232), process/OS (78), collaboration (52), plugin management (25), window/UI (45), AI model download (27) and network (11 by name). The network count is a floor. FigCraft alone routes 9 more network call sites through `file_access::network_refused()`, including publishing, live sessions, team libraries and fonts. LightCraft has 19 model or database download commands. So name detection undercounts network effects, which is why effects must be declared rather than inferred.

---

## 5. Metadata quality and required additions

### 5.1 Measured quality

| App | Param doc present | `{…}` key-doc form | Prose-only | Result shape documented (`→`) | Typed schema | Separate description | Query/edit flag | Undo flag | Side effects declared | `enabled` predicate | Named-target metadata |
|---|---|---|---|---|---|---|---|---|---|---|---|
| EffectCraft | 99% | 83% | 2% | 12% | 0 | no | `journal` (79 false) | no (body-defined; `begin_group`/`finish_group`) | no | yes | no |
| PhotoCraft | 90% | 76% | 4% | 21% | 0 (2 STRICT) | no | `journal` (117 false) | no (`checkpoint`) | no | yes | no |
| FigCraft | 100% | 87% | 0% | 26% | 0 | no | `history` (287 false) | **yes** (= history; dispatcher wraps an undo step, `session/exec.rs:69`) | no | **no** | no |
| FilmCraft | 98% | 69% | 3% | ~0% | 0 | no | `journal` (66 false) | no (`edit_group`) | no | yes | `named` (derived from doc, 200 cmds) |
| AniCraft | 99% | 93% | 0% | 21% | 0 | no | `mutates` (123 false) | **yes** (= mutates; `lib.rs:605,770`) | partial (`confirm.rs` lossy list) | **no** | no |
| LightCraft | 100% | 93% | 0% | 51% | 0 | no | `journal` (108 false) | no | no | yes | `explicit_targets` (5) |
| VectorCraft | 98% | 96% | 0% | 80% | **8** | no | `journal` (230 false) | no (body; `undoGroup` cmds, `transaction`) | no | yes | no |
| DesignCraft | 100% | 93% | 0% | 35% | 0 | no | `journal` (75 query) | **yes** `undoable` (383 true / 109 noundo; `lib.rs:632-670`) | no | yes | no |
| MaterialCraft | 100% | 98% | 0% | 6% | 0 | no | **none** | **none** | no | yes | no |
| TextureCraft | 100% | 80% | 0% | 17% | 0 | no | **none** | **none** | no | yes | no |
| SoundCraft | 96% | 68% | 16% | 2% | 0 | no | `journal` (21 query) | **yes** `undoable` (237) + `view` (9) | no | yes | no |

**Typed parameter schemas: 8 of 6,193 commands (0.13%).** VectorCraft's `schema: &S` sites are the only ones, using `crates/apischema`. That crate has `validate`, `to_json` (JSON Schema 2020-12), `hash`, and API levels with `SCHEMA_HASHES` (`engine/src/schemas.rs:1-40`). It is the only real schema system in the suite and the obvious seed for a shared one.

**Param docs: about 95% have one, and they are informal.** The grammar (`{key?: type (note), a|b, x=default} → {result}`) is shared by convention. Four apps parse it independently:

- EffectCraft `accepted_params`/`params_schema` (`cmd/schema.rs:7,77`)
- VectorCraft and DesignCraft `cmd/params.rs`; DesignCraft's header says "Ported from VectorCraft"
- FigCraft `doc_param_names`/`doc_needs_params` (`cmd.rs:160-260`)

PhotoCraft enforces unknown-key refusal for only 2 commands (`param_check.rs:25`). None of these parsers gives types usable for bindings.

**The query flag is overloaded:**

- In EffectCraft, PhotoCraft, FilmCraft, LightCraft and VectorCraft, `journal: false` means "do not record in the replay journal". A journaled command can still be pure view state: EffectCraft `view.channel.red` and `view.zoomIn` are `journal: true`. A non-journaled one can still mutate: PhotoCraft `actions.newSet` and `view.colorSamplers.delete` are `journal: false`.
- In FigCraft, `history: false` (287 commands) means "no undo step". That includes selection, view and comment operations, not only reads.
- In AniCraft, `mutates` means an undo step.
- In MaterialCraft and TextureCraft no flag exists.

So **no app can answer "is this command read-only?"**, and that is the first question a permission system asks.

**Undo is mostly not declared.** It is implemented inside each `run` (`Session::edit(label, …)`, `checkpoint`, `push_undo`). Only DesignCraft (`undoable`), SoundCraft (`undoable`, `view`), FigCraft (`history`) and AniCraft (`mutates`) state it, and only those four have a dispatcher that enforces it. Grouping primitives differ:

| App | Undo grouping |
|---|---|
| EffectCraft | `begin_group`/`finish_group` |
| FigCraft | `begin_group`/`finish_group`/`push_undo` |
| MaterialCraft | `begin_group`/`end_group` |
| FilmCraft | `edit_group` |
| AniCraft | `begin_edit` |
| VectorCraft | `transaction`, `undo_group_begin/end`, and the `edit.undoGroup.begin/end` *commands* |
| PhotoCraft | `checkpoint` |
| DesignCraft and SoundCraft | `push_undo` |
| TextureCraft | `push_history` |
| LightCraft | none found in the engine |

VectorCraft's `edit.undoGroup.*` are registry commands, so any MCP client can open an undo group and leave it unbalanced. A plugin must never see them.

**Side effects are declared nowhere.** The closest things are:

- AniCraft's `confirm.rs` list of lossy ids (`lib.rs:597-601`), which the caller can satisfy itself with `confirm: true`.
- VectorCraft's hand-kept `reviewed::ALLOWED/DENIED` plus `DENIED_PREFIXES`.
- EffectCraft's `confine.rs` `REFUSED`/`READS`/`WRITES` tables.
- PhotoCraft's `authorize_engine_command` match (`automation/src/workspace.rs:625-674`).
- MaterialCraft and TextureCraft `authorize_command`.

All of these are **denylists kept outside the registry**, and they drift. PhotoCraft's gate still names `plugin.install`, which no longer exists in photocraft. EffectCraft fixed missing `READS` entries in #1243.

**Other gaps:**

- No `result` schema; 0–80% of commands mention the result shape in prose.
- No stability or versioning except VectorCraft's `API_VERSION` and `SCHEMA_HASHES`.
- No idempotency or retry marker.
- No "needs UI/modal" marker. Commands that open dialogs exist only in the UI registries.
- No cost or duration marker. `jobs.*` exists in EffectCraft, PhotoCraft and FilmCraft, but no command says it starts a job.

### 5.2 Metadata a default-deny, generated bridge needs

These fields should be added to every `CommandSpec`, ideally as one shared `craft_command::Meta` type in craft-libs used by all 11 apps:

```rust
pub struct Meta {
    pub effects: &'static [Effect],   // REQUIRED, non-empty; [] is a compile error via macro
    pub scope: Scope,                 // what the command may touch
    pub undo: Undo,                   // how it relates to document history
    pub run: RunMode,                 // sync / job / interactive
    pub destructive: Destructive,     // None | Reversible | Irreversible
    pub params: Option<&'static ParamSchema>, // typed; VectorCraft apischema promoted to craft-libs
    pub result: Option<&'static ParamSchema>,
    pub exposure: Exposure,           // Internal | Automation | Plugin(canonical op or app op)
    pub since: u32,                   // API level (VectorCraft SCHEMA_HASHES model)
}

pub enum Effect {
    DocRead, DocWrite,                 // document content
    SelectionRead, SelectionWrite,     // selection (not document content)
    ViewWrite,                         // zoom, scroll, channel view, playhead, panels
    AppStateRead, AppStateWrite,       // tools, brushes, swatches, options
    PrefsRead, PrefsWrite,             // persisted preferences, shortcuts
    FsRead(PathSource), FsWrite(PathSource), // PathSource::{Param, Picked, AppDir}
    Net { host: Option<&'static str> },
    ClipboardRead, ClipboardWrite,
    Process,                           // spawn, open URL, reveal, run script, Apple Events
    PluginMgmt, History, UiWindow, Job, ModelInstall,
    Collab,                            // presence, comments to remote, publishing
    AppLifecycle,                      // quit, close window, discard recovery
}
pub enum Scope { None, Selection, Target /* explicit ids */, Document, Project, App }
pub enum Undo { Step, Group /* opens/closes */, NonUndoable, External /* outside document: files, prefs */, NotApplicable }
pub enum RunMode { Sync, Job { cancellable: bool, progress: bool }, Interactive /* needs UI */ }
```

**Rules that make this enforceable:**

1. **Make `effects` mandatory.** The `cmd!`-style macros take it as a required argument. A workspace test, modelled on VectorCraft's `every_command_is_reviewed` (`cmd/plugin_call.rs:684`), fails when any command lacks it. Bulk back-fill can start from this census: the `cls` and `flags` columns map onto `Effect`.
2. **Enforce declarations at runtime.** Adopt FigCraft's pattern: a single `file_access` module, plus `clippy.toml` `disallowed-methods` banning `std::fs` in the engine (`figcraft/crates/engine/clippy.toml`). Add equivalent choke points for network (`network_refused`), process spawn and clipboard. The dispatcher sets a thread-local "allowed effects" mask from `Meta.effects` and the caller's grant. A choke point reached with its effect undeclared returns an error. Then a wrong declaration fails safe instead of leaking.
3. **Undo:**
   - `Undo::Step` commands run in a host-owned group, as the dispatchers in DesignCraft, AniCraft and FigCraft already do.
   - `Undo::Group` (`edit.undoGroup.*`) and `Effect::History` are never plugin-exposed.
   - `Undo::External` requires a permission class distinct from `document.edit`.
4. **Replace the `query` and `journal` bits** with `effects ⊆ {DocRead, SelectionRead, AppStateRead, PrefsRead}`, and keep `journal` as a separate replay concern.
5. **Generate everything from `Meta`.** That covers the MCP `inputSchema`/`outputSchema` (replacing EffectCraft's key-only schema and LightCraft's `additionalProperties:true`), CLI help, plugin TypeScript/Rust bindings, and the permission prompt text.
6. **Mark path-bearing parameters** with schema type `Path(Read|Write)`, so the bridge can strip them or substitute picked handles. The PhotoCraft and FigCraft adapters already special-case `path`, `file`, `target`, `author` and `pluginData` by hand.

### 5.3 Per-app work to reach that bar

| App | Cmds | Starting point | Main additions |
|---|---|---|---|
| EffectCraft | 757 | `journal`, `confine.rs` tables, key-only schema | effects, undo (body-defined today), typed schema; move `READS`/`WRITES`/`REFUSED` into `Meta` |
| PhotoCraft | 954 | `journal`, `authorize_engine_command`, 2 strict | effects; consolidate ~40 `spec!` macros into one; schema for the filter/adjustment families (generated tables make this cheap); shell commands need `Meta` too |
| FigCraft | 630 | `history`, `file_access` choke point | split `history:false` into SelectionWrite/ViewWrite/DocRead; add `enabled`; schema |
| FilmCraft | 686 | `journal`, `named` | effects, undo, sandbox (none today), schema |
| AniCraft | 365 | `mutates`, `confirm.rs`, namespace policy | effects (namespace policy → per-id), `enabled`, sandbox; `confirm` becomes host consent |
| LightCraft | 391 | `journal`, `explicit_targets` | effects (19 model/DB downloads), undo, sandbox, schema |
| VectorCraft | 1,044 | `journal`, `schema` (8), reviewed ALLOWED/DENIED (718) | effects (reviewed lists convert directly); extend `apischema` to all commands |
| DesignCraft | 567 | `journal`, `undoable`, opt-in roots | effects, schema; confine `--connect` sessions |
| MaterialCraft | 133 | nothing | query and undo flags from scratch, effects, schema |
| TextureCraft | 183 | nothing | same as MaterialCraft |
| SoundCraft | 483 | `journal`, `undoable`, `view` | effects (native VST3/CLAP scan and load), schema; prose docs (16%) need rewriting |

---

## 6. Cross-app canonical vocabulary

### 6.1 How differently the apps name the same thing today

Only 245 ids appear in two or more apps, and only 15 in five or more: `edit.undo`, `edit.redo` (11 apps), `edit.copy`/`paste`/`cut` (9), `edit.duplicate` (8), `file.new`, `file.open`, `file.save`, `file.saveAs`, `file.close` (7), `file.revert` (6), `edit.clear`, `text.fonts` and `prefs.set` (5). Even those differ underneath:

- SoundCraft says `session.open/save`.
- VectorCraft says `document.open/save`.
- PhotoCraft's `file.open` and `file.save` are shell/job operations, not engine commands.
- `edit.clear` means "delete selection" in six apps, while FigCraft uses `edit.delete`.

| Concept | Ids today (one per app where found) |
|---|---|
| Document inspect | EffectCraft `project.summary`/`comp.info`, PhotoCraft `document.inspect`, FilmCraft `project.inspect`, AniCraft `doc.info`, LightCraft `catalog.stats`, VectorCraft `document.info` and `document.inspect`, DesignCraft `document.inspect`, MaterialCraft `session.inspect`, TextureCraft `project.inspect`, SoundCraft `session.inspect` |
| Layer/node tree | EffectCraft `layer.tree`, FigCraft `node.get`/`node.find`, TextureCraft `layer.list`, SoundCraft `track.inspect`, others only via `document.inspect` |
| Set selection | EffectCraft, PhotoCraft, AniCraft and TextureCraft `layer.select`; FigCraft and VectorCraft `select.set`; DesignCraft `selection.set` |
| Get selection | no registry command in most apps; it is part of inspect output. The adapters invented `selection.get` |
| Select all / none | `edit.selectAll` (EffectCraft, FilmCraft, DesignCraft, MaterialCraft), `select.all` (PhotoCraft, FigCraft, VectorCraft), `library.selectAll` (LightCraft); none = `edit.deselectAll`, `select.deselect`, `select.none`, `edit.deselect`, `library.selectNone`, `edit.selectNone` |
| Create layer/node | EffectCraft `layer.newSolid/newText/newShape…`, PhotoCraft `layer.new.layer`, FigCraft `node.create`, AniCraft, VectorCraft and DesignCraft `layer.new`, SoundCraft `track.new`, TextureCraft `layer.add.*` |
| Generic property set | EffectCraft `prop.set` (path-addressed), PhotoCraft and VectorCraft `layer.setProps`, FigCraft `node.set`, DesignCraft and TextureCraft `layer.set`, VectorCraft `object.setProps` |
| Group/ungroup | FigCraft, VectorCraft and DesignCraft `object.group`; FilmCraft and SoundCraft `clip.group`; AniCraft `modify.group`; LightCraft `stack.group`; TextureCraft `layer.group`; PhotoCraft `layer.new.group` |
| Bring to front | PhotoCraft `layer.arrange.bringToFront`, FigCraft `arrange.front`, FilmCraft `graphics.bringToFront`, VectorCraft `object.arrange.bringToFront`, DesignCraft `object.bringToFront` |
| Rotate/scale | FigCraft and DesignCraft `transform.rotate`, VectorCraft `object.rotate`, PhotoCraft `edit.transform.rotate90Ccw`; scale = VectorCraft `object.scale`, DesignCraft `transform.scale`, FigCraft `transform.resize` |
| Fill/stroke | VectorCraft `paint.setFill`/`paint.setStroke`, FigCraft `fill.add`/`stroke.set`, TextureCraft `layer.fill.set`, AniCraft `stroke.width_at` |
| Zoom | EffectCraft and FigCraft `view.zoomIn`, FigCraft `view.zoom100/zoomAt`, SoundCraft `view.zoom_in/zoom_fit/zoom_set`; other apps keep zoom in UI-only registries |
| Playhead | EffectCraft `time.set`, FilmCraft `playhead.set`, AniCraft `timeline.goto`, SoundCraft `transport.locate` |
| Markers | EffectCraft `comp.addMarker`/`layer.addMarker` (the adapter calls it `marker.add`); FilmCraft `markers.*` |
| Apply effect | EffectCraft and VectorCraft `effect.apply`, FilmCraft `effects.apply`, FigCraft `effect.add`; PhotoCraft has one id per filter (`filter.blur.gaussianBlur`) |
| History list | EffectCraft `edit.history`, FilmCraft and LightCraft `history.list`, DesignCraft `document.history`, MaterialCraft `session.history`, SoundCraft `engine.history`, AniCraft `macro.history` (a different meaning) |
| List commands | EffectCraft, PhotoCraft and FilmCraft `command.list`; SoundCraft `engine.commands`; VectorCraft `engine.apiReference` |
| Batch | EffectCraft `engine.batch`, FigCraft and VectorCraft `command.batch`, VectorCraft `action.batch` |

Shapes differ too:

- **How a target is named:**
  - EffectCraft: `layer: id|name|#n`.
  - PhotoCraft: `target`/`layer`.
  - FilmCraft: `clips: [id]`/`items: [id]`, derived as `NamedKeys`.
  - LightCraft: `ids`/`id`, only when `explicit_targets` is set.
  - FigCraft: `ids`.
- **Selection:** some apps act on the selection when no target is given and others refuse.
- **Colour:** `#hex`, `[r,g,b]` 0..1, or `{r,g,b}` depending on the app.
- **Time:** EffectCraft `Tick` (254,016,000,000 per second), FilmCraft frames or timecode, SoundCraft samples or bars.

### 6.2 Naming convention

- **Id form:** `<domain>.<verb>[<Qualifier>]`, in `lowerCamelCase` segments, ASCII only, 2 or 3 segments. AniCraft and SoundCraft keep their `snake_case` ids as app-scoped aliases.
- **Verbs** come from a closed set:

  | Kind | Verbs |
  |---|---|
  | Reads | `get`, `list`, `find`, `inspect` |
  | Lifecycle | `create`, `delete`, `duplicate` |
  | Changes | `set` (partial update), `move`, `reorder`, `group`, `ungroup`, `apply` |
  | Modes | `begin`, `end`, `cancel` (gestures and jobs only) |

  Labels carry the UI wording. Ids stay mechanical.
- **Domains are singular nouns** for the object acted on: `node`, `selection`, `doc`, `history`, `view`, `time`, `style`, `text`, `effect`, `marker`, `asset`, `export`.
- **Targets** always use `{"targets": [TargetRef]}`. `TargetRef` is an opaque host id string. When `targets` is omitted, the command uses the captured selection, and `Meta.scope` says so. Names and `#n` indices are UI conveniences and are not accepted from plugins.
- **Units:** canonical ops use document units declared by `doc.get` (`unit`, `timeBase`). Colour is `{space, components, alpha}`. Time is a rational `{value, rate}`.
- **App-specific operations** are namespaced `app.<appId>.<registryId>`, for example `app.photocraft.filter.blur.gaussianBlur`. They need an explicit per-app permission. Plugins then cannot assume portability by accident.

### 6.3 Proposed canonical operations (84)

These should behave identically in every app that has the concept. Apps without the concept answer `unsupported`, a stable error, not "unknown op". "Effect" is the `Meta.effects` class each operation maps to.

| # | Op | Effect | Notes and current sources |
|---|---|---|---|
| 1 | `host.info` | – | app id, version, API level, supported ops |
| 2 | `doc.get` | DocRead | units, size, timebase, pages/comps/sequences (all `*.inspect`/`*.info`) |
| 3 | `doc.list` | DocRead | open documents (PhotoCraft `documents.list`) |
| 4 | `doc.setActive` | ViewWrite | |
| 5 | `doc.metadata.get` | DocRead | |
| 6 | `doc.metadata.set` | DocWrite | |
| 7 | `container.list` | DocRead | pages / artboards / comps / sequences / graphs |
| 8 | `container.create` | DocWrite | |
| 9 | `container.delete` | DocWrite | |
| 10 | `container.setActive` | ViewWrite | FigCraft `page.select` |
| 11 | `node.tree` | DocRead | layers/nodes/tracks/clips; EffectCraft `layer.tree` |
| 12 | `node.get` | DocRead | |
| 13 | `node.find` | DocRead | FigCraft `node.find` |
| 14 | `node.children` | DocRead | |
| 15 | `node.create` | DocWrite | `{kind, parent?, index?, props}` |
| 16 | `node.delete` | DocWrite | |
| 17 | `node.duplicate` | DocWrite | |
| 18 | `node.set` | DocWrite | partial props; replaces `layer.setProps`/`node.set`/`layer.set` |
| 19 | `node.rename` | DocWrite | |
| 20 | `node.move` | DocWrite | reparent / reorder |
| 21 | `node.reorder` | DocWrite | front/back/forward/backward |
| 22 | `node.group` | DocWrite | |
| 23 | `node.ungroup` | DocWrite | |
| 24 | `node.setVisible` | DocWrite | |
| 25 | `node.setLocked` | DocWrite | |
| 26 | `node.align` | DocWrite | |
| 27 | `node.distribute` | DocWrite | |
| 28 | `selection.get` | SelectionRead | |
| 29 | `selection.set` | SelectionWrite | |
| 30 | `selection.add` | SelectionWrite | |
| 31 | `selection.remove` | SelectionWrite | |
| 32 | `selection.all` | SelectionWrite | |
| 33 | `selection.none` | SelectionWrite | |
| 34 | `selection.invert` | SelectionWrite | |
| 35 | `transform.get` | DocRead | |
| 36 | `transform.set` | DocWrite | matrix or TRS |
| 37 | `transform.translate` | DocWrite | |
| 38 | `transform.rotate` | DocWrite | |
| 39 | `transform.scale` | DocWrite | |
| 40 | `transform.flip` | DocWrite | |
| 41 | `style.fill.set` | DocWrite | |
| 42 | `style.stroke.set` | DocWrite | |
| 43 | `style.opacity.set` | DocWrite | |
| 44 | `style.blend.set` | DocWrite | |
| 45 | `style.list` | DocRead | shared styles (FigCraft, DesignCraft, PhotoCraft) |
| 46 | `style.apply` | DocWrite | |
| 47 | `color.swatches.list` | DocRead | |
| 48 | `text.get` | DocRead | |
| 49 | `text.set` | DocWrite | content |
| 50 | `text.style.set` | DocWrite | |
| 51 | `text.fonts.list` | AppStateRead | `text.fonts` (5 apps) |
| 52 | `effect.list` | DocRead | |
| 53 | `effect.apply` | DocWrite | |
| 54 | `effect.set` | DocWrite | |
| 55 | `effect.remove` | DocWrite | |
| 56 | `effect.reorder` | DocWrite | |
| 57 | `mask.create` | DocWrite | |
| 58 | `mask.delete` | DocWrite | |
| 59 | `mask.set` | DocWrite | |
| 60 | `prop.get` | DocRead | path-addressed animatable property (EffectCraft, FilmCraft, AniCraft, MaterialCraft, TextureCraft) |
| 61 | `prop.set` | DocWrite | |
| 62 | `key.add` | DocWrite | |
| 63 | `key.delete` | DocWrite | |
| 64 | `key.set` | DocWrite | value/ease |
| 65 | `time.get` | ViewRead* | playhead and ranges |
| 66 | `time.set` | ViewWrite | playhead; not DocWrite |
| 67 | `marker.list` | DocRead | |
| 68 | `marker.add` | DocWrite | |
| 69 | `marker.set` | DocWrite | |
| 70 | `marker.delete` | DocWrite | |
| 71 | `guide.list` | DocRead | |
| 72 | `guide.add` | DocWrite | |
| 73 | `guide.delete` | DocWrite | |
| 74 | `view.get` | ViewRead* | zoom/scroll/active pane |
| 75 | `view.set` | ViewWrite | |
| 76 | `view.zoom` | ViewWrite | `{to: fit|selection|1:1|factor}` |
| 77 | `raster.read` | DocRead | bounded pixels (PhotoCraft `pixels.get`, 4 MiB cap) |
| 78 | `raster.write` | DocWrite | |
| 79 | `asset.import` | FsRead(Picked) → DocWrite | host-picked handle only |
| 80 | `export.render` | DocRead → FsWrite(Picked) or bytes | format plus options; returns bytes or writes a picked handle (PhotoCraft `document.export`, FigCraft `export.svg`) |
| 81 | `job.status` | Job | |
| 82 | `job.cancel` | Job | host-run long operations on behalf of a plugin |
| 83 | `history.label` | – | names the host-owned undo group for the current event; no raw undo/redo |
| 84 | `project.data.get/set` | DocRead/DocWrite | plugin-private document storage (already in all 3 adapters) |

\* `ViewRead` is a read of view state. It can be free or bundled with `SelectionRead`.

**Deliberately absent:** `undo`, `redo`, `history.*`, `prefs.*`, `shortcuts.*`, `tool.*` (tool switching), `plugin.*`, `file.open/save/close/revert`, raw `clipboard.*`, `app.quit`, `window.*`, `script.*`, `macro.*`/`actions.play`, `help.*`/URL opening and model downloads.

Clipboard and file I/O reach plugins only through host-mediated flows: a picked handle, or an explicit `clipboard.read`/`clipboard.write` permission that the host performs. They do not run as registry commands.

The adapters already overlap heavily with this list:

- EffectCraft's 36 ops, PhotoCraft's 37 and FigCraft's 85 cover 1–11, 28–29, 36, 41–44, 60–61, 65–68, 71–72, 77–80 and 84.
- The three adapters spell the same op differently: `marker.add` vs `comp.addMarker`, `node.update` vs `node.set`, `selection.set` vs `select.set`.

Fixing the canonical names now is cheaper than after third-party plugins ship.

---

## 7. Adapter coverage

The full adapter detail is in a companion fork's notes. These are the census-relevant numbers.

| Adapter | Crate | Ops exposed | Registry ids reached | Share | How | Permission classes | Generic dispatch? |
|---|---|---|---|---|---|---|---|
| EffectCraft (beta) | `effectcraft/crates/plugins` (`contract.rs:79-192`, `ops.rs:177-300`) | 36 (9 query, 20 edit, 4 UI…) + `raster.filter@1` | **28** (19 pass-through + 9 mapped) | **3.7%** of 757 | `Session::execute` with registry ids. Captured `comp` injected, args capped at 32 fields / 64 KiB, only numeric `prop.set` | `document.read/edit:{selection,document}`, `ui.window`, `app.events` | No (`shell.exec` asserted absent) |
| PhotoCraft (beta) | `photocraft/crates/plugins` + `engine/src/plugin_ops.rs:593-870` | 37 (13 query, 15 edit, 4 effect, 4 UI) + `raster.filter@1` | **≈106** (12 literal, 57 `filter.*` allowlist, 17 adjustments + 15 adjustment layers, masks) | **≈11%** of 954 | Hand-written match. Raw reads such as `pixels.get` bypass the registry. Reserved keys refused (`plugin_ops.rs:59`) | adds `files.read/write:picked` | No |
| FigCraft (beta) | `figcraft/crates/plugins` (`contract.rs:22-90`, `runtime.rs:302-343`) | ≈85 (15 hand-written + 67 registry ids under their own names) | **72** | **11.4%** of 630 | Id allowlist (`EDIT_COMMANDS` 61, `QUERY_COMMANDS` 6). `OpSpec` copies the engine's `label`/`params` text. JSON bounded at 256 KiB, depth 32 | `document.read`, `document.edit:document`, `ui.window` only (args ignored for scope) | No, but this is the closest existing thing to a registry bridge |
| VectorCraft (upstream, legacy) | `vectorcraft/crates/plugins` + `engine/src/cmd/plugin_call.rs`, `plugin_call/reviewed.rs` | per-manifest `commandAllowlist` | **up to 705** (ALLOWED); 13 DENIED; 47 denied prefixes | **≈68%** of 1,044 | Every command must be classified, enforced by a test (`plugin_call.rs:684`). Runs with `AutomationRoots::none()`. File-param keys denied at any depth | `doc.read`, `doc.write`, `effect.render`, `export.write`, `import.read`, … | Yes, allowlisted |
| AniCraft (upstream, legacy) | `anicraft/crates/plugins` + `engine/src/cmds_plugin.rs:32-97` | `{"op":"command",id}` | **≈274** (30 allowed namespaces) | **≈75%** of 365 | Namespace policy `plugin_may_call`; no grants | none | Yes, by namespace |

**Findings:**

1. **The beta adapters choose a small subset by hand and rebuild metadata by hand.** Each one restates kind, permission and arguments, and gets it slightly differently. Selecting is classified three ways:
   - EffectCraft `layer.select` needs `document.edit:document`.
   - PhotoCraft `layer.select` needs `document.read:document`.
   - FigCraft `selection.set` is a `Query` op needing `document.read`.

   EffectCraft's `time.set` needs `document.edit`, and FigCraft's `viewport.set` is free. The census explains why: the registries have no selection, view or app-state effect class, so each adapter guesses. The `SelectionWrite`/`ViewWrite` effects in §5.2 remove the guesswork.
2. **VectorCraft's reviewed-list model is the right governance pattern.** Every id must be classified before it ships, a test enforces it, and new commands are denied by default. It still keeps the classification outside the registry. Moving `ALLOWED`/`DENIED` into `Meta.exposure` in each app gives that model with no drift.
3. **The legacy registry bridges need dispositions under the suite replacement rule.** VectorCraft reaches 68% of its registry and AniCraft 75%. A bridge built from `Meta` can carry over VectorCraft's 705 allowed ids as `exposure = Plugin(app op)` where they map to canonical ops, and retire the rest with an explanation.
4. **The adapters agree on undo, which is good.** Each event or gesture is one labelled host-owned undo group. It is rolled back on failure, on the 5 s timeout and on revocation. Revalidation happens on every call. `Meta.undo = Step` commands fit this directly. `Undo::External` commands are the files PhotoCraft publishes per file at commit, and they need the separate consent the adapters already give file handles.

---

## 8. MCP safety model

A companion fork surveyed this; the summary follows.

| App | MCP crate | Transport and auth | Registry reachability | Path sandbox | Command gate | Other limits |
|---|---|---|---|---|---|---|
| EffectCraft | `crates/automation` | stdio; app control port loopback, **no token** (`apps/effectcraft/src/control_server.rs:27`) | all + raw `control` passthrough | opt-in `--automation-read/write-root` (`confine.rs:81-140`) | `REFUSED` (scripts, ScriptUI, `executeFile`, reveal, `editOriginal`, watch folder…), all `plugin.*`, UI tools, **only when confined** (`confine.rs:44-70`, `tools.rs:107-117`) | long-job progress/cancel, `catch_unwind` |
| PhotoCraft | `crates/automation` (rmcp) | stdio + `serve`; **challenge-response token** (`bridge.rs:188-214`), 16 connections, 1 MiB lines | all by id; file ids gated | **on by default**, cap-std, missing root = deny (`workspace.rs:1-60,557-579`) | `authorize_engine_command` (`workspace.rs:625-674`): `file.*` except ~13, path-granting `prefs.set`, `plugin.reload`; applied to `actions.play` steps and desktop automation | batch 256, 8 MiB response, render caps (`budgets.rs`) |
| FigCraft | `crates/mcp` | stdio; `--attach` loopback **no token** | all | opt-in, but **one choke point**: `file_access` + clippy ban on `std::fs` | network refused when confined (`network_refused`, 9 sites) | capped reads, symlink limit |
| FilmCraft | `crates/automation` (rmcp) | stdio; control port **requires `FILMCRAFT_CONTROL_TOKEN`** | all | **none** | none | panic catch |
| AniCraft | `crates/cli/src/mcp.rs` | stdio; per-run 64-hex token file | all | **none** | `confirm: true`, which the caller sets itself | 10,000 steps |
| LightCraft | `crates/mcp` | stdio; control loopback **no token** | all, twice (generic + `cmd_<id>` tools) | **none** (only refuses overwriting catalogued originals) | none | progress/cancel; exposes a `clipboard` tool |
| VectorCraft | `crates/mcp` | stdio; `--connect` 127.0.0.1:7979 **no token** | all + UI-only menu ids when connected | opt-in `AutomationRoots`, with an `unconfined()` escape for app folders | plugin grants need `InstallApproval`, which no command can set | busy/cancel intake |
| DesignCraft | `crates/mcp` | stdio; `--connect` loopback **no token** | all | opt-in; **cannot combine with `--connect`** | none | unknown-key refusal |
| MaterialCraft | `crates/automation` (rmcp) | stdio + `serve`; **token required** | all non-file | **on by default** (cap-std) | `authorize_command`: `file.*` and any `path`/`dir` param refused (`workspace.rs:167-178`) | batch 256 |
| TextureCraft | `crates/automation` (rmcp) | same as MaterialCraft | all non-file | **on by default** | `authorize_command` (`workspace.rs:134-149`) | batch 256 |
| SoundCraft | `crates/automation/src/mcp.rs` | stdio; `--connect` loopback **no token** | all | opt-in `AutomationRoots` | none | batch 1,000 |

**What an MCP client can do today that a plugin must not:**

| Capability | Examples | Census class | Count |
|---|---|---|---|
| Read and write arbitrary paths | `file.*`, `*.export*`, `open_file`, `import` (always in FilmCraft, LightCraft and AniCraft; by default in the opt-in apps) | file system (+path params) | 1,140 |
| Run code or other programs | `file.runScript`, `file.executeFile`, `scriptui.*`, `edit.editOriginal`, `layer.openSource`, `designcraft:script.run`, `photocraft:actions.play`, `anicraft:macro.*`, LightCraft `addToPhotos` (Apple Events) | process/OS | 78 |
| Open URLs, reveal files, print | `help.website`, `help.discord`, `file.revealInFinder`, `file.print` | process/OS / network | ⊂ above |
| Manage plugins, load native code | `plugin.install/remove/setEnabled` (VectorCraft, AniCraft), `soundcraft:engine.vst3_plugins {rescan}`, `mix.insert {plugin:"vst3:…"}` | plugin mgmt | 25 |
| Network and model downloads | `face/roto.model.download`, `transcript.downloadModel`, `tts.downloadVoices`, `denoise.models.download`, `lens.downloadDatabase`, `font.download`, publishing, live sessions | network / AI | 38+ |
| Write preferences and shortcuts | `prefs.set`, `prefs.reset`, `shortcuts.set` (only PhotoCraft blocks `integrations.readRoot/writeRoot`) | app state | 232 |
| Destroy work outside undo | `file.revert`, `file.discardRecovery`, `edit.purgeUndo`, `edit.clearHistory`, `project.delete`, `library.emptyRecentlyDeleted`, `app.quit` | hard-destructive | 107 |
| Drive the UI directly | `ui_click/key/type/menu_invoke`, `control`/`control_call`, `answer_dialog` (only EffectCraft refuses these while confined) | n/a | n/a |
| Unbalance history | `edit.undoGroup.begin/end` (VectorCraft) | undo/history | 57 |

**Takeaways for the bridge:**

- MCP is a trusted local operator channel. Its safety comes from the operator choosing flags, and most apps default to open. The plugin bridge must not inherit MCP's tool surface or its gates. It needs its own default-deny allowlist generated from `Meta`.
- Three patterns are worth reusing:
  - PhotoCraft, MaterialCraft and TextureCraft give directory handles fail-closed through `cap-std`.
  - FigCraft enforces at a choke point, so coverage of new commands comes by construction.
  - VectorCraft's `InstallApproval` gives host-side consent the guest cannot forge.
- Six control ports accept any local process: EffectCraft, FigCraft, LightCraft, VectorCraft, DesignCraft and SoundCraft. A native plugin helper (out of process, not OS-sandboxed) can therefore drive the whole app by connecting to 127.0.0.1. This is a bypass of plugin permissions today. Two mitigations:
  - Every control port gets a per-run token, PhotoCraft challenge-response or FilmCraft env-token style, and the token is withheld from plugin helper environments.
  - Native-plugin consent text states this residual risk.

---

## 9. Recommendations

1. **Make the registry the single source of truth for safety.** Introduce one shared `CommandMeta` (in craft-libs, built on VectorCraft's `apischema`) with mandatory `effects`, `scope`, `undo`, `run`, `destructive`, `params`/`result` schema, `exposure` and `since`. Add it to all 11 `CommandSpec` structs. Retire `journal`-as-query, `history`, `mutates` and `view`, keeping `journal` only for replay. Effort is about 6,200 declarations. This census's `cls`/`flags` columns pre-fill about 85% of `effects`.
2. **Gate in CI and fail closed at runtime.** Add a workspace test per app, in the style of VectorCraft's `every_command_is_reviewed`, that fails on any command without `Meta`. Add FigCraft-style choke points with lint bans for fs, net, process and clipboard. The dispatcher checks the reached effect against the declared and granted set, so a mis-declared command errors instead of leaking.
3. **Generate the bridge rather than writing it by hand.** Plugin-visible ops are `exposure = Plugin(..)` commands whose effects are a subset of `{DocRead, DocWrite, SelectionRead, SelectionWrite, ViewWrite}`. Permissions follow from effects, which requires two new classes in `craft-extensions/crates/ext-core/src/permission.rs`:
   - `document.read` ↔ DocRead
   - `document.edit:<scope>` ↔ DocWrite + `Meta.scope`
   - `selection.write` ↔ SelectionWrite (new)
   - `view.write` ↔ ViewWrite (new)

   Typed TypeScript and Rust bindings and MCP `inputSchema`s come from the same schemas. Path-typed params are refused or replaced by picked handles automatically.
4. **Adopt the §6 canonical vocabulary (84 ops)** as the portable plugin API. Each app's adapter maps canonical ops onto its registry ids through `Meta.exposure = Plugin(canonical("node.set"))`. Raw registry ids reach plugins only as `app.<appId>.<id>` under an explicit, per-app permission. Unify the three beta adapters' names now (`marker.add`, `node.set`, `selection.set`).
5. **Keep the denied set structural.** Plugins never get these effect classes: `PrefsWrite`, `Process`, `PluginMgmt`, `History` (raw undo/redo and undo groups), `UiWindow` beyond the adapter's own panel/window ops, `AppLifecycle`, `ModelInstall`, `Collab`, or `Net` (except a host-performed `network:<host>` fetch). The host mediates `FsRead`/`FsWrite` (picked handles only), `ClipboardRead`/`ClipboardWrite` (host-performed with a permission) and `Job` (host job handles with cancel).
6. **Normalise parameter shapes before generating bindings:**
   - `targets: [TargetRef]` replaces `layer`, `layers`, `ids`, `clips`, `items` and `id|name|#n`.
   - One colour object and one rational time type.
   - Document units declared by `doc.get`.

   FilmCraft's `NamedKeys` and LightCraft's `explicit_targets` already make the "explicit target skips the selection-based `enabled` check" semantics explicit. Make that the rule for every app.
7. **Bring the shell/UI commands into scope.** PhotoCraft has 146 `SHELL_COMMANDS`, and LightCraft, DesignCraft, FilmCraft, EffectCraft and SoundCraft have about 70–210 each. They hold `file.open`/`file.save` (PhotoCraft), zoom and panels. They need `Meta` too, or a plugin bridge and MCP will disagree about what exists. They are mostly `ViewWrite`/`UiWindow`, so plugins mostly do not need them.
8. **Close the MCP side doors that would undermine plugin permissions:**
   - Per-run tokens on all control ports.
   - Default-on sandboxing, as in the cap-std trio, for FilmCraft, LightCraft, AniCraft and the opt-in apps.
   - Remove `edit.undoGroup.*` from the public registry, or mark it `Internal`.
   - Turn AniCraft's `confirm: true` into host consent.
9. **Order of work:**
   1. Shared `Meta` type and CI test in craft-libs.
   2. Back-fill the three beta apps (EffectCraft, PhotoCraft, FigCraft) and regenerate their adapters from it.
   3. Back-fill MaterialCraft and TextureCraft (smallest, with no flags today).
   4. Back-fill VectorCraft (convert `reviewed.rs`) and AniCraft (convert the namespace policy) as their legacy systems are replaced.
   5. Back-fill FilmCraft, LightCraft, DesignCraft and SoundCraft.

   Re-run `scripts/` after each step to track coverage. The CSV `cls`, `flags` and `params_shape` columns serve as the burn-down list.

### Residual uncertainty

- Counts are static extractions. Exact for AniCraft and VectorCraft, validated by menu catalogs for FigCraft and EffectCraft, about ±20 for PhotoCraft.
- Runtime plugin-registered commands are excluded.
- Classification is heuristic (about 85–90% primary-class accuracy on a 70-row spot check). Network effects are undercounted by name matching.
- Nothing was executed against a running app. MCP and adapter behaviour is from source reading only.

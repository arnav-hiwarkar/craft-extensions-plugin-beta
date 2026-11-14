# Plugin capabilities by app

What each app lets a plugin do: the contribution kinds it runs, the contracts it provides, the
operations a session plugin may call and the permission each one needs, and what is **not**
available.

Sources:

- The generated contract pages [contracts/effectcraft.md](contracts/effectcraft.md),
  [contracts/photocraft.md](contracts/photocraft.md) and [contracts/figcraft.md](contracts/figcraft.md)
  (`<app>-cli plugin contract`), which hold every operation's arguments and results.
- The adapters: `effectcraft/crates/plugins/src/{contract.rs,ops.rs,runtime.rs}`,
  `photocraft/crates/plugins/src/{contract.rs,runtime.rs}` with
  `photocraft/crates/engine/src/{plugin_ops.rs,plugin_sessions.rs,plugin_filters.rs}`, and
  `figcraft/crates/plugins/src/{contract.rs,runtime.rs}`.

Where a contract page and the adapter code disagree, **the code wins**; each difference is listed
under [Where the contract pages and the code differ](#where-the-contract-pages-and-the-code-differ).

How permissions are enforced on each call is explained in
[how-it-works.md](how-it-works.md#permissions-are-checked-on-every-host-call).

This reference describes inspected source, not a local CLI run or GUI/platform verification.
See the [workflow maps](maps.md), [formats and versioning](formats-and-versioning.md), and
[native protocol](native-protocol.md). All three adapters enforce tree levels with `validate_at`
using the contribution's declared session level; this includes EffectCraft and FigCraft.

## Summary

| | EffectCraft | PhotoCraft | FigCraft |
|---|---|---|---|
| Session contract | `effectcraft.session@1.1` | `photocraft.session@1.1` | `figcraft.session@1.1` |
| Other contracts | `raster.filter@1.0`, `resource.preset@1.0`, `resource.lut@1.0`, `craft-theme@1.0` | `raster.filter@1.0`, `resource.preset@1.0`, `resource.lut@1.0`, `craft-theme@1.0` | `resource.font@1.0`, `craft-theme@1.0` |
| `raster.filter` (effects) | Yes: effects | Yes: destructive filters | **No** |
| Operations | 32 | 35 | 83 |
| File operations | None | Picker handles (`files.*`, `document.export`, `layer.import`) | None |
| Network or clipboard operations | None | None | None |
| Native helpers | command, panel, tool, event-hook | command, panel, tool, event-hook | command, panel, tool, event-hook |
| Windows | 4 per plugin (5th refused) | 32 surfaces (panels + windows) per app | 4 per plugin (5th silently dropped) |
| UI level enforced from the declared session minor | Yes | Yes | Yes |
| Project plugin references | Yes | No | Yes |
| Safe mode | `--safe-mode` / `EFFECTCRAFT_SAFE_MODE=1` | `PHOTOCRAFT_SAFE_MODE=1` | `FIGCRAFT_SAFE_MODE=1` |

Session level 1.1 carries UI level 1.1 (`crates/ext-core/src/ui/tree.rs`, `ui_level_for_session`).
`craft-ext`'s built-in table of these contracts is `crates/ext-cli/src/known_apps.rs`.

## Contribution kinds

| Kind | EffectCraft | PhotoCraft | FigCraft |
|---|---|---|---|
| `effect` (`raster.filter`) | Yes. Effects & Presets and Effect ▸ category; instances are ordinary animatable effect property groups. | Yes. Filter ▸ Plugins ▸ category. Applied destructively as one history step; RGB documents only. | **Not available.** No `raster.filter` contract; effect contributions are refused. |
| `command` | Plugins ▸ plugin ▸ label; command id `plugin.<id>.<contribution>` | Plugins ▸ plugin ▸ label | Plugins ▸ plugin ▸ label |
| `panel` | Host-drawn floating panel opened from the Plugins menu | Floating panel toggled from the Plugins menu; starts closed and stays closed when the user closes it | Docked panel toggled from the Plugins menu |
| `tool` | Yes. Pointer gestures in the composition viewer | Yes. Gestures on the canvas in image pixels | Yes. Plugin tool strip and Plugins menu; gestures in page units |
| `event-hook` | Yes, needs `app.events` | Yes, needs `app.events` | Yes, needs `app.events` (FigCraft delivers app events to any contribution that declares `app.events`) |
| `preset` | Yes. Each preset becomes a Plugins menu command that applies its effects to the selected layers in one undo step | Yes. Plugin filter presets | **Not available** |
| `lut` | Yes. Listed by `plugin.resources` as `plugin://` references | Yes. Color Lookup ▸ From plugins, `plugin.resources` | **Not available** |
| `font` | **Not available** | **Not available** | Slot listed, but fonts are not activated in this beta |
| `theme` | `craft-theme` provided, but theme contributions are refused ("not supported here yet") | `craft-theme` provided, no consumer | `craft-theme` provided, no slot, not loaded |
| `data` | **Not available** | **Not available** | **Not available** |

Theme and font activation are beta deferrals ([beta-design.md](beta-design.md#b12-evidence-limits-and-deferred-items)).

## Session events

| Event | EffectCraft | PhotoCraft | FigCraft | Needs `app.events` |
|---|---|---|---|---|
| `init` | Yes | Yes | Yes | no |
| `command` | Yes | Yes | Yes | no |
| `ui` | Yes | Yes | Yes | no |
| `tool` | Yes | Yes | Yes | no |
| `files.picked` | — | Yes (answer to `files.pick`) | — | no |
| `app` | `selection.changed`, `time.changed` (at most 4 per second), `document.changed`, `document.opened`, `render.finished` | `document.opened`, `selection.changed` | `selection.changed`, `document.opened`, `page.changed`, `document.changed` (read-only: edits are refused) | **yes** |
| `dispose` | Reserved, not sent | Reserved, not guaranteed | Listed, but not sent | no |

## EffectCraft operations

Abbreviations: **RD** = `document.read:document`, **ED** = `document.edit:document`.

| Operation | Kind | Permission |
|---|---|---|
| `project.summary`, `layer.tree`, `layer.list`, `comp.info`, `marker.list`, `prop.get` | Query | RD |
| `selection.get` | Query | `document.read:selection` |
| `time.get` | Query | free |
| `comp.new`, `comp.settings`, `time.set` | Edit | ED |
| `layer.addItem`, `layer.newText`, `layer.newShape`, `layer.newSolid`, `layer.newNull`, `layer.setSwitch`, `layer.select` | Edit | ED |
| `layer.rename` | Edit | `document.edit:selection` if the layer was selected when the event started, else ED |
| `marker.add`, `markers.set`, `markers.delete` | Edit | ED |
| `prop.set`, `prop.addKey`, `prop.reset` | Edit | ED |
| `effect.apply` | Edit | `document.edit:selection` |
| `effect.remove`, `effect.reorder` | Edit | ED |
| `ui.panel.set` (own panels), `ui.notify`, `canvas.overlay.set` (own overlay) | Ui | free |
| `ui.window.open` | Ui | `ui.window` |

Notes from the code (`effectcraft/crates/plugins/src/ops.rs`):

- Parameter values for `prop.set`, `prop.addKey` and `effect.apply` must be numbers, booleans or
  numeric vectors of up to 4 values. Strings and paths are refused; string properties are redacted
  on read.
- Effect pixels are premultiplied RGBA `f32` in the project's working space (`linear-srgb` when
  Linearize Working Space is on, else `srgb`).
- While a plugin tool gesture is open, journaled edits from outside the plugin (Undo, menu
  commands) are refused (`effectcraft/crates/engine/src/ext.rs`).

## PhotoCraft operations

| Operation | Kind | Permission |
|---|---|---|
| `document.info`, `layer.list`, `layer.get`, `pixels.get`, `histogram.get`, `guides.list`, `channels.list`, `history.list` | Query | RD |
| `documents.list` | Query | `document.read:all` |
| `selection.get`, `selection.mask.get` | Query | `document.read:selection` |
| `tool.state`, `plugin.resources` | Query | free |
| `layer.rename`, `layer.create`, `layer.delete`, `layer.set`, `pixels.set` | Edit | ED |
| `layer.select` | Edit | RD (changes only which layer is selected) |
| `selection.set`, `selection.save`, `guide.add`, `guides.clear`, `mask.add` | Edit | ED |
| `filter.apply`, `adjustment.apply` | Edit | ED (`filter.apply` runs only allowlisted built-in filters) |
| `layer.import` | Edit | `files.read:picked` and ED |
| `files.pick` | Effect | `files.read:picked` for `open`; `files.write:picked` for `folder` and `save` |
| `files.read` | Effect | `files.read:picked` |
| `files.write` | Effect | `files.write:picked` |
| `document.export` | Effect | `files.write:picked` and RD |
| `ui.panel.set`, `ui.notify`, `canvas.overlay.set` | Ui | free |
| `ui.window.open` | Ui | `ui.window` |

Notes from the code:

- Operations run on PhotoCraft's document thread as canonical engine commands, and are authorized
  there (`photocraft/crates/plugins/src/document_thread.rs`,
  `photocraft/crates/engine/src/plugin_sessions.rs`).
- Plugin filters refuse non-RGB documents (CMYK, Lab, indexed), layer-mask targets, channel subsets,
  Quick Mask, and profiles that cannot be converted to sRGB. Filters receive premultiplied RGBA
  `f32`: `srgb` for 8- and 16-bit documents, `linear-srgb` for 32-bit. The result commits as one
  history step only if enablement and grants are unchanged
  (`photocraft/crates/engine/src/plugin_filters.rs`). Smart filters are not supported.
- Session pixel operations use straight (not premultiplied) RGBA in the document's values,
  base64-encoded.
- While a plugin tool gesture is open, journaled commands other than `view.*` and plugin commands are
  refused (`photocraft/crates/engine/src/session/execute.rs`).

### File access in PhotoCraft

PhotoCraft is the only app with file operations. A plugin never sees a path.

1. `files.pick {mode}` asks the person through the host's dialog, after the event commits. At most
   4 picks per event and 32 pending.
2. The answer arrives as a `files.picked` event with an opaque handle (`h<n>`) and a display name.
   A pick reply is single-use and bound to the document and its epoch; it is checked against the live
   grants. The handle itself works only for that plugin and that file or folder, until the app quits.
3. `files.read` and `layer.import` need a handle picked in `open` mode. Save targets must be safe
   names; hidden and executable names and symlinked targets are refused.
4. Writes are staged during the event, then **published one file at a time before accepting the
   document/UI commit**, with per-file
   atomic replacement, in order. Before each publication and after the last one the host checks live
   grants, cancellation, captured scope and the event deadline.
5. **This is not batch-atomic.** If a later write or the final check fails, the commit returns an
   error that reports how many files were already published; those files stay. Document and UI
   changes roll back. A synchronous write in progress cannot be interrupted.
6. File operations are refused during tool gestures, because rollback cannot undo outside I/O.

(`photocraft/crates/engine/src/plugin_ops.rs`, `plugin_sessions.rs`.)

## FigCraft operations

FigCraft's list is a closed list copied from its engine command registry
(`figcraft/crates/plugins/src/contract.rs`, `figcraft_engine::find_command` in
`figcraft/crates/engine/src/cmd.rs`). Every operation except `selection.get` uses document scope;
`requires` ignores its arguments, so a plugin cannot claim selection scope.

| Operations | Kind | Permission |
|---|---|---|
| `document.info`, `page.list`, `page.select`, `node.get`, `node.children`, `node.find`, `selection.set`, `style.list`, `variable.list`, `image.presets`, `comment.list`, `export.svg` | Query | RD |
| `selection.get` | Query | `document.read:selection` |
| `viewport.get`, `viewport.set` | Query | free |
| `node.create`, `node.update`, `node.delete`, `node.reparent`, `node.reorder`, `edit.duplicate` | Edit | ED |
| `page.add`, `page.delete`, `page.duplicate`, `page.move`, `page.rename`, `page.setBackground` | Edit | ED |
| `component.*` (7), `instance.*` (3) | Edit | ED |
| `style.*` (6 edits: apply, create, delete, detach, duplicate, update) | Edit | ED |
| `variable.*` (10 edits) | Edit | ED |
| `vector.create`, `vector.flatten`, `vector.outlineStroke`, `bool.union`, `bool.subtract`, `bool.intersect`, `bool.exclude` | Edit | ED |
| `text.create`, `text.onPath`, `text.pathOptions`, `layout.add`, `layout.remove`, `layout.setTrack` | Edit | ED |
| `object.group`, `object.ungroup`, `object.resizeToFit`, `guide.add`, `guide.move`, `guide.remove`, `guide.clear` | Edit | ED |
| `image.place`, `image.adjust`, `image.replace` | Edit | ED (a `path` argument is refused) |
| `comment.add`, `comment.reply`, `comment.resolve` | Edit | ED (`author` is refused) |
| `ui.panel.set`, `ui.notify`, `canvas.overlay.set` | Ui | free |
| `ui.window.open` | Ui | `ui.window` |

Notes from the code (`figcraft/crates/plugins/src/runtime.rs`):

- `node.update` maps to the engine's `node.set`, `node.delete` to `edit.delete`, `selection.set` to
  `select.set` and `viewport.set` to `view.set`. Other operations run the engine command of the same
  name.
- `export.svg` returns an SVG string of at most 4 MiB; it does not touch the file system.
- During a `document.changed` event, Edit operations are refused to prevent feedback loops.
- An event may make at most 64 UI changes.

## Not available

| Capability | Status |
|---|---|
| `raster.filter` / effects in FigCraft | Not available. FigCraft does not provide the contract. |
| Native effects in any app | Not available. Native packages cannot contribute effects. |
| Network requests | No app offers a network operation. The `network:<host>` permission can be declared and granted but no operation uses it. |
| Clipboard | No app offers a clipboard operation. `clipboard.read` and `clipboard.write` exist as permissions only. |
| File access in EffectCraft and FigCraft | No file operations. Paths in arguments are refused. |
| Raw paths, processes, sockets, environment | Not reachable from wasm: only the `craft.*` imports exist. Only native helpers can do these, with full-access consent. |
| Undo, history, settings, plugin management | Never exposed as operations. |
| Calling other plugins | Not available (beta deferral). |
| Temporal frame access, tiling or region requests for filters | Not available (beta deferral). |
| Theme and font activation | Not available (beta deferral). |

## Catalogue refresh by app

| App | Schedule |
|---|---|
| EffectCraft | One scheduler per process; repeats 6 hours after the last successful refresh. Failures retry after 5 min, 30 min, then every 2 h (`effectcraft/crates/plugins/src/lib.rs`). |
| PhotoCraft | At start-up, then every 6 hours (`photocraft/crates/plugins/src/lib.rs`). |
| FigCraft | When the last refresh is older than 6 hours; failures retry after 60 s, doubling up to 15 min; off in safe mode (`figcraft/crates/plugins/src/lib.rs`). |

## Where the contract pages and the code differ

The operation lists, kinds and permission texts of the three contract pages match the adapters. The
differences are in behaviour and slot descriptions:

| App | Contract page says | Code does |
|---|---|---|
| EffectCraft | `lut`: "absolute `.cube` paths" | `plugin.resources` lists `plugin://<id>/<contribution>/<file>` references, resolved by the host (`effectcraft/crates/plugins/src/runtime.rs`, `lib.rs`). |
| EffectCraft | `theme`: "Stored and listed" | The loader refuses theme contributions as not supported yet. |
| PhotoCraft | `files.write`: "≤ 1 MiB" | No per-operation size check in `plugin_ops.rs`; the 1 MiB host argument cap and a 512 MiB staging budget apply. |
| PhotoCraft | (not stated) | No four-window limit; at most 32 surfaces per app and 4 MiB of trees in total. |
| FigCraft | `dispose`: "The worker is stopping" | `dispose` is never sent; workers are dropped. |
| FigCraft | Overlays "cleared when a tool gesture ends" | An overlay set outside a gesture stays until the next gesture ends, a resume or a reload. |
| FigCraft | "Coordinates are document units" | Tool and overlay coordinates are page units. |
| FigCraft | (not stated) | A fifth window is silently dropped, and the call still succeeds. |

Source comment that is out of date: `crates/ext-host/src/native.rs` says permissions are "moot" for
native helpers. In all three apps each native call still passes through `authorize`, so declared and
granted permissions are checked.

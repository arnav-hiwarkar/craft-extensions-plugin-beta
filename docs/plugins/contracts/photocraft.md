# PhotoCraft plugin contract

Refreshed from literal adapter metadata by `scripts/photocraft-contract-source.py` without compiling PhotoCraft. The Rust adapter remains authoritative.

## Contracts provided

| Contract | Version |
|---|---|
| `photocraft.session` | 1.1 |
| `raster.filter` | 1.0 |
| `resource.preset` | 1.0 |
| `resource.lut` | 1.0 |
| `craft-theme` | 1.0 |

## Where contributions appear

| Kind | Placement |
|---|---|
| `effect` | Filter ▸ Plugins ▸ category |
| `command` | Plugins ▸ plugin ▸ label |
| `panel` | A floating plugin panel, toggled by Plugins ▸ plugin ▸ label |
| `tool` | Plugins ▸ plugin ▸ label selects it; the canvas sends its gestures |
| `event-hook` | App event subscriptions (needs `app.events`) |
| `preset` | Plugin filter presets |
| `lut` | Color Lookup ▸ From plugins, `plugin.resources` |

## Operations (session plugins: `craft::call(op, args)`)

### `document.info` (Query)

The active document.

Permission: document.read:document

```json
// args
{}
// result
{"id": 3, "name": "photo.jpg", "width": 1920, "height": 1080, "mode": "rgb", "depth": 8, "resolution": 72, "revision": 12, "active_layer": 7, "layer_count": 4, "has_selection": false}
```

### `documents.list` (Query)

Every open document (metadata only); `active` marks the one plugin operations act on.

Permission: document.read:all

```json
// args
{}
// result
[{"id": 3, "name": "photo.jpg", "width": 1920, "height": 1080, "active": true}]
```

### `layer.list` (Query)

The layers, top to bottom, with groups expanded (`parent` names the group; at most 64 levels).

Permission: document.read:document

```json
// args
{}
// result
[{"id": 7, "name": "Sky", "kind": "pixel", "parent": null, "visible": true, "opacity": 100, "blend": "normal", "bounds": [0, 0, 1920, 600], "has_mask": false, "clipped": false, "locked": false}]
```

### `layer.get` (Query)

One layer (default: the active layer), with the fields of `layer.list`.

Permission: document.read:document

```json
// args
{"layer": 7}
// result
{"id": 7, "name": "Sky", "kind": "pixel", …}
```

### `selection.get` (Query)

The layer selection and the pixel selection's bounds, captured when the event started.

Permission: document.read:selection

```json
// args
{}
// result
{"layers": [7], "active": 7, "has_selection": true, "bounds": [10, 10, 200, 120]}
```

### `selection.mask.get` (Query)

The pixel selection's coverage as 8-bit gray (255 = selected), base64. `rect` defaults to the whole image. At most 4 Mi pixels.

Permission: document.read:selection

```json
// args
{"rect": [0, 0, 256, 256]}
// result
{"x": 0, "y": 0, "width": 256, "height": 256, "format": "gray8", "data": "AAAA…"}
```

### `pixels.get` (Query)

Pixels of a pixel layer (default: the active layer) or, with `composite: true`, of the visible image. `rect` [x, y, width, height] in image pixels defaults to the whole image and is clipped to it. Straight RGBA in the document's own values; `format` `rgba8` (default), `rgba16` (little-endian u16) or `rgba32f` (little-endian f32, linear light in 32-bit documents). `data` is base64. At most 4 MiB of pixel data per call. RGB documents only.

Permission: document.read:document

```json
// args
{"layer": 7, "rect": [0, 0, 64, 64], "format": "rgba8"}
// result
{"x": 0, "y": 0, "width": 64, "height": 64, "format": "rgba8", "data": "…"}
```

### `histogram.get` (Query)

256-bin histograms of a pixel layer (default: the active layer) or the visible image (`composite: true`). `rect` limits the area; the pixel selection limits it too unless `selection: false`. Transparent pixels are not counted. Luminosity is Rec. 709. Computed in bounded tiles; at most 64 Mi pixels per call (pass a smaller `rect` for larger images).

Permission: document.read:document

```json
// args
{"composite": true, "selection": true}
// result
{"count": 2073600, "red": [256 counts], "green": [...], "blue": [...], "luminosity": [...], "mean": {"red": 118.2, "green": 120.9, "blue": 99.1, "luminosity": 117.4}}
```

### `guides.list` (Query)

The guides in image pixels.

Permission: document.read:document

```json
// args
{}
// result
{"horizontal": [100.0], "vertical": [960.0]}
```

### `channels.list` (Query)

The colour channels and saved alpha channels.

Permission: document.read:document

```json
// args
{}
// result
[{"index": 0, "name": "Red", "kind": "color"}, {"index": 3, "name": "Alpha 1", "kind": "alpha"}]
```

### `history.list` (Query)

The undo history labels, oldest first; `current` counts the steps that can be undone.

Permission: document.read:document

```json
// args
{}
// result
{"entries": ["Open", "Levels"], "current": 2}
```

### `tool.state` (Query)

The tool gesture under way (the last `tool` event's payload), or null.

Permission: free

```json
// args
{}
// result
{"tool": "gradient", "phase": "update", "at": [120.5, 40.0], "start": [10.0, 10.0], "modifiers": {"shift": true}, "zoom": 0.5}
```

### `layer.rename` (Edit)

Rename a layer (default: the active layer). Names are at most 4096 bytes.

Permission: document.edit:document

```json
// args
{"layer": 7, "name": "Sky"}
// result
null
```

### `layer.create` (Edit)

Add a layer above the active one and make it active: `pixel` (empty), `group`, or `fill` (a solid colour, straight RGBA 0–1). At most 100 per event.

Permission: document.edit:document

```json
// args
{"kind": "pixel", "name": "Gradient", "opacity": 100, "blend": "normal"}
// result
{"id": 9}
```

### `layer.delete` (Edit)

Delete a layer.

Permission: document.edit:document

```json
// args
{"layer": 9}
// result
null
```

### `layer.set` (Edit)

Change a layer's name, visibility, opacity or fill opacity (0–100) or blend mode (`normal`, `multiply`, `screen`, `overlay`, `soft-light`, `hard-light`, `color-dodge`, `color-burn`, `darken`, `lighten`, `difference`, `exclusion`, `hue`, `saturation`, `color`, `luminosity`).

Permission: document.edit:document

```json
// args
{"layer": 9, "visible": true, "opacity": 60, "blend": "multiply"}
// result
null
```

### `layer.select` (Edit)

Make a layer the active (and only selected) layer.

Permission: document.read:document

```json
// args
{"layer": 9}
// result
null
```

### `pixels.set` (Edit)

Write straight RGBA pixels into a pixel layer (default: the active layer) at [x, y]. `format` and `data` as `pixels.get`; the area must lie inside the image. With `selection: true` the pixels blend in through the pixel selection. Locked layers are refused; locked transparency keeps alpha. Arguments are at most 1 MiB, so write large images in tiles (one event is still one undo step).

Permission: document.edit:document

```json
// args
{"layer": 9, "x": 0, "y": 0, "width": 64, "height": 64, "format": "rgba8", "data": "…", "selection": false}
// result
null
```

### `selection.set` (Edit)

Change the pixel selection: `{"all": true}`, `{"none": true}`, a `rect` [x, y, width, height], an `ellipse` [x, y, width, height], or a `mask` (gray8 base64 with x, y, width, height). `mode` is `replace` (default), `add`, `subtract` or `intersect`.

Permission: document.edit:document

```json
// args
{"rect": [10, 10, 100, 50], "mode": "add"}
// result
null
```

### `selection.save` (Edit)

Save the pixel selection as a new alpha channel.

Permission: document.edit:document

```json
// args
{"name": "Sky mask"}
// result
null
```

### `guide.add` (Edit)

Add a guide at a position in image pixels (within ten image sizes of the canvas; at most 1000 guides).

Permission: document.edit:document

```json
// args
{"orientation": "vertical", "position": 960}
// result
null
```

### `guides.clear` (Edit)

Remove every guide.

Permission: document.edit:document

```json
// args
{}
// result
null
```

### `mask.add` (Edit)

Add a layer mask to a layer (default: the active layer): `reveal` is `all`, `none` or `selection`.

Permission: document.edit:document

```json
// args
{"layer": 7, "reveal": "selection"}
// result
null
```

### `filter.apply` (Edit)

Run an allowlisted built-in pixel filter on the active layer: `{filter, params}`. Filters that read files, other documents or channels, or run other commands, are refused.

Permission: document.edit:document

```json
// args
{"filter": "filter.blur.gaussianBlur", "params": {"radius": 4}}
// result
filter result
```

### `adjustment.apply` (Edit)

Apply a built-in adjustment to the active layer, or add it as an adjustment layer with `as_layer: true`: brightnessContrast, levels, curves, exposure, vibrance, hueSaturation, colorBalance, blackWhite, photoFilter, channelMixer, invert, posterize, threshold, gradientMap, selectiveColor; destructive only: desaturate, equalize. `params` as the adjustment's command (at most 16 KiB).

Permission: document.edit:document

```json
// args
{"adjustment": "levels", "params": {"inputBlack": 10, "inputWhite": 240}, "as_layer": true}
// result
adjustment result
```

### `files.pick` (Effect)

Ask the person to pick a folder (`folder`), a file to write (`save`, with a suggested `name`) or an image to read (`open`). Returns at once with a `request` number; the answer arrives later as a `files.picked` event with an opaque `handle` (or `cancelled`). Handles last until the app quits and only work for the plugin that asked. Headless: `photocraft-cli run --pick <PATH>` answers in order.

Permission: `files.read:picked` for `open`; `files.write:picked` for `folder` and `save`

```json
// args
{"mode": "folder", "title": "Export layers to"}
// result
{"request": 1}
```

### `files.write` (Effect)

Write a small file (≤ 1 MiB, `text` or base64 `data`) to a picked save file, or into a picked folder as `name` (a plain file name; hidden and executable names are refused).

Permission: files.write:picked

```json
// args
{"handle": "h1", "name": "layers.json", "text": "{…}"}
// result
{"bytes": 1234}
```

### `files.read` (Effect)

Read a picked file (at most 4 MiB), base64.

Permission: files.read:picked

```json
// args
{"handle": "h2"}
// result
{"name": "notes.txt", "bytes": 120, "data": "…"}
```

### `document.export` (Effect)

Export the image, or one layer (`layer`), as an image file: to a picked save file, or into a picked folder as `name`. The format follows the extension: png, jpg, jpeg, tif, tiff, webp or psd. At most 500 exports per event.

Permission: files.write:picked and document.read:document

```json
// args
{"handle": "h1", "name": "Sky.png", "layer": 7}
// result
{"bytes": 52311, "width": 1920, "height": 600}
```

### `layer.import` (Edit)

Place a picked image file as a new embedded layer.

Permission: files.read:picked and document.edit:document

```json
// args
{"handle": "h2"}
// result
place result
```

### `ui.panel.set` (Ui)

Draw a panel your manifest declares (UI level 1.1); published when the event commits.

Permission: free

```json
// args
{"panel": "levels", "tree": {…}}
// result
{"rev": 4}
```

### `ui.window.open` (Ui)

Open or redraw a floating window (at most 32 surfaces per app).

Permission: ui.window

```json
// args
{"id": "about", "title": "About", "tree": {…}}
// result
{"rev": 5}
```

### `ui.notify` (Ui)

Show a short notification (at most 32 queued, 4000 characters).

Permission: free

```json
// args
{"message": "Exported 4 layers"}
// result
null
```

### `canvas.overlay.set` (Ui)

Draw guides over the canvas: a UI draw list (no images) whose coordinates are image pixels; stroke widths and text sizes are screen points. Published when the event commits, cleared when a tool gesture ends. At most 20 000 ops and 1 MiB.

Permission: free

```json
// args
{"ops": [{"op": "line", "from": [0, 0], "to": [100, 100], "width": 1, "rgba": [1, 0, 0, 1]}]}
// result
null
```

### `plugin.resources` (Query)

The enabled presets and LUTs of every plugin.

Permission: free

```json
// args
{}
// result
[{"plugin": "…", "id": "film-warm", "label": "Film warm", "kind": "lut", "files": ["…"]}]
```

## Events (`craft_event`)

| Event | Payload | Needs `app.events` | Meaning |
|---|---|---|---|
| `init` | `{"event":"init","contribution"}` | no | After the plugin loads and whenever plugins reload; draw panels here |
| `command` | `{"event":"command","id","params"}` | no | A command contribution ran (menu, `photocraft-cli run --cmd`, automation) |
| `ui` | `{"event":"ui","surface","rev","widget","value"}` | no | A panel or window widget changed (see the UI reference for values) |
| `tool` | `{"event":"tool","tool","phase":"begin&#124;update&#124;commit&#124;cancel","at":[x,y],"start":[x,y],"modifiers":{shift,alt,ctrl,cmd},"button","pressure","zoom"}` | no | A gesture of your selected tool, in image pixels; `zoom` is screen points per image pixel. Escape sends `cancel` |
| `files.picked` | `{"event":"files.picked","request","handle","name"} or {…,"cancelled":true}` | no | The answer to `files.pick` |
| `app` | `{"event":"app","name","payload"}` | yes | document.opened, selection.changed (layers or pixels) |
| `dispose` | `{"event":"dispose"}` | no | Reserved lifecycle event; beta shutdown does not guarantee guest notification |

## Notes

Each session event is one transaction and one undo step labelled "<label> · <plugin>"; an event that fails, times out, is revoked or finds the document changed rolls back. A tool gesture (begin … commit) is one undo step; Escape, a failed event, a document switch or revocation mid-gesture rolls the whole gesture back. Coordinates are image pixels (fractional; the top-left pixel spans 0–1). While a plugin tool is selected the canvas still zooms and pans (wheel, pinch, Space, middle button, the Zoom keys); choosing any built-in tool deselects it. From the CLI a tool command takes `{"points": [[x, y], …]}` (begin at the first point, commit at the last; `"cancel": true` cancels instead). A plugin whose events fail three times in a row is paused; the Plugins window shows it with Resume, as does the engine command `plugin.resume {"plugin": id}` (pauses last for one process).

Pixel operations work on RGB documents (8, 16 and 32 bit). Pixel data is straight (not premultiplied) RGBA in the document's own values, base64-encoded; a 32-bit document's values are linear light. Plugin filters, by contrast, receive premultiplied RGBA f32 in sRGB for integer depths or linear sRGB for float, with CMS conversion. Other modes are disabled with a reason. Filters apply destructively as one history step; plugin smart filters are unsupported.

Picked files: file operations (`files.*`, `document.export`, `layer.import`) are refused during tool gestures because document rollback cannot undo external I/O. A plugin never sees a path. `files.pick` returns a request number, and the person's answer arrives as a `files.picked` event with an opaque handle that only that plugin can use, for that file or folder only, until the app quits. Writes are staged until event commit, then published in order using per-file atomic replacement without following a target symlink. Live grants, cancellation, captured scope and the event deadline are checked before each publication and after the last one. A synchronous file write cannot be interrupted; cancellation during that write may leave it published. On a write or final validation failure, commit returns an error and document/UI changes roll back; earlier published files remain and the error reports their count. The failing destination may also have changed if a final durability flush failed. This is not a multi-file atomic transaction. Hidden and executable save names are refused for both save and folder handles. Plugins get no raw file system, network, shell or process access; native plugins (full access, separate consent) are the only way to get it.

Effect dialogs use Form schema 1 (`contribution.form`, optional worker `craft_describe`). Filter context `params` and CLI/automation parameters are Values keyed by param key: choice/radio option ids, straight RGBA including HDR RGB, and `[x,y]` points in their declared space. Defaults are filled; unknown keys and invalid values are refused. `layer`, `target`, `coalesce` and the `__` prefix are host-reserved command keys.

The host draws all form kinds, units, help and sections, evaluates visible/enabled conditions, and calls optional `craft_update` on workers after edits. Event fuel, grant checks and a 100 ms deadline bound form calls. Patches apply atomically at the current revision; stale edit results are discarded. Plugin presets accept `{id,label,effects:[{effect,values}]}` single-effect recipes and legacy direct `{id,label,effect,values}` records inside a `presets` array; the dialog offers matching, validated presets and omits multi-effect recipes.

Panels, windows and filter dialogs are drawn by the shared renderer (`craft_ext_ui::plugin_ui`, UI level 1.1) with the engine chosen in Preferences ▸ Plugins (egui by default); plugins cannot choose or detect it. A contribution declaring `photocraft.session@1.0` may only send level-1.0 trees. No framework component library is registered: every `component` node draws its fallback. Sessions execute on workers; document operations are authorized and committed on the document thread.

## Error codes

`-1` invalid input, `-2` permission denied, `-3` not found, `-4` stale or busy, `-5` cancelled, `-6` over budget, `-7` internal.

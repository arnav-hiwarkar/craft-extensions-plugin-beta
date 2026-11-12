# EffectCraft plugin contract

Generated from the `effectcraft` adapter by `craft-ext contract-docs`. Do not edit by hand.

## Contracts provided

| Contract | Version |
|---|---|
| `craft-theme` | 1.0 |
| `effectcraft.session` | 1.0 |
| `raster.filter` | 1.0 |
| `resource.lut` | 1.0 |
| `resource.preset` | 1.0 |

A contribution declaring `contract = "name@major.minor"` works here when the major matches and this minor is equal or higher.

## Where contributions appear

| Kind | Placement |
|---|---|
| `effect` | Effects & Presets and Effect ▸ under the contribution's category (default **Plugins**); instances are ordinary effect property groups (animatable, expressions, Effect Controls). |
| `command` | **Plugins ▸ <plugin name> ▸ <label>**, and the command id `plugin.<plugin id>.<contribution>` for agents and the CLI. |
| `panel` | A host-drawn panel opened from **Plugins ▸ <plugin name> ▸ <label>** (beta: a floating panel). |
| `event-hook` | Receives `app` events (needs `app.events`). |
| `preset` | Each preset becomes a command in **Plugins ▸ <plugin name>** that applies its effects to the selected layers (one undo step); also listed by `plugin.resources`. |
| `lut` | Listed by `plugin.resources` (absolute `.cube` paths for Apply Color LUT's file parameter). |
| `theme` | Stored and listed; applied once craft-theme lands (deferred). |

## Operations (session plugins: `craft::call(op, args)`)

### `selection.get` (Query)

The layers selected in the active composition when this event started.

Permission: `document.read:selection`

```json
// args
{}
// result
{"layers": [{"id": 12, "name": "Title", "index": 1}]}
```

### `comp.info` (Query)

The active composition: id, name, size, frame rate, duration, work area and its layers.

Permission: `document.read:document`

```json
// args
{}
// result
{"id": 3, "name": "Main", "width": 1920, "height": 1080, "frameRate": 30, "duration": 10, "layers": [...]}
```

### `layer.list` (Query)

Every layer of the active composition, top to bottom.

Permission: `document.read:document`

```json
// args
{}
// result
{"layers": [{"id": 12, "name": "Title", "index": 1}]}
```

### `layer.rename` (Edit)

Renames a layer of the active composition (name: 1–256 characters).

Permission: `document.edit:selection` when the layer was selected when the event started, else `document.edit:document`

```json
// args
{"layer": 12, "name": "Lower third"}
// result
null
```

### `marker.add` (Edit)

Adds a composition marker to the active composition, at `time` (seconds) or the current time.

Permission: `document.edit:document`

```json
// args
{"label": "Note", "time": 2.5}
// result
{"time": 2.5}
```

### `effect.apply` (Edit)

Applies an effect (built-in id such as `ec.blur.gaussian`, or `<plugin id>/<effect>`) to the selected layers, then sets the given parameters.

Permission: `document.edit:selection`

```json
// args
{"effect": "ec.blur.gaussian", "params": {"blurriness": 12}}
// result
{"effects": [41], "paths": ["effects/#2"]}
```

### `time.get` (Query)

The current time of the active composition, in seconds.

Permission: free

```json
// args
{}
// result
{"time": 1.5, "comp": 3}
```

### `ui.panel.set` (Ui)

Replaces the content of one of this plugin's declared panels with a UI tree (see craft-ext-core `ui::Node`).

Permission: free (own panels only)

```json
// args
{"panel": "notes", "tree": {"type": "column", "children": [{"type": "label", "text": "Hi"}]}}
// result
{"rev": 4}
```

### `ui.window.open` (Ui)

Opens (or replaces) a host-drawn floating window with a UI tree. At most 4 windows per plugin.

Permission: `ui.window`

```json
// args
{"id": "about", "title": "About", "tree": {"type": "label", "text": "Hello"}}
// result
{"rev": 1}
```

### `ui.notify` (Ui)

Shows a short message (at most 300 characters), prefixed with the plugin's name.

Permission: free

```json
// args
{"message": "Done", "error": false}
// result
null
```

## Events (`craft_event`)

| Event | Payload | Needs `app.events` | Meaning |
|---|---|---|---|
| `init` | `{"event":"init"}` | no | Once after the plugin loads: draw your panels. |
| `command` | `{"event":"command","id":"<contribution>"}` | no | A command contribution ran (one undo step). |
| `ui` | `{"event":"ui","surface":"<panel>","rev":4,"widget":"<id>","value":null}` | no | A widget in your panel or window changed; stale revisions are dropped. |
| `app` | `{"event":"app","name":"selection.changed","payload":{"layers":[12]}}` | yes | `selection.changed`, `time.changed` (at most 4 per second), `document.opened`, `render.finished`. |
| `dispose` | `{"event":"dispose"}` | no | The plugin is being turned off or reloaded. |

## Notes

Pixels: premultiplied RGBA f32 in the project's working space (`color` is `linear-srgb` when Linearize Working Space is on, else `srgb`); values may exceed 1 in 32 bpc projects. `time` is layer time in seconds. `scale` is buffer pixels per layer pixel (previews at reduced resolution have scale < 1 and `quality: "draft"`). `origin` is the buffer's top-left inside the layer's full frame and `full` the layer's size, both in buffer pixels. Point parameters: defaults are fractions of the layer size ([0.5, 0.5] = centre); values reach the filter in buffer pixels. Angle parameters are degrees. Select parameters are 0-based indices. Plugin effects are treated as time-dependent by the frame caches. Ops act on the composition and selection captured when the event started. Effect ids are `<plugin id>/<contribution>`.

## Error codes

`-1` invalid input, `-2` permission denied, `-3` not found, `-4` stale or busy, `-5` cancelled, `-6` over budget, `-7` internal.

Tools are not supported in EffectCraft yet. Tool contributions are unavailable in compatibility and setup.

# EffectCraft plugin contract

Generated from the `effectcraft` adapter by `<app>-cli plugin contract`. Do not edit by hand.

## Contracts provided

| Contract | Version |
|---|---|
| `raster.filter` | 1.0 |
| `effectcraft.session` | 1.1 |
| `resource.preset` | 1.0 |
| `resource.lut` | 1.0 |
| `craft-theme` | 1.0 |

A contribution declaring `contract = "name@major.minor"` works here when the major matches and this minor is equal or higher.

## Where contributions appear

| Kind | Placement |
|---|---|
| `tool` | Select from Plugins; primary pointer draws in the composition viewer. Middle-button and Space pan, wheel zoom remain available. |
| `effect` | Effects & Presets and Effect ▸ under the contribution's category (default **Plugins**); instances are ordinary effect property groups (animatable, expressions, Effect Controls). |
| `command` | **Plugins ▸ <plugin name> ▸ <label>**, and the command id `plugin.<plugin id>.<contribution>` for agents and the CLI. |
| `panel` | A host-drawn panel opened from **Plugins ▸ <plugin name> ▸ <label>** (beta: a floating panel). |
| `event-hook` | Receives `app` events (needs `app.events`). |
| `preset` | Each preset becomes a command in **Plugins ▸ <plugin name>** that applies its effects to the selected layers (one undo step); also listed by `plugin.resources`. |
| `lut` | Listed by `plugin.resources` (absolute `.cube` paths for Apply Color LUT's file parameter). |
| `theme` | Stored and listed; applied once craft-theme lands (deferred). |

## Operations (session plugins: `craft::call(op, args)`)

### `comp.new` (Edit)

Create a composition, including when no composition is open. Subsequent events capture the new active composition.

Permission: `document.edit:document`

```json
// args
{name?,width?,height?,frameRate?,duration?}
// result
{comp:id}
```

### `layer.addItem` (Edit)

Add an existing project composition/asset as a layer; never imports a path.

Permission: `document.edit:document`

```json
// args
{item,time?,index?,position?,duration?}
// result
object
```

### `layer.newText` (Edit)

Create a text layer with engine-validated typography.

Permission: `document.edit:document`

```json
// args
{text,name?,position?,size?,font?,fill?}
// result
object
```

### `layer.newShape` (Edit)

Create a procedural rectangle, ellipse, star or polygon.

Permission: `document.edit:document`

```json
// args
{kind?,name?,size?,fill?,stroke?,position?}
// result
object
```

### `layer.setSwitch` (Edit)

Change layer video/audio/solo/lock/3D and other engine switches.

Permission: `document.edit:document`

```json
// args
{layers,switch,value?}
// result
object
```

### `project.summary` (Query)

Project items, compositions and asset metadata; no file contents.

Permission: `document.read:document`

```json
// args
{}
// result
object
```

### `layer.tree` (Query)

Layer property tree, effects and keyframe metadata; bounded depth.

Permission: `document.read:document`

```json
// args
{layer,depth?,time?}
// result
object
```

### `time.set` (Edit)

Move the playhead through the canonical timeline command.

Permission: `document.edit:document`

```json
// args
{time}
// result
object
```

### `markers.set` (Edit)

Edit a composition marker by index (time in composition seconds).

Permission: `document.edit:document`

```json
// args
{index,time?,comment?,duration?}
// result
object
```

### `markers.delete` (Edit)

Delete a composition marker by index.

Permission: `document.edit:document`

```json
// args
{index}
// result
object
```

### `effect.reorder` (Edit)

Move an effect in its layer stack (one-based index).

Permission: `document.edit:document`

```json
// args
{layer,effect,index}
// result
object
```

### `comp.settings` (Edit)

Change active composition dimensions, duration, frame rate or work area using engine validation.

Permission: `document.edit:document`

```json
// args
{width?,height?,duration?,frameRate?}
// result
object
```

### `canvas.overlay.set` (Ui)

Replace this plugin's bounded viewer guides, in composition pixels; cleared after a gesture.

Permission: free (own overlay only)

```json
// args
{ops:[draw op]}
// result
null
```

### `marker.list` (Query)

Composition markers, including times and comments.

Permission: `document.read:document`

```json
// args
{}
// result
[marker]
```

### `prop.get` (Query)

Read a property and its keyframes in the captured composition.

Permission: `document.read:document`

```json
// args
{layer,path,time?}
// result
{value,evaluated,keys}
```

### `prop.set` (Edit)

Set a property through the engine's validated property command.

Permission: `document.edit:document`

```json
// args
{layer,path,value,time?}
// result
object
```

### `prop.addKey` (Edit)

Add a keyframe; time defaults to layer seconds, timeBase may be comp.

Permission: `document.edit:document`

```json
// args
{layer,path,value,time?,timeBase?}
// result
object
```

### `prop.reset` (Edit)

Reset a property.

Permission: `document.edit:document`

```json
// args
{layer,path}
// result
object
```

### `effect.remove` (Edit)

Remove an effect instance by uid.

Permission: `document.edit:document`

```json
// args
{layer,effect}
// result
object
```

### `layer.newSolid` (Edit)

Create a solid layer in the captured composition.

Permission: `document.edit:document`

```json
// args
{name?,color?,width?,height?}
// result
object
```

### `layer.newNull` (Edit)

Create a null for motion paths and controls.

Permission: `document.edit:document`

```json
// args
{name?}
// result
object
```

### `layer.select` (Edit)

Select layer ids in the captured composition.

Permission: `document.edit:document`

```json
// args
{layers:[id]}
// result
object
```

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
| `tool` | `{"event":"tool","phase":"begin|update|commit|cancel","at":[0,0],"time":0,"modifiers":{}}` | no | Pointer gesture in composition pixels at the current time; Escape cancels; one undo step per gesture. |
| `init` | `{"event":"init"}` | no | Once after the plugin loads: draw your panels. |
| `command` | `{"event":"command","id":"<contribution>"}` | no | A command contribution ran (one undo step). |
| `ui` | `{"event":"ui","surface":"<panel>","rev":4,"widget":"<id>","value":null}` | no | A widget in your panel or window changed; stale revisions are dropped. |
| `app` | `{"event":"app","name":"selection.changed","payload":{"layers":[12]}}` | yes | `selection.changed`, `time.changed` (at most 4 per second), `document.changed` (coalesced per pump; layers, keys, markers, effects, assets/settings), `document.opened`, `render.finished`. |
| `dispose` | `{"event":"dispose"}` | no | The plugin is being turned off or reloaded. |

## Notes

Pixels: premultiplied RGBA f32 in the project's working space (`color` is `linear-srgb` when Linearize Working Space is on, else `srgb`); values may exceed 1 in 32 bpc projects. `time` is layer time in seconds. `scale` is buffer pixels per layer pixel (previews at reduced resolution have scale < 1 and `quality: "draft"`). `origin` is the buffer's top-left inside the layer's full frame and `full` the layer's size, both in buffer pixels. Plugin effects are treated as time-dependent by the frame caches. Ops act on the composition and selection captured when the event started. Effect ids are `<plugin id>/<contribution>`.

## Effect forms (schema 1)

`contribution.form` supplies the effect controls; optional `craft_describe` supplies a validated runtime form. Form parameter keys identify render `params` values. Choice/radio values are option IDs, colors straight RGBA, angles degrees, and points use their declared image-percent or layer-pixels space (not buffer pixels). Effect Controls shows units and section headings and evaluates visible/enabled conditions. Optional `craft_update` runs with the shared event budget; patches are validated atomically and resulting value edits share one undo step. Visibility and enablement overrides are transient per-instance UI state.

Panel Form nodes render host widgets with parameter-key events. Component nodes currently always render their fallback. Preset effect entries use `{effect, values}`.

## Error codes

`-1` invalid input, `-2` permission denied, `-3` not found, `-4` stale or busy, `-5` cancelled, `-6` over budget, `-7` internal.

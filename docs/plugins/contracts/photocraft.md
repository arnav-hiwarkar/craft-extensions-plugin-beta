# PhotoCraft plugin contract

Generated from the `photocraft` adapter by `craft-ext contract-docs`. Do not edit by hand.

## Contracts provided

| Contract | Version |
|---|---|
| `raster.filter` | 1.0 |
| `photocraft.session` | 1.0 |
| `resource.preset` | 1.0 |
| `resource.lut` | 1.0 |
| `craft-theme` | 1.0 |

A contribution declaring `contract = "name@major.minor"` works here when the major matches and this minor is equal or higher.


## Where contributions appear

| Kind | Placement |
|---|---|
| `effect` | Filter → Plugins → category |
| `command` | Plugins → plugin → label |
| `panel` | Plugin panels |
| `preset` | Plugin filter presets |
| `lut` | plugin.resources |

## Operations (session plugins: `craft::call(op, args)`)


### `document.info` (Query)

Read the active document

Permission: document.read:document

```json
// args
{}
// result
{}
```

### `layer.list` (Query)

List document layers

Permission: document.read:document

```json
// args
{}
// result
{}
```

### `selection.get` (Query)

Read the captured selection

Permission: document.read:selection

```json
// args
{}
// result
{}
```

### `layer.rename` (Edit)

Rename a captured layer

Permission: document.edit:document

```json
// args
{}
// result
{}
```

### `filter.apply` (Edit)

Apply {filter, params} on the active layer

Permission: document.edit:document

```json
// args
{}
// result
{}
```

### `ui.panel.set` (Ui)

Set a declarative panel

Permission: none

```json
// args
{}
// result
{}
```

### `ui.window.open` (Ui)

Open a declarative window

Permission: ui.window

```json
// args
{}
// result
{}
```

### `ui.notify` (Ui)

Show a notification

Permission: none

```json
// args
{}
// result
{}
```

### `plugin.resources` (Query)

List enabled presets and LUTs

Permission: none

```json
// args
{}
// result
{}
```

## Events (`craft_event`)

| Event | Payload | Needs `app.events` | Meaning |
|---|---|---|---|
| `init` | `{}` | no | init |
| `command` | `{}` | no | command |
| `ui` | `{}` | no | ui |
| `app` | `{}` | yes | document.opened, selection.changed |
| `dispose` | `{}` | no | dispose |

## Notes

RGB(A) 8/16/32-bit only, premultiplied RGBA f32 in the named document working space. Other modes: This filter works on RGB images. Destructive filters need no project record. TODO(PhotoCraft-plugin-projects): persist non-destructive plugin references and refuse export if missing. TODO(PhotoCraft-plugin-integration): bind session operations, menus, parameter dialogs and panels to the document thread before enabling execution.

## Error codes

`-1` invalid input, `-2` permission denied, `-3` not found, `-4` stale or busy, `-5` cancelled, `-6` over budget, `-7` internal.

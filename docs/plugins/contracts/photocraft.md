# PhotoCraft plugin contract

Generated from the `photocraft` adapter by `<app>-cli plugin contract`. Do not edit by hand.

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

Tools are not supported in PhotoCraft yet. Tool contributions are unavailable even when their declared contract matches.

RGB(A) 8/16/32-bit only; premultiplied RGBA f32 in sRGB for integer depths or linear sRGB for float, with CMS conversion. Other modes are disabled with a reason. Filters apply destructively as one history step; plugin smart filters are unsupported.

Effect dialogs use Form schema 1 (`contribution.form`, optional worker `craft_describe`). Filter context `params` and CLI/automation parameters are Values keyed by param key: choice/radio option ids, straight RGBA including HDR RGB, and `[x,y]` points in their declared space. Defaults are filled; unknown keys and invalid values are refused. `layer`, `target`, `coalesce` and the `__` prefix are host-reserved command keys.

The host draws all form kinds, units, help and sections, evaluates visible/enabled conditions, and calls optional `craft_update` on workers after edits. Event fuel, grant checks and a 100 ms deadline bound form calls. Patches apply atomically at the current revision; stale edit results are discarded. Plugin presets accept `{id,label,effects:[{effect,values}]}` single-effect recipes and legacy direct `{id,label,effect,values}` records inside a `presets` array; the dialog offers matching, validated presets and omits multi-effect recipes.

Session panels/windows render `Node::Form` with local edit state and param-key events. No framework component library is registered: every `Node::Component` renders its validated fallback. Sessions execute on workers; document operations are authorized and committed on the document thread.

## Error codes

`-1` invalid input, `-2` permission denied, `-3` not found, `-4` stale or busy, `-5` cancelled, `-6` over budget, `-7` internal.

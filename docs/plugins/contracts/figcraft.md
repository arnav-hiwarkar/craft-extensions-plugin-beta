# FigCraft plugin contract

Generated from the `figcraft` adapter by `<app>-cli plugin contract`. Do not edit by hand.

## Contracts provided

| Contract | Version |
|---|---|
| `figcraft.session` | 1.0 |
| `resource.font` | 1.0 |
| `craft-theme` | 1.0 |

A contribution declaring `contract = "name@major.minor"` works here when the major matches and this minor is equal or higher.

## Where contributions appear

| Kind | Placement |
|---|---|
| `command` | Plugins ▸ <plugin> ▸ <label> |
| `panel` | Floating host-drawn panel, toggled by its title in the Plugins menu (docking pending) |
| `tool` | Plugin tool strip above the main toolbar, and Plugins ▸ <plugin> ▸ <label> |
| `event-hook` | App event subscriptions |
| `font` | App-local font list |

## Operations (session plugins: `craft::call(op, args)`)

### `document.info` (Query)

The active document's name and session id

Permission: document.read:document

```json
// args
{}
// result
{name, uid}
```

### `page.list` (Query)

The document's pages as layer objects

Permission: document.read:document

```json
// args
{}
// result
[page]
```

### `selection.get` (Query)

The selected layer ids

Permission: document.read:selection

```json
// args
{}
// result
{ids: [id]}
```

### `node.get` (Query)

One layer as JSON

Permission: document.read:document (selection-only support deferred)

```json
// args
{id}
// result
layer
```

### `node.create` (Edit)

Create a layer (the engine's node.create)

Permission: document.edit:document

```json
// args
{type: rectangle|ellipse|polygon|star|line|arrow|vector|text|frame|section|component, x?, y?, width?, height?, parent?, index?, name?, text?, props? {node.set keys, e.g. fill: "#3380e6"}, select?}
// result
{id}
```

### `node.update` (Edit)

Change a layer's properties (the engine's node.set)

Permission: document.edit:document (selection-only support deferred)

```json
// args
{id, props: {x?, y?, width?, height?, name?, fill?, …}}
// result
engine result
```

### `node.delete` (Edit)

Delete a layer

Permission: document.edit:document

```json
// args
{id}
// result
engine result
```

### `selection.set` (Query)

Replace the selection (the engine's select.set)

Permission: document.read:document

```json
// args
{ids: [id]}
// result
engine result
```

### `viewport.get` (Query)

The canvas view (pan and zoom)

Permission: free

```json
// args
{}
// result
view
```

### `ui.panel.set` (Ui)

Draw a declared panel; applied when the event commits

Permission: free

```json
// args
{panel, tree} | {panel, patch}
// result
null
```

### `ui.window.open` (Ui)

Open or redraw a plugin window (at most four)

Permission: ui.window

```json
// args
{id, title, tree} | {id, patch}
// result
null
```

### `ui.notify` (Ui)

Log a short message for the person

Permission: free

```json
// args
{message}
// result
null
```

### `canvas.overlay.set` (Ui)

Draw guides on the canvas in page units; cleared when a tool gesture ends

Permission: free

```json
// args
{ops: [line|rect|circle|text draw op]}
// result
null
```

## Events (`craft_event`)

| Event | Payload | Needs `app.events` | Meaning |
|---|---|---|---|
| `init` | `{}` | no | Startup, and again when a document opens; draw panels here |
| `command` | `{id, params}` | no | A command contribution ran (menu, Actions, CLI `run --plugins`) |
| `ui` | `{surface,rev,widget,value}` | no | A panel or window widget changed; form widgets send the parameter key |
| `tool` | `{phase:begin|update|commit|cancel,at:[x,y],modifiers}` | no | Pointer gesture of the selected tool, in page units; Escape sends cancel |
| `app` | `{type|name:selection.changed|document.opened|page.changed}` | yes | Something changed in the app |
| `dispose` | `{}` | no | The worker is stopping |

## Notes

Each event is one undo step labelled "<label> · <plugin>"; a failed, timed-out or revoked event rolls back. Tool gestures (begin..commit) are one step; cancel rolls back. Coordinates are document units. No raster.filter contract is advertised until image-effect dispatch is integrated. Font/theme activation is pending. node.get/update/delete currently require document scope; selected-target authorization must use host-captured scope, never guest flags. Panels and windows render Node::Form (every parameter kind, defaults, host-evaluated conditions; unknown components show their fallback); form widget events carry the parameter key and choices their option id. {panel|id, patch} patches a root form: patch schema 1, base_rev is the surface revision, rev must increase, published atomically with the event. Optional wasm describe/update exports are unused until raster.filter is offered.

## Error codes

`-1` invalid input, `-2` permission denied, `-3` not found, `-4` stale or busy, `-5` cancelled, `-6` over budget, `-7` internal.

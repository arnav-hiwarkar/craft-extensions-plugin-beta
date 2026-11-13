# FigCraft plugin contract

Generated from the `figcraft` adapter by `<app>-cli plugin contract`. Do not edit by hand.

## Contracts provided

| Contract | Version |
|---|---|
| `figcraft.session` | 1.1 |
| `resource.font` | 1.0 |
| `craft-theme` | 1.0 |

A contribution declaring `contract = "name@major.minor"` works here when the major matches and this minor is equal or higher.

## Where contributions appear

| Kind | Placement |
|---|---|
| `command` | Plugins ▸ <plugin> ▸ <label> |
| `panel` | Docked host-drawn panel, toggled by its title in the Plugins menu |
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

### `page.add` (Edit)

Add page

Permission: document.edit:document

```json
// args
{name?} (a name starting with a hyphen adds a divider)
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `page.rename` (Edit)

Rename page

Permission: document.edit:document

```json
// args
{id?, name} (an empty page named with a leading hyphen becomes a divider)
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `page.delete` (Edit)

Delete page

Permission: document.edit:document

```json
// args
{id?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `page.duplicate` (Edit)

Duplicate page

Permission: document.edit:document

```json
// args
{id?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `page.move` (Edit)

Move page

Permission: document.edit:document

```json
// args
{id?, index}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `page.setBackground` (Edit)

Page background

Permission: document.edit:document

```json
// args
{id?, color}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `component.create` (Edit)

Create component

Permission: document.edit:document

```json
// args
{ids?}  (one frame/group becomes a component; several layers are wrapped in one)
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `component.createMultiple` (Edit)

Create multiple components

Permission: document.edit:document

```json
// args
{ids?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `component.createSet` (Edit)

Create component set

Permission: document.edit:document

```json
// args
{ids? (layers, not components, sets or instances)}  (each becomes a component, wrapped in place in a component set: variants "Property 1=<name>" left to right, 20 px around)
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `component.combineVariants` (Edit)

Combine as variants

Permission: document.edit:document

```json
// args
{ids? (main components; none already a variant, a library copy or inside an instance)}  (wraps them in a new component set; names without `=` become "Property 1=Default", "Property 1=VariantN")
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `component.addVariant` (Edit)

Add variant

Permission: document.edit:document

```json
// args
{id? (component set, variant, or a main component not in a set yet: it becomes a set of itself and the new variant), property?, value?}  (duplicates the variant; `property`=`value` names the new one)
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `instance.create` (Edit)

Create instance

Permission: document.edit:document

```json
// args
{component (component or set: its default variant), x?, y?, parent?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `component.detach` (Edit)

Detach instance

Permission: document.edit:document

```json
// args
{ids?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `instance.swap` (Edit)

Swap instance

Permission: document.edit:document

```json
// args
{ids?, component}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `component.resetInstance` (Edit)

Reset all changes

Permission: document.edit:document

```json
// args
{ids?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `style.create` (Edit)

Create style

Permission: document.edit:document

```json
// args
{type: paint|text|effect|grid, name, from? (node id; default first selected), description?, value? (paints/effects/grids/text style JSON)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `style.apply` (Edit)

Apply style

Permission: document.edit:document

```json
// args
{ids?, style (id or name), target? (fill|stroke; paint styles), start?, end? (text styles: these chars; while editing text, the selected chars)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `style.detach` (Edit)

Detach style

Permission: document.edit:document

```json
// args
{ids?, which: fill|stroke|effect|grid|text, start?, end? (text: these chars; while editing text, the selected chars)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `style.update` (Edit)

Edit style

Permission: document.edit:document

```json
// args
{style, name?, description?, value?, from? (node id: redefine from it)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `style.delete` (Edit)

Delete style

Permission: document.edit:document

```json
// args
{style}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `style.duplicate` (Edit)

Duplicate style

Permission: document.edit:document

```json
// args
{style} → {id} (right below it, named "<name> copy")
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `variable.createCollection` (Edit)

Create variable collection

Permission: document.edit:document

```json
// args
{name, modes? [names]}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `variable.addMode` (Edit)

Add mode

Permission: document.edit:document

```json
// args
{collection, name, from? (mode id or name to duplicate; default: the default mode)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `variable.create` (Edit)

Create variable

Permission: document.edit:document

```json
// args
{collection, name, type: color|number|string|boolean, value (default mode) | values {modeName: value}, description?}  a value may also be {alias: variable} (same type); a colour value may also be {alpha: {variable, opacity 0..1}} (that variable's colour with its alpha replaced by the opacity) or {mix: {from, to, t}} (operands {variable: variable} or {literal: colour}; t is 0..1, 0 gives from and 1 to; blended per channel over the stored values, in the document's colour profile, including alpha; converting the profile keeps a mix's ends exact and moves its colours in between slightly), each alone in its object
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `variable.set` (Edit)

Set variable value

Permission: document.edit:document

```json
// args
{variable, mode? (name or id; default first), value | {alias: variable} (same type, no loops) | {alpha} | {mix} (colour variables; shapes as in variable.create; no loops)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `variable.delete` (Edit)

Delete variable

Permission: document.edit:document

```json
// args
{variable} (bound layers keep their values; variables aliasing it take its values; colour calculations reading it keep the colour they show)
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `variable.bind` (Edit)

Bind variable

Permission: document.edit:document

```json
// args
{ids?, field (fill|stroke (first paint)|fills/<i>/color|strokes/<i>/color|fills/<i>/stops/<j>/color (gradient stops)|effects/<i>/<offsetX|offsetY|radius|startRadius|spread|color|lightAngle|lightIntensity|refraction|depth|dispersion|splay> (glass: `radius` is its frost; intensity, refraction, dispersion and splay in percent)|layoutGrids/<i>/<size|count|gutter|offset>|componentProperties/<name> (instances: boolean/text properties; variant properties take string variables naming a value)|width|height|opacity (percent)|cornerRadius|gap|paddingHorizontal|characters|visible|fontSize…; list indices bottom first), variable (null unbinds)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `variable.setMode` (Edit)

Set variable mode

Permission: document.edit:document

```json
// args
{ids? (frames/page), collection, mode (id or name; null: Auto, inherited from the parent)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `variable.update` (Edit)

Edit variable

Permission: document.edit:document

```json
// args
{variable, name?, description?, scopes? ([scope ids]; null or ["ALL_SCOPES"] = every supported property, [] = hidden from pickers), hiddenFromPublishing? (left out when the file is published as a library), codeSyntax? {WEB?, iOS?, ANDROID?: name code uses ("" clears)}}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `variable.renameCollection` (Edit)

Rename collection

Permission: document.edit:document

```json
// args
{collection, name}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `variable.deleteCollection` (Edit)

Delete collection

Permission: document.edit:document

```json
// args
{collection} (with its variables; bound layers keep their values; variables elsewhere aliasing them take their values and colour calculations reading them keep the colour they show)
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `vector.create` (Edit)

Vector

Permission: document.edit:document

```json
// args
{path (SVG path data) | points [[x,y],..], closed?, x?, y? (move the path's own origin: the layer's left/top is x/y plus the box's unscaled offset), width?, height? (scale the path; one keeps its proportion), parent?, index?, name?, props?, select? (default true)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `image.place` (Edit)

Place image

Permission: document.edit:document

```json
// args
{data (base64) | path, x?, y?, width?, height?, parent?, name? (default Image), target? (an existing layer: the image is added on top of its fills instead of making a layer), mode? (fill|fit|crop|tile)} → {id, image, width, height, imageProfile (its embedded colour profile; null: taken as sRGB)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `comment.add` (Edit)

Add comment

Permission: document.edit:document

```json
// args
{x, y, text, page?, author?, anchor? (node id to pin to; true = innermost frame under the point)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `comment.reply` (Edit)

Reply to comment

Permission: document.edit:document

```json
// args
{id | link, text, author?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `comment.resolve` (Edit)

Resolve comment

Permission: document.edit:document

```json
// args
{id | link, resolved? (default true)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `layout.setTrack` (Edit)

Set grid track size

Permission: document.edit:document

```json
// args
{ids?, axis: columns|rows, index (0-based), sizing: fixed|hug|fill, value? (fixed: px, default the track's current size; fill: fr, default 1)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `image.adjust` (Edit)

Adjust image

Permission: document.edit:document

```json
// args
{ids?, target? (fill|stroke, default fill), index? (default every image paint), preset? (vivid|muted|warm|cool|blackAndWhite|fade|reset, or the name of a preset saved in the file), filters? ({exposure, contrast, saturation, temperature, tint, highlights, shadows}: each -1..1, applied over the preset)}  sets the adjustments of the layers' image paints; returns {changed}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `image.replace` (Edit)

Replace image

Permission: document.edit:document

```json
// args
{ids?, target? (fill|stroke, default fill), index? (default each layer's top image paint), data (base64) | path}  puts the image in the paint, keeping its fill mode, crop, position and adjustments; a paint at `index` that isn't an image becomes one, filling the layer; returns {image, width, height, changed}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `text.create` (Edit)

Text

Permission: document.edit:document

```json
// args
{text | characters, x, y, parent?, index?, name? (default: the first line), width? (fixed width wraps), props? (fontSize, fontFamily, …), select? (default true)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `text.onPath` (Edit)

Text on path

Permission: document.edit:document

```json
// args
{text? (text layer), path? (vector or shape layer; default: the selected text and other layer), start? (0..1 along the path), flip?}  the text follows the path and takes its effects and its fills when one shows, and the path layer is removed → {id}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `text.pathOptions` (Edit)

Text path options

Permission: document.edit:document

```json
// args
{start? (0..1 along the path), flip?, ids?}  for text on a path
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `guide.add` (Edit)

Add guide

Permission: document.edit:document

```json
// args
{vertical (bool), offset, frame? | page?} → {index}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `guide.move` (Edit)

Move guide

Permission: document.edit:document

```json
// args
{index, offset, frame? | page?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `guide.remove` (Edit)

Remove guide

Permission: document.edit:document

```json
// args
{index?, frame? | page?} (no index: the selected guide)
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `guide.clear` (Edit)

Remove all guides

Permission: document.edit:document

```json
// args
{frame? | page?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `node.reparent` (Edit)

Move layer into

Permission: document.edit:document

```json
// args
{ids?, parent, index? (final position among parent's children, bottom first; default top)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `node.reorder` (Edit)

Reorder layer

Permission: document.edit:document

```json
// args
{id, index}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `edit.duplicate` (Edit)

Duplicate

Permission: document.edit:document

```json
// args
{ids?, dx?, dy? (page offset; default: the offset of the last duplicate when ids are its copies, still selected; else top-level frames to the right with a 100 gap, everything else in place), map? (true: also return `map`, each original layer's id, descendants included, → its copy's id)}  a main component's copy is an instance of it (a variant stays a new variant in its set)
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `object.group` (Edit)

Group selection

Permission: document.edit:document

```json
// args
{ids?, name?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `object.ungroup` (Edit)

Ungroup selection

Permission: document.edit:document

```json
// args
{ids?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `object.resizeToFit` (Edit)

Resize to fit

Permission: document.edit:document

```json
// args
{ids?} (frames, sections and components shrink or grow to their contents; the contents stay put)
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `layout.add` (Edit)

Add auto layout

Permission: document.edit:document

```json
// args
{ids?, direction? (horizontal|vertical|grid)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `layout.remove` (Edit)

Remove auto layout

Permission: document.edit:document

```json
// args
{ids?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `bool.union` (Edit)

Union selection

Permission: document.edit:document

```json
// args
{ids?} — layers inside boolean groups: sets their own operation (how each combines with the operands below it); one boolean group: sets its operation (every operand follows it); otherwise wraps the layers in a new boolean group
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `bool.subtract` (Edit)

Subtract selection

Permission: document.edit:document

```json
// args
{ids?} — layers inside boolean groups: sets their own operation (how each combines with the operands below it); one boolean group: sets its operation (every operand follows it); otherwise wraps the layers in a new boolean group
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `bool.intersect` (Edit)

Intersect selection

Permission: document.edit:document

```json
// args
{ids?} — layers inside boolean groups: sets their own operation (how each combines with the operands below it); one boolean group: sets its operation (every operand follows it); otherwise wraps the layers in a new boolean group
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `bool.exclude` (Edit)

Exclude selection

Permission: document.edit:document

```json
// args
{ids?} — layers inside boolean groups: sets their own operation (how each combines with the operands below it); one boolean group: sets its operation (every operand follows it); otherwise wraps the layers in a new boolean group
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `vector.flatten` (Edit)

Flatten

Permission: document.edit:document

```json
// args
{ids?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `vector.outlineStroke` (Edit)

Outline stroke

Permission: document.edit:document

```json
// args
{ids?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `instance.setProperties` (Edit)

Set instance properties

Permission: document.edit:document

```json
// args
{ids?, properties {name: value}} (variant properties switch variants)
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `node.find` (Query)

Find layers

Permission: document.read:document

```json
// args
{root? (strict descendants; default current page), name?, type? (canonical layer type), text?, component? (main component name), limit? (0-10000; default 100)}: every filter must match; name, text and component are case-insensitive substrings, type is exact (any case); returns {ids, total} and does not select
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `style.list` (Query)

List styles

Permission: document.read:document

```json
// args
{type?}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `variable.list` (Query)

List variables

Permission: document.read:document

```json
// args
{}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `image.presets` (Query)

Image presets

Permission: document.read:document

```json
// args
{} → {builtIn: [{id, label, filters}], saved: [{name, filters}]}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `page.select` (Query)

Go to page

Permission: document.read:document

```json
// args
{id | index}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `comment.list` (Query)

List comments

Permission: document.read:document

```json
// args
{page? (default: current; "all" for every page), filter? (all|open|resolved|mine|unread; default all), sort? (date|page; default date), query? (search text), frame? (a top-level frame id), resolved? (false hides resolved ones), author? (whose threads and read state), mentions? (a name, any case: only threads where a message @mentions it, as each message's `mentions` reports)}
// result
engine result; JSON arguments <= 256 KiB, depth <= 32, arrays <= 4096; image.place/replace require data and forbid path
```

### `viewport.set` (Query)

Set the canvas view

Permission: free

```json
// args
{zoom?, cx?, cy?, vw?, vh?}; numbers finite and bounded
// result
engine result
```

### `export.svg` (Query)

Export layers to an SVG string without filesystem access

Permission: document.read:document

```json
// args
{ids: [id] (1..256)}
// result
{svg} (at most 4 MiB)
```

### `node.children` (Query)

List direct child layer objects (at most 4096)

Permission: document.read:document

```json
// args
{id}
// result
[layer]
```

## Events (`craft_event`)

| Event | Payload | Needs `app.events` | Meaning |
|---|---|---|---|
| `init` | `{}` | no | Startup, and again when a document opens; draw panels here |
| `command` | `{id, params}` | no | A command contribution ran (menu, Actions, CLI `run --plugins`) |
| `ui` | `{surface,rev,widget,value}` | no | A panel or window widget changed; form widgets send the parameter key |
| `tool` | `{phase:begin|update|commit|cancel,at:[x,y],modifiers}` | no | Pointer gesture of the selected tool, in page units; Escape sends cancel |
| `app` | `{type|name:selection.changed|document.opened|page.changed|document.changed}` | yes | Something changed in the app; document.changed is a read-only notification (edits are refused to prevent feedback loops) |
| `dispose` | `{}` | no | The worker is stopping |

## Notes

Each event is one undo step labelled "<label> · <plugin>"; a failed, timed-out or revoked event rolls back. Tool gestures (begin..commit) are one step; cancel rolls back. Coordinates are document units. No raster.filter contract is advertised until image-effect dispatch is integrated. Font/theme activation is pending. node.get/update/delete currently require document scope; selected-target authorization must use host-captured scope, never guest flags. Panels and windows use the shared UI 1.1 renderer (including tables, tabs, scrolling, rich forms and draw lists), with the engine chosen by the app. Panels and windows render Node::Form (every parameter kind, defaults, host-evaluated conditions; unknown components show their fallback); form widget events carry the parameter key and choices their option id. {panel|id, patch} patches a root form: patch schema 1, base_rev is the surface revision, rev must increase, published atomically with the event. Optional wasm describe/update exports are unused until raster.filter is offered.

## Error codes

`-1` invalid input, `-2` permission denied, `-3` not found, `-4` stale or busy, `-5` cancelled, `-6` over budget, `-7` internal.

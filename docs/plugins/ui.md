# Plugin UI reference (UI level 1.1)

Plugins **describe** their panels and windows as data; the app **draws** them. A plugin never runs
code on the UI thread, never sees pixels of the interface, and cannot tell how it is drawn. This page
is the complete reference for that description: the UI tree, forms, the canvas draw list, the events
that come back, the limits, and how the vocabulary is versioned.

Code: `craft_ext_core::ui::tree` (types and validation), `craft_ext_ui::plugin_ui` (the renderer
every app shares), `craft_guest::ui` (builders for plugin authors).

## The host owns the engine

An app draws plugin UI with an engine **it** chooses (`craft_ext_ui::plugin_ui::EngineChoice`):
`Egui` today, `Framework` (the craft-framework renderer and component library) when it lands.
Plugins cannot choose, request or detect the engine. The same tree produces the same events
whichever engine draws it, and the theme, fonts, spacing and accessibility come from the app. Write
trees for meaning ("a table with selectable rows"), not for looks.

## Sending a tree

| Operation | Arguments | Notes |
|---|---|---|
| `ui.panel.set` | `{panel, tree}` | Replaces a panel your manifest declares. Free (no permission). |
| `ui.window.open` | `{id, title, tree}` | Opens a floating window. Needs `ui.window`. |

Each accepted tree gets a new **revision** (`rev`). Every widget event carries the revision that was
on screen; the app drops events whose revision is no longer current, so you never act on a control
that has since changed. A tree is validated as a whole before anything is drawn: one invalid node
refuses the whole call with a reason.

Every **interactive control** needs a stable `id` (1–64 bytes) and a visible `label` (1–200
characters, no control or invisible formatting characters). Ids are yours: keep them stable across
revisions so in-progress edits survive a redraw.

## Versioning

The vocabulary has a level. **ui 1.0** is the original set (marked 1.0 below). **ui 1.1** adds
everything else on this page. A level-1.0 tree is valid, unchanged, at 1.1.

- **Declare it.** If your tree uses anything from 1.1, declare your session contract at `@1.1`
  or later: `contract = "effectcraft.session@1.1"` (likewise `photocraft.session`,
  `figcraft.session`). An app that only draws 1.0 then reports *"needs effectcraft.session 1.1, this
  app has 1.0 — update the app"* in the manager **before** your plugin runs, instead of failing to
  parse your tree.
- **Check it.** `craft_guest::ui::level(&tree)` returns the level a tree needs;
  `craft_guest::ui::check(&tree)` runs the host's validation. Hosts may refuse a 1.1 tree from a
  plugin that declared `@1.0` (`Node::validate_at`), so declare what you use.
- **Forward compatible.** A node whose `type` the app does not know is not an error: it draws its
  `fallback` node if it has one, otherwise a visible notice ("This part of the panel needs a newer
  version of the app"). Any node may carry `"fallback": {…}` for this purpose. A *known* type with
  wrong fields is still refused, with the reason.
- New fields on existing nodes only arrive with a new level; unknown fields are refused.

## Nodes

All nodes are JSON objects tagged by `"type"`. Optional fields may be left out. "Sends" is the
`value` of the `ui` event; see [Events](#events).

### Layout

| `type` | Fields | Level |
|---|---|---|
| `column` | `children` | 1.0 |
| `row` | `children` (wraps when narrow) | 1.0 |
| `group` | `label`, `children` (a framed, titled box) | 1.0 |
| `separator` | — | 1.0 |
| `tabs` | `id`, `tabs: [{id, label, icon?, children}]` (1–32, unique ids), `selected?` (tab id; default first). Only the shown tab's children are drawn. Sends the tab id (string) when the person switches. | 1.1 |
| `collapsible` | `id`, `label`, `open` (default false), `children`. Sends `true`/`false` when toggled. | 1.1 |
| `scroll` | `children`, `max_height?` (points), `horizontal?` (also scroll sideways) | 1.1 |
| `grid` | `columns` (1–16), `children` (row by row), `striped?` | 1.1 |
| `spacer` | `size` (0–512 points) | 1.1 |

### Display

| `type` | Fields | Level |
|---|---|---|
| `label` | `text` (≤ 4000 chars); 1.1 adds `style?` (`body`, `heading`, `strong`, `small`, `weak`, `mono`) and `icon?` | 1.0 |
| `image` | `width`, `height` (1–2048), `rgba` (byte array, exactly `width*height*4`); 1.1 adds `name?` (drawable on a canvas), `hidden?` (register without showing), `size?: [w,h]` (display size in points), `alt?` (tooltip) | 1.0 |
| `progress` | `value?` (0–1; leave out for an indeterminate "working" indicator), `label?` | 1.1 |
| `icon` | `name` (see [Icons](#icons)), `size?` (default 16), `label?` (tooltip), `rgba?` (tint; default the theme's text colour) | 1.1 |

### Controls

Every control below except `canvas` accepts, from 1.1, `hint?` (tooltip, ≤ 1000 chars) and
`disabled?` (greyed out; sends nothing).

| `type` | Fields | Sends | Level |
|---|---|---|---|
| `button` | `id`, `label`; 1.1 adds `icon?`, `icon_only?` (needs `icon`; the label becomes the tooltip) | `null` on click | 1.0 |
| `checkbox` | `id`, `label`, `value` | bool | 1.0 |
| `toggle` | `id`, `label`, `value` (a switch) | bool | 1.1 |
| `slider` | `id`, `label`, `value`, `min` < `max`; 1.1 adds `step?` | number, once when the drag ends | 1.0 |
| `number` | `id`, `label`, `value`, `min?`, `max?`, `step?`, `decimals?` (0–10), `unit?` (≤ 16 chars) | number, when the drag or typing ends | 1.1 |
| `text-input` | `id`, `label`, `value`; 1.1 adds `placeholder?` | string, on Enter or when focus leaves | 1.0 |
| `text-area` | `id`, `label`, `value` (≤ 65 536 chars), `rows?` (1–50), `placeholder?`, `mono?` | string, when focus leaves or on Ctrl/Cmd+Enter | 1.1 |
| `select` | `id`, `label`, `options` (1–256 strings), `selected` (index) | index | 1.0 |
| `radio` | `id`, `label`, `options` (1–256), `selected` (index), `horizontal?` | index | 1.1 |
| `color` | `id`, `label`, `rgba` | `[r,g,b,a]` | 1.0 |
| `table` | `id`, `label`, `columns: [{label, width?, align?}]` (1–32), `rows: [{key, cells}]` (≤ 10 000, unique keys, at most one cell per column), `selection?` (`none`, `single`, `multi`), `selected?` (keys), `max_height?` | selected keys `["a","c"]` on change; `{"activate": key}` on double click | 1.1 |
| `list` | `id`, `label`, `items: [{key, label, detail?, icon?}]` (≤ 10 000), `selection?`, `selected?`, `max_height?` | as `table` | 1.1 |
| `canvas` | see [Canvas](#canvas) | pointer and key events | 1.0 |
| `form` | see [Forms](#forms) | the parameter's value, keyed by parameter | 1.0 |

In a multi-selection table or list, Ctrl/Cmd-click toggles a row and Shift-click selects a range.

Edits are sent when they **end**, not on every frame: a slider or number field sends once when the
drag is released, a text field on Enter or when focus leaves. While the person is editing, a new
revision of your tree does not snap the control back; once they stop, the control shows the value
your latest tree gives. To reflect an edit, answer with a new tree.

### Components

`{"type":"component","key","name","version","props","fallback"}` (1.0) asks for a named,
versioned framework component (`version` ≥ 1, `props` ≤ 64 KiB of JSON). An app that has it draws
it; every other app draws `fallback`, which is validated like any node. Until the framework
component library lands, every app draws the fallback.

## Forms

`{"type":"form","form":{…},"values":{…}}` (1.0) draws a [form schema 1](author-guide.md#forms-schema-1)
in the panel: the same rows as an effect's controls. Widget events use the **parameter key** as the
widget id and the typed value (`number`, `bool`, choice id string, `[r,g,b,a]`, `[x,y]`, text).

- `values` may be partial; missing keys take their defaults. Invalid values refuse the tree.
- `visible_when` and `enabled_when` conditions are evaluated by the app on the current values.
- 1.1 adds `hidden?` and `disabled?`: lists of parameter keys you hide or disable, as a `hide` /
  `disable` patch would.
- Units are shown after the control; `help` is the hover text of the row.
- Colours are edited numerically so HDR working-space values are never clamped; a picker is offered
  when all channels are in 0–1.
- Choices and radios send the option id, never the label. Only values the schema accepts are sent.

## Canvas

```json
{"type":"canvas","id":"pad","label":"Sketch pad","width":300,"height":200,
 "listen":["move","wheel","double-click","keys"],"cursor":"crosshair","hint":"Draw here",
 "ops":[ … ]}
```

`width` and `height` are points (0 < side ≤ 8192). Coordinates in ops and events are points from
the canvas's top-left corner. Colours are straight RGBA in 0–1. 1.1 adds `listen?`, `cursor?` and
`hint?`.

### Draw ops

Each op is an object tagged by `"op"`. At most 20 000 ops per canvas.

| `op` | Fields | Level |
|---|---|---|
| `line` | `from`, `to`, `width` (0–256), `rgba` | 1.0 |
| `rect` | `min`, `max`, `fill` (paint), `stroke?` (rgba); 1.1 adds `stroke_width?` (default 1), `radius?` (corner radius) | 1.0 |
| `circle` | `center`, `radius`, `fill` (paint); 1.1 adds `stroke?`, `stroke_width?` | 1.0 |
| `text` | `at`, `text` (≤ 4000 chars), `size` (0–256), `rgba`; 1.1 adds `align?` (`left`, `center`, `right`: which side of the text `at` is on), `valign?` (`top`, `center`, `bottom`), `mono?` | 1.0 |
| `path` | `path` (commands, below), `fill?` (paint), `stroke?`, `stroke_width?` | 1.1 |
| `polygon` | `points` (3–4096), `fill?`, `stroke?`, `stroke_width?`. Convex or not; edges must not cross. | 1.1 |
| `image` | `image` (the `name` of an `image` node in the same tree), `min`, `max`, `uv?: [[u0,v0],[u1,v1]]` (0–1), `tint?` | 1.1 |
| `clip` | `min`, `max`: later ops are clipped to this rectangle (within the canvas) | 1.1 |
| `reset-clip` | — : ends the clip | 1.1 |

**Paths** are a list of commands: `{"move":[x,y]}`, `{"line":[x,y]}`,
`{"quad":[[cx,cy],[x,y]]}`, `{"cubic":[[c1x,c1y],[c2x,c2y],[x,y]]}` and `"close"`. A `move` starts a
new subpath. Each subpath is filled on its own: holes (even-odd or non-zero) are not supported, so
draw a ring as a stroke or as two shapes. At most 4096 commands per path.

**Paints** (`fill`) are a colour `[r,g,b,a]` (1.0) or, from 1.1, a gradient in canvas coordinates:

```json
{"gradient":"linear","from":[0,0],"to":[200,0],"stops":[{"at":0,"rgba":[1,0,0,1]},{"at":1,"rgba":[0,0,1,1]}]}
{"gradient":"radial","center":[50,50],"radius":40,"stops":[{"at":0,"rgba":[1,1,1,1]},{"at":1,"rgba":[1,1,1,0]}]}
```

2–16 stops, `at` in 0–1 and not decreasing. Outside the first and last stop the end colours
extend. Strokes are solid colours.

**Images** on a canvas come from an `image` node of the same tree with that `name`; set
`"hidden": true` on it to register it without showing it. Names are unique per tree (≤ 64 named
images, ≤ 64 MiB of pixels per tree). An `image` op naming no image refuses the tree.

### Canvas events

The canvas sends `ui` events whose value is a pointer object:

```json
{"kind":"drag","at":[35,35],"button":"primary","mods":{"shift":true},"delta":[15,5]}
```

| `kind` | When | Extra fields |
|---|---|---|
| `down` | A button is pressed on the canvas | `button` |
| `drag` | The pointer moves while that button is held (also outside the canvas) | `button`, `delta` (movement since the last event) |
| `up` | The button is released (a plain click is `down` then `up`) | `button` |
| `move` | `listen` has `move`: the pointer moves over the canvas with no button held | — |
| `leave` | `listen` has `move`: the pointer leaves the canvas | — |
| `wheel` | `listen` has `wheel`: the wheel or trackpad scrolls over the canvas | `delta` (points). The canvas takes the scroll; enclosing scroll areas do not move. |
| `double-click` | `listen` has `double-click` | `button` |
| `key-down`, `key-up` | `listen` has `keys` and the canvas has keyboard focus | `key` |

- `button` is `primary`, `secondary` or `middle`. `mods` lists the modifiers held (`shift`, `alt`,
  `ctrl`, `cmd`; `cmd` is Command on macOS and Ctrl elsewhere); it is left out when none are.
- **Keys**: a press on the canvas gives it focus (a focus ring is drawn); Escape releases focus and is
  never sent. While focused, Tab and the arrow keys go to the canvas. Key names: `A`–`Z`, `0`–`9`,
  `ArrowLeft`, `ArrowRight`, `ArrowUp`, `ArrowDown`, `Enter`, `Tab`, `Space`, `Backspace`, `Delete`,
  `Home`, `End`, `PageUp`, `PageDown`, `F1`…, and punctuation names such as `Comma`.
- At most one `drag` or `move` is sent per frame, and only when the position changed.
- `cursor` is a hint while the pointer is over the canvas: `default`, `pointer`, `crosshair`,
  `text`, `move`, `grab`, `grabbing`, `not-allowed`, `resize-horizontal`, `resize-vertical`,
  `resize-nwse`, `resize-nesw`, `zoom-in`, `zoom-out`.

Level-1.0 plugins that read only `kind` and `at` keep working: the extra fields are additions.
(1.0 renderers sent only `up` for a quick click; the shared renderer sends `down` then `up`.)

## Events

Widget changes reach your session as:

```json
{"event":"ui","surface":"<panel or window id>","rev":4,"widget":"<id or parameter key>","value":…}
```

| `value` shape | From |
|---|---|
| `null` | `button` |
| `true` / `false` | `checkbox`, `toggle`, `collapsible`, form bool |
| number | `slider`, `number`, form numbers and angles |
| integer index | `select`, `radio` (node) |
| string | `text-input`, `text-area`, `tabs` (tab id), form text and choice/radio ids |
| `[r,g,b,a]` | `color`, form colours |
| `[x,y]` | form points |
| `["key", …]` | `table` / `list` selection |
| `{"activate": key}` | `table` / `list` double click |
| `{"kind": …, "at": …}` | `canvas` |

In Rust, `craft_guest::ui::value(&value)` returns a typed `UiValue`, and
`craft_guest::ui::pointer(&value)` a `PointerEvent`.

## Icons

Icons are named; you cannot ship icon images. The names follow the craft-framework icon
catalogue, and an engine draws them in the app's own style:

`brush`, `check`, `chevron_down`, `chevron_left`, `chevron_right`, `chevron_up`, `circle`, `clock`,
`close`, `copy`, `crop`, `error`, `eye`, `eye_off`, `eyedropper`, `file`, `flag`, `folder`, `funnel`,
`grid`, `hand`, `home`, `image`, `info`, `layers`, `link`, `lock`, `lock_open`, `menu`, `minus`,
`moon`, `more`, `move`, `palette`, `pause`, `pencil`, `play`, `plus`, `redo`, `refresh`, `save`,
`search`, `settings`, `sort_ascending`, `sort_descending`, `square`, `star`, `success`, `sun`, `text`,
`trash`, `triangle`, `undo`, `warning`, `zoom_in`.

A name must be lowercase letters, digits and `_` (≤ 32). A well-formed name the app does not know
draws a neutral placeholder, so newer names degrade instead of failing.

## Limits

| What | Limit |
|---|---|
| Nodes per tree | 2000 |
| Depth | 16 |
| Ids (widget, tab, row, image name) | 1–64 bytes |
| Control labels, tab and column labels, options | 200 characters |
| Hints, placeholders | 1000 characters |
| Label text, table cells, canvas text | 4000 characters |
| Text area value | 65 536 characters |
| Select / radio options | 256 |
| Tabs | 32 |
| Grid columns | 16 |
| Table columns | 32 |
| Table rows / list items | 10 000 each; 100 000 cells per tree |
| Draw ops per canvas | 20 000 |
| Path commands or polygon points | 4096 per op; 100 000 per tree |
| Gradient stops | 2–16 |
| Image side | 2048 pixels |
| Named images | 64 per tree; 64 MiB of RGBA per tree |
| Canvas side | 8192 points |
| Stroke width | 0–256 |
| Text size | 0–256 (drawn at 4–256) |
| Icon size | 0–256 |
| Spacer | 0–512 |
| Text area rows | 1–50 |
| Number decimals | 0–10 |
| Component props | 64 KiB of JSON |
| Every number | finite |

Beyond these, the app's memory budget for pending results and UI trees applies (1 MiB per call).

## Building trees in Rust

```rust
use craft_guest::ui::{self, Cursor, Listen, NodeExt, Selection, TextStyle, draw, paint};

let tree = ui::column(vec![
    ui::text("Layers", TextStyle::Heading),
    ui::table("layers", "Layers", &["Name", "Size"], rows).selection(Selection::Multi, &[]).max_height(200.0),
    ui::tabs("view", vec![
        ui::tab("draw", "Draw", vec![
            ui::canvas("pad", "Sketch pad", 300.0, 200.0, vec![
                draw::rounded_rect([0.0, 0.0], [300.0, 200.0], 8.0,
                    paint::linear([0.0, 0.0], [300.0, 0.0], vec![paint::stop(0.0, [0.1, 0.1, 0.1, 1.0]), paint::stop(1.0, [0.3, 0.3, 0.3, 1.0])])),
                draw::path(path_cmds).stroke([1.0, 1.0, 1.0, 1.0], 2.0),
            ]).listen(&[Listen::Move, Listen::Wheel]).cursor(Cursor::Crosshair),
        ]),
        ui::tab("notes", "Notes", vec![ui::text_area("notes", "Notes", &notes, 6)]),
    ], None),
    ui::number("width", "Width", 64.0).range(1.0, 4096.0).unit("px"),
    ui::button("delete", "Delete").icon("trash").hint("Delete the selected layers"),
]);
debug_assert_eq!(ui::level(&tree), Ok(ui::UI_LEVEL)); // declare <app>.session@1.1
ui::set_panel("layers", tree)?;
```

## For app makers

`craft_ext_ui::plugin_ui` is the one renderer; apps do not keep their own.

- `PluginUi::new(EngineChoice::Egui)` once per app. `window(ctx, key, title, rev, tree_json, &mut
  open, |w| w.default_pos(…), host)` draws one plugin surface as a window and returns its
  `UiEvent`s; `draw_json` / `draw` draw into any `Ui` (a dock tab, a side panel). Trees are parsed
  and validated once per revision. `forget(key)` when a surface closes.
- Route each returned `UiEvent` to the plugin as `{"event":"ui", …}` (`serde_json::to_value(&e)`
  gives `surface`, `rev`, `widget`, `value`) and drop it when `e.is_stale(current_rev)`.
- `Host` carries what stays yours: `widget` (automation ids, accessibility), `component` (native
  framework components), `tr` (your translations of host words) and `form_titles`.
- Validate trees in your `ui.panel.set` / `ui.window.open` handler with
  `craft_ext_core::ui::tree::Node::parse` (or `validate_at(ui_level_for_session(declared))`).
  Provide `<app>.session@1.1` once the shared renderer draws your panels.
- `FormView` holds a form with its own state for dialogs (live edits, `apply_patch`,
  `replace_form`). `paint_ops(painter, Xf::new(offset, zoom), clip, ops, images)` paints draw
  lists such as canvas overlays.
- The `framework` cargo feature marks where the craft-framework engine plugs in; until it exists,
  `EngineChoice::Framework` resolves to egui.

## Relation to craft-framework

The form schema mirrors craft-framework's `Param` / `ParamKind` names. Differences to resolve when
the framework's renderer becomes an engine here (none affect plugins, which only see the schema):

- The framework's `Range` has a `scale` (linear, log, power) and `decimals`; `Number` has `step` and
  `decimals`; `Int` bounds are optional. Schema 1 has `Range.step` only and required `Int` bounds.
- The framework distinguishes `hint` (placeholder of a text row) from `tooltip` (row hover text);
  schema 1 has `help`, which is the tooltip. The SDK offers `Form::tooltip` as an alias.
- The framework has `Grid`, `Channel`, `OptionalNumber` and host-drawn `Custom` rows, and stores
  colours as sRGB `[r,g,b]`; schema 1 has none of these and keeps straight working-space RGBA with
  optional alpha. Its `Point` is always percent; schema 1 declares the space.
- Its tool pointer events (`Down`/`Move`/`Up` with button, modifiers and pen samples) match the
  canvas events here, minus pressure and tilt.
- Its icon catalogue uses meaning-based names; the icon names here are a subset of them.


The host-side installation boundary and the checked future framework/theme revision
are documented in [framework integration](framework-integration.md). No future
framework runtime is linked or activated by this beta source change.

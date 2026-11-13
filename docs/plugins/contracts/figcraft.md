# FigCraft plugin contract

The FigCraft adapter foundation lives in `figcraft/crates/plugins`. It declares
`figcraft.session@1.0`, `resource.font@1.0`, and `craft-theme@1.0`.
The session adapter is active; resource activation remains deferred.
No `raster.filter` contract is advertised pending image-fill dispatch integration.

## Contracts provided

| Contract | Version |
|---|---|
| `figcraft.session` | 1.0 |
| `resource.font` | 1.0 |
| `craft-theme` | 1.0 |

## Session protocol

Session events: init, command `{id}`, ui `{surface,rev,widget,value}`, tool
`{phase:begin|update|commit|cancel,at:[x,y],modifiers}`, app
`{type:selection.changed|document.opened|page.changed}`, dispose.
App events need `app.events`.

| Operation | Required permission |
|---|---|
| document.info, page.list, node.get, selection.set | document.read:document |
| selection.get | document.read:selection |
| node.create, node.update, node.delete | document.edit:document |
| viewport.get, ui.panel.set, ui.notify, canvas.overlay.set | free |
| ui.window.open | ui.window |

Selected-target scope support is deferred: node operations conservatively require
whole-document access. Never trust a guest-supplied selection flag.
Document dispatch, history transactions, commands, panels/windows, tools, native helpers,
project prompts and management are integrated. Font/theme activation remains deferred. See FigCraft `docs/plugin-migration.md` for named TODOs.

## Form schema v1 renderer

Panels and windows render all Node::Form kinds, defaults and host-evaluated conditions.
Choice/radio events use option IDs; widget IDs are parameter keys with surface tree revisions.
Unknown components render fallback. Working-space HDR color channels use numeric controls.

ui.panel.set {panel, patch} and ui.window.open {id, patch} patch an existing root Form.
Patch schema is 1; base_rev is the surface revision and rev must increase. Validation and
publication are atomic with event commit. Full replacements clear hide/disable flags;
show/enable restore original conditions. Optional WASM describe/update exports are unused
until a raster.filter consumer is integrated; session forms arrive through UI operations.

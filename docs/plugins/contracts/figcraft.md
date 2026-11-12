# FigCraft plugin contract (integration pending)

The FigCraft adapter foundation lives in `figcraft/crates/plugins`. It declares
`figcraft.session@1.0`, `resource.font@1.0`, and `craft-theme@1.0`.
These declarations are not yet activated in the application.
No `raster.filter` contract is advertised pending image-fill dispatch integration.

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
Document dispatch, history transactions, command registration, panel rendering,
tools, native helpers, project prompts, font registration and management integration
remain pending. See FigCraft `docs/plugin-migration.md` for named TODOs.

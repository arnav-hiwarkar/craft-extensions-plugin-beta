# Shapes (FigCraft example)

A session plugin for FigCraft (`figcraft.session@1.1`). Everything it makes is an ordinary
FigCraft layer: uninstalling the plugin keeps the layers.

- **Shapes** panel: a form (shape, size, count, colour) and a **Create shapes** button.
- **Create shapes** command (Plugins ▸ Shapes ▸ Create shapes): adds a row of `count` shapes with
  `node.create`. One run is one undo step.
- **Draw rectangle** tool: press, drag and release on the canvas. A live rectangle and a dashed
  guide (`canvas.overlay.set`) follow the pointer; Escape cancels and FigCraft rolls the whole
  gesture back.
- Needs `document.edit:document`. Without it the panel explains instead of failing.
  `app.events` is optional: when granted, the panel shows the latest app event.

```sh
craft-ext build   # copies figcraft_shapes.wasm here
craft-ext check
craft-ext pack --out figcraft-shapes.craftplugin
figcraft-cli plugin install-file figcraft-shapes.craftplugin
```

See FigCraft's `docs/plugin-testing.md` for the full walk-through.

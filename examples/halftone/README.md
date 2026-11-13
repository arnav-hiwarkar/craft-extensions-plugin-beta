# Halftone

A `raster.filter@1.0` effect for EffectCraft and PhotoCraft. FigCraft does not implement this contract. It draws round dots on a
rotated grid. Each dot's area follows the darkness of its cell, so shadows get big dots and highlights
get small ones.

The form is built by `describe()` in `src/lib.rs`; the manifest also carries a static form.
Use `craft-ext pack --describe` to embed the SDK description without editing the source manifest:

- **Dot size** (2–64, default 8): grid cell in layer pixels.
- **Grid angle** (degrees, default 45): rotation of the grid.
- **Black and white** (default on): black dots on white paper. Off keeps each cell's colour.

Output is premultiplied RGBA `f32`. The result depends only on the context and the pixels, so
renders are repeatable.

Build the module:

```sh
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/halftone.wasm .
```

`craft-ext build` and `craft-ext check` do the same and check the manifest against the module.

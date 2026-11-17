# Tonemask

A `photocraft.session@1.1` panel for luminosity masks. Pick one of ten tone ranges (Lights 1 to 4,
Midtones 1 and 2, Darks 1 to 4) to select it in the active layer, then **Save as channel** or **Add
as layer mask**, each as one undo step.

```sh
cargo test
craft-ext build --release && craft-ext check && craft-ext test
craft-ext pack --out tonemask.craftplugin
```

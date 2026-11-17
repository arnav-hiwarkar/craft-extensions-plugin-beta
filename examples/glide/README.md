# Glide

An `effectcraft.session@1.2` panel. Pick an easing curve (Gentle, Smooth, Strong, Slow start,
Slow end or Snap) and a property, then **Apply to selected layers** eases every key of that
property on each selected layer through `prop.ease`, as one undo step.

Glide reads the selection and the document and needs `document.edit:document` to change keys.

```sh
cargo test
craft-ext build --release && craft-ext check && craft-ext test
craft-ext pack --out glide.craftplugin
```

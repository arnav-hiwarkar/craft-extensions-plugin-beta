# Softskin

A `raster.filter@1.0` portrait effect for PhotoCraft and EffectCraft. It smooths skin with
`kernel.bilateral`, which keeps edges, then restores fine texture by **Keep detail**. With
**Skin only** on, a soft skin-tone mask from `kernel.hsl_range` limits the smoothing to skin, so
eyes, lips and hair stay sharp. **Amount** mixes the result with the original through
`kernel.mix_masked`.

```sh
cargo test
craft-ext build --release && craft-ext check && craft-ext test
craft-ext pack --describe --out softskin.craftplugin
```

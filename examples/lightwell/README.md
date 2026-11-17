# Lightwell

A `raster.filter@1.0` effect for EffectCraft and PhotoCraft. It finds the brightest parts of the
image above **Threshold**, blurs them at several scales and screens the glow back over the image.
**Light rays** adds a directional streak at **Ray angle** with **Ray length**.

All heavy work runs in host kernels (`kernel.blur`, `kernel.directional_blur`, `kernel.blend`), so
a 1080p frame takes about half a second in the sandbox. The unit tests run the same kernels
natively:

```sh
cargo test
craft-ext build --release && craft-ext check && craft-ext test
craft-ext pack --describe --out lightwell.craftplugin
```

# Cross-app vignette (MIT)

Sandboxed `raster.filter@1.0`, no permissions. Premultiplied RGB is darkened with a
quadratic distance ramp; alpha and HDR values are preserved. Coordinates come from
full-frame origin/size so rendering a tile gives the same result as that region
of the full image. No randomness, state, clock or filesystem access.

The manifest form includes a percent amount, stable oval/round choice, and a
pixel radius visible only in round mode. Defaults and conditions are host-owned.

```sh
craft-ext build --release
craft-ext check
craft-ext test
craft-ext pack --out cross-vignette.craftplugin
```

Run `../../scripts/cross-app-smoke.sh` for app CLI verification. EffectCraft and
PhotoCraft provide the filter contract. **FigCraft currently lacks raster filter
dispatch and refuses installation**; listing it in `apps` expresses the intended
portability, not a completed FigCraft integration. Do not advertise the contract
until actual image-fill dispatch exists.

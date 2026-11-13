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
PhotoCraft provide the filter contract. FigCraft has no raster filter dispatch,
so this package declares only EffectCraft and PhotoCraft. Cross Review demonstrates
session contributions for all three apps.

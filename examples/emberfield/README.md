# Emberfield

A `raster.filter@1.0` particle effect for EffectCraft. Sparks leave the **Emitter** point at
**Particles per second**, live for **Lifetime**, and move with **Speed**, **Direction**, **Spread**,
**Gravity** and **Turbulence**. Colour fades from **Birth colour** to **Death colour**.

Particles are simulated from the frame time and a fixed seed, so any frame renders the same on
its own, in any order. They are drawn with `kernel.splat` and softened with `kernel.blur` for
**Glow**.

```sh
cargo test
craft-ext build --release && craft-ext check && craft-ext test
craft-ext pack --describe --out emberfield.craftplugin
```

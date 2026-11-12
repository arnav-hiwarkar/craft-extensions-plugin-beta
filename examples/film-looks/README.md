# Film looks

Data only: no code runs. It adds two things to EffectCraft and PhotoCraft:

- **Film warm** (`luts/film-warm.cube`): a 2×2×2 `.cube` 3D LUT that lifts blacks slightly, keeps
  reds and greens, and cools blues. Replace it with a real 17³ or 33³ LUT before you ship.
- **Halftone print looks** (`presets/print-looks.json`): two presets that set the Halftone effect
  (`io.github.arnav-hiwarkar.halftone`) to a black-and-white and a colour grid.

It requires the Halftone package (`^1.0`). Install both to use the presets.

Validate the data before packing with `craft-ext check`.

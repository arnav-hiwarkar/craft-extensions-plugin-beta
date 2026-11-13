# Historical Linux testing guide

Do not execute the build/test scripts below in the source-only continuation phase. Relative paths were relative to the old Linux workspace root. The signed local test catalogue and signing key are not included in the source transfer.

# Testing the Craft plugin beta yourself

Takeover note (2026-11-13): the binaries and signed catalogue below were prepared before the latest UI 1.1, tool and review fixes. Rebuild the affected apps/CLIs and refresh the catalogue before treating these instructions as current acceptance evidence. See STATUS.md. Commands come from the app guides linked in [Known limits](#7-known-limits); `ROOT=~/projects/plugin-beta`.

## 0. What's built and where

| What | Path |
|---|---|
| EffectCraft desktop / CLI | `effectcraft/target/debug/effectcraft`, `effectcraft-cli` |
| PhotoCraft desktop / CLI | `photocraft/target/debug/photocraft`, `photocraft-cli` |
| FigCraft desktop / CLI | `figcraft/target/debug/figcraft`, `figcraft-cli` |
| ArtCraft Launcher / author tool | `craft-launcher/target/debug/artcraft-launcher`, `craft-extensions/target/debug/craft-ext` |

All are **debug**: fine for panels, menus and installs, wrong for judging speed. Effects run in a single-threaded Wasm interpreter (halftone ≈17 s for 1920×1080 here) and debug-built *plugin modules* break the per-pixel fuel budget, so every render fails as "too slow"; catalogue archives are release-built.

## 1. One-time environment setup

```sh
ROOT=~/projects/plugin-beta
export CRAFT_EXT_DIR="$(mktemp -d)"                    # throwaway plugin store
export CRAFT_CATALOGUE_URL="file://$ROOT/test-catalogue/"
export CRAFT_CATALOGUE_ALLOW_FILE=1
```

Apps from this shell share the store and catalogue; the flag is harmless for `file://`, needed only for `https` catalogues with local assets, and never skips signatures or revocations. A replay refusal (lower serial) means a fresh `CRAFT_EXT_DIR`.

## 2. Quick automated check

`cd $ROOT/craft-extensions && ./scripts/e2e.sh` rebuilds `craft-ext`, authors effect/panel/data projects, packs the examples plus variants, signs the catalogue, then walks installs, permissions, updates, rollback, revocations, local files and project references in all three apps. Needs Rust with `wasm32-unknown-unknown`, Python 3, `flock` and the signing key, shares the lock, and ends with `E2E PASS`. It republishes `test-catalogue/` while running, so let it finish first; it runs no GUI.

## 3. Plugin maker flow

```sh
X=$ROOT/craft-extensions/target/debug/craft-ext
cd "$(mktemp -d)"
$X new effect my-effect --id io.github.you.my-effect && cd my-effect   # effect | panel | data
$X build --release   # what the catalogue ships; debug modules blow the fuel budget
$X check             # manifest, ABI and contract checks
$X test              # deterministic guest runs
$X pack --describe --out my-effect.craftplugin
$X plugin --app effectcraft install-file my-effect.craftplugin
$X plugin --app effectcraft setup io.github.you.my-effect --enable <contribution> --grant <permission>
```

A local catalogue instead of one archive: `catalogue build --local-assets <archives> --out <catalogue>` then `catalogue sign --repo <catalogue> --key ~/.config/craft/catalogue.key`. An optional `revocations.toml` there supplies revocations; local builds get Community tiers and zero stars.

## 4. Per-app checklists

### EffectCraft

`B=$ROOT/effectcraft/target/debug/effectcraft-cli`; `H=io.github.arnav-hiwarkar.halftone`,
`N=io.github.arnav-hiwarkar.color-notes`, `F=io.github.arnav-hiwarkar.film-looks`.

```sh
$B plugin refresh && $B plugin search halftone   # 1.0.0 compatible, 2.0.0 flagged incompatible
$B plugin install $H && $B plugin permissions $H && $B plugin setup $H --all
$B --empty run comp.new '{"name":"Main","width":320,"height":180,"duration":2}' \
  layer.newSolid '{"name":"Grey","color":"#808080"}' --save-as base.ecproj
$B --project base.ecproj run layer.select '{"layers":["Grey"]}' \
  effect.apply "{\"effect\":\"$H/halftone\"}" --save-as fx.ecproj
$B --project fx.ecproj render-frame --out fx.png
$B --project fx.ecproj set - Grey effects/#1/size 20 --save
$B --project fx.ecproj props - Grey --flat | grep effects/   # values survive reopen
$B plugin install $N && $B plugin setup $N --all --grant document.read:selection
$B plugin install $F --version =1.0.0 && $B plugin setup $F --all
$B --empty exec plugin.resources                              # LUT path + presets
$B plugin updates && $B plugin update $F && $B plugin rollback $F 1.0.0
```

GUI, same store: **Plugins ▸ Manage Plugins…** → search, Download, Enable. In `fx.ecproj` add **Effect
▸ Stylize ▸ Halftone**; **Plugins ▸ Color notes ▸ Color notes** opens the panel (**Add note marker** is
one undo step); **Plugins ▸ Film looks ▸ Halftone print** applies a preset, and the LUT uses **Effect ▸
Utility ▸ Apply Color LUT**. Save/reopen (values, id, version, sha256), disable the plugin and reopen
(placeholder plus "This project uses 1 plugin…"), then **File ▸ Export ▸ Add to Render Queue** → Render
must refuse.

### PhotoCraft

```sh
cd $ROOT/photocraft && CLI=./target/debug/photocraft-cli
HALF=io.github.arnav-hiwarkar.halftone
$CLI plugin refresh && $CLI plugin install $HALF && $CLI plugin setup "$HALF" --all
$CLI run --new '{"width":32,"height":32,"background":"#806040"}' \
  --cmd "plugin.$HALF/halftone" --params '{"size":6,"angle":45,"mono":true}' \
  --out /tmp/halftone.pcraft
$CLI run /tmp/halftone.pcraft --out /tmp/halftone.png
$CLI run --new '{"width":4,"height":4,"mode":"cmyk"}' --cmd "plugin.$HALF/halftone"   # refused
$CLI run --new '{"width":4,"height":4,"mode":"lab"}'  --cmd "plugin.$HALF/halftone"   # refused
```

GUI: **Filter ▸ Plugins ▸ Stylize ▸ Halftone**; change Dot size, Grid angle and Black and white, try the
"Halftone print" and "Colour halftone" presets, Apply, confirm one "Halftone · Halftone" History entry,
Undo/Redo, save `.pcraft`, export PNG, repeat in RGB 32-bit. CMYK and Lab must disable the filter with
"This filter works on RGB images"; masks, channels and Quick Mask show their refusal; **Image ▸
Adjustments ▸ Color Lookup** lists "From plugins / Film warm".

### FigCraft

```sh
cd $ROOT/figcraft && CLI=./target/debug/figcraft-cli
ID=io.github.arnav-hiwarkar.figcraft-shapes
$CLI plugin refresh && $CLI plugin install $ID && $CLI plugin permissions $ID
$CLI plugin setup $ID --all        # refused: "Permission needed. Fix: plugin grant …"
$CLI plugin setup $ID --all --grant document.edit:document --grant app.events
echo "[{\"command\":\"plugin.$ID.create-shapes\",\"params\":{}}]" > /tmp/shapes.json
$CLI run /tmp/shapes.json --plugins --out /tmp/shapes.figcraft   # 3 shapes, one undo step
$CLI plugin project /tmp/shapes.figcraft                          # "ready"
```

GUI: **Plugins ▸ Manage plugins…** → Browse → "shapes" → Install 1.0.0 → enable the three contributions
and allow "Change the current document". **Plugins ▸ Shapes** toggles the panel: set
Shape/Size/Count/Colour and click **Create shapes** — one undo removes the row, Redo restores it; the
menu command matches. The **D** tool press-drags a live rectangle with pink outline, cross-hairs and a size
label (the guide overlay); release keeps one undo step, Escape cancels. Save `.figcraft`, remove Shapes,
reopen: it is listed missing and **Open without them** keeps every layer.

## 5. Launcher Plugins button

```sh
CRAFT_EXT_DIR=<dir> CRAFT_CATALOGUE_URL="file://$ROOT/test-catalogue/" \
  CRAFT_CATALOGUE_ALLOW_FILE=1 $ROOT/craft-launcher/target/debug/artcraft-launcher
```

Open **Plugins** in the header (desktop only): the index refreshes on first open and files download only
when chosen. It lists plugins for every Craft app, installs from the local catalogue, and offers setup per
app.

## 6. Things that must be refused

- **Kill-listed** package: install/enable fails and `plugin project` says "unavailable" or "release
  withdrawn". Build a second catalogue whose `revocations.toml` names halftone, re-sign it with a higher
  serial, refresh.
- **Incompatible halftone 2.0.0** (advertises `raster.filter@2.0`, offers 1.0.0): `plugin versions` says
  incompatible and `install --version =2.0.0` says no listed version works; FigCraft refuses both
  versions because it offers no `raster.filter` at all.
- **Enabling without a required permission**: exits 4 with "Permission needed"; `--allow-all` is
  rejected, and a native package without `--full-access` needs explicit consent.
- **Revoking mid-run** (EffectCraft `document.edit:document`, FigCraft mid-gesture): the plugin gets a
  recoverable denial, panel/menu items vanish next frame, and an in-flight gesture is rolled back with no
  layers and no undo step; granting again recovers.

## 7. Known limits

Migration status and remaining gaps: [effectcraft](effectcraft/docs/plugin-migration.md),
[photocraft](photocraft/docs/plugin-migration.md), [figcraft](figcraft/docs/plugin-migration.md).

Also: debug plugin modules fail the fuel budget; halftone is slow and single-threaded; a saved LUT path
points into the active version's release directory, so re-pick it after update, rollback or pruning; safe
mode (`--safe-mode`, `EFFECTCRAFT_SAFE_MODE=1`, `FIGCRAFT_SAFE_MODE=1`) loads no plugin code; FigCraft has
no `raster.filter` contract; PhotoCraft's renderer is still egui (`plugin_form::draw` is the seam). **No
GUI flow has been run automatically** — the clicks above are your job; the CLIs drive the same commands.
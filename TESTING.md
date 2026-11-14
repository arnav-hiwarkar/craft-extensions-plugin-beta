# Testing the Craft plugin beta on a Mac

Builds and automated checks are recorded by the `plugin-beta` workflow in
[arnav-hiwarkar/craft-extensions-plugin-beta](https://github.com/arnav-hiwarkar/craft-extensions-plugin-beta/actions/workflows/plugin-beta.yml).
You do not need Rust or a local build: download the artifacts of a green run and test those.
All builds are **debug** builds. Do not use them for real work.

## 1. Download a green run

```sh
REPO=arnav-hiwarkar/craft-extensions-plugin-beta
RUN=$(gh run list --repo $REPO --workflow plugin-beta --branch plugin-beta --status success -L 1 --json databaseId -q '.[0].databaseId')
B=~/craft-beta && mkdir -p $B && cd $B
for a in effectcraft-macos-arm64 photocraft-macos-arm64 figcraft-macos-arm64 craft-launcher-macos-arm64 craft-ext-macos-arm64 test-catalogue wasm-packages; do
  gh run download $RUN --repo $REPO -n $a -D $a
done
mkdir -p bin && for t in */*.tgz; do tar -C bin -xzf "$t"; done
xattr -dr com.apple.quarantine bin 2>/dev/null || true   # only needed if you downloaded in a browser
ls bin   # effectcraft effectcraft-cli photocraft photocraft-cli figcraft figcraft-cli artcraft-launcher craft-ext
```

The binaries are unsigned. If macOS still refuses one, right-click it in Finder ▸ Open once.
Artifacts expire after 14 days; select a newer successful run with available artifacts.
For separate run folders and prepared beta bundles, follow
[isolated beta installation](docs/plugins/installation-and-testing.md).

## 2. Point every app at the test catalogue

The `test-catalogue` artifact is a complete signed catalogue containing every example. Its asset
URLs are relative, so it works from wherever you unpacked it. It is signed with a throwaway
**test** key (`TEST-KEY.pub`). Debug builds trust that key only when all three variables below are
set and the catalogue URL is `file://`; release builds and HTTPS catalogues never trust it.

```sh
cd ~/craft-beta
export CRAFT_CATALOGUE_URL="file://$PWD/test-catalogue/"     # keep the trailing /
export CRAFT_CATALOGUE_ALLOW_FILE=1
export CRAFT_CATALOGUE_TEST_KEY=$(cat test-catalogue/TEST-KEY.pub)
export CRAFT_EXT_DIR="$(mktemp -d)"                          # a throwaway plugin store
BETA_SETTINGS="$(mktemp -d)"
export EFFECTCRAFT_CONFIG_DIR="$BETA_SETTINGS/effectcraft"
export PHOTOCRAFT_CONFIG_DIR="$BETA_SETTINGS/photocraft"
export FIGCRAFT_CONFIG_DIR="$BETA_SETTINGS/figcraft"
mkdir -p "$EFFECTCRAFT_CONFIG_DIR" "$PHOTOCRAFT_CONFIG_DIR" "$FIGCRAFT_CONFIG_DIR"
touch "$PWD/bin/portable.txt" # launcher settings in bin/ArtCraftLauncherData
export PATH="$PWD/bin:$PATH"
```

`CRAFT_EXT_DIR` isolates only the plugin store. The three config overrides isolate app
preferences, and the portable marker isolates launcher settings. These directories persist
after quitting.

Launch the apps **from this shell** so they inherit the variables (`effectcraft &`, `photocraft &`,
`figcraft &`, `artcraft-launcher &`). The catalogue expires 30 days after the run that built it.
If an app says the catalogue is a replay (lower serial than one the store has seen), start again
with a new `CRAFT_EXT_DIR`.

What the catalogue contains, by app:

| App | Packages |
|---|---|
| EffectCraft | halftone, film-looks, color-notes, cross-vignette, cross-review, effectcraft-channel-tint, effectcraft-cue-board, effectcraft-point-tool, native-system-stamp |
| PhotoCraft | halftone, film-looks, cross-vignette, cross-review, photocraft-gradient-tool, photocraft-levels-scope, photocraft-layer-export |
| FigCraft | cross-review, figcraft-grid, figcraft-placeholder, figcraft-shapes, native-system-stamp |

FigCraft has no raster filters, so halftone and cross-vignette are not offered there.

## 3. Quick CLI check (two minutes)

```sh
effectcraft-cli plugin refresh                     # "Catalogue refreshed" (signature verified)
effectcraft-cli plugin search ""                   # every EffectCraft package, with tier and stars
effectcraft-cli plugin install io.github.arnav-hiwarkar.color-notes      # installs, enables nothing
effectcraft-cli plugin permissions io.github.arnav-hiwarkar.color-notes
effectcraft-cli plugin setup io.github.arnav-hiwarkar.color-notes --all # refused: permission needed
effectcraft-cli plugin setup io.github.arnav-hiwarkar.color-notes --all --grant document.read:selection
effectcraft-cli plugin list
```

`photocraft-cli plugin …` and `figcraft-cli plugin …` work the same; all apps share one store, but
enablement and grants are per app. `craft-ext plugin --app <app> …` is the same manager without an app.

## 4. EffectCraft checklist

1. **Plugins ▸ Manage Plugins…** opens Settings ▸ Plugins (Browse / Installed / Updates / Problems).
   Browse lists the catalogue with tiers and stars. Download **Halftone**: nothing is enabled yet.
   **Enable** opens the setup sheet; confirm. Effect ▸ Stylize ▸ **Halftone** now exists.
2. New project, add a solid layer, apply **Halftone**. Effect Controls shows Dot size, Grid angle,
   Black and white. Change a value, **Edit ▸ Undo** reverts exactly one step.
3. **Save**, quit, relaunch, reopen: the effect and its values are intact.
4. Disable Halftone in the manager and reopen the project: “This project uses 1 plugin…” offers
   Continue / Open without. The effect stays as a placeholder; rendering refuses and names the plugin.
5. Install **Color notes**, enable with “read the selection”. **Plugins ▸ Color notes** opens its panel;
   select a layer: its name appears. Press **Add note marker**: refused until you grant “edit the
   document”. Grant it: one marker, one Undo step named “Add note marker · Color notes”. Revoke it in
   the manager: the next press is refused at once.
6. **Point tool**, **Cue board**, **Channel tint**: install with the requested grants. Drag with the
   point tool (one keyframe, one Undo; Escape cancels), add a cue (one marker), apply Channel tint.
7. **Native System Stamp**: setup requires the edit permission plus the separate full-computer consent (“Native: full access to
   your computer”). After consent, **Plugins ▸ System stamp** adds a text layer naming your OS; Undo
   removes it. Withdraw consent in the manager: the helper stops and the command is gone.
8. **Updates / pins / rollback**: the catalogue has Film looks 1.0.0 and 1.1.0.
   `effectcraft-cli plugin install io.github.arnav-hiwarkar.film-looks --version =1.0.0`, then the
   manager's Updates tab offers 1.1.0; update, then roll back to 1.0.0 from Installed (or
   `plugin rollback <id> 1.0.0`); `plugin pin <id> =1.0.0` stops update offers until `plugin unpin`.
9. Safe mode: `effectcraft --safe-mode` (or `EFFECTCRAFT_SAFE_MODE=1`; PhotoCraft and FigCraft use
   `PHOTOCRAFT_SAFE_MODE=1` / `FIGCRAFT_SAFE_MODE=1`): no plugin code runs, the manager and projects still work.

## 5. PhotoCraft checklist

1. **Plugins ▸ Plugins…** opens the manager; install **Halftone** and **Film looks**, enable both.
2. Open or create an RGB image. **Filter ▸ Plugins ▸ Stylize ▸ Halftone**: change Dot size / angle /
   Black and white, try the presets. Apply: one “Halftone” History entry; Undo/Redo.
3. CMYK or Lab document: the filter is disabled with “This filter works on RGB images”.
4. **Image ▸ Adjustments ▸ Color Lookup ▸ From plugins ▸ Film warm**, also as an adjustment layer.
5. **Gradient tool**, **Levels scope** (histogram panel, Auto levels layer), **Layer export**
   (exports to a folder you pick; the plugin never sees a path). Revoke `files.write:picked` and retry:
   refused. Exported files are not removed by Undo (by design).
6. Save `.pcraft`, reopen: filters are applied destructively (baked pixels), so a PhotoCraft
   document never needs a plugin to reopen and there is no missing-plugin prompt. A plugin LUT
   layer keeps its `plugin://` reference and an embedded table.

## 6. FigCraft checklist

1. **Plugins ▸ Manage plugins…**, install **Shapes**, allow “Change the current document”.
2. **Plugins ▸ Shapes** docks the panel. Create shapes: a row of layers, one Undo step.
3. The **D** tool draws a rectangle with a live guide overlay; Escape cancels with no History entry.
4. Revoke the edit permission: panel, menu items and tool disappear on the next frame.
5. **Grid**, **Placeholder**, **Cross review** (renames the selected node “Reviewed”; one Undo),
   **System stamp** (native, needs full-access consent).
6. Save as `.figcraft`, remove Shapes, reopen: the project prompt lists it; Open without keeps every layer.

## 7. ArtCraft Launcher

`artcraft-launcher &` from the same shell. **Plugins** in the header shows the same catalogue for all
apps. Install a plugin there and set it up per app; open that app: it is already enabled there and
nowhere else.

## 8. What CI already checked

The latest recorded evidence is [Run #20](https://github.com/arnav-hiwarkar/craft-extensions-plugin-beta/actions/runs/43551011455),
2026-11-14, commit `909897f`. The recorded Linux x86_64 and macOS arm64 jobs passed:

- the shared crates (`ext-core`, `ext-store`, `ext-host`, `ext-ui`, `ext-cli`, `guest`) with all tests;
- each app and its CLI (debug), plus its plugin adapter and plugin UI-host tests (Linux);
- all 15 examples: release wasm, `craft-ext check`, `test`, `pack`; native helpers for Linux x86_64 and macOS arm64;
- the signed test catalogue, then on **both** Linux and macOS: every package installed in every app it
  declares (install never enables; setup refused without grants; revocation bumps the grant generation),
  the cross-app smoke (edit, Undo, revoke, render determinism), the EffectCraft session smoke, and on
  Linux the shared end-to-end manager flow (updates, pins, rollback, kill list, forged signature,
  changed metadata, local installs, project references).

The numbered app checklists above are procedures, not completed GUI evidence. Targeted
local Cross Review edit/Undo checks do not establish full app verification. No new app,
guest, build or test-suite run was performed for this documentation correction.

Not checked by CI: anything needing a display (GUI clicks above), GPU rendering, Windows (optional
compile check only), notarised or release builds.

## Known limits

See [STATUS.md](STATUS.md#open-items-and-known-limits). The docs are in [docs/plugins](docs/plugins/README.md).

## Rules for these repositories

Nothing is ever pushed to storytold repositories. All branches, CI, artifacts and settings live only
in `arnav-hiwarkar/*` and the private Forgejo mirror. No releases, tags, PRs or issues without approval.

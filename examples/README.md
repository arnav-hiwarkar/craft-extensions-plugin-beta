# Plugin examples

These directories contain plugin source and manifests. Source availability is
separate from runtime verification. New Cross Review panels and helper regression
source have not been compiled or executed in the current source-only pass. Older
README build and smoke commands describe later workflows, not completed checks.

| App | Examples and purpose |
| --- | --- |
| EffectCraft | `effectcraft-cue-board`: composition markers and layer notes; `effectcraft-point-tool`: pointer-driven point editing; `effectcraft-channel-tint`: raster channel tint; `color-notes`: selection notes and optional edits |
| PhotoCraft | `photocraft-gradient-tool`: gradient tool and options; `photocraft-levels-scope`: histogram and levels; `photocraft-layer-export`: picked-location layer export |
| FigCraft | `figcraft-grid`: editable card grids; `figcraft-placeholder`: placeholder content; `figcraft-shapes`: shape creation and rectangle tool |
| Cross-app | `cross-review`: canonical layer/node rename commands and shared-vocabulary panels in all three apps; `cross-vignette`: raster effect for EffectCraft and PhotoCraft; `halftone`: raster effect; `film-looks`: LUT and preset resources |

Read each `craft-plugin.toml` for the actual supported apps, contribution IDs,
contract levels, module names, forms, and requested permissions. FigCraft has no
`raster.filter` dispatch; Cross Review demonstrates cross-app session behavior,
not a FigCraft raster effect.

Session contracts are app-specific (`effectcraft.session`, `photocraft.session`,
`figcraft.session`). UI 1.1 controls require a matching session minor level.
The same declarative UI vocabulary is drawn by the shared host renderer. Plugins
must not detect its backend. Named components require valid fallback trees when
used. Cross Review uses basic controls and needs no named component.

Permissions apply to the calling contribution and live grants. Document edit
permission covers reads of the same scope; selection scope does not authorize
whole-document enumeration. Optional permissions still require user grants.
Picked file permissions use opaque host-issued handles rather than arbitrary
paths. Exported files can survive a later error and are not restored by document
Undo. Native contributions require separate full-access consent.

The consumer parser accepts comma-separated contribution IDs after `--enable`
and repeated `--grant` options. For an already built package, examples are:

```sh
effectcraft-cli plugin install-file cross-review.craftplugin
effectcraft-cli plugin setup io.github.arnav-hiwarkar.cross-review --enable effectcraft-apply,effectcraft-panel --grant document.edit:document
photocraft-cli plugin install-file photocraft-layer-export.craftplugin
photocraft-cli plugin setup io.github.arnav-hiwarkar.photocraft-layer-export --enable export,export-all --grant document.read:document --grant files.write:picked
figcraft-cli plugin install-file figcraft-grid.craftplugin
figcraft-cli plugin setup io.github.arnav-hiwarkar.figcraft-grid --enable panel,apply --grant document.edit:document
```

Installation never grants or enables. `plugin permissions <id>` shows the
permission sheet; `plugin disable <id> <contribution>` and
`plugin revoke <id> <permission>` remove access. These commands come from the
shared `ext-cli` parser. Cross Review's README explains its separate panel module
outputs; the default author library build is insufficient for that package.

`native-system-stamp` demonstrates an out-of-process helper for EffectCraft and
FigCraft. Its committed executable mapping is **Linux x86_64 only**
(`linux-x86_64 = bin/helper`). It does not supply a Mac executable. Native means
full-computer access, not an OS sandbox, and external effects are not undoable.
A future verified native package requires explicit `--full-access` during setup.

No binary, package, signing, catalogue, release, or CI output is produced by this
source pass. Builds, app/guest execution, tests, smoke scripts, and platform
validation remain for a later authorized phase. In particular, the cross-app
smoke scripts invoke builds and must not be used as a syntax-only check.

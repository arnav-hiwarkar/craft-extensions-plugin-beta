# Craft plugin beta status

2026-11-14. Source for the shared platform (this repository), the three app adapters
(EffectCraft, PhotoCraft, FigCraft), the launcher integration and 15 examples is in place
on the `plugin-beta` branches. The latest recorded build and automated verification
is run #20 below. It does not establish GUI or GPU behavior. See
[TESTING.md](TESTING.md) to download the artifacts and test on a Mac, or use the
[isolated installation guide](docs/plugins/installation-and-testing.md) beside normal apps.

## Repository rule

Nothing is ever pushed to storytold repositories. All code, branches, CI runs, workflow
files, artifacts, visibility changes and secrets live only in `arnav-hiwarkar/*`
(`*-plugin-beta` and `craft-plugins`) and the private Forgejo mirror. Every `gh` call
names `--repo arnav-hiwarkar/<name>`. No releases, tags, PRs or issues without approval.

## Latest green run

[Run #20](https://github.com/arnav-hiwarkar/craft-extensions-plugin-beta/actions/runs/43551011455)
(2026-11-14, commit `909897f`): every job passed on Linux x86_64 and macOS arm64, and the
Windows compile check passed for craft-extensions, EffectCraft, PhotoCraft and FigCraft.
It includes both review rounds. 15 examples built, checked, tested and packed; the signed test
catalogue (all examples plus Film looks 1.0.0) installed in every declared app on both platforms;
cross-app, EffectCraft session and shared e2e smokes passed. Windows is compile-only: nothing
was run there.

## Automated coverage recorded in run #20

- Shared crates (`ext-core`, `ext-store`, `ext-host`, `ext-ui`, `ext-cli`, `guest`): debug
  build with all features and every unit test.
- Each app and its CLI (debug, Linux and macOS arm64) and the launcher; each app's plugin
  adapter and plugin UI-host tests on Linux.
- All 15 examples: release wasm, `craft-ext check`, `test`, `pack`; native helpers for
  Linux x86_64 and macOS arm64 packed into one archive.
- A signed, portable TEST catalogue with every example (artifact `test-catalogue`).
- On Linux and macOS: every package installed in every app it declares (install never
  enables; setup refused without required grants; revocation bumps the grant
  generation); the cross-app smoke (Cross Review and Native System Stamp edit with one
  Undo step in each app, deterministic plugin renders, refusal after revocation,
  FigCraft refuses raster filters); the EffectCraft session smoke; on Linux the shared
  end-to-end manager flow (signatures, forged metadata, updates, pins, rollback, kill
  list, local installs, project references).

Not verified by CI: GUI clicks, GPU paths, Windows (optional compile check only),
release or notarised builds.

## Documentation and workflow maps

The [plugin handbook](docs/plugins/README.md) links user, author, app-adapter and
maintainer procedures, plus CLI, capability, UI, native-protocol and versioning
references. The [workflow maps](docs/plugins/maps.md) include a mind map and ten
workflow diagrams across that page and the native protocol reference. An offline
[interactive atlas](docs/plugins/maps/overview.html) provides role navigation and
five workflow tabs. The [coverage index](docs/plugins/coverage.md) maps guides to
source owners and separates implemented behavior from verification evidence.

This documentation pass checked local links and heading anchors, fenced JSON,
diagram headers/fences, and atlas controls and mobile overflow. Run
`python3 scripts/check-plugin-docs.py` to repeat the structural checks without
building or executing plugins. The checker does not fully parse/render Mermaid.
No new app, guest, build, suite or GUI workflow verification accompanied these edits.

## Fixed in this phase

From CI: `setup --all` and `craft-ext dev` tried to enable other apps' contracts;
leftover UI for PhotoCraft's retired Install Plug-in dialog; several test fixtures that
had never run (FigCraft notification races, PhotoCraft publication failures, the native
host test). From two independent reviews: native consent is now per version; disabling
one contribution stops its running work; host calls cost fuel; advisory revocations no
longer block installs; an interrupted install no longer blocks that version; apps reload
plugins only when their own plugin setup changes; EffectCraft and PhotoCraft refuse
outside edits during a plugin gesture; EffectCraft reports rolled-back events as
failures; PhotoCraft panels stay closed.

Second round (deferred review items): FigCraft honours the missing-plugin prompt per
document and asks once per document; EffectCraft and FigCraft keep saved plugin references
on the loaded release when the saved state version matches, listing only used
contributions; native helpers run in their own process group, which is stopped with them;
the catalogue cache is one file written atomically under the store lock after the serial
check; all three apps enforce the declared UI level; unavailable plugin commands explain
why; the guest SDK exposes tool modifiers, pressure, button, command params and the full
event JSON.

## Open items and known limits

- Native is not an OS sandbox: a helper can still start processes outside its group on
  purpose, and its external effects cannot be undone.
- PhotoCraft applies plugin filters destructively and records no project references, so
  it never shows a missing-plugin prompt (LUT layers keep an embedded table). A pending
  PhotoCraft file question is dropped silently if that plugin reloads.
- For plugin authors: no persistent storage, network or clipboard operations, background
  jobs, panel open/close events, or panel form patches outside FigCraft yet.
- A plugin panel can request costly polygon drawing every frame (no per-revision cache).
- Pre-existing, not plugin work: the launcher's i18n tests race on a process-wide
  language, so CI runs them on one thread.
- Beta deferrals unchanged: theme activation, TUF, accounts and reviews, OS confinement
  of native helpers, temporal/tiling APIs.

Release CI and publication need separate approval.

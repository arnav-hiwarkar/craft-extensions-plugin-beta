> **Copied here on 04/11/2026 for handover.** This folder is now the source of truth. Paths such as `../craft-launcher-ext` are folders in Jeremy's local workspace; the handover section below gives the matching storytold branches. Run `python build.py` here after changing the plan.

# Craft extensions browser, new-session handoff (16/10/2026)

## Current checkpoint for the next worker

- **Source of truth:** `plan_tasks.py` now has Phase 4b (`ui-gate`, `ui-port`, `ui-link`, `ui-verify`) and Delivery (`ship-1` to `ship-4`). `plan_data.py` carries the integration decision; regenerate `index.html` with `build.py` after edits. The older design notes below describe how we got here, not permission to skip these gates.
- **Order:** complete the first-party core/store, catalogue download and PhotoCraft plug-in host with a real installed plug-in; pin source dependencies so a clean checkout builds; only then extract the approved Extensions UI from the combined TEST BUILD into a clean launcher feature branch. Keep AI integration separate. Test end to end, obtain Jeremy's approval on the exact candidate, package a private MSI and portable ZIP, then seek explicit approval for source PRs and, separately, for any release-branch push.
- **Live worktrees checked 16/10/2026:** `../craft-extensions` at `33adcb4` and `../photocraft-wt-ext` at `ecb056cc` are clean. `../craft-launcher-ext` is `feat/extensions` at `70b9612` with 21 changed paths. `../craft-launcher-wt-combine` is `test/ext-plus-ai` at `f6a3a77` with 36 changed paths. Preserve both sets of uncommitted work; the combined branch is a showcase build, never the release source. Recheck `git status` and `git log` before changes, since Jeremy may have moved them.
- **Showcase is running:** the actual TEST BUILD executable at `../craft-launcher-wt-combine/target/dev/artcraft-launcher-ext-and-ai.exe` was relaunched with `--demo-data --open=extensions?kind=plugin` after the prior 0.4.1 instance exited through its idle-only replacement protocol. Process 45248 had the flags, and the app's accessibility tree showed `ArtCraft Launcher (sample data)`, `Example data, not installed`, and per-app Plug-ins sections for LightCraft, PdfCraft and PhotoCraft. The Plug-ins dropdown was selected manually and left visible. A later process ID or reboot will change this state. Do not interpret those example rows as actual installations.
- **Observed routing gap:** a cold `--open=extensions?kind=plugin` displayed a plug-in filter banner while Themes remained selected and empty. Selecting `Plug-ins 4` manually exposed the grouped examples. Task `ui-link` requires the deep link to select the matching visible section, with cold-start and running-instance tests. This is a real showcase finding, not an end-to-end plug-in pass.
- **Not done:** no real plug-in catalogue/install/host round-trip, no reproducible clean-checkout CI for the combined worktree, no candidate package, no source PR, no release. Third-party remains switched off. UI tests previously passed (233 in the combined checkout), but that is not verification of those integration gates.

**Operative scope:** a shared plug-in, theme and extension browser, native to ArtCraft Launcher and reachable from every Craft app. Jeremy's decisions (16/10/2026): internal (first-party) extensions first with the third-party structure built in behind heavy security gates and switched off (d3); the shared store uses the AI store's shared-folder approach (d5); the crates live in a standalone workspace that is a public repo on his GitHub (d7); build Phase 1 end to end into his test build. Only his words are Decided; `index.html` lists them.

## Handover state (04/11/2026, supersedes older locations below)

All work was committed as unreviewed work in progress and pushed to storytold. No PRs are open.

- `storytold/craft-launcher`: `test/ext-plus-ai` at `be3bff3e26` (throwaway, holds the approved UI and AI work, never the release source) and `feat/extensions` at `6ede77f93f` (373 commits behind main).
- `storytold/craft-extensions`: `wip/extensions-handover` at `183c506deb` and `chore/repo-url-storytold` at `4f11728727`. `main` is still `33adcb4`.
- `storytold/photocraft`: `feat/extensions-theme-adapter` at `ecb056cc74`. The `photocraft-wt-ext` worktree no longer exists; the branch is in `photocraft-src`.
- Local checkouts: `craft-extensions` now sits on `wip/extensions-handover`. A stray `NUL` file in `craft-launcher-wt-combine` was left out on purpose (it is `cargo metadata` output).
- `storytold/craft-launcher` is no longer read-only for Jeremy.
- Checked on 04/11/2026: format, core and store tests (116), launcher extensions tests (18 and 27), `cargo check` on the combined tree. Not run: clippy, wasm, the full suites, CI on these branches.

## Future roadmap (Jeremy, 04/11/2026)

- **Themes** connect later to a ThemeCraft app, once it exists (`theme-1`, `theme-2`).
- **Workflows tab** for custom ComfyUI API JSON files that certain app UIs can adopt. Future only (`wf-1`, `wf-2`).
- **Plug-ins** should support a vast range of ability across all apps (`plc-1`, `plc-2`, decision d12). The current rule of empty permissions in version 1 stays until `plc-1`.
- **Ranking** of all plug-ins by downloads and other signals (`rank-1`, `rank-2`, decision d11). Expected to be very complex.

## Read first

1. Load `craft-suite-shared-work`, `test-driven-development`, and `artcraft-ai` (the store-folder question is shared). Load `photocraft-codebase` before touching PhotoCraft.
2. Read this file, `index.html` (generated by `build.py` from `plan_data.py` and `plan_tasks.py`; run `python build.py` after changing data, never edit the HTML alone) and `design-notes.md`.
3. Verify against the live repos before acting: `git fetch`, `git status`, `git log` in `../craft-extensions`, `../craft-launcher-ext`, `../photocraft-src`. Jeremy may have shipped fixes outside Hermes.

## Where things are (16/10/2026)

| What | Where | State |
|---|---|---|
| Crates (public) | `../craft-extensions`, GitHub `storytold/craft-extensions` (moved from `Jeremy8776/craft-extensions` on 04/11/2026) | `main` pushed at `33adcb4` (16/10/2026, on Jeremy's yes). CI green on that commit: fmt, clippy, wasm check, tests on Linux and Windows |
| Threat model | `../craft-extensions/docs/THREAT-MODEL.md` | Draft, 20 threats, 13 gates, unreviewed |
| Mutation harness | `../craft-extensions/docs/mutation_check.py core` or `store` | Run alone; it edits sources. core 27/27 caught; store 31 caught, 3 equivalent with reasons |
| Launcher worktree | `../craft-launcher-ext`, branch `feat/extensions` from `origin/main` `307efcf` | **Extensions page `1f34346` and open-from-another-app `70b9612`, pushed to the fork `Jeremy8776/craft-launcher` (remote `fork`)**; `origin` is read-only for him. No PR yet. The old `../craft-launcher-src` is on `feat/ai-connections` with other work: never edit it, but its `target` dir is used for builds (see Build notes) |
| PhotoCraft | `../photocraft-wt-ext`, branch `feat/extensions-theme-adapter` from `upstream/main` `2a952f16` | **Adapter `ecb056cc`, pushed to `Jeremy8776/photocraft` (`origin`)**; no PR yet. `../photocraft-src` untouched |
| This handoff | `extensions-handoff/` | Not a git repo |

## What is built and verified

- `craft-ext-core` (wasm-safe, 63 tests): extension and app ids, package paths, display text rules, versions, the manifest (`craft-extension.json`), the theme palette with WCAG contrast and dark flag checks, trust tiers and `Policy`, the catalogue index (expiry, no older index, revocation list).
- `craft-ext-store` (desktop, 48 tests): store location (`CRAFT_EXT_DIR` override, else the local data folder; same folder in portable mode), bounded zip unpack, state file with generation counter, install, uninstall, enable per app, choose a theme, and the `Reader` apps use.
- Gates run: `cargo fmt --check`, per-crate `clippy --all-targets -D warnings`, tests with and without `--features third-party`, core wasm32 check, both mutation harnesses. Third-party stays off: `Policy::admit` refuses it unless the feature is on, and a test fails if the feature is a default.

## Not built, not verified

- **Neither screen nor the restyled PhotoCraft has been looked at in a window.** The test builds exist but nothing was launched.
- **The Extensions screen has never been looked at in a window.** Only headless tests (egui_kittest) and a real-folder test ran. Layout, the two menu buttons and the row alignment in the Installed list are unchecked by eye. Jeremy tests by hand.
- No catalogue, no network, no updates (phase 2). Plug-ins and Resources have no tab yet; an installed plug-in shows only in Installed.
- Launcher gates run: `cargo fmt --all --check`, clippy `-D warnings --all-targets` on `artcraft-launcher-ui-egui` and `artcraft-launcher`, all ui-egui tests (single-threaded, 58 lib + 7 new + the existing 51), the shell's 3 store tests, `cargo xtask wasm`. A manual mutation check over `plan.rs` (14 mutants, 2 survived at first and the tests were fixed; 0 survive now).
- Not run: macOS, a real network, heavy contention on the OS lock beyond the 8-thread test. Linux ran only through CI (tests passed there, the mutation harness did not run there).
- The rollback after a failed state save is written but no test makes the save fail.
- `unpack`: two size checks are backstops the zip reader makes unreachable (mutants documented as equivalent), and the existing-folder check only matters on Unix.
- Signatures (sg2), signed index metadata (sg3), revocation wired into the launcher (sg4), fuzzing (sg8), provenance (sg9), review process (sg10), outside review (sg12): not built. Gate sg13 stays Closed.

## PhotoCraft adapter (a-photo-1, built 16/10/2026, local commit ecb056cc)

- New `crates/ui-egui/src/ext_theme.rs` (+ `ext_theme_tests.rs`). `restyle(kind, &Active)` maps the palette onto `Tokens`: canvas and dock from the background, chrome and card from the surface, hover, pressed and selected rows shifted only as far as text stays at 4.5:1, accent text pulled toward the text colour until it reads, primary button label picked from background, text, white or black. The Pro or Studio layout of the person's current theme is kept; only colours change. Then PhotoCraft's own override names (`chrome`, `canvas`, `accent` and so on) apply, **the contrast pairs are re-checked**, overrides are dropped if they break it, and the theme is not worn at all if the palette alone cannot be made to read.
- `theme::apply` asks `ext_theme::tokens_for(ctx, kind)`, so any re-apply (menu pick, font change) keeps the theme. `Watcher` polls the store's generation at most once a second (plus a 2 s wake-up) and re-applies on change. It starts **off**: only `apps/photocraft/src/main.rs` calls `follow_extensions()`, so tests never read a real store.
- Gates: fmt, clippy `-D warnings` on `photocraft-ui-egui` and `photocraft` (all targets), `cargo xtask layers`, `cargo check --target wasm32-unknown-unknown -p photocraft-ui-egui`, 14 new tests, the 63 `theme` tests. Manual mutation check: 19 mutants caught; "generation ignored" is equivalent (an optimisation, the same result).
- **Known gaps:** (1) picking a built-in theme in PhotoCraft's own menu does nothing while a launcher theme is chosen; the person must untick it in the launcher (r3's panel should add "use PhotoCraft's own look"). (2) Status colours are held to 3:1, not the 4.5:1 PhotoCraft's own themes keep on resting surfaces, because that is all the shared validator guarantees. (3) The first frame shows the built-in look before the first poll. (4) Not run: the full ui-egui suite (about 3000 tests), GPU render tests, panic_hunt, test-corpus.
- Cargo: `craft-ext-core` and `craft-ext-store` are path dependencies in the workspace `Cargo.toml` (`../craft-extensions`); pin to a git revision before any PR.

## Test builds (t1)

- Launcher with the Extensions page **and the AI connections work**: `../craft-launcher-wt-combine/target/dev/artcraft-launcher-ext-and-ai.exe` (throwaway branch `test/ext-plus-ai` = `feat/extensions` + `feat/ai-connections` + the AI tree's uncommitted features/notes/i18n changes, copied not edited; the AI tree's own files untouched; catalogs for es, cs, pl, hu, ja show English for the AI strings; never for a PR). To test open-from-app: `artcraft-launcher-ext-and-ai.exe --open "extensions?app=photocraft&kind=theme"`. The installed release launcher (`%LOCALAPPDATA%\Programs\ArtCraft Launcher`) has no such page, and a second start hands over to a running launcher, so quit the installed one first.
- PhotoCraft: `PhotoCraft-dev/photocraft.exe` = `upstream/main` + `feat/ai-port` + `feat/extensions-theme-adapter` (`dev-additions.txt` now lists the adapter). **Four older additions were skipped for conflicts** (`fix/color-field-fills-panel`, `feat/swap-arrow-bold`, `fix/toolbar-chips-follow-tools`, `feat/sam3-object-selection`): they lag `upstream/main` by 700 to 1300 commits and need their own rebase. The previous exe is in `PhotoCraft-dev/backup/`.
- To see it work: in the launcher open Extensions, Add from file (`extensions-handoff/samples/`), Themes tab, Use in... PhotoCraft; a running PhotoCraft restyles within about two seconds. Both use the real store unless `CRAFT_EXT_DIR` is set (set it the same for both).

## Open from another app (r1, r2, built 16/10/2026, 70b9612)

- `crates/native/src/deeplink.rs`: `Target::parse("extensions?app=photocraft&kind=theme")`. Only the page `extensions` is accepted; an app must look like an app id, a kind must be `theme`, `plugin` or `resource`; anything else is dropped (page still opens unnarrowed). `from_args` reads `--open <text>` and `--open=<text>`.
- `single_instance.rs`: after the hello (which already shows the window, so old launchers still do), a start with a place sends `open <place>` on a second connection. A newer start that takes over, or a first start, carries the place itself. `Request` is no longer `Copy`.
- UI: `LauncherUi::open_extensions(app, kind)` picks the tab (themes, else Installed), narrows the lists, and shows "Showing only works with PhotoCraft, themes" with a Show all button. Plug-ins and resources have no Themes view, so they land on Installed.
- Not run: a real second process hitting a real running launcher (covered by loopback tests only).

## Built-in themes, demo data, tab layout (16/10/2026, uncommitted in craft-launcher-ext)

- Themes tab now ends with "Built into your apps": 14 PhotoCraft, 4 LightCraft and the launcher's own 2, drawn with the same preview cards, display only ("Choose it inside PhotoCraft"). `features/extensions/builtin.rs` is a snapshot of colours read from each app's theme code (PhotoCraft via a throwaway test, the others from `theme.rs`); update it when an app changes a built-in theme. Other suite apps have no theme list yet.
- `--demo-data` (desktop launcher): `SampleSource::demo()` for Extensions (8 themes, 4 plug-ins, 4 resources, some switched off, 3 in use) and, in the combined test build only, `history_demo.rs` (40 made-up AI jobs). The window title says "(sample data)"; nothing real is read or written. Start menu: "ArtCraft Launcher TEST BUILD (sample data)".
- Options that fit are never stacked: page switch and tabs sit in a horizontal row (test fails if they stack).
- Not committed or pushed. `history_demo.rs` exists only in `craft-launcher-wt-combine`.

## Launcher screen (built; 1f34346 and 70b9612)

- New module tree `crates/ui-egui/src/features/extensions/`: `source.rs` (the `ExtensionSource` trait and `Row`, wasm-safe), `plan.rs` (what a click means, pure, tested), `screen.rs` (drawing, collects actions then runs them), `card.rs` (theme preview painted from the palette), `sample.rs` (in-memory store for tests), `mod.rs` (state). `views.rs` untouched.
- `app.rs` gained about 16 lines (it was already over the limit at 734): `Host::pick_file`, `Host::extensions`, the `extensions` state field, and a `page_switch` row (Apps | Extensions) at the top of the central panel. The switch only draws where the host has a store, so the web build is unchanged.
- Desktop side: `apps/artcraft-launcher/src/extensions.rs` implements the trait over `craft-ext-store`; `main.rs` holds it in `DesktopHost`. A theme card has a "Use in" menu (per app) and "All apps"; choosing switches the theme on for that app first when needed. Installed tab lists everything with "Switched on in" and a two-step Remove.
- The page polls the store's generation once a second, so an install in another app appears without a restart.
- Strings: 34 new, in all ten non-English catalogs, written by me; ask for a native read before release. `i18n/tests.rs` now includes the three new source files in its coverage scan.
- Sample packages for hand testing: `extensions-handoff/samples/*.zip` (Midnight, Paper, Ocean; ids `org.example.*`, since a file may not use first-party ids). Regenerate with `CRAFT_EXT_SAMPLES_OUT=<folder> cargo test -p artcraft-launcher --bin artcraft-launcher -- --ignored write_sample_packages`.
- Cargo: workspace `craft-ext-core` and `craft-ext-store` are **path dependencies** on `../craft-extensions`; replace with a pinned git revision before any PR. `Cargo.lock` grew by about 130 lines (zip, sha2, craft-atomic and friends); the shell has new dev-dependencies for its tests.

## Build notes

- Builds use `CARGO_TARGET_DIR=../craft-launcher-src/target` so the cached dependencies are reused (a test compile takes seconds instead of a cold build). That dir is shared with the AI branch: building the `artcraft-launcher` binary overwrites its `target/debug/artcraft-launcher.exe`. This session backed that exe up, built, copied the result to `craft-launcher-ext/target/dev/`, and restored the AI exe byte for byte (its modified time changed). Do the same, or use a separate target dir. `cargo xtask wasm` also rewrites `target/debug/xtask.exe`.
- The dev build runs against the real store at `C:\Users\<name>\AppData\Local\ArtCraft\extensions`. Set `CRAFT_EXT_DIR` to a scratch folder to keep test installs apart.

## Design choices worth knowing

- The tier is assigned by the host from where a package came; a package has no tier field and cannot claim one. A file may not use first-party id prefixes and may not replace a first-party extension.
- Installs enable the extension for every app in its `compat`; installing never changes anyone's look. A theme is worn only after the person (or app) calls `set_active_theme`.
- An app reads with `Reader::open(root)` and polls `generation()`; bad or missing data counts as absent, so an app keeps its own look.
- Per-app theme overrides use names only the app knows. The shared validator cannot judge them, so **each app must re-check its own contrast pairs after applying overrides** (put this in the adapter checklist).
- The launcher consumes these crates by path while building locally. A path dependency must be replaced with a pinned git revision of the public repo before any launcher or app PR.

## Next steps, in order

1. **Done 16/10/2026: l1 to l3** (see Launcher screen above; pushed to the fork).
2. **Done 16/10/2026: a-photo-1** (see above).
3. **Done 16/10/2026: t1** (see Test builds).
4. **Done 16/10/2026: r1 and r2** (launcher `--open`, see Open from another app).
5. **Open, needs one answer first: r3** (PhotoCraft Themes & Extensions panel, `extension.*` commands, Browse more button; also closes known gap 1). Where the panel lives (a Window menu dialog, a docked panel or a Preferences page) is not decided, and every new command needs the engine shell-command table, parity checklist and a label in PhotoCraft's own catalogs. Ask Jeremy which before building.
6. Update `plan_tasks.py` statuses, run `python build.py`, update this file and the todo list in the same turn.

## Rules that apply

No push, PR, issue comment or message without his specific yes. No em or en dashes. Files under 500 lines (700 hard). Never crash: no unwrap, expect or panic in non-test code. Test keys only; no real key material anywhere. Say plainly what was not run.

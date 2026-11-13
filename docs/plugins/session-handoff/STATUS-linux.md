# Current handoff status (2026-11-13)

Recovered source is committed for Mac continuation; incomplete verification is recorded in each app’s docs/plugin-takeover-validation.md. See craft-extensions/docs/plugins/session-handoff/MAC-CONTINUATION.md and transfer-revisions.json. No workspace Cargo/rustc processes remained when the tool connection recovered. No new compilation or suites ran during handoff.

The material below is historical. Current instruction: source and documentation completion only; no cargo build/run, app or guest release builds, broad tests, CI or releases. Only genuinely cheap scoped checks and small tests are allowed. Runtime/platform gates remain separate and unverified unless recorded otherwise.

# Craft plugin beta: status

Updated 2026-11-13, about 06:00 IST, at the end of the overnight autonomous run. Every repo is committed and pushed to both `github` and `forgejo` (branch `plugin-beta`; craft-plugins uses `main`).

- **Design** (authoritative): [beta-design.md](craft-extensions/docs/plugins/beta-design.md)
- **Code map, workflows and the requirement checklist:** [beta-implementation.md](craft-extensions/docs/plugins/beta-implementation.md)
- **Decisions and what they supersede:** [draft/decisions.md → Beta decisions B-01…B-17](craft-extensions/docs/plugins/draft/decisions.md)
- **Guides:** [author](craft-extensions/docs/plugins/author-guide.md), [user](craft-extensions/docs/plugins/user-guide.md), [adapter](craft-extensions/docs/plugins/adapter-guide.md); per-app contract pages in `craft-extensions/docs/plugins/contracts/`

## Takeover in progress (2026-11-13)

The earlier status below predates the UI 1.1 and app capability work. It is historical, not a current release-readiness claim.

- Shared plugin renderer and EffectCraft/FigCraft capability expansions are committed; their interrupted review fixes are being validated.
- EffectCraft adapter regression suite: **22 passed** on Linux during takeover, including idle tools, revocation rollback, pause/resume and document path redaction. The viewer input regression passed; current desktop builds are deferred.
- FigCraft: recovered wake-up/event-loop/comment fixes plus narrowed large-ID validation are under targeted validation.
- PhotoCraft: interrupted renderer/tool/operation work is being completed. Review found file-effect rollback, no-follow writes, save-name validation and allocation gaps; these are being repaired before readiness is claimed.
- Cross-app examples and smoke runner were recovered uncommitted. FigCraft does **not** implement raster.filter; its effect integration remains unsupported.
- Initial PhotoCraft example test compiled but **1 test was ignored** because the package fixture environment was absent; this is not an end-to-end pass.
- All new app builds, full gates, refreshed signed catalogue and desktop click-through checks are deferred by the user; do not start them in the source-completion phase. CI and releases require the user's final approval and remain disabled.
- Codex Sol medium owns PhotoCraft completion; Luna owns only the recovered cross-app examples. DeepSeek 4.1 Flash Free was requested but unavailable from the connected provider. No fallback external model was selected.

## Ready to test (2026-11-13)
Start with [TESTING.md](TESTING.md). Debug binaries are built for all three apps, their CLIs, the launcher and `craft-ext`, and the local signed test catalogue is at `test-catalogue/`. `craft-extensions/scripts/e2e.sh` passes. The maker → store → user flows ran for real against all three apps:
- **EffectCraft:** effect, session, data, projects, kill list.
- **PhotoCraft:** filter, colour modes, LUTs.
- **FigCraft:** panel, command, tool, overlay.

The GUIs were built but not launched, so clicking through is for you to do.

## Done

### craft-extensions (shared platform)
- **ext-core:** the manifest (`craft-plugin.toml`, schema 2) with contracts and permissions declared per contribution; contract compatibility (compatible, partly, or incompatible, with reasons); the permission vocabulary; catalogue records, search and version resolution; the OSI licence check; project references; the UI tree.
- **ext-store:**
  - store state in `store-v2.json`, kept apart from the old format;
  - per-app setup, grants, pending updates and pins;
  - GrantView, which fails closed when the store can't be read;
  - the signed catalogue client;
  - the Manager facade that the app managers, the launcher and the CLI all use.
- **ext-host:**
  - the wasmi sandbox, guest ABI v1;
  - permissions checked on every call, limited to what the calling contribution declared, so revocation works;
  - the native out-of-process helper, with consent checked before it starts and supervision afterwards;
  - budgets, and re-instantiation after a failure.
- **ext-ui:** the egui manager (Browse, Installed, Updates, Problems), the setup sheet, launcher mode, and the project prompt.
- **ext-cli:** the `plugin …` command every app embeds, plus `craft-ext` for authors (new, build, check, test, pack, dev) and maintainers (catalogue check, build, sign, keygen).
- **SDK and examples:** `craft-guest` with the `filter!` and `session!` macros; examples halftone (a wasm effect for three apps), film-looks (a LUT plus presets) and color-notes (an EffectCraft panel plus command); templates.
- **Catalogue signing:** the real beta key is in `TRUSTED_KEYS`. The secret is at `~/.config/craft/catalogue.key` (mode 0600), never committed.

### Apps
- **EffectCraft:**
  - old plugin crate, helper-host and Plug-in Library removed;
  - effects are added and removed at runtime (`effects/external.rs`), and the `effectcraft-plugins` adapter handles effects, sessions (commands, panels, windows), the project plugin list, the open prompt (Open without and Skip are honoured), export refusal and safe mode;
  - the Plugins window and `effectcraft-cli plugin` (including `contract` and `project`) are in place.
- **PhotoCraft:**
  - old filter ABI removed;
  - plugin filters appear in Filter ▸ Plugins with dialogs, presets, CMS-honest colour and one history step; masks, channel subsets and non-RGB modes are refused with a reason;
  - sessions work (commands, panels, windows, events), `filter.apply` is limited to an allowlist, and plugin LUTs show in Color Lookup;
  - the Plugins window and the CLI are in place.
- **FigCraft:**
  - old system removed; saved plugin data is preserved and retired wire tags fail with a clear error;
  - sessions, commands, panels, windows and **tools** work (Escape rolls back), as do canvas overlays and the plugin references in collaboration properties;
  - the project prompt, the Plugins window, the CLI and safe mode are in place.
- **craft-launcher:** a header Plugins button opens the shared manager for every app. The launcher is never required.

### craft-plugins (catalogue repo)
- Rules, tiers, vendor registration, submission and maintainer guides, the report form and the kill list.
- `dist/` holds signed serial 1, which lists film-looks only. Halftone and color-notes wait in `examples/pending/` until their wasm is built and released.

### Form schema v1 (2026-11-13, after the aiongg sync)
- `craft-ext-core::form`: Form/Param/ParamKind/Word/Cond/Values/Patch, named after craft-framework's `Param`/`ParamKind` (aiongg's branch), serde-only and wasm-safe. It is mirrored here until a shared storytold crate exists.
- Contributions declare a `form`. Panels gain `Form` and named `Component` nodes (with a fallback until the component library lands). There are optional `craft_describe` and `craft_update` exports. The SDK has the `Form::new(...)` builder plus `t!`, `Params` and `Patch`. `craft-ext pack --describe` saves the built form into the package.
- All three apps render forms (effect controls, filter dialogs, panels), evaluate conditions themselves and apply patches. film-looks 1.1.0 is published in signed catalogue serial 2.
- Next: once aiongg's renderer and component registry land on storytold, swap the egui form and component renderers for his `render_form` and components, and move the schema into a shared crate.

### Reviews
- sol-high reviewed the design, the shared core (phase 1), EffectCraft, PhotoCraft and FigCraft (one phase review each), and the whole suite against the brief. All findings are fixed.

## Stubbed or remaining (named TODOs live in each app's `docs/plugin-migration.md`)
- **FigCraft:** plugin panels float instead of docking (PLUGIN-PANEL-DOCK); zoom and pan don't work while a plugin tool is selected (PLUGIN-CANVAS-NAV); the 3-failure pause has no resume (PLUGIN-FAILURE-RESET).
- **PhotoCraft:** the examples haven't been checked end to end (PhotoCraft-plugin-examples).
- **Tools** work in FigCraft only. EffectCraft and PhotoCraft mark tools unavailable with a reason.
- **Fonts and themes** are stored but not activated; this waits for the craft-theme framework, which will be the only theme format.
- **Deferred by design (beta-design B12):** TUF; leases across processes; OS confinement for native plugins; the account permission; native effects; temporal frame access and tiling; plugin object types; workflows; cross-plugin calls; web builds.

## Verification limits
CLI and headless flows ran end to end against the local test catalogue. Desktop GUIs were built but not launched, and there is no hostile-module conformance suite yet. Real GitHub downloads haven't been tested, because no releases exist: the `file://` test catalogue stands in for them.

## Next steps
1. Build and run each app once and walk the workflows in beta-implementation.md: install from the catalogue, the setup sheet, an effect or filter, a panel, revocation mid-run, the project prompt, and export refusal.
2. Build the halftone and color-notes examples, publish GitHub releases, move them from `examples/pending` into `plugins/`, then rebuild and re-sign the catalogue.
3. Fix the FigCraft TODOs. Add tool support to EffectCraft and PhotoCraft if wanted.
4. Before upstreaming: switch the app path dependencies (`../craft-extensions/crates/*`) to git dependencies pinned to a craft-extensions revision; move the catalogue to an org-owned repo (`DEFAULT_URL`); restore CI.
5. Add the hostile-module conformance suite and run it on every desktop target.

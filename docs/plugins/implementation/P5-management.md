# P5 and P5b: Manager component, `plugin` CLI, and L0 in every app

Status: **build-ready plan, 2026-11-10.** Units P5 and P5b of [the architecture](../architecture.md#14-management-in-every-app-and-the-launcher); deepens [implementation plan section 6, P5 and P5b](../implementation-plan.md#p5-and-p5b-management). Each PR still needs its own approval.

**Code read (nothing built or run):**
- craft-libs `fd5e2fe`: `crates/keymap`, `crates/menu`, `crates/ui`, `crates/i18n`, `crates/settings` and `crates/update`;
- the launcher `origin/feat/extensions` branch, listed only;
- craft-extensions `a6c5fae`.

## 1. Goal and exit criteria

PC01–PC03 reconciliation, 2026-11-11: the first usable app is EffectCraft, proving one L0 data package alongside one L1 transform. Manual installation imports and validates a package, then requires explicit enablement; dropping a package into the app uses that flow. Package bytes remain shared, with setup and grants separate per app. The broader L0 rollout and catalogue/launcher delivery boundaries below still need reconciliation. These choices do not mark all L0 data kinds complete or authorize implementation. See the [post-call answers](../design-sessions/post-call-reconciliation.md#pc01-answer-2026-11-11).

PC09 reconciliation, 2026-11-11: the user verified egui as the framework intended in the call notes. Use existing egui components for the manual MVP without waiting for the separate theme framework. The manager must adopt shared theme tokens when their contract is ready. Token schema, ownership, and timing remain open; this decision adds no new dependency or implementation authorization.

**Goal.**
- One set of management actions, used by the GUI and the CLI alike.
- One egui manager component, shown in every app (that app only) and in the launcher (all apps).
- One `plugin` command library that each app's CLI embeds.
- In P5b, the L0 level in every app: manager, store, CLI and data add-ons. The launcher is never required.

**Exit criteria (architecture section 22: "full lifecycle driven headless"; P5b: "manager and a theme or preset install work in each app").**

| # | Criterion | Proving test |
|---|---|---|
| X1 | The full lifecycle runs headless through the shared actions: install (file) → set up → grant → update (pending on wider rights) → accept → revoke → disable → choose older version → uninstall | `manage::full_lifecycle_headless` |
| X2 | The GUI and the CLI produce identical state for identical choices | `manage::gui_and_cli_setup_identical` (the manager driven through `egui_kittest`, against the CLI library, on the same temporary store) |
| X3 | Every screen is keyboard-reachable and exposes names and states to AccessKit; labels never rely on colour alone | `manager_ui::a11y_tree_snapshot` per screen, plus a manual screen-reader pass recorded in the PR |
| X4 | Setup export and import never restore grants or folder tokens silently | `setup_io::import_asks_permissions_again` |
| X5 | Safe mode runs no plugin code and no plugin theme; the manager still works | `safe_mode::no_plugin_code_runs` (with P3's instance counter) |
| X6 (P5b) | In each app, the manager opens, and a theme or preset installs from a file and applies | per-app `l0::theme_or_preset_round_trip`, through the app's headless entry |

## 2. Code areas (verified)

**craft-libs `fd5e2fe`**

| Concern | Where | What it means for P5 |
|---|---|---|
| egui | egui 0.37, patched to `storytold/egui` (workspace `Cargo.toml`) | The manager crate must build against the same egui. An app's `[patch]` applies to the manager too, so depend on plain `egui = "0.37"` and on `craft-ui` |
| Panels | `crates/ui/src/docking/workspace.rs:198` `Workspace::reconcile(registered, permissions, default)`; `surface.rs:837` `trait DockContent<P> { fn body…; fn logical_label… }` | Plugin panels that appear and disappear fit `reconcile`; `logical_label` is the accessible name |
| AccessKit | No shared helpers; widgets call `ctx.accesskit_node_builder` directly (`crates/ui/src/combo.rs:599`) | P5-03 adds small helpers in `craft-ui` (a craft-libs PR) rather than inside the manager |
| i18n | `crates/i18n/src/lib.rs:63` `I18n::new(embedded: &'static Embedded, offered)`. Catalogues are `'static` and embedded at build time; there is **no runtime catalogue** | Plugin and translation-pack strings need a runtime catalogue API: a craft-libs PR (CL-3) before L0 translations ship |
| Keymap | `crates/keymap/src/keymap.rs:500` `Keymap::new(catalog, scopes, platform)` (catalogue fixed at build); there are only two binding layers, defaults and the user's `Overrides`; `assign(cmd, binding, Policy::Refuse)` (`:638`) and `is_reserved` (`:593`) exist; `Problem::UnknownCommand` (`:415`) keeps overrides for missing commands | "Suggested only if free" needs a third, lowest-priority layer (CL-1). Today a plugin command means rebuilding the `Keymap` |
| Menus | `crates/menu/src/model.rs:10` `MenuBar`, `:104` `Node`, `:423-435` `from_items(tops, rows)`; no extension points, groups or anchors; `driver.rs:349` `sync_with(state, build)` rebuilds on change | "Appended after built-ins, grouped by plugin, in declared extension points" needs named anchors (CL-2) |
| Settings | `crates/settings/src/lib.rs:71` `trait Settings`; unknown keys kept (`toml_edit`) | Per-plugin host settings survive while a plugin is absent |
| Download | `crates/update/src/http.rs:263` `Http::download(url, dest, size, sha256, stop, progress)`: a SHA-256 is required, Range resume, pause and cancel, `https_only` on the agents (`:73-116`) | Large-resource downloads (P5-07) and the catalogue client (P7) reuse this. Nothing new is extracted from the launcher |

**Launcher.** `origin/main` `753a710` has no extensions code. The unmerged `origin/feat/extensions` (`6ede77f`) has `apps/artcraft-launcher/src/extensions.rs` and `crates/ui-egui/src/features/extensions/*` over `craft-ext-store`, through path dependencies. That branch is a source of UI lessons, not a base: its store semantics predate C1–C9.

## 3. Layout and API sketches

```text
craft-extensions
  crates/ext-manage/       actions and results shared by GUI and CLI (no egui)            FROZEN action meanings
  crates/ext-manager-ui/   the egui component (Browse, Detail, Install and setup, Installed, Updates, Problems)
  crates/ext-app-cli/      the `plugin` command library apps embed (clap derive), human and --json output
  crates/ext-store-web/    browser-local store reusing ext-store/src/model.rs (P1)
craft-libs (separate PRs, under craft-libs AGENTS.md)
  CL-1 craft-keymap: a "suggested" binding layer below defaults; suggestion used only when free
  CL-2 craft-menu:   named extension points (anchors) and plugin groups appended after built-ins
  CL-3 craft-i18n:   a runtime catalogue layer (owned strings) that can never override reserved keys
  CL-4 craft-ui:     AccessKit helpers (role, name, state, value) used by manager screens
```

```rust
// ext-manage
pub enum Action {
    InstallFile { path: PathBuf }, InstallListing { key: ReleaseKey },          // listing install needs P7
    SetUp { app: AppId, key: ReleaseKey, contributions: BTreeSet<ContributionId>, grant: PermissionSet },
    Grant { app: AppId, id: ExtensionId, add: PermissionSet }, Revoke { app: AppId, id: ExtensionId, remove: PermissionSet },
    Accept { app: AppId, id: ExtensionId },  Disable { app: AppId, id: ExtensionId },
    ChooseVersion { id: ExtensionId, key: ReleaseKey }, Remove { id: ExtensionId },
    ExportSetup { apps: Vec<AppId> }, ImportSetup { file: SetupFile },
}
pub enum Outcome { Done(Report), NeedsConsent(ConsentSheet), NeedsDependents(Vec<ExtensionId>), Refused(ManageError) }
pub struct ConsentSheet { pub app: AppId, pub sentences: Vec<HostSentence>, pub dangers: Vec<DangerCombination>, pub author_why: Vec<(Permission, String)> }
pub fn plan(action: &Action, ctx: &ManageContext) -> Outcome;           // pure: what would happen, the sheet to show
pub fn apply(action: Action, consent: Option<ConsentToken>, store: &Store) -> Result<Report, ManageError>;

// ext-app-cli
#[derive(clap::Subcommand)] pub enum PluginCommand { Search, Info, Versions, Install, Setup, Grant, Revoke, Update, Remove, Permissions, ExportSetup, ImportSetup, Test }
pub fn run(cmd: PluginCommand, app: &AppId, out: OutputMode) -> ExitCode;   // exit codes from ext-ops (0..5)
```

- **One planner.** `plan` produces the consent sheet for both the GUI and the CLI. The CLI prints the same host-written sentences and asks for `--yes` only on `plugin grant` (never on execution commands).
- **Danger combinations** (architecture section 8) are computed in one place. For example, `document.read` + `network.connect:api.example.com` gives "could send your document to api.example.com".

## 4. Ordered PR list

| PR | Scope | Size | Test that fails first | Depends |
|---|---|---|---|---|
| P5-01 | `ext-manage`: actions, `plan`/`apply`, consent sheets and danger combinations | M | `manage::full_lifecycle_headless`; `manage::danger_combination_spelled_out` | P1-12, P1-13 |
| P5-02 | `ext-app-cli`: the `plugin` command library; human and `--json` output; exit codes; `plugin grant` separate from execution | M | `cli::grant_is_separate_never_a_flag` (no execution command accepts permission flags); `cli::missing_plugin_exit_4_prints_install` | P5-01, P2-07 |
| P5-03 | Manager UI: Installed and setup sheet screens first, on `craft-ui` with CL-4 helpers | M | `manager_ui::a11y_tree_snapshot(installed)`; `manage::gui_and_cli_setup_identical` | P5-01, CL-4 |
| P5-04 | Manager UI: Problems screen, safe mode, "Find the problem plugin" bisect with a persisted bisect state that survives a second crash | M | `bisect::second_crash_resumes_not_restarts` | P5-03 |
| P5-05 | Setup export and import (`plugin-setup.json`); grants and tokens never restored | M | `setup_io::import_asks_permissions_again` | P5-01 |
| P5-06 | Browser-local store: `ext-store-web` over IndexedDB or OPFS, reusing `model.rs`; file installs only until the P7 web download question is settled | M | `web_store::same_transitions_as_desktop` (the P1 model property test run in a headless browser) | P1-11 |
| P5-07 | Data add-ons: shared importers (themes exist; presets, LUT, ICC, gradients, swatches), one PR per importer, each with a fuzz target | M each | `importer::<kind>::hostile_inputs_refused` | P1-10 |
| P5-08 | Large resources: setup offers "Download now" or "When first needed"; resumable, digest-checked, content-addressed `resources/sha256/<digest>` shared between packages, removed when unreferenced; via `craft_update::http::Http::download` | M | `resources::shared_blob_removed_only_when_unreferenced` | P1-11, P7-02 (shares the downloader) |
| P5-09 | Diagnostics: size-capped rotated log with plugin lines tagged and redacted; Plugin console; "Create support report" preview without tokens or content | M | `diag::report_redacts_paths_urls_tokens` | P3-05 |
| P5-10 | Browse, Detail and Updates screens against a **fixture catalogue** (real fetch arrives in P7) | M | `ranking::official_first_within_tier_only` (a fuzzy official match never outranks an exact non-official one) | P5-03 |
| CL-1..CL-4 | craft-libs PRs listed in section 3 | S/M each | CL-1 `suggested_binding_used_only_when_free`; CL-2 `plugin_items_after_builtins_grouped`; CL-3 `runtime_catalogue_cannot_override_consent_strings`; CL-4 `helpers_set_role_name_state` | — (start any time) |

**P5b, L0 in every app** (one small set of PRs per app, built by that app's maintainers from the generated skeleton):

| Step | PR in the app | Test |
|---|---|---|
| L0-1 | Adapter crate skeleton (`<app>-plugins`) from the generator; the L0 checklist passes | `adapter::l0_checklist` (from P3-07) |
| L0-2 | Embed the manager (Settings ▸ Extensions and a menu item) | `l0::manager_opens_headless` |
| L0-3 | `plugin` subcommand in the app's CLI | `l0::cli_plugin_list_json` |
| L0-4 | Data add-ons the app supports (theme and preset at minimum) | `l0::theme_or_preset_round_trip` |

## 5. Data formats

`plugin-setup.json`, exported by the manager and CLI:

```json
{ "format": "craft-plugin-setup", "version": 1, "exported_at": "2026-11-10T12:00:00Z",
  "packages": [ { "id": "io.github.ink.grain", "release": "sha256:9f2c…", "version": "1.2.0", "source": "listing" } ],
  "apps": { "effectcraft": { "io.github.ink.grain": { "contributions": ["halftone"], "settings": { } } } } }
```

There is no `grant` field. Import installs the packages and then shows each app's consent sheet again.

## 6. Test strategy

- **Headless UI tests** use `egui_kittest`. `craft-ui` already has per-language harnesses behind `screen-testing` (`crates/ui/src/testing/mod.rs:95-183`). The tests cover keyboard traversal and AccessKit tree snapshots, and run in pseudo-locale to catch text that is not translated.
- **CLI tests** parse the `--json` output and check exit codes against the P2 golden table.
- **Property tests** reuse P1's model against `ext-store-web` in a browser.
- **Manual.** A screen-reader pass (VoiceOver, NVDA, Orca) is recorded in the PR that adds each screen. This is reconciled plan risk 11.

## 7. Risks and decisions

| # | Risk | Experiment | Before |
|---|---|---|---|
| R1 | egui accessibility gaps for tables and sheets | Prototype the Installed table with AccessKit and inspect it with Accessibility Inspector and NVDA | P5-03 |
| R2 | Flatpak cannot share the store folder (reconciled plan risk 12) | Two sandboxed apps and the launcher with only `--filesystem` for the folder | P5b packaging |
| R3 | Browser downloads of GitHub assets fail cross-origin (open item) | A real browser `fetch` of a release asset through its redirect | Web catalogue installs (P7) |

**Decisions for maintainers:**
- **D1 (carried from the reconciled plan): L0 timing in pilot apps.** Recommended: in EffectCraft, FigCraft and PhotoCraft, L0 lands inside each app's atomic cutover (P6 E8b, P8, P9), so their working plugin systems are not removed early. In apps without an executable plugin system, L0 ships early.
- **D2. Where the craft-libs prerequisites live.** CL-1..CL-4 are general app infrastructure, so craft-libs is recommended over craft-extensions.

## 8. Effort and parallel work

- **Counts:** about 10 P5 PRs (all M), 4 craft-libs PRs (S/M), and 4 small PRs per L0 app.
- **Parallel:**
  - P5-01 and P5-02 can run beside P4.
  - P5-03 onwards, and CL-1..CL-4, can run beside P6 E2–E7.
  - P5b in non-pilot apps follows P5-03 and runs beside P6–P9.

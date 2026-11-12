# P6: EffectCraft pilot

Status: **build-ready plan, 2026-11-10.** Unit P6 of [the architecture](../architecture.md#22-delivery-plan); deepens [implementation plan section 6, P6](../implementation-plan.md#p6-effectcraft). Each PR still needs its own approval. EffectCraft's plugin areas stay on the [merge hold](../merge-hold-list.md) until this unit lands.

**Code read.** EffectCraft **`origin/main` `fb62b9d7`** (2026-11-06), read with `git show` and `git grep`; no branch was switched. The local checkout is on `ci/windows-leg` `f4c683b5`, and local `main` `ce9eaec6` is 22 commits behind `origin/main`. Line numbers below are at `fb62b9d7`. Nothing was built or run.

## 1. Goal and exit criteria

PC01 reconciliation, 2026-11-11: the user selected EffectCraft as the first usable MVP, with one manually installed L0 data package and one L1 transform, including save/reopen and failure handling. Exact examples remain open. The full P6 requirements below remain the baseline; old-system cutover, dispatcher dependencies, CLI/MCP parity, and the first slice's exclusions need reconciliation before changing the PR sequence. PC01 does not authorize implementation or waive those requirements. See the [post-call answer](../design-sessions/post-call-reconciliation.md#pc01-answer-2026-11-11).

**Goal.**
- Every caller of EffectCraft's engine goes through one invocation path with real principals.
- Edits are dispatcher-owned transactions.
- Attach is authenticated and local-only.
- Effects can be unloaded.
- Plugin features survive in projects as placeholders, and a final export refuses to run without them.
- Then, in one atomic cutover, the old plugin crate, Plug-in Library and helpers are replaced by the `effectcraft-plugins` adapter, the shared manager and `plugin` CLI commands.

**Exit criteria (architecture section 22).**

| # | Criterion | Proving test |
|---|---|---|
| X1 | **GUI, fresh CLI and MCP parity**: the same configured package is eligible and gives the same frame digest in all three | `parity::same_plugin_same_output` (section 6.2) |
| X2 | **Export refuses a missing plugin**: preflight lists it with the install command; a failure mid-export leaves no finished-looking file | `export::refuses_missing_plugin_before_start`, `export::trap_mid_export_leaves_no_file` |
| X3 | **No open TCP port**: attach works only through a user-only socket or pipe with a token | `attach::no_tcp_listener_by_default`, `attach::wrong_token_refused` |
| X4 | No caller-less production entry point compiles | `compile_fail` doctest calling `Session::execute(id, params)` |
| X5 | Two nested edits make one undo step labelled with origin; a failure in step two restores project and history and runs no queued effect | `txn::nested_edits_one_step`, `txn::step_two_failure_restores_everything` |
| X6 | An effect can be unregistered; its metadata is freed; an export holding the old registration finishes with it | `registry::unregister_frees`, `registry::export_keeps_old_registration` |
| X7 | A removed-system or missing plugin effect keeps all its data through open and save | `project::missing_plugin_effect_round_trips` |
| X8 | The old plugin runtime, Plug-in Library, helper host and `library.*`/`effect.plugins.*` commands are gone | deletion checklist (section 4, E8b) and `commands::no_legacy_plugin_ids` |

## 2. Code areas (verified at `fb62b9d7`)

### 2.1 Dispatcher and call sites (recounted)

- **Definitions** (`crates/engine/src/session/dispatch.rs`):
  - `execute(&mut self, id: &str, params: Value) -> Result<Value>` (`:9`);
  - `execute_checked` (`:35`): `commands::check_params`, then `execute`;
  - `execute_for_script` (`:45`): bumps `script_calls`;
  - `running_for_script` (`:53`).

  `execute`:
  1. finds the spec;
  2. checks `enabled`, skipping it when the parameters name an explicit target (`:11-20`);
  3. runs `guard::guarded`, which uses `catch_unwind` (`system/guard.rs:11`);
  4. calls `learn_observe`;
  5. pushes to `journal`, capped at 10,000 with 1,000 drained (`:24-28`).

**Recounted call sites.** Non-test calls to `.execute(`, `.execute_checked(` and `.execute_for_script(`:

| Area | `fb62b9d7` | (`f4c683b5`, same method) |
|---|---|---|
| `crates/ui-egui` | 190 | 189 |
| `crates/engine` (nested inside commands) | 98 | 96 |
| `apps/effectcraft-cli` | 10 | 10 |
| `apps/effectcraft` (desktop) | 9 | 9 |
| `apps/effectcraft-web` | 8 | 8 |
| `crates/automation` | 1 | 1 |
| `crates/script` | 1 | 1 |
| **Total** | **317** (310 `execute`, 5 `execute_checked`, 2 `execute_for_script`) | **314** |

- **The 691 figure in the reconciled plan is wrong.** Excluding only `tests/` folders and `tests.rs` files still leaves 729 matching lines at `f4c683b5` and 741 at `fb62b9d7`. Most of the difference is inline `#[cfg(test)] mod … { }` blocks. For example, `cmd/scripts.rs` has 52 matches and all of them sit after its `#[cfg(test)]` at `:605`.
- **Method.** A script reads every `.rs` file through `git show`. It removes:
  - `tests/`, `benches/` and `*tests.rs` files, and files declared as `#[cfg(test)] mod x;`;
  - inline `#[cfg(test)] mod x { … }` blocks, by brace matching;
  - `//` comments.

  It then counts `\.(execute|execute_checked|execute_for_script)\s*\(`. A manual check of `cmd/app_more.rs` matches the script's 26.
- **Wrappers.** Most UI calls go through wrappers that themselves call `execute` once, so the UI migration is cheaper than 190 edits suggest:
  - `menus/invoke.rs:186` `invoke` (90 callers) and its `run_engine` (24 callers);
  - `panels/panel_kit.rs:56` `exec`;
  - `control.rs:87` `execute_engine`, which uses `execute_checked` at `:89` and `:95`.

### 2.2 Registry, authority, undo

| What | Where |
|---|---|
| Spec | `crates/engine/src/cmd.rs:105` `CommandSpec { id, label, menu, shortcut, alt_shortcuts, params: &'static str, enabled, run, journal }`; `cmd!` `:123`, `query!` `:143`, `command_specs()` `:173`, `find` `:278` |
| Size | About **760** specs, counted statically (621 `cmd!`, 76 `query!`, 43 `fe!`, 5 `res!`, 5 `fast!`, 7 `chan!`, 3 literal). The comment at `:279` says "~2,000"; `capabilities` (`cmd/query.rs:187`) reports the real number at run time |
| Params | Free-text doc string; `cmd/schema.rs:77` `params_schema`, `:107` `check_params` |
| Authority lists to replace | `cmd/scripts.rs:34` `RUNS_SCRIPTS`; `cmd/batch.rs:14` `NOT_IN_BATCH`; `crates/automation/src/confine.rs:45` `REFUSED`, `:79` `READS`, `:124` `WRITES`, `:151` `IMPLICIT_WRITES` |
| Script permission in a generic setter | `cmd/settings.rs:21` `prefs_set`, refused at `:37` when `running_for_script()` and the permissions differ; `prefs_reset` `:52` keeps them. Other script checks: `cmd/effect.rs:230`, `cmd/blender.rs:61`, `cmd/app_more.rs:284` |
| Undo | `history.rs:27` `History { undo: Vec<(String, Arc<Project>)>, redo, merge_key, branches }`; `session/edit.rs:10` `edit(label, merge, f)` |
| Grouping today (four mechanisms) | `cmd/app_more.rs:20` `grouped`; `cmd/batch.rs:66` `batch` and `:123` `fold`; `background/jobs.rs:311` `collapse_undo`; script `beginUndoGroup` (`crates/script/src/runtime.rs:248`, `collapse` `:280`) |
| Learning | `home/learn.rs:354` `learn_observe` |
| Journal | `Session.journal: Vec<(String, Value)>` (`crates/engine/src/lib.rs:165`) |

### 2.3 Plugin system to remove

| What | Where |
|---|---|
| Runtime | `crates/plugin` (`lib.rs`, `wasm.rs`): `load_wasm` (`lib.rs:41`), `loader` (`:55`), `preview` (`:71`); wasmi 2 with `deterministic` and `wat` (workspace `Cargo.toml:73`); `pool: Mutex<Vec<Inst>>` (`wasm.rs:62`) |
| Registry | `crates/effects/src/plugin.rs`: `SLOTS: [OnceLock<Arc<dyn EffectPlugin>>; 1024]` (`:393`); `plugin_specs()` (`:396-399`); `render_errors()` (`:408-411`); `leak` (`:435-437`); leaking `param_spec` (`:439-461`) and `register_plugin` (`:467-490`); const-generic `TRAMPOLINES` (`:554`); `run_slot` (`:665-693`) **returns `before` on error** (`:684-688`) |
| Helpers | `crates/helper-host` registers external processes as API-2 plugins |
| Plug-in Library | `crates/engine/src/system/library.rs` (`CATALOG_URL` on `:34`, the `effectcraft-plugins` index); commands `library.{list, refresh, preview, install, update, uninstall, missing, loadInstalled}`; UI `crates/ui-egui/src/panels/settings.rs` from `:565` (`library_ui` and helpers to about `:752`) |
| Wiring | `crates/host/src/lib.rs:88-103` `session()` sets `plugin_loader`, `helpers`, `plugin_previewer`, `plugins_dir = <config>/Plug-ins` |
| Start-up | `apps/effectcraft/src/main.rs:285-293` runs `effect.plugins.load` and `library.loadInstalled`. **The CLI and MCP load no plugins today**, so parity (X1) is a behaviour change, not a refactor |
| Examples and docs | `examples/plugin-catalog/`, `examples/plugins/posterize-bands/`, `docs/plugins.md`, `docs/publishing-plugins.md` |

### 2.4 Render, export, project, attach

| What | Where |
|---|---|
| Missing effects dropped | `crates/render/src/pipeline.rs:620-630`: the stack is built with `filter_map(… effectcraft_effects::find(effect).map(…))`, so an unknown ID vanishes. Execution runs `:632-678`; CPU calls `effectcraft_effects::apply` (`effects/src/lib.rs:1450`, `(spec.render)(ctx, buf)`) |
| Infallible rendering | `RenderFn = fn(&EffectCtx, Buf) -> Buf`; `Session::render(...) -> Image` (`session/render.rs:5`) |
| Cache keys | `render/src/cache.rs` `layer_key` `:512`, `input_key` `:526`, `footprint_key` `:606`, `key_any` `:619`; disk salt `disk_cache.rs:1048` `RENDER_VERSION = "effectcraft-render-6"`. No plugin version or digest anywhere |
| Export | `crates/export/src/lib.rs:168` `export(job, progress)`; `render_queue.rs:494` `start_render(wait)`, thread `render-queue` (`:569`); **`export/src/out.rs:26` creates the final path directly** (no temp name); pre-start checks are `check_outputs` and `output_guard::duplicate_refusal`, with no plugin check |
| Effect lookup | `effects/src/lib.rs:752` `find` (built-ins, then `plugin::plugins()`); about 30 non-test callers of `find` |
| Project | `crates/project/src/props.rs:183-186` `GroupKind::Effect { effect: String }`; `PropGroup` `:249`; params as `Node::Prop` (`:58`). `SCHEMA_VERSION = 2` comes from craft-libs `crates/ecproj-format/src/lib.rs:65`; `check_schema` (`:85`) refuses newer files |
| Unknown effects | Survive load and save (the group is generic), **but** unknown JSON fields are dropped on save (no `flatten`). A schema-1 project opened without its plugin skips the effect-space shift for that effect (`session/edit.rs:351` `upgrade_effects`) yet is stamped schema 2, so the shift is lost: a data-fidelity bug E6 must fix |
| Control server | `apps/effectcraft/src/control_server.rs:27` `TcpListener::bind(("127.0.0.1", port))`, port from `--control` or `EFFECTCRAFT_CONTROL_PORT`; no token (`docs/control-protocol.md:33`) |
| MCP | `crates/automation/src/backend.rs:86` headless calls `execute_checked`; the bridge (`bridge.rs:26`) speaks TCP to the control server; the CLI flag is `--bridge PORT` (`apps/effectcraft-cli/src/main.rs:40`). There is no `--attach` |
| Layering | `xtask/src/main.rs:30` `LAYERS` (engine 4, host 4, plugin 3, helper-host 3, ui-egui 5). New crates must be added there |
| Lints and CI | Workspace `unsafe_code = "forbid"`; `clippy.toml` has `disallowed-types` only (no `disallowed-methods`); MSRV 1.95; CI calls `storytold/craft-common-actions` `gate.yml` and runs `cargo xtask ci` |

## 3. Layout and API sketches

```text
crates/plugins/            effectcraft-plugins (NEW; layer 4; depends on engine, effects, render, project, craft-ext-*)
  src/adapter.rs           ContractProvider (raster.filter@1), ProjectHooks, JobHooks; Level L1
  src/raster.rs            EffectCraft Buf <-> raster.filter@1 conversion (premultiplied f32 RGBA is native: no conversion)
  src/registry.rs          registers eligible contributions into effects::dynamic
  src/project.rs           PluginRef read/write, placeholders, plugin list
  src/manager.rs           embeds the shared manager (P5) in Settings ▸ Extensions
  src/cli.rs               `effectcraft-cli plugin …` via the shared command library (P5)
crates/effects/src/dynamic.rs    NEW: unloadable registry replacing SLOTS/trampolines
crates/engine/src/cmd/markings.rs NEW: one table, one row per command id
crates/engine/src/session/ops.rs  NEW: impl craft_ext_ops::OpApp for Session
```

```rust
// effects/src/dynamic.rs
pub enum EffectHandle { Builtin(&'static EffectSpec), Plugin(Arc<PluginEffect>) }
pub struct PluginEffect { pub meta: EffectMeta, pub release: ReleaseKey, pub runner: Box<dyn PluginRunner> }
pub trait PluginRunner: Send + Sync { fn render(&self, ctx: &EffectCtx, buf: Buf, mode: RenderMode) -> Result<Buf, EffectFailure>; }
pub fn resolve(id: &str) -> Option<EffectHandle>;          // built-in fast path stays &'static
pub fn meta(id: &str) -> Option<EffectMetaRef<'_>>;        // for UI and commands that need names and params only
pub fn register(effect: PluginEffect) -> Registration;     // returns a guard; dropping it unregisters
pub enum RenderMode { Preview, Final }
pub enum EffectFailure { Missing { id: String }, Plugin(HostFailure), Ineligible(Unavailable) }

// render pipeline
pub fn render_final(&mut self, …) -> Result<Image, RenderFailure>;   // export path; preview keeps `render` + diagnostics

// project/src/props.rs
Effect { effect: String,
         #[serde(default, skip_serializing_if = "Option::is_none")] plugin: Option<PluginRef> }
pub struct PluginRef { pub contribution: String, pub version: Version, pub contract: String,
                       pub digest: Option<String>, pub state: Option<PluginState> }
pub struct PluginState { pub schema: u32, pub data: Box<serde_json::value::RawValue> }   // written back byte for byte
// Project gains `plugins: Vec<PluginListEntry>` {id, version, catalogue link | "local"}

// engine: invocation in the Session
impl Session {
    pub fn execute_as(&mut self, inv: &Invocation, id: &str, params: Value) -> Result<Value>;  // the one entry
    pub fn execute_gui(&mut self, id: &str, params: Value) -> Result<Value>;                 // = execute_as(&Invocation::gui(), …)
    pub(crate) fn call(&mut self, id: &str, params: Value) -> Result<Value>;                 // nested: child of the current frame
}
```

- **Nested calls** (`call`) take the invocation on top of a session-held stack. They do not thread it through 98 call sites. The child is re-authorized ([P2 section 3](P2-ops.md#3-module-layout-and-api-sketches)), so a Normal command's body cannot lend a plugin a Management command.
- **The markings table** is the review artefact: about 760 rows of `(id, kind, marking)`. A test checks that every registered id appears exactly once. EffectCraft's ~760 macro calls stay untouched, which keeps a 760-line mechanical diff out of the macros.

## 4. Ordered PR list

Sizes as in the implementation plan (S ≤ 250 changed lines, M ≤ 600, L ≤ 1,000). Mechanical migrations are reviewed separately from behaviour changes.

| PR | Scope | Size | Files | Test that fails first | Mutants | Depends |
|---|---|---|---|---|---|---|
| E0 | **Inventory and fixtures, no behaviour.** Dump every command (`capabilities`, `command.describe`); classify kind and marking in a reviewed spreadsheet-like CSV under `docs/plugins/`; list authority-changing settings; capture legacy project fixtures with WASM API-1 (`posterize-bands`), Rust API-2 and helper effects, plus Plug-in Library installs | S (docs + fixtures) | `docs/plugin-ops-inventory.csv`, `crates/project/tests/fixtures/legacy-plugins/*.ecproj` | — | — | — (week 1) |
| E1a | Markings table and `kind`/`marking` lookup; `REFUSED`, `NOT_IN_BATCH` and `RUNS_SCRIPTS` derived from it | M | `cmd/markings.rs`, `cmd/batch.rs:14`, `cmd/scripts.rs:34`, `automation/src/confine.rs:45` | `markings::every_command_has_exactly_one_marking` (fails: table empty against about 760 ids); `markings::management_refused_by_confinement` | "marking ignored by confine" | E0, P2-03 |
| E1b | `scripting.setPermissions` (Management) split out of `prefs.set`; generic setters refuse scripting keys from every caller; the UI uses the new op | S | `cmd/settings.rs:21-52`, `ui-egui` prefs dialog | `settings::generic_prefs_set_cannot_change_scripting_permissions` (from the UI principal too) | "prefs.set allows scripting keys" | E1a |
| E2a | `OpApp` for `Session`; `execute_as`, `execute_gui`, `call`; the old three entry points become wrappers, banned by `clippy::disallowed_methods` outside named transition modules | M | `session/ops.rs`, `session/dispatch.rs`, `clippy.toml` | `invocation::principal_reaches_journal_and_undo_label` (the journal entry carries the principal; it fails because the journal holds `(String, Value)`) | "journal drops principal" | P2-05, E1a |
| E2b | ~35 entry points with real principals: CLI → `Cli`; control channel and MCP → `Automation` (Q1); script → `User(Script)`, replacing `script_calls` and `running_for_script` | M | `apps/effectcraft-cli/src/main.rs`, `apps/effectcraft/src/main.rs`, `apps/effectcraft-web/src/*.rs`, `automation/src/backend.rs:86`, `crates/script`, `ui-egui/src/control.rs:87-95` | `principals::script_cannot_set_scripting_permissions`; `principals::mcp_headless_is_automation`; `principals::control_channel_is_automation` | "script principal is Gui" | E2a, E1b |
| E2c | 98 nested engine calls → `s.call(…)` (**mechanical**; separate review) | M | `crates/engine/src/**` | `nested::vr_environment_keeps_outer_principal` (`create_vr_environment`, `cmd/app_more.rs:549`, run as Automation: every nested journal entry is Automation) | — | E2a |
| E2d | 190 UI calls and wrappers → `execute_gui` (**mechanical**, in 2–3 PRs by panel group) | M ×2–3 | `crates/ui-egui/src/**` | existing UI tests unchanged | — | E2a |
| E2e | Delete `execute`, `execute_checked`, `execute_for_script` and `ScriptCalls`; remove the transition allowances | S | `dispatch.rs`, `lib.rs:275`, `clippy.toml` | `compile_fail` doctest calling `Session::execute` | — | E2b..d |
| E3a | Dispatcher-owned transactions: outermost Edit snapshots (`Arc<Project>`, history, editor state, event mark), nested calls join, commit folds into one step labelled `"<label> · <origin>"`, rollback restores; replaces `grouped` (`app_more.rs:20`); merge-key folding (`edit.rs:10`) kept | M | `session/ops.rs`, `cmd/app_more.rs` | `txn::nested_edits_one_step`; `txn::step_two_failure_restores_everything` (project, history, editor state and events) | "nested opens own step"; "rollback skips history" | E2c |
| E3b | `engine.batch` (`batch.rs:66`, `fold` `:123`), `collapse_undo` (`jobs.rs:311`) and script undo groups (`script/runtime.rs:248-280`) moved onto E3a; `$N` references resolved before authorization | M | as named | `batch::refs_resolved_before_authorization`; the existing `batch_refs_one_undo_step_and_rollback` (`batch.rs:180`) still passes | — | E3a, P2-04 |
| E3c | Outbox: Effect-kind commands (file writes, `file.executeFile`, render-queue start, network) queued inside a transaction and run after commit; journal, learning (`learn_observe` only for `User(Gui)`), logs and events carry origin; events delivered after commit | M | `session/ops.rs`, `home/learn.rs:354` | `txn::failed_edit_runs_no_queued_effect`; `learn::automation_does_not_train_habits` | "effects before commit"; "learning from plugins" | E3a |
| A1 | **Shared attach crate** (placement: D5; recommended craft-libs `craft-attach`): Unix socket in a 0700 directory, Windows named pipe or AF_UNIX with a user-only ACL, a per-session token in a user-only file, the existing JSON-lines framing; token code modelled on PhotoCraft `crates/automation/src/security.rs` (`generate_token` `:79`, constant-time `token_matches` `:104`) | M | new crate | `attach::wrong_token_refused`, `attach::other_user_cannot_connect` (Unix: mode check; Windows: ACL check) | "token compare short-circuits" | R4 experiment |
| E4 | Control server on `craft-attach`; TCP removed; `effectcraft-cli --attach` replaces `--bridge PORT`; MCP bridge on the socket; "<client> is controlling EffectCraft · Stop" banner. **A security fix; may land before the rest** | M | `apps/effectcraft/src/control_server.rs`, `automation/src/bridge.rs`, CLI `main.rs`, `ui-egui` banner | `attach::no_tcp_listener_by_default`; `attach::banner_shows_while_client_connected` | — | A1 |
| E5 | Unloadable registry (`effects/src/dynamic.rs`): `EffectHandle`, `resolve`, `meta`, `register` guard; `SLOTS`, `TRAMPOLINES` and `leak` removed; the old loader moved onto it, so there is still only one system; ~30 `find` callers moved to `resolve` or `meta` | M | `effects/src/{plugin,dynamic,lib}.rs`, callers | `registry::unregister_frees` (`Arc::strong_count` and a drop counter reach 0); `registry::export_keeps_old_registration` | "unregister keeps metadata" | — (can start early) |
| E6 | Plugin references and placeholders: `PluginRef` with `RawValue` state; a project plugin list; **project schema 3 in craft-libs `craft-ecproj-format` first** (D4), so older apps refuse instead of dropping fields; the effect-space shift recorded per unresolved effect | M (+ S in craft-libs) | `project/src/props.rs:183-186`, `project/src/lib.rs`, `session/edit.rs:351`, craft-libs `crates/ecproj-format/src/lib.rs:65` | `project::missing_plugin_effect_round_trips`: each E0 fixture opened without its plugin and saved keeps the effect group's params, keyframes, expressions, label and order, and its `state` bytes identical; `project::schema1_missing_effect_keeps_pending_space_shift` | "state re-serialised" | E0, P1-05 (`ReleaseKey`) |
| E7a | Render failures: `RenderMode`; `pipeline.rs:620-630` keeps missing effects as explicit steps; Preview passes through with a badge diagnostic; `render_final` returns `Err`; export preflight in `start_render` (`render_queue.rs:494`) lists missing, failed or blocked plugins (CLI exit code 4 with `plugin install` lines) | M | `render/src/pipeline.rs`, `session/render.rs`, `render_queue.rs` | `export::refuses_missing_plugin_before_start`; `render::preview_passes_through_with_badge` | "missing effect dropped in final" | E5, E6, P2-07 |
| E7b | Output written to a temporary name and renamed when complete (`export/src/out.rs:26`); partial output, including written sequence frames, deleted on failure or cancel | M | `crates/export/src/out.rs`, `lib.rs:168` | `export::trap_mid_export_leaves_no_file`; `export::cancel_removes_partial_sequence` | "rename before finish" | E7a |
| E7c | Cache identity: plugin release digest, contract version, parameters and quality in `layer_key`/`key_any` and the disk salt | S | `render/src/cache.rs:512-619`, `disk_cache.rs:1048` | `cache::update_invalidates_only_that_plugin`; `cache::preview_never_reused_as_final` | "digest not in key" | E5 |
| E8a | `crates/plugins` adapter crate with tests, **not wired in** (support code only): raster conversion (none needed, since premultiplied f32 RGBA is native), registry glue, project hooks, job leases; added to `LAYERS` | M | `crates/plugins/**`, `xtask/src/main.rs:30` | `adapter::halftone_renders_through_raster_filter_v1` (the planned P4 fixture, not a selected MVP example) | — | P1-13, P3-07, P4-04, P4-06 pack/verify or an explicitly reviewed equivalent fixture producer, E5..E7 |
| E8b | **Atomic cutover**: wire the adapter in `host/src/lib.rs:88-103`; the CLI and MCP load eligible plugins from the store; embed the manager (replacing `library_ui`); add `effectcraft-cli plugin …`; delete per the checklist below | L (mostly deletions; reviewed by directory) | see checklist | `parity::same_plugin_same_output`; `commands::no_legacy_plugin_ids` | — | E8a, E2e, E3c, E4, P1-14, P5-01..03, selected L0 importer from P5-07 |
| E9 | Safe points and lifecycle: an update during export keeps the leased release; transforms switch on next use; safe start ("Start with plugins off?", `--safe-mode`, Help ▸ Restart with plugins off); Plugin console; per-plugin cost in the manager | M | `crates/plugins`, `apps/effectcraft/src/main.rs` | `lifecycle::update_during_export_keeps_leased_release`; `safe_mode::no_plugin_code_runs` | — | E8b, P1-07 |
| E10 | Bake plugin result (keeps a disabled original; un-bake); "Include plugins" when collecting files (off by default; never run on open; "Install from project" is a file install); full pilot acceptance run | M | `crates/plugins`, collect-files code | `bake::keeps_disabled_original`; `include::carried_package_never_runs_on_open` | — | E8b |
| E11 | Docs and release notes: old plugins stop running; native helpers unavailable until P10; `docs/plugins.md` and `docs/publishing-plugins.md` rewritten to point at the SDK | S (docs) | `docs/` | — | — | E8b |

**E8b deletion checklist** (each line ticked in the PR description with the remaining-reference grep):
- [ ] `crates/plugin/` (and its `LAYERS` row)
- [ ] `crates/helper-host/` (and its `LAYERS` row)
- [ ] `crates/effects/src/plugin.rs`: `EffectPlugin`, `PluginHost`, `register_plugin`, manifests, `run_slot`
- [ ] `crates/engine/src/system/library.rs` and the `library.*` commands
- [ ] `effect.plugins.*` commands (`cmd/effect.rs`), and `plugin_loader`, `plugin_previewer`, `helpers`, `plugins_dir` on `Session` (`engine/src/lib.rs:202-211`, `:297`)
- [ ] `apps/effectcraft/src/main.rs:285-293` start-up loading
- [ ] `crates/ui-egui/src/panels/settings.rs` `library_ui` and helpers (about `:565-752`)
- [ ] `examples/plugin-catalog/`, `examples/plugins/posterize-bands/`
- [ ] wasmi features `deterministic` and `wat` in the workspace `Cargo.toml:73`, if nothing else uses them

**Order.** E0 and E5 start in week 1. Then:

```text
E1a → E1b → E2a → {E2b, E2c, E2d} → E2e → E3a → {E3b, E3c} → E6 → E7a → {E7b, E7c} → E8a → E8b → {E9, E10, E11}
```

- A1 → E4 can land as soon as the attach crate exists. It is a security fix.
- E2a also needs P2-05, and E3b needs P2-04.
- E8a also needs P1-13, P3-07, P4-04 and a reviewed packed-fixture producer. E8b additionally needs caller-less API removal E2e, outbox/origin E3c, authenticated attach E4, known-revocation enforcement P1-14, manager/CLI P5-01..03, and the selected L0 importer from P5-07. These are the broad baseline's explicit predecessors, not approval of a reduced MVP migration.
- Wiring E8b alone is not completed pilot acceptance. E9's lifecycle/safe-mode proof and all selected workflow exit criteria must pass before the cutover is delivered as a usable MVP. If an E8b release would reach users independently, its required lifecycle protection must be included before that release. AR05 tracks the final activation graph; exact examples and other deferred choices remain open.

**Changed from the reconciled plan:**
1. **The call-site count is corrected** (317, not 691).
2. **Nested calls use a session-held invocation stack** instead of passing the invocation through 98 call sites, and they are re-authorized.
3. **The markings table** replaces editing about 760 macro invocations.
4. **The attach transport is a shared crate (A1)**, not EffectCraft code. PhotoCraft's token code is the starting point.
5. **E7 is split** into failure propagation, temporary output and cache identity. **E8 is split** into dormant support code and the cutover.
6. **Two findings** need their own work:
   - the CLI and MCP load no plugins today;
   - the schema-1 effect-space shift is lost for missing effects.

## 5. Data formats and migrations

**Project file.**
- `GroupKind::Effect` gains `plugin: Option<PluginRef>`, and `Project` gains `plugins: Vec<PluginListEntry>`.
- Older EffectCraft builds drop unknown fields on save, so the change ships with **`SCHEMA_VERSION = 3`** in craft-libs `craft-ecproj-format`. Older apps then refuse the file through `check_schema` instead of silently losing plugin data.
- A schema-2 project opens unchanged. Old-system plugin effects become placeholders with reason "from a removed plugin system".

```json
{ "uid": 41, "match": "io.github.ink.grain/halftone", "name": "Halftone",
  "kind": { "Effect": { "effect": "io.github.ink.grain/halftone",
            "plugin": { "contribution": "io.github.ink.grain/halftone", "version": "1.2.0",
                        "contract": "raster.filter@1", "digest": "sha256:9f2c…",
                        "state": null } } },
  "enabled": true, "children": [ … params as today … ] }
```

**Fixtures.**
- `crates/project/tests/fixtures/legacy-plugins/` (from E0).
- `crates/plugins/tests/fixtures/halftone.zip` (from P4, with its digest).
- `…/projects/halftone-v3.ecproj`.

**Placeholder fidelity.** "Byte for byte" applies to `PluginRef.state`, which is a `RawValue` written verbatim. Params, keyframes and expressions use EffectCraft's normal serialisation; they are compared as values, not bytes. The test states this exactly.

## 6. Test strategy

### 6.1 Layers

| Layer | Tests |
|---|---|
| Engine unit | Markings table, principals, nested calls, transactions, outbox, learning — all against a headless `Session` |
| Process | Attach: token, permissions, banner, no TCP; a fresh CLI refuses to write a project open in a running app |
| Fixture compatibility | Legacy projects round-trip; schema-3 refusal by a frozen schema-2 reader (`check_schema`) |
| Render | Preview badge; final refusal; mid-export trap; cancel; cache identity |
| Hostile | The P3 suite run through the adapter (`ext-conformance::hostile` with EffectCraft's `ContractProvider`) |
| End to end | The parity test below; the full lifecycle: install (file) → set up → render → save → reopen without the plugin → placeholder → export refused → install → export succeeds |

### 6.2 The parity test

`parity::same_plugin_same_output` uses one temporary store with Halftone installed and set up for `effectcraft`, and one project with a Halftone effect on a solid. It renders frame 12 three ways:
1. **GUI:** a `Session` built by the desktop path through `Invocation::gui()`;
2. **Fresh CLI:** `effectcraft-cli render --frame 12`;
3. **MCP headless:** a `tools/call` render.

It asserts that all three listed the same eligible release (digest) and produced the same frame digest.

### 6.3 Benchmarks

- **Built-in performance must not regress.** The E5 registry change must leave built-in effect-stack renders within noise. Record before and after on `effectcraft-render` benches.
- **Plugin cost.** Halftone at HD and 4K, through the adapter, compared with a native port of the same algorithm.
- **Dispatch overhead on a slider drag** (`prop.set` with a merge key), before and after E2a.

## 7. Risks, experiments, decisions

| # | Risk | Smallest experiment | Before |
|---|---|---|---|
| R1 | The mechanical UI migration (E2d) collides with ongoing UI work | Codemod script, landed in panel batches within days; agree a short freeze window with the maintainer | E2d |
| R2 | Unifying four grouping mechanisms breaks merge-key folding or script undo groups | A two-edit batch whose second step fails with a queued file output (reconciled plan risk 2); plus a slider drag; plus a script with nested `beginUndoGroup` | E3a |
| R3 | The unload registry slows the built-in hot path | Benchmark `resolve` against today's `find` with 1,000 lookups per frame; keep built-ins `&'static` | E5 |
| R4 | A user-only Windows named pipe needs Win32 security descriptors (`unsafe` FFI) | Try AF_UNIX on Windows 10+ in `%LOCALAPPDATA%` (inherits a user/SYSTEM/Administrators ACL), or a maintained crate with a safe API; check the licence | A1 |
| R5 | Project schema 3 needs a craft-libs release that EffectCraft pins | Coordinate a small craft-libs PR (ecproj-format) before E6 | E6 |
| R6 | Removing `helper-host` drops features users rely on until P10 | E0 lists every helper in use (first party or community) | E8b |
| R7 | 4K plugin effects are too slow for preview | P3-08 and the P4 benchmarks | E8b budget sign-off |

**Decisions for maintainers:**
- **D1. Principal of a fresh MCP server process.** Recommended: `Automation`, document authority only. Management stays with `effectcraft-cli plugin …` run by the user.
- **D2. What "basic temporal" means in P6.** Recommended: time, frame rate and seed in the `raster.filter@1` context (stateless). Stateful temporal effects with "backward seek resets state" wait for a `temporal` contract, by demand.
- **D3. Accept before E8b** that old plugins stop running (release notes only) and native helpers stay unavailable until P10.
- **D4. Project schema 3** in craft-libs `craft-ecproj-format`.
- **D5. Placement of the attach transport.** Recommended: craft-libs `craft-attach`, since it is automation infrastructure every app needs, plugins or not. The alternative is craft-extensions, as architecture section 5 implies for "every shared crate".
- **D6. Typed schemas.** Commands without typed argument schemas stay hidden from plugins until typed ([P2 R3](P2-ops.md#7-risks-and-open-questions)).

## 8. Effort and parallel work

| PR | Effort | Parallel with |
|---|---|---|
| E0 | S (1 week, with the maintainer) | P1-01..03 |
| E5 | M | E0, P1 |
| E1a, E1b | M, S | P2-04 |
| E2a..e | M, M, M, 2–3 × M, S | P3, P4 |
| E3a..c | M each (schedule a specialist review) | P4-05..07 |
| A1, E4 | M, M | any time after the R4 experiment |
| E6, E7a..c | M, M, M, S | P5-01..03 |
| E8a, E8b | M, L | — |
| E9, E10, E11 | M, M, S | P7 |

That is about 22 PRs: 5 S, 16 M and 1 L. E0, E5, A1 and E4 can run ahead of the platform units. E8b waits for P1-13, P3, P4 and P5.

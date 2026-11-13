# Adding plugin support to a Craft app (adapter guide, beta)

This guide is for app developers. It explains what an app must provide so the shared plugin
platform can run plugins in it, using the three working adapters as references.

The platform is shared code in `craft-extensions`:

| Crate | What it owns |
|---|---|
| `craft-ext-core` | Manifests, ids, permissions, contracts, forms, UI trees, project references |
| `craft-ext-store` | The shared store, installs, grants, `GrantView`, the catalogue and kill list, `Manager` |
| `craft-ext-host` | The wasm sandbox, native helpers, `AppApi`, `Grants`, `authorize` |
| `craft-ext-ui` | `ManagerView`, `ProjectPrompt`, and the plugin UI renderer (`plugin_ui`) |
| `craft-ext-cli` | `run_plugin`, the `<app>-cli plugin` command library |

The app adds **one adapter**: code in the app's repository that maps the platform onto the app's
documents, commands, undo, threads and UI. It contains no copy of shared logic.

## Reference adapters

| App | Adapter | App-side pieces |
|---|---|---|
| EffectCraft | `effectcraft/crates/plugins/src/`: `contract.rs` (contract), `ops.rs` (operations), `runtime.rs` (workers, transactions, tools, surfaces, app events, project references), `effects.rs` (effects and forms), `grants.rs` (`StoreGrants`), `lib.rs` (`start`, `manager`, `run_cli`) | `effectcraft/crates/engine/src/ext.rs` (`PluginRuntime` trait, `plugin.*` commands, gesture guard), `effectcraft/crates/engine/src/session/dispatch.rs`, `effectcraft/crates/ui-egui/src/plugins_ui.rs`, `effectcraft/crates/host/src/lib.rs` (safe mode) |
| PhotoCraft | `photocraft/crates/plugins/src/`: `contract.rs`, `runtime.rs` (ordered workers, pauses, surfaces), `document_thread.rs` (worker-to-document bridge), `filter.rs` and `session_filters.rs`, `resources.rs`, `lib.rs` (`safe_mode`, `store::generation`) | `photocraft/crates/engine/src/plugin_ops.rs` (operations), `plugin_sessions.rs` (transactions, gestures, command registry), `plugin_filters.rs` (effects as filter commands), `session/execute.rs` (gesture guard), `photocraft/crates/ui-egui/src/plugin_manager.rs`, `plugins_ui.rs`, `plugin_form.rs` |
| FigCraft | `figcraft/crates/plugins/src/`: `contract.rs`, `runtime.rs` (workers, operations, transactions, tools, app events, references), `grants.rs`, `lib.rs` (`Bridge`, `safe_mode`, `run_cli`), `runtime_tests.rs` | `figcraft/crates/engine/src/ext.rs` (`PluginRuntime` trait, `begin_group`/`finish_group`, `plugins.*` commands), `figcraft/crates/engine/src/session/exec.rs` (gesture guard), `figcraft/crates/ui-egui/src/plugins_ui.rs`, `plugin_dock.rs` |

## What an adapter owns

### 1. The contract

Implement `contract() -> AppContract` (`craft_ext_host::api`). It lists:

- `provided`: the contracts and minor versions the app supports, for example
  `("figcraft.session", 1, 1)`. Effects need `raster.filter`; data needs `resource.*`.
- `ops`: one `OpSpec` per operation, with `id`, `kind` (`Query`, `Edit`, `Ui`, `Effect`), example
  arguments and result, a `requires` function and a human `permission_rule`.
- `events`: what `craft_event` can deliver, and whether each needs `app.events`.
- `slots`: where each contribution kind appears.
- `notes`: colour, units, coordinates and limits.

`AppContract::to_markdown()` renders the page authors read. Serve it as `<app>-cli plugin contract`
and commit it as `docs/plugins/contracts/<app>.md`. Add the app's contracts to
`crates/ext-cli/src/known_apps.rs`; its test fails if that table and the contract page disagree.

Bump a contract minor when you add an operation, field or event. A breaking change needs a new
major.

### 2. Operations mapped to engine commands

Each operation must go through the app's own command path, so locks, journaling, validation and
undo behave as they do for a person. Examples:

- EffectCraft `ops::run` calls `Session::execute`: `layer.rename` becomes the engine's
  `layer.rename` on the captured composition (`effectcraft/crates/plugins/src/ops.rs`).
- PhotoCraft `plugin_ops.rs` uses the engine's commands or `Session::edit`, caps sizes before
  allocating, and returns recoverable `OpError`s.
- FigCraft `runtime.rs` maps `selection.set` and the other operations to validated engine commands
  with bounded arguments.

Treat every argument as hostile. Look targets up; never index by them. Refuse fields a plugin must
not set (FigCraft refuses `author` on comments and `path` on images).

Never expose undo, history, settings, shortcuts, plugin management or other plugins' data. An
operation that is not listed cannot be called: `authorize` refuses unknown ids.

### 3. Permission requirements per operation

`requires(args)` must return **every** permission the call needs for those arguments. `authorize`
then checks that the calling contribution declared each permission and that the package holds a
grant that covers it.

If `requires` depends on app state, such as whether a target was selected when the event began,
return `true` from `AppApi::authorizes_on_owner_thread` and call `authorize` on the document
thread with the captured state. EffectCraft does this with `CAPTURED_SELECTION`: renaming a layer
needs `document.edit:selection` only if that layer was selected at the start of the event.

Write a test that every operation has a total rule (PhotoCraft:
`every_operation_has_a_total_permission_rule`).

### 4. Transactions: one event, one undo step

Implement `AppApi`:

- `begin(inv)`: capture scope (document, selection, composition) and open an edit group.
- `call(inv, op, args)`: re-check the invocation, authorize, run the operation inside the group. An
  operation error is returned to the plugin; it does not end the transaction.
- `finish(inv, commit)`: commit as one undo step labelled `inv.label` (`<label> · <plugin name>`),
  or roll back everything.

Re-check on the owner thread before each call and again before commit: the invocation is not
cancelled or past its deadline, the plugin is not stopped, the grant generation is unchanged, and
the document is the one captured. If any check fails, roll back.

Stage UI changes (panel trees, windows, overlays) during the event and publish them only on
commit. Run side effects (PhotoCraft file writes) only after commit, and re-check grants,
cancellation and scope before each one. Report external effects that a rollback cannot undo.

| | EffectCraft | PhotoCraft | FigCraft |
|---|---|---|---|
| Group | `Session::begin_group` / `finish_group` | Document state and journal, folded by `finish_plugin_edits` | `Session::begin_group` / `finish_group` |
| Transaction | `Runtime::transaction` | `Session::plugin_transaction` | `Runtime::serve` |
| Hold limit | 5 s (`TX_TIMEOUT`) | 5 s | 5 s |
| Event deadline | 8 s | 8 s | 9 s |

### 5. Worker threads and the UI thread

Plugin code never runs on the thread that owns the document.

- Each (plugin, module) gets a worker thread with an ordered queue. The worker owns the
  `craft_ext_host::wasm::Session` or `native::Helper`.
- The worker's `AppApi` is a bridge that sends `Begin`, `Call` and `Finish` requests to the document
  thread and waits with a timeout.
- The document thread serves requests in its pump (each frame in the desktop app; inline in the
  CLI). A transaction is served from begin to finish in one pump, so no other edit interleaves.
- Defer requests from other invocations until the open transaction ends.

See EffectCraft `Bridge` and `Runtime::pump` (`runtime.rs`), PhotoCraft `DocumentApi`
(`document_thread.rs`) and `Session::pump_plugins` (`plugin_sessions.rs`), and FigCraft `Bridge`
(`lib.rs`) and `Runtime::pump` (`runtime.rs`).

### 6. Grants and reloading

Implement `craft_ext_host::api::Grants` over `craft_ext_store::view::GrantView`, and refresh it on
every check. Fail closed: if the store cannot be read, report the plugin as stopped.
EffectCraft `StoreGrants` refreshes a cached view (`refresh_if_changed`, throttled to 250 ms).
PhotoCraft opens a fresh view for each authorization.

Reload plugins only when **`StoreState::app_fingerprint(app)`** changes. It covers this app's
setups (active and pending versions, enabled contributions, grants, consent, grant generation,
pins), the bytes of each active release, and kill-list hits. Catalogue refreshes, unrelated
installs and other apps' setups leave it unchanged, so they must not reload anything.

- EffectCraft: `GrantView::refresh_if_changed` compares fingerprints; `pump` unloads and reloads
  when `StoreGrants::changes` moves.
- PhotoCraft: `photocraft_plugins::store::generation()` returns the fingerprint; filter and
  session-command registries rebuild when it changes (checked at most every 2 s).
- FigCraft: `Runtime::pump` compares the fingerprint and reloads.

On reload, cancel any open tool gesture, stop workers, unregister effects and drop surfaces of
contributions that are no longer enabled.

### 7. Effects

For `raster.filter@1`, convert pixels honestly: premultiplied RGBA `f32` in the working space you
name in `ctx.color`. Refuse modes you cannot convert, with a reason. Run the transform on a worker
with a `check` callback that stops it when the plugin is disabled, revoked or kill-listed.

- EffectCraft registers effects with `effectcraft_effects::external`. Missing effects pass through
  with a status badge, and final renders refuse while any remain.
- PhotoCraft exposes each effect as a filter command `plugin.<id>/<contribution>` under
  Filter ▸ Plugins and applies it as one history step. Non-RGB documents, masks, channel subsets
  and unconvertible profiles show why the filter is unavailable.

If you support effect forms, call `wasm::Plugin::update` after edits with a deadline and apply
patches with `Form::apply_patch`.

### 8. Panels, windows, tools and overlays

Draw every panel and window with the shared renderer, `craft_ext_ui::plugin_ui::PluginUi`. The app
chooses the engine (`EngineChoice::Egui` or `Framework`); a plugin can never choose or detect it.
The renderer returns `UiEvent`s tagged with the revision drawn. Drop events for an old revision,
then send a `ui` event to the owning worker.

- Validate trees against the UI level of the contribution's declared session contract.
- Never reopen a panel the person closed (PhotoCraft test
  `panel_updates_never_reopen_a_closed_panel`).
- Limit windows (EffectCraft and FigCraft: four per plugin).

For tools, keep one gesture: `begin` opens an edit group and captures the document, selection and
grant generation; `update` events run inside it; `commit` closes it as one undo step; `cancel`,
a failed event, a document switch or a revocation roll the whole gesture back. Clear overlays
when the gesture ends.

**Refuse outside edits during a gesture.** While a plugin gesture is open, refuse Undo, menu
commands and agent edits; allow only queries, view navigation and the plugin's own operations.
Otherwise those edits would be folded into the gesture or lost on cancel. See
`Session::refused_during_gesture` (EffectCraft `engine/src/ext.rs`), `Session::execute` (PhotoCraft
`engine/src/session/execute.rs`) and FigCraft `session/exec.rs` with `NAVIGATION_COMMANDS`.

### 9. Project references and the missing-plugin prompt

- Record a `craft_ext_core::project::PluginRef` for each plugin a document depends on, with the
  contributions used and their `state_version`s. EffectCraft derives them from effect instances
  and required contributions (`runtime::project_refs`). FigCraft records a reference when a plugin
  event commits an edit.
- Store each use as a `FeatureRef`, and keep its `params` and `state` byte for byte when the plugin
  is missing.
- On open, call `Manager::check_project` and show `craft_ext_ui::ProjectPrompt`
  ("This project uses N plugins", with Download, Enable, Skip, "Open without them" and Continue).
  Opening never downloads or enables anything.
- Refuse final export only while rendering contributions (effects) are missing.
- Serve `<app>-cli plugin project <file>` by extracting references and calling
  `craft_ext_cli::print_project_status`.

PhotoCraft does not record references yet.

### 10. Management UI and CLI

- Embed `craft_ext_ui::ManagerView` (Browse, Installed, Updates, Problems). Use
  `show_with_app_problems` to add paused plugins with a **Resume** button to the Problems tab, as
  FigCraft does. Pauses happen after three failed events in a row and last for the session.
- Add `<app>-cli plugin ...` by passing arguments to `craft_ext_cli::run_plugin` with a `Manager`
  for this app. Handle `plugin contract`, `plugin project <file>` and `plugin resume <id>` in the
  app. See the [CLI reference](cli-reference.md).
- Start one background catalogue refresh per process (every six hours).

### 11. Safe mode

Read `<APP>_SAFE_MODE=1` (EffectCraft also accepts `--safe-mode`). In safe mode, load no plugin
code and refuse plugin events, but keep the manager and project checks working, so a person can
disable or remove the plugin that breaks startup. Show a notice in the manager.

## Checklist for a new adapter

1. Remove the app's old plugin system completely.
2. Write `contract()` with honest `requires` rules. Add the app to `known_apps.rs` and commit the
   generated contract page.
3. Implement each operation through the app's own commands, with bounded arguments.
4. Implement the bridge `AppApi` and the owner-thread transaction: capture scope, authorize,
   re-check before commit, one labelled undo step, full rollback.
5. Implement `Grants` over `GrantView`, failing closed.
6. Load enabled contributions at start; reload only when `app_fingerprint` changes.
7. Run plugin code on workers; serve requests on the document thread with timeouts.
8. Draw panels and windows with `plugin_ui`; drop stale revisions; stage UI until commit.
9. Implement tool gestures and refuse outside edits while one is open.
10. Register effects with honest pixel conversion, if the app has raster layers.
11. Record project references, keep missing data, show the prompt, block final export only for
    missing effects.
12. Embed `ManagerView`, add the CLI commands, add the background refresh.
13. Implement safe mode.

### Tests to write

Name each test after the behaviour it proves. The existing adapters have one of each:

| Behaviour | Example test |
|---|---|
| Every operation has a total permission rule | PhotoCraft `every_operation_has_a_total_permission_rule`; EffectCraft `contract_is_complete_and_renders` |
| Selection-scoped rules use captured state | EffectCraft `rename_needs_selection_scope_only_for_captured_layers` |
| Several edits in one event are one labelled undo step | PhotoCraft `multiple_session_edits_fold_to_one_labelled_history_step`; FigCraft `edits_record_reference_once_and_form_one_undo_step` |
| A failed event restores the document, history and journal | PhotoCraft `failed_event_restores_document_history_revision_and_journal` |
| A denied operation does not abort the event | FigCraft `denied_single_operation_does_not_abort_event`; EffectCraft `denied_and_invalid_ops_allow_panel_redraw_in_the_same_transaction` |
| Revocation and generation changes take effect immediately | FigCraft `grant_reads_fail_closed_and_generation_changes_are_immediate` |
| Queued requests are refused after disable, release change or a new document | EffectCraft `queued_requests_reject_disabled_contribution_release_change_and_new_document` |
| A revoked or failed gesture rolls back every phase and the overlay | EffectCraft `revoked_gesture_rolls_back_all_phases_and_overlay`; FigCraft `failed_phase_rolls_back_gesture_even_when_guest_returns_zero` |
| Outside edits wait while a gesture is open | EffectCraft `edits_wait_while_a_plugin_gesture_is_open`; PhotoCraft `outside_edits_wait_while_a_tool_gesture_is_open` |
| Gesture events record nothing until commit | PhotoCraft `gesture_events_record_nothing_until_the_gesture_commits` |
| Stale surface revisions are dropped | EffectCraft `queued_surface_revision_is_rechecked_at_service`; FigCraft `canvas_widget_pointer_events_reach_the_plugin_and_stale_revisions_are_dropped` |
| Closed panels stay closed | PhotoCraft `panel_updates_never_reopen_a_closed_panel` |
| Three failures pause; Resume starts a fresh instance | EffectCraft `three_traps_pause_and_resume_reinstantiates`; PhotoCraft `three_failures_pause_and_resume_clears_the_pause` |
| Store changes unload disabled contributions | FigCraft `store_generation_change_removes_disabled_contributions_and_workers` |
| Project references and saved state survive | EffectCraft `project_refs_include_required_contributions_and_state_versions`, `missing_effects_include_plain_and_unknown_ec_ids_and_preserve_parameters` |
| External writes roll back the document when they fail | PhotoCraft `failed_file_publication_rolls_back_document_history_and_picks` |

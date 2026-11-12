# P8: FigCraft (session style, panels, tools, canvas, consent UI)

Status: **plan, 2026-11-10.** Unit P8 of [the architecture](../architecture.md#22-delivery-plan); deepens [implementation plan section 6, P8](../implementation-plan.md#p8-figcraft-p9-photocraft-p10-native-p11-p12). It is lighter than the critical-path units and is to be deepened when P6 lands. Each PR still needs its own approval.

**Code read:** FigCraft `main` `0e86f359`, read with `git show` and `git grep`. Nothing was built or run.

## 1. Goal and exit criteria

**Goal.** FigCraft is the first app at **L2**. Session plugins call FigCraft's exposed operations through the dispatcher. Host-drawn declarative panels, pointer tools, canvas widgets and plugin object types arrive, with consent sheets and first-use prompts. The old plugin system (manifest API levels 1–6) is removed in the same change set.

**Exit criteria (architecture section 22).**

| # | Criterion | Test |
|---|---|---|
| X1 | Panel and CLI parity: a panel action and the same operation from the CLI give the same document | `parity::panel_and_cli_same_result` |
| X2 | Escape cancels a pointer tool with no change and no undo step | `tool::escape_cancels_unchanged` |
| X3 | One undo step per plugin edit, labelled with origin | `txn::one_step_per_plugin_edit` |
| X4 | A plugin cannot reach Management or Core operations, nor history | `ops::plugin_denied_protected` (run against FigCraft's inventory) |
| X5 | The control channel is authenticated and local-only (on `craft-attach`) | `attach::no_tcp_listener_by_default` |

## 2. Code areas (verified at `0e86f359`)

| What | Where |
|---|---|
| Old plugin API levels | `crates/plugin/src/manifest.rs:11` `API_VERSION = 6`; a missing `api` means 1 (`:62`); validated at `:348-365`. **The architecture and the reconciled plan say "APIs 1–4". The real range is 1–6**: 2 panels and settings, 3 widgets, 4 `save_file` and widget fields, 5 widget opens panel, 6 widget `line`/`svg` (`docs/plugins.md:16-23`). The `sandbox.rs:5-6` header still says "ABI (version 4…)" |
| Sandbox | `crates/plugin/src/sandbox.rs` (614 lines): `engine()` `:266`; `Limits` `:68` (1 G fuel, 10 k per host call, +1 per byte, 64 MiB); `StoreLimits` `:339`; `check_module` `:278`; `run` `:317`; `Instance` `:322` keeps memory across calls; `IMPORTS` `:49-64` (`call`, `selection`, `reply_len`, `reply_read`, `log`, `notify`, `set_output`, `ui_show`, `ui_close`, `setting_get/set/delete`, `widget_render`, `save_file`); wasmi 2.0.0 |
| Engine host | `crates/engine/src/extensions/plugins.rs:234` `impl Host for EngineHost`; `refusal()` `:148` (allowlist; `Access::Read` blocks `history`); `call_plugin` `:529` wraps a run in `grouped` (one undo step); commands `plugin.list/remove/run/event/ui/close`, `plugins.runLast`, `plugins.manage` (`specs()` `:653`) |
| Dispatcher | `crates/engine/src/session/exec.rs:58` `execute_as(agent: Option<&str>, id, params)`, `:71` `execute(id, params)` (one mutation = one undo step; failure restores the snapshot); registry `crates/engine/src/cmd.rs:36` `CommandSpec { id, label, params, history, run }`, `find_command` `:142` |
| Grouping | `crates/engine/src/session/undo.rs:256` `grouped(label, f)`; events buffered while `group_depth > 0` (already post-commit delivery) |
| Panels today | `ui_show` with a `Panel` JSON; drawn as a floating window in `crates/ui-egui/src/dialogs/plugins_ui.rs:232-259`; control changes return through `plugin.event` |
| Widgets | `crates/engine/src/extensions/widgets.rs`, `crates/plugin/src/widget.rs` (`WidgetRender`) |
| Control channel | `apps/figcraft/src/control_server.rs:99` `TcpListener::bind(("127.0.0.1", port))`; no token; MCP attached backend over it (`crates/mcp`) |
| Call sites | **About 94** non-test `.execute(`/`.execute_as(` calls: 107 by the P6 method, less 13 in `*_tests.rs` files included through `#[path]`. Engine 52 (10 of them test files), UI 32, MCP 10, desktop 6, CLI 5, web 2. Some MCP hits are a different type's `execute` (`crates/mcp/src/headless.rs:57` defines its own) |

## 3. Layout and API sketches

- **`figcraft-plugins`** (new adapter crate, L2):
  - `OperationMarkings` from a markings table, as in P6;
  - `UiExtensionPoints` (panels, menu anchors through craft-libs CL-2, inspector sections, canvas widgets, tools);
  - `ProjectHooks` (node references, fallback previews);
  - `JobHooks`.
- **Shared, in craft-extensions** (needed first here):
  - `ext-contracts::ui` (provisional until a second L2 app): the declarative UI tree, events, canvas draw lists, tool gestures.
  - Session-contract guest bindings generated from FigCraft's inventory (`craft-guest-figcraft`, crate major = contract major).
  - Broker implementations from P2-09 in `ext-host`. These are needed before files, network or clipboard are advertised.

```rust
// ext-contracts::ui (provisional)
pub enum UiNode { Column(Vec<UiNode>), Row(Vec<UiNode>), Label(Label), Button { id: ControlId, label: Label },
                  Slider { id: ControlId, label: Label, range: RangeInclusive<f64>, value: f64 }, Toggle { … }, Choice { … },
                  TextField { … }, Canvas { id: ControlId, size: [u32; 2], content: DrawList } }      // every control requires a Label
pub enum UiEvent { Clicked(ControlId), Changed(ControlId, Value), CanvasPointer(ControlId, PointerEvent), Closed }
pub enum ToolPhase { Begin(Point), Update(Point), Commit, Cancel }                                       // Escape = Cancel, no edit
pub struct DrawList { pub ops: Vec<DrawOp> /* bounded: 10 000 ops, 1 MiB */ }
```

## 4. PR list (outline)

| PR | Scope | Size | Test that fails first | Depends |
|---|---|---|---|---|
| F0 | Inventory and fixtures: every command classified; plugins at API levels 1–6 captured as legacy fixtures (manifests plus projects with plugin widgets) | S | — | — |
| F1 | Markings table; `refusal()` and the allowlist rules derived from markings | M | `markings::every_command_has_exactly_one_marking` | F0, P2-03 |
| F2a..e | Mandatory invocation, using P6's sequence: `execute_as(agent)` becomes `execute_as(&Invocation)`; entry points; nested calls; UI; deletion | M ×3, S ×2 | `invocation::principal_reaches_undo_label`; `compile_fail` on the caller-less `execute` | F1, P2-05 |
| F3 | Dispatcher-owned transactions on `grouped` (`undo.rs:256`) | M | `txn::nested_edits_one_step` | F2 |
| F4 | Control channel on `craft-attach` (TCP removed) | M | `attach::no_tcp_listener_by_default` | P6-A1 |
| U1 | `ext-contracts::ui`, tested in the reference host first (layout bounds, label requirement, draw-list caps) | M | `ui::unlabelled_control_refused`; `ui::draw_list_over_cap_refused` | P4-01 |
| U2 | Tool gestures and canvas widgets in the reference host | M | `tool::escape_cancels_unchanged` (reference) | U1 |
| U3 | Consent UI in the manager: setup sheet, first-use prompt from a user-started GUI action only, host-drawn | M | `consent::prompt_only_from_user_action` | P5-03 |
| F5 | Object types: host-managed node type with plugin rendering and inspector; fallback preview saved | M | `object::opens_without_plugin_as_locked_preview_keeping_data` | U1, P6-E6 pattern |
| F6 | Project and job hooks: plugin list, placeholders, export refusal | M | `export::refuses_missing_plugin` | F5 |
| F7a | `figcraft-plugins` adapter, dormant | M | `adapter::l2_checklist` | U1..U3, F3 |
| F7b | **Atomic cutover**: wire the adapter, manager and `plugin` CLI (L0 included); delete `crates/plugin/` (sandbox, manifest, widget), `extensions/plugins.rs`, `extensions/widgets.rs`, `dialogs/plugins_ui.rs`, the `plugin.*`/`plugins.*` commands | L | `parity::panel_and_cli_same_result`; `commands::no_legacy_plugin_ids` | F7a, P5 |

## 5. Data formats

- **Inventory:** `figcraft@1.0.json` in the [P2 encoding](P2-ops.md#51-operation-inventory-appcontractmajorminorjson-shared-with-p4).
- **UI tree:** JSON in the control plane, with an inventory section `ui@1`.
- **Object type data:** a typed plugin `state` (`RawValue`) plus a saved fallback preview (SVG or PNG) per object.

## 6. Test strategy

The P6 layers apply, plus these UI checks:
- **Panel tests:** `egui_kittest` checks of host rendering, AccessKit names and keyboard order.
- **Tool tests:** pointer sequences replayed headless.
- **Off the UI thread:** each plugin event runs as an invocation under the command budget, on a worker; the test asserts the UI frame time stays under budget while the guest loops.

## 7. Risks and decisions

- **R1, session latency.** One instance per package per window, on a worker. Measure the round-trip time from panel event to commit.
- **R2, `check` inference.** Session plugins call operations through one dispatcher import, so `check` warns from the inventory's `requires` upper bounds and the runtime enforces (reconciled plan risk 13).
- **D1. Correct the architecture's "APIs 1–4" to "API levels 1–6"** in sections 22 and 25 and in the implementation plan.
- **D2.** Which FigCraft operations are typed first. Only typed operations reach plugins ([P2 D4](P2-ops.md#7-risks-and-open-questions)).

## 8. Effort and parallel work

- **Counts:** about 16 PRs (3 S, 12 M, 1 L).
- **Parallel:**
  - F0, F1 and F4 can start while P6 runs.
  - U1–U3 run in craft-extensions beside P6 E8.
  - F7b waits for P6 E8b, so the shared adapter traits have one real user first.

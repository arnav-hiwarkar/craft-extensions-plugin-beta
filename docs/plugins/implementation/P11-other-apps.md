# P11: Other apps (by demand; FilmCraft next) and the themes revisit

Status: **plan, 2026-11-10.** Unit P11 of [the architecture](../architecture.md#21-app-adapters). Each app's maintainers build and own their adapter from the skeleton and guide the platform team writes after P6, P8 and P9. No other app's source was re-read for this plan. App facts are in the [readiness audits](../readiness/README.md) at their recorded revisions, and in the [merge-hold list](../merge-hold-list.md).

## 1. Goal and exit criteria

- **Goal.** Every remaining app reaches L0 early (P5b), and L1 or L2 by demand, with the same proofs as the pilots.
- **Exit criteria per app:**
  - an inventory exported, with every operation marked;
  - the adapter checklist passing for its level;
  - persistence with placeholders and export refusal (for L1 and above);
  - atomic replacement of any old plugin system;
  - headless and GUI proofs;
  - the attach transport on `craft-attach`, where the app has a control channel.

## 2. Per-app template (one plan file per app when it starts)

| Step | Output | Test |
|---|---|---|
| A0 | Read the app at `main`; recount dispatcher call sites with the [P6 method](P6-effectcraft.md#21-dispatcher-and-call-sites-recounted); list control channels and any old plugin system | — |
| A1 | Markings table and inventory export in the app's CI | `markings::every_command_has_exactly_one_marking` |
| A2 | Mandatory invocation (entry points, nested calls, UI, deletion) | `compile_fail` on the caller-less dispatcher |
| A3 | Dispatcher-owned transactions | `txn::nested_edits_one_step` |
| A4 | Attach on `craft-attach` | `attach::no_tcp_listener_by_default` |
| A5 | Adapter at its level, dormant | `adapter::<level>_checklist` |
| A6 | Atomic cutover | app-specific parity test |

## 3. Known per-app points

These come from the architecture, section 21 and section 9 ("industry-format hosts kept"):

| App | Level target | Hard part | Notes |
|---|---|---|---|
| FilmCraft (next) | L1 temporal, L2 timeline ops | Timebase, export jobs | Needs the `temporal` contract that P6 deferred ([P6 D2](P6-effectcraft.md#7-risks-experiments-decisions)) |
| VectorCraft | L2 object and path contract | Units, geometry validation | `path2d` is a candidate shared contract; promote it only on a second consumer |
| SoundCraft | L0; native hosts | Real-time rules | VST3, CLAP and AU hosting kept as **native full access**, listed and managed in the shared manager (P10 labels); no Craft real-time audio contract until there is evidence |
| CADCraft | L2 own operations; native | Step-by-step prompts as headless parameters | AutoLISP kept as native full access; **`defun` over a built-in command is refused** so the protected core holds: a test redefines a built-in and expects refusal |
| GridCraft | L1 functions | Recalculation, cycles | Workbook operations as a session contract later |
| LightCraft | L0 | — | Its unauthenticated control channel is a security fix under the merge-hold exceptions; move it onto `craft-attach` early |
| Others | L0, then by demand | — | Per the [work packages](../draft/shared-operations-and-apps.md#per-app-work-packages) |

## 4. Themes revisit

Themes stay at colours, fonts and density (architecture section 17) until the user asks for more. When asked:
1. reopen the T14 question with options;
2. extend `palette.rs` only through manifest schema rules (P1-10);
3. keep the WCAG contrast check (`ext-core/src/palette.rs:62` `contrast_ratio`).

## 5. Effort

Per app: L0 is about 4 S PRs. L1 and L2 follow the pilot shapes, roughly 10–20 PRs depending on the size of the dispatcher migration, which A0 measures.

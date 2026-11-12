# Shared operations and application changes

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: reconciled architecture proposal, 2026-11-08. The user chose maximum useful reuse between plugin and CLI functionality, with app-specific adapters and explicit permissions. This document defines that boundary and the per-app work. It does not authorize app implementation or claim an exhaustive current-source audit.

## Concrete draft choices

- Each app owns a typed Rust operation registry. It generates one versioned inventory used for SDK bindings, CLI discovery/help, documentation and conformance coverage.
- GUI, CLI and plugins reach the same validated app behavior with distinct caller authority. A bounded policy-checked dispatcher is allowed; caller-elevating CLI bridges are forbidden.
- Extend existing core/store in craft-extensions. Share operation-support mechanisms in craft-libs only with real common consumers; app data and semantics remain in apps.
- Every adapter implements the same lifecycle checklist, advertises only implemented contracts, and proves its app-specific document/media behavior.
- The first store changes remove automatic enablement, preserve grants/preferences, retain multiple releases and block cross-source takeover before executable integration relies on them.

## Common library boundary

| Share | Keep app-specific | Reason |
|---|---|---|
| Operation descriptor/schema support and inventory generation | Actual operation definitions and engine implementation | A common generator can describe a timeline command without owning timeline code |
| Caller/principal envelope, capability/scope comparison, policy result | Document targets, validated edits and argument-dependent side effects | Common checks cannot infer whether an app command uploads media or mutates another document |
| Job/result/error/progress/artifact vocabulary | Scheduler integration, app transactions and output commit | Shared terminal states prevent CLI discrepancies; lifetime and atomicity remain app-owned |
| Opaque handle mechanics, cancellation/deadline and budget accounting | Handle dereference, revisions, selection and data conversion | Shared mechanics must not allow forged or stale app references |
| Package identity, compatibility, grants, resolution and retention | Contribution registration/evaluation | No second installer, per-app package database or competing grant interpretation |
| Parameter/declarative UI descriptions, accessibility metadata | App controls, gestures, rendering and semantic theme mapping | Author convenience does not require exporting internal UI structs |
| State/reference envelope and migration result vocabulary | Project serialization, unknown-feature preservation and native fallback | Apps retain file format and recovery ownership |
| Conformance harness contracts and fixture reporting | Literal expected app behavior and media tolerance | A shared test runner cannot invent correct pixels, geometry, recalculation or audio latency |

Reuse existing primitives where their definitions fit. The workspace lists `craft-keymap`; inspect its actual entry points before integrating contributed shortcuts. Do not invent pre-existing command/job/schema crates. This table is an allocation of responsibilities, not a new crate list or a request for generalized infrastructure before the pilots.

## Operation inventory procedure

For each app, implementation work begins with a bounded source audit at a recorded candidate revision. Existing research records named revisions and discovery limits; it is not a substitute for checking a changed app.

1. Enumerate existing GUI actions, CLI commands, engine operations, plugin APIs and background jobs for the first supported contribution families.
2. Open their definitions. Record targets, parameters, output, mutation/side effects, undo, selection, resources, asynchronous behavior and current tests.
3. Identify behavior duplicated in UI/CLI/plugin glue. Move reusable behavior into the app engine where necessary, while retaining app semantics and compatibility adapters.
4. Add or derive operation descriptors. Explicitly classify allowed callers, required capabilities computed from arguments/targets, interaction requirements and transaction/job behavior.
5. Generate the host profile and SDK inventory. Unsupported or unsafe-to-expose functions remain absent from plugin discovery, even if a trusted desktop command exists.
6. Exercise GUI/CLI/plugin parity plus denied operations, stale scope and cancellation. Migrate affected internal callers and remove duplicate paths within that scope.

A future machine-readable audit record contains app/revision, operation ID, implementation path, callers, authority, side effects, transaction/job owner, contract version, fixture and status. Generate its readable reference from the same registry. The draft does not create an unused parallel database of declarations.

## Policy-checked dispatch

For new command/query contracts, recommend generated typed SDK wrappers over one bounded operation dispatch entry point, plus necessary lifecycle/progress imports. The dispatcher validates the operation version and payload before passing a host-derived principal through centralized authorization. Existing ABI adapters can expose separate host imports that converge on this same policy engine.

The number of imports does not itself establish security. Both dispatcher and wrappers need bounded decoding, typed operation-specific validation and unchanged caller context for nested/queued work. Large image/audio buffers can use separately designed handles/streams rather than forcing every workload through serialized command messages. No interface is made component-model compatible by assertion.

CLI callers authenticate to the intended session when attached. Persistent servers require a scoped transport/session policy before use; an open port is not authorization. Plugin callers cannot reach privileged management commands, mint grants, or invoke a remote/CLI bridge as a user. The user's explicit grant determines plugin authority, not the existence of the underlying app function.

## Required shared-store changes

The source investigation at craft-extensions `ffd8512e` and the Opus comparison with local main `a6c5fae` identify concrete prerequisites. These are source-derived observations, not reproduced failures.

| Existing behavior | Required change | Observable future acceptance |
|---|---|---|
| Install enables every app listed by the manifest; update repeats it | New installs enable nothing until explicit app setup; updates preserve prior per-app choices | Install multi-app package, enable PhotoCraft only, update, confirm FigCraft remains disabled |
| State selects one release and update immediately deletes old bytes | Release records keyed by bound package identity and digest; separate default/pins/active-reader ownership | Update/remove during a long export and lazy-resource import without losing inputs |
| Installed records omit verified manifest, source and release digest | Persist authoritative admission data and source-bound identity | Changed disk bytes and same-ID local takeover fail at load/update |
| Reader filters largely by kind/app/enabled/directory | Central eligibility plus host build policy, compatibility, revocation, grants and actual module checks | Same fixture yields same GUI/CLI/reopen/load refusal; release host rejects gated development records |
| Old writers ignore unknown state fields and discard them on save | Explicit schema migration and old-writer refusal; one atomic coherent snapshot | Alternate old/new clients cannot erase authority data or return false successful empty state |
| Index lacks completed authentication and exact-source revocation enforcement | Authenticated catalogue interpretation, digest/source-scoped revocation and live enforcement | Wrong-source asset, same-version changed metadata and revoked installed release rejected |
| Crash can leave a published directory without selected state | Recoverable publication journal/state boundary and idempotent verified reuse/quarantine | Fault injection at each write boundary; retry converges without deleting another active operation |

## Per-app work packages

All rows inherit discovery/compatibility/activation/loading, caller policy, transactions, persistence, jobs, CLI/artifacts, errors, themes where supported and conformance fixtures from [the full adapter checklist](../research/app-adapters.md#checklist-shared-by-every-adapter). “Gap” means an existing source-derived concern or an integration not established by the survey. “Design” names additional functionality to support, not functionality already implemented. Concrete code edits require the operation audit above.

| App | Expose through app-owned contracts | Known gap or required integration | App-specific proof |
|---|---|---|---|
| EffectCraft | Composition/layer/selection queries, keyframes and effect parameters, effect evaluation, frame dependencies, render jobs and artifacts | Fresh CLI loader differs from desktop; missing effects can be omitted; process registration/update and pooled state need explicit lifecycle; preserve existing WASM/helper semantics | Independent layer states, seek/reset, update during export, cancelled render, missing required effect prevents success |
| FigCraft | Node/selection queries, geometry/layout edits, components, commands/panels/tools/widgets, file artifacts | Headless clears offered-file UI requests; disk modules need integrity continuity; same-ID replacement/settings and permissions need integration | Same native result through menu/panel/CLI; denied nested/file operation; undo/stale handles/panel close |
| PhotoCraft | Pixel/layer/mask/selection operations, filters, smart features, colour-aware data, import/export and themes | Bridge existing band/filter ABI and theme branch; failed/missing smart filter can be skipped; fresh CLI and exact smart-feature references need work | Alpha/colour modes, overlap seams, selected regions, smart save/reopen, theme fallback and explicit output failure |
| FilmCraft | Timeline/clip/track/selection edits, transitions, media inspection, export jobs; later frame/audio processing | No processing plugin host established by sampled sources; design temporal interface and independent theme mapping; preserve existing job completion behavior | Timebase/frame dependencies, timeline undo, awaited export, playhead/selection/highlight/contrast preserved |
| VectorCraft | Object/geometry queries and edits, units, live appearances, format/export handlers and resources | Preserve distinct object/live-effect/export contracts; connect release/state/retention and cached appearance evaluation | Malformed geometry rejected; units preserved; live effect save/reopen; output artifact completed |
| SoundCraft | Routing/project edits, prepare/process/reset, instruments/MIDI/automation, latency/tail, preset/state and editor lifecycle | Existing native formats are not sandbox proof; dedicated real-time backend and worker-failure semantics required | Deadline/latency/tail evidence, transport discontinuity, state restore, worker failure without unsafe callback behavior |
| LightCraft | Catalog queries and scoped metadata/develop edits, RAW/develop context, presets and export jobs | Define catalogue access scopes and recipe/live references; protected DB access must remain behind app operations | Nondestructive reopen, missing source media, export cancellation and selected-item scope enforcement |
| PDFCraft | Page/object queries and edits, annotations/forms where app supports them, validation/preflight and output | Survey establishes facade/tool table, not a plugin object model; add scoped handles and transactional file/output path | Stale page/object rejection, history, preflight result and completed PDF write |
| DesignCraft | Stories/frames/pages/spreads/styles, layout/reflow, pointer tools, prepress and export | Move UI-bound behavior into engine where audit proves duplication; define gesture/reflow transactions | Cancelled gesture, reflow undo, units, invalid layout rejection and prepress artifact |
| CADCraft | Entity/model queries, units/constraints where supported, explicit-parameter commands and tools | Interactive prompts need headless parameter contracts or explicit UI-only designation | Missing prompt returns InputRequired, unit-correct edit, rollback and model save |
| SolidCraft | Candidate model/entity commands, feature/rebuild operations, geometry and saved procedural state | Existing manifest listing does not establish executable host; implement actual runtime/transaction path before claiming support | Real command invocation, validated model mutation, missing procedural feature preserved and failed rebuild reported |
| TextureCraft | Mesh/UV/texture-set/channel queries and edits, brushes/strokes, material/bake/composite jobs | Resource/mesh lifetime, stroke transaction and cache ownership need adapter integration | Wrong texture-set rejected, cancel stroke/bake, update during lazy resource use, UV invalidation |
| WordCraft | Durable text ranges, stories/styles, structure/layout edits, validators and output | Define positions that survive or explicitly invalidate across edits; no raw persistent string offsets | Concurrent insertion/stale-range handling, undo, layout and exported document |
| GridCraft | Typed range/cell/workbook operations, import/export; dedicated formula/function extension contract | Custom functions need recalculation graph, cycles/error propagation, deterministic evaluation and budgets | Range edit undo, cycle/timeout, type errors, recalc invalidation and workbook save |
| DeckCraft | Slides/shapes/text, transitions, assets, presentation/export jobs and optional playback callbacks | Playback subscriptions have a different lifetime from commands; connect saved state/output contracts | Shape save/reopen/export, stop callback disposal, stale slide target and cancelled export |
| ArtCraft | Media/model/provider jobs, input/output roles, progress/cancel/result ingestion and workflow resources | Native target/provider jobs established, but plugin host/common CLI not established; egress/credential scopes needed | Provider restart/cancel, no credential disclosure, remote/offline readiness and durable result ingestion |

Candidate operations do not authorize exposing every internal action. App maintainers choose the supported contract, and denied side effects remain denied even when a plugin invokes a compound operation through parameters.

## Full contribution coverage

Beyond compute/editing, design versioned contributions for format-open/save associations, settings pages, context-menu/inspector sections, shortcuts/keymaps, localization packs and document validators/preflight. Headless-only operations use the same compiled command contract and can omit GUI placement. No interpreted script runtime or unrestricted startup macro system is selected.

Plugin-exported APIs and resident background services are named future families. They need bounded dependency graphs, origin-preserving delegation, lifecycle ownership and resource/credential policy before availability. This is deferred implementation, not omission from the architecture. Ordinary panels and declared jobs do not gain resident service privileges automatically.

## Common-base acceptance

At least EffectCraft and FigCraft must demonstrate that common descriptors, identity, errors, jobs/artifacts and policy can be reused while their data/transactions remain distinct. PhotoCraft media fixtures and FilmCraft timeline/theme fixtures test the limits of that commonality. No shared crate should be introduced solely because its name appears in this draft.

The [delivery plan](delivery-and-acceptance.md) orders these work packages. All app rows still need candidate-revision source audit and runtime acceptance; the present document does not claim every implementation change is already known or equally small.

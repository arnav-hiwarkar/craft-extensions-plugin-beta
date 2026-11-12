# Delivery, lifecycle procedures and acceptance

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: architecture proposal, 2026-11-08. This document completes the full design scope with operational procedures, realistic limits and phased implementation evidence. No implementation phase starts from this document alone. The community release gate stays closed, including for data-only community kinds unless a separate explicit policy change is agreed.

## Concrete draft choices

- Deliver the existing first-party PhotoCraft slice, then validate the expanded host model in EffectCraft and FigCraft in that order.
- Give app users access to the same shared management client through native in-app UI and launcher UI. No duplicate installer or required launcher daemon.
- Require explicit enablement and grants; offer CLI management with exact app/release/scope confirmation as a proposed technical-user path.
- Detect updates automatically, notify by default, and make automatic application an explicit preference. No review, authority or project-pinning bypass.
- Keep generous workload-based profiles with controlled higher ceilings and observable diagnostics. No arbitrary universal memory/time value is selected without measurement.

## Implementation order

| Unit | Concrete work | Evidence before advancing |
|---|---|---|
| D0: freeze contracts/policy | Confirm licence treatment, repository/operator, per-release review, registry authority, source/UUID aliases, state schema and grant continuity | Reviewed design, actual source definitions, fixtures specified; no unpublished infrastructure assumed |
| D1: store invariants | Explicit schema migration; retained manifests/digests/origin; no auto-enable; preserve app choices; multi-release records, pins and active-reader retention; exact revocation and read-side eligibility | Old/new writer fixture, local/cross-source collision, disabled update, crash/retry, release-host policy and concurrent read/update/remove tests |
| D2: SDK vertical slice | Registry-generated inventory, existing ABI wrappers, one author template and compiled deterministic package, generated release/review report | Real package check/pack repeated; schema/import rejection and author fixture work through actual adapter |
| D3: first-party PhotoCraft | Reconcile filter/theme branches and pinned dependencies, authenticated GitHub catalogue path, actual install/enable/grant/run/reopen/remove | Real theme/invert round trip, mapped appearance review, bounded processing and recovery in candidate |
| D4: EffectCraft | Desktop/fresh-CLI common eligibility, exact live effect references, independent instances, truthful export and retained jobs | GUI/CLI render parity, missing effect refusal, reset/seek, concurrent update during export |
| D5: FigCraft | Shared command registry/policy, explicit edit/read scopes, panel/CLI operation parity, output artifacts and lifecycle | Denied nested/file operation, undo/rollback, stale result, panel disposal and completed headless file |
| D6: wider contract proof | PhotoCraft colour/band fixtures, FilmCraft timeline/jobs/themes, additional families and per-app work packages | Literal domain results and compatible generated inventories; generalize only demonstrated common semantics |
| D7: releasable first-party candidate | Extract approved UI from the mixed showcase into clean source, clean-checkout pinned dependency resolution, packaging, platform/web and effective feature checks | Candidate source/revision, truthful evidence and regular implementation review; no release from showcase |
| D8: community acceptance | Human review/publishing, key custody/recovery, adversarial fixtures, operations, independent security review and required approvals | Complete sg13 prerequisites against actual candidate; explicit community enablement decision |

These are verifiable delivery units, not separate mandatory new repositories or one PR per table row. Cross-repo work is scoped to each app's maintainer and actual changes. Store and metadata compatibility must be agreed before pilots write new records. The existing first-party slice is complementary reuse evidence; it does not replace the selected EffectCraft-then-FigCraft functional proof.

## Management and author procedures

| Procedure | Normal sequence | Adverse outcome and recovery |
|---|---|---|
| Author/release | Create package, add app contributions, declare permissions, compile, test, pack, prepare release metadata, upload compiled GitHub assets | SDK explains unsupported contract/undeclared import/licence or version mismatch; no user-side source build or install hook |
| Catalogue admission/update | Human-reviewed exact release PR, generated change report, trusted metadata publication | Invalid provenance/changed digest/unknown publisher denied; upstream update remains undiscoverable until reviewed |
| Search | Cached verified metadata, query/app/kind filters, official precedence, dated repository-star ordering | Stale/unavailable statistics only affect ranking; metadata failure cannot grant official identity or new acquisition |
| Install | Download exact approved asset, verify archive, stage, validate, atomically commit complete record | Disk/full/busy/crash leaves previous state intact; verified retry/recovery handles orphan bytes without stealing active work |
| App setup | Inspect app contribution requirements, explicitly enable and grant required/optional scopes, configure | Denied optional rights disable only dependent operations; other apps remain unconfigured and disabled |
| Execute | Resolve eligibility, acquire retained bytes/resources, instantiate under host policy, run operation/evaluator/job, validate and commit | Unsupported/stale/denied/trapped/cancelled result is structured; no successful partial edit or implicit fallback export |
| Save/reopen/transfer | Store exact live references/state or native one-shot output; preserve missing features and offer a dependency plan | Opening never auto-installs/grants; ambiguous aliases/missing/revoked versions remain preserved; explicit upgrade or bake |
| Update | Stage reviewed release, compare source/rights/state, preserve app choices, set eligible default for new uses | Wider rights remain pending; current eligible default can stay available; active projects/jobs retain exact selection |
| Disable/uninstall | Stop new eligibility, dispose long-lived interactions safely, preserve project references/settings unless explicitly deleted | Byte cleanup waits for readers/pins; explain unknown external projects; completed native edits remain |
| Rollback/repair | Select an eligible retained release or verify exact replacement; recover coherent state with protected trust history | No rollback to revoked code, inferred grants or reset freshness; unreadable state preserved for diagnostics |
| Offline use | Resolve installed eligible resources under cached policy; keep app-local logs and history | Remote-dependent features report network requirement; no claim about unseen revocations; stale browse labelled |
| Incident | Receive report, investigate exact release/source, publish authenticated advisory/block/recovery, prioritize reviewed fixes | Blocking denies new calls and cancels at safe contract boundaries; no unreviewed emergency release bypass |

Settings pages and diagnostics inside each app use these same shared operations. Launcher deep links select the correct app/kind and native management context. If launcher installation is absent, in-app management remains available. Network code can be omitted from app targets that deliberately offer only installed/local management, but that is an advertised product/target choice rather than another state format.

## CLI management and execution

Proposed management intents are list/search/inspect, install/update, enable/disable, grant/revoke, pin/unpin, remove/repair and job/status/log inspection. Use existing app command naming conventions where possible; these are not claims of existing subcommands.

Install does not grant or enable. An explicit technical-user grant request must name app, exact reviewed/local release identity, normalized capabilities and scopes, and acknowledge the displayed authority summary. Interactive terminal confirmation is one option; an exact machine-readable consent record/fingerprint is a proposed automation path. A broad `--yes` must not silently approve an unknown permission set. Whether to offer noninteractive grants is an open product detail. Normal execution never prompts or grants. Plugin callers and unauthenticated remote clients cannot invoke grant-management operations.

Fresh processes, attached editors and persistent servers use the same resolver but different owners. The attached target session is authoritative for documents, store/profile and grants. Remote/local transports need authentication and scope before exposing automation, even if their command behavior is otherwise shared. Persistent jobs survive only under an explicit remaining owner; a one-shot process waits for required output.

Results contain stable status/error codes, contribution/release identity, job identity when durable, artifacts and remediation. Existing app numeric exit conventions are mapped to truthful success/non-success without breaking their callers accidentally. Do not invent one new global numeric mapping before auditing existing CLIs. No UI request may be discarded while required output is reported complete.

## Workload profiles and resource readiness

| Profile | Typical work | Limits to measure and expose |
|---|---|---|
| Interactive | Queries, commands, UI callbacks and tools | Response latency, transaction size, event/log rate, cancellation and total memory |
| Preview | Repeated image/geometry/frame evaluation | Quality/scale, memory and copy costs, dependency requests, reset, cancellation, fairness |
| Background/export | Large batches, full-resolution renders/imports and model jobs | Peak memory, streaming/tiling, disk staging, queued work and bounded completion/cancel |
| Real-time audio | Prepared blocks and events | Hard callback/deadline behavior, allocation/locking constraints, latency/tail and worker failure |

Every profile has per-instance and app-wide accounting. Raise a package/app ceiling explicitly within a host-supported maximum; a signed manifest cannot demand unlimited resources. Explain a budget failure with measured usage and a safe higher-profile option where supported. Changing a ceiling does not grant files/network or ignore hard engine limits. Concurrent work must still fit total host capacity. Numerical defaults and supported increases need candidate workloads/reference hardware before a release promise.

Large resources are separately declared data assets with exact digests, sizes, licence, importer/model contract and readiness purpose. The host downloads them explicitly, verifies them and retains them for readers. They are not runtime code installation. Keep code packages small enough for existing bounded archive admission. Use streaming/chunked data paths only when the actual consumer needs them, with total-size and hash verification and resumable download integrity.

GitHub currently documents a per-release-file limit under 2 GiB. Do not promise arbitrary single-file model sizes through that route. Split resources only with a specified whole/chunk inventory and total-disk policy, or require an explicit future approved-origin decision. The user's no-Craft-binary-hosting choice remains. [GitHub release quotas](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases#storage-and-bandwidth-quotas).

Readiness distinguishes installed module, missing/changed model, unsupported backend, insufficient grant, unavailable remote service and incompatible resource importer. A missing optional model leaves unrelated operations usable. Resource paths, network access and external workflows do not bypass the same user-consent policy.

## Acceptance evidence classes

Source inspection establishes structure at a recorded revision. A targeted runtime test proves only the invoked case. Candidate CI verifies pinned clean checkout, effective release features/platforms and required repository gates. Visual candidate review checks actual app appearance and interaction. Independent security assessment evaluates actual code and operating procedures. None substitutes for the others.

All acceptance cases below are future tests/exercises in approved implementation or CI. This design pass completed documentation/source checks only, recorded in [verification](verification.md). Current broad workspace Rust checks are not run here. Future craft-libs changes still need its required fmt/clippy/test gates through the applicable resource/CI policy.

## Coverage of every aspect

Each row maps the original [aspect register](../aspect-register.md) to a concrete part of this draft and the minimum adverse-case evidence. Full normal paths and responsible owners are in the [SDK](sdk-and-contracts.md), [security](security-and-distribution.md), [operations/app plan](shared-operations-and-apps.md), and procedure table above.

| Aspect | Draft contract/owner | Future acceptance |
|---|---|---|
| Vocabulary and identity | Core/store UUID plus source binding, readable handles, aliases and glossary | Local/cross-source UUID collision, ownership transfer and ambiguous legacy reference |
| Package and SDK | Registry inventory, Rust SDK, successor envelope and generated release report | Multi-app author journey, repeated packaging, unknown-critical rejection and denied import |
| Compatibility | Resolver host profile and per-contribution versions/requirements | Compatible/partial/unsupported/malformed package and engine/target mismatch |
| Dependencies | Compiled inventory plus explicit data readiness; inter-plugin graph family deferred | Missing/changed optional/required model, offline resource and future bounded cycle/conflict fixture |
| Catalogue | New reviewed metadata repo, shared client, filters and cached star statistics | Wrong-source entry, rate limit, stale search and exact approved release selection |
| Authenticity | Established update client, trusted root, approved digests and policy targets | Key recovery/rotation, replay/freeze/rollback, same-version metadata change and asset substitution |
| Provenance | Optional verified publisher attestations plus human change report | Subject/builder mismatch and honest unverified claim; no inferred benign behavior |
| Installation | Existing store extended with staged atomic publication and bounded admission | Path/link/index-allocation attack, disk exhaustion, lock race and crash/retry at every boundary |
| Activation | Per-app enablement/grants and shared load-side eligibility | No auto-enable, disabled update, missing grant, release app refusing gated development record |
| Updates and removal | Multi-release state, defaults, pins, reader ownership and collection | Update/remove/GC during export or lazy import; rollback without trust reset |
| Offline and time | Installed cached policy and explicit network requirements | Disconnected creation/export, known revocation, clock rollback and stale metadata |
| Proprietary packages | Separate product terms, optional restricted GitHub credential broker | Chosen local/catalogue licence policy, authentication denial and secret-free records |
| Operations | Named catalogue/review/key/app operators, incident/appeal/priority procedure | Report-to-revocation-to-recovery drill and review record for exact security-fix bytes |
| Sandbox | Engine profiles, platform table, typed dispatch and brokers | Malformed/forbidden module, host-call bypass, trapped/cancelled work on supported platforms |
| Capability grants | Host principal, normalized per-app/contribution scopes and continuity | Nested/queued/parameter escape, forged handle, widened scope and mid-run denial |
| Resource budgets | Interactive/preview/job/audio profiles and total accounting | Infinite guest, blocked host work, huge output/log/event stream and controlled higher ceiling |
| Documents and undo | App transactions, revisions and gesture lifecycle | Stale jobs/handles, invalid edits, reentrancy, undo/redo and cancellation rollback |
| Persistent features | Exact references, independent state, explicit migration/bake and preserved placeholders | Missing/revoked/updated reopen, failed migration, collaboration reference and two independent instances |
| Media processing | App-specific colour/alpha/precision/bands/time/units/state contracts | Full/band equivalence, temporal reset, colour tolerance, invalid shape and geometry units |
| UI contributions | Versioned declarations, native panels/tools/canvas plus format/settings/keymap/localization/validator families | Keyboard/accessibility, callback disposal, trusted prompt isolation and headless unavailable result |
| CLI and automation | Shared resolver, scoped management, authenticated sessions and artifacts | Fresh/attached/server parity, explicit grant separation, InputRequired and awaited output |
| GPU and native backends | Typed compute/worker contracts, platform-specific admission | Device loss/cancel/fallback and future confinement proof per OS before availability |
| Audio | SoundCraft prepare/process/reset/state/latency/tail contract | Dedicated deadline/automation/MIDI/worker-failure evidence before backend choice |
| Caches and scheduling | Digest/context keys, reset ownership and total scheduler accounting | Parameter/time/resource invalidation, instance isolation and measured interactive/export fairness |
| Themes | ThemeFile, WidgetPalette separation and app token mapping | PhotoCraft/FilmCraft selection/status/contrast/preview/fallback; future ThemeCraft round trip |
| Resources and workflows | Typed importers, declared data, graph readiness and separate executor | Hostile LUT/brush/SVG/template, missing node/model, no install-time code and denied executor action |
| Diagnostics | Structured results, app history, redaction, safe startup and quarantine | Useful GUI/CLI missing dependency, untrusted plugin logs, bounded offline history and opt-in support bundle |

The sixteen app rows in [the work packages](shared-operations-and-apps.md#per-app-work-packages) retain their own operation/data ownership. Candidate packaging, clean UI extraction, dependency pins, legacy migration, independent review and platform coverage appear in D0–D8. No row's presence means its implementation is finished.

## Established practices and simplicity

Use specific patterns backed by primary documentation, not claims that another product proves Craft's security.

| Source inspected | Pattern used in this design | Boundary of the comparison |
|---|---|---|
| [VS Code contribution points](https://code.visualstudio.com/api/references/contribution-points) | Declarative extension registration for commands, configuration, menus and other feature kinds | Does not establish Craft sandbox/capability guarantees or require copying its runtime |
| [Figma plugin manifest](https://developers.figma.com/docs/plugins/manifest/) | Explicit editor/API/document/network declarations and generated author setup | Its documented network/UI limitations must not be assumed equivalent to Craft's proposed brokers |
| [Zed extension development](https://zed.dev/docs/extensions/developing-extensions) | Rust/WASM extension tooling, feature manifest and explicitly labelled development installation | Does not prove existing Craft engines support its WASI/component target; no unverified WIT-backend assertion |
| [TUF specification](https://theupdateframework.github.io/specification/latest/) | Authenticated update state, root lifecycle and freshness/rollback rules | Does not certify package behavior, review quality or runtime enforcement |
| [GitHub releases/assets](https://docs.github.com/en/rest/releases/assets) | Publisher-hosted compiled versioned artifacts and optional provenance evidence | Hashes and reviewed policy remain necessary; availability is controlled by upstream |

Keep one store/state authority, one operation registry per app, one generated SDK inventory per contract version, one shared management implementation, and one reviewed catalogue publication path. Avoid new always-on daemons, binary hosting, build farms, a universal document model, duplicate schemas or a separate service for every function. Add catalogue sharding, alternative state storage and distributed workers only after measured scale or a real app need.

## Compatibility maintenance

Before a public SDK stability claim, publish supported contract versions, schema/runtime targets and a deprecation policy. Proposed starting policy: retain current and previous stable major contracts for at least one app-release overlap where safe, with migration guidance and explicit incompatibility diagnostics. Security-critical breaks can end unsafe support with a documented reason; this numerical support commitment needs maintainer confirmation rather than being inferred from this draft.

Generate compatibility docs and conformance coverage from the registry. Version fixtures and package state examples alongside the owning contract. Track migration of actual callers and remove obsolete internal APIs in the same scoped change. Prefer one clear implementation over compatibility glue with no supported consumer. Resource/event/operation behavior changes require new contracts where observable semantics change, even if JSON shape stays the same.

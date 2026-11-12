# Scenario and workflow coverage

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: discussion/reference checklist, not executed tests. Use it with each [topic](topics.md). The aim is one coherent state model that handles normal and adverse combinations, without a separate subsystem for every case. All expected behavior below is accepted direction or a clearly proposed engineering default; unresolved policy stays open.

## Dimensions to model

| Dimension | States to distinguish |
|---|---|
| Origin/authenticity | Official/community authenticated catalogue, unverified local file, labelled development, restricted asset; claimed versus verified identity |
| Contribution | One-shot command, live feature, long-lived UI/tool, processing job, theme/resource, external workflow/provider |
| App relationship | One app, several apps, partially supported bundle, unsupported app/contract |
| Installation | Absent, acquiring/staging, verified published release, interrupted/orphaned, changed/missing bytes |
| App eligibility | Disabled, enabled, missing required grant, optional grant denied, incompatible, resource/backend unavailable, eligible |
| Authority/policy | Same/narrower/wider rights, changed scope, current/revoked grant, advisory/block-new/emergency, quarantine |
| Version identity | Exact release, different same-source release, same version changed bytes, different publisher/source, legacy alias, transfer |
| Session | Fresh GUI/CLI, attached editor, persistent server, worker/job; authenticated and scoped versus unknown caller |
| Project | New native edit, exact live reference/state, missing/unknown feature, explicit migration/bake, shared/transferred project |
| Connectivity/time | Online/fresh, offline/cached, expired metadata, unseen versus known revocation, suspect clock rollback |
| Concurrency | Two app readers, simultaneous installers, update/remove/GC during a job, stale document, crash/restart |
| Execution/data | CPU/WASM, host GPU, native/remote/audio backend, malformed/oversized input, device loss, deadline/budget/cancel |

Some combinations are invalid by construction: a UI-only panel cannot report a successful headless computation, an absent package cannot be eligible, and a manifest cannot grant itself authority. Define those constraints explicitly.

After structures are agreed, derive transition tests from the model. Cover independent pairs plus targeted multi-step/three-or-more-factor cases at the dangerous boundaries below. Pairwise coverage alone is not proof that update/revocation/concurrency sequences are safe. No finite hand-written list proves every possible plugin behavior; new real requirements add cases against the same invariant rather than ad hoc flags.

## Required combinations

| ID | Situation | Expected behavior/invariant to settle | Main topics |
|---|---|---|---|
| S01 | One package supports PhotoCraft and FigCraft; only PhotoCraft is configured | Install bytes once; grants, enablement, settings and active contributions stay app-specific | T01, T03, T05, T08 |
| S02 | Package offers supported and unsupported contributions | Show compatibility per contribution; reject unknown critical package semantics, do not silently reinterpret them | T02, T07 |
| S03 | Plugin or local file claims a builtin/foreign UUID, operation or readable name | Names cannot acquire core identity or another publisher's grants/update stream; explicit origin-aware selection | T01, T07, T09 |
| S04 | Two exporters both write `.png` | Distinct IDs and explicit selection; builtin/default resolution stays authoritative until user changes it | T01, T11, T12 |
| S05 | Extension shortcut conflicts with a core shortcut | No installation-time steal; display conflict and offer deliberate user assignment | T01, T11, T12 |
| S06 | Same algorithm targets PhotoCraft bands and EffectCraft temporal frames | Reuse algorithm where valid; adapt colour/alpha/overlap/time/state through purpose contracts | T02, T03, T06 |
| S07 | Guest requests an operation through a nested/queued CLI-like dispatch | Preserve originating plugin identity and scopes; no elevation to trusted-user authority | T04, T05, T15 |
| S08 | Benign-looking command parameter reads a file or uploads content | Parameter-dependent rights checked at the actual operation boundary; no hidden side effects | T04, T05, T06 |
| S09 | Required rights denied; optional network right denied | Required operation unavailable with reason; unrelated offline operations remain usable | T05, T11, T16 |
| S10 | Update keeps or narrows rights, then another update widens them | Verified same-source normalized continuity controls reuse; widened scopes require explicit renewed app consent | T05, T07, T08 |
| S11 | New contribution is added to a previously granted package | Do not inherit another contribution/app's authority implicitly; present new requirements | T03, T05, T08 |
| S12 | Repeated install/retry after a crash between bytes and state publication | Coherent state; verified idempotent reuse or safe quarantine; do not delete another live installer's data | T08 |
| S13 | Launcher updates/removes while a long CLI export lazily reads a LUT | Pin exact executable and resource access until the reader is finished; collection cannot rely only on age | T08, T13, T15, T16 |
| S14 | Old client writes after a new authority-bearing state migration | Refuse unsupported writing; preserve settings/data and explain newer-client requirement | T07, T08 |
| S15 | Publisher replaces bytes under the same version/tag/filename | Authenticated exact approved identity detects mismatch; do not silently accept a new hash | T07, T09, T10 |
| S16 | Publisher uploads a new release but catalogue PR is not approved | It does not become a discovered/automatic update through `latest` | T09, T10, T11 |
| S17 | Signing authority changes, metadata is replayed, or time goes backward | Established trust/root/version/freshness recovery; do not reset known restrictions | T10, T16 |
| S18 | Publisher/repository ownership transfers | Reviewed identity transition; preserve readable references but do not silently inherit user consent | T05, T07, T09, T10 |
| S19 | Repo stars are stale/manipulated/missing; repository hosts multiple packages | Advisory dated repository-star semantics; stable useful ranking; no effect on safety, identity or grants | T09, T11 |
| S20 | Public versus private GitHub assets and cross-origin redirects | Public path has no added account; optional credentials stay at authorized origin and out of logs/redirect forwarding | T09, T10, T17 |
| S21 | Local file has no catalogue review or signature | Useful local installation with truthful provenance and the same runtime/grant policy; different bytes have no inferred publisher continuity | T05, T07, T09 |
| S22 | Known malicious digest is imported as a local file while offline | Origin relabelling cannot evade cached digest blocks; unseen revocation cannot be claimed known | T06, T10, T16 |
| S23 | Development client and release client share store contents | Reading build applies its own admission policy; development identity never launders a gated record into release use | T06, T08, T10 |
| S24 | Reopen on another machine with a missing or incompatible live effect | Preserve bounded state/placeholder; offer a deliberate dependency plan; no downloads or new grants on open | T07, T13, T15 |
| S25 | Explicit migration fails or rollback state is incompatible | Work on a copy; preserve old state and selection; no silent backwards migration or revoked-release rollback | T08, T13 |
| S26 | Preview can fall back but final export requires unavailable processing | Label preview fallback; final output cannot report success with silently omitted processing | T13, T15 |
| S27 | Plugin job completes after the document/selection changed | Revision/handle validation; reject or explicitly reapply; do not overwrite newer work | T04, T12, T13, T15 |
| S28 | Undo follows a plugin edit that also wrote a file or uploaded data | Document transaction has clear undo; external effects are separately authorized and not falsely described as reversible | T04, T05, T13 |
| S29 | Panel closes during callbacks, tool loses capture, widget is missing on reopen | Dispose subscriptions/handles, bounded event delivery, commit/cancel gesture correctly, preserve saved instance state | T12, T13 |
| S30 | UI-only contribution is invoked headlessly; job owner exits too early | Structured unavailable/input-required result; required artifacts are awaited under a remaining owner | T12, T15 |
| S31 | Malformed brush/LUT/SVG/template, bad workflow or missing model/node | Typed bounded import/readiness; nothing executes on install; execution requires separate supported contract | T06, T14 |
| S32 | Theme passes package validation but loses contrast/selection meaning in an app | App token checks, deliberate activation and built-in fallback; trusted permission UI unaffected | T11, T14 |
| S33 | Huge model/resource, disk full, partial download or resource removed during import | Explicit total size/digest/licence/storage policy, recoverable acquisition and retained reader access | T08, T09, T14, T16 |
| S34 | Temporal processor seeks backward or two exports share a pool | Explicit reset/instance isolation and digest/context cache keys; no state cross-contamination | T02, T06, T13, T16 |
| S35 | GPU device loss, arbitrary shader request or native helper on unsupported OS | Advertised backend contract, truthful failure/fallback, confinement proof before availability; no implicit unrestricted backend | T02, T06, T16 |
| S36 | Audio callback deadline fails or plugin worker crashes | Dedicated real-time contract, no blocking UI/allocation/log work on callback, explicit silence/failure/recovery policy | T02, T06, T16 |
| S37 | Guest loops, host call blocks, output/events/logs grow without bound | Per-workload and total accounting, host cancellation, bounded output; controlled user raises cannot disable hard protections | T06, T12, T16 |
| S38 | Corrupt state, repeated startup failure, quarantine and opt-in support report | Safe startup preserving projects; repair does not reset trusted history; private bounded app-local diagnostics | T08, T10, T16 |
| S39 | Restricted/proprietary licence or bundled dependency conflicts with selected policy | Distinguish catalogue/local/product policy from repository dependency/asset rules; no guessed exemption or new licence approval | T09, T17 |
| S40 | Two plugins want to call each other or run a resident service | Named future dependency/delegation/lifetime contract, not implicit authority through existing job/panel APIs | T03, T05, T06, T17 |

## Four actor workflows

| Actor | Normal journey | Questions to carry through every relevant topic |
|---|---|---|
| End user | Discover -> inspect -> install -> set up current app/consent -> use -> save/reopen -> update -> disable/remove/repair | What do I see, what choices are needed, what happens offline, what data/work survives, how do I understand or fix a failure? |
| Plugin maker | Create -> add app contributions -> implement typed behavior -> test/dev -> compile/check/pack -> publish own GitHub asset -> submit metadata -> respond to review -> maintain versions | What do I write versus generate, what is supported, how do I debug, how are rights/resources/state declared, what remains stable across apps/releases? |
| Catalogue maintainer | Enrol -> validate exact release/change report -> human review -> publish authenticated metadata -> refresh advisory stats -> receive reports -> revoke/recover/transfer | Who is accountable, what did we actually verify, how do keys/review/appeals work, what cost and tooling are required, what is logged privately? |
| App/common developer | Audit existing definitions -> choose exposed operation -> add descriptor/adapter -> enforce caller scope -> integrate history/jobs/state/CLI/UI -> prove conformance -> release compatible host -> maintain | Which code is reused, which domain types stay local, how are core features protected, what tests prove the actual behavior, and how do old clients/projects migrate? |

These workflows share components and evidence. They are not four independent managers. For each design decision, trace both its normal experience and its failure/recovery experience across affected actors.

## Twenty-seven-aspect crosswalk

| Aspect | Topics |
|---|---|
| Vocabulary and identity | T01, T07 |
| Package and SDK | T03, T07, T09 |
| Compatibility | T02, T07, T17 |
| Dependencies | T03, T14, T17 |
| Catalogue | T09, T11 |
| Authenticity | T09, T10 |
| Provenance | T09, T10, T17 |
| Installation | T08, T09, T11 |
| Activation | T05, T08, T11 |
| Updates and removal | T07, T08, T13 |
| Offline and time | T10, T16 |
| Proprietary packages | T09, T17 |
| Operations | T10, T17, T18 |
| Sandbox | T05, T06 |
| Capability grants | T04, T05, T06 |
| Resource budgets | T06, T16 |
| Documents and undo | T04, T12, T13 |
| Persistent features | T07, T13 |
| Media processing | T02, T06, T13 |
| UI contributions | T03, T11, T12 |
| CLI and automation | T04, T15 |
| GPU and native backends | T02, T06, T16 |
| Audio | T02, T06, T16 |
| Caches and scheduling | T13, T16 |
| Themes | T11, T14 |
| Resources and workflows | T02, T14 |
| Diagnostics | T11, T15, T16 |

Delivery/ownership, clean candidate/dependency pinning, source/runtime/CI/visual/security evidence and the unchanged gate are discussed in T18. Every current app has a row in [the family map](app-families-and-sdk.md#all-sixteen-apps). Scenario IDs are discussion anchors, not claims that tests exist or passed.

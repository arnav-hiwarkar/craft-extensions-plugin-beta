# Plugin and add-on design aspect register

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](architecture.md); where this document disagrees with it, the architecture wins.

This is a research checklist, not an approved implementation plan. Every aspect needs a concrete normal path, failure path, responsible owner, proposed contract, alternatives, recommendation, user decision where material, and observable acceptance evidence. Preserve the accepted defaults in [the decision record](../plugin-functional-decisions.md).

## Shared management and distribution

| Aspect | Questions and failure paths to resolve | Required design evidence |
|---|---|---|
| Vocabulary and identity | Package, contribution, instance, publisher, source, runtime, and resource must have distinct meanings. Handle duplicate IDs, namespacing, renamed publishers, and source collisions. | Example identities across official, community, and local files without accidental replacement |
| Package and SDK | Required author metadata, generated inventory, schema negotiation, multiple contributions, platform variants, reproducible packaging, licensing, documentation, and examples | Small author journey, complete generated artifact, and old-client rejection behavior |
| Compatibility | App version, purpose-specific interface, runtime ABI, OS, architecture, hardware, and optional features | Resolver results for compatible, unsupported, partially compatible, and malformed bundles |
| Dependencies | Shared resources, optional models, plugin dependencies, conflicts, cycles, private sources, offline availability, and recursive bounds | Bounded resolution with deterministic errors. Decide whether executable inter-plugin dependencies are needed initially |
| Catalogue | Owner, index administration, source addition/removal, fetch caps, caching, throttling, URL restrictions, search, ranking, and truthful labels | Signed fixture catalogue and rejected wrong-source entries. No assumption that crafthub is approved |
| Authenticity | Publisher enrolment, reserved IDs, signing, metadata freshness, rotation, compromised keys, revocation, and emergency recovery | Trusted root lifecycle and replay/rollback/freeze examples, including equal-version changed content |
| Provenance | Publisher build versus hub build, source commit, attestations, private first-party code, rebuild costs, and build-service compromise | Costed alternatives and honest limits of what each attestation proves |
| Installation | Verify before unpack, archive bounds, path/link escape, hostile parsers, disk exhaustion, interrupted staging, locking, and cleanup | Transaction state diagram and restart/retry outcomes at each write boundary |
| Activation | Installed, enabled, compatible, permitted, quarantined, revoked, missing resources, and active-version selection | Eligibility rules shared by GUI, CLI, load, and enable paths |
| Updates and removal | Active jobs, retained versions, project references, state migration, rollback, changed authority, uninstall, repair, and garbage collection | Concurrent app/job examples and recovery without deleting referenced bytes or silent state loss |
| Offline and time | Expired metadata, known revocation, clock rollback, disconnected creation/export, and cached dependencies | Explicit policy for new installation versus continuing existing projects |
| Proprietary packages | Public versus restricted access, optional entitlements, offline licensing, credential storage, and code confidentiality limits | Separate download, execution, and entitlement policies without assuming paid distribution |
| Operations | Disclosure, takedowns, appeal, maintainer review, malicious updates, support diagnostics, and privacy | Incident exercise and documented responsibility for store, source, publisher, and host |

## Execution and application integration

| Aspect | Questions and failure paths to resolve | Required design evidence |
|---|---|---|
| Sandbox | Module admission, imports, filesystem/network denial, host-call validation, compilation limits, crashes, cancellation, and process boundaries | Per-platform enforcement table with unimplemented protections named |
| Capability grants | Per-app rights, read/write scope, selection/document limits, delegation, revocation mid-run, and permission changes | A denied operation cannot be reached through another command or hidden parameter |
| Resource budgets | Memory, fuel, elapsed time, host work, output, logs, events, subscriptions, parallel jobs, and caches | Bounded operation and cancellation semantics across plugin and host work |
| Documents and undo | Validated edits, transactions, selection, events, stale document handles, reentrancy, partial failures, and concurrent edits | Failed command rollback and explicit ownership of long-lived interactions |
| Persistent features | Contribution references, exact releases, parameters, instance state, migrations, missing dependencies, baking, and collaboration | Save/reopen/export with missing, revoked, incompatible, and updated plugins |
| Media processing | Color spaces, alpha, precision, image bands, overlap, timeline time, frame dependencies, geometry units, and temporal state | Purpose-specific input/output contracts and literal equivalence fixtures |
| UI contributions | Commands, menus, declarative panels, widgets, localization, accessibility, event disposal, and headless limitations | Same useful operation through panel and CLI, with explicit UI-only exceptions |
| CLI and automation | Fresh process, attached editor, persistent server, eligible installed plugins, consent-free denial, output destinations, jobs, and exit status | Fresh CLI uses installed eligibility and waits for required output without hidden dialogs |
| GPU and native backends | Brokered compute versus isolated helper, driver risk, transfer overhead, hardware fallback, device loss, and platform enforcement | Feasibility experiments proposed separately. No claim that a process or GPU queue is a sandbox |
| Audio | Real-time allocation/locking limits, latency, tail, transport, MIDI, state, editors, deadlines, and worker failures | Dedicated SoundCraft contract and latency evidence before choosing a runtime |
| Caches and scheduling | Identity keys, parameter/context dependencies, state reset, pooling, invalidation, cancellation, fairness, and deterministic export | Cache correctness and measured bottleneck targets, not guessed optimizations |
| Themes | Packaged ThemeFile, shared WidgetPalette, app-token allowlists, contrast, preview, fallback, active preference, and future ThemeCraft round trips | PhotoCraft and FilmCraft mapping preserving status, selection, timeline, and accessibility meaning |
| Resources and workflows | Existing swatch/LUT/brush/template importers, hostile data, resource versions, models, ComfyUI JSON, missing nodes, and external execution | Data installation never silently runs code. Workflow execution has a separate explicit contract |
| Diagnostics | Error taxonomy, structured output, redacted logs, privacy, crash isolation, safe mode, and repair | Useful missing-dependency and quarantine errors in GUI and headless callers |

## Delivery and ownership

Each app needs an adapter checklist covering discovery, compatibility, activation, runtime loading, contributions, validated commands, transactions, persistence, job ownership, CLI bootstrap, output artifacts, errors, theme mapping, and conformance fixtures. The existing app survey is a starting point, not a promise that every integration is small.

Resolve the rollout relationship between the existing PhotoCraft invert/theme slice and the broader EffectCraft/FigCraft functional pilots. Keep first-party shipping evidence separate from community enablement evidence. Gate sg13 remains closed until its own prerequisites and explicit approvals pass.

Plan clean dependency pinning, reproducible checkout, release-feature verification, independent security review, platform coverage, UI extraction from the combined showcase branch, migration of old plugin IDs, and candidate packaging. Do not duplicate the existing store or turn this research into unapproved implementation.

Every proposed acceptance check must state whether it is source inspection, a test to write later, a completed local check, or CI-only evidence. Explain which minimal experiment would settle an uncertain design, and obtain implementation scope before building it.

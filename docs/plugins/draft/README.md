# Complete extension system architecture draft

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: design draft, 2026-11-08. This applies the user's [latest decisions](decisions.md) to the earlier [research](../research/README.md). Accepted direction and proposed mechanisms are distinguished below. No implementation, new repository, dependency, publication, release or community gate enablement is authorized. Organizations remain out of scope.

The later [readiness investigation](../readiness/assessment-and-first-work.md) checks actual app/store gaps and prioritizes the development work. Read it alongside this target architecture; source limitations and corrections there refine implementation planning without discarding accepted decisions.

## The intended experience

A person opens Extensions inside PhotoCraft, searches for a package, sees official entries first and community matches ranked using GitHub stars, and opens its details. The details describe what the package provides in PhotoCraft, which other apps it supports, required permissions, compatibility, licence, review status, download size and network requirements.

Install downloads an approved compiled package from its publisher's GitHub Release. Verification completes before installation is committed. The person explicitly enables its PhotoCraft contributions and allows the requested scopes. The same installed bytes can serve FigCraft, but FigCraft setup, grants and settings are separate. The package cannot activate itself in other apps.

The feature appears in the expected app location: a filter, command, exporter, panel or tool. A long operation shows progress and cancellation. Saving preserves native edits or exact live-feature references as appropriate. Reopening explains missing requirements and offers a deliberate repair plan. A fresh CLI uses the same eligible installation and app operation, with explicit output and no hidden dialogs.

Updating changes the default for new uses after review and eligibility checks. Existing projects and running work retain their chosen release until an explicit upgrade or policy event. Local package files work without catalogue review, under the same validation, sandbox and explicit-grant rules. Offline-capable work continues without a catalogue connection.

This describes target behavior, not a completed system. Earlier source reviews found missing identity records, load-time policy checks, CLI bootstrap and final-output handling. Those require scoped implementation in the owning repositories.

## Read the draft

| Document                                                           | Design responsibility                                                                                  |
| ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------ |
| [Decisions and open details](decisions.md)                         | Exact user direction, superseded alternatives and engineering proposals                                |
| [SDK and contracts](sdk-and-contracts.md)                          | Author journey, package format, operation model, permissions, UI and processing contracts              |
| [Security and GitHub distribution](security-and-distribution.md)   | Reviewed catalogue, release authentication, local installation, runtime boundaries, state and recovery |
| [Shared operations and app changes](shared-operations-and-apps.md) | Common-library boundaries, per-app discovery/fixes and adapter obligations                             |
| [Delivery and acceptance](delivery-and-acceptance.md)              | Full lifecycle procedures, workload profiles, maintenance and evidence required to ship                |
| [Independent Opus design review](opus-review.md)                   | Preserved independent architecture proposals, objections and alternatives                              |
| [Review reconciliation](review-reconciliation.md)                  | Primary assessment and corrections after the independent review                                        |
| [Evidence and verification](verification.md)                       | Source limits, official patterns consulted and rerunnable documentation validation                     |

Detailed earlier [lifecycle workflows](../research/lifecycle.md), [sixteen-app adapter matrix](../research/app-adapters.md), and [twenty-seven-aspect coverage](../research/coverage.md) remain supporting references. The current decisions override earlier catalogue hosting, hub-build and ranking recommendations.

## System ownership

| Part | Owns | Does not own |
|---|---|---|
| craft-ext-core in craft-extensions | Identity, manifests, compatibility declarations, package/contribution types and pure policy rules | App document objects or network credentials |
| craft-ext-store in craft-extensions | Installed releases, coherent state, activation preferences, grant records, revocation cache, resolution and retention | Application edit/render semantics |
| Distribution client | Authenticated metadata, bounded GitHub download, cache, source provenance and verified-artifact creation | Authorizing requested runtime permissions |
| Shared operation support | Typed descriptors, caller context, parameters, effects classification, result/job/artifact envelopes and generated tooling | A universal document model or unchecked command dispatcher |
| App engine and adapter | Actual queries/edits/evaluation, scope enforcement, undo, persistence, job integration, CLI bootstrap and theme mapping | Reimplementing package installation or trusting manifest claims as policy |
| SDK | Plugin authoring, generated bindings/metadata, packaging, fixtures, diagnostics and release preparation | Trusting its own output on behalf of the host |
| Manager UI | Discovery, app-specific setup/grants, progress, dependencies, update history and recovery explanations | Independent copies of resolver or permission logic |
| Catalogue maintainers | Publisher enrolment, exact-release review, signed catalogue publication, incident/revocation decisions | Hosting publisher plugin binaries or granting user authority |

Keep existing core/store code in craft-extensions. Any future common operation libraries in craft-libs must have at least two real consumers, inherit workspace rules and depend on no app. Responsibilities in this table are not a proposed crate-per-row split. Existing shared UI, image, geometry and command utilities should be reused where their definitions fit.

## The common path for plugin and CLI functionality

The app engine provides a typed operation with parameter validation, target resolution, effect classification, transaction or job behavior, and a typed result. The GUI, CLI and plugin adapter call that operation through their respective caller contexts.

```text
GUI action / CLI request / plugin host call
  -> resolve operation and validate parameters
  -> apply caller identity, allowed effects, scopes and current policy
  -> execute app-owned query, transaction, evaluator or job
  -> validate result, commit document or artifact where appropriate
  -> return result / progress / structured failure
```

This common behavior is the user's intended reuse opportunity. Sharing a function does not give every caller equal rights. A plugin receives the intersection of declared permissions, explicit grants, app capabilities, current scope and live policy. An attached CLI uses the target app session's authority; an external automation connection also needs session authentication and explicit access policy.

Operations such as project inspection, selection query, validated edits, parameter validation, job cancellation and artifact delivery can share mechanisms. PhotoCraft pixel data, FilmCraft timeline semantics, WordCraft text positions and GridCraft recalculation retain domain-specific contracts. Promote code when actual semantics agree rather than forcing all engines into a universal command bus.

## Full scope without an unrestricted execution promise

The contract catalogue covers commands, queries, tools, declarative panels, canvas widgets, live features, image/video/geometry/audio processing, import/export, resources, workflows and provider jobs. App adapters advertise exactly which versions are implemented. Authors can inspect this before installation or invocation.

The complete architecture includes brokered files/network/accounts, GPU operations and native/remote workers as explicit backend classes. A class becomes available only when the implementation demonstrates its declared boundary. WASM does not make a blocking host operation bounded automatically; a helper process does not itself prevent file access. “Full functionality” therefore means a designed path for real needs with enforcement, not a global allow-all permission.

Ordinary compiled plugins receive no shell, process spawning, arbitrary filesystem, ambient account credentials or runtime dependency installation. A supported host operation may offer powerful file export or model computation using scoped host-owned handles. New rights follow the same declaration, explicit grant, enforcement and review process as initial ones.

## Important records

All names below are design sketches, not implemented APIs.

| Record | Required meaning |
|---|---|
| Package identity | Stable package UUID, publisher/source continuity, human-readable name and legacy aliases |
| Release | Version plus exact archive digest, retained verified manifest/inventory, origin and review reference |
| Contribution | Stable contribution ID, app/contract versions, module/resources, authority and headless/UI declarations |
| App installation preference | Enablement, configuration and selected new-use defaults for this app |
| Grant | Explicitly approved capability/scope for this app/package, authority contract and verified continuity |
| Project reference | Exact release/contribution, parameters, independent state schema/data and optional labelled baked result |
| Runtime identity | Host-created principal, scopes, current policy generation and bounded invocation context |
| Job | Owner/session, pinned release/resource handles, captured document revision, progress, cancellation and terminal result |
| Review record | Human approval bound to exact release bytes and relevant contracts/permissions/licences |

UUIDs identify packages; they do not authenticate publishers. GitHub repository identity and release asset locators aid continuity; trusted catalogue metadata binds approved bytes. Presentation names, stars and uploaded manifest fields cannot create official identity.

Use “extension” as the UI umbrella, “package” as the install/update unit, “contribution” as one app feature, and “plugin” for executable extension functionality. Add-on is the user's broader synonym. A saved feature instance, runtime instance and installed release have different owners and lifetimes.

Concrete draft defaults after review: one app-owned Rust registry produces the authoritative operation inventory; typed wrappers converge on bounded policy-checked dispatch; craft-ext-store initially keeps one atomic versioned state snapshot plus immutable bytes/cache; its release records are source-bound and multi-version. The catalogue is one authenticated index delivered through established update tooling. App management UI can invoke the shared client directly without duplicating it or requiring a launcher daemon.

## Lifecycle invariants

1. Catalogue/search listing never executes plugin code. Official priority and popularity never bypass eligibility or grants.
2. Reviewed catalogue metadata identifies exact approved assets. The client never installs arbitrary `latest` releases just because the package was listed once.
3. Local installation needs no catalogue submission or account, but cannot bypass runtime policy.
4. No installation activates a theme or grants a new right implicitly. App-specific choices survive updates.
5. Module admission and loading use the actual engine policy and verified bytes; granted authority remains scoped at every host operation.
6. A document edit commits through its owning transaction system. Cancelled, stale or invalid results do not partially mutate it.
7. Live features preserve exact references/state. Missing required processing does not become a successful final export.
8. Updates/removal retain modules and lazy resources while active readers need them. Project pins and rollback policy are visible.
9. Offline-capable installed work needs no routine internet check. Known restrictions remain effective; unseen revocations cannot be promised offline.
10. Logs and reports are useful, bounded and redacted by default. Plugin-generated log content is untrusted and cannot impersonate a verified manager event.
11. State migration and repair cannot reset trusted freshness/revocation restrictions or infer grants from installed folders.
12. Unsupported backend features are reported accurately. Release builds enforce their own community admission gate even when sharing a development store.

## Long-term maintenance

Stability comes from generated contract bindings, explicit versions and migration tests, shared executable conformance fixtures, immutable release references, and narrow app-owned adapters. A service split or microservice fleet is not needed for the first version. GitHub hosts metadata and release assets; local apps own execution.

Contract changes add explicit versions and deprecation windows tied to real consumers. Package schema, SDK, app contract, app version, runtime profile and project state evolve separately. A machine-readable operation inventory should generate SDK reference pages, CLI discovery and adapter coverage, so documentation is checked against the same descriptions used by the app.

Do not promise perpetual API support before the initial pilot contracts prove useful. Support policy and numerical compatibility windows need maintainer ownership, app release cadence and actual implementation evidence.

## Open choices and proof

The exact licence allowlist, official proprietary terms, repository name/maintainers, confirmation of per-release review cadence, future backend encodings, native confinement and numerical budgets remain draft details. The Rust registry/inventory is the recommended interface authority now. See [decisions](decisions.md#remaining-decisions). Current named-revision observations and verification limits remain in [the research evidence record](../research/verification.md).

The requested Opus review is preserved and [reconciled](review-reconciliation.md). Linked procedures, per-app obligations and [acceptance coverage](delivery-and-acceptance.md) describe the complete proposed scope. No source reading, document validator or human design review certifies a running sandbox. Implementation and release evidence remain future scoped work.

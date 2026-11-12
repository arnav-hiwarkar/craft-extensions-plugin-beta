# Proposed lifecycle workflows

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: design reference, 2026-11-08. All paths and acceptance cases below are proposed, unless identified as existing source behavior. No application acceptance test ran. Contract names and data shapes are illustrative. Read [the design](design.md) for recommendations and [the decisions](decisions.md) for unresolved choices.

## Author a release

1. The author chooses an app contract, such as an EffectCraft effect or FigCraft command, and generates SDK bindings for that contract.
2. The author supplies stable package/contribution IDs, version, licence, required capabilities, parameters, state schema, resources and examples.
3. Tooling builds an executable with the chosen toolchain and validates its imports/exports against the selected contract. This describes future tooling, not a build performed here.
4. Tooling derives the inventory, sizes and digests, packages the existing ZIP envelope with a negotiated manifest schema, and checks literal input/output fixtures.
5. The author tests GUI and headless behavior through the app's adapter harness, including denied calls and cancellation. A UI-only contribution declares that limit.
6. The publisher submits immutable bytes and provenance to a source. Source tooling handles authentication; authors do not hand-edit signature structures.

An unchanged version with changed bytes is a conflicting release, not a silent update. Unknown critical schema requirements, excess size, forbidden imports, missing licence, ambiguous IDs or absent required resources fail with an actionable diagnostic. Platform variants declare their OS, architecture, runtime and hardware requirements. Reproducible packaging requires fixed ordering and normalized metadata plus repeated artifact comparison; it is a check to design, not an existing guarantee.

Owner: SDK tooling plus the app contract owner. Compatibility follows the targeted schema and ABI; a new SDK must still emit explicitly supported old contracts or report a required upgrade. Performance evidence should measure author validation and package size before adding incremental build machinery. Future acceptance: generate a minimal filter, reject a forbidden import, produce an identical archive twice, and run an example through the real adapter.

## Enrol a source and discover packages

The person explicitly adds a source or uses the configured official source. The client establishes its trusted root and stable enrolled identity, authenticates bounded metadata, records freshness/version state, and caches searchable presentation data. Browsing never runs a package. Results show verified publisher/source labels separately from unverified claims, contribution compatibility, download size, required rights, and licence. Search order is explainable; initial ranking does not require app telemetry.

A redirected or lookalike URL does not acquire the previous source's authority. Key rotation follows the authenticated chain. Invalid, expired, oversized or older metadata cannot authorize acquisition. Source removal stops fetching while preserving installed identity. Throttling and network failure keep cached inspection available and state the last verified time. Private-source credentials stay in the host credential store and never enter package manifests or project references.

Owner: launcher/source client, using core metadata contracts. Catalogue location, moderation and source-enrolment UI remain decisions. Future acceptance: signed fixture metadata, wrong-source payload, expired timestamp, root rotation, revoked key, replay, equal-version changed metadata, bounded pagination and offline cached search. Tests must use the selected protocol rather than a sketch of signatures.

## Install without activating new authority

| Step | Proposed state and owner | Failure/restart behavior |
|---|---|---|
| Preflight | Source client identifies exact release, required schema, archive size and expected digest | No store mutation on unsupported host, bad metadata, unknown source, inadequate space or denied policy |
| Acquire | Client downloads with byte/time/redirect limits or reads a bounded local file | Partial content is never installed; retry can reuse only verified cached content |
| Admit | Common validator compares expected identity/digest, archive inventory, paths, links and bounded decoded data | Reject before publishing. Archive indexing allocations need preflight limits, not only post-index entry counts |
| Stage | Store creates a private staging directory; engine profile validates executable modules without running plugin hooks | Failure removes this operation's staging data; preserve unrelated active staging directories |
| Publish bytes | Under the existing writer lock, recheck current state and place immutable release files | A same-digest existing directory is verified and reused. Conflicting bytes fail; no blind overwrite |
| Commit record | Atomically select a complete state/authority generation referencing existing verified bytes | Crash before commit leaves an orphan; crash after commit leaves a complete selected release |
| Expose | Apps observe generation, refresh metadata and report eligibility | New rights remain ungranted. Theme activation remains a separate choice |

There is no executable install hook. A bounded data preview uses the same importer restrictions as real use. Local-file authorship can remain unverified but runtime policy is mandatory. First-party source identity must come from trusted acquisition, not a UI-supplied tier string.

Restart recovery examines the publication boundary, not merely file age. A stale staging directory can be removed only after proving it has no active owner. A committed release is not deleted because the previous caller failed to receive success. StoreFull, Busy, InvalidArchive, UnsupportedSchema and UntrustedSource are distinct outcomes. Repeated installation of the same exact release converges to the same state.

Future acceptance: inject failure before and after every boundary, exhaust disk during acquisition/staging/state write, race two installers, retry after an orphan directory, reject path/link escapes and excessive indexing allocation, and verify the old installation still works. Existing bounded unpack and lock code is a baseline, not proof these crash cases pass.

## Enable and grant

The manager shows contributions per app and the rights each needs. Enabling changes preference; granting changes scoped authority. The resolver validates this build's admission policy, compatibility, current revocation, byte identity and resources before creating any runtime instance. It returns all actionable blockers without executing module startup merely to list contributions.

An update preserves disabled state and existing grants. New app entries start pending or disabled. Increased requested authority blocks the affected release until the person approves it. Denial leaves installation intact. GUI and CLI inspect the same reasons. A fresh CLI reports MissingGrant or Disabled rather than prompt. An explicitly approved interactive management command can have a separate grant flow; its vocabulary remains undecided.

Future acceptance: disable then update, introduce a new app contribution, widen a capability, revoke a document scope mid-session, and load a development-installed third-party record in a release-profile host. No path may regain authority through a different command wrapper.

## Run a one-shot command or interaction

FigCraft captures the selected objects and document revision, resolves an installed command, acquires verified bytes and scoped instance authority, then calls it with explicit parameters. A panel invokes that same operation. Proposed edits remain bounded and app-validated. The app commits one transaction and returns native objects plus result data. Undo affects committed edits; it does not retract an externally written artifact.

Stale revision, invalid objects, forbidden nested operations, cancellation, trap and budget exhaustion discard the uncommitted transaction. Plugins cannot hold raw document pointers across callbacks. Long-lived tools use begin/update/commit/cancel semantics and disposable subscriptions. Reentrancy and event depth are bounded; a callback cannot recursively create unbounded work. Panel closure releases subscriptions and instance resources. If a job result targets a changed document, the app rejects or offers explicit reapplication rather than silently overwriting it.

Owner: app adapter and transaction engine, with shared authority/result envelopes. Future acceptance: compare menu, panel and CLI document results; undo/redo; reject a stale job result; cancel during a gesture; close the panel during a callback; prove that a hidden file-write parameter cannot bypass capability enforcement.

## Evaluate a persistent effect

EffectCraft stores a contribution reference and independent parameters for each layer instance. At render admission, the app resolves the exact release and required resources, pins their lifetimes, and evaluates keyframes and time context. The adapter translates the app's existing effect/helper semantics; it does not assume its no-import WASM ABI supports every helper feature.

PhotoCraft's analogous path supplies colour/alpha mode, band dimensions and overlap, selection blending, preview scale and declared processing context. The host validates output shape and values before applying the filter. A destructive filter commits pixels; a live feature retains its dependency. FilmCraft needs separate timebase, frame and temporal-dependency semantics before adopting a processing contract.

Temporal state belongs to the document instance and evaluation session. Parallel previews and exports do not share mutable processor state accidentally. Seek/reset and frame-dependency rules determine reuse. Malformed output, unsupported formats, device loss and missing context fail explicitly. Required final processing is never silently omitted.

Future acceptance: two identical effects with different parameters, concurrent preview/export, seek/backward render, tiled versus full-frame equivalence, alpha/colour fixtures, invalid output dimensions, missing temporal frames and reset behavior. Specify exact expected results or a documented numerical tolerance per contract. No equivalence result is claimed yet.

## Save, reopen, migrate and exchange a project

Save records source/package identity, contribution, exact selected release, parameters, contract and state schema. One-shot edits normally need no continuing executable dependency. Live effects, widgets and procedural features do. State has per-instance size limits and no credentials or machine-specific authority.

Reopen resolves installed dependencies without downloading or enabling anything. Missing, disabled, revoked or incompatible contributions become preserved placeholders with reason and remediation. Inspection and native edits remain possible where the app can support them. Saving preserves unknown feature state. A final export that needs unavailable processing fails before reporting a successful artifact; preview fallbacks are visibly labelled.

An explicit upgrade migrates a copy of old state, validates it, and commits through app history or a saved-project copy. Migration cannot use authority the new release lacks. A failed migration preserves the old state and release selection. An explicit bake stores native output and records that the live dependency was replaced. Collaboration exchanges references and state, not credentials, grants or executable auto-installs.

Future acceptance: save/reopen with present/missing/revoked/updated modules, unknown state preservation, failed migration, rollback, two independent instance states, project transfer between different source configurations, and a labelled baked fallback. Legacy ID mapping must reject ambiguous source matches.

## Use CLI and automation

| Session | Plugin environment and owner | Completion rule |
|---|---|---|
| Fresh process | App bootstraps the resolved shared store and installed eligibility; process owns its jobs | Await all work required for requested save/export before successful exit |
| Attached editor | Editor's session, versions, documents and grants are authoritative | Return the editor's result or job handle; client-local installs do not alter that session implicitly |
| Persistent headless server | Server owns document sessions, instances and jobs across requests | Poll/subscribe/cancel a server-owned job; shutdown follows explicit drain/cancel semantics |

CLI calls supply parameters, document targets and output destinations explicitly. UI-only panels report unavailable headlessly. A FigCraft offered-file result becomes an artifact the host writes to the supplied target, not a discarded save-dialog request. FilmCraft's long-operation model informs job ownership; every app still defines its own output artifacts.

Structured output includes terminal status, reason, release/contribution, job ID where durable, output descriptors and remediation. Numeric exit-code assignments remain open; at minimum unavailable/input-required/failed/cancelled work must not look like success. No hidden dialogs, implicit downloads, automatic grants or detached jobs with no remaining owner. Diagnostics disclose the selected store configuration without leaking personal paths by default.

Future acceptance: fresh GUI/CLI eligibility parity, missing output target, attached-session version mismatch, server restart, cancellation, partial output cleanup, and a process that receives a job ID but must await artifact completion. The source record identifies current EffectCraft initialization and FigCraft UI-request risks; these are not reproduced defects here.

## Update, rollback, disable and remove

An update stages and validates a new release while the selected old release remains available. Under the store lock it rechecks continuity, versions, preferences, grants and publication state, then commits a coherent generation. Existing jobs keep pinned bytes/resources. New uses select the new default only when eligible; saved exact references do not silently float.

An authority increase leaves the new release pending. The old eligible release may remain usable unless revoked or explicitly disabled; the manager shows that distinction. An explicit downgrade/rollback still checks provenance, compatibility, revocations and state compatibility. It never resets freshness counters or silently migrates project state backwards.

Disable stops new use for one app and triggers the app's running-work policy. Uninstall is global management: show known active use and known pins, remove eligibility for new use, retire panels/subscriptions, and defer byte deletion while readers retain handles. Already committed native edits remain. Unknown projects on disk cannot all be discovered. Data removal and preference/grant cleanup are explicit consequences, not claimed universal project preservation.

Garbage collection removes only verified unreferenced releases with no active readers and no retention pin, under the store's retention policy. Keep one rollback version by default subject to disk policy. Importers either retain resource access for the full import or copy data into app-owned storage. Large lazy resources cannot rely solely on the executable being in memory.

Future acceptance: launcher update during a CLI export; uninstall while a panel is open; update after disable; retry after publication crash; clean while a long job reads a LUT; rollback to a revoked release; disk pressure with active leases; and old/new clients writing alternately. Resource leases and state migration are required design work before claiming update safety.

## Offline use, incidents and safe startup

Previously verified eligible installations can run offline under cached policy. New acquisition requires fresh authenticated metadata under the selected protocol. Clock rollback cannot clear seen-version records. Restricted downloads and runtime entitlements are independent decisions if they become necessary. The system must explain which condition failed.

The source operator handles disclosure, investigation, takedown, appeal, authenticated revocation and recovery metadata. Publisher and host maintainers identify affected releases and contract vulnerabilities. The store caches revocations; app hosts enforce them on load and calls. Advisory notices may allow completion; emergency blocking revocation denies new authority and cancels affected work at a safe boundary. In-flight GPU or real-time work needs a contract-specific boundary. Final artifacts from aborted work are not reported as successful.

Safe startup opens projects without executable plugins and with built-in appearance. Quarantine identifies the failed release and app, preserves projects, and explains repair or reinstatement. Repair verifies bytes and state backups without treating a package re-download as permission to restore revoked rights. Support bundles are opt-in and redact content, paths and credentials by default.

Future acceptance: corrupted/newer state, broken theme, startup trap, repeated quarantine, offline known revocation, compromise during export, recovery-key exercise and an opt-in redacted support bundle. These are planned checks, not completed incident drills.

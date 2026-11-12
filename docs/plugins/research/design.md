# Reconciled plugin and add-on design

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: recommendation for discussion, 2026-11-08. This file does not authorize implementation. The [accepted decisions](../../plugin-functional-decisions.md) remain authoritative. Organizations are excluded. The third-party release gate stays closed. See the [research index](README.md) for evidence and companion references.

## The experience this design supports

An author packages an existing EffectCraft effect. The person installs it once, enables it for EffectCraft, uses it on two layers with independent parameters, and saves. A fresh CLI can export that project using the same eligible installed release. If the release is missing, disabled, revoked, or incompatible, export reports the dependency and produces no successful final artifact.

A package can also provide a FigCraft command. Its panel and CLI invoke the same operation. The app validates the resulting changes and commits one undo transaction. Installing this package does not grant it every editor operation. PhotoCraft filters retain their colour, alpha, selection, and overlap semantics. FilmCraft retains its own timeline and job semantics rather than inheriting an image-filter interface.

These are proposed outcomes. The baseline establishes several pieces of each path, not a completed shared implementation. [The source record](../source-and-verification-record.md) distinguishes them.

## Ownership follows the existing implementation

Extend `craft-ext-core` and `craft-ext-store` in craft-extensions. Preserve its ZIP envelope, manifest validation, bounded file admission, store lock, atomic state publication, generations, and theme reader as the starting point. This is reuse of code at a named revision, not certification of its guarantees.

| Owner | Responsibility | Boundary |
|---|---|---|
| craft-ext-core | Package identity, versioned manifests, contribution declarations, compatibility and policy vocabulary | No app document types and no app dependencies |
| craft-ext-store | Verified installation records, immutable releases, enabled preferences, authority snapshots, resolution and retention | No app-specific edits or rendering |
| Launcher source client and extension UI | Fetch, cache, authenticate source metadata, present management and grants, submit verified artifacts | UI never assigns privileged trust from publisher-controlled strings |
| Each app adapter | Advertise contracts, apply runtime policy, invoke contributions, enforce scoped calls, own documents/jobs/output/themes | Installed metadata does not register arbitrary app operations |
| SDK and package tooling | Generate file inventory and bindings, validate contract fixtures, explain diagnostics | No install hooks and no runtime approval by the author |
| Source operator | Publisher enrolment, trusted metadata, review, incident and recovery procedures | Operator and repository location remain undecided |

No crate move into craft-libs is proposed now. A future move needs real consumers and an agreed boundary. Historical ownership and portable-store suggestions in [the lifecycle review](../../plugin-lifecycle-review.md) are superseded here by the current reuse direction and the existing shared-store location decision. An explicit environment override may select an isolated store; portable launchers do not silently create a competing default.

## Identity records what was admitted

Use distinct concepts throughout the system.

| Concept | Proposed meaning |
|---|---|
| Source | An enrolled distribution authority with a stable local identity and authenticated key history |
| Publisher | The party that claims authorship, with verified identity only to the extent the source establishes it |
| Package | The installed/update unit, identified by source and stable package ID |
| Release | Immutable artifact identity plus its version label, verified manifest, and file inventory |
| Contribution | One app-specific operation, processor, panel, theme, or typed resource within a release |
| Document instance | One saved use of a contribution, with independent parameters and state |
| Runtime instance | A temporary host-created execution context with fixed identity and scoped authority |

Store both the archive digest used by authenticated distribution and a digest of exact retained manifest bytes. The manifest binds every admitted payload file by size and digest. Do not casually substitute a canonicalized manifest hash for the archive hash a source authenticates. The exact identity algorithm and encoding remain a schema decision.

An enrolled source ID must survive legitimate key rotation. Opus proposes the current root-key fingerprint as SourceId; that would change identity on rotation. Prefer a stable enrolled ID with the initial root anchor and verified successor chain. A URL move does not change authority. Changing source authority needs an explicit transition, never same-name replacement.

Local unsigned files have unverified authorship. A local import records its origin and release bytes; it does not gain official identity by declaring an ArtCraft publisher or matching a dotted ID. A signed file requires an independently trusted key binding. A matching display ID from two sources stays distinct internally and requires an explicit selection or replacement path. Legacy bare IDs need an app-owned migration map with ambiguity errors.

Opus found that current installed records lack release digests, source and publisher continuity, and the admitted manifest. The primary reconciler confirmed the record and reader definitions at `ffd8512e`. [The independent note](opus-contracts-trust.md#3-identity-and-source-trust) documents the paths and source-derived replacement cases. New checks cannot rely on data the old format never saved.

## One eligibility policy, applied by every host

Resolve metadata without executing modules. A shared resolver receives app version, supported contracts, build admission policy, installed state, saved reference, grants, resource readiness, and cached revocations. It returns either a resolved release or structured reasons for refusal. Names below describe a proposed interface, not public API.

```text
resolve(host_profile, contribution_ref, authority_snapshot)
  -> eligible {release, contribution, grant_scope, generation}
   | unavailable {reasons, remediation}

acquire(eligible, job_context)
  -> verified_module_bytes + retained_resource_handles + instance_authority
```

Eligibility requires installation, enabled state for the app, compatible app and contract, admitted source class under this build, verified loaded bytes, no blocking revocation or quarantine, sufficient existing grants, and required resources/backend. A release may contain supported and unsupported contributions. Show those individually, but reject the whole manifest if the host cannot understand a security-critical package requirement.

Recheck policy on enable, project resolution, new invocation, and load. Long-running sessions observe authority generation changes and enforce revoked rights at host calls. A release app must refuse gated third-party records even if another client installed them in the shared store. A `FromFile` label never exempts externally authored code from module or runtime admission.

The current reader filters by kind, app, enabled state, and directory existence. It does not implement this full policy. [Reader source](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-store/src/reader.rs) is evidence for the gap, not evidence that any executable exploit ran.

Enablement and grants are different records. Preserve each app's enabled choice on update. Newly introduced app contributions start pending or disabled. A release with additional capabilities can be installed without becoming runnable. Fresh CLI sessions never download, enable, widen grants, or open a hidden consent dialog. They return a reason and the management action needed.

## Package and state compatibility need separate versions

Schema 1 remains its constrained, empty-permission contract. Multi-contribution packages and host imports require an explicit successor schema and purpose-specific interface negotiation. Older parsers must reject it before considering execution. Optional presentation metadata may be ignorable; required capabilities and interpretation rules may not.

Require author-written stable IDs, release version, target contracts, required capabilities, parameter/state schemas, resource readiness declarations, and licence metadata. Derive file sizes, digests, export inventories, and boilerplate. Tooling should validate rather than invent capabilities from imports. A stable SDK promise follows real EffectCraft and FigCraft fixtures, not merely a manifest freeze.

The following formats evolve independently: package manifest, source metadata, installed/authority records, and app project references. A source format bump does not migrate an EffectCraft project. A project migration does not upgrade an app runtime.

The current state parser ignores unknown fields and serializes only known fields. Old writers would erase newly added authority fields. Do not solve that by adding fields under schema 1. Prefer an explicit versioned state migration with old writers refusing to write. Preserve the old state backup and immutable releases. Readers must distinguish an absent store from a newer or corrupt store; theme fallback can remain quiet in the canvas while management explains the failure.

Opus recommends splitting grants and revocations into files. That may help theme-only readers, but locks alone do not give lock-free readers a coherent multi-file transaction. Recommended first design: one authoritative snapshot, or immutable generation files selected by one atomic root record. Keep separate files only if their generation and failure semantics are equally explicit. Do not add a bespoke database or new dependency in this research. Test a mixed old/new app writer sequence before choosing the layout.

## Source trust and update security

Authenticity, provenance, execution authority, and licence are independent. A signature authenticates bytes to a key. It does not prove safe code or that a claimed commit produced the artifact. A hub build proves what its builder attests and transfers risk to that service. Private first-party source does not imply paid licensing.

For the production distribution path, recommend established authenticated update tooling for official and community sources alike. TUF describes root rotation, timestamp/snapshot/target verification, expiry and rollback checks. It does not choose the Craft catalogue owner, certify plugin safety, or enforce capabilities. Its clock assumptions also need application policy. [TUF specification, client workflow and repository operations](https://theupdateframework.github.io/specification/latest/).

Opus suggests an app-bundled first-party digest allowlist for a limited pilot before a catalogue exists. Retain it as an optional test-distribution mechanism, with app release cadence and revocation limits explicit. It does not test catalogue freshness or complete Jeremy's catalogue tasks. Prefer completing an authenticated first-party catalogue for the full distribution pilot. Do not design a temporary custom signed-index protocol.

Recommend one curated default source and optional explicitly enrolled sources later. This limits initial operations burden while preserving accepted third-party distribution. Source removal stops discovery/update access; it does not silently remove installations or transform their identity. Bad metadata retains the last verified cache for inspection, but cannot authorize new installs or updates from expired trust metadata. Bound response bytes, pages, redirects, archive sizes, concurrency, retries, and cache storage. Reuse exact verified cached artifacts only under an explicit local-import or authenticated-install policy.

Hub-only builds give centralized reproducibility and review but incur build capacity, private-source handling and service-compromise costs. Publisher builds with source/commit claims cost less to operate but offer weaker provenance; attestations help only when their builder and claim are verified. No monetary estimate is available. Recommend a small first-party build path first and a costed community provenance decision before sg13, rather than promise every package is reproducible.

## Runtime authority and budgets

Each app advertises only implemented contracts. The shared layer defines policy; the actual engine validates and compiles the exact verified bytes with the same feature/import configuration used to run them. A standalone validator with different settings cannot certify admission. Do not select one runtime for every app by assertion.

The host creates instance identity. Every imported operation checks capability, app/document scope, parameter-dependent side effects, and the current authority generation. A generic command dispatcher is not an unrestricted plugin import. Nested commands and output paths cannot bypass the originating instance's rights. File outputs are host-written artifacts; network/account operations need separate scoped brokers if later approved.

Before running, bound module parsing and compilation, memory, stack, inputs, output, total host allocations, logs, events, subscriptions, and concurrent work. During execution, bound module instructions/time and host work separately. Cancellation has to reach both. Wasmtime documents sandbox isolation and interruption mechanisms, but this is not a decision to replace current engines. [Wasmtime security](https://docs.wasmtime.dev/security.html), [interruption API](https://docs.wasmtime.dev/api/wasmtime/struct.Config.html#method.epoch_interruption).

Use host-mediated, declared GPU operations initially. Include upload/download cost, queue limits, device loss, and declared backend requirements in compatibility. CPU fallback is a contribution promise, not a host invention. Arbitrary plugin shaders and native helpers remain deferred until platform confinement and failure handling are proven. WebGPU discusses availability and driver risks; a queue or separate process alone is insufficient proof. [WebGPU security considerations](https://www.w3.org/TR/webgpu/#security-considerations).

No OS confinement has been demonstrated for new native helpers on macOS, Windows, or Linux. Desktop WASM policies still need engine-specific validation on each supported platform. Web targets retain existing builds and expose only available contracts; do not load native helpers there. SoundCraft needs an audio-specific feasibility study before choosing a runtime, including real-time deadlines, latency, tails, automation, MIDI and state.

## Documents and running work have different lifetimes

Proposed saved references include source/package identity, contribution ID, exact release identity and version, contract/state schema, parameters, and bounded opaque instance state. A compatible update may become the default for new uses while an existing project continues with its selected release. Upgrade project state through a copy, validate it, and commit explicitly through the app; never rewrite on mere inspection.

One-shot FigCraft edits leave native objects and normal undo history. Persistent EffectCraft effects retain executable dependencies. Missing/revoked features remain preserved placeholders, with inspection and deletion possible. Final export fails rather than omit required processing. Baked output is a separately labelled fallback chosen by the person; it is not proof the live effect ran. Collaborative documents exchange dependency references and state, never grants or automatic executable installation.

At job admission, pin the release and relevant resources. Keep old bytes until all jobs/instances that may read them finish. Reading a module fully into memory settles only that module's byte lifetime; lazy resources, module reload, and dependent stages still need retained handles. Recommend retention accounting from the first executable job; choose an in-memory ownership token or cross-process lease only after tracing actual readers. A grace period alone cannot prove safety for a long export.

The store cannot enumerate every unopened project on disk. Retain one rollback release by default and explicit user/project pins, report disk use, and warn that removal may make other projects unresolved. Never claim garbage collection preserves unknown offline projects. Rollback still applies revocation and grant checks; it is not a security bypass.

## Themes, resources, and authoring

ThemeFile remains package data; WidgetPalette remains a drawing API. PhotoCraft's adapter maps supported tokens, checks contrast combinations, and falls back. FilmCraft needs independent mapping of timeline, selection and status semantics. Installing a theme does not select it. Preview is cancellable and does not replace host-owned permission UI. ThemeCraft editing and re-export are roadmap work with a round-trip fixture, not a new theme format selected here.

The author delegate found concrete mapping constraints: ThemeFile accepts opaque RGB and at most 64 overrides per app, while FilmCraft has Dark, Medium and Light families and a larger token set. Recommend sparse semantic overrides, preserving the user's Medium/highlight/accessible-contrast preferences, rather than exporting every token or inferring a family from luminance. PhotoCraft's translucent derived tokens need an override allowlist that avoids flattening their alpha. The reconciler confirmed the palette cap and opaque RGB definition; detailed FilmCraft counts remain the delegate's source evidence. [Theme mapping analysis](haiku-author-resources-recovery.md#5-photocraft-and-filmcraft-theme-semantics-m2-part-2).

Resources use typed importers with byte, decoder, dimension, object-count, and recursion limits. A brush or LUT can attack an importer without executable code. Host-owned copies or retained handles prevent installed-resource removal from corrupting an active import. Fonts must follow craft-fonts ownership and attribution rules; a resource ZIP is not an exception. Optional model downloads are separate readiness and explicit network operations with size, identity, licence and storage policy. Do not execute workflows on install. ComfyUI JSON needs a separate execution adapter, node compatibility and permission design before it can run.

Recommend bundled or digest-pinned data dependencies initially, with no executable inter-plugin dependency graph. Optional resources expose unavailable operations explicitly. This keeps recursive resolution, cycles, private credential propagation and transitive grants out of the first pilot. Revisit executable dependencies only with a real consumer; bound graph nodes, depth and total bytes when introduced.

Compilation caches key release digest, engine/configuration, target and contract. Evaluation caches also include inputs, parameters and relevant context such as time, colour mode or units. Stateful processors declare reset and instance isolation. Schedule interactive and export jobs separately within total budgets. Measure copies, compilation and host-call cost in later scoped experiments; neither delegation nor source reading establishes a performance win.

## Recovery and operational responsibility

Store repair verifies retained release inventories, repairs interrupted publication, and preserves unreadable state for diagnosis. Restoring a backup cannot reset trusted freshness counters, known revocations, or grant restrictions. Safe startup disables executable contributions and uses built-in appearance without deleting projects. Repeated failures quarantine an exact release for the app; a clean restart or reinstall does not silently restore its authority. Diagnostics include package/release/contract, store selection, generation, job stage and reason. They omit document content, user paths and credentials by default.

On compromise, the source operator publishes authenticated revocation and a recovery release; store clients cache policy; hosts deny new use and apply the running-job rule. Recommend immediate denial of revoked host capabilities and cancellation of affected transactional work at a safe boundary. Emergency revocation can fail an export and discard its uncommitted artifact. Advisory notices can allow completion. A universal immediate-kill promise is inappropriate for GPU queues and audio deadlines.

New installs/updates need fresh trusted metadata. Previously verified installed eligible packages can continue offline under cached policy. Offline clients cannot learn unseen revocations. Persist seen metadata versions and detect suspicious clock rollback; require trusted recovery for new acquisition when time cannot be established. Do not use an untrusted system time jump as permission to reset replay counters. Product entitlements, if needed, require a separate offline policy and credential store. Recommend no new licensing machinery until restricted access is an actual requirement.

## What must be decided next

The [decision register](decisions.md) orders the choices by consequence. The [workflow reference](lifecycle.md) traces each user path. The [adapter matrix](app-adapters.md) preserves app ownership, and the [coverage register](coverage.md) maps every requested aspect to evidence and pending checks. None of those checks has run as application acceptance in this documentation task.

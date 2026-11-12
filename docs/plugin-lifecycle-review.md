# Shared plugin lifecycle review

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Status: design discussion, 2026-11-07. No implementation decisions are approved. This expands the [initial ideas](plugin-architecture-ideas.md) and [resource register](plugin-resources.md). The linked ChatGPT conversation and Claude artifact remain unavailable. Their contents are not evidence here.

Follow-up on 2026-11-08: the user chose sandboxed executable plugins initially, official and third-party distribution, and FigCraft second after EffectCraft. Those choices resolve the corresponding questions below and supersede the earlier distribution recommendation. The current functional investigation and remaining choices are in the [decision record](plugin-functional-decisions.md) and [combined model](plugin-functional-model.md).

## Evidence and limits

This is a source and architecture review. No application was executed, no performance was measured, and no sandbox was penetration-tested. Recommendations below describe target behavior, not existing guarantees.

The following existing contracts were inspected:

- EffectCraft at `ce9eaec65015a5508c72af0e2351cd02612c9450`: [effect registration and rendering](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/effects/src/plugin.rs), [WASM runtime](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/plugin/src/wasm.rs), and [package library](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/engine/src/system/library.rs). Effects register in a process-wide registry and cannot currently be unloaded. Library updates finish on restart.
- PhotoCraft at `655295b2a2c400bded0640f5d97367000a057b2e`: [runtime limits](https://github.com/storytold/photocraft/blob/655295b2a2c400bded0640f5d97367000a057b2e/crates/plugins/src/runtime.rs), [pixel application](https://github.com/storytold/photocraft/blob/655295b2a2c400bded0640f5d97367000a057b2e/crates/plugins/src/apply.rs), and [manifest](https://github.com/storytold/photocraft/blob/655295b2a2c400bded0640f5d97367000a057b2e/crates/plugins/src/manifest.rs). Filters process bounded bands in the document's colour model. The contract includes overlap, pixel units, selection blending, and limits on a run's combined instance memory.
- VectorCraft at `065a646799f1f20f5454845d6648e09a5252c452`: [extension kinds](https://github.com/storytold/vectorcraft/blob/065a646799f1f20f5454845d6648e09a5252c452/crates/plugins/src/manifest.rs) and [object input and output](https://github.com/storytold/vectorcraft/blob/065a646799f1f20f5454845d6648e09a5252c452/crates/plugins/src/objects.rs). Supports object filters, live geometry effects, and exporters. Geometry output has finite-coordinate and count limits.
- FigCraft at `0e86f3597a7443b15c93ec3ab2091202f8802c95`: [engine policy and transactions](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/crates/engine/src/extensions/plugins.rs) and [execution limits](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/crates/plugin/src/sandbox.rs). Supports commands and panel callbacks with command restrictions, per-call accounting, and rollback on failed document operations.

These are sampled app contracts, not a completed inventory of every Craft app. FilmCraft, PDFCraft, LightCraft, and other apps need equivalent discovery before integration commitments.

## 1. Define the protection promise

Recommendation: plugins extend designated editor capabilities. They cannot patch app executables, replace internal implementations, access arbitrary host memory, change security settings, install other plugins, or silently acquire access to other documents.

An editing plugin must be able to change the user's document. Those changes pass through approved editor operations and undo. A permitted action can still be unwanted or artistically incorrect. Validation protects structural integrity, while previews, scoped grants, and undo protect the user's workflow.

The trusted computing base includes parsers, the runtime, host functions, brokers, and GPU drivers where used. Avoid absolute promises that plugins can never crash the app. Native code without enforced restrictions cannot satisfy the protection promise merely because it runs in another process.

Decision: does protecting the app permit third-party commands, panels, and canvas tools inside designated areas, while forbidding replacement of built-in behavior?

## 2. Separate extension purpose from execution

Support purpose-specific contracts under one package envelope:

| Purpose | Required contract and controls |
|---|---|
| Assets, presets, brushes, LUTs, templates | Typed data schema, parser limits, attribution, bounded expansion, no install scripts |
| Image filters and generators | Pixel format, colour space, alpha convention, bounds, overlap, selection, preview scale |
| Video and temporal effects | Frame request times, input dependencies, temporal bounds, deterministic state, cancellation |
| Audio processors | Sample rate, channel layout, block size, latency and tail reporting, transport state |
| Geometry and procedural objects | Coordinate units, object schema, output-size limits, live versus committed edits |
| Importers and exporters | Stream or temporary output, document schema, progress, failure recovery, file grants |
| Commands and automation | Typed requests, document scope, version checks, transactions, undo |
| Panels, widgets, and tools | Namespaced declarative UI, bounded callbacks, selection events, undoable gestures |
| AI and heavy computation | Model identity and size, resource budgets, cancellation, local versus remote processing |
| Cloud and hardware integrations | Account or device capabilities, network restrictions, explicit external side effects |

Data-only packages still contain hostile input and need parser limits. A panel's layout can be data-only while its callbacks require executable code. A package may contribute more than one extension. Installation must not imply activation of every contribution.

Decision: which purposes must work in the first release? Recommendation: existing WASM effects plus one command-based app integration before freezing a suite-wide stable interface.

## 3. Establish identity and a package manifest

Use a stable package identity, verified publisher identity, immutable release digest, and stable contribution IDs. A display name or reverse-domain string alone does not prove ownership. Preserve existing effect IDs during migration, with explicit package-to-contribution mappings.

The manifest declares schema version, release version, platform artifacts, runtime contract versions, contributions, required and optional capabilities, bounded assets, and state schemas. Reject unknown required capabilities. Keep optional features explicitly unavailable rather than silently claiming support.

Separate package release versions, host interface versions, and saved-state versions. Compare versions with defined semantics. Do not guess compatibility from a shared crate version or a plugin name.

Decision: can one package contain multiple app-specific modules? Recommendation: yes, with each module declaring its contract and platform requirements. Shared packaging does not guarantee shared algorithms.

## 4. Give authors an SDK and a development path

Provide Rust SDK bindings, a manifest validator, sample plugins, and a harness using the same runtime and checks as production. Authors implement public contracts, never link against private app structures. Other languages may target a supported WASM ABI.

The local development path builds a package, validates it, loads it into a development session, and displays actionable errors. Development trust is visibly scoped to that session. It must not disable production checks or automatically register a permanent startup plugin.

Declare dependencies and assets in the package. Initially avoid arbitrary plugin-to-plugin dependency resolution, shell installers, and downloaded executable dependencies. Large AI model resources need separate consent, size limits, and integrity verification. Review dependency and asset licences under the repository's applicable rules.

Decision: support a development reload command immediately? Recommendation: session-only reload where registration can be safely replaced, otherwise an explicit restart.

## 5. Publish and review releases

An author submits an immutable release artifact and source reference. Automated review validates manifests, identities, licences, capability requests, archive limits, ABI exports, and basic behavior. Human review evaluates misleading permissions and distribution policy. Neither review nor signing proves code is harmless.

Catalog metadata authenticates exact artifact digests. Include key rotation, metadata freshness, rollback protection, and a response to compromised releases. A signature without freshness permits an attacker to replay an old signed catalog. [TUF](https://theupdateframework.github.io/specification/latest/) defines these update threats and a concrete client workflow. Use an established approach rather than inventing cryptography. Selecting an implementation and dependency is separate work.

Keep publisher trust distinct from platform code signing. Any catalogue listing or developer bypass must have a clear provenance shown to users.

Decision: official catalog only, official catalog plus sideloading, or third-party catalogs too? Recommendation: official catalog plus explicit local sideloading first. Broader catalogs add trust and recovery work.

## 6. Discover and check compatibility without executing code

Browsing uses cached metadata and does not instantiate modules. Show what the plugin contributes, supported apps, requested access, download size, model size, and restart requirements.

Eligibility checks cover app identity, interface versions, platform, architecture, runtime availability, supported pixel or object contracts, and GPU features. A plugin can be compatible but not yet permitted. Optional CPU fallback must be declared and semantically compatible.

Metadata must be readable without executing plugin initialization. Existing embedded-manifest formats can use a legacy adapter that reads metadata under execution limits. Compare embedded metadata with verified package metadata before activation.

Decision: show incompatible packages for discovery? Recommendation: show them with a specific explanation and no misleading Install action.

## 7. Preview with the same isolation as execution

Preview is execution. Run it on synthetic sample data or an explicitly chosen document snapshot with no unrequested files, accounts, or network access. Keep preview settings separate from permanent grants.

Limit compute, output, logs, and downloads. Cancel superseded requests and reject results from an old document revision. Permit authors to provide media, but identify promotional media separately from host-generated results.

Decision: synthetic samples or current-selection previews by default? Recommendation: synthetic samples for browsing, opt-in selection snapshots for creative evaluation.

## 8. Install transactionally and recover after interruption

The install path is: verify catalog metadata, select artifact, bound download, verify digest, inspect archive, extract to private staging, validate manifest and modules, publish an immutable version directory, then atomically record installation.

Reject traversal, absolute paths, symlinks and hard links, ambiguous duplicate names, platform filename collisions, excessive file counts, and compressed expansion beyond the limit. No plugin install hooks run with the app's privileges. Startup functions used during validation remain sandboxed and metered.

Interrupted downloads or failed validation leave the existing version usable. A package published before its registry record commits is an orphan for later cleanup, not a reason to erase valid records. Corrupt metadata produces a recovery state rather than silently appearing as a new empty store.

Decision: choose a package archive extension and format only after these invariants are agreed. The container name itself is not the security design.

## 9. Separate stored files from app-owned state

Recommendation: share immutable package bytes where platform storage permits, but keep grants, activation choices, preferences, and project state scoped to their owner. Portable installations may need their own store.

Never overwrite a version used by a running process. Identify artifacts by verified digest, not version string alone. Limit locking to store mutations and define atomic commit points. Apps must not overwrite one shared preferences JSON file with competing copies.

Garbage collection preserves active references, rollback versions, and explicit pins. Project files outside a known registry cannot all be discovered, so deletion needs a documented retention policy rather than a false promise of preserving every historical dependency.

Decision: shared downloads with per-app activation, or fully separate stores? Recommendation: shared immutable bytes and per-app state, with portable-mode exceptions.

## 10. Grant narrow capabilities and enforce them on calls

The effective authority is the intersection of host policy, package requests, user grants, document scope, and platform support. Identify the plugin from the running instance, never from a caller-supplied plugin ID.

Grant access to a selected document or resource handle instead of arbitrary filenames. Reserve permission UI for the host. Optional capabilities can be requested when first needed. Revoke access for new calls immediately, cancel outstanding work where supported, and document that already-disclosed data cannot be retrieved.

Do not expose the complete CLI or MCP command catalog. A wrapper command that runs other commands must retain the same checks for every nested operation. Reject unknown commands by default. File reads hidden inside command parameters must receive the same scrutiny as explicit file commands.

Decision: install-time required grants with contextual optional requests? Recommendation: yes. Avoid repeated prompts for routine frame processing.

## 11. Broker files, networking, accounts, and devices

Plugins request a user-selected file or bounded export destination. The host controls the chooser and final write. Restrict shared memory and resource handles by ownership, size, access mode, and lifetime.

Networking goes through a broker with explicit destinations, methods, size limits, redirects, cancellation, and private-network policy. Resolve destinations safely rather than relying only on a string prefix. Account tokens stay in the credential owner. A plugin uses an account operation rather than receiving a long-lived secret when practical.

Document access combined with network access permits data disclosure. Explain the combination. Uploads, emails, paid requests, and remote publishing are external side effects, not undoable document changes.

Decision: support cloud integrations in the first release? Recommendation: defer them until the broker and explicit external-action UX exist.

## 12. Activate lazily and negotiate the contract

Keep independent records for package presence, grants, desired enabled state, and runtime sessions. Runtime states include unloaded, initializing, ready, stopping, and failed. A staged update can coexist with an older active version.

At first use, negotiate the exact supported interface, validate requested contributions, initialize within limits, and register only after success. Run initialization outside the UI thread. Do not run plugins merely because a project mentions them.

Cancel pending startup on disable. Failed initialization leaves the manager usable and explains what failed. A restart requirement is an honest supported state.

Decision: activate all installed plugins at startup or lazily? Recommendation: lazy activation, with bounded registration metadata available before code runs.

## 13. Apply edits through the app's document model

For an editing operation, capture a document revision and scoped input snapshot, run the plugin, validate its result, then commit through the app's existing transaction mechanism. If the document changed meanwhile, reject or explicitly recompute. Do not silently apply stale geometry to a new selection.

One command or meaningful gesture produces one undo step. Preview changes remain temporary. Bound output objects, points, strings, recursion, and transaction size. Host-side validation also consumes a budget. Use coalesced events, bounded queues, and reentrancy guards to prevent a plugin's edits from triggering an infinite callback loop.

A transaction can roll back document edits. It cannot roll back an upload or a device action. Stage exports and validate all edits before committing external side effects. Large workflows need explicit checkpoints instead of holding a document lock while the plugin computes.

Decision: require confirmation for every edit or only destructive or external operations? Recommendation: ordinary user-invoked edits use undo, with confirmation where reversal is unavailable.

## 14. Render through purpose-specific data contracts

An image contract must define channel layout, alpha convention, colour representation, stride, bounds, overlap, selection, and preview scale. PhotoCraft's document-colour samples and EffectCraft's premultiplied RGBA frames cannot be treated as identical buffers without conversion.

A temporal effect declares frame dependencies and time units. Bound recursion and requested frames. A procedural object preserves editable state separately from generated geometry. An audio contract needs latency and block semantics, with no allocation, locks, or network calls on a real-time callback. Real-time audio is a separate integration gate, not a claim that image runtimes work unchanged.

Preserve input until output validates. Interactive preview may bypass a failed effect with a visible warning. Final export must fail or require explicit acceptance of missing effects, rather than quietly produce different output.

Decision: guarantee identical cross-backend results or define numerical tolerances? Recommendation: define per-contract tolerances and require exactness only where the algorithm promises it.

## 15. Choose execution and optimize from measurements

Use WASM for portable logic, host GPU operations for supported compute, and restricted native helpers for demonstrated SDK or acceleration needs. WASM can orchestrate GPU calls. Native execution is not automatically faster after transfer and isolation costs.

Compile a verified module once per compatible runtime configuration. Key caches by artifact digest, contract, host version, and relevant configuration. Bound compiled-module and instance caches. Reset pooled instances reliably, or keep fresh instances when state could make output depend on call order. Cancellation must not leave a half-used instance in the pool.

Use binary buffers for image and audio data. JSON remains suitable for bounded control messages. Process tiles or bands with declared halos. Account for input copies, outputs, rollback buffers, compilation, and UI previews in aggregate memory budgets. Queue bounded work and prioritize interactive feedback over background jobs. Cancel obsolete previews.

Benchmark startup, warm execution, transfer cost, peak memory, cancellation latency, and end-to-end frame time. No performance winner has been measured in this review.

Decision: accept portable WASM as the initial path while measuring candidate accelerated paths? Recommendation: yes.

## 16. Treat GPU and native backends as separate security work

Start GPU exposure with approved host operations and bounded inputs. Arbitrary plugin shaders require separate policy and validation. Limits on dispatch counts are not a proof that a shader terminates. Cancellation between dispatches does not guarantee interruption of a running GPU kernel. Plan for device loss and reserve resources for the editor. [WebGPU security considerations](https://www.w3.org/TR/webgpu/#security-considerations) document driver and denial-of-service risks.

Native helpers need enforceable restrictions before untrusted code starts. Close unnecessary inherited handles and environment entries. Validate broker messages and resource transfers, bound CPU and memory, and recover if the worker dies. Desktop and web support will differ.

Platform mechanisms require actual probes and tests: [AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation), [Apple sandbox entitlements and process inheritance](https://developer.apple.com/library/archive/documentation/Miscellaneous/Reference/EntitlementKeyReference/Chapters/EnablingAppSandbox.html), and [Linux Landlock](https://www.kernel.org/doc/html/latest/userspace-api/landlock.html) have distinct capabilities and restrictions. Naming one does not establish a working cross-platform sandbox. Disable unsupported execution paths rather than silently reducing isolation.

Decision: allow unsandboxed third-party helpers? Recommendation: no initially. Trusted vendor integrations would need a separate explicit policy.

## 17. Save, reopen, collaborate, and export predictably

Save contribution ID, package identity, exact release digest or version reference, parameters, instance state schema, and deterministic seed or external-input references where needed. Keep user preferences and secrets outside project files.

On reopen, resolve the recorded version. If missing or disallowed, preserve opaque state and show a placeholder. Opening a document never automatically trusts its suggested catalog or installs code. Offer explicit installation or substitution. Never silently replace an exact render dependency with the newest release.

Migrations run on copies under limits, preserve the original, and cannot silently upload data. Shared documents synchronize declared plugin state or committed edits, not process memory. Rendering collaborators need matching dependencies or a documented baked representation. Headless export must report unresolved requirements.

Decision: retain exact project dependency references? Recommendation: yes, with explicit upgrade and optional baking where the app can support it.

## 18. Update without changing active work

Download and validate a candidate independently. Compare publisher identity, compatibility, and requested capabilities. Increased authority requires consent. Stage state migrations on copies. Retain the working release until activation succeeds.

Pin running jobs and project sessions to a version. Activate updates at a safe session boundary or restart. Do not swap modules midway through rendering. Rolling back executable code may also require rolling back migrated state. Replacing a release artifact under the same version is rejected by its digest.

Recommendation: notify and download according to user settings, but require explicit activation initially. Security advisories can recommend disabling a release. A revocation blocks further use where policy requires, preserves projects and data, and explains the effect on export.

Decision: manual activation, automatic activation at restart, or per-plugin policies? Recommendation: explicit activation first, especially for render-affecting plugins.

## 19. Disable, remove, recover, and diagnose

Disable removes entry points, cancels eligible jobs, stops callbacks, and revokes handles. Already submitted GPU work and external actions require their documented completion semantics. Loaded legacy effects may require restart.

Separate disabling in one app, removing its activation record, deleting package files, and deleting user data. Remove bytes only when no active user remains and retention policy permits. Never erase project references on uninstall.

Detect repeated startup failures and offer safe mode that opens without plugin execution. Record bounded, redacted diagnostics with plugin identity, version, operation, resource usage, and error. Do not log document text, tokens, or file contents by default. A malicious plugin must not be able to forge host permission prompts through logs or UI.

Decision: retain settings on uninstall? Recommendation: retain by default with a separate explicit data-removal choice.

## 20. Own the right modules and migrate incrementally

craft-libs owns common parsing, identity, compatibility, store transactions, grants, runtime lifecycle, and optional manager UI. Apps own document transactions, render scheduling, colour or geometry conversion, native file dialogs, and capability-specific policy. Host callback interfaces must not import application types into craft-libs.

Keep these as cohesive modules first. Do not create a separate crate for every concept before their dependency requirements are known. Share one authoritative grant check and installation flow across UI, CLI, and automation entry points.

First deliver an EffectCraft path using existing ABI v1 effects and IDs, with existing commands routed through shared management. Preserve legacy project resolution and make migration resumable. Then validate the contract with a FigCraft command plugin or PhotoCraft filter. Only after a second adapter works should we call it a stable suite contract. Migrate each app's old management callers before removing that app's obsolete implementation.

Decision: which app is the second integration? Recommendation: FigCraft if proving commands, transactions, and panels matters most. Choose PhotoCraft if a portable image contract is the priority.

## Acceptance evidence before implementation is called complete

Each delivery slice needs observable checks at its public interface:

1. Invalid identity, unsupported capabilities, or an incompatible ABI produce specific failures without activation.
2. Bad digests, expired or replayed metadata, and wrong signatures leave the working install unchanged.
3. Traversal, symlink, archive expansion, filename collision, and interrupted-install cases stay contained and recover on retry.
4. Concurrent installs and garbage collection cannot corrupt app records or remove an active version.
5. Revoked access and nested commands cannot bypass grants, including through file-like parameters.
6. Runaway initialization, host calls, oversized results, and event recursion stop within the defined budgets.
7. Failed or stale edits leave the document unchanged. Successful user actions have coherent undo and redo.
8. Pixel conversion, overlap, selection, timeline sampling, and frame-order tests preserve declared semantics.
9. Missing plugins preserve saved state. Export reports unresolved effects instead of silently bypassing them.
10. Permission escalation, rejected migration, activation failure, and rollback preserve the previous working version and state.
11. Plugin crashes allow safe startup and document recovery. Diagnostics do not disclose protected data.
12. GPU and native execution remain unavailable until platform-specific isolation and recovery evidence exists.

No such runtime checks were run for this documentation-only review.

## Decisions to resolve first

Start with three choices because they determine the rest of the design:

1. Protection policy: sandboxed third-party code only, or a separately labelled trusted native mode too?
2. Distribution: official catalog plus local sideloading, official-only, or third-party catalogs immediately?
3. Initial scope: EffectCraft effects followed by FigCraft commands, or EffectCraft followed by PhotoCraft filters?

All other decision prompts above remain open. Agreement to one item is not approval of the entire design or authorization to implement it.

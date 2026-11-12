# SDK and application contracts

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: proposed design, 2026-11-08. The user chose a plugin-first SDK with app-specific declarations and explicit permissions. Command names, records and layouts below are illustrative. No SDK or schema has been implemented. [Decisions](decisions.md) distinguish accepted goals from unselected mechanisms.

## One authoring model

An author creates a package once, then adds contributions for the apps it supports. Repeated facts have one authoritative source. The SDK emits compatible compiled modules, a validated package, release metadata and conformance fixtures. It does not need private app Rust types.

Proposed SDK functions, exposed through a single tool whose name is still open:

| Function | Author input | Generated or checked result |
|---|---|---|
| Create | Package display identity, licence and initial app contract | Stable UUID, template, contribution IDs, sample fixture and generated-binding setup |
| Add app/contribution | App contract and intended operation | App-specific descriptor, parameter types, entry-point glue and minimum rights guidance |
| Inspect host | App/version or checked-in host profile | Available contracts, operations, capabilities, supported targets, UI/compute/resource features |
| Check | Source metadata and compiled modules | Version consistency, schema, imports/exports, parameter/state shapes, licences, inventory and authority mismatches |
| Pack | Validated compiled artifacts/resources | Existing ZIP envelope with successor manifest, file sizes/digests, module mapping and deterministic packaging |
| Test | Real host adapter harness and literal input/output fixture | Behavior, denied scope, transaction rollback, cancellation, headless output and state round trip |
| Develop | Explicit development store and chosen app session | Labelled reload, inspectable logs, isolated grants and bounded execution |
| Prepare release | Package, source revision and selected target assets | GitHub Release asset set, dependency inventory, checksum data, optional provenance, catalogue-PR metadata |

No function publishes or creates a remote without its separately authorized release procedure. Production installs consume compiled release assets. A user's app never builds a downloaded source repository or executes package installation scripts.

Rust is the proposed first supported SDK language. The compiled interface can admit other languages when bindings and behavior are verified. Do not call every language supported merely because it can emit WASM. SDK portability is distinct from support for an existing Obsidian JavaScript plugin or ComfyUI Python node.

## Required author declarations

The author specifies package UUID, human-readable name, publisher claim, release version, licence/usage terms, description and support links. Each contribution declares its stable ID, app, contract and compatible versions, entry point, parameters, required/optional capabilities, resources and supported environments. Persistent features declare state schema and migration support. Network-dependent contributions disclose that requirement.

The SDK derives file inventory/digests/sizes, consistent repeated versions, export lists, generated documentation and bindings. It may detect that an import needs a permission but must not silently add the permission to the manifest. The host checks actual imports/calls against approved authority even if the SDK was bypassed.

Licence declarations cover the package and bundled dependencies/assets. An SBOM or equivalent dependency inventory identifies names, versions and SPDX expressions. No secrets, user grants, trusted tier, review status or official ownership are author-controlled package fields.

The SDK pins each app's generated contract inventory by version and digest in the author's build lock data. Updating that inventory is an explicit development change with generated compatibility/permission differences. Downloaded SDK definitions cannot add runtime rights; the installed host remains authoritative. Release tooling records the actual inventory and engine profile used, not just an arbitrary “latest SDK” string.

An illustrative author file follows. These contract names/versions and tool syntax are proposed, not implemented or supported versions. `MIT` is an example declaration, not selection of the ecosystem licence policy.

```toml
[package]
uuid = "8d33c547-e506-4f4d-a03f-b3948b50a9ac"
name = "Soft Glow"
version = "1.0.0"
licence = "MIT"

[[contributions]]
id = "photo-glow"
app = "photocraft"
contract = "image-filter"
contract_version = 1
module = "modules/photo-glow.wasm"
required_capabilities = ["document.read:selection", "document.edit:selection"]
state_schema = 1
headless = true

[[contributions]]
id = "effect-glow"
app = "effectcraft"
contract = "effect"
contract_version = 1
module = "modules/effect-glow.wasm"
required_capabilities = []
state_schema = 1
headless = true
```

Separate contribution modules adapt the same author algorithm to each app's pixel/time semantics. A packaged module need not receive a document-read import if the host supplies an already-approved input buffer; contract scopes and the execution plan must make that distinction explicit. Parameters use generated typed definitions and state includes its migration contract. Resource/UI/target variants extend this author model through declared contribution types rather than another unrelated SDK.

## Successor package shape

Reuse ZIP and `craft-extension.json`. The next schema explicitly adds multi-contribution entries, typed contracts/capabilities/resources and state metadata. Schema 1 retains its single-kind, empty-permission semantics. Its modules do not gain host imports because a newer parser recognizes extra fields.

```text
package/
  craft-extension.json
  modules/<app-or-profile>/<module>.wasm
  ui/<declarative-panel-data>
  resources/<declared-data>
  docs/<bounded-help-and-localization>
  licences/<applicable-notices>
```

This is an envelope sketch, not the final allowed-path grammar. The generated manifest must enumerate every admitted file. Unknown critical requirements fail closed. Optional presentation fields may be ignored. Modules can serve multiple contributions only where their declared contract permits it. Platform variants are exact alternative artifacts, not scripts selecting arbitrary downloads at runtime.

Keep archive digest, manifest-byte digest and payload-file digests distinct. Signed catalogue metadata binds the expected artifact bytes. Retain the verified manifest after install so load/update/repair has authoritative data to compare. A version string is not an integrity check.

Use SemVer-style release and contract compatibility with an explicit parser grammar. Existing craft-ext versions require three numeric parts; prerelease/build syntax cannot be assumed supported without a negotiated format. Stable/preview channels can initially be metadata selections, avoiding silent changes to the parser.

## Operation descriptions are the shared base

The app engine's operation description is authoritative. A proposed logical record is:

```text
OperationDescriptor {
  stable_operation_id, contract_version,
  input_schema, result_schema, context_requirements,
  allowed_callers, effect_classification,
  capability_requirements_for(arguments, targets),
  transaction_or_job_behavior, cancellation_behavior,
  headless_support, artifact_roles, event_policy,
  workload_profile, documentation
}
```

Descriptors generate discovery, SDK bindings, CLI help/schema and conformance inventories. Actual domain implementations remain in the app. The descriptor's safety metadata is maintainer-owned, not asserted by plugin code.

Recommended source of truth: each app's typed Rust registry exports a versioned machine-readable inventory during development/release tooling. It generates all guest wrappers and public reference material. A schema artifact is generated from that registry, not maintained as a competing IDL. New command/query contracts use bounded policy-checked dispatch with typed wrappers; large media buffers retain purpose-specific handles or streaming paths. See [shared operations](shared-operations-and-apps.md#policy-checked-dispatch).

An operation may be a read query, transactional edit, persistent evaluation, resource import, artifact-producing job or external side effect. An export can read the document and write an output; an AI provider call can send content externally. Classify compound behavior rather than marking a function safe solely because its outer name looks harmless.

Inputs use validated semantic types and opaque handles: document/selection revisions, object IDs, user-selected file tokens, artifact destinations and job/resource handles. No private app pointers, unchecked caller-provided paths, shell fragments or another plugin's identity cross the boundary.

CLI execution and plugins share implementation, schemas, transactions and jobs. A user CLI may have authority from explicit command arguments/session policy that a plugin does not have. Every nested call retains the originating caller context. Plugins cannot use a caller-elevating CLI bridge or unchecked operation dispatch to regain user authority. A dispatcher applying the original principal and operation policy is the intended common route.

## Capability and explicit consent contract

A package declares permissions per app and contribution. The manager shows understandable purposes, affected scope and network/data effects before enablement. Each user explicitly accepts the requested set for that app. A capability name alone is insufficient; scope can mean current selection, current document, a selected input/output token, approved domain, or bounded plugin-private storage.

```text
effective_authority = declared_requirements
                  intersect explicit_app_grants
                  intersect host_supported_capabilities
                  intersect invocation_scope
                  intersect current_policy
```

The host derives the calling principal from the runtime instance. Host calls validate arguments, handle ownership, document revision, scope, budgets and live revocation. Handles cannot be forged into broader rights, survive a closed document without checks, or be passed to another plugin as implicit delegation.

Required rights must be granted before the dependent operation becomes eligible. Optional rights leave unrelated operations usable. Grouping the initial consent avoids per-call prompt fatigue; increased authority or scope requires explicit renewed consent. Denial is an ordinary structured outcome. Headless calls never approve rights or wait on a hidden dialog.

Grant continuity is concrete: authenticated catalogue policy must bind both releases to the same package UUID and enrolled source/publisher authority, or to an explicitly reviewed transfer that requires renewed consent. Compare normalized rights per contribution and app, including targets, domains, storage and budget profiles. Reuse only rights whose effective scope is unchanged or narrower. Renaming a contribution or introducing a new one does not inherit another contribution's grant. New executable behavior remains a review concern even when its declared authority is unchanged.

An unsigned local file with different bytes has no authenticated publisher continuity merely because it repeats a UUID. It requires explicit renewed setup/consent and cannot inherit catalogue grants. Reinstalling the exact same already approved local digest is idempotent. Optional future publisher signing needs trusted key enrolment, rotation and revocation; a signature claimed inside the same untrusted file is insufficient.

Avoid a single “all permissions” checkbox as an execution bypass. Powerful operations can have explicit broad scopes where justified, but they remain enumerated and enforced. Version updates can reuse an existing grant only under verified package/publisher continuity and unchanged effective authority. Account credentials are brokered host resources rather than strings read from the app's settings.

## Contribution families

Each family defines normal invocation, independent state, adverse outcomes and headless behavior. The following is the complete design scope; individual app availability is advertised, never presumed.

| Family | Contract and app ownership | Failure/lifecycle rule |
|---|---|---|
| Query/command | Typed targets/parameters, permitted queries and validated edit proposals | One transaction; stale/invalid/cancelled work rolls back; CLI invokes same operation |
| Panel/menu | Declarative controls/actions/localization/accessibility, stable command bindings | No discovery-time execution; close/unsubscribe disposes callbacks; UI-only status explicit |
| Pointer tool | Begin/update/commit/cancel, input capture, app-coordinate/selection context | Bounded events, transactional gesture; capture loss/cancel leaves coherent document |
| Canvas widget | App-owned render description, hit testing and bounded event callbacks | Saved instance state separate from panel/runtime; missing widget preserves fallback/state |
| Image effect/generator | Colour space/model, alpha, precision, bands/overlap, bounds, scale and selection | Validate output dimensions/data before apply; one instance cannot leak state to another |
| Temporal/video effect | Timebase, frame/layer dependencies, evaluated parameters, reset/seek and temporal state | Bound dependency requests; exact releases/resources pinned across preview/export |
| Geometry/live feature | Units, object schema, topology/bounds and appearance/rebuild dependencies | Validate geometry, invalidate relevant caches, preserve missing-feature state |
| Import/export | Host input tokens, bounded decoding/encoding options and artifact roles | Host writes destination; incomplete output never reported complete; import is transactional |
| Audio processor/instrument | Prepare/reset, sample rate/layout, blocks, sample-offset events, latency/tail and state | Dedicated scheduling/failure contract; no allocation/locking/host-dialog work on real-time callback |
| Resource | Typed payload/importer version, exact identity and readiness | Installation never invokes it; importer validates hostile data; lazy readers retain access |
| Workflow | Bounded graph/data, required nodes/models and typed input/output roles | Inspect readiness without execution; explicit executor permission and job for running |
| Provider/model job | Scoped input/output, backend/model requirements, progress/cancel and durable job identity | Declare remote data egress or offline support; credentials remain with broker |

Additional designed families cover file-format open/save handlers, settings pages, keymaps/shortcuts, localization packs, context-menu/inspector sections, document validators/preflight and headless-only operations. Declarative registration does not grant the underlying operation more rights. Format handlers use bounded input and host-owned document/output integration. Settings never expose account credentials. Keymaps respect app collision resolution and cannot bind hidden privileged commands. Localization text is bounded data, with trusted permission explanations remaining host-owned. Validators return structured findings; any proposed fix is a separately authorized transaction.

Plugin-to-plugin exported APIs and resident background services remain named future families. They require dependency/delegation, service-owner lifetime and budget contracts before becoming available. Headless-only operations are compiled commands, not approval of a new interpreted scripting runtime.

Full UI needs include custom canvas drawing, pointer tools and app layout integration. Prefer app-native declarative UI and validated draw lists. An arbitrary webview, native window handle or custom shader is not implied. Evaluate such a requirement as a versioned backend with independent confinement rather than quietly widening the panel contract.

## Runtime bindings and standards

Use the Rust registry and generated versioned inventory as the recommended authoritative interface definition. WIT provides interfaces, import/export worlds and resource handles for the WebAssembly Component Model and can be a generated future encoding if needed. It is not a second source of truth or proof that existing wasmi/core-WASM backends execute components. [WIT reference](https://component-model.bytecodealliance.org/design/wit.html).

First delivery preserves EffectCraft/FigCraft/PhotoCraft ABIs behind adapters. The inventory generates the supported core-WASM wrappers. A later backend experiment can determine whether component resources or another supported guest language justifies WIT generation and a component-capable engine. That experiment does not block the first SDK. Do not introduce a hand-maintained parallel serialization definition to claim standards adoption.

Contract compatibility must include the actual engine's allowed features/imports, target and resource rules. A package can carry separate app modules generated from one authoring workspace. A shared SDK does not require a universal effect ABI. Existing native/helper interfaces remain inventoried and need sandbox-compatible admission before becoming part of the new ordinary plugin path.

## Processing, state and migration

Every persistent feature stores its exact release, contribution, parameters and per-instance state schema/data. State is bounded, versioned and free of machine grants or credentials. The app owns save/reopen and explicit upgrade. A migration operates on a copy under ordinary sandbox limits, cannot exceed grants, and commits only after the app validates the result.

Runtime state declares whether an instance can be reused, reset, pooled or evaluated concurrently. Temporal processors define seek, frame-order and external dependency behavior. Audio declares sample-rate changes, latency/tail and transport discontinuities. Reset must be testable; instance pooling is an optimization after correctness, not an undocumented state-sharing contract.

Host data conversions are explicit and fixture-tested. A PhotoCraft straight-alpha colour-model band is not interchangeable with EffectCraft premultiplied RGBA. A generic pixel-buffer shape cannot hide conversion cost or precision changes. GPU paths declare supported formats, transfer ownership and fallback availability. CPU fallback cannot be invented for a GPU-only contribution.

## Jobs, events, artifacts and errors

An invocation result is Completed, Failed, Cancelled or a job owned by a persistent session. Jobs pin releases/resources, carry a captured document revision, expose bounded progress and support cancellation. The app rejects stale results or offers explicit reapplication. A one-shot CLI waits for required terminal artifacts.

Events have subscription ownership, sequence/session identity, rate limits and disposal. Avoid synchronous callback reentrancy into a transaction in progress. Plugin origin follows queued work, so delayed execution cannot escape its original scope.

Artifacts are typed output roles with validated byte/format bounds. The host resolves approved destinations and writes atomically where applicable. No plugin needs a raw user path simply to offer an exported image. Permission/input failure occurs before any avoidable external side effect.

Shared errors include UnsupportedContract, Disabled, MissingGrant, ResourceMissing, BackendUnavailable, InvalidInput, StaleDocument, BudgetExceeded, Cancelled, Revoked, IntegrityFailure and NeedsNewerClient. Stable machine codes coexist with plain-language app messages and safe remediation. Logs never substitute for a terminal error result.

## SDK acceptance

Future acceptance must create a real multi-app package from a template, generate repeated metadata, compare repeated packaging output, install its compiled asset, and invoke the same command through GUI and CLI. Denied/nested/parameter-dependent operations must fail without edits or unauthorized output. Save/reopen must preserve independent instances, and failed migration must preserve the old project.

Additional fixtures cover malformed module imports, unsupported required schema features, host profile mismatch, UI close/cancel/subscription disposal, stale handles/jobs, band/full-frame image equivalence, temporal reset, resource removal during use and audio deadlines. No fixture has run in this documentation draft.

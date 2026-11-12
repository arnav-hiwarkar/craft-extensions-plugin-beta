# Shared plugin functional model

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Status: proposed functional model, 2026-11-08. This combines the [primary investigation](plugin-functional-primary.md) and [independent Opus investigation](plugin-functional-opus.md). Their source revisions and limits are recorded there and in the [inventory](plugin-functional-inventory.json). No app was built or run. No benchmark, isolation guarantee, implementation approval, or rollout acceptance is claimed.

The user chose sandboxed executable plugins initially, official and third-party distribution, EffectCraft first, and FigCraft second. The present task is to settle functionality before the detailed safety discussion. See [the decision record](plugin-functional-decisions.md).

Subsequent context: the user accepted the recommended functional defaults, clarified CLI loading and package scope, and supplied Jeremy's existing craft-extensions handoff. The current [security proposal](plugin-security-design.md) starts from that implementation. Statements below about shared ownership are functional responsibilities, not a decision to duplicate or relocate craft-ext-core/store into craft-libs.

## 1. The shared unit is a package with contributions

A package is installed and updated. A contribution is something the app can use: an effect, command, panel, exporter, widget, preset, or provider. A document feature instance is one saved use of a contribution. A runtime instance is temporary code execution used to run a command or evaluate that feature.

For example, a package can contribute an EffectCraft effect and a FigCraft command. An EffectCraft composition can use its effect on five layers with different parameters. One export can require several runtime instances. Removing a panel does not remove saved effect instances. Updating a package does not imply changing an already-running export.

This is one management system, not one universal document model or algorithm interface. A package is cross-app only where it declares suitable contributions. Sharing the name or category does not make an existing EffectCraft module run in PhotoCraft.

## 2. What we agree and what we qualified

Both investigations recommend preserving current app contracts behind adapters, sharing package lifecycle and runtime mechanics, making headless initialization explicit, and recording project dependencies separately from installed state.

The synthesis qualifies several tempting simplifications:

- Processors are not universally stateless. Temporal effects, audio processors, and procedural features may need persistent state, reset rules, or dependencies.
- GPU access does not inherently exclude WASM. A host can expose compute operations. CPU fallback must be declared, not assumed to exist for every algorithm.
- A command bus is a common editor pattern, not a common vocabulary. CADCraft's prompt model, PDFCraft's tool table, and ArtCraft's provider jobs are different integration surfaces.
- An empty plugin-host search does not prove an app has no extension system. The matrix below labels incomplete discovery.
- Current native hosts are functionality to account for. They do not satisfy the new sandbox policy merely by existing or using another process.

## 3. What an author ships

The proposed package contains:

- Identity, publisher and release metadata, with supported app and interface requirements.
- A contribution list. Each entry has a stable ID, purpose, parameters, context requirements, execution mapping, output schema, and supported environments.
- One or more modules. A module can supply several contributions if its contract supports that.
- Assets such as icons, localization strings, presets, LUTs, samples, and optional documentation.
- Resource declarations for separately supplied model weights or large data. State whether they are mandatory, optional, bundled, or fetched.
- A versioned persistent-state description where the contribution stores more than parameters.
- Examples and fixtures showing input, output, and expected behavior.

The author SDK supplies serialization, bindings, entry-point glue, and structured errors. The author supplies domain behavior. The app supplies familiar controls, file output integration, undo, rendering, and document saving.

No dependency on private application Rust types is required. Existing SDKs and ABI exports remain usable through migration adapters. Source-only plugins that the user compiles locally, native vendor formats, plugin-to-plugin dependencies, and externally fetched executable modules are separate future scope choices.

## 4. Contribution contracts we need

| Contribution | Input from the host | Output and integration |
|---|---|---|
| Command or automation | Parameters, document/selection context, query and edit operations | Result data and an editor transaction |
| Image filter or generator | Samples, colour/alpha description, bounds, overlap, scale, parameters | Validated image output, applied destructively or stored as a live feature |
| Video or temporal effect | Frames, time, render scale, parameter values, declared frame dependencies | Frame output and bounded requests for other inputs |
| Geometry processor | Objects, units, selection, parameters | Validated geometry changes or a live appearance result |
| Importer/exporter | Input bytes or document representation, options, output target description | Document data or output artifacts that the host writes |
| Panel or tool | Declarative controls, callback values, pointer/selection events | UI updates, command invocations, and gesture transactions |
| Persistent widget/procedural feature | Stored instance state, parameters, evaluation context | Renderable or rebuildable content plus declared saved state |
| Audio processor/instrument | Prepared sample rate, blocks, channel layout, transport and event offsets | Samples, latency/tail reporting, parameters, state |
| Resource package | Typed assets and metadata | Host-owned presets, brushes, templates, or other content |
| AI/provider integration | Inputs, model/options, job context, result destinations | Progress and media/data artifacts through a job lifecycle |

An app advertises only contracts it implements. A command-capable integration does not automatically support rendering a new effect, registering spreadsheet functions, adding a codec, or supplying a new solver.

Custom GridCraft functions need recalculation and dependency rules. Real-time SoundCraft processing needs a dedicated scheduling contract. CAD prompt-driven tools need explicit headless parameters or a declared interactive path. These cannot be inferred from a generic command callback.

## 5. Common systems and app-owned systems

| Shared in craft-libs | Supplied by each app |
|---|---|
| Package and contribution identity, manifests and version requirements | Actual supported contribution kinds and versions |
| Official and third-party source normalization, package and resource retrieval | App-specific distribution entry points and presentation |
| Store records, install/update/remove lifecycle, metadata discovery | Per-app configuration and activation policy |
| Runtime compilation/instance mechanics and job status vocabulary | When and where to invoke each contribution |
| Parameter and declarative UI descriptions | Rendering controls with the app's widgets, theme and localization |
| Context and artifact envelope conventions | Colour, geometry, time, document or audio data conversion |
| Common invocation/result/error conventions | App command names, schemas, transaction and undo behavior |
| Resource readiness and compute interfaces | Available GPU operations, model services and workload scheduling |
| Reference and state envelopes | Serialization into the app's project format and missing-feature behavior |
| SDK, validators and adapter fixtures | App-specific conformance and save/reopen fixtures |

These are module responsibilities, not an approved crate list. craft-libs must not depend on app crates. Each app builds the shared libraries into its GUI and CLI hosts. A separate central daemon is not required by this model.

## 6. Functional path from author to user

1. The author chooses supported app contracts and implements contributions using the SDK.
2. The author builds modules and packages metadata, resources, and examples. Validation checks functional completeness.
3. An official or third-party source describes the same normalized package and releases. Source identity remains visible.
4. The manager discovers metadata and shows which contributions work in the current app and environment.
5. Installation makes the package available. Resource readiness and app activation remain explicit states.
6. The app registers supported contributions in its own effect browser, command list, format registry, or extension UI.
7. The user invokes a contribution or creates a persistent document feature from it.
8. The app resolves parameters, context, resources, and the selected runtime, then invokes the contribution.
9. The app integrates validated output through its existing rendering, document, export, or job system.
10. Saving records the required contribution reference and instance data when the result remains dependent on it.
11. A later session resolves those references and evaluates features using compatible installed code. Missing requirements remain visible and preserve saved state.

Detailed source trust, permission UX, and installation safety mechanisms remain in the later safety discussion. Supporting third-party distribution is agreed; the exact third-party catalog protocol is not.

## 7. EffectCraft journey

An author implements a pixel effect and parameter schema. An installation exposes its effect ID through the app's existing effect registry. The user applies it to a layer. The app creates ordinary properties for its controls, so keyframes and expressions remain host features.

For each evaluation, the app supplies parameter values at the requested time, input pixels, scale, and the context supported by that effect API. The plugin returns pixel output. A WASM ABI v1 effect must not be offered richer native/helper services it does not implement.

The layer's saved effect instance retains the contribution ID, parameters, and proposed package/version reference. Reopening resolves the reference before required rendering. Headless export uses the same resolution and evaluation path, not a parallel plugin renderer.

Both investigations found a source-derived functional gap: the current CLI session initializes loader hooks but does not perform the desktop installed-plugin loading sequence. The inspected renderer omits unresolved effect IDs from its stack. This combination needs a runtime reproduction before it is reported as a confirmed defect. The proposed model requires dependency checks before final export so missing processing cannot silently change the result.

EffectCraft also has a helper protocol for requesting other frames, other layers, and parameter samples. That is relevant functionality for temporal contracts, while approval of a sandboxed implementation remains separate.

## 8. FigCraft journey

An author declares a command and typed parameters, optionally adding a panel or widget. Installation makes its command contribution discoverable. A menu form collects parameters; CLI callers supply them directly.

The module queries the editor or requests commands through the app adapter. The app retains ownership of the document and groups the run into an undo transaction. A panel can stay open and invoke the operation repeatedly; each callback has its own coherent transaction. A widget additionally stores instance data in the document.

Current FigCraft panels use host-rendered declarative controls and keep a runtime instance alive while open. Its CLI script runner starts with a fresh session, and its headless backend drops UI requests. In particular, the current offered-file callback queues a save dialog that programmatic callers drop. Shared functionality therefore needs a headless artifact destination and explicit panel/headless behavior, not merely the existing callback names.

The proposed default is to expose the useful operation separately from its UI. A panel's Arrange button and a CLI Arrange call use the same operation. Truly UI-only contributions can exist if they declare that limitation.

## 9. Per-app functional integration requirements

| App/group | Required adaptation and current evidence |
|---|---|
| EffectCraft | Existing effects and library. Adapt layer/time context, controls, frame services, project references and CLI bootstrap. First integration. |
| FigCraft | Existing commands/panels/widgets and server directories. Adapt editor transactions, selection/events, saved widget data and headless artifacts. Second integration. |
| PhotoCraft | Existing WASM filters. Preserve bands, colour models, alpha, selection and overlap; integrate smart/destructive filter usage. |
| VectorCraft | Existing object filters, live effects and exporters. Preserve object schemas, units, appearance state and format output. |
| SoundCraft | Existing native AU/VST3/CLAP hosts. Audio blocks, events, latency, state and editors need a dedicated sandboxed execution design. Not automatically included in the WASM pilot. |
| FilmCraft | Command and CLI entry points inspected. Adapt clip/sequence actions first; processing needs frame, transition and audio contracts. Executable plugin support not established in this survey. |
| LightCraft | Catalog/develop command and job paths inspected. Preserve photo selection, RAW/develop stage placement, nondestructive state and exports. |
| PDFCraft | Document facade and automation table inspected. Adapt page/object operations, document history and artifact writing. |
| DesignCraft | Command/script entry points inspected. Preserve stories, frames, pages, spreads, units, layout and export. |
| CADCraft | Programmatic and prompt-based command paths inspected. Preserve entities, units, transactions and interactive input semantics. |
| SolidCraft | Current preferences list manifests; its executable runtime is described as future work. Commands and persistent parametric-feature evaluation need implementation. |
| TextureCraft | Session and command paths inspected. Adapt mesh/UV/material context, texture-set channels, bake/composite invalidation and strokes. |
| WordCraft | Command/script paths inspected. Adapt text ranges, styles, stories and layout. |
| GridCraft | Command/macro paths inspected. Adapt ranges and workbook operations; new functions require a distinct calculation contract. |
| DeckCraft | Command/CLI paths inspected. Adapt slides, shapes, text and export; playback extensions require new lifecycle work. |
| ArtCraft native | Current Rust egui target and durable provider-job code inspected. Generation/provider contributions need task submission, result/media ingestion and persistence. No plugin or CLI host parity was established. |

This is a functional survey with deeper traces in the first four apps. It is not a commitment that every suggested extension kind can be added through a small adapter.

## 10. CLI, automation, and jobs

Distinguish a fresh one-shot process, a persistent headless server, and a client attached to a running editor. They share invocation contracts, but their session lifetime and ownership differ.

For a fresh process: initialize the chosen plugin environment, discover installed metadata, parse/open the project, resolve its required contributions, load the needed modules, execute commands, await necessary jobs, save/export, and shut down. Management operations do not require a document. Merely loading a module does not persist edits between separate CLI invocations.

For an attached client: the running app owns its plugin environment, documents, active versions, callbacks and jobs. The CLI must not accidentally execute against a fresh local session when the user expects to control that app.

For a persistent server: commands can share a document and jobs across requests. A returned asynchronous job ID must have such a surviving owner. Otherwise a one-shot CLI waits for the operation needed to produce its output.

Common management intents include source listing, search, compatibility/contribution inspection, install, update, enable/disable, dependency inspection, and removal. Execution intents include invoking a command, applying a persistent feature, exporting a contributed format, and observing/canceling jobs. Final command names and flags are not chosen. Existing `run`/`exec` paths should wrap the same host operations rather than become a second implementation.

CLI results need values, logs, warnings, output artifacts, and terminal job status. Inputs needed from a user become a structured request or error. A hidden form must not leave headless execution waiting. Multiple output files need declared destinations or an output directory, not an invisible save dialog.

## 11. Resources, performance, and persistence

Separate module readiness, model/resource readiness, and hardware/backend availability. A plugin can be installed while its optional model is absent. Report the missing condition precisely.

Resources are resolved by the host. Compute contributions request supported operations and receive results through declared data contracts. The host owns scheduling, transfer formats, progress, cancellation, and result integration. CPU and GPU paths can coexist without pretending they are always interchangeable.

Compile and cache only where identity and state make reuse valid. Cache keys include package digest, contract, parameters, relevant inputs, and declared context. Stateful panels and temporal effects require different pooling/reset rules from fresh per-band filters. No performance winner is claimed without measurements.

Keep package data, per-app preferences, per-document instance state, process state, and regenerable caches separate. Saved features identify dependencies. One-shot edits normally leave ordinary native objects that no longer require plugin execution. Persistent features still require evaluation or a declared baked fallback.

## 12. Functional build sequence and acceptance

1. Define package/contribution metadata and management operations around one EffectCraft effect. Preserve legacy IDs and existing commands.
2. Connect desktop and headless initialization, dependency resolution, parameter controls, rendering, saving/reopening, and export in that app.
3. Add FigCraft's command and panel operation using its existing ABI through an adapter. Prove document transactions and headless file outputs.
4. Consolidate runtime mechanics and shared UI only where both integrations establish the actual common requirements.
5. Expand image/geometry contracts to PhotoCraft and VectorCraft, then evaluate the other workload families separately.

Acceptance must demonstrate a community-built package installed from an official and a third-party source; a saved EffectCraft feature evaluated correctly in desktop and headless export; a FigCraft operation invoked from both panel and CLI with equivalent document results; saved independent instance state; observable job completion; and precise unsupported/missing-resource results. These checks have not run yet.

## Functional decisions for the next discussion

1. Require a headless-callable operation for every editing/compute contribution, with UI-only contributions explicitly separate? Recommendation: yes.
2. Preserve current app ABIs behind adapters before designing standardized cross-app purpose contracts? Recommendation: yes.
3. Allow one package to contain several app-specific contributions and modules? Recommendation: yes.
4. Which plugin environment should a fresh CLI use by default: installed compatible packages or an explicit isolated/project environment?
5. Package existing command scripts/macros and data presets initially, or start with executable contributions only?
6. Add persistent widgets and live features in the FigCraft pilot, or prove one-shot commands and panel callbacks first?

These questions concern functional scope. The detailed permission, sandbox, source-trust and update-safety design follows after this model is agreed.

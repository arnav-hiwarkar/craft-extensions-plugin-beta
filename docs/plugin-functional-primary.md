# Primary functional investigation

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Recorded on 2026-11-08, independently of the new Opus investigation. This is source inspection, not runtime acceptance. No app was built or run. The [source inventory](plugin-functional-inventory.json) records 15 editor app revisions and entry points, plus a subsequent check of ArtCraft's native target. The [decision record](plugin-functional-decisions.md) governs scope: sandboxed initially, official and third-party distribution, EffectCraft first and FigCraft second. Detailed safety design follows this functional study.

## Findings that change the earlier model

1. A package, a contributed feature, a saved feature instance, and a running module are different things. One package can supply several effects. One effect can appear on many layers with independent parameters. Rendering can require several runtime instances. Installation state cannot represent all of those lifetimes.
2. The suite already has multiple plugin families. Sharing package management must preserve those contracts before attempting to standardize new ones.
3. CLI processes do not automatically inherit desktop session state. Shared initialization and explicit session modes are functional requirements.
4. EffectCraft already has a helper-process effect protocol in addition to WASM effects. SoundCraft already has native audio-format hosts. These are existing systems to inventory, not proof of compliance with the newly chosen sandbox requirement.
5. SolidCraft's plugin preferences currently enumerate manifests and disabled IDs; its own source says they do not execute plugins. The richer WIT and parametric-plugin document is a design, not implemented runtime evidence.

## Actual extension and application contracts

EffectCraft's [host](https://github.com/arnav-hiwarkar/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/host/src/lib.rs) wires plugin loader and preview hooks. Its [effect interface](https://github.com/arnav-hiwarkar/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/effects/src/plugin.rs) turns a contribution into a normal effect with controls and time-dependent rendering. Its [helper host](https://github.com/arnav-hiwarkar/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/helper-host/src/lib.rs) registers externally processed effects into the same registry. The helper protocol supports another-time inputs, layer inputs, parameter sampling, and instance destruction. WASM ABI v1 does not provide all those services.

FigCraft's [plugin engine](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/crates/engine/src/extensions/plugins.rs) runs commands and panel callbacks as document transactions. Its [sandbox host interface](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/crates/plugin/src/sandbox.rs) exposes calls, selection, notifications, logs, settings, and offered file outputs. The app owns the actual editor operations and UI requests.

PhotoCraft's [application path](https://github.com/storytold/photocraft/blob/655295b2a2c400bded0640f5d97367000a057b2e/crates/plugins/src/apply.rs) handles the layer surface, colour-mode samples, alpha, selection blending, processing bands, and neighbourhood overlap. A generic RGBA buffer would not preserve its complete contract.

VectorCraft has separate [object changes](https://github.com/storytold/vectorcraft/blob/065a646799f1f20f5454845d6648e09a5252c452/crates/plugins/src/objects.rs), [live effects](https://github.com/storytold/vectorcraft/blob/065a646799f1f20f5454845d6648e09a5252c452/crates/plugins/src/effect.rs), and [exporters](https://github.com/storytold/vectorcraft/blob/065a646799f1f20f5454845d6648e09a5252c452/crates/plugins/src/export.rs). An exporter returns file data for the app to write. A live effect participates in appearance evaluation; it is not merely an editing command.

SoundCraft's [DSP interface](https://github.com/storytold/soundcraft/blob/da02c19fcca8/crates/dsp/src/lib.rs) includes prepare, reset, parameters, audio processing, transport, MIDI, latency, tails, state blobs, and editor handles. The [CLAP host](https://github.com/storytold/soundcraft/blob/da02c19fcca8/crates/clap-host/src/lib.rs) explicitly loads native binaries in-process, has web stubs, and exposes hosted instances through that DSP interface. A new sandboxed backend must preserve those audio semantics and cannot be declared finished by putting the scanner in another process.

## Per-app integration inventory

In the remaining apps, the survey established engine and CLI entry points. It did not establish a complete current plugin runtime. Proposed contribution types below are integration candidates, not implemented capabilities.

| App | Observed foundation | App-owned functional work |
|---|---|---|
| EffectCraft | WASM effects, helper protocol, package library, CLI and host hooks | Layer/time context, controls and keyframes, effect registration, frame requests, project dependencies, headless loading |
| FigCraft | WASM commands, panels, widgets, server directories, CLI command scripts | Document queries and transactions, selection, callback lifecycle, canvas widgets, preference persistence, headless outputs |
| PhotoCraft | WASM filters over surface bands | Colour and alpha conversion, overlap, selection, filter dialogs, surface changes, preview and save semantics |
| VectorCraft | Object filters, live geometry effects, exporters | Object schemas and units, appearance evaluation, output admission, export integration, cache invalidation |
| SoundCraft | AU, VST3, and CLAP adapters | Audio blocks and transport, instrument events, latency compensation, parameter automation, opaque state, editor lifecycle, new sandboxed execution |
| FilmCraft | Command-driven timeline engine and CLI | Clip and sequence operations initially; explicit frames/time/audio contracts for processing plugins later |
| LightCraft | Catalog/develop commands and background render jobs | Catalog targets, develop parameters, RAW/develop stage placement, nondestructive recipes, export behavior |
| PDFCraft | Open-document facade and automation tool table | Page/object queries and edits, document history, PDF writes, import/export output; no plugin object-model interface established yet |
| DesignCraft | Commands, scripts, and journaled pointer tools | Stories, frames, pages/spreads, units, layout/reflow, prepress and export lifecycle |
| CADCraft | Programmatic commands plus interactive prompts | Entity/model units, explicit command parameters for headless calls, transactions; interactive tools need a separate prompt/input path |
| SolidCraft | Manifest listing and disabled IDs; runtime described as future work | Command contribution first; parametric feature evaluation and stored fallback require implementation |
| TextureCraft | Project/mesh/texture-set session, caches, stroke state, command registry | Material channels, mesh and UV context, texture-set targets, bake/composite dependencies, stroke transactions |
| WordCraft | Command-backed text/document editing | Text ranges, styles, stories and layout; durable positions and edit semantics rather than raw string offsets |
| GridCraft | Command-backed workbook engine | Ranges, cell types, recalculation and dependencies; custom functions need a calculation contract beyond command automation |
| DeckCraft | Command-backed presentation engine and CLI | Slides, shapes, text, transitions and export; slideshow callbacks would need a separate declared lifecycle |

Post-investigation clarification: the `artcraft` repository is an AI image/video creation app, not evidence of a suite launcher. Its [current workspace](https://github.com/storytold/artcraft/blob/1f969a9224934a55417daf10359a6fb1458b63f4/Cargo.toml) includes `artcraft-native`, a Rust egui target. Its [provider jobs](https://github.com/storytold/artcraft/blob/1f969a9224934a55417daf10359a6fb1458b63f4/artcraft-native/src/backend/providers/jobs.rs) persist generation and result-upload progress. Proposed plugin integration there would need model/provider operations, durable job handling, and media-result ingestion. A plugin host, common command dispatcher, and CLI integration have not been established by this check. Any separate suite launcher could consume manager UI, but that does not provide execution inside the editors.

## What authors supply

A community or first-party plugin supplies a package manifest, one or more declared contributions, executable modules where needed, control and parameter schemas, versioned saved-state schemas, resources, and functional test fixtures. Each contribution names a supported app interface, such as an EffectCraft effect or a FigCraft command. Those names are illustrative contracts, not finalized identifiers.

Resources include presets, icons, language strings, model weights, lookup tables, samples, and optional documentation. Declare whether each is bundled or separately fetched, its identity and size, and whether it is needed for inspection, execution, or only an optional feature. A missing model must produce a resource-needed state rather than masquerading as an unsupported plugin. Author dependencies compiled into a module are different from resources fetched by the host and dependencies on another plugin.

The SDK handles serialization and entry-point glue. The author implements the algorithm or command behavior. The app renders ordinary parameter controls, hosts declarative panels, and owns file writing. Headless-capable contributions must accept explicit parameters and produce data without requiring a user to press a panel button.

## Shared systems to build or extract

- Package identity, versions, sources, discovery, installation records, update and resource retrieval. Official and third-party providers feed one common metadata model while preserving source identity.
- Contract and feature discovery. Distinguish app support, execution availability, resource readiness, and instance compatibility.
- Runtime/module lifecycle and job reporting. Separate module compilation from contribution registration and instance evaluation. Provide progress, cancellation, logs, errors, and shutdown.
- Parameter and declarative UI definitions, with host-provided rendering and event delivery.
- Resource and compute interfaces for bounded binary inputs, outputs, and optional GPU operations. Preserve workload-specific formats.
- Per-app activation and preferences plus versioned project dependency references and instance state envelopes.
- Common management operations exposed through the app command system. CLI, desktop controls, and automation wrap the same operations.
- Author SDK and test harness, host adapter conformance fixtures, diagnostics, and capability descriptions.

These are responsibilities, not a commitment to one crate per item. App document types never become dependencies of craft-libs.

## Three functional lifetimes

### One-shot command

The user or CLI names a command and explicit targets. The app resolves the contribution, captures context, resolves parameters and resources, then executes. The plugin queries allowed app operations or returns a proposed edit. The app validates and commits a transaction, refreshes selection and caches, reports output, and saves only when requested. A failed operation has no successful document-edit result.

### Persistent feature

An effect on a layer, a procedural widget, or a parametric object stores a contribution reference and independent instance parameters. The app evaluates it on demand during rendering or rebuilding. The app supplies the current time, scale, geometry, or block context according to the contract. Instance state must remain independent across duplicate layers, previews, and concurrent exports. The document owns the saved feature; a process instance is temporary.

### Long-running interaction or job

A panel stays available until closed. Callbacks consume input state and return panel updates or document transactions. A canvas tool has begin/update/commit/cancel semantics. A heavy job has an identity, progress, result, and terminal status. UI and CLI can observe the same job while providing different input surfaces. A plugin may have a compute contribution and a panel that invokes it, rather than hiding its only functionality inside GUI code.

## CLI and automation requirements

There are three distinct session modes:

1. A new headless process initializes a session, resolves the selected plugin environment, opens documents, loads needed contributions, executes a command sequence, awaits jobs, saves or exports explicitly, then exits.
2. A connection to an already-running app uses that app's session, installed versions, open documents, and jobs. Client-local plugin state must not be mistaken for app state.
3. A persistent headless server keeps a session across many requests and owns runtime instances until shutdown.

The current [EffectCraft CLI](https://github.com/arnav-hiwarkar/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/apps/effectcraft-cli/src/main.rs) creates a host session but does not run the desktop's installed-plugin startup sequence in its `session` initializer. The [VectorCraft CLI](https://github.com/storytold/vectorcraft/blob/065a646799f1f20f5454845d6648e09a5252c452/apps/vectorcraft-cli/src/main.rs) explicitly attaches persisted plugin stores in `with_stores`. The [FigCraft headless backend](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/crates/mcp/src/headless.rs) clears UI requests after commands. Its CLI script runner starts a fresh session and runs a sequence together. These source differences prevent assuming automatic GUI/CLI parity.

Provide common management intents: list sources and packages, inspect compatibility and contribution schemas, install or update, enable or disable, inspect dependencies and jobs, and remove. Do not invent a second execution engine for convenient CLI subcommands. Existing app commands such as `effect.apply` or `plugin.run` remain app adapters' entry points.

For a one-shot CLI, awaited completion must be the default for work needed to save an output. An emitted job ID is useful only if a persistent owner remains alive. FilmCraft already handles this distinction for long commands in its CLI backend. Missing user input must return a structured input-required result, never wait forever on a hidden dialog. UI-only panels can be inspected or report unavailable; they do not silently pretend to have run interactively.

## Two target functional journeys

### EffectCraft effect

An author builds a WASM effect and describes controls. A source lists its package. Installation makes it resolvable, then the adapter registers its effect ID. Applying it creates an effect instance on a layer. Keyframes change host-owned parameters. Rendering supplies evaluated parameters and frame context. Saving preserves the feature and dependency. Reopening resolves that dependency before required evaluation. A headless export performs the same resolution and frame evaluation with explicit output and completion reporting. No new app build is needed for another compatible effect.

### FigCraft command with an optional panel

An author declares a command, parameters, and editor operations it requires. Installation registers a namespaced contribution. Invoking it from the menu opens a parameter form or directly calls its command. A panel can invoke the same underlying operation repeatedly. Each callback's edits form an undo transaction. CLI execution supplies the command parameters directly and receives output plus updated document state. Opening the panel itself is not required for computing the result. Widget contributions additionally require saved instance data and evaluation/render hooks.

## Functional choices to discuss after synthesis

1. Require every compute or editing contribution to expose a headless operation, while allowing separate UI-only contributions?
2. Preserve existing app ABIs behind adapters first, then standardize new purpose-specific interfaces where at least two apps agree?
3. Make installed compatible plugins available to headless runs by default, with an explicit isolated-environment option?
4. Treat one-shot commands and persistent document features as separate contribution kinds?
5. Route GPU, model, and external resource requests through host-owned compute/resource interfaces instead of author-specific installation logic?

The independent study and synthesis may refine these recommendations. Implementation and the detailed safety review are not authorized by this investigation.

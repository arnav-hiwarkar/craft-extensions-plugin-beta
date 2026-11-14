# Compiled hosts and lessons from Blender extensions

Research date: 2026-11-14. This explanation asks which explicit extension hooks make a compiled graphics application programmable, and which lessons apply to Craft's beta SDK. It proposes requirements, not an implementation plan or release promise.

## Evidence and limits

Blender's versioned 4.5 manual and Python API are the documentation baseline, not a claim that 4.5 is the latest release. These URLs were retrieved on the research date. Versioned documentation can still change. The source anchor is Blender tag `v4.5.0`, resolved through the [official repository reference](https://api.github.com/repos/blender/blender/git/ref/tags/v4.5.0) to `8cb6b388974a817afedf1317ce26f0c75aa5f181`.

| Evidence class | Meaning in this document |
|---|---|
| Inspected source | A small source excerpt read at a pinned revision. This supports structural claims, not runtime behavior. |
| Official documented behavior | Blender's manual, API reference, and examples. Examples were read, never executed. |
| Vendor feature claim | A capability asserted by its publisher. No proprietary algorithm, performance result, or installation success is inferred. |
| Craft baseline | Existing beta documentation and the task's supplied revision `3c7e06f`. No fresh adapter source audit is claimed here. |
| Recommended target | A requirement derived from the comparison. It is not an existing API. |
| Unknown | Something these sources and this read-only investigation cannot establish. |

Craft references are [capabilities](capabilities.md), [author guide](author-guide.md), [UI](ui.md), [architecture](how-it-works.md), and [framework integration](framework-integration.md). The [existing backlog](community-comparison-and-backlog.md) already covers docking, jobs and media, masks, motion, private settings, control groups, author tools, services, themes, and rendering depth. The requirements below add contract details and acceptance cases to that work. They do not rediscover those gaps.

Web-tool retrieval failed for Blender pages; direct HTTPS retrieval succeeded. No app, plugin, installer, build, or test was run. Remote text was treated as evidence only. This note does not analyze PluginCraft or the public EffectCraft release loader.

## A compiled application can expose runtime extension points

A compiled executable can load a native shared library and exchange functions through a C application binary interface, or ABI. A stable ABI specifies calling conventions, value layouts, ownership, and supported versions. Loading code into the host process normally gives that code the host's process privileges and address space. A bad pointer or incompatible binary can crash or corrupt the host. A C ABI does not provide isolation.

The bounded example is OpenFX. Its [official core header](https://github.com/AcademySoftwareFoundation/openfx/blob/4249fe1b590a7e65caf4b1e4693b6b53220703f5/include/ofxCore.h) declares C-compatible interfaces, plugin entry callbacks, and an `OfxHost.fetchSuite` callback that requests named, versioned host suites. An unavailable suite can return null. The inspected revision is `4249fe1b590a7e65caf4b1e4693b6b53220703f5`, resolved from the official repository on the research date. This establishes an interface pattern; it does not establish OpenFX support in Craft or compatibility with any particular commercial plugin.

Embedding an interpreter is another option. Blender's [Python overview](https://docs.blender.org/api/4.5/info_overview.html) describes an embedded interpreter with `bpy` access to application data, classes, and functions. The pinned [`bpy.cc`](https://github.com/blender/blender/blob/8cb6b388974a817afedf1317ce26f0c75aa5f181/source/blender/python/intern/bpy.cc) is C++ code connecting Python, RNA, operators, properties, and other host modules. RNA is Blender's structured data and property interface. This is evidence of a compiled C/C++ application exposing Python hooks, not Python replacing its engine.

A host can also execute sandboxed WebAssembly with an explicit import list, or supervise a separate helper through versioned interprocess communication, or IPC. Craft already uses those two arrangements. None requires recompiling the host for each plugin when the needed hooks already exist. Adding a missing engine operation, evaluator, file-format integration, or UI slot can still require a host change.

| Extension mechanism | Useful property | Cost or boundary | Craft implication |
|---|---|---|---|
| Native shared library, C ABI | Direct callbacks and access to host-provided suites | Process privileges, crash coupling, platform ABI maintenance | A possible architecture, not a beta feature or proposed default |
| Embedded Python | Concise authoring and direct object-model access | Interpreter and dependency maintenance; access is not automatically sandboxed | Learn from Blender's typed model without promising embedded Python |
| Sandboxed Wasm | Explicit imports and bounded host mediation | Only exposed operations are reachable; copies and budgets matter | Preserve the beta's closed adapter contracts |
| Supervised helper process | Separate address space and independent libraries | OS access remains broad without confinement; IPC and lifecycle costs | Keep consent, protocol versions, and host-call authorization distinct |

There is no universally preferred row. A pixel filter, an editor panel, and a model-assisted tool have different data, latency, and trust requirements.

## Blender separates packaging from application behavior

The [extension package guide](https://docs.blender.org/manual/en/4.5/advanced/extensions/getting_started.html) defines a ZIP archive with `blender_manifest.toml`. An add-on needs Python entry code, commonly `__init__.py`; a theme uses a theme XML file. The manifest records package identity, version, license, minimum Blender version, optional exclusive maximum version, supported platforms, wheels, and resource permission reasons. Package format version, package version, and Blender compatibility are separate fields.

The [extension command reference](https://docs.blender.org/manual/en/4.5/advanced/command_line/extension_arguments.html) documents package build and validation commands, installation, and repository listing generation. Building an extension packages it; it does not rebuild Blender. Validation checks package requirements, not semantic correctness or absence of harmful code. Publication on Blender's platform includes review according to the package guide, but this note makes no claim about review effectiveness.

The [add-on migration guide](https://docs.blender.org/manual/en/4.5/advanced/extensions/addons.html) explains repository-qualified package namespaces and relative imports. That reduces collisions when repositories contain identically named extensions. It also documents extension-specific writable storage through `bpy.utils.extension_path_user`, rather than assuming the installed package directory is writable. Craft should preserve the separation between immutable release bytes, private local preferences, and project data already identified in its backlog.

The [wheel guide](https://docs.blender.org/manual/en/4.5/advanced/extensions/python_wheels.html) supports bundled Python dependencies, requires their dependencies to accompany them, and discusses matching binary compatibility tags. It documents platform-specific package splitting to reduce archive size. Its policy discussion explicitly leaves some dependency-conflict guidance unfinished. A wheel can include native binaries; Python packaging therefore does not remove platform, architecture, Python ABI, license, or native-code risks. These docs do not prove arbitrary native wheels work across Blender versions.

For Craft, dependency metadata should distinguish a package's contract support from its helper's operating-system support. A compatible panel tree does not prove a bundled helper can start. This extends NATIVE-SETUP and DISCOVERY with separate compatibility failures, rather than adding another installer mechanism.

## Blender's useful power comes from explicit host hooks

The Python API distinguishes data access from actions performed in an editor context. [`Panel`](https://docs.blender.org/api/4.5/bpy.types.Panel.html) subclasses declare an editor space, region, and optional context. Their drawing code builds a host UI layout. [`Operator`](https://docs.blender.org/api/4.5/bpy.types.Operator.html) subclasses supply actions, properties, execution, invocation, polling, and optional modal behavior. Polling determines whether an action applies in the current context. Calling an operator is therefore not equivalent to calling a context-free data function.

[`PropertyGroup`](https://docs.blender.org/api/4.5/bpy.types.PropertyGroup.html) supplies structured custom properties attached to Blender data. Property values can be saved, while Python class definitions must be registered again. This distinction matters when an extension is absent: persisted values do not themselves reconstruct the behavior or UI that understands them.

The [`Depsgraph` examples](https://docs.blender.org/api/4.5/bpy.types.Depsgraph.html) distinguish original data from evaluated data with animation, modifiers, and constraints applied. Evaluation can be expensive. Temporary geometry has explicit ownership and cleanup requirements. Craft's adapters need equivalent clarity wherever a future operation returns source versus rendered data; copying Blender's vocabulary alone would not establish the semantics.

Other hooks have specific scopes:

| Documented Blender hook | What the evidence supports | Boundary for Craft comparison |
|---|---|---|
| [`Node`](https://docs.blender.org/api/4.5/bpy.types.Node.html) registration, sockets, drawing, and update callbacks | Extensions can participate in documented node-tree abstractions | Registration does not prove arbitrary Python nodes execute inside every built-in evaluator |
| Depsgraph geometry access | Exporters and tools can obtain evaluated geometry | Does not imply Craft has mesh objects or a geometry evaluator |
| [`RenderEngine`](https://docs.blender.org/api/4.5/bpy.types.RenderEngine.html) callbacks | A registered engine can supply final rendering and viewport updates through explicit hooks | Does not imply a beta raster filter can replace Craft's renderer |
| Operator actions and data access | Tools can transform data and form import/export workflows | Each Craft app still needs canonical import/export operations and supported formats |

No general node-provider ABI or arbitrary Geometry Nodes kernel injection is established by the inspected `Node` reference. This note makes no such claim. Custom properties also differ from inventing a new native database type. Such features require separate evidence and engine support.

Blender's [`gpu` module](https://docs.blender.org/api/4.5/gpu.html) documents batches, buffers, shaders, offscreen rendering, and viewport draw-handler examples. GPU resources have context lifetimes; the offscreen warning ties resource validity to its graphics context. These are explicit drawing hooks. They are neither unrestricted access to every UI region nor evidence that all extensions can supply arbitrary widgets.

Craft's host-rendered UI 1.0 and 1.1 trees, overlays, and component fallbacks remain the contract. New UI slots should identify placement and lifecycle without exposing the renderer backend. A richer draw vocabulary would need resource quotas and cleanup semantics; it must not silently turn panel data into executable GPU code.

## Lifecycle, transactions, and permissions require separate contracts

Blender's [threading guidance](https://docs.blender.org/api/4.5/info_gotchas_threading.html) warns that Python threads can cause crashes and says Python integration is not thread safe. It recommends independent processes for independent Python work. This is a concrete limit on copying the apparent convenience of direct host access.

[Application handlers](https://docs.blender.org/api/4.5/bpy.app.handlers.html) expose loading, saving, dependency updates, rendering, and undo events. The docs warn that data mutation during rendering can conflict with viewport access. Handlers are normally removed on file load unless marked persistent. [Timers](https://docs.blender.org/api/4.5/bpy.app.timers.html) schedule callbacks and can also persist across file loads. Scheduling a callback does not make it an isolated worker or an atomic transaction.

The Operator reference documents undo participation and cautions about cancellation after changes. Craft should keep mutation on the canonical document owner, define rollback explicitly, and commit asynchronous results in a fresh authorized transaction. An external file write remains an external effect even if document changes roll back. PhotoCraft already documents partial file publication; a common SDK must retain that distinction.

Blender's manifest lists files, network, clipboard, camera, and microphone permission reasons. The add-on guide requires network users to check `bpy.app.online_access`, then explicitly states that Blender cannot prevent add-ons from violating that setting. Thus declarations and platform policy are not proof of an OS sandbox or enforced destination filtering.

Craft's Wasm lane currently exposes no raw paths, shell, process, network, or clipboard API. Declaring a network permission does not create a network operation. Native helpers run out of process with full OS access after per-version consent. Their adapter calls still require declared rights, live grants, and captured scope. Process separation limits address-space coupling; it does not make native access permission-scoped at the OS level.

## Recommended contract additions and acceptance cases

These are target requirements. Current coverage comes from the beta documentation at the supplied revision: shared core, store, host, UI, CLI, and guest SDK; app adapters with 32 EffectCraft, 35 PhotoCraft, and 83 FigCraft operations. FigCraft has no `raster.filter`. EffectCraft and PhotoCraft beta panels float; FigCraft panels dock. Legacy EffectCraft docked ScriptUI and `ec_*` Wasm are separate from beta `craft_*` interfaces.

| Requirement | Current coverage and proposed addition | Explicit acceptance use case |
|---|---|---|
| Typed object access and stable identity | Operations exist; specify handle lifetime, object type, units, and source/evaluated semantics. Generate typed wrappers from authoritative contracts. | A panel captures a layer, the user deletes it and creates another, and the old reference fails without editing the replacement. Rename preserves identity. Undo and reopen behavior are documented. |
| Context and transaction boundaries | Captured scope and per-call authorization exist. Extend JOBS with document epoch, read snapshot, apply transaction, and reentrancy rules. | A job finishes after selection changes or a document closes. Apply either uses its still-authorized captured targets or returns a stale-target error, with no partial document edit. |
| UI slots and lifecycle | Trees and overlays exist; extend UI-LIFECYCLE with ownership and cleanup rules for future slots. | Closing a panel, disabling a plugin, and changing workspaces release resources. A UI 1.0 plugin still receives identical widget values under another host renderer. Unsupported components retain declared fallbacks. |
| Language ergonomics | Rust guest helpers and CLI exist. Extend AUTHOR-TOOLS with generated operation signatures, capability inspection, and actionable incompatibility reports. | An author requests an operation absent in FigCraft and gets an app/contract-specific diagnosis before distribution. The SDK never suggests a generic engine-command escape hatch. |
| Permissioned services and native dependencies | Wasm services are absent; native consent and mediated calls exist. Extend SERVICES and NATIVE-SETUP with distinct host-service and OS-access descriptions. | A helper has native consent but loses document-edit permission. Its next host edit fails. A missing native dependency produces a setup error without enabling a broken contribution. |
| Versioned project data | Project plugin references exist in EffectCraft and FigCraft, not PhotoCraft. References alone are not a general saved-data schema. Add namespaced payload versions, quotas, migration ownership, and unknown-data preservation. | Save a project with plugin data, reopen without the plugin, save again, and reinstall. Data survives. A downgrade preserves unsupported newer data and reports incompatibility without running saved code. |
| Diagnostics and performance contracts | Runtime limits exist; extend AUTHOR-TOOLS with structured error categories and documented budgets. Measure before choosing new thresholds. | An oversized tree, stale UI event, expired handle, denied grant, or timed-out callback produces a distinguishable diagnostic. Bounded log output identifies plugin version and operation without exposing document content or secrets. |

The persistence case adds a concrete obligation beyond private SETTINGS. The handle and snapshot cases sharpen JOBS and MEDIA. Diagnostics should report time spent in host dispatch separately from helper computation and UI rendering where measurable. This note supplies no benchmark or new numerical threshold.

Blender demonstrates how many carefully defined hooks can make a compiled application programmable. Craft can adopt that explicitness while retaining its own app-specific contracts and sandbox boundary. The next integration decision is which real workflow justifies each additional hook; these sources do not prove runtime quality, proprietary algorithms, or broad compatibility for either system.

# SDK strategy and capability roadmap

This is the integration analysis from the 2026-11-14 community and ecosystem review.
It proposes a target and acceptance cases, not implemented features or a release
promise. The beta baseline is shared revision `3c7e06f`; the public EffectCraft
baseline is [v0.24.1](public-effectcraft-release-extensions.md). No application,
third-party plugin, build or CI ran for this research.

The aim is a deep, coherent SDK that lets authors deliver complete workflows with
less setup and fewer app-specific surprises. “Better than every plugin system”
needs named outcomes rather than an API-size claim. Authoring effort, supported
workflow coverage, UI integration, data preservation, reliability and permission
clarity must be evaluated separately. This research supplies no benchmark or
proof of superiority.

The [interactive architecture and workflow map](maps/sdk-capabilities.html) compares
extension mechanisms and shows current coverage, missing interfaces and acceptance
outcomes for six reference workflows. It is a self-contained offline page.

## Research map

| Question | Evidence and detailed analysis |
|---|---|
| What does PluginCraft ship? | [Four inspected packages and registry trust](plugincraft-review.md) |
| What does the released EffectCraft support? | [Pinned public release loaders and migration regressions](public-effectcraft-release-extensions.md) |
| How does a compiled host support plugins? | [Blender, C ABI, embedded Python, Wasm and helper processes](compiled-host-and-blender-extension-lessons.md) |
| What do major Adobe ecosystems expose? | [Adobe architectures and six product-sized requirements](adobe-extension-and-plugin-requirements.md) |
| What did the earlier community review establish? | [BOUNCraft](community-bouncraft-review.md), [EC Scripts](community-ecscripts-review.md), [NZO evidence limits and Anonymizer](community-nzo-review.md) |
| What is actually implemented in the beta? | [Capability reference](capabilities.md), [UI](ui.md), [native protocol](native-protocol.md), [architecture](how-it-works.md) |
| What should be tracked for implementation? | [Prioritized backlog](community-comparison-and-backlog.md) |

The requested NZO store remains inaccessible. Registry labels and vendor product
pages do not establish plugin internals. SDK access restrictions prevented verifying
some current Adobe native suites. Those limits remain in the linked notes.

## Why compiled applications can be extended

Compilation fixes the host program, not every possible input it accepts. The host
can expose an interpreter, a native ABI, a Wasm runtime or a process protocol.
Plugins conform to those interfaces. They can compose existing operations or
provide computation through explicit hooks. Adding a missing document type,
evaluator, importer or renderer integration still requires host work.

Blender is a compiled C/C++ host exposing Python through structured data and operator
interfaces. AE separates scripting and panels from native image evaluation. Photoshop
separates UXP document/pixel access, hybrid compute and native plugin types. Premiere
separates sequence automation from media-processing integration. Their breadth comes
from domain APIs and lifecycle rules, not one universal plugin entry point.

Craft already has Wasm and native session execution, a shared manager/store/CLI,
versioned UI and app adapters. It should deepen those boundaries rather than expose
arbitrary engine commands or reinterpret every plugin as a pixel filter.

## Four responsibilities of a complete SDK

| Responsibility | Author should be able to express | Host must define |
|---|---|---|
| Workflow and document automation | Inspect typed objects, create/edit supported content, react to app events and apply a change plan | Stable identity, units, context, permissions, owner thread, atomic document commit, Undo and reentrancy |
| UI and interaction | Docked panels, controls, custom canvas, tools, overlays, command/menu/shortcut entry points | Placement, focus, clipping, input revisions, appearance, accessibility, resource cleanup and backend independence |
| Data and computation | Chosen assets, image/audio snapshots, long jobs, native libraries and rendering callbacks where supported | Source/evaluated semantics, color/alpha, exact time, quotas, cancellation, concurrency, device/buffer ownership and result validation |
| Persistence and distribution | Portable project data, private presets, packages, dependencies, updates and diagnostics | Schema migration, unknown-data preservation, platform compatibility, trust, consent, reproducible setup and honest failures |

A panel is not a render callback. A native compute helper is not an app adapter.
A saved project reference is not a general plugin-state schema. A catalogue is not
an execution sandbox. Keeping these responsibilities explicit makes a broad SDK
maintainable.

## App-specific depth

**EffectCraft:** first cover motion workflows completely. Selected properties and
keys, easing/interpolation, expression/control-group management, layer timing and
precomposition, chosen media, audio analysis and render jobs are concrete needs
from FlowCraft, PrimeCraft, BOUNCraft and Auto Beat Marker. Existing numeric property
and key insertion APIs are useful but insufficient for those full workflows.

Rendering needs its own migration work. The public legacy Wasm ABI 2 already
supplies neighbor frames, while the beta has no temporal filter contract. Public
linked/native helper effects have additional host services. Do not replace the
public path without a compatibility decision and affected-project behavior.

**PhotoCraft:** deepen image-result workflows around selections, pixel formats,
mask/layer creation, local preview and result commit. Preserve the distinction
between destructive filters, editable returned layers and future persistent smart
filters. A portrait-processing helper must not commit stale pixels after source
changes. File publication can remain externally visible after document rollback;
keep that error contract honest.

**FigCraft:** use its existing node, vector, text, layout, component and style depth
for layout tools and editable cross-app transfer. Define geometry, coordinate and
font substitution semantics before promising an Overlord-like transfer. FigCraft
still has no raster.filter dispatch. Source image export and editable geometry
transfer are different operations.

**Future FilmCraft adapter:** an AutoPod-like workflow needs an NLE domain model:
tracks, clips, source offsets, rational time, linked A/V, multicam selection, audio
analysis, edit plans and exports. None is established by these three adapters. Add
one adapter using FilmCraft's canonical commands when its source/design is in scope;
do not fake this capability in EffectCraft or advertise it from a manifest label.

## SDK ergonomics without weakening authorization

The current `OpSpec` in [ext-host api.rs](../../crates/ext-host/src/api.rs) stores
`args` and `result` as example text. Adapter examples include forms such as
`{name?,width?}`. These are not complete machine-readable schemas. Typed binding
generation therefore needs structured argument/result definitions, including IDs,
enums, units, bounds, errors, supported contexts and versions. Derive them from
canonical types/registries wherever practical; avoid a second independently edited
API catalogue. Migrate a small operation family first and prove schema/dispatch
agreement before expanding.

Offer a high-level author layer over those stable contracts: typed object handles,
change-plan builders, component helpers, forms, canvas utilities and scoped jobs.
Convenience wrappers must preserve the host's required permissions and captured
scope, never reduce them based on caller flags. Context menus and shortcuts should
invoke the same authorized contribution as the plugin panel or CLI.

Rust is the existing Wasm SDK path, not a requirement inherent to plugins. Document
the ABI independently. Evaluate JS/TS author ergonomics and generated bindings
against a real port before selecting another runtime/toolchain; that is a design
investigation, not a promised embedded engine. Native protocol clients can use
other languages when their platform executable/runtime is packaged correctly, with
full-access consent. Native Python is not sandboxed merely because it is Python.
Legacy JSX can remain a separate compatibility route, but its evaluator must not
become a hidden Wasm host operation.

Development tooling should explain what failed: unavailable operation, wrong
contract/UI level, missing native platform, denied grant, stale object/surface,
invalid pixel layout, missing asset, timeout or helper/device failure. Add an
operation explorer, UI-tree/event inspector, bounded structured logs, hot reload
and a host simulation harness. Simulators must model errors and scope changes;
package checks alone cannot prove UI or render behavior.

## Compute and rendering contracts

Preserve a sandboxed workflow/filter lane and a clearly consented native lane.
A future render suite must define source versus evaluated inputs, region/padding,
other-layer dependencies, temporal windows, color/alpha, quality, thread safety,
cache invalidation and cancellation. CPU and GPU support are separate capabilities.
A helper's ability to run GPU code externally is not a host GPU buffer-sharing API.
Shared memory, GPU brokerage or shader support require a separate feasibility and
resource-lifetime design. UI guests still cannot choose or detect the UI renderer.

Long jobs must release the document transaction. Apply results in a new owner
transaction, checking document epoch, stable targets, input revision, plugin version
and current grants. Render callbacks must work independently of visible panels and
current selection, tolerate out-of-order/concurrent requests, and avoid UI-thread
mutation. Cache state needs explicit ownership and bounded memory.

Provide brokered host services only for concrete needs: chosen-file access, private
settings, network destinations, clipboard or credential references. Do not expose
raw OS paths, general sockets, shell/process calls or the engine dispatcher to Wasm.
Native external OS access remains broad and is not retroactively constrained by
host-call grants. Project edits and external writes have different rollback limits.

## Persistent data and compatibility

Use namespaced, versioned document payloads with quotas and migration ownership.
Separate those from private preferences, presets, caches, licensing and source
assets. Preserve unknown/newer data through open-without, save, reinstall and
rollback. Saved values must never be executed as code on load. A filename or reused
object number is not sufficient identity after deletion or document replacement.

Keep UI 1.0 compatibility, stable IDs and component fallbacks. Framework adoption
is a host renderer change, as the [framework handoff](framework-integration.md)
already specifies. Docking belongs to each host workspace; it should not require
guests to link egui, emit arbitrary DOM or discover the backend.

Public-release compatibility needs a deliberate disposition for legacy Wasm ABI
1/2, ScriptUI, native helper effects and existing project instances: a tested shim,
an explicit author port, or a clearly reported unsupported case with preserved data.
Do not silently substitute a pass-through render or claim Adobe/OFX binary support
because a native helper process exists. Catalogue license/admission policy is
separate from technical package/runtime capability; no commercial-plugin catalogue
policy change is proposed by this research.

## Delivery order and proof of value

| Slice | Concrete acceptance outcome | Dependencies and evidence needed |
|---|---|---|
| 1. Prevent regressions and finish hosting | Dock an existing beta panel in EC/PC; enumerate and explain every public-release extension type and affected project | UI-DOCK/UI-LIFECYCLE, public ABI/helper inventory, no silent state loss |
| 2. Complete an animation tool | Recreate an original FlowCraft/BOUNCraft-style editor: select keys, edit easing, preview, apply/Undo, save a preset, reopen and revoke permissions | Structured types, motion/control operations, settings, live permissions and supported UI; source prototype then runtime acceptance |
| 3. Complete an image workflow | Original PhotoCraft retouch-style processing with preview, separate layers and one apply/Undo; reject stale results after source edit | Pixel/color contracts, jobs and chosen assets, source revision binding; native setup only if computation needs it |
| 4. Add temporal/native render depth | Original small temporal processor works at clip edges, subframes, reverse seek, concurrent export, helper loss and cancellation | Public ABI 2 migration decision, render suite and memory/thread/time semantics; preview/export equivalence |
| 5. Prove editable transfer and NLE scope | Fig-to-EC vector subset preserves editability; future FilmCraft plan previews/applies/undoes multicam cuts | Explicit transfer fidelity and per-app operations; FilmCraft canonical engine review before implementation |
| 6. Expand based on measured demand | Advanced 3D, new formats or custom evaluator providers have a named complete workflow | Host-specific extension ABI, persistence and asset semantics; no promise that today's raster/session contract already supports them |

Measure author steps/time to a first useful plugin, exact supported actions in the
reference ports, install-to-first-result steps, failure recovery, state preservation,
preview/export agreement and actual resource use. Choose targets after establishing
baselines. This review does not claim lower latency, stronger isolation than every
competitor, or universal binary compatibility.

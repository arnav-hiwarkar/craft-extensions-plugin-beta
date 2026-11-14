# Public EffectCraft release extension baseline

Inspected 2026-11-14. [The latest public release](https://github.com/storytold/effectcraft/releases/tag/v0.24.1)
was v0.24.1, published 2026-11-12, at commit
`582344dd4ef462362c02e56e3094de7af01847ab`. The release API and tag reference agreed.
This is a pinned source/documentation review, not a run of the downloaded application.
The private beta baseline is shared revision `3c7e06f` with the app revisions recorded
in the existing community notes. A beta checkout is not evidence of public-release behavior.

## The released app has several extension mechanisms

| Mechanism | How the compiled host uses it | Evidence |
|---|---|---|
| JSX/JS scripts and docked ScriptUI | An embedded Boa JavaScript runtime exposes an AE-style app object model. Mutations use canonical engine commands. ScriptUI widget/draw descriptions are rendered by the host, with callbacks routed back to the script. | [Runtime source](https://github.com/storytold/effectcraft/blob/582344dd4ef462362c02e56e3094de7af01847ab/crates/script/src/lib.rs), [script installation](https://github.com/storytold/effectcraft/blob/582344dd4ef462362c02e56e3094de7af01847ab/crates/engine/src/cmd/scripts.rs) |
| Legacy Wasm effect modules | The wasmi loader selects `ec_render` or `ec_render2` from the module's ABI value. It provides memory and parameter/frame buffers, not the whole application. | [Loader and render implementation](https://github.com/storytold/effectcraft/blob/582344dd4ef462362c02e56e3094de7af01847ab/crates/plugin/src/wasm.rs) |
| Linked Rust effects | Code implements `EffectPlugin` and registers with the host. This route requires an application linking that implementation, rather than loading an arbitrary Rust shared library. | [Native trait and render services](https://github.com/storytold/effectcraft/blob/582344dd4ef462362c02e56e3094de7af01847ab/crates/effects/src/plugin.rs) |
| External effect helper | A separately launched program describes effects and exchanges frame references with the host. The public protocol includes render-time callbacks. | [Released helper protocol](https://github.com/storytold/effectcraft/blob/582344dd4ef462362c02e56e3094de7af01847ab/docs/external-helpers.md) |
| Plug-in Library | A catalogue supplies effect downloads and expected digests. Installed releases load through the legacy effect path. | [Released plugin guide](https://github.com/storytold/effectcraft/blob/582344dd4ef462362c02e56e3094de7af01847ab/docs/plugins.md#plug-in-library) |

Compilation does not forbid any of these. The executable contains the interpreter,
loader, dispatcher and render hooks. Authors provide code or data conforming to
those hooks. A missing hook still needs a host change. Neither an Adobe native
binary nor a Python Blender add-on becomes executable in EffectCraft by changing
its filename or manifest.

Platform and build features still determine which hooks a particular binary ships.
This note establishes the pinned release source's interfaces, without running its
published platform binaries.

## App scripting is broader than the beta operation list

The released scripting source includes compositions, layers, masks, selected
properties and keys, easing, expressions, render queues, imports, persistent app
settings and scheduled callbacks. This explains why motion tools can be powerful
without changing the core engine. It does not prove every AE script is compatible.
File and network gates differ from beta contribution-scoped grants; an Undo group
is not an automatic rollback guarantee for all script or external effects.

An installed ScriptUI panel receives a host `Panel`. Running the same file directly
can instead create a floating palette. Therefore “community plugins float” cannot
be generalized to installed ScriptUI panels. The beta's EffectCraft and PhotoCraft
panel-placement gap is separate from that legacy route.

## Rendering capabilities that migration must preserve deliberately

The released Wasm loader rejects imports, consumes fuel and limits memory/tables.
ABI 1 is a current-frame in-place effect. ABI 2 passes the previous and next frame,
their times and presence flags. These are specific temporal inputs, not unlimited
access to arbitrary frames. Loader/render source was read; no module was executed.

The native `EffectPlugin`/`PluginHost` source separately supports padding, source
and other-layer inputs at requested times, evaluated parameters, frame rate,
instance identity and source ranges. Its callbacks are not all exposed to Wasm ABI 1
or 2. Distinguish the Rust API version, Wasm ABI and beta contract versions.

The released helper protocol supports externally computed effects. It uses opt-in
scanning/startup controls, request/reply messages and a host-created frame-exchange
file. Failure handling and restart policies are documented there. Process separation
reduces crash coupling but is not OS confinement. This is a different protocol from
the beta's native session helper; both calling themselves protocol 1 does not make
them interchangeable.

## Public release versus shared beta

| Area | Public v0.24.1 evidence | Beta at the inspected baseline | Migration requirement |
|---|---|---|---|
| Panels | Docked ScriptUI host and palette fallback | Shared versioned trees; EC/PC panels float, FigCraft docks | Add workspace hosting while preserving old ScriptUI installations. |
| Automation | Broad AE-style object model | Closed, permissioned per-app operations | Add missing selected-key/easing/expression/media operations deliberately; preserve captured scope. |
| Wasm effect ABI | `ec_*` ABI 1 and temporal ABI 2 | `craft_*` ABI 1 and `raster.filter@1.0` | Detect formats and provide an explicit migration/compatibility decision. Identical version integers are insufficient. |
| Native image computation | Released external effect helper protocol | Native command/panel/tool/event sessions; native effects and pixel transport deferred | Record this functional loss before replacing the public path. A wrapper needs real transport/lifecycle support. |
| Render inputs | Neighbor frames for Wasm ABI 2; richer linked-native host callbacks | No temporal/region filter contract | Make temporal, padding, other-input and concurrency requirements explicit before claiming effect parity. |
| Package management | Manual installs plus legacy Library | Shared signed metadata, pins, rollback, grants and revocation | Improve lifecycle without silently losing existing project dependencies. |

These are source/contract comparisons, not measured quality or security rankings.
The beta need not duplicate every historical implementation, but release migration
must name unsupported cases and preserve project data. Rejecting an old ABI honestly
is preferable to accepting it and rendering incorrectly, yet rejection is still a
compatibility gap.

## Acceptance cases before a public replacement

1. Classify installed ScriptUI, legacy ABI 1/2 effects, linked effects and helper
   effects separately. Identify affected projects and explain each disposition.
2. Exercise a temporal effect at boundaries, subframes and absent-neighbor cases.
   Compare defined inputs and outputs before claiming compatibility.
3. Exercise a helper crash, cancellation, reload and missing helper on reopen.
   Preserve parameters and block unsafe final export when required.
4. Demonstrate a docked motion-tool port and an image effect port, including Undo,
   saved state, missing-plugin recovery, live revocation and normal/beta isolation.

See [SDK strategy](sdk-strategy-and-capability-roadmap.md) and the
[community backlog](community-comparison-and-backlog.md). No build, plugin execution,
app launch or new CI was performed for this review.

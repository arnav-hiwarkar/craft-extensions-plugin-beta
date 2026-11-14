# Adobe extension and plugin requirements

Research date: 2026-11-14. This explanation compares extension mechanisms and derives requirements for original Craft implementations of selected workflows. It does not propose loading Adobe binaries, translating Adobe SDK calls, or reproducing proprietary algorithms.

## Evidence and scope

Four evidence classes remain separate throughout this document:

| Class | Meaning and limits |
|---|---|
| Official documented behavior | Adobe developer and help pages inspected on the research date. These are rolling pages, not a guarantee about every installed host version. |
| Inspected source | Small public Adobe sample files, read without execution and pinned below. Samples establish what their code calls, not current runtime success. |
| Vendor claims | Product pages establish advertised workflows. They do not expose algorithms, performance measurements, or every internal integration mechanism. |
| Inferred Craft requirements | Proposed host contracts and acceptance outcomes. These are neither existing APIs nor implementation commitments. |

The beta baseline is revision `3c7e06f`, supplied by the task, cross-checked against the current [capability reference](capabilities.md), [author guide](author-guide.md), [UI reference](ui.md), [runtime explanation](how-it-works.md), and [framework integration](framework-integration.md). Adapter source was not independently re-audited here. EffectCraft, PhotoCraft, and FigCraft expose 32, 35, and 83 operations respectively. No application, installer, plugin, test, or build ran during this research.

Primary sources only support the claims below. Adobe's [After Effects SDK entry point](https://developer.adobe.com/console/servicesandapis/ae) exposes a console shell without readable SDK contents in this session. Premiere's [developer page](https://developer.adobe.com/premiere-pro/) says its C++ documentation accompanies the download and acknowledges restricted access. No gated bundles were obtained. Exact current native suites, signatures, compatibility flags, and threading exceptions remain unknown where noted. Community SDK guides were not treated as primary evidence.

## Different mechanisms grant different capabilities

A panel is a user interface. Automation changes host objects. An effect participates in image or audio evaluation. A codec imports or exports media. Native compute accelerates a calculation. Combining these mechanisms requires explicit host integration; none implies all the others.

| Host and mechanism | Documented purpose | Boundary relevant to Craft |
|---|---|---|
| Photoshop UXP plugin | Panels and commands, document and layer DOM, action execution, and Imaging API | DOM automation and pixel transfer are distinct. State-changing API-v2 work requires modal execution. |
| Photoshop native C++ SDK | Filters, additional image formats, selectors, and low-level integration | A native filter or format plugin has a different host contract from a UXP panel. |
| Photoshop UXP hybrid | JavaScript imports a compiled C++ addon | Native calculation does not itself register an Adobe filter or file format. |
| Premiere UXP | Modern panels and asynchronous host automation, documented for 25.6 onward | Project, sequence, and track-item objects have their own versioned methods. Photoshop's modal rule cannot be assumed. |
| Premiere CEP with ExtendScript | HTML-based panels with older scripting automation | Existing products may use this route. UXP parity must be established operation by operation. |
| Premiere native SDK | Low-level media integrations, formats, codecs, effects, and transitions | Frame evaluation, audio processing, and import/export are separate integration problems from sequence editing. |
| After Effects scripts and ScriptUI | ExtendScript automation and dockable script panels | Scripts operate through the scripting object model. They are not per-frame native effects. |
| After Effects CEP | Panel frontend invokes ExtendScript | Adobe's sample demonstrates this bridge, rather than a shared UXP DOM across all three apps. |
| After Effects compiled effects and native extensions | Image-buffer processing, parameter controls, and project integration | Effect SDK, AEGP, custom UI, SmartRender, GPU, and Multi-Frame Rendering need separate contract analysis. Exact current SDK details were not available here. |

Sources: [Photoshop developer overview](https://developer.adobe.com/photoshop/), [Photoshop hybrid distinction](https://developer.adobe.com/photoshop/uxp/guides/hybrid-plugins/), [Premiere overview](https://developer.adobe.com/premiere-pro/), [After Effects overview](https://developer.adobe.com/after-effects/), and [After Effects scripting help](https://helpx.adobe.com/after-effects/desktop/automate-in-after-effects/automate-animation/scripts.html). The Photoshop overview includes historical compatibility statements; this document does not extrapolate them to current architectures.

### Photoshop pixels and mutation

Adobe documents [executeAsModal](https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/executeasmodal) as exclusive access for state-changing operations under API version 2. Asynchronous JavaScript therefore does not imply concurrent document mutation. The [Imaging API](https://developer.adobe.com/photoshop/uxp/2022/ps_reference/media/imaging/) reads layer or composite pixels, masks, and selections, and provides pixel writes. Its data carries bounds, color profile, component depth, layout, and alpha information. Pixel allocations require disposal. Reduced-size requests can return cached pyramid levels.

This supports original pixel-processing tools without proving that every Photoshop feature is scriptable. A UXP pixel round trip also does not establish persistent smart-filter evaluation. Adobe explicitly distinguishes hybrid addons from C++ filter and format plugins. Current [shared hybrid documentation](https://developer.adobe.com/uxp/guides/how-to/hybrid-plugins/) lists Photoshop 24.2+ and Premiere 26.2+; this later Premiere support must not be lost by relying only on the older Photoshop guide. Native SDK availability and native addon availability remain different facts.

### Premiere projects, media, and background work

The [Premiere UXP reference](https://developer.adobe.com/premiere-pro/uxp/ppro-reference/) describes asynchronous methods and synchronous property access. [Project](https://developer.adobe.com/premiere-pro/uxp/ppro-reference/classes/project) exposes project and sequence operations. [SequenceEditor](https://developer.adobe.com/premiere-pro/uxp/ppro-reference/classes/sequenceeditor) creates insert, overwrite, clone, and removal actions, with tick-based times. These pages provide concrete automation evidence, not evidence that panel code receives decoded audio samples or neighboring render frames.

Native video effects need evaluated image buffers; audio processors need sample blocks and channel layouts; importers decode source media; exporters encode rendered output. These describe required integration roles. Current Premiere native entry points, supported audio plugin formats, GPU ownership, and export callback rules were not verified from an accessible Adobe SDK here. They must remain open questions rather than inferred UXP methods. Likewise, async APIs and a hybrid compute library do not establish durable background jobs, headless export, or permission to call project APIs on arbitrary threads.

### After Effects automation and rendering

Adobe documents dockable ScriptUI panels and a preference controlling script file writes and network communication. This is a different authorization model from beta contribution rights. The public [Adobe CEP samples](https://github.com/Adobe-CEP/Samples/tree/e4946b73ac1e566dced8e95dba10811c31036927) are pinned to `e4946b73ac1e566dced8e95dba10811c31036927`, resolved through GitHub's public commit endpoint on the research date. The inspected [After Effects script](https://github.com/Adobe-CEP/Samples/blob/e4946b73ac1e566dced8e95dba10811c31036927/AfterEffectsPanel/jsx/After%20Effects.jsx) creates compositions and layers, assigns expressions, and reads text. It was not executed.

Adobe's native overview documents parameter controls and 8-bit through floating-point image buffers. AEGP denotes broader native host integration, distinct from an image effect. Custom UI denotes effect-specific interaction rather than a general HTML panel. SmartRender concerns render dependencies and requested image regions; GPU rendering concerns device resources; Multi-Frame Rendering concerns concurrent frame evaluation. These are the questions a Craft render contract must answer. Exact current checkout procedures, AEGP thread permissions, SmartRender selectors, GPU dispatch/fallback rules, and MFR opt-in flags remain unverified against primary SDK contents. No particular flag or suite is proposed here.

Adobe's [MFR announcement](https://blog.adobe.com/en/publish/2021/03/10/multi-frame-rendering-now-in-after-effects-beta) establishes concurrent-frame rendering as a host concern. It is historical evidence, not a current SDK compatibility matrix. Automation of a render queue and participation in rendering are separate responsibilities.

## Six product workflows and their host dependencies

All feature statements in this table are vendor claims. Required interfaces are deductions about an original implementation; internal algorithms and measured fidelity are unavailable.

| Product and vendor evidence | Advertised behavior | Required host interfaces | Beta feasibility and missing work |
|---|---|---|---|
| Photoshop: [Nik Color Efex](https://www.dxo.com/en/nik-collection/nik-color-efex/) | Stacked creative adjustments, presets, local U Point or AI masking, color grading, and halation | Pixels, local selection coordinates, preview, parameter state, color conversion, and result commit | PC can support an original bounded RGB filter or pixel-edit workflow. EC can evaluate an original spatial raster effect. Neither fact establishes Nik algorithms or PC smart filters. Fig has no raster filter. |
| Photoshop: [Retouch4me Panel](https://static.retouch4.me/retouch4mepanel) | Orchestrates retouch plugins, offers local or cloud processing, and returns separate or merged layers | Layer snapshot, processing transport, returned pixel alignment, layer creation, blend/opacity metadata, and cancellation | PC has pixel and layer operations for a limited original local workflow. Model inference may need a native helper. Cloud processing needs an explicitly designed service path; Wasm has no network API. |
| Premiere: [Neat Video](https://www.neatvideo.com/features) | Temporal and spatial denoising, noise profiles, preview, high-bitdepth host integration, CPU/GPU optimization | Neighbor-frame requests, source time mapping, per-instance profile state, preview/render consistency, and compute scheduling | EC's single-frame filter is a spatial starting point only. PC can process a still. Temporal video evaluation needs new host capabilities; an NLE port needs a FilmCraft adapter. |
| Premiere: [AutoPod](https://www.autopod.fm/) and [vendor FAQ](https://www.autopod.fm/pricing) | Multicamera edits based on separate microphones, silence cuts, social sequences, and batch export | Audio analysis, clip/track identity, exact time, linked edits, sequence duplication, project mutation, and export jobs | No complete workflow in these adapters. Requires FilmCraft domain operations and an analysis path. A native helper cannot create missing timeline host calls. |
| After Effects: [Overlord](https://battleaxe.co/overlord) and [integration setup](https://battleaxe.co/overlord/docs/app) | Transfers editable shapes and text between creative apps; also transfers rasterized artwork | Vector geometry, transforms, groups, text runs, asset transfer, target construction, and identity mapping | Fig supplies substantial source geometry; EC creates basic layers but lacks arbitrary shape/text-object transfer coverage. A narrow geometry subset is plausible after new EC operations. |
| After Effects: [Element 3D](https://www.videocopilot.net/products/element2/) and [vendor help](https://www.videocopilot.net/assets/public/misc/Element_Help_Files.pdf) | GPU-backed 3D object/particle rendering with scene setup | Model/material assets, scene parameters, camera/light sampling, native GPU compute, persistent state, and render integration | Requires native/GPU and host contracts beyond beta. A flat RGBA result can be an output boundary, but does not supply camera/light access or the scene editor. |

Two distinctions affect feasibility. Retouch4me's layered output can remain editable even when computation happens outside the host; that does not imply every processing stage is a persistent smart filter. Overlord's [current setup documentation](https://battleaxe.co/overlord/docs/app) explicitly identifies CEP for After Effects and Illustrator, UXP for Photoshop, and a Figma plugin. The shared product name does not imply one extension runtime. Its [rasterization documentation](https://battleaxe.co/overlord/docs/rasterize) requires saved image assets for After Effects imports, unlike editable geometry transfer.

## What the beta actually supplies

Shared packaging, store, host, CLI, SDK, and host-rendered UI 1.0/1.1 are useful foundations. EC and PC panels float; Fig panels dock. Legacy EC ScriptUI docking and legacy `ec_*` Wasm do not establish beta `craft_*` compatibility. Wasm receives no raw-path, shell, process, network, or clipboard API.

Native helpers run out of process with full OS access after per-version consent. Their adapter calls still require declared and live rights plus captured scope. Consent enables external computation, not absent host operations or a GPU render ABI. External OS writes also do not acquire transactional Undo.

PC filters currently bake RGB pixels and lack smart-filter instances. Their premultiplied floating RGBA contract differs from straight RGBA session pixel operations. EC filters use premultiplied floating RGBA in the project's working space. Fig's vector operations and SVG-string export do not constitute raster processing or cross-app asset transport. These are documented beta limits, not runtime results from this research.

## Additional requirements and acceptance workflows

The [existing backlog](community-comparison-and-backlog.md) already covers docking, jobs/media, masks, motion/expressions/selected properties, private settings, control groups, author tools, services, theme, and render depth. The following requirements refine those areas with product-sized acceptance cases rather than repeat their discovery. Names below are requirement identifiers, not API names.

| Priority and requirement | End-to-end acceptance case | Distinct host obligation |
|---|---|---|
| P0: image-result transaction | In PC, select a portrait layer, run an original local retouch operation, preview, commit separate result layers, adjust opacity, and undo once. Cancel, revoke rights, or edit the source before completion; no stale pixels commit. | Bind source identity, document revision, pixel format, bounds, and result layers into a validated commit. Report helper failure without partial document edits. |
| P0: effect persistence | Apply an original EC spatial grading effect, animate strength, save, close, reopen, and reproduce the result. Reopen without the plugin, save again, reinstall, and recover parameters. | Preserve plugin identity, contribution/version information, parameter schema, and opaque state where needed. Define migration, missing-effect placeholders, and explicit render errors. PC persistent effects remain new work. |
| P1: temporal dependency evaluation | Denoise a short clip with an original neighbor-frame processor. Seek backward, render frames out of order, compare preview and export, and exercise clip edges, retiming, and cancellation. | Define requested time domain, neighbor availability, cache keys, memory limits, and dependency invalidation. A session time-change event cannot substitute for renderer frame access. |
| P1: audio-driven edit plan | In a future FilmCraft adapter, analyze two independent microphone tracks and three camera tracks. Preview an edit plan, apply it, undo, save/reopen, and export a short sequence. | Preserve linked audio/video, source offsets, rational frame/sample times, track locks, and stable item identity. Reject stale plans. This workspace supplies no FilmCraft adapter evidence. |
| P1: editable vector transfer | Transfer a Fig selection containing a nested group, Bezier path, text, and gradient to EC. Match placement and edit the resulting objects; undo the import. | Specify coordinate transforms, path closure/winding, text/font substitution, gradient mapping, and unsupported-object reporting. Raster fallback requires a declared asset route and an explicit fidelity choice. |
| P2: native scene renderer | Load an original small 3D scene, change camera and material, save/reopen, render on supported hardware, then simulate helper loss and device loss. | Define assets, project ownership, scene state, render cancellation, GPU lifetime, and recovery. Fail explicitly when no supported renderer exists; do not silently substitute an unrelated effect. |

### Color, time, and render-thread rules

An image contract must identify channel order, row layout, alpha convention, working color space/profile, transfer function, and numeric range. Photoshop's documented 16-bit default range is 0–32768, with a full-range option; floating values can exceed display white. A port must convert deliberately instead of assuming all 16-bit buffers or floating RGB values mean the same thing. Acceptance fixtures need transparent edges, saturated colors, and HDR values through preview, commit, save, and export.

Temporal contracts must distinguish source time from composition/sequence time, define frame-boundary rounding, and specify handling of missing neighbors. Audio needs sample rate, channel layout, sample time, and analysis bounds. Cached results must depend on source revision, parameters, color interpretation, quality, and relevant temporal inputs.

Render callbacks must not depend on panel visibility, mutable selection, or UI-thread document edits. Proposed compute state must tolerate concurrent and out-of-order requests. Immutable snapshots or explicitly synchronized caches need bounded ownership. The host must define which calls are legal on render threads, how cancellation releases CPU/GPU resources, and whether fallback is supported. These are Craft requirements, not claims about uninspected Adobe SDK rules.

Project persistence must distinguish document-owned effect state from user presets, caches, licenses, and external assets. Missing plugins must preserve recoverable state and explain whether rendering stops, bypasses, or uses a declared cached result. Export errors must identify the failing contribution and affected frame/job. Reinstalling or upgrading must not silently discard unknown parameters. Helper crashes, expired grants, missing assets, and unsupported formats need actionable errors with no fabricated success.

“Plugins can do everything” is not a useful acceptance criterion. Each workflow needs the appropriate object model, data access, compute route, persistence, and lifecycle guarantees. The beta supports meaningful original spatial and automation tools today; the six complete product workflows require different, explicitly bounded additions.

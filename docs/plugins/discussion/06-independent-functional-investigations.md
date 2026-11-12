# Independent functional investigations

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Historical discussion archive, captured on 2026-11-08. This preserves technical requests and substantive responses; routine progress updates and tool logs are omitted. Local checkout links are normalized. Later decisions and source corrections take precedence. Start with [the current documentation index](../README.md).

## User request

for now, protection wise, lets initially say sandboxed, and districbution we'll do offficial and third party, and second integration, sure choose figCraft.
Now I want you to understand all the things which will go into plugins when we or the community make them, and everything which we need to keep for them to work.
Firstly, i want you to study how plugins will work with the applications. so spawn another opus thread, and both of you should separately take up going through the process and understanding all the things that'll happen for different apps and how plugins would work.
So what systems will each app use, how does cli work for plugins, what will need to allow, what systems will the plugins need and we need to make, what are common, what would be stuff each app needs specific etc.
then properly once thats done, we'll go over details of the process and safety, first lets try to functionaly nail it down

## Research response 1

Both investigations are complete. Opus and I worked independently, then I reconciled the findings into the [functional model](../../../docs/plugin-functional-model.md).

Your choices are recorded: sandboxed plugins initially, official and third-party distribution, EffectCraft first, and FigCraft second. Detailed safety design remains the next stage.

The main conclusion is that **we can share plugin management and host services while preserving the different ways plugins participate in each app.**

A useful distinction is:

- A **package** is what someone installs.
- A **contribution** is a feature that package supplies, such as an effect, command, panel, or exporter.
- A **document instance** is one use of that feature, with its own parameters and saved state.
- A **runtime instance** is the temporary execution used to process a frame, run a command, or handle a callback.

For example, one package could supply an EffectCraft effect and a FigCraft command. The effect could appear on five layers with different settings. That does not mean the package needs five installations.

The shared system would provide the following machinery:

| Shared system | What it provides |
|---|---|
| Package management | Identity, releases, installation, updates, removal, official and third-party sources |
| Contribution discovery | What the package adds and which apps support it |
| Environment resolution | Which installed versions and resources a session uses |
| Runtime services | Module loading, execution lifecycle, results, logs, and errors |
| Parameter descriptions | Controls, defaults, units, choices, and validation |
| Resource handling | Assets, LUTs, models, samples, and their readiness |
| Jobs | Progress, cancellation, completion, and output artifacts |
| Persistence conventions | Package references and versioned instance-state envelopes |
| Author tooling | SDK, packaging, validation, examples, and test fixtures |

Each app then supplies the actual editor integration: its document operations, selection, undo, rendering, data conversions, menus, output writing, and saved-file handling.

This is more than adding a dependency to `Cargo.toml`. Each app needs a host adapter that connects those systems.

There are several distinct functional paths.

1. **A plugin performs an edit once.**

   A FigCraft plugin might arrange selected objects. The app supplies the selection and parameters. The plugin requests editor operations or returns proposed changes. The app applies them as an undoable transaction.

   Once committed, those objects are ordinary document content. The plugin generally does not need to run again just to display them.

2. **A plugin becomes a persistent feature.**

   An EffectCraft effect remains on a layer. A procedural widget remains in a FigCraft document. A live VectorCraft effect remains in an appearance stack.

   The document stores the contribution reference and instance settings. The app invokes it again when rendering or rebuilding requires it. Removing the plugin therefore affects future evaluation, even though its saved settings should remain intact.

3. **A plugin supplies an interactive interface.**

   A panel declares controls and handles callbacks. A tool receives a gesture lifecycle such as begin, update, commit, and cancel.

   The interface should invoke a separately defined operation where possible. For example, an Arrange button in a panel and an Arrange command from the CLI should call the same underlying functionality.

4. **A plugin supplies processing or an output format.**

   An image filter receives image data. A geometry processor receives objects. An exporter receives the document representation its contract defines and produces output artifacts.

   The app handles integration and final writing. Processing pixels, changing document objects, and exporting a file should not be forced into one indistinguishable callback.

5. **A plugin starts a longer job.**

   AI generation, simulation, baking, or heavy processing may need resources, progress, cancellation, and delayed results.

   The job needs an owner that survives until completion. Its result then enters the app through the appropriate document or media-import operation.

For EffectCraft, the intended path is concrete: an author implements an effect and declares its controls; installation makes its effect ID available; applying it creates an instance on a layer; EffectCraft evaluates keyframes and expressions; rendering passes the resulting parameter values and frame context to the plugin; saving preserves the dependency and instance settings.

The plugin author does not implement EffectCraft’s keyframe editor, effect browser, or project-saving system. Those remain host responsibilities.

For FigCraft, an author declares a command and its parameters, optionally adding a panel. The command operates through FigCraft’s editor interface. The app groups changes into an undo transaction. A panel can invoke the operation repeatedly, with each callback forming a coherent edit. Widgets additionally need persistent document state and evaluation behavior.

These existing differences determine what each app needs:

| App | App-specific integration |
|---|---|
| EffectCraft | Layer and time context, parameter animation, frame processing, effect registration, saved dependencies |
| FigCraft | Object queries and edits, selection, transactions, panel callbacks, widgets |
| PhotoCraft | Image bands, colour models, alpha, selection blending, neighbourhood overlap, filter integration |
| VectorCraft | Object schemas, units, live geometry effects, appearance evaluation, exporters |
| SoundCraft | Audio blocks, transport, MIDI/events, latency, parameter automation, state, editor lifecycle |
| FilmCraft | Clip and sequence operations; separate frame, transition, and audio processing contracts |
| LightCraft | Catalog targets, develop controls, RAW-processing stages, nondestructive settings |
| PDFCraft | Page and object operations, document history, PDF writing, output artifacts |
| DesignCraft | Stories, frames, pages, spreads, units, layout and reflow |
| CADCraft | Entities, units, transactions, programmatic commands, interactive prompts |
| SolidCraft | Command integration and eventual persistent parametric-feature evaluation |
| TextureCraft | Mesh/UV context, material channels, texture sets, strokes, bake dependencies |
| WordCraft | Text ranges, styles, stories, layout-aware edits |
| GridCraft | Ranges and workbook operations; custom functions also need recalculation contracts |
| DeckCraft | Slides, shapes, text, export; playback extensions need an additional lifecycle |
| ArtCraft native | Provider/model operations, durable generation jobs, media-result ingestion |

The deeper traces cover the first four apps. The others received a bounded source survey; this table describes integration requirements, not a claim that those plugin interfaces already exist.

**CLI support needs deliberate design.** There are three different situations:

- A fresh CLI process creates a new headless session.
- A CLI client controls an already-running editor.
- A persistent headless server keeps documents and jobs alive across requests.

They can share operations, but they do not share session state automatically.

A fresh process should initialize its plugin environment, discover installed metadata, open the project, resolve required contributions, load the needed code, execute operations, await necessary jobs, and save or export explicitly.

An attached CLI should use the running app’s environment and documents. It should not accidentally execute against a new local session.

A one-shot command cannot return an asynchronous job ID and exit unless another process owns that job. Otherwise it needs to wait for the output.

Both investigations found concrete source-level gaps:

- EffectCraft’s CLI initializes plugin-loader hooks but does not perform the desktop’s installed-plugin loading sequence. Its renderer also omits unresolved effects. That combination needs runtime reproduction before treating it as a confirmed defect.
- FigCraft’s headless path drops UI requests. Its offered-file operation currently queues a save dialog, so headless output needs an explicit artifact destination.
- SoundCraft already hosts native audio formats in-process. Preserving that functionality under the chosen sandbox requirement needs a dedicated backend design.

For plugin authors, the proposed deliverable is a package containing metadata, contributions, modules, parameter descriptions, assets, resource requirements, saved-state schemas where needed, and examples.

We should distinguish resources compiled into the module from resources the host retrieves. A missing model, unsupported GPU operation, and incompatible plugin are different conditions and should produce different results.

GPU processing also remains compatible with this functional model. A sandboxed module can request host compute operations. Whether an algorithm has a CPU fallback is a declared property; we should not assume one exists.

I recommend these functional defaults:

1. Preserve existing app ABIs through adapters initially, then standardize purpose-specific interfaces where multiple apps establish a real common requirement.
2. Allow one package to contain multiple app-specific contributions.
3. Require editing and compute contributions to expose headless-callable operations, while allowing explicitly UI-only contributions.
4. Prove EffectCraft’s complete save/reopen/headless-export path, followed by FigCraft commands, panel callbacks, and headless outputs.

The remaining functional choices I’d discuss next are:

- Should a fresh CLI automatically use compatible installed plugins, or require an explicit project/plugin environment?
- Should the initial package format include existing scripts, macros, and data presets, or begin with executable contributions?
- For FigCraft, should we first prove commands and panels, then add persistent widgets?

The [primary study](../../../docs/plugin-functional-primary.md) and [Opus study](../../../docs/plugin-functional-opus.md) preserve the evidence and qualifications. Opus completed its pass in <t3-thread://v2/5df2eb29-79b9-431d-99a9-a501614c1c2f/mcp:launch:71ace9a20c78c12d5ddc68722dcf6f81ebcfba21f5edaa217742978b392e9237>.

Documentation checks passed. No application code changed, and no apps or runtime tests were run.


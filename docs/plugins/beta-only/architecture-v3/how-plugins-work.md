# How plugins work in EffectCraft, PhotoCraft and FigCraft

A plugin is a package of code and data that adds to an app. It can add effects, filters, brushes, panels, windows, tools, menu commands, hooks into app behaviour, file formats and canvas widgets. The three beta apps run plugins on one shared platform. Each app adds only a thin adapter that maps the platform's words onto its own document model.

This document follows a plugin through its whole life: how it gets onto the machine, how the user approves it, what happens each time it calls the app, and how it is updated, removed or stopped. Three flagship plugins, one per app, show the path end to end with the ops, permissions and contracts they touch. The last two sections cover failure and the few things no plugin can do in this build.

Three rules shape everything below, so they are stated once here.

- **The app keeps authority.** On desktop and web, plugin code never runs inside the app process. It runs in a separate sandboxed worker and holds only handles the host gave it. On iOS, which allows no extra processes, it runs in an in-process memory-safe interpreter under the same gate. Every effect on files, clipboard, processes or the document is checked where it happens [REQ-COM-principle-app-keeps-authority].
- **The UI never waits.** The thread that draws the app never runs plugin code and never waits on it. A slow plugin makes its own output late, not the app.
- **Fail safe and visible.** Every time limit has a fallback that ends in a visible badge or notice. Nothing fails silently, and no plugin failure crashes the app or damages a document.

## From install to first use

### The package

A plugin ships as one `.craftplugin` file. It is a zip with strict rules: no links, no absolute paths, at most 4,096 entries and 4 GiB. Its sha256 is its identity [CON-COM-package-zip]. Inside are the manifest `craft-plugin.toml`, WebAssembly modules, JavaScript, WGSL shaders, models, fonts, icons and translations.

The manifest uses schema 3, the only schema. It names what the package contributes, which apps it works in, which permissions it needs and why. It never states its own risk level or tier. The host computes both, so a package cannot talk its way into looking safer than it is.

One package can serve several apps. A colour-grading plugin like Prism ships one package that becomes an EffectCraft effect, a PhotoCraft filter and a FigCraft fill [FLG-COM-prism].

### Getting it onto the machine

The user finds plugins in Window > Plugins inside any app, in the suite launcher's store, or through the command line. The host, not the plugin, fetches the catalogue and the package. The catalogue is signed with an offline root key and checked for revocations every hour.

Each package has a **tier**, which says who stands behind it: Official (built by ArtCraft), Verified, Community, Local (imported by the user from disk) or Dev (an author's own build). The tier is shown beside the risk level so the user sees both who made it and what it can do.

Before anything runs, the host **verifies** the package: signature, digest against the catalogue pin, provenance, zip rules, manifest parse, contract compatibility with each app, permission parse, a lookalike-name check and the licence [REQ-COM-lifecycle-verify]. A failure stops the install with a named reason.

### Preparing it

Next the host **prepares** the package, always inside a worker and never inside the app. WebAssembly is compiled ahead of time and stored in a tagged cache. JavaScript is compiled to engine bytecode. Shaders are validated for their GPU tier and their worst-case cost is estimated. Model files are checked against their sha256. The host then rebuilds `index.bin`, a small prebuilt index of every menu item, label and icon the plugin adds.

Preparing ahead means the first click on a plugin feature does not pay for compilation.

### The setup sheet: the one moment of consent

The user approves a plugin once, on one host-drawn **setup sheet** [REQ-COM-lifecycle-setup]. It lists everything the plugin asks for:

- every permission, in plain sentences;
- every hook into app behaviour and every kind of automatic run;
- commands it offers to AI agents (on by default only for L0 and L1 plugins; only the user changes them);
- services, processes, devices, companion apps and hotkeys;
- declared GPU quota and each model's peak memory, with a one-tap raise if it exceeds the default;
- the overall risk level and, for anything not Official, a third-party disclaimer.

The risk level is the highest level of any permission the plugin holds:

| Level | Sentence the user sees |
|---|---|
| L0 | Works on what you select |
| L1 | Works with your documents |
| L2 | Changes how the app works |
| L3 | Reaches your files and programs |
| L4 | Full access to your computer |

Each permission has one level, and its scope can lower it: `document.edit:selection` is L0, while `document.edit:document` is L1. The host computes the plugin's level from this table, which is generated from the permission records. Every flagship card shows its sheet and the level it derives.

| Level | Permissions at this level |
|---|---|
| L0 | app.view, bus:<topic>, clipboard.write, document.edit:selection, document.read:selection, files.read:picked, fonts.read, licensing.install_id, os.reveal, selection.write, service:<plugin>/<name>, ui.open_url |
| L1 | app.automation, app.events, app.intercept:before-close, app.intercept:before-export, app.intercept:before-print, app.intercept:before-save, app.intercept:drop, app.intercept:import, app.intercept:input.filter, app.intercept:open, app.intercept:paste, app.intercept:pen.filter, app.intercept:tool:<id>, app.session, clipboard.read, collab.presence, collab.read, comments.write, document.create, document.edit:all, document.edit:document, document.lifecycle, document.links, document.read:all, document.read:document, document.scripts, files.document-folder:{read,write}, files.folder:picked, files.watch, fonts.activate, gpu.compile, gpu.direct, gpu.timing, history.control:navigate, history.control:snapshot, history.read, identity.user, models.install, os.handoff, plugins.data:<id>:{read,write}, plugins.list, print, threads=shared, ui.web, ui.window:floating |
| L2 | app.intercept:color.picker and every other Override point, app.intercept:command:<id>, app.intercept:command:<prefix-glob>, app.intercept:export.transform, app.lifecycle, app.preferences:<group>, app.scripts, app.shortcuts, app.ui.automate, app.windows, clipboard.monitor, collab.write, devices.companion:<id>, devices.gamepad, devices.hid:{picked,<vid>:<pid>}, devices.midi, devices.serial:picked, devices.usb:{picked,<vid>:<pid>}, files.paths, fonts.system, input.app, input.hotkey:<chord>, media.capture:{camera,microphone}, os.open, plugins.control, plugins.install, ui.window:{modal,fullscreen,topmost,overlay} |
| L3 | files.known, files.read:home, media.capture:screen, native.restricted, process.run:confined |
| L4 | files.write:home, input.monitor, native.full, process.run:full, process.run:shell |
| Never | N1 to N7: no grant exists |


Only real hardware input can approve the sheet. The Enable button stays inert for 1.5 seconds so a stray click cannot approve it. An L4 plugin adds a second sheet with a checkbox that must be ticked by hand [CON-SEC-consent-sheets@1]. The reason is simple: a plugin must never be able to produce consent for itself, and an agent cannot install, enable or grant anything.

There are no surprise prompts later. If a plugin calls something it was not granted, the call returns `-2 Denied` with the missing permission named. The plugin may ask the host to show a request sheet; the user decides.

### Enabled, then loaded

Once approved, the grants are stored per app and the plugin's menus, panels, tools and hooks are registered from `index.bin`. No plugin code has loaded yet. The plugin is **Enabled**.

At app start the shell reads only `index.bin`. Panels that were open last time draw from a saved display list, so they appear in the first frame without running the plugin. Workers start after the first frame, a few at a time, visible panels first.

The first real event (a click, a render, a document change) **loads** the plugin. A warm, pre-started worker from a pool is assigned to the package, so loading costs microseconds rather than a process launch. Each package version has its own worker.

The whole path is one state machine shared by every app, the launcher and the command line [CON-COM-lifecycle-states]:

Available → Downloaded → Installed (disabled) → Prepared → Enabled → Loaded

From there a plugin can go idle, be held by an update that asks for more, be quarantined after repeated faults, or be removed. Later sections cover each.

## What happens on every call

### Where plugin code runs

Plugin code runs in `craft-plugin-worker`, a separate process in an OS sandbox with no network, no direct file access and no access to other apps. WebAssembly runs on Wasmtime. JavaScript and TypeScript run on V8 on desktop and QuickJS on the web builds. Plugins that need native machine code ship it as a native helper, which runs in its own confined process and talks to the plugin over shared memory.

The worker holds no OS authority of its own. When a plugin opens a file the user picked, the host hands over a descriptor for that one file.

### Three ways to reach the app, one gate

A plugin can reach the app three ways:

1. **Canonical ops.** 87 portable operations (`node.get`, `selection.set`, `prop.set`, `effect.apply` and so on) that mean the same thing in every app. An app without a concept answers `-8 Unsupported`.
2. **Platform ops.** Things the platform itself provides: UI (`ui.set`, `ui.window.open`), storage, jobs, GPU, inference (`nn.run`), files through handles.
3. **The app bridge.** `app.run` and `app.batch` reach any plugin-visible app command by id. This is what gives full reach. Plugins reach 755 of EffectCraft's 757 commands, and every one of FigCraft's 630 commands that is not refused [REQ-FC-reach-every-command].

All three go through one function, `gate::admit` [CON-COM-call-surfaces@1]. Adapters only map; they never decide.

### What the gate does

Every call passes the same ordered steps [OP-COM-gate-admit]. In plain terms:

1. **Resolve** the op and any references to earlier steps in a batch.
2. **Check the shape** of the arguments against the schema, with bounded sizes. Any path argument is checked to refuse network locations.
3. **Look up exposure.** Each app command declares metadata (`Meta`) describing what it does: reads the document, writes it, touches files, opens a URL and so on. A pure function turns that metadata into the one permission the command needs, or into a refusal.
4. **Derive** the exact permission for these arguments. Writing to a file derives `files.write`; writing to a live shared document derives `collab.write`.
5. **Check the grant and the context.** The permission must be declared and granted. Some effects are gesture-only: they need the user to have just pressed something, not an automatic event.
6. **Apply policy.** Build switches, organisation policy and the kill list.
7. **Check scope.** A plugin granted `document.edit:selection` can write only the selection captured when the event started, plus anything it creates.
8. **Check size and memory.** Nesting limits, budget and whether the step would push memory past the app's headroom.
9. **Route.** Cheap calls run directly. Heavy ones run off the UI thread on a snapshot.
10. **Execute,** then **post-check** that every change stayed inside the scope.

Steps before execution cost at most 5 microseconds at p99 and allocate nothing.

The gate's output is an **Authorization**: origin, effect mask, scope token and file authority. It travels by value with the call into every sink. A sink is the single choke point for one kind of effect (files, clipboard, processes, preferences, print). The sink checks the Authorization again. Nothing about who is calling is stored in a global, so a queued job can never pick up someone else's authority.

### Context: invoked or automatic

The runtime, not the plugin, decides the context of each call. A call is **Invoked** only when it traces back to hardware input: a menu click, a tool drag, a shortcut. Hooks, timers, document-change events and jobs are **Automatic**. Some effects, such as opening a link in the browser, are allowed only when Invoked. Automatic writes to the document need `app.automation`.

### Reads, writes and undo

Reads run on an immutable snapshot of the document, off the UI thread. A plugin can read 100,000 nodes without the UI noticing.

Writes collect in a private transaction log. When the event ends, the commit is one quick hop onto the owner (UI) thread. If the user edited something else meanwhile, the runtime merges the two sets of changes off the owner thread. If they overlap, it replays once, then takes a short lease to finish.

One plugin event is one transaction and one labelled undo step [CON-COM-one-event-one-step]. A plugin that creates a thousand nodes in a batch makes one IPC call and one undo step. The SDK handles stale revisions, batching and retries, so authors do not have to get these right.

The owner thread gives all plugins together a fixed slice of each frame, shared round-robin. No plugin command can stall it past that slice.

### Time limits

Every kind of call has a named class with its own budget, held in one table, `BUDGETS`. A few examples:

- a user-invoked event gets 5 seconds, then becomes a modal operation with a progress bar and Cancel;
- an automatic event gets 1 second, then is cancelled and rolled back;
- a slider preview gets 16 ms at p95, then drops to half resolution, quarter resolution, then the last good frame with a badge;
- a before-save hook gets 500 ms for all plugins together, then save goes ahead.

Final renders and exports have no deadline and never degrade. They show a heartbeat and can be stopped.

### Pixels, audio and GPU

Media never gets copied through messages. Pixels and audio move through shared memory arenas the host creates and seals. GPU work stays on the GPU and is shared by texture handle.

GPU code comes in four tiers, named GT1 to GT4 so they are not confused with risk levels. GT1 kernels and GT2 pass graphs are free. GT3 (recorded command lists) needs `gpu.direct`. GT4 draws in the viewport. Every shader is validated. New shaders start on a low-priority **probation** device. A shader is **promoted** onto the app's own render path once its cost is proven bounded, or after 120 clean frames within budget. A shader with unbounded loops stays on probation, so it can never hold up the app's frame.

## EffectCraft

**Flagship: Ember** [FLG-EC-ember]. Ember is a GPU particle system in the class of Trapcode Particular: emitters, physics, sprites, up to 10 million particles and a Designer window. It is a milestone M5 plugin.

### What the sheet says

| Permission | Why Ember needs it |
|---|---|
| `document.read:document` | Emitters can be other layers; particles react to comp cameras and lights |
| `document.edit:selection` | Edits Ember's settings and keyframes on the selected layers, for example from the Designer |

Risk level L1, "Works with your documents". Ember is Official, so the sheet carries no third-party disclaimer. A Community build of the same plugin would add one.

The sheet is not written by hand. A test derives each flagship's permissions from the ops it actually calls and fails if they differ from the manifest. Reading only its own layer through `render.request` derives no grant at all.

### Install and prepare

Ember's main contribution is an effect under `raster.filter@2`, the one contract for effects and generators on CPU, GPU, stepped simulations and native code [CON-COM-raster.filter@2]. A manifest can list an ordered fallback of execution paths, for example GPU passes first and a CPU WebAssembly module second.

At prepare the host validates every WGSL module, derives its GPU feature needs and worst-case cost, and queues declared-hot pipelines to warm at enable. So the first frame after Enable does not hitch on a shader compile.

### Applying it

The user drags Ember from Effects & Presets onto a layer. The host applies it as the user's own edit, creates an effect instance and draws Ember's parameters in Effect Controls using the shared parameter kinds. Point parameters such as emitter position use **declared handles** in the viewer: the host draws and drags them itself with no call to the worker. Dragging a handle is one undo step.

The Designer is a plugin palette window opened with `ui.window.open`, which needs no grant. Its layout is a retained UI tree the plugin sends once and patches when values change. The app paints it; the plugin never paints a frame. The UI thread applies prebuilt patches within its per-frame slice and never calls the worker.

### Each frame

For each frame the render path asks Ember for a **plan**: the region it will draw, the inputs it needs and how it will execute. Inputs come through ops:

- its own layer and bound parameters: free, no prompt;
- emitter layers matched by name or tag through `comp.layers`: needs `document.read:document`, and validation refuses Ember without it;
- the comp camera and lights at that time.

Ember is a **stepped** effect: frame N depends on frame N−1. It keeps simulation state in SimCache, a set of checkpoints charged to Ember's own GPU, CPU and disk accounts. Over GPU quota they spill to disk off-frame. At the disk cap they re-space evenly, so a seek never steps more than a known number of frames. Seeking anywhere in a 10-million-particle sim takes at most 250 ms.

GPU passes run on the app's queue once promoted. While a new shader is on probation, the viewer shows the last completed render and stays responsive. Pixels never leave the GPU: the target is zero readbacks.

### When a frame is too slow

The render ladder decides what to show [CON-COM-renderdecision]. It returns one of five answers: Full, Scaled, LastGood (with a badge), Durable (with a badge) or Frozen. EffectCraft only shows what it is told. During playback each plugin's share of the frame is predicted. The largest predicted cost steps down first, and only when a miss is predicted.

RAM preview fill (Preview > Play) runs at full quality with no per-plugin deadline and caches only true frames. Export never degrades either. A stalled export shows the plugin's name after 10 seconds.

### Without Ember

Final renders write **durable output**: EXR frames of the layer after its last plugin effect. If someone opens the project on a machine without Ember, those frames render with a badge saying Ember is missing. The project still opens, plays and exports.

If the GPU resets, the app keeps the project open. Ember's GPU resources are rebuilt through `gpu::Persistent`, SimCache falls back to CPU or disk checkpoints, and rendering resumes within 5 seconds.

### Accepted when

- 1 million particles at 1080p30 with 0 dropped frames;
- under 50 µs of worker time per frame and 0 readbacks;
- golden images match on three GPU backends within the tolerant class;
- RAM preview writes 0 durable bytes.

| Step | Op or contract | Permission |
|---|---|---|
| Apply to layer | `raster.filter@2` (user's own drag) | none |
| Edit settings | `prop.set`, `key.set` | `document.edit:selection` |
| Read emitter layers | `comp.layers` | `document.read:document` |
| Read own layer | `render.request` | none |
| Designer window | `ui.window.open`, `craft.ui@1.5` | none (palette window) |
| Simulation state | SimCache on plugin accounts | none |
| GPU passes | `gpu.pass`, tier GT2 | none |

## PhotoCraft

**Flagship: Lumina** [FLG-PC-lumina]. Lumina is a Topaz-class enhancer: denoise, sharpen, face recovery and 2× to 6× upscale with packaged models, autopilot suggestions, a split before/after preview and folder batch [CAP-PC-lumina-features].

### What the sheet says

| Permission | Why Lumina needs it |
|---|---|
| `document.edit:document` | Adds result layers and smart filters anywhere in the document |
| `document.create` | Upscale can produce a new document |
| `files.folder:picked` | Batch reads and writes a folder the user picks |

Risk level L1. The sheet also shows each model's peak memory. A 4 GB model loads at default settings; if a model would exceed the plugin's default memory account, the sheet offers a one-tap raise.

Running inference itself needs no permission. Compute is free and never prompts.

### Install and prepare

The models ship inside the package. At prepare the host checks each one against its sha256 in the manifest. A wrong hash is refused. Models live in one memory-mapped store shared across the suite, charged to the plugin that maps them. No plugin fetches a model from anywhere.

### Applying it as a smart filter

The user picks Filter > Lumina > Denoise on a layer. Lumina runs as a `raster.filter@2` smart filter, so the layer stays editable and the filter can be re-tuned.

Inference runs through `nn.run` in `craft-nn-helper`, a confined helper process started per plugin. It uses CoreML on Mac, ONNX Runtime on Windows, or candle as a fallback. Tensors are shared memory views or stay on the GPU; nothing is copied between worker and host. The image is processed in tiles, so a 100-megapixel photo never has to fit in memory at once.

While the user drags the strength slider, the preview runs under the slider-preview budget: 16 ms at p95, first pixels within 50 ms. If a frame would miss, the ladder drops to half or quarter scale, or shows the last good result with a badge. On release the full-quality result renders as a job. The split before/after view is a host T4 viewer, so zoom and pan never call the plugin.

While a denoise runs, UI frames stay within one vsync at p99.

### Upscale and batch

Upscale outputs at `scale(k)` or into a new document, which is why `document.create` is on the sheet.

Folder batch uses the `automate@1` contract over a folder the user picked. Lumina spawns jobs with `job.spawn` and wraps the run in `txn.modal`, which gives one progress bar, one Cancel button and one undo step. Each plugin may run 4 jobs at once, the app 16. Cancel takes effect within 100 ms, and a cancelled batch leaves no half-written files.

### Without Lumina

The smart filter stores **durable output** at full resolution by default. The file opens and renders without Lumina, with a badge. Saving after one edit writes only the tiles that changed.

### Accepted when

- 24 MP 2× upscale in at most 8 s on CoreML, 20 s on candle;
- output within the neural-network golden class on every backend;
- the durable output renders without the plugin;
- wrong model hash refused; a cancelled batch leaves 0 outputs.

| Step | Op or contract | Permission |
|---|---|---|
| Model check | prepare, sha256 | none |
| Smart filter | `raster.filter@2` | `document.edit:document` |
| Inference | `nn.run` in `craft-nn-helper` | none |
| New document | upscale output | `document.create` |
| Folder batch | `automate@1`, `job.spawn`, `txn.modal` | `files.folder:picked` |

## FigCraft

**Flagship: Flowline** [FLG-FC-flowline]. Flowline draws connectors between frames. They follow when frames move, snap to frame edges, and carry styles, labels and a relaunch button. It matches Autoflow and works in live multi-user sessions.

### What the sheet says

| Permission | Why Flowline needs it |
|---|---|
| `document.edit:document` | Connectors attach to frames anywhere on the page |
| `app.automation` | Redraws connectors automatically when frames move |
| `collab.write` | Those redraws must reach other people in a live session |

Risk level L2, "Changes how FigCraft works". The sheet adds a notice explaining that this plugin's edits sync to collaborators. Every L2 permission must carry a short reason written by the author, shown on the sheet.

Live sync is a host-owned channel, not plugin network. The FigCraft server checks, stamps and meters each plugin write, and the Inspector shows which plugin wrote each piece of plugin data [DEC-COM-d-o].

### Drawing a connector

The user picks the Flowline tool and drags from one frame to another. While dragging, snapping uses the `snap.provider@1` contract. Flowline returns candidate points once, at gesture start. The host then snaps on every pointer move without calling the plugin. This needs no permission.

On release Flowline creates the connector. The SDK's `cx.edit` batches the creates, moves and styling into one `app.batch` with step references, so the new connector's id can be used by later steps without a round trip. That is one IPC call and one undo step. Flowline stores which frames each connector links as **plugin data** on the connector node. The data lives in Flowline's private namespace; only Flowline writes it.

### Following moves

Flowline subscribes to `document.changed`. Each frame, the host sends one columnar change list: which nodes moved, were created or deleted, and why. When a frame moves, Flowline calls `app.figcraft.node.query` to find connectors whose plugin data names that frame. The query is indexed and answers in at most 8 ms at 100,000 nodes. Then Flowline rewrites the connector path.

These are Automatic events, which is why the sheet needs `app.automation`. Each has a 1-second budget. Edits from automatic events merge into one undo entry per plugin until the user's next command. Undo after a drag removes the drag and the connector update together, not one step per frame.

On the user's own machine the connector redraws within one frame at 60 and 120 Hz. Reads run on the snapshot off the owner thread. With 50 connectors in a drag the UI stays within one vsync at p99.

### In a live session

When two people edit the same file, the gate derives `collab.write` for any plugin write to the live document. It checks the grant per step and again at commit. Widgets are the one exception: a widget's votes, checks and state travel on the host's own widget channel, not as document writes, so a widget declaring only `collab.read`, `collab.presence`, `ui.web` and `ui.notify` needs no `collab.write` [CON-COM-collab-live-write@1]. Writes to a live document take a live-writer lease, so a peer's incoming batch always applies within one frame and never waits behind a plugin.

Only one machine should redraw a connector for a given move. The peer who dragged redraws it when it has Flowline; otherwise the server elects one eligible **runner** among peers that do [CAP-FC-flowline-features]. Peers see the redraw within 100 ms. Flowline's broadcasts are capped at 30 batches a second per document. A peer without Flowline still sees the connectors, because they are ordinary nodes in the document.

Relaunch buttons appear in the Inspector's Plugins section on connector nodes. Clicking one is an Invoked event.

### Accepted when

- connectors redraw within 1 frame for the dragging user and within 100 ms for peers;
- runner-only redraw within 100 ms;
- one undo step per drag;
- at most 30 broadcasts a second.

| Step | Op or contract | Permission |
|---|---|---|
| Snap while drawing | `snap.provider@1` | none |
| Create connector | `app.batch` via `cx.edit` | `document.edit:document` |
| Link data | plugin data, private namespace | none |
| React to moves | `document.changed`, `app.figcraft.node.query` | `app.automation` |
| Sync to peers | `collab-live-write@1` | `collab.write` |

## How plugins are managed

### One manager, everywhere

Every app has Window > Plugins, built from one shared model [REQ-COM-manager-in-app]. It has tabs for Installed, Discover, Updates, Activity, Problems and Settings, plus a page per plugin. The same actions exist on the command line as `<app> plugins ...` and in the suite launcher. Headless runs never prompt. Approving a widening from the command line needs `--approve-widening` on a real terminal.

On each plugin's page the user controls every optional permission, every contribution, every hook and every agent-exposed command. The user can also run a plugin's GPU work on an isolated device. The **Activity** tab shows a per-plugin log written only by the host, with filters and a Reveal button for files the plugin touched. Reports drawn from it exclude text the plugin produced, so a plugin cannot plant messages there.

Organisation policy is read before any plugin loads. It can only narrow what the build allows, never widen it.

### Idle and suspend

A plugin declares how long its worker should live. With `keep_alive = "60s"` the worker is suspended after 60 seconds of idle. It is not suspended while audio is armed, a live preview or visible panel is open, a render or job is running, or a transaction is open. With `"session"` the worker stays while the app runs, and the manager shows its memory. Either way, suspension gives the plugin 50 ms to save its state, and the next event brings it back with that state.

### Updates

Updates install automatically after a cool-off: 72 hours for Community, 24 for Verified, none for Official [REQ-COM-lifecycle-update]. The delay gives revocation time to catch a bad release.

If an update asks for anything more than the user approved, it is a **widening**. The host compares the whole capability surface, not just the permission list: new hooks, a larger GPU quota, new processes, new agent exposure. A widening holds the update. The plugin stays on its current version, and the new version waits in state **Held** until the user approves a new sheet.

The previous version is kept as a rollback slot. The user can pin a plugin to a version; pinned versions never auto-update. Rolling back restores that version's grants.

When the app, a JavaScript engine, the OS or a GPU driver updates, cached compiled code misses. Re-preparation is lazy: the first frame never waits, a background queue re-prepares at most two plugins at a time, and anything used before its turn is prepared on demand.

### Removal

Uninstall kills the plugin's workers and unregisters its contributions. The package is deleted once no app uses it. The host asks whether to delete settings and private storage too. Documents keep their durable outputs and plugin data, so files stay usable without the plugin.

Unknown plugin data in a document is kept byte-identical on save. Opening and saving a file on a machine without the plugin loses nothing.

### Switches

Some capabilities were owner decisions with two possible answers, such as clipboard monitoring or live writes. Each one is built and tested both ways and chosen by one field in `BUILD_DEFAULTS`. Changing a decision is a one-line data change, not new code. If a switch is off, manifests that use it still parse. The plugin shows as Held with the reason, and its grants and data stay.

## When things go wrong

The platform plans for each kind of failure. The rule is the same throughout: contain it to the plugin, roll back half-finished edits, show what happened, keep the document safe.

### The plugin crashes or hangs

A crash kills only the worker. The current event's transaction rolls back. The worker restarts on the next event. The user sees a notice naming the plugin.

A hang is cut off by its time limit. On revocation, a call is trapped within 260 ms, or the worker is killed.

Each fault is a **strike**. Three strikes in ten minutes disable the plugin for the session. Three such sessions **quarantine** it [CON-COM-quarantine-triggers]. Quarantine ends only when the user re-enables the plugin in the manager.

Revocation is different. When the store revokes a release as unsafe, that version becomes **Revoked**, never Quarantined. The user cannot re-enable it. It leaves that state only by updating to a release the store has not revoked, or by rolling back to one [CON-COM-lifecycle-states].

If an app command panics while serving a plugin, the runtime catches it. The transaction rolls back to its mark and the plugin gets `-7 Internal`. That counts no strike, because the fault was the app's.

### It is too slow

Each call class has a fallback. Automatic events are cancelled and rolled back after 1 second. User events become a modal operation with Cancel after 5 seconds; scripts after 2. Renders step down the ladder to a badged last-good or durable frame. Input filters on a pen stream fall back to raw input if they miss 8 ms.

Hooks have budgets per hook point, never per plugin, so adding plugins does not add delay [CON-COM-hook-budget-per-point]. A save waits at most 550 ms however many before-save hooks loop. Hooks fail open: no plugin can block save, export, quit or close. The user can also choose Save, Export or Render without plugin checks.

### It uses too much

Every allocation the host makes for a plugin is charged to that plugin's account: CPU memory, GPU memory, disk, undo and prompts. Over its cap, only that plugin's call fails with `-6 Budget`. Under memory pressure the host first spills its own cache, then evicts plugin caches, suspends idle plugins, drops persistent GPU targets and checkpoints simulations to disk. Only after all that does a plugin call fail.

A plugin that floods calls is throttled at 64 in-flight owner calls. A test runs ten chatty plugins together and checks that the UI frame holds.

### The GPU misbehaves

GPU work pending for more than 1 second interactively, or 10 seconds in export, strikes its shader. A journal records which plugin and shader ran before each submit, so a reset is blamed on the right plugin and never on the app. Two device losses or three watchdog strikes in 24 hours quarantine the plugin. On a device loss the app keeps its documents open and rebuilds the GPU device behind the scenes.

### The user wants it all to stop

**Stop all plugins** is a fixed key chord, handled before any plugin sees input and impossible to rebind [CON-COM-stop-all]. It cancels every running plugin event, closes plugin modals and full-screen windows, unregisters hotkeys and kills helpers within 1 second. Double Esc, or holding Esc, always reaches the host.

**Safe mode** starts the app with no plugin code, hooks, intercepts, filters, shortcut overrides or data contributions. The manager still works. It starts with `--safe-mode`, Shift held at launch, `CRAFT_SAFE_MODE=1`, or automatically when the app crashes twice within 60 seconds of start and the crash journal names a plugin.

### Saving through all of this

Save, autosave, export, collaboration sync and crash-recovery snapshots read only the last committed revision, never a transaction in progress. Saves are atomic: a crash leaves either the old file or the new one, complete. A plugin's half-finished work can never reach disk.

### What a plugin sees

Every failure returns a code from one table:

| Code | Meaning |
|---|---|
| -1 Invalid | Bad arguments, too large, too deep |
| -2 Denied | Missing permission, named in `needs` |
| -3 NotFound | Absent, or outside the plugin's scope (identical answer) |
| -4 Stale | Document moved on; the SDK retries |
| -5 Cancelled | User cancel or Stop all |
| -6 Budget | Over a time, memory or size cap |
| -7 Internal | App fault; no strike |
| -8 Unsupported | This app or device lacks it |
| -9 Excluded | Never available in this build |
| -10 StaleUi | A UI patch was built on an old revision of its surface; the SDK resends from the current one |
| -11 Gesture | Needs a real user gesture, so it is refused from timers, hooks and jobs |
| -12 Locked | A pixel, transparency or position lock covers the target; FigCraft does not use it [DEC-FC-no-locked-error] |

An out-of-scope target returns the same answer as a missing one. This way a plugin cannot probe for things it is not allowed to see.

## What plugins cannot do in this build

Almost everything the app can do through its MCP server or command line, a plugin can do too, plus windows, docked panels, menus, tools, overlays and behaviour hooks [REQ-COM-zero-capability-compromise]. Broad reach is granted openly, with a risk level and disclaimer, rather than refused. Full file access, running any program and system-wide input monitoring are all available at L4 behind the two-step sheet.

What remains is one short list, the **never list**, N1 to N7. Each has a reason:

- **N1, consent on the user's behalf.** Only hardware input approves sheets or counts as a gesture.
- **N2, acting as another principal.** A plugin cannot act as the user, an agent or another plugin.
- **N3, host security state.** A plugin cannot change grants, policy, trust or the never list itself.
- **N4, safety rails.** A plugin cannot disable Stop all, safe mode, Esc or attribution strips.
- **N5, code inside the app process.** On desktop and web every plugin runs out of process. On iOS, where the OS allows no extra processes, plugins run in a memory-safe interpreter on their own thread under the same gate and containment.
- **N6, network.** Covered below.
- **N7, blocking save, export, quit or close.**

### No plugin network

This build gives plugins no network path. That is an owner decision [DEC-COM-no-plugin-network], not a gap to fill. No worker, helper or web page has a socket. Network permissions are not in the vocabulary. Node-style `http`, `net` and `dns` modules return `-9 Excluded`. Every app's network sink refuses any call chain that includes a plugin.

The decision costs little in practice. EffectCraft plugins reach 755 of its 757 commands; the two left out are the `*.model.download` commands. All 48 top Figma plugins work offline: 31 at full parity, 8 through an offline equivalent and 9 without their network-only part. Models ship in packages or are picked by the user. A plugin can still hand a link to the user's browser after a sheet showing the full address, and the browser does the network. Host-owned channels stay open, because the host runs them and the plugin never holds a socket: the store, the catalogue, FigCraft live sync under `collab.write`, and a signer-pinned relay to local licensing daemons.

### Placement rules, not missing features

A few things are not refused but placed. Plugins never ship file codecs; image, video and PDF decoding runs in the host's confined decoder, which plugins call through codec ops. Expression libraries evaluate only in the dedicated expression worker. No app keeps a script engine or a JIT in its own process.

### Owner decisions still open

Three decisions wait on the owner. None removes a plugin capability. D-D and D-K are built for either answer, so nothing waits on them. D-T is different: the M6 beta exit, where the three apps ship, is blocked until the owner answers it.

- **D-D:** whether imported web panels dock (recommended: dock them);
- **D-K:** when native plug-in hosts for Photoshop and After Effects binaries ship in public releases (recommended: in beta now, public after review);
- **D-T:** approval of hardware CI runners (recommended: approve them). Hosted runners close M4 and M5, but M6 needs a recorded hardware run of every flagship target.

The decision records list the options and the reasoning.

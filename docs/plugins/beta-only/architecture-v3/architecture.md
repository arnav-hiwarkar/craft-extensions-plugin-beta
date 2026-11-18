# Plugin architecture

This is the architecture an engineer builds from. It names what exists, how the parts connect and which decisions shaped them. Ids in square brackets point to the specification records. The generated tables are the reference. The prose explains them.

## The system in one page

One plugin platform serves every Craft app. The shared code lives in two repositories. `craft-libs` holds the libraries every app already uses, such as commands, GPU, UI and text. `craft-extensions` holds the plugin platform: the runtime, the worker, the engines, the store, the UI framework, the SDKs and the author tools. Each app adds one thin adapter crate, `crates/plugins`, that maps the app onto the platform. Apps take `craft-libs` and `craft-extensions` as path dependencies through a patch table, so all fifteen sibling repositories build together [DEC-COM-craft-libs-patch-table].

The owner's rules set the shape. Plugins get full capability, so anything the app can do through its MCP server or CLI, a plugin can do too, plus windows, docked panels, menus, tools, overlays and behaviour hooks [REQ-COM-zero-capability-compromise]. The only exceptions are network access, self-elevation, the safety rails and running code inside the app process. Apps must stay fast and never crash, so on desktop and web no plugin code runs in the app process, on iOS an in-process memory-safe interpreter runs under the same gate, and the UI thread never waits on a plugin. Behaviour lives in one place, so the runtime owns every rule and adapters only map.

A plugin call takes the same path in every app:

1. Plugin code runs in a sandboxed worker process (`craft-plugin-worker`) or a confined helper, never in the app.
2. The call crosses ABI 2: descriptors and shared memory rings, with media passed as shared views or GPU handles.
3. The runtime copies the payload out of shared memory once, then hands it to `gate::admit`.
4. The gate resolves the call, checks schema, exposure, permission, context, policy, scope, shape and budget, then chooses where it runs.
5. The command runs on a snapshot, on the job pool or as one hop on the app's owner thread, inside a transaction.
6. Every effect leaves through a sink that checks the `Authorization` the gate minted. Results go back over the same rings.

Four principles follow from this path. There is one gate for every call surface [REQ-COM-principle-one-gate]. The app keeps authority and the plugin holds only host-minted handles. The UI thread never waits on or runs plugin work. Media moves without copies. Each principle has a deterministic test, because the owner wants code as proof and no code comments.

No plugin network exists in this build. This is an owner decision, recorded as DEC-COM-no-plugin-network. Network permissions are absent from the vocabulary, every app's network sink refuses any call chain that holds a plugin, and the sandboxes deny network. The host keeps its own channels for the store, the catalogue and live document sync.

## Layers and components

The platform has seven layers. Each layer depends only on the layers listed before it.

- **libs** (`craft-libs`). Shared building blocks. `craft-command` owns command metadata (`Meta`), effects, scope, undo, cost, `ParamSchema`, `exposure()`, the never list, `Origin` and `Authorization`. `craft-command` sinks are the only path to files, clipboard, processes, preferences, fonts, print, devices and network. `craft-gpu` owns the device broker, memory accounts and the GPU scheduler. `ui`, `ui-editors-core`, `craft-text`, `craft-media`, `craft-graph`, `craft-mesh` and `craft-preserve` complete the set.
- **platform** (`craft-extensions`). `ext-core` defines ids, the manifest, the permission vocabulary, risk and notices. `ext-runtime` holds `Runtime<A>`, the gate, contexts, transactions, events, jobs, hooks, brokers, budgets, accounts, policy and audit. `ext-worker`, `ext-engine`, `ext-ipc` and `ext-shm` run plugin code and move data. `ext-broker` and `craft-sandbox-launch` supervise and confine processes. `ext-gpu`, `ext-ui-model`, `ext-ui-paint`, `ext-ui-fw`, `ext-web` and `ext-surface` cover GPU and UI. `ext-store` and `ext-manager` install and manage packages. The native hosts (`craft-pshost`, `craft-aehost`, `craft-ofx-bridge`, `craft-audiohost`) run incumbent binaries.
- **sdk**. `craft-guest` is the Rust guest runtime. `craft-native` is the helper SDK. `ext-bindgen` generates bindings for Rust, C, Zig, AssemblyScript and TypeScript from the locks. `@craft/node-compat` and the compatibility shims (`figma-compat`, `photoshop-compat`, `extendscript@1`, `cep-compat`) live in `ext-js`. `MockHost` runs plugin tests against the real lock and gate.
- **adapter**. Each app's `crates/plugins`: `impl CraftApp`, the `DocumentHost`, the canonical map, the change source, pixel and GPU views, hook points, drop targets and `HostUi`.
- **app**. The app's own crates, changed only where they meet the platform: `CommandSpec` gains `Meta`, sessions take an `Authorization` by value, and render, export and UI code gain plugin slots.
- **tool**. `craft-ext` (new, build, dev, test, bench, replay, lint, pack, sign, submit, doctor), `ext-conformance`, the docs generators and `craft-launcher`.
- **plugin**. Flagship plugins and vendor bridges, built on the public SDK like any third-party package.

The adapter boundary is the most important seam. `CraftApp` is the only app adapter trait [CMP-COM-craft-app-trait]. It gives the runtime the app's info, command registry, canonical map, documents, change source, plugin data, pixels, GPU views, hooks, context keys, surfaces, host UI, GPU, audio, memory headroom and document limits. `AppExtensions` adds app-specific ops, formats, timeline and audio hosts, native host kinds, drop and row targets, viewers and event families. An adapter op declares `Meta` like a registry command and passes the same gate. No app may define a second snapshot, restore or extension trait.

Interfaces freeze at fixed milestones so streams can work in parallel. WS1 owns the `craft_command` types. WS2 owns `CraftApp`, `DocumentHost`, `AppExtensions`, `HostUi`, `ViewerHost` and `PickHost` and the `platform.lock.json` schema, frozen at M1. The permission and risk interface freezes at M1 in WS7. ABI 2 and the launcher profiles freeze at M2. The GPU interface and the SDK surface freeze at M3. After a freeze, a change needs the owner thread's version bump.

Six lock files pin everything a plugin can see [CON-COM-lock-files@1]. Each app has `api.lock.json` with every exposed command, its schema hashes, derived permissions and exposure. `platform.lock.json` holds op schemas, contract levels, the permission vocabulary, contexts, errors, budgets and manifest keys. `api-widening.log` records every widening with a reason, and the owner reviews each one. Documents carry `plugins.lock`. Headless nodes read an admin-signed `farm-policy.toml`. Each plugin source has `craft.lock`. SDKs, API pages and permission tables are generated from these locks, and CI fails on drift.

The table counts the components in each layer and names the five that the most other components depend on. Every component has its own record in the spec, with its repository, dependencies and owning workstream.

| Layer | Components | Repositories | Most depended-on |
|---|---|---|---|
| libs | 20 | craft-extensions, craft-libs, egui | CMP-COM-craft-command-sinks, CMP-COM-craft-command, CMP-COM-craft-gpu, CMP-COM-ui-editors-core, CMP-COM-craft-audio-dsp |
| platform | 116 | craft-extensions, craft-libs, others | CMP-COM-ext-runtime, CMP-COM-craft-app-trait, CMP-COM-ext-engine, CMP-COM-ext-worker, CMP-COM-runtime |
| sdk | 19 | craft-extensions, craft-guest, figcraft | CMP-COM-ext-js-node-compat, CMP-COM-craft-guest, CMP-COM-craft-native, CMP-COM-craftapp-kernels, CMP-COM-mockhost |
| adapter | 86 | craft-extensions, effectcraft, others | CMP-FC-plugins, CMP-COM-ext-js-e4x, CMP-COM-ext-js-jsxbin, CMP-COM-aecommands, CMP-COM-app-extensions |
| app | 85 | effectcraft, figcraft, photocraft | CMP-EC-expr-split, CMP-EC-render-cache, CMP-EC-api-lock, CMP-EC-automation-confine, CMP-EC-automation-tools |
| tool | 21 | craft-extensions, craft-launcher, others | CMP-COM-ext-bindgen, CMP-COM-check-submission, CMP-COM-craft-ext, CMP-COM-craft-launcher, CMP-COM-ext-cli-import |
| plugin | 2 | craft-ofx-bridge, craft-plugins | CMP-COM-deckhand-vendor-bridge, CMP-EC-sdk-ofx-bridge |


### Contracts between components

A contract is a named, versioned guarantee with a budget and the tests that prove it. Plugin-facing contracts (`raster.filter@2`, `audio.effect@1.0`, `anim.ease@1.0`) define what a contribution kind must do. Internal contracts (`owner-frame-budget@1`, `abi2-transport@2`, `gate-authorize@1`) define what one component promises another. The `CallClass` rows are the budget table: every kind of plugin work has exactly one row, and every row ends its degrade ladder in a visible badge.

The table lists the contracts named in this document with their budgets. The spec holds the full set.

| Contract | Budget |
|---|---|
| CON-COM-raster.filter@2 | Per-CallClass interactive budget on a mid-range GPU; output_intermediate_copies = 0 |
| CON-COM-audio.effect@1.0 | CMP-COM-craft-audio-rt row: process <= 50% of a 128-frame/48 kHz block, hop <= 50 us p99.9, 0 added latency blocks… |
| CON-COM-anim.ease@1.0 | <= 2 us per evaluated sample in render, 257-sample baked fallback built <= 1 ms per ease |
| CON-COM-owner-frame-budget | OwnerFrame per BUDGETS (9.5); 0.5 ms or 8 calls per turn; <= 64 in-flight owner calls per plugin; 0 repaints per wake… |
| CON-COM-abi2-transport@2 | batch step <=5 us; native <=50 us |
| CON-COM-abi2-copy-out@2 | one copy per call payload |
| CON-SEC-gate-authorize@1 | admit p99 <= 5 us per call excluding execute; admit_path is one table lookup with no I/O (reference machine) |
| CON-COM-call-surfaces@1 | one gate pass per call on every surface; bridge dispatch overhead <= 5 us per step on the reference machine |
| CON-COM-gpu-tiers@1 | GpuSubmit per segment |
| CON-COM-lock-files@1 | No runtime budget; CI check |
| CON-COM-context-matrix | <= 1 us p99 per call |
| CON-COM-input-pump | routing p99 ≤0.2 ms per event on the UI thread |
| CON-COM-lifecycle-states | state-machine transitions only; no runtime budget |
| CON-COM-collab-live-write@1 | local follow-ups every frame (60-120 Hz); broadcast 30 batches/s per plugin per document, peers <=100 ms… |
| CON-COM-permission-vocabulary | parse-time |
| CON-SEC-never-list@1 | 0 byte differences |


## Runtime and the gate

`Runtime<A: CraftApp>` is the app's single entry into the platform [CMP-COM-runtime]. The app calls `new` at start, `dispatch` for events, `render` for render requests, `frame` once per painted frame, `drain_owner` on every event-loop wake, `input` for each input event, `hook` and `intercept` at hook points, and `stop_all`, `manager` and `safe_mode` from the shell. `hook` and `intercept` return at once. `input` routes from retained hit regions and never calls a guest. `frame` only applies UI the runtime has already built, within the UI budget.

### Documents, snapshots and transactions

The app's owner thread is the only thread that changes a live document. The runtime keeps plugin work off it. Reads and pure commands run on runtime threads against O(1) snapshots. A commit is one owner hop [DEC-COM-doc-access-snapshots]. The reason is the owner rule: the UI must not wait for a plugin.

`DocumentHost` is the trait that makes this work. It gives the runtime `snapshot` (lock-free, O(1)), `committed` (never holds an open transaction), `exec_detached` (pure commands on a private copy-on-write snapshot), `begin`, `install`, `commit` and `abort`, `touched` (the set an edit changed, from a mandatory recorder), `graft`, `validate`, `scratch`, `size`, `live`, `show_working` and `busy`. Each app implements it over its own document model. FigCraft uses `imbl` maps. EffectCraft uses a `Touch` recorder inside `Session::edit`. PhotoCraft uses `DocSnapshot`.

`ext-runtime::txn` implements transactions once for every app. One plugin event is one transaction and one labelled undo step. Edits from automatic events coalesce into one undo entry per plugin until the next user command. If the live document moved during a transaction, the runtime grafts when the touched sets are disjoint. Otherwise it replays once off the owner, then takes a lease. Rebase never runs on the owner thread. Save, autosave, export and sync read only `committed`, so a plugin can never leave half an edit in a file [REQ-COM-committed-reads-persistence].

`Meta` and the document's size choose where a call runs, never the plugin author [DEC-COM-execution-chosen-by-meta]. Pure calls run on the runtime pool through `exec_detached`, with read-your-writes. Heavy calls (`Cost::heavy(size)` above the `OwnerFrame` budget) run on the job pool, report progress after 250 ms and stay cancellable. Other writes take the live-writer lease and hop to the owner. Cost estimates start as static drafts in `Meta` and are corrected by a measured moving average per command and size bucket.

The owner drain loop shares one `OwnerFrame` budget per frame across all live transactions. Each turn serves at most 8 calls or 0.5 ms. Each plugin has at most 64 owner calls in flight. Extra calls wait in the plugin's own worker and the wait is charged to that plugin. The loop runs on every wake, not on paint, so work progresses while a window is hidden. In egui apps the call site is the fork's `App::on_wake`.

### The gate

`gate::admit` in `ext-runtime/src/gate.rs` is the single admission path for every plugin call [CMP-COM-gate]. It is also the only constructor of a plugin `Authorization`, `EffectMask` and `FileAuthority`. Adapters cannot reach the registry with a plugin origin except through it, and a lint enforces that. The budget is 5 µs p99 per call, excluding execution.

The steps run in a fixed order [OP-COM-gate-admit]:

- 0–1: resolve batch references, then resolve the call to a canonical op, platform op, adapter op or bridged app command.
- 2–3: check the schema with bounded, sanitised strings, and check every path argument with `admit_path`.
- 4: read exposure from the lock. `Excluded` and `Never` stop here.
- 5–7: derive the permissions the call needs from `Meta` and the arguments, check them against declared and granted permissions and the current context, then apply switches, organisation policy and the kill list.
- 8–10: rewrite scope, check shape against `LIMITS` and `DocLimits`, and check budget and memory growth against headroom.
- 11: route heavy and long calls off the owner.
- 12: execute.
- 13: check the write set against the call's scope, then log activity and feed cost back.

Three call surfaces share this one gate [CON-COM-call-surfaces@1]. Canonical ops (87 of them) work in every app that has the concept and answer `-8 Unsupported` elsewhere. Platform ops are identical in every app. The registry bridge (`app.run`, `app.batch`, `app.commands`, `app.describe`) reaches every exposed app command from `api.lock.json`. Adapter ops live under `app.<appId>.*` and never shadow a platform or canonical name. A target outside the plugin's scope returns `-3 NotFound`, byte-identical to an absent id, so scope leaks nothing.

### Contexts, hooks, events and containment

The runtime, not the plugin, picks the context of each call [CON-COM-context-matrix]. Only hardware input makes a call `Invoked`. `Automatic` and `Job` calls need `app.automation` to edit. `Hook` and `Render` calls cannot edit outside their point. An `Agent` call carries the agent's own grants and never counts as a gesture. The rows are data in `context.rs`.

Each app declares its hook points in `CraftApp::hooks`, with a kind, payload, result schema, call class and permission. Hooks are ordered and fail open. Budgets apply per hook point, not per plugin, so delay does not grow with the number of plugins. No hook can block save, export, quit or close [DEC-COM-no-plugin-block-save]. Intercepts rewrite a command and the rewritten call runs as the plugin, through the gate. Events are defined once in `ext-runtime::events`, and `redact` strips each payload down to what the subscriber may read.

Every adapter entry runs inside `ext_runtime::contain` [CMP-COM-contain]. It catches a panic, rolls the transaction back to its mark, returns `-7 Internal` and records the command in the activity ring. The app keeps running. Safe mode (`--safe-mode`, Shift at launch, `CRAFT_SAFE_MODE=1`, or two crashes within 60 s that name a plugin) starts with no plugin code. **Stop all plugins** is always reachable and returns within the cancel poll plus one owner turn.

## Worker, engines and data plane

### Processes

No plugin machine code runs in an app process on desktop or web [REQ-SEC-inv-01]. Plugin code runs in two kinds of process. `craft-plugin-worker` hosts the engines, one sandboxed worker per package version. Helpers run what cannot run in a worker: native code (`native.restricted` or `native.full`), codecs, inference, web pages, audio plug-ins and incumbent binaries. Decode and inference helpers run per plugin so one crash spares the others [DEC-COM-helper-per-plugin].

`ext-broker` is the supervisor. It launches through `craft-sandbox-launch`, which applies the per-OS profile, runs a self-test and fails closed. It keeps a zygote pool so the first event gets a warm worker, budgets workers by resident memory rather than count, leases render permits and keeps the crash journal and kill list. A worker suspends after 60 s idle unless audio is armed, a preview or surface is visible, a transaction is open or a job runs.

iOS has no worker processes. Each plugin runs interpreted on its own thread inside the app, under the same gate [CMP-COM-ext-engine-inproc]. Wasm runs on `wasmi` with fuel. JavaScript runs as a Craft-built `quickjs.wasm` inside `wasmi`, so the only in-process interpreter is memory-safe Rust. Memory caps, guarded stacks, `contain` at every host entry and Stop all keep the app alive.

### Engines

`ext-engine` hosts every language from day one [DEC-COM-languages-all-day-one]. Wasmtime runs two engines, one for sessions and one for media, with an ahead-of-time cache. `wasmi` is the CI reference and the iOS engine. JavaScript runs on V8 on every desktop OS and on QuickJS on web and iOS. JIT runs only in the worker, never in the app [DEC-COM-no-jit-in-app-process]. Ahead-of-time compilation happens inside a worker during the Prepare step and the cache is HMAC-tagged.

The guest imports exactly seven `craft.*` functions. There is no WIT or component model. One schema source, `ParamSchema`, generates every binding, and Wasm and JavaScript cross the same descriptors [DEC-COM-no-wit-component-model]. All scripts, including user and agent scripts, run in a worker: `extendscript@1` on the JIT engine, and expressions in `ext-expr`, which evaluates per-frame batches with look-ahead.

### Data plane

`ext-ipc` defines descriptors, the CBOR codec, fuzzed decoders and the wake policy. `ext-shm` provides segments, arenas, rings and read-only lending. ABI 2 moves batches, columnar results and shared media views over power-of-two single-producer rings [CON-COM-abi2-transport@2]. A batch step costs at most 5 µs. A native helper call costs at most 50 µs. Desktop media is never copied.

The host never acts on bytes a worker can still write. Payloads, views, lists and lengths are copied out once before validation [CON-COM-abi2-copy-out@2]. Producer outputs map read-only into consumers. Pixel, audio and filter calls use `repr(C)` views over arena offsets. Device data reaches plugins through rings the broker fills in place.

Codecs run host-side. Plugins never ship file codecs. `image.decode`, `media.decode` and `pdf.page.render` run over `craft-media` in a confined helper and return arena views or GPU textures [DEC-COM-no-plugin-codecs]. Inference runs where it is cheapest and safest: WGSL on the app device, CPU `tract` in the worker over read-only model segments, and CoreML, ONNX Runtime or `candle` in `craft-nn-helper` [DEC-COM-ai-backends].

### GPU

There is one `wgpu` host [DEC-COM-one-wgpu-host]. `craft-gpu` owns the `DeviceBroker` with app, job and probation devices, the memory accounts, the scheduler and loss recovery. `ext-gpu` validates plugin GPU work and runs it at four tiers, GT1 to GT4, all built now. The tiers are named apart from risk levels L0 to L4. GT1, GT2 and GT4 are free. GT3 is a recorded `wgpu` command list checked by the GT2 validator, and it needs `gpu.direct` [CON-COM-gpu-tiers@1].

New GPU work starts on probation. A digest is promoted when its bounded cost fits the `GpuSubmit` budget, or after 120 clean frames. Unpromoted work runs on the probation device, never on the app queue, so a looping shader cannot block the next frame [DEC-COM-probation-device-not-app-queue]. Document render and UI present are separate submits. The viewer shows the last completed render. Over-quota work fails only the plugin's call. App queues never wait on a helper-signalled fence; the host waits CPU-side within budget, then shows the last good frame.

Audio runs on a real-time class. Audio slots run on a real-time thread in the device workgroup. Processing stays within half of a 128-frame block, and three overruns in 10 s bypass the plugin.

## Permissions and trust

### Vocabulary and risk

`Permission` is one closed enum in `ext-core` [CMP-SEC-ext-core-permission]. Every variant has a host sentence, a risk level, a context mask and an asked rule, written as exhaustive match arms. `risk()` is the only source of levels, from L0 (free) to L4 (full access). Compute and kernels are never prompted. A package's risk is the maximum of its permissions.

Everything is grantable except the never list [REQ-SEC-everything-grantable]. N1–N7 refuse consent on the user's behalf, acting as another principal, changing host security state, the safety rails, in-process code, network and blocking save or quit. The list is minimal on purpose. Everything else is exposed by name with a permission, a risk level and a disclaimer.

### Enforcement

Permissions are enforced where the effect happens [REQ-SEC-enforce-at-effect]. The gate computes an `EffectMask` per call. Every sink checks it. Manifests and locks are inputs to that check, never the check itself. `clippy-sinks.toml` bans direct `std::fs`, `std::process::Command`, clipboard and preference calls in every app, so the sinks are the only path out. `Authorization` travels by value with every queued item; thread-locals, task-locals and statics that hold an origin are banned and a conformance walk fails on them [DEC-COM-origin-by-value].

The worker holds no OS authority. Host brokers in `ext-runtime/src/broker/` own files, processes, devices, capture, clipboard and OS hand-offs, and pass descriptors to the worker. Files resolve against the `FileAuthority` from picked handles, persisted folder handles or root grants. Package assets open through `pkg.open` with no file grant.

### Consent and disclaimers

Only hardware input approves. The setup sheet is built only by `ext-ui-fw::setup`. It lists every permission, hook, automatic run, agent exposure candidate, process, device and companion. The Enable button stays inert for 1.5 s and has no automation id. L4 needs a second sheet with a hardware-ticked checkbox. Enable waits until nothing covers the sheet [QAS-SEC-consent-not-automatable].

Every non-Official package shows host-owned third-party notices at setup, on widening, in the manager and in the window chrome [REQ-COM-disclaimer-all-non-official]. Notice text lives only in `notice.ftl`. L3 notices list the reach. L4 adds the full-access notice. There are no just-in-time prompts: a missing permission returns `-2` and the plugin may open a request sheet.

### Switches

Each switchable capability is built and tested in both states and selected by one field of `BUILD_DEFAULTS` [REQ-SEC-decisions-flip-data]. Organisation policy can only turn a switch off. An owner veto is a one-line data change. Every switch is on except two: web panels start at `CefWindow` (D-D) and the Windows network filter (`windows_wfp`) applies when installed [DEC-COM-bd-defaults]. Neither switch decides whether a plugin has network. That guarantee holds with every switch in either state.

### Supply chain and lifecycle

`ext-store` installs, verifies and grants. The catalogue index uses TUF with an offline root, hourly revocation snapshots, Sigstore provenance, pinned hashes and signers, and HMAC-tagged grants [REQ-SEC-tuf-index]. Updates wait out a cool-off of 72 h for Community, 24 h for Verified and none for Official. An update that widens the capability surface is held until the user approves it.

One state machine in `ext-store::state` drives every app, the launcher and every CLI [CON-COM-lifecycle-states]. A package moves from Available to Downloaded, Installed, Prepared, Enabled and Loaded. Strikes or crashes move it to Quarantined, and only the user re-enables it. Revocation moves that version to Revoked, which is terminal: only an update or a rollback to an unrevoked release leaves it, and the user cannot re-enable it. Revocation traps instances within 260 ms or kills them.

### Sandboxes

Each OS has its own profile, applied by `craft-sandbox-launch`. Windows uses LPAC, which holds no network capability, and a WFP filter on the plugin helper group. Without the filter, confined profiles keep no network path, and the host refuses to launch unconfined helpers [DEC-COM-d-u]. macOS uses signed helpers with sandbox profiles. Linux uses Landlock. Native hosts always run `restricted+ui+gpu`. `native.full` and `process.run:full` get full file and process reach behind the two-step L4 consent, and still have no network.

### Network

Plugins are offline by construction [REQ-SEC-offline-by-construction]. No plugin process or page has a network path. Full helpers deny direct and indirect network. T6 pages make zero DNS queries. Live document sync is a host-owned channel that plugin document writes reach only under `collab.write` at L2; widget state sync uses the host's widget path and needs no `collab.write`. Companion apps talk over local IPC, not loopback IP. All of this follows DEC-COM-no-plugin-network.

### Accounts and agents

Each plugin has one account for CPU, memory, GPU, disk, undo and prompts. The host writes the per-plugin activity log, and the manager shows it. Agents are separate principals: they cannot install, enable, grant or approve, and they treat plugin text as data.

## UI extensibility

Plugins describe UI and the host draws it. The host draws all chrome and trust UI and never waits on a plugin to paint [REQ-COM-principle-host-draws-chrome]. Every app uses the same egui fork, which adds `App::on_wake`, `Painter::recreate_device` and surface configuration and nothing else [DEC-COM-egui-fork].

Plugin UI is laid out once in `ext-ui-model` and painted once by `ext-ui-paint` [REQ-COM-ui-one-painter]. `ext-ui-fw` integrates it with each app's shell: docking, OS and palette windows, the manager, the setup sheets, theme tokens and the input pump. The runtime thread decodes, validates, diffs, lays out and tessellates `ui.set` and `ui.patch`. The UI thread only applies the result, within 0.12 vsync per frame for all plugin surfaces, and never half-applies a tree.

Six UI tiers are built now, all on the same layout engine [DEC-COM-ui-tiers-t1-t6]:

- **T1** is a retained declarative tree with keyed patches, bindings, styles and media nodes.
- **T2** is a retained layered canvas with hit regions, pen input, gestures and host animation.
- **T3** is a set of host editors (curve, gradient, colour wheel, node graph, timeline lanes, key graph, path, meter, scope) fed by plugin models. Their state machines live in `ui-editors-core`.
- **T4** is a viewer for image, texture or surface handles with host zoom and pan.
- **T5a** composites a helper's shared texture as a remote surface. **T5b** is a helper-owned OS window that the broker parents, clamps and marks with an identity strip.
- **T6** is a web window on CEF in `craft-plugin-web`, one confined process per app with one request context per plugin.

Contributions attach UI to the app: panels, windows, dialogs, menus, palette entries, tools, overlays, inspector sections, status items, settings pages, row decorations, drop targets and start cards. Panels dock like built-in panels. The host routes input in a fixed order: the Stop-all chord, trust surfaces, input filters on canvas streams, the focused plugin surface, tools and shell, then non-swallowing app-wide copies [CON-COM-input-pump]. A plugin can never activate a host control with synthetic input.

CEF is the only web engine, so T6 behaves the same on every OS [DEC-COM-one-engine-cef]. All three web modes are built and tested. The built state is `CefWindow`, where T6 panels float as palettes that follow a reserved dock slot. `craft-ext migrate --to-t1` ports UXP and CEP panels to docking T1 panels. Whether web panels dock directly is open decision D-D.

## Per-app adapters

Adapters map; they never decide [REQ-COM-adapter-mapping-only]. Each `crates/plugins` implements `CraftApp` and `AppExtensions` and nothing else. A structural lint fails on any adapter function that matches a runtime job: gate, grants, transactions, quarantine, jobs, UI limits, budgets, ladders, badges or durable-output fallback. Line budgets keep adapters small. EffectCraft's target is at most 1,600 lines, down from 3,802.

### Command metadata first

Every app command declares `Meta` next to its definition [DEC-COM-v1-2]. `Meta` states effects, scope, undo, run mode, cost, runner and parameter and result schemas. `meta::*` constructors set all of these together, so app authors make no fine-grained choices. One function, `exposure(meta, principal)`, decides what plugins, agents and automation can reach. A command without metadata is agent-only, and the three beta apps reach zero undeclared commands before their adapters ship [DEC-COM-undeclared-commands-zero]. This is why WS1 and the metadata stream come first: the gate cannot derive permissions from metadata that does not exist.

### FigCraft

FigCraft integrates first, at M2 [DEC-FC-first-adapter]. Its 630 commands get full `Meta` with real schemas and `PerNode` costs. `DocumentHost` uses `imbl` copy-on-write maps, so `snapshot` is an `Arc` clone and `touched` is the dirty set since the mark. The 7 s blocking bridge is deleted. Collab protocol 4 makes the server own plugin data per key, and live writes reach peers only under `collab.write`. The plugin shell runs on `ext-ui-paint`. `figma-compat` runs Figma plugins over `@craft/figcraft`. The web build runs plugins in a Web Worker on QuickJS behind the same gate.

### EffectCraft

EffectCraft integrates at M3. `crates/engine/src/ext.rs` shrinks to a `PluginHost` seam that `Runtime<EffectCraft>` implements. 757 commands across 86 modules declare `Meta`. Effects, transitions, generators, retimers and audio effects plug into `EffectHost` v2 through the filter service. Render cache keys hash plugin identity. Expressions move to `ext-expr` in the worker, and `expr-boa` stays only as an in-process fallback for untainted batches with no library bindings. ExtendScript runs as `extendscript@1` on V8 in a worker over the shared `gatebackend`. `craft-aehost` hosts installed After Effects plug-ins, and `craft-ofx-bridge` hosts OFX.

### PhotoCraft

PhotoCraft integrates at M3. Its adapter replaces `plugin_ops`, `plugin_sessions`, `plugin_filters` and the in-process host; transactions move to `ext-runtime::txn` [DEC-PC-common-documenthost-txn]. A single pixel request shape covers every target, space and depth, with tiles paged into instance arenas. Brush engines, input filters, smart filters and adjustment layers are contribution kinds. `photocraft-psdesc` is the one descriptor table for Actions, 8bf `ActionControl`, `batchPlay` and `executeAction`. `craft-pshost` hosts 8bf, 8BAM and TWAIN plug-ins. `photoshop-compat` and `photoshop.jsx@1` run UXP and ExtendScript plugins.

### Shared compatibility

The compatibility layers are shared, not per app. One `gatebackend` serves both ExtendScript dialects. One ScriptUI converter turns ScriptUI dialogs into T1 panels. One `node-compat` module serves Node APIs to every shim, with network modules returning `-9`. One clean-room `.jsxbin` decoder and one E4X lowering serve both apps.

### The other Craft apps

The eight other apps get adapters at M7–M9 on the same traits [REQ-COM-ws15-incumbent-kinds]. Each hosts its incumbents' main plugin kinds. FilmCraft hosts OFX. SoundCraft hosts VST3, AU and CLAP and `audio.effect@1.0`. VectorCraft and DesignCraft host Illustrator and InDesign plug-ins. LightCraft hosts `.lrplugin` packages. TextureCraft and MaterialCraft evaluate `.sbsar`. AniCraft hosts `.jsx` rigs and `anim.ease@1.0`.

### Hard cut

Old plugin systems are deleted only when their replacement passes the cut gate on all three operating systems [DEC-COM-hard-cut-gated-by-parity]. Kept behaviours are tested before deletion. `ext-host` is deleted, and no plugin code runs in process after the cut.

## Quality attributes

The owner's rules translate into five attributes. Each has scenarios with a stimulus, a measurable response and named tests.

- **Performance.** The UI frame stays within one vsync at p99, with zero frames over twice the vsync [QAS-COM-ui-frame-guarantee]. Every kind of plugin work has one `CallClass` row in `BUDGETS`, and each degrade ladder ends in a visible badge, never silent pass-through.
- **Availability.** A plugin crash, hang, flood, trap, out-of-memory error or GPU hang never crashes or stalls the app. The document hash equals the last commit after each failure.
- **Security.** No plugin network path exists, consent cannot be automated, and revocation completes within its bound.
- **Modifiability.** A switch flip changes one data field and no code.
- **Usability.** Consent needs a human. The user can always see, stop and recover.

Measurement has two tiers [DEC-COM-acceptance-measurement]. Pull requests gate on deterministic counters: more than 2% extra instructions, or any rise in a logical counter, fails the check. Wall-clock targets run nightly on three hardware classes (hw-mac, hw-win, hw-linux). M6 closes on a hardware wall-clock run. Whether those runners are approved is open decision D-T, and the M6 beta exit is blocked until the owner answers it. `ext-conformance` runs the gates C0–C24, including the structural lints, chaos suites and UI goldens.

| Scenario | Attribute | Stimulus | Response measure | Tests |
|---|---|---|---|---|
| Batch step overhead | performance | N-step app.batch | <=5 us per step | TST-COM-c17b-abi-call-overhead |
| Control call latency | performance | Control call | IPC latency row | TST-COM-c17b-abi-call-overhead |
| Crash-safe save | availability | kill -9 in save | hash-identical | TST-COM-crash-during-save-seeded |
| Decoder crash | availability | Fuzzed file | 0 app crashes | TST-COM-cs1-codec-services |
| heavy-command-no-owner-stall | performance | Plugin calls an exposed command at schema extremes on 100 MP or 100k-node documents | No owner task exceeds OwnerFrame | TST-COM-exposed-commands-at-schema-extremes |
| Native call cost | performance | 4K native.call | <=50 us | TST-COM-nc1-native-zero-copy |
| nn helper crash | availability | Helper killed | 0 drops | TST-COM-nn-helper-killed-mid-frame |
| plugin-flood-survival | availability | Plugin loops layer.duplicate and image.size on large fixtures | RSS below headroom, doc hash equals last commit | TST-COM-plugin-layer-flood-cannot-oom |
| Post-install app work stays off owner | performance | Plugin commit triggers cache invalidation and rebuild | frames_over_2x_vsync = 0; install owner task under OwnerFrame | TST-COM-post-install-invalidation-async |
| T6 pages make no network traffic | security | Plugin page attempts requests, navigation or DNS | 0 DNS queries, 0 packets | TST-COM-o6-per-app, TST-COM-o6-web |
| UI frame guarantee | performance | UI frame p99 target in a test or budget | UI p99 <= 1 vsync (16.6 ms at 60 Hz, 8.3 ms at 120 Hz); frames_over_2x_vsync = 0 | TST-COM-ui-64-panels-30-videos, TST-COM-gate-c21, TST-COM-frame-literal-lint |
| Viewer tap cost | performance | 1080p60 scope | <=0.3 ms GPU | TST-COM-vt1-viewer-tap-scope |
| Plugin expression rig performance | performance | Playback of a 200-layer Duik-generated rig at 1080p30 on a reference machine class | <=1x AE expression time; 0 dropped frames at 1080p30 on same machine class | TST-EC-ec-t51, TST-EC-expr-export-mfr |
| Helper stall never blocks UI | availability | A native GPU helper stalls or dies mid-frame during 1080p30 playback | UI frame time <=1 vsync p99 | TST-EC-ec-t41b |
| Viewer output stays in sync | performance | 60 s of RAM-cached 1080p59.94/1080p60 playback to an output device | latency <=1 frame per frame; UI p99 <=1 vsync | TST-EC-vo1 |
| Containment: Malformed widget or codegen output | availability | Malformed widget or codegen output | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-widget-bad-tree-renders-fallback |
| Containment: A plugin floods non-pure calls | availability | A plugin floods non-pure calls | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-ten-chatty-plugins-keep-frame, TST-FC-fc-owner-flood-charges-only-flooder |
| Containment: Engine command panics while serving a plugin | availability | Engine command panics while serving a plugin | 0 app exits over 1,000 seeded panics; document hash equals the mark after each; 100% of calls return -7 | TST-FC-fc-panicking-command-poisons-txn |
| Containment: Display pass too slow or invalid | availability | Display pass too slow or invalid | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-display-pass-over-budget-switched-off |
| Containment: Plugin never finishes an event | availability | Plugin never finishes an event | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-stuck-event-releases-gesture-lock |
| Containment: First GPU use stalls the UI | availability | First GPU use stalls the UI | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-first-gpu-filter-no-ui-hitch |
| Containment: GPU filter hang or device loss | availability | GPU filter hang or device loss | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-image-fill-filter-device-loss-shows-durable |
| Containment: Importer emits hostile structure | availability | Importer emits hostile structure | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fuzz-format-import-tree |
| Containment: Hostile or broken image or video | availability | Hostile or broken image or video | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-video-decoder-error-shows-poster, TST-FC-fc-media-decode-out-of-process |
| Containment: Huge plugin data or state | availability | Huge plugin data or state | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-plugin-data-quota-refuses |
| Containment: Plugin writes leak into a live session | availability | Plugin writes leak into a live session | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-live-write-needs-collab-write-server-fixture |
| Containment: Malformed vector write | availability | Malformed vector write | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-vector-network-write-rejects-non-finite |
| Containment: Peer floods plugin data | availability | Peer floods plugin data | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-server-refuses-foreign-plugin-namespace, TST-FC-server-plugin-data-rate-limit |
| Containment: A plugin reads or writes 100k nodes | availability | A plugin reads or writes 100k nodes | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-nodes-get-100k-off-owner, TST-FC-fc-nodes-set-100k-ui-p99, TST-COM-heavy-commands-run-off-owner |
| Containment: Safe mode | availability | Safe mode | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-safe-mode-starts-no-worker |
| Containment: Plugin Wasm traps, loops or hits OOM | availability | Plugin Wasm traps, loops or hits OOM | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-infinite-loop-returns-within-deadline, TST-FC-fc-oom-plugin-leaves-app-alive |
| Containment: Document with unknown plugin data or nodes | availability | Document with unknown plugin data or nodes | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-legacy-plugin-data-round-trips-byte-identical |
| Containment: Worker process dies | availability | Worker process dies | Named tests pass; the app stays alive and the failure is contained to the plugin. | TST-FC-fc-worker-kill-rolls-back-and-panel-recovers |
| GPU hang | availability | Plugin shader hangs | 0 crashes | TST-PC-nocrash-plugin-trap |
| Handoff hang | availability | Vendor app hangs | <=1 s | TST-PC-nocrash-plugin-trap |
| Memory pressure | availability | Memory pressure | 0 crashes; ladder order | TST-PC-nocrash-plugin-trap, TST-COM-b2-suite |
| Page hang | availability | Page hangs in sync call | 0 app stalls | TST-PC-nocrash-plugin-trap |
| 100 MP apron-64 filter | performance | 100 MP 16-bit apron-64 filter (Dfine, Portraiture class) | <=2.5 s on D-T hardware; <=1.0x Dfine and Portraiture on the same machine | TST-PC-bench-100mp-apron |
| 100 MP point filter, resident | performance | 100 MP 16-bit point filter, resident | <=1.0 s | TST-PC-bench-100mp-point-filter |
| 100 MP point filter, spilled | performance | 100 MP 16-bit point filter, tiles spilled to SSD | <=3 s | TST-PC-bench-100mp-point-filter |
| App start with 100 plugins | performance | App start with 100 plugins installed | <=5 ms added | TST-PC-bench-app-start |
| Heavy app.batch on 100 MP | performance | app.batch of heavy commands on 100 MP 16-bit | UI frame p99 <=1 vsync; total <=1.0x built-in time + bridge-call IPC per call | TST-PC-bench-batch-heavy |
| Bridge command call | performance | Plugin command via bridge | IPC per CON-COM-ipc-latency power row (Ac p50 <=5 us) + command cost | TST-PC-bench-bridge-call |
| Brush engine latency | performance | 240 Hz pen stroke | input-to-ink <=1 frame + 4 ms p99 at 240 Hz input | TST-PC-bench-brush-latency |
| Generative fill while painting | performance | Generative fill 1024x1024 while painting and panning | canvas frame p99 <=1 vsync | TST-PC-bench-generative-fill |
| GPU brush engine latency | performance | emits = gpu engine, 2,000 px brush, 100 MP | input-to-ink <=1 frame + 4 ms p99 at 240 Hz within the Slot::Paint budget | TST-PC-bench-gpu-brush-latency |
| Incremental save | performance | Save of 100 MP 16-bit with two smart filters after one edit | writes only the changed durable tiles | TST-PC-bench-incremental-save |
| Input filter batch | performance | Input filter | <=2 ms p99 per batch | TST-PC-bench-input-filter |
| Plugin memory accounting | performance | Plugin memory use | common 9.8 accounts and caps; durable smart-filter output is spillable app cache | TST-PC-bench-memory |
| Tiled nn.run denoise | performance | Tiled nn.run denoise on 24 and 100 MP 16-bit | CoreML 24 MP <=10 s, 100 MP <=40 s; <=1.0x Topaz Photo AI on the same machine; UI frame p99 <=1 vsync | TST-PC-bench-nn-denoise |
| Overlay drag | performance | Overlay drag | 0 guest calls per frame | TST-PC-bench-overlay-drag |
| Panel update | performance | Panel update | ui.patch of 100 nodes <=0.3 ms host time | TST-PC-bench-panel-update |
| Live preview slider drag | performance | Slider drag at 100 MP fit-to-screen | <=16 ms p95 and first pixels <=50 ms | TST-PC-bench-preview-drag |
| Hosted 8bf call overhead | performance | 8bf re-render, Last Filter or Action step on 100 MP, cold and warm | Cold <=150 ms to first band; warm <=40 us per band; <=1.0x the same 8bf in Photoshop | TST-PC-bench-pshost-call |
| Slow smart-filter re-render | performance | Smart-filter re-render taking 20 s | proxy or stale output within one frame; UI frame p99 <=1 vsync during the job; one durable swap | TST-PC-bench-slow-smart |
| Smart filter preview bytes | performance | Smart filter preview re-render | 0 bytes written or allocated for durable output | TST-PC-smart-durable-no-copy |
| Smart filter on unrelated edit | performance | Smart filter re-render on unrelated edit | 0 ms | TST-PC-bench-smart-unrelated-edit |
| 24 MP 6x upscale | performance | 24 MP 16-bit 6x upscale at defaults | Default backend <=1.0x Topaz Gigapixel: hw-mac CoreML <=75 s, Windows ORT <=90 s; candle fallback <=180 s; RSS under memory_headroom() | TST-PC-bench-upscale-6x |
| UXP getters and sync calls | performance | UXP panel getters and sync calls | mirrored getter <=5 us with zero IPC; sync bridge p50 <=0.3 ms, p99 <=2 ms | TST-PC-bench-uxp-getters |
| Wet-canvas stroke on 100 MP | performance | Wet-canvas stroke (layer-sized state) on 100 MP | Readback = dirty pages; commit of a 2,000 px stroke <=60 ms off the UI thread | TST-PC-bench-wet-canvas |
| Consent needs a human | usability | A plugin or script tries to approve the setup sheet | Enable inert 1.5 s; 0 automation ids; L4 needs a hardware-ticked checkbox | TST-SEC-u3 |
| No plugin network path | security | A plugin or helper tries to reach the network directly or through an indirect channel | 0 reachable denied channels in CI (O4b); 0 remote lookups (O9); allowed host-owned channels: store, catalogue, collab.write sync via sink::collab and the licen… | TST-SEC-o4b, TST-SEC-o9, TST-SEC-d-daemon |
| Plugin cannot stall the app | availability | A plugin hangs, crashes or a hook times out | 0 UI-thread waits on a plugin; a hook past the BeforeHook BUDGETS row counts as no findings and save completes in 100% of 200 hung-hook runs | TST-SEC-r1 |
| Revocation latency | security | Revoke while running | Revocation row | TST-SEC-r1 |
| Switch flip is a data change | modifiability | Owner or organisation flips a BUILD_DEFAULTS switch before or after install | 1 field changed; 0 manifest parse failures | TST-SEC-bd1 |
| Sync probe off UI | performance | Roots probed | 0 UI calls | TST-SEC-s-sync |


### Capability coverage

Every incumbent API and feature row traces to a capability, an owner decision or the owner's network exclusion. The table counts rows by app and kind. `AE`, `PS` and `FG` are the After Effects, Photoshop and Figma incumbents. The network column comes from DEC-COM-no-plugin-network. The gaps column is zero.

| App:row kind | Covered | Owner decision | Excluded by owner (network) | Gaps |
|---|---|---|---|---|
| -:REQ | 296 |  |  |  |
| AE:API | 302 | 4 | 6 |  |
| AE:INC | 732 | 7 | 11 |  |
| COM:FLG | 8 |  |  |  |
| EC:FLG | 22 | 1 |  |  |
| FC:FLG | 16 |  |  |  |
| FG:API | 228 |  | 7 |  |
| FG:INC | 294 |  | 21 |  |
| PC:FLG | 10 |  |  |  |
| PS:API | 345 |  | 5 |  |
| PS:INC | 683 |  | 1 |  |


## Decisions

Three owner decisions are open. D-D and D-K each have a built state, so nothing waits on them. D-T does not: the M6 beta exit is blocked until it is answered.

- **D-D, docking web panels** [DEC-COM-d-d]. Built state: `CefWindow`, where T6 panels float as slot-following palettes. Recommendation: `CefDocked`, so incumbent panels dock as they do today, at one frame of latency at 60 fps. The narrower option, `CefDockedCompat`, docks only imported UXP and CEP panels.
- **D-K, native hosts in public releases** [DEC-COM-d-k]. `craft-pshost`, `craft-aehost` and the OFX bridge are clean-room and ship in beta. Recommendation: beta now, public releases after a review of Adobe SDK header licensing.
- **D-T, CI hardware runners** [DEC-COM-d-t]. Recommendation: approve three self-hosted runners, because hosted runners lack real GPUs, audio and displays. Without the runners, hosted runners with device mocks close M4 and M5, but M6 and the beta ship wait for hardware.

The decided records that shaped this architecture fall into six groups.

- **One place for behaviour.** One gate over canonical ops and the bridge [DEC-COM-v1-1]. One runtime in `ext-runtime` with thin adapters. One state machine, one vocabulary, one risk function and one notice source.
- **Speed.** Snapshot reads with one owner hop per commit. Execution chosen by `Meta` and size. Separate render and present submits. A probation device for unpromoted GPU work. Zygotes, ordered cold start, and workers budgeted by memory.
- **Never crash.** No in-process plugin code on desktop and web, an in-process memory-safe interpreter on iOS under the same gate, and no JIT in the app. Per-plugin helpers. `contain` at every adapter entry. Hooks fail open and no hook can block save or quit.
- **Full capability.** All languages, all GPU tiers and all UI tiers built now. Native full access from any tier behind two-step consent [DEC-COM-d-g]. Native hosts for incumbent binaries. A minimal never list.
- **Trust.** Enforcement at the sink. `Authorization` carried by value. Hardware-only consent. TUF with hourly revocation. Third-party notices on every non-Official package.
- **No plugin network.** The owner decision DEC-COM-no-plugin-network removes network, secrets, OAuth and loopback from the vocabulary. The host keeps the store, catalogue, live sync and the licensing relay.

The full decision list, with choices and what each affects, is in the decisions document.

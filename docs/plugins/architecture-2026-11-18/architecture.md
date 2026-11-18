# Craft plugin architecture

Date: 2026-11-18. Status: proposed. It needs the owner decisions in [decisions.md](decisions.md); no
code has changed yet. The build order is in [roadmap.md](roadmap.md).

This document answers one question for every ArtCraft creative app: what system lets a plugin do
almost anything the app can do, while staying permissioned, sandboxed, fast and simple to write? It
merges eight research reports, a reconciliation pass and a security red team. All of them sit in
[research/](research/). The full command census is in [census/](census/), and the benchmark code is in
[bench/](bench/).

Apps covered, read at today's upstream `main` or the `plugin-beta` branch:

- EffectCraft, FilmCraft, AniCraft;
- PhotoCraft, LightCraft, FigCraft, VectorCraft, DesignCraft;
- MaterialCraft, TextureCraft, SoundCraft;
- the shared libraries in craft-libs.

Evidence levels:

| Mark | Meaning |
|---|---|
| *measured* | measured on an Apple M5 |
| *source* | read in the code |
| *cited* | from public documentation |
| *estimate* | anything else |

## 1. The short answer

**Use both: a small shared vocabulary plus every app command, behind one gate.** Plugins should
neither keep calling hand-written SDK ops alone nor call native commands directly.

1. **Canonical operations.** About 84 ops mean the same thing in every app: documents, selection,
   nodes and layers, properties, pixels, time, export, UI, events, jobs, files, network and settings.
   They ship in tranches, about 30 first. A plugin written against them runs in every app that offers
   them.
2. **The registry bridge.** `app.run`, `app.batch` and `app.commands` reach every app command that has
   declared what it does. That is MCP and CLI breadth, without hand-written plugin ops. The census
   counts 6,193 engine commands across the 11 apps (*source*).
3. **One gate.** Both surfaces pass through a single function in the shared runtime. It checks the
   schema, works out the permission from the command's declared effects, checks scope, charges the
   budget and wraps the call in the plugin's transaction. Adapters never decide permissions.

**Why not plain native commands?** Today no command says what it touches. None of the 6,193 declares
its side effects, about 9 have machine-readable parameters, and "query" means four different things
across the apps (*source*). Some commands named as reads write files: DesignCraft's
`file.exportIdml`, and LightCraft writes an XMP sidecar after every command. Exposing commands raw
would hand out file, network, preference and process access by accident.

**Why not only hand-written SDK ops?** The three beta adapters reach 3.7%, 11% and 11% of their
registries after months of work (*source*), and their copies of shared logic disagree with one another.

**The first thing to build is command metadata, not the calling interface.** Every command declares
its effects, scope, undo behaviour and parameter schema in one shared crate, and the build fails
without them. Permissions, the allowlist, typed SDK bindings, MCP schemas and docs are then generated
from that metadata.

## 2. Principles

1. **The app keeps authority; the plugin holds capabilities.** A plugin never holds a raw path,
   socket, process, preference handle or OS window. The host hands out handles (picked files, image
   regions, jobs, requests) and checks them where the side effect happens.
2. **Default deny, generated, checked in CI.** A command a plugin can call is the output of one
   function over declared metadata. Its permission is pinned in a reviewed lock file, and a widening
   needs a security owner.
3. **Compiled plugin code never runs inside the app.** JIT-compiled Wasm runs in a sandboxed worker
   process, one per plugin package version. A Cranelift bug or a guest abort then cannot take the
   user's unsaved document with it.
4. **Zero copies for media, batches for structure.** Pixels, audio and meshes move through shared
   memory, GPU handles and rings. Structured calls are batch-shaped, because each one crosses a
   process.
5. **The host draws all chrome and trust UI.** A plugin describes its UI as data. The host renders it
   in the app's framework and theme, and never calls the plugin once per frame.
6. **One behaviour, one place.** Behaviour lives in `ext-runtime`; an adapter only maps. A shared op
   has one schema in every app, and the conformance suite fails when an app drifts.
7. **Documents outlive plugins.** Every plugin result the render path depends on is stored with a
   durable output. Files open, render and export without the plugin, and say so.
8. **Fail safe, never silent.** A slow, crashing or missing plugin degrades to proxy, last good result,
   dry signal or a badge. It never silently passes through, and never silences the output.

## 3. Layers and crates

```text
plugin (Rust today; TypeScript/AssemblyScript, C, Zig next) ── craft-guest SDK + generated apps::<app> modules
        │  ABI v1 craft.call (JSON, control) · craft.call_buf (binary, bulk) · shared-memory views · GPU handles · rings
        ▼
craft-plugin-worker (one per package version, OS-confined; Wasmtime + Cranelift, AOT cache)
        │  IPC: call descriptors over a socket pair, media in shared arenas
        ▼
ext-runtime::Runtime<A: CraftApp>  (one copy, in craft-extensions)
   gate ─ schema → exposure → permission (declared ∩ granted ∩ invocation) → scope → budget
   txn ── one event = one transaction = one undo step; nested registry calls; coalesced change events
   events · jobs · services · net · files · settings · secrets · UI model · filter service · GPU tiers · audio transport
        ▼
adapter (per app, mapping only): canonical map, DocumentHost, change source, pixel/GPU views, HostUi
        ▼
app: CommandSpec + Meta (craft-command) · effect choke points · open registries · tolerant formats · change events · shell placements
```

| Layer | Owns | Crates (new ones in bold) | Repo |
|---|---|---|---|
| Shared libraries | Command metadata and schemas, GPU device broker, AI, real-time audio transport, DSP, graph runtime, mesh, tolerant serde, UI components | **`craft-command`** (grown from VectorCraft's `apischema`), **`craft-preserve`**, **`craft-gpu`**, **`craft-nn`**, **`craft-audio-rt`**, **`craft-audio-dsp`**, **`craft-graph`**, `craft-mesh`, framework editors (curve, gradient, graph, timeline lane, meter, viewer) | craft-libs |
| Platform | Manifest, permissions, contracts, runtime, gate, transactions, events, jobs, services, engines, worker, GPU tiers, UI model and renderers, store, manager, CLI, SDK | `ext-core`, **`ext-runtime`**, `ext-host`, **`ext-worker`**, **`ext-gpu`**, `ext-kernels`, **`ext-ui-model`**, `ext-ui`, `ext-ui-fw`, `ext-store`, `ext-manager`, `ext-cli`, `guest`, `native-sdk` | craft-extensions |
| Adapter | Mapping only | `crates/plugins` in each app | each app |
| App | Commands with metadata, choke points, open registries, tolerant formats, change events, shell placements | existing crates | each app |
| Catalogue | Signed index, packages, models stored as data | – | craft-plugins |
| Launcher | Cross-app handoff, shared model store, store page | – | craft-launcher |

### 3.1 The one adapter trait

```rust
pub trait CraftApp: Send + Sync + 'static {
    type Doc: 'static;
    fn info(&self) -> AppInfo;
    fn registry(&self) -> &dyn craft_command::Registry;
    fn canonical(&self) -> &'static CanonicalMap;
    fn documents(&self) -> &dyn DocumentHost<Self::Doc>;
    fn changes(&self, doc: &Self::Doc, since: Revision) -> ChangeSet;
    fn plugin_data<'d>(&self, doc: &'d mut Self::Doc, at: &Target) -> Result<&'d mut PluginData, OpError>;
    fn pixels(&self, doc: &Self::Doc, req: &PixelRequest) -> Result<PixelSource, OpError>;
    fn host_ui(&self) -> &dyn HostUi;
    fn gpu(&self) -> Option<craft_gpu::DeviceRef>;
}

pub trait DocumentHost<D>: Send + Sync {
    fn active(&self) -> Option<DocToken>;
    fn on_owner(&self, doc: DocToken, f: OwnerTask<D>) -> Pending;
    fn snapshot(&self, doc: &mut D) -> Snapshot;
    fn restore(&self, doc: &mut D, s: Snapshot);
    fn commit(&self, doc: &mut D, label: &str);
    fn busy(&self, doc: &D) -> bool;
}
```

- `on_owner` returns `Pending` and never blocks the UI thread. The runtime awaits it against the
  transaction deadline.
- Everything else lives in `Runtime<A>`: gate, transactions, grants, workers, quarantine, events,
  jobs, UI limits, filter service, references, catalogue refresh and generated bindings.
- About 1,500 lines duplicated across today's three adapters disappear, with their behavioural drift.
  Adapter code shrinks by about 65% (*estimate*).

## 4. Command metadata

Every app already declares commands as a `CommandSpec` with an id, label and free-text parameters,
dispatched by id from menus, CLI, control port and MCP (*source*). Every `CommandSpec`, shell and UI
commands included, gains one required field:

```rust
pub struct Meta {
    pub effects: &'static [Effect],
    pub scope: Scope,
    pub undo: Undo,
    pub run: RunMode,
    pub destructive: bool,
    pub params: ParamSchema,
    pub result: ParamSchema,
    pub exposure: Exposure,
    pub hosts: &'static [&'static str],
    pub pref_group: Option<&'static str>,
    pub since: u32,
}

pub enum Effect {
    DocRead, DocWrite, SelectionWrite, ViewWrite, SessionWrite,
    FsRead, FsWrite, PickFile, Net, ClipboardRead, ClipboardWrite,
    Process, PrefsWrite, PluginMgmt, History, UiWindow, AppLifecycle,
    ModelInstall, Collab, Print, SystemFonts, Job, Dispatch,
}
pub enum Scope { None, Params(&'static str), Selection, Document, App }
pub enum Undo { Step, Coalesced, None, Manages }
pub enum RunMode { Instant, Long }
```

| Field | Rule |
|---|---|
| `effects` | The full list of things the command can touch, including indirect effects such as "apply preset" reading a file or "replace image" fetching a URL. It also includes work the app does after any command, such as LightCraft writing an XMP sidecar after every command (*source*). `Dispatch` marks commands that run other commands: `edit.repeat`, `command.batch`, `actions.play`, `macro.run` and journal replay |
| `params`, `result` | Typed schemas: `Id`, `Ids`, `Enum`, `Point`, `Color`, `Time`, `Object`, `Path`, `File`, `Url`, `Script`. A `Path`, `Url` or `Script` parameter makes the command unreachable from plugins unless a `File` or handle twin exists. Results containing paths are redacted to handles |
| Compile check | The `cmd!` macros refuse to compile without `Meta` |
| Placeholders | `Meta::UNDECLARED` exists only as an explicit, greppable placeholder. A command carrying it is **agent-only**: MCP and CLI keep it, plugins never see it |

### 4.1 Exposure

One platform function, `exposure(meta) -> Exposure`, decides what plugins may call. The first matching
rule wins.

| # | Rule | Exposure |
|---|---|---|
| 1 | Undeclared schema or effects | Agent-only |
| 2 | `PluginMgmt`, `Process`, `AppLifecycle`, `ModelInstall`, `Collab`, `Print`, `SystemFonts`, or `PrefsWrite` on a group that controls plugins, scripts, automation, security or grants | Refused |
| 3 | `Undo::Manages` (raw undo, redo, undo groups), or `Dispatch` (it would run another principal's commands) | Refused; plugins use `app.batch` |
| 4 | `Path`/`Url`/`Script` param without a handle twin | Refused |
| 5 | `UiWindow`, or a command that needs a modal dialog | Refused; plugins show their own form, then call with explicit params |
| 6 | The app's own `Net` without declared `hosts` | Refused. With declared hosts it is plugin-visible under `network:<host>` for each host (owner decision 9) |
| 7 | Command contributed by a plugin | Refused through the bridge; use `service.call` |
| 8 | Otherwise | Plugin-visible, with permissions derived from effects and scope |

Derived permissions:

| Effect | Permission |
|---|---|
| `DocRead` | `document.read:<scope>` |
| `DocWrite` | `document.edit:<scope>` |
| `SelectionWrite` | `selection.write` |
| `ViewWrite` | `app.view` |
| `SessionWrite` | `app.session` |
| `PrefsWrite` (allowed groups) | `app.preferences:<group>` (owner decision 8) |
| `Net` (declared hosts) | `network:<host>` for every declared host |
| `PickFile` | the file portal |
| `ClipboardRead`, `ClipboardWrite` | `clipboard.*` |
| `Long` or `Job` | callable only through `job.run_command` |

The output is `api.lock.json` in each app repo, checked in and reviewed.

### 4.2 Enforcement

Declarations alone are not trusted. There are three layers of enforcement:

1. **Choke points.** File, network, process, clipboard and preference access in each engine goes
   through one module per effect. FigCraft's `file_access` module is the model (*source*). Clippy's
   `disallowed-methods` bans direct calls, as craft-extensions already does for HTTP.
2. **Origin tagging.** The dispatcher sets the effect mask the current call is allowed, and tags the
   call `Origin::Plugin(id)`. The origin travels through every nesting: batches, replays, macros,
   Actions, service calls, jobs and writes the app makes after a command commits. Every choke point
   refuses an effect outside the mask. File, network, preference, history, plugin-management and agent
   sinks refuse a plugin origin whatever the lock says.
3. **CI checks**, all failing closed:

| Check | Proves |
|---|---|
| C0 compile | Every command has `Meta` |
| C1 lock drift | Exposure matches the reviewed `api.lock.json` |
| C2 widening | A wider exposure or a weaker permission needs a line in `api-widening.log`, plus a security CODEOWNER |
| C4 scope | For every exposed command, an out-of-scope id returns `-2` and leaves the document hash unchanged |
| C5 undo | Inside a plugin transaction, rollback restores the hash, and commit adds exactly one undo step and no journal entries |
| C6 refusal table | Property test of `exposure()` over all effect combinations |
| C7 compatibility | A schema change without an API-level bump fails; removing a command without two levels of deprecation fails |
| C8 conformance | Canonical ops behave identically in every app |
| C9 bindings fresh | Generated SDK bindings equal the lock |

An effects probe, which runs each command under a recording environment, comes later as hardening.

### 4.3 Workload

About 7,200 declarations: 6,193 engine commands plus about 1,000 shell and UI commands. The census
scripts pre-fill about 85% of the effects (*estimate*). Agents draft the rest and humans review, at
about 200 a day, which is 35–40 review-days spread across app owners (*estimate*).

The census lists, per command, the drafted class and flags as a burn-down: [census/all_apps.csv](census/all_apps.csv).

## 5. Call surfaces and ABI

### 5.1 Canonical ops

| Rule | Detail |
|---|---|
| Naming | `noun.verb` in lowerCamelCase: `selection.set`, `node.set`, `marker.add`, `doc.get`, `time.set` |
| Targets | One `targets: [TargetRef]` shape replaces the `layer`, `layers`, `ids`, `clips` and `items` variants |
| Shared types | One colour object, one rational time type, document units declared by `doc.get` |
| First tranche | About 30 ops, mostly ones the beta adapters already cover. The three beta adapters rename their divergent ops before any third-party plugin ships |
| Promotion | A concept becomes canonical when a second app implements it with the same schema and passes the conformance suite. No alias tables between apps; aliases hide semantic differences |

### 5.2 The registry bridge

- **`app.run {command, params}`.** `command` is `app.<appId>.<id>`.
- **`app.batch {steps, on_error}`.** `on_error` is `rollback` (the default), `stop` or `continue`.
  Every step is schema-checked before the first one runs.
- **`app.commands {prefix}`.** Lists what this plugin may call in this app, at this API level.

The ABI stays dynamic, a command id plus JSON. Generated SDK modules make it typed for authors:
`craft::apps::vectorcraft::object::align(&Align { ids, to })`.

The command's own work (schema check, snapshot, journal) costs microseconds to milliseconds, so the
roughly 1 µs JSON envelope does not matter (*measured*).

### 5.3 Transport

| Data | Path | Measured cost (Wasmtime, in-process baseline) |
|---|---|---|
| Control calls | `craft.call` JSON, kept for ABI v1 compatibility | 930 ns per call; 365 ns with generated serde structs |
| Bulk structured data | `craft.call_buf` with a compact binary encoding of the same schema; columnar `nodes.get` and `nodes.set` | 12–68 ns per item |
| Pixels, audio, meshes | Shared-memory arenas backing guest memory; `#[repr(C)]` view descriptors; host producers write straight in | 1080p RGBA32F guest loop 0.87 ms with no copy; 0.31 ms in 8 bands. Today: 52 ms |
| Streams | SPSC rings (meters, scopes, audio) | – |
| GPU data | Texture and buffer handles on the device that holds the data | No readback |

**IPC budget.** Each call crosses into the worker. The target is ≤5 µs p50 and ≤20 µs p99. That
target is *unmeasured* and is the Phase 2 gate. The core API is batch-shaped because of it.

**Interface language.** WIT and the component model are deferred. With every call crossing a process,
WIT's 24–81 ns in-process advantage (*measured*) no longer applies. One schema source,
`craft-command`'s `ParamSchema`, describes both registry commands and platform ops. It generates Rust
bindings first, then TypeScript/AssemblyScript, C and docs. WIT is reviewed again once the worker is
measured. This is an owner decision.

## 6. Execution and isolation

| Topic | Design |
|---|---|
| Engine | Wasmtime with Cranelift and `parallel-compilation`. Compiled ahead of time at install (8.9–11.5 ms per 120–160 KB module, *measured*) into a host-owned cache keyed by sha256 plus engine version and protected by an HMAC. Loading is 0.03–0.05 ms, pooled instantiation 6 µs, and 100 installed plugins cost 3.8 ms and 26 MB loaded eagerly (*measured*); loading is lazy. Epoch deadlines (1 ms; 0.25 ms when audio is armed). Pooling allocator (about 256 slots, memory zeroed between runs). SIMD128 on, relaxed SIMD off. NaN canonicalization after a benchmark |
| Interpreter | wasmi with tail-call dispatch and SIMD (11.3 ms vs 42.1 ms today for the 1080p loop, *measured*). It is the deterministic reference in CI and the fallback where a worker cannot be confined. Not on the desktop hot path |
| Not used | Winch, Pulley and an in-browser engine, until a web or iOS plugin target exists |
| Worker process | `craft-plugin-worker`, one per package version, started lazily and reaped when idle. The exceptions are while audio is armed or a live preview is open |
| macOS | An XPC service with App Sandbox entitlements and `allow-jit` |
| Linux | Landlock, seccomp and `no_new_privs` |
| Windows | An LPAC AppContainer with a Job object and mitigation policies, when Windows ships |
| Worker rights | No file, network, exec or persistence. All authority stays in the host broker |
| Hostile-tenant settings | Guard regions, explicit stack limit, Spectre mitigations (including `use_csdb` on Apple Silicon for Community and Local tiers), closed import set, no WASI file system or sockets. Wasmtime is pinned to supported releases with an advisory update SLA |
| Native helpers | Restricted: OS-confined helper started by a host-signed, self-testing launcher that refuses to run if the sandbox did not apply (this replaces `sandbox-exec`). Full: Verified tier or higher, signer identity pinned in the catalogue, consent per version. Both run out of process; native code never loads into the app |
| Control ports | Today six apps' local control ports accept any local process (*source*). Every port gets a per-run token, withheld from helper environments. This is a Phase 0 fix: without it, any native helper can drive the app around every permission |

### 6.1 Budgets

Every plugin class has a time and memory budget, and a ladder of what happens when it is exceeded.

| Class | Budget | When exceeded |
|---|---|---|
| Slider preview | ≤16 ms p95 at preview scale | Proxy scale, then last good frame, then bypass with a badge |
| Playback | Frame budget minus the app's own share | Same ladder |
| Export or render | No deadline, but a progress obligation | Never degrades; fails with a named error |
| Panel UI | No guest call per frame; patches coalesced | – |
| Audio block | ≤50% of the block period | Dry signal with a 64-sample crossfade; auto-bypass after repeated faults |
| GPU submit | ≤4 ms interactive; ≤50 ms for jobs while interactive, ≤250 ms when idle | Strike at 1 s pending; quarantine by shader digest on device loss |
| Host calls | Charged by bytes and wall time, not a flat fuel count | Batch ops instead |

## 7. Permissions

### 7.1 Vocabulary

This extends `ext-core/src/permission.rs` without renaming anything.

| Permission | Grants | Asked |
|---|---|---|
| `document.read:{selection,document,all}` | Reads within scope | At setup |
| `document.edit:{selection,document,all}` | Edits within scope; covers read | At setup |
| `document.create` | New and duplicate documents; closing documents the plugin created | Setup |
| `selection.write` | Changing the selection | At setup. Works only in events the user invoked; refused in background, automatic, agent and service runs |
| `app.view` | Viewport, zoom, playhead, active tool, panel visibility | Declared, never prompted. Refused in automatic events without `app.automation` |
| `app.session` | App state outside the document, not preferences | Setup, low-risk wording |
| `app.preferences:<group>` | Writing app preferences in one group (for example `canvas`, `export`, `units`). Groups that control plugins, scripts, automation, security or grants are never grantable. Refused in automatic contexts | Setup, strong wording ("Can change your PhotoCraft export settings") |
| `app.events[:family]` | Subscriptions | Setup ("runs automatically when…") |
| `app.automation` | Edits without a user gesture (hooks, timers, change follow-ups). Needed on top of `document.edit` | Setup, strong wording |
| `app.intercept:<point>` | App-declared hook points: `before-save`, `before-export`, `input.filter`, `tool:<id>` | Setup, plus a per-contribution toggle |
| `files.read:picked`, `files.write:picked`, `files.folder:picked` | Portal handles; folder handles persist and are revocable | The picker is the grant |
| `network:<host>` | `net.fetch` to declared hosts, and app commands whose metadata declares only those hosts (cloud libraries, font and model services). Refused in automatic and render contexts | At setup, showing the manifest's reason |
| `net.local:<port>` | Loopback to a companion app (Verified tier and above). The companion must answer a challenge, and Craft control, MCP and bridge ports are always refused | Setup, at the Online risk level. Owner decision |
| `clipboard.read`, `clipboard.write` | Clipboard during user gestures | At setup; both work only in events the user invoked |
| `ui.window` | Floating windows, never modal or always-on-top | Setup |
| `service:<plugin>/<name>` | Calling another plugin's declared service; the provider runs with its own grants | Setup, showing transitive reach |
| `media.capture:{audio,video}` | Host-brokered frames | Just in time |

Fonts, presets, workspaces and default settings are data contributions. The user applies or
installs them from the manager; no plugin permission does it.

**Free, never prompted:**

- compute and kernels, GPU shaders, `nn.run` with packaged models;
- the plugin's own settings, secrets and document data;
- package assets, panels, menus, commands and shortcuts (only if unbound), timers, logging,
  `host.capabilities`.

**Never grantable, with no spelling:**

- processes, raw paths and sockets;
- preferences that control plugins, scripts, automation, security or grants; undo history, plugin management, grants;
- collaboration identity, invoking agents or MCP;
- other plugins' data or secrets.

### 7.2 Scopes and attenuation

| Mechanism | Rule |
|---|---|
| Selection scope | The selection captured at event start, its descendants, and objects this transaction created. Changing the selection inside the event never widens scope |
| Enforcement on the write set | Scope is checked on what actually changes, not only on parameter ids. Commands that act on "the current selection" have their implicit targets rewritten to explicit ids before they run. Every step's write set must be a subset of the captured scope plus created objects, or the transaction rolls back. This stops `select.all` followed by an implicit-target edit. Out-of-scope ids return the same error as absent ones. CPU and GPU views are cropped to scope |
| Document scope | The document the event targets, not whichever is active when a call arrives |
| Object references | A parameter referencing an object (`Layer`, `Mask`, `Path`, `File`) grants read access to that object only |
| Setup grants and gestures | Every permission, selection scope included, is listed and approved once when the plugin is enabled (owner decision 5). No permission is granted implicitly. `selection.write`, clipboard and `ui.open_url` additionally work only in events the user invoked. Only hardware input makes an event user-invoked: input carries its origin, and MCP `press_key`, `answer_dialog` or UI-click automation runs the plugin as the agent principal |
| Trust UI input | Consent, just-in-time, install, native and widening sheets ignore automation input entirely |
| Handles | Read-only document handles can be lent to services. Job handles hold read snapshots. File and folder handles come only from portals. All handles die with their scope |
| Revocation | Takes effect mid-event: the next call fails |

### 7.3 Risk levels

The host computes one label from the manifest and grants. Authors never set it. Tier (Official,
Verified, Community, Local, Dev) is shown beside the level, never merged with it.

| Level | Label | Condition |
|---|---|---|
| L0 | Works on what you select | Selection scope only, no events, no network |
| L1 | Works with your documents | Document scope, events, automation, hooks and interception, input filters, tool replacement, clipboard read, folders |
| L2 | Online, or changes app settings | Any `network:`, `net.local:` or `app.preferences:`. The sentence names the hosts or setting groups, and says whether the plugin also reads documents |
| L3 | Native, confined | Native restricted helper |
| L4 | Full access to your computer | Native full; two-step consent |

The risk function is total: every grantable variant has an exhaustive match, so a new permission cannot
ship without a level.

**Prompt rules:**

- A widening update is held until the user approves it. Widening is computed over the whole capability
  surface: permissions, automatic-run kinds, hooks, agent exposure, service exports, shortcuts, apps and
  risk level. An app-side lock widening re-surfaces the grants it affects.
- No more than one just-in-time prompt per minute per plugin.
- Unused optional grants reset after 90 days.
- Headless runs never prompt.
- Each plugin has an activity log of commands, network hosts and files.
- Organisations can set a policy file.

### 7.4 Agents and plugins

- Agents and plugins are separate principals. Neither can install, enable or grant the other.
- A plugin's commands reach MCP agents only if the plugin opts in. Agent access is on by default for
  L0–L1 plugins and off for L2 and above.
- An agent-invoked plugin runs only with its own persistent grants.
- Plugin text returned to an agent is marked as untrusted data.
- MCP tool schemas and their read-only and destructive hints are generated from the same `Meta`.

## 8. Transactions, events, jobs and hooks

- **Transactions.** One plugin event is one transaction and one labelled undo step. Registry calls
  inside it run nested: no journal entry, no undo push, no redraw. One coalesced
  `document.changed {revision, changes[]}` is emitted at commit.
- **Long work.** Commands that are `Long` or spawn jobs run only through `job.run_command`:
  1. it snapshots the inputs and shows progress and cancel;
  2. it delivers the result as a new transaction with a revision check;
  3. `-4 stale` lets the plugin decide what to do.
- **Events.** `events.subscribe {names, filter}` with canonical payloads: `document.changed`,
  `selection.changed`, `time.changed`, `tool.changed`, `color.changed`, `history.changed`,
  `document.saved`, `panel.opened/closed`, `dispose`. Changes are coalesced per frame.
  - Each app needs a real change source. Today every app polls, so these come from FigCraft's
    collaboration diffs, LightCraft's operation log and the other apps' command journals.
  - Edits made in response to an event run as a deferred follow-up event under `app.automation`. A
    hook's own changes do not retrigger it, the depth limit is 1, and plugins are rate-limited.
- **Hooks.** Behaviour changes go through app-declared hook points only:
  - providers (defaults, presets, snapping, export steps);
  - `before-save` and `before-export` (fail open, with a deadline);
  - input filters;
  - tool alternatives the user chooses.

  Hooks run in a deterministic order, with a bypass modifier, and safe mode disables all of them. There
  is no generic "wrap any command" hook.
- **Services.** `[[provides.service]]` in the manifest and `service.call` from consumers. The provider
  runs in its own sandbox and acts only on handles the consumer passed. The consumer's authority
  applies, never the provider's. The consumer's risk level is the highest over the providers it may
  call, pinned at grant time. Services are versioned like contracts, re-entry into a busy instance
  returns `-4`, and the depth limit is 2.
- **Automatic execution has no capabilities.** These contexts get no network, secrets, files,
  clipboard, service calls or registry writes:
  - render, cook, expression, preview and widget contexts;
  - hooks caused by a collaboration peer's change;
  - hooks caused by an externally sourced document.

  Network-dependent results are stored, and re-run only by an explicit user action. L2+ plugins do not
  auto-run on `document.opened` or `document.changed` for an external document until the user acts in
  it.
- **Plugin output is untrusted input.** Importer output and plugin edits pass the app's normal parser
  validation. They cannot create path links, scripts, expressions, foreign `plugin_data` namespaces or
  external references. Primary exports are produced by the host and cannot be rewritten.
- **Revocation is complete.** Disabling or quarantining a plugin stops every execution path: events,
  render, replay, undo-triggered re-render, services and agents. The kill list is checked at every
  instantiation.
- **One resource account per plugin.** CPU, memory, GPU, disk (caches, models, settings, plugin data,
  logs), undo slots, prompts and audit entries are all counted. Plugin activity never evicts the user's
  undo history, the app's own caches or security audit records.

## 9. GPU

All 11 apps use wgpu 30.0.1 and naga 30.0.1 (*source*), so one plugin GPU host serves the suite.
GPU-resident work avoids the readback that CPU plugins force today: 5.2 ms per 1080p RGBA32F round
trip in EffectCraft's GPU chain, against 0.39 ms for the same pass on the GPU (*measured*).

| Piece | Design |
|---|---|
| `craft-gpu` (craft-libs) | Device broker, texture pool, memory cap counted from resource descriptors (wgpu reports nothing on Metal or WebGPU), timestamp scheduler, device-loss handling, pipeline warm-up at install or enable (a new Metal pipeline costs about 64 ms the first time, 0.2 ms once cached, *measured*) |
| `ext-gpu` (craft-extensions) | naga validation (0.08 ms, *measured*) and IR rules, host-composed shaders, L1 fusion, L2 DAG execution, per-plugin quotas, crash journal |
| Device choice | The device that already holds the data. Interactive GPU chains (EffectCraft, PhotoCraft canvas, FilmCraft, LightCraft) use the app device. CPU-resident apps (TextureCraft's tile compositor, MaterialCraft's cook) use `craft-gpu`'s job device |
| L1 `gpu.kernel` | A WGSL function body. The host writes the entry point and bindings, and fuses chains. No storage, workgroup memory, atomics or barriers in plugin code. Loops are bounded by wgpu's safe path. Community tier eligible after a fuzz and CTS gate |
| L2 `gpu.pass` | Full WGSL compute or fragment modules in a manifest-declared DAG. Storage buffers the host created for this plugin, capped workgroup memory, atomics on plugin-owned buffers. Verified tier first. Enables particle simulation, reductions and sorts |
| L3 imperative `gpu.*` | Deferred until the catalogue names a need. A validating WebGPU proxy would be the largest attack surface in the design |
| L4 viewport | Overlays through retained draw lists with hit regions first; custom 3D render passes later |
| Invariant | Every binding holds only data the plugin may read, cropped to its scope. App textures are bound read-only, without a copy. Shaders are package artefacts identified by validated-IR digest plus package id; nothing compiles WGSL generated at run time. Every pooled texture and arena is cleared or owner-tagged before a plugin sees it. No fusion across packages. No GPU timing is exposed to plugins |
| Hang safety | Tiled submits sized from timestamp estimates; per-frame budget; watchdog; a journal written before each submit turns a GPU hang or crash into quarantine by digest on the next start |
| CPU fallback | Not a mandatory second implementation. Headless and CI use software adapters (lavapipe, WARP; Metal on macOS runners). Golden images with a tolerance, by determinism class |
| Defaults | RGBA16F interchange, f32 on request. Timestamp queries requested everywhere they are available |

## 10. UI

- **Rendering tiers**, all composited by the host:
  - **T1.** Retained declarative tree: UI 1.3 with keyed patches, value bindings, live controls and
    lazy rows.
  - **T2.** Retained layered canvas with hit regions and theme-token paints.
  - **T3.** Host editor components: curve, gradient, node graph, timeline lane, meter, scope.
  - **T4.** Viewer of image handles produced by the GPU or job tiers.
  - **T5.** Native windows, for native-tier plugins only, labelled as external.
- **No webview.** It cannot be docked, clipped or themed in egui or wgpu, it adds WebKitGTK on Linux,
  and it opens a phishing surface. This is an owner decision.
- **Contribution points** are a closed, versioned catalogue:
  - commands, menus, palette, shortcuts;
  - toolbars, tools with gestures and gizmos;
  - panels and docks, inspectors, context menus, status items;
  - canvas and viewport overlays, node-editor nodes, timeline lanes;
  - import and export entries, settings pages, notifications, dialogs and windows;
  - workspaces, themes and keymaps (as data the user applies).

  Every surface has a stable id, `plugin.<id>.<local>`. Its saved place survives the plugin being
  missing, through the framework's `reconcile`.
- **Limits** (suite-wide, in the runtime):

  | Per plugin | Per app |
  |---|---|
  | 16 panels, 4 windows, 2 status items, 32 menu rows, 1 modal | 64 plugin panels, 32 open windows |

- **Zero guest calls per frame.** The host keeps the tree, applies patches, reads bound values and
  lock-free rings, and coalesces input to one event per widget per frame. Images travel by handle.
  Nothing on the UI thread waits for a plugin; today PhotoCraft's tool path can block for up to 10 s
  (*source*).
- **Trusted chrome.** Host chrome names the plugin on every surface. Consent and permission UI sits on
  a host-only layer that freezes input to plugin surfaces. Plugins cannot:
  - draw outside their rects;
  - take focus without a user gesture;
  - create always-on-top or transparent windows;
  - capture keys globally;
  - hide built-in UI;
  - block save or quit.
- **Framework.** PhotoCraft is on the craft framework. Nine apps are on legacy `craft-ui` with egui,
  and AniCraft is on stock egui 0.36 (*source*). One UI model (`ext-ui-model`) has two renderers:
  - the framework renderer is the end state;
  - the egui renderer is a bridge, deleted once the last app ports.

  Each app shell implements `HostUi` once.

## 11. AI

- **`craft-nn`** in craft-libs:
  - a candle backend for curated architectures (SAM, Whisper, depth, upscalers, denoisers);
  - LightCraft's WGSL convolutions as a fast path on the app device;
  - a generic ONNX backend chosen by a spike over tract and candle-onnx, tested on five target models.

  LightCraft's own ONNX reader supports only six operation types, so it cannot be the runtime
  (*source*).
- **Model store.** One content-addressed store for the whole suite, with models memory-mapped. It
  replaces five separate inference setups and three model downloaders (*source*).
- **Models are package data.** They are pinned by sha256, and their size is shown at install.
- **`nn.run`** runs as a job and is cancellable. It needs no separate permission; downloads go through
  the consented network path.

## 12. Audio

- **SoundCraft fixes first** (*source*):
  - make automation sample-accurate, so bounce equals playback;
  - give each slot its own fault handling, so one panic no longer silences the whole output;
  - add event lists;
  - add a guard against allocation on the audio thread.
- **One transport, `craft-audio-rt`.** Both kinds of plugin share it: Wasm audio on a real-time thread
  in the package worker, and native CLAP, VST3 and AU in `craft-plugin-bridge`. Both use shared-memory
  rings and one wake per block.
  - The host measures p99 at activation.
  - A slot may switch to anticipative mode (one block of reported latency) only before playback
    starts.
  - Faults output the dry signal, never silence.
- **Contracts.** `audio.effect@1`, `audio.instrument@1` and `audio.note-fx@1`, with CLAP semantics:
  - sample-accurate parameter and note events;
  - modulation separate from automation;
  - gestures for automation recording;
  - note expressions, MIDI 1 and 2;
  - latency fixed at activation;
  - sidechain ports, state, meter rings.

  An adapter imports WebCLAP modules.
- **Cost.** About 1.5× native CPU on a JIT (WebCLAP figure, *cited*). The cross-process wake latency
  is *unmeasured* and is the audio phase gate.

## 13. Documents, determinism and missing plugins

- **Determinism class.** Every manifest declares one: `exact`, `seeded`, `tolerant`, `stateful` or
  `networked`. Cache keys include `(id, version, sha256, contract, determinism, backend class)`.
- **Durable output.** Every plugin output the render path depends on is stored as
  `{plugin, version, digest, params, determinism, durable_output}`. The durable output is either a
  baked table (eases, curves, LUTs, deformation) or a cached result.
  - A missing, quarantined or version-mismatched plugin renders its durable output with a badge.
  - Export uses the durable output and says so, or refuses unless the user chooses "render without".
  - Silent pass-through ends. Today it happens in FilmCraft, PhotoCraft smart filters and VectorCraft
    effects (*source*).
- **Tolerant loading.** `craft-preserve` keeps unknown enum variants and fields, so a document with
  unknown plugin data opens and saves byte-identical. Today one unknown variant makes a whole
  TextureCraft document fail to load, and FilmCraft drops unknown data (*source*). A per-app round-trip
  test gates every adapter.
- **Render farms.** A project plugin lock, plus an admin-signed farm policy file listing the grants
  allowed on render nodes.
- **Collaboration.** The FigCraft server must own plugin-data integrity instead of trusting clients. A
  document from a peer runs no plugin automatically; see section 8.

## 14. Distribution and trust

| When | What |
|---|---|
| Now | ed25519 catalogue signature; hourly signed revocation snapshot; widening holds updates; auto-update cool-off of 72 h for Community, 24 h for Verified, none for Official |
| Before public launch | TUF with an offline threshold root. The app verifies GitHub or Sigstore provenance offline for native, Verified and Official packages. Catalogue CI rebuilds Wasm packages reproducibly for a "Reproduced" badge |
| Submission checks | The existing `check-submission` (lookalike names, widened permissions, immutable releases) plus `api.lock.json`-aware permission diffs |

## 15. Authoring and agents

- **SDK.** `craft-guest` with generated `apps::<app>` modules and the canonical ops. Typed errors
  carry the permission needed and the reason. `host.capabilities` and `host.api_level(app)` let
  plugins degrade explicitly.
- **Tools:**
  - `craft-ext new <kind>` templates per contribution kind, each with a passing test;
  - `craft-ext dev --watch` with hot reload that keeps panel state;
  - `craft-ext test` with a MockHost loaded from the real `api.lock.json`, session scripts in JSONL
    and golden images;
  - `craft-ext run --app --doc --script` as one headless runner;
  - `craft-ext bench`;
  - an in-app console with per-call CPU, GPU, bytes, IPC and fuel.
- **Languages.** Rust first; generated headers for C and Zig; TypeScript/AssemblyScript bindings.
  JavaScript through a bundled QuickJS module later.
- **For maintainers and agents:**
  - one conformance suite every adapter must pass;
  - one `api.lock.json` per app as the security boundary;
  - generated docs and bindings, never hand-written;
  - the comment ratchet;
  - AGENTS.md rules: behaviour in the runtime only; every command has `Meta`; every widening has a log
    line; every threat has a test.

## 16. What each app needs

The detail is in research reports 05–07. This table is the minimum for an app to host plugins at full
breadth.

| App | Commands | App work before the adapter | Adapter | Unlocks |
|---|---|---|---|---|
| EffectCraft | 757 | `Meta`; dock plugin panels (today they float); GPU path for external effects; render-request API | Pass through `EffectHost` (other frames, layers, audio, camera, depth/ID); `raster.filter@2`; expressions, masks, paths, text, keys, tracking, render queue via the bridge | Particular, Saber, Optical Flares, Twixtor, Neat-class denoise, Sapphire, Mocha-style panels, Duik, Newton, BeatEdit |
| FilmCraft | 686 | `Meta`; plugin data in projects; keep unknown fields; external effect, transition and audio registries; refuse silent pass-through | `filmcraft.session@2`; `raster.filter@2` on clips; transitions; audio effects | Universe transitions, looks, captioning, scene detection, review panels |
| AniCraft | 365 | `Meta`; egui 0.37; plugin panel; open filter and tool registries | `anicraft.session@2`; bridge for its own ABI v1 plugins, then removal | Rigging, lip sync, inbetweening, exporters, brush engines |
| PhotoCraft | 954 + 146 shell | `Meta`; compute queue; smart filters dispatching plugins with durable output; dynamic command overlay | Tile-native pixels with colour space and bit depth; masks, channels, styles, text, paths, smart objects; paint strokes; Actions journaling | Topaz, Nik, Portraiture, Lumenzia, Coolorus, brush engines, UXP-style panels |
| LightCraft | 391 + 213 UI | `Meta`; batched sidecar writes; export and publish hooks; custom metadata fields; profile registry | `lightcraft.session@2`; `export.service@1`, `publish.service@1`, `metadata.provider@1` | Publish services, Negative Lab Pro, LRTimelapse, metadata tools |
| FigCraft | 630 | `Meta`; server-owned plugin data | Bridge (from 72 to about 500 commands); image fills; PNG/PDF export; codegen; selection-scoped grants; change lists | Unsplash, Iconify, Autoflow, Stark, Tokens Studio, codegen tools |
| VectorCraft | 1,044 | Convert its reviewed list (718 ids) into `Meta`; appearance registry with durable output | `vectorcraft.session@2`; `vector.effect@1`; formats; legacy shim, then removal | Astute tools, CADtools, GIS import, Phantasm |
| DesignCraft | 567 + 194 UI | `Meta`; reclassify file-writing "queries"; change sets; string preflight kinds | `designcraft.session@2`; `document.check@1`; story import | Data publishing, automated layout, preflight, imposition |
| MaterialCraft | 133 | `Meta`; open node registry; SBS dependency URLs; change events | `resource.node-library@1` first (its library is already 549 data graphs), then `graph.node@1` | Designer-style nodes, Material Maker-class shaders, batch cooking |
| TextureCraft | 183 | Tolerant loading first; plugin variants in 17 closed enums; compositor path | `texturecraft.session@2`; `graph.node@1`; resources; bakers; viewport overlays | Smart materials, generators, asset libraries, custom bakers |
| SoundCraft | 483 | Sample-accurate automation; per-slot faults; event lists; no-allocation guard; move native hosting into the bridge | `craft:` (sandboxed Wasm) and `bridge:` (native, out of process) plugin kinds; `audio.*` contracts | Valhalla- and FabFilter-class effects, synths, JSFX-style scripts, stem separation |

**Competing systems to replace:** VectorCraft's legacy plugins (about 12,300 lines), AniCraft's ABI
v1, and SoundCraft's in-process native hosting.

**Automation engines that stay**, but classify through the same `Meta`: EffectCraft scripts and
ScriptUI, VectorCraft JS, DesignCraft's step runner and PhotoCraft Actions.

## 17. The honest boundary

Plugins get about 79% of the command surface directly: document edits, queries, selection and view.
About 14% more comes through host mediation: file handles, clipboard, jobs and host-owned undo.

About 7% stays for the user and agents only, by design:

- shortcuts, and preferences that control plugins, scripts, automation or security;
- process and OS actions;
- plugin management;
- collaboration identity;
- window management;
- model installs;
- raw undo groups;
- app lifecycle.

Plugins still reach most of those outcomes by safer routes: ordinary preferences through
`app.preferences:<group>`, presets and workspaces as data, app network commands with declared hosts
or `net.fetch`, and `handoff.open` instead of launching processes.

**Native plugins remain for:**

- hardware: tethering, colorimeters, controllers outside the app's device layer;
- licence daemons;
- closed vendor engines: UVPackmaster, Houdini Engine, CUDA-only kernels;
- the commercial VST3, AU and CLAP catalogue;
- round trips with apps outside the suite.

All of them run out of process. Of the roughly 300 incumbent plugins catalogued, about 85% become
possible as sandboxed plugins (*estimate* across reports 05–07).

## 18. Security invariants

These are non-negotiable, ranked by the red-team review. Each states a deterministic check that fails
the build. The attack chains behind them are in [research/09-redteam.md](research/09-redteam.md), and
[roadmap.md](roadmap.md) schedules the checks.

| # | Invariant | Check |
|---|---|---|
| 1 | No plugin-derived machine code is compiled or runs in an app process. A missed latency target is answered with batching, never with in-process JIT | `cargo tree -i cranelift-codegen` is empty for app crates; the main app lacks `allow-jit`; `disallowed-methods` bans Wasmtime module creation outside the worker |
| 2 | Authority is checked at the sink, with the caller's origin carried through every nesting. The lock file is a second layer, never the only one | Mock sinks that fail on a plugin origin reached from any exposed command; any re-entrant dispatch must declare `Dispatch` |
| 3 | Scope is enforced on what changes. Implicit targets become explicit ids; the write set is a subset of captured scope plus created objects; selection writes are gesture-only; out-of-scope ids look absent; views are cropped | Generated per-command scope tests from the lock; differential error test; masked-view sentinel test |
| 4 | Only hardware input approves trust UI or mints a gesture | Per-sheet tests that `answer_dialog`, `press_key`, UI clicks and dialog confirms cannot change grants |
| 5 | No plugin reaches the operator channel. Every control, MCP and bridge port needs a per-run token that workers and helpers never inherit; loopback grants need a companion challenge and refuse Craft ports; restricted sandboxes deny loopback and Unix sockets | Live connect tests from the worker, a restricted helper and a `net.local` grant; environment and file-descriptor scan for tokens |
| 6 | One closed permission vocabulary and one total risk function. Automation, interception, input filters and tool replacement are at least L1 | Enum-iteration test; no document or code names a permission the enum cannot parse |
| 7 | Automatic execution has no capabilities: no network, secrets, files, clipboard, preference writes, services or registry writes in render, cook, expression, preview, widget, peer-triggered or external-document contexts | Contract-table test per contribution kind; open-and-render fixture with a network sink that fails |
| 8 | Plugin output is untrusted input to the host: no path links, scripts, expressions, foreign plugin data or external references; primary exports are host-produced | Sentinel-link test; importer fuzz targets; export-hash equality |
| 9 | Agents treat all document and plugin text as data, and see plugin commands only by opt-in | Per-server MCP snapshot tests with injection fixtures; `actions.play` principal test |
| 10 | GPU code is a validated package artefact with owner-initialised resources: no run-time WGSL, per-package strikes, pooled resources cleared, no cross-package fusion, no timing to plugins | Pool-reuse sentinel test; run-time pipeline refusal; per-package quarantine; fusion namespace test |
| 11 | Service calls carry the consumer's authority, not the provider's | Cross-scope service edit test; provider-widening hold test |
| 12 | Widening is computed over the whole capability surface and holds the update | Manifest-pair table test; lock-diff review flag test |
| 13 | Revocation is bounded in time and complete in reach. Stale revocation data degrades L2+ Community and native plugins | Instantiation counter after kill; clock-injected freshness test |
| 14 | Every plugin-attributable resource is on one per-plugin account, and plugin activity never evicts user state or security evidence | Undo-flood, audit-eviction, per-store quota and app-cache invariance tests |
| 15 | Every host parser reachable by plugin bytes, and every plugin string shown in chrome, is bounded and sanitised: no bidi or control characters, no built-in labels or button words, always attributed | Contribution-kind to fuzz-target table test; Unicode refusal fuzz; snapshot tests; threat-map coverage check |

**Accepted residual risks.** These are stated so nobody "fixes" them by widening a grant:

- A user who grants `document.read` plus `network:<host>` consents to that host receiving their work.
- Native full access is total. Its consent says it can drive other Craft apps whose automation ports
  are on.
- GPU driver bugs reachable from valid WGSL remain.
- Steganography in exports the user saves remains.
- Peers can always write bad data into a shared document. The defence is that nothing runs on it
  automatically with authority.

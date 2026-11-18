# Craft plugin platform: centralization, capability and performance gap analysis

Date: 2026-11-18. Scope: `craft-extensions` (shared platform) and the three adapters in
`effectcraft`, `photocraft` and `figcraft` on the `plugin-beta` branches. I read the source and
ran scoped benchmarks on an Apple M5 (10 cores, 24 GB). Nothing was built in the app repos, and no
repository file was changed. The adapter findings come from reading the code, not from running it.
The benchmark sources and raw outputs are in `bench/`.

## Summary

1. **The sandbox runs at a quarter of the speed it already has.** `ext-host` enables
   `wasmi = { features = [..., "portable-dispatch"] }` (`crates/ext-host/Cargo.toml`). That forces
   wasmi 2.0's loop-based interpreter. Without the feature, the same wasmi 2.0 uses tail-call
   dispatch, and the per-pixel loop goes from **42.1 ms to 11.3 ms** at 1080p with fuel on (3.7×).
   Changing one line of Cargo.toml would make every existing plugin faster. Wasmtime 50 with
   Cranelift runs the same module in **0.96 ms**, as fast as native Rust (≈1.06 ms). Wasmi with the
   current features is ≈40× slower than native.
2. **The platform copies too much.** `Filter::run` costs ≈10 ms of copy, byte-conversion and
   validation on top of the 42 ms loop at 1080p. A Wasm filter that calls `kernel.blur` takes
   28.4 ms, but the kernel alone takes 13.7 ms, so about half the time goes on copying f32 data
   between Wasm memory, `Vec<u8>` and `Vec<f32>`.
3. **Host calls are cheap, but the fuel budget limits how many an event can make.** A
   `craft.call` round trip with a small JSON payload costs **10.5 µs (≈95 k calls/s)**. An event
   holds at most ≈19 k calls (2e9 fuel ÷ 100 k fuel per call). 30 k calls fail with `TooSlow`, so a
   FigCraft plugin that touches every node of a large file cannot finish in one event.
4. **About 11.8 k lines of adapter code sit on 26 k lines of shared code. Roughly 1.5 k of those
   adapter lines are the same logic written three times, and each copy behaves differently.** It
   covers grants, the transaction loop, workers and quarantine, data ops, `ui.*` limits, job
   delivery, app events, catalogue refresh and project references. The copies differ in:
   - rollback replies;
   - timeout messages;
   - event payloads;
   - how permission matching is done (`covers` or exact equality);
   - refresh schedules;
   - grant caching. PhotoCraft and FigCraft reopen the grant store on every permission check; for
     PhotoCraft that is ≈8 `GrantView::open` and 3–4 contract rebuilds per op.
5. **Op contracts are shared in name only.** Only 6 ops are identical in all three apps.
   `selection.get` returns three different shapes, and `layer.select` needs different permissions
   in different apps. A plugin cannot be written once for the whole suite.
6. **Missing pieces for a "do anything" sandbox, most important first:**
   - a JIT engine;
   - zero-copy pixel views and tiles;
   - GPU (WGSL) effects;
   - settings and secrets storage;
   - network with consent;
   - clipboard;
   - timers;
   - inter-plugin services;
   - a host inference op (EffectCraft already ships candle for SAM2);
   - one headless runner;
   - a typed ABI (WIT, or generated bindings over the JSON ABI);
   - a mock host for session tests.

## Centralization review

### Size of the code

| Area | Files | Lines (non-test unless noted) |
|---|---|---|
| Shared crates | ext-core 5,912; ext-ui 3,994; ext-host 2,999; ext-manager 2,955; ext-store 2,604; ext-cli 2,233; ext-ui-fw 2,119; guest 2,115; ext-kernels 988; native-sdk 256 | ≈26,200 |
| EffectCraft adapter | `crates/plugins/src/*` 3,771 (runtime.rs 2,247, ops.rs 554, effects.rs 386, contract.rs 305, lib.rs 165, grants.rs 114); `engine/src/ext.rs` 363; `ui-egui/src/plugins_ui.rs` 207; plugin bits of `panels/viewer.rs` | ≈4,350 |
| PhotoCraft adapter | `crates/plugins/src/*` 2,036; `engine/src/plugin_ops.rs` 1,402, `plugin_sessions.rs` 1,196, `plugin_filters.rs` 424; `ui-egui/src/plugins_ui.rs` 527, `plugin_form.rs` 182, `plugin_manager.rs` 134 | ≈5,900 |
| FigCraft adapter | `crates/plugins/src/{runtime 559, contract 122, lib 200, grants 31}`; `ui-egui/src/plugins_ui.rs` 405 (+1,569 test lines) | ≈1,320 |

Today's seam is `AppApi` (`crates/ext-host/src/api.rs:166-172`): `contract()`, `begin(inv)`,
`call(inv, op, Value)` and `finish(inv, commit)`. The seam is too shallow. Everything around it is
left to each app: threads, timeouts, the owner-thread handshake, grants, reloads, workers, events,
jobs and references. Each adapter therefore rebuilt that machinery itself.

### Logic duplicated across adapters

| Concern | EffectCraft | PhotoCraft | FigCraft | ≈ duplicated lines | How the copies diverge |
|---|---|---|---|---|---|
| **Grants** (`Grants` impl, safe mode) | `plugins/src/grants.rs:1-77`: cached `GrantView`, 250 ms throttle. But `validate_request` (`runtime.rs:836-861`) opens a fresh view and runs twice per call (`:868`, `:914`). | `plugins/src/runtime.rs:25-44` `LiveGrants`: new `GrantView` and rebuilt `contract()` on every check. ≈8 opens per op (`plugin_sessions.rs:511,536,562`) | `plugins/src/grants.rs:1-31`: reopens on every check by design | 150 | Caching, failure modes and safe-mode switches (flag vs env) all differ |
| **Transaction loop** (begin/finish, owner-thread handshake, rollback, timeout) | `runtime.rs:863-972` + `engine/src/ext.rs:115-160` | `plugin_sessions.rs:465-613`: clones the whole `DocState`, restores history by hand | `runtime.rs:260-301` | 300 | Nested Begin: roll back / error and continue / mark failed. `Finish(false)` replies `Ok(null)` / `Cancelled "Event rolled back"` / `Cancelled "plugin transaction rolled back"`. Only EffectCraft tells the user about the 5 s timeout |
| **Wait-and-serve loop** (UI thread waits on worker) | `load` waits up to 8 s (`runtime.rs:499-516`) | `plugin_wait` up to 10 s | `deliver` up to 9 s | 120 | Three different UI-thread stalls |
| **Workers and quarantine** | `worker_main :1065-1165` | `:128-220` | `spawn :183-230` | 200 | Same three-strike policy, written three times around `policy::Strikes` |
| **Plugin data ops** | `ops.rs:133-174` | `plugin_ops.rs:545-584` | `runtime.rs:419-439` | 120 | FigCraft stores JSON as text under `plugin:<id>` and writes past the edit API |
| **`ui.*` limits** (surfaces, windows, ids) | `ops.rs:186-300` | `plugin_ops.rs`, `runtime.rs:436-494` | `runtime.rs:302-401` | 200 | 4 windows per plugin (EffectCraft, FigCraft) vs 32 surfaces per app (PhotoCraft). Different error texts. FigCraft `ui.notify` only logs |
| **Job delivery** | `runtime.rs:527-570` | `runtime.rs:339-363` + `plugin_sessions.rs:115-183` | `runtime.rs:160-177` | 110 | Document key (`document-N` / `doc.id` / `uid`) and what blocks delivery differ |
| **App events** | `:1277-1325` | `plugin_sessions.rs:121-130` | `:497-511` | 100 | Different payloads for the same `selection.changed`. Permission matched by `covers` / string compare / exact enum. PhotoCraft drops hook errors |
| **Project references / missing plugins** | `runtime.rs:1167-1238,1528-1588` + `engine/src/ext.rs:169-256` | `lib.rs:69-80,224-240` | `:69-113` | 100 | PhotoCraft writes `state_versions: None`, so it cannot detect a state schema mismatch |
| **Discovery, reload and catalogue refresh** | `lib.rs:91-120` (6 h, backoff 5 m/30 m/2 h; `thread::spawn` can panic) | `lib.rs:35-53` (every launch, no backoff, also runs in safe mode) | `lib.rs:113-138` (60 s doubling to 15 min, off in safe mode) | 90 | Three schedules. PhotoCraft and FigCraft parse the whole store state on every pump |
| **Raster filter running** | `effects.rs:131-262`: per-thread instance pool, in place (good) | `lib.rs:104-218`, `plugin_filters.rs:279-390`: reads and compiles the module from disk twice per run; ≈6 full-buffer copies; no tiling or cache; the form starts a thread per slider change | none | — | Only EffectCraft has an instance pool |
| **UI glue** | `plugins_ui.rs:45-140` | `plugins_ui.rs:51-161` (its own jobs popover duplicates `ext-ui/src/jobs.rs`) | `plugins_ui.rs:62-122` | 200 | Two manager stacks: `craft_ext_ui::PluginManager` (EffectCraft, FigCraft) and `craft_ext_ui_fw::ManagerState` (PhotoCraft) |

Total: ≈1,500 lines of clear duplication, plus about 2,000 more lines of op glue. Not all of the op
glue is truly app-specific. For example, the FigCraft and EffectCraft ops forward to app commands
after a generic size check.

### What should move into craft-extensions

1. **`GrantCache`** in ext-store or ext-host. It should:
   - watch metadata with a single throttle;
   - fail closed;
   - expose a monotonic `changes()`;
   - share one instance per process.

   Remove all three adapter impls. This fixes the PhotoCraft and FigCraft per-call store reopen.
2. **`Txn` state machine** in ext-host. It should:
   - send begin, call and finish to the document owner thread over a channel the platform owns;
   - handle nested begin;
   - roll back on a failed check;
   - give one `Finish(false)` reply;
   - enforce one hold timeout (5 s) and one event timeout, with one user message;
   - avoid UI-thread waits, using a poll-based `pump(&mut doc)` call with a per-frame budget.
3. **`Workers`**: per-plugin worker threads, Strikes, quarantine, safe mode (one env name,
   `CRAFT_SAFE_MODE`, with the app aliases as fallbacks), crash markers and restart.
4. **Shared op handlers** for:
   - `ui.panel.set`, `ui.window.open`, `ui.notify` (a host toast callback), with one limit table
     (surfaces per plugin, bytes per app) and form patches for every app;
   - `canvas.overlay.set`;
   - `project.data.*` (already in `data::handle`, but each adapter re-wraps it);
   - `job.*`;
   - `plugin.resources`;
   - a new `settings.*`, `clipboard.*`, `net.fetch`, `timer.*` and `service.*`.
5. **Event bus**: canonical names and payload schemas for `document.opened`, `document.changed`,
   `selection.changed`, `time.changed` and `page.changed`; coalescing (time at most 4/s); queue
   limits; and `covers`-based permission checks.
6. **Job delivery**: `JobBoard::deliver` already exists. Move the document-key mapping and the
   "busy" predicate behind the adapter trait (below) so delivery has one implementation.
7. **References and missing plugins**: a shared `ReferenceTracker` that records use on commit,
   updates to the installed release, and keeps `state_versions` and contributions. The adapters keep
   only their prompt placement.
8. **Catalogue refresh scheduler**: one `std::thread::Builder` scheduler using FigCraft's policy
   (respect `last_refresh`, backoff, off in safe mode).
9. **Filter service**:
   - an instance pool per render thread (the EffectCraft design);
   - a compiled-module cache keyed by sha256;
   - tiling with apron (halo) requests;
   - a result cache keyed by (module sha, params, input hash, region);
   - zero-copy pixel views (below).

   PhotoCraft's twice-per-run compile from disk disappears.
10. **UI glue**: one manager stack. Either ext-ui-fw everywhere, or ext-manager as the model with
    thin egui and fw views. Also one jobs list, a problems banner, and Escape-to-cancel.
11. **Headless driver**: `craft-ext run --app <app> --doc <file> --script <jsonl>`, built on the
    same runtime that every app CLI links. Today each app CLI has its own syntax
    (`effectcraft-cli run <id> <json>`, `photocraft-cli run --cmd`, `figcraft-cli run script.jsonl --plugins`).

Bugs to fix whether or not code moves (all found by reading the code):
- FigCraft's read-only guard misses `selection.set` and `viewport.set`, because they are declared
  Query.
- `layer.select` needs edit permission in EffectCraft but only read permission in PhotoCraft.
- FigCraft `ui.notify` is invisible to the user.
- EffectCraft repaints every 100 ms when idle.
- PhotoCraft re-serializes every tree on each `ui.*` call.
- `figcraft/crates/plugins/src/lib.rs:17-104` (`Bridge`, `Worker`) looks like dead code.
- The author guide says PhotoCraft has no project references (§13), while `capabilities.md` says it
  does.

## Proposed adapter trait

Goal: the app implements **document primitives and its own ops**. The platform owns
**everything about plugins**. This is the "deep module" seam: a small interface with the lifecycle
behind it.

```rust
pub trait CraftApp: Send + Sync + 'static {
    type Doc: 'static;
    type Snapshot: Send;

    fn app(&self) -> AppId;
    fn provided(&self) -> Provided;
    fn ops(&self) -> &'static OpTable<Self>;

    fn active_document(&self) -> Option<DocToken>;
    fn with_document(&self, doc: DocToken, f: Box<dyn FnOnce(&mut Self::Doc) + Send>);

    fn snapshot(&self, doc: &mut Self::Doc) -> Self::Snapshot;
    fn restore(&self, doc: &mut Self::Doc, s: Self::Snapshot);
    fn commit(&self, doc: &mut Self::Doc, label: &str);
    fn busy(&self, doc: &Self::Doc) -> bool;

    fn plugin_data<'d>(&self, doc: &'d mut Self::Doc, at: &Target) -> Result<&'d mut PluginDataMap, OpError>;
    fn references<'d>(&self, doc: &'d mut Self::Doc) -> &'d mut Vec<PluginRef>;

    fn raster(&self, doc: &mut Self::Doc, at: &Target, region: Rect) -> Result<PixelView<'_>, OpError> { Err(OpError::not_found("raster")) }

    fn host(&self) -> &dyn HostUi;
}

pub struct OpDef<A: CraftApp + ?Sized> {
    pub id: &'static str,
    pub kind: OpKind,
    pub requires: fn(&Value) -> Vec<Permission>,
    pub schema: &'static OpSchema,
    pub run: fn(&A, &mut A::Doc, &Invocation, Value) -> Result<Value, OpError>,
}

pub trait HostUi: Send + Sync {
    fn notify(&self, level: Level, text: &str);
    fn request_repaint(&self);
    fn workspace(&self) -> &dyn WorkspaceDock;
    fn pick_file(&self, req: PickRequest, reply: Box<dyn FnOnce(Option<Handle>) + Send>);
}
```

The app emits events through a platform handle, `runtime.emit(AppEvent::SelectionChanged { .. })`,
with a typed payload. It never builds JSON.

What the platform does on top of this (`craft_ext_runtime::Runtime<A>`):

| Platform-owned | Replaces in adapters |
|---|---|
| Load, enable, reload and quarantine; workers; safe mode; crash markers | `runtime.rs` load/worker code in all three |
| Grants cache and every permission check (`covers` everywhere) | `grants.rs` ×3, `validate_request`, `LiveGrants` |
| Transaction state machine using `snapshot`/`restore`/`commit`, timeouts, one user message | `begin`/`finish` code ×3, `GroupMark` plumbing |
| Op dispatch from `OpTable`: schema check, permission, `contain()` panic guard, timing | `match op` blocks ×3 |
| Built-in ops: `ui.*`, `canvas.overlay.set`, `project.data.*`, `job.*`, `kernel.*`, `settings.*`, `net.*`, `clipboard.*`, `timer.*`, `service.*`, `plugin.resources` | per-app copies of the first five |
| Event bus with canonical payloads | three event emitters |
| Job delivery (`busy` + `active_document`) | three `deliver` loops |
| Reference tracking + missing-plugin prompt model | three reference algorithms |
| Filter service (pool, module cache, tiles, result cache) | PhotoCraft `plugin_filters.rs`, EffectCraft `effects.rs` pool |
| Contract docs, typed guest bindings and an app-CLI subcommand generated from `OpTable` | `contract.rs` ×3 (shrinks to op handlers only) |

Expected adapter size after the move:
- EffectCraft ≈1,500 lines (op handlers, the `ExternalRender` bridge and the viewer overlay);
- PhotoCraft ≈1,800 lines (op handlers, pixel IO and file pickers);
- FigCraft ≈700 lines (op handlers mapping onto existing engine commands).

That is about 65% less adapter code, and each copy behaves the same way.

Migration path: introduce `Runtime<A>` beside `AppApi`, port FigCraft first (smallest, already
command-based), then EffectCraft, then PhotoCraft. Then delete `AppApi::begin`/`finish` and the
`authorizes_on_owner_thread` flag.

## Platform capability gaps

Priority: P0 = blocks "Wasm can do almost anything fast", P3 = later. Effort: S ≤ 3 days,
M ≤ 2 weeks, L > 2 weeks.

| # | Gap | Today (evidence) | Proposal | P | Effort |
|---|---|---|---|---|---|
| 1 | **Interpreter dispatch** | wasmi 2.0.0 is the register-based engine, but `portable-dispatch` forces the slow loop backend (`ext-host/Cargo.toml`) | Drop `portable-dispatch` (keep `auto-dispatch`), so aarch64 and x86_64 get tail-call dispatch. Measured 3.7× faster | P0 | S |
| 2 | **JIT engine** | Interpreter only | Add `wasmtime` (Cranelift) behind an `Engine` trait for desktop. Keep wasmi as the fallback and reference engine. Use **epoch interruption** for slicing (+34% vs +75% for fuel here). Keep a byte-budget counter for host calls. Cache compiled artifacts by module sha256 in the store (compile costs 35–50 ms). Winch is a fast-compile tier (6.6 ms compile, about 4× slower than Cranelift) | P0 | M |
| 3 | **Zero-copy pixels** | `Filter::run` copies pixels twice in, twice out, and validates every value (`wasm.rs:637-690`). `kernel_call` converts Wasm bytes to `Vec<f32>`, then the kernel clones twice, then the result goes back to bytes (`wasm.rs:247-300`) | Write the host buffer straight into linear memory with one `memcpy` (`Memory::data_mut` + `bytemuck`). Let kernels run on `&mut [f32]` views of guest memory (no `Image` copy). Validate alpha and finite values in one fused pass | P0 | S |
| 4 | **Tiles, regions and temporal access** | One full frame per call, no apron, no other frames (`capabilities.md` "Not available") | Filter ABI 2: `craft_filter_region(ctx, out_rect, in_rects[])`, a host-requested apron, and `frame.get {time}` for temporal effects (EffectCraft public ABI 2 parity). Run tiles in parallel across instance pools | P0 | M |
| 5 | **SIMD128** | Refused: the module fails to load with "SIMD support is not enabled" (measured) | Enable the wasmi `simd` feature now. It works with wasmtime. Build the SDK templates with `+simd128` | P1 | S |
| 6 | **GPU compute and shader plugins** | No GPU path. EffectCraft (`crates/gpu`, wgpu 30) and PhotoCraft (wgpu 30) render on the GPU, so plugin effects force a CPU round trip | `shader` contribution kind: a WGSL compute or fragment shader in the package. Validate it with naga at install time and on load (no host pointers, bounded workgroups, a texture-only binding layout of input, output and uniforms). Params come from the form. The app runs it in its existing wgpu device. Wasm code can chain shader passes with `gpu.dispatch`. Use a CPU fallback through naga's interpreter or the Wasm path for headless runs | P1 | L |
| 7 | **Threads / shared memory** | Single-threaded guest. Kernels use 8 host threads | Short term: the host parallelises tiles (#4). Long term: wasmtime `shared-memory` + `wasi-threads` is not stable for untrusted code, so prefer host-side tile parallelism | P2 | M |
| 8 | **Settings storage (outside documents)** | None (`author-guide.md` §13) | `settings.get/set` per plugin in the store directory: JSON, 1 MiB, survives updates, removed on uninstall, no permission. `settings.ui` lets the host show a generated settings page from a form | P0 | S |
| 9 | **Secrets** | None | `secrets.get/set` backed by the OS keychain (Keychain, Credential Manager, Secret Service), namespaced by plugin id, with a `secrets` permission and consent | P1 | S |
| 10 | **Network** | `network:<host>` permission exists, no op (`capabilities.md`) | `net.fetch {url, method, headers, body}` as an async op that returns a request id, with completion as an event (like jobs). It should: allow only declared domains; ask for consent on first use; enforce HTTPS only; use the shared ureq agent; enforce size, time and rate limits; strip cookies; log requests in a per-plugin network log in the manager | P0 | M |
| 11 | **Clipboard** | Permission only | `clipboard.read/write` for text, image (RGBA f32 handle) and app-private MIME. Writes are allowed during a user-initiated event only. Reads need a grant | P1 | S |
| 12 | **Drag and drop** | None | `ui` node `drop_target` with accepted types. Dropped files arrive as picker-style handles (the PhotoCraft model). `drag_source` for plugin-made data | P2 | M |
| 13 | **File portals** | PhotoCraft only (`files.pick/read/write` handles); none in EffectCraft and FigCraft | Move PhotoCraft's handle model into the platform (`files.*` shared ops through `HostUi::pick_file`). Add a persistent "folder grant" for exporters | P1 | S |
| 14 | **Timers and animation frames** | None. Plugins must wait for user events | `timer.set {ms, repeat}` as a `timer` event (coalesced, minimum 16 ms, paused when hidden). `ui.animate` for host-driven tweening without a Wasm callback | P1 | S |
| 15 | **Events and subscriptions** | Any `app.events` grant receives everything; payloads differ per app; no panel open or close events, no `dispose` | `events.subscribe {names, filter}` with canonical payloads. Add `panel.opened/closed`, `dispose` and `document.saved` | P1 | S |
| 16 | **Inter-plugin services** | "Not available (beta deferral)" | `[[provides.service]]` in the manifest. `service.call {plugin, name, args}` runs in the provider's sandbox and transaction with the provider's own grants. The consumer declares `service:<id>/<name>`. Versioned with the same `name@major.minor` rule as contracts | P2 | M |
| 17 | **AI inference** | None. EffectCraft already links candle-core 0.9.2 (`crates/sam2`) | `ai.run {model, inputs}`. Models are data contributions (safetensors or ONNX) verified by sha256. The host runs them on candle/wgpu or the app device, and they count against the job budget. Start with host-blessed models (SAM2, depth, upscale) exposed as kernels (`kernel.segment`, `kernel.depth`) | P1 | L |
| 18 | **Fonts and themes** | Can be installed, not activated (`capabilities.md`) | Activate through the craft-framework font and theme library. Text rasterization as a kernel (`kernel.text`) so effects can draw type | P2 | M |
| 19 | **Localisation** | `t!` keys exist in forms (`guest/src/form.rs:126`). No runtime catalogue for plugin strings | `locales/<lang>.ftl` in the package. The host resolves `Word.id` for forms and UI trees. Add `ui.locale` to `init` | P2 | S |
| 20 | **Scripting and automation** | Three app CLIs with different syntax; no macros | A shared `craft-ext run` headless runner (see centralization). A macro recorder that logs op calls during an event as a replayable script. Batch "apply plugin to N files" through jobs | P1 | M |
| 21 | **Typed ABI** | `call(&str, Value)` JSON (`guest/src/host.rs:118`). Schemas only for 6 ops (`ext-host/src/schema.rs`) | Short term: generate typed Rust guest bindings and `.d.ts`/C headers from `OpTable` schemas (all ops, not 6). Long term: a WIT world `craft:plugin@2` (component model with wasmtime) with resources for documents, images and jobs; JSON ops remain for dynamic/app ops | P1 / P2 | M / L |
| 22 | **Capability negotiation** | Contract `major.minor` checks at load; ops that do not exist return `-3` | `host.capabilities` op returning ops, kernels, UI level and limits, so plugins degrade explicitly | P1 | S |
| 23 | **Other guest languages** | Rust only | The ABI is four imports plus `craft_alloc`/`craft_reset`/`craft_abi`, so C, Zig and AssemblyScript need only a 150-line header and a template. JS through QuickJS/javy needs a prebuilt JS runtime module (≈1 MB) plus the `craft` shim. Ship templates and conformance tests | P2 | M |
| 24 | **Raised budgets for host-call-heavy events** | 100 k fuel per call puts an event at ≈19 k calls (measured: 30 k → `TooSlow`) | Bill host calls by wall time or bytes rather than flat fuel. Add batch ops (`node.get {ids:[...]}`, `ops.batch`) | P1 | S |
| 25 | **Session memory** | 64 MiB (`MAX_SESSION_MEMORY`), smaller than two 1080p f32 frames | Make it per-contribution in the manifest (up to 512 MiB) with app-level accounting (already present for jobs) | P1 | S |
| 26 | **Native helper effects / hybrid** | Native cannot contribute effects | With a JIT, GPU shaders, inference and net, keep native for device/OS integrations only. Do not add native effects | P3 | — |

## Benchmarks

### Method

- Machine: Apple M5, 10 cores, macOS 25.5; Rust 1.97.1; release builds (`opt-level=3`). Guests
  were built for `wasm32-unknown-unknown` with LTO, `codegen-units=1` and `panic=abort`.
- Bench crates live in `bench/` with their own target dir:
  - `guest-bright`: a `craft_guest::filter!` that multiplies RGB by a gain. A `+simd128` build sits
    in `target-simd/`.
  - `guest-blur`: a filter calling `kernel::blur(r=20)`.
  - `guest-session`: `session!`, looping `craft_guest::call("layer.get", {"layer": i, "name": "Layer"})`.
  - `host`: the real `craft-ext-host` with its pinned wasmi features, plus raw wasmi with the same
    features.
  - `alt`: wasmi 2.0 default features plus `simd`, and wasmtime 50.0 (Cranelift and Winch).
- Image: 1920×1080 (and 3840×2160) RGBA f32, about 33 MB (133 MB).
- Each figure is the median of 5 timed runs after 1 warm-up. "Raw" times only the `craft_filter`
  call. "E2E" is `Filter::run`, including the copies in and out and output validation.
- Session: `Session::dispatch` of a command event with a mock `AppApi` and `Grants` that always
  allow. Per call = event time ÷ N.
- Kernels: `craft_ext_kernels::run` directly, including one image clone per run (0.6 ms at 1080p,
  2.5 ms at 4K).

### (a) Per-pixel loop: brightness

| Engine / config | 1080p ms | 4K ms | vs native (1080p) | Compile ms |
|---|---|---|---|---|
| Native Rust (same loop) | ≈1.06 | — | 1× | — |
| **Platform today**: `Filter::run` E2E (wasmi 2.0, portable dispatch, fuel) | **52.0** | **213.8** | 49× | 0.9 |
| Raw wasmi, portable dispatch, fuel | 42.1 | — | 40× | — |
| Raw wasmi, portable dispatch, no fuel | 37.9 | — | 36× | — |
| Raw wasmi 2.0, **tail-call dispatch**, fuel | **11.3** | 35.9 | 11× | 0.4–1.9 |
| Raw wasmi 2.0, tail-call, no fuel | 9.4 | 36.6 | 9× | 0.4 |
| Raw wasmi 2.0 tail-call, simd128 module | 9.5 | 36.5 | 9× | 0.4 |
| Wasmtime 50 Winch, no fuel | 3.6 | 14.7 | 3.4× | 6.6 |
| Wasmtime 50 Cranelift, epoch interruption | 1.29 | 5.1 | 1.2× | 40 |
| Wasmtime 50 Cranelift, fuel | 1.68 | 6.6 | 1.6× | 51 |
| **Wasmtime 50 Cranelift, no metering** | **0.96** | 3.6 | ≈0.9× | 34–42 |

The simd128 build ran no faster on any engine. The loop is memory-bound, and the SDK's
`chunks_exact_mut(4)` loop did not vectorize much. SIMD matters for compute-bound kernels
(convolution, colour transforms), not for this test. Platform wasmi refuses the simd128 module at
load.

### (b) `craft.call` round trip

| Measure | Result |
|---|---|
| Empty event (dispatch, begin/finish, reset, event JSON) | 0.008 ms |
| 1 call in an event | 20.8 µs |
| 1,000 calls | 10.46 ms, **10.5 µs per call, 95.6 k calls/s** |
| 10,000 calls | 104.8 ms, 10.5 µs per call |
| 30,000 calls in one event | **fails: `TooSlow`** (fuel: 100 k per call + 100 per byte vs 2e9 per event) |
| Native serde_json for the same encode/decode work (both directions) | 0.57 µs |

About 95% of the per-call cost is outside host JSON work. It is guest-side serde running in the
interpreter (args encoded, reply decoded) plus two extra boundary crossings (`call`, then `take`).
Tail-call dispatch or a JIT should cut this sharply (not measured here). A typed or binary ABI
helps further.

### (c) Host kernels

| Kernel | 1080p ms | 4K ms |
|---|---|---|
| `kernel.blur` gaussian r=20 (3 box passes) | 13.7 | 67.4 |
| `kernel.blur` gaussian r=5 (separable exact) | 9.2 | 37.1 |
| `kernel.blend` overlay, opacity 0.7 | 4.9 | 19.6 |
| `kernel.blend` normal | 4.6 | 18.2 |
| Wasm filter → `kernel.blur r=20`, E2E | 28.4 | 127.9 |

### Interpretation

- **Quick win: drop `portable-dispatch`.** It is one line and makes every plugin about 3.7×
  faster. Check that the deterministic feature still holds (dispatch does not affect results).
  Also enable the `simd` feature.
- **A JIT makes kernels optional rather than required.** Cranelift runs plain Rust loops at native
  speed, so authors can write their own algorithms. That is the "do almost anything" goal. Compile
  cost (35–50 ms for a 126 KB module) needs an on-disk artifact cache (wasmtime
  `Module::serialize`), with the cache key covering the wasmtime version, so it is paid once per
  install. Use epoch interruption for cancel and deadlines; it costs +34% on this tight loop,
  versus +75% for fuel.
- **Copies now cost as much as the compute.** At 1080p, 10 ms of `Filter::run` overhead is
  already 10× a Cranelift brightness pass, and half the Wasm-to-kernel blur time is conversion. Fix
  this before or together with the JIT, or the JIT gains disappear in memcpy.
- **Kernel quality.** Blend 4K at 18–20 ms works out to ≈2 ns per pixel over 8 threads. That is
  reasonable, but `blend_channel` matches the mode string per channel per pixel
  (`ext-kernels/src/lib.rs:414`); hoisting it to a function pointer is a cheap fix.
  `separable` clones the image twice. A GPU path (#6) would make blur r=20 at 4K about 1–2 ms
  (estimate).
- **For context:** the program doc cites Lightwell at 478 ms per 1080p frame and Softskin at
  602 ms. With tail-call dispatch, zero-copy and a JIT, those should drop to tens of ms (estimate,
  not measured).

## Author and agent experience

| Area | Today | Gap | Recommendation |
|---|---|---|---|
| Templates | `templates/{effect,panel,data}`; `craft-ext new <effect\|panel\|data>` | No command, tool, event-hook, job, multi-app or native templates (gap audit §14). No non-Rust templates | Add `new command/tool/hook/job/shader` plus C/Zig/AssemblyScript minimal templates. Each template ships a passing test |
| Typed API | `craft_guest::call("layer.get", json!(...))`, stringly typed | Op names and arg shapes are discovered from generated Markdown. Typos fail at runtime with `-3`/`-1` | Generate `craft_guest::ops::{effectcraft,photocraft,figcraft,common}` with typed args and results from `OpTable` schemas. Shared ops get one type |
| Mock host | Off-Wasm, `abi::call` returns `-7` (`guest/src/abi.rs:127-135`). Kernels run natively. `craft-ext test` covers effects only | Session plugins cannot be unit-tested | A `craft_guest::testing::MockHost` that records calls, returns scripted replies, and can load the real app contract (ops, permissions, schemas) to reject invalid calls. `craft-ext test` runs session scripts (`tests/*.jsonl`: events in, expected calls out) |
| Inner loop | `craft-ext dev` packs and installs into a dev store; restart the app with `CRAFT_EXT_DIR` | No watch, no hot reload, no in-app console (gap audit §14) | `craft-ext dev --watch`: rebuild on save and send a reload. The runtime already reinstantiates sessions on fault, so expose "reload plugin" and keep panel state. Add an in-app plugin console (log, op trace with timings, fuel, memory) |
| Errors | Codes `-1..-7` + message. `PluginError::Failed(code, "")` drops the guest message in filters (`wasm.rs:173,383,663,804`); hook errors dropped in PhotoCraft | Authors see "the plugin reported an error (-1): " with an empty message | Carry the guest's `Err(String)` through `craft.result` on failure. Include op, arg path and schema rule in `-1` messages (schema.rs does this for 6 ops) |
| Debugging | Logs only | No source-level debugging | wasmtime supports DWARF debugging (lldb) in dev mode. A `--trace-ops` JSONL trace makes replay possible |
| Determinism checks | `craft-ext test` runs effects twice and compares bits | Sessions untested. No golden images | Golden-image tests (`tests/golden/*.png` + tolerance) and session script replay. Both are deterministic and fit CI and agents |
| Docs for agents | Many docs (≈50 files), generated contract pages, `maps.md`, `coverage.md` | Spread out. Contract pages diverge from code (`capabilities.md` "Where the contract pages and the code differ"). Author guide contradicts capabilities on PhotoCraft references | One generated `ops.json` (machine-readable contract of all apps), checked in CI against `OpTable`, from which the Markdown is generated. A single "AGENT-START" page listing the 5 commands an agent needs (`new`, `build`, `check`, `test`, `dev --watch`) |
| Budgets | Fuel/memory limits documented | Authors can't see their usage | Report `fuel_used`, `peak_memory`, `host_calls` per event in the console and in `craft-ext test` output |
| Maintainers and agents | Three adapter crates to keep in sync; the behaviour divergences above crept in through copies | — | The `CraftApp` trait plus a shared conformance test suite (`craft_ext_runtime::conformance::run::<MyApp>()`) that every adapter must pass: transaction rollback, revocation mid-event, quarantine, data limits, events, jobs, missing plugin. Agents then change behaviour in one place |

## Sources

Shared platform (`craft-extensions`):
- `crates/ext-host/Cargo.toml` (wasmi features), `Cargo.lock` (wasmi 2.0.0)
- `crates/ext-host/src/api.rs` (AppApi, authorize), `wasm.rs` (engine config, limits, fuel,
  linker, Filter, Session, kernel and job calls), `schema.rs`, `data.rs`, `jobs.rs`, `policy.rs`
- `crates/ext-kernels/src/lib.rs` (rows/threads, blur, blend, cost)
- `crates/guest/src/{lib.rs, host.rs, abi.rs, filter.rs, kernel.rs, form.rs, session.rs}`
- `docs/plugins/{program-2026-11-17.md, compute-and-jobs.md, capabilities.md, author-guide.md,
  cli-reference.md, suite-replacement-rule.md, native-protocol.md}`,
  `research/gap-audit-2026-11-17.md`, `program-trail.tsv`
- wasmi 2.0.0 source (`build.rs`, `Cargo.toml` features `auto-dispatch`, `portable-dispatch`, `simd`)

Adapters:
- EffectCraft: `crates/plugins/src/{runtime.rs, ops.rs, effects.rs, contract.rs, grants.rs, lib.rs}`,
  `crates/engine/src/ext.rs`, `crates/ui-egui/src/plugins_ui.rs`, `crates/gpu/`, `crates/sam2/Cargo.toml`
- PhotoCraft: `crates/plugins/src/*`, `crates/engine/src/{plugin_ops.rs, plugin_sessions.rs, plugin_filters.rs}`,
  `crates/ui-egui/src/{plugins_ui.rs, plugin_form.rs, plugin_manager.rs}`
- FigCraft: `crates/plugins/src/{runtime.rs, contract.rs, grants.rs, lib.rs}`, `crates/ui-egui/src/plugins_ui.rs`

Benchmarks: `bench/host/src/main.rs`, `bench/alt/src/main.rs`,
`bench/guest-*/src/lib.rs`, raw results `bench/host-results.txt` and
`bench/alt-results.txt`.

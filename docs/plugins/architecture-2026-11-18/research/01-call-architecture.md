# 01 · Plugin call, ABI and permission architecture

Date: 2026-11-18. Author: research pass 1 of 8 (call/ABI/permission). Status: decision input. No repository
was edited, and no app repo was built. Benchmarks were measured on an Apple M5 (10 cores), Rust 1.97.1, in a
throwaway crate: `bench/calls`.

## 1. Summary and recommendation

**Question.** How should a sandboxed plugin call into ten-plus Craft apps so that it can do roughly what the
app's MCP server and CLI can do, under enforced permissions, quickly, and with one SDK?

**Recommendation: a hybrid, called "typed core plus classified registry bridge".** It has two call surfaces
behind one permission engine:

1. **Typed core (`craft:plugin@2`).** A small, hand-designed, cross-app vocabulary: documents, selection,
   node trees, properties, pixels and geometry handles, UI, events, jobs, files, net, settings and services.
   It is written in WIT, uses resources (handles), and is fast enough for per-object loops. The measured cost
   is 24–81 ns per call on Wasmtime, against 930 ns for today's JSON `craft.call`.
2. **Registry bridge (`app.run`, `app.batch`).** Every command in an app's existing `CommandSpec` registry is
   reachable automatically, with no hand-written ops. Exposure is decided by a **declared effects class on
   every command**, a required field, so a new command does not compile without one. One platform function
   maps the effects class to a permission or to refusal. CI pins the result in a per-app `api.lock.json` and
   proves it with generated effect, scope and undo tests. The ABI for this surface stays dynamic (a command
   id and bytes), because the command's own work dominates its cost. It becomes typed at the SDK layer
   through generated bindings, and a batch form makes many-object edits one host crossing and one undo step.

**Why this split.** The measurements decide most of it:

| Path (Wasmtime 50 Cranelift unless noted) | ns per op |
|---|---|
| Today's ABI: JSON `craft.call` + `take`, `serde_json::Value` on both sides | **930** |
| Same transport with generated serde structs instead of `Value` | **365** (2.5× better, no ABI change) |
| JSON `ops.batch` of 1000 in one crossing | 970 (batching JSON buys nothing: the cost is serde, not crossings) |
| Component model typed call, `record` with a `string` | **77–81** |
| Component model resource method (`layer.set-opacity`) | **52** |
| Component model `list<op>` batch, 1000 edits | **12.6** |
| Core-wasm typed import, scalar | 4 |
| wasmi 2.0.1 tail-call, same JSON call | 3,900 |
| Today's platform, measured earlier (wasmi portable dispatch plus fuel plus full Session path) | 10,500 |

- **JSON is fine for breadth and wrong for loops.** A registry command costs microseconds to milliseconds
  inside the app (schema check, journal, snapshot), so a 0.4–0.9 µs envelope is noise there. A plugin
  reading 100 k node properties pays 93 ms in JSON and 5–8 ms typed.
- **Batching only helps a typed encoding.** The 1000-op JSON batch costs the same per op as 1000 single
  calls. The typed `list<op>` batch is 12.6 ns per op.
- **The component model's overhead over raw core-wasm imports (10–50 ns per call) is negligible** next to
  any real host work. It buys resources and handle lifetimes, multi-language bindings, semver'd packages,
  and the async machinery that jobs, network and permission prompts need.

**The hard part is metadata, not ABI.** Only VectorCraft has machine schemas: 12 commands carry a
`vectorcraft_apischema::ParamSchema` with a `Writes` class
(`vectorcraft/crates/apischema/src/lib.rs:98`, `engine/src/schemas.rs`). Every other registry has free-text
`params: &'static str`, and undo is declared five different ways:

- `history` (FigCraft);
- `journal` (EffectCraft, PhotoCraft);
- `journal + undoable` (DesignCraft);
- `journal + undoable + view` (SoundCraft);
- nothing at all (MaterialCraft, TextureCraft).

Neither typed bindings nor safe automatic exposure is possible until each command declares a schema and its
effects. That work is the critical path for every option, including option 2 and option 3, so it should
start first: promote VectorCraft's `apischema` crate to a shared crate and make `effects` a required field.

**Engine consequence.** The typed core should be authored in WIT and run as a real component on Wasmtime.
The interpreter path does not need component support from wasmi. A `wit-bindgen` guest is a plain core
module before componentization, with canonical-ABI imports plus a `component-type` custom section; this was
verified on the bench guest (§4.4). The Wasmtime host componentizes it at install (0.6–1.4 ms measured). An
interpreter host can link the same imports directly through a generated lowering for our restricted WIT
subset. So one plugin artifact serves both engines.

Pulley, Wasmtime's portable interpreter, also runs components, but it was about 3× slower than wasmi on
guest-heavy JSON code (§4).

## 2. What the code says today (evidence)

| Fact | Evidence |
|---|---|
| Plugins call one import `craft.call(op, op_len, args, args_len) -> i64` and read the reply with `craft.take`. Args and reply are JSON `serde_json::Value` on both sides | `craft-extensions/crates/guest/src/abi.rs:80-120`, `guest/src/host.rs:118-140` |
| The adapter seam is `AppApi { contract, begin, call(inv, op, Value), finish }`. Authorization is `authorize_call`: schema check (only 6 shared ops have schemas), preflight (cancel, stopped, grant generation), then `OpSpec::requires(args)` against declared and granted permissions with `covers` | `crates/ext-host/src/api.rs:166-225`, `ext-host/src/schema.rs` |
| The permission vocabulary is closed and `family:scope`: `document.read/edit:{selection,document,all}`, `files.read/write:picked`, `network:<host>`, `clipboard.read/write`, `ui.window`, `app.events`. "Never grantable" things have no spelling | `crates/ext-core/src/permission.rs:1-140` |
| FigCraft exposes engine commands to plugins through two **hand-maintained allowlists**, `EDIT_COMMANDS` and `QUERY_COMMANDS`, with one permission each. `selection.set`, `page.select` and `viewport.set` are classed Query even though they change state, the read-only-guard bug the platform review found | `figcraft/crates/plugins/src/contract.rs:24-37,59,79-91` |
| VectorCraft's script engine uses a **default-allow prefix deny-list** (`refused_to_scripts`): `script.`, `plugin.`, `prefs.`, `print.`, `library.`, `edit.undoGroup.` plus 13 ids. A command added tomorrow under a new prefix is allowed by default | `vectorcraft/crates/engine/src/script/mod.rs:146-178` |
| VectorCraft's `apischema` already has the right shape: typed params (`Id`, `Ids`, `Color`, `Point`, `Enum`, nested `Object`), `since`/`deprecated` API levels, a `cost`, a `Writes {Nothing, Session, Preferences, Document}` class, JSON Schema 2020-12 export for MCP, `validate_saved` for old recorded steps, and a per-API-level schema hash pinned by a test | `vectorcraft/crates/apischema/src/lib.rs:1-240`, `engine/src/schemas.rs:1-60` |
| Registries carry inconsistent undo and effect flags: `history` (FigCraft); `journal` (EffectCraft, PhotoCraft); `journal, undoable` (DesignCraft); `journal, explicit_targets` (LightCraft); `journal, named` (FilmCraft); `journal, undoable, view` (SoundCraft); none (MaterialCraft, TextureCraft) | each app's `crates/engine/src/cmd.rs` or `commands.rs` |
| EffectCraft's `Session::execute` already routes plugin commands, scripts (`execute_for_script`), agents (`execute_checked`, unknown params refused) and the UI through one function. It has nesting so inner commands are not journaled, and a rule that scripts may not change the settings that limit scripts | `effectcraft/crates/engine/src/session/dispatch.rs:9-80` |
| Command counts: EffectCraft 435+, FigCraft about 630, VectorCraft about 888, DesignCraft about 635, LightCraft about 325, TextureCraft about 80, MaterialCraft about 50. Plugin-reachable today: about 37, about 74, 0, 0, 0, 0 and 0 respectively | `craft-extensions/docs/plugins/research/capability-program-2026-11-18.md` §1.4 |

The direction is already set: "one generated, permission-classified bridge over these registries"
(capability program §2.3) and a WIT world as item D8. This report decides the shape of each and how they fit
together.

## 3. Options compared

### 3.1 The five options

1. **Hand-written SDK ops over JSON `craft.call`.** This is today's design: every op is hand-written per
   adapter in an `OpTable`/`AppContract`.
2. **Registry bridge.** Expose each app's native command registry (`app.run {command, params}`) with
   generated per-command permission classes. The analogues:
   - Photoshop UXP `batchPlay` (descriptors, `executeAsModal` for state changes, `dialogOptions: "silent"`,
     `continueOnError`);
   - Blender `bpy.ops` (operators with `poll()` and `bl_options = {'REGISTER','UNDO'}`);
   - VS Code `commands.executeCommand`.
3. **Typed WIT / component model generated from the registries.** Analogues: Zed extensions (Wasmtime
   `bindgen!`, versioned `since_vX` WIT directories), Fermyon Spin, Extism (bytes in and out, host functions),
   proxy-wasm (scalar ABI plus buffers), and Shopify Functions (moved from JSON stdin to a lazy NaN-boxed
   import ABI, `shopify_function_v2`).
4. **Object-capability, handle-based.** Analogues:
   - WASI preview 2 resources with no ambient authority;
   - Deno `--allow-net=host` and `Deno.permissions.request`;
   - Chrome MV3 `permissions` / `optional_permissions` / `host_permissions` and `activeTab`, a temporary
     grant on user gesture;
   - Figma `networkAccess.allowedDomains` with `reasoning`, and `documentAccess: "dynamic-page"`;
   - Obsidian, which has no sandbox;
   - VS Code proposed API gating (`enabledApiProposals`, refused for Marketplace);
   - JetBrains, which has no sandbox and relies on review.
5. **Hybrid.** A typed core plus generated typed bindings over the registry for breadth.

### 3.2 Matrix

Scores: ++ strong, + adequate, − weak, −− disqualifying for the owner's goal. "Breadth" means the share of
app capability reachable without hand-writing per-command glue.

| Criterion | 1 JSON ops (today) | 2 Registry bridge | 3 WIT generated from registries | 4 Pure ocap/handles | 5 Hybrid (recommended) |
|---|---|---|---|---|---|
| Breadth | −− (37/435 in EffectCraft; adapters drift) | ++ (all classified commands, automatically) | ++ in theory, but needs a schema on every command first | − (handles alone cover only modelled objects) | ++ |
| Cross-app authoring | − (6 ops identical in all three apps) | − (per-app ids such as `layer.duplicate` and `node.duplicate`) | − (each app's WIT differs) | + (one object model) | ++ (shared core for portable code; per-app namespaces for breadth) |
| Call overhead (measured) | 930 ns JIT, 3.9 µs wasmi, 10.5 µs today | Same envelope as 1, but command work dominates | 24–81 ns | 52 ns per resource method | 24–81 ns core; about 0.4–0.9 µs per bridge call; 12.6–68 ns per item in batches |
| Many-object edits | − (no batch; fuel caps an event at about 19 k calls) | + with `app.batch` and one undo step | ++ (`list<T>`) | + | ++ |
| Versioning over years | − (hand-written docs drift: `capabilities.md` vs code) | + if API levels, schema hashes and deprecation are generated (VectorCraft pattern) | − (a WIT package bump per app release; hundreds of interfaces to keep loading) | + (small surface) | ++ (small, slow-moving WIT core; fast-moving registry versioned by API level and lock file) |
| Permission granularity | Per op, hand-coded `requires` | Per command class, generated; per object via typed `Id`/`Ids` params | Per function | Per object/handle, attenuable | Per command class, per object scope, per handle attenuation |
| Safe automatic exposure of new commands | n/a (nothing is automatic) | Only with **default-deny** classification plus CI; VectorCraft's deny-list shows the failure mode | Same requirement | n/a | Required effects field (compile-time) plus lock file plus generated probes |
| Auditability | Hand-written `permission_rule` text | Generated `api.lock.json` diffable per release | Generated | Handle tables are runtime state, hard to audit statically | Lock file plus per-plugin op trace |
| Undo/transactions | Per-adapter, three behaviours | Must suppress per-command undo and journal inside a plugin event (Blender's lesson) | Same | Same | One platform transaction state machine for both surfaces |
| Engines | wasmi and Wasmtime | Any | Component model needs Wasmtime (or a lowering on wasmi) | Same as 3 | Core in WIT on Wasmtime; generated core-ABI lowering keeps wasmi viable |
| Author ergonomics | Stringly typed; typos fail at runtime with `-3`/`-1` | Stringly typed unless bindings are generated | Fully typed, but huge generated APIs | Pleasant (objects with methods) | Typed core with objects; typed generated breadth modules |
| Agents (MCP) sharing | Separate surfaces | Same registry; classes become MCP tool annotations | Same | n/a | Same registry and classes for both |

### 3.3 Notes per option

**Option 1, JSON ops (today).** It works and is simple, but it cannot meet "plugins can do what the app
can". Every op is hand-written in each adapter, so coverage stays a small fraction of the registry and
diverges between apps. The platform review found:

- `selection.get` has three shapes;
- `layer.select` needs edit in EffectCraft and read in PhotoCraft;
- FigCraft `selection.set` is classed Query.

The transport itself can be kept. Generated serde structs cut its cost 2.5× without any ABI change
(365 ns against 930 ns).

**Option 2, registry bridge.** This is the right way to get breadth. Every incumbent with scripting breadth
has one:

- `batchPlay` ([Adobe](https://developer.adobe.com/photoshop/uxp/2022/ps_reference/media/batchplay/));
- `bpy.ops`;
- `executeCommand`.

Each of those has well-known failure modes, and the design must avoid them:

- **Ambient breadth with no classification.** UXP's batchPlay can run any descriptor once the plugin is
  loaded. The UXP manifest's `requiredPermissions` (network domains, `localFileSystem`, `launchProcess`,
  `clipboard`, `ipc`) gate platform services, not Photoshop commands. The batchPlay page itself describes
  no command-level permission model, only `executeAsModal`, `dialogOptions` and developer-mode gates.
- **Default-allow lists rot.** VectorCraft's `refused_to_scripts` is a prefix deny-list. Any new command
  family is allowed until someone remembers to add it.
- **Per-call undo and redraw in loops.** Blender's documentation steers scripts away from operators in
  loops because each operator call does context checks, a possible undo push and a scene update.
- **Modal UI during automation.** UXP defaults `dialogOptions` to "silent" for exactly this reason.

**Option 3, typed WIT generated from registries.** Typed per-command WIT makes registry churn ABI churn:

- 888 VectorCraft commands become hundreds of WIT functions;
- every app release potentially publishes a new package version;
- the host must keep instantiating all old ones, as Zed does with `since_v0.0.1` … `since_v0.x` directories
  and per-version wrappers
  ([Zed blog](https://zed.dev/blog/zed-decoded-extensions)).

The speed gain, about 1 µs per call, is invisible when the command takes 10 µs–10 ms. Type safety can be
had at the SDK layer through codegen without freezing the ABI. Typed WIT is the right tool for a **small,
stable, hot** surface, not for the registry.

**Option 4, pure object capabilities.** Handles give the best granularity:

- per object;
- attenuable (read-only or selection-only views);
- time-limited (dropped at transaction end).

Chrome's `activeTab` shows how a user gesture can grant scoped, temporary access without an install-time
prompt
([Chrome docs](https://developer.chrome.com/docs/extensions/develop/concepts/activeTab)).

As the only model, though, ocap fails breadth: you would have to model every app concept as a resource
type. It also fails auditability, because "who can reach what" becomes a runtime graph, not a reviewable
table. Use ocap where objects matter (documents, nodes, images, jobs, files, sockets, service calls) and
classes where commands matter.

**Option 5, hybrid.** This takes the strengths of 2, 3 and 4, and every incumbent that scaled ended up here:

- **Photoshop:** the DOM API, with `batchPlay` for the rest.
- **Figma:** a typed node API, plus `networkAccess` and `permissions` in the manifest
  ([Figma manifest](https://developers.figma.com/docs/plugins/manifest/)).
- **Blender:** the `bpy.data`/`bpy.types` typed data API, with `bpy.ops` for breadth.
- **VS Code:** the typed `vscode` API, `executeCommand` for breadth, and proposed APIs gated by
  `enabledApiProposals`.

The cost is two surfaces. The design keeps one permission engine and one transaction model under both, so
authors see one SDK.

## 4. Benchmarks

### 4.1 Method

- Machine: Apple M5, 10 cores, macOS 25.5. Rust 1.97.1. Release builds; guest LTO, `codegen-units=1`,
  `panic=abort`, target `wasm32-unknown-unknown`.
- Guests:
  - `guest-core` is a core module importing `craft.call`/`craft.take` with exactly today's signatures
    (`guest/src/abi.rs`), plus typed imports `bench.{noop,set_opacity,get_layer,batch}`;
  - `guest-comp` is a `wit-bindgen 0.57.1` guest of `wit/bench.wit`, componentized at load by
    `wit-component 0.247`.
- Host: Wasmtime 50.0.2 (Cranelift, `epoch_interruption(true)` as the platform plan proposes, deadline far
  away). wasmi 2.0.1 with default features, which use tail-call dispatch; today's platform pins
  `portable-dispatch`. Wasmtime Pulley (`pulley64`) was used as an interpreter comparison.
- Workload:
  - the host holds 1000 layers (`id: u64, name: String, visible: bool, opacity: f32`);
  - `layer.get` returns one layer (the reply carries a string) and `set_opacity` validates and writes;
  - every path performs a permission check (a grant-table lookup through `black_box`), and the JSON path
    does an op-name hash lookup, as `authorize` does;
  - the JSON host parses into `serde_json::Value`, as `AppApi::call` does, builds a `json!` reply,
    serializes it to a pending buffer and copies it out on `take`.
- Timing: each figure is the median of 9 timed runs after 1 warm-up, N = 10,000 single calls or one batch
  of 1,000. Three full repetitions agreed within ±8% (the worst case was the fixed-struct get, 26–37 ns).
  The one-shot "export call" figure is at `Instant` resolution (about 41 ns) and is an upper bound.
- Build cost: a cold Wasmtime build with `component-model` took about 85 s, plus 55 s for a rebuild with
  serde, at `CARGO_BUILD_JOBS=4` under `nice`. That is well under the 10-minute limit. Component compile at
  load was 30–34 ms for a 100 KB component. Componentize (encode) took 0.6–1.4 ms.

Sources: `arch-bench/{wit/bench.wit, guest-core/src/lib.rs, guest-comp/src/lib.rs, host/src/main.rs}`.
Raw output: `arch-bench/results-run{1,2,3}.txt` and `results-pulley.txt`.

### 4.2 Results (run 3; ns per operation)

| # | Path | Wasmtime Cranelift | wasmi 2.0.1 tail-call | Wasmtime Pulley |
|---|---|---|---|---|
| a1 | **JSON `craft.call`+`take`, `layer.get` (Value both sides)** | **931** | **3,921** | 11,702 |
| a2 | JSON `craft.call`, `set_opacity` (reply `null`) | 424 | 1,376 | 4,075 |
| a3 | JSON with generated serde structs (both sides), `layer.get` | 366 | 1,484 | 4,811 |
| a4 | JSON string over a component import (`call(string,string)`) | 896 | – | 10,298 |
| b1 | Core-wasm noop import | 1.5 | 8.5 | 28 |
| b2 | Core-wasm typed scalar `set_opacity(i32,f32)->i32` | 4.0 | 12.7 | 52 |
| b3 | Core-wasm typed `get_layer` into a fixed 72-byte struct | 26 | 35 | 189 |
| b4 | Component noop import | 15 | – | 47 |
| b5 | **Component typed `set-opacity(u32,f32) -> result<_,string>`** | **24** | – | 130 |
| b6 | **Component typed `get-layer -> result<record{…string…}>`** | **81** | – | 456 |
| b7 | **Component resource method `layer.set-opacity`** | **52** | – | 161 |
| b8 | Component `open-layer` + method + drop (handle churn) | 136 | – | 393 |
| c1 | **JSON `ops.batch`, 1000 × `layer.get`, one crossing** | **971** | 4,063 | 14,774 |
| c2 | JSON `ops.batch`, 1000 × set | 678 | 2,750 | 10,609 |
| c3 | **Component `batch(list<op>) -> list<reply>`, 1000 × get** | **68** | – | 533 |
| c4 | Component `batch(list<op>)`, 1000 × set | **12.6** | – | 113 |
| c5 | Core-wasm binary batch, 1000 × get into fixed structs | 20 | 27 | 77 |
| n1 | Native Rust: the same JSON encode, host parse, reply and decode (no Wasm) | 671–765 | | |
| n2 | Native Rust: direct typed `set_opacity` | 0.5 | | |
| ref | Today's platform `craft.call` (wasmi portable dispatch, fuel, full `Session::dispatch`), from `platform.md` §(b) | 10,500 | | |

### 4.3 What the numbers mean

1. **JSON's cost is serde, not the boundary.** Under Cranelift, a1 (931 ns) is about 1.3× the native cost
   of the same encode and decode (n1). The crossing itself is about 1.5 ns (b1).
   - So `ops.batch` over JSON (c1) gives **no** per-op win.
   - Batching only pays with a typed or binary encoding: c3 and c4 are 14–75× cheaper per op than c1.
   - The platform target "≤ 1 µs typed binary ABI" (program §3) is met by any typed option: 12–81 ns.
2. **Generated typed bindings over the same JSON transport are a free 2.5×** (a3 against a1). They also
   remove the `Value` tree from guest code, which matters most on interpreters: 1.48 µs against 3.92 µs on
   wasmi. This can ship before any engine change.
3. **The component model costs 10–60 ns per call more than hand-rolled core imports** (b5 against b2,
   b6 against b3). The extra comes from canonical-ABI lifting, `cabi_realloc` for strings and lists, and
   resource-table lookups (b7). It is the price of strings, records, variants, lists, results and handles
   being generated and safe. At 50–80 ns, a plugin can do about 12–20 M typed calls per second, so this is
   never the bottleneck.
4. **Handles are cheap to use and moderately cheap to create** (b8 is 136 ns including drop). Per-object
   handles for every node in a 100 k-node document cost about 14 ms. That is acceptable, but the API should
   use id lists and columnar reads for bulk work, with handles for documents, images, jobs and long-lived
   objects.
5. **Engine choice matters more than ABI for guest-side work.** The same JSON call costs 931 ns (Cranelift),
   3.9 µs (wasmi tail-call), 10.5 µs (today's platform config) and 11.7 µs (Pulley). Pulley runs components,
   but it is about 3× slower than wasmi for guest-heavy code. wasmi stays the better interpreter, which is
   why the design keeps a component-free lowering for it (§5.2).
6. **The real per-command cost is inside the app.** Not measured here, because app repos must not be
   built: `Session::execute` runs param checks, the enabled predicate, journal, snapshot or copy-on-write
   (`Arc::make_mut(&mut self.project)` in EffectCraft) and the command. Expect µs to ms. The registry
   bridge therefore should not chase ABI nanoseconds; it should avoid per-command undo pushes, journal
   writes, change events and redraws inside a plugin transaction (§5.7). That was Blender's lesson.

### 4.4 Verified: one artifact can serve both engines

`python3 -I arch-bench/imports.py guest_comp.wasm` on the `wit-bindgen` output, before componentization,
shows a plain core module:

```
craft:bench/doc@0.1.0 :: get-layer (i32,i32) -> ()
craft:bench/doc@0.1.0 :: [method]layer.set-opacity (i32,f32,i32) -> ()
craft:bench/doc@0.1.0 :: [resource-drop]layer (i32) -> ()
craft:bench/doc@0.1.0 :: batch (i32,i32,i32) -> ()
custom: component-type:wit-bindgen:0.57.1:craft:bench@0.1.0:plugin:encoded world
```

The Wasmtime host turns this into a component in 0.6–1.4 ms (`wit_component::ComponentEncoder`). A wasmi
host can link these canonical-ABI imports directly as core functions:

- flattened scalars;
- a pointer and length for strings and lists;
- a return pointer for compound results;
- an `i32` for resource handles;
- guest `cabi_realloc` for host-to-guest allocation.

A generator for our restricted WIT subset (no futures or streams on wasmi) is a bounded job. The
[`wasm_component_layer`](https://crates.io/crates/wasm_component_layer) crate is prior art for running
components on non-Wasmtime backends; it was not evaluated here.

## 5. Recommended design

### 5.1 Layers

```
 plugin code (Rust / C / Zig / JS via componentize-js / AssemblyScript)
   ├─ craft SDK  ── typed core: craft::doc, craft::ui, craft::events, craft::job, craft::files, craft::net …
   │             └─ generated per-app modules: craft::app::figcraft::{node::duplicate(..), ..}  (typed params)
   │                               │ encodes to
   ▼                               ▼
 ABI  craft:plugin@2 (WIT)  ─ typed imports + resources;  app.run / app.batch carry (command-id, bytes)
   ▼
 Runtime<A: CraftApp>  (craft-extensions, one copy)
   ├─ Gate: preflight → schema validate → classify → permission (declared ∩ granted ∩ invocation) → scope check
   ├─ Txn: one transaction per event; snapshot/restore/commit; nested-command suppression; change coalescing
   ├─ Jobs, events, UI, files/net portals, services, budgets (bytes + wall time), op trace
   ▼
 Adapter (per app, thin)  → CommandRegistry (the app's CommandSpec + Effects + ParamSchema)  → Session::execute
                          ← also driven by MCP server, CLI, scripts (same registry, same classes, other principals)
```

Rules, each made deterministic by a check (§5.6):

- One authorization path for both surfaces, and for Wasm and native helpers. Today's
  `api::authorize_call` stays the single gate.
- The adapter never classifies, authorizes or snapshots. It maps typed core functions to engine calls and
  exposes `registry()`.
- The registry is the single source for MCP tools, CLI help, scripts, plugin bindings and docs.

### 5.2 ABI

**v1 (keep):** `craft.log`, `craft.call`, `craft.take`, `craft.result`, `craft_alloc`, `craft_reset`. ABI v1
modules keep loading for the deprecation window, and the host answers new ops itself, per the existing
AGENTS rule.

**v1.5 (Phase 1, no engine change):**

- `app.run {command, params}`, `app.batch {steps, on_error}` and `app.commands {prefix?}` as host-answered
  ops on `craft.call`.
- Generated typed SDK bindings: serde structs for every exposed command and every core op, giving a 2.5×
  gain (a3).
- `craft.call_buf(op_id, in_ptr, in_len, out_ptr, out_cap) -> i64`, with `op_id` interned once through
  `craft.op_id(name)`, so hot paths skip string lookup and the second `take` crossing.
- A binary `nodes.get {ids, fields}` / `nodes.set` in columnar layout for bulk work (c5 is 20 ns per item).

**v2 (Phase 2, with Wasmtime on desktop):** the WIT world `craft:plugin@2`:

```wit
package craft:plugin@2.0.0;

interface types {
  type node-id = u64;
  enum scope { selection, document, all }
  variant error { invalid(string), denied(permission-needed), not-found(string), stale, cancelled, budget(string), internal(string) }
  record permission-needed { permission: string, reason: string }
}

interface doc {
  use types.{node-id, scope, error};
  resource document {
    id: func() -> string;
    revision: func() -> u64;
    selection: func() -> list<node-id>;
    set-selection: func(ids: list<node-id>) -> result<_, error>;
    children: func(parent: option<node-id>) -> result<list<node-id>, error>;
    get: func(ids: list<node-id>, fields: list<string>) -> result<list<u8>, error>;     // columnar, binary
    set: func(ids: list<node-id>, field: string, values: list<u8>) -> result<_, error>;
    image: func(target: node-id, region: rect) -> result<image, error>;
    read-only: func() -> document;                                                       // attenuation
  }
  resource image { size: func() -> tuple<u32,u32>; read: func(region: rect, into: list<u8>) -> result<_, error>; write: func(region: rect, px: list<u8>) -> result<_, error>; }
  active: func() -> option<document>;
}

interface registry {
  use types.{error};
  record command-info { id: string, since: u32, class: string, permissions: list<string>, schema-hash: u64 }
  commands: func(prefix: option<string>) -> list<command-info>;
  run: func(command: string, params: list<u8>) -> result<list<u8>, error>;              // JSON params/result
  record step { command: string, params: list<u8> }
  enum on-error { rollback, stop, continue }
  batch: func(steps: list<step>, on-error: on-error) -> result<list<result<list<u8>, error>>, error>;
}

world plugin {
  import types; import doc; import registry; import ui; import events; import job; import files; import net;
  import settings; import secrets; import service; import host;
  export handle: func(event: event) -> result<_, string>;
}
```

The listing is illustrative. Function sets are owned by the other research tracks (UI, jobs, pixels).

Choices in v2:

- **Registry calls stay `list<u8>` (JSON) in the ABI.** Breadth churn never changes the WIT, and the SDK
  generates typed wrappers. JSON is kept because it is the registry's native form, already validated by
  `ParamSchema`, shared with MCP, and readable in traces.
- **Typed core functions are batch-shaped** (`get(ids, fields)`, not `get(id)`). The measured per-call cost
  is 24–81 ns, but the host-side work per call, such as locking the document owner, dominates, so lists
  amortize it.
- **Resources for things with lifetimes and authority:** `document`, `image`, `job`, `file`, `request`,
  `panel`, and the attenuated views (`read-only`, selection-scoped). Nodes stay `u64` ids authorized by
  scope (§5.5), because node ids must persist in plugin data and documents.
- **Engines:**
  - Wasmtime Cranelift on desktop, with components;
  - Pulley where no JIT is allowed (components, slower);
  - wasmi with the generated canonical-ABI lowering for web and as the reference;
  - ABI v1 JSON plugins on all three during the deprecation window.

### 5.3 One source of truth per command

Promote `vectorcraft_apischema` to a shared crate. It "knows no command and depends on no other VectorCraft
crate" (`apischema/src/lib.rs:20`). Proposed home: `craft-libs/crates/craft-apischema`. Extend it with
`Effects`, and make both `schema` and `effects` **required** fields of every app's `CommandSpec`:

```rust
pub struct Effects {
    pub reads: Reach,          // Nothing | Target | Document | Documents | App
    pub writes: Writes,        // Nothing | View | Session | Document | Documents | Preferences | Library | External
    pub target: Target,        // None | Params(&'static str /* the Id/Ids param */) | Selection | Document | App
    pub io: Io,                // bitflags: PICK_FILE, PATH, NETWORK, CLIPBOARD_READ, CLIPBOARD_WRITE, PROCESS, PRINT, SYSTEM_FONTS
    pub ui: Ui,                // None | SilentOrModal (params replace the dialog) | ModalOnly | Window
    pub history: History,      // Step | Coalesced | None | Manages  (Manages = undo/redo/clearHistory/undoGroup)
    pub duration: Duration,    // Instant | Long  (Long: may exceed 50 ms on a large document; runs as a job when called by a plugin)
    pub spawns_jobs: bool,
    pub control: bool,         // configures automation itself: plugin.*, script.*, scripting prefs, grants, shortcuts that run code
}
```

- `Writes` subsumes `history`, `journal`, `undoable` and `view` (they become derived values). The table
  that maps today's flags is a one-off migration tool. Its output is then reviewed and checked in, because
  `journal: false` does not imply "reads only", as FigCraft's `selection.set` shows.
- `Type` gains `Path` (a filesystem path) and `File` (a portal handle). A `Path` param makes a command
  unreachable from plugins unless the app provides a `File` twin. The runtime resolves `File` through
  `HostUi::pick_file` handles, so a plugin never sees or forges a path.
- `ParamSchema.returns` becomes a result schema (it is free text today), so result types can be generated
  too.

**Interim for apps without schemas.** A command whose schema is `ParamSchema::UNDECLARED` is **MCP and CLI
only**: never plugin-exposed, and never in bindings. This gives automatic availability without a hole. A
command becomes available to plugins on the release in which its author writes the schema and effects. That
is a two-line change next to the command, not a change in the adapter.

### 5.4 Binding generation

```
<app>-cli api export            → target/api/<app>.api.json  (commands, ParamSchema→JSON Schema, Effects, exposure, permissions, since, hash)
craft-ext api lock --app <app>  → <app>/crates/plugins/api.lock.json   (checked in; reviewed)
craft-ext bindgen               → crates/guest/src/apps/<app>.rs, sdk/ts/<app>.d.ts, sdk/c/<app>.h, docs/plugins/apps/<app>.md, MCP tool annotations
```

- One generated Rust module per app: `craft::app::vectorcraft::object::align(&Align { ids, to, .. })`. The
  types come from `Type` (Id→`NodeId`, Ids→`Vec<NodeId>`, Enum→a Rust enum, Point→`[f64;2]`,
  Object→struct), and `since` becomes `#[doc]` plus a runtime check against `host.api_level(app)`.
  Deprecated params get `#[deprecated]`.
- Shared core bindings come from `craft:plugin@2` through `wit-bindgen`. For v1.5 they come from `OpTable`
  schemas, as program item D1 proposes.
- `.d.ts` and C headers come from the same `api.json`. This makes the "simple for authors" claim
  language-neutral.
- MCP gets tool annotations generated from `Effects`, matching the MCP spec's `readOnlyHint`,
  `destructiveHint`, `idempotentHint` and `openWorldHint`. Agents and plugins then read the same truth.

### 5.5 Permission model

**Where it is enforced:** only in `Runtime<A>::gate`, one function, before the adapter sees the call. The
order is:

1. Preflight: cancel, stopped, grant generation. This exists today.
2. Schema validation: `ParamSchema::validate`, with unknown keys refused.
3. Classify: `Effects` → `Exposure`, from the lock file at load time, so it costs one table lookup per call.
4. Required permissions: `requires(effects, params)` must be covered by **declared ∩ granted ∩ invocation
   context**.
5. Scope check: every `Id`/`Ids` param resolves inside the scope set.
6. Budget charge: bytes plus a cost unit from `ParamSchema.cost`.
7. Dispatch.

**Who enforces:** the platform. Adapters cannot express permissions. A conformance test calls every exposed
command with each insufficient grant and expects `-2`.

**Granularity:**

- per command class;
- per object scope through typed ids;
- per handle through attenuation;
- per invocation context (user gesture, automatic, agent).

#### Vocabulary (final; extends `permission.rs` without renaming anything)

| Permission | Scopes | Grants | Prompt |
|---|---|---|---|
| `document.read` | `selection`, `document`, `all` | Read commands and core reads within the scope | At first use, or covered by invocation (below) |
| `document.edit` | `selection`, `document`, `all` | `Writes::Document` commands with targets in scope; covers read | Same |
| `document.create` | – | New and duplicate documents, and closing documents the plugin created | Yes |
| `app.view` | – | `Writes::View`: viewport, zoom, active tool, panel visibility, switching the active document | Free, but declared and listed |
| `app.session` | – | `Writes::Session`: app state outside the document and preferences (guides visibility, proof settings, workspace) | Yes, low-risk wording |
| `app.events` | optional family: `app.events:document`, `:selection`, `:time`, `:tool`, `:history` | Subscriptions | Yes (exists) |
| `app.automation` | – | Edits **without** a user gesture (event hooks, timers, `document.changed` follow-ups). Required in addition to `document.edit` | Yes, with a strong sentence: "Can change your document automatically when …" |
| `app.intercept` | `<hook-point>` (`before-save`, `before-export`, `input.filter`, `tool:<id>`) | Participation in app-declared hook points, including veto or replacement | Yes, and a per-contribution on/off toggle in the manager |
| `files.read`, `files.write` | `picked` | Portal handles | Yes (exists) |
| `files.folder` | `picked` | A persistent folder handle (exporters) | Yes |
| `network` | `<host>` or `*.<domain>` | `net.fetch` to those hosts only | Yes (exists) |
| `clipboard.read`, `clipboard.write` | – | Clipboard; writes only during a user gesture | Read yes, write declared (exists) |
| `ui.window` | – | Floating windows | Yes (exists) |
| `secrets` | – | The plugin's own keychain namespace | Yes |
| `compute.nn` | – | Model download and inference | Yes, showing the download size |
| `service` | `<plugin-id>/<name>` | Calling another plugin's declared service | Yes |
| `handoff` | `<app>` | Cross-app open with a return path | Yes |

**Free (not permissions):** compute, kernels, logging, private settings, own plugin data in documents,
panels, menu/command/shortcut contributions, own forms, timers that do not edit, `host.capabilities`,
`app.commands`.

**Never grantable (no spelling; classified `Refused`):**

- processes and native code;
- raw sockets;
- raw paths;
- app preferences (v1);
- `History::Manages`;
- `control: true` commands: plugin and script management, settings that limit scripts or plugins, grants;
- `io: PRINT | SYSTEM_FONTS` (until a portal exists);
- the app's own network features (`io: NETWORK`; v1 refuses them, because they are a covert exfiltration
  channel);
- other plugins' data.

#### Scopes, defined precisely

- **`selection`:** the selection captured at event start, plus the descendants of selected containers, plus
  objects created by this transaction. A `set-selection` inside the event does **not** widen the scope;
  otherwise `select.all` plus an edit would escalate. Selection-scoped plugins may change the selection only
  to subsets of the scope, plus created objects.
- **`document`:** the document the event targets (`Invocation.document`), not the active one at call time.
  A user switching tabs mid-event yields `-4 stale`.
- **`all`:** every open document.
- **Scope enforcement is generic** because params are typed. The runtime walks every `Type::Id`/`Ids` in
  the validated params and checks membership. Commands with `Target::Selection` (operate on the current
  selection when no ids are given) are in scope by construction. `Target::Document` needs document scope,
  and `Target::App` needs `app.session` or is refused.

#### Grant durations and invocation context (attenuation in time)

- **Durations:** grant once (this invocation), for this session, or always, as browsers offer. Grants are
  recorded with a generation; revocation mid-event already fails the next call (`api.rs` preflight).
- **Invocation grant (the `activeTab` analogue).** When the user explicitly runs a plugin's command (menu,
  palette, shortcut, button), the runtime may treat the declared `document.read/edit` at `selection` scope
  as granted for that one event, with no prompt. Background runs (events, timers, jobs delivering results,
  agent calls) never get invocation grants. This removes most prompts for the common "select, run plugin"
  flow without widening anything persistent. Owner decision (§8).
- **Principals:** user gesture, automatic, agent (MCP invoking a plugin command), and service (another
  plugin). A plugin command called by an agent runs with the plugin's persistent grants only. The agent's
  own trust never flows into the plugin.

#### Handles and attenuation (where object capabilities apply)

- `document.read-only()` returns a handle whose writes return `-2` whatever the plugin's grants. Plugins
  pass it to a service so the provider can read but not write the consumer's document.
- `job` handles snapshot a read view of the document at start. They cannot write. Results come back
  through a new transaction with a revision check.
- `file` and `folder` handles come only from portals. They are unforgeable `i32` resource handles in v2 and
  opaque tokens in v1.
- All handles die at the end of their scope (event, job, or session for folder grants). The resource table
  is per instance, and the runtime drops it on reset.

#### Auditability

- `api.lock.json` is the static answer to "what can a plugin with permission P do in app X at API level N".
- The per-plugin op trace already proposed (D4/D9) records `(event, principal, command, class, permission
  used, scope size, duration)`. The manager shows a per-plugin "what it did" log next to the network log.

### 5.6 Registry classification rules and CI checks

**Classification** is one platform function, `fn exposure(spec) -> Exposure`, unit-tested, with the first
matching rule winning:

| # | Rule | Exposure |
|---|---|---|
| 1 | `schema` is `UNDECLARED`, or `effects` is `UNCLASSIFIED` | `AgentOnly` (MCP and CLI; not plugins) |
| 2 | `effects.control` | `Refused("controls automation")` |
| 3 | `io` has `PROCESS`, `PRINT` or `SYSTEM_FONTS`, or `writes` is `Preferences`, `Library` or `External` | `Refused` |
| 4 | `history == Manages` | `Refused` |
| 5 | `io` has `PATH` and the schema has no `File` twin | `Refused("paths")` |
| 6 | `ui == ModalOnly` or `Window` | `Refused("needs app UI")` |
| 7 | `io` has `NETWORK` (the app's own) | `Refused` (v1) |
| 8 | Command id is contributed by a plugin (`plugin_command`) | `Refused` via the bridge; use `service.call` |
| 9 | Otherwise | `Plugin(perms)`, with perms derived below |

Derived permissions:

- `reads` maps to `document.read:<scope from target>`.
- `writes` maps as follows: Document → `document.edit:<scope>`; Documents → `document.edit:all`;
  View → `app.view`; Session → `app.session`.
- `io` adds `PICK_FILE` → `files.*:picked` (served by the portal), `CLIPBOARD_*` → `clipboard.*`.
- `duration == Long` or `spawns_jobs` → callable only through `job.run_command` (§5.7).
- `ui == SilentOrModal` → the bridge always runs silent. Required params are required, so no dialog ever
  opens.

**Checks (deterministic, in each app's CI; all fail closed):**

| Check | What it proves | How |
|---|---|---|
| C0 compile | Every command declares schema and effects | Non-`Option` fields on `CommandSpec`; the `UNDECLARED` constants exist only for rule 1 |
| C1 lock drift | Exposure equals the reviewed table | `craft-ext api lock --check`: regenerates `api.lock.json` and fails on any diff |
| C2 widening | No silent widening | The diff tool lists widenings (AgentOnly/Refused→Plugin, weaker permission, broader target, new `io` bit cleared). CI fails unless `api-widening.log` gains one line per widened id, and CODEOWNERS on the lock file and log requires a security owner |
| C3 effects probe | Declared effects are true | Engine IO goes through an `Env` trait (fs, net, prefs, clipboard, dialogs, process, fonts). For each exposed command, a headless session runs it with schema-generated params (seeded, deterministic) under a recording `Env`. Recorded effects must be ⊆ declared, and a recorded document write under `Writes::Nothing` fails |
| C4 scope | Typed-id scope enforcement holds | For each exposed command with `Id`/`Ids`: a run under `document.edit:selection` with an out-of-selection id must return `-2`, and the document hash must be unchanged |
| C5 undo | Plugin transactions are atomic | Run each exposed edit inside a plugin transaction: rollback restores the document hash, commit adds exactly one undo step, the journal gains nothing, and at most one coalesced `document.changed` is emitted |
| C6 refusal table | Refused classes never expose | Property test over all `Effects` combinations against `exposure()` |
| C7 schema levels | Compatibility | VectorCraft's `SCHEMA_HASHES` pattern per app: a schema change without an `API_VERSION` bump fails. Removing or renaming an exposed command without a `deprecated` level ≥ 2 levels earlier fails |
| C8 shared core conformance | Cross-app sameness | `conformance::run::<A>()` (program §2.2) over `craft:plugin@2` |
| C9 bindings fresh | Generated SDK equals the lock | `craft-ext bindgen --check` |

C3 is the expensive one, because it needs the `Env` seam in each engine. It can roll out command family by
command family. Until a family is probed, the lock keeps it AgentOnly unless it is hand-reviewed.

### 5.7 Transactions, undo, batches and long work

- **One plugin event is one transaction and one undo step**, labelled by the contribution. The
  platform-owned `Txn` state machine (program §2.2) wraps both the typed core and `app.run`.
- **Inside a transaction, the bridge calls `Session::execute` in nested mode.** EffectCraft has
  `replay::Nesting`, and VectorCraft has `depth`. In nested mode:
  - no journal entries are written (the plugin event records one replayable step);
  - no per-command undo pushes happen;
  - change events are coalesced to one `document.changed {revision, changes[]}` at commit;
  - no redraw happens until commit.

  This is the Blender lesson made structural. C5 tests it.
- **`app.batch(steps, on_error)`:**
  - every step is schema-checked before the first one runs (VectorCraft's `command.batch` already does this);
  - steps run in order;
  - `rollback`, the default, restores the transaction snapshot on the first failure;
  - `stop` keeps earlier steps;
  - `continue` records per-step errors, like UXP `continueOnError`;
  - budget is charged per step cost, not per call (program R7).
- **Bulk edits stay fast because of the ABI and the app together:**
  - most registry commands already take `ids` lists, so one step covers N objects;
  - the typed core's columnar `get` and `set` covers property sweeps (c3, c5: 12–68 ns per item);
  - snapshots are copy-on-write once per transaction.
- **Long commands (`Duration::Long`, `spawns_jobs`)** are never run synchronously on the owner thread for a
  plugin:
  1. `job.run_command {command, params}` snapshots the inputs and runs on a worker where the engine allows,
     or as a cancellable sliced task on the owner thread where it doesn't;
  2. it shows progress in the shared jobs list;
  3. it delivers the result as a new transaction that checks the document revision (`-4 stale` if the user
     edited meanwhile, and the plugin decides).

  The plugin can cancel. A deadline uses epoch interruption.
- **Modal app operations** (dialogs, interactive tools) are never opened by the bridge (rule 6 and silent
  mode). A plugin that wants input shows its own form, then calls the command with explicit params.

### 5.8 Versioning

| Surface | Unit | Rule |
|---|---|---|
| ABI v1 (`craft.call`) | `craft_abi` export | Kept for at least 12 months after v2 ships, or two app majors. Host-answered new ops keep v1 modules working. Removal criteria are recorded in the bridge's owner note (suite replacement rule) |
| `craft:plugin@2` WIT | semver package | Additive minors only (new functions, new variant cases behind `host.capabilities`). A major means a new `since_v3` directory and a host wrapper per supported major, as Zed does. The target is one major every few years |
| Registry per app | `API_VERSION` integer (VectorCraft) | A manifest declares `[requires] vectorcraft = { api = ">=4" }`. Commands and params carry `since` and `deprecated`. Old params are accepted through `validate_saved` semantics plus shims next to the command (VectorCraft already keeps `effect.apply {id}` as an older key). Removed commands return `-3` with "removed at level N; use X" |
| Shared vocabulary semantics | conformance suite | A shared op or type changes only with all adapters passing C8 |
| Generated SDK | crate version = (core WIT version, per-app API level) | `craft-ext check` warns when a plugin calls a command whose `since` exceeds its declared minimum and the call isn't guarded by `host.api_level` |

### 5.9 One plugin, several apps

- **The shared core is the portable layer.** Code written only against `craft::doc`, `craft::ui` and
  `craft::events` runs in every app that provides the core. `host.capabilities()` says which optional
  interfaces exist (for example `doc.image` in vector apps that rasterize).
- **Per-app namespaces are explicit.** `craft::app::figcraft::*` compiles everywhere, but calls return
  `NotProvided` outside FigCraft. The SDK offers `match craft::host::app() { App::FigCraft(f) => …,
  App::VectorCraft(v) => … }` so authors branch once.
- **Shared vocabulary governance** follows the existing rule in `permission.rs`: "a new family needs a
  second app that wants it". A concept moves into the core once two apps implement it with one schema and
  pass C8. Until then it stays namespaced. No cross-app alias tables in the bridge, because aliases hide
  semantic differences.
- **The manifest** lists `apps = [...]` and per-app contributions. The store shows per-app availability
  from the lock files.

### 5.10 "Change app behaviour" plugins

Express behaviour as **declared contributions placed by the host**, never as imperative patching:

| Intent | Mechanism | Safety |
|---|---|---|
| Add commands, menus, shortcuts, palette entries | `[[contributes.command]]` with menu path and shortcut. The host merges them into the app's dynamic command overlay (PhotoCraft needs one, program §6), so they appear in menus, palette, Actions and MCP | Free. Shortcut conflicts are resolved by the host (user binding wins) |
| Add panels or windows | `[[contributes.panel]]`; `ui.window` for floating windows | Panels free; windows need permission |
| Add a tool | `[[contributes.tool]]` receiving pointer events with pen fidelity; edits go through the same `Txn` per gesture | `document.edit` scope applies |
| Replace a built-in tool or brush engine | `[[contributes.tool]] replaces = "tool.brush"` | Needs `app.intercept:tool:tool.brush` plus a user toggle. Safe mode and the manager can revert it. The built-in stays one click away |
| Auto-run on events | `[[contributes.hook]] on = "document.saved"`. The handler runs as a deferred follow-up event (H4), with loop detection: a hook's own changes don't retrigger it, and the depth limit is 1 | Edits need `app.automation` plus `document.edit` |
| Veto or transform at app points | App-declared hook points only (`before-save`, `before-export`, `input.filter`, `preflight.rule`), each with a deadline (for example 200 ms) and fail-open or fail-closed declared by the app | `app.intercept:<point>`. No generic "wrap any command" |
| Change defaults or preferences | Not grantable in v1. Plugins ship presets, themes and workspaces as data contributions the user applies | – |

## 6. Edge cases

| Case | Handling |
|---|---|
| A plugin calls a command that opens a file dialog | Effects mark `io: PICK_FILE`. The bridge runs it through the portal: the runtime calls `HostUi::pick_file` outside the transaction (the event yields), then re-enters with a `File` handle. Requires `files.*:picked`. Commands that take a `Path` are refused unless they have a `File` twin |
| A command hits the network (font download, cloud library, AI service) | `io: NETWORK` is refused for plugins in v1 (exfiltration via app-owned endpoints). Plugins use `net.fetch` with their own `network:<host>`. Later, individually reviewed commands may move to Plugin with `network:<that host>` derived from the schema |
| Side effects outside the document (prefs, app state, other windows) | `writes: Preferences` is refused. `Session` needs `app.session`. `View` needs `app.view`. C3 proves the declarations with a recording `Env` |
| Commands that spawn jobs | `spawns_jobs` or `Long` means plugins go through `job.run_command`. The job's result transaction checks the revision. Jobs hold read snapshots, never write handles |
| Recursion: plugin A's command triggers plugin B | Plugin-contributed commands are refused through `app.run` (rule 8). Cross-plugin calls go through `service.call` (declared, versioned, run in B's sandbox with B's grants). Depth limit 2. Re-entering a busy instance returns `-4`. Cycles are detected by the invocation chain in `Invocation` |
| Agent (MCP) calls a plugin command, which calls the registry | The plugin runs as principal "agent-invoked": no invocation grant, only persistent grants. The registry call is gated as usual. The agent cannot grant on the plugin's behalf |
| Long-running modal operation (content-aware fill, trace) | `Duration::Long` means a job with progress and cancel. The owner thread is never blocked longer than the slice budget. Epoch deadline plus cancel flag |
| Schema changes between app versions | API level per app; `since`/`deprecated`; `validate_saved` accepts old keys the command still reads; shims live next to the command; removed commands give a helpful `-3`. Generated bindings carry the level, and `craft-ext check` flags unguarded newer calls |
| MCP agents and plugins sharing one registry | One `Effects` and schema set feeds both. MCP gets tool annotations. Plugins get exposure through the lock. A command can be AgentOnly (rule 1) while it waits for classification, so agents never lose breadth |
| Permission prompt mid-operation | Never block inside a transaction holding the document. A call needing an ungranted permission returns `-2 {permission, reason}`, and the transaction rolls back unless the plugin handles it. `permissions.request([...])` is allowed only in a user-gesture event; it ends the event, shows the sheet, then delivers `permissions.changed`, and the plugin re-runs its action. Grant generation changes mid-event already fail closed |
| A plugin edits in response to `document.changed` | Needs `app.automation`. Runs as a deferred follow-up event. Changes caused by the hook are tagged and don't re-trigger it. A per-plugin rate limit applies (for example 10 automatic transactions per second, then quarantine strike) |
| Selection changes during a selection-scoped event | Scope is the captured set plus created objects. Calling `select.all` doesn't widen it |
| Undo/redo inside a plugin | Refused (`History::Manages`). The plugin's own transaction is the undo unit. "Undo my last plugin action" is the user's normal undo |
| Batch with 10 k steps | Schema pre-check of all steps. One snapshot. Budget by cost units. Progress and cancel checked between steps. If it exceeds the slice, the plugin is told to use `job.run_command` (or the runtime slices it with a stale check) |
| The app adds a command next release | It cannot compile without `schema` and `effects`. Without a real schema it is AgentOnly. With one, C1 shows the lock diff, C2 forces a widening entry and owner review, C3–C5 run automatically, and bindings regenerate |
| Command result contains paths or secrets (`file.info`, `links.list`) | Results also have schemas. `Type::Path` in a **result** is redacted to a portal handle or the basename unless the plugin holds `files.read` for that handle. C3 records result types |
| Commands that are reads in name but mutate (FigCraft `selection.set`, `page.select`) | Effects are declared on the command, not inferred from `history`/`journal`. C3 catches a write under `Writes::Nothing` |
| A plugin replaces a tool and crashes | Quarantine (three strikes) disables the contribution and the host restores the built-in. Safe mode starts with no replacements |
| Native helper plugins | Same gate (`authorize_call` is already shared). Same classification. A native helper's OS-level access is outside this model and covered by consent (existing native protocol) |

## 7. Migration from today

| Step | Work | Where | Removal criteria for the old path |
|---|---|---|---|
| M0 | Generated serde-struct bindings for today's ops over `craft.call` (a3: 2.5×). Drop `portable-dispatch` (R1) | `crates/guest`, `ext-host` | – |
| M1 | Promote `apischema` to `craft-libs/craft-apischema`. Add `Effects` and the `File`/`Path` types | craft-libs, VectorCraft first | – |
| M2 | Per app: make `schema` and `effects` required. Run a one-off importer that maps `history`/`journal`/`undoable`/`view` and the FigCraft allowlists and VectorCraft deny-list into `Effects`, emitting `UNDECLARED` where unsure. Review the output and check it in | each app engine | FigCraft `EDIT_COMMANDS`/`QUERY_COMMANDS` and VectorCraft `refused_to_scripts` are deleted once their apps' locks pass C1–C6 (scripts then use the same `exposure()` with a "script" principal) |
| M3 | `Runtime<A>::gate` plus `app.run`, `app.batch` and `app.commands` on ABI v1. `api.lock.json`, C1, C2, C6, C7 in CI. FigCraft first (it is already command-based), then EffectCraft, then PhotoCraft | craft-extensions plus adapters | Hand-written adapter ops that duplicate a registry command are deleted when the bridge passes C4 and C5 for it |
| M4 | `Env` seams and C3 probes, family by family. Families move from AgentOnly to Plugin as they pass | app engines | – |
| M5 | Wasmtime engine on desktop (R4). The `craft:plugin@2` core WIT, componentize at install, and the wasmi lowering generator. SDK v2 on `wit-bindgen` | craft-extensions | ABI v1 is deprecated at v2 GA and removed after the window in §5.8 |
| M6 | New adapters (FilmCraft, VectorCraft, DesignCraft, LightCraft, MaterialCraft, TextureCraft, SoundCraft) start directly on `Runtime<A>` plus the bridge, so day-one breadth equals their classified registry | each app | VectorCraft legacy `crates/plugins` is removed per the suite replacement rule |

## 8. Open decisions for the owner

| # | Decision | Options | Recommendation |
|---|---|---|---|
| D1 | Should the ABI for registry commands be typed per command (WIT per command) or dynamic? | Typed ABI / dynamic ABI with typed SDK | **Dynamic ABI, typed SDK.** Registry churn must not become ABI churn. The cost difference (about 0.4–0.9 µs) is below command cost |
| D2 | Typed core in WIT on the component model? | WIT and components / own core-wasm IDL | **WIT.** It is the industry IDL (Zed, Spin, WASI) with multi-language guests. 24–81 ns measured. The wasmi lowering keeps the interpreter path |
| D3 | Unschematized commands | Expose with coarse permission / AgentOnly | **AgentOnly** until a schema and effects exist. That is default-deny with no lost agent breadth |
| D4 | Invocation grants (`activeTab`-like) | Off / on for selection scope / on for document scope | **On for selection scope**, for explicit user invocations only. Background and agent runs never get them |
| D5 | App-owned network commands for plugins | Refuse / allow with derived `network:` | **Refuse in v1.** Revisit per command |
| D6 | Preferences writes | Refuse / `app.preferences` permission | **Refuse in v1.** Ship presets and workspaces as data |
| D7 | Tool and behaviour replacement | Not allowed / `app.intercept` plus user toggle | **`app.intercept` plus toggle**, only at app-declared hook points |
| D8 | wasmi long-term | Keep (web, reference) / Pulley only | **Keep wasmi as the interpreter.** Pulley measured about 3× slower on guest code. Re-measure when a web or iOS target actually ships; none exists in the four app trees inspected |
| D9 | ABI v1 deprecation window | 6 / 12 / 24 months | **12 months or two app majors**, whichever is longer, with automatic v1→v2 SDK migration notes |
| D10 | Who reviews `api.lock.json` widenings | App owner / security owner / both | **A security owner via CODEOWNERS**, with the app owner as second. The lock is the security boundary (program decision 3) |

## 9. Confidence and limits

- **Measured here:** every number in §4, and the core-module import shape of a `wit-bindgen` guest (§4.4).
- **Read in source, not run:** registry fields, the FigCraft allowlists, the VectorCraft deny-list and
  apischema, EffectCraft dispatch and nesting, the permission vocabulary, and `authorize_call`.
- **Not measured:**
  - in-app cost of `Session::execute` per command;
  - the effects-probe feasibility per engine (it needs an `Env` seam that most engines lack today);
  - the wasmi canonical-ABI lowering, which is designed but not prototyped;
  - `wasm_component_layer`.
- **External claims, from the cited pages:**
  - UXP batchPlay modal, silent-dialog and continue-on-error behaviour;
  - Figma `networkAccess`, `permissions` and `enableProposedApi` (not allowed in published plugins);
  - VS Code proposed APIs (Insiders only, not for Marketplace);
  - Zed versioned WIT on Wasmtime;
  - Chrome `activeTab` semantics;
  - Shopify Functions v2 import ABI;
  - Blender permissions being declarative only, as recorded in
    `craft-extensions/docs/plugins/research/blender-and-comfyui.md:12`.

  Shopify instruction and input limits could not be confirmed from official pages and are not used.

## 10. Sources

- Craft source:
  - `craft-extensions/crates/{ext-host/src/api.rs, ext-core/src/permission.rs, guest/src/abi.rs, guest/src/host.rs}`
  - `figcraft/crates/{engine/src/cmd.rs, plugins/src/contract.rs}`
  - `vectorcraft/crates/{apischema/src/lib.rs, engine/src/cmd.rs, engine/src/schemas.rs, engine/src/session/execute.rs, engine/src/script/mod.rs}`
  - `effectcraft/crates/engine/src/{cmd.rs, session/dispatch.rs}`
  - `{photocraft,designcraft,lightcraft,soundcraft}/crates/engine/src/cmd.rs`
  - `{filmcraft,materialcraft,texturecraft}/crates/engine/src/commands.rs`
- Craft docs: `craft-extensions/docs/plugins/research/capability-program-2026-11-18.md`,
  `research/capability-2026-11-18/platform.md`, `research/blender-and-comfyui.md`, `suite-replacement-rule.md`.
- Photoshop UXP batchPlay: https://developer.adobe.com/photoshop/uxp/2022/ps_reference/media/batchplay/
- Photoshop UXP manifest v5 permissions: https://developer.adobe.com/photoshop/uxp/2022/guides/uxp_guide/uxp-misc/manifest-v5/
- Figma plugin manifest: https://developers.figma.com/docs/plugins/manifest/
- VS Code proposed API: https://code.visualstudio.com/api/advanced-topics/using-proposed-api
- Zed extensions on Wasmtime and WIT: https://zed.dev/blog/zed-decoded-extensions
- Chrome `activeTab`: https://developer.chrome.com/docs/extensions/develop/concepts/activeTab
- Shopify Functions Wasm API (v2 import ABI): https://shopify.dev/docs/apps/build/functions/programming-languages/webassembly-for-functions
- Wasmtime `bindgen!` and the component model: https://docs.rs/wasmtime/latest/wasmtime/component/macro.bindgen.html, https://component-model.bytecodealliance.org/
- WASI 0.2 resources: https://github.com/WebAssembly/WASI/tree/main/wasip2
- MCP tool annotations: https://modelcontextprotocol.io/specification/2025-06-18/server/tools
- Blender extension manifest: https://docs.blender.org/manual/en/latest/advanced/extensions/getting_started.html
- Deno permissions: https://docs.deno.com/runtime/fundamentals/security/
- Fermyon Spin manifest (`allowed_outbound_hosts`): https://developer.fermyon.com/spin/v3/manifest-reference
- Extism host functions: https://extism.org/docs/concepts/host-functions
- proxy-wasm ABI: https://github.com/proxy-wasm/spec

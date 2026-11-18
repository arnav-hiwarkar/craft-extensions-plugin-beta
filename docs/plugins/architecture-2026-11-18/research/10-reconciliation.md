# 10 · Reconciliation: one plugin architecture for the Craft suite

Date: 2026-11-18. Role: chief architect's reconciliation review of reports 01–08 and the prior plan
`craft-extensions/docs/plugins/research/capability-program-2026-11-18.md` (called **the program**).
Status: decision input. Read-only: nothing was edited, built, committed or pushed, and nothing under the
`storytold` org was contacted.

**What was checked in source for this review** (all under `plugin-beta`):

| Claim in dispute | What the source says |
|---|---|
| AniCraft has no source (program §6) | `anicraft/crates/{plugins,engine,gpu,…}` exist; `Cargo.toml:59,66` pins eframe/egui **0.36** |
| wgpu version per app | `Cargo.lock` resolves `wgpu 30.0.1` in AniCraft, EffectCraft, MaterialCraft and SoundCraft (and per 03, all others) |
| wasmi features | `craft-extensions/crates/ext-host/Cargo.toml:18`: `std, validate, deterministic, portable-dispatch` |
| VectorCraft typed schemas (12 / 32 / 8) | `crates/engine/src/cmd.rs:279-284` macro arm; `schema: &…` used at about 9 commands (`cmd/views.rs`, `cmd/apiref.rs`). **8–10**, not 12 or 32 |
| LightCraft ONNX coverage | `lightcraft/crates/denoise/src/onnx.rs:1-6`: Conv, LeakyRelu, ConvTranspose, MaxPool, Concat, DepthToSpace only |
| TextureCraft tolerant decoding | 0 occurrences of `serde(other)` in `texturecraft/crates/core/src` |
| EffectCraft plugin panels | `effectcraft/crates/ui-egui/src/plugins_ui.rs:126` draws every surface with `renderer.window(…)`: floating |
| SoundCraft panic handling | `soundcraft/crates/playback/src/callback.rs:57-58`: one `catch_unwind` sets `failed` for the whole stream |
| Command totals | Census lexer output `census/all_apps.csv`: 6,193 rows; per app as in 08 §1 |
| Manifest spelling | `craft-extensions/examples/color-notes/craft-plugin.toml`: `schema = 2`, `apps`, `[why]`, `[[contribution]]` with `kind`, `contract`, `permissions` |
| Repos and branches | Beta forks on `plugin-beta`: craft-extensions, effectcraft, photocraft, figcraft, craft-launcher. The other eight apps and craft-libs sit on `main` tracking upstream |

---

## 0. Verdict in brief

The eight reports agree on the shape and disagree on four load-bearing points:

1. **Where JIT code runs.** Report 02 forbids Cranelift code in the app process. Reports 01, 03 and 07
   measured and designed for in-process Wasmtime, including on SoundCraft's audio thread. I side
   with 02 and change the data plane and audio designs to fit it.
2. **Which IDL.** Report 01 wants a WIT core on the component model. Once JIT code lives in a worker,
   every host call crosses a process anyway, so WIT's 24–81 ns advantage disappears. I keep one schema
   source (the shared command metadata crate) and generate typed bindings from it. WIT is deferred.
3. **Where command metadata lives.** Five proposals, one answer: a shared `craft-command` crate in
   craft-libs, with mandatory `Meta` next to each command in each app, and a generated, reviewed lock
   file. No hand-kept bridge tables.
4. **GPU and AI bases.** App device by default, chosen by where the data lives. candle, not LightCraft's
   six-op ONNX reader, is the AI base, with a generic-ONNX spike still owed.

The program's phase plan is not buildable as written: its Phase 1 alone is two to three times its
stated four to eight weeks, and it puts UI contribution points, plugin data and the registry bridge
before their prerequisites. Section 5 gives a dependency-ordered plan.

---

## 1. Contradiction register

"Cost" is what the chosen position costs compared with the rejected one.

### 1.1 Facts

| # | Topic | Positions | Evidence | Resolved position | Cost |
|---|---|---|---|---|---|
| F1 | Command counts | Program: EC 435+, VC ~888, MC ~50, TC ~80. 05: EC 691, FilmCraft 619. 06: PC ~809, VC ~1,034+290, DC 567+194, LC 391+213. 07: MC 133, TC 183, SC 389. 08: **6,193** (EC 757, PC 954, VC 1,044, FilmCraft 686, FigCraft 630, DC 567, SC 483, LC 391, AC 365, TC 183, MC 133) | 08 uses a lexer that also counts `fe!`/`chan!`/`fast!`/`res!` (EC) and `flag!`/`toggle!`/`option!`/`radio!` (SC: 389 `cmd!` + 94 helpers); it validated against AniCraft's docs (365/365) and VC's reviewed list (718/718). 05 counted only `cmd!`/`query!` | **Adopt 08.** Plus about 1,000 shell/UI commands that also need `Meta` | Metadata backfill is about 7,200 declarations, not ~4,000 |
| F2 | Typed parameter schemas | 01: 12 VC commands. 06: 32. 08: 8 | Verified 8–10 `schema:` sites | **About 9 of 6,193.** Conclusion unchanged: schemas are the critical path | none |
| F3 | Adapter reach | Program: EC ~37, FigCraft ~74, PC ~40 ops. 05: EC 35. 06: FigCraft 84, PC 37. 08: EC 28 registry ids of 757, PC ≈106, FigCraft 72 | Op names and registry ids are different units | **Use 08's registry-id reach** (3.7%, 11%, 11%). Op names are a vanity metric | none |
| F4 | AniCraft | Program: empty clone, no plugin system | Verified: 365 commands, MCP, own wasmi ABI v1 with four kinds (05, 08), eframe 0.36 | **Migration, not greenfield.** AC needs a v1 bridge with removal criteria and an egui 0.37 bump | +M for the bridge; saves designing AC from zero |
| F5 | Competing systems | Program: only VectorCraft | 05, 06, 07, 08 | **Three plugin systems to replace** (VC legacy, ~12.3k lines; AC ABI v1; SC in-process CLAP/VST3/AU hosting moves out of process). **Four automation engines stay** (EC `script`/ScriptUI, VC `crates/js`, DC step runner, PC Actions) but must classify through the same `Meta` | Each automation engine needs a caller audit and a shared-classification change |
| F6 | Docking | Continuation: EC and PC panels float. 04/06: PC docks | Verified EC floats; 06 cites PC `panels/registry.rs:260-307` | **EC floats, PC docks, FigCraft has its own dock** | none |
| F7 | MaterialCraft extensibility | Program: closed Rust enums | 07: native catalogue is a closed `OnceLock` of `fn` pointers, but the visible library is 549 SBS packages loaded from a folder; missing nodes are kept as placeholders | **The graph tier already works as data.** `resource.node-library@1` is the first MC plugin type, before any code tier | Brings MC forward in the order |
| F8 | SoundCraft engine | Program: fixed 512 blocks, parallel strips | 07: variable blocks ≤512, bounce 1024, strips serial on the audio thread, plugin params per block | **Adopt 07.** Bounce ≠ playback for automated plugin params today; SC-2/SC-3 come before any Wasm audio | Audio tier starts later |
| F9 | Cranelift compile time | Program: 34–42 ms. 03: 8.9–11.5 ms | 03 enabled `parallel-compilation` | **Adopt 03**, with the feature on | none |
| F10 | AI runtime base | Program K8, 03 §6.2, 07 CL-9: lift LightCraft's ONNX reader + WGSL convs as `craft-nn` | Verified six op types | **candle is the base; LC's WGSL convs are a fast path**, not the runtime (06) | A generic-ONNX spike is still needed (D22) |
| F11 | Manifest spelling | 03: `[[contributes.effect]]`; 04: `[[contributions]]` | Shipping format is `[[contribution]]` | **Keep the shipping format**; add per-kind placement tables to it | none |

### 1.2 Design choices

| # | Topic | Positions | Resolved position and reasoning | Cost |
|---|---|---|---|---|
| X1 | **Where JIT code runs** | 01 and 03 benchmark and design in-process Wasmtime; 07 SC-5 runs Wasm on SoundCraft's audio thread; 02 D1: worker process, mandatory, never in-process as a fallback | **Out-of-process `craft-plugin-worker`, one per package version, OS-confined.** Cranelift bugs are real (CVE-2023-26489 CVSS 9.9; 2026 advisories where a guest aborts the host). An abort in-process takes the user's unsaved document with it, so the worker is crash isolation as well as security. Every precedent that runs untrusted compiled code at scale does this (Chrome, AUv3, FxPlug 4, Bitwig). The data plane stays zero-copy through shared-memory arenas (X8). **Owner decision** | Host calls pay an IPC hop: target ≤5 µs p50 against 0.08–0.9 µs in-process. A chatty plugin doing 100 k single reads pays 0.5 s instead of 93 ms (01's JSON figure), so the core API must be batch-shaped. About 6–10 weeks of transport and per-OS confinement work |
| X2 | Engines | Program: wasmi default + Wasmtime desktop; 01: keep wasmi, add a WIT lowering; 03: Wasmtime only on desktop, browser engine on web, wasmi for iOS and CI; 04: wasmi on web | **Desktop: Wasmtime+Cranelift in the worker, for every contribution kind.** wasmi stays as the deterministic reference in CI and as the in-process fallback where a worker cannot be confined (old kernels, some Flatpak setups). No browser backend and no Pulley until a web or iOS plugin target exists; none ships today (03 §1: no wasm32 code in `ext-host`). No Winch (03 D3) | Authors never choose an engine. Dropping the dual-engine idea loses wasmi's in-process call latency for panels; IPC cost is comparable to wasmi's 3.9 µs JSON call anyway |
| X3 | **IDL and ABI** | 01: ABI v1 JSON now, WIT `craft:plugin@2` on the component model later, with a generated wasmi lowering. 02: WIT only with Craft-only interfaces, no async. 03: control plane JSON, data plane views; canonical ABI copies lists. 07: no-JSON struct ABI for audio | **One schema source: `craft-command`'s `ParamSchema`** describes registry commands *and* platform ops. From it, generate Rust bindings first, then TypeScript (AssemblyScript), C and docs. Transport: `craft.call` JSON for control (kept for ABI v1 compatibility), `craft.call_buf` with the same schema in a compact binary encoding for bulk structured data, and views/handles/rings for media. Hot values (`param`, `time`, `cancelled`) are worker-local reads from the call descriptor (1.8 ns, 03 F). **WIT deferred**, revisit at Phase 6. **Owner decision** | No free `wit-bindgen` multi-language guests; we maintain our own generator (01 already proposed one for `.d.ts` and C). Avoids a second IDL, the unprototyped wasmi lowering, and component-model async advisories |
| X4 | **Where command metadata lives** | 01: required `schema`+`effects` on `CommandSpec`, apischema promoted to craft-libs. 05: bitset + JSON Schema string on `CommandSpec`. 06: reviewed `bridge.toml` per app. 07: `craft-command` crate with schemars. 08: `Meta` in craft-libs with an `effects` list and `exposure` | **`craft-libs/crates/command` (`craft-command`)**: `Meta` (08's effect list, scope, undo, run mode, destructive, since), `ParamSchema` moved from VC `apischema`, a `Registry` trait, the platform's `exposure()` function (01 §5.6), MCP schema generation, and the CI test helper. Metadata sits on each `CommandSpec` in the app repo and the `cmd!` macros require it. The bridge allowlist is **generated** into `api.lock.json` in the app repo and reviewed by a security CODEOWNER (01 + 06) | Upstream-visible change to 11 apps' `cmd.rs`. Rejects schemars (would mean typed parameter structs for 6,193 commands that take `&Value` today) and hand-kept tables (they drift: PC's gate still names a removed `plugin.install`, 08 §5.1) |
| X5 | Effects encoding | 01: struct of axes (`reads`, `writes`, `io` bits, `ui`, `history`, `duration`, `control`). 05: class + bitset. 08: list of `Effect` values | **08's effect list plus 01's derivation rules.** One list per command is what an agent can review in a diff. `Scope`, `Undo` and `RunMode` stay separate fields | 01's `control: bool` becomes the `PluginMgmt`/`PrefsWrite` effects |
| X6 | Commands without metadata | 01: AgentOnly. 02/06: refused by default in a reviewed table | **AgentOnly**: MCP and CLI keep their breadth, plugins see nothing. A command becomes plugin-visible in the release that gives it `Meta` | none; this is default-deny |
| X7 | Proving declared effects | 01 C3: effects probe through an `Env` seam in every engine. 02/08: choke points and `Origin::Plugin` at sinks | **Choke points first** (FigCraft's `file_access` + clippy `disallowed-methods` for fs, net, process and clipboard; the dispatcher sets the allowed-effect mask). The probe is Phase 6 hardening | Choke points need engine edits in each app, but fail safe on a wrong declaration |
| X8 | Data plane under X1 | 03: host writes straight into guest linear memory (0.87 ms) and rejects mmap aliasing (page faults, 2.12 ms first run). 02: guest memory backed by shared memory through `MemoryCreator` | **Persistent shared-memory arenas** created at instantiation, backing the guest's linear memory in the worker and mapped in the app. The host's producer (decode, convert, render) writes straight into the arena, which is 03's option B across a process boundary. 03's page-fault cost applies once per arena lifetime; `linear_memory_keep_resident` and prefaulting at activate remove it from the per-frame path | The `MemoryCreator` path is not benchmarked. Phase 2 gate: worker e2e within 10% of in-process (02 §4.1) |
| X9 | Canonical vocabulary | 01: small typed core with resources (`document`, `image`). 08: 84 canonical ops with `targets: [TargetRef]`. Program: 6 shared ops | **08's naming and shapes**, shipped in tranches (about 30 in Phase 1, the beta adapters already cover most of them). Handles only for things with lifetimes (images, jobs, files, requests). App-specific commands are `app.<appId>.<id>` | Renames three beta adapters' ops now (`marker.add`, `node.set`, `selection.set`), before third-party plugins ship |
| X10 | Permission names | `app.automation` (01) vs `app.automate` (04); `app.intercept:<point>` (01) vs `tool.replace`/`tool.augment`/`export.hooks` (04); `app.view` (01) vs `selection.write`+`view.write` (08) vs `ui.view` (05); `secrets` permission (01) vs free (02); `compute.nn` (01, 03) vs free (02); `settings.propose` (04) vs preferences refused (01, 08); `net.local:<port>` (02) vs `network:loopback` (06) | **`app.automation`; `app.intercept:<point>`; one `app.view` covering view and selection writes (declared, never prompted, refused in automatic events without `app.automation`); own secrets free; no `compute.nn` (models are package data shown at install); preferences refused in v1, `settings.propose` deferred; `net.local:<port>` Verified-only** (owner decision) | Fewer prompts and spellings; selection-write is not separately revocable |
| X11 | Gesture grants | 01 D4: selection-scoped document access only. 02 D9: also `clipboard.write` and `ui.open_url` | **02's union**, explicit user invocations only | none |
| X12 | **GPU device** | 02 D3, 03 D6, program: the app device, with an "Isolate GPU plugins" diagnostic. 07 TC-11 and decision 2: a dedicated cook device in TextureCraft | **Run on the device that holds the data** (03 §1). Interactive GPU-resident chains (EC, PC canvas, FilmCraft, LC) use the app device. Apps whose data is CPU-resident (TC's u16 tile compositor, MC's CPU cook) pay an upload anyway, so they use `craft-gpu`'s job device, which also isolates the viewport | One rule, no per-app debate. TC pays VRAM duplication on 8 GB GPUs |
| X13 | WGSL feature limits | 02 §4.5: no storage, workgroup memory, atomics or barriers in any plugin code. 03: L1 none, L2/L3 allow declared resources. 05 D4: full pipelines behind a P2 flag | **L1 function bodies: 02's rules, Community-eligible after the fuzz gate. L2 passes: storage buffers and textures the host created for this plugin, workgroup memory up to a host cap, atomics on plugin-owned buffers; Verified tier first.** The invariant that matters is 02's own: bindings hold only data the plugin may read. L3 imperative command lists deferred | Particle simulation, reductions and sorts become possible; timer construction from storage+atomics is the same exposure WebGPU gives every web page |
| X14 | GPU time budgets | 02: ≤8 ms interactive, ≤100 ms jobs, abort a tile >250 ms. 03: ≤4 ms per tile group, watchdog 1 s / 10 s | **One table (D15)**: interactive ≤4 ms GPU per submit; jobs ≤50 ms per submit while the app is interactive and ≤250 ms when idle or exporting; strike at 1 s pending, quarantine by digest on device loss | none |
| X15 | GPU CPU fallback | Program `gpu.shader@1` and 05 P-GPU: mandatory Wasm or kernel fallback. 03: declared fallback only for web/headless, software adapters in CI. 07: CPU reference re-cook of the naga IR | **No mandatory second implementation on desktop.** Headless and CI use software adapters (lavapipe, WARP; Metal on macOS runners). 03 rejects a naga CPU interpreter for good reasons (no maintained crate, second compiler to secure), which also rules out 07's re-cook. The reference render for a tolerant GPU node is the same shader on the same backend, with golden images and a ΔE tolerance | MC and TC `AGENTS.md` rules ("GPU must match the CPU reference") do not extend to plugin nodes. **Owner decision** |
| X16 | AI runtime | See F10; also `nn.run` (most) vs `model.run` (05) | **`nn.run` on `craft-nn`**: candle for curated architectures (SAM, Whisper, depth, upscalers) with plugin-supplied weights pinned by sha256; LC's WGSL convs as a fast path on the app device; a generic-ONNX path chosen by a spike (D22). `ort` only as an optional native runtime pack later | candle needs Rust code per architecture; plugins with novel architectures wait for the ONNX path |
| X17 | **Real-time audio placement** | 07 SC-5: `craft:` Wasm on the audio thread in-process. 02: worker with an RT thread, shared-memory rings and one wake per block. Both agree native CLAP/VST3/AU move to a bridge process | **One out-of-process audio transport (`craft-audio-rt`) for both**: Wasm audio runs on an RT thread in the package worker, native plugins in `craft-plugin-bridge`. Synchronous wake per block by default; the host measures p99 at activate (07 §6.4) and switches a slot to anticipative mode (one block of reported latency) only before playback starts, never mid-session. Faults output the dry signal. **Owner decision** (also: default native isolation mode) | Unmeasured wake latency (07 estimates 20–100 µs per round trip). Ten Wasm plugins from ten packages in one chain is ten round trips per block; the Phase 5 gate measures it |
| X18 | Durable plugin output | 05: bake on apply. 06 S6: stored recipe + cached output. 07: magenta placeholder downstream. EC: pass-through with badge, final render refuses | **One rule (D16)**: every plugin output the render path depends on is stored as `{plugin, version, digest, params, determinism, durable_output}`, where `durable_output` is a baked table (eases, curves, LUTs, deformation) or a cached result. Missing, quarantined or version-mismatched plugins render the durable output with a badge; export uses it and says so, or refuses if none exists unless the user chooses "render without" | Larger documents (06 open decision 7: 100 MP smart filters) |
| X19 | Webview | 04 O1: none now. Others assume declarative UI | **No webview.** Revisit only with catalogue evidence. **Owner decision** (already open as O1) | Some UXP/CEP panel ports need rewriting into UI trees |
| X20 | Tool replacement and defaults | 01: `app.intercept` at app-declared points with a user toggle. 04: user-chosen alternative implementation; `settings.propose` | **User-chosen alternatives at app-declared points under `app.intercept:tool:<id>`**; preferences unchanged by plugins in v1 | none |
| X21 | AniCraft UI | 05 D5: keep ADR 0075 dock, add `Panel::Plugin`, bump egui. 04: move to the suite egui fork first | **Bump to egui 0.37 fork (required to embed `craft-ext-ui`); keep AC's own dock with a `Panel::Plugin` variant until AC ports to the framework** | Touches AC's whole shell dependency set |
| X22 | Native restricted helpers | Suite replacement rule: native helpers "remain out of process and supervised; they are not OS-sandboxed". Code: `NativeMode::Restricted` uses `sandbox-exec`/`bwrap`. 02: strengthen restricted mode with a self-testing launcher, Landlock+seccomp, LPAC | **Keep restricted mode and 02's macOS launcher; Linux Landlock+seccomp; Windows deferred until Windows ships.** The rule's sentence describes full mode; **owner decision** to amend its wording | none beyond 02's estimate |
| X23 | Agent access | 01: an agent-invoked plugin runs with the plugin's grants only. 02: opt-in per plugin, default on for L0/L1 | **Both** | none |

---

## 2. Feasibility and complexity critique

### 2.1 Over-engineered for a small team (cut or defer)

| Item | Source | Why it is too much now | Disposition |
|---|---|---|---|
| WIT world + component model + wasmi canonical-ABI lowering + Pulley | 01 §5.2 | Three engine paths and a second IDL. X1 removes the speed argument | Defer (X3) |
| Three engine backends (Wasmtime, wasmi, browser) | 03 §4.1 | No web or iOS plugin target exists | Two backends; browser later |
| GPU L3 imperative `gpu.*` with Dawn-wire-style validation, L4b/L4c | 03 §5.2 | A validating WebGPU proxy is the largest attack surface in the design | L1 and L2 first; L4a overlays via draw lists; L3 only for a named catalogue need |
| Effects probe with `Env` seams in every engine (C3) | 01 §5.6 | Needs I/O abstraction in 11 engines | Choke points first (X7) |
| 84 canonical ops at once | 08 §6.3 | Each op needs semantics and a conformance test in every app | About 30 in Phase 1 |
| Per-OS confinement on three OSes, TUF, offline Sigstore, reproducible builds, enterprise policy, audit log, 5 risk levels, cool-off | 02 | A product-security programme, not a first milestone | macOS confinement with the worker; audit log and risk levels with net; TUF and provenance before public launch; Windows when Windows ships |
| T3 host editors (node graph, timeline lanes) as plugin components | 04 §3.4 | Each is a large framework component | Curve and gradient first; graph editor when MC/TC adapters need it |
| `expression.function@1`, `runtime.script@1`, `widget@1`, `control.surface@1`, `sample.stream`, ARA-like clip editing, host modulators | 05, 06, 07 | Each is a product in itself | Phase 6+ |
| Contract families (program §2.4 lists 25+) | program | Each contract is a versioned promise | v1 ships 8 (D18) |

### 2.2 Under-specified

1. **Worker semantics.** How a host call from the worker reaches the app's document owner thread;
   whether the worker blocks while the app runs a command; re-entrancy (a filter's `frame.at` asking the
   app to render a frame that runs another plugin in another worker); how a transaction timeout cancels
   both sides. Needs a written state machine and a conformance test per case before code.
2. **Thread ownership in `Runtime<A>`.** The program's `with_document(doc, Box<dyn FnOnce>)` hides
   which thread runs it and how long the UI thread may wait. 04 says the UI thread never waits; PhotoCraft
   waits up to 10 s today (`plugin_sessions.rs:330-383`).
3. **Tolerant loading per app format.** TC fails to load on one unknown variant; FilmCraft drops unknown
   data and renders unknown effects as silent pass-through (`render/src/effects.rs:508`); AC's `Filter` is
   closed. No report specifies a shared helper.
4. **Contract migration.** How `raster.filter@1` packages run under `@2`, and how ABI v1 is retired.
5. **Plugin package size and hosting** for GB-scale model weights in the catalogue.
6. **GPU texture usage flags.** 03 §5.5 says app textures need `TEXTURE_BINDING`/`STORAGE_BINDING`;
   which app textures qualify is not inventoried.
7. **Repo authority.** Eight of the eleven apps are on upstream-tracking `main`. Plugin work there needs
   a `plugin-beta` branch per app and the owner's authorization; it also lands in code those upstreams
   keep changing.
8. **Verification budget.** The workspace allows only cheap scoped checks on the Mac; GPU, OS-sandbox and
   full-app suites need approved Actions runs. Several gates below need GPUs.

### 2.3 Hidden dependencies

| Dependent | Needs first | Why |
|---|---|---|
| Registry bridge (`app.run`) | `craft-command` `Meta` in that app | Without effects, default-deny exposes nothing (X6) |
| Generated bindings and MCP schemas | `ParamSchema` on commands | 9 of 6,193 have one |
| Any plugin data or contribution in TC, FilmCraft, AC | Tolerant decoding (D17) | One unknown variant breaks the document today |
| Node, effect, filter, tool contributions in MC, TC, FilmCraft, AC | Open the closed registries (MC-1, TC-2, FC-A1, AC-A5/A6) | Dispatch is a closed `match` |
| New adapters | `Runtime<A>` extraction from the three beta adapters | Otherwise four more copies of 1,500 duplicated lines |
| UI contribution points (menus, inspector, status) | Each app shell exposing extension groups through `HostUi`; AC egui 0.37 | Plugins cannot place into menus the shell does not expose |
| Automatic hooks (`document.changed`) | Per-app change sources (A-ALL-6, S4, MC-5, TC-5, SC-15) | Every app polls today |
| Wasmtime shipping | Worker transport + macOS confinement + AOT cache HMAC | X1 |
| GPU tier in an app | `craft-gpu`; usage flags on that app's textures; MC/TC `AGENTS.md` amendment | 03 §5.5; 07 decision 1 |
| Stateful effects (particles, denoise profiles) | Per-instance state + `cache.*` + stepped state contract | MFR rule (05 §5.2) |
| Wasm audio | SC-1 event lists, SC-2 sample-accurate automation, SC-4 per-slot faults, SC-7 no-alloc guard | 07 §5.3 |
| **Native tier being meaningful at all** | Per-run tokens on control ports | Six control ports accept any local process (08 §8): a native helper can already drive the app around every permission |
| Public launch | TUF, hourly revocation, provenance | 02 §6 |

### 2.4 Effort sanity

Team assumed: two engineers plus AI agents, one build owner, Mac cheap checks, approved Actions runs.

| Work | Prior estimate | Realistic | Reason |
|---|---|---|---|
| Program Phase 0 | 1–2 weeks | 2 weeks | Fine, plus control-port tokens |
| Program Phase 1 | 4–8 weeks | **16–22 weeks**, split here into Phases 1 and 2 | It bundles `Runtime<A>`, three ports, the bridge, net/secrets/OAuth/events, image handles, binary transport, overlays, parameter kinds, `raster.filter@2`, `EffectHost` passthrough and bindings |
| `Meta` backfill | – | ~7,200 declarations; census prefills ~85% of effects. Agent-drafted, human-reviewed at ~200 a day: **35–40 review-days**, spread across app owners | Security review cannot be delegated to an agent's judgement alone |
| Each new adapter | L | 4–8 weeks each after its app prerequisites; 8 apps ≈ 8–12 months serial, 5–7 months with two streams | 05, 06, 07 per-app lists |
| Worker + macOS confinement | not in program | 6–10 weeks | X1 |
| `ext-gpu` L1+L2 with `craft-gpu` | L | 10–14 weeks | Validation, scheduler, quotas, device loss, two app integrations |
| `craft-nn` | L | 8–12 weeks after the ONNX spike | |
| Audio (SC prerequisites + Wasm tier + native bridge) | Phase 3 | 5–7 months, mostly SoundCraft app work | 07 §5.3 |
| **Whole suite to the decisions below** | ~6–8 months implied | **14–20 months** | |

### 2.5 The honest boundary of "anything the MCP can do"

MCP is a trusted local operator channel (08 §8). Plugins get the census's ≈79% directly (document
edits, queries, selection and view) and ≈14% through host mediation (file handles, clipboard, jobs,
host-owned undo). The remaining ≈7% stays operator-only by design: preferences and shortcuts, process
and OS actions, plugin management, collaboration identity, window management, model installs, raw
undo groups and app lifecycle. Plugins still get the *outcome* of several of these through safer
routes: presets and workspaces as data, `net.fetch` instead of app network commands, `handoff.open`
instead of process launch.

---

## 3. Author experience: six plugins end to end

Each walkthrough assumes the decisions in §4. "Prompts" lists what the user actually sees.

### 3.1 Particular-like particles in EffectCraft

```toml
schema = 2
id = "io.example.particles"
apps = ["effectcraft"]
determinism = "seeded"

[[contribution]]
id = "particles"
kind = "effect"
contract = "raster.filter@2"
module = "particles.wasm"
output = { bounds = "comp" }
state = "stepped"
exec = ["gpu.pass"]
params = "params.toml"
```

- **APIs.** Parameters include `Layer` (emitter), `Point3`, `Gradient` and `Curve` kinds. Context gives
  `scene.camera`, `scene.lights`, `layer.at(emitter, t)` as a GPU handle. Simulation is an L2 pass on
  plugin-owned storage buffers (positions, velocities), with a splat pass to the output texture.
- **Prompts.** None. Applying an effect is a user action, and the emitter layer comes from a parameter
  the user picked, so the parameter is the capability (fix A2).
- **Performance path.** Entirely on the app device inside EC's frame graph; no readback. The Wasm module
  runs only `step` setup logic in the worker, per frame, with a few control values.
- **What breaks.** (1) Multi-frame rendering: frames render out of order on rayon threads, but the sim
  needs frame *n−1*. Today an author would have to write a pre-roll job and a cache by hand. (2) 02's
  rule forbids storage buffers and atomics in plugin WGSL, which makes a GPU sim impossible. (3) The
  program required a CPU fallback for every GPU contribution: the author would write the sim twice.
- **Fixes adopted.** A1 *stepped state*: the author implements `step(state, t) -> state` and
  `render(state, t) -> image`; the host runs pre-roll in order, caches state handles per frame keyed by
  `(digest, params hash, t)`, and invalidates on parameter change. X13 allows storage in L2. X15 removes
  the mandatory CPU twin.

### 3.2 Topaz-like AI upscaler in PhotoCraft

```toml
[[contribution]]
id = "upscale"
kind = "effect"
contract = "raster.filter@2"
module = "upscale.wasm"
output = { scale = [2, 4] }
tiles = { overlap = 32 }
accepts = ["rgba16f"]
run = "job"

[[data]]
id = "model-x4"
kind = "model"
url = "https://releases.example.com/x4.safetensors"
sha256 = "…"
bytes = 268435456
```

- **APIs.** `nn.load` on the pinned data, `nn.run` with image handles in and out per tile; the host's
  pre/post-processing kernels; `job.progress`.
- **Prompts.** None at run time. The install sheet shows "includes a 256 MB model".
- **Performance path.** Tiles stream from PC's 256-px tile store through host handles to GPU buffers on
  the app device; output tiles are written straight into the resized layer. Jobs keep each submit under
  50 ms while the app is interactive.
- **What breaks.** (1) The output is bigger than the input; today the adapter has no resize or new-document
  op, and the author would have to orchestrate `image.canvasSize` through the bridge plus pixel writes
  in one transaction. (2) PC converts every plugin buffer to sRGB, clipping wide-gamut 16-bit documents.
  (3) The weights sit unencrypted in the package; a commercial vendor cannot protect them. (4) A model
  with an architecture `craft-nn` does not know cannot run.
- **Fixes adopted.** A3: `output.scale` in the contract makes the host resize the canvas inside the same
  transaction and hand the plugin a correctly sized output handle. A4: pixel requests carry
  `space = native | rgb-linear` and `depth`; the host refuses rather than clipping (06 §5). Weight
  protection is out of scope for the sandbox: a vendor that needs it uses the native tier. Architecture
  coverage is D22.

### 3.3 Unsplash-like panel in FigCraft

```toml
[why]
"network:api.unsplash.com" = "Searches Unsplash for photos"
"network:images.unsplash.com" = "Downloads the photo you choose"

[[contribution]]
id = "browser"
kind = "panel"
contract = "figcraft.session@2"
module = "unsplash.wasm"
permissions = ["network:api.unsplash.com", "network:images.unsplash.com"]
[contribution.panel]
area = "right"
```

- **APIs.** A T1 tree with a lazy `grid` (`rows_source = "plugin"`), `ui.values.set` for the search
  state, `net.fetch` for JSON, `secret.get` for the user's own API key if the plugin asks for one,
  `node.create` + `style.fill.set` with an image handle, and drag from grid cells onto the canvas.
- **Prompts.** One consent sheet on first search listing both hosts with the reasons. Inserting into the
  selection needs no prompt (gesture grant).
- **Performance path.** Control plane only. Thumbnails never enter guest memory (fix A5).
- **What breaks.** (1) Today `net.fetch` would return JPEG bytes to the guest, which would pass them back
  to the host to make an image: two copies, a decoder in Wasm, and megabytes through the 1 MiB call cap.
  (2) Two hosts mean two JIT prompts under 02's per-host rule. (3) The risk label "Online: can send your
  work to unsplash.com" is false: the plugin never reads the document. (4) FigCraft's adapter has no
  image-fill op and no drag source.
- **Fixes adopted.** A5: `net.fetch {into: "image"}` returns a host-decoded image handle; `into: "bytes"`
  stays for JSON. A6: one network consent per plugin covering every declared host. A7: the risk sentence
  depends on read access; network without `document.read` reads "Online. Cannot read your documents."
  FigCraft gains `image.create` and drag sources in its Phase 1 port.

### 3.4 Substance-like custom node in MaterialCraft and TextureCraft

```toml
apps = ["materialcraft", "texturecraft"]

[[contribution]]
id = "edge-wear"
kind = "node"
contract = "graph.node@1"
domains = ["material", "texture-generator"]
exec = "wgsl"
source = "edge_wear.wgsl"
determinism = "tolerant"
neighbourhood = { radius_px = 4 }
inputs = [{ id = "curvature", kind = "gray" }, { id = "ao", kind = "gray" }]
outputs = [{ id = "mask", kind = "gray" }]
params = "params.toml"
```

- **APIs.** None at run time: an L1 WGSL `fn cook(p: Pixel) -> vec4f` with generated `load_curvature`,
  `load_ao`, `params.*`. In TC the host binds `mesh_maps.curvature` and `mesh_maps.ao` automatically when
  the user leaves the inputs unconnected.
- **Prompts.** None; the package has no permissions.
- **Performance path.** MC: chains of WGSL nodes fuse into one shader on `craft-gpu`'s job device; CPU
  native nodes upload once per boundary. TC: per 16×16-tile block, u16→f16 once per block, the
  neighbourhood radius joins TC's existing neighbourhood precompute.
- **What breaks.** (1) Neither app can host it today: MC dispatch is a closed `fn` table, TC's enums are
  closed and a document carrying an unknown variant fails to load. (2) MC and TC `AGENTS.md` require GPU
  output to match a CPU reference; the author has no CPU path. (3) The node needs one manifest entry per
  app slot (MC library, TC generator menu).
- **Fixes adopted.** App prerequisites MC-1, MC-2, TC-1, TC-2, TC-3 are Phase 3 gates. X15: the reference
  is the same shader with golden images; owner decision on the `AGENTS.md` rule. A8: `domains` places one
  node in every compatible slot; each app maps domains to its own menus.

### 3.5 Valhalla-like reverb in SoundCraft (and FilmCraft, EffectCraft)

```toml
apps = ["soundcraft", "filmcraft", "effectcraft"]

[[contribution]]
id = "plate"
kind = "audio-effect"
contract = "audio.effect@1"
module = "plate.wasm"
buses = { main = "stereo" }
latency = 0
tail = "params.decay"
determinism = "exact"
params = "params.toml"
```

- **APIs.** `process(ctx)` with sorted `param_value`/`param_mod` events, planar views of the block,
  `state.save/load`, a meter ring. The device face is host-generated from parameters (knobs, meters);
  an optional T2 canvas draws the decay curve from bound values.
- **Prompts.** None.
- **Performance path.** The worker's RT thread joins the host's audio workgroup; one wake per block;
  SIMD128; no allocation after `activate`. Target p99.9 ≤2× native.
- **What breaks.** (1) SoundCraft applies plugin parameters once per block and CLAP events at
  `time: 0`, so automation of the decay is zipper-prone and bounce ≠ playback. (2) A panic anywhere
  silences the master. (3) If the host had to pick anticipative mode mid-session, latency would jump.
  (4) The DSP code is decompilable; commercial vendors may keep native builds.
- **Fixes adopted.** SC-1/SC-2/SC-3/SC-4 are Phase 5 prerequisites. A9: the latency mode is chosen once
  at activate from measured p99 and reported to PDC before playback. The same package runs in FilmCraft's
  `AudioEffect` slot and EC's audio effects through the same contract.

### 3.6 Duik-like rigging panel in AniCraft

```toml
apps = ["anicraft", "effectcraft"]

[[contribution]]
id = "rig"
kind = "panel"
contract = "anicraft.session@1"
module = "rig.wasm"
permissions = ["document.edit:selection"]
optional_permissions = ["document.edit:document"]

[[contribution]]
id = "controller"
kind = "overlay"
contract = "rig.controller@1"
module = "rig.wasm"
```

- **APIs.** Canonical `node.*`, `transform.*`, `key.*`; app ops `app.anicraft.bone.add`,
  `app.anicraft.bone.setIk` through `app.batch` (one snapshot, one undo step); overlay gizmos (`point`,
  `rotate`) mapped to parameters; in EffectCraft the same panel branches to `app.effectcraft.prop.setExpression`
  and null layers.
- **Prompts.** None for "rig the selected symbol". Rigging the whole scene asks once for document scope.
- **Performance path.** Control plane: a few `app.batch` calls with hundreds of steps each.
- **What breaks.** (1) AniCraft has no plugin panels (closed 13-variant `Panel` enum, egui 0.36), and its
  own plugin system must be bridged first. (2) Rig creation makes new bones and layers outside the
  selection. (3) AniCraft has no expressions, so live controllers need a contract; baked keys go stale
  when the user moves a controller. (4) Ids are `snake_case` in AC and `lowerCamelCase` elsewhere.
- **Fixes adopted.** X21 and AC-A1/A2 in Phase 3. Objects created by the transaction are in scope
  (01 §5.5). `rig.controller@1` drags write parameters, and the host re-bakes affected keys in the same
  undo step (bake on apply, D16). App ops keep native spelling under `app.anicraft.*`; canonical ops are
  `lowerCamelCase` everywhere.

### 3.7 Fixes that change the design

| Fix | Change | Decision |
|---|---|---|
| A1 | Stepped temporal state in `raster.filter@2` | D18 |
| A2 | Parameter kinds that reference objects (`Layer`, `Mask`, `Path`, `File`) grant read access to the chosen object only | D12 |
| A3 | Declared output geometry; the host resizes inside the transaction | D18 |
| A4 | Pixel requests with space and depth; refuse instead of clipping | D10 |
| A5 | `net.fetch {into: image | bytes | file}` | D13 |
| A6 | One network consent per plugin | D13 |
| A7 | Risk sentence depends on read access | D12 |
| A8 | `domains` for cross-app node placement | D18 |
| A9 | Audio latency mode fixed at activate | D23 |

---

## 4. Reconciled architecture

### 4.1 Layers and crates

| Layer | Owns | Crates (new in **bold**) | Repo |
|---|---|---|---|
| Shared libs | Command metadata, schemas, GPU device and pool, AI, audio RT transport, DSP, graph runtime, mesh, tolerant serde, UI components | **`craft-command`** (from VC `apischema`), **`craft-preserve`**, **`craft-gpu`**, **`craft-nn`** (from `craft-ai`), **`craft-audio-rt`**, **`craft-audio-dsp`**, **`craft-graph`**, `craft-mesh` (`MeshData`), framework components on `next` | craft-libs |
| Platform | Manifest, permissions, contracts, runtime, gate, transactions, events, jobs, services, engines, worker, GPU tiers, UI model and renderers, store, manager, CLI, SDK | `ext-core`, **`ext-runtime`** (`Runtime<A>`, gate, txn, conformance), `ext-host` (wasmi + worker client), **`ext-worker`** (binary `craft-plugin-worker`), **`ext-gpu`**, `ext-kernels`, **`ext-ui-model`**, `ext-ui`, `ext-ui-fw`, `ext-store`, `ext-manager`, `ext-cli`, `guest` (+ generated `apps/*`), `native-sdk` | craft-extensions |
| Adapter | Mapping only: canonical ops → registry, document access, change source, pixels/GPU views, `HostUi` | one `crates/plugins` per app | each app |
| App | Commands with `Meta`, effect choke points, open registries, tolerant formats, change events, shell extension groups | existing crates | each app |
| Catalogue | Signed index, packages, models as data | – | craft-plugins |
| Launcher | `handoff.open`, shared model store location | – | craft-launcher |

### 4.2 The one adapter trait

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

`AppInfo` carries the app id, API level, contract list, placements, `when` context keys and limits.
`CanonicalMap` maps each supported canonical op to a registry id and a parameter mapping, generated from
`Meta.exposure`. `PixelSource` is a CPU tile view or a GPU texture view on the device that owns it.
`on_owner` returns a `Pending` and never blocks the caller; the runtime awaits it with the transaction
deadline. Everything else (gate, transactions, events, jobs, grants, UI limits, workers, filter service,
bindings) lives in `Runtime<A>`.

### 4.3 Decisions

"Owner" marks decisions that need the owner before the phase that depends on them.

| # | Decision | Chosen | Rejected | Owner |
|---|---|---|---|---|
| D1 | Layer rule | Behaviour lives in `ext-runtime`; an adapter only maps. A shared op has one schema in every app, enforced by `conformance::run::<A>()` | Per-adapter logic (today's ~1,500 duplicated lines) | |
| D2 | Command metadata | `craft-command` in craft-libs; mandatory `Meta {effects, scope, undo, run, destructive, params, result, exposure, since}` on every `CommandSpec`, shell commands included; macros refuse to compile without it | `bridge.toml` (06); schemars (07); JSON-schema strings (05); adapter tables | |
| D3 | Exposure | Computed by one platform function from `Meta`, first matching rule wins (01 §5.6 rules on 08's effects). Output is `api.lock.json` in the app repo; CI fails on drift; widening needs a log line and a security CODEOWNER | Reviewed lists outside the registry | Owner names the security reviewer |
| D4 | Commands without `Meta` | AgentOnly | Exposed with a coarse permission | |
| D5 | Effect enforcement | Choke points for fs, net, process, clipboard, prefs; dispatcher-set effect mask; `Origin::Plugin` refused at those sinks | Trust in declarations alone; per-engine probe first | |
| D6 | Call surfaces | Canonical ops (08 naming, `targets: [TargetRef]`, one colour type, rational time, document units from `doc.get`) + `app.<appId>.<id>` through `app.run`/`app.batch`. Same gate, same transaction | Per-adapter hand ops; aliases between apps | |
| D7 | ABI and IDL | ABI v1 `craft.call` kept; `craft.call_buf` for bulk structured data; views, handles, rings for media; bindings generated from `ParamSchema` (Rust, then TS and C) | WIT/component model now; per-command WIT | **Owner** (WIT deferral) |
| D8 | Engines | Wasmtime 50+ Cranelift, `parallel-compilation`, AOT at install into a host-owned HMAC'd cache, epochs at 1 ms (0.25 ms with audio), pooling allocator with ~256 slots, NaN canonicalization (after benchmark), SIMD128 on, relaxed SIMD off. wasmi (without `portable-dispatch`, with `simd`) as reference and confined-worker fallback | Winch; Pulley; browser backend now; dual engine by contribution | |
| D9 | Process topology | `craft-plugin-worker` per package version, lazy, idle-reaped after 60 s (not while audio is armed or a live preview is open); macOS XPC service with App Sandbox and `allow-jit`; Linux Landlock + seccomp + `no_new_privs`; host-side broker holds all authority | In-process JIT, including as a fallback | **Owner** |
| D10 | Data plane | Persistent shared-memory arenas backing guest memory; host producers write into them; `#[repr(C)]` view descriptors; RGBA16F default interchange; handles for large or random-access data; SPSC rings for streams; validation only at commit, export or GPU upload; pixel requests carry target, space, depth and level | mmap aliasing of app frames; `list<u8>` media; per-call validation | |
| D11 | Transactions | One event = one transaction = one labelled undo step; registry calls run nested (no journal, no undo push, no redraw); one coalesced `document.changed` at commit; `app.batch {steps, on_error: rollback | stop | continue}` with schema pre-check; `Long`/`Job` commands only through `job.run_command`, results applied with a revision check | Per-command undo inside plugin events | |
| D12 | Permissions | Existing vocabulary plus `app.view`, `app.automation`, `app.intercept:<point>`, `files.folder:picked`, `service:<id>/<name>`, `media.capture:*`, `fonts.install`, `net.local:<port>` (Verified only). Free: compute, GPU, `nn.run`, own settings, own secrets, own plugin data, panels, menus, timers. Never: processes, raw paths and sockets, preferences, history, plugin management, collaboration identity, agents. Gesture grants for selection scope, `clipboard.write`, `ui.open_url`. Object-reference parameters grant that object only. Five host-computed risk levels; the sentence reflects read access | `compute.nn`; `secrets` prompt; `settings.propose`; separate `selection.write` | **Owner** (`net.local`, gesture grants) |
| D13 | Network | `net.fetch` host-performed, HTTPS, declared hosts, one consent per plugin, no redirects off-list, private IPs refused after DNS, `into: image | bytes | file`, `auth: "secret:<name>"` injected host-side, per-plugin log | Raw sockets; app-owned network commands for plugins (v1) | |
| D14 | GPU stack | `craft-gpu` (device broker, texture pool, memcap, timestamp scheduler, device-loss, pipeline warm-up) in craft-libs; `ext-gpu` (naga validation, IR rules, host-composed shaders, L1 fusion, L2 DAG) in craft-extensions. Device = the one holding the data. L1 Community after the fuzz gate; L2 Verified first; L3 deferred | Dedicated plugin device for interactive work; raw device access | **Owner** (MC/TC `AGENTS.md` wgpu rule) |
| D15 | GPU budgets | Interactive ≤4 ms per submit, plugins ≤50% of frame GPU; jobs ≤50 ms per submit interactive, ≤250 ms idle; strike at 1 s; quarantine by digest on device loss; quotas by descriptor (256 MiB effect, 1 GiB job, 64 MiB overlay defaults) | Separate tables per report | |
| D16 | Durable output and determinism | Manifest `determinism = exact | seeded | tolerant | stateful | networked`; cache keys include `(id, version, sha256, contract, determinism, backend-class)`; durable output stored per instance; missing plugin renders it with a badge; export refuses without it unless "render without"; project plugin lock for farms | Silent pass-through; global cache generation bumps | |
| D17 | Tolerant loading | `craft-preserve::Unknown<T>` and `#[serde(flatten)] extra` helpers; a suite rule and a per-app round-trip test (unknown variant and field survive open/save byte-identical) gate every adapter | Per-app ad hoc fixes | |
| D18 | v1 contracts | `<app>.session@2`, `raster.filter@2` (ROI, apron, tiles, output geometry, extra inputs, `frame.at`, stepped state, `quality`, `proxy`), `graph.node@1` (with `domains`), `format.import@1`/`format.export@1`, `audio.effect@1`, `resource.*` data kinds, `resource.node-library@1`, `gpu` exec modes inside the first two | 25+ contracts at once | |
| D19 | UI rendering | T1 retained tree (UI 1.3 keyed patches, value bindings including `param:`, live controls, lazy rows), T2 retained layered canvas with regions, T4 viewer of handles, T5 native windows only for the native tier; zero guest calls per frame; images by handle; no webview | Immediate-mode callbacks; raw GPU surfaces; webview | **Owner** (no webview) |
| D20 | UI hosting | `HostUi` implemented once per app shell; `ext-ui-model` owns surfaces, limits, patches, placeholders; framework engine is the end state, egui engine is a bridge removed when the last adapter app ports; closed placement catalogue with host-evaluated `when`; trusted chrome and a host-only trust layer | Per-app plugin UI logic; a third renderer | |
| D21 | Behaviour hooks | Declared hook contributions at app-declared points, user-ordered, fail-open for save/export/quit, bypass modifier, safe mode disables all; tool replacement as a user-chosen alternative | Generic command wrapping; hiding built-in UI | |
| D22 | AI | `craft-nn` in craft-libs: candle backend for curated architectures, LC WGSL convs as fast path, a generic ONNX backend picked by a spike over tract and candle-onnx on five target models (upscaler, denoiser, depth, matting, stem separation); suite-wide content-addressed model store; `nn.run` as a job | LC's six-op reader as the runtime; `ort` in the base | **Owner** (ONNX runtime choice after spike) |
| D23 | Audio | `craft-audio-rt` (CLAP-shaped events, rings, wake, dry crossfade, no-alloc guard) shared by Wasm-in-worker and `craft-plugin-bridge` for native; latency mode fixed at activate; faults go dry, never silent | Wasm on the in-process audio thread; native in process | **Owner** (default native isolation mode) |
| D24 | Native tiers | Restricted (OS-confined helper, self-testing launcher) and full (consent per version, Verified+ with signer pin) out of process; per-run tokens on every control port, withheld from helper environments | Native full for Community; untokened control ports | **Owner** (replacement-rule wording) |
| D25 | Legacy systems | VC legacy: shim inside the platform runtime, data migration, removal after two releases at parity. AC ABI v1: bridge, removal after platform builds of its samples ship. SC in-process hosting → bridge process. EC scripts, VC js, DC step runner, PC Actions stay as automation and classify through `Meta` | Leaving any competing loader | |
| D26 | Distribution | Now: ed25519 index plus an hourly signed revocation snapshot, widening holds updates, cool-off 72 h Community / 24 h Verified. Before public launch: TUF with offline root, offline provenance verification for native, Verified and Official | TUF as a Phase 1 blocker | **Owner** (launch gate) |
| D27 | Authoring | `craft-guest` with generated `apps::<app>` modules; `craft-ext dev --watch`, `test` (MockHost loaded from real `api.lock.json`), `run --app --doc --script`, `bench`; templates per contribution kind; plugin console with per-call CPU, GPU, bytes and IPC | Hand-written per-app docs | |
| D28 | Agents | Plugin commands reach MCP by per-plugin opt-in (default on for L0/L1); agent-invoked plugins use only persistent grants; plugin text returned to agents marked untrusted; MCP schemas generated from `craft-command` | Shared authority between agents and plugins | |

---

## 5. Roadmap

### 5.1 Dependency graph

```text
P0 quick wins ──────────────┬──────────────────────────────────────────────┐
                            │                                              │
P1a craft-command + Meta ───┼─> P1b Runtime<A> + 3 ports + bridge ─> P2a worker + views ─> P2b ext-gpu L1 + craft-gpu
(beta apps)                 │        │                                   │                   │
                            │        └─> P1c net/secrets/events/UI 1.3   │                   │
                            │                                            v                   v
P3a app prerequisites ──────┴──────────────────────> P3b new adapters (MC/TC, VC, AC, FilmCraft, LC, DC)
(tolerant load, open registries, Meta, change events)                    │
                                                                         v
                                              P4 L2 passes, craft-nn, formats, jobs depth
                                                                         │
P5a SoundCraft prerequisites (SC-1..4, SC-7) ──> P5b craft-audio-rt + Wasm audio + native bridge
                                                                         │
                                                                         v
                                              P6 hardening: TUF, provenance, probe, Windows, L3, T3, WIT review
```

P3a and P5a are app work with no platform dependency, so they can start as soon as the owner authorizes
branches in those repos.

### 5.2 Phases

**P0 · Quick wins and safety (2 weeks).** Repos: craft-extensions, effectcraft, photocraft, figcraft,
and the control-port owners.

| Work | Acceptance (deterministic) |
|---|---|
| Drop `portable-dispatch`, enable wasmi `simd`; zero-copy `Filter::run` (one write into guest memory, kernels on views, fused validation at commit) | Bench crate in CI on a fixed runner, median of 5: 1080p per-pixel loop e2e ≤15 ms (from 52.0), `craft.call` small op ≤4 µs (from 10.5); fails on >20% regression |
| Correctness bugs from program §5, each with a failing test first | FigCraft: `selection.set`, `viewport.set`, `page.select` in a change handler return `-2`; `layer.select` same permission in EC and PC; FigCraft `ui.notify` reaches the toast model; guest error text preserved |
| Per-run tokens on EC, FigCraft control ports (others listed for their owners) | Connecting without the token is refused; helper environment has no token variable |
| Dock EC plugin panels (`PanelKind::Plugin`) | Dock-model test: a plugin panel id round-trips through a saved workspace |

**P1 · Metadata, one runtime, breadth (14–18 weeks).** Repos: craft-libs (new `craft-command`,
`craft-preserve`), craft-extensions, effectcraft, photocraft, figcraft.

| Work | Acceptance |
|---|---|
| P1a `craft-command` with VC's `apischema` moved in; `Meta` mandatory in the three beta apps (EC 757, PC 954 + 146 shell, FigCraft 630); census-assisted backfill tool drafting `effects` and `ParamSchema` from the params mini-language | `every_command_has_meta` passes in each app; property test of `exposure()` over all effect combinations; `api lock --check` clean; coverage report shows ≥60% of edit/query commands with real schemas, the rest AgentOnly |
| P1b `ext-runtime` with `CraftApp`, gate, transaction, events, jobs, grants, UI limits; FigCraft, then EC, then PC ported; `app.run`, `app.batch`, `app.commands`; ~30 canonical ops | `conformance::run::<A>()` green for all three with identical results on the canonical tranche; scope test (out-of-selection id → `-2`, document hash unchanged) and undo test (rollback restores hash; commit adds exactly one undo step) for every exposed command; duplicated adapter code down ≥50% by line count |
| P1c `settings.*`, `net.fetch` (D13), `secret.*`, `auth.oauth`, events from existing change sources, `host.capabilities`; UI 1.3 patches, values, live controls; `HostUi` in three shells; generated Rust bindings | Local test server: undeclared host, plain HTTP, IP literal, off-list redirect, DNS to 10/8, 127/8, 169.254/16 all refused; one consent per plugin; `ui.patch` of 100 nodes ≤0.3 ms host time; `bindgen --check` clean; zero guest calls per frame asserted by a counter in renderer tests |

**P2 · Worker, data plane, GPU L1 (12–16 weeks).** Repos: craft-extensions (`ext-worker`, `ext-gpu`),
craft-libs (`craft-gpu`), effectcraft, photocraft.

| Work | Acceptance |
|---|---|
| P2a Wasmtime worker on macOS (then Linux); shared-memory arenas; AOT cache with HMAC; `raster.filter@2` with ROI, tiles, apron, extra inputs, `frame.at`, stepped state, output geometry; image handles; EC `EffectHost` passthrough; PC tile-native pixels with space and depth | Worker suite in Actions: file, network, exec and persistence denial (02 N1–N4); infinite loop returns within deadline + 50 ms; revocation traps within 260 ms; pooled-instance zeroing; tampered cache refused. IPC p50 ≤5 µs, p99 ≤20 µs. 1080p loop e2e in the worker ≤1.1× the in-process Cranelift figure; 100 installed plugins add ≤5 ms to start |
| P2b `craft-gpu` + `ext-gpu` L1 with fusion; EC `walk.rs` plugin node; PC canvas preview; timestamp queries requested where available | Metal runner: WGSL IR fixtures with storage, workgroup, atomics or entry points refused in L1; `loop {}` completes or aborts within budget; device-loss hook recovers and records a strike; a crash-journal entry quarantines the digest on restart; a plugin brightness pass in an EC GPU chain performs zero readbacks (instrumented counter) and ≤0.5 ms GPU at 1080p 16F |

**P3 · App prerequisites and new adapters (6–9 months, two streams).** Repos: each app needs a
`plugin-beta` branch (owner authorization); craft-libs `craft-graph`, `craft-mesh`.

| Order | App work first (P3a) | Adapter (P3b) | Acceptance |
|---|---|---|---|
| 1 | MC: `Meta` (133), open registry (MC-1), SBS dependency URLs (MC-2), change events. TC: tolerant decoding (TC-1), `Plugin` variants (TC-2), compositor path (TC-3), `Meta` (183) | `resource.node-library@1` (MC, data only), then `graph.node@1` WGSL in both | One node package produces golden images within ΔE ≤1 in MC and TC; a TC document with an unknown effect variant round-trips byte-identical; missing node renders durable output with a badge |
| 2 | VC: convert `reviewed.rs` (718 ids) into `Meta`; appearance registry with durable output | `vectorcraft.session@2`, `vector.effect@1`, formats; legacy shim | Every legacy kind and host import has a conformance-tested equivalent; shim removal date recorded |
| 3 | AC: `Meta` (365), egui 0.37, `Panel::Plugin`, `Filter::Plugin` with unknown preservation | `anicraft.session@2`; ABI v1 bridge | The four v1 sample plugins run through the bridge; v1 policy test replaced by `api.lock.json` |
| 4 | FilmCraft: `plugin_data`, unknown preservation, external effect registry, missing-effect refusal, `Meta` (686) | `filmcraft.session@2`, `raster.filter@2` on clips | Unknown effect id no longer passes through silently (test on `render/src/effects.rs` path) |
| 5 | LC: `Meta` (391), batched sidecar writes, export hooks | `lightcraft.session@2`, `export.service@1` | One XMP write per photo per plugin event |
| 6 | DC: `Meta` (567), reclassify file-writing queries, string preflight kinds | `designcraft.session@2`, `document.check@1` | `file.exportIdml` classified `FsWrite` and absent from the plugin lock |

**P4 · Depth (10–14 weeks).** Repos: craft-extensions, craft-libs (`craft-nn`), photocraft, effectcraft,
craft-plugins.

| Work | Acceptance |
|---|---|
| ONNX spike, then `craft-nn` with candle and WGSL convs; model store; `nn.run` jobs | 2× upscale of a 24 MP image ≤20 s on an M-class GPU; a model whose sha256 mismatches is refused; cancel within 100 ms |
| L2 `gpu.pass` DAG with plugin-owned storage (Verified) | Golden images on Metal and lavapipe within tolerance; quota exceed fails only the plugin call |
| `format.import@1`/`format.export@1`, overlays with hit regions, parameter kinds | Format round-trip fixtures per app; overlay drag → one undo step |

**P5 · Audio (5–7 months).** Repos: soundcraft (branch), craft-libs (`craft-audio-rt`,
`craft-audio-dsp`), craft-extensions, filmcraft.

| Work | Acceptance |
|---|---|
| P5a SC-1 event lists, SC-2 sample-accurate automation, SC-3 test, SC-4 per-slot faults, SC-7 no-alloc guard | Bounce at blocks 64/128/512/1024 is bit-identical for deterministic built-ins; a panicking slot leaves the master running; playing a session allocates nothing on the audio thread |
| P5b Wasm audio in the worker; native bridge; FilmCraft `AudioEffect` slot | Wasm EQ at 128 frames: p99.9 block time ≤2× native, zero dropouts over 10 minutes; bridge crash → dry until reload, session continues |

**P6 · Hardening (ongoing).** TUF and offline provenance before public launch; effects probe;
Linux and Windows confinement parity; L3 if a catalogue need is named; T3 graph and timeline editors;
a WIT/component-model review. Acceptance per 02 §8 (S1–S12, N5–N7, G7) and a CI map from every threat
id to at least one test.

### 5.3 Owner decisions, by the phase that needs them

| Before | Decision |
|---|---|
| P1 | Security reviewer for `api.lock.json` widenings (D3); gesture grants (D12); WIT deferral (D7) |
| P2 | Worker topology over in-process JIT (D9); no webview (D19) |
| P3 | `plugin-beta` branches for the eight upstream-tracking apps; MC/TC `AGENTS.md` wgpu rule and CPU-reference rule for plugin nodes (D14, X15); `net.local` (D12) |
| P4 | ONNX runtime after the spike (D22); catalogue hosting for GB-scale models |
| P5 | Default native audio isolation mode (D23); replacement-rule wording on restricted native (D24) |
| Launch | TUF and provenance gate (D26) |

---

## 6. Confidence

- **Verified in source for this review:** the rows in the opening table.
- **Taken from reports without re-checking:** all benchmark numbers (01, 03 and the program; measured on
  an Apple M5), per-app file and line citations in 05–08, the census classification (±10%), and every
  external product or standards claim.
- **Not measured anywhere:** worker IPC latency, `MemoryCreator`-backed arenas, cross-process audio wake
  latency, NaN-canonicalization cost, in-app `Session::execute` cost per command, and the generic-ONNX
  candidates (tract and candle-onnx were not evaluated by any report; their inclusion here is mine).
- **Estimates:** every effort figure in §2.4 and §5.

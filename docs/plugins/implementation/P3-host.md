# P3: Host runtime and hostile suite (`craft-ext-host`, `craft-ext-conformance`)

Status: **build-ready plan, 2026-11-10.** Unit P3 of [the architecture](../architecture.md#9-runtime-and-safety); deepens [implementation plan section 6, P3](../implementation-plan.md#p3-runtime-and-hostile-suite-cratesext-host-cratesext-conformance). Each PR still needs its own approval.

**Code read (nothing built or run):**

Review qualification, 2026-11-12: [AR07/AR10/AR11](../design-sessions/architecture-review-2026-11-12.md) require one revocation observation/commit contract, per-request reset/buffer lifetimes, and host-buffer/cache/blocking-work accounting. Fuel-slice checks and proposed cross-process polling are mechanisms to verify, not an agreed instantaneous global revoke guarantee. Instance reuse does not itself prove determinism.

| App | Commit | Files |
|---|---|---|
| PhotoCraft | `main` `655295b2` | `crates/plugins/src/runtime.rs`, `apply.rs`, `tests/abi.rs` |
| EffectCraft | `origin/main` `fb62b9d7` | `crates/plugin/src/wasm.rs`, `crates/effects/src/plugin.rs` |
| FigCraft | `main` `0e86f359` | `crates/plugin/src/sandbox.rs` |

All three resolve **wasmi 2.0.0** in `Cargo.lock`.

## 1. Goal and exit criteria

**Goal.** One shared WebAssembly host profile that no app can loosen. It covers:
- admission;
- fuel, deadline and cancel, with memory accounted across instances;
- instance lifetimes and the failure contract;
- small adapter traits;
- a hostile-module suite that proves each claim on every platform where it is made.

**Exit criteria (architecture section 22: "suite passes on all desktop targets and web").**

| # | Criterion | Proving test |
|---|---|---|
| X1 | No module input makes the host panic, hang or exceed memory | `hostile::*` (the full table in section 6), on Linux, Windows, macOS and in a real browser |
| X2 | Unknown imports, unsupported features, malformed sections and oversize tables are refused **before** instantiation | `admission::*` |
| X3 | An infinite loop in the start function or any export stops within the deadline; cancel stops within one fuel slice | `budget::looping_start_stops`, `budget::cancel_latency_under_one_slice` |
| X4 | Total memory across parallel instances never exceeds the app-wide ceiling | `memory::ledger_refuses_growth_over_ceiling` |
| X5 | No state reaches another document or job; a trapped instance is never reused | `instance::no_cross_document_state`, `instance::trapped_instance_discarded` |
| X6 | A revoke or a stale grant generation stops the plugin at its next host call, and the edit rolls back | `revoke::stops_at_next_host_call` (with P2's reference app) |
| X7 | Every failure maps to a structured `HostFailure` and a user message; nothing reaches `catch_unwind` | `failure::all_variants_mapped`, plus clippy denies in the crate root |

## 2. Code to learn from (verified)

| Concern | PhotoCraft `655295b2` `crates/plugins/src/runtime.rs` (start here) | EffectCraft `fb62b9d7` `crates/plugin/src/wasm.rs` | FigCraft `0e86f359` `crates/plugin/src/sandbox.rs` |
|---|---|---|---|
| Engine config | `:181` `consume_fuel(true).enforced_limits(EnforcedLimits::strict()).set_max_recursion_depth(limits.max_recursion_depth.max(16))` | `:67-68` `Config::default()` + `consume_fuel(true)` only | `:266` `fn engine()` with `consume_fuel(true)` |
| wasmi features (Cargo) | `std, validate, stable, portable-dispatch` | workspace `Cargo.toml:73` `std, validate, wat, deterministic, portable-dispatch` | root `Cargo.toml:108` `std, stable, validate, portable-dispatch` |
| Fuel | `:17` `FUEL_SLICE = 20_000_000`; `Limits` `:21`, defaults `:45-55`: `fuel_base 50 M`, `fuel_per_sample 4 k`, `max_module_bytes 32 MiB`, `max_memory_bytes 512 MiB`, `max_total_memory_bytes 2 GiB`, `wall_time_ms 60 000`, `max_recursion_depth 1024` | `:11-12` 50 M + 20 k per pixel | `Limits` `:68`: 1 G per run, 10 k per host call, +1 per byte |
| Sliced call | `:250-274` `call()`: `call_resumable`; on `TypedResumableCall::OutOfFuel`, `deadline.check()` then refuel by `left.min(FUEL_SLICE.max(need))` | none | none |
| Start function | `:242` given at most one slice | — | — |
| Deadline | `:86-121` `Deadline`: an `Instant` **only when not wasm32**, plus a shared abort flag. **On the web build there is no time limit** | none | none |
| Memory | `:63` `parallel_bands(threads) = threads.min(max_total/max_memory).max(1)` (planning only, not enforcement); `:237` `StoreLimitsBuilder…memory_size(max_memory_bytes).memories(1).tables(4).table_elements(100_000).instances(1)` | `:14` `MAX_MEMORY 1 GiB`; `:44` `StoreLimitsBuilder…memory_size(MAX_MEMORY).instances(1)` | `:339`, same limits as PhotoCraft |
| Imports | none; `load` refuses any import (`:183-189`) | none | fixed table `IMPORTS` `:49-64`, module `figcraft` |
| Instance reuse | fresh per band (`:277`) | **`pool: Mutex<Vec<Inst>>` across calls** (`:62`, `take` `:95`, `give_back` `:102`): state can cross documents | `Instance` keeps memory across calls with fresh fuel (`:352-362`) |
| wasmi 2.0 bug | **`check_code_section` `:151`, called from `load` at `:179`**. Doc `:146-150`: a code section declaring no functions but more bytes "panicked" under strict limits. Regression test `tests/abi.rs:223` `a_code_section_claiming_no_functions_but_many_bytes_is_refused` | not guarded | not guarded |

**What this means:**
- **EffectCraft and FigCraft can be crashed today by the PhotoCraft fixture**: they use wasmi 2.0.0 without the code-section guard. EffectCraft does not set strict limits, so it may not hit the same path. Run the fixture against both apps first (P3-00). If either one panics, it is a crash fix the merge-hold exceptions allow at once ([merge hold, exceptions](../merge-hold-list.md#exceptions)).
- **`std::time::Instant::now()` panics on `wasm32-unknown-unknown`.** That is why PhotoCraft compiles the deadline out on web. The shared host takes an injected `Clock` (`ext-ops/clock.rs`) so the web build has a real deadline. On web the clock is backed by `performance.now()`, through a small dependency such as `web-time` (MIT/Apache-2.0; licence to confirm with `cargo-deny`).

## 3. Module layout and API sketches

```text
crates/ext-host/src/
  lib.rs           crate root denies; no egui, no filesystem store
  profile.rs       EngineProfile (FROZEN rules), ProfileId (hash of the rules, for cache keys)
  admission.rs     pre-parse checks (size, sections, the code-section guard), import/export table checks
  module.rs        AdmittedModule: digest + profile id + wasmi Module; ModuleCache (in memory, LRU)
  budget.rs        Budget, BudgetProfile {Preview, FinalRender, SessionCommand, Job}, FuelSlicer
  memory.rs        MemoryLedger (app-wide), LedgerLimiter (wasmi ResourceLimiter reserving from the ledger)
  instance.rs      InstanceKey {digest, contribution, document, job}, InstancePool, reset rules
  call.rs          call_sliced(): fuel slices, deadline, cancel, generation check between slices and at host calls
  linker.rs        Linker built from a contract's import table (transforms: none or `craft.log`)
  failure.rs       HostFailure, user messages, repeated-failure pause counter
  metrics.rs       per-contribution cost: fuel, wall time, peak memory, copies
  adapter/         ContractProvider, OperationMarkings, UiExtensionPoints, ProjectHooks, JobHooks; Level; checklist
crates/ext-conformance/
  src/hostile.rs   the hostile suite as a library (apps run it in their own cargo test)
  src/reference.rs reference host for transforms (used by `craft-ext test`)
  fixtures/wat/    handwritten WAT, compiled at test time with the `wat` crate
  fixtures/bin/    malformed binaries that WAT cannot express
  benches/         blur, halftone, LUT at 256², 1920×1080, 3840×2160
```

### Frozen early

```rust
// profile.rs — the rules apps cannot loosen
pub struct EngineProfile {
    pub engine: EngineId,                 // "wasmi 2.0.x"; part of every cache key
    pub features: FeatureSet,             // pinned allow-list, decided by P3-00 (no threads; no relaxed SIMD for transforms)
    pub strict_limits: bool,              // always true
    pub max_recursion_depth: u32,         // PhotoCraft uses 1024 (min 16)
    pub max_module_bytes: u64,            // 32 MiB (PhotoCraft and EffectCraft agree)
    pub max_tables: u32, pub max_table_elements: u32,   // 4 and 100 000 (PhotoCraft and FigCraft agree)
    pub max_memories: u32,                // 1
    pub max_custom_section_bytes: u64,    // 256 KiB (contributions JSON lives here)
    pub unknown_imports: Refuse,          // a type with one value; there is no "allow"
}
pub fn admit(bytes: &[u8], profile: &EngineProfile, imports: &ImportTable) -> Result<AdmittedModule, AdmissionError>;

// failure.rs
pub enum HostFailure {
    Trap { message: String },                   // the plugin's message is untrusted text: bounded and sanitised
    OutOfTime, OutOfFuel, OutOfMemory { requested: u64, ceiling: u64 },
    InvalidOutput(OutputError), Denied(DenyReason), Revoked, Cancelled,
    Admission(AdmissionError), AbiMismatch { want: u32, got: u32 }, Engine(String),
}
impl From<HostFailure> for craft_ext_ops::OpError { … }    // PluginFailed / OutOfBudget / Cancelled / Denied

// call.rs
pub struct CallLimits { pub fuel: u64, pub deadline: Deadline, pub cancel: CancelToken, pub generation: GenerationWatch }
pub fn call_sliced<P, R>(inst: &mut LiveInstance, f: &TypedFunc<P, R>, p: P, l: &CallLimits) -> Result<R, HostFailure>;
// between slices: deadline, cancel, generation; at every host call: generation, cancel, per-call fuel + per-byte copy fuel
```

### Provisional

```rust
pub struct MemoryLedger { ceiling: u64, used: AtomicU64 }          // app-wide plugin memory ceiling
impl wasmi::ResourceLimiter for LedgerLimiter {                     // reserve on memory_growing; refuse over ceiling
    fn memory_growing(&mut self, current: usize, desired: usize, max: Option<usize>) -> Result<bool, wasmi::Error>;
    …
}
pub struct InstancePool { /* keyed by InstanceKey; reuse only within the same document and job; discard on any failure */ }

// adapter/ — small traits, one per job (architecture section 21)
pub trait ContractProvider { fn contracts(&self) -> &[ContractOffer]; }                 // L1
pub trait OperationMarkings { fn inventory(&self) -> &OperationInventory; }             // L2
pub trait UiExtensionPoints { fn points(&self) -> &[ExtensionPoint]; }                  // L2
pub trait ProjectHooks { fn plugin_refs(&self, doc: DocId) -> Vec<SavedFeature>; fn placeholder(&mut self, f: &SavedFeature, why: Unavailable); } // L1
pub trait JobHooks { fn lease(&self, keys: &[ReleaseKey]) -> Result<LeaseGuard, HostFailure>; }                  // L1
pub enum Level { L0, L1, L2 }
pub fn checklist<A: ?Sized>(level: Level, report: &AdapterReport) -> Result<(), Vec<MissingHook>>; // an L0 checklist never asks for L2 hooks
```

**Generation checks.** `GenerationWatch` holds the grant generation captured at instance creation and an `Arc<AtomicU64>` epoch. The store bumps the epoch at once for a revoke in the same process. For another process it polls the state header every 250 ms (see [P1 R4](P1-store.md#7-risks-experiments-and-open-questions)). The documented bound is "the next host call after the revoke is seen".

## 4. Ordered PR list

| PR | Scope | Size | Files | Test that fails first | Mutants | Depends |
|---|---|---|---|---|---|---|
| P3-00 | **Experiment (days 1–5, a report, not merged code)**. (a) Build minimal guests with the pinned toolchain and record which WebAssembly proposals Rust enables by default on `wasm32-unknown-unknown` (multivalue, bulk memory, sign-ext, reference types, nontrapping float-to-int…). (b) Load each under wasmi 2.0 with each feature toggle, to confirm the real `Config` setters; read wasmi 2.0's docs for what the Cargo features `stable` and `deterministic` mean, since this plan does not know. (c) Run PhotoCraft's code-section fixture against EffectCraft's and FigCraft's loaders. (d) Measure fuel-slice overhead and cancel latency. (e) Run one fixture on Linux, Windows, macOS and in a browser, and compare output digests (determinism check, see [P4 section 5.3](P4-contracts-sdk.md#53-determinism-and-tolerance)) | — (report) | `docs/experiments/p3-wasmi-profile.md` | — | — | P0 |
| P3-01 | Crate scaffolds (`ext-host`, `ext-conformance`); `profile.rs`; `admission.rs` including the ported code-section guard | M | as named | `admission::code_section_count_zero_with_bytes_refused` (PhotoCraft's three variants), `admission::unknown_import_refused`, `admission::second_memory_refused`, `admission::oversize_table_refused`, `admission::disallowed_feature_refused` (one per feature P3-00 disables) | "guard skipped"; "unknown import allowed" | P3-00, P2-01 |
| P3-02 | `call.rs` sliced calls with deadline and cancel; `clock.rs` from P2 used for the deadline | M | as named | `budget::looping_start_stops` (a `(start $f)` with `loop br 0` fails `OutOfTime`); `budget::looping_export_stops`; `budget::cancel_latency_under_one_slice` | "deadline unchecked between slices"; "start function unlimited" | P3-01 |
| P3-03 | `memory.rs` ledger and limiter | M | as named | `memory::ledger_refuses_growth_over_ceiling`: four instances each under their own 512 MiB cap but together over a 1 GiB ceiling; the fourth `memory.grow` returns −1 or traps per the profile, and the ledger returns to 0 after drop | "limiter ignores ledger"; "ledger not released on drop" | P3-01 |
| P3-04 | `instance.rs` pool and reset rules; generation watch at host calls | M | as named | `instance::no_cross_document_state` (a guest that counts calls in a global gives 1, 1 across two documents, never 1, 2); `instance::trapped_instance_discarded`; `revoke::stops_at_next_host_call` | "pool ignores document"; "trapped instance returned" | P3-02, P2-05 |
| P3-05 | `failure.rs` mapping, user messages (through `craft-i18n` keys), repeated-failure pause; `metrics.rs` | S/M | as named | `failure::all_variants_mapped` (exhaustive match plus a golden message table); `failure::pauses_after_n_consecutive` | "pause never triggers" | P3-02 |
| P3-06 | **Hostile suite and reference host** in `ext-conformance`; runs on Linux, Windows and macOS, and under `wasm-bindgen-test` in a headless browser (job requested in craft-common-actions) | M | `hostile.rs`, `reference.rs`, `fixtures/` | every row of section 6, written as failing tests before P3-01..05 make them pass (kept red on a branch, then landed with each PR) | — | P3-01..05 |
| P3-07 | Adapter traits and the checklist test; an empty `L0` adapter compiles with no L1/L2 hooks | S | `adapter/` | `adapter::l0_checklist_needs_no_l2_hooks`; `adapter::l1_checklist_reports_missing_project_hooks` | "checklist demands L2 at L0" | P3-04 |
| P3-08 | Benchmarks and metrics baseline (criterion or divan, licence-checked) | M | `benches/` | — (records numbers) | — | P3-06, P4-04 (Halftone) |

**Order.** P3-00 → P3-01 → {P3-02, P3-03} → P3-04 → P3-05 → P3-06 → P3-07; P3-08 after P4-04. P3 runs beside P1 and P2.

## 5. Data and the host–guest boundary

P3 owns the **host side** of the guest ABI. P4 owns the contract-specific layout ([P4 section 5](P4-contracts-sdk.md#5-data-formats)). The host enforces these rules for every contract:
1. **Exports by name and signature.** A missing or wrong-typed export gives `AbiMismatch`, before instantiation where possible.
2. **Memory access only through bounds-checked `Memory::read`/`write`.** Every guest-supplied offset and length is checked against the current memory size, with `checked_add`. A packed `ptr | len << 32` (PhotoCraft's `pc_manifest`) is split with checked arithmetic.
3. **Every host call charges fuel**: a per-call amount plus a per-byte amount (FigCraft's pattern). Checks run in this order: generation, then cancel, then the operation.
4. **Every byte copied is counted** for metrics (architecture section 19: "counted and minimized buffer copies").
5. **Text from the guest** (logs, trap messages, errors) is bounded (4 KiB a message, 64 KiB a call) and sanitised with `ext-core::text` before display.

**Budget profiles.** The numbers come from P3-08 and the P4 benchmarks; these are placeholders.

| Profile | Fuel | Wall time | Memory per instance | Notes |
|---|---|---|---|---|
| Preview | base + per-unit × work, capped | Short, so playback stays smooth | 512 MiB | On failure: pass-through plus a badge |
| Final render | Generous | Long | 512 MiB | On failure: the export fails |
| Session command | Strict | Short; holds the document | 256 MiB | — |
| Job | Generous | Long; cancellable | 512 MiB | — |

## 6. Test strategy: the hostile suite

Each row is a test in `ext-conformance::hostile`. Apps run all of them against their own adapter, through `cargo test`.

| Case | Fixture | Expected |
|---|---|---|
| Infinite loop in start | WAT `(start)` with `loop br 0` | `OutOfTime` within deadline + 1 slice |
| Infinite loop in each export | WAT, one per contract export | `OutOfTime` |
| Memory bomb | `memory.grow` in a loop | `OutOfMemory`; ledger back to 0 |
| Huge initial memory | `(memory 65536)` | refused at admission or instantiation; no allocation attempted beyond the cap |
| Deep recursion | self-call without base case | trap; host stack intact |
| Huge tables | `(table 10000000 funcref)` | refused |
| Many functions or locals | generated binary over validation caps | refused |
| Unknown import | `(import "env" "x" (func))` | refused at admission |
| Malformed binaries | PhotoCraft code-section cases; truncated sections; duplicate sections; bad LEB128; overlong custom section | `Admission` error, never a panic |
| Trap in every callback | `unreachable` in each export | `Trap`; instance discarded |
| Invalid outputs | wrong size, NaN, out-of-range pointer, length overflow | `InvalidOutput`; nothing committed |
| Calls after revoke | revoke between slices and between host calls | `Revoked` at the next check |
| Cancel mid-work | cancel token set during a long loop | `Cancelled` within one slice |
| Concurrent instances | N instances under the ledger | none exceeds the ceiling |
| Pointer games | guest returns `ptr + len` that overflows `u32` | `InvalidOutput` |

- **Fuzzing.** A `cargo fuzz` target over `admit()`, and another over the ABI result decoder. They run on a schedule, as requested in the craft-common-actions issue.
- **Browser runs.** These are real runs, not `cargo check`. `wasm-bindgen-test` runs the suite in headless Chrome and Firefox, with the guest running on a Web Worker so cancellation can be measured (risk 10 in the implementation plan).
- **Benchmarks** (P3-08, recorded, not gated):
  - validation time;
  - cold and warm instantiation;
  - fuel per pixel;
  - cancel latency;
  - copies and peak memory;
  - for each, built-in versus plugin on the same machine.

## 7. Risks and open questions

| # | Risk | Experiment | Before |
|---|---|---|---|
| R1 | wasmi 2.0's accepted features differ from what Rust emits by default | P3-00 (a, b) | P3-01 |
| R2 | The code-section panic, or others like it, also hit EffectCraft and FigCraft today | P3-00 (c); `cargo fuzz` for 24 h on `admit()` | P3-01 (and a crash fix in the apps if positive) |
| R3 | Fuel slicing makes transforms too slow | P3-00 (d): slice sizes from 1 M to 50 M; overhead per slice | P3-02 |
| R4 | `ResourceLimiter` behaviour in wasmi 2.0 differs from this sketch (its signature, or whether a refusal traps) | Write the limiter against the real trait in P3-00 | P3-03 |
| R5 | The browser main thread blocks during a guest call | Worker fixture; measure input latency while a guest loops | Before claiming web support |
| R6 | 4K effects are too slow in an interpreter (engine re-evaluation trigger, architecture section 19) | P3-08 with Halftone and a blur at 4K, built-in versus plugin | E8 budget sign-off |

**Questions for maintainers:**
- **Q1.** The app-wide memory ceiling default. Proposed: 2 GiB on desktop, as PhotoCraft's `max_total_memory_bytes`, and 512 MiB on web.
- **Q2.** Should the hostile suite run on every PR, or on merge and on a schedule? It needs a browser job. Recommended: on every PR for `ext-host`; on a schedule for apps.

## 8. Effort and parallel work

| PR | Effort | Parallel with |
|---|---|---|
| P3-00 | 5 days (experiment) | P1-00..03, P2-00 |
| P3-01, P3-02, P3-03 | M each | P1-04..06, P2-02..04 |
| P3-04 | M | P2-05 |
| P3-05, P3-07 | S/M, S | P4-01 |
| P3-06 | M | P4-02 |
| P3-08 | M | P4-05 |

That is 7 PRs (1 S, 1 S/M, 5 M) plus the experiment report.

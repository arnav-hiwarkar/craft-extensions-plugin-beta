# Implementation plan

Status: **reconciled draft, 2026-11-10.** Two plans were written independently, one by Opus and one by GPT-6.1 Sol (high reasoning), from the same sources. They were then merged into this one; section 12 lists where they differed and what was taken. The plan builds [the architecture](architecture.md) as accepted in storytold/craft-extensions#1 and does not redesign it. Each pull request still needs its own approval.

**Per-unit plans (2026-11-10):** [docs/plugins/implementation/](implementation/README.md) has one build-ready plan per unit ([P1](implementation/P1-store.md), [P2](implementation/P2-ops.md), [P3](implementation/P3-host.md), [P4](implementation/P4-contracts-sdk.md), [P5](implementation/P5-management.md), [P6](implementation/P6-effectcraft.md), [P7](implementation/P7-catalogue.md), [P8](implementation/P8-figcraft.md), [P9](implementation/P9-photocraft.md), [P10](implementation/P10-native.md), [P11](implementation/P11-other-apps.md), [P12](implementation/P12-launch.md)). They give verified code references, PR lists with failing-first tests, formats and risks. Where this page and a unit plan differ, the unit plan is newer; [its index](implementation/README.md#corrections-to-the-reconciled-plan-and-the-architecture) lists the corrections, and the ones that change this page are applied below.

## 1. Starting point

- craft-extensions `main` `a6c5fae` builds and passes 111 tests locally (cargo 1.97.1). The workspace declares Rust 1.90 as the minimum. CI runs fmt, clippy, a core wasm check and tests on Linux and Windows. PRs from the fork `arnav-hiwarkar/craft-extensions` wait for a maintainer to approve CI.
- Jeremy8776 approved the design and fixed the first steps: a threat-model and README docs PR, then C1, C2 and C4 as separate store PRs with failing-first tests.
- **No app ships `craft-ext-store` yet.** The launcher and PhotoCraft integrations are unmerged branches. This makes the state-migration problem (section 4) much smaller, but it must be confirmed (P0).
- **EffectCraft is the largest piece of pilot work.** *Corrected 2026-11-10:* on `origin/main` `fb62b9d7` there are **317** non-test calls to `execute`, `execute_checked` or `execute_for_script`. That is 98 inside the engine, 190 in the egui UI and about 30 at entry points (CLI 10, desktop 9, web 8, automation 1, script 1). The earlier figure of 691 counted inline `#[cfg(test)]` modules ([method](implementation/P6-effectcraft.md#21-dispatcher-and-call-sites-recounted)). EffectCraft registers about 760 commands. Its plugin registry uses permanent `OnceLock` slots and leaked `&'static` metadata (`crates/effects/src/plugin.rs:393-435`), so effects cannot be unloaded today. A plugin render error returns the previous buffer, and the render pipeline drops missing effects (`crates/render/src/pipeline.rs:622-632`).
- **A CI finding:** `.github/workflows/ci.yml:34` says the core crate is checked for wasm "with and without the third-party feature", but line 35 checks only the default.

## 2. How we build

1. **Freeze meanings at the seams; keep implementations provisional.** State on disk, the manifest, the catalogue, contract inventories and the guest ABI get a schema document, golden fixtures and compatibility tests before an app relies on them. Internal Rust types can change until a second app uses them.
2. **One vertical slice before breadth.** One transform plugin goes through every step in EffectCraft: SDK, pack, install, setup, run, save, reopen and export. Only then do we widen.
3. **Every safety rule has a failing-first test and a mutant.** A rule nobody can break in a test is not built.
4. **Small PRs, one concern each.** Sizes: S is 50–250 changed lines, M is 250–600, L is 600–1,000. A larger mechanical migration is reviewed separately from behaviour changes. These are limits, not reasons to split an atomic safety change.
5. **No dual systems.** An app's old plugin system disappears in the same change set that activates its adapter. Preparation PRs may add support code, but they never activate a second system.
6. **Measure before tuning.** Budgets, engine choice and caching follow benchmark numbers.
7. **Retain on doubt.** When lease liveness or a reference is uncertain, keep the bytes. A temporary disk leak beats a broken export.

## 3. Engineering systems

| System | Decision |
|---|---|
| PR flow | One branch per review unit, from `main`, opened from the fork. Before coding: read the issue, search related PRs and recent commits, and post a claim. The PR carries parent-failing and head-passing evidence, the tested base SHA and its limits. No agent attribution. Maintainers merge and lift holds |
| Issues | One issue per PR-sized task, plus a tracking issue per unit (P1–P12) with a task checklist, owner, dependencies and open decisions. Labels by unit and area (`store`, `ops`, `host`, `sdk`, `ui`, `catalogue`, `docs`) |
| Stacks | Prefer merged prerequisites for the first sequence (docs, C1, C2, C4). Later stacks state their base and blockers; update dependents after each merge |
| CI | House rule: workflows call `storytold/craft-common-actions`, and CI changes never ride in feature PRs. Open one shared-CI work item there covering the third-party feature configuration (fixing the line 34–35 mismatch), macOS, MSRV 1.90 and the pinned author toolchain, wasm for every crate that claims web support, browser execution of host and storage fixtures, generated-file drift, deterministic builds, mutation jobs in disposable checkouts, the hostile-module suite, scheduled fuzzing and `cargo-deny` |
| Mutation harness | It edits sources in place, so it runs only in a disposable checkout and never next to normal tests. A compile failure never counts as a kill. Add mutants for: automatic enablement, deletion despite a lease, an old-state write, trust promotion, a skipped digest, a missing generation check, an explicit-target bypass, premature effects, export fallback and catalogue rollback |
| Schemas | `docs/schemas/<name>.md` for state, manifest v2, catalogue records, contract inventory and guest ABI. Each has golden fixtures under `crates/*/fixtures/` that old and new code must both read |
| Test layers | Pure core (bounded parsing, compatibility, dependency cycles) · store process tests (real OS locks, concurrent writers, lease and cleanup races, kill and restart) · hostile Wasm (handwritten WAT and malformed binaries) · contracts (whole versus tiled, formats, determinism, cancellation) · reference operations (scope, nested transactions, outbox rollback, origin) · real apps (undo, save and reopen, export, attach auth, GUI/CLI/MCP parity) · distribution (TUF test roots, expiry, rotation, asset swap, resume, offline revocation) |
| Fixtures | Versioned, with expected outputs and provenance; original test images; attribution recorded. Placeholder fidelity tests compare **raw bytes**, because serde round trips can reorder or normalise JSON |
| Benchmarks | Blur, halftone and LUT at small, HD and 4K; panel callback and geometry edit later. Record validation time, cold and warm instance cost, copies, peak memory, fuel per unit and cancel latency, on named hardware. Shared CI timing detects regressions but never sets a hard budget |
| Versions | Host platform crates share one `0.x` line. Generated guest binding crates take the contract major as their own major, as an explicit exception. Apps pin reviewed **upstream** full git SHAs, never a fork commit, with lockfiles committed. Tags until crates.io is authorized. The `craft-atomic` git dependency must be resolved before any crate is published. Wire and state versions are separate from crate versions |
| Docs discipline | A behaviour PR updates `ARCHITECTURE.md` or its schema doc and the changelog in the same PR. Generated inventories carry the app revision and digest, and are copied by reviewed PR, never fetched from a moving branch |
| Threat model | Each gate names the tests that prove it. A gate is "built" only when those tests run in CI |

## 4. P0 and P1: store foundation

PC11, 2026-11-11: the user confirmed small, independently reviewable PRs with complete EffectCraft L0/L1 integration proof. The size guidance above remains; integration verification must cover the whole selected workflow. Owners, exact PR contents, and estimates remain unresolved. This confirmation does not authorize implementation.

### P0 (decisions and docs, no code)

| Task | Output |
|---|---|
| P0.1 | **Threat-model and README docs PR:**<br>• G6 becomes declared, host-granted and generation-checked<br>• Wasm confinement and native full access are described separately<br>• G10 applies to native listings and releases only<br>• G12 is kept for P12<br>• the P7 feature switch is explained separately from public launch<br>• launcher-only wording is replaced by in-app management<br>• no store builds; source links and pinned digests; stars without telemetry<br>• the "not accepted" status is updated without implying the code exists<br>• gate IDs are kept, with a requirement-to-test mapping |
| P0.2 | Owners for each unit and a second craft-extensions reviewer; hold boundaries (the EffectCraft plugin areas stay held until P6) |
| P0.3 | Write access for the contributor, or an agreed routine for approving fork CI |
| P0.4 | **Confirm that no build shipped to users writes `state.json`**: launcher, PhotoCraft or any other. This decides the migration path in P1.d |
| P0.5 | Tracking issues for P1–P12 |

### P1 order

```text
C1 → C2 (conservative) → C4 → C7 migration gate → C5 → C8 → C2 leases + cleanup → C3
   → C10 → C11 → C9 → C6 → C13      (C14 any time; C12 belongs to P7)
```

**C7 comes before any new persistent field ships.** A schema-1 writer ignores fields it doesn't know and erases them on its next save. Keeping `"schema": 1` does not prevent that. *Clarified 2026-11-10:* frozen schema-1 code refuses to read or write a file whose `schema` is greater than 1 (`state.rs:93-95`, `store.rs:128`). So moving to schema 2 is safe against old writers; adding fields under schema 1 is not ([P1-04](implementation/P1-store.md#4-ordered-pr-list)). The permission vocabulary (P2-02) must land before P1.k and P1.m.

| # | Task | Size | Fails first | Depends |
|---|---|---|---|---|
| P1.a (C1) | Install enables nothing; explicit setup enables only the chosen app | S | Multi-app install leaves every app disabled (rewrites `store/tests.rs:52`, `:117`) | P0.1 merged |
| P1.b (C2, first step) | **Stop deleting the replaced release**, on update and on uninstall (`store.rs:229` also deletes at once). Retain it conservatively, with no lease schema yet | S | Install v1, hold its files as a reader, install v2, read v1 | C1 |
| P1.c (C4) | A file never replaces a store-installed ID; state and bytes unchanged | S | Use a scoped fixture or explicit feature-enabled test; first-party protection stays covered | C2 |
| P1.d (C7) | **Migration gate**:<br>1. capture a frozen schema-1 reader and writer fixture;<br>2. reproduce the destructive rewrite;<br>3. implement `write_version`/`read_version` (refuse writes above the writer's version, refuse reads above the reader's, checked under the lock before moving bytes, one-time migration, never turn unreadable into empty-writable);<br>4. maintainers choose the deployment boundary using P0.4 | M | Newer additive state is readable by an older supported reader; that writer refuses to change it and leaves the bytes identical | C1–C4, P0.4 |
| P1.e (C5) | Verified release record: archive digest, retained verified manifest, host-assigned source and label | M | Package metadata cannot promote its label | C7 |
| P1.f (C8) | Digest-addressed folders `<version>-<digest>` | M | Same ID and version, different bytes, never share a folder | C5 |
| P1.g (C2, full) | Leases with a **unique process instance ID** (PIDs get reused) and a reliable liveness check; a lease is acquired under the same synchronization as cleanup | M | A second process updates and removes while a leased job keeps reading | C8 |
| P1.h | Cleanup and crash recovery | M | Killing a writer at every transition leaves the old committed state or the new complete release; cleanup never deletes referenced bytes | P1.g |
| P1.i (C3) | Explicit version choice, older allowed; the same digest is idempotent; an ordinary update never downgrades | S/M | Explicit v2 → v1 succeeds | C5, C8 |
| P1.j (C10) | ID length bound (checked against GitHub limits and Windows path length) | S | Longest agreed ID installs on Windows; one byte more fails | Length decision |
| P1.k (C11) | `docs/schemas/manifest-v2.md`, then the parser beside schema 1 | M | Schema 1 keeps its meaning; unknown critical schema fails before interpretation | C10 |
| P1.l (C11) | Contribution and permission validation | M | Duplicate contribution IDs, a custom section that disagrees with the manifest, forbidden imports | P1.k, P2 vocabulary |
| P1.m (C9) | Per-app setup and grants (with generation) | M | An update keeps choices, leaves new contributions off and holds widened rights pending; other apps are unaffected | C7, C11 |
| P1.n (C6) | One `eligibility()` function | M | A tampered file, unsupported contract, stale grant or policy denial returns a structured reason before loading | C5, C9, P2 |
| P1.o (C13) | Revocation enforcement, including file installs by digest | M | A revoked digest from a file cannot be enabled or loaded | C6 |
| P1.p | Data dependencies; private storage | M each | A cycle or conflict activates nothing; another package cannot read private storage | P1.k, P1.m, P1.g |
| P1.q (C14) | Repository URL | S | Check the existing `chore/repo-url-storytold` branch first | — |

Do **not** bundle C5, C6, C8 and C13 into one "verified release" PR. Each has its own rejection point.

**AR01 confirmed, 2026-11-12:** retain all referenced releases: defaults, effective app selections, pins, pending-consent references, required dependency releases, and live leases. Cleanup removes only unreferenced releases. This refines T08's earlier current-plus-leases choice and is reflected in the architecture/P1. The exact effective-release and chained pending-consent state representation remains open (AR02).

## 5. Interfaces to settle first

Dependency direction:

```text
ext-core ← ext-ops
ext-core ← ext-store
ext-core + ext-ops ← ext-contracts
ext-core + ext-ops + ext-contracts ← ext-host        (no egui, no native filesystem store)
ext-host + ext-contracts ← ext-conformance
ext-store + ext-catalog ← management actions ← manager UI / app CLI
app adapter → platform crates + app engine
```

| Component | Settle early | Keep provisional |
|---|---|---|
| `ext-core` | Release key (ID + version + 32-byte digest), contribution identity, manifest dispatch, compatibility outcomes | Optional future fields |
| `ext-ops` | Mandatory invocation, principal propagation, markings, resolved-target authorization, errors, transaction ownership, exit codes | Scheduler and transaction internals |
| `ext-contracts` | Inventory encoding, version rules, extension negotiation, raster buffer, colour and region semantics | Temporal beyond the pilot; audio; geometry |
| `ext-host` | Admission profile, verified-module boundary, failure model, budget enforcement, small adapter traits | Engine internals, cache format, budget values |
| Conformance | Fixture format, literal expected outputs, suites per level | Runner details |
| SDK | Contract encoding, parameter and state declarations, custom-section identity | Macro ergonomics |
| CLI and management | Command meanings, structured errors, deterministic pack; action and result types shared by GUI and CLI | Output layout, screens |
| Catalogue client | Verified snapshot and pinned-download boundary | TUF adapter, transport |

Rules that the APIs must enforce:
- An `Invocation` has private fields, is built only by trusted entry points, and **can never be deserialized from guest input**.
- `$N` batch references and explicit targets are resolved, boundedly and without side effects, before permissions are computed.
- A verified release carries checked bytes or a checked handle, not a bare path.
- Module caches key on engine and profile identity as well as digest.
- Generation, cancellation and revocation are checked at host calls, at job start and between fuel slices.
- A string-named dynamic operation call is never exempt from per-call authorization. Session imports resolve at load.
- Adapter traits are small and per job. An L0 checklist never requires L2 hooks.

## 6. Units P2–P12 as PR-sized work

### P2 operations kit (`crates/ext-ops`)

1. **`docs/design/ops.md`**, reviewed with the EffectCraft and FigCraft maintainers before any code.
2. **Invocation and descriptors (M).** Fails first: a plugin reaches a Management, Core or Internal operation; it must be denied before the handler runs.
3. **Resolved authorization (M).** Fails first: an explicit target or resolved batch reference escapes the captured selection.
4. **Transaction protocol (M)** with a reference executor. Fails first: nested edits make several undo steps, or a failed edit runs its queued effects.
5. **Jobs and origin (M).** Fails first: a stale job commits; queued work loses its principal; event recursion passes its cap.
6. **Inventory export and breaking-change diff (M).** Fails first: an unmarked operation exports, or a breaking change passes without a major bump.
7. **Capability brokers for files, network and clipboard (one M PR each).** Each lands before its capability is advertised. No accounts in the first release. *2026-11-10:* P2 defines only the traits and the network IP policy; the implementations go in `ext-host` before P8 ([P2-09](implementation/P2-ops.md#4-ordered-pr-list)).

### P3 runtime and hostile suite (`crates/ext-host`, `crates/ext-conformance`)

1. **Experiment, days 1–5.** Compile minimal guests with the pinned toolchain, run them under the exact wasmi profile, and record accepted features, deterministic-mode cost and fuel-slice overhead. **This freezes the SDK target features.**
2. **Engine profile (M).** Fails first: an unsupported feature, unknown import, malformed code section or oversized table reaches execution. Port PhotoCraft's malformed-section regression and check whether its workaround is still needed.
3. **Fuel, deadline and cancel (M).** Fails first: an infinite start function or callback hangs.
4. **Total memory accounting (M).** Fails first: concurrent jobs each stay within their own limit but exceed the app-wide ceiling.
5. **Instance lifecycle (M).** Fails first: state leaks between documents, or a trapped instance returns to the pool.
6. **Hostile suite and reference host (M)**, run on Linux, Windows and macOS, plus **real browser execution** (a `cargo check` is not proof).
7. **Metrics and baseline benchmarks (M).**

### P4 contracts, SDK, CLI

1. **Inventory format and generator (M).** Fails first: regeneration differs, or a removal passes without a major bump.
2. **`raster.filter@1` (M)**, reviewed by maintainers of two consuming apps. It defines exact **tolerances and supported formats**; "deterministic" alone does not guarantee identical floating-point results across platforms.
3. **Guest SDK (M).** Fails first: an unlabelled control compiles, or the emitted descriptor disagrees with the contribution.
   - **Decide the guest binding technique first.** WebAssembly exports need `extern "C"` functions and raw memory access, which the house "no `unsafe`" rule forbids. Either generated code confines it under a named exception, or a binding approach without handwritten `unsafe` is used.
4. **Generated raster bindings and the Halftone example (M).**
5. **`craft-ext create/check/inspect/pack/verify/test` (several M PRs).**
   - `check` **warns** about permissions it can infer from imports.
   - The runtime **enforces** them, because session operations go through one dispatcher import and cannot be fully inferred.
6. **Deterministic `prepare-release` (M).** Wasm and archive digests are compared separately, so any nondeterminism can be located. It never publishes.
7. **Isolated `dev` loop (M).** Fails first: dev writes to the production store or reuses production grants.

### P5 and P5b management

1. **Shared management actions (M).** Fails first: CLI and GUI setup differ.
2. **App `plugin` command library (M).**
3. **Manager UI screens (several M PRs)** on `craft-ui` and `craft-i18n`, including a keyboard and screen-reader pass.
4. **Problems, safe mode and bisect (M).** Define how restoration works if the app crashes again during bisect.
5. **Setup export and import (M).** Grants and folder tokens are never restored silently.
6. **Browser-local store (M).**
7. **Data add-ons and large resources (one M PR per importer or download behaviour).**
8. **Diagnostics (M):** bounded logs, plugin console, report preview without tokens or content.

**P5b rollout conflict, for the maintainers to decide:**
- In the **three pilot apps**, an "early" L0 adapter would remove their working plugin systems months before L1 replaces them, because each app's old system goes when its adapter activates.
- **Recommendation:** in pilot apps, L0 lands as part of each app's atomic cutover (P6, P8, P9).
- In **apps with no executable plugin system today**, L0 ships early as planned.

### P6 EffectCraft

| PR | Scope | Fails first |
|---|---|---|
| E0 | Recount call sites on `main`; inventory commands and authority-changing settings; capture legacy project fixtures (can start in week 1 with the EffectCraft maintainer) | — |
| E1 (M) | Descriptor kinds and markings for every command; scripting and automation permissions split out of `prefs.set` | Generic preference writes cannot change scripting authority; CI fails on an unmarked command |
| E2 | `Invocation` made mandatory in three steps:<br>1. add `execute_as(&Invocation, …)`; the old `execute` wraps it as `User(Gui)` and is blocked outside the UI crate with clippy `disallowed-methods`;<br>2. move the ~30 entry points with their real principals;<br>3. turn the 98 engine-internal calls into nested `call`s that take the invocation from a session-held stack and are re-authorized, move the 190 UI calls (mostly through the `invoke`, `run_engine` and `panel_kit::exec` wrappers) in batches by panel, and delete the old `execute`. Detail: [P6 E2a–E2e](implementation/P6-effectcraft.md#4-ordered-pr-list) | Caller-less production entry points no longer compile; normal command fixtures behave the same |
| E3 (L) | Dispatcher-owned transactions on `session/edit.rs`; batches, journal, learning and events carry origin | Two nested edits make one undo step; a failure in step two restores project and history and runs no queued effect; automation does not train habits |
| E4 (M) | Attach over a user-only socket or pipe with a token; TCP control server removed (a security fix; can land early) | Missing or wrong token fails; no TCP listener by default |
| E5 (M) | Unloadable plugin registrations replacing `OnceLock` slots and leaked metadata, without touching built-in performance | Repeated register and unregister frees metadata; an export keeps its old registration while new work uses the update |
| E6 (M) | Placeholder representation near `crates/project/src/props.rs`, keeping removed-system data as opaque raw bytes | Removed-system effect data and keyframes survive open and save byte for byte |
| E7 (M) | Render error propagation **during** export, not only preflight; replaces "return the previous buffer" and the silent drop | A missing, failed or blocked plugin prevents final output; a trap mid-export leaves no completed file; preview shows a labelled fallback |
| E8 (L) | **Atomic cutover:** activate `effectcraft-plugins`, the manager and `plugin` commands; remove the old plugin crate, Plug-in Library and helpers. Review by directory, with a deletion checklist | GUI, fresh CLI and MCP load the same configured package; the old runtime and index commands are gone |
| E9 (M) | Basic temporal hooks, cache identity, safe points, safe mode, diagnostics | Backward seek resets state; update during export keeps the leased release |
| E10 (M) | Bake, include-plugins, full pilot acceptance | Bake keeps a disabled original; a carried package never runs on open |

**Before E8, the maintainers must accept two consequences.** Old plugins stop running (release notes only, as decided), and **native helpers stay unavailable until P10**.

### P7 catalogue (alongside P6–P9)

PC08, 2026-11-11: serve signed catalogue metadata through GitHub Pages initially; a custom domain comes later and is not an initial P12 prerequisite. Keep maintainer-controlled TUF signing without requiring separate publisher keys; source/namespace checks and pinned hashes remain. The [P7](implementation/P7-catalogue.md) and [P12](implementation/P12-launch.md) plans reflect this change. Deferred PC04, PC05, and PC07 do not replace existing feature, checking, or review policies.

1. **Metadata and namespace rules (M).** Fails first: the same login with a different numeric owner ID inherits ownership.
2. **Pinned downloader (M).** *Corrected 2026-11-10:* reuse craft-libs `craft-update`'s `Http::download` (`crates/update/src/http.rs:263`), which is already shared, requires a SHA-256, resumes and enforces `https_only`. Do not extract the launcher's copy, whose download agent does not set `https_only` ([P7-02](implementation/P7-catalogue.md#4-ordered-pr-list)). The digest comes from trusted catalogue metadata only.
3. **TUF adapter and C12 (M)**, after a spike on candidate clients (for example `tough` and `rust-tuf`) covering maintenance, licence, expiry, rotation and rollback fixtures, on native and web.
4. **Secretless bot (several M PRs).** Fails first: wider permissions or a native release qualifies for auto-merge.
5. **Protected publication and the 48-hour hold (M)**, through shared CI.
6. **Revocation sync (M).**
7. **Browse and ranking (M).** Fails first: an official fuzzy match outranks a non-official exact match, or a search sends anything over the network.
8. **Enablement gate (S)**, behind the `third-party` feature.

The store stays offline. Catalogue downloads and guest network permission are separate policies.

### P8 FigCraft, P9 PhotoCraft, P10 native, P11, P12

- **P8:**
  1. The declarative UI schema, tool gestures, canvas bounds and object persistence, tested in the reference host first.
  2. Migrate `crates/engine/src/session/exec.rs` to mandatory invocations.
  3. Project and job hooks.
  4. **Atomic cutover**, deleting manifest API levels 1–6 (*corrected 2026-11-10*: `crates/plugin/src/manifest.rs:11` has `API_VERSION = 6`, not 4) and the old `crates/plugin/src/sandbox.rs` host path.
- **P9:**
  1. Raster band and format conversion (alpha, precision, selection, overlap, CMYK and Lab refusal).
  2. Smart-filter references and export preflight.
  3. **Atomic cutover** removing the old filter ABI.
  4. The **unchanged** Halftone artifact passes in both apps, with the same digest recorded in both reports. It compares identical normalised contract inputs, not identical appearance.
- **P10:** a bounded, versioned helper protocol; process lifecycle and revocation termination; labels; the EffectCraft native adapter; catalogue enforcement of **two distinct maintainer approvals**. Process isolation contains crashes, not malice. It is never described as a security guarantee.
- **P11:** each app gets an inventory, hooks, persistence, atomic replacement if needed, and headless plus GUI proof. FilmCraft is next. Real-time audio stays gated on SoundCraft evidence.
- **P12:** author guide, policy and runbooks; a root-rotation rehearsal; the **outside security review**, whose findings become narrow tested PRs; the maintainers' launch decision.

## 7. Critical path and parallel queues

```text
P0 → docs → C1 → C2 → C4 → C7 gate → C5 → C8 → leases/cleanup → setup/grants → eligibility ─┐
P2 ops (after P0, against agreed identity and permission types) ───────────────────────────┤
P3 experiment → host → hostile suite → P4 raster/SDK → P5 management ──────────────────────┼→ P6 E8 → P8 → P9 → P10 → P12
E0 inventory, E1 markings, E2 invocation, E4 socket (EffectCraft, from week 1) ─────────────┘
P7 catalogue alongside P6–P9 once P1 and the validator are ready
```

With one main contributor, these are queues, not simultaneous work. Use review waits for fixtures, experiments and inventories, never to start dependent persistence work. **Do not build sixteen adapters against an unsettled P4/P5 API.**

## 8. Risks, ranked, with the smallest experiment

| # | Risk | Experiment | Deadline |
|---|---|---|---|
| 1 | A schema-1 writer destroys new state | Frozen old writer against a state with new fields; compare bytes (P1.d) | Before any new state write |
| 2 | Nested command transactions leave history, caches or effects committed | EffectCraft two-edit batch whose second step fails, with a queued file output | Before E3 |
| 3 | The effect registry cannot unload safely | Register, render and unregister repeatedly while an old export holds a reference; measure retained memory | Before E5 |
| 4 | wasmi versus toolchain feature mismatch | P3 step 1 | Before the SDK template freezes |
| 5 | Validation, start functions or host calls escape fuel cancellation | Malformed module, looping start and blocking broker fixtures | Before P3 exit |
| 6 | wasmi too slow at 4K | The same effect and input through the built-in and plugin paths, with copies separated from compute | Before E8 budget sign-off |
| 7 | The guest SDK needs `unsafe` | Build the smallest guest; inspect the generated export code against the lints | Before P4.3 |
| 8 | No suitable TUF client | P7 step 3 spike | Before P7.3 |
| 9 | Browser downloads of GitHub assets fail | A real browser fetch through the redirects | Before advertising web catalogue installs |
| 10 | Web cancellation blocks the event loop | A looping guest in a worker; time from cancel to stop | Before claiming web support |
| 11 | egui accessibility gaps | Keyboard and screen-reader pass over setup, permissions and revoke | Before P5 and P8 acceptance |
| 12 | Flatpak cannot share the store folder | Two sandboxed apps plus the launcher with only the folder permission | Before P5b packaging |
| 13 | `check` overstates what it can infer | A guest with dynamic operation names and parameter-dependent file use; runtime must catch it | Before P4 `check` claims |
| 14 | Placeholder data not preserved exactly | Non-canonical JSON and binary state through a missing-plugin cycle | Before E6 |
| 15 | Pins and pending consent conflict with cleanup | Two apps on different releases, one waiting on approval; run cleanup | Before P1.h |
| 16 | Review and key-holder capacity | Name people; rehearse a native approval and a key rotation | Before P7 operations |

Gaps to close at their owning PR, each with a testable interpretation:
- raster tolerances;
- migration budgets;
- large-resource limits (today's archive caps do not cover external resources);
- remembered-folder tokens per OS;
- job behaviour after an app restart (disconnect is specified, restart is not);
- bisect restoration after a second crash;
- what "no longer available upstream" means for compatibility display.

## 9. People and decisions

| Responsibility | Who |
|---|---|
| Platform implementation | Contributor; a platform maintainer approves shared contracts and merges |
| Store and security review | Someone experienced with persistence, filesystem races and hostile input. **Not all reviews default to Jeremy8776**; approving the design is not unlimited review capacity |
| Operations migration | Platform engineer with the EffectCraft engine maintainer |
| Raster contract | Maintainers of at least two consuming apps |
| Manager | A UI and accessibility reviewer plus each app's integration owner |
| Catalogue | Repository owner, incident owner, signing operators, three root-key custodians |
| Native releases | Two distinct maintainers per release |
| Launch | Independent outside reviewer and an accountable maintainer |

Keep one or two substantive PRs waiting on the same specialist at a time, and schedule transaction, schema and cryptography reviews explicitly.

Decisions due:

| When | Decisions |
|---|---|
| **P0** | Owners, holds, write access, the shipped-writer question (P0.4) |
| **Before new state writes** | The deployment boundary |
| **Before C10** | ID and path limits |
| **Before P4** | Raster semantics, MSRV and toolchain, guest binding policy |
| **Before P5b** | Pilot L0 timing |
| **Before E8** | Old-plugin and native-helper consequences |
| **Before P7** | Repository, Pages endpoint, TUF dependency, key holders, enablement; custom domain later (PC08) |
| **Before publication** | Crate ownership |
| **Before P12** | Outside-review scope and launch authorization |

## 10. First two weeks

| Day | Main contributor | In parallel, if staffed | Result |
|---|---|---|---|
| 1 | Docs PR (P0.1); tracking issues; ask about write access, a second reviewer, and P0.4 | Maintainers name owners and holds | Docs PR open |
| 2 | C1 failing test and fix | E0: EffectCraft legacy project fixtures | C1 PR |
| 3 | C2 conservative retention | E0: command inventory, authority-changing settings | C2 PR |
| 4 | C4 | P3 step 1 wasmi experiment | C4 PR |
| 5 | Reproduce the old-writer data loss; compatibility matrix | Shared-CI work item in craft-common-actions | Migration evidence |
| 6 | Compatibility-aware read and write rules | P2 design note | P1.d draft PR |
| 7 | Cross-process old and new behaviour; crash boundary | P3 admission and hostile fixtures | Migration gate review |
| 8 | C5 | Raster contract fixtures | C5 PR |
| 9 | C8 | P3 fuel and cancel | C8 PR |
| 10 | Review queue; plan leases; estimate dates from measurements | Benchmark baseline; pilot inventories | Week-2 evidence; next queue |

**Dates beyond two weeks are estimated only after three measurements:** the wasmi benchmark, the EffectCraft command inventory and the migration evidence. They decide the real size of P1, P3 and P6.

## 11. Milestones

| Milestone | Exit criteria |
|---|---|
| M0 approved execution | Owners named; docs aligned; holds explicit |
| M1 safe initial store | C1, C2, C4 merged with parent-failing evidence |
| M2 P1 complete | Migration, verified releases, leases and cleanup, setup and grants, eligibility; crash and concurrency tests pass |
| M3 executable foundation | P2 denial and transaction proofs; P3 hostile suites on every claimed platform |
| M4 author loop | Halftone builds reproducibly, validates, passes the raster suite, runs in an isolated dev store |
| M5 management and L0 | Local lifecycle through shared GUI and CLI; L0 checklists pass in non-pilot apps |
| M6 EffectCraft | Atomic replacement; parity, authenticated attach, placeholders, safe unload, truthful export |
| M7 catalogue | TUF, hold, pinned downloads, revocation, protected publication |
| M8 FigCraft and PhotoCraft | Session UX proofs; the unchanged Halftone passes both raster adapters |
| M9 native | Helper failure containment, consent, termination, two-maintainer enforcement |
| M10 launch | Outside review complete, blockers fixed, keys and recovery rehearsed, maintainers authorize |

## 12. Where the two plans differed, and what was taken

| Topic | Opus draft | Sol plan | Reconciled |
|---|---|---|---|
| Schema-1 writers | Assumed versioning solves mixed versions | Old binaries cannot honour new version fields; needs a gate and a deployment decision | Sol, plus Opus's finding that no app ships the store yet (P0.4 makes it a confirmation, not a crisis) |
| First C2 PR | Leases in one step | Conservative retention first, leases after C7 | Sol |
| Retained releases | "Current only" taken literally | Pins, pending consent and leases must also be retained; the design wording conflicts | Sol, flagged as a design clarification |
| Lease identity | PID with a liveness check | PIDs are reused; needs a unique instance ID | Sol |
| P5b in pilot apps | Early everywhere | Early L0 would remove pilots' working plugins too soon | Sol; maintainer decision |
| EffectCraft invocation migration | Wrapper, lint, batches | One mandatory mechanical PR | Opus steps with Sol's end state (old `execute` deleted) |
| EffectCraft detail | Call-site counts; socket early | Registry `OnceLock` and leaks, mid-export failure, native helpers gone until P10 | Both |
| CI | Changes in their own PRs | Through `craft-common-actions`; MSRV; browser runs | Sol (house rule 11), plus the wasm-feature mismatch Opus found |
| SDK and `unsafe` | Not seen | Wasm exports need `unsafe`; needs a policy | Sol |
| Permission inference | `check` computes permissions | Dynamic dispatch limits inference; runtime enforces | Sol |
| Determinism | Assumed | Floating-point needs tolerances | Sol |
| Downloader extraction | `install.rs` routine | The real code is `http.rs::Http::download` | Sol; *superseded 2026-11-10*: reuse craft-libs `craft-update` instead |
| Review load | Second reviewer named | Don't default everything to one maintainer; schedule specialist reviews | Both |
| Issues, labels, schema docs folder | Detailed | Lighter | Opus |
| Dates | Two-week table | Two-week table; dates only after measurements | Sol's caution, both tables merged |

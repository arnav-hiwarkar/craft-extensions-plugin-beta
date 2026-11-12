# Unit implementation plans

Status: **build-ready plans, 2026-11-10.** One file per delivery unit of [the architecture](../architecture.md#22-delivery-plan). They deepen [the reconciled implementation plan](../implementation-plan.md), which stays the top-level view. They do not redesign anything. Every pull request still needs its own approval, and nothing here authorizes repositories, publication or releases.

## Files

Architecture review, 2026-11-12: the [walkthrough](../design-sessions/system-walkthrough.md) explains workflows/components/stages; the [correction register](../design-sessions/architecture-review-2026-11-12.md) distinguishes source corrections, user-approved AR01 retention, and unresolved contracts. P1 retention/manifest assumptions, P2 consent wording, P3/P4 proof qualifications and P6 dependency gates are reconciled locally. The unit labels and old PR lists are not a fully finalized MVP activation graph. No implementation or runtime verification was performed.

Post-call reconciliation, 2026-11-11: PC01 selects EffectCraft with one manually installed L0 data package and one L1 transform, including save/reopen and failure handling. PC02 confirms validated package import followed by explicit enablement; PC03 confirms shared package files and separate app setup. P5 and P6 carry the affected scope notes. The remaining [post-call decisions](../design-sessions/post-call-reconciliation.md) must be resolved before treating the existing sequence as a finalized MVP plan. No implementation is authorized.

PC08 changes initial catalogue serving to GitHub Pages, with a custom domain later, and retains maintainer-controlled TUF signing without separate publisher keys. P7 and P12 reflect that choice. PC09 verifies egui and removes the unfinished theme framework as a prerequisite for the manual MVP, with later token adoption required; P5 records that dependency. PC04, PC05, and PC07 remain deferred, preserving existing milestone, checking, and review requirements.

| Unit | Plan | Depth | Repositories |
|---|---|---|---|
| P0 | Section [P0 below](#p0-decisions-before-code) | — | Maintainers |
| P1 | [Store foundation and manifest schema 2](P1-store.md) | Deep | craft-extensions |
| P2 | [Operations kit](P2-ops.md) | Deep | craft-extensions |
| P3 | [Host runtime and hostile suite](P3-host.md) | Deep | craft-extensions |
| P4 | [`raster.filter@1`, guest SDK, `craft-ext`, Halftone](P4-contracts-sdk.md) | Deep | craft-extensions |
| P5, P5b | [Manager, `plugin` CLI, L0 in every app](P5-management.md) | Medium | craft-extensions, craft-libs, each app |
| P6 | [EffectCraft pilot](P6-effectcraft.md) | Deep | EffectCraft, craft-libs, craft-extensions |
| P7 | [Catalogue, bot, TUF, hold, revocation](P7-catalogue.md) | Medium | craft-extensions, new catalogue repository |
| P8 | [FigCraft](P8-figcraft.md) | Outline+ | FigCraft, craft-extensions |
| P9 | [PhotoCraft](P9-photocraft.md) | Outline+ | PhotoCraft |
| P10 | [Native helper protocol](P10-native.md) | Outline | craft-extensions, EffectCraft, catalogue |
| P11 | [Other apps and themes](P11-other-apps.md) | Template | Each app |
| P12 | [Public launch](P12-launch.md) | Checklist | All |

Each plan has the same eight sections:
1. goal and exit criteria;
2. verified code areas;
3. layout and API, marked frozen or provisional;
4. ordered PRs, each with a failing-first test, mutants and dependencies;
5. data formats;
6. test strategy;
7. risks and questions;
8. effort and parallel work.

## Revisions read

Nothing was built, run, committed or pushed. Every line reference was read with `git show`, `git grep` or the file itself at these revisions:

| Repository | Revision | Note |
|---|---|---|
| craft-extensions | `main` `a6c5fae` (read through PR #1 head `5ba16aa`; no code difference) | Tests not run; the 111-test figure is from the reconciled plan |
| craft-libs | `fd5e2fe` (this worktree) | — |
| EffectCraft | **`origin/main` `fb62b9d7`** | Local `main` `ce9eaec6` is 22 commits behind; the checkout is on `ci/windows-leg` |
| FigCraft | `main` `0e86f359` | — |
| PhotoCraft | `main` `655295b2` | — |
| craft-launcher | `origin/main` `753a710` | Local `main` is 38 commits behind |

## Conventions

- **PR IDs.** Each ID names its unit and its order: P1-04, P2-05, E3a (EffectCraft), F2 (FigCraft), PH1 (PhotoCraft), CL-1 (craft-libs) and A1 (the attach crate).
- **Sizes.** S is 50–250 changed lines, M 250–600, L 600–1,000. Mechanical migrations are reviewed separately from behaviour changes.
- **Every PR follows the house rules:**
  - an issue and a claim first;
  - a test that fails on the parent and passes on the head, with both runs reported;
  - mutants added to the mutation harness, which runs only in a disposable checkout;
  - no `unsafe` and no panics outside named exceptions;
  - no CI changes in feature PRs; the CI work items go to `storytold/craft-common-actions`;
  - no agent attribution.
- **Frozen early versus provisional.** A frozen meaning gets a schema document under `docs/schemas/`, golden fixtures and compatibility tests before any app relies on it. Provisional Rust types may change until a second app uses them.

## P0: decisions before code

| Task | Output | Blocks |
|---|---|---|
| P0.1 | Threat-model and README docs PR, listed as [P1-00](P1-store.md#4-ordered-pr-list) | P1-01 |
| P0.2 | Owners per unit; a second craft-extensions reviewer; a store/security reviewer who is not by default the same maintainer | All |
| P0.3 | Write access, or an agreed routine for approving CI on fork PRs | All |
| P0.4 | Confirm that no shipped build reads or writes `state.json`. **Source check done:** no `craft-ext` dependency on the `main` refs of EffectCraft, FigCraft, PhotoCraft or the launcher; only the unmerged launcher branches `origin/feat/extensions` and `origin/test/ext-plus-ai` use it. Release artefacts were not checked | P1-04 |
| P0.5 | Tracking issues P1–P12 | — |
| P0.6 | Open the shared-CI issue in craft-common-actions: the `third-party` feature check (`ci.yml:34-35` says "with and without" but runs only the default), macOS, MSRV 1.90, browser runs, the mutation job in disposable checkouts, the reproducible-build job, fuzzing, `cargo-deny`, generated-file drift | P3-06, P4 X1 |

## Dependency graph and critical path

```text
P0 ─┬─ P1-00..03 ─ P1-04 (state v2) ─ P1-05..06 ─┬─ P1-07 leases ─ P1-11 cleanup ──────────────┐
    │                                           ├─ P1-08 ─ P1-10 manifest v2 ─ P1-12 setup ─ P1-13 eligibility ─ P1-14 ─┐
    ├─ P2-00 ─ P2-01 ─ P2-02 permissions ───────┘ (into P1-10, P1-12)                                                    │
    │          └─ P2-03 ─ P2-04 ─ P2-05 dispatch/txn ─ P2-06, P2-08 inventory ───────────────┐                          │
    ├─ P3-00 experiment ─ P3-01..05 ─ P3-06 hostile suite ─ P3-07 adapter traits ──────────┤                          │
    │                     └─ P4-00 ─ P4-01 ─ P4-02 raster.filter@1 ─ P4-03 ─ P4-04 Halftone ─ P4-05..10 ─┐              │
    │                                                                                    P5-01..03 ──┤              │
    └─ EffectCraft: E0, E5 (week 1) · A1 → E4 (security, early) · E1 → E2 → E3 → E6 → E7 ────────────┴─ E8a ─ E8b ─ E9..E11
                                                                                                         │
                                       P7 (after P1-10, P1-14) runs beside P6..P9 ───────────────────────┴─ P8 ─ P9 ─ P10 ─ P12
```

**The critical path** runs P2-05 (the dispatcher) → E2a/E3a. EffectCraft's migration is the longest single queue: about 22 PRs. Beside it, P1-13 (eligibility), P3-07 and P4-04 must all be merged before E8a.

**Can start in week 1 without waiting:**
- P1-00..03, P2-00, P3-00 and P4-00 (experiments);
- E0 (inventory and fixtures) and E5 (the unloadable registry, which changes no behaviour);
- the A1 attach-crate experiment;
- CL-1..CL-4 in craft-libs.

## Corrections to the reconciled plan and the architecture

The reconciled plan has been updated to point here. These are the findings that change it:

| # | Finding | Evidence | Where handled |
|---|---|---|---|
| 1 | **The count of EffectCraft `execute` call sites is 317, not 691**: 190 UI, 98 engine, about 30 at entry points. The old figure counted inline `#[cfg(test)]` modules | Method and table in [P6 section 2.1](P6-effectcraft.md#21-dispatcher-and-call-sites-recounted) | P6 sizing |
| 2 | **A frozen schema-1 writer cannot erase schema-2 state**: `State::read` refuses `schema > 1` before any write. The danger exists only if new fields ride under `"schema": 1`. Architecture section 11's "keeps what current apps read in place" is the unsafe option | `ext-store/src/state.rs:93-95`, `store.rs:128` | [P1-04](P1-store.md#4-ordered-pr-list); wording for the architecture proposed |
| 3 | **Uninstall also deletes release files at once**, not only update (C2) | `ext-store/src/store.rs:229` | P1-02 |
| 4 | **Use the shared downloader in craft-libs, `craft-update`**: `Http::download`, with `https_only`. Do not extract the launcher's copy, whose download agent sets no `https_only` | craft-libs `crates/update/src/http.rs:263`, `:73-116` | [P7-02](P7-catalogue.md#4-ordered-pr-list), P5-08 |
| 5 | **FigCraft's old plugin API has levels 1–6, not "1–4"** | FigCraft `crates/plugin/src/manifest.rs:11` | [P8](P8-figcraft.md) D1; architecture sections 22 and 25 should say "API levels 1–6" |
| 6 | **The CLI and MCP load no plugins in EffectCraft today**, so GUI/CLI/MCP parity is new behaviour | `apps/effectcraft/src/main.rs:285-293`; no plugin loading in `apps/effectcraft-cli` | E8b |
| 7 | **EffectCraft loses the schema-1 effect-space shift for missing effects** | `crates/engine/src/session/edit.rs:351` | E6 |
| 8 | **Export writes the final path directly; there is no temporary name** | EffectCraft `crates/export/src/out.rs:26` | E7b |
| 9 | **EffectCraft and FigCraft may be crashable by the wasmi 2.0 code-section input** that PhotoCraft guards against | PhotoCraft `runtime.rs:146-151`, `tests/abi.rs:223` | P3-00 (c); a crash fix may merge under the hold exceptions |
| 10 | **`Instant::now()` panics on `wasm32-unknown-unknown`**; PhotoCraft compiles its deadline out on web | PhotoCraft `runtime.rs:86-100` | P2 `clock.rs`, P3-02 |
| 11 | **WebAssembly output is bit-reproducible once NaN is refused**, so tolerances apply to tile invariance and host conversions, not to cross-platform runs | WebAssembly core float semantics; to confirm in P3-00 (e) | [P4 section 5.3](P4-contracts-sdk.md#53-determinism-and-tolerance) |
| 12 | **The permission vocabulary must land before manifest v2**, so it moves into `ext-core` (P2-02) | — | P1-10, P1-12 dependencies |
| 13 | **The shared libraries lack plugin hooks**: craft-keymap has no "suggested" layer, craft-menu no extension points, craft-i18n no runtime catalogue, and craft-ui no AccessKit helpers | craft-libs `crates/keymap/src/keymap.rs:500`, `crates/menu/src/model.rs`, `crates/i18n/src/lib.rs:63`, `crates/ui/src/combo.rs:599` | [P5 CL-1..CL-4](P5-management.md#4-ordered-pr-list) |
| 14 | **EffectCraft has four grouping mechanisms** that the dispatcher transaction replaces | `cmd/app_more.rs:20`, `cmd/batch.rs:66/:123`, `background/jobs.rs:311`, `crates/script/src/runtime.rs:248-280` | E3a–E3b |
| 15 | **PhotoCraft already authenticates automation with a token**; EffectCraft and FigCraft do not | PhotoCraft `crates/automation/src/security.rs:79-112` | A1 basis |
| 16 | EffectCraft registers about **760** commands, not the "~2,000" its own comment says | static count from `crates/engine/src/cmd.rs` macros; `:279` comment | E1a sizing |

## Decisions needed from maintainers (consolidated)

| When | Decision | Recommendation | Plan |
|---|---|---|---|
| P0 | Owners, reviewers, write access, shipped-writer check | — | P0 |
| Before P1-04 | Deployment boundary: move to state schema 2, so unupdated apps stop seeing installs | Accept (nothing shipped) | P1 D2 |
| Before P1-04 | Legacy v1 installs (no digest) | Require a reinstall | P1 D3 |
| Before P1-08 | ID cap | 128 bytes | P1 D1 |
| P1-16 | Private storage cap | 64 MiB per package per app | P1 D4 |
| Architecture text | Retained-set wording; schema-1 sentence; "API levels 1–6" | Accept | P1 D5, P8 D1 |
| Before P2-04 | MCP headless principal; script principal and management; chain and depth caps | Automation; no; 8 and 32 | P2 Q1–Q3, P6 D1 |
| Before E1a | Untyped operations hidden from plugins until typed | Yes | P2 Q4, P6 D6 |
| Before P4-03 | **Named `unsafe` exception for the guest ABI** | Confine to `craft-guest::abi` and macro output | P4 D1 |
| Before P4-02 | v1 formats; parameter clamping; guest crate versioning | RGBA f32 only; clamp; major = contract | P4 D2–D4 |
| Before A1 | Placement of the attach transport | craft-libs `craft-attach` | P6 D5 |
| Before E6 | Project schema 3 in craft-libs `craft-ecproj-format` | Yes | P6 D4 |
| Before E8b | Old plugins stop; native helpers unavailable until P10; meaning of "basic temporal" | Accept; stateless time and seed | P6 D2, D3 |
| Before P5b | L0 timing in pilot apps | Inside each cutover | P5 D1 |
| Before P7 | Repository, domain, TUF crate, key holders, enablement moment | — | P7 D1–D3 |
| Before P12 | Outside reviewer and scope (costs money); crates.io; community enablement | — | P12 |

## Cross-repository work this creates

| Repository | Work | Plan |
|---|---|---|
| craft-libs | CL-1 keymap suggested layer; CL-2 menu extension points; CL-3 i18n runtime catalogue; CL-4 AccessKit helpers; `craft-attach` (if D5 is accepted); `craft-ecproj-format` schema 3 | P5, P6 |
| craft-common-actions | One shared-CI issue (P0.6) | P0 |
| catalogue repository | Created and named by maintainers | P7 |
| Apps | One adapter crate each; old systems removed in the same change set | P6, P8, P9, P11 |

## Limits

- Nothing was compiled or run: not the counts' build targets, not the tests, not the experiments. Every "test that fails first" is a plan, not an observed failure.
- Not read for this plan:
  - EffectCraft's `crates/helper-host` protocol and `crates/script` internals beyond the undo-group functions;
  - FigCraft UI wiring beyond the plugin dialog;
  - PhotoCraft's automation beyond `security.rs`;
  - the launcher's `feat/extensions` code beyond its file list;
  - every app other than the three pilots.
- **wasmi 2.0 behaviour is not verified here**: its feature flags, `Config` setters and `ResourceLimiter` signature. P3-00 settles it.
- GitHub limits (owner 39 characters) are as GitHub documents them. Recheck before P1-08.

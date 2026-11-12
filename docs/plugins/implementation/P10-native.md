# P10: Native helper protocol and the full-access label

Status: **plan, 2026-11-10.** Unit P10 of [the architecture](../architecture.md#9-runtime-and-safety). Each PR still needs its own approval.

**Code read:** EffectCraft `origin/main` `fb62b9d7` `crates/helper-host`, at the level of its registration and failure behaviour only. Its protocol framing was not read line by line.

## 1. Goal and exit criteria

**Goal.** Native plugins run only out of process, through one bounded, versioned helper protocol. Each carries the "Native: full access to your computer" label and its own consent. A revocation kills the process. Every native listing and release needs two distinct maintainer approvals, and the bot never auto-merges one. Process isolation contains crashes, not malice, and the plan never describes it as a security guarantee.

**Exit criteria (architecture section 22: "native crash contained; revocation kills process").**

| # | Criterion | Test |
|---|---|---|
| X1 | A helper that crashes, hangs or writes garbage never crashes the app; the frame or operation fails with the plugin named | `native::crash_hang_garbage_contained` (fixture helpers that abort, sleep forever, and write random bytes) |
| X2 | A revocation or disable kills the helper process within the stated bound, and running work fails | `native::revoke_kills_process` |
| X3 | Native updates always ask; the consent screen states full access | `manage::native_update_always_asks` |
| X4 | The catalogue refuses to merge a native release with fewer than two distinct maintainer approvals | `bot::native_needs_two_distinct_maintainers` |

## 2. Code areas

| What | Where |
|---|---|
| Lessons | EffectCraft `crates/helper-host`: helpers register as API-2 plugins; a crash or hang leaves the frame unchanged, the helper restarts, and a `blocklist` disables repeat offenders. **The new protocol keeps restart and blocklist. It replaces "leave the frame unchanged" with the P6 failure contract**: preview passes through with a badge, and final fails |
| Removal | `helper-host` is deleted in P6 E8b. Native helpers are unavailable from then until this unit lands (maintainer decision P6 D3) |
| Packaging | Manifest v2 `native: true` with per-platform artefacts ([P1 section 5.2](P1-store.md#52-manifest-schema-2-craft-extensionjson-written-by-craft-ext-pack)); catalogue `[[assets]] platform = …` ([P7 section 5](P7-catalogue.md#5-data-formats)) |

## 3. Layout

`craft-ext-host::native`:
- a process supervisor: spawn with the release's verified binary, never a path from the package;
- the helper protocol codec;
- watchdog and restart policy;
- kill on revoke, using the generation watch from P3.

**Helper protocol v1:**
- Length-prefixed frames over the child's stdin and stdout. The cap is 64 MiB a frame; pixels are chunked.
- Messages: `hello {protocol, helper_version, contributions}`, then request and response pairs carrying the same `raster.filter@1` request type, with `cancel` and `bye`.
- No shared memory in v1. It would need `unsafe` mapping, which the house rules forbid without a named exception.

## 4. PR list (outline)

| PR | Scope | Size | Test that fails first |
|---|---|---|---|
| N1 | Protocol spec (`docs/contracts/native-helper@1.md`) and codec with bounds | M | `codec::oversize_frame_refused`; `codec::garbage_never_panics` (fuzz target) |
| N2 | Supervisor: spawn, watchdog, restart, blocklist, kill on revoke | M | `native::crash_hang_garbage_contained`; `native::revoke_kills_process` |
| N3 | Labels and native consent in the manager; updates always ask | M | `manage::native_update_always_asks` |
| N4 | Catalogue bot: native static checks; smoke test only on an ephemeral runner with no secrets; the two-distinct-maintainer check as a required status | M | `bot::native_needs_two_distinct_maintainers` |
| N5 | EffectCraft native adapter (raster through a helper) | M | `adapter::native_raster_through_helper` |
| N6 (optional, later) | OS confinement per platform (macOS App Sandbox, Windows AppContainer, Landlock or bubblewrap). It is an improvement, never a listing requirement | M each | — |

## 5. Risks and decisions

- **R1, pixel throughput over pipes at 4K.** Measure before N5. If it is too slow, propose a shared-memory exception or a file-backed transfer for maintainer review.
- **R2, Windows process-tree kill.** A Job object needs Win32 calls. Check for a safe crate and its licence.
- **D1.** Kill bound after a revoke. Proposed: the next check, at most 1 s.

## 6. Effort

About 5 PRs, all M, plus optional confinement work. P10 needs P3, P5-03 and P7-06.

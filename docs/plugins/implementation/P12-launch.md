# P12: Public launch

Status: **plan, 2026-11-10.** Unit P12 of [the architecture](../architecture.md#22-delivery-plan). **Launch is a maintainer decision.** This plan lists what must be true before the maintainers decide. Publication, release and repository or domain creation each need their own authorization (house rule 9).

## 1. Goal and exit criteria

**Goal.** The community can find, install and publish plugins safely. Maintainers have the keys, people and runbooks to run the catalogue. An **independent outside security review** has covered the platform, and its findings are fixed.

**Exit criteria.**

| # | Criterion | Evidence |
|---|---|---|
| X1 | The outside security review is complete, covering the sandbox host, store client, catalogue signing and attach transport (architecture section 23). Every finding is fixed by a narrow tested PR or accepted in writing by the maintainers | Review report and a findings tracker |
| X2 | The catalogue is live on GitHub Pages initially (PC08); TUF metadata is fresh and the expiry monitor is green. A custom domain comes later | A live fetch from a clean machine |
| X3 | Root keys are held 2-of-3 on hardware by named custodians; a **root rotation and an online-key leak recovery have been rehearsed** | Rehearsal notes, dated |
| X4 | The listing rules, appeals process, disclaimer and takedown process are published in the catalogue README and shown in the manager | Links |
| X5 | The author guide, templates per contribution kind, examples and generated contract reference are published (git tags until crates.io is authorized) | Links |
| X6 | Every threat-model gate maps to tests that run in CI (implementation plan section 3, "Threat model") | Gate-to-test table |
| X7 | Maintainers record the community-enablement decision | Decision record row |

## 2. PR and task list (outline)

| Task | Scope | Owner |
|---|---|---|
| L1 | Author guide, listing rules, incident runbook, appeals, disclaimer | Platform and catalogue maintainers |
| L2 | Templates for each contribution kind; examples beyond Halftone (a LUT data pack, a FigCraft panel) | Platform |
| L3 | Generated contract reference and changelog (GitHub Pages until docs.rs) | Platform |
| L4 | GitHub Pages serving and expiry monitoring; custom domain later (PC08) | Catalogue maintainers (needs D1) |
| L5 | Key ceremony; rotation and leak-recovery rehearsal | Key custodians |
| L6 | Outside review: scope and statement of work agreed by the maintainers; findings tracked; fixes as narrow PRs, each with a failing-first test | Maintainers and reviewer |
| L7 | Gate-to-test audit across the threat model | Platform |
| L8 | Launch decision recorded | Maintainers |

## 3. Risks and decisions

- **R1, review capacity and cost.** The outside review costs money. That needs maintainer approval (house rule 9).
- **D1.** Pages endpoint and publishing configuration. Domain and later hosting arrangements are deferred by PC08.
- **D2.** The outside reviewer and scope.
- **D3.** crates.io publication of the SDK (architecture section 24).
- **D4.** Community enablement.

## 4. Effort

This is mostly documentation and operations. The engineering effort is whatever the review's findings require. Schedule the review once P6–P9 and P7 are stable, so it reviews the code that ships.

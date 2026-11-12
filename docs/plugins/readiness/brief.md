# Implementation-readiness investigation brief

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Requested 2026-11-08 after the complete architecture draft. This phase investigates actual application capability gaps, Jeremy reuse, a simple SDK/catalogue distribution path, and ordered user/developer workflows. It does not start app implementation, create external issues/repositories, publish, change CI, or open sg13.

## Latest user direction

The user asks for the biggest issues and system changes before starting development, a fresh source audit of what applications provide and lack, and all relevant plugin functions while protecting core application features. They want a thorough yet simple plugin SDK, compiled packages on publishers' GitHub accounts, human-reviewed catalogue admission, official precedence and GitHub-star popularity. Public install/use should not gain unnecessary login steps. If “login” meant a specific new account requirement, that remains unselected; default design is no account for public packages, publisher GitHub login for submission, optional host credentials for restricted assets.

The user explicitly confirmed that “OpenWhispr agent” meant an Opus 5.5 medium coordinator using Haiku 5.5 xhigh workers, with no additional Opus or Sonnet delegates. Use exactly three focused Haiku workers for the app coverage below. No further descendants or speculative Sol workers are necessary. The root retains final oversight and can answer coordinator questions; it is not a second competing audit.

## Read these records in order

1. [Latest accepted decisions](../draft/decisions.md), then [architecture overview](../draft/README.md).
2. [Complete Jeremy artifact](../sources/jeremy-extensions-todo.txt), [current source record](../source-and-verification-record.md), and [the handoff crosswalk](jeremy-crosswalk.md).
3. [SDK/contracts](../draft/sdk-and-contracts.md), [security/distribution](../draft/security-and-distribution.md), [shared operations/app work](../draft/shared-operations-and-apps.md), [delivery/acceptance](../draft/delivery-and-acceptance.md).
4. [Opus design objections and reconciliation](../draft/review-reconciliation.md), [earlier app inventory](../../plugin-functional-inventory.json), [detailed app/CLI/state evidence](../research/haiku-app-cli-state.md).

All earlier notes and originals stay intact. Avoid reading every historical page again unless a specific ambiguity needs it. Source revisions establish source structure, not runtime behavior or current upstream status.

## Core-feature protection

Identify existing registration/collision/default-handler behavior in every app. Proposed required rules are additive, namespaced contribution IDs; built-in command/effect/type IDs cannot be shadowed; plugins cannot patch engine code or register an unrestricted command bridge; contributed handlers/shortcuts/menu placements cannot silently replace built-in defaults. Explicit user selection of an extension command, approved handler, theme or binding is distinct from installation changing the app's core behavior. State which of these rules code enforces today, which are missing, and which require design.

Document read/edit access remains through allowed app operations and scoped explicit grants. A permission declaration is not enforcement. CLI and plugin callers may share operation behavior but never acquire each other's broader identity/authority.

## Worker coverage and output ownership

| Worker | Apps | Assigned output |
|---|---|---|
| Haiku A | EffectCraft, PhotoCraft, FilmCraft, LightCraft, ArtCraft | `docs/plugins/readiness/haiku-media-apps.md` |
| Haiku B | FigCraft, VectorCraft, PDFCraft, DesignCraft, DeckCraft | `docs/plugins/readiness/haiku-design-apps.md` |
| Haiku C | SoundCraft, CADCraft, SolidCraft, TextureCraft, WordCraft, GridCraft | `docs/plugins/readiness/haiku-domain-apps.md` |

Each worker writes only its own note. Coordinator writes README, `system-gaps-and-roadmap.md`, `catalogue-and-sdk.md`, `user-and-developer-workflows.md`, and `evidence-index.md` here. Root owns this brief and the initial Jeremy crosswalk, then performs final reconciliation/docs checks. No actor edits another worker's note or shared app checkout.

For each app, inspect bounded source at a recorded committed local HEAD from the existing clone under `/Users/ash/Projects/<app>`. Verify the clone path and revision; use `git show` so dirty work stays untouched. Compare relevant baseline revision only where it clarifies a changed finding. Do not fetch broadly, change branches, create app worktrees, run builds/tests or modify apps. If source is absent, record the precise limitation and use named baseline evidence without claiming a current audit.

Required per-app evidence: app/commit and exact paths/symbols inspected; available engine/CLI/plugin operations and UI features; contribution families; transaction/job/persistence/output ownership; collision/core override behavior; existing sandbox/grants/loading; reusable APIs; missing adapter/SDK hooks; app-specific data constraints; smallest implementation slice and behavioral proof. “Not found in inspected files” is not proof a runtime does not exist anywhere. At least trace a real app operation definition beyond the entry point.

## Coordinator synthesis

Produce a prioritized list of actual system blockers and app changes, with evidence, owner, dependencies, smallest useful change, compatibility/security implications, and future acceptance. Separate existing/reusable, partially integrated, absent/not established, and proposed functionality. Do not turn this into a universal engine rewrite.

Design the minimum coherent SDK-to-release-to-review-to-search-to-install-to-grant-to-use-to-update-to-recovery path. Define catalogue record fields, exact-release PR/review/publication, upstream GitHub assets, cached star ranking and tie-breaks, offline/privacy/credentials, local files, licences and revocation. Review exact digest rather than silently following `latest`. No binary hosting, mandatory login for public use, mandatory build farm, microservice fleet, duplicate installer or parallel contract schema.

Lay out concrete end-user, author, maintainer and app-developer workflows. Explain SDK/host boundaries and how builtin functionality stays authoritative. Reconcile PhotoCraft first-party reuse with the accepted EffectCraft-then-FigCraft expanded functional proof. Cover every [aspect-register](../aspect-register.md) row by a section or explicit deferred reason. Remaining product choices are not approvals.

Use authoritative external sources for novel claims. Reuse already retrieved official GitHub/TUF/VS Code/Figma/Zed/WIT references when adequate; do not invent a runtime guarantee from another product's pattern. No library choice, licence audit, new key, remote, credential or security certification is authorized.

## Orchestration and completion

Coordinator checks actual provider settings before its first child delegation and uses `delegate_task` with Haiku 5.5 xhigh. It may ask root for missing context; do not re-request the already archived Jeremy material from the user. Workers start no agents. Respect returned asynchronous continuation instructions rather than polling.

Do not claim the investigation is complete until all three worker results are incorporated and the coordinator synthesis exists. Preserve disagreements and qualify overbroad findings. Final coordinator summary gives biggest blockers, evidence limits, file paths and unresolved decisions. Root then checks local links/coverage and presents the work to the user. No runtime proof is expected or claimed from this source/documentation phase.

# Craft plugin work handoff

Prepared 2026-11-11. The user requested a local consolidation of the plugin discussion, architecture, plans, and GitHub documentation, followed by a new thread in this existing checkout.

Latest activity, 2026-11-12: the user authorized a detailed local architecture/workflow review with agents and read-only inspection of open PRs, explicitly without external comments. Start with the [system guide](docs/plugins/system-guide.md), [workflow walkthrough](docs/plugins/design-sessions/system-walkthrough.md), and [correction register](docs/plugins/design-sessions/architecture-review-2026-11-12.md). AR01 retention is confirmed; other new contract proposals remain open. The original preparation scope below is historical. There is still no implementation/build/commit/push/publication authorization. The original transfer hashes describe the imported snapshot; later recorded design edits intentionally change some destination hashes.

## Read in this order

1. [Current understanding and explicit deferrals](docs/plugins/design-sessions/post-call-reconciliation.md#current-understanding-after-the-first-pass).
2. [Current architecture](docs/plugins/architecture.md).
3. [Design-session and post-call decisions](docs/plugins/draft/decisions.md#design-session-decisions), then the recent [session record](docs/plugins/design-sessions/session-record.md).
4. [Main implementation plan](docs/plugins/implementation-plan.md) and [per-unit plan index](docs/plugins/implementation/README.md). Read individual units when working on their area.
5. [Craft house rules supplied with the discussion](docs/plugins/sources/house-rules.md) and the [local threat-model reconciliation](docs/THREAT-MODEL.md).

The [full documentation index](docs/plugins/README.md) links the research, readiness audits, discussion archive, source inventory, and original handover. Older proposals and code references are evidence at recorded revisions; they do not override later user choices.

## Current scope

The first usable MVP is EffectCraft with one manually installed L0 data package and one L1 transform, including save/reopen and failure handling. Exact examples remain open. Presets and Halftone were proposed but not selected as the MVP examples.

Packages are imported and validated, then explicitly enabled. Package files are shared; enablement, settings, and permissions are per app. WebAssembly permissions remain host-enforced. Native plugins retain their separate full-access contract.

Catalogue metadata initially comes from GitHub Pages, with a custom domain later. Maintainers control TUF signing; separate publisher signing keys are not required. Source/namespace checks and authenticated package hashes remain required.

"IGUI" meant egui. The manual MVP uses existing egui components without waiting for the separate theme framework, and adopts shared theme tokens later. Shared platform code stays in craft-extensions; app adapters stay in their app repositories. Delivery uses small reviewable PRs with complete integration proof. The manager and catalogue retain a short disclaimer.

## Decisions still needed

The user deferred PC04 milestones and the meaning of "version 7", PC05 scanning policy, PC07 reviewer roles, and the exact MVP examples. Preserve those deferrals. Existing P7/P12 wording, checking requirements, outside security review before public launch, and two-maintainer approval for every native listing and release remain the baseline.

Other open details include "verified against main", detailed MVP acceptance and old-system cutover, other-app rollout, theme-token schema/ownership/timing and early AI UI adoption, release-version propagation, named owners/navigation/PR contents/estimates, and exact disclaimer wording/formal legal-review timing. The existing unit plans are not a fully reconciled replacement MVP sequence.

## Local and GitHub state

The code checkout remains at `a6c5fae8b8f6a92180c02e9188cd3559161c174b`. Only documentation was prepared; no source, dependency, feature, workflow, or branch was updated.

GitHub main was read at `98a151022a4bd4ff8aadda5161abb36789ae43b2`. [PR #1](https://github.com/storytold/craft-extensions/pull/1) and [PR #3](https://github.com/storytold/craft-extensions/pull/3) are merged. Their documentation tasks are complete. [Preserved upstream files and review context](docs/plugins/sources/github-main/SNAPSHOT-README.md) include exact revisions and hashes. The original handover at `ffd8512e5a3755637f58cf31cfcf2fdb09cfb943` is retained in the [handover archive](docs/plugins/sources/jeremy-handoff/README.md).

The current design lives in `docs/plugins/`. The root architecture, decisions, topic-proposals, and implementation-plan entry pages link there. The local threat model aligns G2 with confirmed TUF signing; its unaltered GitHub source is archived. No gate is implemented or verified by these documentation edits. PC04's feature/public-launch wording remains unresolved.

## Continue in the new thread

Read the documents above, summarize the current state, and wait for the user's next task. This preparation request authorizes local documentation and read-only retrieval, plus creation of the new thread. It does not authorize implementation, builds, commits, pushes, issues/PRs, external messages, publication, releases, new repositories, or new remotes. Keep the uncommitted documentation available in this checkout and preserve unrelated files. Start no subagents unless the user explicitly asks.

When the user chooses further design work, resolve only the relevant open choices and record answers in the existing reconciliation and session record. When implementation is explicitly requested, first reconcile its affected plan and preserve this uncommitted bundle before any checkout change.

## Transfer checks

The [local transfer manifest](docs/plugins/local-transfer-manifest.json) records every copied file's original SHA-256 and destination SHA-256, including seven documented link rewrites to pinned craft-libs source URLs. The [GitHub snapshot manifest](docs/plugins/sources/github-main/snapshot.json) records Git blob hashes for the upstream files. Original source documents remain in place. No builds, tests, mutation harness, or archive scripts were run during this preparation.

The [transfer verification report](docs/plugins/local-transfer-verification.json) records passing source/destination hashes, upstream Git blob hashes, original handover hashes, local Markdown file links, and the unchanged code revision. Hashes establish transfer integrity, not runtime correctness.

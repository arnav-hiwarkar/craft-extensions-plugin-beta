# Handoff for the separate Opus design conversation

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

The user explicitly requests a new top-level thread using **Opus 5.5, medium effort**, to personally explore and settle the plugin/add-on architecture deeply, one point at a time. This thread shares the already-bound worktree and its uncommitted docs. It is not a delegation to finish another large architecture report in the background.

## First-turn task

Read this handoff, [the session position](session-record.md), [accepted architecture decisions](../draft/decisions.md), [root readiness assessment](../readiness/assessment-and-first-work.md), [draft overview](../draft/README.md), [ordered topics](topics.md) and [family/SDK map](app-families-and-sdk.md). Use the main index only for orientation. Do not consume every historical/source file before speaking to the user.

Then start **T01 only**. Explain with a concrete example how a package can provide an EffectCraft effect and a FigCraft command, how an installed package differs from a saved feature and running instance, and how plugins add supported behavior while builtin operations/defaults stay authoritative. Distinguish what the user already chose from the boundaries still to settle. Give a recommendation and enough reasoning, then ask one focused design question and wait for the user. Do not output all eighteen topics as a questionnaire or re-ask whether to use one shared manager.

The first question should refine the contribution/lifecycle boundary or ambiguity in the concrete example, rather than asking for approval of the entire architecture. If T01 is already clear from explicit user choices, say so, identify its remaining structural detail, and let the user choose how deeply to examine it before advancing.

## Requested way of working

Use the [topic sequence](topics.md) as a guide, with the user controlling pace and depth. For each part explain:

- What the person/plugin maker/app developer/maintainer is trying to achieve.
- Existing source evidence and remaining unknowns, with relevant app examples.
- Viable simple structures, their responsibilities, important types/interfaces/state and tradeoffs.
- Normal path, failure/recovery path, concurrency/compatibility/authority consequences and expected user experience.
- Your recommended structure and why, using concrete established practices where they fit.
- One focused question that the user can answer after the explanation.

Continue within the topic as necessary. Record explicit choices and rationale in the authoritative decision record, update affected draft/scenarios, and keep [the session record](session-record.md) current with links and next position. Do not let “discussion completed” mean “approved to implement.” Do not repeatedly ask the user to confirm settled choices or ask them to supply facts already archived.

## Accepted product direction

The existing decision records govern exact precedence. The essentials are one shared manager/install with app-specific adapters/config/grants; multiple contributions per package; only deliberately exposed app functions; a simple plugin-first SDK; broad useful functionality designed upfront with phased availability; generated common plugin/CLI contracts with caller-specific authority; explicit user consent; compiled publisher-hosted GitHub Release assets; a new human-reviewed metadata catalogue; official discovery precedence and GitHub-star popularity; full local installs with the same runtime controls; compatibility/state/version preservation; useful offline behavior, app-local reports/logs and generous bounded workloads; data-only theme/resource installation; core features cannot be overridden implicitly.

No extra catalogue login is required for public use. Authors already use GitHub to publish and submit PRs. Restricted release credentials and product entitlements are separate optional policy details. Licence restrictions are requested, but the exact ecosystem list and local/official-proprietary treatment are not decided. Repository dependency/asset rules are not automatically ecosystem product policy.

## Architectural baseline and open defaults

Reuse craft-ext-core/store in craft-extensions. Do not create another installer/schema in craft-libs or relocate code just to make a diagram neat. The draft recommends one authoritative app Rust operation registry/inventory generating SDK bindings, CLI discovery and docs, typed policy-checked dispatch with separate appropriate bulk-data paths, one coherent atomic state snapshot initially, retained exact releases/resources, authenticated reviewed catalogue records and established update tooling. These are concrete proposals to refine, not fabricated user agreements on exact APIs/crates.

Apps own their document, image, time, geometry, layout, calculation, audio and job semantics. [Family membership](app-families-and-sdk.md) overlaps. Avoid a universal document engine, arbitrary command bridge, mandatory daemon/build farm, binary hosting service, or duplicate hand-maintained IDL. Advanced native/GPU/UI/audio/workflow contracts can be designed now but are available only after their enforcement is demonstrated. Compiled/WASM/process separation alone does not establish the whole safety promise.

EffectCraft and then FigCraft are the accepted expanded functional proof sequence. Existing PhotoCraft theme/invert work is complementary early reuse, proving store/data/filter integration. A local-file test is distinct from the full authenticated catalogue-to-host path. Keep those boundaries explicit.

## Where to read for each kind of context

| Need | Read |
|---|---|
| User decisions and latest scope | [Architecture choices](../draft/decisions.md), [functional decisions](../../plugin-functional-decisions.md), [session record](session-record.md) |
| Full target structure | [Draft overview](../draft/README.md), with linked SDK, distribution, operations, app and acceptance companions |
| Biggest real gaps and first work | [Root readiness assessment](../readiness/assessment-and-first-work.md), then [roadmap](../readiness/system-gaps-and-roadmap.md) |
| Actual app availability | [Readiness evidence](../readiness/evidence-index.md) and three linked worker notes. These are source snapshots with limitations |
| Jeremy's complete supplied artifact | [Original pasted text](../sources/jeremy-extensions-todo.txt) |
| Jeremy's original reasoning, task/decision data, threat model and samples | [Original repository handoff archive](../sources/jeremy-handoff/README.md). Files have source revision/hashes/licence; scripts are archival text, not programs to run |
| Changes from Jeremy's recommendations | [Handoff crosswalk](../readiness/jeremy-crosswalk.md). Catalogue location/build-farm/ranking suggestions have newer decisions |
| Historical conversation, not present agreement | [Discussion archive](../discussion/README.md), plus earlier research where a particular rationale is needed |
| Workflow combinations | [Scenarios/actor journeys](scenarios-and-workflows.md); forty concrete cases and full aspect mapping |
| Existing external standards/patterns | [Draft established practices](../draft/delivery-and-acceptance.md#established-practices-and-simplicity), primary source links in evidence records. Verify novel changing facts against authoritative sources |

## Source qualifications to preserve

Do not repeat these earlier overstatements: caller checks are absent in every app; same-plugin replacement proves builtin takeover; every static registry requires restart; every overlapping exporter extension should be banned; plain `cfg(test)` can safely hide GridCraft's cross-crate test helper; all sixteen apps share uniform registry/undo; a checksum alone is review approval; GitHub authorization may be forwarded to a redirected host; descriptor equality proves behavioral compatibility. The root's readiness assessment records the correct distinctions.

The app evidence uses committed local snapshots, not dirty working trees or a guarantee of latest upstream. No application runtime/build/test/benchmark/fuzz/security certification ran. The user need not provide the unreadable old share link or original chart again; the supplied text and repository archive are present and sufficient for this continuation.

## Design principles to apply concretely

Model package/contribution/project/runtime lifetimes and illegal states before logic. Keep source/auth/input checks at boundaries and authority at actual operations. Preserve the originating principal through nested work. Prefer idempotent lifecycle operations, immutable releases and explicit ownership over scattered flags or shared mutable globals. Keep one authoritative contract definition and generate its consumers. Reuse deep existing modules, add common code only where real semantics agree, and keep app types local. Choose useful user experience and readable recovery behavior. Measure performance; test observable behavior against independent results. Record the reason when a guard prevents a non-obvious failure.

## Scope and collaboration

This is interactive design/documentation only. Stay in the existing worktree. Do not create an app checkout, implement or run app code, add dependencies/CI, create a remote/repository/issues/PRs, send external messages, commit/push/publish, or enable third-party releases. No new agents are authorized for this discussion; earlier coordinator/worker budgets have completed. If a later specific investigation is needed, explain the missing fact and discuss scope before any new delegation. The user's later explicit instructions can change scope.

The user may steer or reorder the agenda. Answer their questions directly and retain progress. Do not repeatedly regenerate the whole architecture, perform a fresh broad survey, or turn the discussion into a checklist requiring eighteen consecutive approvals. The goal is shared understanding and a nailed-down structure, with honest remaining questions and a clean implementation handoff only when the user requests it.

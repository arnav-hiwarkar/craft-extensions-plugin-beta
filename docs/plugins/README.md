# Craft plugin and add-on design record

> **Beta implementation (2026-11-12):** the code follows [beta-design.md](beta-design.md), which simplifies this design where they differ (one-maintainer store admission, signed index instead of TUF, per-contribution contracts and permissions, no bot). The superseded points are listed in [draft/decisions.md](draft/decisions.md#beta-decisions-2026-11-12).

Updated 2026-11-11. Start here when working on Craft plugins and add-ons.

## Read this first

**[The Craft plugin architecture](architecture.md)** is the single current description of the system. It consolidates the original eighteen design topics and confirmed post-call choices. Read the [post-call current understanding](design-sessions/post-call-reconciliation.md#current-understanding-after-the-first-pass) for the confirmed manual EffectCraft L0/L1 MVP, explicit deferrals, and remaining questions. The first pass is complete; the entire replacement implementation plan is not newly finalized.

Design only. Implementation, new repositories, publication and releases each need their own approval. The [delivery plan](architecture.md#22-delivery-plan) orders the work; [open items](architecture.md#24-open-items) lists what still needs an owner or evidence.

## Where things live

| You want | Go to |
|---|---|
| How the system works | [Architecture](architecture.md) |
| A plain-language walkthrough of every part and workflow, with 2026-11-12 review findings | [System guide](system-guide.md) |
| What a plugin's panels and windows can contain, and how apps draw them | [Plugin UI reference](ui.md) |
| Detailed workflow/component/stage traces and all forty scenarios | [Walkthrough](design-sessions/system-walkthrough.md), [scenario coverage](design-sessions/workflow-coverage-2026-11-12.md) |
| Six-agent architecture corrections, open contracts, and PR evidence | [Correction register](design-sessions/architecture-review-2026-11-12.md) |
| Latest meeting reconciliation, confirmed choices, and deferrals | [Post-call current understanding](design-sessions/post-call-reconciliation.md#current-understanding-after-the-first-pass) |
| Top-level and per-unit implementation detail | [Main plan](implementation-plan.md), [unit plans P0–P12](implementation/README.md) |
| What was decided, when and in whose words | [Decision record](draft/decisions.md#design-session-decisions) |
| The question-and-answer trail of the design sessions | [Session record](design-sessions/session-record.md) |
| Each topic's proposal with evidence and rejected alternatives | [Family map and session proposals](design-sessions/app-families-and-sdk.md) |
| Which plugin-related merges are held and when they lift | [Merge-hold list](merge-hold-list.md) |
| Source evidence from the sixteen apps at recorded revisions | [Readiness audits](readiness/README.md), [evidence index](readiness/evidence-index.md) |
| Earlier drafts, research and discussion | [Draft](draft/README.md), [research](research/README.md), [discussion archive](discussion/README.md), and the `docs/plugin-*.md` files |
| Jeremy's original handoff | [Repository archive](sources/jeremy-handoff/README.md), [supplied text](sources/jeremy-extensions-todo.txt), [crosswalk](readiness/jeremy-crosswalk.md) |

Every document other than the architecture, the decision record, the session record, the family map and the merge-hold list carries a status note: it is history and evidence, and the architecture wins where they disagree.

## Detailed evidence and investigations

| Document | Purpose and status |
|---|---|
| [Resource register](../plugin-resources.md) | Supplied links, retrieval limits, source references, and research history |
| [Jeremy's handoff summary](../jeremy-extensions-handoff-summary.md) | Structured summary of the supplied Claude text, with historical status caveats |
| [Complete supplied handoff](sources/jeremy-extensions-todo.txt) | Verbatim artifact text pasted by the user, including every task, dependency, completion condition, and standing gate |
| [Jeremy's complete repository handoff archive](sources/jeremy-handoff/README.md) | Original design reasoning, decision and task data, detailed checkpoint, generated page, threat model, and sample themes, preserved with source revision and licence |
| [Source and verification record](source-and-verification-record.md) | Inspected revisions, live issue context, and limits of evidence |
| [Primary functional investigation](../plugin-functional-primary.md) | Runtime paths and app requirements from source inspection |
| [Independent Opus functional investigation](../plugin-functional-opus.md) | Separate functional analysis, qualified during reconciliation |
| [App source inventory](../plugin-functional-inventory.json) | Machine-readable 16-repository inventory with recorded source revisions and hashes |
| [Runtime and theme investigation](../plugin-security-runtime-themes.md) | App-specific runtime constraints, the two theme layers, and security boundaries |
| [Opus review of the existing code](../plugin-security-handoff-opus.md) | Review against actual craft-extensions, launcher, and PhotoCraft branches |
| [Initial architecture ideas](../plugin-architecture-ideas.md) | Earlier alternatives, before later choices and existing-source discovery |
| [Early Opus distribution pass](../plugin-security-distribution-opus.md) | Historical research. Organization-management material is out of scope and later ownership findings supersede it |

## History and precedence

The [discussion archive](discussion/README.md) preserves the technical requests and substantive answers from before the design sessions. Historical responses can contain proposals or facts later corrected. Their presence is not approval.

When records differ, use this order: the user's latest explicit choice (decision and session records), then [the architecture](architecture.md), then source evidence at a named revision, then earlier drafts and research. Source inspection establishes what code says at that revision; it does not establish runtime behaviour or current branch state.

The ChatGPT share link remains unreadable and its conversation contents are not archived. The Claude chart's visual layout and original artifact source remain unavailable. Its complete user-supplied text is archived. Do not claim these missing materials were reviewed.

## Keeping the record current

1. Record an explicit user choice in the [decision record](draft/decisions.md), with what it replaces, and its trail in the [session record](design-sessions/session-record.md).
2. Update [the architecture](architecture.md) in the same change, so it stays the one current description.
3. Put proposals, evidence and alternatives in the relevant design-session section or investigation, marked with their status.
4. Keep original material in `sources/` unchanged. Preserve corrections without presenting old conclusions as current.

## Working boundaries

- Reuse craft-extensions as the home of every shared plugin-platform crate. Do not create a competing installer, schema or store anywhere else.
- Shared code stays independent of application crates. Each app's adapter lives in that app.
- Each app's current plugin system is replaced, not extended, in the same change set that lands its adapter.
- No implementation, repository creation, publication or release without separate approval.

All files here are local worktree changes until separately committed. No Rust builds, app runtime tests, release packages or security certification were produced by this documentation work.

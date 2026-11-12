# Guided architecture sessions

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Prepared 2026-11-08 for the user's separate Opus 5.5 medium discussion thread. The purpose is to understand and settle the system one part at a time. This is a discussion map over the existing evidence and draft, not a replacement architecture or authorization to implement.

## Start here

For the new discussion after the team call, start with [Post-call plugin decisions](post-call-reconciliation.md). It preserves the supplied technical notes, compares them with the existing agreement, and tracks the questions that still need answers.

Read [the Opus handoff](opus-handoff.md) for conversation instructions and authority, then [the ordered topics](topics.md). The first session covers what an extension can add, what it cannot replace, and the distinction between a package, contribution, saved instance and running instance. Do not start with a long list of schema fields or ask the user to decide every policy at once.

| Guide | Purpose |
|---|---|
| [Ordered topics](topics.md) | Eighteen bounded discussions, dependencies, questions and expected design artifacts |
| [App families and SDK layers](app-families-and-sdk.md) | Proposed SDK separation, all sixteen apps, domain contracts and common-library boundaries |
| [Scenario and workflow coverage](scenarios-and-workflows.md) | State dimensions, adverse combinations, actor journeys and the twenty-seven-aspect crosswalk |
| [Session record](session-record.md) | Current position and one durable record for decisions, reasons, unresolved work and follow-up |
| [Opus handoff](opus-handoff.md) | Reading order, accepted direction, source limits, conversation style and scope |
| [Verification](verification.md) | Rerunnable link/coverage checks and preservation of Jeremy's archived source bytes |

## Keep the existing record authoritative

Latest explicit user choices take precedence. Record new agreed choices in [the accepted architecture decisions](../draft/decisions.md), with a summary and rationale in the session record. The [earlier functional decisions](../../plugin-functional-decisions.md) remain part of the accepted constraints. Do not silently turn draft defaults into user agreement or reopen settled requirements.

Use [the root readiness assessment](../readiness/assessment-and-first-work.md) for qualified source findings and [the full architecture draft](../draft/README.md) for the proposed target. Independent notes remain preserved evidence, including their stated limits and later corrections. The [main index](../README.md) links all stages.

Jeremy's [complete supplied text](../sources/jeremy-extensions-todo.txt) and [original repository handoff archive](../sources/jeremy-handoff/README.md) are available. Read originals for historical reasoning or task gates; do not execute archived scripts, adopt their old machine paths, or mistake historical test reports for new verification.

## How each conversation works

Explain one concrete user/author task, then the relevant existing source and constraints. Compare the viable structures with consequences for users, authors, maintainers and app developers. Give a recommendation, describe ownership/data/interfaces and normal/adverse behavior, and ask one focused question. Continue within the topic as deeply as the user wants.

After an explicit decision, record the user's choice and reason, update the affected proposal, and identify invalidated assumptions or scenarios. An unanswered question stays open. A topic can be provisionally understood while a dependent detail remains open; do not force artificial approval gates or stop useful discussion.

The broad design can cover every family now while implementation remains phased. Unsupported backend features stay unavailable until demonstrated. Simplicity means cohesive responsibilities, one source of truth and reusable proven mechanisms, not deleting necessary compatibility, consent or recovery behavior.

## Reading depth

The new thread initially needs the handoff, decision record, root assessment, draft overview, topic list and session record. Read the relevant companion and app evidence when its topic comes up. Avoid re-reading all source audits for every question. Ask for preferences and tradeoffs, not facts already available in the docs.

All work remains in this existing worktree, including its uncommitted documentation. No new checkout is needed. The thread is a separate conversation, not a new implementation branch. No new agents, code changes, builds, external messages/issues, repositories, pushes or publication are part of the requested discussion. The third-party gate remains closed.

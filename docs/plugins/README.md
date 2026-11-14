# Craft plugins

Plugins for EffectCraft, PhotoCraft and FigCraft share one platform: the crates in this repository,
one adapter in each app, and the signed catalogue in `craft-plugins`.

## Current guides

| Guide | For |
|---|---|
| [Workflow atlas and mind map](maps.md) | Visual map of roles, ownership and every main lifecycle |
| [Documentation coverage](coverage.md) | Feature-to-guide, source and evidence index |
| [Community comparison and follow-up backlog](community-comparison-and-backlog.md) | Existing community workflows, capability gaps and proposed priorities |
| [Author guide](author-guide.md) | Writing, building, testing and packing a plugin |
| [How it works](how-it-works.md) | Components, lifecycle, sandbox, native helpers, permission checks, undo, safe mode |
| [Adapter guide](adapter-guide.md) | Connecting an app to the platform |
| [Capabilities](capabilities.md) | What each app offers: contribution kinds, operations and permissions, what is not available |
| [UI reference](ui.md) | Panels, windows, forms, canvas, events and UI levels |
| [Formats and versions](formats-and-versioning.md) | Manifest, ABI, contracts, UI, state and catalogue version domains |
| [Native helper protocol](native-protocol.md) | JSON-line records, errors, deadlines and process lifecycle |
| [CLI reference](cli-reference.md) | `<app>-cli plugin …` and `craft-ext` |
| [Store and distribution](store-and-distribution.md) | Catalogue format, signing, vendors, tiers, reports, kill list, local test catalogues |
| [Maintainer guide](maintainer-guide.md) | Listing admission, signed publication and incident response |
| [User guide](user-guide.md) | Finding, installing, setting up and removing plugins |
| [Testing beside normal Mac apps](installation-and-testing.md) | Separate catalogue trust, stores, preferences and launch methods |
| [Testing the beta](../../TESTING.md) | Downloading CI builds and testing them against the test catalogue |

Reference pages generated from the adapters: [EffectCraft contract](contracts/effectcraft.md),
[PhotoCraft contract](contracts/photocraft.md), [FigCraft contract](contracts/figcraft.md).
The beta design is specified in [beta-design.md](beta-design.md); the future renderer seam is in
[framework integration](framework-integration.md).

## History and design record

Everything below is history and evidence. Where it disagrees with the current guides or
[beta-design.md](beta-design.md), those win.

### Design and plans

| Document | Purpose |
|---|---|
| [Beta design](beta-design.md) | The specification the beta source follows; supersedes older proposals |
| [Beta implementation map](beta-implementation.md) | Where each beta part lives in code (dated checklist) |
| [Shared beta tests](testing.md) | The `scripts/e2e.sh` flow |
| [Architecture](architecture.md) | The long-term architecture proposal |
| [System guide](system-guide.md) | Plain-language walkthrough of every part, with 2026-11-12 review findings |
| [Walkthrough](design-sessions/system-walkthrough.md), [scenario coverage](design-sessions/workflow-coverage-2026-11-12.md) | Workflow, component and stage traces and the forty scenarios |
| [Correction register](design-sessions/architecture-review-2026-11-12.md) | Architecture corrections and open contracts |
| [Post-call reconciliation](design-sessions/post-call-reconciliation.md#current-understanding-after-the-first-pass) | Confirmed choices and deferrals |
| [Main plan](implementation-plan.md), [unit plans P0–P12](implementation/README.md) | Implementation planning |
| [Decision record](draft/decisions.md#design-session-decisions) | What was decided, when and in whose words |
| [Session record](design-sessions/session-record.md) | The question-and-answer trail of the design sessions |
| [Family map and session proposals](design-sessions/app-families-and-sdk.md) | Each topic's proposal with evidence and rejected alternatives |
| [Merge-hold list](merge-hold-list.md) | Which plugin-related merges were held and when they lift |
| [Aspect register](aspect-register.md), [research brief](research-brief.md) | Early research checklists |
| [Draft](draft/README.md), [research](research/README.md) | Earlier drafts and research |

### Evidence and investigations

| Document | Purpose |
|---|---|
| [Source and verification record](source-and-verification-record.md) | Inspected revisions and limits of evidence |
| [NZO community review](community-nzo-review.md), [EC Scripts review](community-ecscripts-review.md) | Source-backed community research, with inaccessible evidence kept explicit |
| [BOUNCraft source and release](community-bouncraft-review.md) | Inspected motion-tool workflow, legacy Wasm builder and migration gaps |
| [Resource register](../plugin-resources.md) | Supplied links, retrieval limits and research history |
| [Jeremy's handoff summary](../jeremy-extensions-handoff-summary.md) | Structured summary of the supplied handoff text |
| [Complete supplied handoff](sources/jeremy-extensions-todo.txt) | Verbatim handoff text |
| [Jeremy's repository handoff archive](sources/jeremy-handoff/README.md) | Original design reasoning, threat model and sample themes, with source revision and licence |
| [Primary functional investigation](../plugin-functional-primary.md) | Runtime paths and app requirements from source inspection |
| [Independent Opus functional investigation](../plugin-functional-opus.md) | Separate functional analysis |
| [App source inventory](../plugin-functional-inventory.json) | Machine-readable repository inventory |
| [Runtime and theme investigation](../plugin-security-runtime-themes.md) | Runtime constraints, theme layers and security boundaries |
| [Opus review of the existing code](../plugin-security-handoff-opus.md) | Review of earlier branches |
| [Initial architecture ideas](../plugin-architecture-ideas.md) | Earlier alternatives |
| [Early Opus distribution pass](../plugin-security-distribution-opus.md) | Historical research; later ownership findings supersede it |

### Precedence

When records differ, use this order: the user's latest explicit choice (decision and session
records), then the current guides, [beta design](beta-design.md) and the generated contract pages,
then source evidence at a named revision, then earlier drafts and research. Source inspection shows
what code says at a revision; it does not show runtime behaviour. CI results are recorded in
[TESTING.md](../../TESTING.md) and the `plugin-beta` workflow runs.

### Keeping the record current

1. Record an explicit user choice in the [decision record](draft/decisions.md), with what it
   replaces, and its trail in the [session record](design-sessions/session-record.md).
2. Update the current guides and [beta design](beta-design.md) in the same change as the code.
3. Keep original material in `sources/` unchanged.

### Working boundaries

- craft-extensions is the home of every shared plugin-platform crate. Do not create a competing
  installer, schema or store elsewhere.
- Shared code stays independent of application crates. Each app's adapter lives in that app.

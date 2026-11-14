# Documentation coverage and source map

This is the reference index for the current beta documentation. It maps each part
of the system to a reader, a workflow, source ownership and its evidence boundary.
The [workflow atlas](maps.md) supplies the diagrams. The [handbook index](README.md)
identifies current guides and separates them from historical design records.

## Document types

| Type | Purpose | Pages |
|---|---|---|
| Tutorial and procedures | Complete a task with expected results | [Author guide](author-guide.md), [user guide](user-guide.md), [Mac testing setup](installation-and-testing.md), [maintainer guide](maintainer-guide.md) |
| Explanation | Understand runtime ownership and design choices | [How it works](how-it-works.md), [workflow atlas](maps.md), [framework transition](framework-integration.md) |
| Reference | Look up supported operations, fields, commands and limits | [Capabilities](capabilities.md), [UI](ui.md), [CLI](cli-reference.md), [native protocol](native-protocol.md), [app contracts](contracts/effectcraft.md) |
| Integration contract | Connect a new app to canonical platform behavior | [Adapter guide](adapter-guide.md), [beta design](beta-design.md) |
| Evidence and testing | Reproduce checks and understand what they establish | [Testing walkthrough](../../TESTING.md), [current status](../../STATUS.md), [source history](source-and-verification-record.md) |

Author and user guides have progressive sections for the actual task sequence.
Detailed formats belong in the linked references rather than separate competing
schemas in each tutorial.

## Parts and workflows

| Part | Reader and workflow | Detailed documentation | Canonical source |
|---|---|---|---|
| Package identity and archive layout | Author: choose an ID and package referenced files | [Manifest and authoring](author-guide.md), [distribution](store-and-distribution.md) | [Manifest](../../crates/ext-core/src/manifest.rs), [unpacking](../../crates/ext-store/src/unpack.rs) |
| Package dependencies and semver | Author or user: resolve compatible releases | [CLI](cli-reference.md), [store](store-and-distribution.md) | [Versions](../../crates/ext-core/src/version.rs), [Manager](../../crates/ext-store/src/manager.rs) |
| Contract compatibility | Author: declare per-app capability needs | [Capabilities](capabilities.md), [author guide](author-guide.md) | [Contracts](../../crates/ext-core/src/contract.rs), [known app table](../../crates/ext-cli/src/known_apps.rs) |
| Wasm ABI and sandbox | Author: build a filter or session | [Author guide](author-guide.md), [sandbox explanation](how-it-works.md#the-wasm-sandbox) | [Host](../../crates/ext-host/src/wasm.rs), [SDK](../../crates/guest/src/lib.rs) |
| Native executables and protocol | Author: package a platform helper and handle consent | [Native reference](native-protocol.md), [author guide](author-guide.md) | [Native host](../../crates/ext-host/src/native.rs), [example](../../examples/native-system-stamp/README.md) |
| Permission declaration and grants | User or adapter author: grant least required access | [Authorization flow](maps.md#5-host-call-authorization), [capabilities](capabilities.md) | [Vocabulary](../../crates/ext-core/src/permission.rs), [authorization](../../crates/ext-host/src/api.rs) |
| Live revocation and app reload | Adapter: stop invalid work without unrelated reloads | [Adapter guide](adapter-guide.md#6-grants-and-reloading), [revocation](how-it-works.md#revocation-in-practice) | [GrantView](../../crates/ext-store/src/view.rs), [per-app fingerprint](../../crates/ext-store/src/state.rs) |
| Shared install and management | User: browse, install, set up, disable or remove | [User guide](user-guide.md), [install flow](maps.md#3-discover-install-and-consent) | [Manager](../../crates/ext-store/src/manager.rs), [manager UI](../../crates/ext-ui/src/lib.rs) |
| Launcher parity | User: configure one app without requiring the launcher | [Testing](../../TESTING.md#7-artcraft-launcher), [user guide](user-guide.md) | [Launcher plugin UI](https://github.com/arnav-hiwarkar/craft-launcher-plugin-beta/blob/plugin-beta/crates/ui-egui/src/plugins.rs) |
| Consumer and author CLI | User or author: use the same platform from a terminal | [Complete CLI reference](cli-reference.md) | [Consumer parser](../../crates/ext-cli/src/lib.rs), [author tools](../../crates/ext-cli/src/author.rs) |
| Catalogue, identities and admission | Maintainer: admit a vendor and public-source listing | [Store and distribution](store-and-distribution.md), [maintainer procedure](maintainer-guide.md) | [Catalogue tools](../../crates/ext-cli/src/catalogue.rs), [policy](https://github.com/arnav-hiwarkar/craft-plugins/blob/main/MAINTAINERS.md) |
| Signatures, freshness and assets | Maintainer or user: verify a catalogue snapshot and download | [Store reference](store-and-distribution.md#catalogue-format-implemented) | [Verification and fetch](../../crates/ext-store/src/catalogue.rs), [catalogue types](../../crates/ext-core/src/catalogue.rs) |
| Reports and kill list | User or maintainer: report and triage a harmful release | [Reports](store-and-distribution.md#reports-and-the-kill-list), [publication flow](maps.md#10-catalogue-publication-and-revocation) | [Report form](https://github.com/arnav-hiwarkar/craft-plugins/blob/main/.github/ISSUE_TEMPLATE/report.yml), [revocations](https://github.com/arnav-hiwarkar/craft-plugins/blob/main/revocations.toml) |
| Updates, pins and rollback | User: retain or restore a compatible version | [Version flow](maps.md#8-updates-pins-and-rollback), [CLI](cli-reference.md) | [Store transitions](../../crates/ext-store/src/store.rs), [Manager](../../crates/ext-store/src/manager.rs) |
| Events and Undo | Adapter: capture scope and finalize atomically | [Event sequence](maps.md#4-one-session-event-and-one-undo-step), [adapter guide](adapter-guide.md#4-transactions-one-event-one-undo-step) | [AppApi](../../crates/ext-host/src/api.rs), [PhotoCraft owner](https://github.com/arnav-hiwarkar/photocraft-plugin-beta/blob/plugin-beta/crates/engine/src/plugin_sessions.rs) |
| Tool gestures and overlays | Author or user: drag, commit or cancel one gesture | [Gesture map](maps.md#7-tool-gesture-lifecycle), [UI reference](ui.md) | [EffectCraft runtime](https://github.com/arnav-hiwarkar/effectcraft-plugin-beta/blob/plugin-beta/crates/plugins/src/runtime.rs), [FigCraft runtime](https://github.com/arnav-hiwarkar/figcraft-plugin-beta/blob/plugin-beta/crates/plugins/src/runtime.rs) |
| Panels, windows and forms | Author: send UI data and handle revisioned values | [UI](ui.md), [UI sequence](maps.md#6-shared-ui-and-future-framework), [author guide](author-guide.md) | [UI tree](../../crates/ext-core/src/ui/tree.rs), [forms](../../crates/ext-core/src/form.rs), [renderer](../../crates/ext-ui/src/plugin_ui/mod.rs) |
| Custom canvas input and drawing | Author: use host-rendered draw ops and pointer events | [Canvas reference](ui.md#canvas), [SDK](author-guide.md) | [Drawing](../../crates/ext-ui/src/plugin_ui/paint.rs), [input renderer](../../crates/ext-ui/src/plugin_ui/egui_engine.rs) |
| Framework and theme transition | Adapter: replace renderer without changing plugins | [Framework handoff](framework-integration.md), [UI relation](ui.md#relation-to-craft-framework) | [Engine boundary](../../crates/ext-ui/src/plugin_ui/mod.rs), [component fallback](../../crates/ext-core/src/ui/tree.rs) |
| Picked files and partial effects | PhotoCraft author: ask for a handle and publish safely | [File capabilities](capabilities.md#file-access-in-photocraft), [transaction map](maps.md#4-one-session-event-and-one-undo-step) | [File operations](https://github.com/arnav-hiwarkar/photocraft-plugin-beta/blob/plugin-beta/crates/engine/src/plugin_ops.rs), [owner transaction](https://github.com/arnav-hiwarkar/photocraft-plugin-beta/blob/plugin-beta/crates/engine/src/plugin_sessions.rs) |
| Project references and final export | User: open without plugins while retaining data | [Project flow](maps.md#9-opening-a-project-with-missing-plugins), [user guide](user-guide.md) | [Reference types](../../crates/ext-core/src/project.rs), [manager check](../../crates/ext-store/src/manager.rs) |
| Safe mode, failures and Resume | User: recover without running plugin code | [User guide](user-guide.md), [safe mode](how-it-works.md#safe-mode) | [EffectCraft](https://github.com/arnav-hiwarkar/effectcraft-plugin-beta/blob/plugin-beta/crates/plugins/src/runtime.rs), [PhotoCraft](https://github.com/arnav-hiwarkar/photocraft-plugin-beta/blob/plugin-beta/crates/plugins/src/runtime.rs), [FigCraft](https://github.com/arnav-hiwarkar/figcraft-plugin-beta/blob/plugin-beta/crates/plugins/src/runtime.rs) |
| Examples and multi-app packaging | Author or tester: use complete reference packages | [Examples index](../../examples/README.md), [Cross Review](../../examples/cross-review/README.md), [CLI author builds](cli-reference.md) | [Example source check](../../scripts/check-beta-examples.py), [author build](../../crates/ext-cli/src/author.rs) |
| Testing beside normal apps | Tester: isolate settings and catalogue trust | [Mac setup](installation-and-testing.md), [full checklist](../../TESTING.md) | [Test-key gate](../../crates/ext-store/src/catalogue.rs), [CI workflow](../../.github/workflows/plugin-beta.yml) |
| Community compatibility and follow-up | Author or maintainer: compare legacy tools and plan real ports | [Comparison/backlog](community-comparison-and-backlog.md), [BOUNCraft source review](community-bouncraft-review.md), [NZO evidence limits](community-nzo-review.md), [EC Scripts registry](community-ecscripts-review.md) | Pinned public revisions and release digests in the linked notes; current app contracts remain authoritative. |

Shared source links are relative to this repository. App and catalogue links use
the private GitHub forks so they also work from the published handbook. In a local
sibling workspace their corresponding files can be inspected directly. External
framework links are pinned in the framework handoff.

## Status boundaries

| Statement | Meaning and evidence |
|---|---|
| Implemented | The source defines the behavior. The relevant source and contract are linked above. |
| CI verified | A named workflow run exercised its listed checks at the recorded revisions. See current STATUS; a later source change is not covered automatically. |
| Local installation checked | The Mac beta install fetched run #20, refreshed its signed file catalogue, configured starter sets and exercised Cross Review edit/Undo through each CLI. This is separate from repository-wide verification. |
| GUI manual check | A person exercised the named UI steps. Launching windows or passing CLI smokes does not establish this. |
| Compile-only | A platform compiler accepted the code. No runtime execution is implied. Windows run #20 has this scope. |
| Deferred | No current implementation or delivery guarantee. It must not be drawn as an active workflow. |

## Features that must stay visibly deferred

- Wasm network, clipboard and persistent private-storage operations, background jobs
  and general panel open/close callbacks are not implemented in the current adapters.
- Account sign-in, app reviews and ratings are not implemented. Vendor identity and
  report submission use GitHub, not a Craft login flow.
- Theme/font activation, native effects and pixel transport, temporal/tiling APIs,
  plugin object types, cross-plugin workflows, TUF and native OS confinement remain
  beta deferrals.
- FigCraft has no raster.filter dispatch. PhotoCraft's destructive filters do not
  create a missing-plugin requirement on reopen. Native external effects and
  PhotoCraft's earlier published files cannot be reversed by document Undo.

## Keeping documentation current

When changing behavior, update the authoritative source, contract, detailed guide,
corresponding workflow map and evidence record together. Keep older designs marked
as historical. Do not edit archived source snapshots to make them look current.
Run `python3 scripts/check-plugin-docs.py` to check active guide links, JSON examples,
required coverage pages and diagram fences without compiling apps or running plugins.
This structural check does not prove protocol or UI semantics; source review and
appropriate runtime checks remain necessary.

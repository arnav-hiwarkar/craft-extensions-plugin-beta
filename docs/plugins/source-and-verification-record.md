# Source and verification record

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](architecture.md); where this document disagrees with it, the architecture wins.

This reference captures the investigation state on 2026-11-08. Revisions are evidence anchors, not claims that these branches remain current.

## Workspace

Branch: `feat/artcraft-plugin-system`. The worktree remains at `fd5e2feff6ea25bed371a99cc732b3968d621505`. Fetched `origin/main` is `058b047760dfe5ba51f6f526faa3b0c833552dfa`, so HEAD does not match that fetched main. No merge or rebase was performed. The inspected shared theme files were unchanged across those revisions.

## Existing extension implementation

| Repository and ref | Inspected commit | Evidence |
|---|---|---|
| craft-extensions, wip/extensions-handover | `ffd8512e5a3755637f58cf31cfcf2fdb09cfb943` | Core, store, manifests, palettes, index, unpacking, handoff, and draft threat model |
| craft-launcher, feat/extensions | `6ede77f93fd0d3eaaad43d430f94d703d3564a51` | Local store integration and extension UI |
| craft-launcher, test/ext-plus-ai | `be3bff3e2683df0b73e3bbbc09351b22dc86a800` | Combined approved visual reference, mixed with unrelated AI work. Never a release source |
| photocraft, feat/extensions-theme-adapter | `ecb056cc742848d78f9076893a6b3343e6a75010` | Theme reader, watcher, token mapping, contrast checks, fallback, sibling dependencies |

The complete 16-app survey and selected source hashes are in [the inventory](../plugin-functional-inventory.json). That survey is deeper for EffectCraft, FigCraft, PhotoCraft, and VectorCraft. An entry-point inspection does not prove that every workflow or runtime has been traced.

## Findings that constrain the next design

- Existing schema 1 has Theme, Plugin, and Resource kinds, per-app compatibility, file sizes and SHA-256 digests, and empty permissions. Plugin packages allow the module and optional thumbnail, rather than the proposed arbitrary contribution bundle.
- Unknown fields can be ignored by old parsers. Security-critical capabilities require explicit schema and interface negotiation.
- The store validates bounded archive content, stages files, locks writes, and commits state atomically. It has no networking or WASM execution layer.
- The host supplies the trust tier. This is a provenance integration boundary, not evidence of a demonstrated remote exploit.
- Default admission includes FirstParty and FromFile. ReviewedThirdParty is feature gated. Local-file modules still require executable admission and runtime checks.
- Index version and expiry checks exist. Signatures are absent. Revocation lookup exists without complete startup, enable, and load enforcement.
- Installation enables compatible app entries and replaces the current record, then removes the previous version directory. Active-job retention, grants, and stronger rollback need additional design.
- Shared WidgetPalette applies widget colors. Packaged ThemeFile represents a user theme and app overrides. Neither replaces the other.
- EffectCraft's source shows different desktop and fresh CLI loading paths and unresolved-effect omission in rendering. This is a source-derived concern, not a reproduced export defect.
- FigCraft's headless backend drops UI requests, including the save-dialog artifact path. Headless artifact destinations need an explicit contract.
- An external process alone is not an OS sandbox. GPU access, native execution, and audio deadlines each require their own proven boundary.

## Live issue context inspected

- [craft-libs #94](https://github.com/storytold/craft-libs/issues/94) is closed. Its former phase/tier process is not governing architecture. The closing discussion points back to narrow sharing with real consumers.
- [#115](https://github.com/storytold/craft-libs/issues/115) is an open candidate inventory, not proof of implementation.
- [#79](https://github.com/storytold/craft-libs/issues/79) is an open governance proposal.
- [#126](https://github.com/storytold/craft-libs/issues/126) discusses archive preflight before allocation. A referenced PR is not evidence that a usable helper is merged.
- crafthub's inspected README says "Placeholder for merges". Catalogue ownership and permission remain unresolved.
- The AI-store shared-folder decision is reported in the handoff. Its original decision thread was not independently verified.

## Verification limits

Repository refs were fetched and source was read without changing application checkouts. Documentation checks cover local links, whitespace, inventory structure, and source archive preservation. No app UI was exercised. No Rust build, cargo test, clippy run, benchmark, fuzz run, clean-checkout CI, package build, signature review, or independent security certification ran here.

The supplied handoff's test counts and build reports remain attributed historical reports. They must not become claimed verification for this worktree.

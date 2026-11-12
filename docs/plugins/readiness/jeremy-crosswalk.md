# Jeremy handoff crosswalk for implementation readiness

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: root's initial crosswalk, 2026-11-08. Task/status statements below are historical reports from the [complete supplied artifact](../sources/jeremy-extensions-todo.txt), not work rerun in this phase. The [new decisions](../draft/decisions.md) supersede recommendations where specified. Coordinator/worker evidence will assess actual source without overwriting this original crosswalk.

## Reuse and blockers

| Handoff work | Reuse | Remaining work or changed direction |
|---|---|---|
| Core/store on `ffd8512e` | Manifest/ID/file validation, bounded unpacking, staging, writer lock, atomic state, generation and theme reader | Retained source/digests/manifest, explicit per-app grants/enablement, multi-release retention, stronger crash recovery and load-time eligibility are prerequisites |
| Launcher feature `6ede77f9` | Local-store UI and integration | No completed authenticated network catalogue/download round trip established; sibling dependency pins need clean-checkout proof |
| PhotoCraft theme branch `ecb056cc` | Token mapping, contrast/fallback, watcher | Verify integration at actual candidate; filter/store host and broader persistence/CLI gaps remain separate work |
| Combined showcase `be3bff3e` | Approved UI reference | Not release source; extract into a clean candidate after real infrastructure works |
| n1–n4 | Metadata index, fetch/cache/throttle, hash-pinned download, labels/updates | New reviewed metadata repo and publisher GitHub assets are current choices; authenticate production official metadata too |
| n5 | Bounded download origin policy | Its historical “everything outside Storytold refused” is not the final community policy. Permit only reviewed exact GitHub assets from enrolled publishers after gate acceptance; no unrestricted GitHub URL rule |
| n6 | Preserve web read-only listing target | Do not assume native install/worker contracts are available in web targets |
| sg2–sg4 | Signature/root lifecycle, freshness and revocation requirements | Establish vetted protocol/client ownership and actual enable/start/load/host-call enforcement; lookup helpers alone are insufficient |
| sg6 | Preserve schema 1 empty permissions/no imports; validate modules on install/load | Successor capability contracts explicitly version and enforce allowed host operations. Do not silently widen schema 1 to support FigCraft commands |
| sg7 | New authority/publisher/downgrade needs explicit handling; no implicit updates | Preserve its historical no-auto-third-party-update default unless user explicitly changes policy; exact-release review, project pins and unchanged rights still apply |
| sg8 | Parser/install fuzzing and never-crash regression evidence | Future candidate/runtime work; no fuzzing/build performed in this phase |
| sg9/d9 | Source/commit/build evidence | Mandatory hub-build proposal superseded by publisher-hosted compiled releases. Optional verified provenance is honest evidence, not a safety certificate |
| sg10 | Human checklist, disclosure/takedown, proposed two-maintainer governance | Latest user requires human review; precise reviewer count, staffing and urgent handling must be reconciled rather than silently discarded |
| sg11–sg13 | Community UI/admission feature behavior, independent review and explicit gate approval | Gate remains closed; source/build/profile/runtime and policy acceptance required before enablement |
| pl1–pl3 | Real PhotoCraft store-filter/invert pilot with interface mismatch handling | Finish existing vertical slice rather than build only demo catalogue UI; no claim it already passed |
| pl4/r4/plc1/plc2 | Shared runtime only with real consumers; app adapters, broad capability matrix | Audit app operations and authority first, generate useful SDK/CLI contracts, keep domain types in apps |
| ui-gate/ui-port/ui-link/ui-verify | Actual package round trip, clean UI extraction and exact-candidate experience | Root/app filters and cold/running deep links must agree; demo items never impersonate installs |
| ship1–ship4 | Pinned clean checkout, candidate packages, source review and separate release approval | Not authorized here; no push/PR/release or candidate build from the survey |
| x1/x2 | Typed resources through actual shared importers | First importer selection needs real definitions and hostile-input fixtures, not an invented format |
| theme1/theme2/wf1/wf2 | Theme round trip and workflow readiness with no install-time execution | Design supported data now; execution/adoption requires app contracts; ThemeCraft remains future |
| rank1/rank2 | Explainable source-labelled ranking without hidden activity tracking | Current choice is official precedence and cached GitHub repository stars, not download-count ranking |

## Initial biggest changes to investigate

1. The store currently cannot represent the identity, grants and multi-version lifetimes the design promises.
2. Application engines expose different operations and persistence/media contracts; a common SDK needs generated contracts and app-owned adapters, not an unrestricted dispatcher.
3. Existing app registration and lookup behavior must protect builtin identifiers/default handlers from extension replacement. Source audit must prove what is already enforced.
4. Fresh CLI, headless output and required-effect failure handling need actual app traces, not a manager-only change.
5. Production catalogue authenticity, review of exact release bytes, revocation and safe GitHub download are incomplete; stars are advisory ranking data.
6. Developer UX needs one straightforward create/check/test/pack/release/submission path with generated metadata and explicit per-app authority.
7. Native/GPU/audio and large resources need dedicated contracts and measured limits; “compiled” or “separate process” does not establish confinement.

The investigation must order these into small useful units. It must not flatten every app into one engine or delay real SDK/host proof until all sixteen integrations are complete.

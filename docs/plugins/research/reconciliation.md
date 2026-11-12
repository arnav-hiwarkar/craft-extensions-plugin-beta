# Independent findings and reconciliation

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: primary researcher's assessment, 2026-11-08. The three independent notes are preserved. Their recommendations and wording remain their own; this record states which findings the synthesis adopts, qualifies or leaves open. None is user agreement or runtime certification.

## Contracts and trust

| Independent finding or recommendation | Disposition in the design |
|---|---|
| Installed records lack source/digest continuity and the verified manifest | Adopt. The reconciler opened Installed, State, Reader and Store at `ffd8512e`. Retain verified identity/inventory as prerequisites to reliable load/update checks |
| Development-installed gated packages can be visible to release readers | Adopt as a source-derived policy gap. Reapply this host build's admission policy at resolution/load. No executable exploit was reproduced |
| Updates re-enable disabled packages and delete previous files | Adopt. Preserve preference and pin active readers/resources before deferred collection |
| Old writers discard unknown fields | Adopt. Explicit state negotiation is needed; preserving unknown fields alone cannot teach an old host new authority semantics |
| Source identity equals current root-key fingerprint | Modify. Enrol a stable identity anchored in authenticated key history so normal rotation does not create a new package source |
| Release identity is the canonical manifest hash | Leave exact encoding open. Retain exact manifest bytes and inventory, and separately retain the archive digest authenticated by distribution; do not conflate the two |
| Split installed state, grants and revocations into independently atomic files | Qualify. A writer lock does not make lock-free readers see a coherent generation. Prefer one authoritative snapshot or one atomic root selecting immutable generation files |
| In-memory module bytes mean executable jobs need no leases | Qualify. True only for those already acquired bytes. Lazy module/resource reads and active multi-stage work still need lifetime ownership. A time grace period is insufficient |
| App-bundled first-party allowlist before TUF | Keep as optional preliminary test distribution. It does not finish the catalogue/freshness/revocation path. Recommend an authenticated full first-party distribution pilot |
| Single-digit-millisecond hashing estimate | Do not adopt as evidence. Hash/compile/copy costs need measurements on candidate hardware; no threshold follows from this estimate |

Source: [Opus contracts and trust note](opus-contracts-trust.md). Findings reflect its independent named-revision reading and the bounded reconciler checks recorded in [verification](verification.md).

## App, CLI and persistent state

| Independent finding or recommendation | Disposition in the design |
|---|---|
| Fresh CLI does not initialize installed plugins | Adopt for the inspected EffectCraft/FigCraft/PhotoCraft paths. Do not extend “all apps” to uninspected apps. The baseline actually records VectorCraft explicitly attaching persisted stores. FilmCraft's inspected paths establish no processing plugin host, so an eligibility failure there is not proven |
| Missing effects/smart filters can be omitted | Adopt as source-derived concern. The reconciler confirmed PhotoCraft `apply_to_surface` returns None on missing or failed plugin. Final output must report a required dependency failure; application runtime and exit statuses were not exercised |
| FigCraft offered-file UI is cleared headlessly | Adopt. The reconciler confirmed `ui_requests.clear()` after execution. Use an explicit artifact/destination result. Source inspection does not establish every CLI output route or exact exit code |
| EffectCraft/PhotoCraft/FigCraft saved references lack exact release identity | Adopt. Migration belongs to each app; preserved opaque instance state is distinct from a pinned runtime release |
| WASM paths comply with sandboxed-first; native paths are exceptions | Reject as an approval or compliance conclusion. WASM is a useful isolation mechanism, but full host-call/compile/budget/revocation evidence is incomplete. No accepted native exception exists |
| Labelled first-party native tier could be an alternative default | Reject as an inferred permission. First-party status cannot waive the sandbox requirement. Legacy native functionality remains inventoried separately; new native admission requires proven confinement and scope |
| Export may allow skip with a report | Retain as optional later product behavior only. Recommended default is failure for missing required processing. No skip flag is selected or implied by existing CLI vocabulary |
| FilmCraft developer_mode could become its execution gate | Do not adopt automatically. A developer preference is not release admission, module validation or a capability policy |
| ABI-specific pixel/time/state differences and pooled-state risks | Adopt as requirements for fixtures. Preserve existing ABIs, state isolation/reset and contract-specific data. No pooled-state bug or performance result was reproduced |

Source: [Haiku app/CLI/state note](haiku-app-cli-state.md). Its source table and “not traced” list remain important limits, including EffectCraft exit status, PhotoCraft attached mode and FilmCraft explicit background export behavior.

## Authors, themes, resources and recovery

| Independent finding or recommendation | Disposition in the design |
|---|---|
| Generate repeated version/checksum/inventory data | Adopt. Keep identity, rights and domain contracts explicit; generated facts come from one authoritative metadata source |
| ThemeFile, WidgetPalette and app tokens are separate | Adopt. App token mapping is the compatibility boundary, not a suite-wide token struct |
| PhotoCraft theme watcher is opt-in and polling; adapter exists | Adopt as branch-specific source evidence. A watcher definition is not proof the final app wires or visually validates it. No complete theme integration ran |
| FilmCraft has Medium plus Dark/Light and more colour fields than the package's override cap | Adopt as mapping constraint. Prefer sparse semantic overrides and preserve user appearance/contrast/highlight precedence. A complete token dump is not required; do not raise caps just to copy every app token |
| Translucent PhotoCraft override becomes opaque | Retain as source-derived mapping concern. Exclude inappropriate translucent tokens from the current override allowlist; alpha changes require explicit format design |
| ThemeCraft owns editable source and exports compatible ThemeFile | Recommend as roadmap direction. Round-trip supported fields explicitly; do not claim a typed parser preserves unknown editor metadata |
| Resource subtypes and importer gates needed | Adopt. Resource kinds and importer contract versions should be explicit in successor metadata. Workflows remain data-only on install; execution is separately scoped |
| Streaming install required for large resource types | Qualify. Enforced peak-memory/byte budgets are required. Streaming is one implementation option, not an approval to redesign the installer without measured need |
| Auto-revert once a new release fails to load | Qualify. Rollback must still check revocation, grants, exact project selection and state compatibility. Loading once does not prove every app/job/resource consumer is done with old bytes |
| Compatible newer release may run on reopen with a warning | Prefer exact selected releases for persistent features, with explicit migration/upgrade. A warning alone does not preserve render behavior |
| Corrupt-state recovery can start empty and rebuild | Qualify. Preserve unreadable state and last-known-good data, keep trusted freshness/revocation restrictions, and do not invent grants from package folders. Orphans may be verified and reused or quarantined; never remove active uncommitted installs by age alone |
| Omit provenance for proprietary packages | Qualify. Do not publish private repository URLs or commits without policy, but record an authenticated release origin and any selected build attestation. Public product links and private operator records serve different purposes |
| Licence optional for proprietary packages | Do not adopt as a blanket rule. Record explicit licence/usage terms. Distinguish runtime-distributed proprietary product code from dependencies/assets governed by the repository allowlist. Adding disallowed dependencies/assets would need separate permission |
| Fonts as resources | Keep the house rule: font files live in craft-fonts with proper attribution. An extension package does not override it |
| Keep settings on uninstall, app checkbox defaults and first resource type | Open product details. Recommend preserving project data, separating deletion choices, and explicit per-app enablement. Select the first resource consumer after its importer evidence, rather than naming one by guess |

Source: [Haiku author/resources/recovery note](haiku-author-resources-recovery.md). It preserves the Jeremy task crosswalk and 35 edge cases. Later source-dependent choices, including the first LUT/resource consumer, need their own bounded live check before implementation; no such choice is made here.

## Resulting recommendation

Keep the existing installer/store, strengthen its identity and authority contracts, finish the existing first-party PhotoCraft slice, and then validate the expanded host model with EffectCraft and FigCraft in the accepted order. Adopt common lifecycle policy while leaving document/media behavior in app adapters. Keep production community admission closed pending its own evidence and explicit approval.

The [ordered decision register](decisions.md) is the user-facing agenda. Optional roadmap work stays separate from the minimum package/host proof. No further delegate round was launched for these local reconciliation corrections.

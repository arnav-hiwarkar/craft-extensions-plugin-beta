# Opus collaboration and reconciliation

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: primary design synthesis, 2026-11-08. The requested Opus 5.5 medium collaborator independently proposed architecture and decisions in [its preserved note](opus-review.md). This record resolves disagreements and links corrections. No additional delegate round was started. Recommendations below are draft engineering defaults, not new user agreements or runtime security certification.

## Adopted architectural defaults

The primary and Opus agree on one shared extension/store system, app adapters, plugin/CLI behavior reuse with caller-specific authority, explicit per-app consent, compiled GitHub Release assets, metadata-only catalogue hosting, local installs under the same runtime policy, exact live project references and the closed release gate.

The review made the draft more concrete. Adopt a Rust operation registry in each app as the interface authority, generating versioned inventories, SDK bindings, CLI discovery, documentation and conformance lists. Use typed wrappers and bounded policy-checked dispatch for new command/query contracts. Existing media ABIs retain purpose-specific adapters/buffer paths. WIT is a possible generated backend format when a real runtime/language need justifies it.

Keep one atomic versioned state snapshot initially. It contains source-bound multi-release/default/pin/grant records and trusted-policy identities, referencing immutable package/metadata bytes. This avoids an unnecessary multi-file transaction protocol. One authenticated bounded catalogue data target delivered through established update tooling is sufficient initially. Do not add custom duplicate signatures or a build farm.

## Finding dispositions

| Finding | Resolution | Final location |
|---|---|---|
| C1: install auto-enables all apps and update deletes old bytes | Accepted. Explicit no-auto-enable, preference preservation, multi-release/reader retention and crash recovery are prerequisites, not incidental polish | [Store changes](shared-operations-and-apps.md#required-shared-store-changes), D1 in [delivery](delivery-and-acceptance.md#implementation-order) |
| C2: grant continuity lacks an anchor | Accepted with qualification. Authenticated policy binds UUID and enrolled publisher/source; normalized app/contribution scopes must be unchanged or narrower. Transfers/new contributions need renewed consent; unsigned local changed bytes do not inherit grants | [Consent contract](sdk-and-contracts.md#capability-and-explicit-consent-contract) |
| C3: review scope/staffing/urgent fixes undefined | Accepted. Automated delta report plus stated human checks and named operators; urgent queue retains exact-digest approval. Reject approval after publication. Mandatory two-maintainer/response-time rules remain governance choices | [Review records](security-and-distribution.md#catalogue-repository-and-review-records) |
| C4: catalogue authentication is not implemented | Accepted. No fetched production catalogue, including official, is trusted without verified metadata/root policy. Caller-supplied tier fields cannot create official authority; same-version changes fail | [Authenticated metadata](security-and-distribution.md#authenticated-update-metadata), D3/D8 acceptance |
| H1: unchecked bridge versus policy dispatcher | Accepted distinction. Checked dispatch is allowed. Reject the claim that one import is inherently safest; the policy, parser, argument/scope validation and preserved principal establish enforcement | [Policy dispatch](shared-operations-and-apps.md#policy-checked-dispatch) |
| H2: unselected interface definition blocks action | Accepted. Rust registry generates authoritative inventory now; WIT/component backend does not block first SDK | [SDK registry](sdk-and-contracts.md#operation-descriptions-are-the-shared-base) |
| H3: current revocation range is insufficient | Accepted. Design digest/version/package/source applicability and advisory/block-new/emergency behavior; authenticated restrictions survive repair and govern matching known revoked bytes imported locally too | [Metadata and incident rules](security-and-distribution.md#authenticated-update-metadata) |
| H4: project pins require multi-release state | Accepted. Release/default/pin ownership is explicit. Recent-project history can offer best-effort retention but is not exhaustive proof for unopened projects | [Retention](security-and-distribution.md#state-retention-and-concurrent-clients) |
| H5: local files can replace catalogue identity | Accepted. Local origin does not acquire authenticated identity, grants or update stream through a matching UUID/name. Explicit development/side-load transitions reset applicable authority | [Local files](security-and-distribution.md#local-files-and-development) |
| H6: repository allowlist is not ecosystem product policy | Accepted clarification. No catalogue licence list is selected by this draft. Local, community, official proprietary and bundled dependency/asset policies are distinct decisions | [Licence policy](security-and-distribution.md#licence-and-proprietary-policy), [open details](decisions.md#remaining-decisions) |
| M1: missing contribution families | Accepted. Add format open/save handlers, settings, keymaps, localization, context menus/inspectors, validators/preflight and headless-only compiled operations. Name future inter-plugin APIs/resident services | [Contribution coverage](shared-operations-and-apps.md#full-contribution-coverage) |
| M2: large model/data resources need a practical path | Accepted. Separate exact data manifests/readiness/retention and host downloads from code installation; verify GitHub per-file constraints and specify bounded chunks or future approved origins | [Resources/profiles](delivery-and-acceptance.md#workload-profiles-and-resource-readiness) |
| M3: avoid unnecessarily restrictive budgets | Accepted. Interactive/preview/background/audio profiles, measured defaults, per-app/package increases within host ceilings and useful diagnostics; no unlimited switch | [Profiles](delivery-and-acceptance.md#workload-profiles-and-resource-readiness) |
| M4: development mode versus release profile | Accepted with gate constraint. Labelled local/dev installs can use the same controls; neither dev store nor FromFile bypasses blocked release policy or known digest revocations | [Local/development](security-and-distribution.md#local-files-and-development) |
| M5: CLI management missing | Accepted. List/search/install/update/enable/grant/pin/remove/repair are defined as intents. Explicit technical-user grants are proposed, separate from normal execution, with exact scope acknowledgment; no plugin/unauthenticated remote grant path | [CLI management](delivery-and-acceptance.md#cli-management-and-execution) |
| M6: support/deprecation needs a public policy | Accepted as proposed starting window with maintainer confirmation. Do not promise unsafe compatibility or unsupported app cadence | [Compatibility maintenance](delivery-and-acceptance.md#compatibility-maintenance) |
| L1/L2: vocabulary and excessive hedging | Accepted. Define terms once and lead docs with concrete proposed defaults; preserve real policy uncertainties rather than claiming user agreement | [Overview](README.md), each companion's opening choices |

## Alternatives not selected

| Opus suggestion | Primary decision and reason |
|---|---|
| GitHub numeric IDs are the continuity root | Use them as consistency evidence, with identity authorized by enrolled authenticated catalogue records. Unverified permanence/transfer assumptions cannot be the trust root |
| One dispatch import is safer than many host functions | Central policy is the invariant. Generated wrappers can converge on it; large typed media paths need not be forced through one serialized payload |
| Signed index wrapped in TUF | Use an authenticated catalogue target within the selected protocol. Avoid an extra custom signature layer with ambiguous authority |
| Daily signed star refresh | Cache repository statistics in a bounded publication job with timestamp. Exact daily cadence is optional operations tuning, never a security or availability dependency |
| Two-maintainer emergency fast lane with after-the-fact publication record | Prioritize review and revocation, but require human exact-release approval before installation availability. Do not invent a new approval count or imply review can follow executable publication |
| Launcher-only downloads initially | Prefer in-app manager and launcher using the same client. This meets the user experience without maintaining duplicate implementations or requiring a separate running application |
| Data-only community kinds might bypass sg13 | Not adopted. The current gate stays closed for all covered community admission until an explicit policy change; data parsers also have hostile-input risks |
| Every declared permission must be used | Check actual imports/calls fit declarations and warn on excess declarations. Static analysis cannot generally prove every runtime branch uses a permission, and optional features legitimately declare optional rights |
| Keep all installed work functioning offline | Qualify: eligible locally complete functionality works offline; remote providers/resources and known revoked releases have explicit unavailable states |
| Recent projects protect all saved references | Best-effort pins are helpful; unopened/moved/offline projects cannot all be known. Explicit pins/exported dependency information and honest removal warnings remain necessary |

## Established-system evidence

Opus labelled VS Code, JetBrains, Figma, Zed, Homebrew, Flathub and Android comparisons as unverified analogies. The primary subsequently retrieved official VS Code contribution-point, Figma manifest and Zed extension-development pages. The [delivery reference](delivery-and-acceptance.md#established-practices-and-simplicity) uses only the specific supported patterns and states their limits.

No claim about those products certifies this design, and no general analogy becomes a copied dependency/licence policy. GitHub release/asset/rate-limit documentation and TUF/WIT sources were consulted by the primary earlier in this draft. No TUF library licence, concrete runtime replacement or native confinement mechanism was selected.

## Remaining decisions for the user

Keep the remaining discussion short and consequential: licence policy for catalogue/local/official proprietary packages; catalogue repository/operator/key custody; confirmation of per-executable-release review and staffing; optional explicit CLI grant flow; automatic-update preference/default; supported higher resource ceilings after measurement; and private-release credentials when required. UUID aliases, app registry, single state snapshot, in-app shared client and broad family catalogue are concrete draft defaults, not another round of unanswered architecture alternatives.

The [decision record](decisions.md) distinguishes these proposals from accepted product direction. No approval to implement, publish, create a remote or open sg13 has been inferred.

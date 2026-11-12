# Decisions and validation order

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: discussion register, 2026-11-08. Every new choice below is open. Recommendations are the primary researcher's synthesis; delegate agreement does not make them user decisions. Design only is authorized. Implementation, publication and opening sg13 are excluded.

## Choices already made

The [accepted record](../../plugin-functional-decisions.md) chooses sandboxed executable plugins first, official and third-party distribution, existing app ABIs through adapters, multiple contributions per package, headless editing/compute operations with separate UI-only contributions, automatic fresh-CLI eligibility for installed/enabled/compatible/permitted modules, and a stable low-burden SDK. EffectCraft then FigCraft is the functional validation sequence. PhotoCraft/FilmCraft and theme requirements must be considered. Themes means themes, not organizations.

The existing craft-extensions implementation is the reuse baseline. Third-party release support stays switched off until its separate gates and approvals pass. Catalogue ownership, schema, source policy, detailed capabilities, runtime/backend choice and rollout mechanics were not accepted. Earlier functional-model questions that repeat the accepted defaults do not reopen them.

## Ordered decisions for the user

These questions are ordered by the number of later contracts they affect. They are a discussion agenda, not an approval checklist to complete before continuing research.

### 1. Bind projects and installs to an immutable release

Recommendation: source-qualified package identity with stable source enrolment across authenticated key rotation, retained verified manifests/inventories, and exact release identity in saved live features. Versions describe releases but do not replace their digests. Updates change defaults for new uses, while existing projects upgrade explicitly.

This makes save/reopen and export reproducible with respect to plugin bytes and prevents a matching name from replacing another source. It costs retained disk space and project-reference migration. A looser compatible-version reference saves retention work but permits changed output on reopen. Existing jobs still require pinning under either policy. Choose exact references for live processing, with an explicit upgrade/bake path?

Evidence: the installed record currently lacks continuity data and deletes previous bytes on update. [Opus findings 1, 3 and 6](opus-contracts-trust.md#ranked-objections); [design identity and lifetime rules](design.md).

### 2. Evolve schema and authority state explicitly

Recommendation: negotiate a successor manifest for contributions/capabilities and migrate installed state to a coherent versioned authority snapshot. Keep schema 1's no-host-import meaning intact. Preserve disabled settings on updates, treat enablement and grants separately, and require every loading host to apply its own build gate.

This means old clients may require an upgrade rather than share new authority-bearing state. Maintaining read-only legacy views can preserve theme compatibility, but they must not become an alternate executable admission path. Unknown-field preservation alone cannot make old executable readers enforce new rights. Use one snapshot first; split files only with an atomic generation selection protocol. Accept an explicit minimum compatible client instead of silently extending schema 1?

Evidence: old writers discard new fields; readers suppress state errors; the gate currently applies on install. [Opus §§5–6](opus-contracts-trust.md#6-schema-migration); [current state source](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-store/src/state.rs).

### 3. Choose the first-party distribution pilot and catalogue owner

Recommendation: finish the existing first-party catalogue path with established authenticated metadata, then use it for the full PhotoCraft theme/invert round trip. Confirm its operator/location; crafthub's placeholder does not establish ownership. Keep one curated default source initially, with explicit added-source enrolment in the community design.

An app-bundled digest allowlist is a smaller optional local test slice if catalogue work blocks runtime validation. It requires app updates to change approved artifacts and does not prove freshness, remote revocation or catalogue acceptance. An unsigned catalogue would leave official packages without the intended authentication; a bespoke signature protocol adds avoidable maintenance. Prefer the full authenticated path, or scope the allowlist only as a preliminary test?

Evidence: [Jeremy's complete handoff](../sources/jeremy-extensions-todo.txt), [security proposal distribution](../../plugin-security-design.md#5-distribution-paths-and-updates), and [Opus catalogue challenge](opus-contracts-trust.md#4-catalogue-provenance-and-signing). Neither this recommendation nor the existing handoff authorizes contacting maintainers or publishing.

### 4. Set update and running-work behavior

Recommendation: preserve active job references and lazy resource access, retain one rollback release plus explicit pins, stop new use on disable/removal, and cancel affected work at a safe boundary for emergency revocation. Deny revoked host capabilities immediately on subsequent calls. Required final output fails visibly; completed native edits remain ordinary undo history.

This consumes disk and requires cross-process reader ownership. A grace period is simpler but cannot protect an arbitrarily long export. Keeping jobs running after an emergency block improves continuity but permits known-dangerous execution. Advisory notices can allow completion separately. Accept this distinction, including a failed export after an emergency revocation?

Evidence: [concurrent cases](opus-contracts-trust.md#8-concurrent-update-revocation-and-removal), [lifecycle update/removal](lifecycle.md#update-rollback-disable-and-remove). Arbitrary unopened projects cannot all be discovered; no retention policy can promise that without explicit pins or packaged project dependencies.

### 5. Bound the first capability and backend scope

Recommendation: prove no-import pure computation in EffectCraft, then a narrow FigCraft selection-read/document-edit operation enforced through its transaction system. Keep GPU work host-mediated and native helpers/arbitrary shaders out of the initial sandbox promise. Use bundled/digest-pinned data dependencies first, with no executable inter-plugin dependencies.

This delays vendor-native modules, unrestricted commands, workflow execution and richer network/account integrations. It preserves existing ABIs only where their operations fit the accepted protection boundary; an adapter cannot declare an unrestricted legacy path safe. Each added capability needs enforcement, denial fixtures, output/host-work budgets and independent review. Is this the right first useful scope, before broader backend experiments?

Evidence: [runtime and theme study](../../plugin-security-runtime-themes.md), [app matrix](app-adapters.md), [capability caveats](design.md#runtime-authority-and-budgets). SoundCraft's real-time runtime choice stays deferred for dedicated evidence.

### 6. Set community provenance and release acceptance

Recommendation: require established authenticated update metadata for production community acquisition, source-qualified revocation, tested effective release-feature behavior and independent security review before sg13. Choose hub-built versus verified publisher-built provenance using an operating-cost assessment before opening the gate.

Hub builds centralize infrastructure and make private-source handling harder. Publisher builds reduce that operating burden but leave more build trust with each publisher. Neither signatures nor attestations prove benign behavior. No monetary or staffing estimate was measured here, so the decision needs an operator and a small costed sample, not an invented price. Require independent review, then commission that provenance cost assessment?

Evidence: [TUF specification](https://theupdateframework.github.io/specification/latest/), [handoff d8–d10 and sg13](../sources/jeremy-extensions-todo.txt). Third-party support remains designed but disabled while this is unresolved.

### 7. Confirm resource and offline defaults, without assuming paid plugins

Recommendation: permit previously verified eligible installed releases offline under cached policy. Require fresh trusted metadata for acquisition. Keep theme selection explicit, resources data-only on install, optional model downloads explicit, and credentials outside documents/packages. Add runtime entitlements only if a product requirement actually needs them.

Offline use cannot apply unseen revocations. Requiring constant connectivity improves policy freshness but prevents disconnected creation/export. Restricted downloads alone may satisfy proprietary distribution without runtime licensing. Which proprietary packages, if any, need restricted downloads or offline-limited execution? That answer determines whether licensing work is needed at all.

Evidence: [security proposal offline policy](../../plugin-security-design.md#9-offline-use-revocation-and-recovery), [workflow recovery](lifecycle.md#offline-use-incidents-and-safe-startup). ThemeCraft authoring, workflow execution, rankings and automatic model management remain optional roadmap scope.

## Proposed validation order

This sequence reconciles the two pilots rather than selecting one in place of the other. Every step below is future implementation/acceptance work requiring its own scope. No runtime check has been performed in this research.

1. Agree identity/state negotiation and read-side eligibility boundaries around the existing core/store. Prove mixed-client behavior, preference preservation, source continuity, hostile archive limits and interrupted publication. Pin actual consumer dependencies for clean-checkout verification.
2. Complete Jeremy's first-party PhotoCraft theme and invert slice: acquire through the chosen first-party path, install, enable, select theme deliberately, run a bounded filter, save/reopen, disable/remove and recover. Validate the real appearance. A local allowlist slice, if selected, remains preliminary to the catalogue round trip. This proves existing store/data/filter integration, not the richer functional model.
3. Validate EffectCraft first for the expanded host model: preserve its ABI, add contribution/release resolution, fresh-CLI parity, independent saved instances, pinned jobs and truthful export failure. Exercise an update during export. Do not expose unsupported helper capabilities as no-import WASM functionality.
4. Validate FigCraft second: one command used by menu/panel/CLI, narrow read/edit grants, denied nested operations, transaction rollback, undo and explicit output artifacts. Keep persistent widgets as a separate extension if they would obscure this minimum proof.
5. Use PhotoCraft's richer media fixtures and FilmCraft's theme/command/job fixtures to test what is actually reusable. Define FilmCraft temporal processing only after a concrete consumer. Standardize purpose-specific interfaces where at least two real adapters agree.
6. Extract approved Extensions UI from the combined showcase into a clean candidate source. Verify pinned dependencies from clean checkout, candidate packaging, effective release features and supported platforms. Never ship the mixed showcase branch as release source.
7. Complete source authentication/recovery, provenance operations, revoked/denied/hostile cases and independent security review against the actual candidate. Only explicit completion and required approval of sg13 can open third-party release admission. Functional community fixtures can be exercised in controlled development profiles without enabling releases.

The smallest complete executable pilot is a real first-party package reaching a real host, running with enforced limits, saving/reopening, failing safely when unavailable, and being removed without disrupting active work. The EffectCraft and FigCraft steps add distinct proof: persistent processing and privileged editor transactions. One cannot substitute for the other.

## Deferred scope and the evidence needed to reopen it

| Deferred work | Reason | Smallest useful future evidence |
|---|---|---|
| Arbitrary shaders/native sandbox backends | Platform and driver confinement unproven | One bounded backend experiment on each supported OS, including cancellation/device failure |
| SoundCraft sandbox runtime | Audio deadlines differ from editor commands | Prepare/process/state fixture with latency, tails, automation and worker-failure measurements |
| Executable inter-plugin dependencies | No demonstrated first-pilot requirement | Concrete consumer plus bounded graph/cycle/version/conflict fixtures |
| ThemeCraft round trips | Existing theme consumer can be proven first | Export/import preserves supported ThemeFile fields and app overrides |
| Workflow execution and missing external nodes | Data import is not permission to run tools | Explicit workflow executor contract, node/version/readiness validation and denied-side-effect fixture |
| Rankings and download analytics | Not required for correctness or first-party pilot | Named owner, measurement semantics and privacy/cost assessment |
| Entitlements/licensing service | Proprietary does not establish a payment requirement | Actual restricted-access product policy and offline acceptance examples |
| Additional app runtimes | Most survey rows are entry-point evidence only | One real app-owned operation and its persistence/CLI/output fixtures |

## What the research verifies

Completed evidence consists of baseline reading, bounded named-revision source inspection, independent notes and documentation checks listed in [verification](verification.md). All executable acceptance, visual theme review, performance measurement, clean-checkout release verification and independent security certification remain pending. Recommendations are ready for discussion; implementation is not authorized by this file.

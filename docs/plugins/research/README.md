# Plugin research and design discussion

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

This study proposes a lifecycle around the existing craft-extensions core/store, with app-owned adapters and explicit execution authority. It is design/documentation only. No implementation, publication, security certification or third-party release enablement is authorized. Themes are in scope; organizations are not.

Later user choices and the requested complete architecture now live in [the current draft](../draft/README.md). Its [decision record](../draft/decisions.md) supersedes earlier open recommendations about catalogue hosting, ranking and common-operation design. This research remains the supporting evidence, not a competing current proposal.

Start with [the reconciled design](design.md), then [the ordered decisions](decisions.md). The main recommendation is to preserve exact release identity and user choices, apply one eligibility policy at every entry point, and keep running jobs' bytes/resources alive through updates. Finish the existing PhotoCraft slice, then prove the expanded functional model with EffectCraft followed by FigCraft.

## Reading map

| Document | Purpose |
|---|---|
| [Reconciled design](design.md) | Explanation of ownership, contracts, tradeoffs and recommended defaults |
| [Lifecycle workflows](lifecycle.md) | Reference for authoring through discovery/install, execution, save/reopen, CLI, update/removal and recovery |
| [App adapter matrix](app-adapters.md) | Shared adapter checklist and app-specific obligations for all sixteen sampled apps |
| [Aspect coverage](coverage.md) | Every aspect-register row, its owner, compatibility/performance implications and pending acceptance |
| [Decisions and validation order](decisions.md) | Consequences and ordered questions, minimum pilot, release evidence and deferred scope |
| [Independent findings and reconciliation](reconciliation.md) | Adopted findings, qualified claims, rejected implications and remaining objections |
| [Verification and evidence](verification.md) | Source anchors, completed docs checks and runtime/CI limits |

## Independent notes

The provider catalog and returned delegate settings confirmed one `claude-opus-5-5` with `effort: medium` and exactly two `claude-haiku-5-5` with `effort: xhigh`, through the cross-provider `claudeAgent` provider. No settings were substituted. No grandchildren or additional review round were requested.

| Note | Independent assignment |
|---|---|
| [Opus contracts and trust](opus-contracts-trust.md) | Contracts, lifecycle consistency, source trust, shared-store reuse and concurrent policy changes |
| [Haiku app/CLI/state](haiku-app-cli-state.md) | Four main app adapters, CLI/job/output behavior, persistence and processing semantics |
| [Haiku author/resources/recovery](haiku-author-resources-recovery.md) | Author burden, themes, typed resources, installation/recovery, offline and proprietary edge cases |

Preserve their evidence and objections, but use [the reconciliation](reconciliation.md) to interpret disagreements. Broad phrases in an independent note do not expand what its inspected source proves. For example, the scoped CLI findings do not establish failure in every suite app, and WASM source inspection does not certify sandbox compliance.

## Authority and limits

The [baseline index](../README.md), [accepted decisions](../../plugin-functional-decisions.md), [security proposal](../../plugin-security-design.md), [source record](../source-and-verification-record.md) and [complete supplied Jeremy artifact](../sources/jeremy-extensions-todo.txt) remain preserved. Historical notes contain superseded proposals; recommendations here are also not user agreement.

All output is local and uncommitted under this directory. Source inspection at named revisions establishes code structure, not current branch state or observed application behavior. Every runtime, visual, performance and release acceptance case is pending unless the verification record explicitly says otherwise. No Rust builds/tests, app launches, dependency changes, CI changes, external messages, pushes or PRs occurred in this study.

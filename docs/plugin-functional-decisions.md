# Plugin functional study decisions

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Recorded on 2026-11-08 from the user's instructions. This is a discussion record, not authorization to implement or publish.

## Agreed direction

- Start with sandboxed executable plugins. Detailed safety mechanisms will be discussed after the functional model.
- Support official and third-party distribution. The exact source formats, discovery, and trust policies remain to be designed. This supersedes the earlier recommendation to defer third-party catalogs.
- Use FigCraft as the second integration after EffectCraft.
- First establish how plugins function throughout the apps: supported operations, editor integration, CLI and automation, author requirements, shared systems, and app-specific systems.
- Conduct two independent investigations, one in this conversation and one using Opus, then reconcile them.

## Functional defaults accepted in the follow-up

- Preserve existing app ABIs through adapters initially, standardizing purpose-specific interfaces where real app requirements agree.
- Allow one package to contain multiple app-specific contributions.
- Expose headless-callable operations for editing and compute contributions, with explicitly UI-only contributions allowed separately.
- Use compatible installed plugins automatically in a fresh CLI session. This selects existing eligible installations, not automatic downloading, new permission grants, or activation of disabled plugins. Exact eligibility and policy checks remain part of the security design.
- Begin implementation with executable contributions while keeping the package envelope extensible to other contribution formats and resources. A specific script runtime or archive format is not selected.
- Keep a stable SDK/package contract with a low contributor burden. Derive metadata and generate boilerplate where possible while retaining required compatibility and capability declarations.
- App-specific adapter implementation remains separate future work. PhotoCraft and FilmCraft requirements and existing shared-theme changes need explicit consideration.
- The next design stage covers security and distribution for ArtCraft-produced proprietary plugins and community plugins, using one Opus 5.5 medium pass alongside the primary researcher.

The user clarified that Teams meant themes only. Organization-managed plugins, team roles, and organizational policy are outside this design pass. Theme add-ons remain in scope.

## Existing implementation handoff supplied during the security study

The user supplied Jeremy's Craft Extensions To-Do text from the Claude artifact. It identifies `storytold/craft-extensions` on `wip/extensions-handover`, launcher branches `feat/extensions` and `test/ext-plus-ai`, and PhotoCraft's `feat/extensions-theme-adapter`. Inspect and reuse that implementation before deciding module ownership or creating competing infrastructure in craft-libs.

The handoff's release plan keeps third-party support switched off until its security gates pass. Agreement to support third-party distribution does not authorize bypassing that release gate. Its PhotoCraft-first theme/filter pilot and this conversation's EffectCraft-then-FigCraft functional pilot need explicit reconciliation; neither order silently replaces the other.

## Still open

The latest architecture discussion and [follow-up decisions](plugins/draft/decisions.md) now settle additional product direction: a common manager with app-specific configuration and explicit grants, shared plugin/CLI functionality, a plugin-first SDK, GitHub Release binaries, a new reviewed metadata repository, official precedence and GitHub-star popularity, local installation under the same runtime protections, offline use where functionality permits, app-local logs, and a restricted plugin-licence policy whose exact list is open.

The user requests a complete architecture draft with Opus covering the full application needs. Exact package schema, crate split, execution ABI/backend, CLI vocabulary, state layout, numerical budgets, licence list, and detailed rollout remain proposals. GPU and native backends remain within the sandbox requirement, not an approval to run unrestricted code. Proprietary distribution is a requirement to design for, not permission to add dependencies or assets outside the repository's licence rules. The request remains design/documentation only; the third-party release gate stays closed.

The [initial ideas](plugin-architecture-ideas.md), [lifecycle review](plugin-lifecycle-review.md), and [resource register](plugin-resources.md) supply background. Their unresolved suggestions are not additional user decisions.

## Outputs of this study

The user subsequently requested that all information, resources, decisions, ideas, alternatives, and detailed discussion be stored clearly in this worktree's docs. [The plugin docs entry point](plugins/README.md) organizes that record. The separate Codex 6.1 Sol high research thread was requested with Opus and a small number of Haiku xhigh reviewers. This authorizes further design research and documentation, not plugin implementation.

- [Primary investigation](plugin-functional-primary.md).
- [Independent Opus investigation](plugin-functional-opus.md).
- [Combined functional model](plugin-functional-model.md).
- [Jeremy's supplied handoff summary](jeremy-extensions-handoff-summary.md).
- [Current security and distribution proposal](plugin-security-design.md), based on the existing craft-extensions source.

The three study documents will distinguish observed implementations from proposals and unresolved functional choices. Source inspection is not runtime verification.

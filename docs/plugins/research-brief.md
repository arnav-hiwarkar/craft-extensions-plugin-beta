# Next plugin design research brief

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](architecture.md); where this document disagrees with it, the architecture wins.

Requested on 2026-11-08. The user asks for a separate Codex 6.1 Sol thread at high reasoning effort to examine plugin and add-on management thoroughly, using Opus and a small number of Haiku xhigh delegates. Work is design and documentation only.

## Objective

Turn the existing research into a complete, understandable design discussion across authoring, distribution, management, execution, app adapters, themes, resource add-ons, CLI, persistence, failure recovery, and security. Explain what happens at each boundary, who owns it, which decisions remain open, and which evidence can prove the design. Recommend useful defaults without claiming the user approved new mechanisms.

Read [the index](README.md), accepted decisions, current security proposal, source verification record, and [aspect register](aspect-register.md) before investigating. Use detailed studies and the discussion archive as needed. Preserve the complete supplied handoff and all existing research files.

## Requested collaboration budget

- Codex `gpt-6.1-sol`, `reasoningEffort: high`, owns synthesis, cross-app workflows, decisions, and final documentation.
- One Opus 5.5 delegate at medium effort independently challenges shared contracts, lifecycle consistency, trust, distribution, and existing-source reuse.
- Two Haiku 5.5 delegates at xhigh effort, one for app/CLI/project persistence and processing edge cases, the other for themes/resources/author experience and install/update/recovery edge cases.
- Use cross-provider delegate_task with the requested settings. Check provider/model availability first. If a setting is unavailable, report it rather than silently substituting. Children work themselves and start no further agents.
- Give delegates focused briefs and distinct output paths under `docs/plugins/research/`. They may inspect source and write only their own research note. Codex alone edits shared summary documents. One coherent review round, followed by local reconciliation of small corrections.

## Required outputs

Create a research index, reconciled design, lifecycle workflow reference, per-app adapter matrix, and decision register under `docs/plugins/research/`, split into readable files. Preserve each delegate's evidence and objections. Cover every aspect register row, stating deferred items and their reasons.

For each material issue, record the normal path, adverse cases, responsible module/app, proposed data and interfaces, options, recommended default, compatibility implications, performance considerations, and acceptance criteria. Use concrete EffectCraft, FigCraft, PhotoCraft, and FilmCraft examples. Other apps must retain their own semantics.

Describe the minimum end-to-end pilot and the order needed to validate shared requirements. Reconcile the PhotoCraft handoff and EffectCraft/FigCraft pilots rather than silently changing either. Separate immediate package/host needs from optional roadmap features.

Conclude with a short ordered list of decisions for the user, explaining consequences before asking for thoughts. Record design assumptions explicitly and keep recommendation separate from agreement.

## Constraints and evidence

Themes only, not organization/team management. Existing craft-extensions is the baseline. Third-party distribution is in scope, but its release gate remains closed. Existing schema 1 cannot silently acquire capabilities or arbitrary contribution bundles. Fresh CLI means automatic eligibility for installed/enabled/compatible/permitted modules, not downloads or new grants.

Follow repository AGENTS.md and relevant ArtCraft design/writing skills. No app code, shared runtime implementation, dependency additions, CI edits, app worktree creation, builds, benchmarks, external comments, pushes, PRs, releases, or secrets. Source inspection and docs checks are authorized. Do not claim security certification from source reading.

Use bounded source reads at recorded commits, reuse existing evidence, and verify novel technical claims against primary sources. Avoid repeated broad surveys. Keep implementation proposals small enough to assess and expand only where actual application requirements justify them.

The parent has finished organizing the baseline docs before launching this thread. The new thread owns `docs/plugins/research/` and should not overwrite existing baseline documents during its initial pass.

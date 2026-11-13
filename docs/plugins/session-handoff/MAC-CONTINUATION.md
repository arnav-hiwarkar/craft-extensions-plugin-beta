# Craft plugin beta: Mac continuation

Updated 2026-11-13. This is a source-completion handoff, **not release readiness**. All recovered source is committed for transfer, including explicitly unverified work. Read this first, then inspect the actual code before trusting older claims.

## User direction and limits (highest priority)

Finish the open-source plugin management and usage system across EffectCraft, PhotoCraft, FigCraft and the launcher, with one shared platform, useful plugins, and matching technical specifications and workflows. Make routine architecture/product decisions autonomously, record the rationale, and implement the smallest coherent solution. The user authorizes source fixes, documentation, focused commits and private-branch pushes.

**Do not run cargo build, cargo run, release/guest builds, broad test suites, benchmarks, whole-workspace checks, or commands that compile large UI/engine test crates.** Earlier cargo test commands compiled large crates and overloaded the Linux workspace; the user explicitly stopped that approach. Prefer source review, syntax/manifest parsing and diff checks. Only a genuinely cheap, single-crate cargo check or small isolated test is acceptable; use one build job, one build owner, and do not broaden automatically. Coding completion and runtime verification are separate milestones. Builds and comprehensive testing are for a later user-authorized phase. GitHub Actions and releases remain OFF until the user explicitly approves them after review. Do not infer build approval from the word “build” in the feature request.

Conserve tokens. Main Mac completion thread: Codex GPT-6.1 Sol HIGH, per the 2026-11-13 user correction. Helpers mostly Sol medium or Luna; Terra occasionally. This supersedes the earlier main-thread medium guidance. No Claude. The user permits a little OpenCode DeepSeek 4.1 Flash Free, but the exact model was NOT advertised by the Linux provider; verify availability on Mac, report failure and do not silently substitute. Do not waste free-model quota polling or running ordinary commands. Zero to two narrowly scoped children normally; no nested swarms. Give each owner exact files, invariants, allowed checks and a <=300-word evidence report. One integration writer. Independent coding work needs separate ownership/checkouts; avoid cross-app races and shared Cargo contention.

## Workspace and publication

Use a new sibling workspace `/Users/ash/Projects/plugin-beta/`, preserving existing `/Users/ash/Projects/{effectcraft,craft-extensions,...}` projects. Six repositories:

| Directory | GitHub private repository | Branch |
|---|---|---|
| craft-extensions | arnav-hiwarkar/craft-extensions-plugin-beta | plugin-beta |
| effectcraft | arnav-hiwarkar/effectcraft-plugin-beta | plugin-beta |
| photocraft | arnav-hiwarkar/photocraft-plugin-beta | plugin-beta |
| figcraft | arnav-hiwarkar/figcraft-plugin-beta | plugin-beta |
| craft-launcher | arnav-hiwarkar/craft-launcher-plugin-beta | plugin-beta |
| craft-plugins | arnav-hiwarkar/craft-plugins | main |

Apps intentionally use sibling `../craft-extensions` dependencies. Preserve this layout during beta; do not switch to unrelated upstream HEADs. Inspect remotes before writing. Push only these private branches to `github` and `forgejo` (Linux URLs `forgejo:ash/<private-name>.git`); NEVER upstream/storytold or a similarly named public fork. Clones may initially call the private remote `origin`; normalize safely after checking its URL. No force pushes, history rewrites, resets, cleans, PRs, issues or releases. Commit identity: Arnav Hiwarkar <arnav-hiwarkar@users.noreply.github.com>; truthful Codex coauthor. Stage explicit reviewed paths. Do not commit artifacts, caches, personal data or secrets. The signing key remains on Linux at ~/.config/craft/catalogue.key; never print, commit or transfer it without explicit authorization. Do not run the old workspace setup.sh: it removes CI and rewrites remote configuration.

## Read efficiently

1. `../beta-design.md` is authoritative over older architecture documents; `../draft/decisions.md` records superseded decisions.
2. `../ui.md`, `../author-guide.md`, `../adapter-guide.md`, `../user-guide.md`, `../../THREAT-MODEL.md`, generated `../contracts/*.md` (PhotoCraft page is stale).
3. Each app's `docs/plugin-migration.md`, `docs/plugin-testing.md`, and `docs/plugin-takeover-validation.md`.
4. `original-brief.md` preserves the full original request. Its older model preferences and heavyweight later-session activity do NOT override the current limits above.
5. Historical root STATUS/TESTING snapshots are here for continuity; they include stale claims. Check the timestamp and actual command evidence. See `transfer-revisions.json` for transfer heads (craft-extensions is recorded at its pre-handoff source head to avoid a self-referential commit hash).

## Architecture to preserve

- Shared ext-core types/manifest/contract/permissions; ext-store is the canonical catalogue/store/manager facade; ext-host executes wasmi guests and out-of-process native helpers; ext-ui renders manager and plugin UI; ext-cli and craft-guest provide author/consumer tools. One adapter per app wraps the real engine command/render/history paths. Do not reintroduce the retired legacy plugin systems.
- Sandbox APIs check the calling contribution's declared AND live granted permissions on every call and recheck generation/cancellation at commit. Capture document/selection context on the host, reject stale scopes and replies, bound input/output/recursion/allocations, and fail closed on unreadable grant state. No arbitrary command dispatcher, raw paths, shell, process or management API for Wasm.
- Native is explicitly full-computer access, separately consented, out of process, supervised and killed after consent withdrawal. It is NOT an OS security sandbox; document undo cannot reverse external native effects. The example's committed executable mapping is Linux-only until a real Mac binary is built in the later authorized phase.
- Plugin panels/forms/canvases are versioned data drawn by one shared host renderer. UI 1.1 extends 1.0. Plugins cannot choose or detect the rendering backend. Future craft-framework integration needs a host adapter/fallback, not plugin rebuilds or a second incompatible schema. Named components keep fallbacks. Keep one craft-theme format; theme/font activation remains deferred.
- Each event is transactional for document/history/UI; tools form one undo step across begin/update/commit, Escape/failure/revocation/document replacement rolls back. Pure notifications must not create edit-feedback loops. Idle UI should wake on queued work, not continuously repaint.
- Catalogue is signed metadata, downloads stay on authors' GitHub Releases and are digest-pinned. Semver compatibility is by contracts, not app version. Installation never auto-enables or grants. Pins, rollback, dependencies, kill list, vendor tiers, OSI/public-source listing checks and one-maintainer allow flow must remain consistent across app/launcher/CLI. No review bot for beta.
- Projects preserve plugin references and missing data. Opening a shared project never auto-downloads/enables; the user chooses with permissions shown. Missing render contributions can pass through in preview but final export must refuse clearly.

## Actual state at transfer

- Shared UI 1.1 and expanded EffectCraft/FigCraft contracts are committed from prior work. EffectCraft's recovered lifecycle/path fixes passed 22 adapter tests and one viewer phase regression before the stop instruction. Commit 672a4343 holds those fixes. The subsequent fail-closed deep-redaction change has a regression added but no confirmed run.
- FigCraft's recovered wake-up/content-notification/comment-author fixes and narrowed large-ID validation are preserved. An initial suite had 25 passes, 1 failure, 2 ignored; the wake test bypassed its bridge. The fixture was corrected, then an AppContract clone error was corrected. There is NO confirmed final passing rerun. Review fixture event ordering and identifier field validation. No raster.filter support exists: do not advertise an all-three-app effect falsely.
- PhotoCraft's large recovered expansion adds shared renderer, tools/overlays/Resume, session 1.1 operations, opaque picked files, CLI plumbing and example regression source. Review repaired save-name validation, no-follow writes, bounded composite rectangle reads, tiled histogram and tool file-operation refusal. File publication is ordered, atomic per file, NOT per batch; on failure stop, report any already-published files, roll document/UI back. Earlier files can remain and are not Undo-able. Acknowledge Finish after finalization. **Residual priority:** recheck live grants/cancellation after synchronous publication, before document commit; reason about timeouts and external partial effects. No directory-capability guarantee against hostile parent-directory replacement is claimed. Latest engine compilation/results were lost during tool outage: not a pass. The shared PhotoCraft contract Markdown still needs source-level synchronization; do not cargo run a generator.
- Examples include existing halftone/film-looks/color-notes/figcraft-shapes, EffectCraft cue board/point tool/channel tint, FigCraft grid/placeholder, new PhotoCraft gradient/levels/export, cross-review, cross-vignette, and native-system-stamp. Some new sources/packages have not executed. Native mapping was corrected to stop shipping a fictitious Mac executable. Cross-app smoke source has stronger undo/permission/consent assertions but has not been rerun. It launches prohibited build/test commands: DO NOT execute it now.
- Launcher and catalogue were unchanged by the latest recovery. Older CLI end-to-end evidence used a local signed file:// catalogue and older binaries, not real GitHub releases. No new app/desktop/platform validation is implied by preserved binaries.

## Completion slices

1. Inventory all six heads/dirty state and relevant rules. Make a short requirement-to-code/check ledger, classifying implemented, source defect, deferred-by-design, runtime unverified. No restart or wholesale rewrite.
2. Close confirmed PhotoCraft commit/cancellation gaps at the owning layer; scrutinize stale picker replies, grant refresh, error reporting, budgets and history. Review FigCraft wake-up/feedback/ID fixes and EffectCraft path redaction/tool lifetime. Add small meaningful regression SOURCE; run only allowed cheap checks.
3. Audit shared manager/store/host and three adapters across install/setup/update/revoke/pause/resume/safe-mode/project/export/undo workflows. Check parity, contract versions and same canonical semantics. Fix proven defects without speculative abstractions. Named EffectCraft gaps B5-WINDOW-LIMIT (silent excess windows), B4-DISPOSE, B4-WORKER-ORDER and B9-FEATURE-STATE require explicit triage in plugin-migration.md; do not silently label them done. FigCraft conservative selection grants and raster support are also explicit gaps; choose scope honestly.
4. Finish useful examples, SDK author guidance, app capability specs and generated-page source equivalents without running the app. Cover plugin maker, end user, app adapter author and store maintainer workflows. Explain refusal/failure/rollback and native limitations plainly.
5. One bounded independent read-only review of the frozen candidate. Fix concrete findings centrally. Each logical source/doc slice gets an explicit-path commit and private-branch push. Record checks actually executed vs deferred; no “tests pass” from compilation, ignored tests or historical reports.
6. Deliver a source-complete status, remaining runtime validation matrix, exact revisions and a later Mac/CI verification plan. User approval is still required before any release CI/publication. Do not ask to deploy until the reviewable source work is done.

Explicit beta deferrals: TUF, cross-process leases, OS confinement for native helpers, native effects/pixel transport, temporal frames/tiling, plugin object types, cross-plugin calls/workflows, account permission and web integration. Do not expand scope just to claim a “complete” universal API. Creative reach grows through permissioned canonical app features with clear unsupported reasons, not bypasses.

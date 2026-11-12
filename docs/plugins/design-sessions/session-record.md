# Architecture discussion position

Prepared 2026-11-08. This is the navigation/decision trail for the separate Opus discussion. It links the authoritative choices instead of copying them into a competing specification.

## Current position

All eighteen topics settled (2026-11-10). Consolidated in [the Craft plugin architecture](../architecture.md). The eighteen-topic agenda has been prepared, but the user has not yet held those guided sessions or approved all draft mechanisms. The user wants deep discussion one point at a time, with maintainable simple structures, full actor workflows and realistic combinations covered before implementation.

Accepted direction is in [architecture decisions](../draft/decisions.md) and [functional decisions](../../plugin-functional-decisions.md). Source findings and corrections are in [readiness assessment](../readiness/assessment-and-first-work.md). Proposed details remain proposals even when both prior researchers recommended them.

The user authorized creating a separate top-level Opus 5.5 medium thread on this existing worktree. That is conversation setup, not permission to create app branches, run implementation, publish or reopen the community gate. Earlier research-agent budgets do not automatically authorize agents in the new discussion.

## How to record each explicit choice

Update the existing decision record with the choice, user's reason if given, what it supersedes, affected constraints and implementation authorization status. Then append a concise trail entry here with topic ID, decision-record link, unresolved dependencies, evidence that changed the discussion, and next question. Avoid a full transcript or another architecture copy.

If a choice changes a previous agreement, state what changed and why; update the affected draft and scenario expectations. Do not mark a topic accepted from silence, a source finding, model agreement or the fact that a document was written. A provisional decision must say what evidence or dependent choice it awaits.

## Topic progress

| Topics | Current discussion status | Next action |
|---|---|---|
| T01 | Settled 2026-11-08: grant unit chosen (B). Vocabulary and protected-core boundaries explained | Revisit only if later topics change it |
| T02 | Settled 2026-11-08: two-axis contract structure accepted | Revisit only if later topics change it |
| T03 | Settled: placement A chosen by the user 2026-11-09; other selections delegated 2026-11-08 | Revisit only if later topics change it |
| T04 | Settled 2026-11-09: Q1 A (automation authority), Q2 system accepted | Revisit only if later topics change it |
| T05 | Settled 2026-11-09: families with scopes, open-source store (MIT/Apache), local installs labelled, optional rights at first use, CLI grant command | Revisit only if later topics change it |
| T06 | Settled 2026-11-09: wasmi only with engine-neutral host; native plugins allowed in the store after review (with native safeguards); failed plugin fails the export | Revisit only if later topics change it |
| T07 | Settled 2026-11-09: reverse-domain IDs with store-verified namespace, new-owner notice before update, contract-version compatibility | Revisit only if later topics change it |
| T08 | Settled 2026-11-09: updates picked up at safe points, only the current release kept (user may choose any listed version), always ask before updating | Revisit only if later topics change it |
| T09 | Settled 2026-11-09: bot auto-merges routine updates, quick human look for risky cases; catalogue repository planned for maintainers to create later | Revisit only if later topics change it |
| T10 | Settled 2026-11-09: TUF through an existing Rust client; 48-hour hold for auto-merged updates | Revisit only if later topics change it |
| T11 | Settled 2026-11-10: shared manager component; official first within a relevance tier; offline store suggestions from built-in search | Revisit only if later topics change it |
| T12 | Settled 2026-11-10: host-drawn declarative UI with custom canvas; suggested shortcuts only when free; menu items in declared spots | Revisit only if later topics change it |
| T13 | Settled 2026-11-10: newer compatible version used with a note; bake offered; projects may optionally carry plugin files, off by default | Revisit only if later topics change it |
| T14 | Settled 2026-11-10: workflows run with their own declared permissions; large files offered at setup or on first use; themes limited to colours, fonts and density for now (themes to be revisited) | Revisit themes later at the user's request |
| T15 | Settled 2026-11-10: user-only local socket or pipe with token; controlling banner; attached jobs survive disconnect | Revisit only if later topics change it |
| T16 | Settled 2026-11-10: per-plugin performance shown; automatic safe-start offer plus manual entry and bisect | Work through UI, project/output, resources, automation, workload/offline/recovery cases |
| T17 | Settled 2026-11-10: 12-month contract overlap; listing rules; appeals to a different maintainer within 14 days | Revisit only if later topics change it |
| T18 | Settled 2026-11-10: EffectCraft, FigCraft, PhotoCraft pilot order; old systems removed with each new adapter; labelled local installs before the catalogue | Implementation needs separate authorization |

The user can reorder topics or revisit one at any time. Cross-topic unresolved questions should be named, not silently decided for convenience.

## Initial unresolved policy/structure list

- Exact catalogue/local/official-proprietary licence policy and repository/operator/key custody.
- Confirmation of per-executable-release review, reviewer staffing and urgent response procedure.
- Exact ranking precedence within relevance tiers and automatic-update default/preferences.
- Optional noninteractive explicit CLI grants and restricted GitHub credentials.
- Final package/state fields, precise public contract boundaries and generated SDK/host module organization.
- Native/GPU/audio/backend confinement and measured workload profiles before availability claims.
- Supported-version/deprecation commitment and app-specific migration evidence.

Do not reopen settled requirements such as common installation with app-specific permissions, publisher GitHub assets, human catalogue review, public/local usability, sandbox-first, additive core protection or the closed gate unless the user changes them.

## Evidence discipline

Sixteen app local committed snapshots were source-audited and match the recorded inventory; latest upstream was not fetched. No runtime test, app launch, benchmark, build or security certification was performed by these documentation phases. Reproduce source-derived concerns in later scoped implementation; do not describe them as observed exploits or tested output defects.

Current important corrections: existing caller/file/plugin checks are real; same-plugin replacement is not proof of builtin takeover; PhotoCraft can remove registry entries; GridCraft's test helper has a cross-crate integration-test consumer; exporter extension overlap can be valid under explicit selection. Read root qualifications before turning any older raw finding into a requirement.

## Trail

### T01, Q1 (open, 2026-11-08)

Example: one package, *Ink & Grain*, provides an EffectCraft halftone layer effect (saved live in projects) and a FigCraft command that turns a selection into native halftone vector nodes (no saved plugin reference). Explained installed release, contribution, app setup, saved feature and runtime instance, plus additive namespacing and builtin protection.

Question asked: should granted rights attach to each contribution, or to the package within one app, with per-contribution enablement controlling only what appears? Draft recommendation: grants per (app, package release lineage); enablement per contribution; runtime instance is the enforcement boundary. Reason: contributions sharing one module can share memory, so per-contribution grants would not be enforceable without separate instances.

**Answered 2026-11-08: user chose B.** Recorded in [design-session decisions](../draft/decisions.md#design-session-decisions). No further T01 question pending.

### T02, Q1 (open, 2026-11-08)

Proposal: data-plane families (raster, temporal, audio, resources) get shared family contracts with core plus negotiated extensions, so one compiled module can serve several apps; document-plane families share only mechanics with app-generated operations. Grounded in source facts: EffectCraft WASM v1 premultiplied RGBA f32 with time/scale only; PhotoCraft bands with straight-alpha flag, document colour model and overlap capped at 256 px. Established practice cited: OpenFX and CLAP.

Question asked: accept shared family contracts for the data plane only (one module, many apps), or require per-app modules everywhere with sharing only at source level? Not yet accepted.

### T02, revision (2026-11-08)

The user did not answer Q1 and instead asked for a deeper end-to-end review from every actor's perspective. The review found the first "two planes" split wrong for VectorCraft object filters/live effects (no host imports, host-applied output) and GridCraft functions (pure typed values). Revised model: two axes, domain and interaction style (transform, session, data); three layers (common mechanics, shared domain contracts, app contracts); effective authority = style imports ∩ package grant. Added plugin obligations (determinism, tile invariance), cross-app consistency promise, governance and exclusions (SoundCraft native hosts, CADCraft AutoLISP).

Q2 asked: accept that a transform module written against a shared domain contract runs unchanged in every app that passes that contract's conformance suite, with session contributions always per app? Q1 is superseded by Q2. Not yet accepted.

**Answered 2026-11-08: user accepted Q2.** Recorded in [design-session decisions](../draft/decisions.md#design-session-decisions).

### T03, Q1 (open, 2026-11-08)

User asked for the same end-to-end review for T03 with proven, maintainable, scalable practices. Proposal: all platform crates in craft-extensions; app registry export with breaking-change CI; generated committed guest crates; contributions and parameters declared in code and carried in a custom section; one `Craft.toml` for identity and per-app permissions; three test levels; isolated dev store; deterministic releases with previous-release comparison. Evidence: no SDK exists; craft-extensions has core/store only; PhotoCraft `pc_manifest` shows module-carried metadata.

Question asked: declare contributions and parameters in code (macros, custom section) with only identity and permissions in `Craft.toml`, or declare everything in a hand-written manifest file that the SDK checks against the code? Not yet accepted.

**Answered 2026-11-08: user delegated the T03 choices** ("decide those things as per what's best"). Designer selections recorded in [design-session decisions](../draft/decisions.md#design-session-decisions); publication to crates.io still needs authorization when it happens.

### T04, Q1 (open, 2026-11-08)

Proposal: extend each app's registry with typed args, kind, opt-in plugin exposure, resolved-argument capability function and interaction; one dispatcher whose signature requires an `Invocation` (principal, origin chain, captured scope, cancel, transaction); one authorization function; handles and artifact roles instead of paths; dispatcher-owned transactions with origin-labelled undo; short commands versus revision-checked jobs; JSON control plane with bulk data through T02 handles; generated CLI/MCP/docs. Evidence: no caller at dispatch in audited apps; FigCraft control channel bypasses plugin policy; uniform snapshot/rollback in domain apps.

Question asked: what authority should an authenticated automation client (MCP agent, attached CLI, control channel) get by default: the user's full document authority but no extension-management operations unless the user grants a management scope interactively (recommended), or a narrower declared scope like a plugin? Not yet accepted.

### Side task (2026-11-09)

At the user's request (a maintainer offered to block plugin-related merges until the basic architecture lands), a separate top-level thread was started on this worktree to write `docs/plugins/merge-hold-list.md`. T04 Q1 remains open here.

### T03 reopened (2026-11-09)

The user asked to be asked the T03 questions directly instead of delegating. The delegated selections remain the working proposal. Q1 asked: where the plugin-platform crates live.

**Answered 2026-11-09: user chose A** (all platform crates in craft-extensions) and asked to move on to a deeper T04 review. The other T03 selections stand as delegated.

### T04, deeper review (2026-11-09)

User asked for the same all-perspective review of T04. Checked EffectCraft `ce9eaec` and FigCraft `0e86f359`. New findings: authority-changing settings hide inside generic commands (EffectCraft `prefs.set` can change Scripting permissions; `confine.rs` special-cases it); UI enablement (`spec.enabled`) is used as a gate and bypassed by explicit targets; commands are journaled and fed to habit learning (`learn_observe`) with no origin; batches chain results (`$N`). Revision recorded in the [family map](app-families-and-sdk.md#t04-revision-after-the-all-perspective-review). Q1 re-asked with the evidence: default authority of authenticated automation clients.

User asked how plugin safety holds if automation gets option A. Answer given: Q1 governs automation clients only; plugins stay under their own layered limits (sandbox, style imports ∩ grant, plugin exposure, resolved-argument checks, captured scope and handles, transactions, budgets, no authority transfer). Added link: plugins can never reach an automation channel (no socket import; any future network grant excludes loopback; channel tokens never given to plugins), so A cannot be used by a plugin to gain user authority. Q1 still open.

**Answered 2026-11-09: user chose A** "as long as CLI has full access and plugins are safe". Recorded meaning: a CLI command the user runs directly in a fresh process is the user, with full authority including management (`<app>-cli plugin install`); a shell-capable agent could do the same through the filesystem, so restricting it adds no security. Clients attached to a running app (MCP, control channel, attached CLI) get the user's full document authority; management operations need an in-app management scope the user approves, once per session. Plugins are unaffected and stay under the layered limits, including no route to any automation channel. Q2 asked: accept the T04 system as a whole.

**Answered 2026-11-09: user accepted the T04 system as a whole** (proposal plus the 2026-11-09 revision).

### T05, Q1 (open, 2026-11-09)

User asked for a deep, scalable study of permissions and consent. Proposal recorded in the [family map](app-families-and-sdk.md#t05-session-proposal-permissions-and-consent): shared permission families with scopes, required/optional, picking-is-permission, host-written sentences and combination warnings, consent moments, generation-checked grants in the store, CLI grant commands separate from execution, migration of FigCraft/EffectCraft/PhotoCraft vocabularies. Q1 asked: permission granularity (shared families with scopes, per-operation lists, or coarse levels).

**Direction change, 2026-11-09 (user):** plugins may use the CLI and its tools; do not restrict the community; plugins cannot change main features; plugins free and open source; store plugins reviewed safe; MIT and Apache licences. Recorded in [decisions](../draft/decisions.md) and the [family map](app-families-and-sdk.md#open-ecosystem-direction-2026-11-09-and-its-effect-on-t01t05). T04 exposure flipped to default-on except management, protected core and internal operations. T05 families kept. Questions asked: confirm in-process CLI access rather than launching processes and families (T05 Q1); licences of bundled dependencies and assets; whether non-store local installs must also be open source.

**Answered 2026-11-09: "yup allowed, next".** Recorded as accepting all three recommendations: Q1 yes (in-process CLI operations, shared permission families with scopes); Q2 A (bundled dependencies and assets may use the house licence allowlist, plugin itself MIT and/or Apache-2.0); Q3 A (file installs outside the store allowed with any licence, same sandbox and permissions, labelled "Not reviewed"). Q2 was not named explicitly; read as accepting the recommendation. Next: T05 Q2 (when optional permissions are asked) and Q3 (CLI granting without the GUI).

**Answered 2026-11-09: T05 Q4 A** (optional permissions asked at first use, from a user-started GUI action, host-drawn) **and Q5 A** (a CLI run directly by the user may grant through a separate `plugin grant` command; never as a flag on execution). T05 settled.

### T06, Q1–Q3 (open, 2026-11-09)

Proposal in the [family map](app-families-and-sdk.md#t06-session-proposal-runtime-and-safety). Evidence: all three hosts use wasmi 2 with fuel and StoreLimits but different settings; PhotoCraft has deadline-sliced fuel, total memory and strict limits; EffectCraft pools instances across calls. Questions: Q1 engine (wasmi only now, engine-neutral host, JIT later only if measured too slow); Q2 native plugins and helpers; Q3 export when a plugin fails.

**Answered 2026-11-09:** Q1 A (wasmi only; engine-neutral host; wasmtime on desktop only if measurements show effects too slow). Q2 **C** ("we wanna let the users build whatever plugins"): native plugins may enter the store after review. Q3 A (a plugin failure fails the final render or export, naming plugin and frame). The designer told the user that native code cannot be technically prevented from changing built-in features or acting maliciously, and recorded safeguards that keep C as safe as practical: out-of-process only, store builds native binaries from source, stricter review, distinct full-access label and consent, confirmed updates, OS sandbox where proven.

**Correction, 2026-11-09 (user):** the store only lists plugins: fetched from authors' GitHub Releases, no building, no hosting, no responsibility; light conformance review and test; no restrictions on what plugins do. Recorded; superseded the store-build, full-review and native-confinement safeguards. Kept: pinned asset digest, bot conformance checks, labels, delisting/revocation. Asked: does "it can do anything" also remove the WebAssembly sandbox and permissions (recommended: keep them).

**Answered 2026-11-09: A.** WebAssembly plugins keep the T05/T06 sandbox and permissions; native plugins are the full-access route. T06 and the store-role correction settled. Next: T07.

### T07, Q1–Q3 (open, 2026-11-09)

Proposal in the [family map](app-families-and-sdk.md#t07-session-proposal-package-identity-and-compatibility). Evidence: every app and craft-extensions use reverse-domain IDs, no UUIDs; EffectCraft projects save only a bare effect id. Questions: Q1 identity (readable reverse-domain ID with store-verified namespace, as Flathub, instead of UUID); Q2 owner change (tell the user before the next update); Q3 compatibility basis (contract versions, app version optional).

**Answered 2026-11-09: "yes"** to all three recommendations: Q1 A (reverse-domain IDs, namespace verified by the store as Flathub does; no UUID), Q2 A ("has a new owner" notice and confirmation before the next update), Q3 A (contract versions decide compatibility; app version optional). T07 settled.

### T08, Q1–Q3 (open, 2026-11-09)

Proposal in the [family map](app-families-and-sdk.md#t08-session-proposal-store-and-lifecycle). Evidence: craft-extensions store already has immutable versions, staging, short lock, atomic state and generation; gaps are enable-on-install, immediate deletion of the old release, no digest/source/grants, one schema number. Questions: Q1 when running apps pick up an update; Q2 how many old releases to keep; Q3 default update behaviour.

**Answered 2026-11-09:** Q1 A (safe points; running exports and jobs finish on the leased old release). Q2 B with an addition: keep only the current release (plus any release still leased), and let the user choose any listed version to download, including an older one; this replaces the store's blanket downgrade refusal with an explicit user-chosen version change. Q3 B: always ask before every update. T08 settled.

### T09, Q1–Q2 (open, 2026-11-09)

Proposal in the [family map](app-families-and-sdk.md#t09-session-proposal-github-distribution-and-catalogue). Questions: Q1 merge rule for updates (bot-only auto-merge for routine updates, quick human look for first listing, more permissions, new owner and native); Q2 catalogue repository creation and name (needs maintainer authorization).

**Answered 2026-11-09:** Q1 A (routine updates that pass the bot, add no permissions, keep the owner and are not native are merged by the bot; first listing, more permissions, new owner and native get a quick human look). Q2: the catalogue repository stays planned; name, owner and maintainers to be decided by maintainers later. T09 settled.

### T10, Q1–Q2 (open, 2026-11-09)

Proposal in the [family map](app-families-and-sdk.md#t10-session-proposal-authenticity-and-incident-operations). Questions: Q1 signing (an established update framework, TUF, through an existing Rust client, versus one embedded signing key); Q2 hold period before auto-merged updates reach users.

**Answered 2026-11-09: recommendations accepted.** Q1 A (TUF through an existing Rust client; library licence and maintenance to verify). Q2 A (auto-merged updates wait 48 hours before reaching users). T10 settled.

### T11, Q1–Q2 (open, 2026-11-09)

Proposal in the [family map](app-families-and-sdk.md#t11-session-proposal-search-and-management-ux). Questions: Q1 how official plugins rank (first within a relevance tier); Q2 offline store suggestions when a built-in search finds nothing.

**Answered 2026-11-10:** Q1 A (official first within each relevance tier) and Q2 A (offline store suggestions when a built-in search finds nothing, with a setting to turn them off). T11 settled.

### T12, Q1–Q3 (open, 2026-11-10)

Proposal in the [family map](app-families-and-sdk.md#t12-session-proposal-plugin-ui-and-interactions). Evidence: house rule forbids webviews in native apps; EffectCraft parameters and FigCraft panels and widgets are already declarative. Questions: Q1 UI model (host-drawn declarative with a custom canvas element); Q2 plugin default shortcuts; Q3 where plugin menu items may go.

**Direction, 2026-11-10 (user):** "the current plugin stuff which the apps have, ignore it, we'll remove them completely and override them with our work." Existing app plugin systems (EffectCraft WASM ABI and helpers, FigCraft APIs 1-4, PhotoCraft filter ABI, EffectCraft Plug-in Library) are not compatibility constraints: no legacy app profiles, no legacy ID mapping, no migration of old plugin manifests or permissions. Their code remains evidence of lessons only. Old projects that reference removed plugins open with placeholders that keep their data.

**Answered 2026-11-10:** T12 Q1 A (host-drawn declarative UI plus a custom canvas element taking draw lists or an image), Q2 A (suggested default shortcut used only when free; never takes a built-in or user binding), Q3 A (menu items in declared extension points of the real menus, after built-ins, grouped by plugin). T12 settled.

### T13, Q1–Q3 (open, 2026-11-10)

Proposal in the [family map](app-families-and-sdk.md#t13-session-proposal-live-projects-and-reliable-output). Lesson: EffectCraft's render pipeline silently drops missing effects. Questions: Q1 newer compatible version on reopen (use with a note); Q2 bake; Q3 embedding plugin files in projects (no).

**Answered 2026-11-10:** Q1 A (use the newer compatible version with a quiet note), Q2 A (bake, keeping the original off for un-baking), Q3 **B with default off**: projects may carry plugin files through an explicit "include plugins" option when saving or collecting. Recorded safeguards: carried plugins are never installed or run on open; the user chooses "Install from project", which is an ordinary file install through the store's validation; a digest matching a store listing is treated as that listing, otherwise labelled "Not reviewed"; revocations apply by digest; setup and permissions as usual. T13 settled.

### T14, Q1–Q3 (open, 2026-11-10)

Proposal in the [family map](app-families-and-sdk.md#t14-session-proposal-typed-resources-themes-and-workflows). Questions: Q1 workflow authority (its own declared permissions); Q2 when large resources download (first use, size shown); Q3 theme scope (colours, fonts, density, versus also icons and images).

**Answered 2026-11-10:** Q1 A (workflows run as their package with declared permissions). Q2 A plus an offer at setup, designer to choose the best UX: the setup sheet shows the size with **Download now** (primary) and **When first needed**; first use still downloads with the size shown if not yet present; the CLI takes `--with-resources`. Q3 A for now (colours, fonts, density); the user will revisit themes later. T14 settled.

### T15, Q1–Q3 (open, 2026-11-10)

Proposal in the [family map](app-families-and-sdk.md#t15-session-proposal-automation-and-jobs). Evidence: EffectCraft and FigCraft attach over unauthenticated localhost TCP; EffectCraft MCP long renders already use progress and cancel notifications. Questions: Q1 attach transport (user-only local sockets or pipes with a token); Q2 "is controlling" indicator; Q3 attached job when the client disconnects.

**Answered 2026-11-10:** Q1 A (Unix domain socket in a user-only directory or user-restricted named pipe, plus a per-session token; no network port by default), Q2 A ("<client> is controlling <app> · Stop" while any external client is connected), Q3 A (an attached job continues after the client disconnects and can be rejoined by ID). T15 settled.

### T16, Q1–Q2 (open, 2026-11-10)

Proposal in the [family map](app-families-and-sdk.md#t16-session-proposal-performance-offline-behaviour-and-diagnostics). Questions: Q1 per-plugin performance shown in the manager and an optional viewer overlay; Q2 safe start offered automatically after a failed start, plus manual entry and bisect.

**Answered 2026-11-10:** Q1 A (per-plugin cost and memory in the manager plus an optional viewer overlay) and Q2 A (safe start offered automatically after an unfinished start, plus `--safe-mode`, a Help menu entry and "Find the problem plugin"). T16 settled.

### T17, Q1–Q3 (open, 2026-11-10)

Proposal in the [family map](app-families-and-sdk.md#t17-session-proposal-ecosystem-policy-and-long-term-maintenance). Remaining-decisions table annotated as resolved by the sessions. Questions: Q1 how long an old contract major stays supported; Q2 listing rules; Q3 appeals.

**Answered 2026-11-10:** Q1 A (an old contract major stays supported for 12 months after its successor ships), Q2 A (listing refused or removed only for malicious or deceptive behaviour, impersonation, illegal content, failed checks or unverified namespace; never for quality, competition with built-ins or being native), Q3 A (appeal decided by a different maintainer, normally within 14 days; emergency blocks stand meanwhile). T17 settled.

### T18, Q1–Q3 (open, 2026-11-10)

Proposal in the [family map](app-families-and-sdk.md#t18-session-proposal-delivery-and-evidence). Questions: Q1 pilot order (EffectCraft, FigCraft, PhotoCraft); Q2 old systems removed in the same change set as each app's new adapter; Q3 labelled local-file installs for pilots until the catalogue exists.

**Answered 2026-11-10:** Q1 A (pilots EffectCraft, then FigCraft, then PhotoCraft), Q2 A (each app's old plugin system is removed in the same change set that lands its new adapter), Q3 A (pilots use labelled "Not reviewed" local file installs until the catalogue exists). T18 settled; all topics settled. Nothing here authorizes implementation.

**Direction, 2026-11-10 (user):** management inside every app so the launcher is never required, in launcher and standalone installs and builds. Recorded with the adapter definition (one per app, not per plugin), shared store folder, Flatpak and web handling.

### Consolidation (2026-11-10)

At the user's request, wrote [the Craft plugin architecture](../architecture.md) as the single current description, added a status note to every older document pointing to it, rewrote the plugins index, and updated the merge-hold lifting table to the delivery units. Raised one open item in the architecture: whether replacing existing plugin systems covers SoundCraft's CLAP/VST3/AU hosting and CADCraft AutoLISP (recommended: keep them under the native full-access rules).

### Gap check (2026-11-10)

At the user's request, compared the architecture with the aspect register, the forty scenarios (S01–S40), the readiness blockers (B1–B11) and Jeremy's handoff. Not yet discussed and needing user choices: package dependencies and plugin-to-plugin APIs (S40); plugin-defined document object types (new layers, nodes, objects); user scripting and expressions versus plugins; the account sign-in broker; moving setups between machines; communication and porting for authors of today's plugins; legal text and takedowns for the listing-only store; an independent security review before launch. Discussed but missing from the architecture text: default handler resolution for duplicate formats (S04), development versus release builds (S23), clock rollback (S17), private-storage limits, network broker rules, mapping of existing craft-extensions tiers to labels, EffectCraft's restart-only unregister (B11) as adapter work. Reported to the user; nothing changed in the architecture yet.

**Gap-check answers, 2026-11-10** (asked with options and recommendations): hosting on GitHub plus our domain (recommended); new object types host-managed (recommended); data dependencies now, code later (recommended); scripts separate now (recommended); no sign-ins at first (not the recommendation); export setup with permissions re-asked (recommended); old plugins just removed (not the recommendation); internal security review only (not the recommendation); short disclaimer only (not the recommendation); SoundCraft VST/CLAP/AU and CADCraft AutoLISP kept as full-access (recommended; the user first chose "remove both" without reading the question, then re-answered). Applied to the architecture together with gap items 9–15.

**App adapters, 2026-11-10:** user asked what adapters do; explained and asked four questions. All recommendations chosen: levels with L0 everywhere early; shared traits plus one crate per app; CI fails on unmarked operations; platform team builds the three pilot adapters, then apps own theirs. Architecture section 21 rewritten; P5b added.

**Shared for review, 2026-11-10:** at the user's request the architecture was copied into a Claude Docs document, "Craft Plugin Architecture" (https://claude.ai/artifact/R6tx2t33CZvdTtLBKgtVaM), for the team to comment on. It is private until the user shares it. `architecture.md` stays the source of truth; review comments are to be folded back into it.

**PR opened, 2026-11-10:** at the user's request, after a full read of craft-extensions `main` `a6c5fae` and `wip/extensions-handover` `ffd8512`, opened storytold/craft-extensions#1 from the fork arnav-hiwarkar/craft-extensions (created with the user's approval; the account has no push access). Docs only: `docs/ARCHITECTURE.md` (self-contained copy of the architecture), `docs/DESIGN-DECISIONS.md` (the design-session decision rows), `docs/ARCHITECTURE-GAP-ANALYSIS.md` (14 code changes C1–C14, six conflicts D1–D6 with the repository's threat model and handover decisions) and one README paragraph. No agent attribution, per house rule 15.

**PR review, 2026-11-10:** Jeremy8776 approved storytold/craft-extensions#1 and answered D1–D6: open store accepted as the target with the `third-party` feature off until P7; **outside security review (G12) kept as a gate for public launch (P12)**; **two-maintainer review (G10) kept for native listings**; permission families accepted (G6 becomes declared, host-granted, generation-checked); transform plugins only until P2; native accepted with a human look on every native release, confinement a P10 goal; no store builds, source link and pinned digest required; stars, no telemetry; pilot order EffectCraft, FigCraft, PhotoCraft. Next steps he asked for: a docs PR updating THREAT-MODEL.md and README, then C1, C2, C4. Two non-blocking notes: say in section 2 that native is the exception to "by design, not by review"; owner "Maintainers and bosses" → "Maintainers". Conflicts with the user's earlier choices: outside review (user chose internal only) and two-maintainer native review (user chose a quick human look). Awaiting the user.

**Review applied, 2026-11-10:** the user accepted the outside security review (P12) and two-maintainer native review. Applied to both architecture copies and both decision records; gap analysis section 4 records the answers. Pushed b2ce374 and 5ba16aa (the reviewer's exact wording for the native exception), replied to the review and to both inline suggestion threads. PR #1 head 5ba16aa; approved; CI waiting for maintainer approval of fork workflows; awaiting merge by a maintainer.

**Implementation plan, 2026-11-10:** at the user's request, wrote an implementation plan and, separately, delegated an independent plan to GPT-6.1 Sol (high reasoning) with the same sources. Reconciled both into `implementation-plan.md`; its section 12 lists the differences. Sol's key additions: schema-1 writers cannot honour new version fields (migration gate, P0.4 shipped-writer check), conservative first C2, retained set must include pins and pending consent (design wording to clarify), unique lease instance IDs, P5b early L0 would remove pilot apps' plugins too soon, EffectCraft registry cannot unload (`OnceLock`, leaked metadata), mid-export failure propagation, native helpers gone until P10, CI through craft-common-actions, guest SDK versus the no-`unsafe` rule, runtime rather than `check` enforcement, raster tolerances. Opus additions kept: call-site counts, CI wasm-feature mismatch, issue structure, schema docs. Plan not yet shared.

**Unit implementation plans, 2026-11-10:** at the user's request, wrote one build-ready plan per delivery unit in [`implementation/`](../implementation/README.md) (P1–P12, P5b inside P5, P0 in the index). They deepen the reconciled plan and do not redesign the architecture. Code was read at craft-extensions `a6c5fae`, craft-libs `fd5e2fe`, EffectCraft `origin/main` `fb62b9d7` (no branch switched; the local `main` is 22 commits behind), FigCraft `0e86f359`, PhotoCraft `655295b2` and launcher `origin/main` `753a710`. Nothing was built, run, committed or pushed.

Main findings:
- EffectCraft has **317** non-test `execute` call sites, not 691; the old count included inline test modules. It registers about 760 commands.
- Frozen schema-1 store code already refuses schema-2 state without touching it. The data-loss risk exists only if new fields ride under schema 1, so architecture section 11's "keeps what current apps read in place" needs rewording.
- Uninstall also deletes release files at once.
- The catalogue client should reuse craft-libs `craft-update`'s downloader, not the launcher's copy.
- FigCraft's old plugin API has levels 1–6, not 1–4.
- EffectCraft's CLI and MCP load no plugins today.
- The shared libraries lack the keymap, menu, i18n and AccessKit hooks the manager needs.

Decisions for maintainers are consolidated in the plans' index. The most pressing are:
- the named `unsafe` exception for the guest ABI;
- the deployment boundary for state schema 2;
- the ID cap;
- the placement of the attach transport;
- project schema 3 in `craft-ecproj-format`;
- the principal of a fresh MCP process.

`implementation-plan.md` was updated only to link the unit plans and apply these corrections. The architecture text was not changed.

### Post-call reconciliation started, 2026-11-11

The user supplied two summaries of a team call and requested a new thread to settle the remaining decisions, with the record kept in this repository. [Post-call plugin decisions](post-call-reconciliation.md) preserves the technical notes, distinguishes confirmations from proposed changes, and lists open decisions PC01–PC12. Main delivery direction reported: manual installs, L0 and L1 app adapters first, launcher work later, and theme framework work proceeding separately. Scanning, enforced permissions versus disclosures, physical store layout, catalogue serving, repository structure, and review gates need clarification. Existing outside-launch review and two-maintainer native-release approvals have not been explicitly replaced. Documentation and discussion are authorized. Implementation and external actions are not authorized by this request.

### PC01 first-app discussion opened, 2026-11-11

Read the reconciliation, architecture, accepted design-session decisions, recent session history, and implementation index in the existing worktree at craft-libs `fd5e2feff6ea25bed371a99cc732b3968d621`. Plugin documents remain uncommitted. Explained L0 as package management and data add-ons without plugin code, L1 as bounded transforms over typed input and output, and L2 as session plugins that call app operations.

Recommended EffectCraft first, following the accepted pilot order, with one manually installed data package and one WebAssembly transform. Proposed proving explicit enablement, use, undo, save/reopen, missing-plugin placeholders, and failure handling before catalogue, launcher, native, L2, or theme-framework work. Asked one focused question about EffectCraft with L0 and L1, L0 alone first, or another app with L0 and L1. The user's answer is pending. Example selection and detailed acceptance criteria remain proposals.

The [PC01 proposal and implementation implications](post-call-reconciliation.md#pc01-proposal-presented-2026-11-11) record the dependencies and outstanding boundaries, including P5b rollout, old-system cutover, CLI/automation parity, storage, and runtime authority. No accepted architecture or implementation plan changed. No implementation, builds, commits, pushes, remote checks, or external messages were performed.

### PC01–PC03 answers, 2026-11-11

The user selected the recommended answers to three focused questions:

- PC01: EffectCraft first with L0 and L1, one manually installed data package and one transform, including save/reopen and failure handling. Exact examples and remaining delivery boundaries are open.
- PC02: import and validate a package, then explicitly enable it. Drag-and-drop enters import; loose files in a watched folder are not executed directly.
- PC03: share installed package files, with enablement, settings, and permissions separate per app. Installing in one app never enables another.

The [reconciliation answers](post-call-reconciliation.md#pc01-answer-2026-11-11) preserve the explanations and affected documents. Updated the architecture and decision history, and marked the affected P5/P6 plans for scope reconciliation. Existing safety, parity, cutover, and review requirements remain. The user supplied choices without additional reasons; recorded rationales identify the explanations presented with the recommendations.

Next asked PC06 about keeping host-enforced WebAssembly permissions, distinguishing disclosures from authority and leaving native full access separate. The answer is pending. The meaning of "verified against main" also remains open. No implementation or external action is authorized.

### PC06 enforcement confirmed; PC04 deferred, 2026-11-11

The user chose "Keep enforced permissions (Recommended)." WebAssembly access stays host-enforced, disclosures explain access, and optional permissions remain unavailable until granted. Native full access remains separate. Updated architecture section 8 and the decision history; the unit plans already require this behavior. The phrase "verified against main" remains open.

Asked whether "version 7" meant P7 with P12 still the separate public-launch gate. The user said "leave this point for now." PC04 is deferred, with no new interpretation approved. Existing P7/P12 wording remains the baseline.

Next asked PC05 about catalogue checks on every new or changed release artifact. Explained that listing-text changes alone need no repeat scan of unchanged files, local package validation remains required, and catalogue scanning belongs to catalogue delivery. Human-review triggers are separate. The answer is pending; existing review approvals remain in force. Details are in the [post-call reconciliation](post-call-reconciliation.md#pc06-enforcement-answer-2026-11-11).

### PC05 deferred; PC07 opened, 2026-11-11

The user selected "Leave scanning policy unresolved for now." Recorded PC05 as deferred, preserving existing catalogue checks and local package validation. No replacement policy or exemption was approved; architecture and unit plans remain unchanged by the deferral.

Next asked PC07 whether CodeRabbit assists code review while retaining the outside security review before P12 and two-maintainer approval for every native listing and release. Recommended retaining both requirements. Reviewer identities and scope are not assigned. The answer is pending; see the [reconciliation](post-call-reconciliation.md#pc07-question-presented-2026-11-11).

### PC07 deferred; PC08 serving opened, 2026-11-11

The user selected "Leave reviewer roles unresolved for now." Recorded PC07 as deferred. No replacement review policy or CodeRabbit role is approved; the outside review before P12 and two-maintainer approval of every native listing and release remain the baseline. Architecture and unit plans are unchanged by this deferral.

Next asked PC08 where apps initially fetch signed catalogue metadata. Explained the difference between GitHub records, author-hosted package files, and metadata serving. Recommended GitHub Pages first with a custom domain later, compared with the existing custom-domain primary and Pages backup design. The answer is pending. Publisher authentication remains separately unresolved. See the [reconciliation](post-call-reconciliation.md#pc08-serving-question-presented-2026-11-11).

### PC08 and PC09 answers, 2026-11-11

The user chose GitHub Pages first, custom domain later, and retained maintainer-controlled TUF catalogue signing without separate publisher signing keys. Source/namespace checks and pinned package hashes remain required. Updated the architecture, decision history, P7/P12 plans, plan index, and top-level plan. This supersedes custom-domain primary and Pages backup; a domain is no longer an initial-launch prerequisite. Older publisher-signature G2 wording needs later external-doc reconciliation, not a new signing requirement. No publication or external edit was performed.

The user verified "IGUI" meant egui, then chose using existing egui components for the MVP and adopting shared theme tokens later. The separate theme framework is not an MVP prerequisite. Updated architecture section 14, decision history, P5, and the index. Token schema, ownership, timing, and early AI UI adoption remain open. "Proceed now" was a dependency choice, not implementation authorization. The [reconciliation answers](post-call-reconciliation.md#pc08-answers-2026-11-11) record the reasons and boundaries.

Next asked PC10 whether to retain the existing repository split for the MVP, explore a monorepo before implementation, or leave structure unresolved. Recommended the existing split; explained that a monorepo requires a broader coordinated move. Release-version propagation remains separately open. The answer is pending.

### PC10 repository split confirmed; PC11 delivery opened, 2026-11-11

The user selected retaining the existing repository split: shared platform in craft-extensions, adapters in their app repositories. Updated architecture section 5 and the decision history. Unit plans already follow this split. Release-version propagation remains open; no script or release automation was approved.

Next asked PC11 about small, independently reviewable PRs ending in a verified complete EffectCraft L0/L1 workflow, compared with one larger initial MVP PR or deferring delivery boundaries. Recommended small PRs. Named owners, navigation coordination, exact contents, and estimates remain open. The answer is pending; see the [reconciliation](post-call-reconciliation.md#pc10-repository-answer-2026-11-11).

### PC11 delivery and PC12 disclaimer confirmed, 2026-11-11

The user selected small PRs with complete integration proof for PC11. Updated architecture section 22, decision history, and the top-level plan. Named owners, navigation coordination, exact PR contents, and estimates remain open.

The user selected retaining the short disclaimer for PC12. The manager and catalogue README explain that the project does not guarantee community plugins, alongside existing checks and review requirements. Updated architecture section 12 and decision history. Exact wording and formal legal-review timing remain open.

This completes the first pass through PC01–PC12, with PC04, PC05, and PC07 deferred and partial decisions recorded explicitly. No implementation or external action is authorized. Next asked whether PC01's concrete examples should be a preset data package and the planned Halftone raster transform, recommending that pair. The answer is pending. See the [reconciliation](post-call-reconciliation.md#pc11-delivery-answer-2026-11-11).

### PC01 examples deferred; first-pass checkpoint, 2026-11-11

The user selected "Leave the exact examples open for now." The MVP remains one manually installed L0 data package and one L1 transform in EffectCraft, with save/reopen and failure handling; presets and Halftone are proposals rather than selected MVP examples. Existing P4 Halftone planning remains historical design detail, not a new example selection.

Consolidated a [current understanding table](post-call-reconciliation.md#current-understanding-after-the-first-pass) covering each PC decision, confirmed choices, remaining subquestions, and explicit deferrals. No question card is pending at this checkpoint. PC04, PC05, PC07, and exact MVP examples remain deferred. Other open details include "verified against main", theme-token contracts, release propagation, owners and delivery details, and legal-review timing. Existing outside-launch and native-release review requirements remain approved. No implementation, builds, commits, pushes, remote checks, publication, or external messages were performed.

### Local craft-extensions handoff requested, 2026-11-11

The user requested consolidating all plugin documentation, architecture, and plans into the existing local craft-extensions folder, including the relevant GitHub material, and creating a new top-level thread there for subsequent work. Local documentation transfer and read-only GitHub retrieval are authorized. No push, commit, publication, external message, or implementation is authorized by this preparation request. Preserve the source documents and unrelated changes.

Read-only GitHub retrieval found main at `98a151022a4bd4ff8aadda5161abb36789ae43b2`; PR #1 and PR #3 are both merged. PR #3's head remains `83917e4e0380288155029cb27f1128cca0f6168f`. Its former `action_required` status is historical. The original handover branch remains at `ffd8512e5a3755637f58cf31cfcf2fdb09cfb943`, matching the archived handover. Repository documents and review discussions are being preserved as evidence, with the confirmed post-call design taking precedence. Nothing is rebuilt or runtime-verified by this retrieval.

### System walkthrough and guide, 2026-11-12

The user asked for every part of the plugin system to be broken down and explained one at a time, using their questions as reference: how plugins interact with apps, permissions and enforcement, how modders build plugins and learn what apps offer, native and sandboxed plugins, how apps show windows and change, plugin types and mechanisms, shipping and storage, the store, keeping the separate theme work unobstructed, and safety. At the user's request, three read-only agents assisted: Haiku read the Craft framework document (`aiongg/craft-libs` branch `craft-framework`, `479327ee`), GPT-6.1 Sol (high) reviewed the architecture for scalability, and GPT-6 Luna partially traced EffectCraft at `ce9eaec6`; the main thread verified their key claims against source.

The user reaffirmed the 2026-11-10 direction: no app's current plugin system, EffectCraft's included, is a base for the design; all are removed and replaced by proper adapters. Wrote the [system guide](../system-guide.md) from the new design outward, with eighteen parts, ten end-to-end workflows, the decided/open/deferred list and fifteen review findings (R1–R15) recorded as findings, not decisions. Raised one question: whether SoundCraft's VST3/CLAP/AU hosting and CADCraft's AutoLISP, kept as native full-access mechanisms by the gap-check, remain an exception under the reaffirmed wording. The answer is pending. No architecture decision changed. No implementation, builds, commits, pushes or external actions were performed.

### Six-agent architecture review and AR01, 2026-11-12

The user requested all workflows, components, stages, safety, permissions and distribution in detail, many agents, and inspiration from open PRs, explicitly without external comments. Six additional read-only passes covered delivery/workflows, runtime/permissions, store/distribution, author/adapters, PR/history, and model challenges. The main thread checked key evidence and made only local documentation changes.

Read open PRs #4/#5/#6/#8 at exact heads recorded in the [correction register](architecture-review-2026-11-12.md), preserving patches under `sources/architecture-review-2026-11-12/`. GitHub main was read at `964c71574affa68da560a19d0efc4bb1f0263aee`; local source remains `a6c5fae`. PR6 concurrency interleavings and PR8 frozen-writer limitations are static findings, not executed failures. No external review/comment or other GitHub mutation was made.

The user confirmed AR01 after the two-app/pending-permission example: retain every release referenced by selected/pinned setups, pending consent, dependencies or active work, and remove only unreferenced releases. This refines T08; updated architecture/P1/main plan/decision history and the guide. Exact state fields remain open.

Added the [workflow/component/stage walkthrough](system-walkthrough.md), [all-forty-scenario map](workflow-coverage-2026-11-12.md), and AR01–AR20 correction register. Corrected source claims about manifest retention, legacy writers, downloader reuse and FigCraft API levels; qualified consent/revocation/determinism/native claims and explicit cutover dependencies. All other contract proposals remain open. Preserved the existing system guide and its independent theme/UI findings. No current app plugin implementation is adopted as a base. Existing explicit native-format exceptions, outside review before P12, two-maintainer native approval, PC04/05/07 deferrals and deferred MVP examples remain the baseline unless the user explicitly changes them.

Next asked AR10 whether each logical L1 request should start from the same initial guest state, allowing compiled-module caching and instance reuse only with an equivalent reset. Recommended that behavior after the tile-counter example. The answer is pending; no performance or reset implementation was selected.

### AR10 stateful-transform discussion requested, 2026-11-12

The user selected "Discuss stateful transforms first." Recorded this as a request for explanation, not approval of fresh-state execution or a new stateful ABI. Explained authored/project data, semantically invisible scratch caches, and meaningful processing history separately. A fresh instance can receive explicit previous state, while a reused instance can still have stateless semantics; instance lifetime and effect state are separate choices.

Proposed explicit host-managed state with an output/next-state contract, and identified seeking, tile order, preview/final isolation, checkpoint/persistence, budgets and cancellation as required design questions. Current P4 reserves state schema and defines no migration/state exports; P6's stateless basic-temporal recommendation remains a proposal. The [AR10 discussion](architecture-review-2026-11-12.md#stateful-transforms-distinctions-for-discussion) preserves the explanation. No architecture choice, deferred MVP example or implementation authorization changed.

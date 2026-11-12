# Post-call plugin decisions

Recorded 2026-11-11 from two meeting summaries supplied by the user in the planning conversation. The call was described as yesterday. Its exact date, speaker attribution, and order of statements have not been independently verified. These are summaries, not a transcript.

Status: discussion open. The user requested a new thread to reach a final understanding and keep the discussion in this repository. Documentation and design work are authorized. Implementation, builds, commits, pushes, external messages, releases, publication, and new repositories are outside this request.

## Current understanding after the first pass

Update 2026-11-12: the user requested a six-agent workflow/architecture review and approved AR01, retaining all referenced releases rather than only current-plus-leased releases. Read the [walkthrough](system-walkthrough.md) and [correction register](architecture-review-2026-11-12.md) for source corrections and open contracts. Earlier PC deferrals remain in force.

The first usable MVP is EffectCraft with one manually installed L0 data package and one L1 transform, including save/reopen and failure handling. L0 reads data through bounded importers; L1 executes a transform over typed input and validates its output. Exact examples are deliberately open. Presets and Halftone were proposed but not selected for the MVP.

| Decision | Confirmed understanding | Still open or deferred |
|---|---|---|
| PC01 | EffectCraft first, L0 and L1, with save/reopen and failure handling | Exact examples deferred by the user; detailed acceptance, cutover, and other-app rollout remain open |
| PC02 | Import and validate a package, then explicitly enable it; installation runs no plugin code | Package-format details remain governed by the existing design |
| PC03 | Shared package files; enablement, settings, and permissions separate per app | Exact platform paths |
| PC04 | No replacement milestone policy chosen; existing P7 feature and P12 launch wording remains the baseline | Deferred by the user; "version 7" is unconfirmed |
| PC05 | No replacement checking policy chosen; existing checks and local validation remain | Scanning policy deferred by the user |
| PC06 | WebAssembly permissions stay host-enforced; disclosures explain access; optional permissions unavailable until granted | Meaning of "verified against main" |
| PC07 | No replacement review policy chosen; outside review before P12 and two-maintainer approval for every native listing/release remain | Reviewer roles deferred by the user, including CodeRabbit's role |
| PC08 | GitHub Pages initially, custom domain later; maintainer-controlled TUF signing without separate publisher keys; source/namespace checks and pinned hashes remain | Exact endpoint, operators, and key holders; older G2 wording needs later external-doc reconciliation |
| PC09 | "IGUI" meant egui; use existing components without waiting for the separate theme framework; adopt shared tokens later | Token schema, ownership, timing, and early AI UI adoption |
| PC10 | Shared platform in craft-extensions, app adapters in app repositories; no MVP monorepo move | Release-version propagation |
| PC11 | Small independently reviewable PRs, with complete EffectCraft L0/L1 integration proof | Named owners, navigation coordination, exact PR contents, and estimates |
| PC12 | Short disclaimer in the manager and catalogue README explaining that community plugins are not guaranteed | Exact wording, additional policy changes, and formal legal-review timing |

Confirmed choices are reflected in the architecture and decision history; only affected plans were reconciled. The dated entries below preserve the questions, answers, and explanations. No unanswered question card is pending at this checkpoint. Deferred choices remain deferred, and the unit plans are not a fully reconciled MVP implementation sequence. This is a documentation checkpoint, not implementation authorization.

## Sources and current authority

- [Current architecture](../architecture.md), especially sections 8, 11–14, 17, and 21–24.
- [Accepted decision history](../draft/decisions.md#design-session-decisions).
- [Session record](session-record.md).
- [Top-level implementation plan](../implementation-plan.md) and [unit plans](../implementation/README.md).
- [craft-extensions PR #1](https://github.com/storytold/craft-extensions/pull/1), merged architecture and maintainer answers.
- [craft-extensions PR #3](https://github.com/storytold/craft-extensions/pull/3), reviewed at `83917e4e0380288155029cb27f1128cca0f6168f` in the preceding conversation. It changes README.md, docs/THREAT-MODEL.md, and the workspace repository URL. It implements no plugin functionality. CI reported `action_required` at review time. Recheck status before making a current claim.

The call summaries are new evidence. Conflicting statements remain unresolved until the user clarifies them. The assistant's recommendations below are proposals, not accepted decisions. Existing explicit approvals remain the baseline.

## What the user reported

The technical content of both summaries is retained below. Personal staffing details and account details unrelated to the design are omitted.

### Distribution, reviews, and delivery

- Keep the catalogue on GitHub for now. Revisit self-hosting later if needed. One suggestion was a catalogue in a repository or crate. The intended storage and serving arrangement was not specified.
- Use automated security checks rather than constant manual monitoring. Different passages say scan only new entries, scan new or updated entries but not every update, or defer scanning entirely. These statements do not establish one policy.
- Keep the `third-party` feature off until "version 7." This likely means delivery unit P7, but needs confirmation.
- One passage says the security gate should not accept permission families and should be verified against main. The intended policy is unclear and conflicts with the accepted G6 permission model.
- Use an "at your own risk" disclaimer. Avoid an obligation to audit every publisher repository or every PR in depth. Formal legal review before wider distribution remains an open question.
- Initially use trusted plugins from the existing community. Opening distribution to completely unknown contributors comes later.
- Start building an MVP iteratively. One initial PR was suggested, followed by iterations. CodeRabbit would be the primary initial reviewer.
- Build L0 and L1 adapters first. The initial owner handles early integration, and individual app maintainers subsequently own their adapter work. Coordinate app navigation with those maintainers.
- Prioritize individual apps over universal launcher support until the theme and core architecture are stable.
- Default plugin storage to an app-data ArtCraft plugins folder. Avoid accidental installation across apps. The summaries do not establish whether physical storage is shared or per app.
- Monorepo versus separate repositories remains open. A deterministic release script was suggested to manage separation and patch-version propagation.
- Timing estimates ranged from hours to a week. No measured schedule, fixed deadline, or accepted delivery commitment was supplied.
- Retain provisional status in the repository to discourage premature implementation against unsettled contracts.
- Proposed document corrections were the native exception to protection by design and replacing "maintainers and bosses" with "maintainers."

### Runtime, adapters, projects, and themes

- Provide CLI integration. WebAssembly plugins run in a sandbox. Native programs run in separate processes.
- Each app adapter defines supported operations, data types, and UI placements such as menus, inspectors, tools, and canvas integrations.
- Projects refer to required plugins rather than carrying installation binaries. Missing plugins need clear handling and a prompt or explanation.
- Isolate plugins from each other. Network access and system information were described as optional user disclosures, without a clear distinction between disclosure, consent, and enforced authority.
- A verified-publisher scheme was considered and deferred because of scanning and maintenance costs.
- Manual installation, including drag-and-drop into app plugin folders, is the immediate delivery path. The summaries do not specify whether this imports a package or directly loads loose files.
- Develop a separate reactive UI library on the framework named "IGUI" in the notes. Verify that name before identifying a dependency. The existing plugin architecture names egui.
- Support theme tokens for colors, typography, spacing, and border radius, with per-app overrides. JSON is the current theme format, and the future format is open.
- Allow community themes and saved themes, with installation and selection through a plugins menu.
- AI tools, including floating toolbars, generative fill, and object selection, may ship before full theme conformance. A later idea is generating parameter panels from ComfyUI workflow JSON.
- The theme framework has no confirmed completion date. Whether early AI UI must later adopt its tokens remains open.

## How this differs from the existing agreement

| Topic | Existing agreement | Call evidence and status |
|---|---|---|
| Initial installs | Labelled local file installs before the catalogue | Confirms that path and emphasizes delivering it first |
| App integration | L0 early, then EffectCraft, FigCraft, PhotoCraft pilots | Emphasizes L0 and L1. First app and exact MVP scope still need confirmation |
| Launcher | Optional, sharing platform code and store | Explicitly deferred in delivery priority. Shared storage does not depend on building launcher UI |
| Catalogue | GitHub metadata repository, author-hosted assets, signed metadata served from a domain with Pages backup | GitHub retained. Serving directly from GitHub initially may be a change, but is not yet explicit |
| Scanning | Automated release checks, selected human review, 48-hour hold for bot-merged updates | Contradictory summaries. No accepted replacement policy yet |
| WebAssembly permissions | Declared, host-granted, generation-checked authority on each host call | Optional-disclosure wording could mean a different model. Needs explicit clarification |
| Native review | Full-access label, consent, separate process, two maintainers for every listing and release | Process isolation confirmed. No explicit withdrawal of the two-maintainer rule |
| Launch review | Independent outside security review before public launch P12 | Not clearly addressed. CodeRabbit is not evidence that this approval was replaced |
| Storage | One shared store with per-app enablement and grants | App-data location fits. Separate per-app folders would change the architecture |
| Theme scope | Shared palette now, broader themes revisited later | Adds a separate reactive framework direction and richer tokens. Its contract with plugin UI remains open |
| Repositories | Shared platform in craft-extensions, adapters in app repositories | Monorepo is a proposal awaiting maintainer input |
| PR delivery | Small independently reviewable changes | One initial PR was suggested. Scope must be defined before deciding whether it fits one PR |

## Decision register

The first pass through PC01–PC12 is complete. Confirmed choices cover the MVP app and levels, installation, storage, enforced WebAssembly permissions, catalogue serving/signing, egui and theme dependency, repository split, small PRs, and short disclaimer. PC04, PC05, and PC07 are deferred at the user's request. Exact MVP examples and acceptance details, "verified against main", theme-token contracts, release propagation, named owners and delivery details, and formal legal-review timing remain open. The dated answers below define the scope of each confirmation. Recommendations do not change status. The table retains the initial recommendations for comparison.

| ID | Decision | Initial recommendation |
|---|---|---|
| PC01 | First app, L0 and L1 features, acceptance criteria, and exclusions | Start with EffectCraft unless the user changes pilot order. Prove one data package and one transform through install, enable, use, save, reopen, and failure handling |
| PC02 | Package import versus executable-folder discovery | Drag-and-drop imports a validated package. Explicit enablement follows validation |
| PC03 | Shared package storage versus per-app physical stores | Shared ArtCraft storage with explicit per-app enablement and grants. Installation alone never enables another app |
| PC04 | Local pilot, trusted-community distribution, and unrestricted public launch milestones | Name separate milestones and their gates. Confirm P7 means a delivery unit, not a product version |
| PC05 | Automated scanning triggers and human-review triggers | Defer catalogue services during local MVP. Later check every changed release artifact automatically and select human review by risk |
| PC06 | Enforced permissions versus optional disclosures | Preserve enforced WebAssembly permissions. Treat native full access separately. Clarify what "verified against main" means |
| PC07 | Outside review, native two-maintainer approval, and CodeRabbit's role | Retain explicit existing approvals unless maintainers replace them. Define CodeRabbit as an initial code-review tool |
| PC08 | Catalogue serving and publisher authentication | Clarify GitHub-only serving versus domain and CDN later. Resolve publisher signatures in threat-model G2 versus maintainer-controlled TUF signing |
| PC09 | Plugin manager framework and theme-token ownership | Verify "IGUI." Keep the local manager independent of unfinished theme work and define the later integration contract |
| PC10 | Repository structure, patch releases, and dependency propagation | Retain existing repositories pending an explicit decision. Specify the release problem before designing a script |
| PC11 | Adapter ownership, navigation coordination, PR boundaries, and schedule | Name the first owner and app contacts. Scope the first usable PR before estimating delivery |
| PC12 | Disclaimer text, listing policy, and any formal legal-review milestone | Record the disclaimer policy and leave legal-review timing explicit rather than inferred |

## Work already completed

PR #1 supplied the architecture and included both wording corrections. PR #3 is the threat-model and README follow-up. Those action items are historical, not new tasks to duplicate.

The existing implementation plans remain useful engineering detail. Their delivery order and scope need reconciliation after PC01–PC12. They are not authorization to start coding.

## Record of the new discussion

The first pass through PC01–PC12 is recorded below. PC04, PC05, and PC07 are deferred. Partial choices leave the remaining subquestions explicit. The user's instruction authorizes repository documentation and discussion only.

For each answer, add its date, decision ID, chosen behavior, reason, affected documents, and any remaining dependency here. Update the architecture and decision history only for choices actually confirmed. Preserve earlier decisions as history when superseded.

Start with PC01. Explain L0 and L1 in terms of what the first app's user can do. Present a recommended first usable scope and its consequences, then ask one focused question. Resolve storage and runtime authority before implementation planning. Keep useful independent source reading and documentation moving while answers are pending.

### PC01 proposal presented, 2026-11-11

Historical proposal, presented before the answers below. Only explicitly selected behavior is confirmed; the remaining details are proposals.

The bound craft-libs worktree is at `fd5e2feff6ea25bed371a99cc732b3968d621`; the plugin documents are uncommitted working material. This discussion uses that material, including architecture sections 4, 9, 16, 17, 21, and 22, the design-session decisions, the session-record tail, and the implementation index. No remote state was rechecked.

L0 lets a user manage installed packages and use data add-ons without running plugin code. Examples include presets and LUTs, read through bounded importers. L1 adds transforms: the host supplies typed input, runs the plugin under its sandbox and budgets, validates the result, and integrates it into the app. L2 adds session plugins that call app operations, such as tools and panels. These levels describe app support, not successive product versions.

The proposed first usable MVP is EffectCraft with one data package and one WebAssembly transform, installed manually. EffectCraft follows the accepted pilot order. One example of each kind exercises both data import and executable processing before repeating the integration in other apps. The exact data kind remains to be selected. The existing P4 halftone example and `raster.filter@1` are candidates for the transform, not new confirmed choices.

The proposed workflow and acceptance criteria are:

1. Install a local package with validation and the host-assigned "Not reviewed" label. Installation alone enables nothing. Whether drag-and-drop imports an archive or discovers a folder remains PC02.
2. Explicitly enable its supported contribution in EffectCraft. Apply the existing permission rules; PC06 must resolve the call's conflicting wording before a replacement policy is designed.
3. Use one data contribution and apply one transform through the app's normal editing workflow, with undo and checked output.
4. Save and reopen a project containing the transform, retaining its reference, parameters, and state. Installing or enabling code merely because a project opens remains prohibited.
5. Reopen with the transform unavailable. Preserve its data in a placeholder and refuse final export until the dependency is resolved. A trap, invalid output, or exhausted budget must not crash the app or commit a partial edit.
6. Exercise local package management through the GUI and app CLI. The exact CLI and automation delivery boundary still needs agreement; a smaller MVP must not silently waive the approved parity requirement.

Proposed exclusions from this first slice are catalogue browsing and distribution services, launcher integration, L2 session plugins, native helpers, and the separate theme-framework work. This proposal demonstrates one L0 data kind and one L1 contract. It does not establish that all L0 resource kinds or all L1 transforms are supported. Supporting only L0 first would avoid executable processing in the first slice, but leave transform execution, persistence, and render failures unproven.

The question presented was whether to choose EffectCraft with L0 and L1, EffectCraft with L0 only first, or another app with L0 and L1. Choosing the app and broad scope does not settle the exact examples, all acceptance criteria, or the remaining PC decisions.

### Implementation implications awaiting decisions

This is a map to existing plans, not a revised delivery plan or authorization to implement.

| Existing plan | Implication if the proposed scope is chosen |
|---|---|
| P1 store and manifest | Local installs still need validation, explicit enablement, coherent state, and safe release retention. Physical layout remains PC03 |
| P3 host and P4 contract/SDK | L1 requires a bounded runtime and validated transform output. Manual installation does not replace those protections |
| P5 manager and CLI | Define the complete local workflow before adding catalogue screens. Do not infer a new UI dependency from "IGUI" |
| P5b and P6 app integration | Reconcile one-app delivery with the approved early L0 rollout. Resolve old-system removal timing and the EffectCraft dispatcher, persistence, export, and automation dependencies before reducing P6 |
| P7 catalogue and P12 launch | Keep catalogue delivery, `third-party` feature enablement, and public-launch approval distinct. The meaning of "version 7" remains unconfirmed |
| P10 native and P11 themes/other apps | Deferring these from the first slice does not change their accepted contracts or native review requirements |

At presentation, this proposal changed no architecture, accepted decision row, or unit plan. The answers below now refine those documents. The outside security review before P12 and two-maintainer approval of every native listing and release remain approved.

### PC01 answer, 2026-11-11

The user selected "EffectCraft with L0 and L1 (Recommended)" in response to the first-app/MVP question. Confirmed scope is one manually installed data package and one transform in EffectCraft, including save/reopen and failure handling. This follows the accepted pilot order and proves both data import and transform execution in one app. That rationale was presented with the recommendation; the user supplied no separate rationale.

The exact data package and transform example remain open. The full acceptance checklist above, CLI/automation delivery boundary, other-app rollout, and old-system cutover need refinement; the selection does not waive existing requirements. Architecture section 22 and the decision history now record this first usable scope. The P5/P6 plans and their index carry reconciliation notes rather than a new implementation sequence.

### PC02 answer, 2026-11-11

The user selected "Import and validate a package (Recommended)." Manual installation imports and validates a package, followed by explicit enablement. Dropping a package into the app enters that import flow. The app does not directly execute loose plugin files discovered in a watched folder. Installation runs no plugin code and enables nothing.

The reason presented was that import establishes validated installed contents, whereas loading mutable loose files complicates validation and updates. No separate user rationale was supplied. Architecture section 12, the decision history, and the P5 plan now reflect this choice.

### PC03 answer, 2026-11-11

The user selected "Shared files, separate app setup (Recommended)." Package files are stored once in shared ArtCraft storage; enablement, settings, and permissions remain separate per app. An EffectCraft install does not enable anything in PhotoCraft. This reconfirms the existing architecture rather than choosing per-app physical stores. Exact platform paths are not settled here.

The recommendation explained that sharing avoids duplicate package files and updates while explicit app setup prevents accidental cross-app activation. No separate user rationale was supplied. Architecture section 11, the decision history, and the P5 plan record the confirmation.

### PC06 question presented, 2026-11-11

Runtime authority followed storage because implementation scope depends on the security contract. Asked whether WebAssembly permissions remain host-enforced, with disclosures explaining access and optional permissions unavailable until granted, or whether the user wants to discuss a disclosure-only alternative first. Recommended preserving enforcement. A disclosure describes access but cannot restrict it. Native full access remains a separate accepted contract.

### PC06 enforcement answer, 2026-11-11

The user selected "Keep enforced permissions (Recommended)." WebAssembly permissions stay host-enforced; disclosures explain access, and optional permissions remain unavailable until granted. This reconfirms the existing per-app grants and host-call checks rather than replacing them with disclosure alone. Native full access remains separate. Updated architecture section 8 and the decision history. Existing unit plans already require enforcement and need no behavior change from this confirmation.

The meeting phrase "verified against main" remains unresolved. This answer does not assign it a meaning or close that part of PC06.

### PC04 deferred, 2026-11-11

Asked whether "version 7" meant delivery unit P7 while retaining the separate P12 public-launch gate. The user answered "leave this point for now." PC04 remains unresolved; neither interpretation nor a replacement milestone policy is approved. Keep the existing P7 feature and P12 launch wording as the baseline, without treating it as a new confirmation. Do not infer that the manual MVP permits third-party feature enablement or public distribution.

### PC05 question presented, 2026-11-11

Recommended automated catalogue checks on every new or changed release artifact before availability. Unchanged artifacts need no repeat scan solely because listing text changes. Local MVP package validation remains required; catalogue scanning arrives with catalogue delivery. Human-review triggers remain a separate question, and existing native two-maintainer and outside-launch review requirements remain approved. Asked whether to check every new or changed artifact, only the first listing, or leave scanning policy unresolved.

### PC05 deferred, 2026-11-11

The user selected "Leave scanning policy unresolved for now." No replacement scanning policy or exemption is approved. Preserve the existing architecture and P7 checking requirements as the baseline. The conflicting call summaries remain unresolved, and manual package validation remains required. No architecture or implementation-plan change follows from this deferral.

### PC07 question presented, 2026-11-11

Recommended CodeRabbit as an initial code-review tool alongside human review, retaining independent outside security review before public launch P12 and two-maintainer approval for every native listing and release. Asked whether to retain both requirements, discuss changing them, or leave reviewer roles unresolved. Named reviewers, review scope, and ownership are not assigned by this question.

### PC07 deferred, 2026-11-11

The user selected "Leave reviewer roles unresolved for now." No replacement review policy, CodeRabbit role, or reviewer assignment is approved. The existing independent outside review before P12 and two-maintainer approval for every native listing and release remain in force. Architecture, decision history, and unit plans need no change from this deferral.

### PC08 serving question presented, 2026-11-11

Distinguished the catalogue's GitHub metadata repository, authors' GitHub Release assets, and the endpoint from which apps fetch signed metadata. Recommended serving signed metadata through GitHub Pages initially, with a custom domain later. The prior approved design used a custom domain as primary and Pages as backup. Both retain TUF-signed metadata and author-hosted package files. Asked which serving arrangement to use or whether to leave serving unresolved. No hosting or publishing is authorized by the discussion.

### PC08 answers, 2026-11-11

The user selected "GitHub Pages first, custom domain later (Recommended)." This supersedes the 2026-11-10 custom-domain primary and Pages backup choice. Initial catalogue serving and the P12 serving prerequisite now use Pages. A domain is later work, with no agreed date. The recommendation reduces initial hosting setup while retaining static signed metadata and author-hosted package files.

The user then selected "Retain catalogue signing model (Recommended)." Maintainers control TUF catalogue signing; publishers do not each need a separate signing key. Namespace/source checks remain, and catalogue records pin package hashes. The rationale presented was retaining authenticated metadata without adding per-publisher key management. The older G2 publisher-signature wording should be corrected to this model when external docs work is authorized. No remote document was changed or rechecked here.

Updated architecture sections 12, 13, 22, 24, and 25, the decision history, P7/P12 plans, the unit-plan index, and the top-level implementation plan. Signing operators, root-key holders, repository name, and exact Pages endpoint remain unassigned. Existing scanning and review requirements remain the baseline because PC05 and PC07 were deferred.

### PC09 answers, 2026-11-11

The user confirmed "It meant egui" when asked about "IGUI" in the notes. No different UI framework is intended by that name.

The user selected "Proceed now, adopt tokens later (Recommended)" for using existing egui components without waiting for the separate theme framework. The manual MVP is independent of completing that framework; the manager adopts shared theme tokens when the contract is ready. The rationale presented was removing unfinished theme work as a prerequisite for the first plugin workflow. "Proceed now" answers a design-dependency question and does not authorize implementation in this documentation-only discussion.

Updated architecture section 14, the decision history, P5, and the unit-plan index. The call's proposed colors, typography, spacing, border-radius tokens, and per-app overrides still need an exact schema and ownership agreement. This answer does not itself expand the approved theme contribution format or decide whether early AI UI must adopt the tokens. Timing remains open.

### PC10 repository question presented, 2026-11-11

Recommended retaining shared plugin-platform code in craft-extensions and each app adapter in its app repository for the MVP. The alternative is exploring a coordinated monorepo move before implementation. Asked which path to take or whether to leave structure unresolved. Release-version propagation is a separate open part of PC10; the call's deterministic-script suggestion is not an approved implementation or a reason to infer a repository migration.

### PC10 repository answer, 2026-11-11

The user selected "Retain the existing split (Recommended)." Shared platform code stays in craft-extensions; app adapters remain in their app repositories for the MVP. No monorepo migration is part of this slice. The recommendation preserves the existing ownership and dependency boundaries without a broader coordinated move. Updated architecture section 5 and the decision history. Existing unit plans already use this repository arrangement and need no structural revision.

Release-version propagation remains open. No deterministic release script, version-bump policy, automated publishing, or cross-repository write permission is approved by this choice.

### PC11 delivery question presented, 2026-11-11

Recommended small, independently reviewable PRs ending in a verified complete EffectCraft L0/L1 workflow. This separates shared-library changes from app integration without claiming an incomplete user workflow is the MVP. Asked whether to use that approach, prefer one larger initial MVP PR, or defer delivery boundaries. Named owners, app navigation coordination, exact PR contents, and delivery estimates remain unassigned. This is a design choice about future delivery, not authorization to implement or open PRs.

### PC11 delivery answer, 2026-11-11

The user selected "Small PRs, complete integration proof (Recommended)." The MVP arrives through independently reviewable changes, and integration verification covers the complete EffectCraft L0/L1 workflow. This retains small delivery steps rather than choosing one larger initial MVP PR. Updated architecture section 22, the decision history, and the top-level plan. Existing unit PR lists remain useful detail but are not a newly finalized MVP sequence. Owners, navigation coordination, exact contents, estimates, and implementation authorization remain open.

### PC12 disclaimer answer, 2026-11-11

The user selected "Retain the short disclaimer (Recommended)." The manager and catalogue README explain that the project does not guarantee community plugins. The explanation presented placed that statement alongside existing checks and review requirements. Updated architecture section 12 and the decision history. Existing P12 tasks already cover the disclaimer and need no behavior change. Exact wording, any additional listing-policy changes, and formal legal-review timing remain open. No legal conclusion or publication is approved here.

### PC01 example question presented, 2026-11-11

Recommended a preset data package for L0 and the existing planned Halftone raster transform for L1. These are intended demonstration targets, not claims of implemented support. Asked whether to select that pair, choose different examples, or leave examples open. The preset contents and target, package identities, and fixture outputs would still need specification before implementation.

### PC01 examples deferred, 2026-11-11

The user selected "Leave the exact examples open for now." No preset package or Halftone transform is selected as the MVP demonstration by this discussion. The confirmed scope remains one L0 data package and one L1 transform in EffectCraft, with save/reopen and failure handling. Halftone remains an example in the existing P4 plan; that historical plan is not new approval of the MVP examples. Architecture and unit plans already leave the MVP examples open and need no behavior change from this deferral.

Consolidated the current understanding at the top of this document. No additional choice is inferred from the deferral, and no implementation or external action is authorized.

### Architecture walkthrough and AR01, 2026-11-12

The user requested detailed coverage of all workflows, components, delivery stages, safety, permissions and distribution, with many agents and inspiration from open PRs. They explicitly prohibited external comments. This authorizes the parallel read-only review and local documentation, not implementation, builds, commits, pushes, reviews/comments, publication, releases or new repositories/remotes. Six agents performed independent bounded passes; only the main thread edits the working documentation.

Open PRs #4, #5, #6 and #8 were read at recorded heads and preserved in the [review sources](../sources/architecture-review-2026-11-12/pr-4.json). PR #6's concurrency concerns and PR #8's legacy-writer limitation are static findings, not reproduced failures. Existing PR #1/#3 documentation work is complete and is not repeated. Local code remains `a6c5fae`; GitHub main was read at `964c71574affa68da560a19d0efc4bb1f0263aee` without updating the checkout.

After the two-app/pending-permission example, the user chose "Retain all referenced releases (Recommended)." AR01 keeps defaults, effective/pinned app selections, pending-consent references, dependency releases and active work, and cleans only unreferenced releases. This refines T08's current-plus-leases choice. Updated architecture section 11, P1, main plan and decision history. State field names and pending-update supersession remain open.

The [walkthrough](system-walkthrough.md) and [review register](architecture-review-2026-11-12.md) record further source-backed corrections and proposed contracts. P1 manifest retention, legacy-schema claims, P2 consent comments, P3/P4 proof qualifications, P6 explicit dependencies, downloader/API-level facts and native guarantee wording were reconciled. The broader graph, instance-state, revocation, outbox, descriptor, migration/resource, project-dependency and source-binding choices remain under discussion. No other user decision is inferred from the request to repair the architecture.

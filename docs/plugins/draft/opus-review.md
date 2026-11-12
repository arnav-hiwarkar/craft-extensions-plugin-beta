# Independent Opus design review

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: independent architecture review, 2026-11-08. Written by the requested Opus collaborator. It responds to the primary draft ([decisions](decisions.md), [overview](README.md), [SDK and contracts](sdk-and-contracts.md), [security and distribution](security-and-distribution.md)). It also covers the [aspect register](../aspect-register.md) and Jeremy's [handoff plan](../sources/jeremy-extensions-todo.txt). These are design proposals. They are not user decisions, and nothing here was implemented, built, tested, fetched from the network or run. The primary owns reconciliation. Companion files that the overview links to but that are not written yet are out of scope here.

The review accepts the user's direction recorded in [decisions](decisions.md) as fixed. It disagrees only where a mechanism would contradict that direction, where an operator could not keep a promise, or where a simpler established pattern covers the same ground.

## 1. Recommended architecture and decisions

### 1.1 Summary position

The primary draft gets the shape right: one shared store, app-owned adapters, one operation path shared by GUI, CLI and plugins with caller-specific authority, explicit per-app consent, metadata-only Craft hosting and a closed sg13. What it lacks is commitment. Nearly every mechanism is still "candidate", "evaluate" or "open", so an author, an app maintainer or a reviewer cannot yet act on it. This review proposes concrete defaults for the parts that block authors and app maintainers, and keeps real policy choices open for the user.

The recommended system has five parts, and none of them is a new service:

| Part | Where it lives | One-line job |
|---|---|---|
| Package and trust layer | craft-ext-core and craft-ext-store in craft-extensions, extended | Identity, manifest schema 2, a multi-release store, grants, revocations and resolution |
| Catalogue client | A module in craft-extensions, behind a cargo feature so web and headless builds can omit network code | Fetch and verify signed metadata, then download a GitHub Release asset by its approved digest |
| Operation layer | Each app's engine, plus one small shared descriptor and dispatch crate (it needs two real consumers before it moves to craft-libs) | Typed operation descriptors, caller context and a single policy check point |
| SDK | One `craft-plugin` Rust crate with a CLI subcommand set, plus code generated from app operation inventories | Author, check, test, pack and prepare releases |
| Catalogue repository | One new human-reviewed GitHub repository, with metadata only | Package and release records, review records, revocations, signed published metadata and a star snapshot |

### 1.2 Concrete decisions this review recommends

These are defaults for the primary to adopt or reject. Section 6 lists the decisions that belong to the user.

1. **Store key is a UUID, with a binding to its source.** The SDK generates a random (version 4) UUID at `create`. A UUID only names a package; it proves nothing about who published it. The catalogue binds each UUID to its source: the GitHub owner's numeric ID and the repository's numeric ID. Those numbers stay the same when an owner renames an account or repository. Confirm that against the GitHub REST documentation before relying on it; this review did not fetch it. The store records each installed release as `(uuid, source, release digest)`. A package with the same UUID from a different source counts as a different source: it never replaces the first, never inherits its grants and never receives its updates.
2. **Readable handle.** CLI and UI need something people can type. Use `owner/name` (for example `storytold/invert`). The catalogue, or the installed records, resolve it to a UUID. Projects store the UUID, the contribution ID and the exact release digest, never the handle.
3. **Legacy dotted IDs migrate by alias.** The catalogue publishes an authenticated alias table that maps each `org.storytold.*` ID to its UUID. The store migrates its records once and keeps the aliases. App project readers accept a dotted reference through the alias map and write UUID references on the next save. The reserved first-party prefixes stay reserved as aliases, so a local file still cannot claim them (`tier.rs` `check_id`).
4. **The operation descriptor is the interface definition.** Each app declares its operations in Rust: one registry per app, owned by the app. A build step exports a versioned, machine-readable operation inventory, either JSON or a schema format chosen when this is built. That inventory generates guest bindings in the SDK, CLI help and schema, documentation pages and adapter conformance lists. Choose this now. It needs no new runtime, it works with the wasmi and core-WASM engines the apps use today, and it gives one source of truth.
5. **The guest boundary is one dispatch import, gated by descriptor policy.** New contracts expose a host call shaped like `call(op_id, contract_version, request_bytes) -> response_bytes`, with bounded, versioned encoding, plus a small fixed set of imports for lifecycle, logging and job progress. Every call goes through one point that checks the descriptor's `allowed_callers` and effect class, the declared capability, the grant, the scope, the budget and live revocation. Section 3 (H1) explains why this is safer than the per-function imports the draft implies, and how it differs from the "generic dispatcher" the draft rightly forbids. Existing EffectCraft, FigCraft and PhotoCraft ABIs stay behind adapters, as the draft says.
6. **WIT is a possible later encoding, not a second authority.** Use WIT only if a component-capable backend is adopted later for reasons of its own, and then generate the WIT from the same inventory. This review did not verify whether wasmi supports components. The draft's "feasibility fixture" is the right test, but it should not block the SDK.
7. **The permission vocabulary is small and scoped.** Each permission is written as `capability(scope)`. The proposed initial capability set is in section 2.4. A grant is keyed by app, UUID, source, contribution, capability and scope. Install enables nothing.
8. **The store becomes multi-release.** It keeps several verified releases per UUID: the default for new uses, one rollback release, and releases pinned by projects, by explicit user pins or by active jobs. This needs a change to `state.rs` and `store.rs`; see C1 and H4.
9. **The catalogue is one signed index wrapped in TUF roles.** The existing `Index` format grows into schema 2 and becomes the TUF "targets" payload. Package archives stay on publisher GitHub Releases and are referenced by digest. Signed metadata is hosted on the metadata repository's own GitHub Releases or Pages. That is metadata hosting only, which the user's direction allows. Evaluate a vetted Rust TUF client before writing a custom protocol. Candidate libraries have not been checked here for licence, web compatibility or maintenance.
10. **Every executable release is reviewed, and tooling makes that practical.** Every executable release needs a human approval bound to its digest, including official releases. Automation produces a review report: the difference in requested permissions, in WASM imports and exports, in dependency inventory and licences, and in size, plus whether a GitHub artifact attestation exists. The reviewer approves that report and the source change. Security fixes have a documented fast lane with two maintainers. See C3.
11. **Provenance without a build farm.** Record any GitHub artifact attestation, which links an asset to a repository, workflow and commit, as checked evidence and show it as a "built by the publisher's GitHub workflow" fact. Do not require it. GitHub's own documentation (cited in the draft) says attestations do not guarantee safety, so the approved digest stays authoritative. This replaces Jeremy's d9 hub-build without losing all provenance signal.
12. **Ranking.** Filter first: query relevance, app, kind and compatibility. Then official entries come first, then community entries sorted by repository stars from a signed daily snapshot. The UI labels the number "repository stars" with the date it was taken. Stars never enter resolution or eligibility.
13. **Management UI.** Each app embeds the per-app enablement, consent and diagnostics panel, because those are app-specific. Discovery and install can run in-app through the shared crate, or through the launcher. Section 4 recommends one option and offers the other.

### 1.3 What stays exactly as the primary draft has it

Sandbox-first, with no native admission until confinement is proven on each platform. sg13 stays closed. First-party status, review, stars and consent never waive enforcement. No Craft-hosted binaries and no build farm. Local installation with no catalogue, under identical runtime controls. Exact release references for persistent features. Export fails when required processing is missing. Data-only themes, resources and workflows. Logs are private and bounded. Existing core and store are reused, and no installer or schema is duplicated in craft-libs.

## 2. Concrete SDK, app and user lifecycle

The commands below are illustrative names. They show the journeys a finished design must support.

### 2.1 Author journey: a multi-app filter

```text
craft-plugin new "Soft Glow" --publisher my-gh-user
  -> craft-plugin.toml  (display name, licence SPDX expression, support URL, generated UUID)
  -> src/lib.rs         (one contribution stub)
craft-plugin add photocraft.image-filter@2 --id soft-glow
craft-plugin add effectcraft.video-effect@1 --id soft-glow-video
  -> pulls the operation inventory for each named contract (pinned version in Cargo.lock-like file)
  -> generates typed bindings: Params struct, ctx.read_pixels(), ctx.progress(), etc.
craft-plugin permissions          # shows the capabilities the code's generated calls need
  -> author adds them to craft-plugin.toml explicitly (the SDK never adds them silently)
craft-plugin test                 # runs fixtures against a host harness built from the same inventory
craft-plugin dev --app photocraft # installs into an isolated development store, reloads on rebuild
craft-plugin pack                 # deterministic ZIP: craft-extension.json schema 2, modules, inventory, SBOM
craft-plugin release-notes        # emits catalogue-PR metadata: digest, size, permission summary, licence inventory
```

The author creates a GitHub Release, ideally immutable and built in GitHub Actions with an attestation, and uploads the `.craftext` asset. They then open a PR in the catalogue repository that adds `packages/<uuid>/releases/<version>.toml`, generated by `release-notes`. Validation recomputes the digest from the GitHub asset, checks schema, licences and permissions, and produces the review report. A maintainer approves and merges. Publication then signs new metadata. For an update, the author uploads a new GitHub Release and opens a new PR. Nothing changes for users until that PR is approved.

Rules the SDK enforces locally and the host enforces again at runtime: every declared permission is used, every used import is declared, parameter and state schemas match the inventory, files are listed exactly, and the version string matches in every place it appears.

**Language support.** The SDK supports Rust first. Another language counts as supported only when generated bindings and conformance fixtures exist for it. The draft's caution here is correct.

### 2.2 App maintainer journey

1. Declare the operations the app will expose in its registry, with descriptors (see `sdk-and-contracts.md` "Operation descriptions"). Mark `allowed_callers` as GUI, CLI or plugin per operation.
2. Implement an adapter for each contribution family the app supports, and advertise each contract version in the inventory.
3. Wire store resolution into every entry point: GUI startup, enable, project reopen, CLI startup and module load.
4. Pass the shared conformance fixtures: GUI/CLI parity, denied and nested calls, cancellation, save and reopen with a missing or updated release, and export failing when a required effect is missing.
5. Map ThemeFile to app tokens where themes apply.

The fixes each app needs are recorded per app (see [app adapters](../research/app-adapters.md)) and are scoped separately. Approving the design does not start them.

### 2.3 User journey

| Step | What the user sees | What the system guarantees |
|---|---|---|
| Discover | Search in PhotoCraft → Extensions. Official entries first, then community entries by repository stars (dated). The detail page lists its contributions for each app, the permissions it would request, compatibility, licence, size, network needs, "reviewed release" status and any attestation fact | No plugin code runs. Results come only from verified metadata |
| Install | Download progress, then "Installed. Not enabled in any app yet." | Digest is checked before unpacking. The store transaction is committed. **No app is enabled** (this is a change from today's store, C1) |
| Enable in PhotoCraft | One consent sheet: each permission with its purpose and scope. Required permissions are marked. Optional ones can be switched off individually | The grant is recorded for PhotoCraft only. FigCraft still shows "installed, not enabled" |
| Use | Filter → Soft Glow, with progress and cancel | Effective authority is the intersection described in `sdk-and-contracts.md`. Budgets come from the workload profile |
| Save | Saving works as normal | The project stores the UUID, contribution, release digest, parameters and state. The store pins that release while the project is in recent-project history |
| CLI | `photocraft render in.pcraft -o out.png` | Uses installed, enabled and granted releases. Downloads nothing, grants nothing, opens no dialogs. If a required effect is missing, it exits non-zero with a structured error and leaves out.png unwritten or marked incomplete |
| Update, same authority | "Soft Glow 1.3 available (reviewed)" in the updates list. With the user's automatic-update preference it installs; new uses take the new default | Existing projects keep 1.2 until the user upgrades them explicitly. The grant is reused only with the same source and no wider authority (C2) |
| Update, wider authority | "Soft Glow 2.0 wants network access to api.example.com" | The new release installs but is not eligible in PhotoCraft until the user consents. 1.3 remains the default |
| Offline | Everything installed keeps working. Catalogue shows "last updated 3 days ago" | No routine online check. Known revocations still apply |
| Revocation | "Soft Glow 1.2 was withdrawn: malicious update. It has been switched off." with an upgrade or remove action | Revocation applies on the next metadata refresh, and at startup and enable from the cached metadata |
| Reopen with missing release | "This project uses Soft Glow 1.2, which is not installed. Reinstall 1.2 / Upgrade to 1.3 / Keep the last baked result / Open without it" | Never a silent substitution. Export refuses until the user resolves it |
| Local file | Drag a `.craftext` onto Extensions: "From a file — not reviewed by ArtCraft", then the same consent sheet | Same validation, sandbox and budgets. It cannot take a reserved ID or a catalogue source's UUID binding |
| Uninstall | "Remove from this computer" plus separate checkboxes for "delete settings" and "delete plugin data" | Bytes are kept while active readers or pins need them, and collected afterwards |

### 2.4 Proposed initial capability vocabulary

Each item is enforced by the host, and each can be limited by scope. This vocabulary is complete enough to design against now. Each app advertises the subset it implements.

| Capability | Scopes | Notes |
|---|---|---|
| `document.read` | `selection`, `active-document` | Queries only |
| `document.edit` | `selection`, `active-document` | Validated proposals committed through the app transaction |
| `file.read` | `user-picked` | A host-issued token from the OS picker. No paths |
| `file.write` | `user-picked`, `artifact-role` | The host writes the file atomically |
| `storage.private` | size quota | Namespaced to the app and package |
| `network.fetch` | declared domains (host-enforced, including redirects) | Disclosed as "sends data to …" |
| `account.use` | one named brokered service and action | Credentials never reach the plugin |
| `job.background` | `max-concurrency` | Long-running work with an owner |
| `compute.gpu` | typed host operations | Not arbitrary shaders. Arbitrary shaders would be a separate backend capability |
| `ui.panel`, `ui.tool`, `ui.canvas` | — | Listed for disclosure, not as security rights. They still need declarations so headless callers can report "UI-only" |

## 3. Must-fix objections, by severity

### Critical

**C1. The existing store contradicts accepted direction, and the draft does not say so.**
`install_staged` enables the new package in every app its manifest lists (`craft-extensions` `crates/ext-store/src/store.rs:201` at `main` a6c5fae, and still present at `wip/extensions-handover` ffd8512 line 218). It also deletes the previous release folder immediately after the state commit (`store.rs:213`). The user chose app-specific enablement, retained releases and protected active readers. The research reconciliation recorded this ([reconciliation](../research/reconciliation.md), "Updates re-enable disabled packages and delete previous files"), but the current draft's overview and security file state the target behaviour without naming these as required store changes. *Fix:* list them explicitly as prerequisites in the delivery document: no auto-enable on install, multiple releases per key, deletion deferred to collection under active-reader ownership, and update that preserves per-app enablement.

**C2. "Verified same-authority release continuity" has no defined anchor.**
Decisions proposal 5 and `sdk-and-contracts.md` (consent section) reuse grants under continuity, but publishers have no keys in this design. Binaries are GitHub assets and authenticity comes from catalogue signatures. *Fix:* define continuity concretely as follows. A catalogue release is continuous with an installed one when the signed metadata binds both to the same UUID and the same source (owner and repository numeric IDs, or a reviewed transfer record). A local file is continuous only with an earlier local install of the same UUID whose manifest-declared publisher signature matches, if publisher signing is later added. Otherwise a local update always asks again. Grant reuse also requires that the new effective permission set is a subset of the old one, compared per contribution after normalising scopes.

**C3. Human review of every executable release must state what it asserts, and needs a path for urgent fixes.**
The user wants people to review entries. The draft proposes approving every executable release, including official ones, which this review supports. As written, though, it implies an assurance a reviewer of compiled WASM cannot give, and it has no response-time path. *Fix:*
- Define the review record. The reviewer checks identity and source continuity, a sample of the source change between releases, the automated permission, import, dependency, licence and size difference, and any attestation. The record states that it is not a guarantee of behaviour.
- Add a fast lane for security fixes: two maintainers, permissions no wider than before, published after the fact with its reason.
- Allow data-only kinds (themes, resources, workflows) a lighter checklist while still needing a human merge, because the user asked for human review of catalogue entry.
- State staffing assumptions. Without named maintainers the catalogue does not ship.

**C4. The trust root and index code need closing before any network catalogue, even for first-party packages.**
Today `Index::parse` accepts an index whose version equals the newest one seen (`crates/ext-core/src/index.rs:126` uses `<`), so a changed index with the same version passes. The index is unsigned, and each entry states its own `tier` (`index.rs:100`). The draft correctly requires authenticated metadata and treats same-version changes as failures, but it presents signed metadata as a release-time item. *Fix:* any fetched catalogue must be signed, even a first-party-only one, and the trust root must ship inside the signed app. "Official" must come from a signed policy role, never from a field in an entry. The TUF key custody plan (offline root, online targets and timestamp in a protected GitHub environment, a threshold, a rotation drill) needs a named operational owner as a design deliverable, not an implementation afterthought.

### High

**H1. The draft's ban on a "generic dispatcher" will push designs toward a less safe shape.**
`sdk-and-contracts.md` says plugins cannot use "a generic dispatcher to regain user authority". The real danger is calling a *CLI bridge*, or reaching an operation whose `allowed_callers` excludes plugins. A single typed dispatch import, checked per operation, is the *safest* enforcement shape: one choke point, one audit log, and capability checks computed from the descriptor and the arguments (`capability_requirements_for(arguments, targets)`). Per-function imports spread checks across many host functions, which is how bypasses happen. *Fix:* reword the rule as "no caller-elevating bridge; every operation is dispatched through descriptor policy with the host-derived principal". Adopt the single dispatch import for new contracts.

**H2. Leaving the interface definition open blocks authors and app maintainers.**
The draft keeps "typed IDL, evaluate WIT" open. Authors need bindings, and apps need a registry format. *Fix:* adopt decision 1.2(4) now (the Rust registry generates a versioned inventory). Treat the WIT or component question as a backend decision with its own trigger: a measured need for a second guest language, for component resources, or for a runtime change.

**H3. Revocation granularity is too coarse for the incident cases the draft lists.**
`Revocation { id, below }` (`index.rs`) can only withdraw a version range for one ID. It cannot block one exact digest, block everything from a compromised source across UUIDs, or recover a version after revoking it. *Fix:* schema-2 revocations target an exact release digest, a UUID version range, or a source (owner or repository ID). Each has a severity: `block-now` (cancel at safe boundaries), `block-new` (finish running work), or `advisory`. Keep the highest index version seen and the known revocations across repair.

**H4. Exact project references need multi-release state, which nothing schedules.**
`State.installed` holds one record per ID, and install refuses an equal or lower version (`store.rs`, downgrade and already-installed checks). That cannot satisfy "existing projects keep their release". *Fix:* state schema 2 keyed by `(uuid, release digest)`. Pins come from recent projects, explicit pins, rollback and active jobs. Old writers refuse schema 2 rather than rewrite it, which is already the `StateError::Newer` behaviour. Keep it.

**H5. A local file can take over a catalogue package's identity.**
`install_staged` protects only the first-party tier from replacement (`store.rs:181`). A `FromFile` package with the same ID and a higher version replaces a reviewed third-party install and keeps its enablement. *Fix:* the source binding from 1.2(1). Different source means a separate install record, or an explicit "replace with an unreviewed file" action that resets grants and stops catalogue updates.

**H6. The licence policy draft conflates repository dependency rules with ecosystem product policy.**
The draft proposes the repository SPDX allowlist as a starting point for catalogue packages. That list governs what *Craft repositories* may depend on or ship. Applying it to third-party plugin products would, without anyone deciding so, exclude GPL, MPL and proprietary community plugins. It would also leave official proprietary packages, an existing requirement, in conflict with the policy. *Fix:* present the options neutrally in section 6 and do not label one "starting point". Separately, any package bundled into a Craft app release must comply with the allowlist. That part is not a choice.

### Medium

**M1. Gaps in the contribution families.** The twelve families in `sdk-and-contracts.md` miss several that apps need:
- File-format handlers: open and save associations for new types. This differs from export.
- Settings pages.
- Keymap and shortcut contributions, through the existing `craft-keymap` crate.
- Localization packs.
- Headless automation scripts: CLI-first operations with no GUI entry.
- Context-menu and inspector sections.
- Document validators or preflight checks, such as prepress or accessibility.

State explicitly that two families are designed but deferred: plugin-to-plugin exported APIs (dependencies) and resident background services. The deferral should be on purpose, not an omission.

**M2. Large model and data dependencies.** "No runtime dependency installer" is right for code. Data such as ML models still needs a path. *Fix:* a declared remote-resource contribution: URL on a GitHub Release (or another approved origin in signed metadata), digest, size, licence and readiness. The host fetches it into the store under the same retention rules. It is never fetched by plugin code. GitHub release asset size limits are not verified here and may force multi-part assets.

**M3. Workload budgets need user-adjustable ceilings.** The user said limits must not be unnecessarily restrictive. Define three profiles (interactive, job, preview), each with defaults measured on reference hardware. A per-app, per-package setting can raise memory and time up to a hard ceiling, without reinstalling. Show the budget in diagnostics when a limit is hit ("Soft Glow used 4 GB of its 4 GB limit — raise?"). Never offer "unlimited".

**M4. Development mode across release builds.** Define whether a release app can load `craft-plugin dev` packages. Recommendation: yes, as `FromFile` through an explicit "developer store" toggle in each app, with every runtime control still active. The research found gated development installs visible to release readers ([reconciliation](../research/reconciliation.md)), so readers must reapply policy at load time.

**M5. Missing CLI management design.** The draft covers CLI *use* but not CLI *management*. Recommendation: the launcher or shared management CLI offers `ext list|install|enable|grant|pin|remove`. `grant` requires the exact permission list as arguments, or an interactive confirmation on a terminal, and is never implied by `install`. This counts as explicit user consent. Whether to allow it is a user decision (section 6).

**M6. Contract deprecation needs numbers before the first community release.** The draft rightly avoids promising perpetual support. Authors still need a minimum window. Propose that each app supports at least the current and the previous major contract version, with at least one app release of overlap. Tune it before sg13.

### Low

**L1.** The draft uses "extension", "add-on", "package" and "plugin" interchangeably. Fix them once, ideally in a glossary: *package* (the installable unit), *contribution* (something one app uses), *plugin* (a package with executable contributions), *extension* (the user-facing umbrella word, following Jeremy's d2).

**L2.** The documents hedge almost every sentence. Each companion should open with three to five firm decisions before its caveats.

## 4. Simplifications and alternatives

| Proposal | Instead of | Tradeoff |
|---|---|---|
| Keep **one atomic state file** (the existing `state.json` with `generation`), extended with grants, pins and revocation cache | Splitting grants, revocations and installs into separate files or immutable generation directories | Needs a size budget (4 MB today) and compact grant records. Concurrent readers already see coherent snapshots through atomic replace. Revisit only if size is measured to be a problem |
| **One dispatch import** plus generated typed wrappers | Many per-operation host imports | One choke point. Slight decode cost per call, to be measured |
| **The Rust registry generates the inventory** | A separate IDL file maintained beside app code | Rust-first. Other-language bindings come from the inventory later |
| **Catalogue as one signed index**, TUF-wrapped | Per-package metadata files fetched individually | The size cap of 1 MB and 2000 entries in `index.rs` will need raising or sharding later. Not urgent |
| **Catalogue client in craft-extensions behind a feature** | A separate distribution-client crate or service | Web builds and headless CLIs omit network code |
| **Launcher as the only downloader**, with apps deep-linking to it for discovery (Jeremy's r3 design) | Network and TUF code in every app | Smaller attack and maintenance surface. The downside: the user needs the launcher installed, and in-app search is weaker. Recommended for the first community release. In-app install later reuses the same crate |
| **Attestation recorded as optional evidence** | A mandatory hub build farm (d9) | Weaker than a hub rebuild, but with no infrastructure cost. Honest labelling |
| **Stars snapshot generated daily by the catalogue repository's scheduled job** | Clients calling the GitHub API | Avoids rate limits and privacy leaks. Counts are up to a day old |
| **Data-only kinds go first in the community catalogue**, possibly before sg13 opens for executables | Waiting for the full executable gate for everything | Whether sg13 covers themes and resources at all is a policy question for the user and Jeremy. Section 6 |

Alternatives considered and not recommended as defaults:

- **Publisher-signed packages (minisign or Sigstore) as the main source of authenticity.** This would let local files carry continuity. It adds publisher key management for every author. Recommendation: allow it later as optional extra evidence. The catalogue digest stays authoritative.
- **Following a publisher's `latest` release automatically after the first review.** This contradicts the user's direction that people review entries, and it opens update substitution. Rejected, as in the draft.
- **Embedded web UI for plugin panels.** It would conflict with the native house rule (no webview shell) and is a separate confinement problem. Rejected for the default panel contract. The draft is right.

**Established patterns, all unverified analogies.** This session did not open primary sources for any of them; they are background only:
- VS Code and JetBrains marketplaces: one manifest with contribution points, extensions installed once and enabled per workspace.
- Figma plugins: a sandboxed main thread with an API object, plus a separate UI.
- Zed: WASM extensions built on WIT.
- Homebrew taps and Flathub: a reviewed metadata repository pointing at upstream artifacts.
- Android runtime permissions: consent at the point of use, with upgrades requiring new consent.

The only externally sourced claims this review relies on are the ones the primary draft already cites (TUF specification, GitHub releases and assets, rate limits, supply-chain documentation). This review did not re-fetch those.

## 5. Aspect coverage

"Must include" means the complete design document must specify it now. "Deferred" means implementation can be phased later, but the design still names it. The draft column marks coverage as **ok**, **partial** (named but missing a concrete mechanism) or **gap**.

| Aspect | Must include in the complete design | Deferred implementation | Draft |
|---|---|---|---|
| Vocabulary and identity | Glossary (L1), UUID plus source binding, handles, legacy alias migration, collision rules (H5) | Publisher-signed continuity for local files | partial |
| Package and SDK | Schema 2 fields, author source file, generated manifest, `craft-plugin` command set, deterministic pack, SBOM, old-client rejection (unknown critical field fails) | Non-Rust languages | partial: no concrete author file or commands |
| Compatibility | Separate axes: package schema, contract version, engine profile, app version, OS and architecture, backend, state schema. Resolver outcomes: supported, partial, unsupported, malformed | Hardware feature probing | ok |
| Dependencies | Compiled-in dependencies plus SBOM. Remote data resources (M2). Plugin-to-plugin dependencies deferred, with that decision recorded | Inter-plugin APIs | partial |
| Catalogue | Repository layout, record schema, PR validation, publication, star snapshot, search and filter order, labels | Sharding past 2000 entries | partial: no record schema |
| Authenticity | TUF roles, root shipped in the app, key custody owner, rotation drill, same-version changes rejected (C4) | Threshold tuning, hardware keys | partial |
| Provenance | Attestation as optional evidence, source commit record, honest label | Reproducible-build verification | ok after adopting 1.2(11) |
| Installation | Verify before unpacking (already implemented in `unpack.rs`), staging, locking, interruption | Streaming install for large resources | ok |
| Activation | No auto-enable (C1). One resolver shared by GUI, CLI, load and enable. Quarantine | — | partial: code contradicts it |
| Updates and removal | Multi-release store (H4), pins, rollback, grant-reuse rule (C2), deferred collection | Disk management UI polish | partial |
| Offline and time | Cached policy, stale labels, rollback detection, known revocations | Offline import of a signed metadata bundle | ok |
| Proprietary packages | Options in section 6, a restricted-release credential broker, a statement that code cannot be kept confidential | Entitlements and paid licensing (not selected) | partial |
| Operations | Disclosure, takedown, appeal, fast lane (C3), incident roles, revocation types (H3) | Incident drills | partial |
| Sandbox | Platform enforcement table (present), single dispatch choke point (H1), load-time re-admission | Native and GPU backends | ok |
| Capability grants | Vocabulary (2.4), grant key, subset reuse, CLI grant (M5), mid-run revocation behaviour | Fine-grained, time-limited grants | partial |
| Resource budgets | Profiles, user-raisable ceilings, diagnostics (M3) | Measured numeric defaults | partial |
| Documents and undo | Proposal and commit through the app transaction, stale revision, reentrancy ban | — | ok |
| Persistent features | Exact digest reference, state schema, explicit upgrade, bake, missing-release dialog | Collaboration merge semantics | ok |
| Media processing | Per-contract data shapes and conversion fixtures | Additional colour models | ok |
| UI contributions | Declarative panel schema version, canvas draw lists, tool gesture contract, missing families (M1) | Custom shader backend | partial |
| CLI and automation | Use path (present). Management CLI (M5). Exit codes and artifacts | Persistent automation server authentication | partial |
| GPU and native backends | Contract and "unavailable until proven" rule | Platform confinement experiments | ok |
| Audio | Dedicated contract outline | SoundCraft runtime choice | ok as deferred |
| Caches and scheduling | Cache keys include release digest, parameters and context. Reset contract | Pooling optimizations | ok |
| Themes | ThemeFile plus app mapping, explicit activation, fallback | ThemeCraft round trip | ok |
| Resources and workflows | Typed kinds, importer versions, no code run at install, separate workflow executor permission | ComfyUI execution adapter | ok |
| Diagnostics | Error taxonomy (present), per-app history, redaction, safe start, budget-hit messages | Support-bundle export | ok |

Delivery and ownership rows from the register: an adapter checklist exists ([app adapters](../research/app-adapters.md)). Evidence for first-party and community releases is kept separate in the draft. sg13 stays closed. Still missing: the rollout relationship between the PhotoCraft invert and theme slice and the EffectCraft and FigCraft pilots, and the order of the store changes in C1, H4 and H5 relative to the UUID migration. Both belong in the unwritten delivery document. Migrate store and identity first, because every later step writes these records.

## 6. Remaining user decisions

Each item is framed so it can be answered in one line. Where this review has a recommendation it says so, but the choice belongs to the user.

1. **Catalogue licence policy.** Options:
   - (a) Package licence must be on the repository SPDX allowlist.
   - (b) Allowlist, plus a named official-proprietary class.
   - (c) Any declared licence, including proprietary community packages, shown truthfully. Bundled dependencies must still match the declared terms.
   - (d) Allowlist, plus an explicit list of additional open licences (for example MPL-2.0 or GPL-family) for community packages.

   No default is recommended, because the earlier proprietary requirement conflicts with (a).
2. **Licence policy for local files.** Options: none (show the declared licence only), or the same as the catalogue. Recommendation: none, because local install is the user's own choice. This is still the user's call.
3. **Per-release review** for every executable release, including official ones, with a security fast lane (C3)? Data-only kinds with a lighter checklist?
4. **Catalogue repository** name, owner and named maintainers. It needs at least two, for the fast lane.
5. **CLI grant command** (M5): allowed with an explicit permission list, or GUI only?
6. **Where installs happen**: launcher only at first (recommended for the smaller surface), or in-app from the start?
7. **sg13 scope**: does the community gate cover data-only kinds, or only executables?
8. **Automatic updates** for same-authority reviewed releases: on by default, off by default, or chosen at first run?
9. **User-raisable budget ceilings** (M3): acceptable, and up to what hard cap?
10. **Restricted GitHub releases**: support a credential broker for private repositories in the first community version, or defer it?

## 7. Evidence and limits

What this review read:
- The four primary draft files and the aspect register, in full.
- Research reconciliation and the app adapter matrix, in full. Research README and coverage, partially.
- Jeremy's handoff plan text, in full.
- craft-extensions source on the local `main` checkout at a6c5fae: `ext-core` `manifest.rs`, `id.rs`, `tier.rs` and `index.rs`, and `ext-store` `state.rs` and the install, uninstall and enable paths in `store.rs`.
- The diff from `main` to `origin/wip/extensions-handover` (ffd8512) for `store.rs` and `state.rs`, which confirmed the auto-enable and deletion behaviour on both branches.

What it did not do: it did not run code, builds or tests, and fetched nothing from the network. It did not open app repositories, so app claims come from the research matrix at its recorded revisions. It did not verify GitHub numeric-ID stability, asset size limits, attestation behaviour, TUF library licences, or wasmi component support. Established-system comparisons in section 4 are unverified analogies from background knowledge. Line numbers are for the revisions named and may move.

Nothing here certifies a sandbox, a review process or a PKI. Every acceptance claim remains future work, which the primary's delivery document should label as a test to write, CI-only evidence, or a completed local check.

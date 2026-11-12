# System gaps and roadmap

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: coordinator synthesis, 2026-11-08. Source-only. The coordinator read craft-extensions, craft-launcher and craft-keymap source directly, and reconciled three Haiku app audits ([media](haiku-media-apps.md), [design](haiku-design-apps.md), [domain](haiku-domain-apps.md)) that read all sixteen apps at their committed HEADs. Every HEAD equals the inventory baseline. Nothing was built, run, tested or fetched. Every acceptance item below is a future test. Revisions and spot checks are in the [evidence index](evidence-index.md).

The strongest initial work is around existing behavior: store lifecycle records, originating caller authority, truthful output and protected extension registration. Many surveyed editors have command tables and undo, and several have metered WASM. ArtCraft is an exception to a common command-registry claim, and GUI/undo coverage was not fully traced in every app. These findings justify scoped adapters and fixes rather than a universal engine rewrite. See [the root assessment](assessment-and-first-work.md) for qualifications.

## Biggest blockers, ranked

| # | Blocker | Evidence (source at named revision) | Why it ranks here |
|---|---|---|---|
| B1 | **The shared store cannot represent the promised lifecycle.** Install enables the package in every app it lists, an update re-enables apps the user turned off, and the previous release folder is deleted at once. The installed record keeps no archive digest, manifest, origin or grant. Unknown state fields are dropped when the file is rewritten | craft-extensions `ffd8512e` `ext-store/src/store.rs:215-219`, `229-231`; `state.rs:22-47`, `173-178` | Everything else (grants, pins, rollback, eligibility, CLI) writes into this record. Fix it first, with a schema bump so older readers refuse instead of silently discarding new fields |
| B2 | **A common originating caller/capability contract is missing from the inspected automation and app dispatch paths.** Some trusted/unconfined routes allow code install/helper/script operations; four inspected desktop control paths lack tokens | EffectCraft `library.install`, `effect.helpers.scan` via unconfined MCP/CLI ([media](haiku-media-apps.md#cross-app-findings-blockers-for-this-group) 1, 2, 7); VectorCraft `plugin.install` via MCP; FigCraft control `engine.execute`; CADCraft LISP via MCP. Existing file-authority and FigCraft plugin guards must be preserved | A new plugin adapter must not inherit trusted-user dispatch rights. A loopback control-channel finding is not a demonstrated remote exploit or plugin sandbox escape; authentication and authorization are separate checks |
| B3 | **Same-plugin-ID replacement and future core-registration protection need distinct handling.** PhotoCraft/VectorCraft default to replacing plugin IDs; FigCraft replacement removes settings. WordCraft's internal registry is last-wins; craft-ext-store lacks community/local source continuity | PhotoCraft `plugin_cmds.rs:205`; FigCraft `plugins.rs:462-476`; WordCraft `lib.rs:147-152`; `store.rs:196-207`, `tier.rs:61-66` | Source continuity/settings loss are current concerns. Internal duplicate handling is not evidence untrusted plugins currently replace core commands. Add namespace/core guards before exposing new contribution seams |
| B4 | **Output can silently lose processing.** EffectCraft drops unresolved effect IDs at render, and a failing plugin returns the input frame. PhotoCraft skips a missing smart filter. FigCraft's headless `save_file` reports success and the request is cleared | EffectCraft `render/src/pipeline.rs:622-632`, `effects/src/plugin.rs:684-688` (checked); PhotoCraft `plugin_cmds.rs:150-159` (checked); FigCraft `plugins.rs:315-333`, `mcp/src/headless.rs:68` | Users ship wrong exports with a success status. The fixes stand alone and do not wait on the store |
| B5 | **No authenticated catalogue.** The craft-ext index is unsigned. EffectCraft's working Plug-in Library fetches an unsigned `index.json` from `raw.githubusercontent.com/storytold/effectcraft-plugins/main`. Revocation is only "ID below version" | `ext-core/src/index.rs:1-6`, `36-43`; EffectCraft `crates/engine/src/system/library.rs:1-40` | Blocks any network install of community packages, and should block official ones too. Needs a user decision on keys and operators (W0) |
| B6 | **A fresh CLI does not load installed plugins** in EffectCraft, FigCraft or PhotoCraft. VectorCraft does, from its own folder with no digest | [media](haiku-media-apps.md#summary) and [design](haiku-design-apps.md#summary) summaries; VectorCraft `apps/vectorcraft-cli/src/main.rs:205-213` (checked) | Breaks GUI/CLI parity, which the user accepted as a default |
| B7 | **The inspected live-feature references lack exact package-release identity.** This finding is scoped to the traced formats, not every saved format in all sixteen apps | EffectCraft `project/src/props.rs:183-186`; PhotoCraft `plugin_cmds.rs:133-141`; FigCraft `doc/src/node.rs:543-545`; SoundCraft `model/src/mixer.rs:425-440` | An update may select new processing under old state. Exact references need B1's multi-release records and app-owned migrations |
| B8 | **Six separate install mechanisms already exist.** craft-ext-store (themes, launcher local files), EffectCraft Plug-in Library and its `Plug-ins` folder, FigCraft prefs list and directory server, PhotoCraft folder, VectorCraft `Plug-ins` folder, SoundCraft CLAP/VST3/AU scan | See [all sixteen apps](#all-sixteen-apps) | Converging them on one store without breaking current users is a migration project. Do not add a seventh |
| B9 | **Most apps have no contribution seam,** and none validates registry IDs at runtime. Menus come from `craft-menu` with no registration API. `craft-keymap` defaults to `Policy::Steal` | [design](haiku-design-apps.md#cross-app-blockers-for-this-group) 1, 8; [domain](haiku-domain-apps.md#cross-app-findings-this-group) F1; craft-libs `crates/keymap/src/keymap.rs:333-341` (checked) | Each app needs a small additive registration API. This is per-app work, not a shared engine |
| B10 | **Three existing extension runtimes are unconfined:** CADCraft AutoLISP (file writes to any existing parent, command bridge, `defun` overrides built-ins), SoundCraft in-process native CLAP/VST3/AU, and EffectCraft native helper processes (no sandbox, environment inherited) | CADCraft `lisp/builtins/files.rs:76-107`, `lisp/eval.rs:507-512`; SoundCraft `clap-host/src/registry.rs:54-68`; EffectCraft `helper-host/src/client.rs:390-401` | Catalogue packages must never route into these. "Compiled" and "separate process" are not confinement |
| B11 | **EffectCraft cannot unregister loaded effect slots until restart; other registries need their own lifecycle assessment.** PhotoCraft's mutable registry has explicit install/remove and shared handles | EffectCraft `effects/src/plugin.rs:393`, `474-478`, Library restart note; PhotoCraft `plugins/src/registry.rs:21-49` checked by root | Do not infer restart-only behavior from a static registry. EffectCraft needs a disclosed restart boundary initially or a scoped lifecycle redesign; every app still retains active-reader bytes/resources |

Also noted, not blockers: ArtCraft's native crate says "no webview" but depends on `wry` for a provider sign-in (`artcraft-native/Cargo.toml:3`, `:50`), which is a house-rule 4 question for its maintainers. SolidCraft's `Cargo.toml` names `bherbruck/solidcraft` while the older inventory names `tangency-labs/solidcraft`, and the remote was not read.

## What exists, what is partial, what is missing

| Status | Items |
|---|---|
| **Existing and reusable** | Command registries with snapshot undo in all sixteen apps. Bounded archive validation and atomic state writes (craft-ext-store). Index version, expiry and rollback refusal (craft-ext-core). GitHub download with resume and digest check (craft-launcher). **EffectCraft's Plug-in Library**: a reviewed `index.json` per package release with GitHub asset URL, SHA-256, size, minimum app and API version; GitHub host allowlist including redirects; cached offline catalogue; installed record with digest (`library.rs:1-40`, `46-135`). WASM sandboxes with fuel and memory limits: EffectCraft ABI v1, PhotoCraft ABI 1, VectorCraft ABI 1 (no imports in any of the three); FigCraft ABI 4 (14 host imports, one undo group per run). Caller file-authority model `Filesystem::{Denied, TrustedLocal, Workspace}` in PhotoCraft and TextureCraft. Token-authenticated control channels in PhotoCraft, FilmCraft, PDFCraft, CADCraft, SolidCraft, TextureCraft, WordCraft and GridCraft. `craft-keymap` conflict detection |
| **Partially integrated** | PhotoCraft theme adapter (branch `ecb056cc`, not merged). Launcher local-file install (branch `6ede77f`, not merged). FigCraft permission manifest: `access` read/write enforced, but `commands: [""]` matches every command (`manifest.rs:104-109`, `354-358`). Built-in precedence: EffectCraft refuses plugin effect IDs equal to built-ins; VectorCraft built-in formats resolve first; GridCraft built-in functions win by dispatch order; CADCraft typed commands beat LISP `c:` names |
| **Absent or not established** | Grants anywhere. Caller identity at dispatch. Signed catalogue. Multi-release retention. Exact project references. Fresh-CLI plugin loading (except VectorCraft). Missing-feature export refusal. Contribution seams in PDFCraft, DesignCraft, DeckCraft, LightCraft, FilmCraft, ArtCraft, WordCraft, TextureCraft, SolidCraft and GridCraft. Runtime duplicate-ID refusal in any app registry. Plugin menu placement in `craft-menu`. Any confinement for native or LISP code |
| **Proposed here** | Work packages W0 to W13 below, the catalogue in [catalogue and SDK](catalogue-and-sdk.md), and the workflows in [user and developer workflows](user-and-developer-workflows.md) |

## Core-feature protection today

The brief proposed these rules. Status per rule across the sixteen apps:

| Rule | Enforced in code today | Missing | Needs design |
|---|---|---|---|
| Contribution IDs are additive and namespaced | EffectCraft reserves `ec.` and refuses duplicates (`effects/src/plugin.rs:197`, `471-472`). VectorCraft routes plugin effects as `plugin.*` | No `ext:` namespace anywhere. PhotoCraft, FigCraft and VectorCraft replace on the same ID | Namespace format `ext:<package-uuid>/<contribution-id>` (proposed) |
| Built-in command, effect or type IDs cannot be shadowed | EffectCraft effects; VectorCraft formats; GridCraft functions by order; CADCraft typed commands; SolidCraft render executors; FigCraft plugin commands are not in the registry | WordCraft last-wins; DesignCraft, DeckCraft, CADCraft and TextureCraft first-match with no runtime check; FilmCraft encoder factories registered later are tried first (in-process); CADCraft `defun` overrides built-in LISP functions; GridCraft defined names may take function names; EffectCraft display-name lookup can return a plugin | A shared rule: registration refuses rather than reorders |
| No engine patching or unrestricted command bridge | VectorCraft and PhotoCraft plugins have no imports. FigCraft plugins call through `EngineHost::call` with a denylist | FigCraft `permissions.commands` wildcard. CADCraft `(command …)` reaches every command. EffectCraft ScriptUI panels have the scripting object model | Per-contract typed host functions with a carried plugin principal (W3, W10) |
| Contributed handlers, shortcuts and menus cannot replace defaults | No app has plugin shortcut or format-handler fields, except VectorCraft exporters | VectorCraft: two exporters on one extension, first by plugin ID wins. `craft-keymap` default `Steal`. `craft-menu` has no registration API | Extensions menu location; plugin bindings use `Policy::Refuse`; default handlers change only through settings |
| Explicit user choice is distinct from install | craft-ext-store separates theme activation from enablement (`store.rs:273-295`) | Install auto-enables (`store.rs:217-219`). VectorCraft's install persists although its spec says "for this session" (`cmd/plugin.rs:287-307`, `444`) | Disabled-on-install with an explicit per-app setup step |
| A permission declaration is not enforcement | FigCraft enforces read-only access twice (`plugins.rs:176-178`, `250-252`) | Consent passed as caller booleans: VectorCraft `allowHeavy`, FigCraft `confirm` | Grants recorded in the store and checked at load and at host call |

## Caller authority

Only PhotoCraft and TextureCraft separate a trusted local CLI caller from an automation caller. They do it with a `Filesystem` mode, and PhotoCraft re-checks nested `actions.play` steps for untrusted callers (`photocraft crates/automation/src/headless.rs:30-60`; `texturecraft crates/automation/src/headless.rs:14-24`). This is the seed of the needed caller context. It is not enough for plugins on its own, because the trusted mode deliberately skips nested checks.

Proposed minimum (W3): dispatch receives a host-derived `Caller { kind: Gui | LocalCli | Automation(session) | Plugin(principal) }` plus scopes/session policy. Each exposed operation specifies caller/argument-dependent authority. Code install, helper/script execution and grant changes are denied to plugin principals; trusted user automation requires explicit authenticated authorization. Nested/queued work preserves its origin. Start with exposed operations in current dispatchers, not a whole-registry rewrite. Existing PhotoCraft/TextureCraft modes are useful evidence, not proof that a new common type already matches across apps.

## App operation registries

Fifteen apps have a static command table with one dispatch function, and the audits found their CLI or automation paths using that same table. GUI wiring was not traced everywhere (for example PDFCraft, DesignCraft and DeckCraft). ArtCraft is the exception: a closed provider `match` and 63 Tauri handlers, with no CLI. The audits counted between about 160 (TextureCraft) and 888 (VectorCraft) registration sites per app. These are static grep counts.

So the SDK's "generated from app registries" is realistic, but not from the whole table. Recommendation:

1. Keep each app's existing `CommandSpec` table. Do not replace it.
2. Add an optional descriptor next to the commands that are offered to plugins: typed parameters and result, effect class, allowed caller kinds, required capability, headless support, transaction or job behaviour.
3. Release tooling exports only descriptor-bearing operations as `contracts/<app>/<contract>@<version>.json`. The SDK generates guest wrappers from that file.
4. For the existing pixel ABIs (EffectCraft v1, PhotoCraft 1, VectorCraft 1), the contract is the ABI itself. Describe it once as JSON so the SDK can generate wrappers, and keep the runtime unchanged.

This keeps the Rust registry authoritative, as the [draft](../draft/sdk-and-contracts.md#operation-descriptions-are-the-shared-base) proposed, without a universal command bus.

## All sixteen apps

| App | Plugin or extension mechanism today | CLI loads installed plugins | Collision status (summary) | First useful change (future) |
|---|---|---|---|---|
| EffectCraft | WASM ABI v1, native API 2, native helpers, Plug-in Library | No | Effect IDs: refused. Display names: shadowable | Refuse unresolved effects at export (W5); move the Library onto the shared store (W9) |
| PhotoCraft | WASM ABI 1 filters, folder load; theme branch | No | Replaces same ID by default | `replace:false` default (W4); invert and theme through the shared store (W11) |
| FilmCraft | None (encoder and renderer factories in-process) | n/a | Later factory preempts a built-in encoder | Theme mapping only; no processing plugins yet |
| LightCraft | Resource importers only (presets, LUT profiles, templates) | n/a | Not traced for resources | Token on the control channel (W3); later a namespaced preset pack |
| ArtCraft | None; closed provider enum | n/a (no CLI) | No seam | None now; provider-adapter design later. House-rule webview question for maintainers |
| FigCraft | WASM ABI 4, 14 imports, directory server | No | Plugin commands outside the registry; reinstall replaces and deletes settings | Digest recheck at load, wildcard fix, headless save error (W4, W5); then command contract (W10) |
| VectorCraft | WASM ABI 1 filter, live effect, exporter; no imports | Yes (folder, no digest) | Formats: built-ins first. Exporters: first plugin wins. Install replaces | `replace:false`, exporter-extension refusal, consent out of params (W4) |
| PDFCraft | None (boa runs document JavaScript only) | n/a | Unique by test | None needed now. Keep the token model as reference |
| DesignCraft | None found | n/a | First-match, no test | Runtime duplicate refusal (W4) |
| DeckCraft | None found | n/a | First-match, no test; closed export match | Runtime duplicate refusal (W4) |
| SoundCraft | Native CLAP, VST3, AU in-process, unsandboxed | Loaded lazily by name | Hosted IDs cannot shadow built-ins | Token on control channel (W3); data-only track presets; dedicated audio contract later (W13) |
| CADCraft | AutoLISP with file I/O and command bridge | Explicit `APPLOAD` only | Typed commands beat LISP. `defun` overrides built-ins | Confine LISP and protect built-in functions before any package code (W4, W13) |
| SolidCraft | WASM design document only; manifest listing | No loader | Render executors protected | Read-only plugin feature fallback, per its own design doc |
| TextureCraft | None; MaterialCraft through a subprocess | No loader | First-match, no test | Data-only material or preset import |
| WordCraft | None; Zotero bridge as external caller | No loader | Last-wins registry | Fail-closed registration (W4) |
| GridCraft | Macros with trust policy; doc-hidden runtime registrar | No loader | Built-in functions win; names may take function names | Gate `register_test_command` behind `cfg(test)` (W4); function packages are a high-risk later contract |

Per-app detail, including traced operations, persistence, jobs and data constraints, is in the three audits.

## Work packages

Owners are roles, because no people are assigned. "App maintainer" means that app's repository maintainers. Every package is future work, needs its own scope, and goes through a normal pull request in the owning repository.

| WP | Change | Owner | Depends on | Acceptance (tests to write) | Compatibility and security |
|---|---|---|---|---|---|
| **W0** | Decide: catalogue repository name and owner; licence list; reviewers per release; interim signing (Ed25519) or TUF from the start; whether "official first" applies within a relevance tier; whether EffectCraft's `effectcraft-plugins` catalogue is folded into the new one | User, maintainers | None | Recorded decisions | Nothing is built before this for the network catalogue. W1 to W5 do not wait |
| **W1** | craft-ext-store state schema 2: retain archive digest, manifest and origin (`catalogue`, `local`, `bundled`); multi-release records; install does not enable; update keeps per-app enablement; old release kept until unpinned and unused; grants table; schema bump so schema-1 readers refuse | craft-extensions | W0 not needed | Install leaves every app disabled. Update preserves disabled apps. Old release kept while pinned. Local file cannot replace a catalogue record of the same ID. Schema-1 reader returns `Newer(2)`. Crash between rename and state write leaves the previous state | Schema-1 readers (launcher branch, PhotoCraft theme branch) must update together. Migration preserves settings and disabled choices, labels inherited enablement as legacy rather than proof of consent, and infers no new grants |
| **W2** | Eligibility read API used by every loader: installed, enabled for this app, compatible, granted, not revoked, tier admitted | craft-extensions | W1 | Revoked digest not returned. Ungranted required capability not returned. Release build refuses `ReviewedThirdParty` | Replaces `Reader::installed_for` (`reader.rs:58-63`) for plugins. Themes keep working |
| **W3** | Typed originating caller context and operation-scoped authorization; preserve existing file/plugin guards; authenticate inspected desktop control paths. A denylist is only interim defense, not the final permission model | App maintainers | None | A caller without authority cannot install code, start helpers, grant rights or write outside approved scope; an explicitly authorized user automation flow still works; unauthenticated control request refused | Existing clients may need a token. Authentication does not substitute for argument/scope-aware authorization |
| **W4** | Source-aware explicit plugin replacement preserving settings; reserved core/namespaced contribution IDs; extension keymaps use refusal rather than silent steal; harden broad FigCraft grants; preserve core format resolution while selecting exporters explicitly. Audit internal duplicate/test helpers before changing their public surface | App maintainers; shared keymap support where needed | None | Cross-source takeover and core-ID registration refused; same-authority explicit upgrade preserves settings; two exporters can share an extension and be selected by unique ID; new bindings do not steal core shortcuts | Replacement defaults and permission tightening can affect existing scripts. GridCraft's helper is used by cross-crate integration tests, so plain `cfg(test)` would break a real consumer; no hostile invocation path is established |
| **W5** | Truthful output: required unresolved/failed processing prevents successful final output; preserve labelled preview fallback. PhotoCraft reports missing/failed smart filters; FigCraft headless file output has an explicit destination or structured error | App maintainers | None | Required missing-effect export exits non-zero naming it; headless offered-file output completes or fails visibly; cancelled work leaves no falsely complete artifact | Source indicates omission/fallback paths; final output/exit behavior still needs reproduction. An optional explicit skip/bake policy is a separate product choice, not an already selected flag |
| **W6** | Distribution client: extract the launcher's verified download into a shared crate; catalogue schema 2 merging craft-ext index and EffectCraft Library fields; signature check | craft-launcher, craft-extensions | W0 (signing), W1 | Mismatched digest refused. GitHub digest differing from catalogue refused. Expired metadata blocks new installs but not existing use. Redirect to a non-GitHub host refused | Needs two consumers (launcher and in-app manager). The catalogue digest is the only authority |
| **W7** | Catalogue repository, bot and protected publication workflow | Catalogue maintainers | W0, W6 | Fixture PR with wrong digest fails. Approved PR produces signed metadata | Not authorized now. New repository and keys are user decisions. CI follows craft-common-actions |
| **W8** | SDK first cut: `craft-ext create/check/pack/dev` on the shared validator; contracts for existing pixel ABIs (EffectCraft v1, PhotoCraft 1) | craft-extensions | W1 | Packed fixture installs through the store. Module with an undeclared import fails `check` | Schema 2 manifest. Schema 1 packages keep empty permissions |
| **W9** | EffectCraft functional proof part 1: Library onto shared store and W2 eligibility; fresh CLI loads eligible effects; `GroupKind::Effect` stores package UUID and digest with alias fallback | EffectCraft maintainers | W1, W2, W5 | GUI and CLI renders match. Missing release refuses export. Existing `installed.json` packages migrate with their digests | Saved projects keep opening through the alias. Restart still needed to unload (B11) |
| **W10** | FigCraft functional proof part 2: plugin calls through policy dispatch with `Plugin(principal)`; grants from the store; digest checked at load; registry descriptors for the offered commands | FigCraft maintainers | W2, W3, W4 | Denied command refused through nested calls. Plugin run is one undo step and rolls back on failure. Headless output completes or errors | Existing FigCraft plugins need a manifest migration from `permissions` to grants |
| **W11** | PhotoCraft first-party distribution proof: merge the theme adapter and invert filter through the shared store, install disabled, explicit enable | PhotoCraft maintainers | W1, W2 | Theme and invert install, enable, run, remove, reinstall | Proves the store and distribution path only. ABI 1 has no host imports, so it says nothing about grants or commands |
| **W12** | Add/export descriptors only for the first exposed operations and existing ABIs; grow with SDK and EffectCraft/FigCraft adapters | App maintainers | W3 for privileged operations; ABI descriptions can start independently | Generated contract JSON matches authoritative Rust registry; typed guest calls and CLI discovery agree | Additive. No registry replacement or up-front rewrite of all sixteen command tables |
| **W13** | Confinement design for native helpers, SoundCraft hosted plugins, CADCraft LISP and any GPU path | Maintainers plus a security reviewer | None | Per-platform enforcement table with evidence | Until then catalogue packages cannot target these backends |

### Order

```text
now, independent:   W3   W4   W5            (app fixes, standalone value)
store:              W1 ─▶ W2
                     │     ├─▶ W11 PhotoCraft distribution proof
                     │     ├─▶ W9  EffectCraft functional proof (after W5)
                     │     └─▶ W10 FigCraft functional proof (after W3, W4)
                     └─▶ W8 SDK
decisions:          W0 ─▶ W6 ─▶ W7 catalogue
alongside SDK:      W12 descriptors for first exposed operations
separate proof:     W13 confinement before those backends become available
```

## PhotoCraft reuse versus EffectCraft-then-FigCraft

There is no conflict once each proof answers a different question.

- **PhotoCraft (W11)** answers whether a real package can go from a file or catalogue through the shared store into an app, disabled, then explicitly enabled, run and removed. Its branch already exists and its ABI is the simplest (no imports). It can go first as the store's first consumer.
- **EffectCraft (W9)** answers whether live features survive save, reopen, CLI and export truthfully, and whether an existing GitHub catalogue can move onto the shared store. EffectCraft already runs the closest thing to the target catalogue, so it is the natural first network consumer.
- **FigCraft (W10)** answers whether plugins can call app commands and edit documents under explicit grants, with caller authority preserved and undo intact. It is the only app with a non-trivial host-call surface (ABI 4).

PhotoCraft no-import ABI cannot establish privileged host-call grants. Its slice also proves real filter output and theme/store integration. The accepted expanded functional proof order remains EffectCraft then FigCraft. A local-file vertical slice may precede the catalogue; full authenticated network acceptance additionally depends on W6/W7 and must be reported separately.

## Disagreements and qualifications of worker findings

| Finding | Qualification |
|---|---|
| Design audit: "FigCraft's plugin policy can be bypassed" through `engine.execute` | The plugin denylist applies to plugin calls. The control channel is a different caller: any local process. The real finding is an unauthenticated local channel with full authority (B2), not a plugin escape |
| Media audit: EffectCraft has a "Plug-in Library" | The audit lists it, but its significance is larger. It already implements reviewed per-release digests from GitHub assets. It should seed the catalogue design, not be replaced by it |
| Domain audit: "no Craft package model exists in any app" | True for the six domain apps. EffectCraft, FigCraft, PhotoCraft and VectorCraft have manifests and digests of varying strength |
| Operation counts | Static grep counts. Lower bounds in some apps (EffectCraft says "~2,000 ids" in a comment). Not compiled or de-duplicated |
| Fresh-CLI plugin loading | Source reading only. Not run |

## Coverage of the twenty-seven aspects

| Aspect | Where it is covered here | Status in this phase |
|---|---|---|
| Vocabulary and identity | [Catalogue: identity migration](catalogue-and-sdk.md#identity-migration); B3, W1 | Proposal |
| Package and SDK | [SDK](catalogue-and-sdk.md#sdk-generated-from-app-registries); W8 | Proposal |
| Compatibility | [SDK compatibility](catalogue-and-sdk.md#compatibility); W2 | Proposal |
| Dependencies | Deferred: no executable inter-plugin dependencies in the first version. Licence inventory in release records | Deferred with reason |
| Catalogue | [Catalogue repository](catalogue-and-sdk.md#catalogue-repository); W6, W7 | Proposal, needs W0 |
| Authenticity | [Authenticating downloads](catalogue-and-sdk.md#authenticating-what-the-client-downloads); B5 | Proposal, needs W0 |
| Provenance | Optional attestation field in release records; no build farm | Proposal |
| Installation | B1, W1 | Source gap confirmed |
| Activation | W1, W2; [core-feature protection](#core-feature-protection-today) | Source gap confirmed |
| Updates and removal | [Updates and recovery](catalogue-and-sdk.md#updates-offline-use-recovery-and-credentials); B1, B11 | Source gap confirmed |
| Offline and time | [Updates and recovery](catalogue-and-sdk.md#updates-offline-use-recovery-and-credentials) | Proposal |
| Proprietary packages | Restricted-release credential row in the catalogue doc; licence policy open (W0) | Open decision |
| Operations | [Maintainer workflow](user-and-developer-workflows.md#catalogue-maintainer) | Proposal |
| Sandbox | [What exists](#what-exists-what-is-partial-what-is-missing); B10, W13 | Existing WASM sandboxes; native unconfined |
| Capability grants | B2, W1, W3, W10 | Shared app/source-scoped grant model absent; existing FigCraft access/command checks and file-authority modes partly enforce policy |
| Resource budgets | Existing fuel and memory limits (FigCraft `sandbox.rs:68-100`); measured profiles deferred to the [draft](../draft/delivery-and-acceptance.md#workload-profiles-and-resource-readiness) | Partly existing |
| Documents and undo | App-owned commands/transactions in surveyed editors; FigCraft one group per plugin run | Existing in inspected paths; not verified uniform across ArtCraft or every app workflow |
| Persistent features | B7, W9 | Source gap confirmed |
| Media processing | Pixel ABIs and band overlap ([media](haiku-media-apps.md#cross-app-findings-blockers-for-this-group) 8) | Existing per app; no common contract proposed |
| UI contributions | B9; Extensions menu location; FigCraft panels and widgets | Mostly absent |
| CLI and automation | B2, B6, W3, W5 | Source gap confirmed |
| GPU and native backends | No plugin GPU path in any app; TextureCraft and FilmCraft use GPU for built-ins; W13 | Deferred with reason |
| Audio | SoundCraft real-time constraints ([domain](haiku-domain-apps.md#cross-app-findings-this-group) F7); W13 | Deferred with reason |
| Caches and scheduling | EffectCraft has no cache keyed by plugin output; digest must join any future cache key (media audit item 9) | Deferred to W9 |
| Themes | W11; craft-ext-store theme activation already separate | Partly existing |
| Resources and workflows | LightCraft, CADCraft, TextureCraft and SoundCraft importers as data-only first candidates | Proposal |
| Diagnostics | B4, W5; [user problem table](user-and-developer-workflows.md#problems-and-recovery) | Source gap confirmed |

## Decisions still open

These are the user's or maintainers' to make. Nothing above assumes an answer.

1. Catalogue repository name, owner and maintainers, and whether `storytold/effectcraft-plugins` becomes its seed.
2. Established update tooling and operational/key ownership. Root recommendation remains TUF-class tooling for production; custom interim signing is an unselected alternative, not a required extra phase.
3. Catalogue licence list, and the rules for official proprietary and local packages.
4. Reviewers per executable release.
5. Whether "official first" applies within each relevance tier (proposed) or across all results.
6. Default for automatic updates with unchanged permissions.
7. Whether CLI grant management may be non-interactive.
8. ArtCraft's `wry` dependency under house rule 4, for its maintainers.

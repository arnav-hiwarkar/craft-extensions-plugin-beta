# Security review of the existing Craft Extensions code

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Written on 2026-11-08 by the Opus design pass. It reviews the code at `storytold/craft-extensions` `wip/extensions-handover`, revision [`ffd8512e`](https://github.com/storytold/craft-extensions/tree/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943). `origin/main` is `a6c5fae8`, which is `33adcb4` plus a README edit. Everything here comes from reading the source with `git show`. Nothing was built or tested. Scope is themes and plug-ins, community and proprietary. Organisation and team management are out of scope.

In what follows, **Code** means I read it at that revision, **Unknown** means the source does not settle it, and **Proposed** means it is my recommendation and nobody has decided it.

The primary researcher qualified the findings below during synthesis. In particular, caller-controlled provenance is a planned host-boundary requirement rather than a demonstrated remote exploit, and absence of an executor does not make hostile package parsing safe.

## 1. The handoff's facts, checked against the source

| Handoff claim | Code |
|---|---|
| Third party is switched off | True, in part. [`Policy::admit`](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-core/src/tier.rs#L52-L58) refuses `ReviewedThirdParty` unless `cfg!(feature = "third-party")`. Sections 2 and 3 cover the parts it does not close. |
| sg1 reserved namespaces built | True. There are six [`FIRST_PARTY_PREFIXES`](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-core/src/id.rs#L35-L36) and `check_id` runs both at install and in the index. Ids use a lowercase ASCII alphabet, which blocks homoglyphs. |
| sg5 tier gate built, release-profile proof still to add | True. The [guard test](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-core/src/lib.rs#L23-L42) reads only ext-core's own `Cargo.toml`. It cannot see a downstream crate turning the feature on. The effective feature configuration of each actual release build therefore needs behavioral verification. This does not change separately built apps. |
| sg2 signatures not started | True. No crate has a signature type, key type or crypto dependency. |
| sg3 expiry and monotonic version partly built | True, with a gap. [`Index::parse`](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-core/src/index.rs#L114-L148) refuses an index that has expired, and refuses `version < newest_seen`. A different index with the **same** version number is accepted. Without signatures none of this proves anything about who wrote the index. |
| sg4 revocation helper exists but is not wired in | True. `is_revoked` has no callers. [`install_staged`](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-store/src/store.rs#L174-L233) and [`set_enabled`](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-store/src/store.rs#L253-L270) never check revocation. |
| sg6 empty permissions, no imports | Only half built. The manifest refuses a non-empty [`permissions`](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-core/src/manifest.rs#L181-L182) list. Nothing parses `plugin.wasm`. The store treats it as opaque bytes checked against a SHA-256, so no code rejects imports. No host exists in this repo. |
| sg13 closed | True. [`plan_tasks.py` L56](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/docs/handoff/plan_tasks.py#L56) requires sg1 to sg12 and a yes from both Jeremy and the maintainer. |

The pasted rows left out sg1 and sg5. Their actual text is at [plan_tasks.py L44 and L48](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/docs/handoff/plan_tasks.py#L44-L48). sg1 says "Built … more lookalike cases wait for the threat model review". sg5 says "Built … release-profile proof still to add".

**Plan facts that are out of date:**
- Task `c1` says the manifest has "tier, signature list" fields. The [`Manifest`](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-core/src/manifest.rs#L83-L101) type has neither. Leaving out the tier is deliberate and correct. The signature list does not exist yet.
- The workspace [`repository`](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/Cargo.toml#L10) still points at `Jeremy8776`. The fix sits unmerged on `chore/repo-url-storytold`.
- NEXT.md gives the handover head as `183c506`. The branch has since moved to `ffd8512`, with two documentation commits added.
- The manifest has no licence field. No code anywhere restricts running proprietary plug-ins or ties them to a licence.

## 2. How the gate behaves today, by path

**Install from a file (Code).** `Tier::FromFile` is admitted by the default policy. A community plug-in or theme with an id outside the reserved prefixes can be installed without signature verification and is enabled for apps listed in its `compat`. The closed tier is specifically `ReviewedThirdParty`, not all externally authored files. This repository has no plugin executor, but it still parses hostile archives and assets. Neither that absence nor the empty-permissions rule establishes safety. App-side module validation must cover file installs too.

**Catalogue (Code).** No fetch, download or install-from-index path exists. Tasks n1 to n3 are "Not started" or "Blocked on d4". Nothing connects the `sha256` in an `IndexEntry` to `install_zip`: the store checks each file against the hashes in the manifest, not the archive against the index.

**Provenance boundary (Code).** `install_zip(bytes, tier)` is public and takes its tier from trusted host code. The host is therefore responsible for deriving the tier from verified provenance rather than a package claim or an unsigned catalog field. This is not a demonstrated remote bypass: a compromised host already has the user's authority. A validated-artifact interface could make accidental trust escalation by a future catalog adapter harder.

**Tier provenance (Code).** A *package* cannot claim a tier, which is true. An *index entry* does carry a [`tier`](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-core/src/index.rs#L28) field that whoever serves the index chooses. `check_id` stops a non-first-party entry from using a reserved id. It does not stop an entry under an ordinary id from calling itself `first-party`. While the index is unsigned, "first party from the catalogue" means "whatever the server says". The planned control is the download rule in n5, which only allows storytold-hosted release URLs. It is not built.

**Checksum versus signature (Code and threat model).** SHA-256 pinning shows that a file matches what the index or manifest says. It says nothing about who made the file. The [threat model](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/docs/THREAT-MODEL.md) says the same in its "Not claimed" section. A hash inside a manifest proves nothing about the person who wrote that manifest.

**Upgrade (Code).** Only two upgrade cases are refused: a downgrade, and a non-first-party package replacing a first-party one. A `FromFile` package can replace another `FromFile` package with the same id from an unrelated author. The second author silently takes over the first one's id (T14 and sg7 are not built).

## 3. What is actually built

These are built and covered by tests in the source: bounded unpacking (T1, T2, T3), validation of display text and ids (T4, T5), palette contrast at install and on read (T7), atomic state with a generation counter, namespace checks, the feature-gated tier check, and parsing of index schema, expiry and age. Nothing else in the security column is built: signatures, signed metadata, revocation enforcement, module validation, the sandbox host, fuzzing, provenance policy, the review process, the third-party UI and outside review. The 116 tests the handoff reports were not re-run here.

## 4. Proposed minimum distribution design

Everything in this section is **Proposed**. It does not grant a licensing exception or a sandbox exemption to anyone.

1. **Three channels, one validator and one sandbox.** The channels are first-party (ArtCraft public and ArtCraft proprietary), reviewed community through the catalogue (sg13 still decides when this opens), and file sideloading. They differ only in whose identity is checked and which namespace they may use. Proprietary first-party plug-ins run in the same sandbox, are bound by the same declared capabilities and pass the same validation. Being proprietary affects how a plug-in is distributed and licensed. It does not give it any extra privilege.
2. **Bind admission to verified provenance.** Use a validated-artifact interface rather than letting external metadata become a trusted tier parameter directly. A verifier-produced token can bind source identity, the approved publisher namespace, and artifact digest. This reduces integration mistakes; it is not protection from malicious code already running inside the trusted host. No such interface was implemented in this pass.
3. **Authenticate first-party metadata too.** First-party catalogue entries should also require a signed index (d8), not only community ones. Use a vetted implementation and follow an established update protocol rather than hand-rolling crypto. Check any dependency against the repository licence allowlist; this study adds none and creates no additional dependency-approval rule.
4. **Fix equal-version replay.** Refuse an index with the same version unless its bytes, or its signed hash, are identical to the one already accepted.
5. **Wire in revocation** at launcher start, `set_enabled`, install, and app eligibility checks (sg4). A theme can fall back to a built-in appearance. Executable project dependencies need a visible revoked/unavailable result rather than silent absence in final exports.
6. **Preserve sideloaded identity continuity** (sg7). A verified publisher key can establish continuity. A manifest hash identifies content, not its author, and changes on ordinary updates. Unsigned development packages need an explicit replacement policy rather than a false verified-publisher claim.
7. **Proprietary licence enforcement stays undecided and out of the trust core.** If ArtCraft wants entitlement checks, they belong above the store as a separate policy layer. They must not widen capabilities or weaken validation, and they need their own decision, plus a privacy review if they make network calls.

## 5. EffectCraft and FigCraft capability needs versus the empty-permission v1

The user has accepted official plus third-party as the **architecture**. That decision does not open the existing third-party **release gate**, and nobody has approved shipping. Both are true at once: the third-party structure gets built now, and sg13 stays closed.

**Proposed path that bypasses no gate:**
- Keep `permissions` empty for any `Tier` that a release build admits until plan task `plc-1` lands. `plc-1` says each ability is "granted only when the sandbox can enforce it".
- Add capabilities as a versioned, closed enumeration in `craft-ext-core`. Each capability ships only together with three things: enforcement in the host for that capability, a fuzz target (sg8), and a test that a plug-in which did not declare it gets refused. This replaces the `PermissionsNotEmpty` blanket refusal one capability at a time, rather than removing it all at once.
- **First-party plug-ins do not get a shortcut.** A capability-rich EffectCraft or FigCraft plug-in made by ArtCraft goes through the same capability check. Its first-party status only allows it to use reserved ids. If the existing rule makes first-party plug-ins unable to declare capabilities, the fix is to build the capability, not to exempt the first-party tier.
- Module validation (sg6) goes into a shared crate, not into each app, before any host loads a store-installed module. It checks that every import is one the granted capabilities allow and checks the declared interface version (pl2). It runs at install and again on every load.
- Grants are made per app. The person sees them before install, and any widening triggers sg7. The CLI's automatic loading only selects plug-ins that are already installed, enabled and granted. It never grants anything.

## 6. Security decisions to make next

1. Who owns the trust core: does `craft-ext-core` and `craft-ext-store` stay in craft-extensions (d7), or move to craft-libs once EffectCraft is the second consumer?
2. d8 signing scheme, and whether a signed index is required for first-party entries too. I recommend yes.
3. d4 catalogue location, and the exact n5 URL rule.
4. Whether to define the validated-provenance boundary before integrating catalogue downloads. I recommend yes.
5. Whether sideloaded plug-ins keep the same capabilities as catalogue plug-ins, or get fewer once capabilities exist.
6. The first EffectCraft capability to enforce, and which crate owns validating wasm modules.
7. d9 and d10, provenance and outside review. Both are needed before anyone even requests sg13.
8. Whether licence enforcement for proprietary plug-ins is a requirement at all. No code assumes it.

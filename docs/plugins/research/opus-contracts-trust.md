# Contracts, lifecycle and trust: an independent challenge

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Written 2026-11-08 by the Opus 5.5 research delegate. This is a design note. It contains no code and no approved decisions. It works only with themes and plug-ins. Organization and team management are excluded. The third-party release gate (G13 / sg13) stays closed throughout.

## How to read the evidence labels

- **Code** means I read the source at the named revision with `git show`. It shows what the text says, not how it behaves at runtime. Nothing was built, tested, fuzzed or run.
- **Baseline** means the claim comes from an existing document in this worktree. The document is named, and I did not re-check the claim.
- **Proposed** marks a contract sketch. No one has agreed to it.
- **Recommendation** marks my preferred option. It is not a decision.
- **Unknown** means the inspected source does not settle the question.
- **Test later** names acceptance evidence that does not exist yet.

Source anchors:

| Repository | Revision | What I read |
|---|---|---|
| craft-extensions | `ffd8512e5a3755637f58cf31cfcf2fdb09cfb943` | `ext-core`: `id`, `manifest`, `tier`, `index`, `lib`. `ext-store`: `store`, `state`, `unpack`, `reader`, `location`. Also `docs/THREAT-MODEL.md`, `docs/handoff/plan_tasks.py` and the three sample zips |
| craft-launcher | `6ede77f93fd0d3eaaad43d430f94d703d3564a51` | A grep for store calls, mainly `apps/artcraft-launcher/src/extensions.rs` |
| photocraft | `ecb056cc742848d78f9076893a6b3343e6a75010` | A grep for how it depends on craft-ext |

I did not open EffectCraft or FigCraft source again. Claims about them come from [the source record](../source-and-verification-record.md) and [the functional model](../../plugin-functional-model.md), with those documents' limits.

The [earlier Opus handoff review](../../plugin-security-handoff-opus.md) already covers several issues: equal-version index replay, `is_revoked` having no callers, the guard test reading only one `Cargo.toml`, and caller-supplied tier. I do not repeat them here except where a new finding changes them.

## Ranked objections

These are ranked by how much they would cost if left until after the shared contract is frozen.

1. **The store does not remember what it verified.** Installed records hold no digest, source or publisher. The manifest itself is not written to disk. Checking modules "at install and on every load" (G6), and checking continuity on update (G7), therefore have nothing stored to compare against. (§3, §6)
2. **A build with the feature on can leak third-party records into release apps through the shared store.** The tier gate runs only when installing. The read side re-admits nothing. Every build uses the same store folder. (§5)
3. **An update deletes the previous version's files right away, and readers hold no lease.** A running EffectCraft or FigCraft job, or a fresh CLI export, can lose its bytes partway through. EffectCraft's baseline already omits unresolved effects, so the result could be a silently wrong render. (§8)
4. **An update silently re-enables an extension the person disabled,** and enables it for any app the new version adds. That is harmless for themes today. Once capabilities exist, it would become a grant leak. (§4, §8)
5. **Extending the state file is lossy.** Unknown fields are accepted on read and dropped on the next write. Adding grants, sources or revocation caches under state schema 1 would let an older writer erase them. A schema bump would make older apps show nothing. (§6)
6. **Identity continuity protects only the first-party tier.** A file install can replace a catalogue third-party package with the same ID and a higher version, and the reverse also works. Bare IDs are global across sources. (§3)
7. **No path exists to install first-party packages.** The launcher only installs from files, and that path refuses reserved IDs. The PhotoCraft invert pilot therefore depends on a catalogue path (n1, n3, d4) that has no authentication beyond HTTPS and a URL rule. (§4)
8. **A shared WebAssembly admission check can diverge from each app's engine settings.** The admission check has to be tied to the engine settings the host actually uses. (§5)
9. **Two documents disagree about who owns what.** [The lifecycle review §20](../../plugin-lifecycle-review.md) assigns store transactions and grants to craft-libs. [The README](../README.md) and [the security design §2](../../plugin-security-design.md) say to reuse craft-extensions. Lifecycle review §9 suggests a separate portable store, while `location.rs` records a decision that the portable launcher shares the store. The precedence order resolves both conflicts today. They should be marked superseded so they are not reopened by accident. (§7)

Everything below the line on the store comes from reading source. None of these has been reproduced.

---

## 1. Shared contracts: what exists and what is missing

**Code.** `Manifest` (manifest.rs, lines 82–101) has one `kind` per package. `compat` is a map from `AppId` to `{min_app, interface: Option<u32>}`. It also has `files` with SHA-256 and size, `permissions: Vec<String>` (which must be empty, lines 181–183), and an optional `provenance {source, commit}`. A plug-in package is exactly `plugin.wasm` plus an optional `thumbnail.png` (lines 242–252). `interface` only needs to be at least 1 (line 201). It has no upper bound, no `max_app`, and nothing per contribution.

Gaps in the contract, judged against the accepted functional defaults:

| Accepted default | What schema 1 can express | Gap |
|---|---|---|
| One package holds several app-specific contributions | One kind, one module, one interface number per app | No contribution list, so EffectCraft and FigCraft entries cannot be described separately |
| Preserve existing app interfaces through adapters | One `u32` per app | That number cannot tell EffectCraft's effect interface apart from a FigCraft command contract within the same app |
| Fresh CLI uses compatible installed plug-ins | `min_app` only | No upper bound, and no way to state the runtime engine or interface range. Eligibility cannot say "too new" |
| Capability declarations | Empty list enforced | Correct for v1. See §5 for the upgrade path |

**Proposed: the schema 2 manifest. The names are illustrative.**

```text
schema: 2
id, version, publisher {name}, licence (SPDX)
requires: ["contributions", "capabilities"]      # critical features: host must understand all or refuse
contributions: [
  { id: "blur-plus", app: "effectcraft", contract: "effectcraft.effect", contract_range: "1..=1",
    module: "effectcraft/blur.wasm", capabilities: [] },
  { id: "align-selection", app: "figcraft", contract: "figcraft.command", contract_range: "1..=1",
    module: "figcraft/align.wasm", capabilities: ["figcraft.selection.read", "figcraft.document.edit"],
    ui: "headless" | "panel" }
]
theme: optional ThemeFile path (data-only contribution)
files: [{path, sha256, size}]                     # unchanged semantics
optional presentation fields (description, thumbnails, links): ignorable
```

The `requires` list works like a "critical extension" marker. A host refuses a manifest that lists any feature it does not implement. Presentation fields may be ignored. This puts the "security-critical fields cannot be silently ignored" rule from security design §8 into a mechanical form.

- **Normal path.** A FigCraft build that knows `figcraft.command` version 1 resolves only `align-selection`. EffectCraft resolves only `blur-plus`.
- **Adverse path.** A host that understands `contributions` but not `capabilities` refuses the whole package. It does not load only the parts without capabilities. The package then appears as "needs a newer app".
- **Owner.** craft-ext-core owns the schema and the parser. Each app owns its contract names and version ranges.
- **Compatibility.** Schema 1 hosts already refuse anything where `schema != 1` (line 167). That fails closed. One consequence is that a schema 2 package cannot be installed by a launcher that only knows schema 1. A catalogue that wants to serve both kinds of client during migration has to publish separate artifacts, or else require a minimum launcher version. **Recommendation:** require a minimum launcher version. Nothing has shipped yet, according to the handoff's delivery tasks, which say no candidate has been frozen.
- **Performance.** This is negligible. The manifest is capped at 64 KiB.
- **Test later.** Each of these must be refused: a schema 2 package with an unknown `requires` entry, a contribution whose contract range is outside the host's range, and a module path not listed in `files`.

## 2. Lifecycle records: one vocabulary, one eligibility function

The baseline uses several separate words: installed, enabled, compatible, permitted, quarantined, revoked and active. The store models only *installed* (`state.installed`), *enabled per app* (`state.enabled`) and *active theme per app*. **Code:** `Reader::installed_for` (reader.rs, lines 58–66) filters by kind, app listed in the package, enabled, and the folder existing. It does not check compatibility, revocation, tier admission or grants.

**Proposed: a single resolver used everywhere.** The GUI, the fresh CLI, project reopen, the enable action and loading must all call it.

```text
resolve(app: AppId, app_version, host_contracts, contribution_ref, ctx) -> Result<Resolved, Ineligible>
Ineligible = NotInstalled | Disabled | NotForThisApp | IncompatibleApp{need} | IncompatibleContract{need}
           | TierNotAdmittedInThisBuild | Revoked{reason, severity} | MissingGrant{caps}
           | FilesMissingOrChanged | Quarantined{reason}
Resolved   = { package_digest, contribution, module_path, granted_caps, pinned_at_generation }
```

- **Owner.** craft-ext-store or a small resolver module next to it. Apps supply `host_contracts` and their version. Apps never compute eligibility themselves.
- **Default.** Ineligibility always comes back as a structured error. The GUI shows it in plain language, and the CLI prints it and exits with a non-zero status. A missing render dependency never becomes a silent omission. This is the gap in EffectCraft's baseline.
- **Example.** A fresh `effectcraft export project.ecproj` resolves each saved effect. If `blur-plus` is revoked, the result is `Revoked{severity: block_load}` and the export fails with that reason. If the person disabled it in EffectCraft, the result is `Disabled`. The CLI does not enable it.
- **Test later.** A table-driven test runs the same fixtures through every entry point. GUI, CLI and project-open must give identical results.

## 3. Identity and source trust

**Code facts:**

- `ExtensionId` is lowercase ASCII, dotted, with at most 64 characters (id.rs, lines 44–65). Six reserved first-party prefixes exist (lines 35–36). The reservation is checked only for non-first-party tiers (tier.rs, lines 61–66).
- `Installed` (state.rs, lines 23–35) records `id, version, kind, name, tier, apps, thumbnail`. It has **no digest, no source, no publisher key and no requested permissions**.
- `unpack_into` writes only `manifest.files` to disk (unpack.rs, lines 135–172). The manifest is not one of those files, so **the verified manifest is never kept**.
- The only continuity rule on replacement is that a non-first-party package may not replace a first-party one (store.rs, lines 197–199). Any other same-ID install with a higher version replaces the existing one whatever its tier.
- The launcher installs only with `Tier::FromFile` (launcher extensions.rs, line 79).

**Adverse paths that follow (source-derived):**

- After the third-party gate opens, a person installs `org.example.tone` 1.0 from community source A. They then open a downloaded file containing `org.example.tone` 9.0. The store replaces A's package without any continuity check. The reverse case also works: a later catalogue update from source B replaces a package installed from a file.
- Updates are checked for downgrade, but not for publisher change or authority change. G7 cannot be built on the current record because there is nothing to compare against.
- G6 says modules are re-validated on every load, but that load has no stored digest to compare with. Hashing the file again only shows that the bytes are the bytes that are there now.

**Proposed: the identity model.**

- **Package key** = `(SourceId, ExtensionId)`.
  - For a catalogue source, `SourceId` is the fingerprint of the source's trusted root key, not its URL.
  - For a local file, it is `Local`.
  - For a signed local file, it is the fingerprint of the publisher key that signed the release.
  - A bare `ExtensionId` is unique within the store only for display. Installing the same ID from a different `SourceId` is refused with a "came from somewhere else" error. The person must explicitly choose to replace it, and that choice is recorded.
- **Release identity** = the SHA-256 of the exact canonical manifest bytes. The manifest already binds each file's digest. **Recommendation:** store packages by release digest (`packages/<id>/<release-digest>/`) and keep the verified manifest beside the files. Version stays a display and ordering field.
- **Install record (proposed)** = `{id, version, release_digest, source: SourceId, publisher_key: Option<Fingerprint>, tier_label, requested_caps, contributions, installed_at_generation}`.
- **Tier** becomes a label derived from `SourceId` and the build's policy. It is no longer a parameter the caller supplies. This uses the validated-artifact handle proposed in security design §4 and the earlier review §4.2.

Same-user tampering is threat T20 in the threat model, which accepts it. The store state is only as trustworthy as the person's user account. Stored tier labels and grants record what the person chose. They are not attestations. On load, readers still check the bytes against the stored digest.

- **Owner.** craft-ext-core owns the types and craft-ext-store owns the records. The launcher's source manager produces the verified handles.
- **Compatibility.** The record format changes. See §6.
- **Performance.** Hashing a module of up to 32 MiB at load costs single-digit milliseconds of SHA-256 work on current desktop hardware. That figure is an estimate and has not been measured. The module bytes are already read into memory to compile them, so the hash adds no extra disk I/O.
- **Test later.**
  - Replacing a package from a different source is refused. A package from the same source with a higher version is accepted.
  - The same version with a different digest is refused as a "changed release".
  - A file that changes after install is refused at load as `FilesMissingOrChanged`.

## 4. Catalogue, provenance and signing

**Code.**

- `IndexEntry` (index.rs, lines 23–33) carries `tier` as a field supplied by the catalogue. It also has `url`, `sha256`, `size` and `apps`, but no publisher key.
- `Revocation` (lines 37–43) matches only by ID and an optional `below` version.
- `install_zip(bytes, tier)` does not take the expected ID, version or digest from the index entry, so the caller alone has to check that the archive matches the entry.

**Challenges:**

1. **The first-party trust root today is HTTPS plus a URL rule.** Until d8 and sg2 land, pl3 ("ship invert-rs as the first first-party catalogue entry") authenticates through TLS to an allowed host (n5) and an unsigned index. I recommend writing that down plainly. The first-party pilot then proves the install, enable, run and remove cycle, but not catalogue authenticity. The release notes should not imply otherwise.
2. **No path exists to install first-party packages.** Every sample package uses `org.example.*`. Once signing exists, a reserved-ID first-party plug-in can only arrive through the catalogue. **Options:**
   - (a) Use the catalogue without signatures for the pilot, accepting point 1.
   - (b) Bundle a first-party digest allowlist inside each signed app release. The app then installs a first-party file only if its release digest is on the list. This reuses the app's own code signing as the trust root, with no new key infrastructure.
   - (c) Give the pilot a non-reserved ID and migrate it later.

   **Recommendation:** (b) for the pilot. It is small and fails closed, and it removes the dependency on d4 and crafthub. (c) creates a migration of the kind the handoff already lists as work ("migrate old plug-in IDs").
3. **A catalogue entry must agree with what was installed.** **Proposed:** `install_verified(artifact: VerifiedArtifact)`. `VerifiedArtifact` is created only by the source verifier. It holds `{source, expected id, version, release_digest, archive_sha256}`. The store refuses an archive whose manifest ID, version or kind differs from the entry. This prevents the browser from showing one package and installing another.
4. **Revocation needs a richer key.** Revoking only by ID and range cannot do several things:
   - revoke the single bad release 1.4.0 while keeping 1.4.1 and 1.3.x installed;
   - revoke a release whose content was replaced at the same version;
   - limit a revocation to one source.

   **Proposed revocation entry:** `{source, id, match: digest | version_set, severity: advisory | block_new_use | block_load, reason}`. Matching by digest is exact. Matching by version set uses explicit ranges.
5. **TUF compared with a smaller signed index.** The baseline proposes TUF. I agree with the threat coverage but challenge the timing. TUF's role separation and key rotation mainly pay off for a catalogue that third parties can enter, and that gate is closed. **Options:**
   - (i) Adopt a vetted TUF client now for all sources.
   - (ii) For first-party use only, ship the bundled allowlist (point 2b) and add TUF before any community source is enabled.
   - (iii) Write a custom ed25519-signed index.

   **Recommendation:** (ii). Reject (iii), because it would be hand-rolled update security. Any TUF crate needs its licence checked against the allowlist and its maintenance reviewed. I have not checked either. Choosing it is a dependency decision that sits outside this note.
6. **Provenance.** `Provenance {source, commit}` (manifest.rs, lines 76–80) is checked for its form only: an https URL and 40 hexadecimal characters. It shows what the publisher claims. It does not show that the build came from that commit. The baseline decision on provenance level (security design, decision 3) stands. I add one honest limit: a hub-built artifact moves trust to the build service, and compromising that service affects every package it builds. Whichever option is chosen, it needs its own incident path in G10.

- **Owner.** The launcher owns the source manager, the verifier and network fetches. craft-ext-core owns the index and revocation types. The catalogue operator is undecided (d4).
- **Test later.** A fixture index whose entry says `org.storytold.invert` while the archive manifest says `org.storytold.other` is refused. A revocation by digest disables exactly one release.

## 5. Permissions and sandbox boundaries

### 5.1 Tier gate leak through the shared store

**Code.** `Policy::admit` runs in `install_zip` (store.rs, line 155) and in `Index::admitted`. `Reader` (reader.rs) never calls the policy. `location.rs` deliberately gives every build the same store folder, unless `CRAFT_EXT_DIR` is set.

**Adverse path.** A developer runs a launcher built with `--features third-party` and installs a `ReviewedThirdParty` package. The release build of PhotoCraft on the same machine reads the shared store. It lists and loads that package, because the reader accepts any enabled record. G5 is then enforced only at install, by whichever build installed the package.

**Proposed contract.** The resolver from §2 re-runs `Policy::admit(tier)` under the reading build's own features. Packages that are not admitted are reported as `TierNotAdmittedInThisBuild` and are never loaded. Optionally, development builds can default to a separate store folder.

**Test later.** A release-profile reader that finds a `ReviewedThirdParty` record does not load it.

### 5.2 Enabling a package is not granting it rights

**Code.** Installing enables the package in every app listed in `compat` (store.rs, lines 217–219). The same code runs on update, so an update re-enables the package in apps where the person had disabled it, and enables it in any newly listed app.

- Today this is a change of preference, because packages have no capabilities. It conflicts with the interpretation in security design §6 that the fresh CLI uses only plug-ins that are *enabled*.
- With capabilities, an update that adds FigCraft to a package's `compat` would silently activate code in FigCraft. Grants for that capability must be held separately, so that this never widens authority.

**Proposed:**

- Keep the enabled state per app. On update, preserve each app's existing `enabled` value. Newly listed apps start disabled, or carry a "pending" value that the launcher shows.
- Keep grants in a separate map `grants[(app, package_key)] = {caps, release_digest_at_grant}`.
- An update whose requested capabilities are a superset of the granted set leaves the new release *installed but ineligible* (`MissingGrant`) until the person accepts. This is G7 in mechanical form.
- **Default:** the fresh CLI never prompts. It reports `MissingGrant`.

**Test later.** Disable the package, update it, and confirm it is still disabled. An update that asks for more capabilities stays ineligible until approved, in both the GUI and the CLI.

### 5.3 Who checks modules on admission

The earlier review proposes a shared module validator (sg6 and pl2). The challenge is that admission questions are engine questions: which imports exist, which WebAssembly features are allowed, and what limits apply to tables, memories and functions. EffectCraft, FigCraft and PhotoCraft may each configure their engines differently. **Unknown:** the inspected source does not show whether they share an engine version or its settings. A validator that does not use the host's engine settings can accept a module that the host compiles differently, or the reverse.

**Recommendation:** have the shared crate own a pure *policy*. The policy lists the allowed imports for each granted capability, plus the feature and size limits. Each host applies it to the module its own engine has parsed, using the engine settings it will run with. The decision happens once and is applied by the engine that executes the module.

The capability enumeration is closed and versioned, as the earlier review proposed. Each capability ships with three things:

- its host enforcement,
- a test showing an undeclared capability is denied,
- and a fuzz target.

### 5.4 Which identity a call carries

The host identifies the calling plug-in from the instance handle it created, never from an ID the plug-in supplies. This repeats lifecycle review §10, which I agree with. The resolver's `Resolved` value creates that instance handle. Example: a FigCraft command that calls `document.edit` is checked against `grants[(figcraft, key)]`. Arguments that act like file paths are refused unless a file capability covers them.

### 5.5 Files installed from disk

Baseline decision 5 in the earlier review asks whether sideloaded plug-ins should get fewer capabilities. **Recommendation:** use the same capability vocabulary with stricter grant defaults. A package from a local file starts with no optional grants, and the person must see the capability list before enabling it. Do not invent a separate set of rights for sideloaded packages. The sandbox is the protection. The source only decides how much the store *presumes*.

## 6. Schema migration

Four formats change independently: the manifest, the catalogue index, the state file, and the plug-in references stored in project files.

| Format | Current behaviour (Code) | Risk | Proposed rule |
|---|---|---|---|
| Manifest | `schema != 1` is refused, and unknown fields are ignored | Fails closed. Fields added under schema 1 are silently ignored | Bump the schema for anything with authority. Use `requires` for critical features (§1) |
| Index | `schema != 1` is refused, and unknown fields are ignored | Same as the manifest | Same as the manifest. Keep a separate `newest_seen` counter for each source |
| State | `schema > 1` is refused (state.rs, lines 96–98). Unknown fields are ignored on read (test at lines 174–178) and **dropped on the next save**, because `State` is serialized again without them (store.rs, lines 148–151) | Older writers erase newer data. A bump makes older readers see an empty store. `Reader` treats an unreadable state as "nothing installed" (reader.rs, lines 43–45), so an older PhotoCraft quietly returns to its built-in theme | See below |
| Project references | Not designed in craft-extensions. They belong to the apps | A project that only records the ID can float to a newer release | Store `{package_key, contribution_id, release_digest, version, params, state_schema}`, as lifecycle review §17 recommends |

**State migration (proposed).** The state format has not been released yet, so changing it now is cheap.

1. Keep unknown top-level fields and unknown fields in each record (a flatten-style map), and write them back unchanged.
2. Split the version into `schema`, which is the format, and `min_reader`, which is the oldest reader that can safely use the file. A reader refuses only when `min_reader` is newer than itself. Adding an advisory field leaves `min_reader` alone. Adding grants raises it, because an older reader that ignores grants would load code that has no grant.
3. An older reader that sees a `min_reader` newer than itself reports "Extensions were updated by a newer ArtCraft. Update this app to use them". It does not quietly pretend the store is empty.
4. A writer never lowers `min_reader`.

- **Adverse path.** Grants arrive in state version 2 with `min_reader` set to 2. An older PhotoCraft that runs only themes would then lose its theme. Splitting the authority data into a separate file, such as `grants.json`, leaves theme-only readers unaffected. **Recommendation:** use separate files for each concern: `state.json` for installed and enabled packages and themes, `grants.json`, and `revocations.json` for the cached revocations. Each file is atomic and has its own `min_reader`. Changes that touch more than one file are made under the existing lock, in a fixed order.
- **Test later.** An older-reader fixture keeps fields it does not know. An older reader shows an explicit "needs a newer app" result instead of an empty list.

## 7. Reuse of craft-ext-core and craft-ext-store

I agree with the baseline: extend the existing crates and do not fork them. What to keep and what to change:

| Keep as is (Code shows it is sound in form) | Extend | Replace or redesign |
|---|---|---|
| Rules for IDs and text, path validation, the exact manifest-to-archive match, streaming size and hash checks, `create_new` writes, the cross-process lock, atomic state writes, the generation counter, re-reading and re-checking themes, the plain-language errors, the `deny` lint attributes | Manifest schema 2 (§1), index entries and revocations (§4), state records and split files (§3, §6), a resolver (§2), and a source-verifier handle (§4) | Deleting the old directory right after an update (§8), auto-enabling on update (§5.2), keeping the tier only as a caller parameter (§3), and naming package folders by version alone (§3) |

**Ownership challenge.** Lifecycle review §20 says craft-libs owns "store transactions, grants". That predates the discovery of the existing implementation, and the README's precedence rules already override it. I recommend marking that sentence as superseded. Moving the crates into craft-libs remains a later decision (the earlier review's decision 1). It needs a second real consumer and a boundary that has been agreed. AGENTS.md also requires a crate in craft-libs to serve more than one app and to depend on no app. ext-core and ext-store satisfy that, but the launcher-specific source manager would not.

Lifecycle review §9 suggests that "portable installations may need their own store". That conflicts with Jeremy's decision recorded in `location.rs` (lines 3–6). That decision stands. The env override covers the isolation case.

**CLI parity risk (Code).** `CRAFT_EXT_DIR` (location.rs, lines 68–71) is read from the process environment. A GUI app started by the OS and a CLI started from a shell can therefore resolve different stores. **Proposed:** the CLI prints the store root it resolved in `--verbose` and diagnostic output. Apps record the root they resolved in job diagnostics.

**Performance (Code).** Every `Reader::generation()` call reads and parses the whole `state.json`, which is capped at 4 MiB (reader.rs, lines 43–51). That is fine at today's sizes. If many packages and frequent polling make it measurable, a tiny separate generation file, or a check of the file's modification time, would avoid the full parse. Measure before changing it.

## 8. Concurrent update, revocation and removal

**Code.**

- `install_staged` unpacks outside the lock, moves the folder into place under the lock, saves the state, releases the lock, and *then* deletes the previous version's folder (store.rs, lines 229–231).
- `uninstall` deletes the folder after the lock is released (line 246).
- Readers return paths and hold no lease (reader.rs, lines 53–66).
- If the process crashes after the folder is moved into place but before the state is saved, an orphan `packages/<id>/<version>` folder is left behind. A later install of the same version then hits `destination.exists()` and returns `StoreError::Io`, whose message reads "the extension files could not be saved" (lines 211–213). That is misleading, and it wedges installs until someone cleans up by hand.
- Nothing cleans up `staging/` after a crash, or package folders that no record refers to.

Scenarios below are traced from source and have not been run. The phrase "proposed outcome" means the recommendation.

| # | Scenario | Current outcome (source-derived) | Proposed outcome |
|---|---|---|---|
| C1 | An EffectCraft fresh-CLI export is running with `blur-plus` 1.0. The launcher updates it to 1.1 | The 1.0 folder is deleted after the commit. If the module was already in memory, the job finishes. If the effect loads lazily later in the job, it cannot be resolved, and EffectCraft's baseline omits unresolved effects | The job pins `release_digest` when it starts and reads and verifies all module bytes into memory before it renders. Old folders are removed by deferred cleanup, not by the update |
| C2 | A revocation of `blur-plus` 1.0 arrives during that export | No enforcement path exists (`is_revoked` has no callers) | Behaviour depends on severity. `advisory` lets the job finish and adds a diagnostic. `block_new_use` lets the job finish but blocks new jobs and enabling. `block_load` cancels at the next contribution call boundary, and the export fails with a structured `Revoked` error. It is never silently omitted |
| C3 | A FigCraft panel is open and the person uninstalls the package in the launcher | FigCraft notices only when it polls the generation. The instance stays in memory | The generation change makes the host check eligibility again. Calls that need capabilities return `Revoked`/`NotInstalled` after that. The panel closes at the next idle point, and changes already committed through undo stay in place |
| C4 | A PhotoCraft theme is active and gets updated | The state points at 1.1, whose folder already exists before the commit, and the reader reads the theme again. That is safe | No change needed. Keep a test |
| C5 | The person disables a package, then updates it | It is re-enabled for every listed app (lines 217–219) | Each app's enabled state is preserved (§5.2) |
| C6 | A crash between moving the folder and saving the state | An orphan folder, and the same version then fails with `Io` | Store folders by release digest, so a retry is idempotent: an existing folder with the same digest and verified contents is reused. A cleanup step at launcher start removes stale `staging/*` and package folders that no record refers to and that are older than a grace period |
| C7 | Two processes install versions 2 and 3 at the same time | Both unpack. Whichever commits second wins, or loses with `Downgrade` after doing the unpacking work | This is acceptable. Optionally, read the current version before unpacking so a downgrade fails early |
| C8 | A release-build app and a feature-on development launcher share a store | The release app loads third-party records | The resolver re-admits by tier (§5.1) |
| C9 | A project saved with `blur-plus@digest A` is opened after a revocation and removal | Not designed | The project keeps its saved state as data it does not interpret and shows a placeholder. Export fails with `NotInstalled`/`Revoked`. Opening a project never installs anything |

**Proposed cleanup and retention contract:**

- An update never deletes folders. Uninstall removes the record and marks the folder as "pending removal".
- Cleanup runs under the lock, at launcher start or on demand. It removes folders that are:
  - not referenced by any record,
  - not the previous release kept for rollback (one per package by default),
  - older than a grace period.
- **Leases.** Module bytes are fully loaded into memory and verified, so executable plug-ins need no lease. Large resource packs (up to 256 MiB) that importers read lazily would need either an app-private copy or a lease file. **Recommendation:** postpone resource leases until a real resource consumer reads files lazily. Until then, document that resources are read in full when imported.
- **Owner.** craft-ext-store owns cleanup and retention. Apps own job pinning and checking eligibility again at call boundaries.
- **Performance.** Keeping one previous release can roughly double disk use per package in the worst case: 32 MiB for a plug-in and 256 MiB for a resource pack. Resource packs could opt out of rollback retention.
- **Test later.**
  - Simulated crashes at each write step: after unpacking, after the move, and after saving.
  - A concurrent update during a pinned job.
  - Each revocation severity during a running export.
  - Cleanup never removes the active release or the retained rollback release.

## 9. Pilot ordering, from a contracts point of view

This is a recommendation and does not reorder anything the user has decided.

1. Fix the store invariants (§3, §5.1, §5.2, §6, §8) while there are still no users. All of these are cheap now and costly after release.
2. Run the PhotoCraft theme and invert first-party pilot through a bundled first-party allowlist (§4, option 2b). This proves install, enable, run, reopen and remove, without the catalogue or signing.
3. Run the EffectCraft pilot on manifest schema 2 with zero capabilities. That exercises the contributions, the resolver, the fresh-CLI pathway, job pinning and export failure for unresolved effects.
4. Run FigCraft with the first real capability (for example `figcraft.selection.read`). It goes through host enforcement, a test that an undeclared capability is denied, and a fuzz target.
5. Add authenticated community catalogues (TUF or equivalent) only before anyone asks for sg13. The release gate stays closed.

## 10. Decisions still open, ranked

Each item states what follows from it, so it can be discussed before an answer is given.

1. **Identity model.** Should the package key be `(source, id)`, with the release digest as identity? If yes, store records, folder layout, revocation, project references and continuity checks all build on it. If no, G7 and the replacement of packages across sources have no foundation.
2. **Split enabled state from grants, and preserve enabled state on update.** Yes makes the "fresh CLI never widens authority" rule true by construction. No lets an update silently activate code in a new app.
3. **Re-admit tiers on read.** Yes closes the leak of feature-on development stores into release apps. The cost is one policy call per resolve.
4. **Cleanup and retention instead of deleting immediately.** Yes protects running jobs and allows rollback. The cost is disk space for one retained release.
5. **State file split and the `min_reader` rule.** This decides whether older apps degrade explicitly or quietly. It is cheapest before the first release.
6. **How first-party packages are installed for the pilot.** Choose between a bundled allowlist, an unsigned catalogue over HTTPS, or a temporary non-reserved ID. This decides whether the PhotoCraft pilot waits on d4 and crafthub.
7. **Manifest schema 2 with `requires` critical features.** This decides how multi-contribution packages fail on older hosts.
8. **Revocation keyed by digest and severity.** This decides whether one bad release can be withdrawn without collateral damage, and how running jobs react.
9. **Where module admission happens.** A shared policy applied by each host's engine (recommended), or a standalone shared validator.
10. **When TUF is required.** For first-party use now, or before any community source opens (recommended).

## What this note did not do

- It did not build, test, run, fuzz or benchmark anything.
- Performance figures are estimates, not measurements.
- It did not inspect EffectCraft or FigCraft source again, or check whether the inspected branches are current.
- It did not check any dependency's licence or maintenance.
- It produced no security certification. Interleavings are traced from source text.
- It did not edit, summarize or synthesize any other delegate's note.

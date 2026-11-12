# P1: Store foundation and manifest schema 2

Status: **build-ready plan, 2026-11-10.** Unit P1 of [the architecture](../architecture.md#22-delivery-plan); deepens [implementation plan section 4](../implementation-plan.md#4-p0-and-p1-store-foundation). Each PR still needs its own approval.

**Code read:** storytold/craft-extensions `main` `a6c5fae`. The code was read through the PR #1 branch head `5ba16aa`; `git diff a6c5fae 5ba16aa -- crates Cargo.toml .github` is empty, so the line numbers below are the same on both commits. Nothing was built or run for this plan. The repository README says 111 tests pass with cargo 1.97.1; I counted 112 `#[test]` attributes, one of them behind `#[cfg(not(feature = "third-party"))]`.

## 1. Goal and exit criteria

**Goal.** Make `craft-ext-store` safe to ship:
- it records verified releases and per-app setup;
- it keeps releases that anyone still uses;
- it survives crashes and concurrent writers;
- it evaluates eligibility in one function;
- old apps never destroy newer state.

It also adds manifest schema 2 beside schema 1.

**Exit criteria.** Each one is a named test running in CI on Linux and Windows. macOS joins when shared CI adds it.

| # | Criterion | Proving test (section 6) |
|---|---|---|
| X1 | Install enables nothing; setup enables only the chosen app and contributions | `store::tests::install_enables_no_app` |
| X2 | A crash at every transition leaves either the previous committed state or the new complete release | `crash::kill_at_every_fail_point` (process kill) and `faults::fail_point_matrix` (injected) |
| X3 | Concurrent writers (launcher, two apps, CLI) never lose an update and never corrupt state | `concurrency::four_writers_hundred_changes` |
| X4 | A frozen schema-1 reader and writer never misread or erase schema-2 state | `compat::v1_frozen_code_refuses_v2_without_touching_bytes` |
| X5 | An update during a leased job keeps the old release readable until the lease ends; then cleanup removes it | `lease::update_during_leased_job_keeps_old_release` |
| X6 | Retained set = default ∪ pins ∪ pending-consent ∪ live leases; cleanup never deletes anything in it | `cleanup::retained_set_property` (proptest) |
| X7 | One `eligibility()` returns a structured reason, before loading, for a tampered file, unsupported contract, stale grant, revocation or policy denial | `eligibility::every_reason_is_reachable` |
| X8 | Schema-1 manifests keep their exact meaning; an unknown critical schema fails before interpretation | `manifest::v1_golden_fixtures_unchanged`, `manifest::unknown_schema_refused_first` |

## 2. Code to read and change (verified at `a6c5fae`)

| Area | File:line | What is there today | Change |
|---|---|---|---|
| Install enables apps | `crates/ext-store/src/store.rs:200-202` (`for app in &record.apps { state.enabled…insert }`) | Every app the manifest lists is enabled | P1-01 removes it |
| Old release deleted on update | `store.rs:212-214` (`if let Some(dir) = old_dir { let _ = fs::remove_dir_all(dir); }`) | Deleted right after the state write | P1-02 |
| **Uninstall deletes at once** | `store.rs:229` (`let _ = fs::remove_dir_all(dir);` in `uninstall`, `:219-233`) | Also deletes while readers may hold files. **The gap analysis lists only the update path (C2)**; uninstall has the same flaw | P1-02 covers both |
| First-party-only protection | `store.rs:180-182` (`existing.tier == Tier::FirstParty && tier != Tier::FirstParty`) | A file can replace a non-first-party store install | P1-03 (C4) |
| Downgrade and same version refused | `store.rs:183-188` (`Downgrade`, `AlreadyInstalled`) | — | P1-09 (C3) |
| Folder layout | `store.rs:99-101` `package_dir(id, version)` and **a duplicate** in `reader.rs:53-55` `dir_of` | `packages/<id>/<version>/` built in two places | P1-06 (C8) moves both to one `layout` module |
| State record | `crates/ext-store/src/state.rs:22-32` `Installed {id, version, kind, name, tier, apps}`; `:34-44` `State {schema, generation, installed, enabled, active_theme}` | No digest, source, setup or grants | P1-04, P1-05, P1-12 |
| Schema check | `state.rs:73-101` (`Head {schema}`; `head.schema > SCHEMA` → `Newer` at `:93`; then `state.schema != SCHEMA` → `Corrupt` at `:97`) | One number; serde ignores unknown fields (test `unknown_fields_are_ignored`, `state.rs:169-174`), so a same-schema writer drops fields it does not know | P1-04 (C7) |
| Write path | `store.rs:126-138` `update()` → `State::read` → `change` → `generation += 1` → `craft_atomic::atomic_write` | Read-modify-write under `.lock` (`store.rs:111-123`, `File::try_lock`, 5 s wait) | Kept; becomes the only writer path for every transition |
| Reader | `reader.rs:58-67` `installed_for` returns the folder if `is_dir()`; it never re-hashes, checks revocation or applies policy | — | P1-13 (C6) returns `VerifiedRelease` through `eligibility()` |
| Manifest v1 | `crates/ext-core/src/manifest.rs:16-21` constants; `:80-96` `Manifest`; `:152-166` `parse` reads `Head` first and refuses `schema != 1`; `:174-176` permissions must be empty; `:192-201` plugin needs `interface` | — | P1-10 adds the v2 dispatch beside v1 |
| ID cap | `crates/ext-core/src/id.rs:31` `MAX_EXTENSION_ID = 64` | — | P1-08 (C10) |
| Tiers | `crates/ext-core/src/tier.rs:12-21` `Tier {FirstParty, FromFile, ReviewedThirdParty}`; `:52-58` `admit` behind `cfg!(feature = "third-party")` | — | Kept; mapped to labels in P1-05 |
| Unpack | `crates/ext-store/src/unpack.rs:88-177` `unpack_into` checks per-file SHA-256 only; **no archive digest is computed** | — | P1-05 computes the archive SHA-256 (the catalogue's pinned digest) in `install_zip` before unpacking |
| Path caps | `crates/ext-core/src/path.rs:42-44` `MAX_PATH_BYTES = 200`, `MAX_SEGMENT_BYTES = 64`, `MAX_DEPTH = 8` | — | Inputs to the Windows path budget (P1-08) |
| Atomic write | craft-libs `crates/atomic/src/lib.rs:267` `atomic_write(path, bytes)`: same-dir temp, `F_FULLFSYNC`/`sync_all`, rename, Windows rename retry (`:174`); **a crash leaves temp files the caller must sweep** (`temp_name`, `:557`) | — | P1-11 sweeps them |
| Mutation harness | `docs/mutation_check.py` `STORE` list | 10+ store mutants | Every PR adds mutants (section 4) |
| Crate root lints | `crates/ext-store/src/lib.rs:8`, `crates/ext-core/src/lib.rs:12` deny `unwrap_used, expect_used, panic, unimplemented, todo, unreachable` | Workspace has `unsafe_code = "forbid"` (`Cargo.toml` `[workspace.lints.rust]`) | New modules inherit them |

**Dependency already pinned:** `craft-atomic` is a git dependency on craft-libs `8f5a283` (`crates/ext-store/Cargo.toml`). It must move to a published version or a tag before any crate is published.

## 3. Module layout and API sketches

```text
crates/ext-core/src/
  id.rs            ExtensionId (cap 128), ContributionId (new), AppId
  digest.rs        Digest32 (new): SHA-256, parse/format "sha256:<64 hex>"
  release.rs       ReleaseKey {id, version, digest}  (new, FROZEN)
  permission.rs    Family, Access, Scope, Permission, PermissionSet (new, FROZEN vocabulary; landed by P2-02, used by P1 and P2)
  manifest.rs      v1 Manifest (unchanged meaning) + parse dispatch
  manifest_v2/     ManifestV2, Contribution, AppSection, ContractReq, Dependency, ResourceRef, Media (new)
  compat.rs        Compatibility {Compatible, Partly(Vec<(ContributionId, Reason)>), Incompatible(Reason)} (new, FROZEN outcomes)
  revocation.rs    Revocation, Severity, Matcher, RevocationSet (new; P7 supplies the signed source)
  deps.rs          resolve(), dependents_of() (new, pure)
  eligibility.rs   evaluate(EligibilityInput) -> Eligibility (new, pure, wasm-safe)
  label.rs         Label {Official, Listed, NotReviewed, Development}; from_source() (new)
crates/ext-store/src/
  layout.rs        the only place that builds store paths (new)
  state/mod.rs     State v2 model, Header {schema (=read_version), write_version}
  state/v1.rs      frozen schema-1 reader for migration and compatibility tests (moved from state.rs)
  state/migrate.rs v1 -> v2, under the lock, once
  model.rs         pure transitions State -> State' (no fs; reused by the web store in P5)
  store.rs         Store: fs effects around model.rs transitions
  setup.rs         per-app setup and grants (new)
  lease.rs         Lease, LeaseSet, liveness through held OS locks (new)
  cleanup.rs       retained set, cleanup, crash recovery (new)
  private.rs       private storage folders and quota (new)
  reader.rs        read side; returns VerifiedRelease through eligibility
  faults.rs        #[cfg(any(test, feature = "fault-injection"))] named fail points (new)
```

Rule for the split: `model.rs` holds every decision and `store.rs` holds every effect. Tests of decisions run without a disk, and the browser-local store in P5 reuses `model.rs` unchanged.

### Frozen early (schema document, golden fixtures and compatibility tests before an app relies on them)

```rust
// ext-core/src/release.rs  (FROZEN: on disk, in catalogue records, in projects)
pub struct Digest32([u8; 32]);                 // "sha256:<64 lowercase hex>" on the wire
pub struct ReleaseKey { pub id: ExtensionId, pub version: Version, pub digest: Digest32 }

// ext-core/src/id.rs
pub struct ContributionId { package: ExtensionId, local: LocalName }   // "<package>/<local>", local ≤ 64 bytes, same alphabet
impl ContributionId { pub fn parse(s: &str) -> Result<Self, IdError>; pub fn package(&self) -> &ExtensionId; }

// ext-core/src/permission.rs  (FROZEN vocabulary; architecture section 8)
pub enum Family { Document, Files, Network, Clipboard }        // Account reserved, not parseable in release 1
pub enum Access { Read, Edit, Write, Connect, Use }
pub enum Scope {
    Selection, CurrentDocument, AllOpenDocuments,              // document
    PickedItem, PickedFolder, OwnStorage,                      // files
    Domain(HttpsDomain),                                       // network: a bounded, lowercase, IDNA-checked host
    OnUserAction,                                              // clipboard
    App(AppScope),                                             // app-specific scope inside an existing family
}
pub struct Permission { pub family: Family, pub access: Access, pub scope: Scope }
// Text form "document.edit:selection", "network.connect:api.example.com". parse() refuses unknown text.
pub struct PermissionSet(BTreeSet<Permission>);
impl PermissionSet { pub fn widens(&self, old: &PermissionSet) -> PermissionSet; }  // the added rights, for "ask again"

// ext-core/src/compat.rs  (FROZEN outcomes)
pub enum Compatibility { Compatible, Partly(Vec<(ContributionId, Unavailable)>), Incompatible(Unavailable) }
```

### Provisional (may change until a second app uses them)

```rust
// ext-core/src/eligibility.rs
pub struct EligibilityInput<'a> {
    pub app: &'a AppId,
    pub release: &'a VerifiedRecord,           // from the store: digest checked at install, manifest retained
    pub manifest: &'a ManifestView,            // normalised v1 or v2
    pub host: &'a HostProfile,                 // contracts, majors/minors, engine features, platform
    pub setup: Option<&'a AppSetup>,           // contributions on, grant record, pin, pending
    pub revocations: &'a RevocationSet,
    pub policy: &'a MachinePolicy,             // e.g. "no network for plugins"
    pub admit: &'a tier::Policy,               // third-party feature gate
}
pub struct Eligibility { pub release: ReleaseKey, pub contributions: Vec<(ContributionId, Availability)> }
pub enum Availability { Available { authority: PermissionSet }, Unavailable(Unavailable) }
pub enum Unavailable {
    NotSetUp, Disabled, Revoked { severity: Severity, reason: String }, BlockedByPolicy(PolicyRule),
    ContractMissing { contract: ContractRef, have: Option<ContractVersion> }, EngineFeature(String),
    Platform(String), PendingConsent { added: PermissionSet }, GrantStale, DigestMismatch,
    DependencyMissing(ExtensionId), ThirdPartyClosed,
}
pub fn evaluate(input: &EligibilityInput<'_>) -> Eligibility;   // pure: no fs, no clock (time passed in)

// ext-store/src/store.rs
impl Store {
    pub fn install(&self, archive: ArchiveSource<'_>, source: Source) -> Result<InstallReport, StoreError>;
    pub fn set_up(&self, app: &AppId, release: &ReleaseKey, choice: SetupChoice) -> Result<AppSetup, StoreError>;
    pub fn grant(&self, app: &AppId, id: &ExtensionId, add: &PermissionSet) -> Result<GrantRecord, StoreError>;
    pub fn revoke(&self, app: &AppId, id: &ExtensionId, remove: &PermissionSet) -> Result<GrantRecord, StoreError>;
    pub fn choose_version(&self, id: &ExtensionId, key: &ReleaseKey) -> Result<(), StoreError>;   // C3: older allowed
    pub fn uninstall(&self, id: &ExtensionId) -> Result<UninstallReport, StoreError>;           // asks about dependents first
    pub fn cleanup(&self) -> Result<CleanupReport, StoreError>;
    pub fn lease(&self, holder: &LeaseHolder, keys: &[ReleaseKey]) -> Result<LeaseGuard, StoreError>;
}
pub enum Source { Catalogue { listing: Digest32, catalogue_version: u64 }, File, Bundled, Development, Project }
pub struct InstallReport { pub key: ReleaseKey, pub label: Label, pub replaced_default: Option<ReleaseKey> }

// ext-store/src/reader.rs
pub struct VerifiedRelease { pub key: ReleaseKey, pub label: Label, pub manifest: ManifestView, files: VerifiedFiles }
impl VerifiedRelease { pub fn module_bytes(&self, path: &PackagePath) -> Result<Arc<[u8]>, LoadError>; } // read once, hashed, then handed on: no re-read
impl Reader { pub fn eligible_for(&self, app: &AppId, host: &HostProfile, now: Timestamp) -> Vec<(VerifiedRelease, Eligibility)>; }
```

`VerifiedRelease::module_bytes` returns the same bytes it hashed. The host never reopens the path, so a file swapped between check and load is not a race (architecture section 5, rule "a verified release carries checked bytes").

## 4. Ordered PR list

Sizes as in the [implementation plan](../implementation-plan.md#2-how-we-build): S 50–250 changed lines, M 250–600, L 600–1,000. Each PR has its own issue, claim, parent-failing and head-passing evidence, and mutants added to `docs/mutation_check.py`.

| PR | Gap | Scope | Size | Main files | Test that fails first | Mutants added | Depends |
|---|---|---|---|---|---|---|---|
| P1-00 | — | Historical D1–D6 README/threat-model alignment completed by merged PR #3. Remaining gate-to-test mapping and later local reconciliation are separate work, not a repeat of that PR | S (docs) | `docs/THREAT-MODEL.md`, `README.md` | — | — | PR #1 and #3 merged |
| P1-01 | C1 | Install enables nothing; `set_enabled` is the only enabler; reader unchanged | S | `store.rs:200-202`, `store/tests.rs:43-56, :111-118` | `install_enables_no_app`: a theme for `photocraft` and `launcher` installs, and `is_enabled` is false for both. **Rewrites** the two current tests that assert the opposite | "install enables manifest apps" (restore the loop) | P1-00 |
| P1-02 | C2 (first) | Never delete a replaced or uninstalled release in the same call; record it as `orphaned` (in-memory list returned to the caller, no schema change yet); delete only through an explicit `cleanup_unreferenced()` that the tests call | S | `store.rs:212-214, :229-231` | `update_keeps_old_release_files_until_cleanup`: install 1.0.0, open `theme.json` from it as a reader, install 1.1.0, read the old file successfully. Same for uninstall | "update deletes old dir"; "uninstall deletes dir" | P1-01 |
| P1-03 | C4 | A `FromFile` install never replaces any store-recorded ID unless that record is also `FromFile`. Error `WouldReplaceStoreInstall` | S | `store.rs:178-190`, `StoreError` | `a_file_cannot_replace_a_reviewed_install`: install `ReviewedThirdParty` (test under `--features third-party`) then a file with the same ID: refused, bytes and state unchanged. The first-party case keeps its existing test | "only first party protected" | P1-02 |
| P1-04 | C7 | **State versioning.** Move `state.rs` to `state/v1.rs` frozen. New header: `schema` is the *read version*; add `write_version`. First write migrates v1 to v2 under the lock. Refuse to write when `write_version > WRITER`; refuse to read when `schema > READER`. Never turn "unreadable" into "empty and writable" (a corrupt state blocks writes with `StateError::Corrupt` and is never overwritten) | M | `state/{mod,v1,migrate}.rs`, `store.rs:126-138`, `docs/schemas/state.md`, `crates/ext-store/fixtures/state/` | `v1_frozen_code_refuses_v2_without_touching_bytes`: the frozen v1 `read` gets `Newer(2)`; the frozen v1 `Store::update` returns an error, and the file bytes are identical before and after. Second test: `additive_v2_1_is_read_by_v2_0_and_not_written` | "writer ignores write_version"; "corrupt state treated as empty" | P1-03, P0.4 answer |
| P1-05 | C5 | Verified release record: compute archive SHA-256 before unpack; explicitly retain the original checked manifest bytes and SHA-256 (the current extractor does not persist them); host-assigned `Source` and `Label` | M | `store.rs:141-216`, `unpack.rs`, `state/mod.rs`, `ext-core/src/{digest,release,label}.rs` | File metadata cannot become `Official`; archive digest matches; checked manifest remains available after install and tampering prevents load | "label read from manifest"; "archive digest skipped"; "retained manifest altered" | P1-04 |
| P1-06 | C8 | Digest-addressed folders `<version>-<first 16 hex of digest>`; the full digest lives in state and is rechecked; an existing folder is never reused (`store.rs:194-196` behaviour kept). One `layout` module replaces `package_dir` and `reader.rs:53` | M | `layout.rs`, `store.rs`, `reader.rs` | `same_version_different_bytes_never_share_a_folder`: two archives with ID `org.example.x` 1.0.0 and different bytes land in two folders; the reader returns the default one by digest | "folder ignores digest" | P1-05 |
| P1-07 | C2 (full) | Leases (design in section 5.3): per-process instance ID, liveness through a held OS lock, acquired under the store lock | M | `lease.rs`, `store.rs` | `update_during_leased_job_keeps_old_release`: process A (child test binary) leases 1.0.0 and reads; process B installs 1.1.0 and runs cleanup; A keeps reading; after A exits, cleanup removes 1.0.0 | "dead lease treated as live forever"; "lease ignored by cleanup" | P1-06 |
| P1-08 | C10 | ID cap 128 bytes (from 64), `ContributionId` local part ≤ 64; Windows path budget test | S | `id.rs:31`, `layout.rs` | `longest_id_installs_on_windows`: an ID of exactly 128 bytes with an 8-deep, 200-byte inner path installs and loads on the Windows runner; 129 bytes fails with `TooLong` | "cap off by one" | ID decision (section 7, D1) |
| P1-09 | C3 | Explicit version choice, older allowed; the same digest is idempotent (`Ok(AlreadyThere)`, no generation bump); an ordinary update never downgrades | S/M | `store.rs`, `model.rs` | `explicit_v2_to_v1_succeeds_and_update_never_downgrades` | "update allows downgrade"; "same digest bumps generation" | P1-06 |
| P1-10 | C11 | `docs/schemas/manifest-v2.md` reviewed first; then the v2 parser beside v1 and `ManifestView` | M | `manifest.rs:152-166`, `manifest_v2/` | `v1_golden_fixtures_unchanged` (every v1 fixture parses to the same value as at `a6c5fae`); `unknown_schema_refused_first` (schema 3 with an invalid body returns `UnsupportedSchema(3)`, not a field error) | "v2 parser accepts schema 1 body" | P1-08, P2-02 (permission vocabulary, which manifest v2 parses) |
| P1-11 | — | Cleanup and crash recovery: stray staging, unrecorded folders, `craft-atomic` temp files, dead leases, orphaned folders; Windows "in use" deletions retried next start | M | `cleanup.rs`, `faults.rs` | `kill_at_every_fail_point` (section 6.3) | "cleanup deletes leased"; "cleanup deletes pinned" | P1-07 |
| P1-12 | C9 | Per-app setup and grants with generation; pending update when permissions widen, owner changes or the package is native | M | `setup.rs`, `model.rs`, `state/mod.rs` | `update_keeps_choices_and_holds_widened_rights_pending`: app A on 1.0 with grants; 1.1 adds `network.connect:api.example.com` and a contribution; A stays on 1.0 with `pending`; app B (no grant needed) moves to 1.1 with the new contribution off | "new contribution starts on"; "widened rights auto-granted"; "generation not bumped on revoke" | P1-10, P2-02 |
| P1-13 | C6 | One `eligibility::evaluate`; the reader returns `VerifiedRelease` only through it; module bytes rehashed at load | M | `ext-core/src/eligibility.rs`, `reader.rs:58-67` | `every_reason_is_reachable`: a table of fixtures, one per `Unavailable` variant, each evaluated before any module byte is handed out | "digest not rechecked at load"; "stale grant accepted" | P1-12, P2-02 (permission types) |
| P1-14 | C13 | Revocation enforcement by digest, version range or publisher, at enable, load and update, file installs included; known revocations kept in `revocations/known.json` and never removed by repair | M | `ext-core/src/revocation.rs`, `store.rs`, `reader.rs` | `revoked_digest_from_file_cannot_enable_or_load` | "revocation by id only"; "file installs skip revocation" | P1-13 |
| P1-15 | — | Data dependencies: pure `deps::resolve` (cycles, conflicts, missing) and removal check | M | `ext-core/src/deps.rs`, `store.rs` | `a_cycle_or_conflict_activates_nothing`; `removing_a_needed_package_asks_first` | "cycle accepted" | P1-10, P1-12 |
| P1-16 | — | Private storage per app and package, size cap, deleted on uninstall, kept on disable | M | `private.rs`, `layout.rs` | `another_package_cannot_reach_private_storage` (the API takes a `VerifiedRelease`, never a path); `uninstall_removes_disable_keeps` | "quota unchecked" | P1-07 |
| P1-17 | C14 | Repository URL in `Cargo.toml` | S | `Cargo.toml` | — (check `origin/chore/repo-url-storytold` first; it exists on the remote) | — | — |

**Order and parallelism.** P1-00 → P1-01 → P1-02 → P1-03 → P1-04 → P1-05 → P1-06 → {P1-07, P1-08, P1-09} → P1-10 → P1-11 → P1-12 → P1-13 → P1-14 → {P1-15, P1-16}. P1-17 can land at any time.
- P1-07, P1-08 and P1-09 touch different functions after P1-06 and can be reviewed side by side.
- P1-10 depends on P1-08 and on P2-02 (the permission vocabulary in `ext-core`), so it can be drafted during P1-07 review.

**Changed from the reconciled plan:**
1. **P1-02 includes uninstall.** `store.rs:229` deletes at once too.
2. **P1-04 is simpler than "a gate that must stop old writers".** I read the frozen schema-1 code. `State::read` returns `Newer` for `schema > 1` (`state.rs:93-95`), and `Store::update` calls it before writing (`store.rs:128`). So frozen v1 code already refuses a v2 file for both reading and writing, without touching it. Data loss happens only if new fields ride under `"schema": 1`. The rule is therefore: **never add a persisted field under schema 1; the first new field ships with schema 2.** The cost is that unupdated apps stop seeing installs until they update. P0.4 must confirm no shipped build writes or reads `state.json`. On their `main` refs, EffectCraft `fb62b9d7`, FigCraft `0e86f359`, PhotoCraft `655295b2` and launcher `753a710` have no `craft-ext` dependency. Only the unmerged launcher branches `origin/feat/extensions` and `origin/test/ext-plus-ai` use the store.
3. **Architecture section 11's last sentence needs a correction.** It says "the first move from today's schema 1 keeps what current apps read in place". That is possible only if new fields ride under schema 1, which is the destructive case. Proposed wording: "The first move to schema 2 is refused by apps that only know schema 1, without changing the file; no shipped app writes schema 1 (P0.4)."

## 5. Data formats, migrations and fixtures

All formats get a schema document in `docs/schemas/` and golden fixtures under `crates/<crate>/fixtures/`. Compatibility tests compare **raw bytes**, not serde round trips, where fidelity matters.

### 5.1 State v2 (`state.json`)

```json
{
  "schema": 2,
  "write_version": 2,
  "generation": 41,
  "packages": {
    "io.github.ink.grain": {
      "source": { "kind": "file" },
      "label": "not-reviewed",
      "default": "sha256:9f2c…",
      "releases": {
        "sha256:9f2c…": { "version": "1.2.0", "dir": "1.2.0-9f2c4e1a7b3d5c60", "manifest_sha256": "sha256:…", "installed_at": "2026-11-10T12:00:00Z", "kind": "plugin" }
      }
    }
  },
  "apps": {
    "effectcraft": {
      "packages": {
        "io.github.ink.grain": {
          "release": { "follow": "default" },
          "contributions": { "halftone": true },
          "grant": { "generation": 3, "release": "sha256:9f2c…", "source": "file", "permissions": ["document.read:selection"], "picked": [], "granted_at": "2026-11-10T12:01:00Z" },
          "pending": null,
          "time_raise": { "preview": 2.0 }
        }
      },
      "theme": null
    }
  }
}
```

- `schema` is the read version: a reader with `READER < schema` refuses to read. `write_version`: a writer with `WRITER < write_version` refuses to write. An additive change raises only `write_version`; a breaking one raises `schema`.
- Timestamps go through `chrono` (house rule 5) as RFC 3339 UTC. `chrono` is MIT/Apache-2.0. Time is passed into `model.rs`, never read there.
- `release.follow` is `"default"` or `{ "pin": "<digest>" }`. `pending` is `{ "release": "<digest>", "added": [...], "reason": "permissions" | "owner" | "native" }`.
- Bounds: the existing 4 MiB `MAX_STATE_BYTES` (`state.rs:19`) stays; add caps of 4,096 packages, 64 releases per package, 64 apps, and 512 permissions per grant. Each is refused with a structured error.

**Migration v1 → v2** (`state/migrate.rs`, under the store lock, on the first write):

| v1 | v2 |
|---|---|
| `installed[i]` `{id, version, kind, name, tier, apps}` | `packages[id]`, with one release. The digest is **unknown**, so it is recomputed by hashing the folder's retained archive. If no archive was kept (true of every v1 install), the release is marked `"digest": null, "legacy": true`. Eligibility treats it as `DigestMismatch` until it is reinstalled, so the label never silently becomes "verified" |
| `tier` | `label` via `FirstParty → official`, `FromFile → not-reviewed`, `ReviewedThirdParty → listed` |
| `enabled[app]` set | `apps[app].packages[id].contributions` — all contributions **on**, as v1 meant; the setup record is created with an empty grant |
| `active_theme[app]` | `apps[app].theme = "<id>/<theme>"` |

**Fixtures:**
- `fixtures/state/v1-*.json`: empty, theme-only, plugin, two apps, oversize, corrupt.
- `fixtures/state/v2-*.json`: each transition's expected result.
- `fixtures/state/v2_1-additive.json`: a future field, used to test read-by-older.

### 5.2 Manifest schema 2 (`craft-extension.json`, written by `craft-ext pack`)

```json
{
  "schema": 2,
  "id": "io.github.ink.grain",
  "name": "Ink & Grain",
  "version": "1.2.0",
  "publisher": { "name": "Ink" },
  "license": "MIT OR Apache-2.0",
  "links": { "source": "https://github.com/ink/grain", "homepage": null },
  "description": "Halftone and grain effects.",
  "native": false,
  "files": [ { "path": "modules/grain.wasm", "sha256": "…", "size": 412345 },
             { "path": "media/thumbnail.png", "sha256": "…", "size": 20480 } ],
  "apps": {
    "effectcraft": { "contracts": [ { "id": "raster.filter", "major": 1, "min_minor": 0 } ],
                     "permissions": { "required": [], "optional": [] }, "min_app": null },
    "photocraft":  { "contracts": [ { "id": "raster.filter", "major": 1, "min_minor": 0 } ],
                     "permissions": { "required": [], "optional": [] }, "min_app": null }
  },
  "contributions": [
    { "id": "halftone", "style": "transform", "contract": "raster.filter@1", "module": "modules/grain.wasm",
      "apps": ["effectcraft", "photocraft"], "state_schema": 1 }
  ],
  "dependencies": [ { "id": "io.github.ink.luts", "min": "1.0.0", "below": "2.0.0" } ],
  "resources": [ { "name": "big-lut", "url": "https://github.com/…/big.cube", "size": 104857600, "sha256": "…" } ],
  "media": { "thumbnail": "media/thumbnail.png", "screenshots": [] },
  "build": { "sdk": "craft-guest 0.1.0", "inventories": { "raster.filter@1": "sha256:…" } }
}
```

- **The validator** (`ext-core`) checks every v1 rule plus:
  - contribution IDs unique;
  - every contribution app present in `apps`;
  - `module` listed in `files`;
  - `style` ∈ {transform, session, data};
  - permissions parse and use only grantable families (no process or native for WebAssembly);
  - `native: true` requires per-platform artifacts (P10);
  - a dependency's version range is a structured `{min, below}`, not a text grammar, so no range parser is needed;
  - `resources` are https, ≤ 2 GiB each (the GitHub asset cap), with SHA-256;
  - the thumbnail is PNG, ≤ 512 KiB, decoded only by the host (as on `wip/extensions-handover`).
- **Custom-section agreement** (P4 adds the section): the contributions in the manifest must equal those in the module's `craft.contributions` custom section, byte-identical after canonical JSON. P1-10 defines the comparison; P4-04 produces the section.
- **Unknown fields.** v1 ignores them (`manifest.rs:3-4`). v2 does the same for optional data; anything that changes meaning must raise `schema`. `docs/schemas/manifest-v2.md` lists which fields are critical.
- **Fixtures:** `crates/ext-core/fixtures/manifest/v1/*.json` (every current test input, frozen), `v2/valid-*.json`, `v2/invalid-*.json` (one per validator rule, each with its expected error).

### 5.3 Leases

```text
leases/<instance>.lock   created first, locked exclusively (File::lock) for the process lifetime; never rewritten
leases/<instance>.json   {"instance": "<32 hex>", "app": "effectcraft", "started": "<rfc3339>", "releases": ["sha256:…"]}
                         replaced atomically whenever the leased set changes
```

- `instance` is 128 random bits, generated once per process. PID reuse does not matter, because **liveness is "the `.lock` is still held"**. The OS releases the lock when the process dies, on macOS, Linux and Windows alike (`flock`/`LockFileEx` through `File::lock`). The lock lives on its own file because an atomic replace of the `.json` creates a new inode and would drop a lock held on it.
- **Acquire**: take the store lock, check the release is still recorded, update `.json`, release the store lock.
- **Cleanup**: take the store lock. For each `.json`, `try_lock` its `.lock`. If that succeeds, the owner is dead: delete `.json` first, then `.lock`. The live leases go into the retained set.
- **Doubt means retain.** An unreadable lease, or a lock call that errors without saying "would block", counts as live.
- **Experiment before P1-07** (section 7, R3): locks on network home folders, and in Flatpak with only the folder permission.

### 5.4 Retained set (AR01 confirmed 2026-11-12)

`retained(app_setups, packages, leases) = defaults ∪ effective app releases ∪ pins ∪ pending-consent references ∪ required dependency releases ∪ live leases`. The user confirmed retaining all referenced releases and cleaning only unreferenced releases. This refines T08's earlier current-plus-leases rule. Global uninstall removes that package's setup/default references; dependency handling and live leases still constrain removal.

The JSON example's `follow: default` and pending target do not yet completely represent effective release or chained pending updates. AR02 requires an explicit model before schema freeze. Selection/lease acquisition, retention calculation and deletion need one serialization protocol; uncertain liveness must prevent unsafe cleanup. Test two apps on different releases, pending targets, dependencies, concurrent cleanup and process death.

## 6. Test strategy

| Layer | What | Where |
|---|---|---|
| Unit | Every validator rule, every `model.rs` transition, every `Unavailable` reason | `#[cfg(test)]` beside the code |
| Property (`proptest`, MIT/Apache-2.0) | Random install/update/pin/lease/uninstall/cleanup sequences against a model: retained set never deleted; generation strictly increases; state always parses; no release is reachable without a recorded digest | `crates/ext-store/tests/model_props.rs` |
| Fixture compatibility | v1 manifests and state frozen; v2 golden outputs; raw-byte comparison of unchanged state after refused writes | `fixtures/` + `tests/compat.rs` |
| Injected faults | `faults::point("after-rename")` etc. return `Err` in a `fault-injection` build. Points: after-stage, after-validate, after-rename, before-state-write, after-state-write, during-cleanup-delete, during-migrate | `tests/faults.rs` |
| Process crash | The test binary re-executes itself (`std::env::current_exe()`), with an env var choosing a scenario. The child prints `ready:<point>` and blocks; the parent kills it, restarts a store, runs recovery and asserts state. Runs on Linux and Windows CI | `tests/crash.rs` |
| Concurrency | Four child processes perform 100 random changes each against one store; the final state must contain every acknowledged change (each child logs acknowledgements); never `Corrupt` | `tests/concurrency.rs` |
| Hostile | Oversize state, deep JSON, duplicate keys, 129-byte IDs, NUL in names, symlinked `packages/` (refused), a lease file pointing at an unknown digest | `tests/hostile.rs` |
| Mutation | One mutant per rule, listed per PR in section 4, run by `docs/mutation_check.py` **in a disposable checkout only** (it edits sources in place) | shared CI job (P1-00 issue in craft-common-actions) |

**Windows specifics** (CI runner; the gap analysis did not cover these):
- An open file blocks deleting its folder, so cleanup records the release as `deleting` and retries at the next start.
- Locks are mandatory on Windows: a reader must not hold `.lock` for reading.
- The path budget is tested in P1-08. Rust's `std::fs` adds the `\\?\` prefix to long paths, but other code that opens plugin files may not.

**Benchmarks**: none required. Record the time of `eligible_for` with 500 installed packages, because apps call it at startup.

## 7. Risks, experiments and open questions

| # | Risk | Smallest settling experiment | Before |
|---|---|---|---|
| R1 | A shipped build reads or writes `state.json` (P0.4) | Ask the launcher and PhotoCraft maintainers, and check release notes and published artifacts. The source search above found none on `main` | P1-04 |
| R2 | 16 hex digits in folder names collide | None needed. A collision causes a refused install, never confusion, because the full digest is rechecked and existing folders are never reused | — |
| R3 | OS locks are unreliable on network homes or in Flatpak | A two-process lock test on SMB and NFS homes and in a Flatpak with `--filesystem` limited to the store | P1-07 |
| R4 | Cross-process revoke latency: a host call cannot reread `state.json` | In-process revokes flip an `AtomicU64` epoch at once; other processes poll the state header (one `stat` plus a small read) every 250 ms. Measure the cost and **state the bound** ("within a quarter second for another app") | P1-12, P3-04 |
| R5 | Migrated v1 installs have no digest | Mark them `legacy` and require a reinstall (section 5.1). Confirm with the maintainers that no v1 installs exist on users' machines | P1-04 |
| R6 | Windows paths over 260 bytes break a non-std path user | P1-08 test on the Windows runner | P1-08 |

**Questions for maintainers:**
- **D1, ID cap:** 128 bytes (recommended; `io.github.` + 39-character owner + `.` + a 78-byte name), or another value.
- **D2, deployment boundary:** confirm the move to schema 2 (old apps stop seeing installs until updated), given P0.4.
- **D3, legacy installs:** require a reinstall, or trust them once with a "re-verify" prompt. Recommended: reinstall.
- **D4, private-storage default cap:** proposed 64 MiB per package per app, raisable like time budgets.
- **D5, wording:** AR01 retained-set wording is now user-approved (2026-11-12) and applied to architecture section 11. Legacy-schema wording is technically corrected; the deployment boundary and header representation still need agreement (AR03).

## 8. Effort and parallel work

| PR | Effort | Can run in parallel with |
|---|---|---|
| P1-00 | S | P2-00 design note, P3-00 experiment |
| P1-01, P1-02, P1-03 | S each | P3-00, P6-00 inventory |
| P1-04 | M (the hardest review: schema and crash semantics) | P2-01 |
| P1-05, P1-06 | M each | P3-01, P3-02 |
| P1-07, P1-11 | M each, with process tests | P1-08, P1-09 |
| P1-10 | M | P1-07 |
| P1-12, P1-13, P1-14 | M each | P3, P4 |
| P1-15, P1-16 | M each | P5-01 |
| P1-17 | S | anything |

Roughly 6 S and 11 M PRs. With one contributor and one store reviewer, this means one or two PRs in review at a time. The schedule follows the measurements called for in implementation plan section 10, not guesses.

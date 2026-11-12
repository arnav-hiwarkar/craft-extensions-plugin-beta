# P7: Catalogue, bot, TUF, 48-hour hold, revocation and client fetch

Status: **build-ready plan, 2026-11-10.** Unit P7 of [the architecture](../architecture.md#12-distribution-and-catalogue); deepens [implementation plan section 6, P7](../implementation-plan.md#p7-catalogue-alongside-p6p9).
- Each PR still needs its own approval.
- **Creating the catalogue repository, publishing to Pages, and later adding a domain need authorization.** PC08 chose GitHub Pages first and a custom domain later; it did not authorize publication or creation.
- The `third-party` feature stays off until P7-08.

**Code read (nothing built or run):**
- craft-extensions `a6c5fae`: `crates/ext-core/src/index.rs`;
- craft-libs `fd5e2fe`: `crates/update/src/http.rs`;
- launcher `origin/main` `753a710`: `crates/native/src/http.rs`, `install.rs`.

## 1. Goal and exit criteria

**Goal.**
- A metadata-only catalogue in a GitHub repository.
- GitHub Pages initially serves signed metadata; a custom domain is later work, not a prerequisite. PC08 also retains maintainer-controlled TUF signing without separate publisher keys. Source/namespace checks and pinned hashes remain required.
- A secretless bot that checks every release PR.
- A protected publish job that signs `catalogue.json` with TUF and holds auto-merged updates for 48 hours.
- A client crate that fetches, verifies and caches the catalogue, applies revocations offline, and downloads release assets from authors' GitHub Releases with the pinned digest.

**Exit criteria (architecture section 22: "swapped asset and expired metadata refused; revocation applied offline after sync").**

| # | Criterion | Proving test |
|---|---|---|
| X1 | A swapped asset (same URL, different bytes) is refused and flagged | `client::swapped_asset_refused` |
| X2 | Expired metadata is shown as stale and cannot authorize an install; verification failure never replaces trusted metadata | `client::expired_metadata_cannot_install`, `client::bad_signature_keeps_previous` |
| X3 | Same version with different content is refused (rollback and swap) | `client::same_version_different_content_refused` |
| X4 | A revocation synced once is enforced offline, and survives repair and restore | `client::revocation_applies_offline_after_sync` |
| X5 | The clock running backwards blocks new installs and updates but not installed plugins | `client::clock_rollback_blocks_installs_only` |
| X6 | Wider permissions, a new owner or a native release never qualifies for auto-merge | `bot::auto_merge_eligibility_table` |
| X7 | An auto-merged update is not in the signed catalogue until 48 hours after merge | `publish::hold_48h` |

## 2. Code areas (verified)

| What | Where | Change |
|---|---|---|
| Unsigned index | craft-extensions `crates/ext-core/src/index.rs`: `Index::parse(bytes, now, newest_seen)` (`:114`); **`if index.version < newest_seen` (`:126`) accepts the same version with any content**; revocation by `id` and `below` only (`:36-43`); `IndexEntry.tier` read from the index (`:28`) | Superseded by `ext-catalog` (C12). Keep `index.rs` until no caller remains, then remove it in a separate PR |
| **Downloader** | **craft-libs `crates/update/src/http.rs:263`** `Http::download(&self, url, dest, size, sha256, stop, progress)`: SHA-256 required, Range resume (`part_path` `:576`), pause and cancel, **`https_only` on the agents (`:73-116`)**. The launcher already depends on `craft-update` (`crates/native/Cargo.toml`) | **Reuse `craft-update` from craft-libs.** The reconciled plan said to extract the launcher's `crates/native/src/http.rs::Http::download` (`:235`). That is a second copy, and its download agent does not set `https_only`. Use the shared one, and add the launcher's extra tests to it only if they are missing |
| Checksum fallback | Launcher `install.rs:664-685` falls back to `SHA256SUMS.txt` (`:692`) | **Not used.** The digest comes only from signed catalogue metadata |

## 3. Layout and API sketches

```text
craft-extensions
  crates/ext-catalog/          client: TUF verification (adapter over the chosen crate), snapshot, revocation sync,
                               pinned downloads via craft-update, ranking input; no egui
catalogue repository (maintainers create and name it)
  publishers/<login>.toml      login, numeric owner id, display name, official (maintainers only), status
  packages/<id>.toml           id, repository + numeric repository id, licence, apps, summary, native
  releases/<id>/<version>.toml version, tag, commit, per-platform url/size/sha256, contracts per app, permissions, bot report
  revocations/<n>.toml         digest | {id, min, below} | publisher; advisory | block-new | disable; reason; issued_by; reviewed_by
  transfers/<n>.toml           old and new owner (numeric ids)
  policy.toml                  listing rules version, hold hours (48), minimum client version
  .github/workflows/           bot (secretless, read-only) and publish (protected environment) via craft-common-actions
```

```rust
// ext-catalog (provisional)
pub struct TrustRoot(/* shipped in the app */);
pub struct Snapshot { pub version: u64, pub expires: DateTime<Utc>, pub records: Records, pub revocations: RevocationSet }
pub enum Freshness { Fresh, Stale { since: DateTime<Utc> }, ClockBehind }
pub fn refresh(root: &TrustRoot, transport: &dyn Transport, cache: &CacheDir, now: DateTime<Utc>) -> Result<(Snapshot, Freshness), CatalogError>;
pub fn may_install(s: &Snapshot, f: Freshness) -> bool;                 // false when Stale or ClockBehind
pub fn download(rel: &ReleaseRecord, platform: Platform, http: &craft_update::http::Http, stop: &Stop) -> Result<VerifiedArchive, CatalogError>;
pub fn rank(query: &Query, s: &Snapshot, stars: &Stars) -> Vec<Ranked>;  // filter → tier → official first within tier → stars → newer → id
```

- The **TUF layout** uses the standard roles. The root keys are 2-of-3 offline hardware keys. The `targets`, `snapshot` and `timestamp` keys are online, held only in the protected publish job. `catalogue.json` and `revocations.json` are TUF targets.
- **Clients on web** need an in-memory transport and cache, which the TUF crate spike must cover.

## 4. Ordered PR list

| PR | Scope | Size | Test that fails first | Depends |
|---|---|---|---|---|
| P7-00 | **Spike (a report).** Compare candidate TUF clients (for example `tough` and the `tuf` crate from rust-tuf) on: maintenance; licence (must be on the allowlist); expiry, rotation and rollback fixtures; native and web builds; dependency weight. Recommend one | — | — | P0 |
| P7-01 | Record schemas (`docs/schemas/catalogue.md`) and parsers in `ext-core`; namespace rules (an `io.github.<owner>` ID requires a matching numeric owner ID; other domains need a maintainer-verified record) | M | `records::same_login_new_owner_id_does_not_inherit` (repo-jacking) | P1-10 |
| P7-02 | `ext-catalog` pinned download on `craft_update::http::Http::download`; the digest comes only from a verified record; resume and cancel inherited | M | `client::swapped_asset_refused`; `client::digest_from_metadata_only` (a `SHA256SUMS.txt` beside the asset is ignored) | P7-01 |
| P7-03 | TUF adapter and C12: verification, expiry, rotation, rollback; same version with different content refused | M | `client::same_version_different_content_refused`; `client::expired_metadata_cannot_install`; `client::bad_signature_keeps_previous`; `client::clock_rollback_blocks_installs_only` | P7-00 |
| P7-04 | Revocation sync into the store's `revocations/known.json` (P1-14); applied at start and every few hours when online | M | `client::revocation_applies_offline_after_sync`; `client::known_revocation_survives_repair` | P7-03, P1-14 |
| P7-05 | Bot, part 1 (in the catalogue repository, secretless and read-only): download, digest, size, the shared validator (`ext-core`), licence (MIT and/or Apache-2.0 plus the house allowlist for bundled items), public source link at the tag, namespace | M | `bot::licence_outside_allowlist_fails`; `bot::source_tag_must_exist` | P7-01 |
| P7-06 | Bot, part 2: permission diff against the previous release, owner and transfer check, native detection; auto-merge eligibility; WebAssembly smoke test in the reference host (`ext-conformance`); native static checks only (P10 adds the ephemeral runner) | M | `bot::auto_merge_eligibility_table` (wider permissions, new owner, native and first listing are all `needs-human`; native also `needs-two-maintainers`) | P7-05, P3-06 |
| P7-07 | Publish job (protected environment, maintainer approval): regenerate and sign; the 48-hour hold computed from merge time for auto-merged releases; human-approved releases publish at once | M | `publish::hold_48h` (a release merged by the bot at T is absent from targets signed at T+47h and present at T+48h) | P7-03, P7-06 |
| P7-08 | **Enablement gate**: the `third-party` feature allows `Listed` sources once P7-01..07 are merged and the maintainers decide. The guard test keeps it out of `default` | S | the existing `third_party_feature_is_not_a_default_feature` (`ext-core/src/lib.rs`) stays green; `tier::listed_requires_verified_snapshot` | P7-01..07, maintainer decision |
| P7-09 | Browse ranking and built-in search suggestions from the cached catalogue; the query never leaves the machine | M | `ranking::official_first_within_tier_only`; `suggest::query_never_sent` (the transport records zero requests) | P7-03, P5-10 |
| P7-10 | Stars, fetched centrally by the publish job into the signed snapshot (no client telemetry) | S | `stars::client_makes_no_star_requests` | P7-07 |

**Order.** P7-00 → P7-01 → {P7-02, P7-05} → P7-03 → {P7-04, P7-06} → P7-07 → {P7-09, P7-10} → P7-08.

## 5. Data formats

```toml
# releases/io.github.ink.grain/1.2.0.toml (authored by `craft-ext prepare-release`, PR'd by the author)
version = "1.2.0"
tag = "v1.2.0"
commit = "<40 hex>"
[[assets]]
platform = "any"            # native: one per platform (P10)
url = "https://github.com/ink/grain/releases/download/v1.2.0/io.github.ink.grain-1.2.0.zip"
size = 412345
sha256 = "<64 hex>"
[apps.effectcraft]
contracts = ["raster.filter@1.0"]
permissions = []
```

- **The bot writes its report into the PR**, not into the file. The publish job adds `approved_at`, `approved_by` and `merge_kind = "bot" | "human"`.
- **Fixtures:**
  - test TUF roots under `crates/ext-catalog/fixtures/tuf/`, with test keys only (threat T19);
  - expired, rotated, rolled-back and mixed-version metadata.

## 6. Test strategy

- **Client:** fixture roots for every TUF failure mode; a local HTTP server for downloads (`craft-update` already has `serve` helpers in its tests); browser runs for the web transport.
- **Bot:** table-driven over fixture PRs. Each run on a fixture repository is checked for having no secrets in its environment.
- **Publish:** run in a dry mode against a fixture repository; the hold arithmetic is tested with chrono times passed in (house rule 5).
- **Rehearsals** (P12): root-key rotation, online-key leak recovery, and an emergency disable.

## 7. Risks and decisions

| # | Risk | Experiment | Before |
|---|---|---|---|
| R1 | No suitable TUF client | P7-00 | P7-03 |
| R2 | Browser downloads of GitHub assets fail (CORS on the redirect target) | A real browser fetch | Web listing installs |
| R3 | Review and key-holder capacity (reconciled plan risk 16) | Name people; rehearse one native two-maintainer approval and one key rotation | P7-07 |

**Decisions for maintainers:**
- **D1.** The repository name and owner, the Pages endpoint, the key holders (three root-key custodians), and the incident owner. A custom domain is deferred beyond initial serving by PC08.
- **D2.** The TUF crate (after P7-00).
- **D3.** The enablement moment for `third-party` (P7-08).
- **D4.** The minimum client version policy field: whether the catalogue can tell old clients "update to keep installing".

## 8. Effort and parallel work

- **Counts:** about 10 PRs (2 S, 8 M) plus the spike, over two repositories.
- **Parallel:**
  - Starts once P1-10 (manifest v2) and P1-14 (revocation enforcement) exist.
  - P7-00 can run at any time.
  - The client PRs (P7-02..04) run beside P6.
  - The repository PRs (P7-05..07) run beside P8 and P9.

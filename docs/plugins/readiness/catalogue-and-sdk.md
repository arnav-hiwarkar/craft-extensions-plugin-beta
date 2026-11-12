# Catalogue, distribution and SDK path

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: coordinator design, 2026-11-08. This is a proposal for review. Nothing here is implemented, and no repository, key, workflow or release has been created. It applies the [accepted decisions](../draft/decisions.md) and narrows the [security and distribution draft](../draft/security-and-distribution.md) into the smallest path that works. The [gaps and roadmap](system-gaps-and-roadmap.md) lists the code changes this path depends on. Source anchors are in the [evidence index](evidence-index.md).

## The path in one picture

```text
author repo ──SDK check/pack──▶ GitHub Release (compiled .zip asset, immutable if enabled)
     │                                   │
     └──catalogue PR (release record)────┤ bot downloads asset by URL, recomputes digest,
                                         │ runs the same Rust validator the store uses, writes a report
                                         ▼
                         human maintainer approves exact digest ──▶ merge
                                         │
                    protected publish job: build catalogue.json + stats.json, sign metadata
                                         │
   app / launcher manager ◀── fetch signed metadata (no login) ──┘
        │ search: filter → relevance tier → official first → cached stars → stable tie-break
        │ install: download browser_download_url, require catalogue digest, hand bytes to craft-ext-store
        ▼
   installed (disabled) ──explicit per-app enable + grants──▶ eligible ──▶ GUI / CLI / plugin use
```

Three facts make this small:

1. **Nobody hosts binaries except publishers.** The catalogue stores metadata only. Bytes come from the publisher's GitHub Release asset.
2. **The ordinary public path needs no login or per-package GitHub API discovery.** The authenticated catalogue carries each asset's direct download URL, size and SHA-256. The client can fetch it over HTTPS. Catalogue automation fetches stars/releases centrally; optional private-release or verification integrations may use APIs separately.
3. **One validator.** The catalogue bot, the SDK `check` command and the installer all call the same `craft-ext-core`/`craft-ext-store` validation code. There is no second schema or second installer.

## What already exists and can be reused

| Need | Existing code | Gap |
|---|---|---|
| Bounded archive unpack, inventory and digest check, staging, lock, atomic state | craft-extensions `ffd8512e` `ext-store/src/store.rs:154-233`, `unpack.rs` | Install auto-enables every listed app and deletes the old release (`store.rs:217-219`, `229-231`). See blocker B1 |
| Index with version, expiry, rollback refusal, revocations | `ext-core/src/index.rs:22-55` | No signature. Revocation is only `id` + `below` version. The entry's `tier` field is data from the index, so it is trustworthy only once the index is authenticated |
| GitHub Release download with resume, size cap, SHA-256 check, proxy, throttle | craft-launcher `main` `c9c3aca` `crates/native/src/install.rs:621-659`, `http.rs:227-260`, `core/src/release.rs:245` | Lives inside the launcher's `native` crate. It accepts the publisher's own digest (GitHub `digest` field or `SHA256SUMS.txt`). For plugins the catalogue digest must be the only authority |
| Release feed with fallback and 429 back-off | `crates/native/src/backend.rs:680-830`, `feed_throttle.rs` | App-release feed. Pattern is reusable for the catalogue fetch |
| Catalogue implementation with reviewed-index intent | EffectCraft `ce9eaec` `crates/engine/src/system/library.rs:1-40`, `46-135`: source describes `index.json` in `storytold/effectcraft-plugins`, with per-release version, API, minimum app, GitHub asset URL, SHA-256 and optional size; offline cache and installed digest | Live repository/content/review practice was not checked. Fetch is unsigned and app-specific, using system `curl` and a separate installed list. Reuse behavior/fields in the existing craft-ext schema successor, not as a second authoritative format |
| Local-file install path in launcher UI | `craft-launcher` `6ede77f` `apps/artcraft-launcher/src/extensions.rs:79` | Only `Tier::FromFile`. No network catalogue round trip exists on that branch |

The launcher already refuses an app download that nothing vouches for (`install.rs:621-624`). The plugin client needs a stricter rule: the vouching digest must come from signed catalogue metadata. A publisher can replace a mutable release asset, and GitHub's per-asset `digest` describes whatever was uploaded ([GitHub changelog: release asset digests](https://github.blog/changelog/2025-06-03-releases-now-expose-digests-for-release-assets/)). Treat the GitHub digest as a consistency check, not approval.

**Proposed extraction (work package W6).** Move the download routine (`Http::download` with an expected size and digest, redirect handling, resume, stop flag) into a small shared crate that both the launcher and the in-app manager can call. That gives the "two real consumers" this repository requires. It must not take the launcher's app-catalogue types with it.

## Catalogue repository

Proposed: one new Storytold-owned GitHub repository holding metadata and policy only. Its name, owner and maintainers are still open decisions. Creating it is not authorized by this investigation.

```text
catalogue/
  policy/            licence policy, review checklist, reserved names, maintainers, incident process
  publishers/<publisher-id>.toml
  packages/<package-uuid>/package.toml
  packages/<package-uuid>/releases/<version>.toml
  revocations/<id>.toml
  aliases.toml       legacy dotted id -> package uuid
```

### Record fields

**Publisher**

| Field | Meaning |
|---|---|
| `publisher_id` | Stable catalogue-assigned slug. It is never taken from a package |
| `display_name` | Shown in UI. Not unique proof |
| `github_owner` / `github_owner_id` | Login and numeric account or organisation ID, checked when enrolled. Both are recorded so a rename can be noticed |
| `official` | True only for Storytold-maintained publishers. Set by maintainers, never by a PR from the publisher |
| `enrolled_by`, `enrolled_at` | Maintainer and date |
| `status` | `active`, `suspended` or `transferred` (a transfer links to a reviewed transition record) |

**Package**

| Field | Meaning |
|---|---|
| `uuid` | Stable package identity. Generated by the SDK on `create` |
| `publisher_id` | Owning publisher |
| `name`, `summary`, `kinds`, `apps` | Search data |
| `repository` | `owner/name` plus numeric repository ID. Used as the star source and as continuity evidence |
| `legacy_ids` | Old dotted IDs such as `org.example.tone`, for migration |
| `licence` | SPDX expression of the package |
| `homepage`, `support` | Optional links |

**Release** (one file per approved version, which is the unit a human approves)

| Field | Meaning |
|---|---|
| `version` | Three-part version, matching the manifest |
| `tag`, `commit` | Git tag and the commit it pointed at when reviewed |
| `release_id`, `asset_id`, `asset_name` | GitHub numeric IDs and name |
| `url` | The `browser_download_url` the client fetches |
| `size`, `sha256` | Exact archive bytes bound by the authenticated reviewed release record; a bare hash alone is not approval |
| `manifest_sha256` | Digest of `craft-extension.json` inside the archive |
| `contributions` | Per app: contribution ID, contract and version, required and optional capabilities, headless flag, network use |
| `min_app` | Per app |
| `licences` | Package and bundled dependency SPDX list, from the SDK inventory |
| `immutable_release` | Whether GitHub reported the release as immutable when checked |
| `attestation` | Optional. `verified` only when a maintainer or bot actually verified it, otherwise absent |
| `review` | Reviewer login, PR number, date, and what was inspected |
| `channel` | `stable` or `preview` |

**Revocation**

| Field | Meaning |
|---|---|
| `match` | One of: exact `sha256`; `uuid` with version range; `publisher_id` |
| `severity` | `advisory` (warn), `block-new` (no new installs or enables), `disable` (stop loading at next eligibility check) |
| `reason`, `issued_at`, `supersedes` | Text, time, and an optional earlier record it replaces |

The client-side schema needs these as a successor to `ext-core` index schema 1. Schema 1's `Revocation { id, below }` (`index.rs:36-43`) cannot express digest or publisher matches.

## Review of an exact release

1. The publisher creates a GitHub Release with the SDK-built `.zip`. Publishing it as an [immutable release](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases) is recommended, not required. Immutability stops later asset swaps. It is not a safety review.
2. The publisher opens a catalogue PR adding `releases/<version>.toml`. The SDK's `prepare-release` command writes this file.
3. A bot job with read-only permissions and no secrets:
   - downloads the asset from `url` with a size cap, and checks `sha256` and `size`;
   - compares GitHub's `digest` for that asset, and the release's immutable flag, through the REST API ([release assets](https://docs.github.com/en/rest/releases/assets));
   - runs the shared validator (manifest, inventory, imports and exports, declared capabilities, licences);
   - writes a delta report against the previous approved release: new or changed capabilities, contributions, imports, dependencies, sizes and state schema.
   It never runs the package or its build scripts.
4. A maintainer reads the report and the source diff between the reviewed tags, records what they checked, and approves. A newer GitHub release from the publisher stays invisible until its own record is approved.
5. Merge triggers a protected publication job that regenerates `catalogue.json`, refreshes `stats.json`, and signs the metadata.

Urgent fixes use the same steps with priority. There is no publish-first, review-later path. How many maintainers must approve, and response times, are governance choices that are still open.

This is a CI design for a future repository. It does not change any existing CI. When built, it should call `storytold/craft-common-actions`.

## Authenticating what the client downloads

The existing index has version and expiry checks but no signature (`index.rs:1-6`). Until that is fixed, no fetched catalogue, including official entries, can authorize a network install. This is blocker B5.

Recommendation, in order of preference:

1. **TUF metadata around a single catalogue target**, with a vetted Rust client. The draft already recommends this ([authenticated metadata](../draft/security-and-distribution.md#authenticated-update-metadata)). It provides root key rotation, threshold signing, and protection against freeze and rollback ([TUF specification](https://theupdateframework.github.io/specification/latest/)). The client library, its licence and its maintenance status have not been chosen or checked.
2. If maintainers judge TUF too heavy for the first version, a single Ed25519 signature over `catalogue.json`, with the existing version and expiry fields and a public key embedded in the app, is a smaller start. It does not cover key rotation or recovery, so it must not be the long-term design for community admission. Record this as a deliberate first-party-only interim if chosen.

Root reconciliation retains established update tooling as the production recommendation. The custom interim option above remains unselected. Evaluate the actual implementation fit before building a temporary signature protocol.

Either way:

- The trust root ships inside the signed app.
- Metadata can be served from the catalogue repository's GitHub Pages or raw files. Integrity comes from the signature, not the host.
- A verification failure keeps the last trusted metadata and its revocations. It never falls back to unsigned data.
- `stats.json` (stars) can be signed in the same publication or left unsigned. Stars only affect order. They must never be able to change identity, digests, permissions or the `official` flag.

## Search and ranking

Order of operations:

1. Filter by query, current app, kind, and compatibility with this app version and platform.
2. Relevance tier: exact name or ID match, then prefix match, then text match.
3. Within a relevance tier: official packages first.
4. Then repository stars, descending.
5. Then the newer approval date.
6. Then package UUID, so the order is stable.

Official precedence applies inside a relevance tier, so an official package does not bury an exact community match for a different query. This interpretation of "official takes precedence" is open for the user to confirm.

Stars come from the GitHub repository `stargazers_count` ([repositories API](https://docs.github.com/en/rest/repos/repos)). The publication job fetches each listed repository once per run and writes `{repository_id, stars, fetched_at}`. On a rate limit it stops and keeps the previous numbers ([rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)). The UI shows "repository stars" with the date, because several packages can share one repository. Desktops never ask GitHub for stars. Missing stars sort as zero and never affect eligibility.

## Install, grant and use

1. The client resolves the package to one approved release record for this app and platform.
2. It downloads `url`, enforcing `size` and `sha256` from the signed metadata (the extracted download routine from W6).
3. It hands the bytes plus a verified-origin record (catalogue version, package UUID, publisher ID, release digest) to `craft-ext-store`. The store must stop treating `tier` as a free argument for network installs (blocker B2).
4. The store retains the archive digest, manifest and origin, records the release, and **does not enable it anywhere**.
5. In the app, the user enables the contribution and approves the listed capabilities for that app. That writes a grant record (blocker B3).
6. The app's loader checks eligibility on every load: installed, enabled for this app, compatible, granted, not revoked, and admitted by the build's tier policy. Today the read side checks only `enabled` (`ext-store/src/reader.rs:58-63`).

## Updates, offline use, recovery and credentials

| Situation | Behaviour |
|---|---|
| New approved release | The manager shows it. Applying it is explicit unless the user turns on automatic updates for unchanged authority. Wider capabilities stay pending until the user approves them. Per-app enablement and choices survive. Today an update re-enables every app (`store.rs:217-219`) |
| Upstream asset deleted or replaced | The download fails the digest check. The installed release keeps working. The manager says it is no longer downloadable |
| Offline | Installed eligible features work. Search shows the cached catalogue with its date. New installs need signed metadata that has not expired. Unseen revocations cannot be promised |
| Metadata expired | Browse is labelled stale. No new network install. Existing installs keep running unless a known revocation says otherwise |
| Rollback | Choose a retained older release that is not revoked. Requires multi-release retention (blocker B1) |
| Corrupt state | Keep the damaged file for diagnostics. Never infer grants from package folders. Keep the highest-seen catalogue version and known revocations |
| Restricted release | Optional credential stays in the host OS credential store and is sent only to its authenticated API origin. Strip authorization on cross-origin redirects; fetch approved asset URLs without forwarding the user token. Redact signed URLs/secrets from logs. Public packages need no credential |
| Logs | Download and verification events go to the app's bounded local log. No usage telemetry is needed for ranking |

## Local files

A local `.zip` goes through the same `craft-ext-store` validation. It records origin `local` with the file digest. It needs no account, PR or review. It cannot claim an official label or a catalogue package's identity or grants by reusing its UUID or name. A known blocked digest is refused whatever the origin. It gets the same sandbox and the same explicit grants. Today `check_id` stops only first-party prefixes, and a local file can replace a same-ID `ReviewedThirdParty` install (`store.rs:196-207`, `tier.rs:61-66`). See blocker B2.

## Licences

The catalogue checks package and bundled-dependency SPDX expressions against explicitly selected product policy. The user has not selected the list or official proprietary/local-file treatment. Repository dependency/asset rules govern what Craft repositories bundle; do not silently apply them as the ecosystem product policy or broaden them by editing a copy.

## SDK generated from app registries

### Principle

Each app owns the truth about what plugins may call. The SDK is generated from it, never hand-written alongside it.

```text
app repo: Rust operation registry (descriptor per supported operation)
   └─ release tooling exports  contracts/<app>/<contract>@<version>.json  (inventory + digest)
craft-extensions repo:
   ├─ craft-ext-core / craft-ext-store      (shared validation, used by store, bot and SDK)
   ├─ contracts/<app>/...json               (pinned copies, added by reviewed PR)
   ├─ craft-ext-guest                        (guest-side Rust crate: generated typed wrappers per contract)
   └─ craft-ext (CLI): create, add, inspect, check, pack, test, dev, prepare-release
```

Where the operation registries come from varies by app. The app audits ([media](haiku-media-apps.md), [design](haiku-design-apps.md), [domain](haiku-domain-apps.md)) record what each app has today. The [roadmap](system-gaps-and-roadmap.md#app-operation-registries) explains what to build where.

### Commands

| Command | Does | Uses |
|---|---|---|
| `create` | New package with a UUID, licence and one app contribution | Template |
| `add` | Adds a contribution for another app and contract | Pinned contract JSON |
| `inspect` | Lists contracts, operations, capabilities and limits an app version offers | Contract JSON |
| `check` | Validates manifest, inventory, module imports against declared capabilities, licences and versions | `craft-ext-core` validator |
| `pack` | Builds the deterministic `.zip` with the successor manifest | `craft-ext-core` |
| `test` | Runs literal input/output fixtures in a host harness | App-provided test host. Future |
| `dev` | Installs into a labelled development store and reloads | `craft-ext-store` with a dev profile |
| `prepare-release` | Writes the asset and `releases/<version>.toml` for the catalogue PR | Validator |

The SDK never publishes, never creates repositories, and never adds a permission on its own. If a module imports a host function its manifest did not declare, `check` fails. The host checks again at load.

### Compatibility

Package schema, contract versions, app version and state schema are versioned separately. Record additive contract releases too. Incompatible shape, authority or observable behavior requires an explicit incompatible contract version even when descriptor shape is unchanged. Generated comparison helps but cannot prove behavioral compatibility. Schema 1 keeps empty permissions/no imports; successor manifests use a new schema older hosts refuse (`manifest.rs:166-169`).

### Identity migration

`ExtensionId` requires a lowercase dotted name with at least two segments, up to 64 characters (`id.rs:44-66`). A hyphenated UUID does not satisfy that shape. The successor needs a separate package-UUID field and an `aliases.toml` mapping from old dotted IDs. Saved projects that hold dotted IDs keep resolving through the alias.

## What this deliberately avoids

- A binary host, CDN or mirror.
- A build farm. Optional attestations are evidence, not a requirement.
- Login for public browse, install or use.
- A daemon or a second installer. The launcher and the in-app managers call the same client and store.
- A second contract schema. JSON inventories are generated from Rust registries.
- Desktop calls to the GitHub API.

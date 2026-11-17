# Store and distribution

How plugins are listed, signed, found, installed, reported and withdrawn. Each section says whether
it is **implemented** in source or **planned/deferred**. Nothing marked planned works today.

Main sources: `crates/ext-core/src/catalogue.rs` (format, freshness, search, resolution),
`crates/ext-core/src/listing.rs` (schema 1.1 listing fields, limits, category taxonomy),
`crates/ext-core/src/review.rs` (permission widening, lookalike names),
`crates/ext-store/src/catalogue.rs` (signature check, fetch, download, test key),
`crates/ext-store/src/store.rs` and `manager.rs` (install, revocations), `crates/ext-cli/src/catalogue.rs`
and `catalogue/submission.rs` (maintainer commands, submission checks), `crates/ext-core/src/tier.rs`
(tiers), `crates/ext-manager` (the [Plugin Manager model](manager.md)), and the catalogue repository
`craft-plugins/` (`README.md`, `SUBMITTING.md`, `MAINTAINERS.md`).

## Status at a glance

| Topic | Status |
|---|---|
| Signed catalogue (`index.json` + `index.json.sig`, ed25519) | Implemented |
| Listing schema 1.1 (description, categories, icon, screenshots, homepage, changelog, attestation, downloads, featured) | Implemented in source; no published listing uses it yet |
| `craft-ext catalogue check-submission` | Implemented in source; the CI workflow that runs it is described, not added |
| Build attestation (Built from source badge) | Implemented: recorded by online `catalogue build` and reported by `check-submission` |
| Package files bound by SHA-256 and size | Implemented |
| Serial rollback protection and 30-day expiry | Implemented |
| `craft-ext catalogue keygen / check / build / sign` | Implemented |
| Local test catalogues and the debug-only test key | Implemented |
| Vendor identity via GitHub (vendor files, numeric id, `io.github.<login>`) | Implemented in `catalogue check` |
| Tiers: official, verified, community, local, dev | Implemented |
| Anonymous browsing and installs | Implemented (there are no accounts) |
| Report a plugin (`plugin report` → GitHub issue URL) | Implemented |
| Kill list (`advisory`, `block-new`, `disable`) | Implemented |
| GitHub stars and release download counts in ranking | Implemented (fetched at online build time, dated) |
| User accounts and sign-in | Not implemented. No account sign-ins in the first release ([decision record](draft/decisions.md)) |
| Reviews and ratings | Not implemented and not designed. They would need accounts |
| TUF (threshold and offline root keys) | Deferred; post-beta target ([beta-design.md](beta-design.md#b8-catalogue-and-store-policy-craft-plugins)) |
| Web builds and web store integration | Deferred ([beta-design.md](beta-design.md#b12-evidence-limits-and-deferred-items)) |
| Automatic labelling of reports (bot or CI) | Not implemented. Triage is manual |
| Automatic labelling of submissions | The `check-submission` verdict is ready for a workflow to apply; no workflow exists yet |

## Catalogue format (implemented)

The catalogue is metadata only. Plugin archives (`.craftplugin`) stay on each author's GitHub
Releases. The default location is
`https://raw.githubusercontent.com/arnav-hiwarkar/craft-plugins/main/dist/`; `CRAFT_CATALOGUE_URL`
overrides it for mirrors and tests, and signatures still apply.

```text
dist/
  index.json            schema, schema_minor, serial, issued, expires, vendors, packages (one row each),
                        revocations, featured
  index.json.sig        hex ed25519 signature over the exact bytes of index.json
  packages/<id>.json    every listed release of one package, fetched on demand
  media/<id>/<file>.png icons and screenshots kept in the catalogue repository
```

- **One signature per snapshot.** Only `index.json` is signed. Each package row carries
  `file_sha256` and `file_size` of its `packages/<id>.json`; the app refuses a package file whose size
  or digest differs, so files from different publications cannot be mixed.
- **Keys.** Public keys are compiled into the app (`TRUSTED_KEYS`). Verification uses
  `verify_strict`. The array has room for a second key during rotation, but it holds one key today.
- **Releases.** A release records `version`, `tag`, `commit`, `published`, one or more `assets`
  (`platform` for native packages, `url`, `sha256`, `size`) and the exact `manifest` inside the
  archive. The download is refused unless its size and SHA-256 match; install also refuses an archive
  whose manifest differs from the record. One id+version maps to one digest forever.
- **Asset URLs, checked by the installing app.** Before anything is fetched,
  `catalogue::check_asset_url` requires a remote asset to be
  `https://github.com/<owner>/<repo>/releases/download/<tag>/<file>` of the package row's own
  `repo` (owner and repository compared case-insensitively, plain segments only: no `http://`,
  query, fragment, backslash, white space, percent escape or dot segment). A relative asset is plain
  segments under the catalogue base. A `file://` asset is an absolute local path, never a network
  share (`file:////host/…`), and still needs a `file://` catalogue or `CRAFT_CATALOGUE_ALLOW_FILE=1`.
  Admission checks the tag as well; the client check is defence in depth, carried over from the
  rules EffectCraft's retired Plug-in Library enforced.
- **Freshness.** `Index::parse` refuses a different schema, a `serial` lower than the highest the
  store has seen (rollback or replay), and an index past `expires`. `catalogue build` sets `expires`
  to 30 days after `issued`. A clock earlier than `issued` is refused too. An expired cache can still
  be browsed, but cannot authorise installs.
- **Caps.** Index 8 MiB and 20 000 packages; package file 4 MiB and 2 000 releases; 10 000
  revocations; release download 300 MiB.
- **Cache.** `Manager::refresh` fetches and verifies the index, then, under the store lock and
  after the serial check, writes `<store>/catalogue/index.signed` (the signature line, a newline,
  then the exact index bytes) in one atomic write, and records the serial, refresh time and
  revocations in `store-v2.json`. A reader never pairs one index with another's signature, and a
  slower, older refresh cannot replace a newer cache. Every later read re-verifies the signature;
  an older cache in the two-file layout is still read until the next refresh.

## Listing schema 1.1 (implemented in source)

Schema 1.1 adds optional listing fields. It is additive in both directions:

- **Old apps read new catalogues.** `SCHEMA` stays `1`. A 1.0 app refuses any index whose `schema`
  differs from `1` (`fresh` in `crates/ext-core/src/catalogue.rs`), so raising it would cut every
  installed app off the store. The 1.0 index, package row, release and package-file structs never
  used `deny_unknown_fields`, so a 1.0 reader ignores the new fields. The test
  `a_1_0_reader_reads_a_1_1_index` parses a 1.1 index with a copy of the 1.0 shape.
- **New apps read old catalogues.** Every new field has a serde default, so a 1.0 index parses with
  empty values (`a_1_1_reader_reads_1_0_and_tolerates_unknown_categories`).
- **`schema_minor`** records `1` for 1.1 and is absent (0) in 1.0. Empty new fields are not written,
  so a listing that uses none of them serialises exactly as in 1.0.
- **Forward tolerance.** An app drops category keys it does not know instead of refusing the index,
  so a later taxonomy addition stays readable. Contribution `kinds` are still a closed enum: a new
  contribution kind in the index would make 1.0 and 1.1 apps refuse it, so a new kind needs a
  schema decision of its own.
- **The index size cap is unchanged** (8 MiB, enforced by old apps too). Descriptions live in the
  index because search ranks them; `catalogue build` refuses an index over the cap.

| Field | Where | Limit, refused when exceeded |
|---|---|---|
| `summary` | package row (existing; `listing.toml` may override the manifest's) | 140 characters, one line, no invisible formatting |
| `description` | package row (`listing.toml`, else the manifest's) | 8 KiB |
| `categories` | package row | at most 2, each from the taxonomy and valid for one of the plugin's apps |
| `tags` | package row (existing) | at most 10, each at most 32 characters |
| `icon` | package row | PNG, square, at most 256 KiB, sides at most 4096 px |
| `screenshots` | package row | at most 6 PNGs, each at most 2 MiB |
| `homepage` | package row | an `https://` address |
| `license` | package row (existing) | not empty |
| `downloads` | package row and each release | `{count, date}`; the row sums the releases |
| `changelog` | release | 4 KiB |
| `attestation` | release | `{verified, signer_workflow, source_commit}`; the commit must equal the release commit |
| `featured` | index | at most 24 listed ids, each once |

An image is `{url, sha256, size, width, height}`. Its `url` is either a `.png` path inside the
catalogue (`media/<id>/<file>.png`, copied there by `catalogue build` from `plugins/<id>/media/`) or
a `.png` asset of the package's own GitHub release. The build checks a repository file's size,
digest and PNG dimensions; the manager checks them again after download (`Catalogue::media`).

Validation (`PackageSummary::check_listing`, `Release::check_listing`, `Index::check_listing`) runs
in `catalogue build`, `catalogue check` and `check-submission`. A bad value is refused with the field
named, for example `io.github.alice.test: icon: is 65x64; an icon must be square`; nothing is
clamped. Apps do not re-validate listing fields, so one maintainer mistake cannot hide the whole
store; the signed index is trusted as published.

The category taxonomy is one table, `CATEGORIES` in `crates/ext-core/src/listing.rs`:

| Key | Name | Apps |
|---|---|---|
| `effects-filters` | Effects & filters | EffectCraft, PhotoCraft |
| `generators` | Generators | all |
| `color-looks` | Color & looks | all |
| `text-titles` | Text & titles | all |
| `motion-animation` | Motion & animation | EffectCraft, FigCraft |
| `3d-particles` | 3D & particles | EffectCraft |
| `utilities` | Utilities | all |
| `panels-workflow` | Panels & workflow | all |
| `import-export` | Import & export | all |
| `ai-analysis` | AI & analysis | all |
| `resources` | Resources | all |

## Signing and key handling (implemented)

| Command | What it does |
|---|---|
| `craft-ext catalogue keygen --repo <dir> --out <file>` | Generates a random ed25519 seed, writes it as hex to a new file (mode 0600 on Unix; refuses to overwrite), and prints the public key for `TRUSTED_KEYS`. |
| `craft-ext catalogue check --repo <dir> [--offline] [--only <id>]` | Validates the repository (below) and prints the manual checklist. |
| `craft-ext catalogue check-submission --repo <dir> --id <id> [--offline]` | Runs the [automated submission checks](#automated-submission-checks-implemented-in-source) for one listing change and prints a verdict. |
| `craft-ext catalogue build --repo <dir> [--offline]` | Writes `dist/packages/<id>.json` first and `dist/index.json` last, with `serial` = previous + 1, and deletes any old `index.json.sig`. |
| `craft-ext catalogue build --local-assets <dir> --out <dir>` | Builds an unsigned test catalogue from local `.craftplugin` files (see [Local test catalogues](#local-test-catalogues-implemented)). |
| `craft-ext catalogue sign --repo <dir> --key <file>` | Checks that `index.json` parses and is fresh, then writes `index.json.sig`. Signs `<dir>/index.json` if present, else `<dir>/dist/index.json`. |

All accept `--json`. `keygen` requires `--repo` even though it does not read that directory.
`catalogue::run` validates this option before dispatching `keygen`; omitting it produces
`--repo is required`. Use the command above if an older maintainer instruction omits
`--repo`. The current `craft-plugins/MAINTAINERS.md` includes it.
These are source-backed parser facts, not evidence of a local key-generation run.

Key handling: the secret seed is never committed (the maintainer keeps it at
`~/.config/craft/catalogue.key`). Rotation or a leaked key means shipping an app update with the new
public key in `TRUSTED_KEYS`, then re-signing. The maintainer re-runs `build` and `sign` at least
every three weeks, because signed metadata expires after 30 days.

## The maintainer catalogue repository (implemented)

```text
craft-plugins/
  vendors/<github-login>.toml               login, github_id, name, tier, url?, verified_how?
  plugins/<id>/listing.toml                 id, vendor, repo, repo_id, tags, licence_reviewed?, monetisation_reviewed?,
                                            summary?, description?, categories?, icon?, screenshots?, homepage?,
                                            signer_workflow?
  plugins/<id>/media/                       icon and screenshot PNGs named by path in listing.toml
  plugins/<id>/releases/<version>.toml      core Release record, made by `craft-ext pack --release-record`,
                                            plus an optional changelog
  revocations.toml                          [[revocation]] entries: the kill list
  featured.toml                             ids = [...]: the curated featured list
  examples/pending/                         example listings excluded from builds
  .github/ISSUE_TEMPLATE/report.yml         the Report plugin form
  dist/                                     built and signed output
```

`catalogue check` and `build` refuse:

- a vendor file whose name differs from its `login`, a duplicate vendor, a listing directory that
  differs from its id, or a listing whose vendor does not exist;
- an id in a first-party namespace (`org.storytold.`, `com.artcraft.`, `ai.artcraft.` and others in
  `crates/ext-core/src/id.rs`) unless the vendor tier is official;
- an `io.github.<login>.…` id whose `<login>` differs from the vendor login (other domains only get a
  warning to review `verified_how`);
- unknown fields in a release record or asset (no trust or payment fields can be added; `changelog`
  is the only author field added in 1.1, and attestations and download counts are written by the
  build, never by the author);
- a listing field that breaks a [schema 1.1 limit](#listing-schema-11-implemented-in-source), a
  featured id that is not listed, or a media file whose size, digest or PNG size differs from its
  record;
- a release whose id, version or source differs from the listing, a non-OSI licence term, an asset
  URL that is not `https://github.com/<owner>/<repo>/releases/download/<tag>/<file>` of the listed
  repository and tag, or an invalid digest or size;
- monetisation wording in name, summary or description, unless the listing has a
  `monetisation_reviewed` note;
- removing a published version or changing a published asset digest.

Online (without `--offline`) it also checks with the GitHub API that the repository is public, its
numeric id equals `repo_id`, its owner's numeric id and login equal the vendor's `github_id` and
`login`, its detected licence equals the manifest (or `licence_reviewed` explains why GitHub found
none), the tag resolves to the recorded commit, and each asset downloads, unpacks and carries the
recorded manifest. `GITHUB_TOKEN` is used when set. It also records each release's download count
from the GitHub release (dated) and, when the listing names a `signer_workflow`, the release's build
attestation. A named workflow whose attestation does not verify stops the build. Offline builds keep
the counts and attestations already published.

What automation still cannot prove (free behaviour, sensible permission reasons, intent) stays on the
maintainer's checklist in `craft-plugins/MAINTAINERS.md`. Today one maintainer reviews and merges;
no CI runs the checks yet. The [submission checks](#automated-submission-checks-implemented-in-source)
are written so that a workflow can leave only labelled submissions for a person.

## Automated submission checks (implemented in source)

`craft-ext catalogue check-submission --repo <dir> --id <id>` checks one listing change against the
published `dist/` and prints a report: one row per check, the verdict (`pass`, `needs-review` or
`fail`), and any attestations it verified. A `fail` verdict exits non-zero. Every check that cannot
run (no network, `--offline`, `gh` missing, a GitHub API error) reports `fail`, never `pass`.

| Check | Fails | Needs review |
|---|---|---|
| `schema` | The listing, releases, sizes or media break the repository rules or the 1.1 limits | |
| `names` | The published index cannot be read | The name or id tail looks like another listing (case, `0`/`o`, `1`/`l`/`i`, `rn`/`m` and similar, edit distance 1 from 5 characters and 2 from 9) |
| `permissions` | | A new release adds a permission no earlier permission covers, adds a native helper, or turns a restricted helper into full access |
| `source` | Repository not public, owner or numeric ids differ, licence differs, tag not at the commit, or an asset's digest, size or manifest differs | |
| `immutable-release` | A new GitHub release is not immutable, so its assets could be swapped after listing | |
| `attestation` | `gh attestation verify --repo <repo> --signer-workflow <repo>/<workflow>` refuses an asset, names another commit, or cannot run; or a native listing, or one from a verified or official vendor, names no `signer_workflow` | |

New releases are those missing from the published `dist/packages/<id>.json`. A brand-new listing is
compared with nothing, so only a native helper marks it for review; its permissions are shown to every
person at setup.

The intended workflow (not added to `craft-plugins`; adding it needs the owner's approval):

1. On a pull request that touches `plugins/<id>/`, check out the pull request without running its
   code, install `craft-ext` from a pinned release and `gh`, and run
   `craft-ext catalogue check-submission --repo . --id <id> --json` with a read-only token.
2. On `fail`, fail the job and comment the failing rows. On `needs-review`, add the `needs-review`
   label and comment the rows. On `pass` for an existing community vendor, mark the pull request
   ready to merge.
3. A maintainer reviews only labelled pull requests. Signing stays a local maintainer step with the
   offline key: `catalogue build` (online, so it records attestations and download counts), then
   `catalogue sign`.

## Featured list (implemented in source)

`featured.toml` in the catalogue repository lists up to 24 ids by hand. `catalogue build` copies it
into the signed index as `featured`, refusing unknown or repeated ids. It is the one place human taste
enters ranking: the manager shows a Featured badge and can offer a featured shelf. It needs no
accounts.

## Makers and vendor identity (implemented)

A maker registers once by pull request with `vendors/<github-login>.toml`, including the numeric
GitHub account id from `https://api.github.com/users/<login>`. The numeric id, not the login, is the
identity: a login re-registered by a different account fails the online check. Plugins from a GitHub
vendor use the namespace `io.github.<login>.<name>`. Ids in another domain need a vendor whose
`verified_how` names that domain.

## Tiers (implemented)

Tiers are never read from a package. The host assigns them from where the package came
(`crates/ext-core/src/tier.rs`):

| Tier | Badge | Meaning |
|---|---|---|
| `official` | Official | Vendor flagged official by a maintainer (ArtCraft). Only official packages may use first-party id prefixes. |
| `verified` | ✔ Verified vendor | A maintainer checked the vendor's identity (verified GitHub organisation domain or a DNS TXT record `craft-plugins-vendor=<login>`) and set `tier = "verified"`. |
| `community` | Community | Any registered vendor whose listing a maintainer allowed. |
| `local` | Not reviewed (local file) | Installed from a file, or not in the catalogue. |
| `dev` | Development | Installed by `craft-ext dev` into the separate development store `<store>/development`. |

Badges beside the tier are derived by the catalogue build, never by the package. **Built from
source** means the newest release has a verified attestation (`attestation.verified`), recorded by
`catalogue build` after `gh attestation verify` proved that the listed repository's workflow built
those bytes at the release commit. The app never calls GitHub; it trusts the signed index and checks
the digests it already checks. Native: full access and Native: restricted come from the manifest.

Grants are bound to a package's origin. If a package with the same id arrives from a different origin
(for example a local file after a catalogue install), every app's grants, consent and enablement for
it are cleared (`Store::install_bytes`, `set_active`).

The CLI search ranks matches first, then official, verified and community vendors, then GitHub stars,
then id. The [Plugin Manager model](manager.md#search-and-sorting) ranks name over summary over tags
over description, then adds tier and download boosts. Stars and download counts are fetched by an
online `catalogue build` and shown with their date.

## Anonymous installs (implemented)

There are no app/store accounts. Vendor registration uses a GitHub identity, and filing a
report uses the reporter's GitHub account; neither is app sign-in. Browsing, searching and installing need only the public signed catalogue and
the author's public release asset. The app sends no identity to the catalogue.

## Reviews and ratings (not implemented)

No source implements user reviews or ratings, and the beta design does not specify them. They would
need accounts, which are not in the first release. GitHub stars and GitHub release download counts
are the only popularity signals.

## Reports and the kill list

### Reporting (implemented)

`<app>-cli plugin report <id>` (and the **Report plugin** link in the manager) returns a URL that opens
a prefilled GitHub issue in `arnav-hiwarkar/craft-plugins` with the `report.yml` form, the plugin id
and the installed version (`Manager::report_url`). The CLI prints the URL; it does not file anything.
Filing the issue needs the reporter's own GitHub account.

Triage is manual: `MAINTAINERS.md` asks the maintainer to add the `needs-review` label to reports
labelled `security` or `malware`, or with three reports from different accounts. No bot applies it.

### Revocations (implemented)

The maintainer adds a `[[revocation]]` to `revocations.toml`, then builds and signs. An entry
targets an id (optionally with a version range), a digest or a vendor; an entry with no target never
matches.

| Severity | Effect |
|---|---|
| `advisory` | A warning. Installs, enables and updates still work. |
| `block-new` | No new installs, enables, updates or rollbacks to matching releases. Already-active releases keep running. |
| `disable` | The kill list. Matching releases stop loading and running in every app: sessions stop, native helpers are killed, the package disappears from search. It also matches local installs with the same id or digest. |

Apps apply the list on their next catalogue refresh (at start-up and about every 6 hours; see
[capabilities.md](capabilities.md#catalogue-refresh-by-app)). The list is cached in `store-v2.json`,
so it survives restarts and metadata expiry.

## Local test catalogues (implemented)

For development and CI you can build and sign a private catalogue and point apps at it.

```sh
craft-ext catalogue keygen --repo . --out /tmp/test.key          # prints public_key
craft-ext catalogue build --local-assets ./assets --out ./catalogue
craft-ext catalogue sign --repo ./catalogue --key /tmp/test.key

export CRAFT_CATALOGUE_URL="file://$PWD/catalogue/"
export CRAFT_CATALOGUE_ALLOW_FILE=1
export CRAFT_CATALOGUE_TEST_KEY="PUBLIC_KEY_FROM_KEYGEN" # replace with printed public key
export CRAFT_EXT_DIR="$(mktemp -d)"                               # throwaway store
```

Rules (`crates/ext-store/src/catalogue.rs`):

- **Relative asset URLs.** `build --local-assets` writes a relative URL (for example
  `assets/a.craftplugin`) for each archive inside the output directory, and an absolute `file://` URL
  otherwise. Relative URLs resolve against the catalogue base, so the directory can be moved or copied
  as a unit. They must be plain path segments: no `..`, `.`, empty segments, `\`, `?`, `#` or `:`.
- **File assets.** `file://` asset URLs are downloaded only from a `file://` catalogue or when
  `CRAFT_CATALOGUE_ALLOW_FILE=1`.
- **The test key override.** `CRAFT_CATALOGUE_TEST_KEY` (64 hex characters) is added to the trusted
  keys only when **all** of these hold: the app is a **debug build** (`cfg!(debug_assertions)`; the
  override is compiled out of release builds), `CRAFT_CATALOGUE_ALLOW_FILE=1`, and the catalogue base
  starts with `file://`. It never applies to HTTPS catalogues and skips no other check: digests,
  sizes, freshness and serials are verified as usual.
- **Use a throwaway `CRAFT_EXT_DIR`.** The store records the highest catalogue serial it has seen and
  the catalogue's kill list. A test catalogue would leave its serial and revocations in your normal
  store, and later a lower-serial catalogue is refused as a replay. Start a new store when that
  happens.
- Local builds give every package a community vendor derived from its `io.github.<login>` id and
  delete any stale signature, so you must sign after each build.

[Run #20](https://github.com/arnav-hiwarkar/craft-extensions-plugin-beta/actions/runs/43551011455),
commit `909897f`, records catalogue installation in every declared app on Linux x86_64
and macOS arm64. This is CI evidence, not a full GUI check. See [TESTING.md](../../TESTING.md)
and [isolated beta installation](installation-and-testing.md).
The [formats reference](formats-and-versioning.md) owns format details; [workflow maps](maps.md)
trace installation, setup and updates.

## Planned or deferred

- **TUF** with threshold signing and offline root keys replaces the single online key after the beta.
- **Accounts**: none in the first release. Reviews and ratings are not designed.
- **Web builds** and a browser-local store are deferred.
- **Automatic cleanup** of unused releases is deferred. `Store::remove_unused_versions` exists in
  the store API, but no manager or CLI command calls it yet.

# Store and distribution

How plugins are listed, signed, found, installed, reported and withdrawn. Each section says whether
it is **implemented** in source or **planned/deferred**. Nothing marked planned works today.

Main sources: `crates/ext-core/src/catalogue.rs` (format, freshness, search, resolution),
`crates/ext-store/src/catalogue.rs` (signature check, fetch, download, test key),
`crates/ext-store/src/store.rs` and `manager.rs` (install, revocations), `crates/ext-cli/src/catalogue.rs`
(maintainer commands), `crates/ext-core/src/tier.rs` (tiers), and the catalogue repository
`craft-plugins/` (`README.md`, `SUBMITTING.md`, `MAINTAINERS.md`).

## Status at a glance

| Topic | Status |
|---|---|
| Signed catalogue (`index.json` + `index.json.sig`, ed25519) | Implemented |
| Package files bound by SHA-256 and size | Implemented |
| Serial rollback protection and 30-day expiry | Implemented |
| `craft-ext catalogue keygen / check / build / sign` | Implemented |
| Local test catalogues and the debug-only test key | Implemented |
| Vendor identity via GitHub (vendor files, numeric id, `io.github.<login>`) | Implemented in `catalogue check` |
| Tiers: official, verified, community, local, dev | Implemented |
| Anonymous browsing and installs | Implemented (there are no accounts) |
| Report a plugin (`plugin report` → GitHub issue URL) | Implemented |
| Kill list (`advisory`, `block-new`, `disable`) | Implemented |
| GitHub stars in search ranking | Implemented (fetched at build time, dated) |
| User accounts and sign-in | Not implemented. No account sign-ins in the first release ([decision record](draft/decisions.md)) |
| Reviews and ratings | Not implemented and not designed. They would need accounts |
| TUF (threshold and offline root keys) | Deferred; post-beta target ([beta-design.md](beta-design.md#b8-catalogue-and-store-policy-craft-plugins)) |
| Web builds and web store integration | Deferred ([beta-design.md](beta-design.md#b12-evidence-limits-and-deferred-items)) |
| Automatic labelling of reports (bot or CI) | Not implemented. Triage is manual |

## Catalogue format (implemented)

The catalogue is metadata only. Plugin archives (`.craftplugin`) stay on each author's GitHub
Releases. The default location is
`https://raw.githubusercontent.com/arnav-hiwarkar/craft-plugins/main/dist/`; `CRAFT_CATALOGUE_URL`
overrides it for mirrors and tests, and signatures still apply.

```text
dist/
  index.json            schema, serial, issued, expires, vendors, packages (one row each), revocations
  index.json.sig        hex ed25519 signature over the exact bytes of index.json
  packages/<id>.json    every listed release of one package, fetched on demand
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
- **Freshness.** `Index::parse` refuses a different schema, a `serial` lower than the highest the
  store has seen (rollback or replay), and an index past `expires`. `catalogue build` sets `expires`
  to 30 days after `issued`. A clock earlier than `issued` is refused too. An expired cache can still
  be browsed, but cannot authorise installs.
- **Caps.** Index 8 MiB and 20 000 packages; package file 4 MiB and 2 000 releases; 10 000
  revocations; release download 300 MiB.
- **Cache.** `Manager::refresh` fetches and verifies the index, writes `index.json.sig` and
  `index.json` into `<store>/catalogue/`, and records the serial, refresh time and revocations in
  `store-v2.json`. Every later read re-verifies the cached signature. Known limit: the two cache files
  are each written atomically but not as a pair; an interruption between them leaves a cache that
  fails verification until the next refresh repairs it.

## Signing and key handling (implemented)

| Command | What it does |
|---|---|
| `craft-ext catalogue keygen --repo <dir> --out <file>` | Generates a random ed25519 seed, writes it as hex to a new file (mode 0600 on Unix; refuses to overwrite), and prints the public key for `TRUSTED_KEYS`. |
| `craft-ext catalogue check --repo <dir> [--offline] [--only <id>]` | Validates the repository (below) and prints the manual checklist. |
| `craft-ext catalogue build --repo <dir> [--offline]` | Writes `dist/packages/<id>.json` first and `dist/index.json` last, with `serial` = previous + 1, and deletes any old `index.json.sig`. |
| `craft-ext catalogue build --local-assets <dir> --out <dir>` | Builds an unsigned test catalogue from local `.craftplugin` files (see [Local test catalogues](#local-test-catalogues-implemented)). |
| `craft-ext catalogue sign --repo <dir> --key <file>` | Checks that `index.json` parses and is fresh, then writes `index.json.sig`. Signs `<dir>/index.json` if present, else `<dir>/dist/index.json`. |

All accept `--json`. `keygen` requires `--repo` even though it does not use it
(`crates/ext-cli/src/catalogue.rs`); `craft-plugins/MAINTAINERS.md` shows it without `--repo`.

Key handling: the secret seed is never committed (the maintainer keeps it at
`~/.config/craft/catalogue.key`). Rotation or a leaked key means shipping an app update with the new
public key in `TRUSTED_KEYS`, then re-signing. The maintainer re-runs `build` and `sign` at least
every three weeks, because signed metadata expires after 30 days.

## The maintainer catalogue repository (implemented)

```text
craft-plugins/
  vendors/<github-login>.toml               login, github_id, name, tier, url?, verified_how?
  plugins/<id>/listing.toml                 id, vendor, repo, repo_id, tags, licence_reviewed?, monetisation_reviewed?
  plugins/<id>/releases/<version>.toml      core Release record, made by `craft-ext pack --release-record`
  revocations.toml                          [[revocation]] entries: the kill list
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
- unknown fields in a release record or asset (no trust or payment fields can be added);
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
recorded manifest. `GITHUB_TOKEN` is used when set.

What automation cannot prove (source-to-build provenance, free behaviour, sensible permissions,
impersonation) is the maintainer's manual checklist in `craft-plugins/MAINTAINERS.md`. One maintainer
reviews and merges; there is no bot, no CI and no second approval.

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

Grants are bound to a package's origin. If a package with the same id arrives from a different origin
(for example a local file after a catalogue install), every app's grants, consent and enablement for
it are cleared (`Store::install_bytes`, `set_active`).

Search ranks matches first, then official, verified and community vendors, then GitHub stars, then
id. Stars are fetched by `catalogue build` and shown with their date.

## Anonymous installs (implemented)

There are no accounts. Browsing, searching and installing need only the public signed catalogue and
the author's public release asset. The app sends no identity to the catalogue.

## Reviews and ratings (not implemented)

No source implements user reviews or ratings, and the beta design does not specify them. They would
need accounts, which are not in the first release. GitHub stars are the only popularity signal.

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
export CRAFT_CATALOGUE_TEST_KEY=<public_key>
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

The `plugin-beta` workflow builds such a catalogue with all 15 examples; it installs every package in
every app it declares (verified). See [TESTING.md](../../TESTING.md).

## Planned or deferred

- **TUF** with threshold signing and offline root keys replaces the single online key after the beta.
- **Accounts**: none in the first release. Reviews and ratings are not designed.
- **Web builds** and a browser-local store are deferred.
- **Automatic cleanup** of unused releases is deferred. `Store::remove_unused_versions` exists in
  the store API, but no manager or CLI command calls it yet.

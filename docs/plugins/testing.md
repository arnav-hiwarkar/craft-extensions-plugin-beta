# Shared plugin beta tests

Run `scripts/e2e.sh` from this repository. It requires Rust with the
`wasm32-unknown-unknown` target, Python 3, `flock`, and the maintainer signing key at
`~/.config/craft/catalogue.key` matching `craft_ext_core::catalogue::TRUSTED_KEYS`.
Author wasm builds use the debug profile with optimization level 1 so SDK code
fits the production fuel budget. It runs debug builds with `CARGO_BUILD_JOBS=4`, takes the shared `../.cargo.lock`,
and exercises fresh effect/panel/data projects, the three examples, signed catalogue
operations, permissions, updates, rollback, native helper consent and round trips,
revocation, local installs and project references. Temp stores are printed for inspection.
Panels/commands have no author-side automated runtime test; the native helper is
exercised by the host test suite. No app repositories are built.

The generated catalogue is at `../test-catalogue/` and its archives at `assets/`.
To use it in an app (or `craft-ext plugin --app effectcraft`):

```sh
export CRAFT_EXT_DIR="$(mktemp -d)"
export CRAFT_CATALOGUE_URL="file:///home/ash/projects/plugin-beta/test-catalogue/"
craft-ext plugin --app effectcraft refresh
```

Use `photocraft` or `figcraft` as appropriate. Installing never grants permissions;
use `plugin permissions <id>` and `plugin setup <id> --enable <contribution>
--grant <permission>` to make explicit choices. Native packages require
`--full-access`. The test catalogue includes an incompatible halftone 2.0.0 and a
disabled kill-test entry on purpose.

Build another local catalogue with `craft-ext catalogue build --local-assets <archives>
--out <catalogue>`, then `craft-ext catalogue sign --repo <catalogue> --key <key>`.
An optional `revocations.toml` in the archives directory supplies revocations.
Local builds assign Community tiers and zero stars. The e2e fixture adds synthetic
signed stars and a Verified tier for the `e2e` vendor to test ranking; these are test
values, not real vendor verification.
Online `catalogue check` continues to require GitHub Release asset URLs.

A `file://` catalogue allows local assets. An HTTPS catalogue refuses local assets
unless `CRAFT_CATALOGUE_ALLOW_FILE=1` is explicitly set for development. This flag
never skips signatures, metadata hashes, archive hashes, size limits or revocations.

FigCraft currently advertises no `raster.filter` contract (see
[its contract](contracts/figcraft.md)). The harness verifies that it refuses both
halftone versions with that explanation; only EffectCraft and PhotoCraft can install
halftone 1.0.0 today. FigCraft image-fill dispatch requires adapter work outside this
shared-repository test.

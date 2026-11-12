# Craft plugin system — beta design (2026-11-12)

Status: **authoritative for the beta implementation** (revised after the sol-high design review, same day;
review outcomes are recorded as B-decisions in draft/decisions.md). Where this file and older documents
(`architecture.md`, `system-guide.md`, `implementation/P*`, `draft/*`) disagree, this file wins; each
superseded point is listed in [draft/decisions.md](draft/decisions.md#beta-decisions-2026-11-12). The
long-term architecture keeps its direction (sandbox first, one operation path, contracts not app
versions, metadata-only store); the beta picks the simplest proven mechanism for each part.

## B0. Shape in one page

```text
craft-extensions (shared, one version line)
  craft-ext-core     pure types: ids, semver + ranges, manifest v2 (craft-plugin.toml), contributions,
                     permissions, contracts + compatibility, catalogue records + signature check,
                     project plugin references, SPDX/OSI licence check
  craft-ext-store    disk store v2: releases, per-app setup (enable, grants, pin), cleanup, revocations,
                     catalogue cache + fetch + download (feature `net`), `Manager` facade used by every UI/CLI
  craft-ext-host     wasmi runtime (one locked profile), guest ABI v1, permission-checked host calls,
                     the `AppApi` trait each adapter implements, native helper protocol (out of process)
  craft-ext-ui       the shared egui manager (Browse / Installed / Updates / Problems / Setup sheet /
                     project-open prompt); embedded by every app and the launcher
  craft-ext-cli      `plugin …` command library (embedded as `<app> plugin …`) + the `craft-ext` binary
                     for authors (new, build, check, test, pack, dev) and maintainers (catalogue …)
  craft-guest        author SDK (wasm side): params, filter entry point, session events, host calls, UI tree
  templates/, examples/, docs/plugins/{author-guide,user-guide,adapter-guide}.md

each app: one adapter crate (`<app>-plugins`) implementing AppApi + contract description; the old
          plugin system is deleted in the same change.
craft-plugins (catalogue repo, metadata only): vendors/, plugins/<id>/listing.toml + releases/*.toml,
          revocations.toml, policy; `craft-ext catalogue check|build|sign` produces dist/ (signed).
```

## B1. Package and manifest (one format)

- One human-written TOML file, `craft-plugin.toml`, is both the source manifest and the manifest inside
  the package (`.craftplugin` = zip). No second JSON form, no file list: the **archive digest** pinned
  by the catalogue (or recorded at local install) binds every byte. Unpacking keeps the existing caps
  (entries, sizes, paths, no symlinks).
- Contributions and parameters are declared in the manifest, not in a wasm custom section, so the
  catalogue, manager and adapters can show everything without running code (simpler than macros;
  `craft-ext check` verifies that a wasm module exports what the manifest promises).

```toml
schema = 2
id = "io.github.ink.grain"          # reverse-domain; io.github.<owner> is checked against the repo owner
name = "Ink & Grain"
version = "1.2.0"                   # semver
license = "MIT OR Apache-2.0"       # SPDX; store listing requires every term to be OSI-approved
source = "https://github.com/ink/grain"
summary = "Halftone and grain effects"
apps = ["effectcraft", "photocraft"]   # multi-app: list every app; contracts decide per contribution

[why]                               # author's reasons, quoted under the host's sentence
"network:api.example.com" = "Downloads halftone screens"

[[contribution]]
id = "halftone"
kind = "effect"                     # effect | command | panel | tool | event-hook | preset | lut | font | theme | data
contract = "raster.filter@1.0"      # contract + minimum major.minor; decides availability per app
module = "halftone.wasm"
label = "Halftone"
category = "Stylize"
permissions = []                    # required; per contribution, so a filter never inherits a panel's rights
optional_permissions = ["network:api.example.com"]
state_version = 0
[[contribution.param]]
id = "size"
label = "Dot size"
type = "slider"
default = 8
min = 1
max = 64
```

## B2. Versions and compatibility (contracts, not app versions)

- Packages use semver. Contracts use `name@major.minor`; the contract name+major is also the ABI
  version (no separate ABI number). An app **provides** a list of contracts
  (e.g. EffectCraft: `raster.filter@1.0`, `effectcraft.session@1.0`, `resource.preset@1.0`,
  `resource.lut@1.0`, `resource.font@1.0`, `craft-theme@1.0`). A package needs, per app,
  `{contract: "major.minor"}` = same major, provided minor ≥ needed minor.
- Every contribution names its contract and minimum level; it is available in an app only when that app
  provides it. Native packages also need a matching `os-arch` executable.
- One id+version = one digest forever: the store refuses a second archive claiming an installed
  identity, and the catalogue refuses duplicate versions.
  The result is **compatible / partly compatible (list of unavailable contributions with reasons) /
  incompatible (reason)** — never a silent half-load.
- Apps move forward without breaking plugins as long as the contract version they provide is unchanged
  or only gains minor versions. A contract major change runs side by side with the old one for 12
  months (unchanged policy).
- Resolution for install/update: newest release whose per-app needs are met by **this** app; if none,
  the message says why (`needs raster.filter 1.3, this EffectCraft has 1.2 — update EffectCraft`, or
  `version 2.x needs raster.filter 2; version 1.4.0 works here — install 1.4.0?`).
- Users may install any listed version, pin it per app, and roll back to any retained or listed
  release. Update always asks.

## B3. Permissions (declared, granted per app, enforced on every call, revocable)

- Permission string = `family:scope`: `document.read:{selection|document|all}`,
  `document.edit:{selection|document|all}`, `files.read:picked`, `files.write:picked`,
  `network:<https domain>`, `clipboard.read`, `clipboard.write`, `ui.window`, `app.events`.
  Panels, menus, commands, effects, own storage and logging are free.
- The **adapter** declares, for every operation it exposes, the permission it needs (computed from the
  real arguments). The host checks the grant on **every host call** before the adapter runs anything.
  Unknown op → denied; management/undo/settings ops are never exposed.
- Grants live in the store per (app, package), bound to the package's **origin** (catalogue vendor +
  repo id, or the local file digest): a local file never inherits a catalogue package's grants or tier.
- **Revocation across processes:** every grant change bumps the store generation. Each running app keeps
  a `GrantView` refreshed when the generation changes (checked on every host call — one cheap file
  metadata read throttled to 250 ms — and immediately after its own writes). The host checks the view
  on **every host call and again just before the adapter commits**; a revoked invocation is rolled
  back, its queued work cancelled and late results discarded. Data already sent over the network
  cannot be recalled; the sheet says so.
- **Captured scope:** the adapter captures the selection/document at invocation start; operations act
  only on captured targets and on revision-carrying handles, re-checked at commit.
- **Network:** host-made https requests only to granted hosts, redirects re-checked, private/loopback
  addresses refused after DNS, response size and time capped. **Files:** only through host pickers
  that return opaque tokens.
- Required permissions are granted on the setup sheet before enabling; optional ones are off until the
  user turns them on (sheet or first-use prompt from a user action). Native plugins have a single
  "full access to your computer" consent instead. CLI/projects never grant silently;
  `plugin grant <id> <perm> --app <app>` is the explicit CLI path. The sheet shows the union of the
  permissions of the contributions being enabled.
- Widened permissions on update keep the old release active in that app until accepted.

## B4. Runtime and ABI v1 (fixes R2 and R8)

- wasmi 2, one locked config: fuel metering, 1 MiB value stack, no threads, module ≤ 32 MiB, memory
  ≤ 512 MiB per instance (transforms) / 64 MiB (sessions), unknown imports refused. Fuel runs in
  slices; between slices the host checks deadline, cancel and the grant view.
- **ABI v1** (wasm32; all pointers and lengths are `i32` offsets into the guest's exported `memory`;
  the host bounds-checks every range with checked arithmetic; JSON is UTF-8):

| Export | Signature | Meaning |
|---|---|---|
| `memory` | memory | the guest's linear memory |
| `craft_alloc` | `(len: i32) -> i32` | scratch buffer for host→guest data; 0 = out of memory |
| `craft_reset` | `() -> ()` | frees all scratch; called by the host before every invocation |
| `craft_filter` | `(ctx: i32, ctx_len: i32, px: i32, w: i32, h: i32) -> i32` | transform (`raster.filter@1`) |
| `craft_event` | `(ev: i32, ev_len: i32) -> i32` | session event (`<app>.session@1`) |

| Import (`craft`) | Signature | Meaning |
|---|---|---|
| `log` | `(level: i32, ptr: i32, len: i32)` | to the Plugin console, rate-limited |
| `call` | `(op: i32, op_len: i32, args: i32, args_len: i32) -> i64` | sessions only: run one app op; ≥0 = result length, <0 = error code |
| `take` | `(dst: i32, len: i32) -> i32` | copy the pending result (or error JSON) into guest memory |

  Return codes: `0` ok, `-1` invalid input, `-2` permission denied, `-3` not found, `-4` busy/stale,
  `-5` cancelled, `-6` budget, `-7` internal. Allocation and reset run under the same fuel and memory
  limits. Scratch (from `craft_alloc`) belongs to the host until the next `craft_reset`; persistent
  session state lives in the guest's normal heap. **Transforms must not depend on invocation history**:
  the host may re-instantiate at any time (it does after a trap).
- **Host memory (R8):** pending results, call arguments, decoded resources and UI trees are charged to a
  per-instance byte budget and an app-wide plugin budget; reserve before allocating, over budget = `-6`.
- **`raster.filter@1`** (beta: spatial, whole-frame): `ctx` JSON = `{params, time (seconds, layer time),
  scale (buffer px per layer px), origin [x,y], full [w,h], quality: "draft"|"final", seed, color:
  "linear-srgb"|"srgb"}`; pixels = premultiplied RGBA `f32`, row-major, top row first, in the app's
  working space named by `color`, values may exceed 1.0 (HDR). Output is validated (finite, alpha in
  0..=1 after clamp policy) before use. Apps refuse rather than convert for unsupported modes
  (PhotoCraft CMYK/Lab/indexed: filter shown disabled with the reason). No temporal frame access in v1.
- **Sessions** (`<app>.session@1`): events `init`, `command`, `ui`, `tool` (begin/update/commit/cancel),
  `app` (subscribed, needs `app.events`), `dispose`, delivered in order on a per-plugin worker. Each
  `command` event or tool gesture is **one transaction → one labelled undo step**, rolled back on error,
  trap, cancel or revocation. External effects (network, files) run after commit and report failure
  separately. A document switch cancels a gesture; Escape cancels a tool unchanged.
- Errors: traps, budget exhaustion and invalid output roll back; 3 failures pause the plugin for the
  session. The host never panics because of a plugin.

## B5. Plugin UI (host draws everything)

- Plugins describe UI as a small JSON tree (`column`, `row`, `label`, `button`, `slider`, `checkbox`,
  `text`, `color`, `select`, `image`, `canvas`(draw list), `separator`) with stable widget ids.
  `ui.panel.set {panel, tree}` replaces a declared panel's content; `ui.window.open {id, title, tree}`
  opens a host-drawn floating window (needs `ui.window`); widget events come back as `ui` events with
  the tree revision so stale events are dropped (R12).
- Menus/commands/tools are contributions; the host places them after built-ins, grouped by plugin.
  Plugins cannot change built-ins, shortcuts, settings, undo, or plugin management.

## B6. Native plugins

- `native.executables = { "linux-x86_64" = "bin/helper", … }`, one archive per platform. Beta native
  kinds: **command, panel, event-hook, tool** (no native effects yet; pixel transport is deferred).
- Always **out of process**: the host starts the executable with a clean environment and speaks the
  session protocol as length-bounded JSON lines on stdin/stdout: `hello {protocol:1, app, contracts}`
  → `ready`, then `event` → and `call`/`result` pairs, with timeouts. The host still checks **per-app
  full-access consent, enablement and the kill list on every call** it serves, but the helper itself can
  do anything the user can: the manager shows **"Native: full access to your computer"** with its own
  consent; updates always ask; disable or a kill-list hit kills the process (which cannot undo what it
  already did).

## B7. Store on disk (fixes R3, R4)

- Root from `CRAFT_EXT_DIR` or the per-OS default. **New file names** so frozen schema-1 binaries never
  read or rewrite v2 data (R4): `store-v2.json` and `releases/<id>/<version>-<sha12>/`. Writes: stage +
  validate outside the lock, then under the lock move into place and atomically replace the state with
  generation+1; a newer state schema is refused, never rewritten.
- Records: `packages[id] = {origin, releases[version] = {sha256, tier, installed}}`,
  `apps[app][id] = {active, pending (awaiting consent), pin, enabled contributions, grants, full_access,
  grant_generation}`, `catalogue = {serial high-water, last refresh}`, `revocations` (cached kill list,
  kept even after metadata expires).
- Install never enables. Update: new release installed beside the old; each app moves its `active`
  only when its consent is complete (widened permissions/new native → `pending`). Rollback = choose any
  installed or listed version; if the contribution `state_version` is lower than data saved by the newer
  one, the manager warns and the project keeps the newer data untouched (placeholder) rather than
  corrupting it.
- **No automatic cleanup in beta** (retain everything; leases deferred). "Remove unused versions"
  is an explicit action that deletes releases no app has active, pending or pinned.
- Disable keeps grants; uninstall removes setups, grants and private storage `data/<app>/<id>/`.

## B8. Catalogue and store policy (craft-plugins)

- Metadata only. Files stay on authors' GitHub Releases, pinned by SHA-256 + size.
- **Vendors** register by PR: `vendors/<github-login>.toml` with the numeric GitHub user/org id.
  Tiers: **official** (ArtCraft org, maintainers only), **verified** (identity checked by a maintainer:
  GitHub org with verified domain or DNS TXT record; badge ✔), **community** (default).
- **Listing**: `plugins/<id>/listing.toml` (id, vendor, repo, numeric repo id, summary, tags) and
  `plugins/<id>/releases/<version>.toml` (url, sha256, size, tag, commit, per-app contracts,
  permissions, native, licence) — generated by `craft-ext pack --release-record`.
- **Free and open source only**, checked by `craft-ext catalogue check`: SPDX licence with every term
  OSI-approved; source repo public and its detected licence agrees; the release tag resolves to the
  recorded commit; id namespace matches the vendor; every platform asset downloads and its digest,
  size and manifest equal the record; no payment/licence-key fields exist in the schema; monetisation
  wording is flagged. Automated checks cannot prove the archive was built from that source or that the
  plugin behaves freely; the maintainer checklist (`craft-plugins/MAINTAINERS.md`) covers those
  questions by a quick look.
- **Allow flow**: anyone opens a PR; one maintainer runs `catalogue check`, merges, then runs
  `catalogue build` (fetches stars, writes `dist/index.json`, `dist/packages/<id>.json`,
  `dist/revocations.json`) and `catalogue sign` (ed25519). No bot, no CI, no second approval.
- **Signing (one signed snapshot):** only `index.json` is signed (`index.json.sig`, ed25519); it binds
  every `packages/<id>.json` by SHA-256 and size, so a snapshot is consistent and files from different
  publishes cannot be mixed. `build` writes package files first and the index+signature last. Apps
  store the highest `serial` seen (rollback refused) and refuse metadata past `expires` (30 days) for
  installs; a clock earlier than `issued` of seen metadata pauses installs. The maintainer re-runs
  `build`+`sign` at least every 3 weeks. Key compromise or retirement = new key in `TRUSTED_KEYS` via an
  app update (two slots allow overlap). This is **smaller than TUF** (no threshold/offline root);
  TUF stays the post-beta target.
- **Size (R5)**: the index carries one summary row per package; full release lists live in per-package
  files fetched on demand. Caps: index 8 MiB / 20 000 packages; package file 4 MiB / 2 000 releases.
- **Reports and kill list**: "Report plugin" (manager, launcher, `plugin report`) opens a prefilled
  GitHub issue (`report` template) in craft-plugins. Any report labelled `security`/`malware`, or three
  reports from different accounts, puts it on `needs-review`; the maintainer triages manually and may
  add an entry to `revocations.toml` (`advisory | block-new | disable`, by id + version range, digest or
  vendor). `block-new` stops install, enable, update and rollback to matching releases. `disable` (kill
  list) stops loading, ends sessions, fails final renders and kills native helpers in every app on the
  next refresh (at start and every 6 hours); it applies to matching **local** installs too (by id or
  digest) and is cached so it survives restarts and metadata expiry.
- **Search** (manager, launcher, CLI share one function): filter (query, app, kind, compatible, not
  revoked) → relevance tier (exact id/name, prefix, text) → official, verified → stars → id.

## B9. Projects

- A project stores a plugin list `[{id, version, sha256, source: catalogue|local, name}]` and each
  feature stores `{plugin, contribution, version, contract, state_version, params, state}`. Grants are
  never stored. Statuses: ready, needs enable, needs other version (incompatible / older), downloadable,
  unavailable (not listed, wrong platform, kill-listed — with the reason).
- Opening never downloads or enables. The adapter computes per plugin: ready / installed-but-off /
  not installed (in catalogue) / not available, and the manager shows one prompt
  "This project uses 3 plugins" with permissions and Download / Enable buttons.
- Missing → placeholder that keeps params/state byte for byte, preview passes through with a badge.
  Final export refuses only for missing contributions that **render** (effects); missing panels or
  commands never block output. The CLI prints the `plugin install` commands.

## B10. Management everywhere

- `craft_ext_store::Manager` is the single API (search, info, versions, install(version), update,
  rollback/pin, enable/disable, grant/revoke, remove, refresh, check-project). The egui manager, the
  launcher (all apps) and `<app> plugin …` / `craft-ext plugin …` all call it. The launcher is never
  required.

## B11. Authoring

- `craft-ext new <effect|panel|data> <id>` (template), `craft-ext build` (cargo build
  --target wasm32-unknown-unknown --release + copy), `craft-ext check` (validates the manifest and the
  module's imports/exports against its contract; it *warns* about ops a module may call but cannot prove
  complete permission use), `craft-ext test` (runs a transform on a test image in the reference host), `craft-ext
  pack` (zip + digest + release record); `<app>-cli plugin contract` renders the app contract.
- Each adapter publishes an `AppContract` value (contracts, ops with permission and JSON example,
  events, UI slots); `<app>-cli plugin contract` renders it as Markdown (committed as `docs/plugins/contracts/<app>.md`).

## B12. Evidence limits and deferred items

The beta is verified by `cargo check` and small unit tests only. Runtime behaviour, the hostile-module
suite, adapter conformance and real downloads are **unverified** until run; compiling is not a security
proof.

Deferred (named in STATUS.md):

TUF, leases across processes, OS confinement for native, account permission, themes activation
(theme contributions are stored and listed; apps apply them once craft-theme lands — one format only,
ext-core's palette is removed, R1), plugin object types, workflows, cross-plugin calls, web builds,
native effects and pixel transport, temporal frame access, tiling/region requests, leases.

# Beta implementation map

What exists in code for the beta described in [beta-design.md](beta-design.md), where it lives, and
which promises are only partly wired. Normative design: `beta-design.md`; decisions: B-01…B-17 in
[draft/decisions.md](draft/decisions.md#beta-decisions-2026-11-12). This page is a map, not a
substitute for those. Statuses are honest: "done" means the code path exists and is unit-tested at
most; no runtime, hostile-module, GUI or download evidence was produced (B12).

## Where the code is

| Where | Role | Key types / entry points |
|---|---|---|
| `crates/ext-core` | Pure data and validation; no disk, network or clock | `manifest::Manifest`, `contract::{ContractRef, Provided, Compat, Unmet}`, `permission::Permission`, `catalogue::{Index, PackageSummary, Revocation, search, resolve}`, `project::{PluginRef, Status}`, `tier::{VendorTier, Tier}`, `licence::{check_osi, monetisation_flags}`, `ui` tree, `version` |
| `crates/ext-store` | Shared disk store v2, grants, catalogue cache, one management API | `Store` (`store-v2.json`, `releases/<id>/<v>-<sha12>/`), `Manager` (search, install, update, rollback, pin, enable, grant, revoke, remove, refresh, check_project, report_url), `view::GrantView`, `state`, `catalogue`, `unpack` |
| `crates/ext-host` | Runtime shared by every app | `api::AppApi`, `api::AppContract` (+ `to_markdown`), `api::authorize`, `wasm::Filter`, `wasm::Session`, `native::Helper` |
| `crates/ext-ui` | Embeddable egui manager | `ManagerView` (Browse / Installed / Updates / Problems, setup sheet, versions), `ProjectPrompt`, `Setup` |
| `crates/ext-cli` | `plugin …` command library plus the `craft-ext` binary | `run_plugin` (embedded as `<app> plugin …`), `author::run` (`new`, `build`, `check`, `test`, `pack`, `dev`), `catalogue::run` (`check`, `build`, `sign`, `keygen`), `known_apps` |
| `crates/guest` | Author SDK, wasm side (ABI v1) | `filter!`, `session!`, `FilterCtx`, `Image`, `ui`, `call`, `Event` |
| `effectcraft/crates/plugins` (`effectcraft_plugins`) | EffectCraft adapter: contract, effects, sessions, projects, ManagerView, CLI | `contract.rs`, `effects.rs`, `runtime.rs`, `grants.rs`, `ops.rs` |
| `photocraft/crates/plugins` (`photocraft_plugins`) | PhotoCraft adapter: filters, session bridge, resources | `filter.rs`, `runtime.rs`, `document_thread.rs`, `resources.rs` |
| `figcraft/crates/plugins` (`figcraft_plugins`) | FigCraft adapter: contract and grants only (inactive) | `contract.rs`, `grants.rs` |
| `craft-launcher/crates/ui-egui/src/plugins.rs` | Launcher Plugins dialog, shared `ManagerView` for all apps | header button in `views.rs` (`Glyph::Plugins`); desktop only |
| `../craft-plugins` | Metadata-only catalogue repo | `vendors/<login>.toml`, `plugins/<id>/listing.toml`, `releases/<version>.toml`, `revocations.toml`, `dist/index.json(.sig)`, `MAINTAINERS.md` |

## Workflows, end to end

**Plugin maker**

1. `craft-ext new effect my-effect --id io.github.<login>.<name>` (or `panel`, `data`) — writes from
   `templates/`.
2. Declare contributions, parameters and permissions in `craft-plugin.toml`; write the module
   (`filter!` or `session!`).
3. `craft-ext build` — `cargo build --release --target wasm32-unknown-unknown`, copies the `.wasm`.
4. `craft-ext check` — manifest, files, ABI exports, licence; warns that permission use needs review.
5. `craft-ext test` — runs each effect twice on a test image: deterministic, finite, valid alpha.
6. `craft-ext dev . --app effectcraft` — installs into the development store, labelled "Development".
7. `craft-ext pack --release-record --tag v1.2.0 --commit <sha> --url <asset URL>` — zip, digest,
   release record; upload the `.craftplugin` to that GitHub Release.
8. Open a PR to `craft-plugins`: `vendors/<login>.toml` once, then `listing.toml` +
   `releases/<version>.toml`; run `craft-ext catalogue check --repo . --only <id>`.
9. One maintainer checks and merges; the release appears after the next `build` + `sign`.

**Store maintainer**

1. `craft-ext catalogue check --repo . --only <id>` — schemas, vendor/namespace, OSI licence, public
   source and licence agreement, tag→commit, asset size/digest/manifest, paid wording.
2. Quick look at the parts automation cannot prove (`craft-plugins/MAINTAINERS.md`): plausibility,
   freedom, honest permissions, impersonation.
3. Merge the PR.
4. `craft-ext catalogue build --repo .` — fetches stars, writes `dist/packages/<id>.json`,
   `dist/revocations.json`, then `dist/index.json`.
5. `craft-ext catalogue sign --repo . --key <key>` — ed25519 over `index.json`; commit `dist/`.
6. Repeat build + sign at least every 3 weeks (metadata expires after 30 days).
7. Reports: the "Report plugin" form opens a `craft-plugins` issue; `security`/`malware` or three
   distinct reports get `needs-review`; triage, add `[[revocation]]` (`advisory` / `block-new` /
   `disable`) to `revocations.toml`, then build + sign again.

**End user**

1. Search: app **Plugins** window → Browse, launcher → Plugins, or `effectcraft-cli plugin search …`.
2. Install: **Install** (or `plugin install <id>`, or `plugin install-file ./x.craftplugin`); nothing
   is enabled by installing.
3. Setup sheet: pick contributions, accept required permissions, optionally turn on optional ones,
   read the author's `[why]` reasons; native shows "full access to your computer" consent.
4. Use: effects under the app's effects/filters (PhotoCraft: Filter ▸ Plugins), panels and windows,
   commands, data add-ons.
5. Update: **Updates** tab or `plugin updates` / `plugin update <id>`; always asks; widened
   permissions keep the old release active until accepted.
6. Roll back or pin: `plugin rollback <id> <version>`, `plugin pin <id> <version>` / `plugin unpin`.
7. Revoke: Installed tab or `plugin revoke <id> <permission>`; running work is cancelled and rolled
   back.
8. Remove: Installed tab or `plugin remove <id>` (disable keeps grants).
9. Trouble: **Problems** tab; `EFFECTCRAFT_SAFE_MODE=1` (or the app's `*_SAFE_MODE`) starts with no
   plugin code; **Report plugin** or `plugin report <id>`.

**Project sharing**

1. Save writes the project's plugin list (`{id, version, sha256, source, name}`) and feature refs; open
   calls `Manager::check_project` and installs/enables nothing.
2. `ProjectPrompt` shows one dialog: "This project uses N plugins" with permissions and
   Download / Enable / Skip.
3. Download or enable happens only on that button; grants are never stored in the project.
4. A missing plugin leaves a byte-for-byte placeholder and a preview badge; final export refuses only
   for missing rendering contributions (effects), never for panels or commands.
5. Headless: `effectcraft-cli plugin project <file>` prints statuses plus the `plugin install` and
   `plugin setup` commands.

**App maker**

Follow the eleven-item checklist in [adapter-guide.md](adapter-guide.md): delete the old plugin
system, publish `contract()`, implement `AppApi` (`begin`/`call`/`finish`) and `Grants` over
`GrantView`, register enabled contributions, convert pixels honestly, dispatch sessions on the
document thread, wire projects/`ProjectPrompt`, embed `ManagerView` and `run_plugin`, honour
`<APP>_SAFE_MODE=1`, publish `docs/plugins/contracts/<app>.md` and add the app to
`ext-cli/src/known_apps.rs`.

## Brief checklist

| Requirement | Where implemented | Status |
|---|---|---|
| Native and wasm plugins | `ext-host::wasm` (`Filter`, `Session`), `ext-host::native::Helper` (JSON-lines, out of process) | Done in code; native helpers unrun (`CONFORMANCE`) |
| Permissions enforced and revocable | `ext-core::permission`, `api::authorize` on every call, `GrantView` generation refresh, commit re-check | Done in code; runtime unverified (`CONFORMANCE`) |
| SDK and adapters | `crates/guest`; `effectcraft-plugins`, `photocraft-plugins`, `figcraft-plugins` | Partial — FigCraft adapter is contract+grants only (`SESSION-REGISTRY`, `PANEL-UI`, `MANAGEMENT`) |
| Windows, panels, effects, tools, commands, data | `ext-core::ui`, `wasm::Session`, contribution kinds; EC/PC effects, panels, windows, presets, LUTs | Partial — tool gestures unwired (`TOOL-GESTURES`); fonts/themes stored only (B15) |
| Events | `app.events`, `craft_event`, event hooks; EC and PC deliver ordered events | Done in code for EC/PC; FigCraft pending (`SESSION-REGISTRY`) |
| Contract docs, templates, tooling | `docs/plugins/contracts/*.md`, `templates/{effect,panel,data}`, `ext-cli::author`, `AppContract::to_markdown` | Done — each app prints its contract with `<app>-cli plugin contract`; pages committed under `docs/plugins/contracts/` |
| Catalogue metadata-only | `craft-plugins`: listings + release records, files on author Releases, digest-pinned | Done |
| Vendors | `craft-plugins/vendors/<login>.toml` with numeric GitHub id; namespace check | Done |
| Tiers official / verified / community | `ext-core::tier` (`VendorTier`, `Tier` + Local/Dev), catalogue vendor records | Done |
| Stars | `craft-ext catalogue build` fetches stars; `stars_date` shown | Done (network path unrun) |
| Search everywhere | `ext-core::catalogue::search` via `Manager::search`; `ManagerView`, launcher, `plugin search` | Done |
| Reports and kill list | `Manager::report_url`, `plugin report`, issue form; `revocations.toml`, `Revocation::matches` (id/version/digest/vendor) | Done; 6-hour refresh scheduling is `B8-REFRESH` |
| FOSS enforcement | `licence::check_osi`, `monetisation_flags`, `catalogue check` (licence, source, tag→commit, assets) | Done |
| One-maintainer allow flow | `MAINTAINERS.md`, `catalogue check` → merge → `build` → `sign` | Done |
| Semver, pin, rollback | `ext-core::version`, `Manager::{versions, rollback, pin, unpin}` | Done |
| Contract-based compatibility | `ContractRef`/`Provided`/`Compat`; `Unmet` renders "needs … update the app" | Done |
| Multi-app packages | `Manifest.apps`, per-contribution contracts, per-app setup in `ext-store` | Done |
| Clear incompatibility UX | `Compat::{Partly, Incompatible}` reasons in the setup sheet, versions list and `plugin info` | Done in code; GUI unverified |
| Projects record plugins | `ext-core::project`, EC save/open + `ProjectPrompt`, `plugin project` | Partial — EC `B9-FEATURE-STATE`; FigCraft `PROJECTS`; PhotoCraft needs none today |
| No auto-download or auto-enable | Install never enables; `check_project` is read-only; prompt buttons only | Done |
| Safe degradation | Placeholders, preview pass-through, export refusal, safe mode, rollback, pause after 3 failures | Partial — FigCraft `SAFE-MODE`; EC `B9-FEATURE-STATE` |
| Manage without the launcher | `ManagerView` embedded in each app; `<app>-cli plugin …` | Partial — EC and PC wired; FigCraft `MANAGEMENT` |

Shared deferrals unchanged: theme activation, native effects and pixel transport, temporal access,
tiling, TUF, process leases (B12). Runtime, hostile modules, revocation races, real downloads and GUI
interaction remain unverified; compiling is not a security proof.

# Upstream integration ledger, 2026-11-17

Requirement and evidence ledger for the Mac continuation (Claude Opus 5.5). It follows the
[continuation record](continuation-2026-11-17.md) and the [suite replacement rule](suite-replacement-rule.md).
"Source-complete" means committed after source and diff review; "verified" names the check that ran.
Nothing here was built by CI, run as an app or clicked through.

## Pinned upstream refs

Each upstream `main` was fetched read-only into `refs/upstream-ro/main` (no remote added, nothing
pushed upstream) and merged on a local integration branch, then fast-forwarded and pushed to both
private remotes.

| Repo | Upstream main | Merge base | Upstream-only / beta-only commits | Beta head after merge |
|---|---|---|---|---|
| craft-extensions | `964c715` | `a6c5fae` | 11 / 77 | `4b6086f` (then `6b8239e`) |
| craft-launcher | `b2de8c8` | `66beafa` | 54 / 5 | `7d3be5d` |
| effectcraft | `aeedb92` | `fbfeed2` | 375 / 17 | `32a514f` |
| figcraft | `e188356` | `0d6ce11` | 292 / 21 | `cc75013` (then `af0c4ba`) |
| photocraft | `01b7995` | `f75cfd5` | 1150 / 19 | `31d8774` |

`craft-plugins` has no upstream. Every app and the launcher use the same egui fork
(`storytold/egui@6d518fc`) on both sides; the merges changed no egui pin. Each `Cargo.lock`
is upstream's lock re-resolved with `cargo metadata`, which only added the `craft-ext-*` path
packages and their hashing, signing and zip dependencies (and removed retired legacy crates).

## Changed domains and resolutions

- **craft-extensions.** Upstream added the architecture/decision/topic/gap documents (already
  preserved byte-for-byte under `sources/github-main`) and CI hardening. Beta keeps its pointer docs
  and keeps upstream `ci.yml` out; this beta runs only `plugin-beta.yml`.
- **Launcher.** Upstream added the AI agents/MCP gateway, installer and native fixes and a Ukrainian
  catalogue. The MCP gateway is general automation, not a plugin system, and is unchanged. The
  catalogues keep both sides' rows; `uk.tsv` gained the four Plugins rows.
- **EffectCraft.** Upstream hardened its legacy Plug-in Library (release-asset and repository URL
  rules, no UNC `file:` URLs). The library stays retired (`system/library.rs`, its tests,
  `publishing-plugins.md`, `examples/plugin-catalog`); its rules moved into the shared store
  (below). Viewer: upstream's grab list is computed every frame while native handle hooks stay
  gated off during a plugin tool gesture. Effects re-export upstream's Cineon helpers beside the
  plugin resource resolver.
- **FigCraft.** Upstream added legacy-runtime privacy tests (plugins get names, never paths or
  library links). The retired runtime's tests stay out; the rule was checked against the adapter
  (below). Upstream's `run_script` exit-code assertions were kept with the beta `plugins` flag.
- **PhotoCraft.** Upstream moved `cms`, `codecs` and `affinity` into craft-libs; `photocraft_cms`
  became `craft_cms` (fixed in `plugin_filters.rs`), `photocraft_codecs` stays a package alias. The
  legacy `plugin_cmds` and `menus/plugin_ui.rs` stay removed, with upstream's legacy registry guard
  and install test. A scripted check found no `crate::` module path in beta-touched files that the
  merged crates lack.
- **App workflows.** Upstream workflow files (including FigCraft's new `lockfile.yml`) stay out of
  every app repository, as in the beta start commits.

## Capabilities learned from upstream and carried into the platform

| Lesson (upstream source) | Disposition | Status |
|---|---|---|
| Release assets only from the package's GitHub repository; no UNC `file:`; no dot/encoded segments (EC Plug-in Library) | `craft_ext_store::catalogue::check_asset_url` before every download; `get()` refuses `file:////host` and backslashes | Verified: `cargo test -p craft-ext-store --features net --lib` (28 passed) and `cargo check -p craft-ext-cli --all-targets --all-features` |
| Plugins get names, never paths, library links or other owners' data (FigCraft `doc_list_paths_tests`) | Fig adapter layer reads (`node.get`, `node.children`, `page.list`) drop `plugin_data` | Source-complete; new adapter test not yet compiled |
| One base64 decoder (PhotoCraft) | Beta PhotoCraft plugin code already uses the `base64` crate | No change needed |
| Menu-structure tests must not race plugin installs (PhotoCraft) | Beta menus read the shared store; no ui-egui test writes it, so the legacy guard was dropped | Source review |

## Temporary references with removal criteria

- PhotoCraft `crates/automation/src/workspace.rs` still names `plugin.install` and `plugin.reload`
  in its ambient-path refusal list. Those commands no longer exist, so the entries refuse nothing
  and grant nothing. Remove them when upstream drops them or when the adapter is upstreamed.
- Historical changelog rows in FigCraft's parity docs describe the retired runtime; they are dated
  history, below the migration note.

## Not verified

- No app (EffectCraft, PhotoCraft, FigCraft, launcher) was compiled after the merges; beta code
  against upstream's renamed or reshaped APIs is checked only by source review and scripted path
  checks. A compile or CI run needs its own authorization.
- `cargo clippy -D warnings` on `craft-ext-store` reports four findings in `manager.rs` and
  `store.rs` that predate this work (newer Rust 1.97 lints).
- Recorded run #20 evidence predates every commit listed here.

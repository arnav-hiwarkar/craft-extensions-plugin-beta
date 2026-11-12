# Readiness evidence index

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Recorded 2026-11-08. This lists every source revision used in the readiness phase, who read it, what the coordinator checked independently, and what was not done. All evidence is source reading of committed objects. No build, test, clippy, fuzz, benchmark, app launch, fetch or package execution ran. No app checkout, branch or working tree was changed.

## Workspace

| Item | Value |
|---|---|
| Worktree | `feat/artcraft-plugin-system` at `fd5e2feff6ea25bed371a99cc732b3968d621505` |
| Shared crate read by the coordinator | `crates/keymap/src/keymap.rs` (`Policy` default `Steal` at 333-341; `Keymap::new` first-wins at 497-534; `ConflictKind` at 436-441) |

## Applications

All clones are at `/Users/ash/Projects/<app>` on `main`. Each HEAD equals the baseline in the [earlier inventory](../../plugin-functional-inventory.json). EffectCraft's working tree has uncommitted changes, which no one read.

| App | Revision | Audit |
|---|---|---|
| EffectCraft | `ce9eaec65015a5508c72af0e2351cd02612c9450` | [media](haiku-media-apps.md) |
| PhotoCraft | `655295b2a2c400bded0640f5d97367000a057b2e` | [media](haiku-media-apps.md) |
| FilmCraft | `32749e34f611735460e9773335345db026b7b037` | [media](haiku-media-apps.md) |
| LightCraft | `ac1799db38661e07a855653f86feb3e66081102e` | [media](haiku-media-apps.md) |
| ArtCraft | `1f969a9224934a55417daf10359a6fb1458b63f4` | [media](haiku-media-apps.md) |
| FigCraft | `0e86f3597a7443b15c93ec3ab2091202f8802c95` | [design](haiku-design-apps.md) |
| VectorCraft | `065a646799f1f20f5454845d6648e09a5252c452` | [design](haiku-design-apps.md) |
| PDFCraft | `14ae839a28902a17b9c6adba68d820eb922544c4` | [design](haiku-design-apps.md) |
| DesignCraft | `1256decbeb4a49c6fbe94400efc1e2a838de23ce` | [design](haiku-design-apps.md) |
| DeckCraft | `0c00f2240352b17cba571e93573af48e017a058d` | [design](haiku-design-apps.md) |
| SoundCraft | `da02c19fcca8fbcc49d66c957e5efcc67a63befe` | [domain](haiku-domain-apps.md) |
| CADCraft | `b04273e8974245e1c59ec57eb5ad86cec7a200eb` | [domain](haiku-domain-apps.md) |
| SolidCraft | `4bdffef06180fb3a2e8fbd4b15c7d3f60e55bd11` | [domain](haiku-domain-apps.md) |
| TextureCraft | `9dc33d894e9cd8e25ce84f6074fd1b476dad5698` | [domain](haiku-domain-apps.md) |
| WordCraft | `ffb155a0321bba08da495efda66495104c1c8b9a` | [domain](haiku-domain-apps.md) |
| GridCraft | `178f3eb745a589c63b61ef97dd7b6b2d6fd48515` | [domain](haiku-domain-apps.md) |

Each audit's last section lists every file and line range its worker read.

## Extension infrastructure (read by the coordinator)

| Repository | Revision | Files and findings |
|---|---|---|
| craft-extensions, handover branch | `ffd8512e5a3755637f58cf31cfcf2fdb09cfb943`. Not merged into local `main` (`a6c5fae`). Three commits ahead of it | `ext-store/src/state.rs` (record fields 22-47; unknown fields ignored, test at 173-178); `ext-store/src/store.rs` (install enables every listed app 215-219; old release deleted 229-231; first-party-only replacement guard 196-207; theme activation separate 273-295); `ext-store/src/reader.rs` (`installed_for` checks only enablement, 58-63); `ext-core/src/tier.rs` (`admit` 52-58, `check_id` 61-66); `ext-core/src/manifest.rs` (schema 1, permissions must be empty 181-183, plugin files limited to module and thumbnail 45); `ext-core/src/index.rs` (unsigned, version and expiry 1-6, revocation `id` + `below` 36-43, entry `tier` 28); `ext-core/src/id.rs` (dotted ID shape 44-66, first-party prefixes 35-36); `docs/THREAT-MODEL.md` (gate list headings) |
| craft-launcher, extensions branch | `6ede77f93fd0d3eaaad43d430f94d703d3564a51`. Not merged | `apps/artcraft-launcher/src/extensions.rs:79` installs only as `Tier::FromFile`. The diff against `main` adds UI and demo data, with no network catalogue for extensions |
| craft-launcher, local `main` | `c9c3acab542f962150ffc78b2d38a59e7a342c82` | `crates/native/src/install.rs:621-659` (download refused without a published checksum, which comes from the feed, GitHub's digest or `SHA256SUMS.txt`); `crates/native/src/http.rs:227-260` (resume and verification); `crates/core/src/release.rs:245` (reads GitHub `digest`); `crates/core/src/feed.rs:26` (app-release feed URL); `crates/native/src/backend.rs:680-830` (feed fetch and back-off) |
| EffectCraft Plug-in Library | `ce9eaec` | `crates/engine/src/system/library.rs:1-40` (reviewed `index.json` from `storytold/effectcraft-plugins`, GitHub host allowlist, offline cache, digest per install); `46-135` (catalogue, release and installed record fields); `docs/plugins.md` (Plug-in Library and sandbox sections) |

The existence and current content of `storytold/effectcraft-plugins` on GitHub were not checked. Only the constant in source was read.

## Coordinator spot checks of worker citations

The coordinator re-read these lines at the stated revisions and confirmed they say what the audits report:

| Claim | Location |
|---|---|
| EffectCraft drops unresolved effects at render | `crates/render/src/pipeline.rs:622-632` |
| EffectCraft failing plugin returns the input frame | `crates/effects/src/plugin.rs:680-690` |
| FigCraft reinstall replaces and removes settings | `crates/engine/src/extensions/plugins.rs:460-478` |
| FigCraft control `engine.execute` calls `execute_as` | `crates/ui-egui/src/control.rs:1422-1432` |
| PhotoCraft `replace` defaults to true | `crates/engine/src/cmd/plugin_cmds.rs:200-208` |
| PhotoCraft smart filter skipped on missing plugin | `crates/engine/src/cmd/plugin_cmds.rs:150-159` |
| VectorCraft CLI attaches the plug-in store | `apps/vectorcraft-cli/src/main.rs:205-213` |
| CADCraft LISP writes to a caller path | `crates/engine/src/lisp/builtins/files.rs:98-108` |
| WordCraft registry is last-wins | `crates/engine/src/lib.rs:145-153` |
| PhotoCraft and TextureCraft `Filesystem` caller modes | PhotoCraft `crates/automation/src/headless.rs:30-60`; TextureCraft `crates/automation/src/headless.rs:14-24` |
| `craft-keymap` `Policy` defaults to `Steal` | `crates/keymap/src/keymap.rs:333-341` (this worktree) |

Other worker citations were not re-checked by the coordinator.

## External primary sources

| Claim | Source |
|---|---|
| Immutable releases lock assets and tag, and generate a release attestation | [GitHub Docs: immutable releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases) |
| GitHub computes a SHA-256 `digest` for release assets at upload | [GitHub changelog, 2025-06-03](https://github.blog/changelog/2025-06-03-releases-now-expose-digests-for-release-assets/); [release asset API](https://docs.github.com/en/rest/releases/assets) |
| REST rate limits | [GitHub rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api) |
| Repository star count field | [Repositories API](https://docs.github.com/en/rest/repos/repos) (cited, not re-retrieved in this phase) |
| TUF roles, rotation, freeze and rollback protection | [TUF specification](https://theupdateframework.github.io/specification/latest/) (reused from the draft) |

No TUF library, signing tool or licence was selected or audited.

## Orchestration record

| Item | Value |
|---|---|
| Coordinator | Opus 5.5, medium effort |
| Workers | Three, through `delegate_task`, provider `claudeAgent`, model `claude-haiku-5-5`, option `effort: xhigh`, verified with `orchestrator_capabilities` before launch, shared checkout, research role |
| Worker outputs | Each wrote only its own audit file. All three reported no agents started, no builds or tests, and no app changes. The media worker wrote and deleted one temporary tally file outside the repository |
| Further descendants | The root explicitly authorized three Haiku children of the Opus coordinator. Workers started no agents; no further delegation occurred |

## Root verification and reconciliation

The root independently compared committed local HEADs for all sixteen inventory apps; every revision still matched the baseline, and ArtCraft is present in that inventory. No app refs or working trees were changed. Root source reads confirmed EffectCraft's catalogue/installed-record definitions, PhotoCraft's actual registry removal, FigCraft's existing command/read guards, VectorCraft's built-in-first format resolution, GridCraft's test helper and committed cross-crate test callers, and `craft-keymap` refusal/steal variants.

[The root assessment](assessment-and-first-work.md#root-corrections-to-the-coordinator-findings) records the qualifications. Original worker notes remain intact, including the incorrect media-note statement that the earlier inventory lacks ArtCraft. The coordinator summaries and work packages have been narrowed where those claims affected recommendations.

The root also retrieved the official TUF, GitHub release-asset and repository API pages for update-policy and API/credential/ranking details. No TUF client/dependency or new signing mechanism was selected. Publication authentication, asset identity and optional private credentials remain separate concerns.

Documentation checks and their exact scope/results are in [readiness verification](verification.md). No runtime or exploitability check follows from this source reconciliation.

## Not done

- No runtime behaviour was observed. Fresh-CLI loading, silent export loss and channel authentication are source readings.
- Not traced: several control-channel dispatch targets in the domain apps; GUI wiring in PDFCraft, DesignCraft and DeckCraft; FigCraft widget callbacks; EffectCraft's script engine and export writers; LightCraft's export and sidecar format; ArtCraft's Tauri handler bodies; SoundCraft plugin host internals; the SolidCraft kernel.
- Operation counts are static grep counts.
- Historical task statuses in the [Jeremy handoff](../sources/jeremy-extensions-todo.txt) were not re-run and are not current evidence.

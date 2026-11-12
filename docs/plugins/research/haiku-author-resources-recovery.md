# Author experience, resources and recovery

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: design research, 2026-11-08. Recommendations are marked **R** and are not agreed. Nothing here approves a schema, implementation, catalogue, key, or release.

Scope: themes, plug-ins, and resource add-ons. Organisation and team management are excluded. The third-party release gate (sg13) stays closed, so third-party-facing flows below are design only.

Read first: [plugins index](../README.md), [decision record](../../plugin-functional-decisions.md), [security proposal](../../plugin-security-design.md), [source record](../source-and-verification-record.md), [aspect register](../aspect-register.md), [complete Jeremy artifact](../sources/jeremy-extensions-todo.txt).

## 0. Summary

1. **The baseline is real but thin for authors.** Schema 1 (`craft-extension.json`) has a publisher display name, no licence, no resource type, no contribution list, and no publisher identity beyond text. (S: `craft-extensions` at ffd8512, `crates/ext-core/src/manifest.rs` L83–98.)
2. **Authors type the same facts several times.** EffectCraft's version lives in the crate manifest, the embedded plug-in manifest, the release tag and the catalogue entry, and the checksum and size are computed by hand. An SDK should derive these. (D: `effectcraft` at ce9eaec, `docs/publishing-plugins.md` sections 1, 6 and 7.)
3. **Three theme layers must stay separate.** The package palette is data, `WidgetPalette` is five egui fills rather than a schema, and each app owns its tokens. FilmCraft needs a dark/medium/light mapping that the package's single `dark` flag cannot express.
4. **The ThemeCraft round trip is not safe yet.** Unknown fields are dropped, same-version reinstall is refused, a 64-token-per-app limit sits below FilmCraft's 82 colour fields, and in PhotoCraft an opaque override flattens a translucent token.
5. **Resources have no type, and workflows and model readiness have no contract.** Each resource type needs its own importer and fuzz gate before it ships.
6. **Recovery has the largest gaps.** A crash between the folder move and the state save can leave a folder that blocks retry with a generic message (source-derived; not reproduced). A damaged `state.json` blocks installs and hides every extension from apps, with no backup or repair. Updates delete the previous version at once.
7. **Offline:** installed themes work offline. An expired catalogue index cannot be browsed at all, and revocation has no callers, so "turn it off offline" cannot be promised.
8. **Proprietary packages have no existing listing path.** EffectCraft's public catalogue requires a public repository and an open licence. A provenance URL, if a package gives one, is readable by every installer.

## 1. Scope, sources and limits

Sources were read at named revisions with read-only `git show` in local clones. Nothing was checked out, built, or run.

| Repository | Revision | Used for |
|---|---|---|
| craft-extensions (`wip/extensions-handover`) | `ffd8512e5a3755637f58cf31cfcf2fdb09cfb943` | manifest, palette, store, reader, state, index, tier, version, threat model |
| craft-launcher (`feat/extensions`) | `6ede77f93fd0d3eaaad43d430f94d703d3564a51` | store adapter for the Extensions page |
| photocraft (`feat/extensions-theme-adapter`) | `ecb056cc742848d78f9076893a6b3343e6a75010` | theme adapter, watcher, tests |
| filmcraft (main at check) | `32749e34f611735460e9773335345db026b7b037` | theme kinds, tokens, appearance preferences |
| effectcraft (main at check) | `ce9eaec65015a5508c72af0e2351cd02612c9450` | publishing guide, library, WASM import check |
| figcraft (main at check) | `0e86f3597a7443b15c93ec3ab2091202f8802c95` | plugin docs, manifest, API gate, directory |
| artcraft (main at check) | `1f969a9224934a55417daf10359a6fb1458b63f4` | ComfyUI job type and weights enum (server side) |
| craft-libs worktree | `fd5e2feff6ea25bed371a99cc732b3968d621505` | `WidgetPalette` |

Material issue labels used in headings: M1 author metadata (§3); M2 theme layers and app mapping (§4–§5); M3 ThemeCraft round trip (§6); M4 resources (§7.1); M5 workflows and model readiness (§7.2–§7.3); M6–M9 install, update, uninstall and recovery (§8); M10 offline (§9.1); M11 proprietary and support (§9.2–§9.3).

Evidence labels: **S** source read at a named revision (static reading, not run). **D** repository documentation at a named revision. **A** archived design text, not re-verified here. **J** Jeremy's supplied artifact, used as archived. **R** recommendation, not agreed. **U** unverified; needs a run or a live check.

Limits:
- No build, test run, UI run, screenshot, benchmark, or fuzzing. Historical test counts are not used as evidence.
- Live issues named by Jeremy (for example #140) were not re-read in this pass.
- ThemeCraft has no local repository and is future work. Organisation and team material is out of scope.
- The ChatGPT link is unreadable. The Claude artifact's chart is not available; its complete text is archived.
- External specifications (TUF, Sigstore, SLSA) were not re-fetched; they are cited only through earlier records.
- Two documents in the plugins record changed during this pass (`plugin-functional-decisions.md` and `plugins/README.md`). They were re-read before writing. The accepted defaults are unchanged; the decision record now also notes the separate research request.
- No delegate was used for this note, as the brief requires.

## 2. Jeremy artifact coverage

| Task | Ask (J) | Finding in this note |
|---|---|---|
| n3 | Hash-pinned download; wrong hash refused before unpack | The catalogue entry's `sha256` has no install-path consumer (S: `index.rs` L30). Only per-file hashes inside the manifest are checked after unpack (S: `unpack.rs` L169). See E34. |
| n4 | Trust labels "Made by ArtCraft", "Added from a file" | Tier is assigned by the host and stored in state. Labels should derive from tier only (§8). |
| sg4 | Revocation checked at start and before enable | `Index::is_revoked` has no callers (S: `index.rs` L156). Install and enable paths never check revocation. See E23. |
| sg6 | Empty permissions; re-validate on every load | Non-empty permissions are refused (S: `manifest.rs` L182). No module load path exists; the reader re-validates theme files only (S: `reader.rs` L70–85). |
| sg7 | No silent change of publisher, permission or version | A same-id file replaces another file package with no publisher check (S: `store.rs` L198–206). Equal version with different bytes is refused as "already installed" (S: `store.rs` L204). See E11, E12. |
| pl2 | Interface mismatch explained in plain words | Schema refusal names a number (S: `manifest.rs`). FigCraft refuses an API above its own at import (S: FigCraft `manifest.rs` L348). EffectCraft installs the newest release it implements (D: publishing §7). |
| r3 | PhotoCraft Themes & Extensions panel | PhotoCraft's store follower is opt-in and polls once a second (S: `ext_theme.rs` L229, L243–260). See §5. |
| x1, x2 | First resource type; hostile file refused by importer | Schema 1 has no resource type (S: `manifest.rs` L237). See §7.1. |
| theme-1, theme-2 | Documented theme format; ThemeCraft round trip | See §6. |
| wf-1 | Workflow JSON installs and lists; nothing runs on install | No workflow kind exists. See §7.2. |
| wf-2 | App declares the inputs and outputs it drives | Deferred to an execution contract (§13). |
| ui-verify | Empty and demo modes cannot impersonate installed items | Input to §9.3 diagnostics. Not run. |
| ship-2 | Clean-profile install, launch, restart, uninstall; record SHA-256 | Used as candidate acceptance for §8. Not run. |
| g-closed, g-say | Third-party gate stays closed; state what was not run | Preserved throughout. |

## 3. Author SDK journey and metadata (M1)

### 3.1 Journeys that exist today

- **Theme** (S, D). Write `theme.json`: a palette of nine colours plus a `dark` flag, and optional per-app overrides keyed by app id and token name. Write `craft-extension.json`: id, name, version, kind, publisher name, `compat` by app, and a file list with SHA-256 and size. Zip exactly the listed files; extra or missing files are refused (THREAT-MODEL T3). Install through the store. The repository has no separate check command, so contrast and token errors appear at install or in the app.
- **EffectCraft plug-in** (D). A Rust `cdylib` exports the ABI v1 functions and the manifest is compiled into the module. Test with `effect.plugins.load`. Publish a GitHub release with the module. Compute SHA-256 and size by hand. Open a pull request to a separate catalogue repository with `id, name, author, description, license, repo, tags, versions[]`. Review is by checklist: open licence, public source, reproducible build, behaviour. An update adds a version object. Removal is a pull request.
- **FigCraft plug-in** (D, S). `figcraft-plugin.json` (`id`, `name`, `main`, optional `version`, `api` defaulting to 1, `permissions`, `commands` with typed parameters) plus a module, with an SDK crate in the repository. Development import re-reads the module on each run. Directory publish uses a server key; the server checks the bundle and refuses a version that is not newer. Install shows permissions and SHA-256 before download.

### 3.2 Required and generated metadata (R)

| Field | Where it is typed today | Proposed class |
|---|---|---|
| `id` | manifest and again in the catalogue entry (EffectCraft) | author-required once; SDK copies to catalogue and checks equality |
| `name` | manifest and catalogue | author-required; SDK copies |
| `version` | crate manifest, embedded manifest, tag, catalogue (EffectCraft); optional in FigCraft | author-required once; SDK derives tag and catalogue entry; mismatch refused (E1) |
| `kind` | explicit in schema 1; implied by contract elsewhere | SDK-derived from declared contracts |
| publisher name | `publisher.name` (required, craft-ext); `author` (EffectCraft, FigCraft) | author-required display claim; never identity |
| description | optional, ≤500 characters (craft-ext); catalogue sentence "reviewed" (EffectCraft) | author-required, one sentence |
| licence | absent in schema 1; SPDX `license` in EffectCraft catalogue | **decision** (§14) |
| source repository | optional `provenance {source, commit}` (craft-ext; https and 40-hex commit); `repo` in EffectCraft catalogue | SDK fills from git only when the author marks the source public (E24) |
| apps and minimum version | `compat` map with `min_app`; EffectCraft `minApp` | author-required; SDK derives supported apps from contracts |
| interface or API version | `compat.interface` (required for plug-ins); FigCraft `api`; EffectCraft ABI exports | SDK-derived from the contract the module was built against |
| file list, SHA-256, size | craft-ext manifest; typed by hand in EffectCraft catalogue | SDK-generated; catalogue-authenticated later |
| parameters | EffectCraft typed `params`; FigCraft `commands[].params` | author declares types; SDK derives schema from typed code |
| permissions or capabilities | craft-ext empty (S: `manifest.rs` L182); FigCraft `permissions` | author declares; v1 empty; host enforces |
| thumbnail or preview | optional `thumbnail.png` ≤512 px (craft-ext); optional video link (EffectCraft); up to 32 assets (FigCraft) | author optional |
| tier, trust label, signature, review status, release time | tier is host-assigned; the rest absent | **never author-written**; store or catalogue adds them |
| resource type | absent | author-required for the resource kind (§7.1) |

Minimum author-written set (R): id, name, version, publisher display name, one-sentence description, licence or proprietary declaration (pending §14), supported apps with minimum app version, typed contracts and parameters (plug-ins), declared capabilities (empty in v1), declared resources (if any).

### 3.3 Normal path with an SDK (R)

1. The author fills the minimum set in one metadata source. The format is not chosen here.
2. `check` calls the same validators the store uses (craft-ext-core), so reserved namespaces, text limits, version grammar and token rules cannot drift apart.
3. `pack` writes the envelope and generates the file list, sizes, digests and derived fields. It emits a zip that matches the list exactly (T3).
4. `preview` installs into a temporary store and reports how each app would show the package: a theme as worn or not worn, with the reason; a plug-in as eligible or not, with the reason.
5. Catalogue publishing (later, and gated) reads the same generated facts. Authors never type checksums.

### 3.4 Adverse author cases

- **Version grammar (E2).** craft-ext accepts exactly three numeric parts with no leading zeros (S: `version.rs` L29–42). EffectCraft's guide permits `1.2`. A two-part version is refused by craft-ext. **R:** the SDK refuses it with a plain sentence and does not invent a patch number.
- **Wrong checksum or size (E3).** A catalogue entry that does not match the archive is refused at install. **R:** the SDK generates both values and checks them before publish.
- **Reserved id.** The store refuses first-party prefixes from non-first-party tiers (S: `tier.rs` `check_id`; `id.rs` L35–36). **R:** the SDK calls the same check.
- **Unknown fields (E1 class).** craft-ext ignores unknown fields. A new security-relevant field would therefore be silently ignored by an older host. **R:** any such addition is a schema bump that older hosts refuse with a plain message (see the security note §8).
- **Older host.** A newer schema is refused (S: `manifest.rs`, "format version (n) this app does not know"). EffectCraft negotiates the newest release it implements (D: publishing §7). **R:** use negotiation for catalogue entries and keep the schema refusal for executable packages.
- **Duplicate ids across sources.** EffectCraft refuses a second plug-in with a taken id (D: plugins.md, manifest). craft-ext replaces a same-id file package with no publisher check (E12, §8).
- **Provenance mismatch.** The provenance commit is unchecked in schema 1. **R:** the SDK fills it only from a clean checkout.

Compatibility: the SDK can generate everything schema 1 already requires. Adding a licence or resource type needs schema 2 (§14).
Performance: generation is linear in file count. Limits are 1 theme file, 2 plug-in files, and 512 resource files (S: `manifest.rs`, `Kind::limits`). Not measured.
Acceptance: TW, golden tests per kind and mismatch tests for version, id and checksum. SI now for the refusals cited.

## 4. Theme layers and WidgetPalette (M2, part 1)

| Layer | Where | Owner | Holds | Checked by | In package? |
|---|---|---|---|---|---|
| Package theme (ThemeFile) | craft-ext-core `palette.rs` L83–88 | author; store | nine colours, `dark`, per-app overrides | parse; contrast on eight text and mark pairs (L154–155, L162–164); token names ≤32 characters, lowercase, ≤64 per app (L16, L157); ≤32 apps | yes, schema 1 |
| Render input (WidgetPalette) | craft-libs `crates/ui/src/theme/mod.rs` L14–28 | app code | five widget-state fills, radius, foreground stroke | none; documented as "not a theme schema" | no |
| App tokens | PhotoCraft `Tokens`; FilmCraft `Tokens` (82 colour fields) | app | full colour semantics of the app | app-specific | no |

Rules (R):
- Packages never ship `WidgetPalette` or app `Tokens`. Adapters map ThemeFile to app tokens. App code maps tokens to `WidgetPalette` where a widget needs it.
- Reason: `WidgetPalette` names egui's interaction states. Binding package data to those names couples theme files to egui's widget model. craft-libs already has an egui migration plan that calls for a test that `WidgetPalette` keeps its borders and expansion across egui default-style changes (HEAD: `docs/egui-037-migration-plan.md` L516).
- The package check is a floor shared by all apps. Each app's own check is stronger (§5).

Edge cases:
- **E7.** The package checks `good` at 3:1, but PhotoCraft's mapping never reads `good`, and its token struct has no success colour. A theme can be refused for a colour no app displays. **R:** keep the conservative floor now, since changing it is compatibility-sensitive. Revisit per-app allowlists when a second app needs the colour.
- **E8.** `line` is not checked at package level. PhotoCraft's readable check covers text and status pairs, not borders. Readable does not mean borders are visible. **R:** document it; add a border check only with its own design.
- **E5.** Override names unknown to an app are ignored (S: `ext_theme.rs` L133; test `overrides_with_names_photocraft_does_not_know_change_nothing`). An author's typo is invisible. **R:** the SDK checks names against an allowlist published with each app's contract version. The store keeps accepting unknown names for forward compatibility.
- **E6.** PhotoCraft derives `accent_soft` with alpha in `derive()`, but `accent_soft` is also an override name (S: `ext_theme.rs` L153), and `c32` returns an opaque colour (L46). An override therefore flattens a translucent tint. **R:** mark translucent tokens non-overridable in each app's allowlist until alpha is designed. Alpha belongs in a schema bump, as `#rrggbbaa`.

## 5. PhotoCraft and FilmCraft theme semantics (M2, part 2)

### 5.1 PhotoCraft (S, `ecb056cc`)

- **Path.** `Watcher::poll` checks the store generation at most once a second (L243–260) and, on change, reads the active theme. It starts switched off and follows the default store only when asked (L229).
- **Layout.** `base_kind` keeps the app's own layout family (Pro or Studio). The package's `dark` picks only the light or dark variant within it (L66). A package cannot choose the layout family. **R:** keep.
- **Fallback ladder.** `restyle` (L190) uses overrides if they apply and pass the readability check, otherwise the palette alone. `tokens_for` (L201) falls back to the built-in look when no theme can be worn.
- **Readability.** `readable` (L181) checks text and status tokens over every surface they can sit on. That is stronger than the package check, so a theme can pass install and still not be worn. The app then returns the built-in look without a message (E16). **R:** expose a reason code (§9.3).
- **Tests.** `ext_theme_tests.rs` has 14 `#[test]` functions covering mapping, overrides, fallback, not-worn, watcher arrival and a damaged store. Not run here.

### 5.2 FilmCraft (S, `32749e3`)

- No reader of the extension format exists at this revision. Neither crates nor apps reference craft-ext or ThemeFile.
- **Theme kinds** (`theme.rs` L12–20): `Dark` (preference `darkest`, default), `Medium` (preference `dark`), and `Light`. A single package `dark` flag cannot select Medium (E9).
- **Tokens** (`theme.rs` L48): 82 colour fields. Every name is at most 32 characters, so names fit the package rule. A full-surface export would need 82 overrides, above the package's limit of 64 (E10).
- **User appearance** (`prefs.rs` L72–82): `appearance_mode`, `dark_theme`, `light_theme`, `accessible_contrast`, `highlight_color`, plus `ui_scale` and `animations`. `with_appearance` (`theme.rs` from L339) applies the highlight after the theme. The highlight also sets `focus`, `playhead`, `timecode`, `hot_text` and `track_target`. Accessible contrast is applied last. User preferences therefore sit above the theme.

Proposed FilmCraft mapping (R; for FilmCraft's maintainers to agree, not agreed):

| Package field | Proposed FilmCraft behaviour | Why |
|---|---|---|
| `dark` = true | Dark family (`darkest`) unless the user chose Medium | Medium is a user preference, not a package fact |
| `dark` = false | Light | — |
| `accent` | Accent token; a user highlight preference overrides it | the user's choice wins |
| `background`, `surface`, `text`, `muted_text`, `line` | Panel, field and separator tokens, checked against timeline selection | selection must stay visible |
| `bad`, `warn` | Status tokens where FilmCraft has a matching role | status meaning |
| `good` | No mapping until FilmCraft has a success role | do not invent a role |
| playhead, timecode, focus | Not set from the package; derived by `with_appearance` | keep timecode and playhead meaning |

Rules (R): map only tokens with the same meaning, and do not invent a mapping to satisfy the palette. The mapping should be checkable headlessly. Its report lists the tokens mapped, the tokens ignored, and the contrast result for selection, playhead and timecode.

Options for Medium (R): (A) package `dark` maps to Dark and the user chooses Medium. This is the recommended default. (B) A `family` enum in schema 2 (`dark`, `dark-medium`, `light`). This is more expressive but needs a schema bump. (C) Luminance thresholds. Rejected as fragile.

Compatibility: option A needs no schema change. Performance: mapping runs when a theme is applied; not measured.
Acceptance: TW, a fixture matrix of theme × family × highlight × accessible contrast, checking that selection, playhead and timecode stay distinct. SI now. Visual review in the candidate build is **U**, not run.

## 6. ThemeCraft round trip (M3)

Target (J theme-1, theme-2): a saved package opens, edits and exports with no launcher change, and a ThemeCraft theme appears in Themes and applies in another app.

Blocking facts (S):
1. Same-version reinstall is refused as "already installed" (`store.rs` L204); lower versions are refused as downgrades (L201). Every export must bump the version and regenerate the file list, SHA-256 and size.
2. Unknown fields are neither rejected nor kept. Nothing in ext-core denies unknown fields, so a round trip through the typed struct drops them.
3. Colours are opaque `#rrggbb` (`palette.rs`, `Rgb`). Translucency cannot be represented (E6).
4. At most 64 overrides per app (`palette.rs` L16). FilmCraft's full colour surface is 82, so ThemeCraft must write sparse overrides. A full export is refused at install.
5. `dark` must agree with the luminance relation between background and text (`palette.rs` L162–164). An editor must compute this before saving.
6. Token names are lowercase letters, digits and underscores, start with a letter, and are at most 32 characters (`palette.rs` L157). FilmCraft's names meet this.
7. The reader re-parses on every read and treats a failing theme as absent (`reader.rs` L70–85). A ThemeCraft save that breaks readability disappears in apps with no message (E16).

Options (R):
- **A.** ThemeCraft keeps its own editable source, a superset owned by ThemeCraft, and exports only schema-1 ThemeFile. Recommended default.
- **B.** Schema 2 adds an explicitly ignored `editor` object that hosts preserve and never render. Choose this only if schema 2 is needed for other reasons.
- **C.** Require hosts to keep unknown keys in exports. Rejected as default, because hosts are not editors.

Compatibility: A needs no schema change. B bumps the schema, so older hosts refuse the package (the security note §8 applies).
Performance: a theme file is capped at 64 KiB (`palette.rs`, `MAX_THEME_BYTES`). Parsing and contrast checks are cheap. Not measured.
Acceptance: TW, a round trip of open → edit → export → install → read back, with equal palette and overrides. Also refusal before export of more than 64 overrides, a same-version export, and a `dark` flag that disagrees with the colours. SI now for the refusals cited.

## 7. Resources, workflows and model readiness (M4, M5)

### 7.1 Resources (M4)

Today (S): `Kind::Resource` has no required file and no subtype. Limits are 512 files, 64 MiB per file and 256 MiB in total (`manifest.rs`, `Kind::limits`). Install unpacks and lists the package. The launcher lists resource rows (`extensions.rs` L57). No importer is called from the repositories inspected. The threat model names the existing importer as the attack surface and requires fuzzing each importer before its type ships (THREAT-MODEL T9, G8).

Adverse cases:
- A hostile file reaches the importer (T9).
- A resource package near the total cap. `install_file` reads the whole archive into memory, up to 300 MiB, before unpacking (`store.rs` L164–167; `unpack.rs` caps) (E28).
- Two packages provide the same resource names.
- A project depends on a resource that was uninstalled (E19).
- A font inside a resource package. House rule 3 places font files in the fonts repository. Schema 1 does not decide this, so resource packages should carry no fonts until it is decided.

Proposed contract (R): the resource kind declares a `resource_type` from a vocabulary each app owns. The names are chosen with the owning app, and none is invented here. It also declares an importer version range and a role for each file. The store verifies bytes only. Import is a user action in the app, run through the app's existing importer under its size caps. No install hook runs. Each type ships only after its fuzz gate (G8).

First type (J x1): choose with Jeremy after checking live issue #140 and the LUT `.cube` importer. **U**, not checked in this pass.

Options: (A) a schema 2 `resource_type` field. Recommended. (B) Encode the type in the id namespace. Rejected, because it overloads identity.
Compatibility: schema 1 resource packages stay readable as untyped. They are listed but not imported. Typed packages need schema 2.
Performance: streaming install, or per-entry hashing, is needed before a resource type that approaches the 256 MiB total ships. Not measured.
Acceptance: TW, a hostile-file refusal per importer (fuzzing) and a total-size refusal before the whole archive is read. SI now for the current limits.

### 7.2 Workflows (M5)

Today (S): there is no workflow kind. ArtCraft's platform job type `ComfyUi` is marked deprecated in favour of `VideoRender`, and its weights enum includes ComfyUi (artcraft `inference_job_type.rs` L85–86; `weights_types.rs` L29). These are server-side job records, not a local plugin contract, and this note does not reuse them.

Proposed data-only kind (R, for wf-1):
- A ComfyUI API-format JSON file under a byte cap and structural caps (nesting depth, node count, string length; values not chosen here).
- Validation extracts the referenced node types and model references. No node is executed.
- Readiness compares them with what the local backend offers and reports "missing node X" or "missing model Y" in plain words (J wf-1).
- Nothing runs on install. Execution is a separate contract (wf-2, deferred).

Adverse cases: valid JSON naming unknown node types; a node name that shadows a built-in; a workflow that asks for a download (refused unless declared as a resource); very large parameter strings; a model under a licence the user has not accepted (licence handling undecided).

Owner: store (validation and storage); app (readiness and execution).

### 7.3 Model readiness (M5)

A missing model is not an unsupported plugin (functional primary, per-app section). The states below are separate (R):

| State | Meaning | Plain message (example, R) | Owner |
|---|---|---|---|
| Not installed | package absent | "Not installed" | store |
| Installed | package and module verified (once verification exists) | — | store |
| Resource missing | a declared resource is absent | "This effect needs the X model. It is not installed." | store reports; app decides |
| Resource changed | digest differs from install | "The X model changed since it was installed. Reinstall it." | store |
| Backend unavailable | for example no GPU, or unsupported OS | "This needs a GPU. Choose the CPU option or skip this step." | app |
| Withdrawn | revocation applies (sg4) | "The publisher withdrew this package. Projects keep their data." | store and app |
| Ready | all of the above satisfied | — | app |

Models in packages (R): the 256 MiB total cap in schema 1 is below many model files. Models are not packaged in v1. Any model delivery needs its own declared download, size cap and digest check, as the archived distribution note proposes (A).

Acceptance: TW, the state table as unit tests with fixture packages. SI now.

## 8. Install, update, uninstall and recovery (M6 to M9)

| Path | Normal | Adverse (E-IDs) | Current source (S) | Recommended default (R) | Owner |
|---|---|---|---|---|---|
| Install from file | staged, checked, recorded, moved into place | same id from another file replaces it (E12); same version, different bytes, reported as "already installed" (E11) | `FromFile` is admitted in release builds (`tier.rs` L52–55); compatible apps are auto-enabled (`store.rs` L218) | consent step lists apps as checkboxes, with the initiating app checked; label derived from tier only; refuse replacing a different publisher unless the user uninstalls first; compare digests on same version | store, launcher |
| Interrupted install | staging folder removed on error | process dies after rename and before state save: folder stays, retry fails with generic `Io` (E13) | not reconciled (`store.rs` L211–212) | at start and before install, move unrecorded folders to quarantine and say so; never delete; retry then proceeds | store |
| Concurrent install | exclusive lock | five-second wait then `Busy` (`store.rs` L27, L55) | plain sentence | keep; add "try again" wording | store |
| Update, higher version | new folder; state record replaced | previous folder deleted at once (`store.rs` L230); no revert; a session reading old files could lose them (E18) | `remove_dir_all` after commit | keep the previous version until the new one has loaded once and no session pins it; one-step revert for the same source; restart flag where code was loaded (EffectCraft precedent: `library.rs` L16, L159) | store, apps |
| Update narrows `compat` | — | enablement not pruned (`store.rs` L218 inserts only); the reader still wears a theme in an app dropped from `compat` (`reader.rs` L70–85) (E17, E30) | not handled | prune enablement and active choice for dropped apps; reader checks `apps` as well | store, reader |
| Downgrade | refused | user wants an older version | refused (`store.rs` L201) | keep refused; manual downgrade means uninstall then install, with explicit consent (sg7) | store |
| Uninstall | removes bytes, enablement and theme choice | project references unknown to the store (E19) | immediate; choices cleared (`store.rs` L236–246); the launcher calls it with no store-level prompt (`extensions.rs` L82) | confirmation names what is removed and which projects reference it (apps supply references); keep per-app settings and project references by default; separate "delete data" choice | launcher, store, apps |
| Disable for one app | toggle | disabling a theme clears its choice (`store.rs` L253–270) | reversible | keep | store |
| Corrupt `state.json` | — | installs fail as "damaged" (`store.rs` L139 via `state()?`); apps see nothing (E14) | no backup | keep a last-known-good copy written atomically before each change; move the damaged file aside with a timestamp; start empty; notice with path; "rebuild from installed folders" validates manifests and deletes nothing | store |
| Newer `state.json` | — | older apps show nothing silently; writes are refused, which is safe (E15) | refused (`state.rs` L97) | notice names the newer version; never overwrite | store, apps |
| Worn theme invalid on read | — | the app keeps its built-in look silently (E16) | `None` returned (`reader.rs` L70–85) | reason code available to the launcher and apps | store API |
| Worn theme updated | — | one-poll fallback possible if the state is read before commit and the file after deletion (E20) | not reproduced | read state and file from one snapshot, or keep the old folder until the generation changes | store |

Compatibility: additive state fields (a backup, a quarantine folder) are safe for older readers, because the state type has no deny setting. Changing the meaning of an existing field is not safe.
Performance: a last-known-good write roughly doubles the state write per change. The state is capped at 4 MiB (`state.rs`, `MAX_STATE_BYTES`). A rebuild scan is bounded by install count. Not measured.
Acceptance: TW, fault injection at each write boundary (after staging, after rename, after state save, after old-folder delete). Clean-profile install → restart → uninstall (J ship-2) as candidate evidence. SI now for every line cited.

### 8.1 Notes on updates

- No previous version is kept today. The security note already says retention must be extended before hot updates are claimed.
- The theme reader copies palette and overrides into memory (`ActiveTheme`), so deleting an old theme folder is safe for themes. It is not safe for plug-ins that load from disk, which is why retention matters first for executables.

## 9. Offline, proprietary and support (M10, M11)

### 9.1 Offline

| Case | Current (S) | Recommended default (R) |
|---|---|---|
| Use an installed theme offline | works; the reader is local | keep |
| Use an installed plug-in offline (future) | not built | verify the module digest at start, as EffectCraft does (`library.rs` L456–470); craft-ext does not re-hash on load |
| Browse the catalogue offline | an expired index is refused whole (`index.rs` L123), so there is nothing to show (E21) | keep the last verified index with its expiry; allow browsing with a "last checked" label; install and update need a fresh index |
| Install from file offline | allowed (`FromFile`) | allowed, labelled "Added from a file; not checked by ArtCraft" |
| Revocation offline | `is_revoked` has no callers (E23) | apply the last known revocations; say "not checked since"; do not promise an offline disable |
| Clock set back | `Index::parse` takes `now` from the caller; an expired index parses again if the clock goes back (E22) | persist the highest time seen and refuse anything earlier. A design-level concern; no exploit shown |
| Same version, different index content | accepted (`index/tests.rs` L65) (E35) | refuse unless bytes or signed hash are identical (sg3 gap, security note) |

### 9.2 Proprietary

- **No licence field (E4).** EffectCraft's listing requires a public repository and an open licence (D: publishing §5, §9). Proprietary packages therefore have no listing path there. This is a fact about the existing catalogue, not a decision.
- **Provenance (E24).** Provenance is optional. If given, its URL and commit are readable by every installer. **R:** omit it for proprietary packages, or use a public product URL.
- **Encryption (A).** Encrypting executable code does not keep it secret, because the machine that runs it has the key. **R:** do not describe encryption as protection.
- **Unpublish (E25).** FigCraft unpublish does not disable installed copies (D: plugins.md, directory section, L215). A vendor's emergency response must be revocation (sg4), not removal from a directory.
- **Entitlements.** Paid licensing and entitlements stay outside the default (security note, decision 5).
- **House rules.** A proprietary licence identifier or text placed in these repositories needs the user's decision (rule 9). No secrets, tokens or personal details go into packages, logs or requests (rule 10).

### 9.3 Support

Support needs the package id, version, SHA-256 prefix, source label, app version, interface version, an error code, and the reason a theme or plug-in is not in use. The default is redaction: no document content, tokens, user names, or paths outside the store root.

Current (S): plain sentences exist (store `Display`), but there are no codes. `Io` is a catch-all (`store.rs` L63). The reader returns no reason (E16, E33).

**R:** stable error codes behind the sentences; a "why not shown" diagnostic from the store; a support bundle created only by a user action. Demo and empty modes must never show installed items (ui-verify).

| Report | Diagnostic that settles it | Owner |
|---|---|---|
| "Theme not applied" | enabled for this app? chosen? passed checks? state damaged? | store, app |
| "Plug-in works in the app, not in the CLI" | per-contribution eligibility trace: installed, enabled, compatible, permitted, withdrawn. FigCraft's CLI starts with no plugins (D: plugins.md L147) (E26) | store, CLI |
| "Won't install" | error code, tier, digest prefix | store |
| "Project shows a placeholder" | recorded contribution id and version; installed version; reason | app |
| "I removed it and a project broke" | references listed at uninstall | app, store |
| "Publisher pulled it" | revocation status | store (sg4) |

## 10. Edge-case register

Source column: S (source) or D (docs), with location. Acceptance: TW (test to write later), SI (source inspection, done here), U (unverified; needs a run).

| ID | Case | Source | Recommended handling | Owner | Acceptance |
|---|---|---|---|---|---|
| E1 | Version, id and name typed in several places | D effectcraft §1, §5, §7 | SDK derives; checks equality | SDK | TW |
| E2 | Two-part version accepted by one format, refused by another | S `version.rs` L29–42; D §7 | refuse with a sentence; do not guess | SDK, store | TW |
| E3 | Checksum and size typed by hand | D §6 | generate | SDK | TW |
| E4 | No licence in schema 1; open licence required for an EffectCraft listing | S `manifest.rs`; D §9 | decision §14 | author, store | SI |
| E5 | Unknown override names silently ignored | S `ext_theme.rs` L133 | per-app allowlist in the SDK | SDK, app | TW |
| E6 | An opaque override flattens translucent `accent_soft` | S `ext_theme.rs` L46, L153 | non-overridable until alpha is designed | app | TW |
| E7 | Package checks `good`; no app displays it | S `palette.rs` rows; `ext_theme.rs` | keep the floor; document | store | SI |
| E8 | Borders (`line`) unchecked | S `palette.rs`; `ext_theme.rs` `readable` | document; design a check later | app | SI |
| E9 | `dark` cannot select FilmCraft Medium | S FilmCraft `theme.rs` L12–20 | user preference (§5, option A) | FilmCraft | TW |
| E10 | Full FilmCraft surface (82) exceeds 64 overrides | S `palette.rs` L16; FilmCraft `theme.rs` L48 | sparse export; decide the limit in schema 2 | editor | TW |
| E11 | Same version, different bytes, reported as "already installed" | S `store.rs` L204 | compare digests; specific message | store | TW |
| E12 | Same id from another file replaces it, publisher unchecked | S `store.rs` L198–206 | refuse unless uninstalled first | store | TW |
| E13 | Crash after rename leaves a folder; retry fails generically | S `store.rs` L211–212 | quarantine unrecorded folders | store | TW (not reproduced) |
| E14 | Damaged state blocks installs and hides everything | S `state.rs` L95–101; `store.rs` L139; `reader.rs` L49 | last-known-good; quarantine; rebuild | store | TW |
| E15 | Newer state hides everything silently in older apps | S `state.rs` L97 | notice naming the version | store, apps | TW |
| E16 | Theme invalid on read gives a silent built-in look | S `reader.rs` L70–85; `ext_theme.rs` | reason code | store, app | TW |
| E17 | Update narrows `compat`; theme still worn in the dropped app | S `reader.rs` L70–85; `store.rs` L218 | prune; check `apps` in the reader | store | TW |
| E18 | Update deletes the previous version at once | S `store.rs` L230 | retain until loaded; revert | store | TW |
| E19 | Uninstall without project references | S `store.rs` L236–246 | list references; keep settings | launcher, apps | SI, then TW |
| E20 | Possible one-poll fallback during an update | S `reader.rs`; `ext_theme.rs` | single snapshot | store | U |
| E21 | Expired index cannot be browsed | S `index.rs` L123 | cache with a label | store | TW, fake clock |
| E22 | Clock set back revives an expired index | S `index.rs` L114, L123 | persist the highest time seen | store | TW |
| E23 | Revocation has no callers | S `index.rs` L156 | wire at start and enable (sg4) | store | TW |
| E24 | Provenance URL and commit disclosed to installers | S `manifest.rs` provenance | omit for proprietary | author, SDK | SI |
| E25 | Unpublish keeps installed copies working | D FigCraft plugins.md L215 | revocation for emergencies | store | TW |
| E26 | Fresh CLI sees no FigCraft plug-ins by default | D FigCraft plugins.md L147 | adapter change for the accepted default | FigCraft | TW |
| E27 | Resource untyped; importer is the attack surface | S `manifest.rs` L237; THREAT-MODEL T9 | `resource_type` and a fuzz gate | store, app | TW |
| E28 | Whole archive read into memory (up to 300 MiB) | S `store.rs` L164–167 | streaming before large types ship | store | U |
| E29 | Workflow JSON must not run on install; missing nodes and models must be named | J wf-1 | data-only validation | store | TW |
| E30 | Stale enablement entries after an update | S `store.rs` L218 | prune with E17 | store | TW |
| E31 | Reader parses the full state on every poll | S `ext_theme.rs` L243–260; `reader.rs` L49 | keep; measure before changing | store | U |
| E32 | Launcher uninstall has no store-level prompt | S `extensions.rs` L82 | confirm in UI | launcher | U (UI not checked) |
| E33 | Errors are sentences without codes; `Io` is a catch-all | S `store.rs` L63 | stable codes | store | TW |
| E34 | Catalogue entry hash not bound to the archive | S `index.rs` L30; `unpack.rs` L169 | bind archive to index (n3) | store | TW |
| E35 | Same-version index with changed content accepted | S `index/tests.rs` L65 | refuse unless identical (sg3) | store | TW |

## 11. Recommended defaults (R, not agreed)

| Topic | Default | Alternatives | Status |
|---|---|---|---|
| New security-relevant fields | schema bump; older hosts refuse with a plain message | none recommended | proposal |
| Author minimum set | §3.2 | licence optional for proprietary | proposal |
| Version | one source; three-part grammar | accept two-part by padding (rejected) | proposal |
| Theme floor | keep the conservative package check | per-app allowlists | proposal |
| Overrides | sparse; per-app allowlist; translucent tokens non-overridable | alpha in schema 2 | proposal |
| ThemeCraft source | ThemeCraft owns its source; exports schema 1 | schema 2 `editor` block | proposal |
| FilmCraft Medium | user choice | `family` enum in schema 2 | proposal |
| Install enablement | consent with app checkboxes; initiating app checked | all compatible apps (current code) | proposal; §14 |
| Interrupted write | quarantine; never delete | adopt if digest matches | proposal |
| Update | keep previous version until new one loads; revert for same source | keep N versions | proposal; §14 |
| Downgrade | refused | explicit manual downgrade | proposal |
| Uninstall | keep settings and references; separate delete | remove all with confirmation | proposal; §14 |
| Corrupt state | last-known-good, quarantine, rebuild that deletes nothing | automatic rebuild (rejected: silent) | proposal; §14 |
| Offline catalogue | browse stale with label; install needs fresh | strict freshness | proposal; §14 |
| Resource type | schema 2 `resource_type` | id namespace (rejected) | proposal |
| Workflows | data-only validation before any execution | defer until x2 | proposal |
| Model delivery | not in v1 | declared download with digest | proposal |
| Proprietary provenance | omit | public product URL | proposal; §14 |
| Diagnostics | codes and redacted bundle by user action | none | proposal |

## 12. Acceptance evidence

No local build, test, run, or CI was performed for this note. Every item below is therefore either source inspection already done (SI) or a test to write later (TW).

| Area | SI now | TW later | Local completed check | CI-only |
|---|---|---|---|---|
| SDK generation | refusals cited in §3.4 | golden tests per kind; mismatch tests | none | — |
| Theme mapping | §4 and §5 citations | FilmCraft and PhotoCraft matrices, fallback paths | none | — |
| ThemeCraft round trip | §6 refusals | round-trip fixtures | none | — |
| Resources | current limits (§7.1) | importer fuzzing; total-size refusal before reading | none | — |
| Install and recovery | every §8 citation | fault injection at each write boundary; clean-profile lifecycle (ship-2) | none | — |
| Offline index | §9.1 citations | fake-clock tests (sg3) | none | — |
| Support diagnostics | §9.3 | error-code and "why not shown" tests | none | — |

## 13. Assumptions, deferred items and scope

Assumptions (A):
- **A1.** Schema 1 is the baseline. Security-relevant additions need a schema bump that older hosts refuse. This follows the combined security proposal (§8 there). It is a proposal, not an accepted decision.
- **A2.** Installing a theme never makes it the app's look; the store keeps the active choice separate. Installing a plug-in still auto-enables compatible apps today (decision 3 in §14).
- **A3.** Plug-in execution rules belong to the security note. This note covers metadata, lifecycle and user experience.
- **A4.** Organisation and team material is excluded.
- **A5.** The third-party gate is closed. Third-party-facing flows are design only.
- **A6.** "Proprietary" means privately developed first-party packages. Licensing is undecided.
- **A7.** The accepted fresh-CLI default means eligibility from installed state, with no downloads.
- **A8.** Recommendations are proposals. The user has not approved them.

Deferred, with reasons:
- Catalogue, signatures, freshness and provenance detail: security and distribution note (sg2, sg3, d8, d9).
- Sandbox, capability grants and budgets: security note and functional primary.
- Headless artifact destinations (FigCraft drops save requests from programmatic callers; `crates/engine/src/extensions/plugins.rs` L332): functional note.
- GPU, native helpers and audio: separate gates in the aspect register.
- Caches and scheduling: not until measured.
- x3 AI tool and provider specs: blocked on AI infrastructure (J).
- wf-2 app adoption of workflows: needs an execution contract.
- ThemeCraft itself: does not exist yet (J).
- Download counts and ranking (d11, rank-1, rank-2): later.

## 14. Decisions for the user, in order

Each item states the consequence first and then a recommendation. None is agreed.

1. **Licence and provenance for proprietary packages.** Consequence: decides whether schema 2 carries `license` and how a private package declares itself. Without it, proprietary packages have no listing path. Recommendation: add the field in schema 2 and omit provenance URLs for proprietary packages.
2. **Scope of schema 2.** Consequence: licence, resource type and capability fields in one bump means one compatibility event for older hosts and one SDK freeze. Recommendation: one bump, with plain refusal messages on older hosts.
3. **Install enablement.** Consequence: first-run behaviour in every app. Options: all compatible apps (current code), the initiating app only, or ask per app. Recommendation: ask per app, with the initiating app checked.
4. **Update retention and revert.** Consequence: one extra version on disk per package, and whether users can revert. Recommendation: keep the previous version until the new one has loaded once; revert only for the same source.
5. **Corrupt-state repair.** Consequence: what is moved aside and what users see. Recommendation: last-known-good copy, quarantine, and a rebuild action that deletes nothing.
6. **Uninstall default.** Consequence: whether settings survive removal. Recommendation: keep settings and project references, with a separate delete-data choice.
7. **Stale catalogue browsing.** Consequence: offline browsing versus strict freshness. Recommendation: browse stale with a label; install requires a fresh index.
8. **ThemeCraft source format.** Consequence: whether ThemeCraft owns a superset format or the package carries editor data. Recommendation: ThemeCraft owns its source.
9. **First resource type and workflow timing.** Consequence: which importer is fuzzed first, and whether wf-1 waits for the resource kind. Live checks of #140 and the `.cube` importer come first.

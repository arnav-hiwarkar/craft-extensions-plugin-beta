# Shared themes, plug-ins and extensions browser (launcher-native, reachable from every app)

Status: draft for Jeremy's review. Supporting notes only: the decisions, tasks and ids live in index.html (build.py, plan_data.py, plan_tasks.py), which win on any difference. Nothing built, nothing posted, nothing pushed.
Written 16/10/2026 from a read-only look at craft-libs, craft-launcher, PhotoCraft and the org repo list.

## 1. Goal

One place to find, install, update and switch themes, plug-ins and add-on resources for the whole
Craft App Suite. It lives in ArtCraft Launcher as a first-class screen. Every app can show what it
has installed and jump straight into the launcher's browser, filtered to that app. Install once,
every compatible app sees it (same idea as the AI layer: one shared store, each app owns its own UI).

## 2. What already exists (so we extend, not invent)

| Finding | Where | Consequence |
|---|---|---|
| PhotoCraft already has sandboxed WebAssembly plug-ins: wasmi, no host imports, ABI v1, one kind (`filter`), manifest JSON, install per process, optional "additional plug-ins folder" | `photocraft-src/crates/plugins`, `docs/plugins.md` | The plug-in runtime is not new work. The store becomes where its `.wasm` files come from. Native `.dll` plug-ins stay refused (unsafe FFI, no web build), same stance as PhotoCraft. |
| Every app has its own `theme.rs` and its own `Tokens` struct, different fields (launcher: 16 colours; PhotoCraft: chrome, canvas, grid, round-chip flag and more) | launcher `theme.rs`, PhotoCraft `ui-egui/src/theme.rs`, same file in vectorcraft, designcraft, filmcraft, pdfcraft | A theme file cannot be one app's `Tokens` serialised. It needs a small shared core palette plus per-app overrides, and each app writes a thin adapter. |
| PhotoCraft tests that every text colour holds 4.5:1 on every surface | `docs/ui-design.md` | The shared validator can enforce the same rule at install time. |
| Launcher installs only from `github.com/storytold/<app>/releases`, verifies `SHA256SUMS.txt`, caps downloads and JSON, never panics | `craft-launcher-src/AGENTS.md` | Third-party extensions would break the "storytold releases only" rule. Decided 16/10/2026: stay first-party until the security gates pass (D3, section 3.3). |
| Launcher single-instance listener on loopback 47413 handles `show` and `replace` only | `crates/native/src/single_instance.rs` | A deep link from an app ("open Themes for PhotoCraft") is a small extension of that handshake, no new protocol. |
| `storytold/crafthub` exists, empty, README "Placeholder for merges" | GitHub | Possibly intended for this, possibly not. Ask before using the name. |
| craft-libs wants two or more consumers, small adapters, no giant framework upfront (#115, #94 three-tier lifecycle) | craft-libs issues | Share the manifest/index contract and the store. Do not extract the wasm host until a second app needs it. |
| `craft-archive` ZIP preflight is an open proposal (#126) | craft-libs | Packages are archives and hostile input. Either use #126 when it lands or write bounded unpack in the store crate and migrate later. |
| Launcher `views.rs` is 971 lines, over the 700 hard limit | launcher | New screen goes in its own module tree (`features/extensions/`), not into `views.rs`. |
| Launcher working tree is on `feat/ai-connections` with uncommitted changes | `craft-launcher-src` | Do this on a fresh worktree off upstream `main`; do not touch the AI branch. |

The org has 14 apps, not 12: the launcher's list omits FigCraft and AniCraft. The appendix uses the
org list, not the folders on this PC.

## 3. Design

### 3.1 One package envelope, three kinds

A package is a zip with `craft-extension.toml` at the root.

```
id            reverse-DNS, charset and homoglyph rules copied from photocraft-plugins' manifest checks
name, version, author, description (+ optional per-locale description)
kind          theme | plugin | resource
compat        per app: min app version, plug-in ABI (plug-ins), resource type (resources)
files         listed with sha256; unlisted files are rejected
permissions   none in v1 (field present, must be empty)
```

- **theme**: data only. Core palette (background, surface, text, muted text, line, accent, good,
  warn, bad, dark/light) plus optional `[app.<id>]` overrides using that app's own token names.
  Each app's adapter derives its remaining tokens from the core palette, then applies overrides.
  Unknown app sections are ignored, so one theme file works everywhere and looks tuned where the
  author bothered. No fonts in v1 (craft-fonts rules); a theme may name a family from craft-fonts.
- **plug-in**: a `.wasm` plus manifest. `kind` inside it is per app. v1 ships only PhotoCraft's
  `filter` ABI v1. The sandbox limits stay in the app's host; the package manifest only declares
  which ABI it targets.
- **resource**: files in formats the app already imports (brush sets, LUTs `.cube`, swatches `.ase`,
  templates). No code. Loading goes through the app's existing importers, which are already
  hardened. Later kind, after themes and plug-ins work.
- Future kind, not now: declarative AI tool/provider specs (the AI layer already has a
  data-described adapter). Revisit once the AI infra settles; it fits the envelope without changes.

### 3.2 Storage and who writes

Shared folder (location is the same open question as the AI store, craft-settings policy versus
`%LOCALAPPDATA%`; decide once for both):

```
<craft-root>/extensions/
  packages/<id>/<version>/...      unpacked, read-only after install
  state.json                       installed set, enabled-per-app, active theme per app, generation counter
```

- Installed globally, enabled per app. Themes: one active theme per app, with a "use this everywhere" shortcut.
- Writes are atomic (`craft-atomic`) and bump a generation counter. Apps re-scan when it changes
  (file mtime check once a second at most, no IPC), so installing in the launcher updates a running
  PhotoCraft without a restart.
- Apps only read, except "Install from file…" which is allowed in every app so a no-launcher app is
  not crippled (same requirement as AI connections). That path never touches the network.
- Network, catalogue and updates are launcher-only.

### 3.3 Catalogue and trust (decided direction, 16/10/2026)

Jeremy: start with internal, but build the structure for third party, which requires heavy security gates.

- **Release builds install first-party extensions only** (built and released by storytold) plus files the person adds themselves.
- **The third-party path is built now and switched off.** The schema carries publisher, tier, signatures, provenance and permissions from the first version, so opening it later is a change of state, not a redesign. The code sits behind a cargo feature that release builds never enable (the launcher already does this for its loopback mirror), is refused at run time, and a test fails if default or release features turn it on.
- **First-party integrity:** pinned SHA-256 in an index in the storytold organisation, downloaded only from storytold releases. Hash pinning shows the file matches the index, not who made it; the page says so.
- **Gates before third party opens (phase 3, tasks sg1 to sg13):** reserved first-party namespaces; publisher signatures with pinned and rotatable keys; signed, expiring index metadata with no rollback or freeze (TUF-style); a revocation list that turns a package off in every app; empty permissions for every plug-in plus re-validation at install and every load; no silent key, permission or version change; fuzzing and a panic hunt over every parser; recorded provenance and possibly building plug-ins from source in the hub's CI; a written review process with takedown; an independent outside review; and finally an explicit yes from Jeremy and the maintainer to the specific change that opens it.
- **Threat model first (s0e):** every threat is mapped to the gate that answers it, or accepted in writing.
- Web build of the launcher: browse only, as the catalogue already works there.

### 3.4 Launcher screen

New "Extensions" screen beside the apps and Set up AI: tabs Themes, Plug-ins, Resources; app filter
chips; search; detail page; Installed and Updates lists. Theme cards draw their own preview from the
palette in the launcher's widgets, so no screenshots are needed. All strings through `tl!` into the
four catalogs; extension names and descriptions are third-party data and are not translated.
Plain-language rule applies (no "ABI", "wasm", "sandbox" in anything a person reads: "runs safely, cannot
read your files").

### 3.5 Reachable from each app

Per app, in that app's own repo and in its own look (Jeremy's standing answer on shared pieces):
1. A small Themes & Extensions panel: installed for this app, enable/disable, remove, install from file.
2. A "Browse more…" button that starts the launcher with `--open extensions?app=<id>&kind=<k>`;
   a second start forwards it to the running launcher through the existing loopback handshake
   (`Request::Open`). If the launcher is not installed, open its download page.
3. Commands, not just buttons (PhotoCraft rule 1): `extension.list`, `extension.enable`,
   `theme.set` (already exists, extended), `extension.open_browser`, so MCP and the control channel can drive it.

## 4. Phases

The handoff page (`index.html`) is the source of truth for tasks, ids and dependencies. In outline:

0. Decide and line up, including the threat model.
1. Contract and the theme slice, first-party, with the third-party fields already in the schema.
2. Internal catalogue: first-party index, hash-pinned downloads, updates, plain trust labels.
3. Third-party structure behind security gates: built, tested with test keys only, switched off.
4. Plug-ins: PhotoCraft's host reads the store; first first-party entry is the invert-rs example.
5. Reach every app: launcher deep link, PhotoCraft panel and commands, then one adapter per app by its maintainer.
6. Resources and later kinds (declarative AI specs after the AI infrastructure settles).

## 5. Decisions needed from Jeremy (recommendation first)

| # | Question | Recommendation | If not |
|---|---|---|---|
| D1 | What does "extension" cover? | Three kinds: themes, plug-ins, resources. | Themes and plug-ins only, resources dropped from the envelope. |
| D2 | User-facing name | Section "Extensions", tabs Themes / Plug-ins / Resources. | "Add-ons". PhotoCraft already says Plug-ins, so keep that word. |
| D3 | Whose extensions can the launcher install | **Decided by Jeremy (16/10/2026):** internal first, third-party structure built behind heavy security gates and switched off. | n/a |
| D4 | Where the index lives | `storytold/crafthub` if the maintainers confirm it was meant for this; else a new repo under Jeremy's fork first. | Folder inside craft-launcher. Couples catalogue changes to launcher releases, which is bad. |
| D5 | Store folder | Same answer as the AI store, decided once. | Two folders, two policies, more to explain. |
| D6 | Does the launcher accept themes itself? | Optional, default stays the website-matched brand look. | Launcher never themed; themes are for apps only. |
| D7 | Who owns the contract crates | Jeremy leads, contract in craft-libs after two consumers exist, store/catalogue in a standalone repo, same shape as the AI layer. | Everything in craft-libs from day one; slower review, bigger PR. |

## 6. Risks

- **Theme fidelity.** Apps differ a lot (PhotoCraft's round-chip flag, canvas grid). A core palette gives
  "acceptable everywhere", not "perfect everywhere". Per-app overrides are how authors fix that. Expect the
  adapter to be the real work per app.
- **Trust is only as strong as index review.** A merged entry can still point at a bad plug-in. The sandbox
  (no imports, fuel and memory caps) is what limits damage, so plug-in review must not be treated as the safety net.
- **Scope creep into a marketplace.** No ratings, accounts, payments or uploads in v1. Index PRs only.
- **Two moving owners.** craft-libs is read-only for Jeremy and `skylartaylor` merges. Phases 1 and 2 are
  buildable on our side; landing in craft-libs waits on them, as with `craft-ai-core`.
- **Launcher is already heavy.** Keep the feature in its own module tree and behind the existing Command/Event
  seam so the web build stays catalogue-only.
- **Collides with the AI work in the same launcher.** Separate worktree and branch.

## 7. Files likely to change

New: `craft-ext-core`, `craft-ext-store` (workspace location per D7); launcher
`crates/ui-egui/src/features/extensions/` (list, detail, installed, theme card) and
`crates/native/src/extensions.rs` (fetch, via the existing `http.rs`).
Modified: launcher `crates/core/src/protocol.rs` (Command/Event), `single_instance.rs` (`Request::Open`),
`i18n/*.tsv` (four catalogs), `ATTRIBUTION.md` for bundled sample themes;
PhotoCraft `crates/plugins`, `crates/ui-egui/src/theme.rs`, `docs/plugins.md`, `engine/src/plugin_cmds.rs`.

## Appendix: per-app adoption checklist (org list, 14 apps)

Same checklist for each, PhotoCraft shown only as the first row, not the template:
theme adapter from core palette; Themes & Extensions panel; `extension.*` commands; deep link button;
plug-in host only if the app has a plug-in kind; resource importers listed.

| App | What it is (repo description) | Plug-in kind likely | Notes |
|---|---|---|---|
| PhotoCraft | photo editor | filter (exists) | Own `Tokens` with shape flags; wasmi host exists |
| LightCraft | non-destructive photo editor | filter (second consumer for the wasm host) | LUT and preset resources |
| VectorCraft | vector art tool | later | Own `colortheme.rs` command already; swatch resources |
| DesignCraft | page and layout design | later | Swatches, templates |
| PdfCraft | PDF reader and editor | none yet | Has `desktop_theme.rs`; themes first |
| FilmCraft | nonlinear video editor | later | LUTs, transitions as a future kind |
| EffectCraft | motion graphics and compositing | filter / effect | Likely second wasm consumer |
| SoundCraft | Pro Tools-style audio | later (audio) | Different ABI, audio buffers, separate design |
| WordCraft, DeckCraft, GridCraft | Word, PowerPoint, Excel style | none yet | Ribbon shells; themes and templates |
| CADCraft | AutoCAD-style CAD | none yet | Themes only to start |
| FigCraft | interface design and prototyping | later | Not in the launcher's own list; add the catalog entry first |
| AniCraft | 2D animation editor | later | Not in the launcher's own list; same |

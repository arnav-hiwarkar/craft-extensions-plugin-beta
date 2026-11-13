# Plugin command-line reference

This page lists every `craft-ext` command, every `<app>-cli plugin` command, the app commands
that drive plugins, and the environment variables the platform reads.

Sources: `crates/ext-cli/src/{main.rs,lib.rs,author.rs,catalogue.rs}`, the `plugin` entry of each
app CLI (`effectcraft/crates/plugins/src/lib.rs` `run_cli`, `photocraft/apps/photocraft-cli/src/lib.rs`,
`figcraft/crates/plugins/src/lib.rs` `run_cli`) and the engine command tables named below.

## Conventions

- Every command accepts `--json`. With it, output is one compact JSON document. Without it, a
  plain string prints as text and anything else prints as pretty JSON.
- Every command accepts `--help`.
- An unknown option is an error (`unknown option --x`). A value option needs a value that does not
  start with `--`.
- Errors print as `{"error": "..."}` on standard error. `plugin` commands also include `"code"`.

### Exit codes

| Code | Meaning | Where |
|---|---|---|
| `0` | Success | All commands |
| `1` | The command failed | All commands |
| `2` | Usage error: wrong arguments, unknown option or command, bad id or permission text | All commands |
| `4` | Refused, and the error names the fix (`Fix: plugin ...`): plugin missing, contribution unavailable, permission needed, native consent needed | `plugin` commands only (`run_plugin`) |

## `craft-ext`: author and maintainer tool

The binary is `craft-ext` (crate `craft-ext-cli`, feature `tools`). `craft-ext` with no arguments
or `craft-ext --help` prints the command summary. `craft-ext <command> --help` prints the author
summary (`new | build | check | test | pack | dev`) or the catalogue summary.

`[dir]` defaults to the current directory. Each author command except `new` reads
`<dir>/craft-plugin.toml` first.

### `craft-ext new <effect|panel|data> <dir> --id <id>`

Creates a plugin from a template and replaces `io.github.YOUR-NAME.PLUGIN` with `<id>` in every
file.

- `<dir>` must not exist (`destination already exists`).
- `--id` is required and must be a valid extension id.
- Templates are built into the binary. `CRAFT_TEMPLATES=<folder>` copies `<folder>/<kind>` instead.
- `CRAFT_GUEST_PATH=<path>` points the new `Cargo.toml` at a local `craft-guest` crate. Without it,
  a binary built from a craft-extensions checkout uses that checkout's `crates/guest`; otherwise the
  template's git dependency stays.

```sh
craft-ext new effect grain --id io.github.ink.grain
```

### `craft-ext build [dir] [--release]`

Runs `cargo build --target wasm32-unknown-unknown` in `[dir]` and copies each built `.wasm` to the
`module` path its contribution names.

- When no contribution has a `module` (data or native-only packages), it prints
  `No wasm modules to build` and succeeds.
- If `Cargo.toml` has `[[example]]` targets whose names match a module file name (`-` read as
  `_`), it builds those examples and the library, and nothing else.
- Without `--release`, it uses the dev profile at optimisation level 1 (unoptimised SDK code can run
  out of sandbox fuel). Set `CARGO_PROFILE_DEV_OPT_LEVEL` to override.
- A module is matched to a built file by file stem. If there is only one built `.wasm`, every module
  gets it. Otherwise an unmatched module fails the build.

```sh
craft-ext build examples/cross-review --release
```

### `craft-ext check [dir] [--store]`

Checks a package before you pack it. It fails when:

- a file named by the manifest (modules, data files, native executables) is missing;
- a module is not valid wasm, lacks a required export (`craft_abi`, `craft_alloc`, `craft_reset`,
  plus `craft_filter` for effects or `craft_event` for other code), has a wrong signature, or imports
  anything other than `craft.log`, `craft.call`, `craft.result` and `craft.take`;
- with `--store`, the licence is not OSI-approved.

Warnings (exit 0): a non-OSI licence without `--store` ("Local use only"), paid or trial wording in
the name, summary or description, and a standing reminder that permissions need human review.

```sh
craft-ext check . --store --json
```

### `craft-ext test [dir] [--size WxH]`

Runs each `effect` contribution twice on a generated gradient image with the form's default values
and `color: "linear-srgb"`. It fails if the two outputs differ by any bit, or if any value is not
finite or any alpha is outside 0 to 1. The default size is `64x64`; the limit is 16,777,216 pixels.

Packages without effects succeed with `{"tested": [], "message": "No automated tests for
panels/commands yet; run in the app with `craft-ext dev`"}`. Panels, commands, tools and hooks are
not tested.

```sh
craft-ext test examples/cross-vignette --size 256x256
```

### `craft-ext pack [dir] [--describe] [--out file] [--release-record --tag vX --commit SHA --url URL]`

Writes a `.craftplugin` zip with the manifest and only the files it names. The default output is
`<dir>/<id>-<version>.craftplugin`. Symbolic links and paths outside `[dir]` are refused. Native
executables are stored with mode `755`.

- `--describe` loads each compiled module in the reference sandbox, calls its optional
  `craft_describe`, and embeds the returned form in the packaged manifest. The source manifest is
  not changed. Plain `pack` runs no plugin code. Neither form builds anything.
- `--release-record` also writes `<dir>/<version>.toml`, the catalogue release record. It requires
  `--tag`, `--commit` and `--url` (the release asset URL).

Output: `{"file", "sha256", "size", "needs_grants": [], "development_store": null}`.

```sh
craft-ext pack --release-record --tag v1.0.0 --commit 0123abcd... \
  --url https://github.com/ink/grain/releases/download/v1.0.0/io.github.ink.grain-1.0.0.craftplugin
```

### `craft-ext dev [dir] --app <effectcraft|photocraft|figcraft>`

Packs like `pack` (it accepts the same options), then installs the package into the **development
store**, `<store root>/development`, with the tier "Development".

- It makes the version active for that app.
- It enables every contribution that the app's contracts support, needs no permissions, and is not
  native.
- For every other contribution it lists the commands to run in `needs_grants`
  (`plugin grant ...`, `plugin setup ... --enable ...`, with `--full-access` for native packages).
- `development_store` in the output is the folder. Apps read the normal store, so start the app
  and its CLI with `CRAFT_EXT_DIR` set to that folder to use the development install.

```sh
craft-ext dev . --app effectcraft
CRAFT_EXT_DIR="<development_store>" effectcraft-cli plugin setup io.github.ink.grain --all
```

### `craft-ext catalogue ...`

Maintainer commands for the catalogue repository. Store policy and the publishing workflow are in
[store-and-distribution.md](store-and-distribution.md).

| Command | What it does |
|---|---|
| `catalogue check --repo <dir> [--offline] [--only <id>]` | Validates vendors, listings, release records, namespaces, licences and asset URLs. Without `--offline` it also checks the public repository, numeric ids, licence, tag commit, and downloads each asset to compare its manifest. `GITHUB_TOKEN` is sent to the GitHub API when set. |
| `catalogue build --repo <dir> [--offline]` | Writes `dist/index.json` and `dist/packages/<id>.json`, increases the serial and deletes the old signature. |
| `catalogue build --local-assets <dir> --out <dir>` | Builds an unsigned development catalogue from local `.craftplugin` files. Archives inside `--out` get relative URLs. |
| `catalogue sign --repo <dir> --key <file>` | Signs `index.json` (in `<dir>` or `<dir>/dist`) with a 32-byte hex seed. |
| `catalogue keygen --repo <dir> --out <file>` | Writes a new seed (mode 600, never overwrites) and prints the public key. |

```sh
craft-ext catalogue check --repo ../craft-plugins --only io.github.ink.grain
```

### `craft-ext plugin --app <app> <command> ...`

Runs any `plugin` command below against the shared store for the named app (`effectcraft`,
`photocraft` or `figcraft`). The contracts per app come from `crates/ext-cli/src/known_apps.rs`.
`--app` is required; a missing or unknown app exits 2.

```sh
craft-ext plugin --app photocraft list --json
```

## `<app>-cli plugin`: managing plugins

`effectcraft-cli plugin`, `photocraft-cli plugin` and `figcraft-cli plugin` all run the shared
`craft_ext_cli::run_plugin` with that app's contracts, except for the app-specific commands listed
after this table. `<id>` is a plugin id such as `io.github.arnav-hiwarkar.cross-review`.

| Command | Options | What it does |
|---|---|---|
| `plugin search <words>` | `--kind <kind>` | Searches the cached catalogue for this app. Adds each row's `tier`. Kinds: `effect`, `command`, `panel`, `tool`, `event-hook`, `preset`, `lut`, `font`, `theme`, `data`. |
| `plugin info <id>` | | The catalogue listing and tier. Fails for a plugin that is not listed. |
| `plugin versions <id>` | | Listed and installed versions, each with its compatibility in this app. |
| `plugin permissions <id>` | | The permission sheet of the installed version: required and optional permissions with the app's sentence and the author's reason, warnings, and the native label. |
| `plugin report <id>` | | Prints the pre-filled issue URL for the installed version. |
| `plugin list` | | The whole store state as JSON. |
| `plugin updates` | | Available updates for this app, and problems. |
| `plugin refresh` | | Downloads and verifies the signed catalogue and kill list. |
| `plugin install <id>` | `--version <req>`, `--with-deps` (default) or `--no-deps` | Downloads, verifies and stores a release and its required packages. Never enables or grants. `--no-deps` fails if packages are required. |
| `plugin install-file <path>` | | Installs a local `.craftplugin` as "Not reviewed (local file)". Never enables or grants. |
| `plugin setup <id>` | `--enable c1,c2` or `--all`, `--grant <permission>` (repeatable), `--full-access` | Makes the installed version active, applies the grants and native consent, then enables the contributions. Refuses (exit 4) when a required permission or native consent is missing, and lists each fix. `--all` enables only the contributions whose contract this app provides. |
| `plugin enable <id> <contribution>` | | Enables one contribution. Exit 4 with the fix when a permission or consent is missing or the contribution is unavailable here. |
| `plugin disable <id> <contribution>` | | Disables one contribution and stops its running work. |
| `plugin grant <id> <permission>` | | Grants one permission in this app. |
| `plugin revoke <id> <permission>` | | Revokes one permission in this app. Running work stops and its edits roll back. |
| `plugin update <id>` or `plugin update --all` | | Installs the newest compatible version. If it needs more access, or is a native version you have not consented to, it waits as pending until `setup`. |
| `plugin rollback <id> <version>` | | Downloads the version if needed and makes it active. Warns about saved data from newer versions. |
| `plugin pin <id> <req>` / `plugin unpin <id>` | | Holds updates to a version requirement, or removes the hold. |
| `plugin remove <id>` | `--force` | Removes the plugin. Refuses while another installed package requires it, unless `--force`. |
| `plugin project <references.json>` | | Prints the status of each plugin reference in a JSON array of references, with `install` and `setup` hints. |

Grant and revoke apply to the package in this app. A contribution can still use only the
permissions it declares.

```sh
effectcraft-cli plugin install-file cross-review.craftplugin
effectcraft-cli plugin setup io.github.arnav-hiwarkar.cross-review \
  --enable effectcraft-apply,effectcraft-panel --grant document.edit:document
photocraft-cli plugin setup io.github.arnav-hiwarkar.photocraft-layer-export --all \
  --grant document.read:document --grant files.write:picked
figcraft-cli plugin setup io.github.arnav-hiwarkar.native-system-stamp \
  --enable figcraft-apply --grant document.edit:document --full-access
```

### App-specific `plugin` commands

| Command | EffectCraft | PhotoCraft | FigCraft |
|---|---|---|---|
| `plugin contract` | Prints the contract reference (Markdown). | Same. | Same. |
| `plugin project <file>` | Reads an `.ecproj` (JSON or XML) and prints each plugin's status. Exit 1 on errors. | Not app-specific: the shared command, which reads a references JSON array. | Reads a FigCraft file and prints each plugin's status. Without a file: usage error. |
| `plugin resume <id>` | Prints where pauses live. Pauses last one session; use `plugin.resume` in a running session or **Resume** in the app. Without `<id>`: exit 2. | Checks a fresh session; prints `Resumed <id>.` or explains that pauses last one process and names `plugin.resume`. | Prints that pauses last one session and names the `plugins.resume` script step. Without a valid `<id>`: exit 2. |

```sh
effectcraft-cli plugin project film.ecproj --json
figcraft-cli plugin project board.figcraft
```

## Driving plugins from app CLIs

### EffectCraft

`effectcraft-cli` sessions load enabled plugins unless safe mode is on. Run engine commands with
`run <id> <json> ...` or `exec <id> --params <json>`.

| Command | Parameters | What it does |
|---|---|---|
| `plugin.list` | `{}` | Loaded plugins, plugin commands (`{id, label, plugin}`) and surfaces (`{key, title, window, open, rev, tree}`). |
| `plugin.<plugin id>.<contribution>` | `{}` | Runs a command contribution, or selects a tool contribution. |
| `plugin.showPanel` | `{"surface": "<plugin id>/<panel>", "open"?: bool}` | Opens or closes a panel. |
| `plugin.uiEvent` | `{"surface", "rev", "widget", "value"?}` | Sends a widget event. A stale `rev` is dropped. |
| `plugin.toolEvent` | `{"phase": "begin"\|"update"\|"commit"\|"cancel", "at": [x, y], "modifiers"?}` | Sends a pointer event to the selected plugin tool. |
| `plugin.resume` | `{"id": "<plugin id>"}` | Clears a pause and sends `init` again. |
| `plugin.resources` | `{}` | Presets and LUTs from enabled plugins. |
| `plugin.missing` | `{}` | Plugin effects used by the project that are not loaded. |

```sh
effectcraft-cli run --project shot.ecproj --save \
  plugin.io.github.arnav-hiwarkar.effectcraft-point-tool.main '{}' \
  plugin.toolEvent '{"phase":"begin","at":[100,80]}' \
  plugin.toolEvent '{"phase":"commit","at":[140,80]}'
```

### PhotoCraft

`photocraft-cli run <file> --cmd <id> [--params <json>] ... [--out <file>] [--pick <path>]...`

| Command id | What it does |
|---|---|
| `plugin.session.<plugin id>/<contribution>` | Runs a command (`--params` arrive as the event's `params`). For a panel, opens it. For a tool, `{"points": [[x, y], ...]}` runs a whole gesture (begin at the first point, commit at the last; `"cancel": true` cancels; optional `modifiers` and `zoom`). Without `points` it selects the tool. |
| `plugin.<plugin id>/<contribution>` | Runs a plugin filter (effect) on the active layer with the given form values. |
| `plugin.uiEvent` | `{"surface", "rev", "widget", "value"}` to an open panel. |
| `plugin.paused` / `plugin.resume` | Lists pauses; `{"plugin": "<id>"}` clears one. |

`--pick <path>` answers one `files.pick` question with that file or folder, in order. Without it,
plugins that ask for a file get "cancelled".

```sh
photocraft-cli run photo.png --pick ./out \
  --cmd plugin.session.io.github.arnav-hiwarkar.photocraft-layer-export/export-all
photocraft-cli run photo.png --out ramp.png \
  --cmd plugin.session.io.github.arnav-hiwarkar.photocraft-gradient-ramp/gradient \
  --params '{"points":[[0,0],[400,0]]}'
```

### FigCraft

`figcraft-cli run <script.json|script.jsonl> [--in <file>] [--out <file>] --plugins`

Without `--plugins` no plugin code runs. The script is a JSON array (or JSON lines) of
`{"command": id, "params": {...}}` steps, run as one all-or-nothing undo step. A failed step rolls
the whole script back and exits 1.

| Command id | What it does |
|---|---|
| `plugin.<plugin id>.<contribution>` | Runs a command (with `params`), or selects a tool. |
| `plugins.paused` | Lists plugins paused this session. |
| `plugins.resume` | `{"id": "<plugin id>"}` clears a pause and starts fresh instances. |

```sh
echo '[{"command":"plugin.io.github.arnav-hiwarkar.cross-review.figcraft-apply","params":{}}]' > review.json
figcraft-cli run review.json --in board.figcraft --out board.figcraft --plugins
```

After a contribution is disabled or a required permission is revoked, it is unloaded, so its
command id is reported as unknown.

## Environment variables

| Variable | Read by | Effect |
|---|---|---|
| `CRAFT_EXT_DIR` | Store (`ext-store/src/location.rs`): apps, launcher, CLIs | Store folder. Must be a full path without `..`. Default: `~/Library/Application Support/ArtCraft/extensions` (macOS), `%LOCALAPPDATA%\ArtCraft\extensions` (Windows), `$XDG_DATA_HOME/artcraft/extensions` or `~/.local/share/artcraft/extensions` (others). |
| `CRAFT_CATALOGUE_URL` | Manager (`ext-store/src/manager.rs`) | Catalogue base URL. Default `https://raw.githubusercontent.com/arnav-hiwarkar/craft-plugins/main/dist/`. Signatures are still required. |
| `CRAFT_CATALOGUE_ALLOW_FILE` | `ext-store/src/catalogue.rs` | `1` allows `file://` release assets from a non-file catalogue. Also one of the conditions for the test key. |
| `CRAFT_CATALOGUE_TEST_KEY` | `ext-store/src/catalogue.rs` | An extra ed25519 public key (64 hex characters). Accepted only in debug builds, only with `CRAFT_CATALOGUE_ALLOW_FILE=1`, and only for a `file://` catalogue. Compiled out of release builds. |
| `CRAFT_GUEST_PATH` | `craft-ext new` | Local `craft-guest` crate for the new `Cargo.toml`. |
| `CRAFT_TEMPLATES` | `craft-ext new` | Template folder to copy instead of the built-in templates. |
| `GITHUB_TOKEN` | `craft-ext catalogue check/build` | Sent to the GitHub API for online checks. |
| `EFFECTCRAFT_SAFE_MODE` | EffectCraft (`effectcraft/crates/host/src/lib.rs`) | `1` loads no plugin code. The desktop app also accepts `--safe-mode`. The manager still works. |
| `PHOTOCRAFT_SAFE_MODE` | PhotoCraft (`photocraft/crates/plugins/src/lib.rs`) | `1` loads no plugin code. The manager still works. |
| `FIGCRAFT_SAFE_MODE` | FigCraft (`figcraft/crates/plugins/src/lib.rs`) | `1` loads no plugin code and refuses plugin events. The manager still works. |

A local test catalogue, for example:

```sh
craft-ext catalogue build --local-assets ./packages --out ./catalogue
craft-ext catalogue keygen --repo ./catalogue --out ./test.key   # prints public_key
craft-ext catalogue sign --repo ./catalogue --key ./test.key
CRAFT_CATALOGUE_URL="file://$PWD/catalogue" CRAFT_CATALOGUE_ALLOW_FILE=1 \
CRAFT_CATALOGUE_TEST_KEY=<public_key> effectcraft-cli plugin refresh   # debug build only
```

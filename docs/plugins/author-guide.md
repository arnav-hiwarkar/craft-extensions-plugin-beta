# Making Craft plugins (author guide, beta)

A Craft plugin is a package (`.craftplugin`, a zip) with one manifest, `craft-plugin.toml`, and the
files it names. One package can work in EffectCraft, PhotoCraft and FigCraft. Each part of a
package is a **contribution**, and each contribution names the **contract** it needs. An app runs a
contribution only when it provides that contract.

Related pages: [CLI reference](cli-reference.md), [UI reference](ui.md), the generated contract
pages ([EffectCraft](contracts/effectcraft.md), [PhotoCraft](contracts/photocraft.md),
[FigCraft](contracts/figcraft.md)), and [store and distribution](store-and-distribution.md).

## 1. What you can make

| Kind (`kind =`) | Code | Contract | EffectCraft | PhotoCraft | FigCraft |
|---|---|---|---|---|---|
| `effect` | wasm, `filter!` | `raster.filter@1.0` | Effects & Presets and Effect ▸ category | Filter ▸ Plugins ▸ category | Not available |
| `command` | wasm or native, `session!` | `<app>.session@1.x` | Plugins ▸ plugin ▸ label | Plugins ▸ plugin ▸ label | Plugins ▸ plugin ▸ label |
| `panel` | wasm or native, `session!` | `<app>.session@1.x` | Floating panel from the Plugins menu | Floating panel, toggled from the Plugins menu | Docked panel, toggled from the Plugins menu |
| `tool` | wasm or native, `session!` | `<app>.session@1.x` | Selected from Plugins; draws in the composition viewer | Selected from Plugins; draws on the canvas | Plugin tool strip and Plugins menu |
| `event-hook` | wasm or native, `session!` | `<app>.session@1.x` | Receives app events | Receives app events | Receives app events |
| `preset` | none | `resource.preset@1.0` | Each preset becomes a Plugins command | Presets in the plugin filter dialog | Not available |
| `lut` | none | `resource.lut@1.0` | Listed by `plugin.resources` | Color Lookup ▸ From plugins | Not available |
| `font` | none | `resource.font@1.0` | Not available | Not available | Accepted; not loaded in this beta |
| `theme` | none | `craft-theme@1.0` | Accepted; not applied in this beta | Same | Same |
| `data` | none | — | Stored only | Stored only | Stored only |

The contract tables come from `crates/ext-cli/src/known_apps.rs` and each adapter's
`contract.rs`. FigCraft provides no `raster.filter`, so effects never run in FigCraft. All three
apps provide tools: see `examples/effectcraft-point-tool`, `examples/photocraft-gradient-tool` and
`examples/figcraft-shapes`.

Session contracts are per app (`effectcraft.session`, `photocraft.session`, `figcraft.session`). All
three apps provide `@1.1`. Declare `@1.1` if your panels use UI 1.1 nodes (tabs, tables, lists,
number fields, toggles, icons, paths, gradients and the other nodes in the [UI reference](ui.md));
`ui::level(&tree)` tells you the level a tree needs.

## 2. Start a plugin

Install the Rust target once: `rustup target add wasm32-unknown-unknown`. Then:

```sh
craft-ext new effect grain --id io.github.<your-login>.grain   # or: panel, data
cd grain
craft-ext build              # wasm32-unknown-unknown, dev profile at opt-level 1
craft-ext check              # files, exports, imports, licence
craft-ext test               # effects only: deterministic, valid pixels
craft-ext pack               # writes io.github.<your-login>.grain-0.1.0.craftplugin
```

The three templates are:

- `effect`: one `raster.filter@1.0` effect with a slider. It lists all three apps; remove
  `figcraft` from `apps`, because FigCraft cannot run effects.
- `panel`: an EffectCraft panel and a command in one module.
- `data`: a LUT and a preset file, no code.

`craft-ext build --release` builds the release profile. If your `Cargo.toml` has `[[example]]`
targets named after manifest modules, `build` builds those and the library, and copies each
`.wasm` to its module path. `examples/cross-review` uses this to ship one command module and
three panel modules from one crate.

## 3. Try it in an app

### With the development store

```sh
craft-ext dev . --app effectcraft
```

This packs the plugin and installs it into the development store, `<store root>/development`, with
the tier "Development". It activates the version and enables every contribution that needs no
permission and is not native. For the rest it prints the commands to run in `needs_grants`.

The apps read the normal store. To use the development install, start the app or its CLI with
`CRAFT_EXT_DIR` set to the `development_store` folder that `dev` printed:

```sh
CRAFT_EXT_DIR="<development_store>" effectcraft-cli plugin setup <id> --all --grant document.read:selection
CRAFT_EXT_DIR="<development_store>" effectcraft
```

### With a package file

```sh
effectcraft-cli plugin install-file io.github.ink.grain-0.1.0.craftplugin
effectcraft-cli plugin setup io.github.ink.grain --all --grant document.edit:document
```

Installing never enables or grants anything. `setup` refuses (exit 4) until every required
permission of the chosen contributions is granted, and lists each missing one. `--all` enables
only the contributions whose contract this app provides. Native packages also need
`--full-access`. In the desktop apps, use the Plugins window instead (see the
[user guide](user-guide.md)).

## 4. The manifest

`craft-plugin.toml` is the source manifest and the manifest inside the package. Unknown fields are
refused, so a typo cannot drop a permission. The size limit is 128 KiB.

```toml
schema = 2                                  # the only accepted value
id = "io.github.arnav-hiwarkar.photocraft-levels-scope"
name = "Levels Scope"                       # 64 characters, one line
version = "1.0.0"                           # semver; one version is one file forever
license = "MIT OR Apache-2.0"               # SPDX; the store accepts OSI terms only
source = "https://github.com/arnav-hiwarkar/photocraft-levels-scope"   # https, public
summary = "A live histogram panel"         # 140 characters, one line
apps = ["photocraft"]                       # apps this package supports
# Optional: description (4000 characters), authors (16), tags (8)

[why]                                       # your reason, shown under the app's own sentence
"document.read:document" = "Reads the image's histogram to draw it"
"document.edit:document" = "Adds the Levels adjustment layer when you press the button"
"app.events" = "Redraws the histogram when you open an image or change the selection"

[[contribution]]
id = "levels-scope"                         # lowercase, starts with a letter, 48 characters
kind = "panel"
contract = "photocraft.session@1.1"         # name@major.minor: the minimum version
label = "Levels Scope"                      # 64 characters
module = "photocraft_levels_scope.wasm"
permissions = ["document.read:document"]    # required before it can be enabled
optional_permissions = ["document.edit:document"]   # off until the person allows them

[[contribution]]
id = "follow"
kind = "event-hook"
contract = "photocraft.session@1.1"
label = "Follow the image"
module = "photocraft_levels_scope.wasm"
permissions = ["document.read:document"]
optional_permissions = ["app.events", "document.edit:document"]
```

This is `examples/photocraft-levels-scope/craft-plugin.toml`, shortened.

Rules (from `crates/ext-core/src/manifest.rs`):

- **Id.** Dotted lowercase segments, at most 64 characters. Use `io.github.<login>.<name>`. The
  catalogue refuses an `io.github.` id whose login is not the vendor's GitHub login. Prefixes such
  as `com.artcraft.`, `ai.artcraft.` and `org.storytold.` are reserved for official packages.
- **Apps.** At least one, no duplicates. A contribution can narrow itself with
  `apps = ["photocraft"]`; each entry must also be in the package's `apps`.
- **Contributions.** At least one, at most 128, unique ids. Code kinds need `module` (unless the
  package is native). Data kinds need `files` and cannot have `module`. `files` may also list extra
  files to ship, such as a licence.
- **Paths** are relative, inside the package, without `..` or symbolic links.
- **`[why]`** keys must be permissions that some contribution requests.
- **`category`** sets the menu group (for example `"Stylize"`).
- **`state_version`** is the version of the state this contribution saves in projects. Bump it
  when that shape changes. Rollback warns when an older version has a lower `state_version`, and
  newer saved data is kept as a placeholder.
- **`[requires]`** lists other packages this one needs, with a version requirement. `plugin
  install` installs them too:

  ```toml
  [requires]
  "io.github.arnav-hiwarkar.halftone" = "^1.0"
  ```

- **`[contribution.form]`** describes an effect's parameters (section 7).

### Compatibility

A contribution works in an app when the app provides the same contract major at the same or a
higher minor. The manager shows a package as compatible, partly compatible (with the reason for
each unavailable contribution) or incompatible. Raise your plugin's major version when you remove
a contribution or parameter, or change saved state without a migration.

## 5. Permissions

Declare the smallest set, per contribution. For every operation, the app computes the permissions
needed for those arguments. The call is refused unless the calling contribution declared each one
(required or optional) **and** the person granted it to the package in this app. A grant is
therefore never usable by a contribution that did not declare it.

| Permission | The setup sheet says | Used by |
|---|---|---|
| `document.read:selection` | Read the selected items | Query operations on the captured selection |
| `document.read:document` | Read the current document | Query operations |
| `document.read:all` | Read all open documents | Query operations, for example PhotoCraft `documents.list` |
| `document.edit:selection` / `:document` / `:all` | Change the selected items / the current document / all open documents | Edit operations |
| `files.read:picked` | Open files and folders you pick | PhotoCraft `files.pick` (open), `files.read`, `layer.import` |
| `files.write:picked` | Save to files and folders you pick | PhotoCraft `files.pick` (folder, save), `files.write`, `document.export` |
| `ui.window` | Open its own windows | `ui.window.open` |
| `app.events` | Be told when things happen in the app | Receiving `app` events |

A broader scope covers a narrower one (`:all` covers `:document` covers `:selection`), and edit
covers read for the same scope. The exact rule for each operation is on the app's contract page.
Panels (`ui.panel.set`), notifications, overlays and logging need no permission.

The vocabulary also parses `network:<host>`, `clipboard.read` and `clipboard.write`, and the setup
sheet can show them, but **no operation in any app uses them**. Do not request them: a plugin
cannot reach the network or the clipboard in this beta.

When an operation needs a permission that is not granted, `craft_guest::call` returns
`CallError::PermissionNeeded`. Handle it, for example by showing a hint in your panel, instead of
failing the event (section 6, Color notes).

## 6. The guest SDK

The SDK is the `craft-guest` crate (`crates/guest`). Build modules as `cdylib` for
`wasm32-unknown-unknown`. Each module uses exactly one entry macro: `filter!` for an effect,
`session!` for everything else. The host checks the ABI version (`craft_abi`, version 1).

### Effects

From `examples/cross-vignette/src/lib.rs`:

```rust
use craft_guest::{FilterCtx, Image, filter};
fn apply(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let amount = ctx.f("amount")? / 100.0;
    let round = ctx.choice("shape")? == "round";
    let radius = ctx.f("radius")? * ctx.scale;
    let [w, h] = ctx.full.map(f64::from);
    for y in 0..img.height {
        for x in 0..img.width {
            let dx = ctx.origin[0] + x as f64 + 0.5 - w / 2.0;
            let dy = ctx.origin[1] + y as f64 + 0.5 - h / 2.0;
            let (rx, ry) = if round { (radius, radius) } else { (w / 2.0, h / 2.0) };
            let gain = (1.0 - amount * ((dx / rx).powi(2) + (dy / ry).powi(2)).min(1.0)) as f32;
            let mut p = img.get(x, y);
            for c in &mut p[..3] { *c *= gain; }
            img.set(x, y, p);
        }
    }
    Ok(())
}
filter!(apply);
```

- `img.pixels` is premultiplied RGBA `f32`, row-major. Values may exceed 1.0. The host refuses
  output with non-finite values and keeps alpha within 0 to 1.
- `ctx.color` is the working space: `"linear-srgb"` or `"srgb"`. `ctx.origin`, `ctx.full` and
  `ctx.scale` place the buffer in the full frame; use them so tiles agree.
- `ctx.quality` is `"draft"` or `"final"`. `ctx.time` is layer time in seconds. `ctx.seed` is the
  only source of randomness you may use.
- Output must follow only from the context and pixels. The host may create a new instance at any
  time, and `craft-ext test` fails a non-deterministic effect.
- Read parameters with `ctx.f`, `int`, `b`, `choice`, `text`, `color` and `point`. A missing or
  mistyped value is an `Err`.
- Returning `Err` logs the message and discards the result.
- Effects have no `call` import: they cannot run operations.

In PhotoCraft an effect runs destructively on the active RGB layer as one history step; other
colour modes, masks and channel subsets show the reason it is unavailable. In EffectCraft an effect
is an ordinary effect instance (animatable, with Effect Controls).

### Commands, panels, tools and hooks

A session module receives events. From `examples/cross-review/src/lib.rs` and `review.rs`:

```rust
use craft_guest::{Event, session};
fn handle(event: Event) -> Result<(), String> {
    if let Event::Command { id } = event {
        if let Some(app) = review::App::from_command(&id) {
            review::rename(app)?;   // calls "layer.rename" or, in FigCraft, "node.update"
        }
    }
    Ok(())
}
session!(handle);
```

| Event | When |
|---|---|
| `Event::Init` | After the module loads, after a reload, after Resume. FigCraft also sends it when a document opens. Draw your panels here. |
| `Event::Command { id }` | A command contribution ran. `id` is the contribution id. |
| `Event::Ui { surface, rev, widget, value }` | A widget in your panel or window changed. Events for an old `rev` are dropped before they reach you. |
| `Event::Tool { phase, at }` | A pointer gesture of your selected tool: `"begin"`, `"update"`, `"commit"` or `"cancel"`. |
| `Event::App { name, payload }` | An app event, if the contribution declares `app.events` and it is granted. |
| `Event::Dispose` | Reserved. Not delivered in this beta; never rely on it for cleanup. |
| `Event::Unknown(json)` | Any other event, as raw JSON, for example PhotoCraft's `files.picked`. |

App events per app: EffectCraft sends `selection.changed`, `time.changed` (at most 4 per second),
`document.changed`, `document.opened` and `render.finished`. PhotoCraft sends `document.opened` and
`selection.changed`. FigCraft sends `selection.changed`, `document.opened`, `page.changed` and
`document.changed`; a `document.changed` event cannot edit the document. EffectCraft and PhotoCraft
deliver app events only to `event-hook` contributions. FigCraft delivers them to any contribution
that declares `app.events`.

Run an app operation with `craft_guest::call(op, args)`. Operation names, arguments and
permission rules are on each app's contract page, or run `<app>-cli plugin contract`.

Panels and windows are host-drawn trees. `ui::set_panel(panel, tree)` draws a panel the manifest
declares (free). `ui::open_window(id, title, tree)` opens a window (`ui.window`; at most four per
plugin in EffectCraft and FigCraft, at most 32 surfaces per app in PhotoCraft). Each control needs a stable id and a visible label. Trees and overlays are published only
when the event commits. The app chooses the renderer; a plugin cannot choose or detect it.

Handling a missing optional permission, from `examples/color-notes/src/lib.rs`:

```rust
fn add_marker() -> Result<Option<String>, String> {
    match craft_guest::call("marker.add", json!({"label": "Note"})) {
        Ok(_) => Ok(None),
        Err(CallError::PermissionNeeded(_)) => Ok(Some(format!(
            "Adding markers needs \"{NEED_EDIT}\". Turn it on in the plugin's settings."))),
        Err(e) => Err(e.to_string()),
    }
}
```

`CallError` maps the host codes: `Invalid` (-1), `PermissionNeeded` (-2), `NotFound` (-3), `Stale`
(-4, read again before retrying), `Cancelled` (-5), `Budget` (-6) and `Failed` (-7 and others).
`craft_guest::log(level, text)` writes to the plugin console (4 KiB per line, rate-limited).

A tool, from `examples/effectcraft-point-tool/src/lib.rs`:

```rust
fn handle(event: Event) -> Result<(), String> {
    if let Event::Tool { phase, at } = event {
        if phase == "commit" {
            let selection = craft_guest::call("selection.get", json!({})).map_err(|e| e.to_string())?;
            for layer in selection["layers"].as_array().into_iter().flatten() {
                craft_guest::call("prop.addKey", json!({"layer": layer["id"],
                    "path": "transform/position", "value": at, "timeBase": "comp"}))
                    .map_err(|e| e.to_string())?;
            }
        } else if phase == "begin" || phase == "update" {
            craft_guest::call("canvas.overlay.set", json!({"ops": [{"op": "circle",
                "center": at, "radius": 5, "fill": [0.3, 0.7, 1, 1]}]}))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
```

`at` is in composition pixels (EffectCraft), image pixels (PhotoCraft) or document units
(FigCraft). Overlays are cleared when the gesture ends.

### State

A module instance lives on its own worker thread and keeps its memory between events (for example
in a `static Mutex`, as `examples/photocraft-levels-scope` does). That memory is lost when plugins
reload, when the plugin is resumed, and when the app quits. There is no persistent storage.

## 7. Effect forms

`[contribution.form]` (schema 1) describes an effect's controls. Parameter kinds are `range`,
`int`, `number`, `bool`, `choice`, `radio`, `angle`, `color`, `point`, `text` and `section`.
Choices store option ids, never indices. Colours are straight RGBA in the working space. Angles are
degrees. Points declare their space. `visible_when` and `enabled_when` take conditions such as
`{eq = {key = "shape", value = "round"}}`, `all`, `any` and `not`. Limits: 128 rows; conditions at
most 16 deep and 256 nodes.

From `examples/cross-vignette/craft-plugin.toml`:

```toml
[contribution.form]
schema = 1
key = "vignette"
title = "Vignette"
[[contribution.form.params]]
key = "shape"
label = "Shape"
[contribution.form.params.kind]
type = "choice"
default = "oval"
options = [{id = "oval", label = "Oval"}, {id = "round", label = "Round"}]
[[contribution.form.params]]
key = "radius"
label = "Round radius"
unit = "pixels"
visible_when = {eq = {key = "shape", value = "round"}}
[contribution.form.params.kind]
type = "range"
default = 100
min = 1
max = 2000
```

You can also build the form in code with `craft_guest::Form` and export it with
`filter!(describe = describe, apply = apply)`. `craft-ext pack --describe` runs `describe` in the
reference sandbox and embeds the result in the packaged manifest. With
`filter!(describe = d, update = u, apply = a)`, EffectCraft and PhotoCraft call `update` after
edits; it returns a `Patch` (`set`, `disable`, `enable`, `hide`, `show`) that the host applies
atomically at the current revision.

Preset files (`resource.preset@1.0`) hold `{"presets": [{"id", "label", "effects": [{"effect":
"<plugin id>/<contribution>", "values": {...}}]}]}`; see `examples/film-looks`. LUTs are `.cube`
files.

## 8. Transactions, undo and failure

- **One event is one transaction and one undo step**, labelled `<label> · <plugin name>`. The
  host opens it before your handler runs and commits it when the handler returns `Ok`.
- **Returning `Err` rolls back** every document edit, panel tree and overlay from that event.
  To show an error in your panel, catch it, redraw, and return `Ok` (as Cross Review does).
- A failed operation does not end the event; you can handle it and continue.
- **A tool gesture is one undo step** from `begin` to `commit`. A failed event, Escape (`cancel`), a
  document switch or a revocation rolls the whole gesture back. While a gesture is open, all three
  apps refuse Undo and other edits; view navigation still works.
- **Revocation and disabling stop work.** Granting, revoking, disabling a contribution and native
  consent changes all bump the grant generation. Every operation and every commit checks it, so the
  running event fails and rolls back. The app then reloads plugins and unloads anything no longer
  enabled. Effect instances in EffectCraft render unchanged and show a badge while their plugin is
  missing.
- **Three failed events in a row pause the plugin** for the rest of the session. The person
  resumes it from the Plugins window, which starts fresh instances and sends `init`.
- **External effects are not undone.** Files written through PhotoCraft's `files.*` and
  `document.export` are published in order when the event commits, and stay if a later write
  fails. File operations are refused during a tool gesture. Native helpers' side effects outside
  the document are never undone.

### Sandbox budgets

| Limit | Value | Source |
|---|---|---|
| Module size | 32 MiB | `ext-host/src/wasm.rs` |
| Memory | 512 MiB per effect, 64 MiB per session | same |
| One call's arguments | 1 MiB | same |
| Pending results | 8 MiB per instance, 64 MiB per app | same |
| Fuel per event | 2,000,000,000 | same |
| Fuel per effect run | 50,000,000 plus 4,000 per pixel | same |
| Document hold per event | 5 s, then the event rolls back | each adapter's runtime |
| Whole event | 8 s (EffectCraft, PhotoCraft), 9 s (FigCraft) | same |
| Form `describe`/`update` | 100 ms in PhotoCraft | `photocraft/crates/plugins/src/lib.rs` |
| Queued events per worker | 64 in EffectCraft; app events beyond it are dropped | `effectcraft/crates/plugins/src/runtime.rs` |
| Native helper | 10 s to start, 60 s per event, 1 MiB per line | `ext-host/src/native.rs` |

## 9. Multi-app packages

List every app in `apps` and give each app its own contributions with that app's session
contract. `examples/cross-review` ships `effectcraft-apply`, `photocraft-apply` and
`figcraft-apply` commands and three panels. Each app sees only the contributions whose contract it
provides, and `setup --all` enables only those. Effects need no per-app copy: one
`raster.filter@1.0` contribution runs in EffectCraft and PhotoCraft (see `examples/cross-vignette`).

The operations differ between apps: Cross Review calls `layer.list` and `layer.rename` in
EffectCraft and PhotoCraft, and `page.list`, `node.children` and `node.update` in FigCraft.

## 10. Native helpers

A native package runs a helper executable outside the sandbox. It has **full access to the
person's computer**. The person must give separate consent ("Native: full access to your
computer" in the setup sheet, `--full-access` on the CLI). Consent is per version: an update asks
again.

```toml
[native]
executables = { "linux-x86_64" = "bin/helper-linux-x86_64", "macos-aarch64" = "bin/helper-macos-aarch64" }
```

- Keys are `<os>-<arch>` as Rust names them: `linux-x86_64`, `macos-aarch64`, `windows-x86_64`.
- With a distinct path for each platform, one universal archive carries every helper. If several
  platforms share one path, publish one archive per platform.
- An app shows the package as unavailable when there is no helper for its platform.
- Native packages support commands, panels, tools and event hooks, not effects. Contributions need
  no `module`.
- The helper still needs the permissions its operations require. `examples/native-system-stamp`
  declares `document.edit:document` and needs `--full-access` as well.

The helper speaks protocol 1, JSON lines on stdin and stdout (`crates/ext-host/src/native.rs`):

```text
host → helper  {"type":"hello","protocol":1,"app":"effectcraft","contracts":[["effectcraft.session","1.0"]]}
helper → host  {"type":"ready"}
host → helper  {"type":"event","id":7,"event":{"event":"command","id":"effectcraft-apply"}}
helper → host  {"type":"call","id":1,"op":"layer.newText","args":{...}}
host → helper  {"type":"result","id":1,"ok":...}      or {"type":"result","id":1,"error":{...}}
helper → host  {"type":"log","level":1,"text":"..."}
helper → host  {"type":"done","id":7,"code":0}
```

The helper gets a clean environment and its own working folder. stdout is reserved for the
protocol. The host kills the process on timeout, revocation, disabling or a kill-list hit.

## 11. Publish to the catalogue

1. Push your source to a public GitHub repository under your login.
2. Tag a release, then run `craft-ext pack --release-record --tag <tag> --commit <sha> --url
   <release asset URL>` and upload the `.craftplugin` to that GitHub release. Never replace an
   uploaded file.
3. Open a pull request to the catalogue repository with your vendor record (first time), your
   listing, and the release record. Run `craft-ext catalogue check --repo <checkout> --only <id>`
   first.

The full workflow, vendor registration and store rules are in
[store-and-distribution.md](store-and-distribution.md).

## 12. What plugins cannot do

- Change or remove built-in commands, undo, history, settings, shortcuts or plugin management.
  These are not in any contract, so no operation reaches them.
- Read or call other plugins' data.
- Run processes, open sockets or read arbitrary files from sandboxed code. The only wasm imports
  are `craft.log`, `craft.call`, `craft.result` and `craft.take`. Use a native helper if you need
  the computer.
- See file paths. PhotoCraft's picked files arrive as opaque handles, valid for that plugin and
  that item until the app quits.
- Choose or detect the UI renderer.

## 13. Things you cannot do yet

- **No persistent storage.** No operation saves plugin-private data, per plugin or per document.
  Memory is lost on reload, Resume and quit. You can only write ordinary document content through
  edit operations.
- **No network or clipboard.** The permissions parse, but no operation uses them.
- **No long-running work.** An event may hold the document for about 5 s and must finish within 8
  or 9 s, within its fuel. There are no background jobs.
- **The SDK drops some event fields.** `Event::Tool` exposes only `phase` and `at`, although the
  apps send `modifiers` (and PhotoCraft also `button`, `pressure`, `start` and `zoom`).
  `Event::Command` exposes only `id`, although PhotoCraft and FigCraft send `params`.
- **Panel form patches only in FigCraft.** FigCraft's `ui.panel.set` and `ui.window.open` accept
  `{panel|id, patch}` to patch a panel's root form in place. EffectCraft and PhotoCraft do not.
  (Effect-dialog patches through `craft_update` work in EffectCraft and PhotoCraft.) The SDK has no
  helper for panel patches.
- **No panel open or close notifications.** Opening or closing a panel sends no event.
- **Fonts and themes are not activated.** They can be installed and enabled where the contract is
  provided, but no app loads them.
- **No project references in PhotoCraft.** PhotoCraft does not record which plugins a document
  used, so it shows no missing-plugin prompt.

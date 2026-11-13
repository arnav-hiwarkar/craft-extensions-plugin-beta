# Making Craft plugins (author guide, beta)

One plugin can work in several ArtCraft apps. You write it once against **contracts** (what an app
offers), declare what you need in `craft-plugin.toml`, and publish it from your own GitHub Releases.
The design behind this is [beta-design.md](beta-design.md).

## 1. What you can make

| Kind (`kind =`) | Runs | Contract | Where it appears |
|---|---|---|---|
| `effect` | Sandboxed wasm, `craft_filter` | `raster.filter@1.0` | EffectCraft effects, PhotoCraft filters, FigCraft image fills |
| `panel` | Sandboxed wasm or native, `craft_event` | `<app>.session@1.0` | A dockable panel the app draws from your UI tree |
| `command` | same | `<app>.session@1.0` | Menus, command search, the app's CLI and MCP |
| `tool` | same | `<app>.session@1.0` | FigCraft only: pointer tool (begin / update / commit / cancel) |
| `event-hook` | same | `<app>.session@1.0` | Reacts to app events (needs `app.events`) |
| `preset`, `lut`, `font`, `theme`, `data` | Nothing runs | `resource.*@1.0`, `craft-theme@1.0` | Preset and LUT lists; font and theme assets are stored only |

Panels can open host-drawn **windows** (`ui.window.open`, needs `ui.window`). Panels and windows
are described as data and drawn by the app; the full vocabulary (tabs, tables, lists, canvases with
paths and gradients, pointer and key events, icons) is in the [UI reference](ui.md). Plugins can add
features, but they can never change or remove built-in commands, shortcuts, settings, undo or plugin
management.

**What each app offers** is generated from its adapter: [contracts/effectcraft.md](contracts/effectcraft.md),
[contracts/photocraft.md](contracts/photocraft.md), [contracts/figcraft.md](contracts/figcraft.md), or
run `<app>-cli plugin contract`. Each page lists contracts and versions, operations with their
arguments, results and permission rules, events and where contributions appear.

## 2. Start

```sh
craft-ext new effect my-effect --id io.github.<your-github-login>.my-effect   # or: panel, data
cd my-effect
craft-ext build      # debug profile, optimization level 1, copies the .wasm; --release opts in
craft-ext check      # manifest, ABI signatures, licence, permission hints
craft-ext test       # runs each effect twice on a test image: deterministic and valid output
craft-ext dev . --app effectcraft   # installs into your local store as "Development"
```

Then open the app's **Plugins** window to set it up, or run
`effectcraft-cli plugin setup <id> --enable <contribution> --grant <permission>`.

## 3. The manifest

```toml
schema = 2
id = "io.github.ink.grain"            # reverse-domain; io.github.<login> must be your GitHub login
name = "Ink & Grain"
version = "1.2.0"                     # semver; one version = one file forever
license = "MIT OR Apache-2.0"         # SPDX; the store requires OSI-approved terms only
source = "https://github.com/ink/grain"
summary = "Halftone and grain effects"
apps = ["effectcraft", "photocraft", "figcraft"]

[why]                                 # your reason, shown under the app's own sentence
"network:api.example.com" = "Downloads halftone screens"

[[contribution]]
id = "halftone"
kind = "effect"
contract = "raster.filter@1.0"        # contract and minimum version
module = "halftone.wasm"
label = "Halftone"
category = "Stylize"
permissions = []                      # required: granted before it can be turned on
optional_permissions = ["network:api.example.com"]   # off until the person allows them
state_version = 0                     # bump when saved state changes shape
[contribution.form]
schema = 1
key = "halftone"
title = "Halftone"

[[contribution.form.params]]
key = "size"
label = "Dot size"
[contribution.form.params.kind]
type = "range"
default = 8
min = 1
max = 64
```

Native plugins add `[native] executables = { "linux-x86_64" = "bin/helper", "macos-aarch64" = "bin/helper", "windows-x86_64" = "bin/helper.exe" }`
and ship one archive per platform. Native code gets **full access to the user's computer**. Users see
that label and give separate consent, and every update asks again. Native packages support panels,
commands, tools and event hooks, but not effects. They speak the JSON-lines protocol documented in
`crates/ext-host/src/native.rs`.

## 4. Compatibility and versions

- A contribution works in an app when the app provides the same contract **major** at the same or a
  higher **minor**. Apps keep a major working for at least 12 months after its successor ships. Your
  plugin therefore keeps working as apps update, without a rebuild.
- A package can be **partly compatible**: the manager shows each unavailable contribution with the
  reason.
- Bump your **major** version when you remove a contribution or parameter, or change saved state
  without a migration. Users can choose any listed version, pin it, and roll back.

## 5. Permissions

Declare the smallest set, per contribution. The host checks every call against what *that*
contribution declared **and** what the person granted in *that* app. Revoking a permission stops the
work in progress and rolls back its edit.

| Permission | Allows |
|---|---|
| `document.read:selection` / `:document` / `:all` | Read the selection, the current document, or all open documents |
| `document.edit:selection` / `:document` / `:all` | Change them (edit includes read) |
| `files.read:picked`, `files.write:picked` | Files and folders the person picks in the app's dialog |
| `network:<host>` or `network:*.<domain>` | https to those hosts only, never local addresses |
| `clipboard.read`, `clipboard.write` | The clipboard, on a user action |
| `ui.window` | Open floating windows (panels are free) |
| `app.events` | Receive app events |

Optional permissions return `CallError::PermissionNeeded`. Handle that case, for example by showing
a label that tells the person to turn the permission on.

## 6. Code

The SDK is `craft-guest` (`crates/guest`). Effects:

```rust
use craft_guest::{filter, FilterCtx, Image};
fn exposure(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let gain = 2f32.powf(ctx.f("stops")? as f32);
    for px in img.pixels.chunks_exact_mut(4) { for c in &mut px[..3] { *c *= gain; } }
    Ok(())
}
filter!(exposure);
```

Pixels are premultiplied RGBA `f32` in the app's working space (`ctx.color`), and values may exceed
1.0. `ctx.origin`, `ctx.full` and `ctx.scale` place the buffer in the frame. Output must be
**deterministic** and must not depend on earlier calls: the host may restart your module at any time.

Sessions:

```rust
use craft_guest::{session, ui, Event};
fn handle(event: Event) -> Result<(), String> {
    if let Event::Init = event {
        ui::set_panel("notes", ui::column(vec![ui::label("Hello"), ui::button("go", "Go")])).map_err(|e| e.to_string())?;
    }
    Ok(())
}
session!(handle);
```

Each event is **one transaction and one undo step**, labelled with your contribution and plugin name.
`craft_guest::call(op, args)` runs an app operation from its contract page. UI trees are drawn by the
app with an engine the app chooses; you cannot choose or detect it. Every control needs a stable id
and a visible label, events carry the tree revision, and edits arrive when they end (a released
drag, Enter). See the [UI reference](ui.md) for every node, draw op, event and limit.

**UI levels.** The original nodes are ui 1.0. If your tree uses anything newer (tabs, tables,
lists, number fields, toggles, icons, hints, paths, gradients, canvas wheel/key events…), declare the
session contract at `@1.1`, for example `contract = "effectcraft.session@1.1"`, so an older app
asks the person to update instead of failing to draw your panel. `craft_guest::ui::level(&tree)`
tells you which level a tree needs; `ui::check(&tree)` runs the app's validation in your tests.

Limits: fuel (time), memory (512 MiB for effects, 64 MiB for sessions), 1 MiB per call, 8 MiB pending
results. If a plugin fails three times in one session, it is paused.

## 7. Publish to the store

1. Register once as a vendor in [craft-plugins](https://github.com/arnav-hiwarkar/craft-plugins)
   (`vendors/<login>.toml`). Verified vendors get a ✔ badge.
2. `craft-ext pack --release-record --tag v1.2.0 --commit <sha> --url <release asset URL>`, then
   upload the `.craftplugin` file to that GitHub Release. Never replace an uploaded file.
3. Open a PR adding `plugins/<id>/listing.toml` (first time) and `releases/<version>.toml`. Run
   `craft-ext catalogue check --repo <craft-plugins> --only <id>` first.
4. One maintainer checks and merges. The release appears in apps after the next signed build.

Store rules: free and open source only (OSI licences, public source, no paywalls, trials or licence
keys), honest permissions, and your own namespace. Plugins are never refused for quality or for
competing with built-in features.

Beta availability: `tool` contributions work in FigCraft only. Font and theme contributions are stored and listed but are not activated in this beta.

`craft-ext test` runs effects twice for deterministic, valid output. For packages with no effects it exits successfully with `"tested": []` and the message "No automated tests for panels/commands yet; run in the app with `craft-ext dev`". This does not verify panels or commands.

Standalone `craft-ext plugin project <references.json>` reads a JSON array of plugin references. App CLIs implement `plugin project <project-file>` by extracting references and calling `craft_ext_cli::print_project_status`.

## Forms (schema 1)

`[contribution.form]` replaces parameter tables. It contains `schema = 1`, `key`, `title`,
and `[[contribution.form.params]]` rows with `key`, `label`, and a nested `kind` tagged by `type`.
Words accept a string or `{ id, english }`; `t!("Title")` gives the SDK word a plugin-crate id
and English fallback. Choices/radios store string option ids, never indices.

```rust
use craft_guest::{Form, Unit, t};
fn describe() -> Form {
    Form::new("vignette", t!("Vignette"))
        .slider("amount", t!("Amount"), -100.0..=100.0, 0.0).unit(Unit::Percent)
        .choice("shape", t!("Shape"), &[("round", t!("Round")), ("oval", t!("Oval"))], "round")
        .preview()
}
// filter!(describe = describe, apply = apply);
// Optional: filter!(describe = describe, update = update, apply = apply);
```

Update functions take `Update { values, rev }` and return `Patch::none(rev)` with chained
`.set(key, Value::Number(n))`, `.disable(key)`, `.enable(key)`, `.hide(key)`, or `.show(key)`.
The host must apply a patch atomically with `Form::apply_patch` to its `FormState`.
Conditions use `{"eq":{"key":"mode","value":"round"}}`, `all`, `any`, or `not`.
Forms allow 128 rows; conditions allow depth 16 and 256 nodes. Invalid values are refused,
including non-finite and out-of-range numbers. Color defaults are straight RGBA in the host
working space (HDR RGB allowed); angles use degrees and points declare their coordinate space.
`Params(&values)` provides `f/int/b/choice/color/point/text` access.

Write the form in the manifest, or use `craft-ext pack --describe` to load the already compiled
wasm in the reference sandbox and embed its validated description in the packaged manifest.
Plain `pack` never invokes guest code; neither command builds anything. The source manifest
is unchanged. A missing optional description leaves its manifest form intact.
Preset effect entries are `{ "effect": "plugin/contribution", "values": { ... } }`.

## Current beta portability and failure handling

Use the app contract references to select operations and permissions. Session 1.1
adds tools and shared UI in EffectCraft and PhotoCraft; UI 1.0 remains supported.
Describe UI as data and provide component fallbacks. The host chooses its renderer;
plugins cannot select or detect it. Cross Review demonstrates session contributions
across all three apps; Cross Vignette targets EffectCraft and PhotoCraft because
FigCraft has no raster.filter implementation.

Handle operation refusals and stale revisions. Do not assume events in separate
modules share ordering or state, and do not depend on dispose for cleanup. Native
helpers require separate full-access consent and platform-specific executables;
the system-stamp example currently declares Linux only. Build real Mac/Windows
helpers before adding those platform mappings.

PhotoCraft file publication can leave earlier files after a later failure or
cancellation. The error reports partial effects; document/UI rollback does not
undo external writes. Picker replies are single-use and document-scoped. Reissue a
question after switching documents. No parent-directory capability sandbox is
claimed. Refresh its reference without a build with
`python3 scripts/photocraft-contract-source.py`; Rust metadata is authoritative.

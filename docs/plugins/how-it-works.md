# How the plugin system works

This page explains the moving parts of the Craft plugin platform and how a plugin gets from the
catalogue into an app. Behavior descriptions are based on inspected source. The repository's
[status record](../../STATUS.md#latest-green-run) records
[CI run #20](https://github.com/arnav-hiwarkar/craft-extensions-plugin-beta/actions/runs/43551011455)
at `909897f`: Linux x86_64 and macOS arm64 builds and smokes passed; Windows was compile-only.
That evidence applies to that revision and those checks, not GUI or GPU behavior. See
[evidence coverage](coverage.md) for limits. No local CLI, app or platform checks were performed
for this documentation correction.

For what each app offers plugins, see [capabilities.md](capabilities.md). For the UI vocabulary, see
[ui.md](ui.md). For the catalogue and signing, see
[store-and-distribution.md](store-and-distribution.md).

See also the [workflow maps](maps.md), [formats and versioning](formats-and-versioning.md),
and [native protocol reference](native-protocol.md).

## Components

| Component | Crate | What it does |
|---|---|---|
| Core types | `crates/ext-core` (`craft-ext-core`) | Ids, versions, the `craft-plugin.toml` manifest, contributions, permissions, contracts and compatibility, catalogue records, project references, UI tree types and validation. Pure: no files, no network. |
| Store | `crates/ext-store` (`craft-ext-store`) | The on-disk store (`store-v2.json`, `releases/`), per-app setup (active version, pin, enabled contributions, grants, native consent, grant generation), the cached kill list, catalogue fetch and verification, and the `Manager` facade. |
| Host runtime | `crates/ext-host` (`craft-ext-host`) | The wasm sandbox (`wasm.rs`), the `AppApi` trait and the single `authorize` check (`api.rs`), and the native helper protocol (`native.rs`). |
| Shared UI | `crates/ext-ui` (`craft-ext-ui`) | The egui plugin manager (Browse, Installed, Updates, Problems, setup sheet), the "This project uses N plugins" prompt, and `plugin_ui`, the one renderer for plugin panels, windows, forms and draw lists. |
| CLI | `crates/ext-cli` (`craft-ext-cli`) | The `plugin …` command library embedded by every app, and the `craft-ext` binary for authors (`new`, `build`, `check`, `test`, `pack`, `dev`) and maintainers (`catalogue …`). |
| Guest SDK | `crates/guest` (`craft-guest`) | What a plugin author links into a wasm module: the `filter!` and `session!` entry macros, form and UI builders, and `craft::call` wrappers. |
| App adapters | `effectcraft/crates/plugins`, `photocraft/crates/plugins` (with `photocraft/crates/engine/src/plugin_ops.rs`), `figcraft/crates/plugins` | Each implements `AppApi` for its app: the contract it provides, the operations plugins may call and the permission each needs, how events reach plugins, and how panels, tools and overlays appear. |

The launcher (`craft-launcher/crates/ui-egui/src/plugins.rs`) embeds the same `ManagerView` and
`Manager`. It is never required: every app manages its own plugins.

```text
           craft-plugins (catalogue repo)          author's GitHub Releases
           dist/index.json(.sig), packages/*.json   *.craftplugin files
                         │ signed metadata                 │ bytes, pinned by sha256+size
                         ▼                                 ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │ craft-ext-store: Manager ── Store (store-v2.json, releases/, data/)    │
 └────────▲──────────────────▲───────────────────▲────────────────────────┘
          │                  │                   │
   app manager window    <app>-cli plugin …   craft-ext plugin --app <app> …
   (craft-ext-ui)        (craft-ext-cli)      launcher Plugins dialog
          │
          ▼  enabled contributions + live GrantView
 ┌──────────────────────────── app process ───────────────────────────────┐
 │ adapter (AppApi) ── authorize() ── canonical engine commands ── undo    │
 │      ▲                                                                  │
 │      │ craft.call(op, args)                                             │
 │ wasm sandbox (wasmi)            native helper (separate process,        │
 │ filters and sessions            JSON lines on stdin/stdout)             │
 └────────────────────────────────────────────────────────────────────────┘
```

## Contracts decide what runs where

An app *provides* a list of contracts such as `raster.filter@1.0` or `effectcraft.session@1.1`. Each
contribution in a manifest names one contract and a minimum level. A need is met when the app
provides the same major at an equal or higher minor (`crates/ext-core/src/contract.rs`). A package
can therefore be compatible, partly compatible (some contributions unavailable, each with a reason)
or incompatible. Compatibility never depends on the app's own version number.

## Lifecycle

```text
discover ─▶ install ─▶ setup sheet ─▶ enable ─▶ run
                         (grants,               │
                          native consent)       ▼
                       update / pin / rollback ◀┤
                       revoke / kill list ◀─────┤
                       remove ◀─────────────────┘
```

1. **Discover.** `Manager::search` runs one shared search over the cached, signature-checked
   catalogue (`crates/ext-core/src/catalogue.rs`, `search`): filter by app and kind, then rank by
   match, tier and stars. Packages on the kill list with severity `disable` are never returned.
2. **Install.** `Manager::install` resolves the newest release that this app can use and that no
   `block-new` or `disable` revocation withdraws, including declared package dependencies. It
   downloads each archive, checks its size and SHA-256 against the signed metadata, checks that the
   manifest inside equals the catalogue record, and unpacks it under the store lock
   (`crates/ext-store/src/manager.rs`, `store.rs`). `install-file` does the same for a local file
   with tier **Local**. **Install never enables anything** (source behavior).
3. **Setup and permissions.** The setup sheet (or `plugin setup`) shows the union of the required and
   optional permissions of the chosen contributions, each as a sentence the host writes, followed by
   the author's `[why]` text. Required permissions must be granted before a contribution can be
   enabled; setup without them is refused (source behavior). Native packages need a separate "full access
   to your computer" consent (`--full-access`).
4. **Enable.** Enabling is per app and per contribution. `Store::set_enabled_checked` refuses a
   contribution whose contract is not provided, whose kind the app marks unsupported, whose
   permissions are not granted, whose native consent is missing, or whose release is withdrawn.
5. **Run.** The app loads enabled contributions of the active release
   (`GrantView::enabled_contributions`). Filters run in render workers. Commands, panels, tools and
   event hooks run as sessions on a per-plugin worker.
6. **Update, pin, rollback.** An update installs the new release beside the old one. The app moves to
   it only when consent is complete: if the new release needs a permission that is not granted, or
   is a native release, it stays `pending` and the old release keeps running until the user accepts
   (`Store::set_active`, `accept_pending`). A pin (`plugin pin <id> <req>`) limits which versions
   updates may choose. Rollback activates any installed or listed version, with a warning when the
   older contribution has a lower `state_version` than saved data may need.
7. **Revoke and kill list.** Revoking a grant, disabling a contribution or withdrawing native consent
   bumps that plugin's grant generation, so its running work stops (see below). A `disable`
   revocation in the catalogue stops matching releases in every app on the next refresh.
8. **Remove.** `plugin remove` deletes the package's releases, every app's setup and grants, and its
   private storage `data/<app>/<id>/`. It refuses while another installed package depends on it,
   unless `--force` is given.

## The wasm sandbox

Code plugins are WebAssembly modules run by wasmi 2 with one locked configuration that apps cannot
loosen (`crates/ext-host/src/wasm.rs`):

- **Imports.** Only `craft.log`, `craft.call`, `craft.take` and `craft.result` exist. A module that
  imports anything else fails to load. There is no WASI: no file paths, no processes, no sockets, no
  clock or environment.
- **Exports.** `memory`, `craft_alloc`, `craft_reset` and `craft_abi` (which must return ABI 1) are
  required. Filters export `craft_filter`; sessions export `craft_event`. Optional `craft_describe`
  and `craft_update` supply runtime forms.
- **Memory.** One memory per instance, capped at 512 MiB for filter instances and 64 MiB for session
  and form instances. Modules are capped at 32 MiB. Guest pointers are bounds-checked with checked
  arithmetic.
- **Fuel.** Execution is metered and runs in slices of 20 million fuel. Between slices the host checks
  cancellation, the deadline and, for sessions, whether the plugin was stopped or its grant generation
  changed. Budgets: 50 million base fuel per call; a filter gets an extra 4 000 per pixel; one session
  event, `craft_describe` or `craft_update` gets at most 2 billion.
- **Host calls cost fuel.** Each `craft.call` costs 100 000 fuel plus 100 per argument byte, and each
  `craft.log` 10 000 plus 100 per byte (at most 4 KiB read), so a loop of host calls cannot run
  unbounded work inside one slice.
- **Host-held bytes.** A pending call result may be at most 8 MiB per instance and 64 MiB across all
  plugin instances in the process; over budget the call returns `-6`. Call arguments are at most 1 MiB.
- **Failures.** A trap, fuel or memory exhaustion, or invalid output never panics the host. The
  instance is discarded and a fresh one is made. Filter output must be finite; alpha is clamped to
  0–1 and colour is zeroed where alpha is zero. Three consecutive trap, time or memory failures pause
  a session plugin until the user resumes it.
- **Determinism.** Filters must not depend on invocation history: the host may re-instantiate at any
  time. `scripts/cross-app-smoke.py` compares two EffectCraft vignette renders byte for byte;
  this is not proof that arbitrary plugins are deterministic.

Transforms have no session, so a filter that calls `craft.call` gets `-2` (denied).

## Native helpers

A package with a `native.executables` table (one entry per `os-arch`, for example `macos-aarch64`)
runs a native helper instead of a wasm module (`crates/ext-host/src/native.rs`). Native kinds are
command, panel, event-hook and tool; native packages cannot contribute effects
(`crates/ext-core/src/manifest.rs`).

- **Out of process.** The host starts the executable with an empty environment (only
  `CRAFT_PLUGIN_PROTOCOL=1`), a private working folder, and stderr discarded. It speaks protocol 1 as
  JSON lines on stdin/stdout, with helper output capped at 1 MiB per line: `hello` → `ready`, then `event`, `call` /
  `result`, `log` and `done`. Start-up must finish in 10 s; one event may take at most 60 s.
- **Separate consent.** The manager shows "Native: full access to your computer". Without that
  consent the helper is never started.
- **Consent is per version.** Activating a different version of a native package clears the consent
  and bumps the grant generation, so an update asks again (`Store::set_active`).
- **Checks.** Before each event, for each call it serves, and before commit, the host checks consent,
  the kill list and the grant generation. All three adapters also run the same `authorize` permission
  check on every native call as they do for wasm calls, so a native contribution still needs its
  declared permissions granted. Native System Stamp needs `--full-access` consent plus
  `document.edit:document`, and its edit undoes in one step (source behavior).
- **Supervision.** Adapters call `Helper::supervise`, which kills the helper as soon as consent is
  withdrawn or the plugin is stopped, even while it is idle. A stop during an event rolls the event
  back and kills the helper.

**This is not an OS sandbox.** The helper can do anything the user's account can. Killing it cannot
undo what it already did, such as files it wrote or data it sent. The helper runs in its own process
group, and stopping it stops that whole group (`crates/ext-host/src/native.rs`); a process that leaves
the group on purpose is not covered.

## Permissions are checked on every host call

Permissions are `family:scope` strings: `document.read:{selection|document|all}`,
`document.edit:{selection|document|all}`, `files.read:picked`, `files.write:picked`,
`network:<host>`, `clipboard.read`, `clipboard.write`, `ui.window` and `app.events`
(`crates/ext-core/src/permission.rs`). A broader scope covers a narrower one, and edit covers read
for the same scope. Panels, menus, commands, effects, own storage and logging need no permission.

Every operation an adapter offers has a `requires` function that computes the permissions from the
real arguments. `craft_ext_host::api::authorize` runs before the adapter executes anything:

1. The invocation is not cancelled or past its deadline.
2. The plugin is not stopped in this app (kill-listed, no longer the active release, or nothing
   enabled).
3. The plugin's grant generation equals the one captured when the event started.
4. The operation exists in the app's contract. Unknown operations return `-3`. Management, undo,
   settings and plugin-management operations are never listed, so they are unreachable.
5. Each required permission is **declared** by the invoking contribution (so one contribution cannot
   borrow another's rights) **and** currently **granted** to the package in this app.

Adapters whose `requires` reads app state (for example the selection) authorize on the thread that
owns the document. EffectCraft, PhotoCraft and FigCraft all run `authorize` there for each call.

**Live grants.** Each app keeps a `GrantView` over the store (`crates/ext-store/src/view.rs`). It
re-reads `store-v2.json` when its modification time or size changes, at most every 250 ms. Every
grant, revoke, consent change and contribution disable bumps the plugin's `grant_generation`
(verified for revocation). The sandbox compares the generation between fuel slices and the session
compares it again just before commit; a mismatch stops and rolls back the event.

**Captured scope.** At the start of an event the adapter captures the document and selection. For
example, EffectCraft's `layer.rename` needs only `document.edit:selection` when the target layer was
selected when the event began, and `document.edit:document` otherwise
(`effectcraft/crates/plugins/src/contract.rs`). The guest cannot claim selection scope through an
argument.

Apps reload plugins only when their own plugin setup changes. `StoreState::app_fingerprint` covers
this app's setups, the active release bytes and their kill-list status. A catalogue refresh, an
unrelated install or another app's change does not reload plugins.

## How plugin calls reach the app

```text
guest: craft.call("layer.rename", {...})
  │  fuel charged, args ≤ 1 MiB, JSON parsed
  ▼
host: authorize(contract, live grants, invocation, op, args)
  │  declared ∧ granted, generation unchanged, not stopped
  ▼
adapter: map op ─▶ canonical engine command (same path as menus, CLI, automation)
  │
  ▼
transaction opened by begin() … closed by finish(commit) ─▶ one labelled undo step
```

- Each adapter maps contract operations onto that app's canonical engine commands. EffectCraft runs
  document operations through `Session::execute` (`effectcraft/crates/plugins/src/ops.rs`). PhotoCraft
  executes engine commands on its document thread (`photocraft/crates/engine/src/plugin_ops.rs`,
  `plugin_sessions.rs`). FigCraft looks most operations up in its engine command registry
  (`figcraft_engine::find_command`) and maps a few, such as `node.update` to `node.set`
  (`figcraft/crates/plugins/src/runtime.rs`). No adapter exposes an arbitrary command dispatcher.
- **One plugin event is one undo step.** `AppApi::begin` opens a transaction when a command, UI event
  or tool gesture starts; `finish(true)` records one history step labelled
  "<contribution label> · <plugin name>". A whole tool gesture (begin … commit) is one step.
- **Rollback on failure.** The event rolls back when the guest returns non-zero, traps, runs out of
  fuel or time, is cancelled, or loses its grant before commit. A failed operation returns an error
  code to the guest without aborting the transaction; the guest may handle it and continue.
- `scripts/cross-app-smoke.py` checks Cross Review edits followed by one Undo in EffectCraft,
  PhotoCraft and FigCraft. The recorded CI result above covers that smoke at its recorded revision.
- EffectCraft and PhotoCraft refuse outside edits, such as Undo or menu commands, while a plugin tool
  gesture is open (`effectcraft/crates/engine/src/ext.rs`,
  `photocraft/crates/engine/src/session/execute.rs`).

## CLI and app managers reach the same manager

`craft_ext_store::Manager` is the single management API: search, info, versions, install, update,
rollback, pin, enable, disable, grant, revoke, remove, refresh, project check and report URL.

- `<app>-cli plugin …` is embedded by each app and calls `craft_ext_cli::run_plugin` with a manager
  for that app (`effectcraft/crates/plugins/src/lib.rs`, `photocraft/apps/photocraft-cli/src/lib.rs`,
  `figcraft/crates/plugins/src/lib.rs`).
- `craft-ext plugin --app <effectcraft|photocraft|figcraft> …` builds the same manager from a
  built-in table of each app's provided contracts (`crates/ext-cli/src/known_apps.rs`) and runs the
  same command code (`crates/ext-cli/src/main.rs`).
- The app's Plugins window and the launcher use `craft_ext_ui::ManagerView` over the same `Manager`.

All of them read and write one store, chosen by `CRAFT_EXT_DIR` or the per-OS default
(`crates/ext-store/src/location.rs`). Writes take a cross-process lock and atomically replace the
state with generation + 1. Enablement and grants are per app; installed releases are shared.

## Panels, windows, tools and overlays

Plugins never draw pixels of the interface. They send data; the host draws it.

- **Panels** are declared in the manifest (`kind = "panel"`). The plugin fills one with
  `ui.panel.set {panel, tree}`. No permission is needed.
- **Windows** are opened with `ui.window.open {id, title, tree}` and need `ui.window`.
- **Trees** are declarative JSON validated as a whole before anything is drawn; every widget event
  carries the tree revision so stale events are dropped. See [ui.md](ui.md).
- **Tools** (`kind = "tool"`) receive `tool` events with phases begin, update, commit and cancel.
- **Canvas overlays** are draw lists sent with `canvas.overlay.set {ops}`. The adapter validates them
  with `validate_draw_list` and draws them with `craft_ext_ui::plugin_ui::paint_ops`.

All three apps draw plugin UI with the shared renderer `craft_ext_ui::plugin_ui`; the engine is the
app's choice and plugins cannot see it.

## Safe mode

Safe mode starts an app with no plugin code running; the manager still works.

| App | How to start it |
|---|---|
| EffectCraft | `--safe-mode` or `EFFECTCRAFT_SAFE_MODE=1` (`effectcraft/crates/host/src/lib.rs`) |
| PhotoCraft | `PHOTOCRAFT_SAFE_MODE=1` (`photocraft/crates/plugins/src/lib.rs`) |
| FigCraft | `FIGCRAFT_SAFE_MODE=1` (`figcraft/crates/plugins/src/lib.rs`) |

## Revocation in practice

- **User revocation.** Revoking a permission or disabling a contribution in one app bumps the grant
  generation. Running work for that plugin stops at the next fuel slice or host call and rolls back;
  late results are discarded. A revoked contribution's command is refused (source behavior).
- **Kill list.** Each app refreshes the catalogue at start-up and about every 6 hours (schedules
  differ slightly per app; see [capabilities.md](capabilities.md)). The refresh stores the signed
  revocations in the store. A `disable` entry stops matching releases in every app, including local
  installs with the same id or digest, and the cached list survives restarts and metadata expiry.
  See [store-and-distribution.md](store-and-distribution.md#reports-and-the-kill-list).

## Projects and missing plugins

A project records the plugins it uses as `PluginRef { id, version, sha256, source, name,
contributions, state_versions }` and never stores grants (`crates/ext-core/src/project.rs`). Opening a
project never downloads, installs or enables anything. `Manager::check_project` classifies each
reference as ready, needs enable, needs another version, downloadable or unavailable (with a reason),
and the shared prompt "This project uses N plugins" offers Download, Enable, Skip, Open without them
and Continue. Missing contributions keep their saved parameters and state as placeholders.

| App | Project references |
|---|---|
| EffectCraft | Records references and shows the prompt. Skip and Open without them suppress the listed plugins for that document. Missing effects stay placeholders and rendering refuses, naming the plugin. |
| FigCraft | Records references and shows the prompt once per document. Skip and Open without them suppress the listed plugins for that document: their events are refused, their panels hidden and a running gesture cancelled. |
| PhotoCraft | Records no plugin references. Plugin filters apply destructively, so a document does not depend on a plugin later. |

A saved reference names only the contributions the project uses. After an update it moves to the loaded
release when that release reads the same saved-state version; after a state schema change it keeps the
recorded release, so reopening reports the mismatch instead of reading old state with new code.

`<app>-cli plugin project <file>` prints the same statuses with the `plugin install` and
`plugin setup` commands to run.

## Known limits

- Native helpers are not OS-sandboxed; a process that leaves the helper's group on purpose survives it.
- Private-address refusal for network requests is not implemented in shared code. There are no
  network operations, so nothing can make a request yet.
- A panel can ask for expensive polygon drawing on every frame.
- A pending PhotoCraft file question is dropped silently if that plugin reloads.

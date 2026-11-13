# Adding plugin support to a Craft app (adapter guide, beta)

Every app gets **one adapter crate** (`<app>-plugins`) in its own repository. It translates between
the shared platform, which knows packages, permissions, the sandbox, the store and the catalogue, and
the app, which knows its documents, undo, rendering and menus. It contains no copy of shared logic.
The EffectCraft (`effectcraft/crates/plugins`), PhotoCraft and FigCraft adapters are the reference
implementations.

## Checklist

| # | Job | Shared piece you call | App piece you wrap |
|---|---|---|---|
| 1 | Remove the app's old plugin system completely | — | old crates, commands, menus, docs |
| 2 | Describe what the app offers: `contract() -> AppContract` (provided contracts, ops with `requires`, events, slots, notes) | `craft_ext_host::api` | command registry |
| 3 | Implement `AppApi`: `begin` (capture scope, open one undo group), `call` (run the op through the app's normal command path), `finish(commit)` (one labelled undo step, or roll back) | `AppApi`, `authorize` (the host calls it for you) | undo/history, command execution |
| 4 | Implement `Grants` over `craft_ext_store::view::GrantView` and refresh it on every call and render | `GrantView` | — |
| 5 | Register enabled contributions at start and after store changes: effects/filters, commands, panels, tools, event hooks, data | `GrantView::enabled_contributions`, `wasm::Filter`, `wasm::Session`, `native::Helper` | effect/filter registry, menus, panels |
| 6 | Convert pixels honestly for `raster.filter@1`: premultiplied RGBA f32 in the named working space; refuse unsupported modes with a reason | `wasm::Filter::run` | render/filter pipeline |
| 7 | Run session ops on the document thread (worker → request/reply channel with a timeout) | `Session::dispatch` | main loop |
| 8 | Projects: write the plugin list and feature refs; on open call `Manager::check_project` and show `ProjectPrompt`; keep missing features as byte-for-byte placeholders; block final export only for missing rendering contributions | `craft_ext_core::project`, `ext-ui::ProjectPrompt` | project format, export |
| 9 | Management: embed `ext-ui::ManagerView` (a Plugins window and a Settings entry); add `<app>-cli plugin …` via `ext-cli::run_plugin` and `plugin contract` | `ext-ui`, `ext-cli` | menus, CLI |
| 10 | Safe mode: `<APP>_SAFE_MODE=1` loads no plugin code; the manager still works | — | startup |
| 11 | Publish the contract page to `craft-extensions/docs/plugins/contracts/<app>.md` and add the app to `ext-cli/src/known_apps.rs` | — | — |

## Rules

- **Mark every op honestly.** `requires(args)` must return every permission the call needs for
  *those* arguments. Never expose management, undo/history, settings, shortcuts or other plugins'
  data.
- **Add, never replace.** Plugin menu items go after built-ins and are grouped by plugin. Plugin
  shortcuts apply only when the chord is free.
- **Unload without restart.** Updates, disabling and kill-list hits must unload effects and close
  panels at the next safe point.
- **Contracts, not app versions.** Bump a contract's minor version when you add an op, field or event.
  A breaking change needs a new major that runs alongside the old one for 12 months. A new shared
  domain contract needs agreement from two apps.
- **Levels:** L0 (manager, CLI, data add-ons), then L1 (`raster.filter`), then L2 (sessions:
  panels, commands, tools, events). An app may ship L0 first.

## Commit and asynchronous replies

Revalidate captured document/surface/gesture identity, cancellation, live enablement
and grant generation on the owner thread before each external publication and
again before accepting document/UI commit. Include synchronous work in the event
deadline. File replacement is atomic per destination, not per event: on failure
stop later writes, roll back document/UI and report how many files remain. Do not
promise that Undo removes external files or native-helper effects. Acknowledge
Finish only after finalization. Native dispatch must return failure and terminate
the helper if its final consent check fails.

Bind picker replies to a single pending request and captured document generation;
discard stale/replayed replies before granting handles. Reauthorize the resulting
event and every file operation. Bound pending questions and file/pixel allocations.
Reject excess windows before acknowledging their call, counting staged windows
and existing windows together. Replacement of an existing ID consumes no new slot.

Beta workers serialize by plugin/module. Authors receive no package-wide ordering
or shared-memory guarantee. Revocation stops work immediately; a dispose callback
is reserved, not guaranteed. Opaque per-instance effect-state migration remains
deferred. See the app migration docs for supported contracts and limitations.

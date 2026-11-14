# Formats and versioning reference

The package, runtime, UI, catalogue and app project have different version domains.
Changing one does not implicitly change the others. This reference identifies the
owner of each domain and the compatibility rule a reader can rely on.

| Domain | Current format | Compatibility and owner |
|---|---|---|
| Package manifest | `craft-plugin.toml`, `schema = 2` | `ext-core::manifest`; unknown schema is refused. |
| Package archive | `.craftplugin`, ZIP | The validated manifest identifies modules/resources. Store extraction caps paths, entries and sizes and rejects unsafe contents. |
| Package version | Semver such as `1.1.0` | `ext-core::version`; updates and pins use version requirements. The signed release metadata binds version to archive digest. |
| Contribution contract | `name@major.minor` | Same contract name and major, provided minor at least the requested minor. `ext-core::contract` and app adapters own availability. |
| Wasm ABI | `craft_abi()` returns 1 | `ext-host::wasm` and `craft-guest`; unrelated to the app's marketing version. |
| Native transport | `protocol: 1` | `ext-host::native`; [record reference](native-protocol.md). |
| Effect form | `schema: 1` | `ext-core::form`; keyed parameters, types, defaults, conditions and validation. |
| Form patch | `schema: 1`, `base_rev`, increasing `rev` | Applies only to the matching form revision; validation is atomic. Unsupported or stale patches are refused. |
| Panel tree and draw list | UI levels 1.0 and 1.1 | `ext-core::ui::tree`; session contract minor carries the UI level. UI 1.0 remains supported. |
| Surface revision | Host-assigned `rev` | Identifies the tree shown. Every widget event carries it; stale or closed-surface replies are rejected. It is not a package version. |
| Catalogue index/package metadata | JSON, `schema: 1` | `ext-core::catalogue`; signed index binds hashed per-package metadata. |
| Catalogue serial and validity | Increasing serial, `issued`, `expires` | Highest accepted serial is retained. Older serial, future-issued or expired metadata cannot authorize new installs. |
| Local store | `store-v2.json`, `schema: 2` | `ext-store::state`; per-app active/pending version, contributions, grants, consent and pins. |
| Grant generation | Per-plugin app setup counter | Invalidates captured work when access changes. It must not be reset to revive an old invocation. |
| App fingerprint | Host-derived app setup fingerprint | Drives reload for the affected app; unrelated catalogue/store changes do not require a reload. It is not a public compatibility number. |
| Project plugin references | `PluginRef` and `FeatureRef` | `ext-core::project`; package/source/version plus contribution/state metadata as supplied by the adapter. |
| Contribution state version | Nonnegative `state_version` | Identifies contribution-owned saved data. Preserve unsupported/newer data as placeholders; it is not an automatic migration implementation. |
| Future canonical theme payload | `craft-design-theme`, `schema_version: 1` at the checked framework revision | Parsed by craft-libs, not a new plugin parser. `craft-theme` remains the capability name; activation is deferred. |

The complete manifest procedure is in the [author guide](author-guide.md). Limits,
node fields and value encodings are in the [UI reference](ui.md). Per-operation
arguments, permissions, units and color semantics belong to the app contracts:
[EffectCraft](contracts/effectcraft.md), [PhotoCraft](contracts/photocraft.md) and
[FigCraft](contracts/figcraft.md).

## A multi-app package

Each contribution declares the contract it needs. `apps` declares intended app
support; it does not manufacture a missing implementation. Cross Review uses
separate app session contributions and modules in one package. Cross Vignette and
Halftone use `raster.filter@1.0` across EffectCraft and PhotoCraft. FigCraft does not
provide that contract.

A partly compatible package can be installed while listing its unavailable
contributions and their reasons. Enabling must still check the selected
contribution's contract, supported kind, platform, required grants and native
consent. A compatible app marketing version cannot override an unmet contract.

## Extending UI without breaking plugins

Use stable widget IDs and typed values. New vocabulary belongs to an appropriate
session/UI minor level. Validate the entire tree at the contribution's declared
level. An unknown node can use a fallback; a known node with invalid fields is
refused. A named component carries its own version and validated fallback.

The host chooses the renderer and applies its own theme, fonts and accessibility.
A framework change does not require guests to link the framework. See
[framework integration](framework-integration.md) for the pinned upstream reference,
single-egui-revision requirement and host-only installation boundary.

## Data preservation across updates and rollback

Installing a newer release does not delete the previous release or automatically
grant new access. A candidate requiring consent stays pending. Activating another
native version asks for native consent again. Pins constrain candidate selection;
they do not authorize withdrawn or incompatible code.

When project data names a contribution state version the loaded implementation
cannot handle, retain that data and explain the missing capability. A rollback is
not permission to discard newer saved data. EffectCraft's opaque per-instance state
migration remains deferred. PhotoCraft's current destructive plugin filters store
resulting pixels rather than a persistent plugin filter instance.

See [update flow](maps.md#8-updates-pins-and-rollback) and
[project flow](maps.md#9-opening-a-project-with-missing-plugins).

## Source ownership

- Manifest/schema: `crates/ext-core/src/manifest.rs`.
- Contract levels: `crates/ext-core/src/contract.rs`.
- Wasm ABI and budgets: `crates/ext-host/src/wasm.rs`.
- Native records: `crates/ext-host/src/native.rs`.
- Forms and patches: `crates/ext-core/src/form.rs`.
- UI tree and revisions: `crates/ext-core/src/ui/tree.rs`.
- Catalogue format: `crates/ext-core/src/catalogue.rs`.
- Store state/fingerprints: `crates/ext-store/src/state.rs`.
- Project references: `crates/ext-core/src/project.rs`.

These owners, not duplicated guide tables, determine the format. Update the detailed
reference and its [coverage entry](coverage.md) when a source contract changes.

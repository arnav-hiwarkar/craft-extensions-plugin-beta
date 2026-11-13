# Framework integration without plugin changes

The host owns the renderer, theme, fonts, accessibility and document operations.
Plugin panels and canvases remain versioned data. Wasm modules depend on the guest
SDK, not egui or the framework. Installing another host renderer does not change the
plugin ABI, tree JSON, widget IDs, revision checks or permission rules.

The reference checked on 2026-11-13 is
[aiongg/craft-libs craft-framework](https://github.com/aiongg/craft-libs/tree/8ab6ea366ac67629e2699820f46b0ffd0fc2e6bd).
Its [framework contract](https://github.com/aiongg/craft-libs/blob/8ab6ea366ac67629e2699820f46b0ffd0fc2e6bd/docs/craft-framework.md)
uses the host convention of services in and edits out. Its runtime owns focus,
keyboard routing, accessibility and repeated measuring/painting passes. The app
must install that runtime once for its window, not once for each plugin panel.

## Install the host renderer

Implement `craft_ext_ui::plugin_ui::Engine` in the host-side framework adapter.
Translate the validated `Surface.tree` into framework widgets and return the same
`UiEvent` values, tagged with the original surface key and revision. Install it
with `PluginUi::set_framework_engine(Some(Box::new(engine)))` when the host selects
`EngineChoice::Framework`. Without an installed implementation the same choice
continues to use egui. Removing the implementation restores that fallback.

The renderer does not edit a document or run a plugin. Return control events for the
owner to dispatch after rendering; do not dispatch from measuring passes or replay
records. Preserve stable widget IDs, local edit state and the shared commit-on-edit
semantics. `forget` releases a closed surface's caches in both renderers. Replacing
an installed implementation releases its known surface state before dropping it.

The current `Engine` boundary accepts egui's `Ui` because the future framework also
uses egui. The host workspace must use one egui source/revision for the app, framework
and plugin renderer. At the checked framework revision the root patches the egui
family to `aiongg/egui` revision `42e34e2acf5232928bc5f3fe092733d41b09cfe3`.
Dependency patches are workspace-root decisions and are not inherited from a library.
Do not copy that pin into guest packages or switch app pins during this source-only
phase. Reconcile the app/framework lockfiles when the framework is actually adopted.

## Preserve components and older UI

`Host::component` resolves a named component version when the host supports it.
Otherwise render its validated fallback. Cross Review demonstrates a named action
row with ordinary buttons as its fallback. It stays usable before that component
exists. Keep the same IDs and event values when implementing a native component.
Do not expose renderer choice, framework version probes or backend-specific nodes
to plugins. UI 1.0 trees remain supported by 1.1. New vocabulary must carry a session
minor level and preserve fallbacks; it must not require an egui-specific guest SDK.

## Use the canonical theme engine

The plugin capability name remains `craft-theme`. It names the suite's theme
provider, not a second theme file schema. The checked framework's
[theme reference](https://github.com/aiongg/craft-libs/blob/8ab6ea366ac67629e2699820f46b0ffd0fc2e6bd/docs/suite-themes.md)
uses `craft-design-theme`, `schema_version: 1`, parsed by `craft_theme::format`.
The earlier `craft-theme` file schema is skipped by that library. Do not present
legacy payloads as valid new themes or translate them with another plugin parser.

Theme/font activation remains a beta deferral. When it is added, validate payloads
with the canonical framework library, show errors for unsupported schemas and use
`craft-theme-store` for library selection and revision watching. Widgets consume
semantic tokens and the host's appearance. A plugin cannot restyle built-in UI or
bypass the user's suite/app theme choice. Keep plugin package installation and
user theme activation as separate choices.

## Verification still required

Regression source covers host renderer installation, existing tree/revision events,
forgetting a surface and fallback after removal. It has not run. Before adopting
the framework, compare each shared control's event values, stale revisions, local
edits, component fallbacks, canvas input/clip transforms, focus, shortcuts, scaling,
contrast, reduced motion and screen-reader names. Test repeated passes without
duplicate edits. Current source checks do not establish runtime framework compatibility.

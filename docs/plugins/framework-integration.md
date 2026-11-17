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

`craft_ext_ui_fw::FrameworkEngine` implements `craft_ext_ui::plugin_ui::Engine` for apps on
the craft UI framework. It lives in `crates/ext-ui-fw`, its own Cargo workspace pinned to
PhotoCraft's craft-libs revision, so EffectCraft, FigCraft and the launcher never link the
framework. Install it with `PluginUi::set_framework_engine(Some(Box::new(FrameworkEngine::default())))`;
`EngineChoice::Framework` then draws with it, and without it the same choice draws with egui.
PhotoCraft installs it and defaults `plugIns.pluginUiEngine` to `framework`.

The engine translates the validated `Surface.tree` into framework components
([node to component](ui.md#relation-to-craft-framework)) and returns the same `UiEvent`
values, tagged with the surface key and revision. Plugin text is passed untranslated as
shared labels; host words go through `Host::tr`. Each control's id salt is (surface key,
node id). Controls are given the plugin's value; an event leaves only from a click, an edit
commit or a selection change, and only when the value differs from the one shown. A
committed value is shown until the plugin's value changes or the revision moves on; a tab,
section or selection is kept until the plugin's own value for it changes. `forget` releases
both engines' state for a closed surface.

Canvases and colour inputs are drawn by `EguiEngine::draw_part` inside the framework layout,
with the surface's named images and widget state. The `Engine` boundary still accepts egui's
`Ui`, and the host workspace uses one egui source for the app, framework and renderers.

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

Unit tests cover the node-to-component table, tones, variants, type roles and the event value
each control sends. A headless render with the framework test kit was not run: it needs
`craft/ui-testing` as a dev-dependency, and a non-member path package with dev-dependencies
cannot be tested from PhotoCraft's workspace. Still unverified in the app: focus, keyboard,
screen-reader names, repeated passes, canvas input inside framework containers and the look
in each theme.

# 04: The complete UI extensibility model for Craft plugins

Date: 2026-11-18. Researcher 4 of 8. Read-only study: no repo was edited, built or run, and nothing
under the `storytold` org was fetched (local `upstream*` refs were read as they stand).

**How this was checked.**
- Repository facts come from reading source at these refs:
  - craft-extensions `24cd2ff`;
  - craft-libs `main` `baa619f01` and `upstream/next` `427c816db`. `next` is 2 commits past `a6b9d9c`, the framework rev PhotoCraft pins;
  - the eleven app checkouts at their local `upstream/main` or `upstream-ro/main` (dated 2026-11-17 or 18).
- Host-survey facts come from the public documentation linked in §2. I re-fetched only VS Code's UX overview and searched Zed's extension docs in this pass. The other rows rely on prior knowledge of the documented APIs, and should be re-checked before they are quoted as specs.
- Numbers marked *target* are design budgets, not measurements.

---

## 0. Summary

**The model in one sentence.** Plugins declare *where* they contribute in the manifest and describe *what* to show as versioned data. The host owns every pixel of chrome, every input route and every trust surface, and renders plugin content with the app's own framework and theme. A plugin never runs on the UI thread and is never called per frame.

Decisions this report recommends:

1. **Five rendering tiers, all composited by the host. A webview is not one of them.**

   | Tier | What it is | Status |
   |---|---|---|
   | T1 | Retained declarative tree: UI 1.2 today, 1.3 adds keyed patches, bindings and live values | Default for panels, inspectors, dialogs, settings and status items |
   | T2 | Retained draw-list canvas with keyed layers, host-side hit regions, token paints and host animation | Custom visualisations and overlays |
   | T3 | Host editor components: curve, gradient, node graph, timeline lanes, meter, scope, viewer. The plugin supplies a model and receives typed edits | Custom editors |
   | T4 | GPU viewer: a host image handle produced by the WGSL, kernel or job tier, shown in a node. No raw device access | GPU output in a panel |
   | T5 | Foreign native windows: CLAP/VST3 editors, native-full helpers | Labelled as external, never docked or trusted |

   An in-process HTML webview is rejected for the core platform (§3.6).
2. **Contribution points are a closed, versioned catalogue** (§4). The manifest's `kind = "panel" | "tool" | …` grows placement tables such as `[contributions.panel]` and `[contributions.inspector]`.
   - Every surface has a stable id `plugin.<pluginId>.<localId>`.
   - That id persists in the framework `Workspace` through `reconcile`'s hidden list, which already keeps unregistered panels' locations (`craft-libs next: crates/behavior/src/dock/workspace.rs:280-346`).
3. **Behaviour change goes through typed hooks with permissions** (§5). Hooks cover tool replacement and input filters, observing events, deferred automatic actions, provider hooks (defaults, presets, snapping, export steps), and save/export participation that fails open.
   - Never allowed: drawing over trust UI, global keyboard interception, hiding or disabling built-in UI, faking host chrome, or blocking save or quit.
4. **Performance rule: zero guest calls per frame** (§6).
   - The host keeps the parsed tree, applies patches, reads bound values from a host-side value store or lock-free rings, and coalesces input to at most one event per frame per widget.
   - Images travel by handle, not as RGBA bytes inside JSON.
   - Nothing on the UI thread waits for a plugin. That includes PhotoCraft's synchronous tool path (`photocraft/crates/engine/src/plugin_sessions.rs:330-383` waits up to 10 s).
5. **One SDK, one model, two renderers, then one** (§7).
   - Of the eleven apps on current upstream main, **only PhotoCraft is on the craft framework** (`craft` @ `a6b9d9c`, egui fork `e9711ca`).
   - Nine use legacy `craft-ui` from craft-libs `main` with egui fork `6d518fc`.
   - AniCraft uses stock egui 0.36 and no craft-ui.
   - `craft-ext-ui` (egui engine) serves the legacy apps until they port. `craft-ext-ui-fw` (framework engine) is the end state. The `EngineChoice` bridge is deleted when the last app ports.
6. **Trusted chrome makes spoofing structural, not heuristic** (§8). Every plugin surface sits inside host-drawn chrome that names the plugin. Consent and permission UI lives on a host-only layer that dims and freezes input to all plugin surfaces. Plugins cannot draw outside their rects, take focus without a user gesture, or create always-on-top or transparent windows.

---

## 1. Where the platform stands today

| Area | Current state | Evidence |
|---|---|---|
| UI vocabulary | UI level 1.2: 30-odd node kinds, forms, canvas draw ops, tones and variants, `component` with required fallback. Limits: 2000 nodes, depth 16 | `craft-extensions/crates/ext-core/src/ui/tree.rs:29`, `docs/plugins/ui.md` |
| Ops | `ui.panel.set`, `ui.window.open` (`ui.window` permission), `ui.notify`, `canvas.overlay.set`, `tool.state`. Whole-tree replace on every call; no patch except FigCraft form patches | `docs/plugins/contracts/*.md`, `author-guide.md §13` |
| Contribution kinds | `effect, command, panel, tool, event-hook, preset, lut, font, theme, data` | `crates/ext-core/src/manifest.rs:31-46` |
| Permissions | `document.read/edit:{selection,document,all}`, `files.read/write:picked`, `network:<host>`, `clipboard.*`, `ui.window`, `app.events` | `crates/ext-core/src/permission.rs:75-91` |
| Renderers | `craft_ext_ui::plugin_ui` with an `Engine` trait. `EguiEngine` is the default. `craft_ext_ui_fw::FrameworkEngine` maps nodes to framework components. Canvas and colour nodes fall back to egui inside the framework layout | `crates/ext-ui/src/plugin_ui/mod.rs`, `crates/ext-ui-fw/src/engine.rs`, `ui.md` "Relation to craft-framework" |
| Hosting | EffectCraft: floating `egui::Window` (`effectcraft/crates/ui-egui/src/plugins_ui.rs:126`). PhotoCraft: docked through `DockPanels<String>`, windows as `ViewportWindow`, tools as a `FamilyTool`, overlays painted in view space (`photocraft/crates/ui-egui/src/plugins_ui.rs:205-430`, `panels/registry.rs:260-307`). FigCraft: its own side dock (`figcraft/crates/ui-egui/src/plugin_dock.rs`) | source |
| Per-frame cost | Trees are parsed once per revision (`mod.rs:149-158`). Concave paths are re-triangulated every frame (`ui.md` "Known limit"). Images travel as JSON byte arrays (up to 64 MiB per tree) | `ui.md`, `tree.rs` |
| Missing pieces already identified | U1–U8 in `research/capability-program-2026-11-18.md §4.4` (live controls, overlay images and hit regions, effect handles, parameter kinds, curve, gradient, meter and viewer widgets, pen fidelity, one manager, palette and shortcuts). H4 events, H7 drag and drop, H8 timers, H11 localisation, H12 `host.capabilities` | capability program |
| Framework gaps already raised | Runtime icon registration, a sanctioned `ContentPainter` for plugin canvases, a "registry changed" reconcile helper | `research/framework-guide-2026-11-17.md §4.4` |

Other plugin UIs that the suite replacement rule says must be inspected and then dispositioned:

| App | Existing plugin UI | Detail and evidence |
|---|---|---|
| VectorCraft | Its own Wasm plugin dialogs | Reserves `ui.panel` and `ui.tool` as future permissions (`vectorcraft/crates/plugins/src/permissions.rs:39`, `dialog.rs`) |
| AniCraft | Its own plugin crate | `anicraft/crates/plugins/src/{manifest,registry,runtime}.rs`, with parameter dialogs |
| SoundCraft | Hosts CLAP and VST3 editors in-process | Floating or embedded through `set_parent` (`soundcraft/crates/clap-host/src/ffi/gui.rs:103-194`, `crates/vst3-host/src/ffi/editor.rs`) |
| PhotoCraft upstream | WASM filter dialogs through `HostedParamDialog` | `framework-guide §3.1` |
| Public EffectCraft | Docked ScriptUI | `continuation-2026-11-17.md` |

---

## 2. Survey: how the best hosts let plugins extend UI

### 2.1 Table

| Host | Mechanism and rendering | Contribution points | Isolation | Cost and risk | Lesson for Craft |
|---|---|---|---|---|---|
| **VS Code** ([contribution points](https://code.visualstudio.com/api/references/contribution-points), [UX overview](https://code.visualstudio.com/api/ux-guidelines/overview), [webviews](https://code.visualstudio.com/api/extension-guides/webview), [tree views](https://code.visualstudio.com/api/extension-guides/tree-view)) | Static JSON `contributes` plus a runtime API. Native items (tree views, quick picks, status items, notifications) are host-rendered. Webviews are sandboxed iframes with CSP and `postMessage` | View containers, views, commands, menus (100+ menu ids), keybindings, status bar, custom editors, walkthroughs, settings, themes, icon themes, `when` clauses | Extension host process; webviews in separate iframes. Extension JS is not sandboxed from the OS | Webviews are heavy (a process per webview), off-theme unless they use CSS variables, and have a11y gaps. Microsoft tells authors to use webviews "only when necessary" | **Manifest-first, lazy activation, `when` clauses, a big native vocabulary and a webview of last resort.** Tree views (data in, host draws) are the model for T1 |
| **JetBrains** ([extension points](https://plugins.jetbrains.com/docs/intellij/plugin-extension-points.html), [tool windows](https://plugins.jetbrains.com/docs/intellij/tool-windows.html), [UI components](https://plugins.jetbrains.com/docs/intellij/user-interface-components.html)) | In-process JVM; Swing and the Kotlin UI DSL; JCEF for web | Thousands of extension points: actions, tool windows, settings (`Configurable`), inspections, gutter icons, editor painters, status bar widgets, intentions | None (same JVM) | Full power at full risk: a plugin can freeze the IDE. There is a "Plugin X is slowing things down" freeze detector | **Typed extension points scale; per-plugin attribution of slowness is essential.** A freeze reporter maps to our per-plugin budget accounting |
| **Figma** ([how plugins run](https://www.figma.com/plugin-docs/how-plugins-run/), [creating UI](https://www.figma.com/plugin-docs/creating-ui/), [widgets](https://www.figma.com/widget-docs/)) | Main thread runs in a QuickJS/Realms sandbox with document access. UI is a null-origin iframe that talks through `postMessage`. Widgets are declarative JSX rendered by Figma *on the canvas* | One modal-ish plugin window, menu commands, relaunch buttons, codegen panels in Dev Mode, on-canvas widgets with synced state | Sandbox plus iframe; no DOM in the main thread | The iframe UI never matches the theme unless authors opt into theme CSS variables. Only one plugin UI is open at a time. Widgets are fast because they are declarative | **Split logic (sandbox) from UI.** Widgets show that declarative on-canvas UI scales to many instances. Map to `widget@1` (FigCraft P3) through T1/T2 |
| **Adobe UXP: Photoshop, Premiere, InDesign** ([UXP](https://developer.adobe.com/photoshop/uxp/2022/), [Spectrum UXP](https://developer.adobe.com/photoshop/uxp/2022/uxp-api/reference-spectrum/), [manifest v5](https://developer.adobe.com/photoshop/uxp/2022/guides/uxp_guide/uxp-misc/manifest-v5/)) | A custom JS engine with a *subset* HTML/CSS layout engine. Spectrum Web Components are rendered natively. Not a browser | Manifest `entrypoints`: panels (dockable, sized, with icons per theme), commands; flyout menus; modal dialogs via `<dialog>`; `executeAsModal` for edits; hybrid C++ addons | Per-plugin sandbox with manifest `requiredPermissions` (network domains, clipboard, local file system, launch process) | Not a full browser, so many web libraries break. Themed through Spectrum tokens. Modal state blocks the host while running | **A host-native component set with theme tokens is what makes third-party panels look first-party.** Manifest-declared panels with icons and sizes. Modal edits are bounded, with progress and cancel |
| **After Effects ScriptUI and CEP** ([ScriptUI](https://extendscript.docsforadobe.dev/user-interface-tools/), [CEP](https://github.com/Adobe-CEP/CEP-Resources)) | ScriptUI: declarative-ish resource strings, native widgets, dockable when launched from ScriptUI Panels. CEP: Chromium (CEF) panels calling ExtendScript | Docked panels, dialogs, menus via the Window menu | None for ScriptUI. CEP has a Node.js option, so effectively none | CEP is about 100 MB of Chromium, slow to start and off-theme. Adobe is replacing it with UXP | **A Chromium per panel was a dead end Adobe is leaving.** ScriptUI-class declarative panels are what EffectCraft's public users have today, so T1 must cover them |
| **Lightroom Classic** ([SDK](https://developer.adobe.com/lightroom-classic/)) | Lua with `LrView` declarative view factories; `LrDialogs.presentModalDialog`; Plugin Manager sections | Export and publish service dialog sections, metadata fields and tagsets, menu items, Plugin Manager sections | Lua sandbox, partly restricted | Modal-only UI; no docked panels at all | **Contributed sections inside host dialogs (export, metadata) are a first-class point.** Directly maps to inspector sections and export-dialog sections |
| **Blender** ([Panel](https://docs.blender.org/api/current/bpy.types.Panel.html), [Operator](https://docs.blender.org/api/current/bpy.types.Operator.html), [Gizmo](https://docs.blender.org/api/current/bpy.types.Gizmo.html), [WorkSpaceTool](https://docs.blender.org/api/current/bpy.types.WorkSpaceTool.html), [NodeTree](https://docs.blender.org/api/current/bpy.types.NodeTree.html), [draw handlers](https://docs.blender.org/api/current/bpy.types.Space.html)) | Python immediate-mode `draw(self, context)` builds a host `UILayout` each redraw. Gizmos, GPU draw handlers and custom node trees | Panels per space, region and context; menus; operators with `poll`; tools in the toolbar with keymaps; gizmo groups; custom node trees and sockets; property groups; preferences; header and status items | None (in-process Python) | `draw()` runs Python every redraw, a known cost. `poll` everywhere is cheap but chatty. GPU draw handlers can draw anywhere in the region | **The richest model for creative apps:** context-scoped panels, operator plus poll, tools with gizmos, custom node trees. Copy the *placement* model. Do not copy the per-redraw callbacks (§6) |
| **Unreal Editor** ([Editor Utility Widgets](https://dev.epicgames.com/documentation/en-us/unreal-engine/editor-utility-widgets-in-unreal-engine), ToolMenus, Slate) | C++ Slate (retained) or UMG Blueprints; `UToolMenus` extension by menu name and section | Tabs (nomad tabs), toolbars, menus, detail customizations, asset editors, viewport modes and gizmos | None | Huge power, compile-time coupling | **Detail customization** (replace or extend a type's property rows) is the inspector model. Named menu sections give stable insertion points |
| **Unity Editor** ([EditorWindow](https://docs.unity3d.com/ScriptReference/EditorWindow.html), [Overlays](https://docs.unity3d.com/Manual/overlays-custom.html), PropertyDrawer, Handles) | IMGUI (immediate) and UI Toolkit (retained, USS-styled) | Windows, inspectors via `CustomEditor` and `PropertyDrawer`, Scene view overlays (toolbars and panels floating over the viewport), Handles gizmos, menus, settings providers | None | IMGUI's per-frame C# is costly; Unity moved to retained UI Toolkit for performance and styling | **The move from immediate to retained mode** is the same choice we make. Scene **Overlays** are a good model for viewport-floating plugin toolbars |
| **Zed** ([extensions](https://zed.dev/docs/extensions)) | Rust compiled to Wasm (component model) | Languages, LSP, themes, icon themes, debuggers, MCP context servers. Extension slash commands are being removed in favour of MCP. **No custom UI** | Wasmtime sandbox | Safe and fast, but no UI extensibility at all | **Wasm plus host-owned UI is right.** "No UI" is too little for creative apps. We already go further with T1 |
| **Obsidian** ([views](https://docs.obsidian.md/Plugins/User+interface/Views)) | Plugins get the raw Electron DOM: `ItemView`, ribbon, status bar, settings tab, modals, editor extensions | Views (dockable leaves), ribbon icons, commands, status items, settings tabs, context menus, editor decorations | **None**: full Node access on desktop | Ultimate flexibility, regular breakage, no safety | A cautionary tale. Its *point list* (leaves, ribbon, status, settings tab, file menu) is a good checklist |
| **Krita** ([Python scripting](https://docs.krita.org/en/user_manual/python_scripting.html)) | PyQt `DockWidgetFactory`, `Extension` actions | Dockers, actions and menus, a few canvas hooks | None | PyQt dependency issues; unthemed widgets | Dockers are table stakes |
| **GIMP 3** ([writing a plug-in](https://developer.gimp.org/resource/writing-a-plug-in/)) | Out-of-process plug-ins over the wire protocol (PDB); GTK dialogs drawn *by the plug-in process*; auto-generated dialogs from procedure arguments | Procedures in menus, generated dialogs, GEGL ops | Process separation, no sandbox | A plug-in crash does not kill GIMP; dialogs come from foreign processes | **Auto-generated dialogs from typed arguments** (our Form schema) cover most filters. Process isolation is worth the IPC cost |
| **OBS** ([frontend API](https://docs.obsproject.com/reference-frontend-api)) | Qt docks (`obs_frontend_add_dock_by_id`), browser docks (CEF) for web pages | Docks, tools menu, source property pages generated from `obs_properties` | None, except CEF | Browser docks are popular for stream widgets but heavy | **Generated property pages plus optional docks.** Browser docks are mostly used for *remote web services*, which Craft can serve with the system browser or OAuth instead |
| **DaVinci Resolve Workflow Integrations** (README shipped in Resolve's `Developer/Workflow Integrations` folder) | Each integration is a separate Electron app launched from the Workspace menu, calling the scripting API | One menu entry opening its own window | Separate process, no sandbox | A whole Electron per integration; no docking; looks foreign | Shows the "foreign window" tier (T5) and why it should stay exceptional |
| **Ableton Max for Live** ([docs](https://docs.cycling74.com/)) | Devices in the device chain with a *presentation mode* UI built from `live.*` objects bound to Live parameters | Audio, MIDI and instrument devices; parameters automatable and mappable | Max runtime in-process | Unified look because `live.*` objects are host-styled. Parameters are first-class (automation, macros) | **Bind UI controls to host parameters, not to plugin callbacks**, so automation, undo and mapping work for free. This is T1 bindings |
| **Bitwig and CLAP** ([CLAP gui.h](https://github.com/free-audio/clap/blob/main/include/clap/ext/gui.h), [controller extensions](https://github.com/bitwig/bitwig-extensions)) | CLAP plugins draw their own native window, floating or embedded. Bitwig controller scripts have no UI beyond preferences and notifications. Bitwig Grid is host-drawn | Device editor window; host-generated generic UI from parameters | Bitwig offers per-plugin process sandboxing modes | Foreign GUIs can steal focus and keys; host must route keyboard | **Always offer a host-drawn generic editor** from parameters (Form). Treat plugin-drawn native GUIs as T5 |
| **Chrome extensions** ([side panel](https://developer.chrome.com/docs/extensions/reference/api/sidePanel), [action popup](https://developer.chrome.com/docs/extensions/reference/api/action)) | HTML pages in extension origins; content scripts in isolated worlds; strict CSP in MV3 | Toolbar action with popup, side panel, context menus, omnibox, options page, new-tab override | Process plus origin isolation, permissions with host patterns | Browser chrome (URL bar, permission prompts) is drawn *outside* the content area, so it cannot be spoofed | **Trusted chrome outside plugin pixels** is the anti-spoofing model. *User activation* gates popups and focus. Our §8 adopts both |
| **Photoshop native C++ and OFX/AE effects** | Effect parameters drawn by the host. Custom UI is drawn through host-supplied drawing suites (AE `PF_Event` custom UI, OFX overlay interacts) | Effect controls rows, comp-window overlays and handles | None | Host-drawn params give automation and keyframes for free; custom UI is rare and in-process | **Effect params as data; on-canvas handles as an overlay contract** (U3) |

### 2.2 What the survey settles

- **Every host that is both safe and consistent uses host-rendered, data-described UI.** VS Code tree views, Spectrum UXP, Figma widgets, `live.*` objects, LrView and Zed (by omission) all work this way. Hosts that hand plugins a real toolkit (JetBrains, Blender, Obsidian, Unity IMGUI) get maximum power at the cost of freezes, breakage and an off-theme look.
- **Webviews are always the escape hatch, and every vendor tries to shrink them.** VS Code says use them sparingly. Adobe is retiring CEP. Figma theme-matches only by opt-in. OBS and Resolve use them for remote services or foreign apps.
- **Placement is richer than "panel".** Creative hosts expose context-scoped sections (Blender, Lightroom, Unreal details), tools with gizmos (Blender, Unity, OFX), node types (Blender), device or effect UIs bound to parameters (M4L, OFX), and viewport overlays (Unity, Blender).
- **Parameters bound to the host are what make plugins feel native.** That binding gives undo, automation, keyframes, presets and macros. Plugin-handled callbacks give none of these.
- **Trusted chrome must sit outside plugin pixels**, as in Chrome. A heuristic text filter cannot stop spoofing; only structure can.

---

## 3. Rendering models and decision

### 3.1 Options compared

| Model | Who draws | Guest calls per frame | Theming | A11y | Web build | Risk | Decision |
|---|---|---|---|---|---|---|---|
| **T1 declarative tree** (current, extended) | Host framework components | 0 | Tokens via tone, variant and role | AccessKit through framework components; labels required | Works (egui web) | Low | **Default for all UI** |
| **T2 retained draw-list canvas** | Host painter, from plugin ops, layers and hit regions | 0 (host evaluates hover and hit, animates bindings) | Token paints plus data colours | Hit regions carry names and roles; a canvas summary | Works | Low–medium (op budgets) | **For visualisations, overlays and small custom editors** |
| **Immediate-mode callback** (Blender `draw`, Unity IMGUI) | Host, but driven by a guest call every frame | ≥ 1 per frame per surface | Possible | Hard | Works | Frame stalls, fuel per frame | **Rejected**; T1 patches and T2 bindings replace it |
| **T3 host editor components** | Host editor (node graph, curve, timeline, meter) over a plugin-supplied model | 0 | Native | Native | Works | Medium (component API versioning) | **For custom editors** |
| **T4 GPU viewer** | Plugin WGSL or kernel output into a host texture; host composites | 0 in the UI. Cooks are scheduled jobs on the GPU tier | n/a (content) | `alt` plus a viewer role | WebGPU | Medium (GPU budget, device loss) | **For live previews, scopes and 3D previews** |
| **Raw GPU surface** (plugin owns a wgpu pass in the UI frame) | Plugin | Every frame | None | None | Partly | High: can hang the frame, read other textures | **Rejected** |
| **HTML webview** (wry: WKWebView, WebView2, WebKitGTK) | Browser engine in a child OS window | n/a | Only via injected CSS variables | Separate a11y tree | n/a (iframe) | High (§3.6) | **Rejected for core; open decision O1** |
| **T5 foreign native window** | Plugin process or library | n/a | None | Plugin's own | No | High (focus and key theft) | **Native tier only, labelled external** |

### 3.2 T1, the declarative tree: what changes at UI 1.3 and 1.4

Keep UI 1.0–1.2 unchanged, per the replacement rule: "Preserve UI 1.0, stable IDs, revision checks and validated component fallbacks". Add the following.

**UI 1.3 (performance and liveness):**

- **Keyed patches.** Any node may carry `key` (1–64 bytes, unique per parent). The new op `ui.patch` takes a surface, a base revision and a list of edits:

  ```json
  {"surface":"histogram","base_rev":41,"ops":[
    {"set":"exposure","field":"value","to":0.35},
    {"replace":"preview-row","node":{...}},
    {"insert":"layers","at":3,"node":{...}},
    {"remove":"layer-7"},
    {"move":"layer-2","to":0}
  ]}
  ```

  - The host refuses the patch if `base_rev` is not current (`-7 stale`), so the plugin resends a full tree.
  - Each patch makes a new revision. Stale-event handling is unchanged.
  - Validation is incremental (the changed subtree plus global counters), with the same limits.
- **Bindings.** `value`, `text`, `progress.value` and `badge.text` may be given as `{"bind":"<name>"}`. The host reads a named slot in the surface's **value store**, which the plugin updates with `ui.values.set {surface, values:{name:value}}`. Updates coalesce to one apply per frame and cost no tree revision. Bindings may also name a host source:
  - `{"bind":"host:document.zoom"}` (read-only, permission-checked);
  - `{"bind":"param:<effect-instance>/<key>"}`, which gives undo and automation for free, as `live.*` does;
  - `{"bind":"ring:<id>"}` for meters (§6).
- **Live controls (U1).** `slider`, `number`, `color`, `angle` and `point` accept `live: true`.
  - The host sends `{"event":"ui","phase":"begin|change|end",…}`, with `change` coalesced to at most one per frame and dropped while the previous one is in flight.
  - Without `live`, today's send-on-end stays the default.
- **Lazy collections.** `list` and `table` may set `rows_source: "plugin"` with a `count`. The host asks for windows (`{"event":"ui.rows","from":200,"to":260}`), and the plugin answers with `ui.rows.set`. This brings 1M-row browsers within reach without 10 000-row trees. The framework's collections already virtualise (`framework-guide §1.10`).
- **Context menus on nodes.** `menu: [{id,label,icon?,danger?,children?}]` on list rows, table rows, canvas hit regions and image nodes. The host draws it and sends `{"menu": id, "target": key}`.
- **Drag and drop (H7).** `drag: {kind, payload}` and `drop: {accept:[kinds]}` on rows, images and canvas regions. Kinds are host-typed (`craft/layer-ref`, `craft/color`, `craft/asset-handle`, `text/plain`, `files`). OS file drops arrive as handles.
- **Text references for localisation (H11).** Anywhere text is accepted, `{"t":"key","args":{…}}` resolves against the package's `locales/<lang>.ftl` (Fluent) in the host before render. Missing keys fall back to `en`, then to the key.

**UI 1.4 (vocabulary):**

- `curve`, `gradient`, `color-wheel`, `meter`, `scope`, `viewer`, `thumbnail`, `angle`, `point`, `tree` (hierarchical list), `split` (resizable two-pane), `toolbar` (icon-button row with overflow), `chips`, `kbd`, `link` (opens through the host with a confirm for external URLs), `empty-state`.
- Framework equivalents already exist for most: `AngleDial`, `PointPicker`, `ChipField`, `Kbd`, `Link`, `List` tree mode, `TileGrid` (`framework-guide §1.10`). `curve`, `gradient`, `meter` and `scope` need framework components (raise them on craft-libs `next`, per its rule "A framework gap is fixed in craft-libs next first").

Levels stay additive. A 1.4 node in a 1.3 app draws its `fallback`. Unknown fields are still refused.

### 3.3 T2, retained canvas (the `canvas` node and `canvas.overlay.set`)

Upgrade the draw list from "a list of ops redrawn every frame" to **a retained scene of keyed layers**:

```json
{"type":"canvas","id":"wave","label":"Envelope","size":"fill","aspect":3.0,
 "a11y":"Envelope with 4 points",
 "layers":[
   {"key":"grid","ops":[...],"cache":true},
   {"key":"curve","ops":[...],"transform":{"bind":"view"}},
   {"key":"handles","regions":[
      {"key":"p1","shape":{"circle":[[40,80],6]},"role":"slider","name":"Point 1",
       "cursor":"grab","hover":{"fill":{"token":"accent.solid"}},"drag":true}]}
 ]}
```

- **Layers** tessellate once per layer revision; the host caches the mesh (`cache: true`). This fixes the per-frame triangulation in `ext-ui/src/plugin_ui/paint.rs`.
- **Regions** give hover styling, cursor and drag without guest calls. Only `down`, `drag` and `up` on a region reach the plugin, as region events (`{"region":"p1","kind":"drag","at":[…]}`). Regions are focusable for keyboard (arrow keys nudge) and carry an accessible name and role.
- **Token paints** (`{"token":"text.muted"}`, `{"token":"data.3"}`) resolve against the theme every frame, so theme switches need no plugin round-trip. Raw RGBA stays allowed as *data colour* (`DataColor` in the framework, `contract-api.md:3894`).
- **Host animation.** `transform`, `opacity` and per-layer `visible` may be bound (`{"bind":"…"}`) or animated with `{"animate":{"from":…,"to":…,"ms":…,"ease":"out"}}`. Reduced-motion settings collapse these to their end state.
- **Size.** `"fill"` with an optional `aspect` replaces fixed points, so canvases follow panel size. The plugin receives `{"event":"ui.resized"}` coalesced, at most every 100 ms.
- The framework engine needs the sanctioned `PluginCanvas: ContentPainter` component (gap 2 in `framework-guide §4.4`) so canvases stop falling back to egui.

**Canvas overlays on documents and viewports** (`canvas.overlay.set`, and `viewport.overlay@1` for 3D) use the same layer and region vocabulary in document or world coordinates:

- Strokes stay in screen points (as PhotoCraft does: `plugins_ui.rs:431-514`).
- Overlays draw **below** host HUDs (rulers, selection marching ants, transform handles, tool cursor) and are clipped to the canvas rect.
- Gizmos (U3) are regions whose drags map to **parameter edits in one undo step**. Host-defined gizmo kinds (`move-2d`, `rotate`, `scale`, `point`, `radius`, `angle`, `bezier-handle`, `box`) mean the plugin declares *what* is edited, and the host gives snapping, modifiers and undo.

### 3.4 T3, host editor components for custom editors

Node editors, curve editors and timelines are where "let the plugin draw it" fails. They need zoom, pan, selection, box-select, snapping, undo, keyboard and a11y, which the host already implements for its own editors. So:

| Editor | Plugin supplies | Host provides | Contract |
|---|---|---|---|
| Node graph (MaterialCraft, TextureCraft, EffectCraft node tier, compositing) | **Node types**: ports, params (Form), cook (`graph`, `wgsl` or `wasm`), optional thumbnail viewer; or a whole **graph kind** for a plugin-owned graph (plugin model, host editor) | Graph canvas, wiring, minimap, search-to-add, undo, copy/paste | `graph.node@1` (capability program §2.4); `ui.graph` component at 1.4 |
| Curve and gradient editors | Curve model (keys, tangents, ranges) or gradient stops | Editing UI, snapping, numeric entry | Nodes `curve` and `gradient` at 1.4 |
| Timeline lanes and tracks (EffectCraft, FilmCraft, AniCraft, SoundCraft) | **Track or lane kinds**: row header tree, items (start, duration, label, colour token), keyframes, optional per-item T2 thumbnail layer | Time ruler, zoom, scrub, selection, snapping, ripple, undo | `timeline.lane@1`; draws inside the host timeline, not in a panel |
| Inspector / property sections | A Form or T1 tree for a selected object type or a plugin data type | Placement in the app's inspector, collapse state, multi-selection mixed values | `inspector.section` placement (§4) |
| Meters and scopes | Ring id and shape (`peak`, `rms`, `histogram`, `waveform`, `vectorscope`) | 60 fps drawing from a lock-free ring | Nodes `meter` and `scope` with `bind: ring:*` |

A plugin that genuinely needs a bespoke editor (for example a spectral editor) builds it from T2 layers, regions and a T4 viewer. It still gets host zoom and pan by declaring `view: {zoom:true, pan:true}`. The host then owns `canvasView` commands (`framework-guide §1.7`) and sends view-space transforms as a binding.

### 3.5 T4, GPU viewer

`{"type":"viewer","id":"preview","source":{"image":"h:42"}|{"frame":{"effect":"…","time":…}},"fit":"contain","checker":true}`:

- `image` is a host image handle (R5). It is produced by a job, a kernel graph, the WGSL tier (`gpu.shader@1`), or a host render of the document at a time or region.
- The plugin never touches the GPU device. WGSL runs in the GPU tier under naga validation, bounded loops and a watchdog (`capability-program §1.6`, K7), scheduled by the host at most at display rate and paused when the surface is hidden.
- The viewer gets host zoom, pan, pixel probe and colour-managed display through the app's CMS path.
- This is how a plugin shows "its own GPU-rendered surface in a panel" safely: a preview of a generator, a 3D material ball, a scope.

### 3.6 Why no in-process HTML webview (in the core platform)

| Concern | Detail |
|---|---|
| Composition | wry (the Tauri webview) creates a **child OS view** over the egui/wgpu surface. It cannot be clipped by egui panels, z-ordered under popups or tooltips, faded, docked into framework tabs, or placed in an egui layer. It breaks with multiple viewports and on Wayland. Off-screen rendering to a texture is not stable across all three platforms |
| Size and dependencies | WebView2 is system-provided on Windows 11, WKWebView on macOS. Linux needs WebKitGTK (about 50–100 MB of system dependency, often missing on minimal distros). Bundling CEF is about 100+ MB per app, the cost Adobe is leaving with CEP |
| Security | A full JS engine with a network stack, a cookie jar and its own CVE stream. It needs CSP, a custom protocol, origin isolation and a message bridge that re-implements every permission check. It is a phishing surface (it can render pixel-perfect fake dialogs) |
| Consistency | Off-theme unless the author consumes injected CSS variables; separate a11y tree; separate focus model; separate IME; no framework keyboard rules (`framework-guide §1.6`) |
| Web builds | In the browser builds (`*-web` apps) a webview is an iframe, which works. That asymmetry is itself a maintenance cost |

What webviews are usually wanted for, and the Craft answer to each:

| Need | Craft answer |
|---|---|
| Sign-in or OAuth | `auth.oauth` through the system browser (H3) |
| Rich docs or help | `link` to the system browser; Markdown via a host `markdown` node (P2) |
| Remote dashboards | T1 tables and lists fed by `net.fetch` |
| Reusing existing web UIs | A conversion kit from UXP-style panels to T1 |

Revisit only if a measured catalogue need appears (Open decision O1).

### 3.7 Cross-cutting requirements

| Concern | Rule |
|---|---|
| **Accessibility** | Every interactive node has a label (already enforced). The framework renderer maps nodes to components with AccessKit roles and names (`framework-guide §5`). Canvas regions are focusable with name and role. `canvas.a11y` is a summary. `viewer.alt`. Live regions: `notice` and `ui.notify` announce through the framework announcer. The `Host::widget` rect callback is replaced by the access tree after one release. Conformance: every plugin surface must pass the framework's access-tree check in `craft-ext test` |
| **Theming** | Plugins express meaning only: `tone`, `variant`, type `style`, `token` paints. No pixel sizes except canvas geometry. The framework engine ignores plugin pixel sizes (`ui.md` "Relation to craft-framework"). Theme and density changes cost no plugin round-trip |
| **Localisation** | Host words via `Host::tr` / `Env::words`. Plugin words via `{"t":…}` and Fluent files in the package. Numbers, dates and units via `Env::format` in the host when nodes carry `unit` / `format`. RTL mirroring by the framework, with sliders and number fields never mirrored |
| **High-DPI** | Points everywhere; the host rasterises icons. Plugin raster images may provide `@1x/@2x` variants or vector icons (sanitised SVG subset: no scripts, no external references, size-capped). Plugin icons need the framework's runtime icon registration (gap 1) |
| **Input** | Edge events only (click, commit, selection change), which matches the framework pass model (`framework-guide §1.6`). Canvas keys only while that canvas has focus. **Escape is always the host's**: it cancels a gesture and is never delivered. Tab and F6 navigation is host-owned |
| **Shortcuts** | Only through contributed **commands** in the keymap catalogue (`AppCatalog`, `framework-guide §1.5`). Default chords apply only if unbound. Conflicts leave the plugin command unbound and show a notice. No raw key hooks |
| **Focus** | A plugin may request focus for its surface only within 1 s of a user gesture on that plugin's UI or command (browser-style *transient activation*) |
| **Clipboard** | Writes only during a user-initiated event (H6); reads need a grant |
| **Drag and drop** | Typed payloads (§3.2); cross-plugin drops carry only the declared kinds; file drops as handles, never paths |

---

## 4. Contribution point catalogue

### 4.1 Manifest shape

Keep `[[contributions]]` with `kind` and `contract`. Add one optional placement table per kind. Unknown fields stay refused (`manifest.rs` invariant). Example:

```toml
[[contributions]]
id = "histogram"
kind = "panel"
contract = "photocraft.session@1.3"
label = "Histogram+"
module = "plugin.wasm"
[contributions.panel]
icon = "icons/histogram.svg"          # sanitised SVG in the package, or a catalogue name
area = "right"                        # hint: left | right | bottom | float | document
min_size = [200, 120]
instances = "single"                  # single | per-document
when = "document.open"                # host-evaluated context expression
open_on_install = false

[[contributions]]
id = "levels-inspector"
kind = "inspector"                    # new kind
contract = "photocraft.session@1.3"
label = "Smart Levels"
[contributions.inspector]
target = "layer.pixel"                # app-defined target types, listed in the app contract
order = "after:adjustments"

[[contributions]]
id = "export-webp-plus"
kind = "command"
contract = "photocraft.session@1.3"
label = "Export WebP+"
[contributions.command]
menus = [{ path = "File/Export", group = "plugins" }, { path = "context:layer", when = "selection.count > 0" }]
palette = true
default_keys = ["mod+alt+shift+w"]    # applied only if unbound
icon = "export"
toolbar = { bar = "options", when = "tool == plugin.io.ex.crop" }
```

**The context expression language (`when`)** is a small, host-evaluated, closed grammar:
- identifiers from the app contract's context keys (`document.open`, `selection.count`, `tool`, `layer.kind`, `panel.focused`, `time.playing`);
- operators `== != < > && || !`.

It is evaluated every frame by the host, never by the plugin, as in VS Code `when` clauses and Blender `poll` (minus the per-frame Python).

### 4.2 Universal points: every app

| Point | Manifest | Runtime API | Constraints | Persists in layout | Missing / disabled / crashed |
|---|---|---|---|---|---|
| **Command** (menus, palette, toolbar button) | `kind="command"` with `[…command]` menus, palette, keys, icon, `when` | `Event::Command{id,params}`. Optional `ui.command.state {id, enabled, checked, title}` overrides the manifest `when` | Menu paths only into host-declared *extension groups* (named sections, as Unreal ToolMenus does). Never into File›Quit, Edit›Undo, Plugins›Manage or trust menus. Labels ≤ 64 chars; at most 32 menu rows per plugin | Keymap overrides stored by command id (`craft_keymap` overrides) | Missing: rows omitted; user chords kept dormant. Disabled: same. Crashed: rows greyed with a tooltip naming the plugin and a "Restart plugin" row |
| **Panel / dock** | `kind="panel"` with `[…panel]` | `ui.panel.set` (full), `ui.patch`, `ui.values.set`; events `panel.opened/closed/visible` (H4) | Max 16 panels per plugin, 64 plugin panels per app. Host-drawn tab label shows the plugin glyph | `Workspace<String>` id `plugin.<pid>.<panel>`; unknown ids kept in `hidden` with location (`workspace.rs:280-346`) | Missing: slot kept hidden; Window menu shows nothing. If the user had it open, a **placeholder** panel at its place: "Needs *Histogram+* (io.example.histo) — Install · Remove from layout". Disabled: placeholder with "Enable". Crashed: last tree frozen and dimmed, a `notice` bar "*Histogram+* stopped — Restart · Disable · Report", events refused |
| **Window** (floating, OS or in-app) | `ui.window` permission; optional `[…window]` default size and `remember=true` | `ui.window.open {id,title,tree, size?, modal?:false}`, `ui.window.close`, `window.closed` event | 4 windows per plugin, 32 surfaces per app, unified across apps (EffectCraft and FigCraft already use 4; PhotoCraft 32 per app). No positions, only size hints. Never always-on-top, transparent or borderless. Title is prefixed by host chrome | Framework `SaveKey::named("plugin.<pid>.<id>")` (PhotoCraft already does this: `plugins_ui.rs:237`) | Missing or disabled: not reopened; saved placement kept 90 days. Crashed: window stays with a frozen-tree notice; close works |
| **Modal dialog** | none (free); `[…dialog]` not needed | `ui.dialog.open {id,title,tree|form,buttons:[{id,label,role}]}` returns immediately; the result arrives as `{"event":"dialog","id","button","values"}` | One plugin modal at a time per app; host modals pre-empt it; Escape and Close are always cancel; 10-minute idle auto-cancel; never during a host modal | n/a | Crashed or uninstalled while open: host closes it with cancel and posts a toast |
| **Parameter form dialog** (effects, filters, commands) | `form` in the contribution | Existing Form schema 1 with `craft_update` patches | Host-drawn; Preview, Reset and remembered values from `HostedParamDialog` | Remembered values per contribution | Missing: project prompt (existing) |
| **Notifications / toasts** | free | `ui.notify {message, tone?, title?, action?:{id,label}}` | Toast shows the plugin name and icon; max 3 visible per plugin; rate limit 1 per second; actions send `{"event":"notify.action"}` | n/a | Queued toasts dropped on disable |
| **Status bar item** | `kind="status"` (new) with `[…status] side, priority, when` | T1 subtree limited to `label`, `icon`, `badge`, `progress`, `button`; bindings encouraged | Max 2 per plugin; width cap 200 pt; truncates; click opens a menu or panel | Visibility toggles in the status bar context menu (host prefs) | Hidden when missing or disabled; dimmed icon with tooltip when crashed |
| **Context menus** | command `menus` with `context:<target>` paths | as Command; params carry the target refs | Host-declared targets per app (layer, node, clip, track, keyframe, asset, canvas) | n/a | Omitted |
| **Preferences page** | `kind="settings"` (new) with `[…settings] label, icon, section="Plugins"` | Form or T1 tree; values stored through `settings.get/set` (H1), which maps to `SectionStore` key `plugin.<pid>` | Lives under Preferences › Plugins › *Plugin name*, never among built-in pages | Settings survive updates; removed on uninstall | Missing: page absent, data kept until uninstall. Crashed: page shows a notice |
| **Welcome / start page card** | `kind="start-card"` (P3) | T1 subtree ≤ 200 nodes | One card per plugin, user-dismissable, below built-in content | Dismissal remembered | Absent |
| **Import / export entries** | `format.import@1` / `format.export@1` contributions with extensions, label, `form` | Host file dialog lists them under "Plugin formats"; export dialog shows the plugin's form section | No raw paths; handles only | Last-used export options | Missing: entries gone; documents that need a plugin importer show the existing missing-plugin prompt |
| **Workspace / layout preset** | data contribution `workspace` (layout file using panel ids) | none | Applied only by the user from Window › Workspaces; may reference built-in and own panels; may not hide the Plugins menu or the manager | Stored as a user workspace once applied | Unknown panels reconciled into hidden |
| **Theme** | `kind="theme"`, `craft-theme@1.0` | none | Seeds and pins only; contrast pass enforced by `craft-theme`; cannot restyle trust chrome (trust tokens are pinned) | Suite selection (`selection.json`) | Missing: falls back to the app default theme |
| **Keymap preset** | data `keymap` | none | User applies explicitly; conflicts are shown first; cannot unbind Escape, Cmd+Q, Undo or Redo, or the manager | Keymap overrides | Missing: overrides stay |
| **Icons and cursors** | package SVGs | referenced by name `pkg:<name>` | Sanitised, ≤ 64 KiB each, ≤ 256 per package | n/a | Fallback `IconSource::Initial` |

### 4.3 Raster and photo apps (PhotoCraft, LightCraft, TextureCraft paint mode)

| Point | Manifest | Runtime | Constraints | Missing / crashed |
|---|---|---|---|---|
| **Tool** (toolbar slot, options bar, on-canvas gestures) | `kind="tool"` with `[…tool] icon, group="paint|select|retouch|measure", cursor, options_form, pen = true` | Tool events `begin/update/commit/cancel` with full pen data (U6: pressure, tilt, rotation, pointer type, coalesced samples); `canvas.overlay.set` / `canvas.preview.set` (U2) | Pipelined: the host shows the cursor and a provisional overlay without waiting. One gesture is one undo step (existing) | Tool slot shows a dimmed icon "Needs plugin" and falls back to the previous tool (PhotoCraft `follow_tool_choice`, `plugins_ui.rs:262`) |
| **Input filter** (stabiliser, ruler, perspective guide) | `input.filter@1` | Host calls the filter on batched samples; bounded at 1 ms per batch; output is samples | Applies to built-in brushes; user toggles it in the options bar | Bypassed (raw input) on failure |
| **Brush engine** | `brush.dab@1`, `resource.brush@1` | Dab program in the kernel or WGSL tier | No per-dab Wasm call on the interpreter tier | Falls back to the round brush with a notice |
| **Filters / adjustments** | `effect` (`raster.filter@1/2`) with `form` | Filter dialog, live preview via a T4 viewer or document preview | — | Missing-filter prompt (PhotoCraft gap noted in the continuation) |
| **Layer panel decorations** | `kind="decorator"`, `target="layer"` | `ui.decorate {target, badge:{icon,tone,hint}}` | 1 badge per plugin per row; host chooses the place | Badges vanish |
| **Inspector section** | `kind="inspector"`, targets `layer.pixel`, `layer.text`, `selection`, `document` | T1 tree or Form, scoped to the current target; `inspector.target` event on change | Collapsible; host order; max 1 section per target per plugin | Collapsed placeholder line "Smart Levels (plugin missing)" only if the target has that plugin's data |
| **Metadata fields** (LightCraft) | `metadata.provider@1` with field schemas | Host draws fields in the Metadata panel; searchable | Typed fields only | Values kept; fields shown read-only, greyed |
| **Export / publish service sections** (LightCraft) | `export.service@1` / `publish.service@1` with `form` | Section in the Export dialog; publish collection UI | Like Lightroom's `LrView` sections | Service hidden; collections kept read-only |
| **Library browser** (presets, LUTs, assets) | data contributions plus optional panel | Host-drawn asset grid with plugin categories | — | Assets hidden |

### 4.4 Vector, design and layout apps (VectorCraft, FigCraft, DesignCraft)

| Point | Notes |
|---|---|
| **Tool** | As in §4.3, plus host **snapping providers** (`snap.provider@1`: returns candidate points or lines for a query rect, ≤ 2 ms budget, cached per document revision) and **path gizmos** (`bezier-handle`, `box`) |
| **Live effect in the appearance stack** | `vector.effect@1`: host-drawn appearance row with the effect's Form; on-canvas handles via U3 |
| **Inspector section** | Targets `shape`, `text`, `frame`, `component`, `page`; in FigCraft also Dev Mode **codegen panels** (`codegen@1`, read-only code view node `code` at UI 1.4) |
| **Object renderer for plugin-owned objects** | `item.renderer@1`: a plugin data type renders through a host graph or vector output (never UI drawing), so export and print match. This is the safe form of "override rendering of an item": plugin objects only, never built-in objects |
| **On-canvas widgets** (FigCraft) | `widget@1` P3: a T1/T2 subtree anchored to a document node, synced state in plugin data, drawn by the host at canvas zoom with level-of-detail |
| **Preflight rules and panel** (DesignCraft) | `preflight.rule@1`: findings rendered in the host Preflight panel (`FindingsList` component) with "Show" navigation |
| **Text and story importers** | `story.import@1` entries in Place… |

### 4.5 Motion and time apps (EffectCraft, FilmCraft, AniCraft)

| Point | Notes |
|---|---|
| **Effect Controls rows** | Form params, now animatable (keyframe stopwatch host-drawn), with gestures for automation recording (U1) |
| **Comp-viewer overlays and handles** | U3 `craft_overlay`: handles bound to params; drags make keyframes in auto-key mode; one undo step |
| **Timeline lanes and tracks** | `timeline.lane@1` (§3.4): plugin lane kinds such as markers, beat-detection, captions or AI-annotation lanes. Items and keyframes are host-drawn; plugin data in the project. Missing plugin: the lane shows as a read-only grey lane with stored items, never discarded |
| **Transitions and generators** | `video.transition@1` / `video.generator@1` in Effects & Presets with Form UI |
| **Scopes** | `scope` node fed by a ring or a viewer of a host-computed scope image |
| **Script-panel parity** (public EffectCraft ScriptUI users) | T1 panels with commands cover ScriptUI-class panels. Needs a disposition entry per the replacement rule |
| **Ease editors** (AniCraft) | AniCraft's own ease plug-ins (`anicraft/crates/plugins/src/manifest.rs:130`) map to a `curve` node plus a `kernel.program` ease function |
| **Onion-skin / guide overlays** (AniCraft) | Overlay layers in document space with `time` binding |

### 4.6 Node and 3D apps (MaterialCraft, TextureCraft)

| Point | Notes |
|---|---|
| **Node types** | `graph.node@1` (T3): category, ports, Form params, thumbnail by viewer. Needs the closed node enums opened (capability program finding 9) |
| **Graph kinds / function graphs** | Plugin-owned subgraph editors use the same host editor |
| **3D viewport overlays and gizmos** | `viewport.overlay@1`: world-space lines, points, labels; gizmo kinds `translate-3d`, `rotate-3d`, `scale-3d`, `point-on-surface`; the host projects and depth-tests |
| **Paint tools on meshes** | Tool contract with surface hit info (UV, normal, position) from the host |
| **Bakers** | `texture.bake-map@1`; bake dialog sections |
| **Smart materials / masks** | Data contributions in the host library browser |

### 4.7 Audio app (SoundCraft; audio in FilmCraft and EffectCraft)

| Point | Notes |
|---|---|
| **Device UI for `audio.effect@1` / `instrument@1`** | Default: host-generated editor from parameters (as Bitwig's generic UI or M4L presentation mode), with `knob`, `meter` and `scope` nodes bound to `param:` and `ring:`. Optional T1/T2 custom device face; no per-frame guest call |
| **Foreign editor (CLAP/VST3 native)** | T5: existing SoundCraft path (`clap-host/src/ffi/gui.rs`). Host title chrome names the plugin; floating window; keyboard routed to the host for transport keys unless the editor has text focus |
| **Mixer strip inserts, track header widgets** | `kind="decorator"` targets `track.header` and `mixer.strip`; small T1 subtrees (meter, toggle, badge) |
| **Clip editors** | `audio.clip-editor@1` (ARA-like): a T3 timeline lane plus a viewer for analysis results |
| **Piano-roll / note FX UI** | `audio.note-fx@1` with Form; note highlights through overlay layers in the piano roll |

---

## 5. Behaviour-changing hooks

The owner wants plugins to "change things in the application". The safe reading: **plugins may add, observe, provide, augment and replace their own or explicitly pluggable behaviour, under declared permissions, with transparency and a bypass**. They may not take over the person's control of the app.

### 5.1 Allowed hooks

| Hook family | Examples | Phase | May do | Permission | Budget | Failure policy |
|---|---|---|---|---|---|---|
| **Observe** (`app.events`, H4) | `document.changed {changes}`, `selection.changed`, `tool.changed`, `history.changed`, `document.saved`, `panel.opened` | after | Read the payload; schedule a deferred follow-up edit transaction (loop-detected) | `app.events` | Async; coalesced | Drop events, mark the plugin slow |
| **Automatic actions** | Auto-rename new layers, apply house style on import, run a linter on save | after (deferred transaction) | One undoable transaction labelled "*Plugin*: action"; the user can turn it off per action in the manager | `app.automate` (new) plus document edit scope | 5 s event limit (existing) | Rolled back; notice; auto-disabled after 3 failures |
| **Provide** | Snapping candidates, default new-document presets, export presets, colour palettes, font lists, asset sources, search providers in the command palette | on demand | Return data the host merges and labels with the plugin name | Free (data) or `network:*` if remote | 2–50 ms per call, by kind; cached by document revision | Omitted from results |
| **Pre-save / pre-export validation** | Preflight, "missing alt text", "fonts not embedded" | before | Return findings (info, warn, block-suggestion). The host shows them; the **user** chooses whether to continue | `export.hooks` | 500 ms sync, then the host proceeds and shows "still checking" | **Fail open**: save and export always proceed |
| **Export pipeline step** | Add a sidecar, watermark, upload after export, post-process the bytes | explicit step | Visible as a checked step in the Export dialog with the plugin's name; output by handle; external effects are honest (not undo) | `export.hooks` plus `files.write:picked` / `network:*` | Async job with progress | Step marked failed; primary export kept |
| **Save participation** | Write plugin data into the project; migrate own state | before save | Only its own `project.data` | Free for own data | 200 ms | Last saved plugin data kept; save proceeds |
| **Tool augmentation** | Input filter (stabiliser), extra options in a built-in tool's options bar, custom brush engine, snapping | during | Transform the pen stream; add options; draw guides | `tool.augment` | ≤ 1 ms per sample batch (kernel or JIT tier) | Bypass to raw input |
| **Tool replacement** (user-chosen) | "Use *Smart Crop* for the Crop tool" | while active | A plugin tool registers as an **alternative implementation** for a tool slot; the user picks it in tool options or preferences | `tool.replace` (consent at enable) | Gesture pipeline | Reverts to the built-in tool, with a notice |
| **Item rendering for plugin-owned items** | Custom shape types, custom layer kinds, procedural fills | render | Rendering through `item.renderer@1` with graph, vector or WGSL output | Free (own data) | Render budget, cached | Placeholder rendering with a hatch pattern plus a "missing plugin" badge; export refuses if `renders()` (existing rule) |
| **Command wrapping (observe)** | Log, analytics to the plugin's own panel, macro recording | after | See the command id and params of classified `query` / `edit` commands (registry bridge, capability program §2.3) | `app.events` plus `document.read` | Async | Dropped |
| **Default overrides** | "New documents default to 300 dpi", "default export quality 85" | at use | *Propose* a settings change; the host shows a one-time confirm listing each change, and records it as a user setting attributed to the plugin | `settings.propose` | n/a | Settings stay (they are the user's now); the manager lists them for revert |
| **Context-menu and inspector additions** | §4 | — | — | Free | — | — |

Design rules for every hook:

1. **Declared**: each hook is a manifest contribution (`kind="hook"`, `hook="export.before"`), so the manager shows exactly what a plugin changes before enabling it. Consent copy comes from `Permission::sentence` (pattern at `permission.rs:119-127`).
2. **Ordered deterministically** by user order in the manager (default: install order). No plugin priority numbers.
3. **Transparent**: a status-bar "plugins active" indicator lists hooks that ran in the last minute. Each hook run is logged in the manager (H-log, as the network log in H2).
4. **Bounded**: sync budgets as above. Over-budget hooks go async or are skipped; three strikes auto-disable the hook (existing quarantine policy).
5. **Bypassable**: safe mode disables all hooks. Holding a modifier during save or export (for example Alt) runs it with hooks bypassed, and a menu item says so.
6. **No chains of trust**: a hook's follow-up edits run with that plugin's own grants, in a fresh transaction, and cannot trigger the same hook recursively (loop detection: depth 2).

### 5.2 Never allowed

| Forbidden | Why | Enforcement |
|---|---|---|
| Drawing over, near or inside permission, consent, manager, update, quit or unsaved-changes UI | Spoofing and clickjacking | Trust layer above all plugin layers; plugin surfaces dimmed and input-frozen while it is up (§8.6) |
| Intercepting keystrokes or pointer input globally, or keys not routed to a focused plugin surface | Input theft, key logging | No raw input API; keys only through command bindings or a focused canvas; Escape is never delivered |
| Hiding, disabling, reordering or relabelling built-in UI, menus or commands | The user loses control; also a tool for spoofing | No op exists. Layouts that hide panels are user-applied presets only |
| Replacing built-in tools or commands silently | Behaviour surprise | Replacement only as a user-chosen alternative (§5.1) |
| Faking host chrome (app title bar, app toast, system dialog look, the app's own icon or name) | Spoofing | Host-drawn chrome; reserved words and the app icon refused in plugin titles; plugin toasts always carry a plugin attribution |
| Blocking save, quit, close or undo | Data loss and hostage-taking | Hooks fail open; quit never waits on a plugin more than 2 s |
| Touching undo or history directly, or other plugins' data or surfaces | Integrity | Not in any contract (existing, `author-guide.md §12`) |
| Screen capture or reading UI pixels | Privacy | No op; viewers show host handles only |
| Always-on-top, transparent, click-through or off-screen windows | Overlay attacks | Window ops take no such options |
| Taking focus without a user gesture; opening windows on load | Focus theft, pop-up spam | Transient-activation gate (§3.7) |
| Changing keymaps, themes or workspaces without the user applying them | Control | Data contributions applied explicitly |
| Native code hooking OS input (native-full tier) | Outside platform reach | Out of scope for enforcement; native-full consent text says so (existing `NativeMode::Full` sentence) |

---

## 6. Performance

### 6.1 Costs today (from source)

| Cost | Where | Effect |
|---|---|---|
| Whole-tree replacement per `ui.panel.set`; re-parse per revision | `ext-ui/src/plugin_ui/mod.rs:149-158` | A slider moving 60 times a second means 60 trees, parses and validations, plus a framework settle miss |
| Images as JSON byte arrays (≤ 64 MiB per tree) | `tree.rs` image node | Serialising, copying and parsing megabytes per revision; texture re-upload |
| Concave paths re-triangulated each frame | `ui.md` "Known limit", `paint.rs` | CPU per frame for static art |
| PhotoCraft re-serialises every tree on each `ui.*` call | capability program §5 bug list | Quadratic-ish cost with many surfaces |
| EffectCraft repaints every 100 ms when idle | capability program §5 | Idle battery drain |
| PhotoCraft tool events wait synchronously, up to 10 s | `photocraft/crates/engine/src/plugin_sessions.rs:198-219`, `:330-383` | A slow tool plugin stalls input. Widget events are already posted asynchronously (`post_plugin_ui_event`, `:148`) |
| Interpreter speed | wasmi with `portable-dispatch`: 42 ms → 11 ms without it at 1080p (capability program finding 1, *measured*) | Event handlers are affordable; per-frame calls are not |

### 6.2 Design rules

1. **Zero guest calls per frame.** The host retains the parsed tree, the value store, canvas layer meshes and region tables. Per-frame work is host drawing only.
2. **Retained plus patch.** `ui.patch` costs O(changed nodes). The framework `content()` version (`framework-guide §1.3`) is `(rev, value-store generation)`, so unchanged panels *settle* (replay their last paint).
3. **Values without revisions.** `ui.values.set` updates bound slots; a change re-draws but does not re-validate or re-parse.
4. **Rings for high-rate data.** Meters, scopes and progress from jobs use host-owned SPSC rings: `ring.create {kind:"f32x2", len:4096}` returns an id. The audio thread, a job or a kernel writes to it; the UI reads the latest at frame time; no guest call is involved. Meters at 60 fps cost one ring read each.
5. **Input coalescing.** At most one `change`, `drag` or `move` event per widget per frame (`drag` already works this way, `ui.md`). Drop the intermediate ones while a previous event is in flight (latest-wins). Commits are never dropped.
6. **Nothing waits on the UI thread.**
   - Tool gestures are pipelined: the host draws its own cursor and the last overlay immediately, and the plugin's overlay catches up.
   - If the plugin lags by more than 150 ms, the plugin's surfaces show a subtle busy state. Nothing else is affected.
   - Remove the synchronous `plugin_wait` from GUI tool paths. Keep it for CLI and automation only.
7. **Images by handle.** `{"type":"image","handle":"h:42"}` refers to a host image (R5); the texture uploads once per handle generation. Inline `rgba` stays for ≤ 64 KiB icons and thumbnails (UI 1.0 compatibility).
8. **Host-side animation and timers.** Spinners, progress shimmer and `animate` run in the host. `timer.set` (H8) has a 16 ms minimum, pauses while hidden, and coalesces.
9. **Hidden surfaces cost nothing.** Hidden or collapsed panels do not draw and do not receive value or ring reads. Plugins get `panel.visible false` and should stop pushing.
10. **Idle is idle.** No repaint without input, value change, animation or a ring with new data. Fix EffectCraft's 100 ms idle repaint.

### 6.3 Budgets (*targets*, to verify with craft-libs `ui-bench` on `next` plus a `craft-ext` bench)

| Path | Target |
|---|---|
| All plugin surfaces' draw per frame (10 open panels, about 300 nodes each, framework renderer) | ≤ 2 ms on an Apple M-class laptop; ≤ 4 ms on a 2020-class x86 laptop |
| `ui.patch` of 100 changed nodes (parse, validate, apply) | ≤ 0.3 ms host time |
| Full tree of 2000 nodes (parse and validate) | ≤ 2 ms; budgeted at most 4 per second per surface, beyond which patches are required (`-6` "Use ui.patch") |
| Event round trip (click to new tree on screen), interpreter tier, small handler | ≤ 1 frame (16 ms) typical; ≤ 50 ms p95 |
| Live slider to preview update (document preview) | ≤ 2 frames when the effect renders within budget; the preview may lag but the slider never stalls |
| Meter, scope or ring display | 60 fps (120 on ProMotion), 0 guest calls |
| 10 000-row list | Virtualised; draws ≤ visible rows; lazy `rows_source` above 10 000 |
| Canvas with 20 000 ops, static | Tessellated once, ≤ 0.5 ms per frame to draw cached meshes |

### 6.4 Retained versus immediate: the verdict

egui and the framework are immediate-mode internally, which is fine because **the host** runs that code. The plugin protocol must be retained: tree, patches, values, rings and layers. Blender's `draw()` and Unity IMGUI show the cost of per-frame guest code, and Unity's move to UI Toolkit confirms the direction.

---

## 7. Cross-app consistency and the framework plan

### 7.1 Which UI stack each app is on (current upstream main)

| App | Ref read | UI stack | egui | Plugin UI today |
|---|---|---|---|---|
| PhotoCraft | `upstream-ro/main` `c337d6e4e` | **craft framework** `craft` @ `a6b9d9c` | fork `e9711ca` | Beta: docked framework panels, `ViewportWindow`, `FamilyTool`, `FrameworkEngine` by default |
| EffectCraft | `upstream-ro/main` `cd9ee1e4` | legacy `craft-ui` @ `5a7a63a` | fork `6d518fc` | Beta: floating `egui::Window` |
| FigCraft | `upstream-ro/main` `da2792cb` | legacy `craft-ui` @ `e11f9d9` | fork `6d518fc` | Beta: own side dock |
| FilmCraft | `upstream/main` `0b3f30ed` | legacy `craft-ui` @ `47107f6` | fork `6d518fc` | None |
| VectorCraft | `upstream/main` `ef698d76f` | legacy `craft-ui` @ `64b1819` | fork `6d518fc` | Own competing Wasm dialogs (to migrate and remove) |
| LightCraft | `upstream/main` `27b755a9` | legacy `craft-ui` @ `c5a1cbf` | fork `6d518fc` | None |
| DesignCraft | `upstream/main` `3930cff8` | legacy `craft-ui` @ `92d392c` | fork `6d518fc` | None |
| TextureCraft | `upstream/main` `97f6b81` | legacy `craft-ui` @ `390fe98` | fork `6d518fc` | None |
| MaterialCraft | `upstream/main` `49ebd50` | legacy `craft-ui` @ `453b0da` | fork `6d518fc` | None |
| SoundCraft | `upstream/main` `8ee63dc` | legacy `craft-ui` @ `ea24422` | fork `6d518fc` | CLAP/VST3 native editors in-process |
| AniCraft | `upstream/main` `ab48665` | **no craft-ui**; plain eframe | **stock 0.36** | Own plugin crate with dialogs |

The framework lives on craft-libs `next` (`427c816db`). craft-libs `main` still ships the legacy `craft-ui` "Reusable egui controls" (`crates/ui/Cargo.toml`). Its own `docking` module is generic over the panel key too (`crates/ui/src/docking/dock_model.rs:32`). **No app besides PhotoCraft has moved to the framework.** The plan must therefore serve legacy apps for a while, without a third renderer.

### 7.2 Architecture

```
guest SDK (craft_guest::ui)      ← one API, no renderer dependency
        │  JSON ops (ui.*, canvas.*, ring.*)  — later WIT (D8)
craft-ext-runtime (Runtime<A>)   ← owns ui.* ops, limits, revisions, patches, value store,
        │                          rings, contribution registry, when-evaluation, trust layer state
        ├── HostUi trait (adapter supplies placement services, ~15 methods)
        │
craft-ext-ui-model (no egui)     ← surface model, patch apply, layout ids, manager model
        ├── craft-ext-ui     (EguiEngine; legacy craft-ui apps)   [bridge, removal criteria below]
        └── craft-ext-ui-fw  (FrameworkEngine; framework apps)    [end state]
```

The `HostUi` trait (adapter side; the runtime calls it, never the plugin):

```rust
pub trait HostUi {
    fn contract(&self) -> UiContract;                       // level, placements, targets, context keys, limits
    fn register_panels(&mut self, panels: &[PanelDecl]);    // → DockPanels<String> + reconcile
    fn set_menu_rows(&mut self, rows: &[MenuRowDecl]);      // → craft_menu::Row in extension groups
    fn set_commands(&mut self, defs: &[CommandDecl], generation: u64); // → AppCatalog
    fn set_tools(&mut self, tools: &[ToolDecl]);            // → tool rail + CanvasTool adapter
    fn set_status_items(&mut self, items: &[StatusDecl]);
    fn set_inspector_sections(&mut self, s: &[InspectorDecl]);
    fn set_overlays(&mut self, doc: DocToken, layers: &[OverlayLayer]);
    fn open_window(&mut self, w: WindowDecl) -> Result<(), UiRefusal>;   // ViewportWindow / layer
    fn open_dialog(&mut self, d: DialogDecl) -> Result<(), UiRefusal>;   // StackDialog per instance
    fn notify(&mut self, n: NotifyDecl);                    // runtime::toast::post with attribution
    fn context(&self, key: &str) -> ContextValue;           // when-clause keys
    fn request_repaint(&self);
}
```

What each app must supply:

- **Framework apps** implement `HostUi` once with the framework (PhotoCraft's beta code in `plugins_ui.rs` and `panels/registry.rs` is about 80% of it).
- **Legacy apps** implement it over `craft-ui` docking, their own menu bars and egui windows. That is temporary code, kept small because the runtime owns the logic. This removes the drift the capability program measured: 4 versus 32 window limits, three dock behaviours, an invisible FigCraft `ui.notify`.

### 7.3 One component set, two renderers

- The **node vocabulary is the component set.** The framework engine maps each node to a framework component (`ui.md` "Relation to craft-framework"). The egui engine approximates the same look with `craft-ui` tokens where the legacy theme exposes them.
- **The same tree must produce the same events in both engines.** Add an engine-parity conformance test: golden event traces for scripted inputs, run against both engines in `craft-ext test`.
- New 1.3 and 1.4 nodes land in the framework engine first. The egui engine gets a functional (not pixel-matched) version, or draws `fallback`.
- `component` nodes resolve against a **suite component registry** (name, version, props schema) living in `craft-ext-ui-fw`. Components with plugin-facing semantics (curve, gradient, graph, timeline lane, meter) move into craft-libs `next` as framework components, since craft-libs cannot depend on craft-extensions (`framework-guide §4.3`).

### 7.4 Versioning

| Axis | Mechanism |
|---|---|
| UI vocabulary | `UI_LEVEL` 1.3 (patch, bind, live, lazy rows, menus, DnD, `t` refs), then 1.4 (editor nodes). Session contract minor carries it (`ui_level_for_session`) |
| Placements | Each placement kind is versioned in the app contract (`photocraft.session@1.3` lists `placements: panel@2, inspector@1, status@1, tool@2`). `host.capabilities` (H12) reports them at runtime |
| Fallback placement | A manifest may say `fallback = "panel"` for an inspector section, so apps without inspectors show it as a panel. With no fallback, the contribution is listed as "not available in *App*" in the manager (existing compatibility UI) |
| Component versions | `component {name, version}` with a required fallback (existing) |
| Retirement | A level is never removed; the egui engine is removed (not the level) when all apps are on the framework |

### 7.5 Migration order

| Step | Apps | Work |
|---|---|---|
| 1 | — | **Runtime first**: move `ui.*` and limits into `Runtime<A>` with `HostUi`. Apply this to EffectCraft, PhotoCraft and FigCraft (FigCraft first, as the capability program orders) |
| 2 | PhotoCraft | Finish framework hosting: placeholders for missing panels, inspector and status placements, trust layer, `content()` combining rev and generation |
| 3 | EffectCraft, FigCraft | Dock plugin panels with their current `craft-ui` docking (fixes "EC panels float"); unify window limits. When each app ports to the framework upstream, switch `HostUi` to the framework implementation. No plugin-visible change |
| 4 | VectorCraft, AniCraft | Migrate their own plugin systems (dialogs to Form, plugins to `craft_*`); delete the old loaders per the replacement rule. AniCraft must first move to the suite egui fork |
| 5 | LightCraft, DesignCraft, FilmCraft | New adapters, framework-first if their port lands first; otherwise legacy `HostUi` |
| 6 | MaterialCraft, TextureCraft | Open the node and effect enums first; then T3 node editor integration |
| 7 | SoundCraft | Audio contracts; generic device editors; T5 policy for native editors |

**Removal criteria for the `EngineChoice` bridge and `EguiEngine`:**
- all apps with plugin adapters are on the framework at one craft rev;
- the engine-parity tests pass;
- one release has passed with no egui-only bug reports.

The owner is the craft-extensions UI maintainer.

---

## 8. Edge cases

### 8.1 Multiple monitors, OS windows versus in-app windows

- **A plugin asks for a "window"; the host decides how to show it.** The host uses an OS viewport where available (`ViewportWindow`, which falls back to a layer: `framework-guide §1.4`). On Wayland it uses an OS window without position restore. On web builds it uses an in-app layer.
- Floating plugin panels become OS windows through `DockConfig::native_windows` when `allow_native` is set (`framework-guide §1.3`).
- **Placement restore**: by `SaveKey`. On restore the host clamps to the current monitor set: if the saved monitor is gone, it centres on the main window's monitor. Per-monitor DPI changes are handled in points.
- Plugins cannot set positions, only size hints and an anchor (`near: "panel:<id>" | "cursor" | "center"`).
- Plugin OS windows are owned by (parented to) the main window. They minimise with it, are never always-on-top, and close when the app closes.

### 8.2 Modal flows and deadlocks

- Plugin code never blocks on a dialog: `ui.dialog.open` returns at once and the result is an event.
- Only one plugin modal at a time per app. A second request queues, up to 4, then refuses with `-6`.
- Host modals (consent, unsaved changes, quit) always pre-empt plugin modals. The plugin modal is hidden, not cancelled, and restored afterwards.
- A tool gesture in progress is cancelled when any modal opens (`cancel` event; one undo step rolled back).
- If the plugin worker is busy or crashes while its modal is open, Close still works (it is host chrome). The host sends `cancel` when the worker returns, or drops it if the worker died.
- `executeAsModal`-style exclusive edits (Photoshop) are expressed as jobs with progress and cancel, never as UI modality.

### 8.3 Plugin UI while a long job or event runs

- Surfaces stay drawable: the host has the tree. Controls stay interactive. Events queue (bounded to 64 per surface, latest-wins for `change`) and carry the revision.
- Progress via `ring:` or job binding. Cancel is a host-drawn button wired to the job (`Jobs` / `JobQueue`).
- After 150 ms with an event in flight, the surface shows a busy hint. After 5 s, the existing transaction timeout ends the event with a notice.

### 8.4 Restoring windows and layouts with missing plugins

- **Dock panels**: `reconcile` moves unregistered ids into `hidden` with their location (`workspace.rs:280-346`). The app must write back the reconciled workspace, keeping `hidden`. Re-run reconcile on registry change (framework gap 3).
- **Floating entries for unregistered panels are dropped** (`workspace.rs:301`). Recommend a framework change: keep a floating placement in a capped `hidden_floated` list, so a floating plugin panel returns where it was.
- **Placeholders**: if the missing panel was visible, show a host placeholder in its slot (§4.2), not an empty hole and not a silent removal. Users can "Remove from layout".
- **Windows**: not reopened. Placements are kept (pruned after 90 days or on uninstall).
- **Documents**: plugin data survives open-without, save, reinstall and downgrade (replacement rule). Lanes and inspector sections show read-only placeholders when their data exists.
- **Disabled versus uninstalled**: the same placeholders, with "Enable" versus "Install". Safe mode shows all plugin placeholders as "Off in safe mode".

### 8.5 Crash, hang and update

- **Crash**: the worker is quarantined (three strikes, existing). Surfaces freeze to their last tree, dimmed, with a restart notice. Overlays clear. Tools revert. Hooks are bypassed. Status items dim.
- **Hang**: the epoch or fuel deadline ends the event; same UI as a crash after 3 strikes.
- **Update while open**: surfaces keep their ids. The host sends `ui.reopen` (surface ids, last values) to the new version, which resends trees. If the new version no longer declares a panel, that panel follows the missing-panel path.

### 8.6 Spoofing and trusted chrome

The guarantees are structural:

1. **Plugin pixels are always framed by host pixels that name the plugin.**
   - Panel tab: plugin label plus the plugin glyph.
   - Window title bar, host-drawn: "*Plugin name* — *title*". The plugin cannot draw its own title bar.
   - Dialog header: plugin icon, name and publisher.
   - Toast: the plugin name in the toast header.
   - Status item: the tooltip names the plugin.
   - Overlays: a corner chip "Overlay: *Plugin*" while the overlay is non-empty and a plugin tool is active (PhotoCraft already labels the active plugin tool: `plugins_ui.rs:420`).
2. **A host-only trust layer.**
   - Consent, permission, manager, update, unsaved-changes, quit and native-access dialogs render on a layer above every plugin surface, in a style token set plugins cannot use. Plugin themes cannot repaint pinned trust tokens.
   - While the layer is up, all plugin surfaces are dimmed and **input-frozen** (no events delivered, no focus changes), and plugin OS windows are disabled.
   - Trust dialogs never contain plugin-supplied rich content, only plugin text in quoted, clipped fields (`manifest.rs` text validation).
3. **No pixel lies.**
   - Plugin trees cannot set pixel sizes or positions, so they cannot mimic exact host dialogs inside a panel.
   - Canvas text is drawn in the canvas rect only, and canvases cannot exceed their surface.
   - Plugin windows have no borderless, transparent or top-most options.
   - Plugin labels may not contain the app name or "ArtCraft" as a standalone title word in window and dialog titles (checked at validation, on top of the structural chrome).
4. **Input integrity.**
   - Canvas keys only while that canvas has focus; Escape always host.
   - No global key or pointer hooks. Drags end when the button is released anywhere (existing `drag` semantics).
   - Focus requests need transient activation. Plugins cannot open windows or dialogs at load, only in response to user-initiated events or explicit commands.
5. **Native tier (T5).** Foreign windows (CLAP and VST3 editors, native helper windows) cannot be framed by host chrome on every platform.
   - Where the host creates the parent window (embedded CLAP/VST3 editors), the host title bar names the plugin.
   - Free-floating native-helper windows are outside the guarantee, and native-full consent says "can show its own windows". The manager lists such plugins with a badge.

### 8.7 Web builds

PhotoCraft, EffectCraft and FigCraft ship `*-web` apps (`photocraft/apps/photocraft-web`, …).

- The T1, T2, T3 and T4 renderers are pure egui and work there.
- Windows become layers. The wasmi tier is used (no JIT; capability program R4). The GPU tier uses WebGPU where available, with the required CPU fallback.
- The native tier is absent; T5 contributions are marked "desktop only".
- Files arrive as File System Access handles; `net.fetch` maps to browser fetch, still behind the domain list. Keyboard shortcuts must avoid browser-reserved chords (the keymap already targets `mac | desktop | web` in `craft_menu` layout).

### 8.8 Other edge cases

| Case | Handling |
|---|---|
| Many plugins (menu overflow) | Plugins menu groups by plugin, with more than 12 going into a submenu. The command palette always lists every command. Toolbar plugin tools collapse into one "Plugin tools" flyout |
| Theme switch | Token paints and components re-resolve; no plugin round-trip |
| Collaboration (FigCraft) | Plugin UI state is local; plugin document data syncs through the document |
| Agents and automation | Every plugin widget has a stable automation id `plugin.<pid>.<surface>.<widget>`, inspectable through `ui.inspect` (`framework-guide §5` tests). Plugin dialogs expose `json_fields` |
| Multiple documents | Per-document surface instances (`instances = "per-document"`) are keyed `plugin.<pid>.<panel>#<doc>`. Inspector sections follow the active document |
| RTL and long text | The framework handles RTL. Labels truncate with a tooltip; trees with text over the limits are refused (existing limits) |
| Screen readers on canvases | Regions are focusable and announced. Plugins with canvas-only UI get a manager warning "Not accessible: canvas without regions" |

---

## 9. Open decisions with recommendations

| # | Decision | Options | Recommendation |
|---|---|---|---|
| O1 | HTML webview tier | (a) never; (b) out-of-process system webview as a separate OS window behind a `ui.web` permission and native-like consent; (c) in-panel child view | **(a) now; revisit (b) only with catalogue evidence** that T1–T4 plus the system browser cannot serve a famous plugin class. Never (c): it cannot be composited, themed or framed (§3.6) |
| O2 | Custom editors: plugin-drawn or host components | T2 only; T3 host components | **T3 for node graphs, curves, gradients, timelines and meters; T2 for the long tail.** Put the components in craft-libs `next` |
| O3 | Patch protocol | Full replace only; JSON patches; keyed ops | **Keyed ops (`ui.patch`) at UI 1.3**, with full replace kept. Refuse more than 4 full replaces per second per surface over 500 nodes |
| O4 | Whether plugins may hide or alter built-in UI | Allow with permission; never | **Never programmatically.** Workspace and keymap presets are data the user applies |
| O5 | Plugin default shortcuts | None; only if unbound; may override | **Only if unbound**; conflicts are reported in the manager |
| O6 | GPU content in panels | Raw wgpu callback; WGSL tier to handle to viewer | **WGSL tier to handle to `viewer`**; no raw device access |
| O7 | Plugin SVG icons | Catalogue names only; sanitised SVG | **Sanitised SVG subset, ≤ 64 KiB**, through framework runtime icon registration (gap 1). Catalogue names stay preferred |
| O8 | Hook failure policy | Fail closed; fail open | **Fail open** for save, export and quit; fail closed only for the plugin's own feature (its export step) |
| O9 | Unified limits | Per app; per suite | **Suite-wide in the runtime**: 16 panels and 4 windows per plugin; 64 plugin panels and 32 open windows per app; 2 status items; 32 menu rows; 1 modal |
| O10 | Where suite components live | `craft-ext-ui-fw`; craft-libs | **Generic editors (curve, gradient, graph, timeline lane, meter, scope, viewer) in craft-libs `next`**, the plugin mapping in `craft-ext-ui-fw` |
| O11 | Legacy egui renderer lifetime | Keep; remove | **Remove** once the criteria in §7.5 are met; it is a bridge, not a second system |
| O12 | Native foreign windows (T5) | Ban; allow with labels | **Allow for the native tier only**, host-parented where the protocol allows (CLAP and VST3 embed), labelled, with keyboard routed to the host except during text entry |
| O13 | Floating plugin panels with the plugin missing | Drop (framework today); keep placement | **Keep a capped floated-hidden list**: a framework change on `next` |
| O14 | `when` expressions | Host-evaluated closed grammar; plugin `poll` callbacks | **Host-evaluated grammar**: no per-frame guest call, as VS Code does |
| O15 | Localisation format | JSON; Fluent `.ftl` | **Fluent `.ftl`** per capability program H11; resolved by the host before render |

### Suggested build order (UI work only, small to large)

1. **S**: runtime-owned `ui.*` limits; `ui.notify` attribution; window and dialog chrome with the plugin identity; transient-activation gate; Escape routing.
2. **S/M**: `ui.patch`, `ui.values.set`, `live` controls (U1); remove synchronous tool waits; idle repaint fix.
3. **M**: placements for command menus, status, inspector, settings and context menus through `HostUi`; missing-plugin placeholders; reconcile write-back.
4. **M**: T2 retained layers, regions and token paints; `PluginCanvas` framework component; overlay hit regions and gizmos (U2, U3).
5. **M**: rings plus `meter` and `scope`; image handles (R5) plus `viewer` (T4).
6. **L**: T3 editors (curve, gradient, graph, timeline lane) on craft-libs `next`, then the `graph.node@1` and `timeline.lane@1` placements.
7. **M**: behaviour hooks: provide, export steps, input filters, tool replacement, `settings.propose`.
8. Continuous: engine-parity and accessibility conformance in `craft-ext test`; app-by-app `HostUi` adoption per §7.5.

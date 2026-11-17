# Craft UI framework: a guide for the plugin platform

Written 2026-11-17 from a read-only study. Nothing was built or run, so every claim below comes from reading source and docs.

## 0. Sources and path shorthands

| Short | Path | What it is |
|---|---|---|
| `CL/` | `/private/tmp/claude-501/-Users-ash-Projects-plugin-beta/e1ff60be-d392-4350-8c1f-a3227f97c217/scratchpad/src/craft-libs-a6b9d9c/` | craft-libs at rev `a6b9d9c` (the rev PhotoCraft pins) |
| `KIT/` | `.../scratchpad/src/craft-ui-port-kit/` | The porting kit |
| `PC@up:` | `/Users/ash/Projects/plugin-beta/photocraft` at `refs/upstream-ro/main` | Read with `git show "refs/upstream-ro/main:<path>"` |
| `PC/` | `/Users/ash/Projects/plugin-beta/photocraft/crates/ui-egui/src/` | The plugin-beta working tree |
| `EXT/` | `/Users/ash/Projects/plugin-beta/craft-extensions/crates/` | |

Notes on the sources:
- `refs/upstream-ro/main` now points at `c337d6e4e`. Commit `ed14e1c41` is an ancestor of it. A sub-agent ran `git diff --stat` between the two and reported no changes under `apps/photocraft`, `crates/ui-egui`, `docs` or `Cargo.toml`. I re-checked about a dozen line references against `c337d6e4e` myself.
- `craft-color-ui` is not in the export. Colour-picker facts come from the docs only (`CL/docs/craft-framework.md:594`).
- The egui revision differs between docs and code. `craft-framework.md:343` names `dcd4de1f`, but `CL/Cargo.toml:276-289` and `CL/crates/craft/src/egui_fork.rs:6` pin `storytold/egui` at `e9711ca9…`. Treat the code as the source of truth.

---

## 1. Architecture

### 1.1 Layers and crates

**Tiers** (`CL/docs/craft-framework.md:188-207`):

| Tier | Contents |
|---|---|
| 0 | Services that do not use egui: `craft-theme`, `craft-behavior`, `craft-host`, `craft-keymap`, `craft-menu`, `craft-settings`, `craft-i18n` |
| 1 | The UI library: `craft-ui-contract`, `-widgets`, `-panels`, `-canvas`, `-patterns`, `-shell`, plus the `craft-ui` facade |
| 2 | Families: `craft-color-ui`, `craft-layers` and others |
| 3 | The `craft` facade |

**What each UI library crate holds** (`:225-233`):

| Crate | Holds |
|---|---|
| contract | Runtime, style, primitives, the component trait, test kit |
| widgets | Controls and composites |
| panels | Sections, dialogs, `DialogHost`, docking parts, rails |
| canvas | `Canvas`, tools, full-window dialogs |
| patterns | Form dialogs, `DockArea`, preferences, presets, palette |
| shell | `AppShell`, title bar, `embed`, `ViewportWindow`, `DocumentWindow` |

**How apps depend on it.** An app depends on one crate, `craft`, pinned by git rev (`:117-133`). PhotoCraft does this at `PC@up:Cargo.toml:15`.
- Paths take the form `craft::ui::<module>`. Core types sit at the crate root, UI types at `<family>::ui`.
- `craft::prelude` holds `AppInfo, Host, Section, Tokens, DialogStack, Outcome` (`:242-243`).

**One egui revision.** Every egui crate must come from `craft::egui_fork::{GIT, REV}`. An app's `xtask` calls `check_lockfile` to verify this (`:339-341`). Two revisions in one build produce incompatible `Ui` types. This is the hard constraint on ext-ui (see §4.1).

**Host convention** (`:266-316`):
- The app builds `craft::host::Host` once (i18n, settings sections, commands, files, jobs) and installs it. Features read it with `Host::of(ui)`.
- Features return edit events (`Begin/Update/Commit/Cancel/Set`). The app applies them to its own undo stack.
- Features never open their own windows. Dialogs are values opened on `DialogHost`.
- Work runs on the host's `Jobs` or `JobQueue`, never on a thread the feature spawns (`:311-316`; `CL/crates/host/src/queue.rs:331`).

### 1.2 Shell and frame hooks

Apps with their own eframe `App` use `craft::ui::shell::embed` (`craft-framework.md:137-171`):
- `embed::install(ctx, InstallSpec, &UiFonts)` once at startup.
- `embed::before_input(ctx, |FrameOutput| FrameInput)` in `raw_input_hook`. The closure receives the fired commands, chrome toggles and announcements, and returns `RootEnv`, command states, `Host` and catalog.
- `embed::logic(ctx, || CloseDecision)` handles closing.
- `embed::before_ui`, and optionally `runtime::focus::host_canvas` and `keys::with_window_intents`.
- Apps without their own `App` implement `ShellApp` and call `shell::app::run` (`:135`).

`AppShell::new(WindowKey)` composes a window from slots: `.title_bar_if`, `.options_bar`, `.status`, `.body(|ui, cx, app| …)`, then `.show(ui, app)`.
- The body's `BodyCx` reaches the window's `DialogHost` through `cx.dialogs()` (`ui-dialogs.md:56-72`).
- The shell also hosts the window's toast queue.

### 1.3 Workspace, dock and panels

The dock has three layers (`CL/docs/ui-docking.md:1-11`):
1. The model: `craft_behavior::dock::{Workspace, Layout, Node, Action, Permissions, WorkspaceSet}`. It is serde data and does not use egui.
2. The parts: `craft_ui::docking`.
3. The patterns: `craft_ui::patterns::dock::{DockArea, PaneArea, WorkspaceSwitcher}`.

**Panel keys are generic.** `Workspace<P>` uses `P`, the app's panel key, which must be `Clone + Eq + Hash`.
- PhotoCraft uses `P = String` (`PC@up:crates/ui-egui/src/panels/panel_docking.rs:26`).
- So runtime string ids already work at the model level. Nothing in the framework requires a compile-time enum.

**How a panel is declared.** The app implements `DockPanels<P>` (`CL/crates/ui-panels/src/docking/panels.rs:21-61`):

| Method | Purpose |
|---|---|
| `spec(&P) -> PanelSpec` | Read every pass |
| `permissions` | Default `ALL` |
| `body(&P, ui, cx)` | Draws the panel |
| `content(&P) -> Option<u64>` | A content version: an unchanged version lets the body "settle" (replay its last paint) |
| `group_view`, `group_header` | Optional group-level drawing |
| `panel_rows` | The panel menu's rows |

`PanelSpec` fields are at `CL/crates/ui-panels/src/docking/spec.rs:108-133`: `label: Label`, `icon: Option<IconSpec>`, `limits`, `heights`, `float_size`, `allow_native`, `window_kind`, `canvas`, `grid`. There is no registry in the framework. The panel keys, the registry and the bodies stay in the app (`ui-docking.md:143-147`).

**Showing the dock:** `DockArea::new(label, Bind::bound|given(&mut ws), &mut panels, DockConfig)` with props `.locked() .auto_collapse() .hide() .flyout() .maximized()`.
- `.show(ui)` returns `DockAreaOutput {actions, requested, events, windows, error, …}` (`ui-docking.md:38-82`).
- With `Bind::given`, the host applies the actions itself.
- A body changes the layout only through `docking::request_actions(ctx, actions)` (`:66`).

**Persistence:** `Workspace` is serde. To load one (`ui-docking.md:30-36`):
1. Deserialize it.
2. Call `validate()`. A failing file, or one from a newer version, stays on disk and is never overwritten.
3. Call `reconcile(registered, permissions, &default)`.

**Unknown panel ids** (`CL/crates/behavior/src/dock/workspace.rs:280-346`):
- `reconcile` moves panels that are not registered now into `hidden`, with their location: "(plug-ins, on-demand or numbered panels) move to `hidden` with their location, so they come back once registered".
- `hidden` is capped at `MAX_PANELS`, and the oldest unregistered entries are dropped first (`:338-345`).
- Floating entries for unregistered panels are dropped (`:301`).

So a plugin panel whose plugin is missing at startup keeps its place and returns when the plugin registers, as long as the app writes the reconciled workspace back with its hidden list intact. In PhotoCraft upstream:
- `loaded()` calls `reconcile(|p| registry::registered(p), …)` (`PC@up:…/panels/panel_docking.rs:751`).
- An action on an unknown id is refused with "is not a registered panel" (`:133-135`). `spec()` returns a bare `PanelSpec::new(id)` and `body()` draws nothing (`:154`, `:225`).

**Window menu.** `WindowMenuRows::new(layout, &dyn DockPanels<P>, order: &[P])` builds check rows from any slice of keys (`CL/crates/ui-panels/src/docking/window_menu.rs:21-43`), so runtime panel lists work. PhotoCraft does not use it for panels: its rows are a static catalogue (§3.2).

**Commands and workspaces** (`ui-docking.md:113-121`): `DockCommands` (float, move, close…), `WorkspaceCommands`, `ArrangeCommands`.

**Floating panels as OS windows:**
- `DockConfig::native_windows` defaults to on. Each floating window becomes its own OS window where the window host can report positions (not on Wayland), if `PanelSpec::allow_native` is set (`ui-docking.md:96`, `:123-127`).
- PhotoCraft enables this on desktop (`panel_docking.rs:249`).

### 1.4 Hosted dialogs and real OS windows

`craft_ui::dialog` (`CL/docs/ui-dialogs.md`):
- **`Dialog`** is modal or modeless, with a footer of `FooterButton`s, each carrying a `DialogRole` (`:18-54`). Options:
  - `.viewport()` shows the dialog in its own OS window, falling back to a layer (`:46`).
  - `.modeless()` makes a tool window (`:45`).
  - `.remember(SaveKey)` keeps placement and size.
- **`DialogHost` and `StackDialog`** (`:56-88`):
  - Open with `open`, `open_child`, `open_unless_open`; read the result with `take::<D>()`.
  - There is one open dialog per type and `StackDialog::instance`. Opening a second cancels the first (`:80`), so plugin dialogs must give each surface its own instance id.
  - `json_fields` and `set_json_field` make a dialog automatable through `ui.dialog.*` (`:85`).
  - The hosted forms include `HostedParamDialog`, `HostedAlert`, `HostedAbout` and others.
- **Prompts:** `Prompt`, `StandardPrompt` (confirm_delete, custom, dont_show_again), `Alert`, `UnsavedChangesDialog` (`:94-110`).

**Real secondary OS windows. Yes, there are four routes:**
1. `craft::ui::shell::ViewportWindow::new(title).app_modal(..).remember(key).overlay(..).show(ui, |ui, cx| …)` (`CL/crates/ui-shell/src/shell/viewport_window.rs:1-80`).
   - It opens its own OS window where windows exist, otherwise a dialog layer.
   - The app calls it every pass while the window is open, and stops when `close_requested` is `Some`.
   - This is the right container for a "plugin window".
2. `DocumentWindow` is a document's own viewport (components-index row 234). PhotoCraft uses it in `PC@up:…/canvas/document_windows.rs:78`.
3. A floating dock panel with native windows on (§1.3).
4. `Dialog::viewport` (above).

Low-level control is through `runtime::windows::WindowHost::of(ctx)`. It provides `viewports_available()`, `act(viewport, WindowAction)`, `facts`, `WindowSpec::{tool, document, dialog}` and `WindowLevel::{Normal, Palette, AlwaysOnTop}` (`CL/docs/contract-api.md:3773-3841`).

PhotoCraft calls no raw `show_viewport_*` anywhere, and plugin code should not either.

### 1.5 Commands, menus and keymap

**Command id.** The id is `craft_keymap::CommandRef { id: String, params }`, built with `CommandRef::new` or `with_params`.
- Parameterised keys serialise as `window.panel {"panel":"layers"}` (`CL/crates/keymap/src/lib.rs:27-62`).
- Typed sets use `#[derive(CommandSet)]` with a prefix and scope. An example is `canvasView` `ViewCommands` (`CL/crates/ui-canvas/src/canvas/tools.rs:117-160`).

**Runtime commands.** `AppCatalog::new(defs: Vec<CommandDef>, generation)` is documented as "Commands an app defines at run time (plug-ins, its tool list) … their chords fire them into `FrameOutput.fired`" (`CL/crates/ui-contract/src/runtime/commands/mod.rs:42-55`).
- It refuses duplicate ids and defs outside the app scope.
- Pass it with `CommandInput::catalog` (`:327`). The keymap rebuilds when the generation changes.
- Per-frame state comes from `CommandInput::state(cmd, CommandState::new().enabled().checked().title().hint())`.
- `CommandGate` gates all commands. `runtime::commands::fire(ctx, cmd, FireSource)` fires from menus and automation (`contract-api.md:3103-3160`).

**Menus.** `craft_menu`, `CL/crates/menu/src/lib.rs:1-40`:
- `Row::new(path: &[&str], command, label)` (`model.rs:598`) and `from_items(tops, &rows) -> MenuBar` (`:657`), then `layout(Target)` for mac, desktop or web.
- In-window bar: `MenuBarUi`. On macOS, `mac::install` builds the native menu.
- Item state follows the runtime command state.
- Rows are plain data with string ids, so dynamic rows are trivial.

**Keymap.** `Keymap::new(catalog, Scopes, Platform)`, `lookup`, `Overrides`. Overrides are stored in settings with `#[serde(with = "craft_keymap::keymap::overrides_list")]` (`keymap/src/lib.rs:64-80`).

### 1.6 Focus, IME and pointer routing

`embed::install` takes over parts of input for the whole window (`craft-framework.md:150-169`):
- Tab, F6, F10 and Alt+letter, and Escape route to framework stops.
- Egui's keyboard zoom and quit shortcuts are cleared.
- A window gets one press per pass.

**Raw egui widgets are tolerated.** While a raw egui widget the framework did not make has egui focus, Tab, Escape and the clipboard stay in egui (`:160`). This is why the current egui renderer would still work inside the shell. It would only be degraded (no framework focus ring, no access tree, no roving), and it breaks the "one mechanism" rule.

**IME** belongs to the runtime while a framework editor owns it. Otherwise it is egui's (`:159`). Text and IME go through the one shared engine (`KIT/rules/framework-decisions.md:27`).

**Canvases:**
- A custom document canvas declares itself each pass with `runtime::focus::host_canvas(ctx, id, KeyUse::CANVAS)` (`:147`).
- Keys that no component took are read last with `keys::with_window_intents`.
- On-canvas tools implement `craft_ui::canvas::CanvasTool` (`press`, `each_pass`, …) (`CL/crates/ui-canvas/src/canvas/tool.rs:620`).
- Raw painting is allowed only for allowlisted `ContentPainter` components (`contract-api.md:1747`, `:2227`).

**Pass model.** Two passes may run in one frame, and input reaches only the first. There is no state carried over between passes except the three framework mechanisms (`KIT/rules/porting-rules.md:41-46`). Plugin event emission must therefore be on edges (commit, click), never "value differs from last pass".

### 1.7 Canvas view commands

`ViewCommands` (scope `canvasView`) is at `tools.rs:117-160`:

| Command | Default chord |
|---|---|
| ZoomIn | Cmd+= |
| ZoomOut | Cmd+- |
| FitOnScreen | Cmd+0 |
| ActualPixels | Cmd+1 |
| FillScreen, 200%, PrintSize, FitArtboard, FitLayers, Rotate CW/CCW, ResetRotation | none |

- Commands go to the focused instance, which may be a full-window dialog's canvas.
- The other sets are `MomentaryCommands`, `NavigatorCommands` and `RulerCommands`.
- A plugin canvas surface that wants zoom should be a `canvasView` instance, not handle `Cmd+=` itself.

### 1.8 Settings storage

**Policy** (`CL/docs/settings-storage.md`):
- Editable settings live in `config.toml` in the config dir, through `craft-settings`.
- UI state goes in `ui.ron` beside it, caches in the platform cache dir, secrets in the OS keychain.
- `<APP>_CONFIG_DIR` set to an empty value means isolated (nothing read or written).
- Saves are atomic, unknown keys survive, and a newer file is never overwritten.

**Feature settings** are `craft_host::Section { const KEY }` tables, stored through the app's `SectionStore` (`CL/crates/host/src/services.rs:52-64`). PhotoCraft persists them into `Preferences::sections` via `PrefSections` (`PC@up:…/app/framework.rs:601-641`).

**Remembered dialog values** use `Remembered<T>`, `runtime::memory::set_saved_value` and `saved_value` (`KIT/rules/writer-notes.md:14`).

### 1.9 Themes and design tokens

**Colour pipeline** (`craft-framework.md:457-474`): seeds (neutral, accent, good, warn, bad, info), then 12-step OKLCH ramps, then semantic tokens, then pins, then a contrast pass.

**Semantic groups** (`:898-911`):

| Group | Tokens |
|---|---|
| surface | background, canvas, chrome, panel, raised, sunken, scrim… |
| text | normal, muted, faint, disabled, on accent… |
| line | divider, border, control edge, focus |
| accent | solid, hover, strong, soft, text |
| status | good, warn, bad, info, each with fg, bg, border and solid |
| state | hover, pressed, selected… |
| other | canvas overlay; data (8 series) |

**Typography roles** (`:476-491`): Caption, Label, Body, Body strong, Title, Heading, Body large, Mono. Text size scales the roles only.

**Dimensions** (`:498-525`):
- Spacing scale: 0, 2, 4, 6, 8, 12, 16, 20, 24, 32.
- Semantic roles: control height sm/md, icon sm/md/lg, gaps, pads, radii, bar heights.
- The Compact and Comfortable presets set the sizes. Themes never set sizes.
- Size role names (`size.*`, `space.gap.*`, `space.pad.*`) are at `:983-1011`.

**How code obtains values:**
- **Normal app code does not.** It picks a component plus a `Variant` and a `Tone`:
  - `craft_theme::variant::Tone { Neutral, Accent, Info, Good, Warn, Bad }` (`CL/crates/theme/src/variant.rs:28`).
  - `Variant::{PRIMARY, SECONDARY, GHOST, DANGER}` (`:325-331`).
- Component styles read tokens through `StyleCx`. It is the only constructor of `StyleColor` (`color`, `tone`, `tone_fill`, `ink`, `control()`, `surface(kind)`, `rule(kind)`, `focus_look`) (`contract-api.md:3849-3956`).
- `Env::of(ui)` gives words, number and date formats, `folder()` for search, `collator()` and mode. `Inherited.tokens: Arc<UiTokens>` exists (`:3299-3330`).
- PhotoCraft still has an app-level `crate::theme::Tokens::get(ctx)` for its own legacy drawing (`PC@up:docs/ui-design.md:30`; `theme.rs:879`).

**Light and dark:**
- `RootEnv` carries a theme id or library theme and `.mode(Dark|Light)`.
- Suite-wide choice lives in `selection.json` in `ArtCraft/themes/` (`CL/docs/suite-themes.md:133-200`), with precedence app override, then suite choice, then app default (`craft-framework.md:543-554`).
- `craft-theme-store`'s `SuiteClient` watches for changes. PhotoCraft wires it in `Framework::sync_theme` (`PC@up:…/app/framework.rs:367`).
- There are 10 built-in themes with 17 variants (`:929-944`).

### 1.10 Component inventory for plugin UI

From `CL/docs/components-index.md` (generated) and the module table at `craft-framework.md:1762-1786`:

| Need | Component (module) | Notes |
|---|---|---|
| Button / icon button / toggle / split | `Button`, `IconButton`, `ToggleButton`, `SplitButton`, `Link` (`buttons`) | `Variant::PRIMARY/DANGER/GHOST` |
| Slider | `Slider`, `RangeSlider` (`value_drag`); `SliderField` (`form`) | slider plus number |
| Numeric field | `NumberField` (`text::number`), `AngleDial`, `PointPicker`, `UnitSelect` | edit events Begin…Commit |
| Text | `TextField` (`text_field.rs`; `.lines(n)` multi-line at `:169`; `TextField::search(label, bind)` at `:87`), `ChipField`, `AltTextField`, `SelectableText` | |
| Colour | `Swatch`, `SwatchTable` (widgets); `ColorPicker`, `ColorPopover`, `SwatchPair` (craft-color-ui family) | colour models need the family |
| Dropdown | `Select` (`pickers/select.rs:119` `new(label, bind, &[Entry])`), `ComboBox`, `Autocomplete`, `TreePicker`, `IconPicker` | `Entry::stepped` |
| Check / radio / switch | `Checkbox`, `Radio`/`RadioGroup`, `Switch`, `Segmented`, `Rating` | |
| Lists / tables / trees | `List` + `ListRow` (`list/view.rs`; tree mode `:53`, `ListRow` depth `row.rs:252`), `Table`, `MetadataTable`, `TileGrid`/`Tile`, `EventList`, `FindingsList` | collections virtualise rows |
| Tabs | `TabStrip`, `Tab`, `DocumentTabs` | one Tab stop |
| Sections / cards | `Section` (collapsible; `ui-panels/src/sections/section.rs:55`), `GroupBox`, `SectionStack`, `Panel`, `Tile`, `OpeningCard` | no generic "Card" type; use `Section`, `GroupBox` or `Tile` |
| Badge / kbd | `Badge`, `Kbd` (`display`) | |
| Progress / busy | `Progress` (`progress.rs:61` `new(fraction)`), `Spinner`, `StatusProgress`, `BlockingOverlay` | |
| Empty states | `CollectionProps::empty(text)` on every collection (`ui-contract/src/contract/collection/props.rs:592`), `Collection::empty()` hook | no standalone EmptyState |
| Notices / toasts | `Notice`, `NoticeCenter`, `ToastStack`, `SettingsNotices`, `StatusMessage`; `runtime::toast::{post, keep, dismiss, end}` + `Toast::new(msg).tone().title().lines().action()` (`contract-api.md:3746-3772`) | per-window queue, max cards |
| Icons | `Icon` with `IconSpec::{svg(IconId), glyph, swatch, …}`; `IconSource::Initial(char)` fallback (`primitives/icon.rs:41-57, 89-110`) | app SVG sets: `register_app_icons(&'static AppIcons)` (`svg_set.rs:34`) |
| Layout | `Stack`, `Row`, `Grid`, `Divider`, `Lazy` (`layout_components`) | fill/content sizing rules `:580-584` |
| Forms | `Field`, `FieldGroup`, `ParamRows`, `ParamDialog`, `FormDialog`, `PropertiesDialog` | |
| Search | `TextField::search`, `CommandPalette`, `PreferencesWindow` search, `MetadataTable` search; `Env::folder()` for matching | |
| Menus | `Menu`, `MenuItem`, `PanelMenu`, `context::rows` (craft-menu) | |
| Windows | `ViewportWindow`, `Dialog`, `FullWindowDialog`, `Popover`, `Tooltip` | |

---

## 2. Extension points for runtime-defined content

| Surface | Runtime string ids in the framework? | Where it is static today | Minimal seam |
|---|---|---|---|
| Dock panels | **Yes**: `Workspace<String>`, `DockPanels<String>`, `reconcile` keeps unknown plugin panels hidden (`workspace.rs:280`) | PhotoCraft's `static PANELS: &[PanelEntry]` with `fn` pointers (`PC@up:…/panels/registry.rs:20-41`, `:102-226`) | **App** (§3.1) |
| Panel icons | Partly: `IconId` takes `Cow<'static, str>` (`behavior/src/cursor.rs:50`, `From<Cow>` at `:64`) | SVG registration needs `&'static AppIcons` (`svg_set.rs:34`) | **Framework**: runtime icon registration (owned bytes), else leak per plugin version or use `IconSource::Initial` |
| Hosted dialogs | Yes: `StackDialog` is a trait object on `DialogHost`; `json_fields` for automation | one per type and instance: use `instance()` per surface | none (app implements one `PluginDialog: StackDialog`) |
| OS windows | Yes: `ViewportWindow` per pass | PhotoCraft's `extra_windows` list is app code | **App** |
| Commands and keymap | **Yes**: `AppCatalog` "for plug-ins" (`commands/mod.rs:42`) | PhotoCraft builds the catalog from `command_rows` only on a prefs revision change (`PC@up:…/app/framework.rs:554-564`; `shortcut_dispatch.rs:221-350`); plugin rows are not in it | **App**: include plugin `CommandDef`s and bump generation on registry change |
| Menu items | Yes: `craft_menu::Row` is data | `menu_catalog::CATALOG` static; plugin filter rows already injected (`PC@up:…/menus/items.rs:185`) | **App** |
| Toolbar / options bar items | `BarItems` take command rows; `ToolRail`; `CanvasTool` | PhotoCraft tool list is `Tool::ALL` enum (`framework.rs:527-539`) | **App** for bar buttons; plugin tools need an app `Tool::Plugin(String)`-style variant plus a `CanvasTool` adapter |
| Preferences pages | Yes: "The host builds the pages each pass" (`CL/crates/ui-patterns/src/patterns/preferences/window.rs:43-50`) | PhotoCraft `PrefsDialog` | **App** |
| Toasts | Yes: `runtime::toast::post(ctx, &WindowKey, Toast)` from any code with a `Context` | | none |
| Settings | `SectionStore::read/write(key: &str)`: any string key | `Section::KEY` is `&'static str` | none; use `SectionStore` directly with `plugin.<id>` keys |

### 2.1 Is ui-contract a declarative UI? No.

`craft-ui-contract` is the *component contract*: the `Component`, `Container` and `Collection` traits, the 16-step pipeline, styles and the test kit (`contract-api.md:1665-1760`; `CL/docs/authoring.md`). Components are Rust types shown with `Type::new(..).prop(..).show(ui)`. There is no JSON or tree schema that it renders.

**Nearest declarative pieces, in order of usefulness:**
1. **`ParamKind`, `Param`, `ParamDialog`, `ParamRows`, `HostedParamDialog`** (`CL/crates/ui-patterns/src/patterns/form_dialogs/param.rs:20-181`, `dialog.rs:785-860`).
   - Data-described form rows: Range (linear or log), Int, Number, Bool, Choice, Radio, Angle, Color, Grid, Point, Channel, Text, OptionalNumber, ReadOnlyText, Custom, Blend, Pattern, Files, Section.
   - Values sit in `DialogFields`. It supports remembered values, Preview, Reset, held OK and automation.
   - `Param::parse_list(spec)` parses the registry notation `{"radius":0.1..1000=1,"method":"spin|zoom",…}` (`param.rs:843-847`).
   - PhotoCraft upstream already runs plugin filter dialogs through it: `open_with_spec(app, "plugin.run", name, &m.params_notation(), fixed)` (`PC@up:…/menus/plugin_ui.rs:74`).
   - ext-core `form::ParamKind` (`EXT/ext-core/src/form.rs:84`) maps almost 1:1: Section, Range, Int, Number, Bool, Choice, Radio, Angle, Color, Point, Text.
2. **`PreferencesModel` `RowKind` rows** (preferences pattern).
3. **The access tree plus `craft-automation` `ui.inspect`**, as a read-back for tests.

**Conclusion:**
- Plugin `form` nodes and filter parameter forms should render through `ParamRows` or `HostedParamDialog`.
- Other plugin tree nodes need an interpreter in ext-ui that maps `Node` to framework components (§4.2). The framework will not interpret JSON itself.
- This matches the replacement rule: "Guests emit versioned UI data… Framework adoption is a host renderer change" (`craft-extensions/docs/plugins/suite-replacement-rule.md`).

---

## 3. How PhotoCraft upstream wires the framework, and where the plugin adapter hooks

### 3.1 Wiring map (`PC@up:`, all under `crates/ui-egui/src/` unless noted)

| Concern | Exact place |
|---|---|
| Dependency | `Cargo.toml:15` `craft` rev `a6b9d9c9…`; ui-egui features `ui, layers, color-ui, …, menu, keymap, theme-store` (`crates/ui-egui/Cargo.toml:13`), native adds `dialogs, update` (`:51`) |
| Entry | `apps/photocraft/src/main.rs:370` `main` calls `eframe::run_native` (`:531`); `services::native` (`:537-540`); `PhotocraftApp::setup_window` (`:564`); startup toasts `notices::post(.., Tone::Bad, None)` (`:730-739`) |
| Runtime install | `app/framework.rs:45` `install(ctx, fonts, frame)`: `InstallSpec::new(ShellInfo, SystemWindows)…` then `embed::install` (`:50-70`) |
| Host | `app/framework.rs:239` `build_host`: `Host::builder(AppInfo{id:"photocraft"}).i18n().settings(PrefSections).jobs(default_jobs())` |
| Framework state | `lib.rs:668` `app.framework: Framework { dialogs: DialogHost, catalog: Option<AppCatalog>, fired, suite: SuiteClient, … }` (`framework.rs:78-122`) |
| eframe impl | `app.rs:253-309`: `raw_input_hook` calls `framework_before_input` (`framework.rs:551`), which calls `embed::before_input`; `logic` calls `logic_frame` (`app/frame.rs:55`; `embed::logic` `:100`; `shortcuts::handle` `:117`) |
| Window | `app/frame.rs:232-251` `AppShell::new(WindowKey::main()).title_bar_if(..).options_bar(..).status(..).body(\|ui,cx,app\| { panel_docking::show(..); notices::show(..) }).show(ui, self)` |
| After the shell | `frame.rs:255-276`: `jobs_ui::dialog`, `updates_ui::show`, `dialogs::run` (`:259`), `canvas::extra_windows` (`:260`), `native_menu::sync` (`:276`) |
| Panel registry | `panels/registry.rs`: `PanelEntry` (`:20-41`), `static PANELS` (`:102-226`), `entry` `:244`, `normalize` `:249`, `registered` `:255`, `run_fired` `:276`, presets `:348-382` |
| Dock | `panels/panel_docking.rs`: `impl DockPanels<String> for Panels` (`:172-221`), `spec()` (`:149-169`), `show()` (`:238-297`, `DockArea::new("Panels", Bind::given(..))` at `:252`), commands `window.panel.*` (`:315-348`), `toggle_panel` (`:352`), load and `reconcile` (`:715-756`) |
| Layout persistence | `dock.rs`: `snapshot` `:14`, `persist` `:20` (to `prefs.panel_layout`), `restore` `:31` (called from `prefs/prefs_ui/startup.rs:34`) |
| Dialogs | `dialogs.rs`: `run` (`:488-499`) calls `app.framework.dialogs.run(ui)` then `take_results` (`:477`), which runs `PROMPTS` (`:401`) then `TAKERS` (`:436-472`); `open_param` (`:133-178`); custom `impl StackDialog for PrefsDialog` (`prefs/prefs_ui/dialogs.rs:148-196`) |
| Extra OS windows | `canvas/document_windows.rs:60-114` (`DocumentWindow::new(title).first_placed(rect)…show`) |
| Menus | `menus/items.rs:139` `base_rows`: CATALOG, then `UI_COMMANDS`, then engine specs, then **`plugin_ui::rows` (`:185`)**, then dynamic rows (`:221-300`). Also `build` (`:360-398`), `TOP_MENUS` (`menus.rs:85`: File…Filter, View, Window, Help), `is_live` (`items.rs:17`), cached by `native_menu::state_hash` (`native_menu.rs:81-120`) |
| Dispatch | `input/shortcut_dispatch.rs:550` `run_fired` calls `run_pressed` (`:635`), which tries `framework::run_command` over `COMMAND_SOURCES` (`framework.rs:514-525`), then `menus::invoke` (`menus/invoke.rs:72`), then `invoke_unguarded` (`:395`), which chains `plugin_ui::menu` (`:193`) |
| Keymap | `shortcut_dispatch.rs:221-316` `command_rows`, `:347` `catalog` (`AppCatalog::new` `:350`), `:362` `overrides`; rebuilt only on prefs rev (`framework.rs:554-564`) |
| Theme | `RootEnv` theme and mode in `frame_input` (`framework.rs:323-330`); `sync_theme` (`:367`); legacy `theme::Tokens::get` (`theme.rs:879`) |
| Toasts | `notices.rs`: `post(app, title, lines, Tone, dismiss_pref) -> u64` (`:50`), `error` (`:187`), `show` (`:212-252`, `NoticeCenter::new().timeout(..)`) |
| Existing plugin code | `menus/plugin_ui.rs` (Filter › Plug-ins rows `:25-49`, `menu` `:59-76`); engine `crates/engine/src/cmd/plugin_cmds.rs` (`plugin.run/install/remove/reload/list`); prefs `PlugIns` (`crates/engine/src/prefs.rs:1080-1087`); `docs/plugins.md` (WASM ABI v1) |

Upstream already has its own WASM filter plug-in system: `photocraft-plugins` and the `plugin.*` commands. Under the suite replacement rule it needs an explicit disposition (migrate, bridge, or retire), and its Filter › Plug-ins rows are the natural seam to reuse.

### 3.2 Hook points for the plugin adapter

**1. "Plugins" dock panel family**
- Add a runtime registry beside `PANELS`, for example `PluginPanels { by_id: BTreeMap<String, PluginPanel> }` held in `PhotocraftApp`, with ids `plugin.<pluginId>.<panelId>`.
- Extend `registry::entry`, `registered` and `normalize` (`registry.rs:244-256`) to consult it. `entry` returns `&'static PanelEntry`, so either return an enum or add a parallel lookup.
- Extend `Panels::spec`, `body`, `content` and `panel_rows` (`panel_docking.rs:149-227`):
  - `spec`: `PanelSpec::new(Label::Shared(name)).icon(..).limits(..).allow_native(true)`.
  - `body`: the ext-ui renderer.
  - `content`: the surface revision combined with a framework-state hash. Without this, settled bodies show stale trees.
- Populate the registry **before** `dock::restore` (`prefs_ui/startup.rs:34`). Otherwise saved plugin panels go to `hidden`. That is recoverable, but they would only reappear if the code that re-adds hidden panels on registration runs. Add that step: when the registry changes, call `reconcile` again on the live workspace.
- Add a `CommandSource { states, run }` entry for plugin panel commands (`framework.rs:514`).

**2. Window › plugin panels.** Inject rows in `base_rows` next to `plugin_ui::rows` (`items.rs:185`):
```rust
Row::new(&["Window", "Plugins"], format!("window.panel.{id}").as_str(), name).kind(ItemKind::Check(..))
```
- `toggle_panel` (`panel_docking.rs:352`) already resolves `window.panel.<id>` once `normalize` knows the id.
- Add plugin registry and checked state to `state_hash` (`native_menu.rs:81-120`). Today the registry is not hashed, so a background install does not refresh the menu until the next input.

**3. Plugin windows**
- (a) Hosted: one `PluginDialog: StackDialog`, with `id()` set to `dialog_id("plugin.<id>.<surface>")` and `instance` per surface, and with `json_fields`. Open it with `app.framework.dialogs.open(..)` and collect results with a new `TAKERS` entry (`dialogs.rs:436`).
- (b) Real OS window: `ViewportWindow::new(title).remember(SaveKey)` drawn each pass from `ui_body` next to `canvas::extra_windows` (`frame.rs:260`). Close it when `close_requested`. Alternatively use a floating plugin panel with `allow_native(true)`.
- Replace the plain `egui::Window` used in `PC/plugins_ui.rs:71`.

**4. Plugin Manager**
- Add a `plugins.manager` command that opens a `ViewportWindow` or `StackDialog` hosting the shared manager component (§4.3).
- Keep Edit › Preferences › Plug-ins (`edit.preferences.plugIns`, `menu_catalog.rs:168`) as the settings page. Its "Manage…" button fires the same command.

**5. Menus**
- Filter › plugin filters: generalise `plugin_ui::{rows, is_enabled, menu}`, which are already wired at `items.rs:185`, `menus/state.rs:48` and `invoke.rs:193`. Use the `params_notation` route through `HostedParamDialog` for parameter forms.
- A top-level "Plugins" menu needs a new entry in `TOP_MENUS` (`menus.rs:85`, a fixed-size array) and a mnemonic in `bar.rs:9`.
- Mark new ids live in `is_live` (`items.rs:17`).
- For shortcuts, add plugin `CommandDef`s in `command_rows` (`shortcut_dispatch.rs:221`). Bump the catalog generation on plugin registry change, not only on prefs rev (`framework.rs:554-564`).

**6. Toasts.** Use `photocraft_ui_egui::notices::post(app, title, lines, Tone::…, None)`. It is public, works outside a pass, and keeps the "dismiss pref" plus sticky semantics. In shared ext-ui code use `craft::ui::runtime::toast::post(ctx, &WindowKey::main(), Toast::new(..).tone(..))`.

**7. Plugin tools.** Wrap the plugin tool pointer protocol (`PC/plugins_ui.rs:195`) in a `CanvasTool`. Paint overlays through a framework canvas control. Do not draw raw `paint_ops` into the canvas painter.

---

## 4. Recommendations for `craft-extensions/crates/ext-ui`

### 4.1 Dependency strategy: you cannot simply add `craft` to ext-ui

Facts that constrain the choice:
- **ext-ui today** depends on `egui = "0.37"` with `default-features=false`, plus `craft-ext-core` and `craft-ext-store` (`EXT/ext-ui/Cargo.toml`). Its `framework` feature is an empty marker.
- **The framework** pins `storytold/egui @ e9711ca` and *replaces* the old `craft-ui` crate (`craft-framework.md:850-856`). It has no deprecated aliases.
- **Other apps' pins:**
  - EffectCraft, FigCraft and the launcher pin old craft-libs revs and `storytold/egui @ 6d518fc` (`craft-launcher/Cargo.toml:74-75`).
  - plugin-beta PhotoCraft also pins old `craft-ui @ fe6b443` (`photocraft/Cargo.toml:20`). Only `refs/upstream-ro/main` is on the framework.
- **Consequence:** a crate that depends on `craft` cannot be linked into EffectCraft, FigCraft or the launcher until they port. Two egui revisions or two `craft-ui`s in one build would break.

**Recommended layout** (one renderer per toolkit, one model for all):
1. **`craft-ext-ui-model`** (new, no egui), or move these into ext-core:
   - `helpers.rs` view models: `status_text`, `compatibility`, `permission_sheet`, `sheet_model`, `PermissionChoice`.
   - A new egui-free `ManagerModel`: tabs, query, filters, selected plugin, job state, planned install, setup consent, update list, problems. Its `apply(ManagerAction)` returns effects.
   - `ManagerAction` and `ManagerWords` (the English word table).
   - Tree validation and diff stay in ext-core.
   - This is where all manager *logic* lives, so both UIs share it and it is unit-testable.
2. **`craft-ext-ui`** (existing, egui 0.37): the legacy egui renderer and legacy `ManagerView`, rewritten as thin views over `ManagerModel`. It serves EffectCraft, FigCraft and the launcher in the interim. Fix the token violations now:
   - `Color32::RED` at `lib.rs:150` becomes `ui.visuals().error_fg_color`.
   - Route all manager strings through `Host::tr`.
3. **`craft-ext-ui-fw`** (new crate, or a `framework = ["dep:craft"]` feature on ext-ui) with `craft = { git = storytold/craft-libs, rev = <same rev as the app>, features = ["ui"] }`, plus the optional `color-ui` feature for colour nodes. It contains:
   - `FrameworkEngine: plugin_ui::Engine`. This plugs into the existing seam `PluginUi::set_framework_engine` (`EXT/ext-ui/src/plugin_ui/mod.rs:132`), so trees, revisions and events keep the same contract.
   - `PluginManager`: the framework pattern for the manager (§4.3).
   - `PluginPanelBody`, `PluginWindow` (a `ViewportWindow` wrapper) and `PluginDialog: StackDialog` helpers.
   - A rev-lockstep check: assert in an app xtask that the app's `craft` rev equals the one `craft-ext-ui-fw` was built for, mirroring `craft::egui_fork::check_lockfile`.
   - Only apps on the framework (PhotoCraft upstream now, VectorCraft next, then others) enable it.
   - Note on the feature-flag option: with git deps, any mismatch in craft rev between app and ext-ui creates two `craft-ui` packages. Keep one rev constant per craft-extensions release, or have apps `[patch]` it.
4. **Later:** once every app is on the framework, delete `EguiEngine`, the legacy `ManagerView` and `EngineChoice`. At that point `EngineChoice` is a temporary bridge, and per the replacement rule it needs an owner and removal criteria.

### 4.2 Framework renderer: mapping plugin nodes to components

**Node mapping.** Node kinds are defined at `EXT/ext-core/src/ui/tree.rs:157-218`.

| Node | Framework component | Event mapping (keep the current `UiValue` contract) |
|---|---|---|
| column, row, grid | `Stack`, `Row`, `Grid` | – |
| group | `GroupBox` (titled) | – |
| collapsible | `Section` with `open` bind | `Bool` on toggle |
| scroll | the container's scroll body or `Stack` scroll; children in `Lazy` | – |
| label (TextStyle) | `Text` with type role: Body→Body, Heading→Heading, Strong→Body strong, Small→Caption, Weak→muted ink, Mono→Mono | – |
| separator, spacer | `Divider`; spacer becomes a gap role (no pixel spacer) | – |
| button (icon_only) | `Button` / `IconButton`, tooltip from `hint` | `Click` on `clicked()` |
| checkbox, toggle | `Checkbox`, `Switch` | `Bool` on commit |
| slider | `SliderField` (slider and number) or `Slider` | `Number` on `EditEvent::Commit` (matches today's send-on-end) |
| number | `NumberField` | `Number` on commit |
| text-input, text-area | `TextField` / `TextField.lines(n)` | `Text` on commit (Enter or blur) |
| select, radio | `Select` with `Entry` list, `RadioGroup` | `Index` |
| color | `Swatch` opening `ColorPopover` (color-ui) | `Color` on commit |
| tabs | `TabStrip` | `Text(tab id)` |
| list | `List` + `ListRow` (`.select(SelectMode::Single)` for single; `writer-notes.md:16`), tree via depth | `Keys` on selection, `Activate` on Enter or double click |
| table | `Table` with `ColumnSpec` | as list |
| progress | `Progress` / `Spinner` (indeterminate) | – |
| icon, image | `Icon` (`IconSpec`), `Picture` | – |
| form | `ParamRows` over `Param`s from `form::ParamKind` | keys of `DialogFields` |
| canvas | new allowlisted `ContentPainter` component, e.g. `PluginCanvas` (**framework gap**), plus a `CanvasTool` for tool surfaces | `Pointer(..)` events |
| component | `Host::component` as today; framework engine resolves known names and versions to library components | as declared |
| unknown | its `fallback`, else a `Notice` with `Tone::Warn` | – |

**Rules the renderer must follow:**
- Plugin text is untranslated, so pass it as `Label::Shared(Arc<str>)`. Host words come from a `words!` set through `Env::words`.
- Each node gets a stable id salt of `(surface key, node id)`. Use the plugin's `id` as the automation key and the accessible name. The access tree replaces today's `Host::widget` rect callback: keep the callback for one release, then drop it.
- Emit events only from output edges (`clicked()`, `EditEvent::Commit`, the selection-change event). Never compare against last pass's value.
- Every ad hoc constant in `egui_engine.rs` moves to token roles. That covers `default_width(280)` (`mod.rs:190`), list gaps of 4, 6 and 8, row height 240, radius 2 and 3, and the toggle geometry.
  - Bounds that are safety limits (clamps on `MAX_NODES`, 8192 and so on) stay as validation in ext-core, not as layout.
- `paint_ops`: its colours and strokes are plugin *data*. Inside a framework component they must go through `DataColor` and the content painter. `Draw` accepts only `StyleColor` or `DataColor` (`contract-api.md:3894`).

**Additive tree fields (UI level 1.2).** Plugins can only express meaning if the tree carries it. Add, without breaking 1.0 or 1.1:
- `button.variant: "primary"|"secondary"|"danger"|"ghost"`
- `label.tone` and `badge` nodes with `tone: neutral|accent|info|good|warn|bad`
- `notice` node (title, lines, tone)
- `empty` text on list and table
- `text-input.search: bool`
- `section.collapsible`

All of these map directly to `Variant` and `Tone`. Never accept colours or pixel sizes in the tree. Validation refuses unknown values and falls back.

### 4.3 Plugin Manager and store as a reusable framework component

**Where it lives.** It is a pattern-level assembly, so it lives in `craft-ext-ui-fw` as `PluginManager<'a>`. In time it could move to craft-libs `patterns` only if the store model becomes a craft-libs crate. craft-libs may not depend on craft-extensions (`CL/AGENTS.md`: "No app code").

**Shape**, following framework idioms (`Type::new(required).option(..).show(ui)`; panels take `&mut Model`):
```rust
PluginManager::new(Bind::bound(&mut model))        // ManagerModel from craft-ext-ui-model
    .app(AppScope::App(app_id) | AppScope::Launcher(factory))
    .problems(|ui, cx| { … })                        // app-owned rows, e.g. paused plugins + Resume
    .show(ui) -> PluginManagerOutput { actions: Vec<ManagerAction>, .. }
```
- **Body:** `TabStrip` (Browse, Installed, Updates, Problems) over `List` and `ListRow` (icon, name, detail line, trailing `Badge`s) with `CollectionProps::empty(...)`, plus a detail column of `Section`s. Toolbar: `TextField::search`, a kind `Select`, `Switch`es, Refresh `IconButton`.
- **Install, setup and consent:** `StackDialog`s, so automation can drive them:
  - `PluginSetupDialog`, with `Checkbox` rows per contribution and permission, the native "full access" row as a `Notice` with `Tone::Bad`, and footer `[Install (Accept), Cancel]`.
  - `ProjectPrompt` becomes a `StandardPrompt::custom` or its own `StackDialog`.
- **Work:** submit catalogue, install and update jobs to `Host::of(ui).jobs()` or a `JobQueue` keyed by plugin id, instead of `Jobs`' own thread (`EXT/ext-ui/src/lib.rs:34-64`). Show progress with `Progress` or `StatusProgress`, with a Cancel that drops the key.
- **Results:** completion and errors go out as `Toast`s (`runtime::toast::post`) and Problems rows.
- **Words:** a `words!` set (`ManagerWords`) with English rows, translated through craft-i18n. The current manager has no `tr` at all.

**Where each app hosts it:**

| App | Host surface |
|---|---|
| PhotoCraft (upstream) | `ViewportWindow` opened by `plugins.manager`; Preferences page links to it |
| Launcher (after port) | `ViewportWindow` with `AppScope::Launcher` |
| EffectCraft, FigCraft, launcher (interim) | legacy egui `ManagerView` over the **same** `ManagerModel` and actions, so behaviour is identical. EffectCraft keeps it in Settings › Plugins (`effectcraft/crates/ui-egui/src/plugins_ui.rs:133`); FigCraft in its window (`figcraft/…/plugins_ui.rs:134`); the launcher in its modal (`craft-launcher/…/dialogs.rs:47`) |

**Docked plugin panels in the interim:**
- FigCraft has its own dock (`figcraft/crates/ui-egui/src/plugin_dock.rs:304`) and EffectCraft has older craft-ui docking.
- Keep panel bodies behind `PluginUi::draw` and the `Surface` contract, so the switch to `DockPanels<String>` later is a host change only.
- Use the same id scheme `plugin.<pluginId>.<panelId>` now, so saved layouts carry over.

**Unify the interim `Host` impls.** They differ today: PhotoCraft's is empty, EffectCraft wires `widget` and `tr`, FigCraft wires `tr` and `form_titles`. All apps should at least implement `tr`.

### 4.4 Framework gaps to raise on craft-libs `next`

These are the "gap is library work" items (`craft-framework.md:705`):
1. Runtime icon registration with owned SVG bytes for plugin icons, scoped and removable. Today registration needs `&'static AppIcons` (`svg_set.rs:34`).
2. A sanctioned `ContentPainter` component for plugin canvases: a painter of declarative draw ops with data colours, pointer and key events, and `canvasView` support.
3. Optional: a `Workspace` helper "registry changed: re-place hidden panels", which is `reconcile` run on the live workspace without a default reset. Check first whether `reconcile` alone suffices.
4. Optional: `ParamKind` extensions if ext-core form kinds need something missing, such as int-with-unit.

---

## 5. Rules and conventions plugin code must follow

### Naming and ids
- Components follow `Type::new(required).option(..).show(ui)`. No inherent `show`, no `allocate_*`, `ui.interact` or `ui.put` (`authoring.md:68-108`).
- Command ids are dotted lowerCamel strings, for example `window.panel.layers`, `canvasView.zoomIn`, `edit.preferences.plugIns`. Plugin ids should be `plugin.<id>.<thing>`.
- Dialog ids come from `dialog_id("app.thing")`.
- `DocKey` is a crate-prefixed `"craft-ui.find-field"` (`contract-api.md:441-447`).
- New crates are named `craft-<name>`, use `[lints] workspace = true`, and carry the `deny(clippy::unwrap_used, expect_used, panic, unimplemented, todo, unreachable)` header (`CL/AGENTS.md`; house rule Rust 1).

### No hardcoded look
- "Apps ask for meaning", never a colour or pixel size (`craft-framework.md:95`). Use variants, tones, type roles and size roles.
- Design lint rules are `literal-size/*`, `raw-colour/const`, `style-mutation` and `tooltip/stock`. An exemption needs `// design: allow(reason)` (`:686-691`).
- No component-specific theme values. One layout in every theme and density (`KIT/rules/porting-rules.md:13-17`).
- egui `add_enabled_ui` is an error. Use the component's `enabled` prop (`KIT/rules/lints.toml:10-14`).

### i18n
- Library words come from craft-i18n `common.tsv`. App labels win, and a missing translation never blocks (`craft-framework.md:624-630`).
- Host words come from a `words!` set read through `Env::words`. Plugin-supplied text is passed through untranslated, as `Label::Shared`.
- Use `Env::format`, `dates`, `units`, `relative_age`, `folder` and `collator` for numbers, dates, search and sorting.

### Accessibility (required, `craft-framework.md:604-622`)
Each of these is needed for every component:
- keyboard operation and a visible focus ring
- AccessKit roles, names and values
- RTL mirroring (sliders and number fields never mirror)
- reduced motion
- the edit lifecycle Begin/Update/Commit/Cancel, where Escape restores the value
- one undo step per drag

Keyboard conventions:
- In dialogs, Tab visits fields only. A tab strip or palette is one Tab stop with arrows inside.
- Dialog footer order follows the platform: OK last on macOS and Linux, first on Windows.

### Pass model
- A frame can run two passes. Input reaches only the first, and no state carries over between passes except the framework's three mechanisms.
- Drawing code changes no state and sends nothing. The suite's `measure-pure` case enforces this (`authoring.md:154-156`).
- Act on edges only.

### Services
- Background work goes on `Host` `Jobs` or `JobQueue`.
- Files go through `craft_host::Files` and `PickRequest`. Native dialogs use the non-blocking `craft-dialogs`, never a blocking `rfd` (`CL/crates/dialogs/src/lib.rs:1-15`).
- Settings go through `SectionStore` or `craft-settings`, secrets through the keychain.

### Tests (`craft-framework.md:638-684`)
- Every test names the regression it guards. Do not test wiring, defaults, exact strings or sizes.
- Framework-side code uses `craft_ui::testing::{Fixture, story, trial}`. A new component registers stories for its required states: Default, plus Mixed and Invalid for values, LongText for text, Open for overlays, and Empty, One and Many for items (`authoring.md:140-156`).
- A family-style crate (as `craft-ext-ui-fw` would be) runs `craft_ui::testing::run` from a `harness = false` lib test (`authoring.md:26-56`).
- Trials go through `Fixture::host`, never `Host::install`, and every frame adds 1/60 s (`KIT/rules/writer-notes.md:13`).
- The app smoke trial uses in-memory settings (`porting-rules.md:67`).
- For PhotoCraft-level checks use the automation control channel: `ui.dialog.*`, `ui.inspect`, `ui.screenshot` (`CL/docs/porting.md:124-201`).

### Process
- A framework gap is fixed in craft-libs `next` first, never with an app workaround.
- A fix that adds a mechanism removes the one it replaces in the same change.
- Use the framework's signals, never inferred side effects (`porting.md:256-267`; `porting-rules.md:32-48`).
- Check the gallery (`components-index.md`) before writing any component. Register new components or variants with stories in the same change (`porting-rules.md:50-56`).

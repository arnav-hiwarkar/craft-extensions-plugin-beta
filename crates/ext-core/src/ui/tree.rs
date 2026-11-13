//! The plugin UI vocabulary, level 1.1 ([`UI_LEVEL`]): the tree a plugin sends with
//! `ui.panel.set` / `ui.window.open`, the canvas draw list and the events that come back.
//!
//! This is a JSON superset of level 1.0 ([`super::Node`]): every 1.0 tree parses here unchanged and
//! validates under the same rules. Hosts draw it with the shared renderer
//! (`craft_ext_ui::plugin_ui`); plugins never choose or detect the engine that draws it.
//!
//! Invariants:
//! - Bounded: node count, depth, draw ops, path points, table cells, images and text are capped
//!   ([`limits`]); [`Node::validate`] refuses a tree before anything is drawn.
//! - Every interactive control has a stable `id` and a visible label.
//! - Forward compatible: a node whose `type` this build does not know parses to [`Node::Unknown`]
//!   and is drawn as a visible notice (or its `fallback`), never a parse failure of the whole tree.
//! - Versioned: [`Node::level`] says which UI level a tree needs. A plugin that uses level-1.1
//!   vocabulary declares the session contract minor that carries it (see [`ui_level_for_session`]),
//!   so an older app says "update the app" before any plugin code runs.

use crate::contract::Level;
use crate::form::{Form, Values};
use crate::text::{Lines, validate_text};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub use super::UiError;

/// The UI level this build understands.
pub const UI_LEVEL: Level = Level::new(1, 1);
/// The first UI level: the vocabulary of [`super::Node`].
pub const UI_LEVEL_1_0: Level = Level::new(1, 0);

/// Which UI level a session contract carries. `<app>.session@1.0` carries ui 1.0; a host that draws
/// with the shared renderer provides `<app>.session@1.1` or later, which carries ui 1.1.
pub fn ui_level_for_session(session: Level) -> Level {
    if session.major == 1 && session.minor == 0 { UI_LEVEL_1_0 } else { UI_LEVEL }
}

/// Every limit a tree is checked against.
pub mod limits {
    pub const MAX_NODES: usize = super::super::MAX_NODES;
    pub const MAX_DEPTH: usize = super::super::MAX_DEPTH;
    pub const MAX_DRAW_OPS: usize = super::super::MAX_DRAW_OPS;
    pub const MAX_COMPONENT_PROPS_BYTES: usize = super::super::MAX_COMPONENT_PROPS_BYTES;
    /// Bytes of an id (widget, tab, row or image name).
    pub const MAX_ID: usize = 64;
    /// Characters of a control label, tab, column or option.
    pub const MAX_LABEL: usize = 200;
    /// Characters of a hint (tooltip) or placeholder.
    pub const MAX_HINT: usize = 1000;
    /// Characters of static text (a label node, a table cell, canvas text).
    pub const MAX_TEXT: usize = 4000;
    /// Characters of a text area's value.
    pub const MAX_TEXT_AREA: usize = 65_536;
    /// Options of a select or radio group.
    pub const MAX_OPTIONS: usize = 256;
    pub const MAX_TABS: usize = 32;
    pub const MAX_GRID_COLUMNS: u32 = 16;
    pub const MAX_TABLE_COLUMNS: usize = 32;
    pub const MAX_ROWS: usize = 10_000;
    /// Table cells (rows × columns) in one tree.
    pub const MAX_CELLS: usize = 100_000;
    /// Path commands of one path op, or points of one polygon.
    pub const MAX_PATH_POINTS: usize = 4_096;
    /// Path commands and polygon points over every canvas in the tree.
    pub const MAX_TREE_POINTS: usize = 100_000;
    pub const MAX_GRADIENT_STOPS: usize = 16;
    /// Side of an image, in pixels.
    pub const MAX_IMAGE_SIDE: u32 = 2048;
    /// Named images in one tree.
    pub const MAX_IMAGES: usize = 64;
    /// RGBA bytes of every image in one tree.
    pub const MAX_IMAGE_BYTES: usize = 64 * 1024 * 1024;
    /// Side of a canvas, in points.
    pub const MAX_CANVAS_SIDE: f32 = 8192.0;
    pub const MAX_STROKE_WIDTH: f32 = 256.0;
    pub const MAX_FONT_SIZE: f32 = 256.0;
    pub const MAX_ICON_SIZE: f32 = 256.0;
    pub const MAX_SPACER: f32 = 512.0;
    /// Rows a text area asks for.
    pub const MAX_TEXT_ROWS: u32 = 50;
    /// Decimals a number input shows.
    pub const MAX_DECIMALS: u32 = 10;
    /// A scroll area's or table's height, in points.
    pub const MAX_HEIGHT: f32 = 8192.0;
}
use limits::*;

/// The built-in icon set: names only, drawn by the host in its own style. The names are the
/// meaning-based names of the craft-framework icon catalogue, so a framework engine draws the same
/// icon. An unknown name draws a neutral placeholder, never an error.
pub const ICONS: &[&str] = &[
    "brush",
    "check",
    "chevron_down",
    "chevron_left",
    "chevron_right",
    "chevron_up",
    "circle",
    "clock",
    "close",
    "copy",
    "crop",
    "error",
    "eye",
    "eye_off",
    "eyedropper",
    "file",
    "flag",
    "folder",
    "funnel",
    "grid",
    "hand",
    "home",
    "image",
    "info",
    "layers",
    "link",
    "lock",
    "lock_open",
    "menu",
    "minus",
    "moon",
    "more",
    "move",
    "palette",
    "pause",
    "pencil",
    "play",
    "plus",
    "redo",
    "refresh",
    "save",
    "search",
    "settings",
    "sort_ascending",
    "sort_descending",
    "square",
    "star",
    "success",
    "sun",
    "text",
    "trash",
    "triangle",
    "undo",
    "warning",
    "zoom_in",
];

fn is_false(b: &bool) -> bool {
    !*b
}
fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    *t == T::default()
}

/// One node of a plugin UI tree, tagged by `"type"` in kebab-case.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Node {
    // Level 1.0.
    /// A host-drawn form (schema 1); widget events use the parameter keys.
    Form(FormNode),
    /// A named, versioned framework component, or its fallback.
    Component(Component),
    Column(Stack),
    Row(Stack),
    Group(Group),
    Label(Label),
    Separator,
    Button(Button),
    Checkbox(Check),
    Slider(Slider),
    TextInput(TextInput),
    Select(Select),
    Color(ColorInput),
    Image(Image),
    Canvas(Canvas),
    // Level 1.1.
    Tabs(Tabs),
    Collapsible(Collapsible),
    Scroll(Scroll),
    Grid(Grid),
    Table(Table),
    List(List),
    Number(NumberInput),
    TextArea(TextArea),
    Progress(Progress),
    Toggle(Check),
    Radio(Radio),
    Icon(Icon),
    Spacer(Spacer),
    /// A node from a newer UI level, or a known node with invalid fields (refused by `validate`).
    #[serde(untagged)]
    Unknown(Unknown),
}

/// Node types of level 1.0.
pub const KINDS_1_0: &[&str] =
    &["form", "component", "column", "row", "group", "label", "separator", "button", "checkbox", "slider", "text-input", "select", "color", "image", "canvas"];
/// Node types added in level 1.1.
pub const KINDS_1_1: &[&str] =
    &["tabs", "collapsible", "scroll", "grid", "table", "list", "number", "text-area", "progress", "toggle", "radio", "icon", "spacer"];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormNode {
    pub form: Form,
    pub values: Values,
    /// Parameters hidden by the plugin (as a `hide` patch would).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden: Vec<String>,
    /// Parameters disabled by the plugin (as a `disable` patch would).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disabled: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub key: String,
    pub name: String,
    pub version: u32,
    pub props: Value,
    pub fallback: Box<Node>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stack {
    pub children: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub label: String,
    pub children: Vec<Node>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextStyle {
    #[default]
    Body,
    Heading,
    Strong,
    Small,
    Weak,
    Mono,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Label {
    pub text: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub style: TextStyle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Button {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Draw only the icon; the label becomes the tooltip and accessible name. Needs `icon`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub icon_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

/// A checkbox or (as `toggle`) a switch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub id: String,
    pub label: String,
    pub value: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slider {
    pub id: String,
    pub label: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextInput {
    pub id: String,
    pub label: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Select {
    pub id: String,
    pub label: String,
    pub options: Vec<String>,
    pub selected: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorInput {
    pub id: String,
    pub label: String,
    pub rgba: [f32; 4],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

/// An RGBA8 image (`width * height * 4` bytes, as a byte array). With a `name` it can be drawn on a
/// canvas of the same tree (`image` draw op); `hidden` registers it without showing it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
    /// Display size in points (default: the pixel size).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<[f32; 2]>,
    /// Alternative text (tooltip and accessible name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
}

/// Which extra canvas events the plugin wants. Down, drag and up always arrive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Listen {
    /// `move` while the pointer is over the canvas with no button held, and `leave`.
    Move,
    /// `wheel` with the scroll delta in points.
    Wheel,
    /// `double-click`.
    DoubleClick,
    /// `key-down` / `key-up` while the canvas has keyboard focus (a click gives it focus; Escape
    /// releases it).
    Keys,
}

/// A cursor hint while the pointer is over a canvas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cursor {
    #[default]
    Default,
    Pointer,
    Crosshair,
    Text,
    Move,
    Grab,
    Grabbing,
    NotAllowed,
    ResizeHorizontal,
    ResizeVertical,
    ResizeNwse,
    ResizeNesw,
    ZoomIn,
    ZoomOut,
}

/// A custom drawing area. Pointer events arrive with this id, in canvas coordinates (points from
/// the top-left corner).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Canvas {
    pub id: String,
    pub label: String,
    pub width: f32,
    pub height: f32,
    pub ops: Vec<DrawOp>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub listen: Vec<Listen>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Tabs; switching sends the tab id as text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tabs {
    pub id: String,
    pub tabs: Vec<Tab>,
    /// The tab shown (default: the first).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tab {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub children: Vec<Node>,
}

/// A section that opens and closes; toggling sends a bool.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collapsible {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub open: bool,
    pub children: Vec<Node>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scroll {
    pub children: Vec<Node>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<f32>,
    /// Also scroll sideways.
    #[serde(default, skip_serializing_if = "is_false")]
    pub horizontal: bool,
}

/// Children laid out row by row in `columns` aligned columns.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    pub columns: u32,
    pub children: Vec<Node>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub striped: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Selection {
    #[default]
    None,
    Single,
    Multi,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VAlign {
    #[default]
    Top,
    Center,
    Bottom,
    Baseline,
}

/// A table of text cells. Selection changes send the selected row keys (`["a","c"]`); a double
/// click or Enter on a row sends `{"activate": key}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub id: String,
    pub label: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub selection: Selection,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Column {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub align: Align,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub key: String,
    pub cells: Vec<String>,
}

/// A list of items with the same selection events as [`Table`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct List {
    pub id: String,
    pub label: String,
    pub items: Vec<Item>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub selection: Selection,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub key: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// A number field the person can drag or type into; sends a number when the edit ends.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumberInput {
    pub id: String,
    pub label: String,
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// How far one drag step or arrow key moves the value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    /// Digits after the decimal mark; 0 for whole numbers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u32>,
    /// A short unit shown after the number (`px`, `%`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

/// Multi-line text; sends the text when the field loses focus or on Ctrl/Cmd+Enter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextArea {
    pub id: String,
    pub label: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub mono: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

/// A progress bar: `value` in 0..=1, or `None` for "working" with no known end.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Radio buttons; sends the chosen index (as `select` does).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Radio {
    pub id: String,
    pub label: String,
    pub options: Vec<String>,
    pub selected: u32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub horizontal: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
}

/// An icon from [`ICONS`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Icon {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<f32>,
    /// Tooltip and accessible name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Tint (default: the theme's text colour).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rgba: Option<[f32; 4]>,
}

/// Empty space along the layout direction, in points.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spacer {
    pub size: f32,
}

/// A node this build does not know. With a `fallback` node the host draws that; otherwise a notice
/// that the panel needs a newer version of the app.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Unknown {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Box<Node>>,
    /// The node's other fields, kept only to explain why a known node type did not parse.
    #[serde(flatten, skip_serializing)]
    pub rest: Map<String, Value>,
}

// ---------------------------------------------------------------------------------------------
// Canvas draw list.

/// A canvas draw operation, tagged by `"op"`. Coordinates are canvas points from the top-left;
/// colours are straight RGBA in 0..=1.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum DrawOp {
    // Level 1.0 (fields added in 1.1 are optional).
    Line(LineOp),
    Rect(RectOp),
    Circle(CircleOp),
    Text(TextOp),
    // Level 1.1.
    Path(PathOp),
    Polygon(PolygonOp),
    Image(ImageOp),
    /// Clips every later op to this rectangle (within the canvas) until `reset-clip`.
    Clip(ClipOp),
    ResetClip,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineOp {
    pub from: [f32; 2],
    pub to: [f32; 2],
    pub width: f32,
    pub rgba: [f32; 4],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RectOp {
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub fill: Paint,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<[f32; 4]>,
    /// Outline width (default 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
    /// Corner radius.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CircleOp {
    pub center: [f32; 2],
    pub radius: f32,
    pub fill: Paint,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<[f32; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextOp {
    pub at: [f32; 2],
    pub text: String,
    pub size: f32,
    pub rgba: [f32; 4],
    /// Which side of the text `at` is on, horizontally (default left).
    #[serde(default, skip_serializing_if = "is_default")]
    pub align: Align,
    /// …and vertically (default top).
    #[serde(default, skip_serializing_if = "is_default")]
    pub valign: VAlign,
    #[serde(default, skip_serializing_if = "is_false")]
    pub mono: bool,
}

/// A path of subpaths. Each subpath is filled on its own (no holes); the stroke follows every
/// segment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathOp {
    pub path: Vec<PathCmd>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<Paint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<[f32; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
}

/// `{"move":[x,y]}`, `{"line":[x,y]}`, `{"quad":[[cx,cy],[x,y]]}`,
/// `{"cubic":[[c1x,c1y],[c2x,c2y],[x,y]]}` or `"close"`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PathCmd {
    Move([f32; 2]),
    Line([f32; 2]),
    Quad([[f32; 2]; 2]),
    Cubic([[f32; 2]; 3]),
    Close,
}

/// A closed polygon (convex or not, edges must not cross).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolygonOp {
    pub points: Vec<[f32; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<Paint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<[f32; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
}

/// Draws the image named `image` (an `image` node with that `name` in the same tree) into a
/// rectangle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageOp {
    pub image: String,
    pub min: [f32; 2],
    pub max: [f32; 2],
    /// Source rectangle in 0..=1 texture coordinates (default the whole image).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uv: Option<[[f32; 2]; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tint: Option<[f32; 4]>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipOp {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

/// A fill: a colour (`[r,g,b,a]`) or a gradient object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Paint {
    Solid([f32; 4]),
    Gradient(Gradient),
}

/// `{"gradient":"linear","from":[x,y],"to":[x,y],"stops":[…]}` or
/// `{"gradient":"radial","center":[x,y],"radius":r,"stops":[…]}`, in canvas coordinates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "gradient", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Gradient {
    Linear { from: [f32; 2], to: [f32; 2], stops: Vec<Stop> },
    Radial { center: [f32; 2], radius: f32, stops: Vec<Stop> },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stop {
    /// Position along the gradient, 0..=1, not decreasing from one stop to the next.
    pub at: f32,
    pub rgba: [f32; 4],
}

impl Gradient {
    pub fn stops(&self) -> &[Stop] {
        match self {
            Gradient::Linear { stops, .. } | Gradient::Radial { stops, .. } => stops,
        }
    }

    /// The colour at canvas point `p`.
    pub fn sample(&self, p: [f32; 2]) -> [f32; 4] {
        let t = match self {
            Gradient::Linear { from, to, .. } => {
                let d = [to[0] - from[0], to[1] - from[1]];
                let len2 = d[0] * d[0] + d[1] * d[1];
                if len2 <= f32::EPSILON { 0.0 } else { ((p[0] - from[0]) * d[0] + (p[1] - from[1]) * d[1]) / len2 }
            }
            Gradient::Radial { center, radius, .. } => {
                let (dx, dy) = (p[0] - center[0], p[1] - center[1]);
                if *radius <= f32::EPSILON { 1.0 } else { (dx * dx + dy * dy).sqrt() / radius }
            }
        };
        sample_stops(self.stops(), t)
    }
}

fn sample_stops(stops: &[Stop], t: f32) -> [f32; 4] {
    let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
    let (Some(first), Some(last)) = (stops.first(), stops.last()) else { return [0.0; 4] };
    if t <= first.at {
        return first.rgba;
    }
    for w in stops.windows(2) {
        if let [a, b] = w
            && t <= b.at
        {
            let span = b.at - a.at;
            let k = if span <= f32::EPSILON { 1.0 } else { (t - a.at) / span };
            return std::array::from_fn(|i| a.rgba[i] + (b.rgba[i] - a.rgba[i]) * k);
        }
    }
    last.rgba
}

// ---------------------------------------------------------------------------------------------
// Events.

/// A widget event, sent to the plugin as `{"event":"ui", surface, rev, widget, value}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiEvent {
    /// Panel or window id the tree belongs to.
    pub surface: String,
    /// The tree revision that was drawn; the host drops events whose revision is no longer current.
    pub rev: u64,
    /// Widget id, or form parameter key.
    pub widget: String,
    pub value: UiValue,
}

impl UiEvent {
    /// Whether this event belongs to an older tree than `current` and must be dropped.
    pub fn is_stale(&self, current_rev: u64) -> bool {
        self.rev != current_rev
    }
}

/// What a widget sends. JSON shapes: `null` (click), bool, number, string, `[r,g,b,a]`, `[x,y]`,
/// `["key",…]` (selection), `{"activate":key}` or a pointer object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UiValue {
    Click,
    Bool(bool),
    Number(f64),
    Text(String),
    Index(u32),
    Color([f32; 4]),
    Point([f64; 2]),
    Keys(Vec<String>),
    Activate { activate: String },
    Pointer(PointerEvent),
}

/// A canvas pointer, wheel or key event. `at` is in canvas coordinates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PointerEvent {
    pub kind: PointerKind,
    pub at: [f32; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub button: Option<PointerButton>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub mods: Mods,
    /// Wheel delta in points (`wheel`), or the pointer movement since the last event (`drag`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<[f32; 2]>,
    /// Key name (`key-down` / `key-up`), as egui names keys: `A`, `Enter`, `ArrowLeft`, `Escape`…
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PointerKind {
    Down,
    Drag,
    Up,
    Move,
    Leave,
    Wheel,
    DoubleClick,
    KeyDown,
    KeyUp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
}

/// Modifier keys held (the names the tool events use). `cmd` is Command on macOS, Ctrl elsewhere.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mods {
    #[serde(default, skip_serializing_if = "is_false")]
    pub shift: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub alt: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ctrl: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub cmd: bool,
}

// ---------------------------------------------------------------------------------------------
// Conversion, level and validation.

impl From<super::Node> for Node {
    /// Every level-1.0 tree is a valid tree here, field for field.
    fn from(old: super::Node) -> Self {
        serde_json::to_value(old).ok().and_then(|v| serde_json::from_value(v).ok()).unwrap_or(Node::Unknown(Unknown {
            kind: "invalid".into(),
            fallback: None,
            rest: Map::new(),
        }))
    }
}

impl Node {
    /// Parses and validates a tree from JSON in one step.
    pub fn parse(json: &Value) -> Result<Node, UiError> {
        let node: Node = serde_json::from_value(json.clone()).map_err(|e| UiError(format!("the interface tree is not valid: {e}")))?;
        node.validate()?;
        Ok(node)
    }

    /// The node type as written in JSON.
    pub fn kind(&self) -> &str {
        match self {
            Node::Form(_) => "form",
            Node::Component(_) => "component",
            Node::Column(_) => "column",
            Node::Row(_) => "row",
            Node::Group(_) => "group",
            Node::Label(_) => "label",
            Node::Separator => "separator",
            Node::Button(_) => "button",
            Node::Checkbox(_) => "checkbox",
            Node::Slider(_) => "slider",
            Node::TextInput(_) => "text-input",
            Node::Select(_) => "select",
            Node::Color(_) => "color",
            Node::Image(_) => "image",
            Node::Canvas(_) => "canvas",
            Node::Tabs(_) => "tabs",
            Node::Collapsible(_) => "collapsible",
            Node::Scroll(_) => "scroll",
            Node::Grid(_) => "grid",
            Node::Table(_) => "table",
            Node::List(_) => "list",
            Node::Number(_) => "number",
            Node::TextArea(_) => "text-area",
            Node::Progress(_) => "progress",
            Node::Toggle(_) => "toggle",
            Node::Radio(_) => "radio",
            Node::Icon(_) => "icon",
            Node::Spacer(_) => "spacer",
            Node::Unknown(u) => &u.kind,
        }
    }

    /// The direct children (tabs' children in order, a component's fallback).
    pub fn children(&self) -> Vec<&Node> {
        match self {
            Node::Column(s) | Node::Row(s) => s.children.iter().collect(),
            Node::Group(g) => g.children.iter().collect(),
            Node::Scroll(s) => s.children.iter().collect(),
            Node::Grid(g) => g.children.iter().collect(),
            Node::Collapsible(c) => c.children.iter().collect(),
            Node::Tabs(t) => t.tabs.iter().flat_map(|t| t.children.iter()).collect(),
            Node::Component(c) => vec![&*c.fallback],
            Node::Unknown(u) => u.fallback.iter().map(|b| &**b).collect(),
            _ => vec![],
        }
    }

    /// The lowest UI level that can draw this tree: 1.0 when the level-1.0 parser accepts it as
    /// written, else 1.1. Unknown nodes (from a newer level) do not raise it: they degrade.
    pub fn level(&self) -> Level {
        if self.contains_unknown() {
            // An unknown node never parses at 1.0; judge the rest by its known kinds.
            return if self.first_1_1_feature().is_some() { UI_LEVEL } else { UI_LEVEL_1_0 };
        }
        let as_1_0 = serde_json::to_value(self).ok().and_then(|v| serde_json::from_value::<super::Node>(v).ok());
        if as_1_0.is_some() { UI_LEVEL_1_0 } else { UI_LEVEL }
    }

    fn contains_unknown(&self) -> bool {
        matches!(self, Node::Unknown(_)) || self.children().into_iter().any(Node::contains_unknown)
    }

    /// The first level-1.1 node type in the tree, for messages; `None` when only fields or draw
    /// ops are new (or nothing is).
    pub fn first_1_1_feature(&self) -> Option<&str> {
        if KINDS_1_1.contains(&self.kind()) {
            return Some(self.kind());
        }
        self.children().into_iter().find_map(Node::first_1_1_feature)
    }

    /// Checks that the tree needs no newer UI level than `declared` (the level the plugin's
    /// session contract carries, see [`ui_level_for_session`]), then [`Node::validate`].
    pub fn validate_at(&self, declared: Level) -> Result<(), UiError> {
        let needed = self.level();
        if needed > declared {
            let what = self.first_1_1_feature().map_or_else(|| "level 1.1 fields or drawing".to_string(), |k| format!("\"{k}\""));
            return Err(UiError(format!(
                "this interface uses {what}, which needs ui {needed}; the plugin declares ui {declared} (declare the session contract @1.1 or later)"
            )));
        }
        self.validate()
    }

    /// Checks size, depth, labels, numbers and references before the host draws anything.
    pub fn validate(&self) -> Result<(), UiError> {
        let mut cx = Cx::default();
        self.walk(0, &mut cx)?;
        for name in &cx.image_refs {
            if !cx.images.contains(name) {
                return Err(UiError(format!("canvas draws image \"{name}\", but no image in this tree has that name")));
            }
        }
        Ok(())
    }

    fn walk(&self, depth: usize, cx: &mut Cx) -> Result<(), UiError> {
        cx.count += 1;
        if cx.count > MAX_NODES || depth > MAX_DEPTH {
            return Err(UiError("the plugin's interface is too large".into()));
        }
        let kids = |children: &[Node], cx: &mut Cx| children.iter().try_for_each(|c| c.walk(depth + 1, cx));
        match self {
            Node::Form(f) => {
                f.form.validate_values(&f.values).map_err(UiError)?;
                for k in f.hidden.iter().chain(&f.disabled) {
                    if !f.form.params.iter().any(|p| p.key == *k) {
                        return Err(UiError(format!("form hides or disables unknown parameter \"{k}\"")));
                    }
                }
                Ok(())
            }
            Node::Component(c) => {
                if c.key.is_empty()
                    || c.key.len() > MAX_ID
                    || c.name.is_empty()
                    || c.name.len() > 128
                    || c.version == 0
                    || serde_json::to_vec(&c.props).map_or(true, |b| b.len() > MAX_COMPONENT_PROPS_BYTES)
                {
                    return Err(UiError("invalid component".into()));
                }
                c.fallback.walk(depth + 1, cx)
            }
            Node::Column(s) | Node::Row(s) => kids(&s.children, cx),
            Node::Group(g) => kids(&g.children, cx),
            Node::Label(l) => {
                if l.text.chars().take(MAX_TEXT + 1).count() > MAX_TEXT {
                    return Err(UiError("label text is too long".into()));
                }
                icon_name(l.icon.as_deref())
            }
            Node::Separator => Ok(()),
            Node::Button(b) => {
                control(&b.id, &b.label, b.hint.as_deref())?;
                icon_name(b.icon.as_deref())?;
                if b.icon_only && b.icon.is_none() {
                    return Err(UiError(format!("button \"{}\" is icon-only but has no icon", b.id)));
                }
                Ok(())
            }
            Node::Checkbox(c) | Node::Toggle(c) => control(&c.id, &c.label, c.hint.as_deref()),
            Node::Slider(s) => {
                control(&s.id, &s.label, s.hint.as_deref())?;
                if !(s.min.is_finite() && s.max.is_finite() && s.min < s.max && s.value.is_finite()) {
                    return Err(UiError(format!("slider \"{}\" has an invalid range", s.id)));
                }
                if s.step.is_some_and(|st| !st.is_finite() || st <= 0.0) {
                    return Err(UiError(format!("slider \"{}\" has an invalid step", s.id)));
                }
                Ok(())
            }
            Node::TextInput(t) => {
                control(&t.id, &t.label, t.hint.as_deref())?;
                hint(t.placeholder.as_deref())?;
                text_len(&t.value, MAX_TEXT, &t.id)
            }
            Node::Select(s) => {
                control(&s.id, &s.label, s.hint.as_deref())?;
                options(&s.id, &s.options)
            }
            Node::Color(c) => control(&c.id, &c.label, c.hint.as_deref()),
            Node::Image(i) => {
                if i.width == 0
                    || i.height == 0
                    || i.width > MAX_IMAGE_SIDE
                    || i.height > MAX_IMAGE_SIDE
                    || i.rgba.len() as u64 != u64::from(i.width) * u64::from(i.height) * 4
                {
                    return Err(UiError("image has the wrong size".into()));
                }
                cx.image_bytes = cx.image_bytes.saturating_add(i.rgba.len());
                if cx.image_bytes > MAX_IMAGE_BYTES {
                    return Err(UiError("the interface's images are too large".into()));
                }
                if let Some(name) = &i.name {
                    id(name, "image name")?;
                    if !cx.images.insert(name.clone()) {
                        return Err(UiError(format!("two images are named \"{name}\"")));
                    }
                    if cx.images.len() > MAX_IMAGES {
                        return Err(UiError("too many named images".into()));
                    }
                }
                if let Some([w, h]) = i.size
                    && !(size_ok(w, MAX_CANVAS_SIDE) && size_ok(h, MAX_CANVAS_SIDE))
                {
                    return Err(UiError("image display size is invalid".into()));
                }
                hint(i.alt.as_deref())
            }
            Node::Canvas(c) => {
                control(&c.id, &c.label, c.hint.as_deref())?;
                if c.ops.len() > MAX_DRAW_OPS || !(size_ok(c.width, MAX_CANVAS_SIDE) && size_ok(c.height, MAX_CANVAS_SIDE)) {
                    return Err(UiError("canvas is too large".into()));
                }
                c.ops.iter().try_for_each(|op| op.check(cx))
            }
            Node::Tabs(t) => {
                id(&t.id, "tabs id")?;
                if t.tabs.is_empty() || t.tabs.len() > MAX_TABS {
                    return Err(UiError(format!("tabs \"{}\" need 1 to {MAX_TABS} tabs", t.id)));
                }
                let mut ids = BTreeSet::new();
                for tab in &t.tabs {
                    id(&tab.id, "tab id")?;
                    label(&tab.label, &tab.id)?;
                    icon_name(tab.icon.as_deref())?;
                    if !ids.insert(&tab.id) {
                        return Err(UiError(format!("two tabs are called \"{}\"", tab.id)));
                    }
                    kids(&tab.children, cx)?;
                }
                if let Some(s) = &t.selected
                    && !ids.contains(s)
                {
                    return Err(UiError(format!("tabs \"{}\" select unknown tab \"{s}\"", t.id)));
                }
                Ok(())
            }
            Node::Collapsible(c) => {
                control(&c.id, &c.label, None)?;
                kids(&c.children, cx)
            }
            Node::Scroll(s) => {
                if s.max_height.is_some_and(|h| !size_ok(h, MAX_HEIGHT)) {
                    return Err(UiError("scroll height is invalid".into()));
                }
                kids(&s.children, cx)
            }
            Node::Grid(g) => {
                if g.columns == 0 || g.columns > MAX_GRID_COLUMNS {
                    return Err(UiError(format!("a grid has 1 to {MAX_GRID_COLUMNS} columns")));
                }
                kids(&g.children, cx)
            }
            Node::Table(t) => {
                control(&t.id, &t.label, t.hint.as_deref())?;
                if t.columns.is_empty() || t.columns.len() > MAX_TABLE_COLUMNS || t.rows.len() > MAX_ROWS {
                    return Err(UiError(format!("table \"{}\" has too many or no columns, or too many rows", t.id)));
                }
                for c in &t.columns {
                    label(&c.label, &t.id)?;
                    if c.width.is_some_and(|w| !size_ok(w, MAX_CANVAS_SIDE)) {
                        return Err(UiError(format!("table \"{}\" has an invalid column width", t.id)));
                    }
                }
                cx.cells = cx.cells.saturating_add(t.rows.len().saturating_mul(t.columns.len()));
                if cx.cells > MAX_CELLS {
                    return Err(UiError("the interface's tables are too large".into()));
                }
                let mut keys = BTreeSet::new();
                for r in &t.rows {
                    id(&r.key, "row key")?;
                    if !keys.insert(&r.key) || r.cells.len() > t.columns.len() {
                        return Err(UiError(format!("table \"{}\" has a duplicate row key or too many cells in row \"{}\"", t.id, r.key)));
                    }
                    for cell in &r.cells {
                        text_len(cell, MAX_TEXT, &t.id)?;
                    }
                }
                selection(&t.id, t.selection, &t.selected, &keys, t.max_height)
            }
            Node::List(l) => {
                control(&l.id, &l.label, l.hint.as_deref())?;
                if l.items.len() > MAX_ROWS {
                    return Err(UiError(format!("list \"{}\" has too many items", l.id)));
                }
                cx.cells = cx.cells.saturating_add(l.items.len());
                if cx.cells > MAX_CELLS {
                    return Err(UiError("the interface's lists are too large".into()));
                }
                let mut keys = BTreeSet::new();
                for item in &l.items {
                    id(&item.key, "item key")?;
                    label(&item.label, &l.id)?;
                    if let Some(d) = &item.detail {
                        text_len(d, MAX_TEXT, &l.id)?;
                    }
                    icon_name(item.icon.as_deref())?;
                    if !keys.insert(&item.key) {
                        return Err(UiError(format!("list \"{}\" has a duplicate key \"{}\"", l.id, item.key)));
                    }
                }
                selection(&l.id, l.selection, &l.selected, &keys, l.max_height)
            }
            Node::Number(n) => {
                control(&n.id, &n.label, n.hint.as_deref())?;
                let finite = |o: Option<f64>| o.is_none_or(f64::is_finite);
                if !n.value.is_finite()
                    || !finite(n.min)
                    || !finite(n.max)
                    || n.min.zip(n.max).is_some_and(|(a, b)| a > b)
                    || n.step.is_some_and(|s| !s.is_finite() || s <= 0.0)
                    || n.decimals.is_some_and(|d| d > MAX_DECIMALS)
                {
                    return Err(UiError(format!("number \"{}\" has an invalid value, range, step or decimals", n.id)));
                }
                if let Some(u) = &n.unit {
                    validate_text(u, 16, Lines::One).map_err(|e| UiError(format!("number \"{}\" unit: {e}", n.id)))?;
                }
                Ok(())
            }
            Node::TextArea(t) => {
                control(&t.id, &t.label, t.hint.as_deref())?;
                hint(t.placeholder.as_deref())?;
                if t.rows.is_some_and(|r| r == 0 || r > MAX_TEXT_ROWS) {
                    return Err(UiError(format!("text area \"{}\" asks for 1 to {MAX_TEXT_ROWS} rows", t.id)));
                }
                text_len(&t.value, MAX_TEXT_AREA, &t.id)
            }
            Node::Progress(p) => {
                if p.value.is_some_and(|v| !(0.0..=1.0).contains(&v)) {
                    return Err(UiError("progress is outside 0..=1".into()));
                }
                hint(p.label.as_deref())
            }
            Node::Radio(r) => {
                control(&r.id, &r.label, r.hint.as_deref())?;
                options(&r.id, &r.options)
            }
            Node::Icon(i) => {
                icon_name(Some(&i.name))?;
                if i.size.is_some_and(|s| !size_ok(s, MAX_ICON_SIZE)) {
                    return Err(UiError("icon size is invalid".into()));
                }
                if i.rgba.is_some_and(|c| !rgba_ok(c)) {
                    return Err(UiError("icon colour is invalid".into()));
                }
                hint(i.label.as_deref())
            }
            Node::Spacer(s) => {
                if !(s.size.is_finite() && (0.0..=MAX_SPACER).contains(&s.size)) {
                    return Err(UiError("spacer size is invalid".into()));
                }
                Ok(())
            }
            Node::Unknown(u) => {
                if let Some(why) = explain_known(&u.kind, &u.rest) {
                    return Err(UiError(format!("\"{}\" node is not valid: {why}", u.kind)));
                }
                if u.kind.is_empty() || u.kind.len() > MAX_ID {
                    return Err(UiError("node type is invalid".into()));
                }
                match &u.fallback {
                    Some(f) => f.walk(depth + 1, cx),
                    None => Ok(()),
                }
            }
        }
    }
}

impl DrawOp {
    fn check(&self, cx: &mut Cx) -> Result<(), UiError> {
        let bad = |what: &str| Err(UiError(format!("canvas {what} is invalid")));
        match self {
            DrawOp::Line(l) => {
                if !(pt(l.from) && pt(l.to) && rgba_ok(l.rgba) && width_ok(Some(l.width))) {
                    return bad("line");
                }
            }
            DrawOp::Rect(r) => {
                if !(pt(r.min) && pt(r.max) && paint_ok(&r.fill) && r.stroke.is_none_or(rgba_ok) && width_ok(r.stroke_width))
                    || r.radius.is_some_and(|x| !(x.is_finite() && x >= 0.0))
                {
                    return bad("rectangle");
                }
            }
            DrawOp::Circle(c) => {
                if !(pt(c.center) && c.radius.is_finite() && c.radius >= 0.0 && paint_ok(&c.fill) && c.stroke.is_none_or(rgba_ok) && width_ok(c.stroke_width)) {
                    return bad("circle");
                }
            }
            DrawOp::Text(t) => {
                if !(pt(t.at) && rgba_ok(t.rgba) && t.size.is_finite() && t.size > 0.0 && t.size <= MAX_FONT_SIZE) {
                    return bad("text");
                }
                text_len(&t.text, MAX_TEXT, "canvas")?;
            }
            DrawOp::Path(p) => {
                cx.points(p.path.len())?;
                let ok = p.path.iter().all(|c| match c {
                    PathCmd::Move(a) | PathCmd::Line(a) => pt(*a),
                    PathCmd::Quad(q) => q.iter().all(|a| pt(*a)),
                    PathCmd::Cubic(q) => q.iter().all(|a| pt(*a)),
                    PathCmd::Close => true,
                });
                if !ok || !p.fill.as_ref().is_none_or(paint_ok) || !p.stroke.is_none_or(rgba_ok) || !width_ok(p.stroke_width) {
                    return bad("path");
                }
            }
            DrawOp::Polygon(p) => {
                cx.points(p.points.len())?;
                if p.points.len() < 3 || !p.points.iter().all(|a| pt(*a)) || !p.fill.as_ref().is_none_or(paint_ok) || !p.stroke.is_none_or(rgba_ok) || !width_ok(p.stroke_width) {
                    return bad("polygon");
                }
            }
            DrawOp::Image(i) => {
                id(&i.image, "image name")?;
                let uv_ok = i.uv.is_none_or(|uv| uv.iter().all(|p| p.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v))));
                if !(pt(i.min) && pt(i.max) && uv_ok && i.tint.is_none_or(rgba_ok)) {
                    return bad("image");
                }
                cx.image_refs.insert(i.image.clone());
            }
            DrawOp::Clip(c) => {
                if !(pt(c.min) && pt(c.max)) {
                    return bad("clip");
                }
            }
            DrawOp::ResetClip => {}
        }
        Ok(())
    }
}

/// Checks a bare draw list (a canvas overlay) with the same rules as a canvas's ops. Image ops
/// need a tree to resolve against and are refused here.
pub fn validate_draw_list(ops: &[DrawOp]) -> Result<(), UiError> {
    if ops.len() > MAX_DRAW_OPS {
        return Err(UiError("too many draw operations".into()));
    }
    let mut cx = Cx::default();
    ops.iter().try_for_each(|op| op.check(&mut cx))?;
    if !cx.image_refs.is_empty() {
        return Err(UiError("an overlay cannot draw images".into()));
    }
    Ok(())
}

#[derive(Default)]
struct Cx {
    count: usize,
    points: usize,
    cells: usize,
    image_bytes: usize,
    images: BTreeSet<String>,
    image_refs: BTreeSet<String>,
}

impl Cx {
    fn points(&mut self, n: usize) -> Result<(), UiError> {
        self.points = self.points.saturating_add(n);
        if n > MAX_PATH_POINTS || self.points > MAX_TREE_POINTS {
            return Err(UiError("canvas paths are too large".into()));
        }
        Ok(())
    }
}

fn pt(p: [f32; 2]) -> bool {
    p[0].is_finite() && p[1].is_finite()
}
fn rgba_ok(c: [f32; 4]) -> bool {
    c.iter().all(|v| v.is_finite())
}
fn width_ok(w: Option<f32>) -> bool {
    w.is_none_or(|w| w.is_finite() && (0.0..=MAX_STROKE_WIDTH).contains(&w))
}
fn size_ok(v: f32, max: f32) -> bool {
    v.is_finite() && v > 0.0 && v <= max
}
fn paint_ok(p: &Paint) -> bool {
    match p {
        Paint::Solid(c) => rgba_ok(*c),
        Paint::Gradient(g) => {
            let geometry = match g {
                Gradient::Linear { from, to, .. } => pt(*from) && pt(*to),
                Gradient::Radial { center, radius, .. } => pt(*center) && radius.is_finite() && *radius >= 0.0,
            };
            let stops = g.stops();
            geometry
                && (2..=MAX_GRADIENT_STOPS).contains(&stops.len())
                && stops.iter().all(|s| s.at.is_finite() && (0.0..=1.0).contains(&s.at) && rgba_ok(s.rgba))
                && stops.windows(2).all(|w| w[0].at <= w[1].at)
        }
    }
}

fn id(text: &str, what: &str) -> Result<(), UiError> {
    if text.is_empty() || text.len() > MAX_ID {
        return Err(UiError(format!("{what} \"{text}\" must be 1 to {MAX_ID} bytes")));
    }
    Ok(())
}
fn label(text: &str, owner: &str) -> Result<(), UiError> {
    if text.trim().is_empty() || text.len() > MAX_LABEL {
        return Err(UiError(format!("control \"{owner}\" needs an id and a visible label")));
    }
    validate_text(text, MAX_LABEL, Lines::Many).map_err(|e| UiError(format!("label of \"{owner}\": {e}")))
}
/// An interactive control: a stable id and a visible label, plus an optional hint.
fn control(i: &str, l: &str, h: Option<&str>) -> Result<(), UiError> {
    if i.is_empty() || i.len() > MAX_ID {
        return Err(UiError(format!("control \"{i}\" needs an id and a visible label")));
    }
    label(l, i)?;
    hint(h)
}
fn hint(h: Option<&str>) -> Result<(), UiError> {
    match h {
        Some(h) if !h.is_empty() => validate_text(h, MAX_HINT, Lines::Many).map_err(|e| UiError(format!("hint: {e}"))),
        _ => Ok(()),
    }
}
fn text_len(t: &str, max: usize, owner: &str) -> Result<(), UiError> {
    if t.chars().take(max + 1).count() > max {
        return Err(UiError(format!("text in \"{owner}\" is too long")));
    }
    Ok(())
}
fn options(owner: &str, opts: &[String]) -> Result<(), UiError> {
    if opts.is_empty() || opts.len() > MAX_OPTIONS {
        return Err(UiError(format!("select \"{owner}\" has invalid options")));
    }
    opts.iter().try_for_each(|o| text_len(o, MAX_LABEL, owner))
}
fn icon_name(name: Option<&str>) -> Result<(), UiError> {
    match name {
        Some(n) if n.is_empty() || n.len() > 32 || !n.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_') => {
            Err(UiError(format!("icon name \"{n}\" is invalid")))
        }
        _ => Ok(()),
    }
}
fn selection(owner: &str, mode: Selection, selected: &[String], keys: &BTreeSet<&String>, max_height: Option<f32>) -> Result<(), UiError> {
    let allowed = match mode {
        Selection::None => 0,
        Selection::Single => 1,
        Selection::Multi => usize::MAX,
    };
    if selected.len() > allowed || selected.iter().any(|k| !keys.contains(k)) {
        return Err(UiError(format!("\"{owner}\" selects rows it does not have, or more than its selection mode allows")));
    }
    if max_height.is_some_and(|h| !size_ok(h, MAX_HEIGHT)) {
        return Err(UiError(format!("\"{owner}\" has an invalid height")));
    }
    Ok(())
}

/// Why a node of a known type did not parse, from the serde error of its own struct.
fn explain_known(kind: &str, rest: &Map<String, Value>) -> Option<String> {
    fn why<T: DeserializeOwned>(v: Value) -> String {
        serde_json::from_value::<T>(v).err().map_or_else(|| "its fields do not match".into(), |e| e.to_string())
    }
    let v = Value::Object(rest.clone());
    Some(match kind {
        "form" => why::<FormNode>(v),
        "component" => why::<Component>(v),
        "column" | "row" => why::<Stack>(v),
        "group" => why::<Group>(v),
        "label" => why::<Label>(v),
        "separator" => "a separator has no fields".into(),
        "button" => why::<Button>(v),
        "checkbox" | "toggle" => why::<Check>(v),
        "slider" => why::<Slider>(v),
        "text-input" => why::<TextInput>(v),
        "select" => why::<Select>(v),
        "color" => why::<ColorInput>(v),
        "image" => why::<Image>(v),
        "canvas" => why::<Canvas>(v),
        "tabs" => why::<Tabs>(v),
        "collapsible" => why::<Collapsible>(v),
        "scroll" => why::<Scroll>(v),
        "grid" => why::<Grid>(v),
        "table" => why::<Table>(v),
        "list" => why::<List>(v),
        "number" => why::<NumberInput>(v),
        "text-area" => why::<TextArea>(v),
        "progress" => why::<Progress>(v),
        "radio" => why::<Radio>(v),
        "icon" => why::<Icon>(v),
        "spacer" => why::<Spacer>(v),
        _ => return None,
    })
}

#[cfg(test)]
mod tests;

//! Declarative UI trees (beta-design B5). The host draws every control; a plugin only describes them.
//!
//! The builders return `serde_json::Value`s in exactly the shape `craft-ext-core` parses: an object
//! tagged by `"type"` in kebab-case. Every interactive control needs a stable `id` and a visible
//! `label`, and the host refuses a tree that breaks that rule. Pass trees to [`set_panel`] or
//! [`open_window`].

use crate::host::{CallError, call};
use serde_json::{Value, json};

/// One node of a UI tree, as JSON.
pub type Node = Value;

/// Stacks children vertically.
pub fn column(children: Vec<Node>) -> Node {
    json!({"type": "column", "children": children})
}

/// Lays children out in a row.
pub fn row(children: Vec<Node>) -> Node {
    json!({"type": "row", "children": children})
}

/// A titled group of children.
pub fn group(label: &str, children: Vec<Node>) -> Node {
    json!({"type": "group", "label": label, "children": children})
}

/// Static text.
pub fn label(text: impl Into<String>) -> Node {
    let text: String = text.into();
    json!({"type": "label", "text": text})
}

/// A horizontal rule.
pub fn separator() -> Node {
    json!({"type": "separator"})
}

/// A push button. Clicks arrive as `Event::Ui` with a `null` value.
pub fn button(id: &str, label: &str) -> Node {
    json!({"type": "button", "id": id, "label": label})
}

/// A checkbox. Changes arrive as `Event::Ui` with a boolean value.
pub fn checkbox(id: &str, label: &str, value: bool) -> Node {
    json!({"type": "checkbox", "id": id, "label": label, "value": value})
}

/// A slider. Changes arrive as `Event::Ui` with a number. `min` must be below `max`.
pub fn slider(id: &str, label: &str, value: f64, min: f64, max: f64) -> Node {
    json!({"type": "slider", "id": id, "label": label, "value": value, "min": min, "max": max})
}

/// A single-line text field. Edits arrive as `Event::Ui` with a string value.
pub fn text_input(id: &str, label: &str, value: &str) -> Node {
    json!({"type": "text-input", "id": id, "label": label, "value": value})
}

/// A drop-down. `selected` indexes `options`; changes arrive as an index.
pub fn select(id: &str, label: &str, options: &[&str], selected: u32) -> Node {
    json!({"type": "select", "id": id, "label": label, "options": options, "selected": selected})
}

/// A colour swatch. `rgba` is linear or sRGB as the app's working space says; changes arrive as RGBA.
pub fn color(id: &str, label: &str, rgba: [f32; 4]) -> Node {
    json!({"type": "color", "id": id, "label": label, "rgba": rgba})
}

/// An RGBA8 image. `rgba` must hold exactly `width * height * 4` bytes; the host refuses others.
pub fn image(width: u32, height: u32, rgba: Vec<u8>) -> Node {
    json!({"type": "image", "width": width, "height": height, "rgba": rgba})
}

/// A custom drawing area in canvas units. Pointer events arrive with `id` as the widget.
pub fn canvas(id: &str, label: &str, width: f32, height: f32, ops: Vec<Value>) -> Node {
    json!({"type": "canvas", "id": id, "label": label, "width": width, "height": height, "ops": ops})
}

/// Draw operations for [`canvas`].
pub mod draw {
    use serde_json::{Value, json};

    /// A straight line.
    pub fn line(from: [f32; 2], to: [f32; 2], width: f32, rgba: [f32; 4]) -> Value {
        json!({"op": "line", "from": from, "to": to, "width": width, "rgba": rgba})
    }

    /// A filled rectangle, with an optional outline.
    pub fn rect(min: [f32; 2], max: [f32; 2], fill: [f32; 4], stroke: Option<[f32; 4]>) -> Value {
        let mut op = json!({"op": "rect", "min": min, "max": max, "fill": fill});
        if let Some(stroke) = stroke {
            op["stroke"] = json!(stroke);
        }
        op
    }

    /// A filled circle.
    pub fn circle(center: [f32; 2], radius: f32, fill: [f32; 4]) -> Value {
        json!({"op": "circle", "center": center, "radius": radius, "fill": fill})
    }

    /// Text at a point.
    pub fn text(at: [f32; 2], text: &str, size: f32, rgba: [f32; 4]) -> Value {
        json!({"op": "text", "at": at, "text": text, "size": size, "rgba": rgba})
    }

    /// Text with `at` on the given side of it (level 1.1).
    pub fn text_aligned(at: [f32; 2], text: &str, size: f32, rgba: [f32; 4], align: super::Align, valign: super::VAlign) -> Value {
        json!({"op": "text", "at": at, "text": text, "size": size, "rgba": rgba, "align": super::to_json(align), "valign": super::to_json(valign)})
    }

    /// A rectangle with rounded corners, filled with a colour or gradient (level 1.1). Add
    /// `.stroke(…)` for an outline.
    pub fn rounded_rect(min: [f32; 2], max: [f32; 2], radius: f32, fill: impl serde::Serialize) -> Value {
        json!({"op": "rect", "min": min, "max": max, "radius": radius, "fill": super::to_json(fill)})
    }

    /// A path of move/line/quad/cubic/close commands (level 1.1). Add `.fill(…)` and/or
    /// `.stroke(…)`; each subpath fills on its own (no holes).
    pub fn path(cmds: Vec<super::PathCmd>) -> Value {
        json!({"op": "path", "path": super::to_json(cmds)})
    }

    /// A closed polygon (level 1.1). Add `.fill(…)` and/or `.stroke(…)`.
    pub fn polygon(points: &[[f32; 2]]) -> Value {
        json!({"op": "polygon", "points": points})
    }

    /// Draws the image node named `image` (same tree) into a rectangle (level 1.1).
    pub fn image(image: &str, min: [f32; 2], max: [f32; 2]) -> Value {
        json!({"op": "image", "image": image, "min": min, "max": max})
    }

    /// Clips later ops to a rectangle until [`reset_clip`] (level 1.1).
    pub fn clip(min: [f32; 2], max: [f32; 2]) -> Value {
        json!({"op": "clip", "min": min, "max": max})
    }

    /// Ends a [`clip`].
    pub fn reset_clip() -> Value {
        json!({"op": "reset-clip"})
    }
}

/// Replaces the content of a panel the manifest declares (`ui.panel.set`). Panels are free: no
/// permission is needed.
pub fn set_panel(panel: &str, tree: Node) -> Result<Value, CallError> {
    call("ui.panel.set", json!({"panel": panel, "tree": tree}))
}

/// Opens a floating window (`ui.window.open`). Needs the `ui.window` permission.
pub fn open_window(id: &str, title: &str, tree: Node) -> Result<Value, CallError> {
    call("ui.window.open", json!({"id": id, "title": title, "tree": tree}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trees_have_the_shape_craft_ext_core_parses() {
        let tree = column(vec![label("Hi"), button("go", "Go"), slider("s", "Size", 2.0, 0.0, 10.0), select("m", "Mode", &["A", "B"], 1)]);
        assert_eq!(
            tree,
            json!({
                "type": "column",
                "children": [
                    {"type": "label", "text": "Hi"},
                    {"type": "button", "id": "go", "label": "Go"},
                    {"type": "slider", "id": "s", "label": "Size", "value": 2.0, "min": 0.0, "max": 10.0},
                    {"type": "select", "id": "m", "label": "Mode", "options": ["A", "B"], "selected": 1}
                ]
            })
        );
        let plot = canvas(
            "c",
            "Plot",
            100.0,
            50.0,
            vec![draw::circle([5.0, 5.0], 2.0, [1.0, 0.0, 0.0, 1.0]), draw::rect([0.0; 2], [1.0; 2], [0.0; 4], None)],
        );
        assert_eq!(plot["ops"][0], json!({"op": "circle", "center": [5.0, 5.0], "radius": 2.0, "fill": [1.0, 0.0, 0.0, 1.0]}));
        assert!(plot["ops"][1].get("stroke").is_none(), "an absent stroke is left out, not null");
    }
}

/// A host-rendered form; widget event keys are the form's parameter keys.
pub fn form(form: &crate::Form, values: &crate::Values) -> Node {
    json!({"type":"form", "form":form, "values":values})
}
/// A named framework component, or its fallback on hosts without that version.
pub fn component(key: &str, name: &str, version: u32, props: Value, fallback: Node) -> Node {
    json!({"type":"component", "key":key,"name":name,"version":version,"props":props,"fallback":fallback})
}

// ---------------------------------------------------------------------------------------------
// UI level 1.1. A plugin that uses anything below declares its session contract at `@1.1` or later
// (for example `contract = "effectcraft.session@1.1"`): an older app then says "update the app"
// before any plugin code runs. [`level`] tells you which level a tree needs.

pub use craft_ext_core::contract::Level;
pub use craft_ext_core::ui::tree::{
    Align, Cursor, ICONS, Listen, Mods, PathCmd, PointerButton, PointerEvent, PointerKind, Selection, TextStyle, UI_LEVEL, UiValue, VAlign,
};

fn to_json<T: serde::Serialize>(v: T) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

/// Optional properties on any node. Each sets one JSON field and returns the node, so they chain:
/// `ui::button("del", "Delete").icon("trash").icon_only().hint("Delete the selection")`.
pub trait NodeExt: Sized {
    /// Sets any field. The host refuses fields a node does not have.
    fn set(self, key: &str, value: impl serde::Serialize) -> Self;
    /// Hover text on a control.
    fn hint(self, text: &str) -> Self {
        self.set("hint", text)
    }
    /// Draws the control greyed out; it sends nothing.
    fn disabled(self, disabled: bool) -> Self {
        self.set("disabled", disabled)
    }
    /// An icon from [`ICONS`] on a button, label, tab or list item.
    fn icon(self, name: &str) -> Self {
        self.set("icon", name)
    }
    /// A button that shows only its icon; its label becomes the tooltip.
    fn icon_only(self) -> Self {
        self.set("icon_only", true)
    }
    /// Greyed text shown while a text field or area is empty.
    fn placeholder(self, text: &str) -> Self {
        self.set("placeholder", text)
    }
    /// Text style of a label.
    fn style(self, style: TextStyle) -> Self {
        self.set("style", to_json(style))
    }
    /// Lower and upper bound of a number field.
    fn range(self, min: f64, max: f64) -> Self {
        self.set("min", min).set("max", max)
    }
    /// Drag/arrow step of a number field or slider.
    fn step(self, step: f64) -> Self {
        self.set("step", step)
    }
    /// Digits after the decimal mark of a number field.
    fn decimals(self, decimals: u32) -> Self {
        self.set("decimals", decimals)
    }
    /// Unit shown after a number field's value (`px`, `%`).
    fn unit(self, unit: &str) -> Self {
        self.set("unit", unit)
    }
    /// Selection mode and the selected keys of a table or list.
    fn selection(self, mode: Selection, selected: &[&str]) -> Self {
        self.set("selection", to_json(mode)).set("selected", selected)
    }
    /// Maximum height of a scroll area, table or list, in points.
    fn max_height(self, height: f32) -> Self {
        self.set("max_height", height)
    }
    /// Extra canvas events: moves, wheel, double clicks, keys.
    fn listen(self, events: &[Listen]) -> Self {
        self.set("listen", to_json(events))
    }
    /// The cursor over a canvas.
    fn cursor(self, cursor: Cursor) -> Self {
        self.set("cursor", to_json(cursor))
    }
    /// A canvas-drawable name for an image node (see [`draw::image`]).
    fn named(self, name: &str) -> Self {
        self.set("name", name)
    }
    /// Registers an image for canvases without showing it.
    fn hidden(self) -> Self {
        self.set("hidden", true)
    }
    /// Fill of a path, polygon, rectangle or circle draw op: a colour (`[r,g,b,a]`) or a
    /// [`paint`] gradient.
    fn fill(self, paint: impl serde::Serialize) -> Self {
        self.set("fill", paint)
    }
    /// Outline colour and width of a draw op.
    fn stroke(self, rgba: [f32; 4], width: f32) -> Self {
        self.set("stroke", rgba).set("stroke_width", width)
    }
}

impl NodeExt for Value {
    fn set(mut self, key: &str, value: impl serde::Serialize) -> Self {
        if let Some(obj) = self.as_object_mut() {
            obj.insert(key.to_string(), to_json(value));
        }
        self
    }
}

/// A label in a text style.
pub fn text(text: impl Into<String>, style: TextStyle) -> Node {
    label(text).style(style)
}

/// Tabs. Switching sends the tab id as a string; `selected` is the tab shown first.
pub fn tabs(id: &str, tabs: Vec<Node>, selected: Option<&str>) -> Node {
    let mut n = json!({"type": "tabs", "id": id, "tabs": tabs});
    if let Some(s) = selected {
        n = n.set("selected", s);
    }
    n
}

/// One tab for [`tabs`].
pub fn tab(id: &str, label: &str, children: Vec<Node>) -> Node {
    json!({"id": id, "label": label, "children": children})
}

/// A section that opens and closes. Toggling sends a bool.
pub fn collapsible(id: &str, label: &str, open: bool, children: Vec<Node>) -> Node {
    json!({"type": "collapsible", "id": id, "label": label, "open": open, "children": children})
}

/// A vertically scrolling area (`.max_height(…)` bounds it).
pub fn scroll(children: Vec<Node>) -> Node {
    json!({"type": "scroll", "children": children})
}

/// Children in `columns` aligned columns, row by row.
pub fn grid(columns: u32, children: Vec<Node>) -> Node {
    json!({"type": "grid", "columns": columns, "children": children})
}

/// A table of text. Selection changes send the selected row keys; a double click sends
/// `{"activate": key}`. Add `.selection(Selection::Multi, &[])` to make rows selectable.
pub fn table(id: &str, label: &str, columns: &[&str], rows: Vec<Node>) -> Node {
    let columns: Vec<Value> = columns.iter().map(|c| json!({"label": c})).collect();
    json!({"type": "table", "id": id, "label": label, "columns": columns, "rows": rows})
}

/// One table row: a stable key and its cells.
pub fn table_row(key: &str, cells: &[&str]) -> Node {
    json!({"key": key, "cells": cells})
}

/// A list of items with the same events as [`table`].
pub fn list(id: &str, label: &str, items: Vec<Node>) -> Node {
    json!({"type": "list", "id": id, "label": label, "items": items})
}

/// One list item (`.icon(…)`, `.set("detail", …)`).
pub fn item(key: &str, label: &str) -> Node {
    json!({"key": key, "label": label})
}

/// A number field to drag or type. Sends a number when the edit ends.
pub fn number(id: &str, label: &str, value: f64) -> Node {
    json!({"type": "number", "id": id, "label": label, "value": value})
}

/// Multi-line text. Sends the text when focus leaves or on Ctrl/Cmd+Enter.
pub fn text_area(id: &str, label: &str, value: &str, rows: u32) -> Node {
    json!({"type": "text-area", "id": id, "label": label, "value": value, "rows": rows})
}

/// A progress bar (`value` in 0..=1), or a "working" indicator when `None`.
pub fn progress(value: Option<f32>, label: Option<&str>) -> Node {
    let mut n = json!({"type": "progress"});
    if let Some(v) = value {
        n = n.set("value", v);
    }
    if let Some(l) = label {
        n = n.set("label", l);
    }
    n
}

/// A switch. Changes arrive as a bool.
pub fn toggle(id: &str, label: &str, value: bool) -> Node {
    json!({"type": "toggle", "id": id, "label": label, "value": value})
}

/// Radio buttons. Changes arrive as the chosen index.
pub fn radio(id: &str, label: &str, options: &[&str], selected: u32) -> Node {
    json!({"type": "radio", "id": id, "label": label, "options": options, "selected": selected})
}

/// An icon from [`ICONS`] (an unknown name draws a placeholder).
pub fn icon(name: &str) -> Node {
    json!({"type": "icon", "name": name})
}

/// Empty space, in points.
pub fn spacer(size: f32) -> Node {
    json!({"type": "spacer", "size": size})
}

/// Fills for draw ops: gradients in canvas coordinates. A plain colour is just `[r,g,b,a]`.
pub mod paint {
    use serde_json::{Value, json};

    /// A colour stop: position 0..=1 and colour.
    pub fn stop(at: f32, rgba: [f32; 4]) -> Value {
        json!({"at": at, "rgba": rgba})
    }

    /// A linear gradient from `from` to `to`. 2 to 16 stops, positions not decreasing.
    pub fn linear(from: [f32; 2], to: [f32; 2], stops: Vec<Value>) -> Value {
        json!({"gradient": "linear", "from": from, "to": to, "stops": stops})
    }

    /// A radial gradient around `center`.
    pub fn radial(center: [f32; 2], radius: f32, stops: Vec<Value>) -> Value {
        json!({"gradient": "radial", "center": center, "radius": radius, "stops": stops})
    }
}

/// The UI level a tree needs (1.0 or 1.1), or the reason it is not a valid tree.
pub fn level(tree: &Node) -> Result<Level, String> {
    craft_ext_core::ui::tree::Node::parse(tree).map(|n| n.level()).map_err(|e| e.0)
}

/// Checks a tree with the host's own rules (limits, labels, references): useful in tests.
pub fn check(tree: &Node) -> Result<(), String> {
    craft_ext_core::ui::tree::Node::parse(tree).map(|_| ()).map_err(|e| e.0)
}

/// Reads the `value` of a `ui` event: a click, bool, number, text, selection, activation or
/// canvas pointer/key event. `None` when it is none of these.
pub fn value(value: &Value) -> Option<UiValue> {
    serde_json::from_value(value.clone()).ok()
}

/// The canvas event in a `ui` event's value, if it is one.
pub fn pointer(value: &Value) -> Option<PointerEvent> {
    serde_json::from_value(value.clone()).ok()
}

#[cfg(test)]
mod level_1_1_tests {
    use super::*;

    #[test]
    fn new_builders_make_trees_the_host_accepts_at_level_1_1() {
        let pad = canvas(
            "pad",
            "Pad",
            200.0,
            100.0,
            vec![
                draw::rounded_rect(
                    [0.0, 0.0],
                    [200.0, 100.0],
                    6.0,
                    paint::linear([0.0, 0.0], [200.0, 0.0], vec![paint::stop(0.0, [0.0; 4]), paint::stop(1.0, [1.0; 4])]),
                ),
                draw::path(vec![PathCmd::Move([0.0, 0.0]), PathCmd::Quad([[10.0, 10.0], [20.0, 0.0]]), PathCmd::Close])
                    .fill([1.0, 0.0, 0.0, 1.0])
                    .stroke([0.0, 0.0, 0.0, 1.0], 2.0),
                draw::polygon(&[[0.0, 0.0], [10.0, 0.0], [5.0, 8.0]]).fill([1.0; 4]),
                draw::clip([0.0, 0.0], [50.0, 50.0]),
                draw::image("swatch", [0.0, 0.0], [10.0, 10.0]),
                draw::reset_clip(),
                draw::text_aligned([100.0, 50.0], "Hi", 12.0, [1.0; 4], Align::Center, VAlign::Center),
            ],
        )
        .listen(&[Listen::Move, Listen::Wheel, Listen::Keys])
        .cursor(Cursor::Crosshair);
        let tree = column(vec![
            tabs(
                "tabs",
                vec![
                    tab("a", "Draw", vec![pad, image(1, 1, vec![0, 0, 0, 255]).named("swatch").hidden()]),
                    tab(
                        "b",
                        "Data",
                        vec![
                            table("t", "Rows", &["Name", "Size"], vec![table_row("r1", &["One", "1"])]).selection(Selection::Multi, &["r1"]),
                            list("l", "Items", vec![item("i", "Item").icon("star")]).selection(Selection::Single, &[]).max_height(120.0),
                        ],
                    ),
                ],
                Some("b"),
            ),
            collapsible("more", "More", false, vec![scroll(vec![grid(2, vec![text("A", TextStyle::Heading), spacer(4.0)])]).max_height(80.0)]),
            number("w", "Width", 10.0).range(0.0, 100.0).step(0.5).decimals(1).unit("px").hint("Width"),
            text_area("notes", "Notes", "", 4).placeholder("Type…"),
            progress(Some(0.5), Some("Half")),
            toggle("live", "Live", true).disabled(true),
            radio("mode", "Mode", &["A", "B"], 0),
            icon("info"),
            button("del", "Delete").icon("trash").icon_only(),
        ]);
        check(&tree).unwrap();
        assert_eq!(level(&tree), Ok(UI_LEVEL));
        assert_eq!(level(&column(vec![label("Hi"), button("go", "Go")])), Ok(Level::new(1, 0)));
        assert!(check(&button("x", "")).is_err());
    }

    #[test]
    fn event_values_parse() {
        assert_eq!(value(&json!(["a", "b"])), Some(UiValue::Keys(vec!["a".into(), "b".into()])));
        assert_eq!(value(&json!({"activate": "r1"})), Some(UiValue::Activate { activate: "r1".into() }));
        let p = pointer(&json!({"kind": "wheel", "at": [1, 2], "delta": [0, -3], "mods": {"shift": true}})).unwrap();
        assert_eq!((p.kind, p.delta, p.mods.shift), (PointerKind::Wheel, Some([0.0, -3.0]), true));
        let old = pointer(&json!({"kind": "down", "at": [1, 2]})).unwrap();
        assert_eq!((old.kind, old.button), (PointerKind::Down, None));
    }
}

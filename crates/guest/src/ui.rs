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

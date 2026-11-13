//! Declarative plugin UI: plugins describe, the host draws (theme, accessibility, no spoofed prompts,
//! no plugin code on the UI thread). Sent as JSON by `ui.panel.set` and `ui.window.open`.
//!
//! Invariants: bounded size and depth; every interactive widget has a stable `id` and a visible label
//! (unlabelled controls are refused); events carry the tree `rev` so stale events are dropped.

use serde::{Deserialize, Serialize};

pub const MAX_NODES: usize = 2_000;
pub const MAX_DEPTH: usize = 16;
pub const MAX_DRAW_OPS: usize = 20_000;
/// Maximum encoded JSON bytes for a framework component’s props.
pub const MAX_COMPONENT_PROPS_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Node {
    Form {
        form: crate::form::Form,
        values: crate::form::Values,
    },
    Component {
        key: String,
        name: String,
        version: u32,
        props: serde_json::Value,
        fallback: Box<Node>,
    },
    Column {
        children: Vec<Node>,
    },
    Row {
        children: Vec<Node>,
    },
    Group {
        label: String,
        children: Vec<Node>,
    },
    Label {
        text: String,
    },
    Separator,
    Button {
        id: String,
        label: String,
    },
    Checkbox {
        id: String,
        label: String,
        value: bool,
    },
    Slider {
        id: String,
        label: String,
        value: f64,
        min: f64,
        max: f64,
    },
    TextInput {
        id: String,
        label: String,
        value: String,
    },
    Select {
        id: String,
        label: String,
        options: Vec<String>,
        selected: u32,
    },
    Color {
        id: String,
        label: String,
        rgba: [f32; 4],
    },
    /// RGBA8 image, `width * height * 4` bytes, base64 in JSON is avoided: pixels go as a byte array.
    Image {
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
    /// A custom drawing area; pointer events are reported with this id in canvas coordinates.
    Canvas {
        id: String,
        label: String,
        width: f32,
        height: f32,
        ops: Vec<DrawOp>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
pub enum DrawOp {
    Line {
        from: [f32; 2],
        to: [f32; 2],
        width: f32,
        rgba: [f32; 4],
    },
    Rect {
        min: [f32; 2],
        max: [f32; 2],
        fill: [f32; 4],
        #[serde(default)]
        stroke: Option<[f32; 4]>,
    },
    Circle {
        center: [f32; 2],
        radius: f32,
        fill: [f32; 4],
    },
    Text {
        at: [f32; 2],
        text: String,
        size: f32,
        rgba: [f32; 4],
    },
}

/// A widget event sent to the plugin as `{"event":"ui", ...}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiEvent {
    /// Panel or window id the tree belongs to.
    pub surface: String,
    pub rev: u64,
    pub widget: String,
    pub value: UiValue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UiValue {
    Click,
    Bool(bool),
    Number(f64),
    Text(String),
    Index(u32),
    Color([f32; 4]),
    /// A form point value in its declared coordinate space.
    Point([f64; 2]),
    Pointer { kind: String, at: [f32; 2] },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiError(pub String);

impl Node {
    /// Checks size, depth, labels and numbers before the host draws anything.
    pub fn validate(&self) -> Result<(), UiError> {
        let mut count = 0usize;
        self.walk(0, &mut count)
    }

    fn walk(&self, depth: usize, count: &mut usize) -> Result<(), UiError> {
        *count += 1;
        if *count > MAX_NODES || depth > MAX_DEPTH {
            return Err(UiError("the plugin's interface is too large".into()));
        }
        let labelled = |id: &str, label: &str| {
            if id.is_empty() || id.len() > 64 || label.trim().is_empty() || label.len() > 200 {
                Err(UiError(format!("control \"{id}\" needs an id and a visible label")))
            } else {
                Ok(())
            }
        };
        match self {
            Node::Form { form, values } => form.validate_values(values).map_err(UiError),
            Node::Component { key, name, version, props, fallback } => {
                if key.is_empty()
                    || key.len() > 64
                    || name.is_empty()
                    || name.len() > 128
                    || *version == 0
                    || serde_json::to_vec(props).map_or(true, |b| b.len() > MAX_COMPONENT_PROPS_BYTES)
                {
                    return Err(UiError("invalid component".into()));
                }
                fallback.walk(depth + 1, count)
            }
            Node::Column { children } | Node::Row { children } | Node::Group { children, .. } => {
                children.iter().try_for_each(|c| c.walk(depth + 1, count))
            }
            Node::Label { text } if text.len() > 4000 => Err(UiError("label text is too long".into())),
            Node::Label { .. } | Node::Separator => Ok(()),
            Node::Button { id, label } | Node::Checkbox { id, label, .. } | Node::TextInput { id, label, .. } | Node::Color { id, label, .. } => {
                labelled(id, label)
            }
            Node::Slider { id, label, value, min, max } => {
                labelled(id, label)?;
                if !(min.is_finite() && max.is_finite() && min < max && value.is_finite()) {
                    return Err(UiError(format!("slider \"{id}\" has an invalid range")));
                }
                Ok(())
            }
            Node::Select { id, label, options, .. } => {
                labelled(id, label)?;
                if options.is_empty() || options.len() > 256 {
                    return Err(UiError(format!("select \"{id}\" has invalid options")));
                }
                Ok(())
            }
            Node::Image { width, height, rgba } => {
                if *width == 0 || *height == 0 || *width > 2048 || *height > 2048 || rgba.len() as u64 != u64::from(*width) * u64::from(*height) * 4 {
                    return Err(UiError("image has the wrong size".into()));
                }
                Ok(())
            }
            Node::Canvas { id, label, width, height, ops } => {
                labelled(id, label)?;
                if ops.len() > MAX_DRAW_OPS
                    || !(width.is_finite() && height.is_finite() && *width > 0.0 && *height > 0.0 && *width <= 8192.0 && *height <= 8192.0)
                {
                    return Err(UiError("canvas is too large".into()));
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_validates() {
        let json = r#"{"type":"column","children":[{"type":"label","text":"Hi"},{"type":"button","id":"go","label":"Go"},
            {"type":"slider","id":"s","label":"Size","value":2,"min":0,"max":10}]}"#;
        let n: Node = serde_json::from_str(json).unwrap();
        assert!(n.validate().is_ok());
        let bad: Node = serde_json::from_str(r#"{"type":"button","id":"go","label":" "}"#).unwrap();
        assert!(bad.validate().is_err());
        assert!(serde_json::from_str::<Node>(r#"{"type":"webview","url":"x"}"#).is_err());
    }
}

#[cfg(test)]
mod form_tests {
    use super::*;
    #[test]
    fn validates_forms_and_component_fallbacks() {
        let form = crate::form::Form { schema: 1, key: "form".into(), title: "Form".into(), params: vec![], preview: None };
        Node::Form { form: form.clone(), values: Default::default() }.validate().unwrap();
        let mut component = Node::Component {
            key: "key".into(),
            name: "histogram".into(),
            version: 1,
            props: serde_json::json!({}),
            fallback: Box::new(Node::Form { form, values: Default::default() }),
        };
        component.validate().unwrap();
        if let Node::Component { props, .. } = &mut component {
            *props = serde_json::json!("x".repeat(MAX_COMPONENT_PROPS_BYTES));
        }
        assert!(component.validate().is_err());
        if let Node::Component { props, fallback, .. } = &mut component {
            *props = serde_json::json!({});
            *fallback = Box::new(Node::Button { id: String::new(), label: String::new() });
        }
        assert!(component.validate().is_err());
    }
}

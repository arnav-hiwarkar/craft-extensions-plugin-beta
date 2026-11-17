//! Typed argument schemas for the operations every adapter shares (backlog TYPED-CONTRACTS).
//!
//! `OpSpec::args` and `OpSpec::result` are example text for people. The UI family below is the
//! first family with a structured schema: field names, types, bounds, required fields and
//! alternatives. The host checks a call's arguments against it before any adapter sees the call
//! (`api::check_args`, from the Wasm and native entry points), and the generated contract page
//! and JSON Schema are rendered from the same definition, so the documented shape and the
//! dispatched shape cannot drift apart.
//!
//! A shared schema is the envelope every adapter accepts: unknown fields, wrong types and
//! out-of-range sizes are refused here, while each adapter keeps its own narrower limits (for
//! example EffectCraft's 300-character notifications and 48-character panel ids) and the checks
//! that need app state (declared panels, window counts, UI tree levels).

use serde_json::{Map, Value, json};

/// The type of one argument field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    Bool,
    /// Text of `min..=max` characters.
    Text { min: usize, max: usize },
    /// A JSON object whose contents the adapter validates (`what` names it in the docs).
    Object { what: &'static str },
    /// Any JSON value the adapter validates (`what` names it in the docs).
    Json { what: &'static str },
    /// An array of at most `max` items the adapter validates (`what` names one item).
    List { max: usize, what: &'static str },
}

/// One named argument.
#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub name: &'static str,
    pub ty: Ty,
    pub required: bool,
    pub doc: &'static str,
}

/// A structured operation schema. `one_of` lists fields of which exactly one must be present.
#[derive(Clone, Copy, Debug)]
pub struct OpSchema {
    pub op: &'static str,
    /// Bumped when a field is removed or its meaning changes; adding an optional field is not.
    pub version: u32,
    pub fields: &'static [Field],
    pub one_of: &'static [&'static str],
    pub result: &'static str,
}

const SURFACE_ID: Ty = Ty::Text { min: 1, max: 64 };
const TREE: Ty = Ty::Json { what: "UI tree (validated at the contribution's UI level)" };
const PATCH: Ty = Ty::Object { what: "form patch (`craft_ext_core::form::Patch`)" };

/// The UI operation family shared by EffectCraft, PhotoCraft and FigCraft.
pub static SHARED: &[OpSchema] = &[
    OpSchema {
        op: "ui.notify",
        version: 1,
        fields: &[
            Field { name: "message", ty: Ty::Text { min: 0, max: 4000 }, required: true, doc: "Short text for the person" },
            Field { name: "error", ty: Ty::Bool, required: false, doc: "Show it as an error where the app distinguishes errors" },
        ],
        one_of: &[],
        result: "null",
    },
    OpSchema {
        op: "ui.panel.set",
        version: 1,
        fields: &[
            Field { name: "panel", ty: SURFACE_ID, required: true, doc: "A panel id the manifest declares" },
            Field { name: "tree", ty: TREE, required: false, doc: "The panel's whole content" },
            Field { name: "patch", ty: PATCH, required: false, doc: "A change to the panel's current form, where the app supports patches" },
        ],
        one_of: &["tree", "patch"],
        result: "null",
    },
    OpSchema {
        op: "ui.window.open",
        version: 1,
        fields: &[
            Field { name: "id", ty: SURFACE_ID, required: true, doc: "The plugin's own window id" },
            Field { name: "title", ty: Ty::Text { min: 0, max: 200 }, required: false, doc: "Window title (some apps require it with `tree`)" },
            Field { name: "tree", ty: TREE, required: false, doc: "The window's whole content" },
            Field { name: "patch", ty: PATCH, required: false, doc: "A change to the window's current form, where the app supports patches" },
        ],
        one_of: &["tree", "patch"],
        result: "null",
    },
    OpSchema {
        op: "canvas.overlay.set",
        version: 1,
        fields: &[Field {
            name: "ops",
            ty: Ty::List { max: craft_ext_core::ui::MAX_DRAW_OPS, what: "draw operation" },
            required: false,
            doc: "The plugin's whole overlay; omitted or empty clears it",
        }],
        one_of: &[],
        result: "null",
    },
];

/// The shared schema for `op`, if it belongs to a typed family.
pub fn shared(op: &str) -> Option<&'static OpSchema> {
    SHARED.iter().find(|s| s.op == op)
}

impl Ty {
    fn check(self, value: &Value) -> Result<(), String> {
        match (self, value) {
            (Ty::Bool, Value::Bool(_)) | (Ty::Object { .. }, Value::Object(_)) | (Ty::Json { .. }, _) => Ok(()),
            (Ty::Text { min, max }, Value::String(s)) => {
                // Count at most max + 1 characters: a huge string costs no more than a short one.
                let n = s.chars().take(max.saturating_add(1)).count();
                if n < min {
                    Err(format!("must have at least {min} characters"))
                } else if n > max {
                    Err(format!("must have at most {max} characters"))
                } else {
                    Ok(())
                }
            }
            (Ty::List { max, .. }, Value::Array(items)) if items.len() > max => Err(format!("must have at most {max} items")),
            (Ty::List { .. }, Value::Array(_)) => Ok(()),
            (ty, _) => Err(format!("must be {}", ty.name())),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Ty::Bool => "true or false",
            Ty::Text { .. } => "text",
            Ty::Object { .. } => "an object",
            Ty::Json { .. } => "a JSON value",
            Ty::List { .. } => "an array",
        }
    }

    fn json_schema(self) -> Value {
        match self {
            Ty::Bool => json!({"type": "boolean"}),
            Ty::Text { min, max } => json!({"type": "string", "minLength": min, "maxLength": max}),
            Ty::Object { what } => json!({"type": "object", "description": what}),
            Ty::Json { what } => json!({"description": what}),
            Ty::List { max, what } => json!({"type": "array", "maxItems": max, "items": {"description": what}}),
        }
    }

    fn describe(self) -> String {
        match self {
            Ty::Bool => "boolean".into(),
            Ty::Text { min: 0, max } => format!("text, ≤ {max} characters"),
            Ty::Text { min, max } => format!("text, {min}–{max} characters"),
            Ty::Object { what } | Ty::Json { what } => what.into(),
            Ty::List { max, what } => format!("array of ≤ {max} × {what}"),
        }
    }
}

impl OpSchema {
    /// Check call arguments. `null` (a call with no arguments) counts as `{}`.
    pub fn check(&self, args: &Value) -> Result<(), String> {
        let empty = Map::new();
        let object = match args {
            Value::Null => &empty,
            Value::Object(o) => o,
            _ => return Err("arguments must be an object".into()),
        };
        if let Some(unknown) = object.keys().find(|k| !self.fields.iter().any(|f| f.name == k.as_str())) {
            // Bound the echoed name: it comes from the guest.
            let shown: String = unknown.chars().take(48).collect();
            return Err(format!("unknown argument `{shown}`"));
        }
        for field in self.fields {
            match object.get(field.name) {
                Some(value) => field.ty.check(value).map_err(|why| format!("`{}` {why}", field.name))?,
                None if field.required => return Err(format!("`{}` is required", field.name)),
                None => {}
            }
        }
        if !self.one_of.is_empty() {
            let present = self.one_of.iter().filter(|name| object.contains_key(**name)).count();
            if present != 1 {
                return Err(format!("give exactly one of {}", self.one_of.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ")));
            }
        }
        Ok(())
    }

    /// The arguments as JSON Schema (draft 2020-12), for generated bindings and docs.
    pub fn json_schema(&self) -> Value {
        let properties: Map<String, Value> = self.fields.iter().map(|f| {
            let mut schema = f.ty.json_schema();
            if let Some(o) = schema.as_object_mut() {
                o.insert("title".into(), json!(f.doc));
            }
            (f.name.to_string(), schema)
        }).collect();
        let required: Vec<&str> = self.fields.iter().filter(|f| f.required).map(|f| f.name).collect();
        let mut schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": self.op,
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false,
        });
        if !self.one_of.is_empty() {
            schema["oneOf"] = Value::Array(self.one_of.iter().map(|name| json!({"required": [name]})).collect());
        }
        schema
    }

    /// The field table for the generated contract page.
    pub fn to_markdown(&self) -> String {
        let mut s = format!("Typed arguments (schema version {}; unknown arguments are refused):\n\n| Argument | Type | Required | Meaning |\n|---|---|---|---|\n", self.version);
        for f in self.fields {
            s.push_str(&format!("| `{}` | {} | {} | {} |\n", f.name, f.ty.describe(), if f.required { "yes" } else { "no" }, f.doc));
        }
        if !self.one_of.is_empty() {
            s.push_str(&format!("\nGive exactly one of {}.\n", self.one_of.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ")));
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(op: &str, args: Value) -> Result<(), String> {
        shared(op).unwrap().check(&args)
    }

    #[test]
    fn the_guest_sdk_and_examples_calls_pass() {
        let tree = json!({"type": "label", "text": "Hi"});
        check("ui.notify", json!({"message": "Auto Levels: black 3, white 250"})).unwrap();
        check("ui.notify", json!({"message": "Failed", "error": true})).unwrap();
        check("ui.panel.set", json!({"panel": "notes", "tree": tree})).unwrap();
        check("ui.panel.set", json!({"panel": "notes", "patch": {"rev": 2, "set": {}}})).unwrap();
        check("ui.window.open", json!({"id": "w0", "title": "Window", "tree": tree})).unwrap();
        check("ui.window.open", json!({"id": "w0", "patch": {"rev": 2}})).unwrap();
        check("canvas.overlay.set", json!({"ops": [{"op": "circle", "center": [0, 0], "radius": 5}]})).unwrap();
        check("canvas.overlay.set", json!({})).unwrap();
        check("canvas.overlay.set", Value::Null).unwrap();
    }

    #[test]
    fn wrong_shapes_are_refused_with_the_field_named() {
        let tree = json!({"type": "label", "text": "Hi"});
        let refused = |op: &str, args: Value, why: &str| {
            let e = check(op, args).unwrap_err();
            assert!(e.contains(why), "{op}: {e}");
        };
        refused("ui.notify", json!({}), "`message` is required");
        refused("ui.notify", json!({"message": 4}), "`message` must be text");
        refused("ui.notify", json!({"message": "x".repeat(4001)}), "at most 4000");
        refused("ui.notify", json!({"message": "x", "level": "warn"}), "unknown argument `level`");
        refused("ui.notify", json!({"message": "x", "error": "yes"}), "`error` must be true or false");
        refused("ui.notify", json!(["x"]), "must be an object");
        refused("ui.panel.set", json!({"panel": "", "tree": tree}), "at least 1");
        refused("ui.panel.set", json!({"panel": "p".repeat(65), "tree": tree}), "at most 64");
        refused("ui.panel.set", json!({"panel": "notes"}), "exactly one of `tree`, `patch`");
        refused("ui.panel.set", json!({"panel": "notes", "tree": tree, "patch": {}}), "exactly one");
        refused("ui.panel.set", json!({"panel": "notes", "patch": 3}), "`patch` must be an object");
        // Host-internal staging fields are not guest arguments.
        refused("ui.panel.set", json!({"panel": "notes", "tree": tree, "form_metadata": {}}), "unknown argument");
        refused("ui.window.open", json!({"title": "W", "tree": tree}), "`id` is required");
        refused("ui.window.open", json!({"id": "w", "title": "t".repeat(201), "tree": tree}), "at most 200");
        refused("canvas.overlay.set", json!({"ops": {"op": "line"}}), "`ops` must be an array");
        let many: Vec<Value> = vec![json!({}); craft_ext_core::ui::MAX_DRAW_OPS + 1];
        refused("canvas.overlay.set", json!({"ops": many}), "at most");
        let long_name = "k".repeat(10_000);
        let e = check("ui.notify", json!({"message": "x", long_name.clone(): 1})).unwrap_err();
        assert!(e.len() < 100, "the echoed name is bounded: {e}");
    }

    #[test]
    fn json_schema_and_markdown_come_from_the_same_fields() {
        for s in SHARED {
            let schema = s.json_schema();
            assert_eq!(schema["additionalProperties"], false, "{}", s.op);
            let properties = schema["properties"].as_object().unwrap();
            assert_eq!(properties.len(), s.fields.len(), "{}", s.op);
            let page = s.to_markdown();
            for f in s.fields {
                assert!(properties.contains_key(f.name), "{}", s.op);
                assert!(page.contains(&format!("`{}`", f.name)), "{}", s.op);
            }
        }
        let panel = shared("ui.panel.set").unwrap().json_schema();
        assert_eq!(panel["required"], json!(["panel"]));
        assert_eq!(panel["oneOf"], json!([{"required": ["tree"]}, {"required": ["patch"]}]));
        assert_eq!(panel["properties"]["panel"]["maxLength"], 64);
    }
}

//! Data your plugin keeps inside the document: settings, rig descriptions, generator inputs. It is
//! saved with the file and kept intact when someone opens it without your plugin, so it comes back
//! when they install it. Each place holds at most 64 keys and 256 KiB per plugin. Needs
//! `document.read` to read and `document.edit` to write.

use crate::host::{CallError, call};
use serde_json::{Value, json};

/// Where data lives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// The document as a whole.
    Document,
    /// One layer (EffectCraft, PhotoCraft).
    Layer(u64),
    /// One node (FigCraft).
    Node(u64),
}

fn args(place: Place, key: &str) -> Value {
    match place {
        Place::Document => json!({ "key": key }),
        Place::Layer(id) => json!({ "key": key, "target": { "layer": id } }),
        Place::Node(id) => json!({ "key": key, "target": { "node": id } }),
    }
}

/// Reads one key; `Value::Null` when it was never set.
pub fn get(place: Place, key: &str) -> Result<Value, CallError> {
    call("project.data.get", args(place, key))
}

/// Writes one key; `Value::Null` deletes it. Part of the current event's undo step.
pub fn set(place: Place, key: &str, value: Value) -> Result<(), CallError> {
    let mut a = args(place, key);
    a["value"] = value;
    call("project.data.set", a).map(drop)
}

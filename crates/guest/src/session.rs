//! Session events (`effectcraft.session@1`): what the host delivers to a session plugin.
//!
//! Every event is one JSON object tagged by `"event"`. An event this SDK does not know parses to
//! [`Event::Unknown`] rather than failing, so newer hosts do not break older plugins.

use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// One event from the host. Each is one transaction: its effects commit together as one undo step,
/// or roll back on error.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Sent once after the plugin loads. Draw your panels here.
    Init,
    /// A command contribution ran. `id` is the contribution id.
    Command {
        /// The command contribution's id.
        id: String,
    },
    /// A widget in one of your panels or windows changed. `value` is `null` for a button click.
    /// `rev` is the tree revision the event refers to; events from an older revision are dropped by
    /// the host before they reach you.
    Ui {
        /// The panel or window id.
        surface: String,
        /// The tree revision the widget belongs to.
        rev: u64,
        /// The widget id.
        widget: String,
        /// The new value: a bool, number, string, index, colour or pointer, or `null` for a click.
        value: Value,
    },
    /// A pointer-tool gesture. `phase` is `"begin"`, `"update"`, `"commit"` or `"cancel"`; `at` is the
    /// pointer position as the host sends it.
    Tool {
        /// `"begin"`, `"update"`, `"commit"` or `"cancel"`.
        phase: String,
        /// The pointer position, as the host sends it.
        at: Value,
    },
    /// An app event the plugin subscribed to. Needs the `app.events` permission.
    App {
        /// The event name, such as `"selection.changed"`.
        name: String,
        /// The event's payload.
        payload: Value,
    },
    /// The plugin is being turned off or unloaded. Release anything you hold.
    Dispose,
    /// An event this SDK version does not know, as the raw JSON.
    Unknown(Value),
}

impl Event {
    /// Reads an event from its JSON value. Missing or mistyped fields make it [`Event::Unknown`].
    pub fn from_value(value: Value) -> Self {
        let Some(obj) = value.as_object() else { return Self::Unknown(value) };
        let text = |key: &str| obj.get(key).and_then(Value::as_str).map(str::to_owned);
        let field = |key: &str| obj.get(key).cloned().unwrap_or(Value::Null);
        let event = match obj.get("event").and_then(Value::as_str) {
            Some("init") => Some(Self::Init),
            Some("dispose") => Some(Self::Dispose),
            Some("command") => Some(Self::Command { id: text("id").unwrap_or_default() }),
            Some("ui") => match (text("surface"), obj.get("rev").and_then(Value::as_u64), text("widget")) {
                (Some(surface), Some(rev), Some(widget)) => Some(Self::Ui { surface, rev, widget, value: field("value") }),
                _ => None,
            },
            Some("tool") => text("phase").map(|phase| Self::Tool { phase, at: field("at") }),
            Some("app") => text("name").map(|name| Self::App { name, payload: field("payload") }),
            _ => None,
        };
        event.unwrap_or(Self::Unknown(value))
    }
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Value::deserialize(d).map(Self::from_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_known_events_and_keeps_unknown_ones() {
        let ui: Event = serde_json::from_str(r#"{"event":"ui","surface":"notes","rev":3,"widget":"add-marker","value":null}"#).unwrap();
        assert_eq!(ui, Event::Ui { surface: "notes".into(), rev: 3, widget: "add-marker".into(), value: Value::Null });
        assert_eq!(Event::from_value(json!({"event":"command","id":"add-marker"})), Event::Command { id: "add-marker".into() });
        assert_eq!(Event::from_value(json!({"event":"init"})), Event::Init);
        assert_eq!(Event::from_value(json!({"event":"tool","phase":"update","at":[1,2]})), Event::Tool { phase: "update".into(), at: json!([1, 2]) });
        // A future event, and a ui event missing its revision, are kept as raw JSON, never an error.
        let future = json!({"event":"telemetry","x":1});
        assert_eq!(Event::from_value(future.clone()), Event::Unknown(future));
        let broken = json!({"event":"ui","surface":"notes","widget":"add-marker"});
        assert_eq!(Event::from_value(broken.clone()), Event::Unknown(broken));
        assert_eq!(Event::from_value(json!([1, 2])), Event::Unknown(json!([1, 2])));
    }
}

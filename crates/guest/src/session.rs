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
    /// Reserved unload notification. Beta revocation and shutdown may stop the worker
    /// immediately without delivering it; never rely on this event for cleanup.
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

std::thread_local! {
    /// The full JSON of the event being handled (wasm runs one event at a time).
    static CURRENT: std::cell::RefCell<Value> = const { std::cell::RefCell::new(Value::Null) };
}

/// Records the event being handled; the SDK's session entry calls this before your handler.
pub(crate) fn set_current(value: &Value) {
    CURRENT.with(|c| *c.borrow_mut() = value.clone());
}

/// The complete JSON of the event being handled, including fields [`Event`] does not model (for
/// example a `files.picked` answer, which arrives as [`Event::Unknown`]). `Null` outside an event.
pub fn current_event_json() -> Value {
    CURRENT.with(|c| c.borrow().clone())
}

/// The `params` object of the command event being handled (what a menu, a script or an agent
/// passed when running the command), or `Null`.
pub fn command_params() -> Value {
    CURRENT.with(|c| c.borrow().get("params").cloned().unwrap_or(Value::Null))
}

/// Details of the tool event being handled that [`Event::Tool`] does not carry. Fields an app does
/// not send stay `None` / `Null`: check the app's contract for which it provides.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToolDetails {
    /// Modifier keys, as the app sends them (for example `{"shift":true}`).
    pub modifiers: Value,
    /// Pen pressure from 0 to 1, when the app reports it.
    pub pressure: Option<f64>,
    /// The pointer button, such as `"primary"`, when the app reports it.
    pub button: Option<String>,
    /// Where the gesture began, when the app reports it.
    pub start: Value,
    /// The document time in seconds, when the app reports it (EffectCraft).
    pub time: Option<f64>,
}

/// [`ToolDetails`] of the tool event being handled.
pub fn tool_details() -> ToolDetails {
    CURRENT.with(|c| {
        let v = c.borrow();
        ToolDetails {
            modifiers: v.get("modifiers").cloned().unwrap_or(Value::Null),
            pressure: v.get("pressure").and_then(Value::as_f64),
            button: v.get("button").and_then(Value::as_str).map(str::to_owned),
            start: v.get("start").cloned().unwrap_or(Value::Null),
            time: v.get("time").and_then(Value::as_f64),
        }
    })
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
    fn the_full_event_keeps_fields_the_enum_does_not_model() {
        set_current(&json!({"event":"tool","phase":"update","at":[1,2],"modifiers":{"shift":true},"pressure":0.5,"button":"primary","start":[0,0],"time":1.5}));
        let t = tool_details();
        assert_eq!(t.modifiers, json!({"shift":true}));
        assert_eq!((t.pressure, t.button.as_deref(), t.time), (Some(0.5), Some("primary"), Some(1.5)));
        set_current(&json!({"event":"command","id":"go","params":{"count":3}}));
        assert_eq!(command_params(), json!({"count":3}));
        set_current(&json!({"event":"files.picked","request":7,"handle":"h1"}));
        assert_eq!(current_event_json()["handle"], "h1");
        assert_eq!(tool_details(), ToolDetails::default());
    }

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

//! Original placeholder copy, MIT OR Apache-2.0.
use craft_guest::{Event, json, session};
use std::sync::Mutex;
static ALL: Mutex<bool> = Mutex::new(false);
static COPY: Mutex<String> = Mutex::new(String::new());
fn copy() -> String {
    let s = COPY.lock().unwrap_or_else(|e| e.into_inner());
    if s.is_empty() { "Make room for your next idea.".into() } else { s.clone() }
}
fn call(op: &str, args: craft_guest::JsonValue) -> Result<craft_guest::JsonValue, String> {
    craft_guest::call(op, args).map_err(|e| e.to_string())
}
fn render() -> Result<(), String> {
    call(
        "ui.panel.set",
        json!({"panel":"panel","tree":{"type":"scroll","children":[
            {"type":"label","style":"heading","text":"Placeholder copy"},
            {"type":"toggle","id":"all","label":"All text on this page","value":*ALL.lock().unwrap_or_else(|e|e.into_inner())},
            {"type":"text-area","rows":4,"id":"copy","label":"Copy","value":copy()},
            {"type":"label","style":"weak","text":"Select text layers. Other layer types are preserved."},
            {"type":"button","id":"apply","label":"Fill selected text","icon":"plus"}
        ]}}),
    )?;
    Ok(())
}
fn apply() -> Result<(), String> {
    let all = *ALL.lock().unwrap_or_else(|e| e.into_inner());
    let selected = if all { call("node.find", json!({"type":"TEXT","limit":201}))? } else { call("selection.get", json!({}))? };
    let ids = selected["ids"].as_array().ok_or("selection reply missing ids")?;
    if ids.len() > 200 {
        return Err("Select at most 200 text layers".into());
    }
    for id in ids {
        let node = call("node.get", json!({"id":id}))?;
        if node["type"] == "text" {
            call("node.update", json!({"id":id,"props":{"characters":copy()}}))?;
        }
    }
    render()
}
fn handle(e: Event) -> Result<(), String> {
    match e {
        Event::Init => render(),
        Event::Command { .. } => apply(),
        Event::Ui { widget, value, .. } if widget == "all" => {
            *ALL.lock().unwrap_or_else(|e| e.into_inner()) = value.as_bool().ok_or("scope must be a boolean")?;
            render()
        }
        Event::Ui { widget, value, .. } if widget == "copy" => {
            let text = value.as_str().ok_or("copy must be text")?;
            if text.len() > 4000 {
                return Err("Copy is limited to 4000 bytes".into());
            }
            *COPY.lock().unwrap_or_else(|e| e.into_inner()) = text.into();
            render()
        }
        Event::Ui { widget, .. } if widget == "apply" => apply(),
        _ => Ok(()),
    }
}
session!(handle);

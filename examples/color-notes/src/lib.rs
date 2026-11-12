//! Color notes: a panel that lists the selected layers, and a command and button that add a note
//! marker. Marker edits need `document.edit:document`, an optional permission. When the person has
//! not granted it, the panel says so instead of failing.

use craft_guest::{CallError, Event, json, session, ui};

/// The panel id from `craft-plugin.toml`.
const PANEL: &str = "notes";
/// The button widget id and the command id.
const ADD: &str = "add-marker";
const NEED_EDIT: &str = "document.edit:document";

fn handle(event: Event) -> Result<(), String> {
    let notice = match &event {
        Event::Init => None,
        Event::Command { id } if id.as_str() == ADD => add_marker()?,
        Event::Ui { surface, widget, .. } if surface == PANEL && widget == ADD => add_marker()?,
        // Any other widget change in our panel just redraws it.
        Event::Ui { surface, .. } if surface == PANEL => None,
        // Tools, app events, dispose and anything unknown: nothing to draw.
        _ => return Ok(()),
    };
    render(notice)
}

/// Adds a marker. Returns a message for the panel when the permission is missing.
fn add_marker() -> Result<Option<String>, String> {
    match craft_guest::call("marker.add", json!({"label": "Note"})) {
        Ok(_) => Ok(None),
        Err(CallError::PermissionNeeded(_)) => Ok(Some(format!("Adding markers needs \"{NEED_EDIT}\". Turn it on in the plugin's settings."))),
        Err(e) => Err(e.to_string()),
    }
}

/// Names of the selected layers, from `selection.get` (reads `document.read:selection`).
fn selected_layers() -> Result<Vec<String>, String> {
    let reply = craft_guest::call("selection.get", json!({})).map_err(|e| e.to_string())?;
    let names =
        reply["layers"].as_array().map(|layers| layers.iter().filter_map(|l| l["name"].as_str().map(str::to_owned)).collect()).unwrap_or_default();
    Ok(names)
}

/// Draws the panel: the selection, the button, and the last notice if there is one.
fn render(notice: Option<String>) -> Result<(), String> {
    let layers = selected_layers()?;
    let mut children = vec![ui::label("Selected layers")];
    if layers.is_empty() {
        children.push(ui::label("Select one or more layers to see them here."));
    }
    children.extend(layers.iter().map(|name| ui::label(format!("• {name}"))));
    children.push(ui::separator());
    children.push(ui::button(ADD, "Add note marker"));
    if let Some(text) = notice {
        children.push(ui::label(text));
    }
    ui::set_panel(PANEL, ui::column(children)).map_err(|e| e.to_string())?;
    Ok(())
}

session!(handle);

use craft_guest::{Event, JsonValue, json, ui};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum App {
    EffectCraft,
    PhotoCraft,
    FigCraft,
}
impl App {
    pub fn from_command(id: &str) -> Option<Self> {
        match id {
            "effectcraft-apply" => Some(Self::EffectCraft),
            "photocraft-apply" => Some(Self::PhotoCraft),
            "figcraft-apply" => Some(Self::FigCraft),
            _ => None,
        }
    }
    fn panel(self) -> &'static str {
        match self {
            Self::EffectCraft => "effectcraft-panel",
            Self::PhotoCraft => "photocraft-panel",
            Self::FigCraft => "figcraft-panel",
        }
    }
    fn scope(self) -> &'static str {
        match self {
            Self::EffectCraft => "Active composition, top layer",
            Self::PhotoCraft => "Document layer stack, first layer",
            Self::FigCraft => "First page, first direct child",
        }
    }
}
fn call(op: &str, args: JsonValue) -> Result<JsonValue, String> {
    craft_guest::call(op, args).map_err(|e| e.to_string())
}
fn first(items: &JsonValue) -> Result<JsonValue, String> {
    items.as_array().and_then(|a| a.first()).cloned().ok_or_else(|| "No layers to review. Add a layer, then refresh.".into())
}
fn target(app: App) -> Result<JsonValue, String> {
    if app == App::FigCraft {
        let pages = call("page.list", json!({}))?;
        let page = pages.as_array().and_then(|a| a.first()).ok_or("No page. Create a page, then refresh.")?;
        first(&call("node.children", json!({"id":page["id"]}))?)
    } else {
        let layers = call("layer.list", json!({}))?;
        first(if layers.is_array() { &layers } else { &layers["layers"] })
    }
}
fn rename_request(app: App, layer: &JsonValue) -> (&'static str, JsonValue) {
    if app == App::FigCraft {
        ("node.update", json!({"id":layer["id"],"props":{"name":"Reviewed"}}))
    } else {
        ("layer.rename", json!({"layer":layer["id"],"name":"Reviewed"}))
    }
}
pub fn rename(app: App) -> Result<(), String> {
    let layer = target(app)?;
    let (op, args) = rename_request(app, &layer);
    call(op, args)?;
    Ok(())
}
fn tree(app: App, status: &str) -> JsonValue {
    ui::column(vec![
        ui::group("Review target", vec![ui::label(app.scope()), ui::label(status)]),
        ui::row(vec![ui::button("review-first", "Mark first layer Reviewed"), ui::button("refresh", "Refresh target")]),
        ui::label("Renames one ordinary layer. Undo in the host restores its name."),
    ])
}
fn render(app: App, status: &str) -> Result<(), String> {
    ui::set_panel(app.panel(), tree(app, status)).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn panel_event(app: App, event: Event) -> Result<(), String> {
    let result = match event {
        Event::Init => Ok(()),
        Event::Ui { surface, widget, .. } if surface == app.panel() => match widget.as_str() {
            "review-first" => rename(app),
            "refresh" => Ok(()),
            _ => return Ok(()),
        },
        _ => return Ok(()),
    };
    // A failed query/edit has no successful document mutation preceding it. Keep
    // the error visible instead of returning an error that rolls back the UI.
    let status = match result.and_then(|()| target(app)) {
        Ok(layer) => format!("First layer: {}", layer["name"].as_str().unwrap_or("Unnamed")),
        Err(error) => format!("Review unavailable: {error}"),
    };
    render(app, &status)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_identity_is_exact() {
        assert_eq!(App::from_command("figcraft-apply"), Some(App::FigCraft));
        assert_eq!(App::from_command("effectcraft-apply"), Some(App::EffectCraft));
        assert_eq!(App::from_command("photocraft-apply"), Some(App::PhotoCraft));
        assert_eq!(App::from_command("figcraft-impostor"), None);
    }
    #[test]
    fn canonical_rename_keeps_numeric_and_string_ids() {
        assert_eq!(rename_request(App::EffectCraft, &json!({"id":12})), ("layer.rename", json!({"layer":12,"name":"Reviewed"})));
        assert_eq!(rename_request(App::PhotoCraft, &json!({"id":"layer-a"})), ("layer.rename", json!({"layer":"layer-a","name":"Reviewed"})));
        assert_eq!(rename_request(App::FigCraft, &json!({"id":"node-a"})), ("node.update", json!({"id":"node-a","props":{"name":"Reviewed"}})));
        assert!(first(&json!([])).unwrap_err().contains("No layers"));
    }
    #[test]
    fn panels_share_controls_and_show_errors() {
        for app in [App::EffectCraft, App::PhotoCraft, App::FigCraft] {
            let tree = tree(app, "Permission denied");
            assert_eq!(tree["children"][0]["children"][1]["text"], "Permission denied");
            assert_eq!(tree["children"][1]["children"][0]["id"], "review-first");
            assert_eq!(tree["children"][1]["children"][1]["id"], "refresh");
        }
    }
}

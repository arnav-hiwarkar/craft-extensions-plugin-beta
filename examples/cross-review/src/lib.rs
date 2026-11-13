use craft_guest::{Event, JsonValue, json, session};
fn call(op: &str, args: JsonValue) -> Result<JsonValue, String> {
    craft_guest::call(op, args).map_err(|e| e.to_string())
}
fn handle(event: Event) -> Result<(), String> {
    if let Event::Command { id } = event {
        if id.starts_with("figcraft") {
            let pages = call("page.list", json!({}))?;
            let page = pages.as_array().and_then(|p| p.first()).ok_or("No page")?;
            let nodes = call("node.children", json!({"id":page["id"]}))?;
            let first = nodes.as_array().and_then(|p| p.first()).ok_or("No layers")?;
            call("node.update", json!({"id":first["id"],"props":{"name":"Reviewed"}}))?;
        } else {
            let layers = call("layer.list", json!({}))?;
            let list = layers.as_array().or_else(|| layers["layers"].as_array()).ok_or("No layers")?;
            let first = list.first().ok_or("No layers")?;
            call("layer.rename", json!({"layer":first["id"],"name":"Reviewed"}))?;
        }
    }
    Ok(())
}
session!(handle);

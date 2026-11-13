use craft_guest::{Event, json, session};
fn call(op: &str, args: craft_guest::JsonValue) -> Result<craft_guest::JsonValue, String> {
    craft_guest::call(op, args).map_err(|e| e.to_string())
}
fn handle(event: Event) -> Result<(), String> {
    if let Event::Ui { widget, value, .. } = &event {
        if widget == "rename" {
            let layers = call("layer.list", json!({}))?;
            if let Some(layer) = layers["layers"].as_array().and_then(|a| a.first()) {
                call("layer.rename", json!({"layer":layer["id"],"name":value}))?;
            }
        }
        if widget == "add" {
            call("marker.add", json!({"label":"Cue"}))?;
        }
    }
    if matches!(event, Event::Dispose) {
        return Ok(());
    }
    // No composition at init is normal: keep a useful empty table.
    let markers = call("marker.list", json!({})).unwrap_or(json!([]));
    let rows: Vec<_> =
        markers.as_array().into_iter().flatten().enumerate().map(|(i, m)| json!({"key":i.to_string(),"cells":[m.to_string()]})).collect();
    call(
        "ui.panel.set",
        json!({"panel":"main","tree":{"type":"tabs","id":"views","tabs":[
          {"id":"cues","label":"Cues","children":[{"type":"table","id":"markers","label":"Markers","columns":[{"label":"Cue"}],"rows":rows},{"type":"button","id":"add","label":"Add cue at playhead"},{"type":"text-input","id":"rename","label":"Rename top layer","value":""}]},
          {"id":"help","label":"Guide","children":[{"type":"canvas","id":"guide","label":"Cue guide","width":240,"height":60,"ops":[{"op":"line","from":[10,30],"to":[230,30],"width":2,"rgba":[0.3,0.7,1,1]}]}]}
        ]}}),
    )?;
    Ok(())
}
session!(handle);

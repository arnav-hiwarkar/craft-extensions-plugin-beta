use craft_guest::{Event, json, session};
fn handle(event: Event) -> Result<(), String> {
    if let Event::Tool { phase, at } = event {
        if phase == "commit" {
            let selected = craft_guest::call("layer.list", json!({})).map_err(|e| e.to_string())?;
            // Apply to the selected layers only, obtained from selection.get.
            let _ = selected;
            let selection = craft_guest::call("selection.get", json!({})).map_err(|e| e.to_string())?;
            for layer in selection["layers"].as_array().into_iter().flatten() {
                craft_guest::call("prop.addKey", json!({"layer":layer["id"],"path":"transform/position","value":at,"timeBase":"comp"}))
                    .map_err(|e| e.to_string())?;
            }
        } else if phase == "begin" || phase == "update" {
            craft_guest::call("canvas.overlay.set", json!({"ops":[{"op":"circle","center":at,"radius":5,"fill":[0.3,0.7,1,1]}]}))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
session!(handle);

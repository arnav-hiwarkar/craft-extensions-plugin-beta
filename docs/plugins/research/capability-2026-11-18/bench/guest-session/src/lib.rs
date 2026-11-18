use craft_guest::{session, Event};
fn handle(event: Event) -> Result<(), String> {
    if let Event::Command { id } = event {
        let (op, n) = id.split_once(':').ok_or("bad id")?;
        let n: u32 = n.parse().map_err(|_| "bad n")?;
        for i in 0..n {
            craft_guest::call(op, craft_guest::json!({"layer": i, "name": "Layer"})).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
session!(handle);

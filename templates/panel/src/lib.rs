//! A minimal panel with a button and a command. Edit the tree to suit.

use craft_guest::{Event, session, ui};

fn handle(event: Event) -> Result<(), String> {
    match event {
        Event::Init | Event::Ui { .. } => {
            let tree = ui::column(vec![ui::label("Hello from my plugin"), ui::button("hello", "Say hello")]);
            ui::set_panel("panel", tree).map_err(|e| e.to_string())?;
        }
        Event::Command { id } if id.as_str() == "hello" => craft_guest::log(craft_guest::LogLevel::Info, "hello"),
        _ => {}
    }
    Ok(())
}

session!(handle);

use craft_guest::{Event, session};
mod review;
fn handle(event: Event) -> Result<(), String> {
    if let Event::Command { id } = event {
        if let Some(app) = review::App::from_command(&id) {
            review::rename(app)?;
        }
    }
    Ok(())
}
session!(handle);

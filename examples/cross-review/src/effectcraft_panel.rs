use craft_guest::{Event, session};
mod review;
fn handle(event: Event) -> Result<(), String> {
    review::panel_event(review::App::EffectCraft, event)
}
session!(handle);

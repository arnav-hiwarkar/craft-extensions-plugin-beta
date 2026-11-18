use craft_guest::{filter, FilterCtx, Image};
fn blur(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let r = ctx.f("radius")? as f32;
    craft_guest::kernel::blur(img, r).map_err(|e| e.to_string())
}
filter!(blur);

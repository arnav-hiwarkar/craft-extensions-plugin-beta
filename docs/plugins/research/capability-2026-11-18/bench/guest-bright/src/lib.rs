use craft_guest::{filter, FilterCtx, Image};
fn bright(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let gain = ctx.f("gain")? as f32;
    for px in img.pixels.chunks_exact_mut(4) {
        px[0] *= gain;
        px[1] *= gain;
        px[2] *= gain;
    }
    Ok(())
}
filter!(bright);

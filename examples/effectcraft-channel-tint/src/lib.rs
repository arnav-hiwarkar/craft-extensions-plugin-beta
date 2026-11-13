use craft_guest::{FilterCtx, Image, filter};
fn tint(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let amount = ctx.f("amount")?.clamp(0.0, 1.0) as f32;
    let warm = ctx.b("warm")?;
    let protect = ctx.b("protect")?;
    let mix = ctx.b("mix")?;
    let tint = ctx.color("tint")?;
    for px in img.pixels.chunks_exact_mut(4) {
        if warm && protect && px.get(0).copied().unwrap_or(0.0) > 0.8 {
            continue;
        }
        if mix {
            let alpha = px.get(3).copied().unwrap_or(0.0);
            for (v, t) in px.iter_mut().take(3).zip(tint) {
                *v = *v * (1.0 - amount) + t * alpha * amount;
            }
        }
        let channel = if warm { 0 } else { 2 };
        if let Some(v) = px.get_mut(channel) {
            *v *= 1.0 - amount;
        }
    }
    Ok(())
}
filter!(tint);

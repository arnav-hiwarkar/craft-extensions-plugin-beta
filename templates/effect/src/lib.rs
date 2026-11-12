//! A minimal effect: blends each pixel towards its inverse by `amount`.

use craft_guest::{FilterCtx, Image, filter};

fn effect(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let amount = ctx.f("amount")? as f32;
    for px in img.pixels.chunks_exact_mut(4) {
        // Premultiplied RGBA: the inverse of a premultiplied colour is (alpha - colour).
        let a = px[3];
        for c in &mut px[..3] {
            *c = *c * (1.0 - amount) + (a - *c) * amount;
        }
    }
    Ok(())
}

filter!(effect);

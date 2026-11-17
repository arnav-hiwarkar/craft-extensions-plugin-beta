use craft_guest::kernel::{self, Buffer};
use craft_guest::{FilterCtx, Image, filter};

pub fn skin_likelihood(rgba: [f32; 4]) -> f32 {
    let a = rgba[3];
    if a <= 0.0 {
        return 0.0;
    }
    let (r, g, b) = (rgba[0] / a, rgba[1] / a, rgba[2] / a);
    let warm = (r - b).clamp(0.0, 1.0);
    let ordered = if r > g && g > b { 1.0 } else { 0.0 };
    let bright = ((r - 0.15) / 0.2).clamp(0.0, 1.0) * ((1.0 - r) / 0.05 + 0.5).clamp(0.0, 1.0);
    let band = 1.0 - ((warm - 0.3).abs() / 0.25).clamp(0.0, 1.0);
    ordered * bright * band * a
}

fn skin_mask(img: &Image<'_>) -> Buffer {
    let mut mask = Buffer::new(img.width, img.height);
    for (out, px) in mask.pixels.chunks_exact_mut(4).zip(img.pixels.chunks_exact(4)) {
        let s = skin_likelihood([px[0], px[1], px[2], px[3]]);
        out.copy_from_slice(&[s, s, s, 1.0]);
    }
    mask
}

fn fail(e: craft_guest::CallError) -> String {
    format!("Softskin needs host kernels: {e}")
}

fn softskin(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let scale = ctx.scale as f32;
    if !(scale.is_finite() && scale > 0.0) {
        return Err("the render scale must be positive".into());
    }
    let amount = ctx.f("amount")? as f32;
    if amount <= 0.0 {
        return Ok(());
    }
    let radius = (ctx.f("radius")? as f32 * scale).clamp(1.0, kernel::MAX_BILATERAL);
    let mut smooth = Buffer::copy_of(img);
    kernel::bilateral(&mut smooth.image(), radius, ctx.f("detail")? as f32).map_err(fail)?;
    if ctx.b("skin_only")? {
        let mut mask = skin_mask(img);
        kernel::blur(&mut mask.image(), radius * 0.5).map_err(fail)?;
        kernel::mix_masked(img, &smooth.image(), &mask.image(), amount).map_err(fail)
    } else {
        kernel::mix(img, &smooth.image(), amount).map_err(fail)
    }
}

filter!(softskin);

#[cfg(test)]
mod tests {
    use super::skin_likelihood;

    #[test]
    fn warm_mid_tones_read_as_skin_and_greys_and_blues_do_not() {
        assert!(skin_likelihood([0.85, 0.62, 0.5, 1.0]) > 0.5);
        assert_eq!(skin_likelihood([0.5, 0.5, 0.5, 1.0]), 0.0);
        assert_eq!(skin_likelihood([0.2, 0.3, 0.8, 1.0]), 0.0);
        assert_eq!(skin_likelihood([0.85, 0.62, 0.5, 0.0]), 0.0);
    }
}

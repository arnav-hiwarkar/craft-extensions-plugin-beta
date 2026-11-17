use craft_guest::kernel::{self, Buffer};
use craft_guest::{FilterCtx, Image, filter};

pub const SKIN_HUE: [f32; 2] = [0.0, 50.0];
pub const SKIN_SATURATION: [f32; 2] = [0.12, 0.75];
pub const SKIN_LIGHTNESS: [f32; 2] = [0.2, 0.92];

fn skin_mask(img: &Image<'_>) -> Result<Buffer, craft_guest::CallError> {
    let mut mask = Buffer { width: img.width, height: img.height, pixels: img.pixels.to_vec() };
    kernel::hsl_range(&mut mask.image(), SKIN_HUE, SKIN_SATURATION, SKIN_LIGHTNESS, 0.08)?;
    Ok(mask)
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
        let mut mask = skin_mask(img).map_err(fail)?;
        kernel::blur(&mut mask.image(), radius * 0.5).map_err(fail)?;
        kernel::mix_masked(img, &smooth.image(), &mask.image(), amount).map_err(fail)
    } else {
        kernel::mix(img, &smooth.image(), amount).map_err(fail)
    }
}

filter!(softskin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warm_mid_tones_read_as_skin_and_greys_and_blues_do_not() {
        let mut px = vec![0.85, 0.62, 0.5, 1.0, 0.5, 0.5, 0.5, 1.0, 0.2, 0.3, 0.8, 1.0];
        let img = Image { width: 3, height: 1, pixels: &mut px };
        let mask = skin_mask(&img).unwrap();
        assert!(mask.pixels[0] > 0.9);
        assert_eq!((mask.pixels[4], mask.pixels[8]), (0.0, 0.0));
    }
}

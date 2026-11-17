use craft_guest::kernel::{self, Blend, Buffer, Curves};
use craft_guest::{FilterCtx, Image, filter};

const SCALES: [(f32, f32); 3] = [(1.0, 0.5), (2.5, 0.3), (6.0, 0.2)];

pub fn highlight_curve(threshold: f32, samples: usize) -> Vec<f32> {
    let knee = threshold.clamp(0.0, 0.999);
    (0..samples)
        .map(|i| {
            let v = i as f32 / (samples - 1) as f32;
            ((v - knee) / (1.0 - knee)).max(0.0)
        })
        .collect()
}

pub fn gain_matrix(tint: [f32; 4], gain: f32) -> [f32; 20] {
    let [r, g, b, _] = tint;
    [
        r * gain, 0.0, 0.0, 0.0, 0.0,
        0.0, g * gain, 0.0, 0.0, 0.0,
        0.0, 0.0, b * gain, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0, 0.0,
    ]
}

fn fail(e: craft_guest::CallError) -> String {
    format!("Lightwell needs host kernels: {e}")
}

fn lightwell(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let scale = ctx.scale as f32;
    if !(scale.is_finite() && scale > 0.0) {
        return Err("the render scale must be positive".into());
    }
    let radius = ctx.f("radius")? as f32 * scale;
    let intensity = ctx.f("intensity")? as f32;
    if intensity <= 0.0 {
        return Ok(());
    }
    let mut bright = Buffer::copy_of(img);
    kernel::curves(&mut bright.image(), &Curves { rgb: Some(highlight_curve(ctx.f("threshold")? as f32, 256)), ..Curves::default() }).map_err(fail)?;

    let mut glow = Buffer::new(img.width, img.height);
    for (factor, weight) in SCALES {
        let mut pass = bright.clone();
        kernel::blur(&mut pass.image(), (radius * factor).min(kernel::MAX_RADIUS)).map_err(fail)?;
        kernel::color_matrix(&mut pass.image(), gain_matrix([1.0; 4], weight)).map_err(fail)?;
        kernel::blend(&mut glow.image(), &pass.image(), Blend::Add, 1.0).map_err(fail)?;
    }
    if ctx.b("rays")? {
        let mut rays = bright;
        let length = ctx.f("ray_length")? as f32 * scale;
        kernel::directional_blur(&mut rays.image(), ctx.f("ray_angle")? as f32, length.min(kernel::MAX_LENGTH)).map_err(fail)?;
        kernel::blend(&mut glow.image(), &rays.image(), Blend::Add, 1.0).map_err(fail)?;
    }
    kernel::color_matrix(&mut glow.image(), gain_matrix(ctx.color("tint")?, intensity)).map_err(fail)?;
    kernel::blend(img, &glow.image(), Blend::Add, 1.0).map_err(fail)
}

filter!(lightwell);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_highlight_curve_keeps_only_values_above_the_threshold_and_rescales_them() {
        assert_eq!(highlight_curve(0.5, 5), vec![0.0, 0.0, 0.0, 0.5, 1.0]);
    }

    fn ctx() -> FilterCtx {
        craft_guest::serde_json::from_value(craft_guest::json!({
            "params": {"threshold": 0.5, "radius": 3, "intensity": 1.0, "tint": [1, 1, 1, 1], "rays": false, "ray_angle": 90, "ray_length": 10},
            "time": 0, "scale": 1, "origin": [0, 0], "full": [9, 9], "quality": "final", "seed": 0, "color": "linear-srgb"
        }))
        .unwrap()
    }

    #[test]
    fn a_bright_point_glows_into_its_neighbours_and_dark_pixels_stay_dark() {
        let mut px = vec![0.0f32; 9 * 9 * 4];
        px.chunks_exact_mut(4).for_each(|p| p[3] = 1.0);
        px[(4 * 9 + 4) * 4..(4 * 9 + 4) * 4 + 3].copy_from_slice(&[1.0, 1.0, 1.0]);
        let mut img = Image { width: 9, height: 9, pixels: &mut px };
        lightwell(&ctx(), &mut img).unwrap();
        let red = |x: usize, y: usize| px[(y * 9 + x) * 4];
        assert!(red(5, 4) > 0.0 && red(5, 4) < red(4, 4));
        assert!(red(4, 4) > 1.0, "the source brightens: {}", red(4, 4));
        assert_eq!(px[3], 1.0);
    }

    #[test]
    fn the_gain_matrix_tints_colour_and_keeps_alpha() {
        let m = gain_matrix([1.0, 0.5, 0.25, 1.0], 2.0);
        assert_eq!((m[0], m[6], m[12], m[18]), (2.0, 1.0, 0.5, 1.0));
    }
}

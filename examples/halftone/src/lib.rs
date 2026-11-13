//! Halftone: round dots on a rotated grid, each sized by the darkness of its cell.
//!
//! Deterministic and whole-frame. Dot centres are placed in layer units from each pixel's position in
//! the full frame, so the result does not depend on how the host splits the image.

use craft_guest::{FilterCtx, Image, filter};
use std::f64::consts::PI;

/// Sanitises a host-provided channel: a non-finite value counts as zero.
fn finite(v: f32) -> f64 {
    if v.is_finite() { f64::from(v) } else { 0.0 }
}

fn halftone(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let size = ctx.f("size")?.clamp(2.0, 64.0); // grid cell, in layer pixels
    let angle = ctx.f("angle")?.to_radians(); // the manifest states degrees
    let mono = ctx.b("mono")?;
    let scale = ctx.scale;
    if !(scale.is_finite() && scale > 0.0) {
        return Err("the render scale must be positive".into());
    }
    let (w, h) = (img.width as usize, img.height as usize);
    if w == 0 || h == 0 {
        return Ok(());
    }
    let (sin, cos) = angle.sin_cos();
    let [ox, oy] = ctx.origin;
    // The grid rotates about the middle of the full frame, so every tile agrees on where dots are.
    let cx = f64::from(ctx.full[0]) * 0.5 / scale;
    let cy = f64::from(ctx.full[1]) * 0.5 / scale;
    // Dot colour and darkness come from the unmodified input.
    let src = img.pixels.to_vec();

    // Index of the input pixel nearest a layer-space point, clamped into this buffer.
    let sample = |lx: f64, ly: f64| -> usize {
        let bx = ((lx * scale) - ox - 0.5).round().clamp(0.0, (w - 1) as f64) as usize;
        let by = ((ly * scale) - oy - 0.5).round().clamp(0.0, (h - 1) as f64) as usize;
        (by * w + bx) * 4
    };

    for y in 0..h {
        for x in 0..w {
            let lx = (ox + x as f64 + 0.5) / scale;
            let ly = (oy + y as f64 + 0.5) / scale;
            // Pixel position in grid space (rotated so the grid is axis-aligned).
            let (dx, dy) = (lx - cx, ly - cy);
            let u = dx * cos + dy * sin;
            let v = -dx * sin + dy * cos;
            let (cu, cv) = ((u / size).floor(), (v / size).floor());

            // The strongest dot covering this pixel wins. Neighbouring cells are checked because a
            // large dot can spill into the next cell.
            let mut ink = 0.0_f64;
            let mut tint = [0.0_f64; 3];
            for dv in -1..=1 {
                for du in -1..=1 {
                    let (uc, vc) = ((cu + f64::from(du) + 0.5) * size, (cv + f64::from(dv) + 0.5) * size);
                    let centre = sample(cx + uc * cos - vc * sin, cy + uc * sin + vc * cos);
                    let s = &src[centre..centre + 4];
                    let a = finite(s[3]).clamp(0.0, 1.0);
                    if a == 0.0 {
                        continue;
                    }
                    // Premultiplied -> straight colour.
                    let rgb = [0, 1, 2].map(|i| (finite(s[i]) / a).clamp(0.0, 1.0));
                    let luma = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
                    // Dot area is proportional to darkness: a full cell of darkness gives a dot of
                    // area equal to the cell.
                    let darkness = (1.0 - luma) * a;
                    let radius = size * (darkness / PI).sqrt();
                    if radius <= 0.0 {
                        continue;
                    }
                    let dist = ((u - uc).powi(2) + (v - vc).powi(2)).sqrt();
                    // Anti-aliased edge: about one buffer pixel of ramp.
                    let cover = ((radius - dist) * scale + 0.5).clamp(0.0, 1.0);
                    if cover > ink {
                        ink = cover;
                        tint = rgb;
                    }
                }
            }

            let base = (y * w + x) * 4;
            let own = finite(src[base + 3]).clamp(0.0, 1.0);
            let out = if own == 0.0 {
                [0.0; 4]
            } else if mono {
                // White paper, black dots, keeping this pixel's own alpha.
                let v = ((1.0 - ink) * own) as f32;
                [v, v, v, own as f32]
            } else {
                // Dots in the cell's colour; the paper stays transparent.
                let alpha = ink * own;
                [(tint[0] * alpha) as f32, (tint[1] * alpha) as f32, (tint[2] * alpha) as f32, alpha as f32]
            };
            img.set(x as u32, y as u32, out);
        }
    }
    Ok(())
}

fn describe() -> craft_guest::Form {
    use craft_guest::{Form, Unit, t};
    Form::new("halftone", t!("Halftone"))
        .slider("size", t!("Dot size"), 2.0..=64.0, 8.0).unit(Unit::Pixels)
        .angle("angle", t!("Grid angle"), 45.0).unit(Unit::Degrees)
        .toggle("mono", t!("Black and white"), true).preview()
}

filter!(describe = describe, apply = halftone);

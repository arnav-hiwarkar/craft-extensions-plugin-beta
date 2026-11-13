use craft_guest::{FilterCtx, Image, filter};
fn apply(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let amount = ctx.f("amount")? / 100.0;
    let round = ctx.choice("shape")? == "round";
    let radius = ctx.f("radius")? * ctx.scale;
    let [w, h] = ctx.full.map(f64::from);
    if w == 0.0 || h == 0.0 {
        return Ok(());
    }
    for y in 0..img.height {
        for x in 0..img.width {
            let dx = ctx.origin[0] + x as f64 + 0.5 - w / 2.0;
            let dy = ctx.origin[1] + y as f64 + 0.5 - h / 2.0;
            let (rx, ry) = if round { (radius, radius) } else { (w / 2.0, h / 2.0) };
            let gain = (1.0 - amount * ((dx / rx).powi(2) + (dy / ry).powi(2)).min(1.0)) as f32;
            let mut p = img.get(x, y);
            for c in &mut p[..3] {
                *c *= gain;
            }
            img.set(x, y, p);
        }
    }
    Ok(())
}
filter!(apply);

#[cfg(test)]
mod tests {
    use super::*;
    fn ctx() -> FilterCtx {
        craft_guest::serde_json::from_value(craft_guest::json!({
            "params":{"amount":100,"shape":"oval","radius":100},
            "time":0,"scale":1,"origin":[0,0],"full":[3,3],
            "quality":"final","seed":0,"color":"linear-srgb"
        }))
        .unwrap()
    }
    #[test]
    fn preserves_center_alpha_and_hdr_and_agrees_on_tiles() {
        let c = ctx();
        let mut pixels = [2.0, 1.0, 0.5, 0.5].repeat(9);
        apply(&c, &mut Image { width: 3, height: 3, pixels: &mut pixels }).unwrap();
        assert_eq!(&pixels[16..20], &[2.0, 1.0, 0.5, 0.5]);
        assert!(pixels[0] < 0.3, "corners must darken");
        assert!(pixels.chunks_exact(4).all(|p| p[3] == 0.5));
        let mut tile = [2.0, 1.0, 0.5, 0.5];
        let mut tiled = ctx();
        tiled.origin = [2.0, 0.0];
        apply(&tiled, &mut Image { width: 1, height: 1, pixels: &mut tile }).unwrap();
        assert_eq!(tile, pixels[8..12]);
    }
    #[test]
    fn zero_amount_is_identity() {
        let mut c = ctx();
        c.params.insert("amount".into(), craft_guest::Value::Number(0.0));
        let mut pixels = [0.2, 0.3, 0.4, 0.5];
        let before = pixels;
        apply(&c, &mut Image { width: 1, height: 1, pixels: &mut pixels }).unwrap();
        assert_eq!(pixels, before);
    }
}

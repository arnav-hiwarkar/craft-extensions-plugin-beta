//! Transforms (`raster.filter@1`): the per-call context and the pixel buffer.

use serde::Deserialize;
use serde_json::{Map, Value};

/// The context the host sends with every filter call (beta-design B4, ABI v1).
///
/// Parameters come from the `[[contribution.param]]` table in `craft-plugin.toml`. Every declared
/// parameter is present. The typed helpers return `Err` for a missing or mistyped value, so a filter
/// can use `?` and the host reports the problem instead of rendering garbage.
#[derive(Clone, Debug, Deserialize)]
pub struct FilterCtx {
    /// Parameter values by id.
    pub params: Map<String, Value>,
    /// Layer time in seconds.
    pub time: f64,
    /// Buffer pixels per layer pixel.
    pub scale: f64,
    /// Top-left of this buffer inside the full frame, in buffer pixels.
    pub origin: [f64; 2],
    /// Size of the full frame, in buffer pixels.
    pub full: [u32; 2],
    /// `"draft"` or `"final"`. Use it only to trade speed for accuracy.
    pub quality: String,
    /// Deterministic seed for this layer and render. Derive any randomness from it.
    pub seed: u64,
    /// `"linear-srgb"` or `"srgb"`: the working space the pixel values are in.
    pub color: String,
}

impl FilterCtx {
    /// The raw JSON value of parameter `id`.
    pub fn param(&self, id: &str) -> Result<&Value, String> {
        self.params.get(id).ok_or_else(|| format!("missing parameter `{id}`"))
    }

    /// A number (slider, angle, or pixels parameter).
    pub fn f(&self, id: &str) -> Result<f64, String> {
        self.param(id)?.as_f64().ok_or_else(|| format!("parameter `{id}` is not a number"))
    }

    /// A checkbox.
    pub fn b(&self, id: &str) -> Result<bool, String> {
        self.param(id)?.as_bool().ok_or_else(|| format!("parameter `{id}` is not a checkbox value"))
    }

    /// A select parameter: the zero-based index of the chosen option.
    pub fn index(&self, id: &str) -> Result<u32, String> {
        self.param(id)?.as_u64().and_then(|i| u32::try_from(i).ok()).ok_or_else(|| format!("parameter `{id}` is not an option index"))
    }

    /// A text parameter.
    pub fn text(&self, id: &str) -> Result<&str, String> {
        self.param(id)?.as_str().ok_or_else(|| format!("parameter `{id}` is not text"))
    }

    /// A point parameter `[x, y]`.
    pub fn point(&self, id: &str) -> Result<[f64; 2], String> {
        fixed(id, self.param(id)?)
    }

    /// A colour parameter `[r, g, b, a]`.
    pub fn color(&self, id: &str) -> Result<[f64; 4], String> {
        fixed(id, self.param(id)?)
    }
}

fn fixed<const N: usize>(id: &str, v: &Value) -> Result<[f64; N], String> {
    let items = v.as_array().filter(|a| a.len() == N).ok_or_else(|| format!("parameter `{id}` needs {N} numbers"))?;
    let mut out = [0.0; N];
    for (slot, item) in out.iter_mut().zip(items) {
        *slot = item.as_f64().ok_or_else(|| format!("parameter `{id}` needs {N} numbers"))?;
    }
    Ok(out)
}

/// The pixels of one filter call: premultiplied RGBA `f32`, row-major, top row first.
///
/// Values may exceed 1.0 (HDR). The host validates the output (finite values, alpha clamped to
/// 0..=1) and refuses the result otherwise. The buffer is written back in place.
pub struct Image<'a> {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height * 4` values: R, G, B, A per pixel, premultiplied.
    pub pixels: &'a mut [f32],
}

impl Image<'_> {
    /// Index of the red channel of pixel `(x, y)` in [`Image::pixels`].
    pub fn offset(&self, x: u32, y: u32) -> usize {
        (y as usize * self.width as usize + x as usize) * 4
    }

    /// Reads pixel `(x, y)`. Panics if it is outside the image (the host then reports a trap).
    pub fn get(&self, x: u32, y: u32) -> [f32; 4] {
        let o = self.offset(x, y);
        [self.pixels[o], self.pixels[o + 1], self.pixels[o + 2], self.pixels[o + 3]]
    }

    /// Writes pixel `(x, y)`. Panics if it is outside the image.
    pub fn set(&mut self, x: u32, y: u32, rgba: [f32; 4]) {
        let o = self.offset(x, y);
        self.pixels[o..o + 4].copy_from_slice(&rgba);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctx() -> FilterCtx {
        serde_json::from_value(json!({
            "params": {"size": 8, "angle": 45.5, "mono": true, "mode": 2, "label": "x", "at": [3, 4.5], "tint": [1, 0.5, 0, 1]},
            "time": 1.25, "scale": 2.0, "origin": [0, 16], "full": [640, 480],
            "quality": "final", "seed": 7, "color": "linear-srgb"
        }))
        .expect("valid context")
    }

    #[test]
    fn typed_param_helpers_check_type_and_presence() {
        let c = ctx();
        assert_eq!(c.f("size"), Ok(8.0));
        assert_eq!(c.f("angle"), Ok(45.5));
        assert_eq!(c.b("mono"), Ok(true));
        assert_eq!(c.index("mode"), Ok(2));
        assert_eq!(c.text("label"), Ok("x"));
        assert_eq!(c.point("at"), Ok([3.0, 4.5]));
        assert_eq!(c.color("tint"), Ok([1.0, 0.5, 0.0, 1.0]));
        assert!(c.f("mono").is_err(), "a checkbox is not a number");
        assert!(c.b("missing").unwrap_err().contains("missing parameter `missing`"));
        assert!(c.point("size").is_err(), "a point needs two numbers");
        assert!(c.color("at").is_err(), "a colour needs four numbers");
    }
}

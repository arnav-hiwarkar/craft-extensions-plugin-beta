//! Host compute kernels: fast, multi-threaded image work the app runs on pixels in your module's
//! memory. They need no permission and work in effects, sessions and background jobs. Prefer them
//! to per-pixel loops in Wasm: a 1080p Gaussian blur in a kernel takes milliseconds.
//!
//! Every image is premultiplied RGBA `f32`. A kernel that writes changes the image in place.

use crate::filter::Image;
use crate::host::CallError;
#[cfg(target_arch = "wasm32")]
use crate::host::call;
use serde_json::{Value, json};

/// Largest blur, morphology and splat radius in pixels; larger values are refused.
pub const MAX_RADIUS: f32 = 512.0;
/// Longest directional blur in pixels; longer values are refused.
pub const MAX_LENGTH: f32 = 2048.0;

/// An image your code owns, for intermediate results (a mask, a glow pass, a downscaled copy).
#[derive(Clone, Debug, PartialEq)]
pub struct Buffer {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height * 4` values, premultiplied RGBA.
    pub pixels: Vec<f32>,
}

impl Buffer {
    /// A transparent image.
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, pixels: vec![0.0; width as usize * height as usize * 4] }
    }
    /// A copy of `image`.
    pub fn copy_of(image: &Image<'_>) -> Self {
        Self { width: image.width, height: image.height, pixels: image.pixels.to_vec() }
    }
    /// Borrows the buffer as an [`Image`] for kernels and pixel access.
    pub fn image(&mut self) -> Image<'_> {
        Image { width: self.width, height: self.height, pixels: &mut self.pixels }
    }
}

#[cfg(target_arch = "wasm32")]
fn at(img: &Image<'_>) -> Value {
    json!({ "ptr": img.pixels.as_ptr() as usize, "width": img.width, "height": img.height })
}

#[cfg(target_arch = "wasm32")]
fn dispatch(op: &str, mut args: Value, img: &mut Image<'_>, others: &[&Image<'_>], splats: &[f32]) -> Result<Value, CallError> {
    let names = craft_ext_kernels::lookup(op).map_or(&[][..], |s| s.reads);
    args["image"] = at(img);
    for (name, other) in names.iter().skip(1).zip(others) {
        args[*name] = at(other);
    }
    if op == "kernel.splat" {
        args["points"] = json!({ "ptr": splats.as_ptr() as usize, "count": splats.len() / 8 });
    }
    let out = call(op, args);
    std::hint::black_box((&mut img.pixels, splats));
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn dispatch(op: &str, args: Value, img: &mut Image<'_>, others: &[&Image<'_>], splats: &[f32]) -> Result<Value, CallError> {
    let copy = |i: &Image<'_>| craft_ext_kernels::Image { width: i.width as usize, height: i.height as usize, px: i.pixels.to_vec() };
    let mut images: Vec<_> = std::iter::once(copy(img)).chain(others.iter().map(|o| copy(o))).collect();
    let out = craft_ext_kernels::run(op, &args, &mut images, splats).map_err(|why| CallError::Invalid(format!("{op}: {why}")))?;
    if craft_ext_kernels::lookup(op).is_some_and(|s| s.writes) {
        for (dst, src) in img.pixels.iter_mut().zip(&images[0].px) {
            *dst = if src.is_finite() { *src } else { 0.0 };
        }
    }
    Ok(out)
}

fn run(op: &str, args: Value, img: &mut Image<'_>) -> Result<Value, CallError> {
    dispatch(op, args, img, &[], &[])
}

fn run_with(op: &str, args: Value, img: &mut Image<'_>, others: &[&Image<'_>]) -> Result<Value, CallError> {
    dispatch(op, args, img, others, &[])
}

/// Gaussian blur; `radius` is about three standard deviations, 0..=512 pixels.
pub fn blur(img: &mut Image<'_>, radius: f32) -> Result<(), CallError> {
    run("kernel.blur", json!({ "radius": radius, "kind": "gaussian" }), img).map(drop)
}

/// Box (mean) blur over a `2·radius + 1` square.
pub fn box_blur(img: &mut Image<'_>, radius: f32) -> Result<(), CallError> {
    run("kernel.blur", json!({ "radius": radius, "kind": "box" }), img).map(drop)
}

/// Motion blur along `angle` degrees over `length` pixels (0..=2048).
pub fn directional_blur(img: &mut Image<'_>, angle: f32, length: f32) -> Result<(), CallError> {
    run("kernel.directional_blur", json!({ "angle": angle, "length": length }), img).map(drop)
}

/// Convolution with an odd `size`×`size` kernel (size ≤ 15), divided by `divisor` plus `bias`.
pub fn convolve(img: &mut Image<'_>, size: u32, weights: &[f32], divisor: f32, bias: f32) -> Result<(), CallError> {
    run("kernel.convolve", json!({ "size": size, "weights": weights, "divisor": divisor, "bias": bias }), img).map(drop)
}

/// Grows bright and opaque areas by `radius` pixels (max filter).
pub fn dilate(img: &mut Image<'_>, radius: f32) -> Result<(), CallError> {
    run("kernel.morphology", json!({ "radius": radius, "op": "dilate" }), img).map(drop)
}

/// Shrinks bright and opaque areas by `radius` pixels (min filter).
pub fn erode(img: &mut Image<'_>, radius: f32) -> Result<(), CallError> {
    run("kernel.morphology", json!({ "radius": radius, "op": "erode" }), img).map(drop)
}

/// Largest bilateral radius in pixels.
pub const MAX_BILATERAL: f32 = 32.0;

/// Edge-preserving smoothing: neighbours whose colour differs by more than about `tolerance`
/// (0..1 straight colour) barely count, so edges stay sharp. Skin smoothing, denoise, cartoon looks.
pub fn bilateral(img: &mut Image<'_>, radius: f32, tolerance: f32) -> Result<(), CallError> {
    run("kernel.bilateral", json!({ "radius": radius, "tolerance": tolerance }), img).map(drop)
}

/// Moves `img` toward `src` (same size) by `amount` 0..=1.
pub fn mix(img: &mut Image<'_>, src: &Image<'_>, amount: f32) -> Result<(), CallError> {
    run_with("kernel.mix", json!({ "amount": amount }), img, &[src]).map(drop)
}

/// Like [`mix`], weighted per pixel by `mask`'s luminance: white applies `amount`, black keeps `img`.
pub fn mix_masked(img: &mut Image<'_>, src: &Image<'_>, mask: &Image<'_>, amount: f32) -> Result<(), CallError> {
    run_with("kernel.mix_masked", json!({ "amount": amount }), img, &[src, mask]).map(drop)
}

/// A 4×5 colour matrix on straight (unpremultiplied) RGBA: rows r, g, b, a of `[r, g, b, a, offset]`.
pub fn color_matrix(img: &mut Image<'_>, matrix: [f32; 20]) -> Result<(), CallError> {
    run("kernel.color_matrix", json!({ "matrix": matrix }), img).map(drop)
}

/// Sampled curves: each is 2..=4096 evenly spaced output values over inputs 0..1. `rgb` applies to
/// all three colour channels first, then the per-channel curves.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Curves {
    /// Master curve for red, green and blue.
    pub rgb: Option<Vec<f32>>,
    /// Red.
    pub r: Option<Vec<f32>>,
    /// Green.
    pub g: Option<Vec<f32>>,
    /// Blue.
    pub b: Option<Vec<f32>>,
    /// Alpha.
    pub a: Option<Vec<f32>>,
}

/// Applies [`Curves`] to straight colour.
pub fn curves(img: &mut Image<'_>, c: &Curves) -> Result<(), CallError> {
    let mut args = json!({});
    for (k, v) in [("rgb", &c.rgb), ("r", &c.r), ("g", &c.g), ("b", &c.b), ("a", &c.a)] {
        if let Some(v) = v {
            args[k] = json!(v);
        }
    }
    run("kernel.curves", args, img).map(drop)
}

/// Separable blend modes for [`blend`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blend {
    /// Source over.
    Normal,
    /// Linear dodge.
    Add,
    /// Screen.
    Screen,
    /// Multiply.
    Multiply,
    /// Overlay.
    Overlay,
    /// Soft light (W3C).
    SoftLight,
    /// Lighten.
    Lighten,
    /// Darken.
    Darken,
    /// Difference.
    Difference,
}

impl Blend {
    fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Add => "add",
            Self::Screen => "screen",
            Self::Multiply => "multiply",
            Self::Overlay => "overlay",
            Self::SoftLight => "soft-light",
            Self::Lighten => "lighten",
            Self::Darken => "darken",
            Self::Difference => "difference",
        }
    }
}

/// Composites `src` (same size) over `img` with `mode` at `opacity` 0..=1.
pub fn blend(img: &mut Image<'_>, src: &Image<'_>, mode: Blend, opacity: f32) -> Result<(), CallError> {
    run_with("kernel.blend", json!({ "mode": mode.name(), "opacity": opacity }), img, &[src]).map(drop)
}

/// Fractal value noise settings for [`noise`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Noise {
    /// Feature size in pixels.
    pub scale: f32,
    /// 1..=12 octaves.
    pub octaves: u32,
    /// Seed; the same seed gives the same noise.
    pub seed: u32,
    /// Third coordinate, for evolving noise over time.
    pub z: f32,
    /// Multiply into the image instead of filling it with opaque grey.
    pub multiply: bool,
}

/// Fills or multiplies `img` with deterministic fractal noise.
pub fn noise(img: &mut Image<'_>, n: Noise) -> Result<(), CallError> {
    let mode = if n.multiply { "multiply" } else { "fill" };
    run("kernel.noise", json!({ "scale": n.scale, "octaves": n.octaves, "seed": n.seed, "z": n.z, "mode": mode }), img).map(drop)
}

/// Resamples `src` into `dst` (any sizes), averaging when shrinking.
pub fn resample(src: &Image<'_>, dst: &mut Image<'_>) -> Result<(), CallError> {
    run_with("kernel.resample", json!({}), dst, &[src]).map(drop)
}

/// Fills `dst` with `src` moved by the affine `[a, b, c, d, e, f]` (x' = a·x + b·y + c, y' = d·x + e·y + f).
pub fn warp_affine(src: &Image<'_>, dst: &mut Image<'_>, matrix: [f32; 6]) -> Result<(), CallError> {
    run_with("kernel.warp_affine", json!({ "matrix": matrix }), dst, &[src]).map(drop)
}

/// One soft disc for [`splat`]: particles, sparks, bokeh, light leaks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Splat {
    /// Centre x in pixels.
    pub x: f32,
    /// Centre y in pixels.
    pub y: f32,
    /// Radius in pixels.
    pub radius: f32,
    /// Straight RGBA; alpha scales the disc's strength.
    pub color: [f32; 4],
    /// 0 = soft falloff from the centre, 1 = hard edge.
    pub hardness: f32,
}

/// Adds up to a million soft discs to `img` as light (premultiplied add).
pub fn splat(img: &mut Image<'_>, splats: &[Splat]) -> Result<(), CallError> {
    let flat: Vec<f32> = splats.iter().flat_map(|s| [s.x, s.y, s.radius, s.color[0], s.color[1], s.color[2], s.color[3], s.hardness]).collect();
    dispatch("kernel.splat", json!({}), img, &[], &flat).map(drop)
}

/// 256-bin histograms of straight colour and alpha.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize)]
pub struct Histogram {
    /// Red.
    pub r: Vec<u64>,
    /// Green.
    pub g: Vec<u64>,
    /// Blue.
    pub b: Vec<u64>,
    /// Alpha.
    pub a: Vec<u64>,
}

/// Counts `img`'s values (it is not changed).
pub fn histogram(img: &mut Image<'_>) -> Result<Histogram, CallError> {
    let v = run("kernel.histogram", json!({}), img)?;
    serde_json::from_value(v).map_err(|e| CallError::Invalid(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernels_run_natively_in_author_tests() {
        let mut base = Buffer::new(2, 1);
        base.pixels.copy_from_slice(&[0.2, 0.4, 0.6, 1.0, 0.2, 0.4, 0.6, 1.0]);
        let mut white = Buffer::new(2, 1);
        white.pixels.fill(1.0);
        let expected = base.pixels.clone();
        blend(&mut base.image(), &white.image(), Blend::Multiply, 1.0).unwrap();
        assert_eq!(base.pixels, expected);
        splat(&mut base.image(), &[Splat { x: 0.5, y: 0.5, radius: 0.0, color: [1.0; 4], hardness: 1.0 }]).unwrap();
        assert!(matches!(blur(&mut base.image(), 900.0), Err(CallError::Invalid(m)) if m == "kernel.blur: radius must be 0..=512"));
    }
}

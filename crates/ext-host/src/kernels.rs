
use serde_json::{Value, json};

pub const MAX_SIDE: u32 = 32_768;
pub const MAX_RADIUS: f32 = 512.0;
pub const MAX_CONVOLVE: usize = 15;
pub const MAX_SPLATS: usize = 1_000_000;
pub const MAX_CURVE: usize = 4096;

pub fn cost(op: &str, args: &Value, pixels: u64) -> u64 {
    let radius = args.get("radius").and_then(Value::as_f64).unwrap_or(1.0).clamp(1.0, MAX_RADIUS as f64) as u64;
    let per_pixel = match op {
        "kernel.blur" | "kernel.morphology" => 4 * radius,
        "kernel.directional_blur" => args.get("length").and_then(Value::as_f64).unwrap_or(1.0).clamp(1.0, 2048.0) as u64,
        "kernel.convolve" => args.get("size").and_then(Value::as_u64).unwrap_or(3).min(MAX_CONVOLVE as u64).pow(2),
        "kernel.noise" => 8 * args.get("octaves").and_then(Value::as_u64).unwrap_or(1).clamp(1, 12),
        _ => 4,
    };
    pixels.saturating_mul(per_pixel).saturating_add(10_000)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub px: Vec<f32>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, px: vec![0.0; width * height * 4] }
    }
    fn at(&self, x: isize, y: isize) -> [f32; 4] {
        let x = x.clamp(0, self.width as isize - 1) as usize;
        let y = y.clamp(0, self.height as isize - 1) as usize;
        let i = (y * self.width + x) * 4;
        [self.px[i], self.px[i + 1], self.px[i + 2], self.px[i + 3]]
    }
    fn sample(&self, x: f32, y: f32) -> [f32; 4] {
        if self.width == 0 || self.height == 0 || !x.is_finite() || !y.is_finite() {
            return [0.0; 4];
        }
        let (x, y) = (x - 0.5, y - 0.5);
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let (x0, y0) = (x0 as isize, y0 as isize);
        let mut out = [0.0; 4];
        for (dx, dy, w) in [(0, 0, (1.0 - fx) * (1.0 - fy)), (1, 0, fx * (1.0 - fy)), (0, 1, (1.0 - fx) * fy), (1, 1, fx * fy)] {
            let p = self.at(x0 + dx, y0 + dy);
            for c in 0..4 {
                out[c] += p[c] * w;
            }
        }
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageRef {
    pub ptr: usize,
    pub width: usize,
    pub height: usize,
}

impl ImageRef {
    pub fn parse(v: Option<&Value>, what: &str) -> Result<Self, String> {
        let v = v.ok_or_else(|| format!("{what} is required"))?;
        let field = |k: &str| v.get(k).and_then(Value::as_u64).ok_or_else(|| format!("{what}.{k} must be a whole number"));
        let (ptr, width, height) = (field("ptr")?, field("width")?, field("height")?);
        if width == 0 || height == 0 || width > MAX_SIDE as u64 || height > MAX_SIDE as u64 {
            return Err(format!("{what} must be 1..={MAX_SIDE} pixels on each side"));
        }
        Ok(Self { ptr: usize::try_from(ptr).map_err(|_| format!("{what}.ptr is out of range"))?, width: width as usize, height: height as usize })
    }
    pub fn bytes(&self) -> Option<usize> {
        self.width.checked_mul(self.height)?.checked_mul(16)
    }
}

fn threads() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get()).min(8)
}

fn rows(out: &mut [f32], width: usize, f: impl Fn(usize, &mut [f32]) + Sync) {
    let stride = width * 4;
    if stride == 0 {
        return;
    }
    let height = out.len() / stride;
    let per = height.div_ceil(threads()).max(1);
    std::thread::scope(|s| {
        for (band, chunk) in out.chunks_mut(per * stride).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (i, row) in chunk.chunks_mut(stride).enumerate() {
                    f(band * per + i, row);
                }
            });
        }
    });
}

fn num(args: &Value, k: &str, default: f32) -> f32 {
    args.get(k).and_then(Value::as_f64).map_or(default, |v| v as f32)
}

fn floats(args: &Value, k: &str, max: usize) -> Result<Option<Vec<f32>>, String> {
    let Some(v) = args.get(k) else { return Ok(None) };
    let a = v.as_array().ok_or_else(|| format!("{k} must be a list of numbers"))?;
    if a.len() > max {
        return Err(format!("{k} has more than {max} values"));
    }
    a.iter().map(|x| x.as_f64().map(|f| f as f32).filter(|f| f.is_finite()).ok_or_else(|| format!("{k} must hold finite numbers"))).collect::<Result<Vec<_>, _>>().map(Some)
}

fn gaussian_weights(radius: f32) -> Vec<f32> {
    let r = radius.ceil() as isize;
    let sigma = (radius / 3.0).max(0.3);
    let mut w: Vec<f32> = (-r..=r).map(|i| (-(i * i) as f32 / (2.0 * sigma * sigma)).exp()).collect();
    let sum: f32 = w.iter().sum();
    w.iter_mut().for_each(|v| *v /= sum);
    w
}

fn separable(img: &mut Image, weights: &[f32]) {
    let r = (weights.len() / 2) as isize;
    let src = img.clone();
    rows(&mut img.px, img.width, |y, row| {
        for x in 0..src.width {
            let mut acc = [0.0f32; 4];
            for (k, w) in weights.iter().enumerate() {
                let p = src.at(x as isize + k as isize - r, y as isize);
                for c in 0..4 {
                    acc[c] += p[c] * w;
                }
            }
            row[x * 4..x * 4 + 4].copy_from_slice(&acc);
        }
    });
    let src = img.clone();
    rows(&mut img.px, img.width, |y, row| {
        for x in 0..src.width {
            let mut acc = [0.0f32; 4];
            for (k, w) in weights.iter().enumerate() {
                let p = src.at(x as isize, y as isize + k as isize - r);
                for c in 0..4 {
                    acc[c] += p[c] * w;
                }
            }
            row[x * 4..x * 4 + 4].copy_from_slice(&acc);
        }
    });
}

pub fn blur(img: &mut Image, radius: f32, kind: &str) -> Result<(), String> {
    if !(0.0..=MAX_RADIUS).contains(&radius) {
        return Err(format!("radius must be 0..={MAX_RADIUS}"));
    }
    if radius < 0.5 {
        return Ok(());
    }
    match kind {
        "gaussian" if radius <= 8.0 => separable(img, &gaussian_weights(radius)),
        "gaussian" => {
            for r in box_radii_for_gaussian(radius / 3.0, 3) {
                box_blur(img, r);
            }
        }
        "box" => box_blur(img, radius.round() as usize),
        _ => return Err("kind must be gaussian or box".into()),
    }
    Ok(())
}

fn box_radii_for_gaussian(sigma: f32, passes: usize) -> Vec<usize> {
    let n = passes as f32;
    let ideal = (12.0 * sigma * sigma / n + 1.0).sqrt();
    let mut lower = ideal.floor() as i64;
    if lower % 2 == 0 {
        lower -= 1;
    }
    let lower = lower.max(1) as f32;
    let m = ((12.0 * sigma * sigma - n * lower * lower - 4.0 * n * lower - 3.0 * n) / (-4.0 * lower - 4.0)).round();
    (0..passes).map(|i| if (i as f32) < m { lower } else { lower + 2.0 }).map(|w| ((w - 1.0) / 2.0) as usize).collect()
}

fn box_rows(img: &mut Image, r: usize) {
    let width = img.width;
    let inv = 1.0 / (2 * r + 1) as f32;
    rows(&mut img.px, width, |_, row| {
        let src = row.to_vec();
        let last = width as isize - 1;
        let at = |x: isize| x.clamp(0, last) as usize * 4;
        let mut sum = [0.0f32; 4];
        for i in -(r as isize)..=(r as isize) {
            for c in 0..4 {
                sum[c] += src[at(i) + c];
            }
        }
        for x in 0..width {
            for c in 0..4 {
                row[x * 4 + c] = sum[c] * inv;
            }
            let (add, sub) = (at(x as isize + r as isize + 1), at(x as isize - r as isize));
            for c in 0..4 {
                sum[c] += src[add + c] - src[sub + c];
            }
        }
    });
}

fn transpose(img: &Image) -> Image {
    let mut out = Image::new(img.height, img.width);
    let (w, h) = (img.width, img.height);
    rows(&mut out.px, h, |x, row| {
        for y in 0..h {
            let i = (y * w + x) * 4;
            row[y * 4..y * 4 + 4].copy_from_slice(&img.px[i..i + 4]);
        }
    });
    out
}

fn box_blur(img: &mut Image, r: usize) {
    if r == 0 {
        return;
    }
    box_rows(img, r);
    let mut t = transpose(img);
    box_rows(&mut t, r);
    *img = transpose(&t);
}

pub fn directional_blur(img: &mut Image, angle_deg: f32, length: f32) -> Result<(), String> {
    if !(0.0..=2048.0).contains(&length) || !angle_deg.is_finite() {
        return Err("length must be 0..=2048 and angle finite".into());
    }
    let steps = length.ceil().max(1.0) as usize;
    let (dx, dy) = (angle_deg.to_radians().cos(), angle_deg.to_radians().sin());
    let src = img.clone();
    rows(&mut img.px, img.width, |y, row| {
        for x in 0..src.width {
            let mut acc = [0.0f32; 4];
            for s in 0..=steps {
                let t = if steps == 0 { 0.0 } else { s as f32 / steps as f32 - 0.5 } * length;
                let p = src.sample(x as f32 + 0.5 + dx * t, y as f32 + 0.5 + dy * t);
                for c in 0..4 {
                    acc[c] += p[c];
                }
            }
            for c in 0..4 {
                row[x * 4 + c] = acc[c] / (steps + 1) as f32;
            }
        }
    });
    Ok(())
}

pub fn convolve(img: &mut Image, size: usize, weights: &[f32], divisor: f32, bias: f32) -> Result<(), String> {
    if size.is_multiple_of(2) || size == 0 || size > MAX_CONVOLVE || weights.len() != size * size {
        return Err(format!("size must be odd and at most {MAX_CONVOLVE}, with size×size weights"));
    }
    if divisor == 0.0 || !divisor.is_finite() {
        return Err("divisor must be non-zero".into());
    }
    let r = (size / 2) as isize;
    let src = img.clone();
    rows(&mut img.px, img.width, |y, row| {
        for x in 0..src.width {
            let mut acc = [0.0f32; 4];
            for ky in 0..size {
                for kx in 0..size {
                    let w = weights[ky * size + kx];
                    let p = src.at(x as isize + kx as isize - r, y as isize + ky as isize - r);
                    for c in 0..4 {
                        acc[c] += p[c] * w;
                    }
                }
            }
            for c in 0..4 {
                row[x * 4 + c] = acc[c] / divisor + bias;
            }
        }
    });
    clamp_premultiplied(img);
    Ok(())
}

pub fn morphology(img: &mut Image, radius: f32, dilate: bool) -> Result<(), String> {
    if !(0.0..=MAX_RADIUS).contains(&radius) {
        return Err(format!("radius must be 0..={MAX_RADIUS}"));
    }
    let r = radius.round() as isize;
    if r == 0 {
        return Ok(());
    }
    let pick = |a: f32, b: f32| if dilate { a.max(b) } else { a.min(b) };
    for horizontal in [true, false] {
        let src = img.clone();
        rows(&mut img.px, img.width, |y, row| {
            for x in 0..src.width {
                let mut acc = src.at(x as isize, y as isize);
                for k in -r..=r {
                    let p = if horizontal { src.at(x as isize + k, y as isize) } else { src.at(x as isize, y as isize + k) };
                    for c in 0..4 {
                        acc[c] = pick(acc[c], p[c]);
                    }
                }
                row[x * 4..x * 4 + 4].copy_from_slice(&acc);
            }
        });
    }
    Ok(())
}

fn per_pixel(img: &mut Image, f: impl Fn([f32; 4]) -> [f32; 4] + Sync) {
    let width = img.width;
    rows(&mut img.px, width, |_, row| {
        for p in row.chunks_exact_mut(4) {
            let a = p[3];
            let straight = if a > 0.0 { [p[0] / a, p[1] / a, p[2] / a, a] } else { [0.0, 0.0, 0.0, 0.0] };
            let o = f(straight);
            let oa = o[3].clamp(0.0, 1.0);
            p.copy_from_slice(&[o[0] * oa, o[1] * oa, o[2] * oa, oa]);
        }
    });
}

pub fn color_matrix(img: &mut Image, m: &[f32]) -> Result<(), String> {
    if m.len() != 20 {
        return Err("matrix must have 20 values (4 rows of r, g, b, a, offset)".into());
    }
    per_pixel(img, |p| {
        let mut o = [0.0; 4];
        for (r, out) in o.iter_mut().enumerate() {
            *out = m[r * 5] * p[0] + m[r * 5 + 1] * p[1] + m[r * 5 + 2] * p[2] + m[r * 5 + 3] * p[3] + m[r * 5 + 4];
        }
        o
    });
    Ok(())
}

fn lookup(curve: &[f32], v: f32) -> f32 {
    let n = curve.len();
    if n < 2 {
        return v;
    }
    let t = v.clamp(0.0, 1.0) * (n - 1) as f32;
    let i = (t.floor() as usize).min(n - 2);
    let f = t - i as f32;
    curve[i] * (1.0 - f) + curve[i + 1] * f
}

pub fn curves(img: &mut Image, args: &Value) -> Result<(), String> {
    let mut list = Vec::new();
    for k in ["rgb", "r", "g", "b", "a"] {
        let c = floats(args, k, MAX_CURVE)?;
        if c.as_ref().is_some_and(|c| c.len() < 2) {
            return Err(format!("{k} needs at least 2 values"));
        }
        list.push(c);
    }
    let [rgb, r, g, b, a]: [Option<Vec<f32>>; 5] = list.try_into().map_err(|_| "internal")?;
    per_pixel(img, |p| {
        let mut o = p;
        if let Some(c) = &rgb {
            for v in o.iter_mut().take(3) {
                *v = lookup(c, *v);
            }
        }
        for (i, c) in [&r, &g, &b, &a].into_iter().enumerate() {
            if let Some(c) = c {
                o[i] = lookup(c, o[i]);
            }
        }
        o
    });
    Ok(())
}

pub const BLEND_MODES: &[&str] = &["normal", "add", "screen", "multiply", "overlay", "soft-light", "lighten", "darken", "difference"];

fn blend_channel(mode: &str, b: f32, s: f32) -> f32 {
    match mode {
        "add" => b + s,
        "screen" => b + s - b * s,
        "multiply" => b * s,
        "overlay" => {
            if b <= 0.5 {
                2.0 * b * s
            } else {
                1.0 - 2.0 * (1.0 - b) * (1.0 - s)
            }
        }
        "soft-light" => {
            if s <= 0.5 {
                b - (1.0 - 2.0 * s) * b * (1.0 - b)
            } else {
                let d = if b <= 0.25 { ((16.0 * b - 12.0) * b + 4.0) * b } else { b.sqrt() };
                b + (2.0 * s - 1.0) * (d - b)
            }
        }
        "lighten" => b.max(s),
        "darken" => b.min(s),
        "difference" => (b - s).abs(),
        _ => s,
    }
}

pub fn blend(img: &mut Image, src: &Image, mode: &str, opacity: f32) -> Result<(), String> {
    if !BLEND_MODES.contains(&mode) {
        return Err(format!("mode must be one of {}", BLEND_MODES.join(", ")));
    }
    if src.width != img.width || src.height != img.height {
        return Err("src must be the same size as image".into());
    }
    let opacity = opacity.clamp(0.0, 1.0);
    let width = img.width;
    rows(&mut img.px, width, |y, row| {
        for x in 0..width {
            let i = (y * width + x) * 4;
            let d = &mut row[x * 4..x * 4 + 4];
            let s = &src.px[i..i + 4];
            let (ab, as_) = (d[3], s[3] * opacity);
            let unb = |c: f32, a: f32| if a > 0.0 { c / a } else { 0.0 };
            for c in 0..3 {
                let (b, sc) = (unb(d[c], ab), unb(s[c], s[3]));
                let mixed = blend_channel(mode, b, sc);
                d[c] = (1.0 - as_) * d[c] + as_ * ((1.0 - ab) * sc + ab * mixed);
            }
            d[3] = as_ + ab * (1.0 - as_);
        }
    });
    clamp_premultiplied(img);
    Ok(())
}

fn hash(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841) ^ (z as u32).wrapping_mul(0xcb1a_b31f) ^ seed.wrapping_mul(0x9e37_79b9);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 0x00ff_ffff as f32
}

fn value_noise(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let (x0, y0, z0) = (x.floor(), y.floor(), z.floor());
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (fx, fy, fz) = (s(x - x0), s(y - y0), s(z - z0));
    let (xi, yi, zi) = (x0 as i32, y0 as i32, z0 as i32);
    let mut v = 0.0;
    for (dz, wz) in [(0, 1.0 - fz), (1, fz)] {
        for (dy, wy) in [(0, 1.0 - fy), (1, fy)] {
            for (dx, wx) in [(0, 1.0 - fx), (1, fx)] {
                v += hash(xi + dx, yi + dy, zi + dz, seed) * wx * wy * wz;
            }
        }
    }
    v
}

pub fn noise(img: &mut Image, scale: f32, octaves: u32, seed: u32, z: f32, multiply: bool) -> Result<(), String> {
    if !(scale.is_finite() && scale > 0.0) || !(1..=12).contains(&octaves) || !z.is_finite() {
        return Err("scale must be positive and octaves 1..=12".into());
    }
    let width = img.width;
    rows(&mut img.px, width, |y, row| {
        for x in 0..width {
            let (mut amp, mut freq, mut sum, mut norm) = (1.0, 1.0 / scale, 0.0, 0.0);
            for o in 0..octaves {
                sum += amp * value_noise(x as f32 * freq, y as f32 * freq, z + o as f32 * 17.0, seed);
                norm += amp;
                amp *= 0.5;
                freq *= 2.0;
            }
            let n = sum / norm;
            let p = &mut row[x * 4..x * 4 + 4];
            if multiply {
                for c in p.iter_mut().take(3) {
                    *c *= n;
                }
            } else {
                p.copy_from_slice(&[n, n, n, 1.0]);
            }
        }
    });
    Ok(())
}

pub fn resample(src: &Image, dst: &mut Image) {
    let (sx, sy) = (src.width as f32 / dst.width as f32, src.height as f32 / dst.height as f32);
    let (tx, ty) = (sx.ceil().max(1.0) as usize, sy.ceil().max(1.0) as usize);
    let width = dst.width;
    rows(&mut dst.px, width, |y, row| {
        for x in 0..width {
            let mut acc = [0.0f32; 4];
            for j in 0..ty {
                for i in 0..tx {
                    let p = src.sample((x as f32 + (i as f32 + 0.5) / tx as f32) * sx, (y as f32 + (j as f32 + 0.5) / ty as f32) * sy);
                    for c in 0..4 {
                        acc[c] += p[c];
                    }
                }
            }
            let n = (tx * ty) as f32;
            for c in 0..4 {
                row[x * 4 + c] = acc[c] / n;
            }
        }
    });
}

pub fn warp_affine(src: &Image, dst: &mut Image, m: &[f32]) -> Result<(), String> {
    if m.len() != 6 {
        return Err("matrix must have 6 values [a, b, c, d, e, f]".into());
    }
    let det = m[0] * m[4] - m[1] * m[3];
    if det.abs() < 1e-9 {
        return Err("matrix is not invertible".into());
    }
    let inv = [m[4] / det, -m[1] / det, (m[1] * m[5] - m[4] * m[2]) / det, -m[3] / det, m[0] / det, (m[3] * m[2] - m[0] * m[5]) / det];
    let width = dst.width;
    rows(&mut dst.px, width, |y, row| {
        for x in 0..width {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let (u, v) = (inv[0] * px + inv[1] * py + inv[2], inv[3] * px + inv[4] * py + inv[5]);
            let inside = u >= 0.0 && v >= 0.0 && u <= src.width as f32 && v <= src.height as f32;
            let p = if inside { src.sample(u, v) } else { [0.0; 4] };
            row[x * 4..x * 4 + 4].copy_from_slice(&p);
        }
    });
    Ok(())
}

pub fn splat(img: &mut Image, splats: &[f32]) -> Result<(), String> {
    if !splats.len().is_multiple_of(8) || splats.len() / 8 > MAX_SPLATS {
        return Err(format!("points must be groups of 8 floats, at most {MAX_SPLATS} groups"));
    }
    if splats.iter().any(|v| !v.is_finite()) {
        return Err("points must be finite".into());
    }
    let width = img.width;
    rows(&mut img.px, width, |y, row| {
        let yc = y as f32 + 0.5;
        for s in splats.chunks_exact(8) {
            let (sx, sy, r) = (s[0], s[1], s[2].clamp(0.0, MAX_RADIUS));
            if r <= 0.0 || (yc - sy).abs() > r {
                continue;
            }
            let hard = s[7].clamp(0.0, 1.0);
            let x0 = ((sx - r).floor().max(0.0) as usize).min(width);
            let x1 = ((sx + r).ceil().max(0.0) as usize).min(width);
            for x in x0..x1 {
                let d = ((x as f32 + 0.5 - sx).powi(2) + (yc - sy).powi(2)).sqrt() / r;
                if d >= 1.0 {
                    continue;
                }
                let fall = if d <= hard { 1.0 } else { 1.0 - (d - hard) / (1.0 - hard).max(1e-6) };
                let w = fall * fall * (3.0 - 2.0 * fall) * s[6].clamp(0.0, 1.0);
                let p = &mut row[x * 4..x * 4 + 4];
                for c in 0..3 {
                    p[c] += s[3 + c].max(0.0) * w;
                }
                p[3] = (p[3] + w).min(1.0);
            }
        }
    });
    Ok(())
}

pub fn histogram(img: &Image) -> Value {
    let mut bins = vec![[0u64; 256]; 4];
    for p in img.px.chunks_exact(4) {
        let a = p[3];
        let v = if a > 0.0 { [p[0] / a, p[1] / a, p[2] / a, a] } else { [0.0; 4] };
        for c in 0..4 {
            bins[c][(v[c].clamp(0.0, 1.0) * 255.0).round() as usize] += 1;
        }
    }
    json!({ "r": bins[0].to_vec(), "g": bins[1].to_vec(), "b": bins[2].to_vec(), "a": bins[3].to_vec() })
}

fn clamp_premultiplied(img: &mut Image) {
    for p in img.px.chunks_exact_mut(4) {
        p[3] = if p[3].is_finite() { p[3].clamp(0.0, 1.0) } else { 0.0 };
        for c in 0..3 {
            p[c] = if p[c].is_finite() { p[c].clamp(0.0, p[3].max(0.0) * 64.0) } else { 0.0 };
        }
    }
}

pub fn inputs(op: &str) -> &'static [&'static str] {
    match op {
        "kernel.blend" => &["image", "src"],
        "kernel.resample" | "kernel.warp_affine" => &["src", "image"],
        _ => &["image"],
    }
}

pub fn writes(op: &str) -> bool {
    op != "kernel.histogram"
}

pub const OPS: &[&str] = &[
    "kernel.blur",
    "kernel.directional_blur",
    "kernel.convolve",
    "kernel.morphology",
    "kernel.color_matrix",
    "kernel.curves",
    "kernel.blend",
    "kernel.noise",
    "kernel.resample",
    "kernel.warp_affine",
    "kernel.splat",
    "kernel.histogram",
];

pub fn run(op: &str, args: &Value, images: &mut [Image], splats: &[f32]) -> Result<Value, String> {
    let (image, rest) = match op {
        "kernel.resample" | "kernel.warp_affine" | "kernel.blend" => {
            let [a, b] = images else { return Err("two images are required".into()) };
            if op == "kernel.blend" {
                (a, Some(&*b))
            } else {
                (b, Some(&*a))
            }
        }
        _ => {
            let [a] = images else { return Err("one image is required".into()) };
            (a, None)
        }
    };
    let s = |k: &str, d: &'static str| args.get(k).and_then(Value::as_str).unwrap_or(d).to_owned();
    match op {
        "kernel.blur" => blur(image, num(args, "radius", 0.0), &s("kind", "gaussian"))?,
        "kernel.directional_blur" => directional_blur(image, num(args, "angle", 0.0), num(args, "length", 0.0))?,
        "kernel.convolve" => {
            let w = floats(args, "weights", MAX_CONVOLVE * MAX_CONVOLVE)?.ok_or("weights are required")?;
            let size = args.get("size").and_then(Value::as_u64).unwrap_or(3) as usize;
            convolve(image, size, &w, num(args, "divisor", 1.0), num(args, "bias", 0.0))?;
        }
        "kernel.morphology" => {
            let dilate = match s("op", "dilate").as_str() {
                "dilate" => true,
                "erode" => false,
                _ => return Err("op must be dilate or erode".into()),
            };
            morphology(image, num(args, "radius", 0.0), dilate)?;
        }
        "kernel.color_matrix" => color_matrix(image, &floats(args, "matrix", 20)?.ok_or("matrix is required")?)?,
        "kernel.curves" => curves(image, args)?,
        "kernel.blend" => blend(image, rest.ok_or("src is required")?, &s("mode", "normal"), num(args, "opacity", 1.0))?,
        "kernel.noise" => {
            let multiply = match s("mode", "fill").as_str() {
                "fill" => false,
                "multiply" => true,
                _ => return Err("mode must be fill or multiply".into()),
            };
            let octaves = args.get("octaves").and_then(Value::as_u64).unwrap_or(4) as u32;
            let seed = args.get("seed").and_then(Value::as_u64).unwrap_or(0) as u32;
            noise(image, num(args, "scale", 32.0), octaves, seed, num(args, "z", 0.0), multiply)?;
        }
        "kernel.resample" => resample(rest.ok_or("src is required")?, image),
        "kernel.warp_affine" => warp_affine(rest.ok_or("src is required")?, image, &floats(args, "matrix", 6)?.ok_or("matrix is required")?)?,
        "kernel.splat" => splat(image, splats)?,
        "kernel.histogram" => return Ok(histogram(image)),
        _ => return Err(format!("unknown kernel {op}")),
    }
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: usize, h: usize, p: [f32; 4]) -> Image {
        Image { width: w, height: h, px: p.iter().copied().cycle().take(w * h * 4).collect() }
    }

    #[test]
    fn blurring_a_flat_image_leaves_it_flat_and_spreads_a_dot() {
        let mut flat = solid(9, 7, [0.25, 0.5, 0.75, 1.0]);
        blur(&mut flat, 3.0, "gaussian").unwrap();
        assert!(flat.px.chunks_exact(4).all(|p| (p[0] - 0.25).abs() < 1e-5 && (p[3] - 1.0).abs() < 1e-5));

        let mut dot = Image::new(9, 9);
        dot.px[(4 * 9 + 4) * 4 + 3] = 1.0;
        blur(&mut dot, 2.0, "box").unwrap();
        let alpha: f32 = dot.px.chunks_exact(4).map(|p| p[3]).sum();
        assert!((alpha - 1.0).abs() < 1e-4, "box blur keeps energy, got {alpha}");
        assert!((dot.px[(4 * 9 + 4) * 4 + 3] - 1.0 / 25.0).abs() < 1e-5);
    }

    #[test]
    fn a_large_gaussian_stays_within_ten_percent_of_the_exact_peak_and_keeps_energy() {
        let mut fast = Image::new(121, 121);
        fast.px[(60 * 121 + 60) * 4 + 3] = 1.0;
        let mut exact = fast.clone();
        blur(&mut fast, 30.0, "gaussian").unwrap();
        separable(&mut exact, &gaussian_weights(30.0));
        let energy: f32 = fast.px.chunks_exact(4).map(|p| p[3]).sum();
        assert!((energy - 1.0).abs() < 1e-3, "{energy}");
        let worst = fast.px.iter().zip(&exact.px).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
        let peak = exact.px.iter().copied().fold(0.0f32, f32::max);
        assert!(worst < 0.1 * peak, "worst {worst} vs peak {peak}");
    }

    #[test]
    fn bad_arguments_are_refused_not_clamped_silently() {
        let mut img = Image::new(2, 2);
        assert_eq!(blur(&mut img, 600.0, "gaussian").unwrap_err(), "radius must be 0..=512");
        assert!(convolve(&mut img, 4, &[0.0; 16], 1.0, 0.0).is_err());
        assert!(color_matrix(&mut img, &[1.0; 19]).is_err());
        assert!(blend(&mut img, &Image::new(3, 2), "multiply", 1.0).is_err());
        assert!(run("kernel.nope", &json!({}), &mut [img], &[]).is_err());
    }

    #[test]
    fn color_matrix_inverts_straight_colour_and_keeps_alpha() {
        let mut img = solid(1, 1, [0.1, 0.2, 0.3, 0.5]);
        let invert = [-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        color_matrix(&mut img, &invert).unwrap();
        let expect = [0.4, 0.3, 0.2, 0.5];
        assert!(img.px.iter().zip(expect).all(|(a, b)| (a - b).abs() < 1e-6), "{:?}", img.px);
    }

    #[test]
    fn multiply_blend_of_white_keeps_the_base_and_screen_of_black_keeps_it_too() {
        let base = solid(2, 1, [0.2, 0.4, 0.6, 1.0]);
        let mut a = base.clone();
        blend(&mut a, &solid(2, 1, [1.0, 1.0, 1.0, 1.0]), "multiply", 1.0).unwrap();
        assert!(a.px.iter().zip(&base.px).all(|(x, y)| (x - y).abs() < 1e-6));
        let mut b = base.clone();
        blend(&mut b, &solid(2, 1, [0.0, 0.0, 0.0, 1.0]), "screen", 1.0).unwrap();
        assert!(b.px.iter().zip(&base.px).all(|(x, y)| (x - y).abs() < 1e-6));
    }

    #[test]
    fn curves_map_through_sampled_points() {
        let mut img = solid(1, 1, [0.5, 0.5, 0.5, 1.0]);
        curves(&mut img, &json!({"rgb": [1.0, 0.0], "a": [0.0, 1.0]})).unwrap();
        assert!((img.px[0] - 0.5).abs() < 1e-6 && (img.px[3] - 1.0).abs() < 1e-6);
        curves(&mut img, &json!({"r": [0.0, 0.0, 1.0]})).unwrap();
        assert!(img.px[0].abs() < 1e-6, "{:?}", img.px);
    }

    #[test]
    fn resample_and_affine_warp_keep_a_flat_colour() {
        let src = solid(8, 8, [0.3, 0.3, 0.3, 1.0]);
        let mut small = Image::new(3, 5);
        resample(&src, &mut small);
        assert!(small.px.chunks_exact(4).all(|p| (p[0] - 0.3).abs() < 1e-5));
        let mut shifted = Image::new(8, 8);
        warp_affine(&src, &mut shifted, &[1.0, 0.0, 2.0, 0.0, 1.0, 0.0]).unwrap();
        assert_eq!(shifted.px[3], 0.0, "uncovered pixels are transparent");
        assert!((shifted.px[(8 * 3 + 5) * 4] - 0.3).abs() < 1e-5);
    }

    #[test]
    fn splats_add_light_and_noise_is_deterministic() {
        let mut img = Image::new(10, 10);
        splat(&mut img, &[5.0, 5.0, 3.0, 1.0, 0.5, 0.0, 1.0, 1.0]).unwrap();
        let centre = &img.px[(5 * 10 + 5) * 4..(5 * 10 + 5) * 4 + 4];
        assert_eq!(centre, &[1.0, 0.5, 0.0, 1.0]);
        assert_eq!(img.px[3], 0.0);
        let mut a = Image::new(6, 6);
        let mut b = Image::new(6, 6);
        noise(&mut a, 4.0, 3, 7, 0.0, false).unwrap();
        noise(&mut b, 4.0, 3, 7, 0.0, false).unwrap();
        assert_eq!(a, b);
        assert!(a.px.chunks_exact(4).all(|p| (0.0..=1.0).contains(&p[0]) && p[3] == 1.0));
    }

    #[test]
    fn histogram_counts_straight_values() {
        let img = solid(3, 1, [0.5, 0.0, 0.5, 0.5]);
        let h = histogram(&img);
        assert_eq!(h["r"][255], 3);
        assert_eq!(h["g"][0], 3);
        assert_eq!(h["a"][128], 3);
    }
}

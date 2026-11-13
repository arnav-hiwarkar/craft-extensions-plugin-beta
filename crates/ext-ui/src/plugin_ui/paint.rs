//! Paints a plugin draw list ([`DrawOp`]) with an egui painter: canvases and app overlays share it.
//!
//! Every number reaching egui is finite and clamped here even though `validate` already bounded
//! it: the painter is also used for overlay lists an app validated with older rules.

use craft_ext_core::ui::tree::{Align, DrawOp, Gradient, Paint, PathCmd, VAlign};
use egui::{Color32, Mesh, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, TextureId, pos2, vec2};

/// Maps canvas coordinates to screen points: `offset + p * scale`. Stroke widths and text sizes
/// are screen points and do not scale; positions, radii and corner radii do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Xf {
    pub offset: Pos2,
    pub scale: f32,
}

impl Xf {
    pub fn new(offset: Pos2, scale: f32) -> Self {
        Self { offset, scale: if scale.is_finite() && scale > 0.0 { scale } else { 1.0 } }
    }
    fn pt(&self, p: [f32; 2]) -> Pos2 {
        pos2(self.offset.x + fin(p[0]) * self.scale, self.offset.y + fin(p[1]) * self.scale)
    }
    fn len(&self, v: f32) -> f32 {
        (fin(v) * self.scale).clamp(0.0, 1e6)
    }
    fn back(&self, p: Pos2) -> [f32; 2] {
        [(p.x - self.offset.x) / self.scale, (p.y - self.offset.y) / self.scale]
    }
}

/// Finite or zero, and inside a range egui handles without precision trouble.
fn fin(v: f32) -> f32 {
    if v.is_finite() { v.clamp(-1e7, 1e7) } else { 0.0 }
}

/// Straight RGBA in 0..=1 to egui's colour; non-finite channels are 0.
pub fn color(c: [f32; 4]) -> Color32 {
    let u = |v: f32| if v.is_finite() { (v.clamp(0.0, 1.0) * 255.0).round() as u8 } else { 0 };
    Color32::from_rgba_unmultiplied(u(c[0]), u(c[1]), u(c[2]), u(c[3]))
}

fn stroke(c: Option<[f32; 4]>, width: Option<f32>) -> Option<Stroke> {
    c.map(|c| Stroke::new(width.unwrap_or(1.0).clamp(0.0, 256.0), color(c)))
}

/// Paints `ops` clipped to `clip`. `image` resolves a named image to its texture and pixel size.
pub fn paint_ops(painter: &Painter, xf: Xf, clip: Rect, ops: &[DrawOp], image: &dyn Fn(&str) -> Option<TextureId>) {
    let base = painter.with_clip_rect(clip.intersect(painter.clip_rect()));
    let mut p = base.clone();
    for op in ops.iter().take(craft_ext_core::ui::tree::limits::MAX_DRAW_OPS) {
        match op {
            DrawOp::Line(l) => {
                p.line_segment([xf.pt(l.from), xf.pt(l.to)], Stroke::new(fin(l.width).clamp(0.0, 256.0), color(l.rgba)));
            }
            DrawOp::Rect(r) => {
                let rect = Rect::from_two_pos(xf.pt(r.min), xf.pt(r.max));
                let radius = r.radius.map_or(0.0, |x| xf.len(x)).min(rect.width().min(rect.height()) / 2.0).min(255.0);
                match &r.fill {
                    Paint::Solid(c) => {
                        p.rect_filled(rect, radius, color(*c));
                    }
                    Paint::Gradient(g) => fill(&p, xf, &rounded_rect(rect, radius), &Paint::Gradient(g.clone())),
                }
                if let Some(s) = stroke(r.stroke, r.stroke_width) {
                    p.rect_stroke(rect, radius, s, StrokeKind::Inside);
                }
            }
            DrawOp::Circle(c) => {
                let center = xf.pt(c.center);
                let radius = xf.len(c.radius).min(1e5);
                match &c.fill {
                    Paint::Solid(col) => {
                        p.circle_filled(center, radius, color(*col));
                    }
                    Paint::Gradient(_) => fill(&p, xf, &ellipse(center, radius), &c.fill),
                }
                if let Some(s) = stroke(c.stroke, c.stroke_width) {
                    p.circle_stroke(center, radius, s);
                }
            }
            DrawOp::Text(t) => {
                let h = match t.align {
                    Align::Left => egui::Align::Min,
                    Align::Center => egui::Align::Center,
                    Align::Right => egui::Align::Max,
                };
                let v = match t.valign {
                    VAlign::Top => egui::Align::Min,
                    VAlign::Center => egui::Align::Center,
                    VAlign::Bottom => egui::Align::Max,
                };
                let size = fin(t.size).clamp(4.0, 256.0);
                let font = if t.mono { egui::FontId::monospace(size) } else { egui::FontId::proportional(size) };
                p.text(xf.pt(t.at), egui::Align2([h, v]), &t.text, font, color(t.rgba));
            }
            DrawOp::Path(path) => {
                let subpaths = flatten(&path.path, xf);
                if let Some(paint) = &path.fill {
                    for (pts, _) in &subpaths {
                        fill(&p, xf, pts, paint);
                    }
                }
                if let Some(s) = stroke(path.stroke, path.stroke_width) {
                    for (pts, closed) in subpaths {
                        if pts.len() >= 2 {
                            p.add(if closed { Shape::closed_line(pts, s) } else { Shape::line(pts, s) });
                        }
                    }
                }
            }
            DrawOp::Polygon(poly) => {
                let pts: Vec<Pos2> = poly.points.iter().take(4096).map(|q| xf.pt(*q)).collect();
                if let Some(paint) = &poly.fill {
                    fill(&p, xf, &pts, paint);
                }
                if let Some(s) = stroke(poly.stroke, poly.stroke_width) {
                    p.add(Shape::closed_line(pts, s));
                }
            }
            DrawOp::Image(i) => {
                if let Some(tex) = image(&i.image) {
                    let uv = i
                        .uv
                        .map_or(Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), |[a, b]| Rect::from_two_pos(pos2(a[0], a[1]), pos2(b[0], b[1])));
                    p.image(tex, Rect::from_two_pos(xf.pt(i.min), xf.pt(i.max)), uv, i.tint.map_or(Color32::WHITE, color));
                }
            }
            DrawOp::Clip(c) => {
                let r = Rect::from_two_pos(xf.pt(c.min), xf.pt(c.max));
                p = base.with_clip_rect(r.intersect(base.clip_rect()));
            }
            DrawOp::ResetClip => p = base.clone(),
        }
    }
}

/// A rounded rectangle as a polygon, for gradient fills.
fn rounded_rect(r: Rect, radius: f32) -> Vec<Pos2> {
    if radius <= 0.5 {
        return vec![r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom()];
    }
    let mut pts = Vec::with_capacity(4 * 9);
    let corners = [
        (pos2(r.right() - radius, r.top() + radius), -90.0f32),
        (pos2(r.right() - radius, r.bottom() - radius), 0.0),
        (pos2(r.left() + radius, r.bottom() - radius), 90.0),
        (pos2(r.left() + radius, r.top() + radius), 180.0),
    ];
    for (c, start) in corners {
        for i in 0..=8 {
            let a = (start + 90.0 * i as f32 / 8.0).to_radians();
            pts.push(c + vec2(a.cos(), a.sin()) * radius);
        }
    }
    pts
}

fn ellipse(c: Pos2, r: f32) -> Vec<Pos2> {
    let n = ((r * 0.5) as usize).clamp(16, 128);
    (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            c + vec2(a.cos(), a.sin()) * r
        })
        .collect()
}

/// Flattens path commands into screen-space subpaths, each with whether it was closed.
fn flatten(cmds: &[PathCmd], xf: Xf) -> Vec<(Vec<Pos2>, bool)> {
    let mut out = Vec::new();
    let mut cur: Vec<Pos2> = Vec::new();
    let mut total = 0usize;
    let push = |cur: &mut Vec<Pos2>, p: Pos2, total: &mut usize| {
        if *total < 200_000 {
            cur.push(p);
            *total += 1;
        }
    };
    let steps = |len: f32| ((len / 4.0) as usize).clamp(4, 64);
    for c in cmds {
        match c {
            PathCmd::Move(p) => {
                if cur.len() > 1 {
                    out.push((std::mem::take(&mut cur), false));
                }
                cur.clear();
                push(&mut cur, xf.pt(*p), &mut total);
            }
            PathCmd::Line(p) => push(&mut cur, xf.pt(*p), &mut total),
            PathCmd::Quad([c1, to]) => {
                let a = cur.last().copied().unwrap_or(xf.pt(*c1));
                let (b, d) = (xf.pt(*c1), xf.pt(*to));
                let n = steps(a.distance(b) + b.distance(d));
                for i in 1..=n {
                    let t = i as f32 / n as f32;
                    let u = 1.0 - t;
                    push(&mut cur, pos2(u * u * a.x + 2.0 * u * t * b.x + t * t * d.x, u * u * a.y + 2.0 * u * t * b.y + t * t * d.y), &mut total);
                }
            }
            PathCmd::Cubic([c1, c2, to]) => {
                let a = cur.last().copied().unwrap_or(xf.pt(*c1));
                let (b, c, d) = (xf.pt(*c1), xf.pt(*c2), xf.pt(*to));
                let n = steps(a.distance(b) + b.distance(c) + c.distance(d));
                for i in 1..=n {
                    let t = i as f32 / n as f32;
                    let u = 1.0 - t;
                    let (k0, k1, k2, k3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                    push(&mut cur, pos2(k0 * a.x + k1 * b.x + k2 * c.x + k3 * d.x, k0 * a.y + k1 * b.y + k2 * c.y + k3 * d.y), &mut total);
                }
            }
            PathCmd::Close => {
                if cur.len() > 1 {
                    let first = cur[0];
                    out.push((std::mem::take(&mut cur), true));
                    cur.push(first);
                }
            }
        }
    }
    if cur.len() > 1 {
        out.push((cur, false));
    }
    out
}

/// Fills a simple polygon (screen points) with a colour or gradient.
fn fill(p: &Painter, xf: Xf, pts: &[Pos2], paint: &Paint) {
    let pts = dedup(pts);
    if pts.len() < 3 {
        return;
    }
    if let Paint::Solid(c) = paint
        && convex(&pts)
    {
        p.add(Shape::convex_polygon(pts, color(*c), Stroke::NONE));
        return;
    }
    let tris = triangulate(&pts);
    let mut mesh = Mesh::default();
    match paint {
        Paint::Solid(c) => {
            let col = color(*c);
            for q in &pts {
                mesh.colored_vertex(*q, col);
            }
            for [a, b, c] in tris {
                mesh.add_triangle(a, b, c);
            }
        }
        Paint::Gradient(g) => gradient_mesh(&mut mesh, xf, &pts, &tris, g),
    }
    p.add(Shape::mesh(mesh));
}

/// Subdivides triangles until their edges are short enough for per-vertex colours to follow the
/// gradient, within a fixed triangle budget.
fn gradient_mesh(mesh: &mut Mesh, xf: Xf, pts: &[Pos2], tris: &[[u32; 3]], g: &Gradient) {
    const BUDGET: usize = 16_384;
    const EDGE: f32 = 12.0;
    let mut work: Vec<[Pos2; 3]> =
        tris.iter().filter_map(|t| Some([*pts.get(t[0] as usize)?, *pts.get(t[1] as usize)?, *pts.get(t[2] as usize)?])).collect();
    let mut done: Vec<[Pos2; 3]> = Vec::new();
    while let Some(t) = work.pop() {
        let long = t[0].distance(t[1]).max(t[1].distance(t[2])).max(t[2].distance(t[0])) > EDGE;
        if long && done.len() + work.len() + 4 <= BUDGET {
            let m = |a: Pos2, b: Pos2| pos2((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
            let (ab, bc, ca) = (m(t[0], t[1]), m(t[1], t[2]), m(t[2], t[0]));
            work.extend([[t[0], ab, ca], [ab, t[1], bc], [ca, bc, t[2]], [ab, bc, ca]]);
        } else {
            done.push(t);
        }
    }
    for t in done {
        let base = mesh.vertices.len() as u32;
        for q in t {
            mesh.colored_vertex(q, color(g.sample(xf.back(q))));
        }
        mesh.add_triangle(base, base + 1, base + 2);
    }
}

fn dedup(pts: &[Pos2]) -> Vec<Pos2> {
    let mut out: Vec<Pos2> = Vec::with_capacity(pts.len());
    for q in pts {
        if out.last().is_none_or(|l: &Pos2| l.distance(*q) > 1e-3) {
            out.push(*q);
        }
    }
    if out.len() > 1 && out.first().zip(out.last()).is_some_and(|(a, b)| a.distance(*b) <= 1e-3) {
        out.pop();
    }
    out
}

fn cross(o: Pos2, a: Pos2, b: Pos2) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

fn convex(pts: &[Pos2]) -> bool {
    let n = pts.len();
    let mut sign = 0.0f32;
    for i in 0..n {
        let c = cross(pts[i], pts[(i + 1) % n], pts[(i + 2) % n]);
        if c.abs() > 1e-6 {
            if sign != 0.0 && c.signum() != sign {
                return false;
            }
            sign = c.signum();
        }
    }
    true
}

/// Ear clipping for simple polygons; falls back to a fan for very large or self-crossing input.
pub(crate) fn triangulate(pts: &[Pos2]) -> Vec<[u32; 3]> {
    let n = pts.len();
    let fan = || (1..n.saturating_sub(1)).map(|i| [0, i as u32, i as u32 + 1]).collect::<Vec<_>>();
    if n < 3 {
        return vec![];
    }
    if n > 1024 || convex(pts) {
        return fan();
    }
    let area: f32 = (0..n).map(|i| cross(Pos2::ZERO, pts[i], pts[(i + 1) % n])).sum();
    let ccw = area > 0.0;
    let mut idx: Vec<usize> = (0..n).collect();
    let mut out = Vec::with_capacity(n - 2);
    let mut guard = 0usize;
    while idx.len() > 3 {
        guard += 1;
        if guard > n * n {
            return fan();
        }
        let m = idx.len();
        let mut clipped = false;
        for i in 0..m {
            let (a, b, c) = (idx[(i + m - 1) % m], idx[i], idx[(i + 1) % m]);
            let turn = cross(pts[a], pts[b], pts[c]);
            if (turn > 0.0) != ccw || turn.abs() < 1e-9 {
                continue;
            }
            let inside = idx.iter().any(|&k| {
                k != a && k != b && k != c && {
                    let (d1, d2, d3) = (cross(pts[a], pts[b], pts[k]), cross(pts[b], pts[c], pts[k]), cross(pts[c], pts[a], pts[k]));
                    if ccw { d1 >= 0.0 && d2 >= 0.0 && d3 >= 0.0 } else { d1 <= 0.0 && d2 <= 0.0 && d3 <= 0.0 }
                }
            });
            if !inside {
                out.push([a as u32, b as u32, c as u32]);
                idx.remove(i);
                clipped = true;
                break;
            }
        }
        if !clipped {
            return fan();
        }
    }
    out.push([idx[0] as u32, idx[1] as u32, idx[2] as u32]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concave_polygons_triangulate_without_a_fan() {
        // An L shape: a fan from vertex 0 would cover the notch.
        let l = [pos2(0.0, 0.0), pos2(2.0, 0.0), pos2(2.0, 1.0), pos2(1.0, 1.0), pos2(1.0, 2.0), pos2(0.0, 2.0)];
        let tris = triangulate(&l);
        assert_eq!(tris.len(), 4);
        let area: f32 = tris.iter().map(|[a, b, c]| cross(l[*a as usize], l[*b as usize], l[*c as usize]).abs() / 2.0).sum();
        assert!((area - 3.0).abs() < 1e-4, "{area}");
    }

    #[test]
    fn paths_flatten_into_closed_and_open_subpaths() {
        let cmds = vec![
            PathCmd::Move([0.0, 0.0]),
            PathCmd::Line([10.0, 0.0]),
            PathCmd::Cubic([[10.0, 10.0], [0.0, 10.0], [0.0, 0.0]]),
            PathCmd::Close,
            PathCmd::Move([20.0, 0.0]),
            PathCmd::Quad([[25.0, 5.0], [30.0, 0.0]]),
        ];
        let subs = flatten(&cmds, Xf::new(pos2(0.0, 0.0), 2.0));
        assert_eq!(subs.len(), 2);
        assert!(subs[0].1 && !subs[1].1);
        assert_eq!(subs[1].0.last().copied(), Some(pos2(60.0, 0.0)));
    }
}

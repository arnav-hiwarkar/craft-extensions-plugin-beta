//! The built-in plugin icon set (`craft_ext_core::ui::tree::ICONS`), drawn from simple strokes in
//! the theme's colour. These are original line drawings made for this renderer; the names follow
//! the craft-framework icon catalogue so a framework engine can draw its own glyph for each.

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

/// Paints icon `name` centred in `rect`. Returns false (after drawing a neutral placeholder) for a
/// name this build does not know.
pub fn paint_icon(painter: &Painter, rect: Rect, name: &str, color: Color32) -> bool {
    let side = rect.width().min(rect.height());
    let r = Rect::from_center_size(rect.center(), vec2(side, side)).shrink(side * 0.08);
    let w = (side / 12.0).max(1.0);
    let s = Stroke::new(w, color);
    let p = |x: f32, y: f32| pos2(r.left() + x * r.width(), r.top() + y * r.height());
    let line = |pts: &[(f32, f32)]| {
        painter.add(Shape::line(pts.iter().map(|(x, y)| p(*x, *y)).collect(), s));
    };
    let closed = |pts: &[(f32, f32)]| {
        painter.add(Shape::closed_line(pts.iter().map(|(x, y)| p(*x, *y)).collect(), s));
    };
    let filled = |pts: &[(f32, f32)]| {
        painter.add(Shape::convex_polygon(pts.iter().map(|(x, y)| p(*x, *y)).collect(), color, Stroke::NONE));
    };
    let ring = |x: f32, y: f32, rad: f32| {
        painter.circle_stroke(p(x, y), rad * r.width(), s);
    };
    let dot = |x: f32, y: f32, rad: f32| {
        painter.circle_filled(p(x, y), rad * r.width(), color);
    };
    let arc = |x: f32, y: f32, rad: f32, from: f32, to: f32| {
        let pts: Vec<Pos2> = (0..=16)
            .map(|i| {
                let a = (from + (to - from) * i as f32 / 16.0).to_radians();
                p(x + rad * a.cos(), y + rad * a.sin())
            })
            .collect();
        painter.add(Shape::line(pts, s));
    };
    match name {
        "close" => {
            line(&[(0.2, 0.2), (0.8, 0.8)]);
            line(&[(0.8, 0.2), (0.2, 0.8)]);
        }
        "check" => line(&[(0.15, 0.55), (0.4, 0.78), (0.85, 0.25)]),
        "plus" => {
            line(&[(0.5, 0.15), (0.5, 0.85)]);
            line(&[(0.15, 0.5), (0.85, 0.5)]);
        }
        "minus" => line(&[(0.15, 0.5), (0.85, 0.5)]),
        "chevron_down" => line(&[(0.2, 0.38), (0.5, 0.66), (0.8, 0.38)]),
        "chevron_up" => line(&[(0.2, 0.62), (0.5, 0.34), (0.8, 0.62)]),
        "chevron_left" => line(&[(0.62, 0.2), (0.34, 0.5), (0.62, 0.8)]),
        "chevron_right" => line(&[(0.38, 0.2), (0.66, 0.5), (0.38, 0.8)]),
        "more" => {
            for x in [0.2, 0.5, 0.8] {
                dot(x, 0.5, 0.07);
            }
        }
        "menu" => {
            for y in [0.25, 0.5, 0.75] {
                line(&[(0.15, y), (0.85, y)]);
            }
        }
        "search" | "zoom_in" => {
            ring(0.42, 0.42, 0.27);
            line(&[(0.62, 0.62), (0.88, 0.88)]);
            if name == "zoom_in" {
                line(&[(0.42, 0.3), (0.42, 0.54)]);
                line(&[(0.3, 0.42), (0.54, 0.42)]);
            }
        }
        "settings" => {
            ring(0.5, 0.5, 0.16);
            for i in 0..8 {
                let a = (i as f32 * 45.0).to_radians();
                line(&[(0.5 + 0.28 * a.cos(), 0.5 + 0.28 * a.sin()), (0.5 + 0.4 * a.cos(), 0.5 + 0.4 * a.sin())]);
            }
            ring(0.5, 0.5, 0.29);
        }
        "trash" => {
            line(&[(0.15, 0.25), (0.85, 0.25)]);
            line(&[(0.38, 0.25), (0.42, 0.12), (0.58, 0.12), (0.62, 0.25)]);
            closed(&[(0.25, 0.25), (0.3, 0.88), (0.7, 0.88), (0.75, 0.25)]);
        }
        "copy" => {
            closed(&[(0.35, 0.35), (0.85, 0.35), (0.85, 0.85), (0.35, 0.85)]);
            line(&[(0.65, 0.35), (0.65, 0.15), (0.15, 0.15), (0.15, 0.65), (0.35, 0.65)]);
        }
        "save" => {
            closed(&[(0.15, 0.15), (0.72, 0.15), (0.85, 0.28), (0.85, 0.85), (0.15, 0.85)]);
            closed(&[(0.3, 0.15), (0.3, 0.38), (0.62, 0.38), (0.62, 0.15)]);
            closed(&[(0.3, 0.6), (0.7, 0.6), (0.7, 0.85), (0.3, 0.85)]);
        }
        "folder" => closed(&[(0.1, 0.25), (0.4, 0.25), (0.48, 0.35), (0.9, 0.35), (0.9, 0.82), (0.1, 0.82)]),
        "file" => {
            closed(&[(0.22, 0.1), (0.6, 0.1), (0.8, 0.3), (0.8, 0.9), (0.22, 0.9)]);
            line(&[(0.6, 0.1), (0.6, 0.3), (0.8, 0.3)]);
        }
        "undo" | "redo" => {
            let flip = |x: f32| if name == "redo" { 1.0 - x } else { x };
            arc(0.5, 0.58, 0.3, if name == "redo" { 0.0 } else { 180.0 }, if name == "redo" { -200.0 } else { 380.0 });
            line(&[(flip(0.12), 0.38), (flip(0.2), 0.62), (flip(0.42), 0.55)]);
        }
        "refresh" => {
            arc(0.5, 0.5, 0.32, -60.0, 240.0);
            line(&[(0.6, 0.12), (0.68, 0.24), (0.55, 0.32)]);
        }
        "eye" | "eye_off" => {
            arc(0.5, 0.95, 0.55, -145.0, -35.0);
            arc(0.5, 0.05, 0.55, 35.0, 145.0);
            ring(0.5, 0.5, 0.12);
            if name == "eye_off" {
                line(&[(0.15, 0.85), (0.85, 0.15)]);
            }
        }
        "lock" | "lock_open" => {
            closed(&[(0.2, 0.45), (0.8, 0.45), (0.8, 0.88), (0.2, 0.88)]);
            if name == "lock" {
                arc(0.5, 0.45, 0.2, 180.0, 360.0);
            } else {
                arc(0.5, 0.35, 0.2, 180.0, 330.0);
            }
            dot(0.5, 0.66, 0.05);
        }
        "link" => {
            closed(&[(0.1, 0.4), (0.45, 0.4), (0.45, 0.6), (0.1, 0.6)]);
            closed(&[(0.55, 0.4), (0.9, 0.4), (0.9, 0.6), (0.55, 0.6)]);
            line(&[(0.3, 0.5), (0.7, 0.5)]);
        }
        "image" => {
            closed(&[(0.1, 0.18), (0.9, 0.18), (0.9, 0.82), (0.1, 0.82)]);
            line(&[(0.1, 0.72), (0.38, 0.45), (0.6, 0.66), (0.72, 0.55), (0.9, 0.72)]);
            dot(0.7, 0.35, 0.06);
        }
        "text" => {
            line(&[(0.2, 0.2), (0.8, 0.2)]);
            line(&[(0.5, 0.2), (0.5, 0.85)]);
        }
        "square" => closed(&[(0.18, 0.18), (0.82, 0.18), (0.82, 0.82), (0.18, 0.82)]),
        "circle" => ring(0.5, 0.5, 0.34),
        "triangle" => closed(&[(0.5, 0.15), (0.88, 0.82), (0.12, 0.82)]),
        "star" => {
            let pts: Vec<(f32, f32)> = (0..10)
                .map(|i| {
                    let a = (-90.0 + 36.0 * i as f32).to_radians();
                    let rad = if i % 2 == 0 { 0.4 } else { 0.17 };
                    (0.5 + rad * a.cos(), 0.53 + rad * a.sin())
                })
                .collect();
            closed(&pts);
        }
        "play" => filled(&[(0.28, 0.15), (0.82, 0.5), (0.28, 0.85)]),
        "pause" => {
            filled(&[(0.25, 0.18), (0.42, 0.18), (0.42, 0.82), (0.25, 0.82)]);
            filled(&[(0.58, 0.18), (0.75, 0.18), (0.75, 0.82), (0.58, 0.82)]);
        }
        "info" | "error" | "success" => {
            ring(0.5, 0.5, 0.4);
            match name {
                "info" => {
                    dot(0.5, 0.3, 0.05);
                    line(&[(0.5, 0.45), (0.5, 0.72)]);
                }
                "error" => {
                    line(&[(0.35, 0.35), (0.65, 0.65)]);
                    line(&[(0.65, 0.35), (0.35, 0.65)]);
                }
                _ => line(&[(0.3, 0.52), (0.45, 0.66), (0.7, 0.36)]),
            }
        }
        "warning" => {
            closed(&[(0.5, 0.12), (0.9, 0.85), (0.1, 0.85)]);
            line(&[(0.5, 0.38), (0.5, 0.6)]);
            dot(0.5, 0.72, 0.04);
        }
        "grid" => {
            for k in [0.15, 0.5, 0.85] {
                line(&[(k, 0.15), (k, 0.85)]);
                line(&[(0.15, k), (0.85, k)]);
            }
        }
        "layers" => {
            closed(&[(0.5, 0.15), (0.88, 0.35), (0.5, 0.55), (0.12, 0.35)]);
            line(&[(0.12, 0.52), (0.5, 0.72), (0.88, 0.52)]);
            line(&[(0.12, 0.68), (0.5, 0.88), (0.88, 0.68)]);
        }
        "pencil" => {
            closed(&[(0.68, 0.12), (0.88, 0.32), (0.35, 0.85), (0.12, 0.88), (0.15, 0.65)]);
            line(&[(0.58, 0.22), (0.78, 0.42)]);
        }
        "brush" => {
            line(&[(0.85, 0.12), (0.45, 0.58)]);
            closed(&[(0.42, 0.55), (0.48, 0.62), (0.35, 0.85), (0.12, 0.88), (0.2, 0.68)]);
        }
        "eyedropper" => {
            line(&[(0.15, 0.85), (0.6, 0.4)]);
            closed(&[(0.55, 0.3), (0.72, 0.13), (0.87, 0.28), (0.7, 0.45)]);
            line(&[(0.48, 0.3), (0.7, 0.52)]);
        }
        "move" => {
            line(&[(0.5, 0.1), (0.5, 0.9)]);
            line(&[(0.1, 0.5), (0.9, 0.5)]);
            line(&[(0.4, 0.2), (0.5, 0.1), (0.6, 0.2)]);
            line(&[(0.4, 0.8), (0.5, 0.9), (0.6, 0.8)]);
            line(&[(0.2, 0.4), (0.1, 0.5), (0.2, 0.6)]);
            line(&[(0.8, 0.4), (0.9, 0.5), (0.8, 0.6)]);
        }
        "hand" => {
            for (i, x) in [0.3f32, 0.43, 0.56, 0.69].iter().enumerate() {
                line(&[(*x, if i == 0 { 0.35 } else { 0.18 }), (*x, 0.6)]);
            }
            line(&[(0.3, 0.6), (0.3, 0.72), (0.42, 0.88), (0.69, 0.88), (0.78, 0.6), (0.69, 0.5)]);
        }
        "crop" => {
            line(&[(0.25, 0.08), (0.25, 0.75), (0.92, 0.75)]);
            line(&[(0.08, 0.25), (0.75, 0.25), (0.75, 0.92)]);
        }
        "flag" => {
            line(&[(0.2, 0.9), (0.2, 0.12)]);
            closed(&[(0.2, 0.15), (0.8, 0.15), (0.65, 0.35), (0.8, 0.55), (0.2, 0.55)]);
        }
        "clock" => {
            ring(0.5, 0.5, 0.4);
            line(&[(0.5, 0.25), (0.5, 0.5), (0.68, 0.6)]);
        }
        "home" => {
            line(&[(0.1, 0.5), (0.5, 0.13), (0.9, 0.5)]);
            closed(&[(0.22, 0.4), (0.22, 0.88), (0.78, 0.88), (0.78, 0.4)]);
        }
        "sun" => {
            ring(0.5, 0.5, 0.18);
            for i in 0..8 {
                let a = (i as f32 * 45.0).to_radians();
                line(&[(0.5 + 0.28 * a.cos(), 0.5 + 0.28 * a.sin()), (0.5 + 0.42 * a.cos(), 0.5 + 0.42 * a.sin())]);
            }
        }
        "moon" => {
            arc(0.5, 0.5, 0.36, 60.0, 300.0);
            arc(0.68, 0.38, 0.28, 110.0, 250.0);
        }
        "funnel" => closed(&[(0.1, 0.15), (0.9, 0.15), (0.58, 0.52), (0.58, 0.85), (0.42, 0.75), (0.42, 0.52)]),
        "sort_ascending" | "sort_descending" => {
            let (from, to) = if name == "sort_ascending" { (0.85, 0.15) } else { (0.15, 0.85) };
            line(&[(0.3, from), (0.3, to)]);
            line(&[(0.15, to + (from - to) * 0.25), (0.3, to), (0.45, to + (from - to) * 0.25)]);
            for (i, wdt) in [0.2f32, 0.3, 0.4].iter().enumerate() {
                let y = 0.25 + 0.25 * i as f32;
                line(&[(0.55, y), (0.55 + wdt, y)]);
            }
        }
        "palette" => {
            ring(0.5, 0.5, 0.4);
            for (x, y) in [(0.35, 0.35), (0.6, 0.3), (0.72, 0.5), (0.35, 0.62)] {
                dot(x, y, 0.06);
            }
        }
        _ => {
            painter.rect_stroke(r.shrink(side * 0.1), 2.0, s, egui::StrokeKind::Inside);
            dot(0.5, 0.5, 0.06);
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_listed_icon_is_drawn() {
        let ctx = egui::Context::default();
        ctx.run_ui(egui::RawInput::default(), |root| {
            egui::CentralPanel::default().show(root, |ui| {
                let painter = ui.painter();
                let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(16.0, 16.0));
                for name in craft_ext_core::ui::tree::ICONS {
                    assert!(super::paint_icon(painter, rect, name, egui::Color32::WHITE), "{name}");
                }
                assert!(!super::paint_icon(painter, rect, "not_an_icon", egui::Color32::WHITE));
            });
        })
        .drop_without_applying_deltas();
    }
}

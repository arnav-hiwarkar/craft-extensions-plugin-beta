//! Gradient Ramp for PhotoCraft: a canvas tool. Press, drag and release on the image to draw a
//! linear, radial or reflected gradient; while you drag, a guide (`canvas.overlay.set`) shows the
//! line, its ends and its length and angle. Shift snaps the angle to 45° steps. Release paints the
//! gradient on a new layer (or the active layer); Escape cancels, and PhotoCraft rolls the whole
//! gesture back. The whole gesture is one undo step.
//!
//! The options panel holds the colours, shape and opacity as a form (UI level 1.1).

use craft_guest::ui::{self, NodeExt, TextStyle, draw};
use craft_guest::{CallError, Event, Form, JsonValue, Params, Value, Values, json, session, t};
use std::sync::Mutex;

const PANEL: &str = "options";
/// Bytes of RGBA8 per `pixels.set` call, so its base64 stays well under the 1 MiB argument limit.
const TILE_BYTES: usize = 600 * 1024;

struct State {
    values: Option<Values>,
    /// The drag: where it began.
    start: Option<[f64; 2]>,
    notice: Option<String>,
}

static STATE: Mutex<State> = Mutex::new(State { values: None, start: None, notice: None });

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn form() -> Form {
    Form::new("gradient", t!("Gradient Ramp"))
        .color("from", t!("Start colour"), [0.05, 0.1, 0.35, 1.0], true)
        .color("to", t!("End colour"), [1.0, 0.75, 0.3, 1.0], true)
        .radio("shape", t!("Shape"), &[("linear", t!("Linear")), ("radial", t!("Radial")), ("reflected", t!("Reflected"))], "linear")
        .slider("opacity", t!("Opacity"), 0.0..=100.0, 100.0)
        .toggle("new_layer", t!("Draw on a new layer"), true)
}

fn values() -> Values {
    let mut v = form().0.defaults();
    if let Some(saved) = state().values.clone() {
        v.extend(saved);
    }
    v
}

fn handle(event: Event) -> Result<(), String> {
    match event {
        Event::Init => render(),
        Event::Ui { surface, widget, value, .. } if surface == PANEL => {
            if widget == "reset" {
                state().values = None;
            } else {
                let v: Value = craft_guest::serde_json::from_value(value).map_err(|e| e.to_string())?;
                let mut candidate = values();
                candidate.insert(widget, v);
                form().0.validate_values(&candidate).map_err(|e| e.to_string())?;
                state().values = Some(candidate);
            }
            render()
        }
        Event::Tool { phase, at } => tool(&phase, &at),
        _ => Ok(()),
    }
}

fn point(at: &JsonValue) -> Result<[f64; 2], String> {
    let x = at[0].as_f64().filter(|v| v.is_finite()).ok_or("bad pointer position")?;
    let y = at[1].as_f64().filter(|v| v.is_finite()).ok_or("bad pointer position")?;
    Ok([x, y])
}

/// Shift (from `tool.state`) snaps the drag to 0°, 45°, 90°… keeping its length.
pub fn constrain(start: [f64; 2], end: [f64; 2], snap: bool) -> [f64; 2] {
    if !snap {
        return end;
    }
    let (dx, dy) = (end[0] - start[0], end[1] - start[1]);
    let len = dx.hypot(dy);
    let step = std::f64::consts::FRAC_PI_4;
    let a = (dy.atan2(dx) / step).round() * step;
    [start[0] + len * a.cos(), start[1] + len * a.sin()]
}

fn shift_held() -> bool {
    craft_guest::call("tool.state", json!({})).is_ok_and(|s| s["modifiers"]["shift"].as_bool().unwrap_or(false))
}

fn tool(phase: &str, at: &JsonValue) -> Result<(), String> {
    match phase {
        "begin" => {
            let p = point(at)?;
            state().start = Some(p);
            guide(p, p)
        }
        "update" => {
            let Some(start) = state().start else { return Ok(()) };
            guide(start, constrain(start, point(at)?, shift_held()))
        }
        "commit" => {
            let Some(start) = state().start.take() else { return Ok(()) };
            let end = constrain(start, point(at)?, shift_held());
            paint(start, end)
        }
        "cancel" => {
            state().start = None;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// The guide while dragging: the gradient's line, its ends, and its length and angle.
fn guide(start: [f64; 2], end: [f64; 2]) -> Result<(), String> {
    let f = |p: [f64; 2]| [p[0] as f32, p[1] as f32];
    let (dx, dy) = (end[0] - start[0], end[1] - start[1]);
    let v = values();
    let p = Params(&v);
    let (from, to) = (p.color("from")?, p.color("to")?);
    let shape = p.choice("shape")?.to_owned();
    let mut ops = vec![
        draw::line(f(start), f(end), 3.0, [0.0, 0.0, 0.0, 0.6]),
        draw::line(f(start), f(end), 1.5, [1.0, 1.0, 1.0, 1.0]),
        draw::circle(f(start), 4.0, from).stroke([1.0, 1.0, 1.0, 1.0], 1.5),
        draw::circle(f(end), 4.0, to).stroke([1.0, 1.0, 1.0, 1.0], 1.5),
        draw::text(
            [end[0] as f32 + 8.0, end[1] as f32 + 8.0],
            &format!("{:.0} px · {:.0}°", dx.hypot(dy), -dy.atan2(dx).to_degrees()),
            12.0,
            [1.0, 1.0, 1.0, 1.0],
        ),
    ];
    if shape == "radial" {
        // The radius, as a ring around the start.
        ops.push(draw::circle(f(start), dx.hypot(dy) as f32, [0.0, 0.0, 0.0, 0.0]).stroke([1.0, 1.0, 1.0, 0.7], 1.0));
    }
    craft_guest::call("canvas.overlay.set", json!({"ops": ops})).map_err(|e| e.to_string())?;
    Ok(())
}

/// How far along the gradient pixel centre (x, y) is, 0 at the start and 1 at the end.
pub fn position(shape: &str, start: [f64; 2], end: [f64; 2], x: f64, y: f64) -> f64 {
    let (dx, dy) = (end[0] - start[0], end[1] - start[1]);
    let len2 = (dx * dx + dy * dy).max(1e-9);
    let (px, py) = (x - start[0], y - start[1]);
    match shape {
        "radial" => ((px * px + py * py) / len2).sqrt().min(1.0),
        "reflected" => ((px * dx + py * dy) / len2).abs().min(1.0),
        _ => ((px * dx + py * dy) / len2).clamp(0.0, 1.0),
    }
}

fn lerp(a: [f32; 4], b: [f32; 4], t: f32, opacity: f32) -> [u8; 4] {
    let c = |i: usize| a[i] + (b[i] - a[i]) * t;
    let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [u(c(0)), u(c(1)), u(c(2)), u(c(3) * opacity)]
}

/// Paints the gradient over the whole image, in row bands of at most `TILE_BYTES`.
fn paint(start: [f64; 2], end: [f64; 2]) -> Result<(), String> {
    let v = values();
    let p = Params(&v);
    let (from, to, shape, opacity, new_layer) =
        (p.color("from")?, p.color("to")?, p.choice("shape")?.to_owned(), p.f("opacity")? as f32 / 100.0, p.b("new_layer")?);
    let info = match craft_guest::call("document.info", json!({})) {
        Ok(i) => i,
        Err(CallError::PermissionNeeded(_)) => {
            state().notice = Some("Drawing needs \"document.edit:document\". Turn it on in the plugin's settings.".into());
            return render();
        }
        Err(e) => return Err(e.to_string()),
    };
    let (w, h) = (info["width"].as_u64().ok_or("no width")? as usize, info["height"].as_u64().ok_or("no height")? as usize);
    let layer = if new_layer {
        craft_guest::call("layer.create", json!({"kind": "pixel", "name": "Gradient"})).map_err(|e| e.to_string())?["id"].as_u64()
    } else {
        info["active_layer"].as_u64()
    };
    let layer = layer.ok_or("no layer to paint on")?;
    let tile_width = w.min(1024);
    let rows = (TILE_BYTES / (tile_width * 4).max(1)).max(1);
    let mut y0 = 0;
    while y0 < h {
        let band = rows.min(h - y0);
        let mut x0 = 0;
        while x0 < w {
            let width = tile_width.min(w - x0);
            let mut buf = Vec::with_capacity(width * band * 4);
            for y in y0..y0 + band {
                for x in x0..x0 + width {
                    let t = position(&shape, start, end, x as f64 + 0.5, y as f64 + 0.5) as f32;
                    buf.extend_from_slice(&lerp(from, to, t, opacity));
                }
            }
            craft_guest::call(
                "pixels.set",
                json!({"layer": layer, "x": x0, "y": y0, "width": width, "height": band, "format": "rgba8", "data": base64(&buf)}),
            )
            .map_err(|e| e.to_string())?;
            x0 += width;
        }
        y0 += band;
    }
    state().notice = Some(format!("Drew a {shape} gradient {:.0} px long.", (end[0] - start[0]).hypot(end[1] - start[1])));
    render()
}

/// Standard base64 with padding (PhotoCraft's pixel data encoding).
pub fn base64(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (u32::from(c[0]) << 16) | (u32::from(*c.get(1).unwrap_or(&0)) << 8) | u32::from(*c.get(2).unwrap_or(&0));
        out.push(A[(n >> 18) as usize & 63] as char);
        out.push(A[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { A[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { A[n as usize & 63] as char } else { '=' });
    }
    out
}

fn render() -> Result<(), String> {
    let v = values();
    let notice = state().notice.clone();
    let mut children = vec![
        ui::text("Pick Plugins ▸ Gradient Ramp ▸ Gradient Ramp, then drag on the image. Shift snaps the angle; Esc cancels.", TextStyle::Small),
        ui::form(&form(), &v),
        ui::button("reset", "Reset options").icon("undo"),
    ];
    if let Some(n) = notice {
        children.push(ui::label(n).style(TextStyle::Weak));
    }
    ui::set_panel(PANEL, ui::column(children)).map_err(|e| e.to_string())?;
    Ok(())
}

session!(handle);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_follow_the_shape() {
        let (s, e) = ([0.0, 0.0], [10.0, 0.0]);
        assert_eq!(position("linear", s, e, 5.0, 3.0), 0.5);
        assert_eq!(position("linear", s, e, -4.0, 0.0), 0.0);
        assert_eq!(position("reflected", s, e, -5.0, 0.0), 0.5);
        assert_eq!(position("radial", s, e, 0.0, 5.0), 0.5);
        assert_eq!(position("radial", s, e, 0.0, 50.0), 1.0);
    }

    #[test]
    fn shift_snaps_to_45_degrees_keeping_length() {
        let p = constrain([0.0, 0.0], [10.0, 1.0], true);
        assert!((p[1]).abs() < 1e-9 && (p[0] - 101f64.sqrt()).abs() < 1e-9, "{p:?}");
        assert_eq!(constrain([0.0, 0.0], [10.0, 1.0], false), [10.0, 1.0]);
    }

    #[test]
    fn base64_matches_the_standard_alphabet() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64(&[255, 0, 0, 255]), "/wAA/w==");
    }

    #[test]
    fn the_options_panel_and_the_guide_are_valid_trees() {
        let tree = ui::column(vec![ui::form(&form(), &values()), ui::button("reset", "Reset options").icon("undo")]);
        ui::check(&tree).unwrap();
        let ops = vec![draw::circle([1.0, 1.0], 4.0, [1.0, 0.0, 0.0, 1.0]).stroke([1.0, 1.0, 1.0, 1.0], 1.5)];
        ui::check(&ui::canvas("c", "C", 10.0, 10.0, ops)).unwrap();
    }
}

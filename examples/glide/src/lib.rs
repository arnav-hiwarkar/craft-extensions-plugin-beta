use craft_guest::ui::{self, NodeExt, draw};
use craft_guest::{Event, call, json, session};
use std::cell::RefCell;

pub const PROPERTIES: [(&str, &str); 4] = [("Position", "transform/position"), ("Scale", "transform/scale"), ("Rotation", "transform/rotation"), ("Opacity", "transform/opacity")];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve {
    pub out_influence: f64,
    pub in_influence: f64,
}

pub const PRESETS: [(&str, &str, Curve); 6] = [
    ("gentle", "Gentle", Curve { out_influence: 33.3, in_influence: 33.3 }),
    ("smooth", "Smooth", Curve { out_influence: 60.0, in_influence: 60.0 }),
    ("strong", "Strong", Curve { out_influence: 85.0, in_influence: 85.0 }),
    ("slow-start", "Slow start", Curve { out_influence: 85.0, in_influence: 10.0 }),
    ("slow-end", "Slow end", Curve { out_influence: 10.0, in_influence: 85.0 }),
    ("snap", "Snap", Curve { out_influence: 95.0, in_influence: 5.0 }),
];

impl Curve {
    pub fn point(self, t: f64) -> [f64; 2] {
        let (x1, x2) = (self.out_influence / 100.0, 1.0 - self.in_influence / 100.0);
        let u = 1.0 - t;
        let b = |p1: f64, p2: f64| 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t;
        [b(x1, x2), b(0.0, 1.0)]
    }

    pub fn ease_args(self, layer: u64, path: &str, times: &[f64]) -> craft_guest::JsonValue {
        json!({
            "layer": layer, "path": path, "times": times, "interpolation": "bezier",
            "in": {"speed": 0, "influence": self.in_influence},
            "out": {"speed": 0, "influence": self.out_influence}
        })
    }
}

struct State {
    curve: Curve,
    property: usize,
    note: String,
}

thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State { curve: PRESETS[1].2, property: 0, note: String::new() }) };
}

const W: f32 = 220.0;
const H: f32 = 140.0;

fn preview(c: Curve) -> ui::Node {
    let pad = 12.0;
    let at = |p: [f64; 2]| [pad + p[0] as f32 * (W - 2.0 * pad), H - pad - p[1] as f32 * (H - 2.0 * pad)];
    let grid = [0.6, 0.6, 0.6, 0.35];
    let mut ops = vec![draw::rect([pad, pad], [W - pad, H - pad], [0.0, 0.0, 0.0, 0.0], Some(grid))];
    let points: Vec<[f32; 2]> = (0..=48).map(|i| at(c.point(f64::from(i) / 48.0))).collect();
    for pair in points.windows(2) {
        ops.push(draw::line(pair[0], pair[1], 2.0, [0.35, 0.65, 1.0, 1.0]));
    }
    ui::canvas("curve", "Easing curve", W, H, ops).hint("Speed over the segment between two keyframes")
}

fn draw_panel() -> Result<(), String> {
    STATE.with(|s| {
        let s = s.borrow();
        let names: Vec<&str> = PROPERTIES.iter().map(|p| p.0).collect();
        let tree = ui::column(vec![
            preview(s.curve),
            ui::row(PRESETS[..3].iter().map(|(id, label, _)| ui::button(id, label)).collect()),
            ui::row(PRESETS[3..].iter().map(|(id, label, _)| ui::button(id, label)).collect()),
            ui::slider("out", "Leave (influence %)", s.curve.out_influence, 0.1, 100.0),
            ui::slider("in", "Arrive (influence %)", s.curve.in_influence, 0.1, 100.0),
            ui::select("property", "Property", &names, s.property as u32),
            ui::button("apply", "Apply to selected layers"),
            ui::label(s.note.as_str()).style(ui::TextStyle::Small),
        ]);
        ui::set_panel("glide", tree).map(drop).map_err(|e| e.to_string())
    })
}

fn apply() -> Result<String, String> {
    let (curve, path) = STATE.with(|s| {
        let s = s.borrow();
        (s.curve, PROPERTIES[s.property].1)
    });
    let selection = call("selection.get", json!({})).map_err(|e| e.to_string())?;
    let layers: Vec<u64> = selection["layers"].as_array().into_iter().flatten().filter_map(|l| l["id"].as_u64()).collect();
    if layers.is_empty() {
        return Ok("Select one or more layers first.".into());
    }
    let mut eased = 0;
    for layer in layers {
        let prop = call("prop.get", json!({"layer": layer, "path": path})).map_err(|e| e.to_string())?;
        let times: Vec<f64> = prop["keys"].as_array().into_iter().flatten().filter_map(|k| k["time"].as_f64()).collect();
        if times.len() < 2 {
            continue;
        }
        call("prop.ease", curve.ease_args(layer, path, &times)).map_err(|e| e.to_string())?;
        eased += times.len();
    }
    Ok(match eased {
        0 => "No layer has two or more keyframes on that property.".into(),
        n => format!("Eased {n} keyframes."),
    })
}

fn handle(event: Event) -> Result<(), String> {
    if let Event::Ui { widget, value, .. } = event {
        let number = value.as_f64();
        let note = match widget.as_str() {
            "apply" => Some(apply()?),
            _ => None,
        };
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            if let Some((_, _, c)) = PRESETS.iter().find(|(id, ..)| *id == widget) {
                s.curve = *c;
            }
            match (widget.as_str(), number) {
                ("out", Some(v)) => s.curve.out_influence = v.clamp(0.1, 100.0),
                ("in", Some(v)) => s.curve.in_influence = v.clamp(0.1, 100.0),
                ("property", Some(v)) => s.property = (v as usize).min(PROPERTIES.len() - 1),
                _ => {}
            }
            if let Some(n) = note {
                s.note = n;
            }
        });
    }
    draw_panel()
}

session!(handle);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves_start_and_end_flat_and_cross_the_middle() {
        let c = PRESETS[1].2;
        assert_eq!(c.point(0.0), [0.0, 0.0]);
        assert_eq!(c.point(1.0), [1.0, 1.0]);
        let mid = c.point(0.5);
        assert!((mid[1] - 0.5).abs() < 1e-9 && (mid[0] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn apply_arguments_ease_every_key_on_both_sides() {
        let args = PRESETS[3].2.ease_args(7, "transform/position", &[0.0, 1.0]);
        assert_eq!(args["out"]["influence"], 85.0);
        assert_eq!(args["in"]["influence"], 10.0);
        assert_eq!(args["interpolation"], "bezier");
        assert_eq!(args["times"], json!([0.0, 1.0]));
    }
}

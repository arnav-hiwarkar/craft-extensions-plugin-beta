//! Levels Scope for PhotoCraft: a panel that draws the image's histogram on a UI canvas with
//! draggable black, gray and white points, statistics in a second tab, and buttons that add the
//! points as a Levels adjustment layer (non-destructive) or apply them to the active layer.
//!
//! It reads pixels only through `histogram.get` (`document.read:document`); editing needs the
//! optional `document.edit:document`, and without it the panel says so instead of failing. With the
//! optional `app.events` the `follow` hook redraws the histogram when an image opens or the
//! selection changes. Every event is one PhotoCraft transaction: "Add Levels layer" is one undo step.

use craft_guest::ui::{self, Cursor, NodeExt, TextStyle, draw, paint};
use craft_guest::{CallError, Event, JsonValue, json, session};
use std::sync::Mutex;

const PANEL: &str = "levels-scope";
const AUTO: &str = "auto-levels-layer";
const NEED_EDIT: &str = "document.edit:document";

/// Canvas geometry: 256 levels at `SCALE` points each, inside a margin.
const MARGIN: f32 = 8.0;
const SCALE: f32 = 1.0;
const PLOT_H: f32 = 110.0;
const HANDLES_H: f32 = 16.0;
const WIDTH: f32 = MARGIN * 2.0 + 255.0 * SCALE;
const HEIGHT: f32 = PLOT_H + HANDLES_H + MARGIN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Handle {
    Black,
    Gray,
    White,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Levels {
    pub black: f64,
    pub gamma: f64,
    pub white: f64,
}

impl Default for Levels {
    fn default() -> Self {
        Self { black: 0.0, gamma: 1.0, white: 255.0 }
    }
}

impl Levels {
    /// The input level the gray point sits on: the input that maps to 50 % output.
    pub fn gray_at(&self) -> f64 {
        self.black + (self.white - self.black) * 0.5f64.powf(self.gamma)
    }
    /// Moves the gray point to input `x`, keeping it strictly between black and white.
    pub fn set_gray_at(&mut self, x: f64) {
        let t = ((x - self.black) / (self.white - self.black).max(1.0)).clamp(0.01, 0.99);
        self.gamma = (t.ln() / 0.5f64.ln()).clamp(0.1, 9.99);
    }
    /// Keeps the points in PhotoCraft's Levels ranges, black below white by at least 2.
    pub fn clamp(&mut self) {
        self.black = self.black.round().clamp(0.0, 253.0);
        self.white = self.white.round().clamp(self.black + 2.0, 255.0);
        self.gamma = (self.gamma * 100.0).round() / 100.0;
        self.gamma = self.gamma.clamp(0.1, 9.99);
    }
}

/// Black and white points that clip `fraction` of the counted pixels at each end.
pub fn auto_points(bins: &[u64], fraction: f64) -> (f64, f64) {
    let total: u64 = bins.iter().sum();
    if total == 0 {
        return (0.0, 255.0);
    }
    let limit = (total as f64 * fraction) as u64;
    let mut seen = 0u64;
    let mut black = 0usize;
    for (i, n) in bins.iter().enumerate() {
        seen += n;
        if seen > limit {
            black = i;
            break;
        }
    }
    seen = 0;
    let mut white = 255usize;
    for (i, n) in bins.iter().enumerate().rev() {
        seen += n;
        if seen > limit {
            white = i;
            break;
        }
    }
    let black = (black as f64).min(253.0);
    (black, (white as f64).max(black + 2.0))
}

struct State {
    /// The histogram: luminosity, red, green, blue (256 bins each) and the pixel count.
    bins: Option<([Vec<u64>; 4], u64)>,
    channel: usize,
    composite: bool,
    log_scale: bool,
    levels: Levels,
    dragging: Option<Handle>,
    tab: String,
    notice: Option<String>,
}

static STATE: Mutex<State> = Mutex::new(State {
    bins: None,
    channel: 0,
    composite: true,
    log_scale: false,
    levels: Levels { black: 0.0, gamma: 1.0, white: 255.0 },
    dragging: None,
    tab: String::new(),
    notice: None,
});

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

const CHANNELS: [&str; 4] = ["Luminosity", "Red", "Green", "Blue"];
const KEYS: [&str; 4] = ["luminosity", "red", "green", "blue"];
const TINTS: [[f32; 4]; 4] = [[0.85, 0.85, 0.85, 1.0], [0.95, 0.3, 0.3, 1.0], [0.3, 0.85, 0.35, 1.0], [0.35, 0.5, 1.0, 1.0]];

fn handle(event: Event) -> Result<(), String> {
    match event {
        Event::Init => {
            let _ = refresh();
            render()
        }
        Event::Command { id } if id == AUTO => auto_levels_layer(),
        Event::App { .. } => {
            let _ = refresh();
            render()
        }
        Event::Ui { surface, widget, value, .. } if surface == PANEL => {
            on_widget(&widget, &value)?;
            render()
        }
        _ => Ok(()),
    }
}

/// Reads the histogram again; a missing document or a non-RGB one is shown, not an error.
fn refresh() -> Result<(), String> {
    let composite = state().composite;
    match craft_guest::call("histogram.get", json!({"composite": composite})) {
        Ok(h) => {
            let bins = KEYS.map(|k| h[k].as_array().map(|a| a.iter().map(|v| v.as_u64().unwrap_or(0)).collect()).unwrap_or_else(|| vec![0; 256]));
            let mut s = state();
            s.bins = Some((bins, h["count"].as_u64().unwrap_or(0)));
            s.notice = None;
        }
        Err(e) => {
            let mut s = state();
            s.bins = None;
            s.notice = Some(match e {
                CallError::PermissionNeeded(_) => "Reading the histogram needs \"document.read:document\".".into(),
                CallError::NotFound(_) => "Open an image to see its histogram.".into(),
                other => format!("No histogram: {other}"),
            });
        }
    }
    Ok(())
}

fn on_widget(widget: &str, value: &JsonValue) -> Result<(), String> {
    match widget {
        "plot" => {
            let Some(p) = ui::pointer(value) else { return Ok(()) };
            let level = f64::from(((p.at[0] - MARGIN) / SCALE).clamp(0.0, 255.0));
            let mut s = state();
            match p.kind {
                ui::PointerKind::Down => {
                    let l = &s.levels;
                    let near = |x: f64| (x - level).abs();
                    let (b, g, w) = (near(l.black), near(l.gray_at()), near(l.white));
                    s.dragging = Some(if g <= b && g <= w { Handle::Gray } else if b <= w { Handle::Black } else { Handle::White });
                }
                ui::PointerKind::Drag | ui::PointerKind::Up => {
                    if let Some(h) = s.dragging {
                        let gray = s.levels.gray_at();
                        let l = &mut s.levels;
                        match h {
                            Handle::Black => {
                                // The gray point keeps its place between the ends, as in Levels.
                                l.black = level.min(l.white - 2.0);
                                l.set_gray_at(gray.max(l.black + 1.0));
                            }
                            Handle::White => {
                                l.white = level.max(l.black + 2.0);
                                l.set_gray_at(gray.min(l.white - 1.0));
                            }
                            Handle::Gray => l.set_gray_at(level),
                        }
                        l.clamp();
                    }
                    if p.kind == ui::PointerKind::Up {
                        s.dragging = None;
                    }
                }
                _ => {}
            }
        }
        "black" | "gamma" | "white" => {
            let v = value.as_f64().ok_or("a number was expected")?;
            let mut s = state();
            match widget {
                "black" => s.levels.black = v,
                "white" => s.levels.white = v,
                _ => s.levels.gamma = v,
            }
            s.levels.clamp();
        }
        "channel" => state().channel = (value.as_u64().unwrap_or(0) as usize).min(3),
        "source" => {
            state().composite = value.as_u64() != Some(1);
            refresh()?;
        }
        "log" => state().log_scale = value.as_bool().unwrap_or(false),
        "view" => state().tab = value.as_str().unwrap_or("histogram").to_string(),
        "refresh" => refresh()?,
        "auto" => {
            let mut s = state();
            let channel = s.channel;
            if let Some((bins, _)) = &s.bins {
                let (black, white) = auto_points(&bins[channel], 0.001);
                s.levels = Levels { black, gamma: 1.0, white };
            }
        }
        "reset" => state().levels = Levels::default(),
        "add-layer" => return apply(true),
        "apply" => return apply(false),
        _ => {}
    }
    Ok(())
}

fn levels_params(l: &Levels) -> JsonValue {
    json!({"inBlack": l.black, "gamma": l.gamma, "inWhite": l.white})
}

/// Adds the points as a Levels layer (or applies them destructively). One event: one undo step.
fn apply(as_layer: bool) -> Result<(), String> {
    let levels = state().levels.clone();
    match craft_guest::call("adjustment.apply", json!({"adjustment": "levels", "params": levels_params(&levels), "as_layer": as_layer})) {
        Ok(_) => {
            state().notice = Some(if as_layer { "Added a Levels layer.".into() } else { "Applied Levels to the active layer.".into() });
            let _ = refresh();
            Ok(())
        }
        Err(CallError::PermissionNeeded(_)) => {
            state().notice = Some(format!("Changing the image needs \"{NEED_EDIT}\". Turn it on in the plugin's settings."));
            Ok(())
        }
        // Anything else rolls the event back; the reason shows in PhotoCraft's status bar.
        Err(e) => Err(e.to_string()),
    }
}

/// The command: auto points from the image's luminosity, added as a Levels layer.
fn auto_levels_layer() -> Result<(), String> {
    let h = craft_guest::call("histogram.get", json!({"composite": true})).map_err(|e| e.to_string())?;
    let bins: Vec<u64> = h["luminosity"].as_array().ok_or("no histogram")?.iter().map(|v| v.as_u64().unwrap_or(0)).collect();
    let (black, white) = auto_points(&bins, 0.001);
    let levels = Levels { black, gamma: 1.0, white };
    craft_guest::call("adjustment.apply", json!({"adjustment": "levels", "params": levels_params(&levels), "as_layer": true})).map_err(|e| e.to_string())?;
    state().levels = levels;
    let _ = craft_guest::call("ui.notify", json!({"message": format!("Auto Levels: black {black}, white {white}")}));
    let _ = refresh();
    render()
}

/// The histogram plot: the channel's bins as a filled polygon, the clipped ends shaded, and the
/// three handles below.
fn plot(s: &State) -> JsonValue {
    let mut ops = vec![draw::rounded_rect([0.0, 0.0], [WIDTH, HEIGHT], 4.0, [0.12, 0.12, 0.13, 1.0])];
    let base = MARGIN + PLOT_H - 2.0;
    if let Some((bins, _)) = &s.bins {
        let b = &bins[s.channel];
        let f = |n: u64| if s.log_scale { ((n as f64) + 1.0).ln() } else { n as f64 };
        let max = b.iter().map(|&n| f(n)).fold(1.0, f64::max);
        let mut pts = vec![[MARGIN, base]];
        for (i, &n) in b.iter().enumerate() {
            let h = (f(n) / max) as f32 * (PLOT_H - MARGIN - 4.0);
            pts.push([MARGIN + i as f32 * SCALE, base - h]);
        }
        pts.push([MARGIN + 255.0 * SCALE, base]);
        let tint = TINTS[s.channel];
        let fill = paint::linear([MARGIN, 0.0], [MARGIN + 255.0 * SCALE, 0.0], vec![paint::stop(0.0, [tint[0] * 0.3, tint[1] * 0.3, tint[2] * 0.3, 0.9]), paint::stop(1.0, tint)]);
        ops.push(draw::polygon(&pts).fill(fill));
    } else {
        ops.push(draw::text_aligned([WIDTH / 2.0, PLOT_H / 2.0], "No histogram", 12.0, [0.7, 0.7, 0.7, 1.0], ui::Align::Center, ui::VAlign::Center));
    }
    let l = &s.levels;
    let x = |v: f64| MARGIN + v as f32 * SCALE;
    // Clipped ends.
    ops.push(draw::rect([MARGIN, MARGIN], [x(l.black), base], [0.0, 0.0, 0.0, 0.35], None));
    ops.push(draw::rect([x(l.white), MARGIN], [MARGIN + 255.0 * SCALE, base], [1.0, 1.0, 1.0, 0.12], None));
    let tri = |cx: f32, fill: [f32; 4]| {
        let y = base + 3.0;
        draw::polygon(&[[cx, y], [cx - 6.0, y + 11.0], [cx + 6.0, y + 11.0]]).fill(fill).stroke([0.6, 0.6, 0.6, 1.0], 1.0)
    };
    ops.push(tri(x(l.black), [0.0, 0.0, 0.0, 1.0]));
    ops.push(tri(x(l.gray_at()), [0.5, 0.5, 0.5, 1.0]));
    ops.push(tri(x(l.white), [1.0, 1.0, 1.0, 1.0]));
    ui::canvas("plot", "Histogram", WIDTH, HEIGHT, ops).cursor(Cursor::ResizeHorizontal).hint("Drag the black, gray and white points")
}

fn stats(s: &State) -> Vec<JsonValue> {
    let Some((bins, count)) = &s.bins else { return vec![ui::label("No histogram yet.")] };
    let mut rows = vec![ui::text("Channel", TextStyle::Strong), ui::text("Mean", TextStyle::Strong), ui::text("Median", TextStyle::Strong)];
    for (i, name) in CHANNELS.iter().enumerate() {
        let b = &bins[i];
        let n: u64 = b.iter().sum();
        let mean = if n == 0 { 0.0 } else { b.iter().enumerate().map(|(v, c)| v as f64 * *c as f64).sum::<f64>() / n as f64 };
        let mut seen = 0;
        let median = b.iter().position(|c| {
            seen += c;
            seen * 2 >= n
        });
        rows.push(ui::label(*name));
        rows.push(ui::label(format!("{mean:.1}")).style(TextStyle::Mono));
        rows.push(ui::label(median.map_or("–".into(), |m| m.to_string())).style(TextStyle::Mono));
    }
    vec![ui::label(format!("{count} pixels counted")).style(TextStyle::Small), ui::grid(3, rows).set("striped", true)]
}

fn render() -> Result<(), String> {
    let s = state();
    let l = &s.levels;
    let histogram = vec![
        ui::row(vec![
            ui::radio("source", "Source", &["Image", "Active layer"], if s.composite { 0 } else { 1 }).set("horizontal", true),
            ui::select("channel", "Channel", &CHANNELS, s.channel as u32),
            ui::toggle("log", "Log scale", s.log_scale),
        ]),
        plot(&s),
        ui::row(vec![
            ui::number("black", "Black", l.black).range(0.0, 253.0).decimals(0),
            ui::number("gamma", "Gamma", l.gamma).range(0.1, 9.99).decimals(2).step(0.01),
            ui::number("white", "White", l.white).range(2.0, 255.0).decimals(0),
        ]),
        ui::row(vec![
            ui::button("auto", "Auto").icon("refresh").hint("Clip 0.1 % at each end of the shown channel"),
            ui::button("reset", "Reset"),
            ui::button("refresh", "Refresh histogram").icon("refresh").icon_only(),
        ]),
        ui::row(vec![
            ui::button("add-layer", "Add Levels layer").icon("layers").hint("A Levels adjustment layer above the active layer"),
            ui::button("apply", "Apply to layer").hint("Change the active layer's pixels"),
        ]),
    ];
    let mut children = vec![ui::tabs("view", vec![ui::tab("histogram", "Histogram", histogram), ui::tab("stats", "Statistics", stats(&s))], (!s.tab.is_empty()).then_some(s.tab.as_str()))];
    if let Some(n) = &s.notice {
        children.push(ui::label(n.clone()).style(TextStyle::Weak));
    }
    drop(s);
    ui::set_panel(PANEL, ui::column(children)).map_err(|e| e.to_string())?;
    Ok(())
}

session!(handle);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gray_point_round_trips_through_gamma() {
        let mut l = Levels { black: 20.0, gamma: 1.0, white: 220.0 };
        assert!((l.gray_at() - 120.0).abs() < 1e-9);
        l.set_gray_at(70.0);
        assert!(l.gamma > 1.0, "a darker gray point raises gamma: {}", l.gamma);
        assert!((l.gray_at() - 70.0).abs() < 0.5);
    }

    #[test]
    fn auto_points_clip_the_tails() {
        let mut bins = vec![0u64; 256];
        bins[10] = 1;
        bins[40] = 5000;
        bins[200] = 5000;
        bins[250] = 1;
        assert_eq!(auto_points(&bins, 0.001), (40.0, 200.0));
        assert_eq!(auto_points(&[0; 256], 0.001), (0.0, 255.0));
    }

    #[test]
    fn the_panel_is_a_valid_level_1_1_tree() {
        let mut s = state();
        s.bins = Some(([vec![3; 256], vec![1; 256], vec![2; 256], vec![0; 256]], 768));
        let tree = ui::column(vec![plot(&s)]);
        drop(s);
        ui::check(&tree).unwrap();
        assert_eq!(ui::level(&tree).unwrap(), ui::UI_LEVEL);
    }
}

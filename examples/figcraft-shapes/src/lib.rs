//! Shapes for FigCraft: a form panel, a command that adds a row of shapes, and a drag tool that draws
//! a rectangle with a live guide overlay. Every layer it makes is an ordinary FigCraft layer.
//!
//! Each event is one host transaction: `Err` (or a host refusal) rolls back everything the event did,
//! and a tool gesture is rolled back as a whole when it is cancelled (Escape).

use craft_guest::{CallError, Event, Form, JsonValue, Params, Value, Values, json, session, t, ui};
use std::sync::Mutex;

/// Contribution and widget ids from `craft-plugin.toml`.
const PANEL: &str = "shapes";
const CREATE: &str = "create-shapes";
const NEED_EDIT: &str = "document.edit:document";
const MAX_COUNT: i64 = 10;

struct State {
    /// The panel form's current values (defaults merged with the person's edits).
    values: Option<Values>,
    /// The last notice shown under the form.
    notice: Option<String>,
    /// The last app event, when `app.events` is granted.
    last_event: Option<String>,
    /// The tool gesture: its start point and the live rectangle's node id.
    drag: Option<([f64; 2], Option<u64>)>,
}

static STATE: Mutex<State> = Mutex::new(State { values: None, notice: None, last_event: None, drag: None });

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn form() -> Form {
    Form::new("shapes", t!("Shapes"))
        .choice("shape", t!("Shape"), &[("rectangle", t!("Rectangle")), ("ellipse", t!("Ellipse")), ("star", t!("Star"))], "rectangle")
        .slider("size", t!("Size"), 8.0..=400.0, 80.0)
        .int("count", t!("Count"), 1..=MAX_COUNT, 3)
        .color("color", t!("Colour"), [0.20, 0.50, 0.90, 1.0], false)
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
        Event::Command { id } if id == CREATE => create_shapes(),
        Event::Ui { surface, widget, value, .. } if surface == PANEL => {
            if widget == CREATE {
                return create_shapes();
            }
            set_value(&widget, value)?;
            render()
        }
        Event::Tool { phase, at } => tool(&phase, &at),
        Event::App { name, .. } => {
            state().last_event = Some(name);
            render()
        }
        _ => Ok(()),
    }
}

/// Stores one edited form value after the form validates it.
fn set_value(key: &str, raw: JsonValue) -> Result<(), String> {
    let value: Value = craft_guest::serde_json::from_value(raw).map_err(|e| e.to_string())?;
    let mut candidate = values();
    candidate.insert(key.to_owned(), value);
    form().0.validate_values(&candidate).map_err(|e| e.to_string())?;
    state().values = Some(candidate);
    Ok(())
}

fn hex(c: [f32; 4]) -> String {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", b(c[0]), b(c[1]), b(c[2]))
}

/// Adds `count` shapes in a row. One event, so one undo step for the whole row.
fn create_shapes() -> Result<(), String> {
    let v = values();
    let p = Params(&v);
    let (shape, size, count, color) = (p.choice("shape")?.to_owned(), p.f("size")?, p.int("count")?.clamp(1, MAX_COUNT), hex(p.color("color")?));
    let mut made = 0;
    for i in 0..count {
        let args = json!({
            "type": shape, "name": format!("Shape {}", i + 1),
            "x": i as f64 * (size + 16.0), "y": 0.0, "width": size, "height": size,
            "props": {"fill": color}, "select": false
        });
        match craft_guest::call("node.create", args) {
            Ok(_) => made += 1,
            Err(CallError::PermissionNeeded(_)) => {
                state().notice = Some(format!("Creating shapes needs \"{NEED_EDIT}\". Turn it on in the plugin's settings."));
                // Nothing was created (every create needs the same permission): redraw, keep the event.
                return render();
            }
            // Anything else aborts the event; the host rolls back the shapes already made.
            Err(e) => return Err(e.to_string()),
        }
    }
    state().notice = Some(format!("Created {made} {shape} layer(s)."));
    render()
}

fn point(at: &JsonValue) -> Result<[f64; 2], String> {
    let x = at[0].as_f64().filter(|v| v.is_finite()).ok_or("bad pointer position")?;
    let y = at[1].as_f64().filter(|v| v.is_finite()).ok_or("bad pointer position")?;
    Ok([x, y])
}

/// Normalised box from two corners, at least one unit on each side.
fn rect(a: [f64; 2], b: [f64; 2]) -> (f64, f64, f64, f64) {
    (a[0].min(b[0]), a[1].min(b[1]), (a[0] - b[0]).abs().max(1.0), (a[1] - b[1]).abs().max(1.0))
}

/// Draw rectangle: begin creates a live rectangle, update resizes it and moves the guide overlay,
/// commit keeps it, cancel needs nothing (the host rolls the whole gesture back).
fn tool(phase: &str, at: &JsonValue) -> Result<(), String> {
    match phase {
        "begin" => {
            let start = point(at)?;
            let color = hex(Params(&values()).color("color")?);
            let reply = craft_guest::call(
                "node.create",
                json!({"type": "rectangle", "name": "Drawn rectangle", "x": start[0], "y": start[1], "width": 1, "height": 1, "props": {"fill": color}, "select": false}),
            )
            .map_err(|e| e.to_string())?;
            state().drag = Some((start, reply["id"].as_u64()));
            guide(start, start)
        }
        "update" | "commit" => {
            let now = point(at)?;
            let Some((start, id)) = state().drag else { return Ok(()) };
            let (x, y, w, h) = rect(start, now);
            if let Some(id) = id {
                craft_guest::call("node.update", json!({"id": id, "props": {"x": x, "y": y, "width": w, "height": h}})).map_err(|e| e.to_string())?;
            }
            if phase == "commit" {
                state().drag = None;
                state().notice = Some(format!("Drew a {w:.0} × {h:.0} rectangle."));
                return render();
            }
            guide(start, now)
        }
        "cancel" => {
            state().drag = None;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// The guide: the dragged box outline, cross-hair lines through the start point, and the size.
fn guide(start: [f64; 2], now: [f64; 2]) -> Result<(), String> {
    let (x, y, w, h) = rect(start, now);
    let f = |v: f64| v as f32;
    let accent = [1.0, 0.25, 0.55, 1.0];
    let ops = vec![
        ui::draw::rect([f(x), f(y)], [f(x + w), f(y + h)], [1.0, 0.25, 0.55, 0.08], Some(accent)),
        ui::draw::line([f(start[0]) - 4000.0, f(start[1])], [f(start[0]) + 4000.0, f(start[1])], 1.0, [1.0, 0.25, 0.55, 0.5]),
        ui::draw::line([f(start[0]), f(start[1]) - 4000.0], [f(start[0]), f(start[1]) + 4000.0], 1.0, [1.0, 0.25, 0.55, 0.5]),
        ui::draw::text([f(x + w) + 6.0, f(y + h) + 6.0], &format!("{w:.0} × {h:.0}"), 12.0, accent),
    ];
    craft_guest::call("canvas.overlay.set", json!({"ops": ops})).map_err(|e| e.to_string())?;
    Ok(())
}

/// Draws the panel: the form, the create button and the latest notice and app event.
fn render() -> Result<(), String> {
    let v = values();
    let (notice, last_event) = {
        let s = state();
        (s.notice.clone(), s.last_event.clone())
    };
    let mut children = vec![ui::form(&form(), &v), ui::button(CREATE, "Create shapes")];
    if let Some(text) = notice {
        children.push(ui::label(text));
    }
    if let Some(name) = last_event {
        children.push(ui::label(format!("Last app event: {name}")));
    }
    ui::set_panel(PANEL, ui::column(children)).map_err(|e| e.to_string())?;
    Ok(())
}

session!(handle);

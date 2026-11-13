//! Original auto-layout card grid, MIT OR Apache-2.0.
use craft_guest::{Event, Form, Params, Value, Values, json, session, t};
use std::sync::Mutex;
static VALUES: Mutex<Option<Values>> = Mutex::new(None);
fn form() -> Form {
    Form::new("grid", t!("Layout grid"))
        .int("columns", t!("Columns"), 1..=12, 3)
        .int("rows", t!("Rows"), 1..=12, 2)
        .slider("size", t!("Card size"), 24.0..=400.0, 100.0)
        .slider("gap", t!("Gap"), 0.0..=80.0, 16.0)
}
fn values() -> Values {
    VALUES.lock().unwrap_or_else(|e| e.into_inner()).clone().unwrap_or_else(|| form().0.defaults())
}
fn call(op: &str, args: craft_guest::JsonValue) -> Result<craft_guest::JsonValue, String> {
    craft_guest::call(op, args).map_err(|e| e.to_string())
}
fn render() -> Result<(), String> {
    call(
        "ui.panel.set",
        json!({"panel":"panel","tree":{"type":"scroll","children":[
            {"type":"form","form":form().0,"values":values()},
            {"type":"label","style":"weak","text":"Creates ordinary editable cards in one undo step."},
            {"type":"button","id":"apply","label":"Generate grid","icon":"plus"}
        ]}}),
    )?;
    Ok(())
}
fn apply() -> Result<(), String> {
    let v = values();
    let p = Params(&v);
    let (cols, rows, size, gap) = (p.int("columns")?, p.int("rows")?, p.f("size")?, p.f("gap")?);
    let frame = call(
        "node.create",
        json!({"type":"frame","name":"Card grid","width":cols as f64*(size+gap)-gap,"height":rows as f64*(size+gap)-gap,"props":{"layout":"grid","gridColumns":cols,"gap":gap},"select":false}),
    )?;
    for i in 0..cols * rows {
        call(
            "node.create",
            json!({"type":"rectangle","name":format!("Card {}",i+1),"parent":frame["id"],"width":size,"height":size,"props":{"fill":"#3380e6","cornerRadius":8},"select":false}),
        )?;
    }
    call("layout.setTrack", json!({"ids":[frame["id"]],"axis":"columns","index":0,"sizing":"fixed","value":size}))?;
    render()
}
fn handle(e: Event) -> Result<(), String> {
    match e {
        Event::Init => render(),
        Event::Command { .. } => apply(),
        Event::Ui { widget, value, .. } => {
            if widget == "apply" {
                return apply();
            }
            let value: Value = craft_guest::serde_json::from_value(value).map_err(|e| e.to_string())?;
            let mut v = values();
            v.insert(widget, value);
            form().0.validate_values(&v)?;
            *VALUES.lock().unwrap_or_else(|e| e.into_inner()) = Some(v);
            render()
        }
        _ => Ok(()),
    }
}
session!(handle);

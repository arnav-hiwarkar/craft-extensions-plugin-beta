//! Layer Export for PhotoCraft: export layers as image files into a folder you pick.
//!
//! The panel lists the layers in a multi-select table and holds the options in a form (format,
//! file-name prefix, hidden layers, a `layers.json` index). "Export…" asks PhotoCraft for a folder
//! (`files.pick`); the answer arrives later as a `files.picked` event with an opaque handle (the
//! plugin never learns the path), and each layer is written with `document.export`. The command
//! "Export all layers…" does the same for every layer with the panel's options. The plugin needs
//! `document.read:document` and `files.write:picked` and nothing else: it cannot write anywhere
//! but the folder you picked.

use craft_guest::ui::{self, NodeExt, Selection, TextStyle};
use craft_guest::{CallError, Event, Form, JsonValue, Params, Value, Values, json, session, t};
use std::sync::Mutex;

const PANEL: &str = "export";
const COMMAND: &str = "export-all";

struct Pending {
    request: u64,
    document: u64,
    layers: Vec<u64>,
}

struct State {
    values: Option<Values>,
    selected: Vec<String>,
    layers: Vec<JsonValue>,
    pending: Option<Pending>,
    notice: Option<String>,
}

static STATE: Mutex<State> = Mutex::new(State { values: None, selected: Vec::new(), layers: Vec::new(), pending: None, notice: None });

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn form() -> Form {
    Form::new("layer-export", t!("Layer Export"))
        .choice("format", t!("Format"), &[("png", t!("PNG")), ("jpg", t!("JPEG")), ("webp", t!("WebP")), ("tif", t!("TIFF"))], "png")
        .text("prefix", t!("File name prefix"), "", false)
        .toggle("hidden", t!("Include hidden layers"), false)
        .toggle("manifest", t!("Write layers.json"), true)
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
        Event::Init => {
            reload();
            render()
        }
        Event::Command { id } if id == COMMAND => {
            reload();
            ask(None)?;
            render()
        }
        Event::Ui { surface, widget, value, .. } if surface == PANEL => {
            match widget.as_str() {
                "layers" => {
                    state().selected = value.as_array().map(|a| a.iter().filter_map(|k| k.as_str().map(str::to_owned)).collect()).unwrap_or_default()
                }
                "refresh" => reload(),
                "export" => {
                    let chosen: Vec<u64> = state().selected.iter().filter_map(|k| k.parse().ok()).collect();
                    ask((!chosen.is_empty()).then_some(chosen))?;
                }
                key => {
                    let v: Value = craft_guest::serde_json::from_value(value).map_err(|e| e.to_string())?;
                    let mut candidate = values();
                    candidate.insert(key.to_owned(), v);
                    form().0.validate_values(&candidate).map_err(|e| e.to_string())?;
                    state().values = Some(candidate);
                }
            }
            render()
        }
        // `files.picked` is newer than this SDK's event list: it arrives as raw JSON.
        Event::Unknown(raw) if raw["event"] == "files.picked" => {
            picked(&raw)?;
            render()
        }
        _ => Ok(()),
    }
}

fn reload() {
    match craft_guest::call("layer.list", json!({})) {
        Ok(list) => {
            let mut s = state();
            s.layers = list.as_array().cloned().unwrap_or_default();
            let ids: Vec<String> = s.layers.iter().filter_map(|l| l["id"].as_u64()).map(|i| i.to_string()).collect();
            s.selected.retain(|k| ids.contains(k));
        }
        Err(e) => {
            let mut s = state();
            s.layers.clear();
            s.notice = Some(match e {
                CallError::NotFound(_) => "Open an image to export its layers.".into(),
                other => format!("Could not list the layers: {other}"),
            });
        }
    }
}

/// Layers worth exporting: not adjustment layers, and hidden ones only when asked.
fn exportable(layers: &[JsonValue], include_hidden: bool) -> Vec<u64> {
    layers
        .iter()
        .filter(|l| l["kind"] != "adjustment" && (include_hidden || l["visible"].as_bool().unwrap_or(true)))
        .filter_map(|l| l["id"].as_u64())
        .collect()
}

/// Asks PhotoCraft for a folder. The answer comes back as a `files.picked` event.
fn ask(layers: Option<Vec<u64>>) -> Result<(), String> {
    let include_hidden = Params(&values()).b("hidden")?;
    let layers = layers.unwrap_or_else(|| exportable(&state().layers, include_hidden));
    if layers.is_empty() {
        state().notice = Some("There are no layers to export.".into());
        return Ok(());
    }
    let document = craft_guest::call("document.info", json!({})).map_err(|e| e.to_string())?["id"].as_u64().ok_or("no document")?;
    match craft_guest::call("files.pick", json!({"mode": "folder", "title": "Export layers to"})) {
        Ok(r) => {
            let request = r["request"].as_u64().ok_or("no request number")?;
            let mut s = state();
            s.notice = Some(format!("Choose a folder for {} layer(s)…", layers.len()));
            s.pending = Some(Pending { request, document, layers });
            Ok(())
        }
        Err(CallError::PermissionNeeded(_)) => {
            state().notice = Some("Exporting needs \"files.write:picked\". Turn it on in the plugin's settings.".into());
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

/// A file name PhotoCraft accepts for a picked folder: separators and reserved characters become
/// `_`, no leading dot, at most 120 characters, unique within this export.
pub fn file_name(prefix: &str, layer: &str, ext: &str, taken: &mut Vec<String>) -> String {
    let clean: String = format!("{prefix}{layer}")
        .chars()
        .map(|c| if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c })
        .collect();
    let mut bytes = 0;
    let mut stem: String = clean
        .trim()
        .trim_start_matches('.')
        .trim_end_matches(['.', ' '])
        .chars()
        .take(120)
        .take_while(|c| {
            bytes += c.len_utf8();
            bytes <= 180
        })
        .collect();
    if stem.is_empty() {
        stem = "layer".into();
    }
    let mut name = format!("{stem}.{ext}");
    let mut n = 2;
    while taken.iter().any(|t| t.eq_ignore_ascii_case(&name)) {
        name = format!("{stem} {n}.{ext}");
        n += 1;
    }
    taken.push(name.clone());
    name
}

/// The folder answer: export each chosen layer and, if asked, an index of what was written.
fn picked(raw: &JsonValue) -> Result<(), String> {
    let Some(pending) = state().pending.take_if(|p| raw["request"].as_u64() == Some(p.request)) else { return Ok(()) };
    let Some(handle) = raw["handle"].as_str() else {
        state().notice = Some("Export cancelled.".into());
        return Ok(());
    };
    let current = craft_guest::call("document.info", json!({})).map_err(|e| e.to_string())?;
    if current["id"].as_u64() != Some(pending.document) {
        state().notice = Some("Export cancelled: the active document changed while choosing a folder.".into());
        return Ok(());
    }
    let v = values();
    let p = Params(&v);
    let (ext, prefix, manifest) = (p.choice("format")?.to_owned(), p.text("prefix")?.to_owned(), p.b("manifest")?);
    let layers = state().layers.clone();
    let mut taken = vec!["layers.json".to_string()];
    let mut index = vec![];
    for id in &pending.layers {
        let name = layers.iter().find(|l| l["id"].as_u64() == Some(*id)).and_then(|l| l["name"].as_str()).unwrap_or("layer").to_owned();
        let file = file_name(&prefix, &name, &ext, &mut taken);
        let r = craft_guest::call("document.export", json!({"handle": handle, "name": file, "layer": id})).map_err(|e| format!("{name}: {e}"))?;
        index.push(json!({"layer": id, "name": name, "file": file, "width": r["width"], "height": r["height"], "bytes": r["bytes"]}));
    }
    if manifest {
        let text = craft_guest::serde_json::to_string_pretty(&json!({"folder": raw["name"], "layers": index})).map_err(|e| e.to_string())?;
        craft_guest::call("files.write", json!({"handle": handle, "name": "layers.json", "text": text})).map_err(|e| e.to_string())?;
    }
    let message = format!("Exported {} layer(s) to {}.", index.len(), raw["name"].as_str().unwrap_or("the folder"));
    let _ = craft_guest::call("ui.notify", json!({"message": message}));
    state().notice = Some(message);
    Ok(())
}

fn render() -> Result<(), String> {
    let v = values();
    let s = state();
    let rows: Vec<JsonValue> = s
        .layers
        .iter()
        .filter_map(|l| {
            let id = l["id"].as_u64()?.to_string();
            let size = l["bounds"].as_array().map_or("empty".to_string(), |b| format!("{}×{}", b[2], b[3]));
            let shown = if l["visible"].as_bool().unwrap_or(true) { "" } else { " (hidden)" };
            Some(ui::table_row(&id, &[&format!("{}{shown}", l["name"].as_str().unwrap_or("")), l["kind"].as_str().unwrap_or(""), &size]))
        })
        .collect();
    let selected: Vec<&str> = s.selected.iter().map(String::as_str).collect();
    let count = if selected.is_empty() { "all layers".to_string() } else { format!("{} selected", selected.len()) };
    let mut children = vec![
        ui::text("Layers", TextStyle::Heading),
        ui::table("layers", "Layers", &["Name", "Kind", "Size"], rows).selection(Selection::Multi, &selected).max_height(220.0),
        ui::form(&form(), &v),
        ui::row(vec![
            ui::button("export", &format!("Export {count}…")).icon("save").hint("Pick a folder; nothing is written anywhere else"),
            ui::button("refresh", "Refresh layers").icon("refresh").icon_only(),
        ]),
    ];
    if s.pending.is_some() {
        children.push(ui::progress(None, Some("Waiting for a folder")));
    }
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
    fn file_names_are_safe_and_unique() {
        let mut taken = vec!["layers.json".to_string()];
        assert_eq!(file_name("", "Sky", "png", &mut taken), "Sky.png");
        assert_eq!(file_name("", "Sky", "png", &mut taken), "Sky 2.png");
        assert_eq!(file_name("x-", "a/b:c", "jpg", &mut taken), "x-a_b_c.jpg");
        assert_eq!(file_name("", "..hidden", "png", &mut taken), "hidden.png");
        assert_eq!(file_name("", "layers", "json", &mut taken), "layers 2.json");
        assert_eq!(file_name("", "", "png", &mut taken), "layer.png");
        assert!(file_name("", &"🌈".repeat(120), "png", &mut taken).len() <= 255);
    }

    #[test]
    fn adjustment_and_hidden_layers_are_left_out_unless_asked() {
        let layers = vec![
            json!({"id": 1, "kind": "pixel", "visible": true}),
            json!({"id": 2, "kind": "adjustment", "visible": true}),
            json!({"id": 3, "kind": "type", "visible": false}),
        ];
        assert_eq!(exportable(&layers, false), vec![1]);
        assert_eq!(exportable(&layers, true), vec![1, 3]);
    }

    #[test]
    fn the_panel_is_a_valid_tree() {
        let tree = ui::column(vec![
            ui::table("layers", "Layers", &["Name", "Kind", "Size"], vec![ui::table_row("1", &["Sky", "pixel", "4×4"])])
                .selection(Selection::Multi, &["1"]),
            ui::form(&form(), &values()),
            ui::progress(None, Some("Waiting")),
        ]);
        ui::check(&tree).unwrap();
    }
}

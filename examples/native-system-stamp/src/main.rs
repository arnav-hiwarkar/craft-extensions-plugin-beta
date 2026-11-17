use craft_native::{Event, Hello, Host, json};

fn system() -> Result<String, String> {
    let out = std::process::Command::new("/usr/bin/uname").arg("-sm").output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("uname failed".into());
    }
    let text = String::from_utf8(out.stdout).map_err(|e| e.to_string())?;
    let text = text.trim();
    if text.is_empty() || text.len() > 256 {
        return Err("Invalid uname output".into());
    }
    Ok(format!("System: {text}"))
}

fn handle(host: &mut Host, hello: &Hello, event: Event) -> Result<(), String> {
    let app = hello.app.as_str();
    if app != "effectcraft" && app != "figcraft" {
        return Err("Unsupported app".into());
    }
    let stamp = match &event {
        Event::Init => {
            let tree = json!({"type":"column","children":[
                {"type":"label","text":"Runs /usr/bin/uname locally. Full access to your computer is required."},
                {"type":"button","id":"stamp","label":"Insert system stamp"}
            ]});
            host.call("ui.panel.set", json!({"panel": format!("{app}-panel"), "tree": tree})).map_err(|e| e.to_string())?;
            false
        }
        Event::Command { .. } => true,
        Event::Ui { widget, .. } => widget == "stamp",
        _ => false,
    };
    if stamp {
        let text = system()?;
        let (op, args) = if app == "effectcraft" {
            ("layer.newText", json!({"text": text, "name": "System stamp"}))
        } else {
            ("text.create", json!({"text": text, "x": 0, "y": 0, "name": "System stamp"}))
        };
        host.call(op, args).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    craft_native::run(handle)
}

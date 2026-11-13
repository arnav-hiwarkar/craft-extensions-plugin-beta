//! JSON-lines helper. stdout belongs exclusively to protocol 1.
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
fn send(v: Value) -> Result<(), String> {
    let mut out = io::stdout().lock();
    writeln!(out, "{v}").and_then(|_| out.flush()).map_err(|e| e.to_string())
}
fn read(input: &mut impl BufRead) -> Result<Value, String> {
    let mut line = String::new();
    if input.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
        return Err("Host closed input".into());
    }
    serde_json::from_str(&line).map_err(|e| e.to_string())
}
fn call(input: &mut impl BufRead, op: &str, args: Value) -> Result<Value, String> {
    send(json!({"type":"call","id":1,"op":op,"args":args}))?;
    let v = read(input)?;
    if v["type"] != "result" || v["id"] != 1 {
        return Err("Unexpected host response".into());
    }
    if let Some(error) = v.get("error") {
        return Err(error.to_string());
    }
    Ok(v["ok"].clone())
}
fn system() -> Result<String, String> {
    // Fixed executable and arguments, no shell, paths or arguments supplied by a document.
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
fn run() -> Result<(), String> {
    let mut input = io::BufReader::new(io::stdin());
    let hello = read(&mut input)?;
    if hello["type"] != "hello" || hello["protocol"] != 1 {
        return Err("Protocol 1 required".into());
    }
    let app = hello["app"].as_str().ok_or("Missing app")?.to_owned();
    if app != "effectcraft" && app != "figcraft" {
        return Err("Unsupported app".into());
    }
    send(json!({"type":"ready"}))?;
    loop {
        let message = match read(&mut input) {
            Ok(v) => v,
            Err(_) => return Ok(()),
        };
        if message["type"] != "event" {
            return Err("Expected event".into());
        }
        let event = &message["event"];
        let result = (|| {
            match event["event"].as_str() {
                Some("init") => {
                    call(
                        &mut input,
                        "ui.panel.set",
                        json!({"panel":format!("{app}-panel"),"tree":{"type":"column","children":[
                            {"type":"label","text":"Runs /usr/bin/uname locally. Full access to your computer is required."},
                            {"type":"button","id":"stamp","label":"Insert system stamp"}
                        ]}}),
                    )?;
                }
                Some("command") | Some("ui") if event["event"] == "command" || event["widget"] == "stamp" => {
                    let text = system()?;
                    let (op, args) = if app == "effectcraft" {
                        ("layer.newText", json!({"text":text,"name":"System stamp"}))
                    } else {
                        ("text.create", json!({"text":text,"x":0,"y":0,"name":"System stamp"}))
                    };
                    call(&mut input, op, args)?;
                }
                _ => {}
            }
            Ok::<(), String>(())
        })();
        send(match result {
            Ok(()) => json!({"type":"done","id":message["id"],"code":0}),
            Err(e) => json!({"type":"done","id":message["id"],"code":-7,"message":e}),
        })?;
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

//! Native plugins: never loaded into the app. The host starts the helper executable and speaks the
//! session protocol as JSON lines over stdin/stdout (beta-design B6).
//!
//! Protocol 1 (one JSON object per line, ≤ 1 MiB per line):
//! - host → helper: `{"type":"hello","protocol":1,"app":"effectcraft","contracts":[["effectcraft.session","1.0"]]}`
//! - helper → host: `{"type":"ready"}`
//! - host → helper: `{"type":"event","id":7,"event":{…}}`
//! - helper → host: `{"type":"call","id":1,"op":"doc.selection","args":{…}}` → host: `{"type":"result","id":1,"ok":…}` or `{"type":"result","id":1,"error":{…}}`
//! - helper → host: `{"type":"log","level":1,"text":"…"}`
//! - helper → host: `{"type":"done","id":7,"code":0}` (ends the event's transaction)
//!
//! Invariants: the helper gets a clean environment and its own working folder; every call it makes is
//! checked for full-access consent, enablement and the kill list (permissions are moot: it already has
//! full access); a stopped plugin's process is killed; timeouts kill the process; output lines are
//! size-capped. Killing cannot undo effects the helper already caused.

use crate::api::{AppApi, Code, Grants, Invocation, OpError};
use crate::wasm::{LogFn, PluginError};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

pub const PROTOCOL: u32 = 1;
const MAX_LINE: u64 = 1024 * 1024;
const START_TIMEOUT: Duration = Duration::from_secs(10);
const EVENT_TIMEOUT: Duration = Duration::from_secs(60);

/// The current platform key used in `native.executables` (`linux-x86_64`, `macos-aarch64`, …).
pub fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

pub struct Helper {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<Option<Value>>,
    api: Arc<dyn AppApi>,
    grants: Arc<dyn Grants>,
    log: LogFn,
    next_id: u64,
}

impl Helper {
    /// Starts `exe` (inside the release folder) with `work` as its working folder.
    pub fn start(exe: &Path, work: &Path, api: Arc<dyn AppApi>, grants: Arc<dyn Grants>, log: LogFn) -> Result<Self, PluginError> {
        let mut child = Command::new(exe)
            .current_dir(work)
            .env_clear()
            .env("CRAFT_PLUGIN_PROTOCOL", PROTOCOL.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| PluginError::Load(format!("could not start the helper: {e}")))?;
        let stdin = child.stdin.take().ok_or_else(|| PluginError::Load("no stdin".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| PluginError::Load("no stdout".into()))?;
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                match (&mut reader).take(MAX_LINE).read_line(&mut line) {
                    Ok(0) | Err(_) => {
                        let _ = tx.send(None);
                        return;
                    }
                    Ok(_) => {
                        let v = serde_json::from_str::<Value>(line.trim()).ok();
                        if tx.send(v).is_err() {
                            return;
                        }
                    }
                }
            }
        });
        let mut h = Helper { child, stdin, lines: rx, api, grants, log, next_id: 1 };
        let contracts: Vec<(String, String)> =
            h.api.contract().provided.contracts.iter().map(|(n, l)| (n.to_string(), l.to_string())).collect();
        h.send(&json!({"type":"hello","protocol":PROTOCOL,"app":h.api.contract().app.as_str(),"contracts":contracts}))?;
        match h.lines.recv_timeout(START_TIMEOUT) {
            Ok(Some(v)) if v["type"] == "ready" => Ok(h),
            _ => {
                h.kill();
                Err(PluginError::Load("the helper did not start correctly".into()))
            }
        }
    }

    fn send(&mut self, v: &Value) -> Result<(), PluginError> {
        let mut line = serde_json::to_vec(v).map_err(|e| PluginError::Trap(e.to_string()))?;
        line.push(b'\n');
        self.stdin.write_all(&line).and_then(|()| self.stdin.flush()).map_err(|_| PluginError::Trap("the helper stopped".into()))
    }

    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    fn check(&self, inv: &Invocation) -> Result<(), OpError> {
        if !self.grants.full_access(&inv.plugin) {
            return Err(OpError::denied("full access is no longer allowed for this plugin"));
        }
        if self.grants.stopped(&inv.plugin, inv.version) {
            return Err(OpError::denied("this plugin has been turned off"));
        }
        if inv.cancelled() {
            return Err(OpError::new(Code::Cancelled, "cancelled"));
        }
        Ok(())
    }

    /// One event = one transaction, as for wasm sessions.
    pub fn dispatch(&mut self, inv: Invocation, event: &Value) -> Result<(), PluginError> {
        if let Err(e) = self.check(&inv) {
            self.kill();
            return Err(PluginError::Stopped(e.message));
        }
        self.api.begin(&inv).map_err(|e| PluginError::Failed(e.code as i32, e.message))?;
        let id = self.next_id;
        self.next_id += 1;
        let result = self.run(&inv, id, event);
        let commit = result.is_ok() && self.check(&inv).is_ok();
        self.api.finish(&inv, commit);
        if let Err(PluginError::Stopped(_) | PluginError::TooSlow | PluginError::Trap(_)) = &result {
            self.kill();
        }
        result
    }

    fn run(&mut self, inv: &Invocation, id: u64, event: &Value) -> Result<(), PluginError> {
        self.send(&json!({"type":"event","id":id,"event":event}))?;
        loop {
            let msg = match self.lines.recv_timeout(EVENT_TIMEOUT) {
                Ok(Some(v)) => v,
                Ok(None) | Err(RecvTimeoutError::Disconnected) => return Err(PluginError::Trap("the helper stopped".into())),
                Err(RecvTimeoutError::Timeout) => return Err(PluginError::TooSlow),
            };
            match msg["type"].as_str() {
                Some("log") => (self.log)(inv.plugin.as_str(), msg["level"].as_i64().unwrap_or(1).clamp(0, 3) as i32, msg["text"].as_str().unwrap_or("")),
                Some("call") => {
                    let call_id = msg["id"].clone();
                    let op = msg["op"].as_str().unwrap_or("").to_string();
                    let out = self.check(inv).and_then(|()| {
                        if self.api.contract().op(&op).is_none() {
                            return Err(OpError::new(Code::NotFound, format!("this app has no operation {op}")));
                        }
                        self.api.call(inv, &op, msg["args"].clone())
                    });
                    let stopped = matches!(&out, Err(e) if e.code == Code::Denied);
                    let reply = match out {
                        Ok(v) => json!({"type":"result","id":call_id,"ok":v}),
                        Err(e) => json!({"type":"result","id":call_id,"error":e.to_json()}),
                    };
                    self.send(&reply)?;
                    if stopped {
                        return Err(PluginError::Stopped("full access withdrawn or plugin turned off".into()));
                    }
                }
                Some("done") if msg["id"].as_u64() == Some(id) => {
                    let code = msg["code"].as_i64().unwrap_or(-7) as i32;
                    return if code == 0 { Ok(()) } else { Err(PluginError::Failed(code, msg["message"].as_str().unwrap_or("").to_string())) };
                }
                _ => return Err(PluginError::InvalidOutput("the helper sent an unknown message".into())),
            }
        }
    }
}

impl Drop for Helper {
    fn drop(&mut self) {
        self.kill();
    }
}

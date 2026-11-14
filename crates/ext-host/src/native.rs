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
use craft_ext_core::id::ExtensionId;
use craft_ext_core::version::Version;
use std::sync::mpsc::{Receiver, RecvTimeoutError, sync_channel};
use std::time::{Duration, Instant};

pub const PROTOCOL: u32 = 1;
const MAX_LINE: u64 = 1024 * 1024;
const START_TIMEOUT: Duration = Duration::from_secs(10);
/// Absolute limit for one event, however chatty the helper is.
const EVENT_TIMEOUT: Duration = Duration::from_secs(60);
/// Lines buffered from the helper; when full the reader blocks (backpressure), so a flooding helper
/// cannot grow host memory beyond `QUEUE_LINES * MAX_LINE`.
const QUEUE_LINES: usize = 64;

/// The current platform key used in `native.executables` (`linux-x86_64`, `macos-aarch64`, …).
/// Stops every process in the helper's group (the helper started it with `process_group(0)`, so
/// the group id is its pid). Std has no killpg; the system `kill` does it without unsafe code.
/// Best effort: a process that left the group on purpose is not covered (native is not a sandbox).
#[cfg(unix)]
fn kill_group(pid: u32) {
    for kill in ["/bin/kill", "/usr/bin/kill"] {
        if Path::new(kill).exists() {
            let _ = Command::new(kill).args(["-KILL", "--", &format!("-{pid}")]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
            return;
        }
    }
}
#[cfg(not(unix))]
fn kill_group(_pid: u32) {}

pub fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

pub struct Helper {
    child: Child,
    /// The helper was reaped; its pid (and so its group id) must not be signalled again.
    ended: bool,
    stdin: ChildStdin,
    lines: Receiver<Option<Value>>,
    api: Arc<dyn AppApi>,
    grants: Arc<dyn Grants>,
    log: LogFn,
    next_id: u64,
    plugin: ExtensionId,
    version: Version,
}

impl Helper {
    /// Starts `exe` (inside the release folder) with `work` as its working folder. Refuses before
    /// spawning anything unless the plugin has full-access consent and is not stopped.
    pub fn start(
        plugin: &ExtensionId,
        version: Version,
        exe: &Path,
        work: &Path,
        api: Arc<dyn AppApi>,
        grants: Arc<dyn Grants>,
        log: LogFn,
    ) -> Result<Self, PluginError> {
        if !grants.full_access(plugin) || grants.stopped(plugin, version) {
            return Err(PluginError::Stopped("full access has not been allowed for this plugin".into()));
        }
        let mut command = Command::new(exe);
        // Its own process group, so stopping the helper also stops processes it started.
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        let mut child = command
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
        let (tx, rx) = sync_channel(QUEUE_LINES);
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
        let mut h = Helper { child, ended: false, stdin, lines: rx, api, grants, log, next_id: 1, plugin: plugin.clone(), version };
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
        if self.ended {
            return;
        }
        // Before reaping: an unreaped helper keeps its pid, so the group id cannot be reused.
        kill_group(self.child.id());
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.ended = true;
    }

    /// Supervision: call periodically (the adapter does on its tick). Kills the helper as soon as
    /// consent is withdrawn or the plugin is stopped, even while idle. Returns false once killed.
    /// Adapters must call this at least every second while the helper is idle.
    pub fn supervise(&mut self) -> bool {
        if !self.grants.full_access(&self.plugin) || self.grants.stopped(&self.plugin, self.version) {
            self.kill();
            return false;
        }
        match self.child.try_wait() {
            Ok(None) => true,
            _ => {
                // It exited on its own: stop what it left running. Surviving members keep the
                // group id reserved, so this cannot reach an unrelated process group.
                if !self.ended {
                    kill_group(self.child.id());
                    self.ended = true;
                }
                false
            }
        }
    }

    fn check(&self, inv: &Invocation) -> Result<(), OpError> {
        if !self.grants.full_access(&inv.plugin) {
            return Err(OpError::denied("full access is no longer allowed for this plugin"));
        }
        if self.grants.stopped(&inv.plugin, inv.version) {
            return Err(OpError::denied("this plugin has been turned off"));
        }
        if self.grants.generation(&inv.plugin) != inv.grant_generation {
            return Err(OpError::denied("permissions changed while this was running"));
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
        let result = self.run(&inv, id, event).and_then(|()| self.check(&inv).map_err(|e| PluginError::Stopped(e.message)));
        let commit = result.is_ok();
        self.api.finish(&inv, commit);
        if let Err(PluginError::Stopped(_) | PluginError::TooSlow | PluginError::Trap(_)) = &result {
            self.kill();
        }
        result
    }

    fn run(&mut self, inv: &Invocation, id: u64, event: &Value) -> Result<(), PluginError> {
        self.send(&json!({"type":"event","id":id,"event":event}))?;
        let deadline = Instant::now() + EVENT_TIMEOUT;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(PluginError::TooSlow);
            }
            if self.check(inv).is_err() {
                return Err(PluginError::Stopped("full access withdrawn or plugin turned off".into()));
            }
            let msg = match self.lines.recv_timeout(left.min(Duration::from_millis(250))) {
                Ok(Some(v)) => v,
                Ok(None) | Err(RecvTimeoutError::Disconnected) => return Err(PluginError::Trap("the helper stopped".into())),
                Err(RecvTimeoutError::Timeout) => continue,
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
                    let stopped = self.check(inv).is_err();
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

#[cfg(all(test, unix))]
mod final_validation_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use crate::api::AppContract;
    use craft_ext_core::{contract::Provided, id::AppId, permission::Permission};
    use std::sync::{Mutex, atomic::{AtomicBool, AtomicU64, Ordering}};

    struct App {
        contract: AppContract,
        commits: Mutex<Vec<bool>>,
    }
    impl AppApi for App {
        fn contract(&self) -> &AppContract { &self.contract }
        fn begin(&self, _: &Invocation) -> Result<(), OpError> { Ok(()) }
        fn call(&self, _: &Invocation, _: &str, _: Value) -> Result<Value, OpError> { Ok(Value::Null) }
        fn finish(&self, _: &Invocation, commit: bool) { self.commits.lock().unwrap().push(commit); }
    }
    struct RevokedAtFinish(AtomicU64);
    impl Grants for RevokedAtFinish {
        fn generation(&self, _: &ExtensionId) -> u64 {
            // Initial dispatch and the single queued response are valid; finalization is stale.
            if self.0.fetch_add(1, Ordering::Relaxed) < 2 { 1 } else { 2 }
        }
        fn allowed(&self, _: &ExtensionId, _: &Permission) -> bool { false }
        fn stopped(&self, _: &ExtensionId, _: Version) -> bool { false }
        fn full_access(&self, _: &ExtensionId) -> bool { true }
    }

    #[test]
    fn final_revocation_returns_failure_rolls_back_and_kills_helper() {
        let app = Arc::new(App {
            contract: AppContract {
                app: AppId::new("testcraft").unwrap(), display_name: "Test",
                provided: Provided::new(&[]), ops: vec![], events: vec![], slots: vec![], notes: "",
            },
            commits: Mutex::new(vec![]),
        });
        let inv = Invocation {
            app: AppId::new("testcraft").unwrap(), plugin: ExtensionId::new("org.example.test").unwrap(),
            version: Version::new(1, 0, 0), contribution: "test".into(), grant_generation: 1,
            cancel: Arc::new(AtomicBool::new(false)), deadline: None, label: "Test".into(), declared: vec![].into(),
        };
        let mut child = Command::new("sh").args(["-c", "while read line; do :; done"])
            .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        let stdin = child.stdin.take().unwrap();
        let (tx, lines) = sync_channel(1);
        // Prequeue the protocol result so the test has no response timing dependency.
        tx.send(Some(json!({"type":"done", "id":1, "code":0}))).unwrap();
        let mut helper = Helper {
            child, ended: false, stdin, lines, api: app.clone(), grants: Arc::new(RevokedAtFinish(AtomicU64::new(0))),
            log: Arc::new(|_, _, _| {}), next_id: 1, plugin: inv.plugin.clone(), version: inv.version,
        };
        assert!(matches!(helper.dispatch(inv, &json!({})), Err(PluginError::Stopped(_))));
        assert_eq!(*app.commits.lock().unwrap(), vec![false]);
        assert!(helper.child.try_wait().unwrap().is_some());
    }

    #[test]
    fn killing_a_helper_stops_the_processes_it_started() {
        let dir = tempfile::tempdir().unwrap();
        let pidfile = dir.path().join("child.pid");
        let script = format!("sleep 30 & echo $! > '{}'; while read line; do :; done", pidfile.display());
        let mut command = Command::new("sh");
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        let mut child = command.args(["-c", &script]).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        let stdin = child.stdin.take().unwrap();
        let (_tx, lines) = sync_channel(1);
        let app = Arc::new(App {
            contract: AppContract {
                app: AppId::new("testcraft").unwrap(), display_name: "Test",
                provided: Provided::new(&[]), ops: vec![], events: vec![], slots: vec![], notes: "",
            },
            commits: Mutex::new(vec![]),
        });
        let mut helper = Helper {
            child, ended: false, stdin, lines, api: app, grants: Arc::new(RevokedAtFinish(AtomicU64::new(0))),
            log: Arc::new(|_, _, _| {}), next_id: 1, plugin: ExtensionId::new("org.example.test").unwrap(), version: Version::new(1, 0, 0),
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let pid = loop {
            if let Some(pid) = std::fs::read_to_string(&pidfile).ok().and_then(|s| s.trim().parse::<u32>().ok()) {
                break pid;
            }
            assert!(std::time::Instant::now() < deadline, "the helper never started its child");
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        helper.kill();
        helper.kill(); // a second kill (Drop) must not signal anything again
        let alive = |pid: u32| Command::new("kill").args(["-0", &pid.to_string()]).stderr(Stdio::null()).status().is_ok_and(|s| s.success());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while alive(pid) && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(!alive(pid), "the helper's child process survived");
    }
}


use crate::api::{AppApi, Code, Grants, Invocation, OpError};
use crate::jobs::{JobBoard, JobState, Ticket};
use crate::wasm::{LogFn, PluginError};
use std::collections::BTreeMap;
use std::path::PathBuf;
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
const EVENT_TIMEOUT: Duration = Duration::from_secs(60);
const QUEUE_LINES: usize = 64;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Confinement {
    Full,
    Restricted { release: PathBuf, data: PathBuf },
}

fn seatbelt_quote(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn seatbelt_profile(exe: &Path, release: &Path, data: &Path) -> String {
    let (exe, release, data) = (seatbelt_quote(exe), seatbelt_quote(release), seatbelt_quote(data));
    format!(
        r#"(version 1)
(deny default)
(allow process-exec (literal "{exe}"))
(allow file-read* (literal "{exe}"))
(allow file-read-metadata)
(allow file-read* (literal "/") (subpath "{release}") (subpath "{data}") (subpath "/usr/lib") (subpath "/usr/share") (subpath "/System") (subpath "/Library/Apple") (subpath "/private/var/db/dyld") (subpath "/private/var/db/timezone") (literal "/dev/null") (literal "/dev/random") (literal "/dev/urandom") (literal "/private/etc/localtime"))
(allow file-write* (subpath "{data}") (literal "/dev/null"))
(allow sysctl-read)
(allow mach-lookup (global-name "com.apple.system.opendirectoryd.libinfo") (global-name "com.apple.system.notification_center"))
(allow ipc-posix-shm-read-data)
(allow signal (target self))
"#
    )
}

fn confined_command(exe: &Path, confinement: &Confinement) -> Result<Command, String> {
    let Confinement::Restricted { release, data } = confinement else { return Ok(Command::new(exe)) };
    if cfg!(target_os = "macos") {
        let sandbox = Path::new("/usr/bin/sandbox-exec");
        if !sandbox.exists() {
            return Err("this Mac has no sandbox-exec, so the restricted helper cannot be confined".into());
        }
        let mut c = Command::new(sandbox);
        c.arg("-p").arg(seatbelt_profile(exe, release, data)).arg(exe);
        return Ok(c);
    }
    if cfg!(target_os = "linux") {
        let bwrap = ["/usr/bin/bwrap", "/bin/bwrap"].into_iter().map(Path::new).find(|p| p.exists());
        let Some(bwrap) = bwrap else { return Err("restricted native helpers need bubblewrap (bwrap) on Linux".into()) };
        let mut c = Command::new(bwrap);
        c.args(["--unshare-all", "--die-with-parent", "--new-session", "--proc", "/proc", "--dev", "/dev", "--tmpfs", "/tmp"]);
        for sys in ["/usr/lib", "/usr/lib64", "/usr/lib32", "/lib", "/lib64", "/lib32", "/etc/ld.so.cache", "/etc/localtime"] {
            if Path::new(sys).exists() {
                c.args(["--ro-bind", sys, sys]);
            }
        }
        c.arg("--ro-bind").arg(release).arg(release).arg("--bind").arg(data).arg(data).arg("--chdir").arg(data).arg(exe);
        return Ok(c);
    }
    Err("restricted native helpers are not available on this operating system yet".into())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelperLaunch {
    pub exe: PathBuf,
    pub work: PathBuf,
    pub confinement: Confinement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Launch {
    Wasm(PathBuf),
    Native(HelperLaunch),
}

impl Launch {
    pub fn plan(manifest: &craft_ext_core::manifest::Manifest, c: &craft_ext_core::manifest::Contribution, release: &Path, data: &Path) -> Result<Self, String> {
        use craft_ext_core::manifest::{NativeMode, Runtime};
        match manifest.runtime(c) {
            Runtime::Data => Err(format!("contribution {} runs no code", c.id)),
            Runtime::Wasm(module) => Ok(Self::Wasm(release.join(module))),
            Runtime::Native(mode) => {
                let native = manifest.native.as_ref().ok_or("no native helper")?;
                let exe = native.executables.get(&platform()).ok_or_else(|| format!("no native helper for {}", platform()))?;
                let confinement = match mode {
                    NativeMode::Full => Confinement::Full,
                    NativeMode::Restricted => Confinement::Restricted { release: release.to_path_buf(), data: data.to_path_buf() },
                };
                Ok(Self::Native(HelperLaunch { exe: release.join(exe), work: data.to_path_buf(), confinement }))
            }
        }
    }
}

pub fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

pub struct Helper {
    child: Child,
    ended: bool,
    stdin: ChildStdin,
    lines: Receiver<Option<Value>>,
    api: Arc<dyn AppApi>,
    grants: Arc<dyn Grants>,
    log: LogFn,
    next_id: u64,
    plugin: ExtensionId,
    version: Version,
    jobs: Option<Arc<JobBoard>>,
    live_jobs: BTreeMap<u64, Arc<Ticket>>,
    progress: Option<crate::wasm::ProgressFn>,
}

impl Helper {
    pub fn start(
        plugin: &ExtensionId,
        version: Version,
        launch: &HelperLaunch,
        api: Arc<dyn AppApi>,
        grants: Arc<dyn Grants>,
        log: LogFn,
    ) -> Result<Self, PluginError> {
        if !grants.full_access(plugin) || grants.stopped(plugin, version) {
            return Err(PluginError::Stopped("native code has not been allowed for this plugin".into()));
        }
        let HelperLaunch { exe, work, confinement } = launch;
        let mut command = confined_command(exe, confinement).map_err(PluginError::Load)?;
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
        let mut h = Helper {
            child,
            ended: false,
            stdin,
            lines: rx,
            api,
            grants,
            log,
            next_id: 1,
            plugin: plugin.clone(),
            version,
            jobs: None,
            live_jobs: BTreeMap::new(),
            progress: None,
        };
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

    pub fn set_jobs(&mut self, board: Arc<JobBoard>) {
        self.jobs = Some(board);
    }

    pub fn set_progress(&mut self, progress: crate::wasm::ProgressFn) {
        self.progress = Some(progress);
    }

    fn handle_async(&mut self, msg: &Value) -> bool {
        let job = msg["job"].as_u64();
        match (msg["type"].as_str(), job) {
            (Some("progress"), Some(id)) => {
                if let Some(t) = self.live_jobs.get(&id) {
                    t.progress(msg["value"].as_f64().map(|v| v as f32), msg["note"].as_str().unwrap_or(""));
                }
                true
            }
            (Some("progress"), None) => {
                if let Some(p) = &self.progress {
                    p(msg["value"].as_f64().map(|v| v as f32), msg["note"].as_str().unwrap_or(""));
                }
                true
            }
            (Some("finish"), Some(id)) => {
                if let Some(t) = self.live_jobs.remove(&id) {
                    let state = match msg.get("error") {
                        Some(e) => JobState::Failed(e.as_str().unwrap_or("the helper reported an error").chars().take(300).collect()),
                        None => JobState::Done(msg.get("result").cloned().unwrap_or(Value::Null)),
                    };
                    t.finish(state);
                }
                true
            }
            (Some("log"), _) => {
                (self.log)(self.plugin.as_str(), msg["level"].as_i64().unwrap_or(1).clamp(0, 3) as i32, msg["text"].as_str().unwrap_or(""));
                true
            }
            _ => false,
        }
    }

    fn start_job(&mut self, inv: &Invocation, args: &Value) -> Result<Value, OpError> {
        let board = self.jobs.clone().ok_or_else(|| OpError::new(Code::NotFound, "this app has no jobs list"))?;
        let label = args["label"].as_str().filter(|l| !l.trim().is_empty()).ok_or_else(|| OpError::invalid("job.start: label is required"))?;
        let ticket = board.admit(&inv.plugin, &inv.contribution, label, args["cancellable"].as_bool().unwrap_or(true), inv.document.as_deref()).map_err(|e| OpError::new(Code::Budget, e))?;
        let id = ticket.id;
        self.live_jobs.insert(id, Arc::new(ticket));
        Ok(json!({ "job": id }))
    }

    fn fail_jobs(&mut self, why: &str) {
        for (_, t) in std::mem::take(&mut self.live_jobs) {
            t.finish(JobState::Failed(why.into()));
        }
    }

    pub fn kill(&mut self) {
        self.fail_jobs("the helper stopped");
        if self.ended {
            return;
        }
        kill_group(self.child.id());
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.ended = true;
    }

    pub fn supervise(&mut self) -> bool {
        if !self.grants.full_access(&self.plugin) || self.grants.stopped(&self.plugin, self.version) {
            self.kill();
            return false;
        }
        loop {
            match self.lines.try_recv() {
                Ok(Some(msg)) => {
                    let _ = self.handle_async(&msg);
                }
                Ok(None) => {
                    self.fail_jobs("the helper stopped");
                    break;
                }
                Err(_) => break,
            }
        }
        let cancelled: Vec<u64> = self.live_jobs.iter().filter(|(_, t)| t.cancel.load(std::sync::atomic::Ordering::Relaxed)).map(|(id, _)| *id).collect();
        for id in cancelled {
            if self.send(&json!({"type":"cancel","job":id})).is_ok()
                && let Some(t) = self.live_jobs.remove(&id)
            {
                t.finish(JobState::Cancelled);
            }
        }
        match self.child.try_wait() {
            Ok(None) => true,
            _ => {
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

    pub fn dispatch(&mut self, inv: Invocation, event: &Value) -> Result<(), PluginError> {
        if let Err(e) = self.check(&inv) {
            self.kill();
            return Err(PluginError::Stopped(e.message));
        }
        crate::api::take_panicked();
        crate::api::contain("begin", || self.api.begin(&inv)).map_err(|e| PluginError::Failed(e.code as i32, e.message))?;
        let id = self.next_id;
        self.next_id += 1;
        let result = self.run(&inv, id, event).and_then(|()| self.check(&inv).map_err(|e| PluginError::Stopped(e.message)));
        let result = if crate::api::take_panicked() && result.is_ok() { Err(PluginError::Failed(Code::Internal as i32, "the app failed while serving this plugin; the change was rolled back".into())) } else { result };
        let commit = result.is_ok();
        let _ = crate::api::contain("finish", || {
            self.api.finish(&inv, commit);
            Ok(())
        });
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
            if self.handle_async(&msg) {
                continue;
            }
            match msg["type"].as_str() {
                Some("call") => {
                    let call_id = msg["id"].clone();
                    let op = msg["op"].as_str().unwrap_or("").to_string();
                    let out = self.check(inv).and_then(|()| {
                        if op == "job.start" {
                            return self.start_job(inv, &msg["args"]);
                        }
                        if self.api.contract().op(&op).is_none() {
                            return Err(OpError::new(Code::NotFound, format!("this app has no operation {op}")));
                        }
                        crate::api::authorize_call(self.api.as_ref(), self.grants.as_ref(), inv, &op, &msg["args"])?;
                        crate::api::contain(&op, || self.api.call(inv, &op, msg["args"].clone()))
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
            cancel: Arc::new(AtomicBool::new(false)), deadline: None, label: "Test".into(), declared: vec![].into(), document: None,
        };
        let mut child = Command::new("sh").args(["-c", "while read line; do :; done"])
            .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        let stdin = child.stdin.take().unwrap();
        let (tx, lines) = sync_channel(1);
        tx.send(Some(json!({"type":"done", "id":1, "code":0}))).unwrap();
        let mut helper = Helper {
            child, ended: false, stdin, lines, api: app.clone(), grants: Arc::new(RevokedAtFinish(AtomicU64::new(0))),
            log: Arc::new(|_, _, _| {}), next_id: 1, plugin: inv.plugin.clone(), version: inv.version,
            jobs: None, live_jobs: BTreeMap::new(), progress: None,
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
            jobs: None, live_jobs: BTreeMap::new(), progress: None,
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
        helper.kill();
        let alive = |pid: u32| Command::new("kill").args(["-0", &pid.to_string()]).stderr(Stdio::null()).status().is_ok_and(|s| s.success());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while alive(pid) && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(!alive(pid), "the helper's child process survived");
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn a_restricted_helper_writes_only_its_data_folder_and_reaches_no_network() {
        let dir = tempfile::tempdir().unwrap();
        let (release, data, outside) = (dir.path().join("release"), dir.path().join("data"), dir.path().join("outside"));
        for d in [&release, &data, &outside] {
            std::fs::create_dir_all(d).unwrap();
        }
        let (release, data, outside) = (release.canonicalize().unwrap(), data.canonicalize().unwrap(), outside.canonicalize().unwrap());
        let probe = format!(
            "echo ok > '{d}/in.txt'; echo no > '{o}/out.txt'; exec 3<>/dev/tcp/1.1.1.1/443 && echo net > '{d}/net.txt'; /usr/bin/true && echo exec > '{d}/exec.txt'; exit 0",
            d = data.display(),
            o = outside.display()
        );
        let helper = if cfg!(target_os = "linux") { release.join("helper") } else { PathBuf::from("/bin/bash") };
        if cfg!(target_os = "linux") {
            std::fs::copy("/bin/bash", &helper).unwrap();
        }
        let mut c = confined_command(&helper, &Confinement::Restricted { release: release.clone(), data: data.clone() }).unwrap();
        c.arg("-c").arg(&probe).current_dir(&data).stdout(Stdio::null()).stderr(Stdio::null()).status().unwrap();
        assert!(data.join("in.txt").exists(), "writing the data folder is allowed");
        assert!(!outside.join("out.txt").exists(), "writing elsewhere is refused");
        assert!(!data.join("net.txt").exists(), "the network is refused");
        assert!(!data.join("exec.txt").exists(), "other programs are refused");
    }

    #[test]
    fn restricted_profiles_quote_paths() {
        let p = seatbelt_profile(Path::new("/a/b\"c/helper"), Path::new("/rel"), Path::new("/data"));
        assert!(p.contains(r#"(literal "/a/b\"c/helper")"#), "{p}");
        assert!(p.contains("(deny default)"));
    }
}

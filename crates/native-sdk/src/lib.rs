//! Author SDK for native helpers (protocol 1). A helper is a normal executable that the app starts
//! out of process and talks to over stdin/stdout. Write the handler; this crate runs the loop.
//!
//! ```no_run
//! use craft_native::{Host, Hello, Event};
//!
//! fn handle(host: &mut Host, hello: &Hello, event: Event) -> Result<(), String> {
//!     if let Event::Command { .. } = event {
//!         let job = host.start_job("Upscale").map_err(|e| e.to_string())?;
//!         std::thread::spawn(move || {
//!             for step in 0..10 {
//!                 if job.cancelled() { return; }
//!                 job.progress(Some(step as f32 / 10.0), "upscaling");
//!             }
//!             job.finish(craft_native::json!({"tiles": 10}));
//!         });
//!     }
//!     let _ = hello;
//!     Ok(())
//! }
//!
//! fn main() -> std::process::ExitCode {
//!     craft_native::run(handle)
//! }
//! ```
//!
//! stdout belongs to the protocol: print nothing yourself; use [`Host::log`].

#![warn(missing_docs)]

pub use craft_guest::job::Outcome as JobOutcome;
pub use craft_guest::{CallError, Event};
pub use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// What the app said when it started the helper.
#[derive(Clone, Debug, PartialEq)]
pub struct Hello {
    /// The app id, such as `"photocraft"`.
    pub app: String,
    /// Contracts the app provides, as `(name, level)`.
    pub contracts: Vec<(String, String)>,
}

#[derive(Clone)]
struct Out(Arc<Mutex<Box<dyn Write + Send>>>);

impl Out {
    fn send(&self, v: &Value) {
        let mut w = self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let _ = writeln!(w, "{v}").and_then(|()| w.flush());
    }
}

type Cancels = Arc<Mutex<BTreeMap<u64, Arc<AtomicBool>>>>;

/// A background job started with [`Host::start_job`]. Move it to a worker thread; report progress
/// and finish from there. The event that started it has already ended.
pub struct Job {
    id: u64,
    finished: bool,
    out: Out,
    cancel: Arc<AtomicBool>,
    cancels: Cancels,
}

impl Job {
    /// The job's id, as delivered back in the `job` event.
    pub fn id(&self) -> u64 {
        self.id
    }
    /// Whether the person cancelled it. Check this between steps and stop early.
    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
    /// Reports `fraction` 0..=1 (or `None` while unknown) and a short note.
    pub fn progress(&self, fraction: Option<f32>, note: &str) {
        self.out.send(&json!({"type":"progress","job":self.id,"value":fraction,"note":note}));
    }
    /// Finishes with a result; the plugin's session receives it as a `job` event.
    pub fn finish(mut self, result: Value) {
        self.finished = true;
        self.out.send(&json!({"type":"finish","job":self.id,"result":result}));
    }
    /// Finishes with an error message.
    pub fn fail(mut self, message: &str) {
        self.finished = true;
        self.out.send(&json!({"type":"finish","job":self.id,"error":message}));
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        if !self.finished {
            self.out.send(&json!({"type":"finish","job":self.id,"error":"the helper dropped this job without finishing it"}));
        }
        self.cancels.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(&self.id);
    }
}

/// The app, as seen from inside one event.
pub struct Host {
    input: Box<dyn BufRead + Send>,
    out: Out,
    next_call: u64,
    cancels: Cancels,
}

impl Host {
    fn read(&mut self) -> Option<Value> {
        loop {
            let mut line = String::new();
            if self.input.read_line(&mut line).ok()? == 0 {
                return None;
            }
            let v: Value = serde_json::from_str(line.trim()).ok()?;
            if v["type"] == "cancel" {
                if let Some(flag) = v["job"].as_u64().and_then(|id| self.cancels.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(&id).cloned()) {
                    flag.store(true, Ordering::Relaxed);
                }
                continue;
            }
            return Some(v);
        }
    }

    /// Calls an app operation, checked against the plugin's grants like any other call.
    pub fn call(&mut self, op: &str, args: Value) -> Result<Value, CallError> {
        self.next_call += 1;
        let id = self.next_call;
        self.out.send(&json!({"type":"call","id":id,"op":op,"args":args}));
        let reply = self.read().ok_or_else(|| CallError::from_reply(-7, "the app closed the connection".into()))?;
        if reply["type"] != "result" || reply["id"] != id {
            return Err(CallError::from_reply(-7, "the app sent an unexpected reply".into()));
        }
        match reply.get("error") {
            Some(e) => Err(CallError::from_reply(e["error"].as_i64().unwrap_or(-7) as i32, e["message"].as_str().unwrap_or("").to_owned())),
            None => Ok(reply["ok"].clone()),
        }
    }

    /// Writes to the app's plugin console (0 = debug … 3 = error).
    pub fn log(&self, level: i32, text: &str) {
        self.out.send(&json!({"type":"log","level":level.clamp(0, 3),"text":text}));
    }

    /// Reports progress for the current event's own work.
    pub fn progress(&self, fraction: Option<f32>, note: &str) {
        self.out.send(&json!({"type":"progress","value":fraction,"note":note}));
    }

    /// Starts a background job listed in the app's jobs list.
    pub fn start_job(&mut self, label: &str) -> Result<Job, CallError> {
        let id = self.call("job.start", json!({"label": label}))?["job"].as_u64().ok_or_else(|| CallError::from_reply(-7, "job.start returned no id".into()))?;
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancels.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(id, cancel.clone());
        Ok(Job { id, finished: false, out: self.out.clone(), cancel, cancels: self.cancels.clone() })
    }
}

/// Runs the protocol on stdin/stdout until the app closes it.
pub fn run(handler: impl FnMut(&mut Host, &Hello, Event) -> Result<(), String>) -> std::process::ExitCode {
    let input = Box::new(std::io::BufReader::new(std::io::stdin()));
    match serve(input, Box::new(std::io::stdout()), handler) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(_) => std::process::ExitCode::FAILURE,
    }
}

/// [`run`] over any streams (for tests).
pub fn serve(
    input: Box<dyn BufRead + Send>,
    output: Box<dyn Write + Send>,
    mut handler: impl FnMut(&mut Host, &Hello, Event) -> Result<(), String>,
) -> Result<(), String> {
    let mut host = Host { input, out: Out(Arc::new(Mutex::new(output))), next_call: 0, cancels: Arc::default() };
    let hello = host.read().ok_or("the app sent no hello")?;
    if hello["type"] != "hello" || hello["protocol"] != 1 {
        return Err("this helper speaks protocol 1".into());
    }
    let hello = Hello {
        app: hello["app"].as_str().unwrap_or_default().to_owned(),
        contracts: hello["contracts"]
            .as_array()
            .map(|a| a.iter().filter_map(|c| Some((c[0].as_str()?.to_owned(), c[1].as_str()?.to_owned()))).collect())
            .unwrap_or_default(),
    };
    host.out.send(&json!({"type":"ready"}));
    while let Some(message) = host.read() {
        if message["type"] != "event" {
            continue;
        }
        let id = message["id"].clone();
        let outcome = handler(&mut host, &hello, Event::from_value(message["event"].clone()));
        let done = match outcome {
            Ok(()) => json!({"type":"done","id":id,"code":0}),
            Err(m) => json!({"type":"done","id":id,"code":-1,"message":m}),
        };
        host.out.send(&done);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default)]
    struct Sink(Arc<Mutex<Vec<u8>>>);
    impl Write for Sink {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_command_starts_a_job_that_reports_and_finishes_after_the_event_is_done() {
        let script = [
            json!({"type":"hello","protocol":1,"app":"photocraft","contracts":[["photocraft.session","1.0"]]}),
            json!({"type":"event","id":7,"event":{"event":"command","id":"upscale"}}),
            json!({"type":"result","id":1,"ok":{"job":3}}),
        ]
        .iter()
        .map(|v| format!("{v}\n"))
        .collect::<String>();
        let sink = Sink::default();
        let mut worker = None;
        serve(Box::new(std::io::Cursor::new(script)), Box::new(sink.clone()), |host, hello, event| {
            assert_eq!(hello.app, "photocraft");
            assert_eq!(event, Event::Command { id: "upscale".into() });
            let job = host.start_job("Upscale").map_err(|e| e.to_string())?;
            worker = Some(std::thread::spawn(move || {
                job.progress(Some(0.5), "half");
                job.finish(json!({"tiles": 4}));
            }));
            Ok(())
        })
        .unwrap();
        worker.unwrap().join().unwrap();
        let text = String::from_utf8(sink.0.lock().unwrap().clone()).unwrap();
        let lines: Vec<Value> = text.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
        assert_eq!(lines[0], json!({"type":"ready"}));
        assert_eq!(lines[1], json!({"type":"call","id":1,"op":"job.start","args":{"label":"Upscale"}}));
        assert_eq!(lines[2], json!({"type":"done","id":7,"code":0}));
        assert!(lines.contains(&json!({"type":"progress","job":3,"value":0.5,"note":"half"})));
        assert!(lines.contains(&json!({"type":"finish","job":3,"result":{"tiles":4}})));
        assert_eq!(lines.iter().filter(|l| l["type"] == "finish").count(), 1, "a finished job reports once");
    }
}

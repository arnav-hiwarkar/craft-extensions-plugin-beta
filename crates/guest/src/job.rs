//! Background jobs: long work (renders, analysis, AI models, big generated content) that runs on its
//! own thread without blocking the app or holding the document.
//!
//! 1. In a session event, [`start`] a job with a label and a JSON input (gather what it needs first:
//!    the input is all it sees).
//! 2. The app runs your [`job!`](crate::job) entry in a fresh sandbox. It may use
//!    [kernels](crate::kernel), report [`progress`] and return one JSON result. It has no document
//!    access.
//! 3. When it finishes, your session receives [`Event::Job`](crate::Event::Job) as a normal undoable
//!    event; apply the result there with ordinary operations. Check the document still matches.
//!
//! The person sees each job in the app's jobs list and can cancel it.

use crate::host::{CallError, call};
use serde_json::{Value, json};

/// Starts a background job; returns its id. At most four jobs per plugin run at once.
pub fn start(label: &str, input: Value) -> Result<u64, CallError> {
    start_with(label, input, true)
}

/// Like [`start`]; `cancellable = false` hides Cancel (turning the plugin off still stops it).
pub fn start_with(label: &str, input: Value, cancellable: bool) -> Result<u64, CallError> {
    let v = call("job.start", json!({ "label": label, "input": input, "cancellable": cancellable }))?;
    v.get("job").and_then(Value::as_u64).ok_or_else(|| CallError::Invalid("job.start returned no id".into()))
}

/// Reports progress from a job or an effect: `fraction` 0..=1 (or `None` while unknown) and a short note.
pub fn progress(fraction: Option<f32>, note: &str) {
    let _ = call("job.progress", json!({ "value": fraction, "note": note }));
}

/// Asks one of this plugin's jobs to stop. Returns whether it was running.
pub fn cancel(job: u64) -> Result<bool, CallError> {
    Ok(call("job.cancel", json!({ "job": job }))?.get("cancelled").and_then(Value::as_bool).unwrap_or(false))
}

/// How a job ended, as delivered in [`Event::Job`](crate::Event::Job).
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// It returned this result.
    Done(Value),
    /// It failed (trap, budget, error) with this message.
    Failed(String),
    /// The person or the plugin cancelled it.
    Cancelled,
}

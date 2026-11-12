//! Host calls: the plugin console (`craft.log`) and session ops (`craft.call` + `craft.take`).
//!
//! Only sessions may call ops. Transforms have no `call` import at all, so the host refuses them
//! before any of this runs.

use crate::abi;
use serde_json::Value;
use std::fmt;

/// Plugin console levels, as the host reads them (0 = debug … 3 = error).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    /// Verbose detail.
    Debug = 0,
    /// Normal progress.
    Info = 1,
    /// Something odd that did not stop the call.
    Warn = 2,
    /// The call failed.
    Error = 3,
}

/// Writes a line to the plugin console. The host rate-limits it and reads at most 4 KiB.
pub fn log(level: LogLevel, message: &str) {
    abi::log(level as i32, message.as_bytes());
}

/// Why a session op failed. Codes are the ABI v1 return codes (beta-design B4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallError {
    /// `-1`: the op or its arguments were refused as invalid.
    Invalid(String),
    /// `-2`: a permission the op needs is not granted. Show a hint and carry on; do not retry in a loop.
    PermissionNeeded(String),
    /// `-3`: the thing the op names does not exist.
    NotFound(String),
    /// `-4`: the target changed or is busy. Read it again before retrying.
    Stale(String),
    /// `-5`: the event was cancelled, or the document changed under it.
    Cancelled(String),
    /// `-6`: over the host's budget.
    Budget(String),
    /// Any other code, including `-7` (internal) and codes this SDK does not know.
    Failed {
        /// The ABI return code.
        code: i32,
        /// The host's message, or empty.
        message: String,
    },
}

impl CallError {
    /// The ABI return code.
    pub fn code(&self) -> i32 {
        match self {
            Self::Invalid(_) => -1,
            Self::PermissionNeeded(_) => -2,
            Self::NotFound(_) => -3,
            Self::Stale(_) => -4,
            Self::Cancelled(_) => -5,
            Self::Budget(_) => -6,
            Self::Failed { code, .. } => *code,
        }
    }

    /// The host's message, or a default for the code when the host sent none.
    pub fn message(&self) -> &str {
        match self {
            Self::Invalid(m) | Self::PermissionNeeded(m) | Self::NotFound(m) | Self::Stale(m) | Self::Cancelled(m) | Self::Budget(m) => m,
            Self::Failed { message, .. } => message,
        }
    }

    fn from_reply(code: i32, message: String) -> Self {
        let message = if message.is_empty() { default_message(code).to_owned() } else { message };
        match code {
            -1 => Self::Invalid(message),
            -2 => Self::PermissionNeeded(message),
            -3 => Self::NotFound(message),
            -4 => Self::Stale(message),
            -5 => Self::Cancelled(message),
            -6 => Self::Budget(message),
            _ => Self::Failed { code, message },
        }
    }
}

fn default_message(code: i32) -> &'static str {
    match code {
        -1 => "the host refused the request",
        -2 => "permission needed",
        -3 => "not found",
        -4 => "the target changed; try again",
        -5 => "cancelled",
        -6 => "over the host's limit",
        _ => "the host failed",
    }
}

impl fmt::Display for CallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for CallError {}

/// Replies larger than this are refused. The host keeps at most 8 MiB pending.
const MAX_REPLY_BYTES: usize = 8 * 1024 * 1024;
/// Room for an error reply: `{"error": code, "message": "…"}`.
const ERROR_REPLY_BYTES: usize = 4096;

/// Runs one app operation, such as `selection.get`, with the session's grants and transaction.
///
/// `args` is serialised as JSON. The reply is the op's JSON result. Failures come back as
/// [`CallError`]; [`CallError::PermissionNeeded`] means the person has not granted the permission.
pub fn call(op: &str, args: Value) -> Result<Value, CallError> {
    let args = serde_json::to_vec(&args).map_err(|e| CallError::Invalid(e.to_string()))?;
    let n = abi::call(op.as_bytes(), &args);
    if n < 0 {
        let code = i32::try_from(n).unwrap_or(-7);
        return Err(CallError::from_reply(code, take_error_message()));
    }
    let len = usize::try_from(n).ok().filter(|&l| l <= MAX_REPLY_BYTES).ok_or_else(|| CallError::Budget("the host reply is too large".to_owned()))?;
    let mut reply = vec![0u8; len];
    let got = abi::take(&mut reply);
    if usize::try_from(got).ok() != Some(len) {
        return Err(CallError::Failed { code: -7, message: "could not read the host reply".to_owned() });
    }
    serde_json::from_slice(&reply).map_err(|e| CallError::Failed { code: -7, message: format!("the host reply is not JSON: {e}") })
}

/// Reads the pending error JSON and returns its message ("" when there is none or it is unreadable).
fn take_error_message() -> String {
    let mut buf = vec![0u8; ERROR_REPLY_BYTES];
    let got = abi::take(&mut buf);
    let Some(len) = usize::try_from(got).ok() else { return String::new() };
    serde_json::from_slice::<Value>(&buf[..len]).ok().and_then(|v| v.get("message").and_then(Value::as_str).map(str::to_owned)).unwrap_or_default()
}

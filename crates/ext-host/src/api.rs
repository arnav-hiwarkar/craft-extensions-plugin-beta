//! What an app adapter implements, and what the host checks before the adapter runs anything.
//!
//! Invariants:
//! - Plugins reach an app only through [`AppApi::call`] with an op the adapter listed in its
//!   [`AppContract`]. Unknown ops are refused. Management, undo/history, settings and plugin-management
//!   ops are never listed, so they are unreachable by construction.
//! - The host computes the permissions an op needs from its real arguments (`OpSpec::requires`) and
//!   checks the grant on every call **before** the adapter sees the call, and again before commit.
//! - One session event (command, tool gesture, UI event) = one transaction = one undo step, opened with
//!   [`AppApi::begin`] and closed with [`AppApi::finish`]; `finish(false)` must roll back.

use craft_ext_core::contract::Provided;
use craft_ext_core::id::{AppId, ExtensionId};
use craft_ext_core::permission::Permission;
use craft_ext_core::version::Version;
use serde_json::Value;
use std::fmt::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// Error codes shared by the ABI, the native protocol, the CLI and MCP.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Code {
    Invalid = -1,
    Denied = -2,
    NotFound = -3,
    Stale = -4,
    Cancelled = -5,
    Budget = -6,
    Internal = -7,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpError {
    pub code: Code,
    pub message: String,
}

impl OpError {
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
    pub fn invalid(m: impl Into<String>) -> Self {
        Self::new(Code::Invalid, m)
    }
    pub fn denied(m: impl Into<String>) -> Self {
        Self::new(Code::Denied, m)
    }
    pub fn to_json(&self) -> Value {
        serde_json::json!({ "error": self.code as i32, "message": self.message })
    }
}

impl std::fmt::Display for OpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.message, self.code)
    }
}

impl std::error::Error for OpError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    /// Reads only.
    Query,
    /// Changes the document inside the event's transaction.
    Edit,
    /// Host UI (panels, windows, notifications).
    Ui,
    /// Side effect after commit (network, files).
    Effect,
}

/// One operation an app offers plugins.
pub struct OpSpec {
    pub id: &'static str,
    pub kind: OpKind,
    pub summary: &'static str,
    /// Example arguments (JSON text) for the generated docs.
    pub args: &'static str,
    /// Example result (JSON text) for the generated docs.
    pub result: &'static str,
    /// Permissions needed for these arguments. Must be total: return every permission the call needs.
    pub requires: fn(&Value) -> Vec<Permission>,
    /// Human description of the permission rule, for docs ("document.edit at the target's scope").
    pub permission_rule: &'static str,
}

/// An event the app can send to sessions.
pub struct EventSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub payload: &'static str,
    /// Whether receiving it needs `app.events`.
    pub needs_app_events: bool,
}

/// Where contributions appear in this app.
pub struct SlotSpec {
    pub kind: &'static str,
    pub summary: &'static str,
}

/// Everything an app offers plugins. Rendered to `docs/plugins/contracts/<app>.md` by
/// `craft-ext contract-docs` so authors can discover it.
pub struct AppContract {
    pub app: AppId,
    pub display_name: &'static str,
    pub provided: Provided,
    pub ops: Vec<OpSpec>,
    pub events: Vec<EventSpec>,
    pub slots: Vec<SlotSpec>,
    /// Notes on data conversion and limits (colour, units, time).
    pub notes: &'static str,
}

pub fn none(_: &Value) -> Vec<Permission> {
    Vec::new()
}

impl AppContract {
    pub fn op(&self, id: &str) -> Option<&OpSpec> {
        self.ops.iter().find(|o| o.id == id)
    }

    /// The reference page plugin makers read.
    pub fn to_markdown(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "# {} plugin contract\n", self.display_name);
        let _ = writeln!(s, "Generated from the `{}` adapter by `craft-ext contract-docs`. Do not edit by hand.\n", self.app);
        let _ = writeln!(s, "## Contracts provided\n\n| Contract | Version |\n|---|---|");
        for (n, l) in &self.provided.contracts {
            let _ = writeln!(s, "| `{n}` | {l} |");
        }
        let _ = writeln!(s, "\nA contribution declaring `contract = \"name@major.minor\"` works here when the major matches and this minor is equal or higher.\n");
        let _ = writeln!(s, "## Where contributions appear\n\n| Kind | Placement |\n|---|---|");
        for slot in &self.slots {
            let _ = writeln!(s, "| `{}` | {} |", slot.kind, slot.summary);
        }
        let _ = writeln!(s, "\n## Operations (session plugins: `craft::call(op, args)`)\n");
        for op in &self.ops {
            let _ = writeln!(s, "### `{}` ({:?})\n\n{}\n\nPermission: {}\n", op.id, op.kind, op.summary, op.permission_rule);
            let _ = writeln!(s, "```json\n// args\n{}\n// result\n{}\n```\n", op.args, op.result);
        }
        let _ = writeln!(s, "## Events (`craft_event`)\n\n| Event | Payload | Needs `app.events` | Meaning |\n|---|---|---|---|");
        for e in &self.events {
            let _ = writeln!(s, "| `{}` | `{}` | {} | {} |", e.name, e.payload, if e.needs_app_events { "yes" } else { "no" }, e.summary);
        }
        let _ = writeln!(s, "\n## Notes\n\n{}", self.notes);
        let _ = writeln!(s, "\n## Error codes\n\n`-1` invalid input, `-2` permission denied, `-3` not found, `-4` stale or busy, `-5` cancelled, `-6` over budget, `-7` internal.");
        s
    }
}

/// Who is calling and under what captured scope. Created by the host per event or render.
#[derive(Clone, Debug)]
pub struct Invocation {
    pub app: AppId,
    pub plugin: ExtensionId,
    pub version: Version,
    pub contribution: String,
    /// Grant generation captured at the start; a different current value means revoked.
    pub grant_generation: u64,
    pub cancel: Arc<AtomicBool>,
    pub deadline: Option<Instant>,
    /// Undo label: "<contribution label> · <plugin name>".
    pub label: String,
}

impl Invocation {
    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed) || self.deadline.is_some_and(|d| Instant::now() > d)
    }
}

/// The app side. Implemented once per app by its adapter crate.
pub trait AppApi: Send + Sync {
    fn contract(&self) -> &AppContract;
    /// Opens the event's transaction and captures scope (selection, document, revision).
    fn begin(&self, inv: &Invocation) -> Result<(), OpError>;
    /// Runs one op inside the open transaction. Permissions were already checked by the host.
    fn call(&self, inv: &Invocation, op: &str, args: Value) -> Result<Value, OpError>;
    /// Commits (one labelled undo step) or rolls back. Effects queued by ops run only after commit.
    fn finish(&self, inv: &Invocation, commit: bool);
}

/// Live grant state, implemented over `craft_ext_store::GrantView` by the adapter.
pub trait Grants: Send + Sync {
    /// Current grant generation of `plugin` in this app (changes on every grant or revoke).
    fn generation(&self, plugin: &ExtensionId) -> u64;
    fn allowed(&self, plugin: &ExtensionId, permission: &Permission) -> bool;
    /// Kill-listed or disabled in this app.
    fn stopped(&self, plugin: &ExtensionId, version: Version) -> bool;
    /// Native helper consent.
    fn full_access(&self, plugin: &ExtensionId) -> bool;
}

/// The one authorization function used for every host call (wasm and native).
pub fn authorize(contract: &AppContract, grants: &dyn Grants, inv: &Invocation, op: &str, args: &Value) -> Result<(), OpError> {
    if inv.cancelled() {
        return Err(OpError::new(Code::Cancelled, "cancelled"));
    }
    if grants.stopped(&inv.plugin, inv.version) {
        return Err(OpError::denied("this plugin has been turned off"));
    }
    if grants.generation(&inv.plugin) != inv.grant_generation {
        return Err(OpError::denied("permissions changed while this was running"));
    }
    let spec = contract.op(op).ok_or_else(|| OpError::new(Code::NotFound, format!("this app has no operation {op}")))?;
    for p in (spec.requires)(args) {
        if !grants.allowed(&inv.plugin, &p) {
            return Err(OpError::denied(format!("permission needed: {p}")));
        }
    }
    Ok(())
}

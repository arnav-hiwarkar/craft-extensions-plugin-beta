
use craft_ext_core::contract::Provided;
use craft_ext_core::id::{AppId, ExtensionId};
use craft_ext_core::permission::Permission;
use craft_ext_core::version::Version;
use serde_json::Value;
use std::fmt::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

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
    Query,
    Edit,
    Ui,
    Effect,
}

pub struct OpSpec {
    pub id: &'static str,
    pub kind: OpKind,
    pub summary: &'static str,
    pub args: &'static str,
    pub result: &'static str,
    pub requires: fn(&Value) -> Vec<Permission>,
    pub permission_rule: &'static str,
}

pub struct EventSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub payload: &'static str,
    pub needs_app_events: bool,
}

pub struct SlotSpec {
    pub kind: &'static str,
    pub summary: &'static str,
}

pub struct AppContract {
    pub app: AppId,
    pub display_name: &'static str,
    pub provided: Provided,
    pub ops: Vec<OpSpec>,
    pub events: Vec<EventSpec>,
    pub slots: Vec<SlotSpec>,
    pub notes: &'static str,
}

pub fn none(_: &Value) -> Vec<Permission> {
    Vec::new()
}

impl AppContract {
    pub fn op(&self, id: &str) -> Option<&OpSpec> {
        self.ops.iter().find(|o| o.id == id)
    }

    pub fn to_markdown(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "# {} plugin contract\n", self.display_name);
        let _ = writeln!(s, "Generated from the `{}` adapter by `<app>-cli plugin contract`. Do not edit by hand.\n", self.app);
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
            if let Some(schema) = crate::schema::shared(op.id) {
                let _ = writeln!(s, "{}", schema.to_markdown());
            }
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

#[derive(Clone, Debug)]
pub struct Invocation {
    pub app: AppId,
    pub plugin: ExtensionId,
    pub version: Version,
    pub contribution: String,
    pub grant_generation: u64,
    pub cancel: Arc<AtomicBool>,
    pub deadline: Option<Instant>,
    pub label: String,
    pub declared: Arc<[Permission]>,
}

impl Invocation {
    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed) || self.deadline.is_some_and(|d| Instant::now() > d)
    }
}

pub trait AppApi: Send + Sync {
    fn contract(&self) -> &AppContract;
    fn authorizes_on_owner_thread(&self) -> bool { false }
    fn begin(&self, inv: &Invocation) -> Result<(), OpError>;
    fn call(&self, inv: &Invocation, op: &str, args: Value) -> Result<Value, OpError>;
    fn finish(&self, inv: &Invocation, commit: bool);
}

pub fn contain<T>(what: &str, f: impl FnOnce() -> Result<T, OpError>) -> Result<T, OpError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|_| Err(OpError::new(Code::Internal, format!("the app failed while running {what}"))))
}

pub trait Grants: Send + Sync {
    fn generation(&self, plugin: &ExtensionId) -> u64;
    fn allowed(&self, plugin: &ExtensionId, permission: &Permission) -> bool;
    fn stopped(&self, plugin: &ExtensionId, version: Version) -> bool;
    fn full_access(&self, plugin: &ExtensionId) -> bool;
}

pub fn authorize(contract: &AppContract, grants: &dyn Grants, inv: &Invocation, op: &str, args: &Value) -> Result<(), OpError> {
    authorize_preflight(contract, grants, inv, op)?;
    let spec = contract.op(op).ok_or_else(|| OpError::new(Code::NotFound, format!("this app has no operation {op}")))?;
    for p in (spec.requires)(args) {
        if !inv.declared.iter().any(|d| d.covers(&p)) {
            return Err(OpError::denied(format!("{} does not declare {p}", inv.contribution)));
        }
        if !grants.allowed(&inv.plugin, &p) {
            return Err(OpError::denied(format!("permission needed: {p}")));
        }
    }
    Ok(())
}

pub fn check_args(contract: &AppContract, op: &str, args: &Value) -> Result<(), OpError> {
    match crate::schema::shared(op) {
        Some(schema) if contract.op(op).is_some() => schema.check(args).map_err(|why| OpError::invalid(format!("{op}: {why}"))),
        _ => Ok(()),
    }
}

pub(crate) fn authorize_preflight(contract: &AppContract, grants: &dyn Grants, inv: &Invocation, op: &str) -> Result<(), OpError> {
    if inv.cancelled() {
        return Err(OpError::new(Code::Cancelled, "cancelled"));
    }
    if grants.stopped(&inv.plugin, inv.version) {
        return Err(OpError::denied("this plugin has been turned off"));
    }
    if grants.generation(&inv.plugin) != inv.grant_generation {
        return Err(OpError::denied("permissions changed while this was running"));
    }
    let _spec = contract.op(op).ok_or_else(|| OpError::new(Code::NotFound, format!("this app has no operation {op}")))?;
    Ok(())
}

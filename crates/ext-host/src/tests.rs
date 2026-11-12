#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use crate::api::*;
use crate::wasm::*;
use craft_ext_core::contract::Provided;
use craft_ext_core::id::{AppId, ExtensionId};
use craft_ext_core::permission::Permission;
use craft_ext_core::version::Version;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Shared ABI scaffolding: bump allocator at 1 MiB, reset, abi.
const ABI: &str = r#"
  (memory (export "memory") 32)
  (global $top (mut i32) (i32.const 1048576))
  (func (export "craft_abi") (result i32) (i32.const 1))
  (func (export "craft_reset") (global.set $top (i32.const 1048576)))
  (func (export "craft_alloc") (param $n i32) (result i32) (local $p i32)
    (local.set $p (global.get $top))
    (global.set $top (i32.add (global.get $top) (i32.and (i32.add (local.get $n) (i32.const 7)) (i32.const -8))))
    (local.get $p))
"#;

fn module(body: &str, imports: &str) -> Plugin {
    let text = format!("(module {imports} {ABI} {body})");
    Plugin::load("org.example.test", &wat::parse_str(text).unwrap()).unwrap()
}

fn log() -> LogFn {
    Arc::new(|_, _, _| {})
}

#[test]
fn filter_inverts_and_reuses_instance() {
    // Inverts RGB of premultiplied pixels assuming alpha 1: c = 1 - c.
    let p = module(
        r#"(func (export "craft_filter") (param $ctx i32) (param $cl i32) (param $px i32) (param $w i32) (param $h i32) (result i32)
             (local $i i32) (local $end i32)
             (local.set $end (i32.add (local.get $px) (i32.mul (i32.mul (local.get $w) (local.get $h)) (i32.const 16))))
             (local.set $i (local.get $px))
             (block $done (loop $l
               (br_if $done (i32.ge_u (local.get $i) (local.get $end)))
               (f32.store (local.get $i) (f32.sub (f32.const 1) (f32.load (local.get $i))))
               (local.set $i (i32.add (local.get $i) (i32.const 16)))
               (br $l)))
             (i32.const 0))"#,
        "",
    );
    let mut f = Filter::new(&p, log()).unwrap();
    for _ in 0..3 {
        let mut px = vec![0.25, 0.5, 0.5, 1.0, 1.0, 0.0, 0.0, 1.0];
        f.run(&json!({"params":{}}), &mut px, 2, 1, &|| Ok(())).unwrap();
        assert_eq!(px[0], 0.75);
        assert_eq!(px[4], 0.0);
    }
}

#[test]
fn infinite_loop_is_stopped() {
    let p = module(
        r#"(func (export "craft_filter") (param i32 i32 i32 i32 i32) (result i32) (loop $l (br $l)) (i32.const 0))"#,
        "",
    );
    let mut f = Filter::new(&p, log()).unwrap();
    let mut px = vec![0.0; 4];
    // Cancelled at the first slice boundary…
    assert_eq!(f.run(&json!({}), &mut px, 1, 1, &|| Err(PluginError::Cancelled)).unwrap_err(), PluginError::Cancelled);
    // …and stopped by the fuel budget when nobody cancels.
    let mut f = Filter::new(&p, log()).unwrap();
    assert_eq!(f.run(&json!({}), &mut px, 1, 1, &|| Ok(())).unwrap_err(), PluginError::TooSlow);
}

#[test]
fn unknown_import_is_refused() {
    let text = format!("(module (import \"env\" \"system\" (func)) {ABI})");
    assert!(matches!(Plugin::load("x.y", &wat::parse_str(text).unwrap()), Err(PluginError::Load(_))));
}

struct FakeApp {
    contract: AppContract,
    committed: Mutex<Vec<bool>>,
}

fn needs_edit(_: &Value) -> Vec<Permission> {
    vec![Permission::parse("document.edit:selection").unwrap()]
}

impl AppApi for FakeApp {
    fn contract(&self) -> &AppContract {
        &self.contract
    }
    fn begin(&self, _: &Invocation) -> Result<(), OpError> {
        Ok(())
    }
    fn call(&self, _: &Invocation, op: &str, _: Value) -> Result<Value, OpError> {
        Ok(json!({"op": op}))
    }
    fn finish(&self, _: &Invocation, commit: bool) {
        self.committed.lock().unwrap().push(commit);
    }
}

struct FakeGrants {
    edit: AtomicBool,
    generation: AtomicU64,
}

impl Grants for FakeGrants {
    fn generation(&self, _: &ExtensionId) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }
    fn allowed(&self, _: &ExtensionId, p: &Permission) -> bool {
        self.edit.load(Ordering::Relaxed) && p.to_string().starts_with("document.edit")
    }
    fn stopped(&self, _: &ExtensionId, _: Version) -> bool {
        false
    }
    fn full_access(&self, _: &ExtensionId) -> bool {
        false
    }
}

fn session_fixture() -> (Session, Arc<FakeApp>, Arc<FakeGrants>) {
    // On every event: call "doc.fill" and return its code (>=0 → 0, else the code).
    let p = module(
        r#"(data (i32.const 16) "doc.fill")
           (func (export "craft_event") (param i32 i32) (result i32) (local $r i64)
             (local.set $r (call $call (i32.const 16) (i32.const 8) (i32.const 0) (i32.const 0)))
             (if (result i32) (i64.lt_s (local.get $r) (i64.const 0)) (then (i32.wrap_i64 (local.get $r))) (else (i32.const 0))))"#,
        r#"(import "craft" "call" (func $call (param i32 i32 i32 i32) (result i64)))"#,
    );
    let app = Arc::new(FakeApp {
        contract: AppContract {
            app: AppId::new("testcraft").unwrap(),
            display_name: "TestCraft",
            provided: Provided::new(&[("testcraft.session", 1, 0)]),
            ops: vec![OpSpec { id: "doc.fill", kind: OpKind::Edit, summary: "", args: "{}", result: "{}", requires: needs_edit, permission_rule: "" }],
            events: vec![],
            slots: vec![],
            notes: "",
        },
        committed: Mutex::new(vec![]),
    });
    let grants = Arc::new(FakeGrants { edit: AtomicBool::new(true), generation: AtomicU64::new(1) });
    let s = Session::new(&p, app.clone(), grants.clone(), log()).unwrap();
    (s, app, grants)
}

fn inv(generation: u64) -> Invocation {
    Invocation {
        app: AppId::new("testcraft").unwrap(),
        plugin: ExtensionId::new("org.example.test").unwrap(),
        version: Version::new(1, 0, 0),
        contribution: "fill".into(),
        grant_generation: generation,
        cancel: Arc::new(AtomicBool::new(false)),
        deadline: None,
        label: "Fill".into(),
        declared: vec![Permission::parse("document.edit:selection").unwrap()].into(),
    }
}

#[test]
fn session_permission_checked_on_every_call_and_revocation_rolls_back() {
    let (mut s, app, grants) = session_fixture();
    s.dispatch(inv(1), &json!({"event":"command"})).unwrap();
    // Revoke: generation moves on, the captured one is stale → denied, rolled back.
    grants.edit.store(false, Ordering::Relaxed);
    grants.generation.store(2, Ordering::Relaxed);
    assert!(s.dispatch(inv(1), &json!({"event":"command"})).is_err());
    // A fresh invocation under the new generation is denied by the permission itself.
    assert!(matches!(s.dispatch(inv(2), &json!({"event":"command"})), Err(PluginError::Failed(-2, _))));
    assert_eq!(*app.committed.lock().unwrap(), vec![true, false, false]);
}

#[test]
fn contract_docs_render() {
    let (_, app, _) = session_fixture();
    let md = app.contract.to_markdown();
    assert!(md.contains("`doc.fill`") && md.contains("testcraft.session"));
}

struct OwnerApp {
    app: Arc<FakeApp>,
    grants: Arc<FakeGrants>,
}
impl AppApi for OwnerApp {
    fn contract(&self) -> &AppContract { &self.app.contract }
    fn authorizes_on_owner_thread(&self) -> bool { true }
    fn begin(&self, inv: &Invocation) -> Result<(), OpError> { self.app.begin(inv) }
    fn call(&self, inv: &Invocation, op: &str, args: Value) -> Result<Value, OpError> {
        std::thread::scope(|scope| {
            std::thread::Builder::new().name("app-owner".into()).spawn_scoped(scope, || {
                authorize(self.contract(), self.grants.as_ref(), inv, op, &args)?;
                self.app.call(inv, op, args)
            }).unwrap().join().unwrap()
        })
    }
    fn finish(&self, inv: &Invocation, commit: bool) { self.app.finish(inv, commit) }
}
fn needs_edit_on_owner(args: &Value) -> Vec<Permission> {
    assert_eq!(std::thread::current().name(), Some("app-owner"));
    needs_edit(args)
}
#[test]
fn owner_thread_authorization_and_recoverable_call_errors() {
    let (_, mut app, grants) = session_fixture();
    Arc::get_mut(&mut app).unwrap().contract.ops[0].requires = needs_edit_on_owner;
    let api = Arc::new(OwnerApp { app: app.clone(), grants: grants.clone() });
    // The guest handles an op denial, unknown op or invalid args, then completes the event.
    let p = module(r#"(data (i32.const 16) "doc.fillunknown!")
       (func (export "craft_event") (param i32 i32) (result i32)
         (drop (call $call (i32.const 16) (i32.const 8) (i32.const 0) (i32.const 0)))
         (drop (call $call (i32.const 24) (i32.const 8) (i32.const 0) (i32.const 0)))
         (drop (call $call (i32.const 16) (i32.const 8) (i32.const 16) (i32.const 1)))
         (i32.const 0))"#,
       r#"(import "craft" "call" (func $call (param i32 i32 i32 i32) (result i64)))"#);
    let mut s = Session::new(&p, api, grants.clone(), log()).unwrap();
    s.dispatch(inv(1), &json!({})).unwrap();
    grants.edit.store(false, Ordering::Relaxed);
    s.dispatch(inv(1), &json!({})).unwrap();
    assert_eq!(*app.committed.lock().unwrap(), vec![true, true]);
    grants.generation.store(2, Ordering::Relaxed);
    assert!(s.dispatch(inv(1), &json!({})).is_err());
}

#[cfg(unix)]
struct NativeGrants(AtomicBool);
#[cfg(unix)]
impl Grants for NativeGrants {
    fn generation(&self, _: &ExtensionId) -> u64 { 1 }
    fn allowed(&self, _: &ExtensionId, _: &Permission) -> bool { false }
    fn stopped(&self, _: &ExtensionId, _: Version) -> bool { false }
    fn full_access(&self, _: &ExtensionId) -> bool { self.0.load(Ordering::Relaxed) }
}
#[cfg(unix)]
fn native_helper(script: &str, api: Arc<dyn AppApi>, grants: Arc<dyn Grants>) -> crate::native::Helper {
    use std::os::unix::fs::PermissionsExt;
    let path = std::env::temp_dir().join(format!("craft-helper-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::write(&path, format!("#!/bin/sh\nread hello\nprintf '%s\\n' '{{\"type\":\"ready\"}}'\n{script}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    let i = inv(1);
    let h = crate::native::Helper::start(&i.plugin, i.version, &path, &std::env::temp_dir(), api, grants, log()).unwrap();
    std::fs::remove_file(path).unwrap();
    h
}
#[test]
#[cfg(unix)]
fn silent_native_helper_detects_revocation_without_waiting_for_event_timeout() {
    let (_, app, _) = session_fixture();
    let grants = Arc::new(NativeGrants(AtomicBool::new(true)));
    let mut helper = native_helper("while read line; do :; done", app, grants.clone());
    let revoke = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        grants.0.store(false, Ordering::Relaxed);
    });
    let start = std::time::Instant::now();
    assert!(matches!(helper.dispatch(inv(1), &json!({})), Err(PluginError::Stopped(_))));
    assert!(start.elapsed() < std::time::Duration::from_secs(2));
    revoke.join().unwrap();
}

#[test]
#[cfg(unix)]
fn native_op_denial_is_recoverable() {
    struct Denying(Arc<FakeApp>);
    impl AppApi for Denying {
        fn contract(&self) -> &AppContract { self.0.contract() }
        fn begin(&self, i: &Invocation) -> Result<(), OpError> { self.0.begin(i) }
        fn call(&self, _: &Invocation, _: &str, _: Value) -> Result<Value, OpError> { Err(OpError::denied("operation denied")) }
        fn finish(&self, i: &Invocation, commit: bool) { self.0.finish(i, commit) }
    }
    let (_, app, _) = session_fixture();
    let grants = Arc::new(NativeGrants(AtomicBool::new(true)));
    let mut helper = native_helper(r#"read event
printf '%s\n' '{"type":"call","id":1,"op":"doc.fill","args":{}}'
read reply
printf '%s\n' '{"type":"done","id":1,"code":0}'
while read line; do :; done"#, Arc::new(Denying(app.clone())), grants);
    helper.dispatch(inv(1), &json!({})).unwrap();
    assert_eq!(*app.committed.lock().unwrap(), vec![true]);
}

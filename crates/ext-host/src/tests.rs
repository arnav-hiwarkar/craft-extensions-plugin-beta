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
    let p = module(r#"(func (export "craft_filter") (param i32 i32 i32 i32 i32) (result i32) (loop $l (br $l)) (i32.const 0))"#, "");
    let mut f = Filter::new(&p, log()).unwrap();
    let mut px = vec![0.0; 4];
    assert_eq!(f.run(&json!({}), &mut px, 1, 1, &|| Err(PluginError::Cancelled)).unwrap_err(), PluginError::Cancelled);
    let mut f = Filter::new(&p, log()).unwrap();
    assert_eq!(f.run(&json!({}), &mut px, 1, 1, &|| Ok(())).unwrap_err(), PluginError::TooSlow);
}

#[test]
fn host_calls_cost_fuel_so_a_log_loop_is_bounded() {
    let p = module(
        r#"(func (export "craft_filter") (param i32 i32 i32 i32 i32) (result i32)
             (loop $l (call $log (i32.const 0) (i32.const 0) (i32.const 64)) (br $l)) (i32.const 0))"#,
        r#"(import "craft" "log" (func $log (param i32 i32 i32)))"#,
    );
    let count = Arc::new(AtomicU64::new(0));
    let seen = count.clone();
    let sink: LogFn = Arc::new(move |_, _, _| {
        seen.fetch_add(1, Ordering::Relaxed);
    });
    let mut f = Filter::new(&p, sink).unwrap();
    let mut px = vec![0.0; 4];
    assert_eq!(f.run(&json!({}), &mut px, 1, 1, &|| Ok(())).unwrap_err(), PluginError::TooSlow);
    let calls = count.load(Ordering::Relaxed);
    assert!(calls > 0 && calls < 20_000, "{calls}");
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
            ops: vec![OpSpec {
                id: "doc.fill",
                kind: OpKind::Edit,
                summary: "",
                args: "{}",
                result: "{}",
                requires: needs_edit,
                permission_rule: "",
            }],
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
    grants.edit.store(false, Ordering::Relaxed);
    grants.generation.store(2, Ordering::Relaxed);
    assert!(s.dispatch(inv(1), &json!({"event":"command"})).is_err());
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
    fn contract(&self) -> &AppContract {
        &self.app.contract
    }
    fn authorizes_on_owner_thread(&self) -> bool {
        true
    }
    fn begin(&self, inv: &Invocation) -> Result<(), OpError> {
        self.app.begin(inv)
    }
    fn call(&self, inv: &Invocation, op: &str, args: Value) -> Result<Value, OpError> {
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .name("app-owner".into())
                .spawn_scoped(scope, || {
                    authorize(self.contract(), self.grants.as_ref(), inv, op, &args)?;
                    self.app.call(inv, op, args)
                })
                .unwrap()
                .join()
                .unwrap()
        })
    }
    fn finish(&self, inv: &Invocation, commit: bool) {
        self.app.finish(inv, commit)
    }
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
    let p = module(
        r#"(data (i32.const 16) "doc.fillunknown!")
       (func (export "craft_event") (param i32 i32) (result i32)
         (drop (call $call (i32.const 16) (i32.const 8) (i32.const 0) (i32.const 0)))
         (drop (call $call (i32.const 24) (i32.const 8) (i32.const 0) (i32.const 0)))
         (drop (call $call (i32.const 16) (i32.const 8) (i32.const 16) (i32.const 1)))
         (i32.const 0))"#,
        r#"(import "craft" "call" (func $call (param i32 i32 i32 i32) (result i64)))"#,
    );
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
    fn generation(&self, _: &ExtensionId) -> u64 {
        1
    }
    fn allowed(&self, _: &ExtensionId, _: &Permission) -> bool {
        false
    }
    fn stopped(&self, _: &ExtensionId, _: Version) -> bool {
        false
    }
    fn full_access(&self, _: &ExtensionId) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}
#[cfg(unix)]
fn native_helper(script: &str, api: Arc<dyn AppApi>, grants: Arc<dyn Grants>) -> crate::native::Helper {
    use std::os::unix::fs::PermissionsExt;
    let path = std::env::temp_dir().join(format!(
        "craft-helper-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    std::fs::write(&path, format!("#!/bin/sh\nread hello\nprintf '%s\\n' '{{\"type\":\"ready\"}}'\n{script}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    let i = inv(1);
    let h = crate::native::Helper::start(&i.plugin, i.version, &crate::native::HelperLaunch { exe: path.clone(), work: std::env::temp_dir().to_path_buf(), confinement: crate::native::Confinement::Full }, api, grants, log()).unwrap();
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
        fn contract(&self) -> &AppContract {
            self.0.contract()
        }
        fn begin(&self, i: &Invocation) -> Result<(), OpError> {
            self.0.begin(i)
        }
        fn call(&self, _: &Invocation, _: &str, _: Value) -> Result<Value, OpError> {
            Err(OpError::denied("operation denied"))
        }
        fn finish(&self, i: &Invocation, commit: bool) {
            self.0.finish(i, commit)
        }
    }
    let (_, app, _) = session_fixture();
    let grants = Arc::new(NativeGrants(AtomicBool::new(true)));
    let mut helper = native_helper(
        r#"read event
printf '%s\n' '{"type":"call","id":1,"op":"doc.fill","args":{}}'
read reply
printf '%s\n' '{"type":"done","id":1,"code":0}'
while read line; do :; done"#,
        Arc::new(Denying(app.clone())),
        grants,
    );
    helper.dispatch(inv(1), &json!({})).unwrap();
    assert_eq!(*app.committed.lock().unwrap(), vec![true]);
}

fn form_plugin(output: &str, name: &str) -> Plugin {
    let escaped = output.as_bytes().iter().map(|b| format!("\\{b:02x}")).collect::<String>();
    let params = if name == "craft_update" { "(param i32 i32)" } else { "" };
    module(
        &format!(
            r#"(data (i32.const 0) "{escaped}") (func (export "{name}") {params} (result i32) (call $result (i32.const 0) (i32.const {})))"#,
            output.len()
        ),
        r#"(import "craft" "result" (func $result (param i32 i32) (result i32)))"#,
    )
}
#[test]
fn form_describe_update_and_invalid_outputs() {
    let p = form_plugin(r#"{"schema":1,"key":"test","title":"Test","params":[]}"#, "craft_describe");
    assert_eq!(p.describe(log(), &|| Ok(())).unwrap().unwrap().key, "test");
    assert!(p.update(&Default::default(), 0, log(), &|| Ok(())).unwrap().is_none());
    let p = form_plugin(r#"{"schema":1,"base_rev":2,"rev":3,"ops":[]}"#, "craft_update");
    assert_eq!(p.update(&Default::default(), 2, log(), &|| Ok(())).unwrap().unwrap().rev, 3);
    assert!(p.update(&Default::default(), 1, log(), &|| Ok(())).is_err());
    for text in ["{}", r#"{"schema":9,"key":"test","title":"Test","params":[]}"#] {
        assert!(form_plugin(text, "craft_describe").describe(log(), &|| Ok(())).is_err());
    }
    let p = module(r#"(func (export "craft_describe") (result i32) (i32.const 0))"#, "");
    assert!(matches!(p.describe(log(), &|| Ok(())), Err(PluginError::InvalidOutput(_))));
    let p = module(r#"(func (export "craft_describe") (result i32) (loop $l (br $l)) (i32.const 0))"#, "");
    let calls = AtomicU64::new(0);
    assert_eq!(
        p.describe(log(), &|| { if calls.fetch_add(1, Ordering::SeqCst) > 0 { Err(PluginError::Cancelled) } else { Ok(()) } }).unwrap_err(),
        PluginError::Cancelled
    );
}

#[test]
fn form_result_refuses_invalid_ranges_and_oversize() {
    for (ptr, len) in [(i32::MAX, 16), (0, -1), (0, MAX_CALL_BYTES as i32 + 1)] {
        let p = module(
            &format!(r#"(func (export "craft_describe") (result i32) (call $result (i32.const {ptr}) (i32.const {len})))"#),
            r#"(import "craft" "result" (func $result (param i32 i32) (result i32)))"#,
        );
        assert!(matches!(p.describe(log(), &|| Ok(())), Err(PluginError::Failed(..))));
    }
}

#[test]
#[cfg(unix)]
fn native_packaged_helper_round_trip() {
    let (_, app, _) = session_fixture();
    let grants = Arc::new(NativeGrants(AtomicBool::new(false)));
    let i = inv(1);
    if let Some(path) = std::env::var_os("CRAFT_E2E_NATIVE_EXE") {
        let path = std::path::PathBuf::from(path);
        let work = path.parent().unwrap();
        assert!(crate::native::Helper::start(&i.plugin, i.version, &crate::native::HelperLaunch { exe: path.clone(), work: work.to_path_buf(), confinement: crate::native::Confinement::Full }, app.clone(), grants.clone(), log()).is_err());
        grants.0.store(true, Ordering::Relaxed);
        let mut helper = crate::native::Helper::start(&i.plugin, i.version, &crate::native::HelperLaunch { exe: path.clone(), work: work.to_path_buf(), confinement: crate::native::Confinement::Full }, app.clone(), grants.clone(), log()).unwrap();
        helper.dispatch(i, &json!({"type":"command","id":"smoke"})).unwrap();
        grants.0.store(false, Ordering::Relaxed);
        assert!(!helper.supervise());
    } else {
        grants.0.store(true, Ordering::Relaxed);
        let mut helper =
            native_helper("read event\nprintf '%s\\n' '{\"type\":\"done\",\"id\":1,\"code\":0}'\nwhile read line; do :; done", app.clone(), grants);
        helper.dispatch(i, &json!({})).unwrap();
    }
    assert_eq!(*app.committed.lock().unwrap(), vec![true]);
}

#[test]
fn rust_sdk_reference_types_are_supported() {
    let p = module(
        r#"(table 1 funcref)
        (func (export "craft_filter") (param i32 i32 i32 i32 i32) (result i32)
            (drop (table.get (i32.const 0))) (i32.const 0))"#,
        "",
    );
    let mut filter = Filter::new(&p, log()).unwrap();
    filter.run(&json!({}), &mut [0.0, 0.0, 0.0, 1.0], 1, 1, &|| Ok(())).unwrap();
}

#[test]
fn typed_ui_arguments_are_checked_before_the_adapter_sees_them() {
    let run = |args: &str| {
        let escaped = args.replace('"', "\\\"");
        let p = module(
            &format!(
                r#"(data (i32.const 16) "ui.notify") (data (i32.const 64) "{escaped}")
                   (func (export "craft_event") (param i32 i32) (result i32) (local $r i64)
                     (local.set $r (call $call (i32.const 16) (i32.const 9) (i32.const 64) (i32.const {len})))
                     (if (result i32) (i64.lt_s (local.get $r) (i64.const 0)) (then (i32.wrap_i64 (local.get $r))) (else (i32.const 0))))"#,
                len = args.len()
            ),
            r#"(import "craft" "call" (func $call (param i32 i32 i32 i32) (result i64)))"#,
        );
        let app = Arc::new(FakeApp {
            contract: AppContract {
                app: AppId::new("testcraft").unwrap(),
                display_name: "TestCraft",
                provided: Provided::new(&[("testcraft.session", 1, 0)]),
                ops: vec![OpSpec { id: "ui.notify", kind: OpKind::Ui, summary: "", args: "{}", result: "null", requires: none, permission_rule: "free" }],
                events: vec![],
                slots: vec![],
                notes: "",
            },
            committed: Mutex::new(vec![]),
        });
        let grants = Arc::new(FakeGrants { edit: AtomicBool::new(false), generation: AtomicU64::new(1) });
        let mut s = Session::new(&p, app, grants, log()).unwrap();
        s.dispatch(inv(1), &json!({"event":"command"}))
    };
    assert!(run(r#"{"message":"Done"}"#).is_ok());
    assert!(matches!(run(r#"{"message":"Done","level":"warn"}"#), Err(PluginError::Failed(-1, _))));
    assert!(matches!(run(r#"{"message":7}"#), Err(PluginError::Failed(-1, _))));
    assert!(matches!(run(r#"{}"#), Err(PluginError::Failed(-1, _))));
    let page = AppContract {
        app: AppId::new("testcraft").unwrap(),
        display_name: "TestCraft",
        provided: Provided::new(&[]),
        ops: vec![OpSpec { id: "ui.notify", kind: OpKind::Ui, summary: "", args: "{}", result: "null", requires: none, permission_rule: "free" }],
        events: vec![],
        slots: vec![],
        notes: "",
    }
    .to_markdown();
    assert!(page.contains("| `message` | text, ≤ 4000 characters | yes |"), "{page}");
}

#[test]
fn a_filter_runs_a_host_kernel_on_its_own_pixels_and_reports_progress() {
    let args = r#"{"image":{"ptr":65536,"width":2,"height":1},"matrix":[-1,0,0,0,1, 0,-1,0,0,1, 0,0,-1,0,1, 0,0,0,1,0]}"#;
    let p = module(
        &format!(
            r#"(data (i32.const 16) "kernel.color_matrix")
               (data (i32.const 48) "job.progress")
               (data (i32.const 64) "{{\"value\":0.5}}")
               (data (i32.const 1024) "{}")
               (func (export "craft_filter") (param $ctx i32) (param $cl i32) (param $px i32) (param $w i32) (param $h i32) (result i32) (local $r i64)
                 (memory.copy (i32.const 65536) (local.get $px) (i32.const 32))
                 (drop (call $call (i32.const 48) (i32.const 12) (i32.const 64) (i32.const 13)))
                 (local.set $r (call $call (i32.const 16) (i32.const 19) (i32.const 1024) (i32.const {})))
                 (if (i64.lt_s (local.get $r) (i64.const 0)) (then (return (i32.wrap_i64 (local.get $r)))))
                 (memory.copy (local.get $px) (i32.const 65536) (i32.const 32))
                 (i32.const 0))"#,
            args.replace('"', "\\\""),
            args.len()
        ),
        r#"(import "craft" "call" (func $call (param i32 i32 i32 i32) (result i64)))"#,
    );
    let mut f = Filter::new(&p, log()).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let s2 = seen.clone();
    f.set_progress(Arc::new(move |v, _| s2.lock().unwrap().push(v)));
    let mut px = vec![0.25, 0.5, 0.75, 1.0, 0.0, 0.0, 0.0, 0.0];
    f.run(&json!({"params":{}}), &mut px, 2, 1, &|| Ok(())).unwrap();
    assert_eq!(px, vec![0.75, 0.5, 0.25, 1.0, 0.0, 0.0, 0.0, 0.0]);
    assert_eq!(*seen.lock().unwrap(), vec![Some(0.5)]);
}

#[test]
fn a_session_starts_a_background_job_and_gets_its_result_as_an_event() {
    let p = module(
        r#"(data (i32.const 16) "job.start")
           (data (i32.const 32) "{\"label\":\"Upscale\",\"input\":{\"n\":3}}")
           (data (i32.const 128) "job.progress")
           (data (i32.const 144) "{\"value\":1}")
           (data (i32.const 160) "{\"done\":true}")
           (func (export "craft_event") (param i32 i32) (result i32) (local $r i64)
             (local.set $r (call $call (i32.const 16) (i32.const 9) (i32.const 32) (i32.const 35)))
             (if (result i32) (i64.lt_s (local.get $r) (i64.const 0)) (then (i32.wrap_i64 (local.get $r))) (else (i32.const 0))))
           (func (export "craft_job") (param i32 i32) (result i32)
             (drop (call $call (i32.const 128) (i32.const 12) (i32.const 144) (i32.const 11)))
             (drop (call $take (i32.const 4096) (i32.const 64)))
             (call $result (i32.const 160) (i32.const 13)))"#,
        r#"(import "craft" "call" (func $call (param i32 i32 i32 i32) (result i64)))
           (import "craft" "take" (func $take (param i32 i32) (result i32)))
           (import "craft" "result" (func $result (param i32 i32) (result i32)))"#,
    );
    let (_, app, grants) = session_fixture();
    let mut s = Session::new(&p, app, grants, log()).unwrap();
    let board = crate::jobs::JobBoard::new();
    s.dispatch(inv(1), &json!({"event":"command"})).unwrap_err();
    s.set_jobs(board.clone());
    s.dispatch(inv(1), &json!({"event":"command"})).unwrap();
    let started = std::time::Instant::now();
    let done = loop {
        let d = board.take_finished();
        if !d.is_empty() || started.elapsed().as_secs() > 20 {
            break d;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].label, "Upscale");
    assert_eq!(done[0].progress, Some(1.0));
    assert_eq!(done[0].event(), json!({"event":"job","job":1,"ok":true,"result":{"done":true}}));
}

struct PanickyApp(AppContract, Mutex<Vec<bool>>);

impl AppApi for PanickyApp {
    fn contract(&self) -> &AppContract {
        &self.0
    }
    fn begin(&self, _: &Invocation) -> Result<(), OpError> {
        Ok(())
    }
    fn call(&self, _: &Invocation, _: &str, _: Value) -> Result<Value, OpError> {
        let empty: Vec<u8> = Vec::new();
        Ok(json!(empty[std::hint::black_box(3)]))
    }
    fn finish(&self, _: &Invocation, commit: bool) {
        self.1.lock().unwrap().push(commit);
    }
}

#[test]
fn an_adapter_panic_fails_the_plugin_call_instead_of_the_app() {
    let (_, app, grants) = session_fixture();
    let contract = || AppContract {
        app: app.contract.app.clone(),
        display_name: "TestCraft",
        provided: Provided::new(&[("testcraft.session", 1, 0)]),
        ops: vec![OpSpec { id: "doc.fill", kind: OpKind::Edit, summary: "", args: "{}", result: "{}", requires: needs_edit, permission_rule: "" }],
        events: vec![],
        slots: vec![],
        notes: "",
    };
    let p = module(
        r#"(data (i32.const 16) "doc.fill")
           (func (export "craft_event") (param i32 i32) (result i32) (local $r i64)
             (local.set $r (call $call (i32.const 16) (i32.const 8) (i32.const 0) (i32.const 0)))
             (if (result i32) (i64.lt_s (local.get $r) (i64.const 0)) (then (i32.wrap_i64 (local.get $r))) (else (i32.const 0))))"#,
        r#"(import "craft" "call" (func $call (param i32 i32 i32 i32) (result i64)))"#,
    );
    let mut s = Session::new(&p, Arc::new(PanickyApp(contract(), Mutex::new(vec![]))), grants.clone(), log()).unwrap();
    assert!(matches!(s.dispatch(inv(1), &json!({"event":"command"})), Err(PluginError::Failed(-7, _))));
    assert!(s.dispatch(inv(1), &json!({"event":"command"})).is_err(), "the session survives and runs again");

    let ignores_errors = module(
        r#"(data (i32.const 16) "doc.fill")
           (func (export "craft_event") (param i32 i32) (result i32)
             (drop (call $call (i32.const 16) (i32.const 8) (i32.const 0) (i32.const 0)))
             (i32.const 0))"#,
        r#"(import "craft" "call" (func $call (param i32 i32 i32 i32) (result i64)))"#,
    );
    let app = Arc::new(PanickyApp(contract(), Mutex::new(vec![])));
    let mut s = Session::new(&ignores_errors, app.clone(), grants, log()).unwrap();
    assert!(matches!(s.dispatch(inv(1), &json!({"event":"command"})), Err(PluginError::Failed(-7, _))));
    assert_eq!(*app.1.lock().unwrap(), vec![false], "a half-applied edit is rolled back even when the plugin ignores the error");
}

#[test]
#[cfg(unix)]
fn a_native_helper_runs_a_background_job_after_its_event_ends() {
    let (_, app, _) = session_fixture();
    let grants = Arc::new(NativeGrants(AtomicBool::new(true)));
    let script = r#"read event
printf '%s\n' '{"type":"call","id":1,"op":"job.start","args":{"label":"Upscale"}}'
read reply
printf '%s\n' '{"type":"done","id":1,"code":0}'
printf '%s\n' '{"type":"progress","job":1,"value":0.5,"note":"half"}'
printf '%s\n' '{"type":"finish","job":1,"result":{"tiles":4}}'
while read line; do :; done"#;
    let mut helper = native_helper(script, app.clone(), grants);
    let board = crate::jobs::JobBoard::new();
    helper.set_jobs(board.clone());
    helper.dispatch(inv(1), &json!({"event":"command"})).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let done = loop {
        assert!(helper.supervise());
        let d = board.take_finished();
        if !d.is_empty() || std::time::Instant::now() > deadline {
            break d;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert_eq!(done.len(), 1);
    assert_eq!((done[0].progress, done[0].note.as_str()), (Some(0.5), "half"));
    assert_eq!(done[0].event(), json!({"event":"job","job":1,"ok":true,"result":{"tiles":4}}));
}

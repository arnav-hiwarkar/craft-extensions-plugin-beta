#![allow(clippy::unwrap_used, clippy::panic)]
use crate::{catalogue::verify_index, state::*, store::*, testutil::*, view::GrantView};
use craft_ext_core::{
    catalogue::{Revocation, Severity},
    id::{AppId, ExtensionId},
    permission::Permission,
    tier::Tier,
    version::{Version, VersionReq},
};
fn ids() -> (AppId, ExtensionId) {
    (AppId::new("photocraft").unwrap(), ExtensionId::new("org.example.test").unwrap())
}
fn install(s: &Store, v: &str, p: &str) -> Installed {
    s.install_bytes(&package(v, p, b"wasm"), Tier::Community, Origin::Catalogue { vendor: "example".into(), repo: "test".into() }).unwrap()
}
#[test]
fn consent_and_revocation() {
    let t = tempfile::tempdir().unwrap();
    let s = Store::at(t.path());
    let (a, id) = ids();
    let i = install(&s, "1.0.0", "\"clipboard.read\"");
    assert!(s.state().unwrap().apps.is_empty());
    s.set_active(&a, &id, i.version).unwrap();
    assert!(s.set_enabled(&a, &id, "run", true).is_err());
    let p = Permission::parse("clipboard.read").unwrap();
    s.grant(&a, &id, p.clone()).unwrap();
    s.set_enabled(&a, &id, "run", true).unwrap();
    let mut view = GrantView::open(t.path(), a.clone()).unwrap();
    assert!(view.allowed(&id, &p));
    let g = view.generation(&id);
    s.revoke(&a, &id, &p).unwrap();
    view.refresh_if_changed().unwrap();
    assert!(!view.allowed(&id, &p));
    assert!(view.generation(&id) > g);
    let next = install(&s, "2.0.0", "\"clipboard.write\"");
    s.set_active(&a, &id, next.version).unwrap();
    let st = s.state().unwrap();
    assert_eq!(st.apps[&a][&id].active, Some(i.version));
    assert_eq!(st.apps[&a][&id].pending, Some(next.version));
}
#[test]
fn immutable_and_kill_list() {
    let t = tempfile::tempdir().unwrap();
    let s = Store::at(t.path());
    let (a, id) = ids();
    let i = install(&s, "1.0.0", "");
    assert!(
        s.install_bytes(&package("1.0.0", "", b"different"), Tier::Community, Origin::Catalogue { vendor: "example".into(), repo: "test".into() })
            .is_err()
    );
    s.apply_revocations(vec![Revocation {
        id: Some(id.clone()),
        versions: None,
        sha256: None,
        vendor: None,
        severity: Severity::Disable,
        reason: "withdrawn".into(),
        date: "2026-11-12".into(),
    }])
    .unwrap();
    assert!(s.install_bytes(&package("2.0.0", "", b"wasm"), Tier::Local, Origin::Local { sha256: digest(&package("2.0.0", "", b"wasm")) }).is_err());
    assert!(GrantView::open(t.path(), a).unwrap().is_disabled(&id, i.version, &i.sha256));
}
#[test]
fn cleanup_retains_active_and_pin() {
    let t = tempfile::tempdir().unwrap();
    let s = Store::at(t.path());
    let (a, id) = ids();
    let i = install(&s, "1.0.0", "");
    install(&s, "2.0.0", "");
    install(&s, "3.0.0", "");
    s.set_active(&a, &id, i.version).unwrap();
    s.set_pin(&a, &id, Some(VersionReq::parse("=2.0.0").unwrap())).unwrap();
    s.remove_unused_versions().unwrap();
    let st = s.state().unwrap();
    assert_eq!(st.packages[&id].releases.len(), 2);
    assert!(!st.packages[&id].releases.contains_key(&Version::new(3, 0, 0)));
}
#[test]
fn signature_checks() {
    use ed25519_dalek::Signer;
    let key = ed25519_dalek::SigningKey::from_bytes(&[7; 32]);
    let public = hex::encode(key.verifying_key().as_bytes());
    let sig = hex::encode(key.sign(b"index").to_bytes());
    assert!(verify_index(b"index", &sig, &[&public]));
    assert!(!verify_index(b"changed", &sig, &[&public]));
    assert!(!verify_index(b"index", "bad", &[&public]));
}

#[test]
fn catalogue_publication_rejects_rollback_without_changing_state() {
    let t = tempfile::tempdir().unwrap();
    let s = Store::at(t.path());
    let (_, id) = ids();
    let rev = Revocation {
        id: Some(id),
        versions: None,
        sha256: None,
        vendor: None,
        severity: Severity::BlockNew,
        reason: "withdrawn".into(),
        date: "2026-11-12".into(),
    };
    s.publish_catalogue(10, 100, vec![rev]).unwrap();
    assert!(s.publish_catalogue(9, 200, vec![]).is_err());
    let state = s.state().unwrap();
    assert_eq!((state.catalogue_serial, state.last_refresh, state.generation, state.revocations.len()), (10, 100, 1, 1));
}

#[test]
fn local_versions_and_origin_consent() {
    let t = tempfile::tempdir().unwrap();
    let s = Store::at(t.path());
    let (app, id) = ids();
    for version in ["1.0.0", "2.0.0"] {
        let bytes = package(version, "", b"wasm");
        s.install_bytes(&bytes, Tier::Local, Origin::Local { sha256: digest(&bytes) }).unwrap();
    }
    let state = s.state().unwrap();
    assert_eq!(state.packages[&id].releases.len(), 2);
    for release in state.packages[&id].releases.values() {
        assert_eq!(release.origin, Some(Origin::Local { sha256: release.sha256.clone() }));
    }
    s.grant(&app, &id, Permission::parse("clipboard.read").unwrap()).unwrap();
    s.set_full_access(&app, &id, true).unwrap();
    install(&s, "3.0.0", "");
    let state = s.state().unwrap();
    assert!(state.apps[&app][&id].grants.is_empty());
    assert!(!state.apps[&app][&id].full_access);
}

fn native_package(version: &str) -> Vec<u8> {
    let text = format!(
        r#"schema=2
id="org.example.test"
name="Test"
version="{version}"
license="MIT"
source="https://github.com/example/test"
summary="Native test"
apps=["photocraft"]
[native.executables]
linux-x86_64="helper"
[[contribution]]
id="run"
kind="command"
contract="app.session@1.0"
label="Run"
"#
    );
    zip_of(&[("craft-plugin.toml", text.as_bytes()), ("helper", b"helper")])
}

#[test]
fn native_updates_need_fresh_acceptance_and_enable_checks_context() {
    use craft_ext_core::contract::Provided;
    let t = tempfile::tempdir().unwrap();
    let s = Store::at(t.path());
    let (app, id) = ids();
    for version in ["1.0.0", "2.0.0"] {
        let bytes = native_package(version);
        let i = s.install_bytes(&bytes, Tier::Local, Origin::Local { sha256: digest(&bytes) }).unwrap();
        s.set_full_access(&app, &id, true).unwrap();
        s.set_active(&app, &id, i.version).unwrap();
        assert_eq!(s.state().unwrap().apps[&app][&id].pending, Some(i.version));
        s.accept_pending(&app, &id).unwrap();
    }
    assert!(s.set_enabled_checked(&app, &id, "run", true, Some((&Provided::default(), "linux-x86_64"))).is_err());
    let provided = Provided::new(&[("app.session", 1, 0)]);
    assert!(s.set_enabled_checked(&app, &id, "run", true, Some((&provided, "windows-x86_64"))).is_err());
    s.set_enabled_checked(&app, &id, "run", true, Some((&provided, "linux-x86_64"))).unwrap();
}

#[test]
fn project_digest_mismatch_requires_other_version() {
    use craft_ext_core::{
        contract::Provided,
        project::{PluginRef, Source, Status},
    };
    let t = tempfile::tempdir().unwrap();
    let s = Store::at(t.path());
    let (app, id) = ids();
    let i = install(&s, "1.0.0", "");
    let m = crate::manager::Manager::new(s, Some(app), Provided::default(), "linux-x86_64".into());
    let refs = [PluginRef {
        id,
        version: i.version,
        sha256: Some("0".repeat(64)),
        source: Source::Local,
        name: "Test".into(),
        contributions: vec![],
        state_versions: None,
    }];
    assert!(matches!(m.check_project(&refs).unwrap()[0].1, Status::NeedsOtherVersion { .. }));
}

#[test]
fn committed_catalogue_uses_trusted_beta_key() {
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../craft-plugins/dist");
    if !dist.join("index.json").exists() {
        return;
    }
    let bytes = std::fs::read(dist.join("index.json")).unwrap();
    let sig = std::fs::read_to_string(dist.join("index.json.sig")).unwrap();
    assert!(verify_index(&bytes, &sig, craft_ext_core::catalogue::TRUSTED_KEYS));
}

#[test]
fn project_requires_each_contribution_and_saved_state_version() {
    use craft_ext_core::{
        contract::Provided,
        project::{PluginRef, Source, Status},
    };
    let t = tempfile::tempdir().unwrap();
    let s = Store::at(t.path());
    let (app, id) = ids();
    let i = install(&s, "1.0.0", "");
    s.set_active(&app, &id, i.version).unwrap();
    let m = crate::Manager::new(s, Some(app.clone()), Provided::new(&[("app.session", 1, 0)]), "linux-x86_64".into());
    let mut r = PluginRef {
        id: id.clone(),
        version: i.version,
        sha256: None,
        source: Source::Local,
        name: "Test".into(),
        contributions: vec!["run".into()],
        state_versions: None,
    };
    assert!(matches!(m.check_project(&[r.clone()]).unwrap()[0].1, Status::NeedsEnable { .. }));
    m.store.set_enabled(&app, &id, "run", true).unwrap();
    assert!(matches!(m.check_project(&[r.clone()]).unwrap()[0].1, Status::Ready { .. }));
    r.state_versions = Some(std::collections::BTreeMap::from([("run".into(), 1)]));
    assert!(matches!(m.check_project(&[r.clone()]).unwrap()[0].1, Status::NeedsOtherVersion { .. }));
    r.state_versions = None;
    r.contributions.push("missing".into());
    assert!(matches!(m.check_project(&[r]).unwrap()[0].1, Status::NeedsOtherVersion { .. }));
}

#[test]
fn reinstall_catalogue_bytes_as_local_relabels_and_revokes_consent() {
    let t = tempfile::tempdir().unwrap();
    let s = Store::at(t.path());
    let (app, id) = ids();
    let installed = install(&s, "1.0.0", "\"clipboard.read\"");
    s.set_active(&app, &id, installed.version).unwrap();
    s.grant(&app, &id, Permission::parse("clipboard.read").unwrap()).unwrap();
    s.set_enabled(&app, &id, "run", true).unwrap();
    let before = s.state().unwrap().apps[&app][&id].grant_generation;
    let bytes = package("1.0.0", "\"clipboard.read\"", b"wasm");
    s.install_bytes(&bytes, Tier::Local, Origin::Local { sha256: digest(&bytes) }).unwrap();
    let state = s.state().unwrap();
    assert_eq!(state.packages[&id].releases[&installed.version].tier, Tier::Local);
    let setup = &state.apps[&app][&id];
    assert!(setup.grants.is_empty() && setup.enabled.is_empty() && !setup.full_access);
    assert!(setup.grant_generation > before);
}

#[test]
fn local_digest_change_does_not_inherit_grants() {
    let t = tempfile::tempdir().unwrap();
    let s = Store::at(t.path());
    let (app, id) = ids();
    let first = package("1.0.0", "\"clipboard.read\"", b"wasm");
    s.install_bytes(&first, Tier::Local, Origin::Local { sha256: digest(&first) }).unwrap();
    s.grant(&app, &id, Permission::parse("clipboard.read").unwrap()).unwrap();
    s.set_full_access(&app, &id, true).unwrap();
    let next = package("1.1.0", "\"clipboard.read\"", b"wasm");
    s.install_bytes(&next, Tier::Local, Origin::Local { sha256: digest(&next) }).unwrap();
    let state = s.state().unwrap();
    assert!(state.apps[&app][&id].grants.is_empty());
    assert!(!state.apps[&app][&id].full_access);
}

#[test]
fn resource_reference_follows_active_release_and_dependents_guard_removal() {
    use craft_ext_core::{contract::Provided, manifest::MANIFEST_NAME};
    let t = tempfile::tempdir().unwrap();
    let store = Store::at(t.path());
    let (app, id) = ids();
    let reference = format!("plugin://{id}/look/luts/look.cube");
    for (version, data) in [("1.0.0", b"first".as_slice()), ("1.1.0", b"second".as_slice())] {
        let manifest = format!(
            r#"schema = 2
id = "{id}"
name = "Look"
version = "{version}"
license = "MIT"
source = "https://github.com/example/test"
summary = "Look fixture"
apps = ["photocraft"]
[[contribution]]
id = "look"
kind = "lut"
contract = "resource.lut@1.0"
label = "Look"
files = ["luts/look.cube"]
"#
        );
        let bytes = zip_of(&[(MANIFEST_NAME, manifest.as_bytes()), ("luts/look.cube", data)]);
        let installed = store.install_bytes(&bytes, Tier::Local, Origin::Local { sha256: digest(&bytes) }).unwrap();
        store.set_active(&app, &id, installed.version).unwrap();
        store.set_enabled(&app, &id, "look", true).unwrap();
    }
    let manager = crate::Manager::new(store, Some(app.clone()), Provided::new(&[("resource.lut", 1, 0)]), "linux-x86_64".into());
    assert_eq!(std::fs::read(manager.resolve_resource(&reference).unwrap()).unwrap(), b"second");
    manager.rollback(&id, Version::new(1, 0, 0)).unwrap();
    assert_eq!(std::fs::read(manager.resolve_resource(&reference).unwrap()).unwrap(), b"first");
    assert!(manager.resolve_resource(&format!("plugin://{id}/look/../craft-plugin.toml")).is_err());
    manager.disable(&id, "look").unwrap();
    assert!(manager.resolve_resource(&reference).unwrap_err().to_string().contains(&format!("needs plugin {id}")));
    let manifest = format!(
        r#"schema = 2
id = "org.example.dependent"
name = "Dependent"
version = "1.0.0"
license = "MIT"
source = "https://github.com/example/dependent"
summary = "Dependent fixture"
apps = ["photocraft"]
[requires]
"{id}" = "^1.0"
[[contribution]]
id = "look"
kind = "lut"
contract = "resource.lut@1.0"
label = "Look"
files = ["look.cube"]
"#
    );
    let bytes = zip_of(&[(MANIFEST_NAME, manifest.as_bytes()), ("look.cube", b"look")]);
    manager.store.install_bytes(&bytes, Tier::Local, Origin::Local { sha256: digest(&bytes) }).unwrap();
    assert!(manager.remove(&id).unwrap_err().to_string().contains("org.example.dependent"));
    manager.remove_force(&id, true).unwrap();
}

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

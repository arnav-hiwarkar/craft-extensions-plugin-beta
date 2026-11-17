#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
use craft_ext_core::project::{PluginRef, Status};
use craft_ext_store::{Store, state::Origin};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{Value, json};
use std::{
    io::Write as _,

    time::{Duration, Instant},
};

const APP: &str = "effectcraft";
const PLATFORM: &str = "linux-x86_64";

fn app() -> AppId {
    AppId::new(APP).unwrap()
}

fn id(text: &str) -> ExtensionId {
    ExtensionId::new(text).unwrap()
}

fn archive(id: &str, version: &str, permissions: &[&str], native: bool) -> Vec<u8> {
    let quoted = permissions.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ");
    let native = if native { "\n[native]\nmode = \"full\"\n[native.executables]\nmacos-aarch64 = \"bin/helper\"\n" } else { "" };
    let module = if native.is_empty() { "module = \"run.wasm\"\n" } else { "" };
    let why = if permissions.contains(&"clipboard.read") { "\n[why]\n\"clipboard.read\" = \"Pastes colours\"\n" } else { "" };
    let text = format!(
        "schema = 2\nid = \"{id}\"\nname = \"Test {version}\"\nversion = \"{version}\"\nlicense = \"MIT\"\nsource = \"https://github.com/alice/test\"\nsummary = \"Test package\"\napps = [\"effectcraft\"]\n{native}{why}\n[[contribution]]\nid = \"run\"\nkind = \"command\"\ncontract = \"effectcraft.session@1.0\"\nlabel = \"Run\"\n{module}permissions = [{quoted}]\n"
    );
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(vec![]));
    for (name, bytes) in [(craft_ext_core::manifest::MANIFEST_NAME, text.as_bytes()), ("run.wasm", b"wasm".as_slice())] {
        w.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
        w.write_all(bytes).unwrap();
    }
    w.finish().unwrap().into_inner()
}

fn row(id: &str, name: &str, vendor: &str, extra: Value) -> Value {
    let mut v = json!({"id":id,"name":name,"summary":"A plugin","vendor":vendor,"repo":"https://github.com/alice/test","kinds":["command"],
        "apps":["effectcraft"],"native":false,"license":"MIT","latest":"1.0.0","updated":"2026-11-01","file_sha256":"00","file_size":1});
    if let (Some(base), Some(extra)) = (v.as_object_mut(), extra.as_object()) {
        base.extend(extra.clone());
    }
    v
}

struct Fixture {
    dir: tempfile::TempDir,
    key: SigningKey,
}

impl Fixture {
    fn new() -> Self {
        Fixture { dir: tempfile::tempdir().unwrap(), key: SigningKey::from_bytes(&[9; 32]) }
    }

    fn store_root(&self) -> std::path::PathBuf {
        self.dir.path().join("store")
    }

    fn remote(&self) -> std::path::PathBuf {
        self.dir.path().join("remote")
    }

    fn publish(&self, packages: Vec<Value>, extra: Value) {
        let mut index = json!({"schema":1,"schema_minor":1,"serial":1,"issued":0,"expires":u64::MAX,
            "vendors":[{"login":"alice","github_id":1,"name":"Alice","tier":"community"},{"login":"artcraft","github_id":2,"name":"ArtCraft","tier":"official"}],
            "packages":packages,"revocations":[]});
        if let (Some(base), Some(extra)) = (index.as_object_mut(), extra.as_object()) {
            base.extend(extra.clone());
        }
        let bytes = serde_json::to_vec(&index).unwrap();
        let sig = hex::encode(self.key.sign(&bytes).to_bytes());
        craft_ext_store::catalogue::write_cache(&self.store_root().join("catalogue"), &bytes, sig.as_bytes()).unwrap();
    }

    fn package_file(&self, id: &str, releases: Value) -> (String, u64) {
        let bytes = serde_json::to_vec(&json!({"schema":1,"id":id,"vendor":"alice","releases":releases})).unwrap();
        std::fs::create_dir_all(self.remote().join("packages")).unwrap();
        std::fs::write(self.remote().join(format!("packages/{id}.json")), &bytes).unwrap();
        (craft_ext_store::store::digest(&bytes), bytes.len() as u64)
    }

    fn model(&self) -> Model {
        let provided = Provided::new(&[("effectcraft.session", 1, 1)]);
        let mut config = Config::app(self.store_root(), app(), provided, PLATFORM).with_test_key(hex::encode(self.key.verifying_key().as_bytes()));
        config.catalogue_base = format!("file://{}/", self.remote().display());
        let mut m = Model::new(config);
        settle(&mut m);
        m
    }

    fn model_for(&self, apps: &[&str]) -> Model {
        let targets = apps.iter().map(|a| AppTarget { app: AppId::new(a).unwrap(), provided: Provided::new(&[("effectcraft.session", 1, 1)]) }).collect();
        let mut config = Config::new(self.store_root(), targets, PLATFORM).with_test_key(hex::encode(self.key.verifying_key().as_bytes()));
        config.catalogue_base = format!("file://{}/", self.remote().display());
        let mut m = Model::new(config);
        settle(&mut m);
        m
    }

    fn file(&self, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = self.dir.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }
}

fn settle(m: &mut Model) -> Vec<Effect> {
    let start = Instant::now();
    let mut out = vec![];
    loop {
        out.extend(m.poll());
        if !m.busy() {
            return out;
        }
        assert!(start.elapsed() < Duration::from_secs(20), "the worker did not finish");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn wait_until(m: &mut Model, done: impl Fn(&Model) -> bool) {
    let start = Instant::now();
    while !done(m) {
        m.poll();
        assert!(start.elapsed() < Duration::from_secs(20), "the change was never seen");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn names(m: &Model) -> Vec<&str> {
    m.rows().iter().map(|r| r.name.as_str()).collect()
}

fn notices(effects: &[Effect]) -> Vec<(Word, String)> {
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Notice(n) => Some((n.word, n.subject.clone())),
            _ => None,
        })
        .collect()
}

fn searchable() -> Fixture {
    let f = Fixture::new();
    f.publish(
        vec![
            row("io.github.alice.haze", "Haze", "alice", json!({"description":"A soft glow over everything"})),
            row("io.github.alice.bloom", "Bloom", "alice", json!({"tags":["glow"],"downloads":{"count":5000,"date":"2026-11-16"}})),
            row("io.github.alice.glow", "Glow", "alice", json!({"updated":"2026-10-01"})),
            row("io.github.alice.soft", "Soft Light", "alice", json!({"summary":"Adds glow to highlights","categories":["color-looks"],"updated":"2026-11-10"})),
            row("io.github.alice.grain", "Grain", "alice", json!({"categories":["effects-filters"],"downloads":{"count":90,"date":"2026-11-16"}})),
            row("com.artcraft.titles", "Titles", "artcraft", json!({"categories":["text-titles"],"updated":"2026-11-12"})),
        ],
        json!({"featured":["com.artcraft.titles"]}),
    );
    f
}

#[test]
fn search_ranks_name_then_summary_then_tags_then_description() {
    let f = searchable();
    let mut m = f.model();
    assert_eq!(m.count(Tab::Discover), 6);
    m.apply(Action::SetQuery("glow".into()));
    assert_eq!(names(&m), vec!["Glow", "Soft Light", "Bloom", "Haze"]);
    m.apply(Action::SetQuery("HAZE".into()));
    assert_eq!(names(&m), vec!["Haze"]);
    m.apply(Action::SetQuery("zzz".into()));
    assert_eq!(names(&m), Vec::<&str>::new());
    assert_eq!(Tab::Discover.empty().english(), "No plugins match your search.");
}

#[test]
fn tier_and_downloads_boost_ties_but_never_beat_a_better_field() {
    let f = Fixture::new();
    f.publish(
        vec![
            row("io.github.alice.a", "Alpha", "alice", json!({"summary":"sharpen edges","downloads":{"count":1000000,"date":"2026-11-16"}})),
            row("com.artcraft.b", "Beta", "artcraft", json!({"summary":"sharpen edges"})),
            row("io.github.alice.c", "Gamma", "alice", json!({"summary":"sharpen edges"})),
            row("io.github.alice.sharpen", "Sharpen", "alice", json!({})),
        ],
        json!({}),
    );
    let mut m = f.model();
    m.apply(Action::SetQuery("sharpen".into()));
    assert_eq!(names(&m), vec!["Sharpen", "Alpha", "Beta", "Gamma"]);
}

#[test]
fn filters_sorts_and_badges() {
    let f = searchable();
    let mut m = f.model();
    m.apply(Action::SetFilters(Filters { app: Some(app()), category: Some(Category::ColorLooks), ..Filters::default() }));
    assert_eq!(names(&m), vec!["Soft Light"]);
    m.apply(Action::SetFilters(Filters { tier: Some(Tier::Official), ..Filters::default() }));
    assert_eq!(names(&m), vec!["Titles"]);
    assert_eq!(m.rows()[0].badges, vec![Badge::Tier(Tier::Official), Badge::Featured]);
    assert_eq!(m.rows()[0].badges.iter().map(|b| b.word().english()).collect::<Vec<_>>(), vec!["Official", "Featured"]);
    m.apply(Action::SetFilters(Filters::default()));
    m.apply(Action::SetSort(Sort::Popular));
    assert_eq!(names(&m)[..2], ["Bloom", "Grain"]);
    m.apply(Action::SetSort(Sort::RecentlyUpdated));
    assert_eq!(names(&m)[..3], ["Titles", "Soft Light", "Bloom"]);
    m.apply(Action::SetSort(Sort::Name));
    assert_eq!(names(&m), vec!["Bloom", "Glow", "Grain", "Haze", "Soft Light", "Titles"]);
    m.apply(Action::SetFilters(Filters { installed: true, ..Filters::default() }));
    assert_eq!(names(&m), Vec::<&str>::new());
    m.apply(Action::Select(Some(id("com.artcraft.titles"))));
    assert_eq!(m.detail().unwrap().badges, vec![Badge::Tier(Tier::Official), Badge::Featured]);
}

#[test]
fn kill_listed_plugins_leave_discover() {
    let f = searchable();
    f.publish(
        vec![row("io.github.alice.glow", "Glow", "alice", json!({})), row("io.github.alice.grain", "Grain", "alice", json!({}))],
        json!({"revocations":[{"id":"io.github.alice.glow","severity":"disable","reason":"malware","date":"2026-11-16"},
            {"id":"io.github.alice.grain","severity":"advisory","reason":"slow","date":"2026-11-16"}]}),
    );
    let m = f.model();
    assert_eq!(names(&m), vec!["Grain"]);
    assert_eq!(m.rows()[0].badges, vec![Badge::Tier(Tier::Community), Badge::Advisory]);
}

#[test]
fn install_from_file_opens_setup_and_confirm_turns_it_on() {
    let f = Fixture::new();
    let mut m = f.model();
    let path = f.file("test.craftplugin", &archive("io.github.alice.test", "1.0.0", &["clipboard.read"], false));
    m.apply(Action::InstallFile { path, app: None });
    let effects = settle(&mut m);
    let plugin = id("io.github.alice.test");
    assert!(effects.contains(&Effect::OpenSetup(plugin.clone())));
    let setup = m.setup(&plugin).unwrap();
    assert_eq!(setup.contributions, vec![SetupContribution { id: "run".into(), label: "Run".into(), on: true, unavailable: None }]);
    assert_eq!(
        setup.permissions,
        vec![SetupPermission {
            permission: Permission::parse("clipboard.read").unwrap(),
            sentence: "Read the clipboard when you use it".into(),
            why: Some("Pastes colours".into()),
            required: true,
            on: false
        }]
    );
    assert_eq!(setup.blocker(), Some(Word::SetupGrantRequired));
    assert_eq!(notices(&m.apply(Action::ConfirmSetup { id: plugin.clone() })), vec![(Word::SetupGrantRequired, "Test 1.0.0".to_string())]);
    m.apply(Action::SetupPermission { id: plugin.clone(), permission: Permission::parse("clipboard.read").unwrap(), on: true });
    assert_eq!(m.setup(&plugin).unwrap().blocker(), None);
    let confirm = m.apply(Action::ConfirmSetup { id: plugin.clone() });
    assert_eq!(confirm, vec![Effect::CloseSetup(plugin.clone()), Effect::Changed]);
    assert_eq!(m.flow(&plugin).map(|f| &f.step), Some(&Step::Working(Word::StateApplying)));
    let effects = settle(&mut m);
    assert_eq!(notices(&effects), vec![(Word::NoticeSetUp, "Test 1.0.0".to_string())]);
    assert_eq!(m.flow(&plugin), None);
    let setup = &m.snapshot().state.apps[&app()][&plugin];
    assert_eq!((setup.active, setup.enabled.iter().cloned().collect::<Vec<_>>()), (Some(Version::new(1, 0, 0)), vec!["run".to_string()]));
    m.apply(Action::SetTab(Tab::Installed));
    assert_eq!(m.rows()[0].primary, Primary::Open);
    assert_eq!(m.rows()[0].badges, vec![Badge::Tier(Tier::Local)]);
    assert_eq!(m.count(Tab::Installed), 1);
}

fn catalogue_origin() -> Origin {
    Origin::Catalogue { vendor: "alice".into(), repo: "https://github.com/alice/test".into() }
}

fn installed(f: &Fixture, version: &str, permissions: &[&str]) -> ExtensionId {
    let store = Store::at(f.store_root());
    let bytes = archive("io.github.alice.test", version, permissions, false);
    let installed = store.install_bytes(&bytes, Tier::Community, catalogue_origin()).unwrap();
    store.set_active(&app(), &installed.id, installed.version).unwrap();
    for p in permissions {
        store.grant(&app(), &installed.id, Permission::parse(p).unwrap()).unwrap();
    }
    store.set_enabled(&app(), &installed.id, "run", true).unwrap();
    installed.id
}

#[test]
fn writes_run_on_the_worker_and_the_view_follows_the_store() {
    let f = Fixture::new();
    let plugin = installed(&f, "1.0.0", &["clipboard.read"]);
    let mut m = f.model();
    m.apply(Action::Select(Some(plugin.clone())));
    let panel = &m.detail().unwrap().apps[0];
    assert_eq!(panel.grants.iter().map(|g| (g.sentence.as_str(), g.granted)).collect::<Vec<_>>(), vec![("Read the clipboard when you use it", true)]);
    m.apply(Action::Revoke { id: plugin.clone(), app: app(), permission: Permission::parse("clipboard.read").unwrap() });
    assert_eq!(notices(&settle(&mut m)), vec![(Word::NoticeChanged, "Test 1.0.0".to_string())]);
    assert!(!m.detail().unwrap().apps[0].grants[0].granted);
    Store::at(f.store_root()).quarantine(&app(), &plugin, Version::new(1, 0, 0), "panicked in render").unwrap();
    wait_until(&mut m, |m| m.count(Tab::Problems) == 1);
    assert_eq!(
        m.problems(),
        &[Problem {
            app: app(),
            id: plugin.clone(),
            version: Some(Version::new(1, 0, 0)),
            kind: ProblemKind::Quarantined { reason: "panicked in render".into() },
            fix: Some(Fix::TurnBackOn)
        }]
    );
    assert_eq!(m.problems()[0].kind.word().english(), "Turned off after a crash");
    m.apply(Action::TurnBackOn { id: plugin.clone(), app: app() });
    settle(&mut m);
    assert_eq!(m.problems(), &[]);
    m.apply(Action::Uninstall { id: plugin.clone() });
    assert_eq!(notices(&settle(&mut m)), vec![(Word::NoticeUninstalled, "Test 1.0.0".to_string())]);
    m.apply(Action::SetTab(Tab::Installed));
    assert_eq!(m.rows(), &[]);
}

#[test]
fn unclean_exit_and_pending_consent_are_problems() {
    let f = Fixture::new();
    let plugin = installed(&f, "1.0.0", &[]);
    let store = Store::at(f.store_root());
    let bytes = archive("io.github.alice.test", "2.0.0", &["clipboard.read"], false);
    store.install_bytes(&bytes, Tier::Community, catalogue_origin()).unwrap();
    store.set_active(&app(), &plugin, Version::new(2, 0, 0)).unwrap();
    let mut m = f.model();
    m.set_unclean_exit(app(), vec![plugin.clone()]);
    let kinds: Vec<(ProblemKind, Option<Fix>)> = m.problems().iter().map(|p| (p.kind.clone(), p.fix)).collect();
    assert_eq!(kinds, vec![(ProblemKind::ConsentPending { native: false }, Some(Fix::SetUp)), (ProblemKind::UncleanExit, Some(Fix::TurnOff))]);
    assert_eq!(m.problems()[1].action(), Some(Action::TurnOffAfterCrash { id: plugin.clone(), app: app() }));
    assert_eq!(m.apply(Action::SetUp { id: plugin.clone(), app: app() })[0], Effect::OpenSetup(plugin.clone()));
    assert_eq!(m.setup(&plugin).unwrap().version, Version::new(2, 0, 0));
    assert_eq!(m.setup(&plugin).unwrap().permissions.iter().map(|p| p.on).collect::<Vec<_>>(), vec![false]);
    assert_eq!(m.apply(Action::Cancel { id: plugin.clone() }), vec![Effect::CloseSetup(plugin.clone()), Effect::Changed]);
}

#[test]
fn turning_off_after_a_crash_quarantines_and_replaces_the_unclean_exit_row() {
    let f = Fixture::new();
    let plugin = installed(&f, "1.0.0", &[]);
    let mut m = f.model();
    m.set_unclean_exit(app(), vec![plugin.clone()]);
    m.apply(Action::TurnOffAfterCrash { id: plugin.clone(), app: app() });
    settle(&mut m);
    wait_until(&mut m, |m| m.problems().iter().all(|p| p.kind != ProblemKind::UncleanExit));
    assert_eq!(
        m.problems(),
        &[Problem {
            app: app(),
            id: plugin.clone(),
            version: Some(Version::new(1, 0, 0)),
            kind: ProblemKind::Quarantined { reason: UNCLEAN_EXIT.into() },
            fix: Some(Fix::TurnBackOn)
        }]
    );
}

#[test]
fn full_access_can_be_turned_off_from_the_detail() {
    let f = Fixture::new();
    let plugin = installed(&f, "1.0.0", &[]);
    Store::at(f.store_root()).set_full_access(&app(), &plugin, true).unwrap();
    let mut m = f.model();
    m.apply(Action::Select(Some(plugin.clone())));
    assert!(m.detail().unwrap().apps[0].full_access);
    m.apply(Action::RemoveFullAccess { id: plugin.clone(), app: app() });
    assert_eq!(notices(&settle(&mut m)), vec![(Word::NoticeChanged, "Test 1.0.0".to_string())]);
    wait_until(&mut m, |m| !m.detail().unwrap().apps[0].full_access);
    assert!(!Store::at(f.store_root()).state().unwrap().apps[&app()][&plugin].full_access);
}

#[test]
fn catalogue_pngs_arrive_as_rgba_pixels() {
    let mut bytes = vec![];
    let mut encoder = png::Encoder::new(&mut bytes, 2, 1);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().unwrap().write_image_data(&[255, 0, 0, 0, 0, 255]).unwrap();
    assert_eq!(worker::decode_png(&bytes), Ok(Pixels { width: 2, height: 1, rgba: vec![255, 0, 0, 255, 0, 0, 255, 255].into() }));
    assert!(worker::decode_png(b"not a png").is_err());
}

#[test]
fn rollback_to_an_installed_version_needs_no_new_consent() {
    let f = Fixture::new();
    installed(&f, "1.0.0", &[]);
    let plugin = installed(&f, "1.1.0", &[]);
    let mut m = f.model();
    m.apply(Action::Rollback { id: plugin.clone(), app: app(), version: Version::new(1, 0, 0) });
    assert_eq!(m.flow(&plugin).map(|f| f.kind), Some(FlowKind::Rollback));
    assert_eq!(notices(&settle(&mut m)), vec![(Word::NoticeRolledBack, "Test 1.0.0".to_string())]);
    assert_eq!(m.snapshot().state.apps[&app()][&plugin].active, Some(Version::new(1, 0, 0)));
    m.apply(Action::Select(Some(plugin.clone())));
    let versions: Vec<(Version, bool, Vec<AppId>)> = m.detail().unwrap().versions.iter().map(|v| (v.version, v.installed, v.active_in.clone())).collect();
    assert_eq!(versions, vec![(Version::new(1, 1, 0), true, vec![]), (Version::new(1, 0, 0), true, vec![app()])]);
}

#[test]
fn detail_shows_listing_versions_attestation_and_native_warning() {
    let f = Fixture::new();
    let manifest = craft_ext_core::manifest::Manifest::parse(
        std::str::from_utf8(&{
            let bytes = archive("io.github.alice.native", "1.0.0", &[], true);
            let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
            let mut text = String::new();
            std::io::Read::read_to_string(&mut zip.by_name(craft_ext_core::manifest::MANIFEST_NAME).unwrap(), &mut text).unwrap();
            text.into_bytes()
        })
        .unwrap()
        .as_bytes(),
    )
    .unwrap();
    let release = json!({"version":"1.0.0","tag":"v1.0.0","commit":"c".repeat(40),"published":"2026-11-02","assets":[],"manifest":manifest,
        "changelog":"First release","attestation":{"verified":true,"signer_workflow":".github/workflows/release.yml","source_commit":"c".repeat(40)}});
    let (sha, size) = f.package_file("io.github.alice.native", json!([release]));
    f.publish(
        vec![row("io.github.alice.native", "Native Helper", "alice", json!({"native":true,"file_sha256":sha,"file_size":size,"description":"Long text",
            "homepage":"https://alice.example","categories":["utilities"]}))],
        json!({}),
    );
    let mut m = f.model();
    let plugin = id("io.github.alice.native");
    m.apply(Action::Select(Some(plugin.clone())));
    assert!(m.detail().unwrap().versions_loading);
    settle(&mut m);
    let d = m.detail().unwrap();
    assert_eq!((d.name.as_str(), d.description.as_str(), d.vendor.as_str(), d.homepage.as_deref()), ("Native Helper", "Long text", "Alice", Some("https://alice.example")));
    assert_eq!(d.badges, vec![Badge::Tier(Tier::Community), Badge::NativeFull, Badge::BuiltFromSource]);
    assert_eq!(d.native, Some(NativeWarning::FullAccess));
    assert_eq!(d.native.unwrap().word().english(), "This plugin runs its own program with full access to your computer, outside the plugin sandbox.");
    assert_eq!(d.versions.len(), 1);
    assert_eq!((d.versions[0].changelog.as_str(), d.versions[0].usable_in.clone()), ("First release", vec![app()]));
    assert_eq!(d.versions[0].attestation.as_ref().map(|a| a.signer_workflow.as_str()), Some(".github/workflows/release.yml"));
    assert_eq!(d.report_url, "https://github.com/arnav-hiwarkar/craft-plugins/issues/new?template=report.yml&plugin=io.github.alice.native&version=1.0.0");
    assert_eq!(m.rows()[0].primary, Primary::Install);
}

#[test]
fn install_missing_reports_what_it_cannot_find() {
    let f = Fixture::new();
    f.publish(vec![], json!({}));
    let mut m = f.model();
    let missing: PluginRef = serde_json::from_value(json!({"id":"io.github.alice.gone","version":"1.0.0","source":"catalogue"})).unwrap();
    m.apply(Action::InstallMissing { app: app(), refs: vec![missing] });
    assert_eq!(m.flow(&id("io.github.alice.gone")).map(|f| &f.step), Some(&Step::Working(Word::StateChecking)));
    let effects = settle(&mut m);
    assert_eq!(notices(&effects), vec![(Word::NoticeUnavailable, "io.github.alice.gone".to_string())]);
    assert_eq!(m.flow(&id("io.github.alice.gone")), None);
}

#[test]
fn words_have_unique_stable_keys() {
    let keys: std::collections::BTreeSet<&str> = WORDS.iter().map(|w| w.1).collect();
    assert_eq!(keys.len(), WORDS.len());
    assert!(WORDS.iter().all(|(w, key, english)| w.key() == *key && w.english() == *english && key.starts_with("plugins.manager.")));
    assert_eq!(Word::TabDiscover.key(), "plugins.manager.tab.discover");
    assert_eq!(Word::ActionTurnBackOn.english(), "Turn back on");
    assert_eq!(Tab::ALL.map(|t| t.word().english()), ["Discover", "Installed", "Updates", "Problems"]);
    assert_eq!(Sort::ALL.map(|s| s.word().english()), ["Relevance", "Popular", "Recently updated", "Name"]);
}

#[test]
fn the_catalogue_is_read_once_per_change_not_per_poll() {
    let f = searchable();
    let mut m = f.model();
    let before = Arc::as_ptr(&m.snapshot);
    for _ in 0..50 {
        assert_eq!(m.poll(), vec![]);
    }
    assert_eq!(Arc::as_ptr(&m.snapshot), before);
    assert!(f.store_root().join("catalogue/index.signed").exists());
}

#[test]
fn badges_problems_warnings_and_buttons_each_carry_one_tone() {
    let badges = [Badge::Tier(Tier::Official), Badge::Tier(Tier::Verified), Badge::Tier(Tier::Community), Badge::Tier(Tier::Local), Badge::NativeFull, Badge::NativeRestricted, Badge::Featured];
    assert_eq!(badges.map(Badge::tone), [Tone::Accent, Tone::Good, Tone::Neutral, Tone::Warn, Tone::Bad, Tone::Warn, Tone::Info]);
    let primaries = [Primary::Install, Primary::Update(Version::new(2, 0, 0)), Primary::SetUp, Primary::Open, Primary::Working];
    assert_eq!(primaries.map(Primary::look), [Look::Primary, Look::Primary, Look::Primary, Look::Secondary, Look::Busy]);
    assert_eq!(primaries.map(|p| p.word().english()), ["Install", "Update", "Set up", "Open", "Working…"]);
    assert_eq!(ProblemKind::KillListed { reason: "malware".into() }.tone(), Tone::Bad);
    assert_eq!(ProblemKind::ConsentPending { native: true }.tone(), Tone::Info);
    assert_eq!([NativeWarning::FullAccess, NativeWarning::Restricted].map(NativeWarning::tone), [Tone::Bad, Tone::Warn]);
}

#[test]
fn a_problem_fix_is_the_action_that_fixes_it() {
    let problem = |fix| Problem { app: app(), id: id("io.github.alice.grain"), version: None, kind: ProblemKind::UncleanExit, fix };
    assert_eq!(problem(Some(Fix::TurnBackOn)).action(), Some(Action::TurnBackOn { id: id("io.github.alice.grain"), app: app() }));
    assert_eq!(problem(Some(Fix::Update(Version::new(2, 0, 0)))).action(), Some(Action::Update { id: id("io.github.alice.grain"), app: app() }));
    assert_eq!(problem(Some(Fix::Uninstall)).action(), Some(Action::Uninstall { id: id("io.github.alice.grain") }));
    assert_eq!(problem(None).action(), None);
}

#[test]
fn install_asks_which_app_only_when_several_managed_apps_support_the_plugin() {
    let f = Fixture::new();
    f.publish(
        vec![
            row("io.github.alice.both", "Both", "alice", json!({"apps":["effectcraft","photocraft"]})),
            row("io.github.alice.one", "One", "alice", json!({})),
        ],
        json!({}),
    );
    let photocraft = AppId::new("photocraft").unwrap();
    let launcher = f.model_for(&["effectcraft", "photocraft", "figcraft"]);
    let row_named = |m: &Model, name: &str| m.rows().iter().find(|r| r.name == name).unwrap().clone();
    assert_eq!(launcher.press(&row_named(&launcher, "Both")), Press::ChooseApp(vec![app(), photocraft.clone()]));
    let one = row_named(&launcher, "One");
    assert_eq!(launcher.press(&one), Press::Do(vec![Action::Install { id: one.id.clone(), app: Some(app()) }]));
    assert_eq!(launcher.name_of(&id("io.github.alice.both")), "Both");
    assert_eq!(launcher.name_of(&id("io.github.alice.gone")), "io.github.alice.gone");
    let single = f.model();
    let both = row_named(&single, "Both");
    assert_eq!(single.press(&both), Press::Do(vec![Action::Install { id: both.id.clone(), app: Some(app()) }]));
}

#[test]
fn categories_follow_the_managed_apps_and_have_words() {
    let f = Fixture::new();
    f.publish(vec![], json!({}));
    assert_eq!(f.model().categories().len(), 11);
    let photocraft = f.model_for(&["photocraft"]).categories();
    assert!(!photocraft.contains(&Category::MotionAnimation) && !photocraft.contains(&Category::ThreeDParticles));
    assert_eq!(photocraft.len(), 9);
    for row in craft_ext_core::listing::CATEGORIES {
        assert_eq!(category_word(row.category).english(), row.english);
    }
}

#[test]
fn a_project_status_has_one_word_one_tone_and_at_most_one_fix() {
    let grain = PluginRef {
        id: id("io.github.alice.grain"),
        version: Version::new(1, 2, 0),
        sha256: None,
        source: craft_ext_core::project::Source::Catalogue,
        name: "Grain".into(),
        contributions: vec!["run".into()],
        state_versions: None,
    };
    let statuses = [
        Status::Ready { version: Version::new(1, 2, 0) },
        Status::NeedsEnable { version: Version::new(1, 2, 0), reason: "turned off".into() },
        Status::NeedsOtherVersion { installed: Version::new(1, 0, 0), wanted: Version::new(1, 2, 0), reason: "older".into() },
        Status::Downloadable,
        Status::Unavailable { reason: "withdrawn".into() },
    ];
    assert_eq!(statuses.each_ref().map(|s| project_word(s).english()), ["Ready", "Installed, needs setting up", "Needs another version", "Available to download", "Not available"]);
    assert_eq!(statuses.each_ref().map(project_tone), [Tone::Good, Tone::Info, Tone::Warn, Tone::Info, Tone::Bad]);
    let install = Some((Word::ActionInstall, Action::InstallMissing { app: app(), refs: vec![grain.clone()] }));
    assert_eq!(
        statuses.each_ref().map(|s| project_fix(&app(), &grain, s)),
        [None, Some((Word::ActionSetUp, Action::SetUp { id: grain.id.clone(), app: app() })), install.clone(), install, None]
    );
}

#[test]
fn the_project_prompt_offers_install_all_for_listed_plugins_only() {
    let f = Fixture::new();
    f.publish(vec![], json!({}));
    let m = f.model();
    let r = |id: &str| -> PluginRef { serde_json::from_value(json!({"id":id,"version":"1.0.0","source":"catalogue","name":id})).unwrap() };
    let (a, b, gone, ready) = (r("io.github.alice.a"), r("io.github.alice.b"), r("io.github.alice.gone"), r("io.github.alice.ready"));
    let withdrawn = Status::Unavailable { reason: "release withdrawn".into() };
    let statuses = vec![
        (a.clone(), Status::Downloadable),
        (b.clone(), Status::Downloadable),
        (gone.clone(), withdrawn.clone()),
        (ready.clone(), Status::Ready { version: Version::new(1, 0, 0) }),
    ];
    let mut p = Prompt::new(m.config(), statuses);
    assert_eq!(p.install_all(&m), Some(Action::InstallMissing { app: app(), refs: vec![a.clone(), b.clone()] }));
    assert_eq!(p.fix(&m, &gone, &withdrawn), None);
    p.record([Notice { word: Word::NoticeInstalled, plugin: Some(a.id.clone()), subject: "A".into(), detail: String::new(), error: false }]);
    assert_eq!(p.result(&a.id).map(|n| n.word), Some(Word::NoticeInstalled));
    assert_eq!(p.install_all(&m), None);
    p.skip(b.id.clone());
    assert_eq!(p.finish(false), PromptOutcome { skipped: vec![b.id.clone()], open_without: false });
    assert_eq!(p.finish(true), PromptOutcome { skipped: vec![a.id, b.id, gone.id], open_without: true });
}

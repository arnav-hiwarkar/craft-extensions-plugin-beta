use crate::{Config, FlowKind, Word};
use craft_ext_core::{
    catalogue::{Index, PackageFile},
    id::{AppId, ExtensionId},
    listing::{Image, ImageRole},
    manifest::Manifest,
    permission::Permission,
    project::{PluginRef, Status},
    version::{Version, VersionReq},
};
use craft_ext_store::{Manager, Store, catalogue::Catalogue, state::StoreState};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Duration, SystemTime},
};

pub const WATCH_INTERVAL: Duration = Duration::from_millis(500);
pub const REFRESH_AFTER_SECONDS: u64 = 6 * 60 * 60;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateRow {
    pub app: AppId,
    pub id: ExtensionId,
    pub from: Version,
    pub to: Version,
    pub wider: bool,
}

#[derive(Debug, Default)]
pub struct Snapshot {
    pub state: StoreState,
    pub index: Option<Index>,
    pub index_error: Option<String>,
    pub manifests: BTreeMap<(ExtensionId, Version), Manifest>,
    pub updates: Vec<UpdateRow>,
}

#[derive(Clone, Debug)]
pub struct SetupChoice {
    pub id: ExtensionId,
    pub app: AppId,
    pub version: Version,
    pub contributions: Vec<String>,
    pub grants: Vec<Permission>,
    pub full_access: bool,
}

#[derive(Clone, Debug)]
pub enum Write {
    Enable(AppId, ExtensionId, String, bool),
    Grant(AppId, ExtensionId, Permission, bool),
    Pin(AppId, ExtensionId, Option<VersionReq>),
    TurnBackOn(AppId, ExtensionId),
    Uninstall(ExtensionId),
}

#[derive(Debug)]
pub enum Job {
    RefreshIfStale,
    Refresh,
    Package(ExtensionId),
    Media(String, String, Image, ImageRole),
    Stage { id: ExtensionId, app: AppId, kind: FlowKind, req: Option<VersionReq> },
    StageFile { path: PathBuf, app: AppId },
    Apply(SetupChoice, FlowKind),
    Write(Write),
    Missing { app: AppId, refs: Vec<PluginRef> },
}

#[derive(Debug)]
pub enum Event {
    Loaded(Arc<Snapshot>),
    Refreshed(Result<(), String>),
    Package(ExtensionId, Result<Arc<PackageFile>, String>),
    Media(String, Result<Arc<[u8]>, String>),
    Staged { id: ExtensionId, app: AppId, kind: FlowKind, manifest: Box<Manifest>, needs_setup: bool, dependencies: Vec<(ExtensionId, Version)> },
    FileFailed(PathBuf, String),
    Applied { id: ExtensionId, kind: FlowKind },
    Wrote(Word, ExtensionId, Result<(), String>),
    Failed(ExtensionId, String),
    Settled(ExtensionId, Word, String),
    Idle,
}

pub struct Worker {
    pub jobs: mpsc::Sender<Job>,
    pub events: mpsc::Receiver<Event>,
}

impl Worker {
    pub fn spawn(config: Arc<Config>) -> Worker {
        let (jobs, job_rx) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        std::thread::Builder::new()
            .name("craft-plugin-manager".into())
            .spawn(move || Runner::new(config, event_tx).run(&job_rx))
            .map_or_else(|e| Worker::failed(e.to_string()), |_| Worker { jobs, events })
    }

    fn failed(reason: String) -> Worker {
        let (jobs, _) = mpsc::channel();
        let (tx, events) = mpsc::channel();
        let snapshot = Snapshot { index_error: Some(format!("the plugin manager could not start: {reason}")), ..Snapshot::default() };
        let _ = tx.send(Event::Loaded(Arc::new(snapshot)));
        Worker { jobs, events }
    }
}

type Stamp = Option<(SystemTime, u64)>;

struct Runner {
    config: Arc<Config>,
    events: mpsc::Sender<Event>,
    store: Store,
    stamps: (Stamp, Stamp),
    updates_key: Option<String>,
    updates: Vec<UpdateRow>,
}

fn stamp(path: PathBuf) -> Stamp {
    std::fs::metadata(path).ok().and_then(|m| Some((m.modified().ok()?, m.len())))
}

impl Runner {
    fn new(config: Arc<Config>, events: mpsc::Sender<Event>) -> Self {
        let store = Store::at(&config.store_root);
        Runner { config, events, store, stamps: (None, None), updates_key: None, updates: vec![] }
    }

    fn manager(&self, app: Option<&AppId>) -> Manager {
        let target = app.and_then(|a| self.config.apps.iter().find(|t| &t.app == a));
        let provided = target.map(|t| t.provided.clone()).unwrap_or_default();
        let mut m = Manager::new(self.store.clone(), target.map(|t| t.app.clone()), provided, self.config.platform.clone());
        m.catalogue_base = self.config.catalogue_base.clone();
        m
    }

    fn send(&self, e: Event) {
        let _ = self.events.send(e);
    }

    fn run(mut self, jobs: &mpsc::Receiver<Job>) {
        self.reload(true);
        loop {
            match jobs.recv_timeout(WATCH_INTERVAL) {
                Ok(job) => {
                    let force = self.job(job);
                    self.reload(force);
                    self.send(Event::Idle);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => self.reload(false),
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    fn job(&mut self, job: Job) -> bool {
        match job {
            Job::RefreshIfStale => {
                let stale = self.store.state().is_ok_and(|s| craft_ext_store::store::now().saturating_sub(s.last_refresh) > REFRESH_AFTER_SECONDS);
                stale && self.refresh()
            }
            Job::Refresh => self.refresh(),
            Job::Package(id) => {
                let result = self
                    .index()
                    .and_then(|i| i.package(&id).cloned().ok_or_else(|| "this plugin is not listed".to_string()))
                    .and_then(|row| Catalogue::package_file(&self.config.catalogue_base, &row).map_err(|e| e.to_string()));
                self.send(Event::Package(id, result.map(Arc::new)));
                false
            }
            Job::Media(url, repo, image, role) => {
                let result = Catalogue::media(&self.config.catalogue_base, &repo, &image, role).map(Arc::from).map_err(|e| e.to_string());
                self.send(Event::Media(url, result));
                false
            }
            Job::Stage { id, app, kind, req } => {
                let staged = self.stage(&id, &app, kind, req);
                self.staged(id, app, kind, staged);
                true
            }
            Job::StageFile { path, app } => {
                match self.manager(Some(&app)).install_file(&path) {
                    Ok(r) => self.staged(r.0.id.clone(), app, FlowKind::Install, Ok((r.0.manifest, r.3))),
                    Err(e) => self.send(Event::FileFailed(path, e.to_string())),
                }
                true
            }
            Job::Apply(choice, kind) => {
                let id = choice.id.clone();
                match self.apply(&choice) {
                    Ok(()) => self.send(Event::Applied { id, kind }),
                    Err(e) => self.send(Event::Failed(id, e)),
                }
                true
            }
            Job::Write(w) => {
                let (word, id, result) = self.write(w);
                self.send(Event::Wrote(word, id, result));
                true
            }
            Job::Missing { app, refs } => {
                self.missing(app, refs);
                true
            }
        }
    }

    fn refresh(&mut self) -> bool {
        let result = self.manager(None).refresh(craft_ext_store::store::now()).map_err(|e| e.to_string());
        self.updates_key = None;
        self.send(Event::Refreshed(result));
        true
    }

    fn stage(&self, id: &ExtensionId, app: &AppId, kind: FlowKind, req: Option<VersionReq>) -> Result<(Manifest, Vec<(ExtensionId, Version)>), String> {
        let m = self.manager(Some(app));
        let staged = match (kind, req) {
            (FlowKind::Update, _) => m.update(id).map(|r| (r.0.manifest, r.3)),
            (FlowKind::Rollback, Some(VersionReq::Exact(v))) => {
                m.rollback(id, v).and_then(|_| self.store.state()).and_then(|s| self.store.manifest(&s, id, v)).map(|man| (man, vec![]))
            }
            (FlowKind::Rollback, _) => return Err("choose one version to switch to".into()),
            _ => m.install(id, req).map(|r| (r.0.manifest, r.3)),
        };
        staged.map_err(|e| e.to_string())
    }

    fn staged(&mut self, id: ExtensionId, app: AppId, kind: FlowKind, staged: Result<(Manifest, Vec<(ExtensionId, Version)>), String>) {
        self.updates_key = None;
        match staged {
            Err(e) => self.send(Event::Failed(id, e)),
            Ok((manifest, dependencies)) => {
                let activated = || self.store.state().ok().and_then(|s| s.apps.get(&app).and_then(|a| a.get(&id)).map(|a| a.pending.is_none() && !a.enabled.is_empty()));
                let needs_setup = matches!(kind, FlowKind::Install | FlowKind::InstallMissing) || !activated().unwrap_or(false);
                self.send(Event::Staged { id, app, kind, manifest: Box::new(manifest), needs_setup, dependencies });
            }
        }
    }

    fn apply(&self, c: &SetupChoice) -> Result<(), String> {
        let m = self.manager(Some(&c.app));
        let run = || -> craft_ext_store::store::Result<()> {
            let state = self.store.state()?;
            if let Some(a) = state.apps.get(&c.app).and_then(|a| a.get(&c.id)) {
                for e in &a.enabled {
                    m.disable(&c.id, e)?;
                }
                for p in &a.grants {
                    m.revoke(&c.id, p)?;
                }
            }
            for p in &c.grants {
                m.grant(&c.id, p.clone())?;
            }
            self.store.set_active(&c.app, &c.id, c.version)?;
            self.store.set_full_access(&c.app, &c.id, c.full_access)?;
            if self.store.state()?.apps.get(&c.app).and_then(|a| a.get(&c.id)).is_some_and(|a| a.pending.is_some()) {
                self.store.accept_pending(&c.app, &c.id)?;
            }
            for e in &c.contributions {
                m.enable(&c.id, e)?;
            }
            Ok(())
        };
        run().map_err(|e| e.to_string())
    }

    fn write(&mut self, w: Write) -> (Word, ExtensionId, Result<(), String>) {
        let s = &self.store;
        let (word, id, r) = match w {
            Write::Enable(app, id, c, on) => {
                let m = self.manager(Some(&app));
                let r = if on { m.enable(&id, &c) } else { m.disable(&id, &c) };
                (Word::NoticeChanged, id, r)
            }
            Write::Grant(app, id, p, on) => {
                let r = if on { s.grant(&app, &id, p) } else { s.revoke(&app, &id, &p) };
                (Word::NoticeChanged, id, r)
            }
            Write::Pin(app, id, req) => {
                let r = s.set_pin(&app, &id, req);
                (Word::NoticeChanged, id, r)
            }
            Write::TurnBackOn(app, id) => {
                let r = s.clear_quarantine(&app, &id);
                (Word::NoticeChanged, id, r)
            }
            Write::Uninstall(id) => {
                let r = self.manager(None).remove(&id);
                self.updates_key = None;
                (Word::NoticeUninstalled, id, r)
            }
        };
        (word, id, r.map_err(|e| e.to_string()))
    }

    fn missing(&mut self, app: AppId, refs: Vec<PluginRef>) {
        let found = match self.manager(Some(&app)).check_project(&refs) {
            Ok(f) => f,
            Err(e) => {
                for r in refs {
                    self.send(Event::Failed(r.id, e.to_string()));
                }
                return;
            }
        };
        for (r, status) in found {
            let wanted = VersionReq::Exact(r.version);
            match status {
                Status::Downloadable | Status::NeedsOtherVersion { .. } if !self.has_release(&r.id, r.version) => {
                    let staged = self.manager(Some(&app)).install(&r.id, Some(wanted)).map(|x| (x.0.manifest, x.3)).map_err(|e| e.to_string());
                    self.staged(r.id, app.clone(), FlowKind::InstallMissing, staged);
                }
                Status::Unavailable { reason } => self.send(Event::Settled(r.id, Word::NoticeUnavailable, reason)),
                Status::NeedsOtherVersion { reason, .. } => self.send(Event::Settled(r.id, Word::NoticeUnavailable, reason)),
                Status::Ready { .. } | Status::NeedsEnable { .. } | Status::Downloadable => {
                    self.send(Event::Settled(r.id, Word::NoticeAlreadyReady, String::new()))
                }
            }
        }
    }

    fn has_release(&self, id: &ExtensionId, v: Version) -> bool {
        self.store.state().is_ok_and(|s| s.packages.get(id).is_some_and(|p| p.releases.contains_key(&v)))
    }

    fn index(&self) -> Result<Index, String> {
        let keys = self.config.keys();
        Catalogue::cached_with_keys(&self.store.root().join("catalogue"), &keys.iter().map(String::as_str).collect::<Vec<_>>()).map_err(|e| e.to_string())
    }

    fn reload(&mut self, force: bool) {
        let root = self.store.root().to_path_buf();
        let stamps = (stamp(root.join(craft_ext_store::state::STATE_FILE)), stamp(root.join("catalogue").join(craft_ext_store::catalogue::SNAPSHOT)));
        if !force && stamps == self.stamps {
            return;
        }
        self.stamps = stamps;
        let state = match self.store.state() {
            Ok(s) => s,
            Err(e) => {
                let snapshot = Snapshot { index_error: Some(e.to_string()), ..Snapshot::default() };
                return self.send(Event::Loaded(Arc::new(snapshot)));
            }
        };
        let (index, index_error) = match self.index() {
            Ok(i) => (Some(i), None),
            Err(e) => (None, Some(e)),
        };
        let mut manifests = BTreeMap::new();
        for (id, p) in &state.packages {
            for v in p.releases.keys() {
                if let Ok(m) = self.store.manifest(&state, id, *v) {
                    manifests.insert((id.clone(), *v), m);
                }
            }
        }
        self.check_updates(&state, index.as_ref());
        let snapshot = Snapshot { state, index, index_error, manifests, updates: self.updates.clone() };
        self.send(Event::Loaded(Arc::new(snapshot)));
    }

    fn check_updates(&mut self, state: &StoreState, index: Option<&Index>) {
        let Some(index) = index else {
            self.updates.clear();
            return;
        };
        let active: Vec<_> = state.apps.iter().flat_map(|(app, s)| s.iter().map(move |(id, a)| (app, id, a.active, a.pin))).collect();
        let key = format!("{}:{:?}", index.serial, active);
        if self.updates_key.as_ref() == Some(&key) {
            return;
        }
        self.updates_key = Some(key);
        self.updates.clear();
        for target in &self.config.apps {
            if let Ok((rows, _)) = self.manager(Some(&target.app)).update_check() {
                self.updates.extend(rows.into_iter().map(|(id, from, to, wider)| UpdateRow { app: target.app.clone(), id, from, to, wider }));
            }
        }
    }
}

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]
mod detail;
mod flow;
mod problems;
mod prompt;
mod rank;
mod view;
mod words;
mod worker;

pub use detail::{AppPanel, ContributionRow, Detail, GrantRow, PermissionRow, VersionRow};
pub use flow::{Flow, FlowKind, NativeWarning, Setup, SetupContribution, SetupPermission, Step};
pub use problems::{Fix, Problem, ProblemKind};
pub use prompt::{Prompt, PromptOutcome};
pub use view::{Look, Press, Tone, category_word, project_fix, project_tone, project_word};
pub use rank::{MAX_DOWNLOAD_BOOST, SCORE_DESCRIPTION, SCORE_EXACT_NAME, SCORE_NAME_CONTAINS, SCORE_NAME_PREFIX, SCORE_NAME_WORD, SCORE_SUMMARY, SCORE_TAG, SCORE_TAG_CONTAINS};
pub use words::{WORDS, Word};
pub use worker::{Pixels, REFRESH_AFTER_SECONDS, Snapshot, UpdateRow, WATCH_INTERVAL};

use craft_ext_core::{
    catalogue::PackageFile,
    contract::Provided,
    id::{AppId, ExtensionId},
    listing::{Category, Image, ImageRole},
    manifest::ContributionKind,
    permission::Permission,
    project::PluginRef,
    tier::Tier,
    version::{Version, VersionReq},
};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use worker::{Event, Job, Worker, Write};

pub const UNCLEAN_EXIT: &str = "the app closed unexpectedly while this plugin was running";

#[derive(Clone, Debug)]
pub struct AppTarget {
    pub app: AppId,
    pub provided: Provided,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub store_root: PathBuf,
    pub apps: Vec<AppTarget>,
    pub platform: String,
    pub catalogue_base: String,
    test_keys: Vec<String>,
}

impl Config {
    pub fn new(store_root: impl Into<PathBuf>, apps: Vec<AppTarget>, platform: impl Into<String>) -> Self {
        Config {
            store_root: store_root.into(),
            apps,
            platform: platform.into(),
            catalogue_base: std::env::var("CRAFT_CATALOGUE_URL").unwrap_or_else(|_| craft_ext_core::catalogue::DEFAULT_URL.into()),
            test_keys: vec![],
        }
    }

    pub fn app(store_root: impl Into<PathBuf>, app: AppId, provided: Provided, platform: impl Into<String>) -> Self {
        Self::new(store_root, vec![AppTarget { app, provided }], platform)
    }

    #[cfg(test)]
    fn with_test_key(mut self, key: String) -> Self {
        self.test_keys.push(key);
        self
    }

    fn keys(&self) -> Vec<String> {
        let mut keys = craft_ext_store::catalogue::trusted_keys_for(&self.catalogue_base);
        keys.extend(self.test_keys.iter().cloned());
        keys
    }

    pub fn scope(&self) -> Option<&AppId> {
        match self.apps.as_slice() {
            [one] => Some(&one.app),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Discover,
    Installed,
    Updates,
    Problems,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Discover, Tab::Installed, Tab::Updates, Tab::Problems];

    pub fn word(self) -> Word {
        match self {
            Tab::Discover => Word::TabDiscover,
            Tab::Installed => Word::TabInstalled,
            Tab::Updates => Word::TabUpdates,
            Tab::Problems => Word::TabProblems,
        }
    }

    pub fn empty(self) -> Word {
        match self {
            Tab::Discover => Word::EmptyDiscover,
            Tab::Installed => Word::EmptyInstalled,
            Tab::Updates => Word::EmptyUpdates,
            Tab::Problems => Word::EmptyProblems,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sort {
    #[default]
    Relevance,
    Popular,
    RecentlyUpdated,
    Name,
}

impl Sort {
    pub const ALL: [Sort; 4] = [Sort::Relevance, Sort::Popular, Sort::RecentlyUpdated, Sort::Name];

    pub fn word(self) -> Word {
        match self {
            Sort::Relevance => Word::SortRelevance,
            Sort::Popular => Word::SortPopular,
            Sort::RecentlyUpdated => Word::SortRecentlyUpdated,
            Sort::Name => Word::SortName,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filters {
    pub app: Option<AppId>,
    pub category: Option<Category>,
    pub tier: Option<Tier>,
    pub kind: Option<ContributionKind>,
    pub installed: bool,
    pub has_update: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Query {
    pub text: String,
    pub filters: Filters,
    pub sort: Sort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Badge {
    Tier(Tier),
    BuiltFromSource,
    NativeFull,
    NativeRestricted,
    Advisory,
    Featured,
}

impl Badge {
    pub fn word(self) -> Word {
        match self {
            Badge::Tier(Tier::Official) => Word::TierOfficial,
            Badge::Tier(Tier::Verified) => Word::TierVerified,
            Badge::Tier(Tier::Community) => Word::TierCommunity,
            Badge::Tier(Tier::Local) => Word::TierLocal,
            Badge::Tier(Tier::Dev) => Word::TierDev,
            Badge::BuiltFromSource => Word::BadgeBuiltFromSource,
            Badge::NativeFull => Word::BadgeNativeFull,
            Badge::NativeRestricted => Word::BadgeNativeRestricted,
            Badge::Advisory => Word::BadgeAdvisory,
            Badge::Featured => Word::BadgeFeatured,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primary {
    Install,
    Update(Version),
    SetUp,
    Open,
    Working,
}

impl Primary {
    pub fn word(self) -> Word {
        match self {
            Primary::Install => Word::ActionInstall,
            Primary::Update(_) => Word::ActionUpdate,
            Primary::SetUp => Word::ActionSetUp,
            Primary::Open => Word::ActionOpen,
            Primary::Working => Word::StateWorking,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub id: ExtensionId,
    pub name: String,
    pub summary: String,
    pub vendor: String,
    pub tier: Tier,
    pub badges: Vec<Badge>,
    pub icon: Option<Image>,
    pub categories: Vec<Category>,
    pub installed: Option<Version>,
    pub update: Option<Version>,
    pub downloads: u64,
    pub stars: u32,
    pub updated: String,
    pub listed: bool,
    pub primary: Primary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Loadable<T> {
    Loading,
    Ready(T),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    SetTab(Tab),
    SetQuery(String),
    SetFilters(Filters),
    SetSort(Sort),
    Select(Option<ExtensionId>),
    LoadImage(ExtensionId, Image),
    Refresh,
    Install { id: ExtensionId, app: Option<AppId> },
    InstallFile { path: PathBuf, app: Option<AppId> },
    Update { id: ExtensionId, app: AppId },
    UpdateAll { app: Option<AppId> },
    Uninstall { id: ExtensionId },
    SetContribution { id: ExtensionId, app: AppId, contribution: String, on: bool },
    Grant { id: ExtensionId, app: AppId, permission: Permission },
    Revoke { id: ExtensionId, app: AppId, permission: Permission },
    Pin { id: ExtensionId, app: AppId, req: VersionReq },
    Unpin { id: ExtensionId, app: AppId },
    Rollback { id: ExtensionId, app: AppId, version: Version },
    TurnBackOn { id: ExtensionId, app: AppId },
    TurnOffAfterCrash { id: ExtensionId, app: AppId },
    RemoveFullAccess { id: ExtensionId, app: AppId },
    InstallMissing { app: AppId, refs: Vec<PluginRef> },
    SetUp { id: ExtensionId, app: AppId },
    SetupContribution { id: ExtensionId, contribution: String, on: bool },
    SetupPermission { id: ExtensionId, permission: Permission, on: bool },
    SetupFullAccess { id: ExtensionId, on: bool },
    ConfirmSetup { id: ExtensionId },
    Cancel { id: ExtensionId },
    Report { id: ExtensionId },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub word: Word,
    pub plugin: Option<ExtensionId>,
    pub subject: String,
    pub detail: String,
    pub error: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Changed,
    OpenSetup(ExtensionId),
    CloseSetup(ExtensionId),
    OpenUrl(String),
    Notice(Notice),
}

fn notice(word: Word, subject: impl ToString, detail: impl Into<String>, error: bool) -> Effect {
    Effect::Notice(Notice { word, plugin: None, subject: subject.to_string(), detail: detail.into(), error })
}

pub struct Model {
    config: Arc<Config>,
    worker: Worker,
    snapshot: Arc<Snapshot>,
    loaded: bool,
    in_flight: usize,
    tab: Tab,
    query: Query,
    selected: Option<ExtensionId>,
    packages: BTreeMap<ExtensionId, (String, Loadable<Arc<PackageFile>>)>,
    images: BTreeMap<String, Loadable<Arc<Pixels>>>,
    flows: BTreeMap<ExtensionId, Flow>,
    queued: Vec<(ExtensionId, AppId, FlowKind, Option<VersionReq>)>,
    cancelled: std::collections::BTreeSet<ExtensionId>,
    unclean: BTreeMap<AppId, Vec<ExtensionId>>,
    rows: Vec<Row>,
    problems: Vec<Problem>,
    detail: Option<Detail>,
    counts: [usize; 4],
}

impl Model {
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let query = Query { filters: Filters { app: config.scope().cloned(), ..Filters::default() }, ..Query::default() };
        let mut model = Model {
            worker: Worker::spawn(config.clone()),
            config,
            snapshot: Arc::new(Snapshot::default()),
            loaded: false,
            in_flight: 0,
            tab: Tab::Discover,
            query,
            selected: None,
            packages: BTreeMap::new(),
            images: BTreeMap::new(),
            flows: BTreeMap::new(),
            queued: Vec::new(),
            cancelled: std::collections::BTreeSet::new(),
            unclean: BTreeMap::new(),
            rows: vec![],
            problems: vec![],
            detail: None,
            counts: [0; 4],
        };
        model.send(Job::RefreshIfStale);
        model
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    pub fn loaded(&self) -> bool {
        self.loaded
    }

    pub fn busy(&self) -> bool {
        self.in_flight > 0 || !self.loaded
    }

    pub fn tab(&self) -> Tab {
        self.tab
    }

    pub fn query(&self) -> &Query {
        &self.query
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn problems(&self) -> &[Problem] {
        &self.problems
    }

    pub fn count(&self, tab: Tab) -> usize {
        self.counts[tab as usize]
    }

    pub fn selected(&self) -> Option<&ExtensionId> {
        self.selected.as_ref()
    }

    pub fn detail(&self) -> Option<&Detail> {
        self.detail.as_ref()
    }

    pub fn flow(&self, id: &ExtensionId) -> Option<&Flow> {
        self.flows.get(id)
    }

    pub fn setup(&self, id: &ExtensionId) -> Option<&Setup> {
        match &self.flows.get(id)?.step {
            Step::Setup(s) => Some(s),
            Step::Working(_) => None,
        }
    }

    pub fn image(&self, url: &str) -> Option<&Loadable<Arc<Pixels>>> {
        self.images.get(url)
    }

    pub fn catalogue_error(&self) -> Option<&str> {
        self.snapshot.index_error.as_deref()
    }

    pub fn set_unclean_exit(&mut self, app: AppId, ids: Vec<ExtensionId>) -> Vec<Effect> {
        self.unclean.insert(app, ids);
        self.rebuild();
        vec![Effect::Changed]
    }

    fn about(&self, word: Word, id: &ExtensionId, detail: impl Into<String>, error: bool) -> Effect {
        Effect::Notice(Notice { word, plugin: Some(id.clone()), subject: self.name_of(id), detail: detail.into(), error })
    }

    fn send(&mut self, job: Job) {
        if self.worker.jobs.send(job).is_ok() {
            self.in_flight += 1;
        }
    }

    fn view(&self) -> rank::View<'_> {
        rank::View { snapshot: &self.snapshot, config: &self.config, flows: &self.flows }
    }

    fn rebuild(&mut self) {
        let view = self.view();
        let rows = view.rows(self.tab, &self.query);
        let problems = problems::find(&view, &self.unclean);
        let mut counts = [0; 4];
        for tab in [Tab::Discover, Tab::Installed, Tab::Updates] {
            counts[tab as usize] = if tab == self.tab { rows.len() } else { view.rows(tab, &self.query).len() };
        }
        counts[Tab::Problems as usize] = problems.len();
        let detail = self.selected.as_ref().and_then(|id| {
            let badges = view.entry(id).map(|e| view.badges(&e)).unwrap_or_default();
            detail::build(&view, id, self.packages.get(id).map(|(_, p)| p), badges)
        });
        self.rows = rows;
        self.problems = problems;
        self.counts = counts;
        self.detail = detail;
    }

    fn app_for(&self, id: &ExtensionId, app: Option<AppId>) -> Option<AppId> {
        if app.is_some() {
            return app;
        }
        let supported = self.view().entry(id).map(|e| e.apps.to_vec()).unwrap_or_default();
        self.config.apps.iter().map(|t| &t.app).find(|a| supported.contains(a)).or(self.config.scope()).cloned()
    }

    fn start(&mut self, id: ExtensionId, app: AppId, kind: FlowKind, req: Option<VersionReq>) -> Vec<Effect> {
        if let Some(flow) = self.flows.get(&id) {
            if flow.app != app && !self.queued.iter().any(|(i, a, ..)| i == &id && a == &app) {
                self.queued.push((id, app, kind, req));
            }
            return vec![];
        }
        self.cancelled.remove(&id);
        self.flows.insert(id.clone(), Flow { app: app.clone(), kind, step: Step::Working(Word::StateDownloading) });
        self.send(Job::Stage { id, app, kind, req });
        self.rebuild();
        vec![Effect::Changed]
    }

    fn end_flow(&mut self, id: &ExtensionId, mut effects: Vec<Effect>) -> Vec<Effect> {
        self.flows.remove(id);
        if let Some(at) = self.queued.iter().position(|(i, ..)| i == id) {
            let (id, app, kind, req) = self.queued.remove(at);
            effects.extend(self.start(id, app, kind, req));
        }
        effects
    }

    fn write(&mut self, w: Write) -> Vec<Effect> {
        self.send(Job::Write(w));
        vec![Effect::Changed]
    }

    fn setup_mut(&mut self, id: &ExtensionId) -> Option<&mut Setup> {
        match &mut self.flows.get_mut(id)?.step {
            Step::Setup(s) => Some(s),
            Step::Working(_) => None,
        }
    }

    pub fn apply(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::SetTab(tab) => {
                self.tab = tab;
                self.rebuild();
                vec![Effect::Changed]
            }
            Action::SetQuery(text) => {
                self.query.text = text;
                self.rebuild();
                vec![Effect::Changed]
            }
            Action::SetFilters(filters) => {
                self.query.filters = filters;
                self.rebuild();
                vec![Effect::Changed]
            }
            Action::SetSort(sort) => {
                self.query.sort = sort;
                self.rebuild();
                vec![Effect::Changed]
            }
            Action::Select(id) => {
                self.selected = id.clone();
                if let Some(id) = id {
                    self.load_package(&id);
                }
                self.rebuild();
                vec![Effect::Changed]
            }
            Action::LoadImage(id, image) => {
                let repo = self.snapshot.index.as_ref().and_then(|i| i.package(&id)).map(|p| p.repo.clone());
                let role = if self.snapshot.index.as_ref().and_then(|i| i.package(&id)).and_then(|p| p.icon.as_ref()) == Some(&image) {
                    ImageRole::Icon
                } else {
                    ImageRole::Screenshot
                };
                match repo {
                    Some(repo) if !self.images.contains_key(&image.url) => {
                        self.images.insert(image.url.clone(), Loadable::Loading);
                        self.send(Job::Media(image.url.clone(), repo, image, role));
                        vec![Effect::Changed]
                    }
                    _ => vec![],
                }
            }
            Action::Refresh => {
                self.send(Job::Refresh);
                vec![Effect::Changed]
            }
            Action::Install { id, app } => match self.app_for(&id, app) {
                Some(app) => self.start(id, app, FlowKind::Install, None),
                None => vec![self.about(Word::NoticeFailed, &id, "choose an app this plugin supports", true)],
            },
            Action::InstallFile { path, app } => match app.or_else(|| self.config.scope().cloned()).or_else(|| self.config.apps.first().map(|t| t.app.clone())) {
                Some(app) => {
                    self.send(Job::StageFile { path, app });
                    vec![Effect::Changed]
                }
                None => vec![notice(Word::NoticeFailed, path.display(), "no app is configured", true)],
            },
            Action::Update { id, app } => self.start(id, app, FlowKind::Update, None),
            Action::UpdateAll { app } => {
                let updates: Vec<UpdateRow> = self.snapshot.updates.iter().filter(|u| app.as_ref().is_none_or(|a| a == &u.app)).cloned().collect();
                let mut effects = vec![];
                for u in updates {
                    effects.extend(self.start(u.id, u.app, FlowKind::Update, None));
                }
                effects
            }
            Action::Uninstall { id } => self.write(Write::Uninstall(id)),
            Action::SetContribution { id, app, contribution, on } => self.write(Write::Enable(app, id, contribution, on)),
            Action::Grant { id, app, permission } => self.write(Write::Grant(app, id, permission, true)),
            Action::Revoke { id, app, permission } => self.write(Write::Grant(app, id, permission, false)),
            Action::Pin { id, app, req } => self.write(Write::Pin(app, id, Some(req))),
            Action::Unpin { id, app } => self.write(Write::Pin(app, id, None)),
            Action::Rollback { id, app, version } => self.start(id, app, FlowKind::Rollback, Some(VersionReq::Exact(version))),
            Action::TurnBackOn { id, app } => self.write(Write::TurnBackOn(app, id)),
            Action::TurnOffAfterCrash { id, app } => match self.view().active(&app, &id) {
                Some(version) => self.write(Write::Quarantine(app, id, version)),
                None => vec![],
            },
            Action::RemoveFullAccess { id, app } => self.write(Write::FullAccessOff(app, id)),
            Action::InstallMissing { app, refs } => {
                for r in &refs {
                    self.flows.entry(r.id.clone()).or_insert_with(|| Flow { app: app.clone(), kind: FlowKind::InstallMissing, step: Step::Working(Word::StateChecking) });
                }
                self.send(Job::Missing { app, refs });
                self.rebuild();
                vec![Effect::Changed]
            }
            Action::SetUp { id, app } => self.open_setup(id, app),
            Action::SetupContribution { id, contribution, on } => self.edit_setup(&id, |s| s.set_contribution(&contribution, on)),
            Action::SetupPermission { id, permission, on } => self.edit_setup(&id, |s| s.set_permission(&permission, on)),
            Action::SetupFullAccess { id, on } => self.edit_setup(&id, |s| s.set_full_access(on)),
            Action::ConfirmSetup { id } => {
                let Some(flow) = self.flows.get_mut(&id) else { return vec![] };
                let Step::Setup(setup) = &flow.step else { return vec![] };
                if let Some(blocker) = setup.blocker() {
                    return vec![self.about(blocker, &id, "", true)];
                }
                let choice = setup.choice();
                let kind = flow.kind;
                flow.step = Step::Working(Word::StateApplying);
                self.send(Job::Apply(choice, kind));
                self.rebuild();
                vec![Effect::CloseSetup(id), Effect::Changed]
            }
            Action::Cancel { id } => {
                let was_setup = matches!(self.flows.get(&id).map(|f| &f.step), Some(Step::Setup(_)));
                if self.flows.contains_key(&id) {
                    self.cancelled.insert(id.clone());
                }
                let effects = if was_setup { vec![Effect::CloseSetup(id.clone()), Effect::Changed] } else { vec![Effect::Changed] };
                let effects = self.end_flow(&id, effects);
                self.rebuild();
                effects
            }
            Action::Report { id } => {
                let v = self.view().installed_version(&id).or_else(|| self.snapshot.index.as_ref().and_then(|i| i.package(&id)).map(|p| p.latest));
                vec![Effect::OpenUrl(craft_ext_store::manager::report_url(&id, v.unwrap_or(Version::new(0, 0, 0))))]
            }
        }
    }

    fn edit_setup(&mut self, id: &ExtensionId, f: impl FnOnce(&mut Setup)) -> Vec<Effect> {
        match self.setup_mut(id) {
            Some(s) => {
                f(s);
                vec![Effect::Changed]
            }
            None => vec![],
        }
    }

    fn open_setup(&mut self, id: ExtensionId, app: AppId) -> Vec<Effect> {
        let saved = self.snapshot.state.apps.get(&app).and_then(|a| a.get(&id));
        let version = saved.and_then(|s| s.pending.or(s.active)).or_else(|| self.view().newest_installed(&id));
        let Some(manifest) = version.and_then(|v| self.view().manifest(&id, v).cloned()) else {
            return vec![self.about(Word::NoticeFailed, &id, "install the plugin first", true)];
        };
        let provided = self.config.apps.iter().find(|t| t.app == app).map(|t| t.provided.clone()).unwrap_or_default();
        let setup = Setup::new(manifest, app.clone(), saved, &provided, &self.config.platform, vec![]);
        self.flows.insert(id.clone(), Flow { app, kind: FlowKind::Install, step: Step::Setup(Box::new(setup)) });
        self.rebuild();
        vec![Effect::OpenSetup(id), Effect::Changed]
    }

    fn load_package(&mut self, id: &ExtensionId) {
        let Some(row) = self.snapshot.index.as_ref().and_then(|i| i.package(id)) else { return };
        let current = self.packages.get(id).is_some_and(|(sha, p)| sha == &row.file_sha256 && !matches!(p, Loadable::Failed(_)));
        if !current {
            self.packages.insert(id.clone(), (row.file_sha256.clone(), Loadable::Loading));
            self.send(Job::Package(id.clone()));
        }
    }

    pub fn poll(&mut self) -> Vec<Effect> {
        let mut effects = vec![];
        while let Ok(event) = self.worker.events.try_recv() {
            effects.extend(self.receive(event));
        }
        if !effects.is_empty() {
            self.rebuild();
            effects.push(Effect::Changed);
            effects.dedup();
        }
        effects
    }

    fn receive(&mut self, event: Event) -> Vec<Effect> {
        match event {
            Event::Idle => {
                self.in_flight = self.in_flight.saturating_sub(1);
                vec![]
            }
            Event::Loaded(snapshot) => {
                self.snapshot = snapshot;
                self.loaded = true;
                if let Some(id) = self.selected.clone() {
                    self.load_package(&id);
                }
                vec![Effect::Changed]
            }
            Event::Refreshed(Ok(())) => vec![notice(Word::NoticeRefreshed, "", "", false)],
            Event::Refreshed(Err(e)) => vec![notice(Word::NoticeFailed, "", e, true)],
            Event::Package(id, result) => {
                if let Some((_, slot)) = self.packages.get_mut(&id) {
                    *slot = match result {
                        Ok(p) => Loadable::Ready(p),
                        Err(e) => Loadable::Failed(e),
                    };
                }
                vec![Effect::Changed]
            }
            Event::Media(url, result) => {
                self.images.insert(url, result.map_or_else(Loadable::Failed, Loadable::Ready));
                vec![Effect::Changed]
            }
            Event::Staged { id, app, kind, manifest, needs_setup, dependencies } => {
                if self.cancelled.remove(&id) {
                    return vec![];
                }
                if !needs_setup {
                    let done = Effect::Notice(Notice { word: kind.done(), plugin: Some(id.clone()), subject: manifest.name.clone(), detail: manifest.version.to_string(), error: false });
                    return self.end_flow(&id, vec![done]);
                }
                let tracked = self.flows.get(&id).is_some_and(|f| f.app == app) || kind == FlowKind::Install;
                if !tracked {
                    return vec![];
                }
                let provided = self.config.apps.iter().find(|t| t.app == app).map(|t| t.provided.clone()).unwrap_or_default();
                let saved = self.snapshot.state.apps.get(&app).and_then(|a| a.get(&id));
                let setup = Setup::new(*manifest, app.clone(), saved, &provided, &self.config.platform, dependencies);
                self.flows.insert(id.clone(), Flow { app, kind, step: Step::Setup(Box::new(setup)) });
                vec![Effect::OpenSetup(id)]
            }
            Event::Applied { id, kind, .. } => {
                let done = self.about(if kind == FlowKind::Install { Word::NoticeSetUp } else { kind.done() }, &id, "", false);
                self.end_flow(&id, vec![done])
            }
            Event::Failed(id, e) => {
                let failed = self.about(Word::NoticeFailed, &id, e, true);
                self.end_flow(&id, vec![failed])
            }
            Event::FileFailed(path, e) => vec![notice(Word::NoticeFailed, path.display(), e, true)],
            Event::Settled(id, word, detail) => {
                let settled = self.about(word, &id, detail, word == Word::NoticeUnavailable);
                self.end_flow(&id, vec![settled])
            }
            Event::Wrote(word, id, Ok(())) => vec![self.about(word, &id, "", false)],
            Event::Wrote(_, id, Err(e)) => vec![self.about(Word::NoticeFailed, &id, e, true)],
        }
    }
}

#[cfg(test)]
mod tests;

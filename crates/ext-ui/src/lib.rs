//! Embeddable plugin management. Downloads stage releases; consent activates them.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]
pub mod helpers;
use craft_ext_core::{
    catalogue::PackageSummary,
    id::{AppId, ExtensionId},
    manifest::Manifest,
    project::{self, PluginRef},
    version::{Version, VersionReq},
};
use craft_ext_store::{
    Manager,
    manager::{InstallResult, Update},
    store,
};
use helpers::*;
use std::{
    collections::BTreeMap,
    sync::{Arc, mpsc},
};
type Result<T> = std::result::Result<T, String>;
enum Done {
    Installed(Arc<Manager>, Box<InstallResult>),
    Versions(ExtensionId, Vec<(Version, craft_ext_core::manifest::Compat)>),
    Updates((Vec<Update>, Vec<(ExtensionId, String)>)),
    Refreshed,
}
/// One outstanding job. Cancelling discards its result; an in-flight download may finish staging,
/// but it can never activate or grant access.
#[derive(Default)]
struct Jobs {
    receiver: Option<mpsc::Receiver<Result<Done>>>,
    cancelled: bool,
}
impl Jobs {
    fn start(&mut self, f: impl FnOnce() -> Result<Done> + Send + 'static) {
        if self.receiver.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        self.cancelled = false;
        std::thread::spawn(move || {
            let _ = tx.send(f());
        });
    }
    fn poll(&mut self) -> Option<Result<Done>> {
        let r = self.receiver.as_ref()?.try_recv();
        match r {
            Ok(v) => {
                self.receiver = None;
                if self.cancelled { None } else { Some(v) }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.receiver = None;
                Some(Err("The background operation stopped. Please retry.".into()))
            }
            Err(_) => None,
        }
    }
    fn show(&mut self, ui: &mut egui::Ui) {
        if self.receiver.is_some() {
            ui.spinner();
            ui.label(if self.cancelled { "Cancelling; waiting for the current download to finish" } else { "Working…" });
            if ui.button("Cancel operation").clicked() {
                self.cancelled = true;
            }
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}
struct Setup {
    mgr: Arc<Manager>,
    manifest: Manifest,
    choices: BTreeMap<String, bool>,
    permissions: Vec<PermissionChoice>,
    native: bool,
    error: Option<String>,
}
impl Setup {
    fn new(mgr: Arc<Manager>, manifest: Manifest) -> Self {
        Self { mgr, manifest, choices: BTreeMap::new(), permissions: vec![], native: false, error: None }
    }
    fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut close = false;
        egui::Modal::new(egui::Id::unique("craft-plugin-setup")).show(ctx, |ui| {
            ui.heading(format!("Set up {}", self.manifest.name));
            let Some(app) = self.mgr.app.as_ref() else {
                ui.label("Choose an app before setup.");
                if ui.button("Cancel").clicked() {
                    close = true;
                }
                return;
            };
            let mut changed = self.permissions.is_empty();
            for c in self.manifest.contributions_for(app) {
                let reason = c.contract.check(&self.mgr.provided).map(|r| r.to_string());
                let v = self.choices.entry(c.id.clone()).or_insert(false);
                ui.add_enabled_ui(reason.is_none(), |ui| {
                    changed |= ui.checkbox(v, &c.label).changed();
                });
                if let Some(r) = reason {
                    ui.label(format!("Unavailable: {r}"));
                }
            }
            let sheet = permission_sheet(&self.manifest, app, |c| self.choices.get(c).copied().unwrap_or(false));
            if changed {
                let old = std::mem::take(&mut self.permissions);
                self.permissions = sheet_model(&sheet);
                for p in &mut self.permissions {
                    p.checked = old.iter().any(|o| o.permission == p.permission && o.checked);
                }
            }
            for p in &mut self.permissions {
                ui.checkbox(&mut p.checked, format!("{}{}", p.sentence, if p.required { " (required)" } else { " (optional)" }));
                if let Some(why) = &p.why {
                    ui.label(egui::RichText::new(format!("Author: “{why}”")).italics());
                }
            }
            for w in sheet.warnings {
                ui.label(format!("Warning: {w}"));
            }
            if sheet.native {
                ui.checkbox(&mut self.native, egui::RichText::new("Native: full access to your computer").color(egui::Color32::RED));
            }
            if let Some(e) = &self.error {
                ui.label(e);
            }
            let platform_ok = self.manifest.native.as_ref().is_none_or(|n| n.executables.contains_key(&self.mgr.platform));
            if !platform_ok {
                ui.label("Unavailable: no native executable for this platform");
            }
            let ready = platform_ok
                && self.manifest.compat(app, &self.mgr.provided).usable()
                && self.permissions.iter().all(|p| !p.required || p.checked)
                && (!sheet.native || self.native);
            if ui.add_enabled(ready, egui::Button::new("Confirm setup")).clicked() {
                match self.confirm() {
                    Ok(()) => close = true,
                    Err(e) => self.error = Some(e),
                }
            }
            if ui.button("Cancel").clicked() {
                close = true;
            }
        });
        close
    }
    fn confirm(&self) -> Result<()> {
        let app = self.mgr.app.as_ref().ok_or("Choose an app")?;
        let id = &self.manifest.id;
        let run = || -> store::Result<()> {
            // Disable first so partial write failures cannot enable new work with incomplete consent.
            let state = self.mgr.store.state()?;
            if let Some(a) = state.apps.get(app).and_then(|a| a.get(id)) {
                for c in &a.enabled {
                    self.mgr.disable(id, c)?;
                }
                for p in &a.grants {
                    self.mgr.revoke(id, p)?;
                }
            }
            for p in &self.permissions {
                if p.checked {
                    self.mgr.grant(id, p.permission.clone())?;
                }
            }
            self.mgr.store.set_full_access(app, id, self.native)?;
            self.mgr.store.set_active(app, id, self.manifest.version)?;
            if self.mgr.store.state()?.apps.get(app).and_then(|a| a.get(id)).is_some_and(|a| a.pending.is_some()) {
                self.mgr.store.accept_pending(app, id)?;
            }
            for (c, on) in &self.choices {
                if *on {
                    self.mgr.enable(id, c)?;
                }
            }
            Ok(())
        };
        run().map_err(|e| e.to_string())
    }
}
/// Shared view. Launcher callbacks must return managers pointing at the same store.
#[derive(Default)]
pub struct ManagerView {
    tab: usize,
    query: String,
    kind: String,
    works: bool,
    native: bool,
    jobs: Jobs,
    error: Option<String>,
    setup: Option<Setup>,
    versions: BTreeMap<ExtensionId, Vec<(Version, craft_ext_core::manifest::Compat)>>,
    selected: BTreeMap<ExtensionId, Version>,
    apps: BTreeMap<ExtensionId, AppId>,
    launcher: Option<fn(&AppId) -> Manager>,
    problems: Vec<(ExtensionId, String)>,
    updates: Vec<Update>,
    update_app: Option<AppId>,
}
impl ManagerView {
    pub fn new() -> Self {
        Self { native: true, ..Default::default() }
    }
    pub fn new_launcher(factory: fn(&AppId) -> Manager) -> Self {
        Self { launcher: Some(factory), ..Self::new() }
    }
    pub fn set_problems(&mut self, p: Vec<(ExtensionId, String)>) {
        self.problems = p;
    }
    fn error<T>(&mut self, r: store::Result<T>) {
        if let Err(e) = r {
            self.error = Some(e.to_string());
        }
    }
    fn install(&mut self, mgr: Arc<Manager>, id: ExtensionId, v: Option<Version>) {
        self.jobs.start(move || {
            let req = v.map(|v| VersionReq::parse(&format!("={v}"))).transpose().map_err(|e| e.to_string())?;
            mgr.install(&id, req).map(|r| Done::Installed(mgr, Box::new(r))).map_err(|e| e.to_string())
        });
    }
    fn app_manager(&mut self, ui: &mut egui::Ui, mgr: &Arc<Manager>, m: &Manifest) -> Option<Arc<Manager>> {
        if mgr.app.is_some() {
            return Some(mgr.clone());
        }
        let first = m.apps.first()?.clone();
        let app = self.apps.entry(m.id.clone()).or_insert(first);
        egui::ComboBox::from_id_salt(("app", &m.id)).selected_text(app.to_string()).show_ui(ui, |ui| {
            for a in &m.apps {
                ui.selectable_value(app, a.clone(), a.to_string());
            }
        });
        self.launcher.map(|f| Arc::new(f(app)))
    }
    pub fn show(&mut self, ui: &mut egui::Ui, mgr: &Arc<Manager>) {
        if let Some(r) = self.jobs.poll() {
            match r {
                Ok(Done::Installed(m, r)) => self.setup = Some(Setup::new(m, r.0.manifest)),
                Ok(Done::Versions(id, v)) => {
                    self.versions.insert(id, v);
                }
                Ok(Done::Updates((v, problems))) => { self.updates = v; self.problems = problems; },
                Ok(Done::Refreshed) => {}
                Err(e) => self.error = Some(e),
            }
        }
        ui.horizontal(|ui| {
            for (i, t) in ["Browse", "Installed", "Updates", "Problems"].iter().enumerate() {
                ui.selectable_value(&mut self.tab, i, *t);
            }
            if ui.button("Refresh catalogue").clicked() {
                let m = mgr.clone();
                self.jobs.start(move || m.refresh(store::now()).map(|_| Done::Refreshed).map_err(|e| e.to_string()));
            }
        });
        self.jobs.show(ui);
        if let Some(e) = &self.error {
            ui.label(format!("Could not complete operation: {e}"));
        }
        match self.tab {
            0 => self.browse(ui, mgr),
            1 => self.installed(ui, mgr, false),
            2 => {
                let mut update_mgr = Some(mgr.clone());
                if mgr.app.is_none() {
                    let mut apps = std::collections::BTreeSet::new();
                    if let Ok(state) = mgr.store.state() {
                        apps.extend(state.apps.keys().cloned());
                        for (id, package) in &state.packages {
                            if let Some(v) = package.releases.keys().next_back() {
                                if let Ok(manifest) = mgr.store.manifest(&state, id, *v) {
                                    apps.extend(manifest.apps);
                                }
                            }
                        }
                    }
                    ui.label("Check updates for app");
                    egui::ComboBox::from_id_salt("update-app")
                        .selected_text(self.update_app.as_ref().map(ToString::to_string).unwrap_or_else(|| "Choose app".into()))
                        .show_ui(ui, |ui| {
                            for app in apps {
                                if ui.selectable_value(&mut self.update_app, Some(app.clone()), app.to_string()).changed() {
                                    self.updates.clear();
                                }
                            }
                        });
                    update_mgr = self.update_app.as_ref().and_then(|a| self.launcher.map(|f| Arc::new(f(a))));
                }
                if ui.add_enabled(update_mgr.is_some(), egui::Button::new("Check for updates")).clicked() {
                    let Some(m) = update_mgr.clone() else { return };
                    self.jobs.start(move || m.update_check().map(Done::Updates).map_err(|e| e.to_string()));
                }
                ui.label(format!("{} updates", self.updates.len()));
                for (id, old, new, wide) in self.updates.clone() {
                    ui.label(format!("{id}: {old} → {new}"));
                    if wide {
                        ui.label("Warning: wider permissions or native access; review setup");
                    }
                    ui.label("Review the author's release notes in the source repository and permissions in setup.");
                    if ui.button(format!("Update {id}")).clicked() {
                        if let Some(m) = &update_mgr {
                            self.install(m.clone(), id, Some(new));
                        }
                    }
                }
            }
            _ => self.installed(ui, mgr, true),
        }
        ui.separator();
        ui.label("Plugins are made by their authors. ArtCraft lists them but does not build, host or guarantee them.");
        if mgr.app.is_none() {
            if let Some(mut setup) = self.setup.take() {
                if let Some(m) = self.app_manager(ui, mgr, &setup.manifest) {
                    setup.mgr = m;
                }
                self.setup = Some(setup);
            }
        }
        if self.setup.as_ref().is_some_and(|s| s.mgr.app.is_some()) && self.setup.as_mut().is_some_and(|s| s.show(ui.ctx())) {
            self.setup = None;
        }
    }
    fn chooser(&mut self, ui: &mut egui::Ui, mgr: &Arc<Manager>, id: &ExtensionId, current: Version) -> Version {
        if ui.button("Load listed and retained versions").clicked() {
            let m = mgr.clone();
            let id = id.clone();
            self.jobs.start(move || m.versions(&id).map(|v| Done::Versions(id, v)).map_err(|e| e.to_string()));
        }
        let v = self.selected.entry(id.clone()).or_insert(current);
        egui::ComboBox::from_id_salt(("version", id)).selected_text(format!("Version {v}")).show_ui(ui, |ui| {
            if let Some(versions) = self.versions.get(id) {
                for (ver, c) in versions {
                    ui.selectable_value(v, *ver, format!("{ver}: {}", compatibility(c)));
                }
            }
        });
        *v
    }
    fn browse(&mut self, ui: &mut egui::Ui, mgr: &Arc<Manager>) {
        ui.label("Search plugins");
        ui.text_edit_singleline(&mut self.query);
        ui.label("Filter kind (for example effect, panel, command)");
        ui.text_edit_singleline(&mut self.kind);
        ui.checkbox(&mut self.works, "Works in this app");
        ui.checkbox(&mut self.native, "Include native plugins");
        let all_apps = Manager::new(craft_ext_store::Store::at(mgr.store.root()), None, mgr.provided.clone(), mgr.platform.clone());
        match all_apps.search(&self.query) {
            Err(e) => {
                ui.label(e.to_string());
            }
            Ok(rows) => {
                for p in rows {
                    if (!self.native && p.native)
                        || (!self.kind.is_empty() && !p.kinds.iter().any(|k| k.label().to_lowercase().contains(&self.kind.to_lowercase())))
                    {
                        continue;
                    }
                    let compat = mgr.app.as_ref().is_none_or(|a| p.apps.contains(a));
                    let contracts_work = self.versions.get(&p.id).is_none_or(|versions| versions.iter().any(|(_, c)| c.usable()));
                    if self.works && (!compat || !contracts_work) {
                        continue;
                    }
                    ui.push_id(p.id.clone(), |ui| {
                        ui.group(|ui| {
                            self.browse_row(ui, mgr, p, compat);
                        });
                    });
                }
            }
        }
    }
    fn browse_row(&mut self, ui: &mut egui::Ui, mgr: &Arc<Manager>, p: PackageSummary, compat: bool) {
        ui.heading(&p.name);
        ui.label(&p.summary);
        ui.label(format!("Vendor: {}", p.vendor));
        // Read tier from the signed cached index, never infer trust from package naming.
        match craft_ext_store::catalogue::Catalogue::cached(&mgr.store.root().join("catalogue")) {
            Ok(index) => {
                ui.label(badge_row_text(index.tier_of(&p.id), p.stars, &p.stars_date, p.native));
            }
            Err(e) => {
                ui.label(format!("Trust information unavailable: {e}"));
            }
        }
        ui.label(if compat { "App supported; load versions to check contracts" } else { "Incompatible: app not supported" });
        if mgr.app.is_none() {
            ui.label(format!("Apps: {}", p.apps.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")));
        }
        if !self.versions.contains_key(&p.id) {
            if ui.button("Find compatible releases").clicked() {
                let m = mgr.clone();
                let id = p.id.clone();
                self.jobs.start(move || m.versions(&id).map(|v| Done::Versions(id, v)).map_err(|e| e.to_string()));
            }
            return;
        }
        let versions = &self.versions[&p.id];
        let compatible = newest_compatible(versions);
        let Some(best) = compatible else {
            ui.label("No compatible release for this app");
            return;
        };
        let latest = versions.iter().find(|(v, _)| *v == p.latest);
        if let Some((_, c)) = latest.filter(|(_, c)| !c.usable()) {
            let app = mgr.app.as_ref().map(ToString::to_string).unwrap_or_else(|| "this app".into());
            ui.label(format!("Version {} doesn't work in {app}: {}. Install {best} instead?", p.latest, compatibility(c)));
        }
        let v = self.chooser(ui, mgr, &p.id, best);
        let usable = self.versions[&p.id].iter().any(|(version, c)| *version == v && c.usable());
        if ui.add_enabled(usable, egui::Button::new(format!("Install {v}"))).clicked() {
            self.install(mgr.clone(), p.id, Some(v));
        }
    }
    fn installed(&mut self, ui: &mut egui::Ui, mgr: &Arc<Manager>, problems: bool) {
        let state = match mgr.store.state() {
            Ok(s) => s,
            Err(e) => {
                ui.label(e.to_string());
                return;
            }
        };
        for (id, p) in &state.packages {
            let current = mgr
                .app
                .as_ref()
                .and_then(|a| state.apps.get(a))
                .and_then(|a| a.get(id))
                .and_then(|a| a.active)
                .or_else(|| p.releases.keys().next_back().copied());
            let Some(v) = current else { continue };
            let manifest = match mgr.store.manifest(&state, id, v) {
                Ok(m) => m,
                Err(e) => {
                    ui.label(format!("{id}: {e}"));
                    continue;
                }
            };
            ui.push_id(id, |ui| {
                ui.group(|ui| {
                    ui.heading(&manifest.name);
                    let Some(m) = self.app_manager(ui, mgr, &manifest) else {
                        ui.label("Choose an app using a launcher callback to set up this plugin.");
                        return;
                    };
                    let Some(app) = m.app.as_ref() else { return };
                    let compat = manifest.compat(app, &m.provided);
                    if problems {
                        if !matches!(compat, craft_ext_core::manifest::Compat::Compatible) {
                            ui.label(compatibility(&compat));
                        }
                        for r in &state.revocations {
                            if r.matches(id, v, &p.releases[&v].sha256, p.origin.vendor()) {
                                ui.label(format!("{:?}: {}", r.severity, r.reason));
                            }
                        }
                        return;
                    }
                    let setup = state.apps.get(app).and_then(|a| a.get(id));
                    ui.label(format!("Installed version: {v}"));
                    ui.label(compatibility(&compat));
                    let selected = self.chooser(ui, &m, id, v);
                    if ui.button("Set up selected version / rollback").clicked() {
                        if p.releases.contains_key(&selected) {
                            match m.store.manifest(&state, id, selected) {
                                Ok(target) => {
                                    for c in &manifest.contributions {
                                        if target.contribution(&c.id).is_none_or(|old| old.state_version < c.state_version) {
                                            self.error =
                                                Some("Older version may need older saved data. Keep newer project data as placeholders.".into());
                                        }
                                    }
                                    self.setup = Some(Setup::new(m.clone(), target));
                                }
                                Err(e) => self.error = Some(e.to_string()),
                            }
                        } else {
                            self.install(m.clone(), id.clone(), Some(selected));
                        }
                    }
                    if ui.button("Pin selected version").clicked() {
                        match VersionReq::parse(&format!("={selected}")) {
                            Ok(req) => self.error(m.pin(id, req)),
                            Err(e) => self.error = Some(e.to_string()),
                        }
                    }
                    if ui.button("Unpin").clicked() {
                        self.error(m.unpin(id));
                    }
                    for c in manifest.contributions_for(app) {
                        let mut on = setup.is_some_and(|a| a.enabled.contains(&c.id));
                        if ui.checkbox(&mut on, &c.label).changed() {
                            if on {
                                self.setup = Some(Setup::new(m.clone(), manifest.clone()));
                            } else {
                                self.error(m.disable(id, &c.id));
                            }
                        }
                    }
                    if let Some(a) = setup {
                        for perm in &a.grants {
                            ui.horizontal(|ui| {
                                ui.label(perm.sentence());
                                if ui.button("Revoke").clicked() {
                                    self.error(m.revoke(id, perm));
                                }
                            });
                        }
                        if manifest.is_native() {
                            let mut full = a.full_access;
                            if ui.checkbox(&mut full, "Native: full access to your computer").changed() {
                                if full {
                                    self.setup = Some(Setup::new(m.clone(), manifest.clone()));
                                } else {
                                    self.error(m.store.set_full_access(app, id, false));
                                }
                            }
                        }
                    }
                    if ui.button("Update").clicked() {
                        self.install(m.clone(), id.clone(), None);
                    }
                    if ui.button("Remove").clicked() {
                        self.error(m.remove(id));
                    }
                    ui.hyperlink_to("Report plugin", m.report_url(id, v));
                });
            });
        }
        if problems {
            for (id, why) in &self.problems {
                ui.label(format!("Failed load — {id}: {why}"));
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptOutcome {
    pub skipped: Vec<ExtensionId>,
    pub open_without: bool,
}
/// Project inspection does not download or enable anything until a button is clicked.
pub struct ProjectPrompt {
    statuses: Vec<(PluginRef, project::Status)>,
    view: ManagerView,
    closed: bool,
    skipped: std::collections::BTreeSet<ExtensionId>,
}
impl ProjectPrompt {
    pub fn new(statuses: Vec<(PluginRef, project::Status)>) -> Self {
        Self { statuses, view: ManagerView::new(), closed: false, skipped: Default::default() }
    }
    pub fn show(&mut self, ctx: &egui::Context, mgr: &Arc<Manager>) -> Option<PromptOutcome> {
        if self.closed {
            return None;
        }
        let mut outcome = None;
        if let Some(r) = self.view.jobs.poll() {
            match r {
                Ok(Done::Installed(m, r)) => self.view.setup = Some(Setup::new(m, r.0.manifest)),
                Err(e) => self.view.error = Some(e),
                _ => {}
            }
        }
        if let Some(s) = self.view.setup.as_mut() {
            if s.show(ctx) {
                self.view.setup = None;
                if let Ok(s) = mgr.check_project(&self.statuses.iter().map(|(r, _)| r.clone()).collect::<Vec<_>>()) {
                    self.statuses = s;
                }
            }
            return None;
        }
        egui::Modal::new(egui::Id::unique("craft-project-plugins")).show(ctx, |ui| {
            ui.heading(format!("This project uses {} plugins", self.statuses.len()));
            self.view.jobs.show(ui);
            if let Some(e) = &self.view.error {
                ui.label(e);
            }
            for (r, s) in &self.statuses {
                if self.skipped.contains(&r.id) {
                    continue;
                }
                ui.push_id(&r.id, |ui| {
                    ui.label(format!("{}: {}", r.name, status_text(s)));
                    match s {
                        project::Status::Downloadable | project::Status::NeedsOtherVersion { .. } => {
                            if ui.button("Download").clicked() {
                                self.view.install(mgr.clone(), r.id.clone(), Some(r.version));
                            }
                        }
                        project::Status::NeedsEnable { version, .. } => {
                            if ui.button("Enable").clicked() {
                                match mgr.store.state().and_then(|s| mgr.store.manifest(&s, &r.id, *version)) {
                                    Ok(m) => self.view.setup = Some(Setup::new(mgr.clone(), m)),
                                    Err(e) => self.view.error = Some(e.to_string()),
                                }
                            }
                        }
                        _ => {}
                    }
                    if ui.button("Skip").clicked() {
                        self.skipped.insert(r.id.clone());
                    }
                });
            }
            ui.label("Missing plugins keep their placeholders and saved data.");
            if ui.button("Open without them").clicked() {
                outcome = Some(PromptOutcome { skipped: self.skipped.iter().cloned().collect(), open_without: true });
            }
            if ui.button("Continue").clicked() {
                outcome = Some(PromptOutcome { skipped: self.skipped.iter().cloned().collect(), open_without: false });
            }
        });
        if outcome.is_some() {
            self.closed = true;
        }
        outcome
    }
}

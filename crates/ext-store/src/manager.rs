//! One management facade; project inspection never changes installation or consent.
use crate::{
    catalogue::Catalogue,
    state::Origin,
    store::{Installed, Result, Store, error},
};
use craft_ext_core::{
    catalogue::{self, Index, PackageFile, PackageSummary},
    contract::Provided,
    id::{AppId, ExtensionId},
    manifest::{Compat, Manifest},
    permission::{Permission, warnings},
    project::{self, PluginRef},
    tier::Tier,
    version::{Version, VersionReq},
};
use std::path::Path;
pub struct Manager {
    pub store: Store,
    pub app: Option<AppId>,
    pub provided: Provided,
    pub platform: String,
    pub catalogue_base: String,
}
#[derive(Clone, Debug)]
pub struct PermissionSheet {
    pub required: Vec<(Permission, String)>,
    pub optional: Vec<(Permission, String)>,
    pub warnings: Vec<String>,
    pub why: std::collections::BTreeMap<String, String>,
    pub native: bool,
}
pub type InstallResult = (Installed, Compat, PermissionSheet, Vec<(ExtensionId, Version)>);
pub type Update = (ExtensionId, Version, Version, bool);
impl Manager {
    pub fn new(store: Store, app: Option<AppId>, provided: Provided, platform: String) -> Self {
        Self {
            store,
            app,
            provided,
            platform,
            catalogue_base: std::env::var("CRAFT_CATALOGUE_URL").unwrap_or_else(|_| catalogue::DEFAULT_URL.into()),
        }
    }
    fn cache(&self) -> std::path::PathBuf {
        self.store.root().join("catalogue")
    }
    fn index(&self) -> Result<Index> {
        Catalogue::cached_for(&self.cache(), &self.catalogue_base)
    }
    fn app(&self) -> Result<&AppId> {
        self.app.as_ref().ok_or_else(|| error("choose an app for this operation"))
    }
    /// Resolve only declared resources in the app's enabled, active release.
    pub fn resolve_resource(&self, reference: &str) -> Result<std::path::PathBuf> {
        let rest = reference.strip_prefix("plugin://").ok_or_else(|| error("invalid plugin reference"))?;
        let mut parts = rest.splitn(3, '/');
        let id = ExtensionId::new(parts.next().unwrap_or_default()).map_err(error)?;
        let contribution = parts.next().ok_or_else(|| error("missing contribution"))?;
        let file = parts.next().ok_or_else(|| error("missing resource file"))?;
        let missing = || error(format!("needs plugin {id}: install and enable {contribution}"));
        let state = self.store.state()?;
        let setup = state.apps.get(self.app()?).and_then(|a| a.get(&id)).ok_or_else(missing)?;
        let version = setup.active.ok_or_else(missing)?;
        let record = state.packages.get(&id).and_then(|p| p.releases.get(&version)).ok_or_else(missing)?;
        if !setup.enabled.contains(contribution) || state.blocked(&id, version, &record.sha256, catalogue::Severity::Disable) {
            return Err(missing());
        }
        let manifest = self.store.manifest(&state, &id, version)?;
        let c = manifest.contribution(contribution).ok_or_else(missing)?;
        if !self.compat(&manifest).usable()
            || c.contract.check(&self.provided).is_some()
            || !manifest.contributions_for(self.app()?).any(|entry| entry.id == c.id)
            || !c.files.iter().any(|f| f == file)
        {
            return Err(missing());
        }
        let root = self.store.release_dir(&id, version, &record.sha256).canonicalize().map_err(error)?;
        let path = root.join(file).canonicalize().map_err(|_| missing())?;
        if !path.starts_with(root) || !path.is_file() {
            return Err(error("resource is outside its package"));
        }
        Ok(path)
    }
    pub fn search(&self, query: &str) -> Result<Vec<PackageSummary>> {
        let i = self.index()?;
        Ok(catalogue::search(&i, &catalogue::Query { text: query, app: self.app.as_ref(), include_native: true, ..Default::default() })
            .into_iter()
            .cloned()
            .collect())
    }
    pub fn info(&self, id: &ExtensionId) -> Result<PackageSummary> {
        self.index()?.package(id).cloned().ok_or_else(|| error("this plugin is not listed"))
    }
    fn compat(&self, m: &Manifest) -> Compat {
        if let Some(app) = &self.app {
            if m.native.as_ref().is_some_and(|n| !n.executables.contains_key(&self.platform)) {
                return Compat::Incompatible { reasons: vec!["no native executable for this platform".into()] };
            }
            m.compat(app, &self.provided)
        } else {
            Compat::Compatible
        }
    }
    fn sheet(&self, m: &Manifest) -> PermissionSheet {
        let mut req = std::collections::BTreeSet::new();
        let mut opt = std::collections::BTreeSet::new();
        for c in &m.contributions {
            if self.app.as_ref().is_none_or(|a| c.apps.is_empty() || c.apps.contains(a)) {
                req.extend(c.permissions.clone());
                opt.extend(c.optional_permissions.clone());
            }
        }
        opt.retain(|p| !req.contains(p));
        let all = req.iter().chain(opt.iter()).cloned().collect::<Vec<_>>();
        PermissionSheet {
            required: req
                .into_iter()
                .map(|p| {
                    let s = p.sentence();
                    (p, s)
                })
                .collect(),
            optional: opt
                .into_iter()
                .map(|p| {
                    let s = p.sentence();
                    (p, s)
                })
                .collect(),
            warnings: warnings(&all),
            why: m.why.clone(),
            native: m.is_native(),
        }
    }
    #[cfg(feature = "net")]
    fn package(&self, id: &ExtensionId) -> Result<PackageFile> {
        Catalogue::package_file(&self.catalogue_base, &self.info(id)?)
    }
    #[cfg(not(feature = "net"))]
    fn package(&self, _id: &ExtensionId) -> Result<PackageFile> {
        Err(error("network support is disabled; enable the net feature"))
    }
    pub fn versions(&self, id: &ExtensionId) -> Result<Vec<(Version, Compat)>> {
        let mut versions = std::collections::BTreeMap::new();
        if let Ok(p) = self.package(id) {
            for r in p.releases {
                versions.insert(r.version, self.compat(&r.manifest));
            }
        }
        let s = self.store.state()?;
        if let Some(p) = s.packages.get(id) {
            for v in p.releases.keys() {
                versions.insert(*v, self.compat(&self.store.manifest(&s, id, *v)?));
            }
        }
        Ok(versions.into_iter().rev().collect())
    }
    /// Resolve the complete graph before downloading or changing the store.
    #[cfg(feature = "net")]
    pub fn install_plan(&self, id: &ExtensionId, req: Option<VersionReq>) -> Result<Vec<craft_ext_core::catalogue::Release>> {
        let state = self.store.state()?;
        let index = self.index()?;
        let bytes = serde_json::to_vec(&index).map_err(error)?;
        Index::parse(&bytes, crate::store::now(), state.catalogue_serial).map_err(error)?;
        if crate::store::now() < index.issued {
            return Err(error("the clock is earlier than the catalogue"));
        }
        let mut selected = std::collections::BTreeMap::new();
        let mut visiting = std::collections::BTreeSet::new();
        let mut ordered = Vec::new();
        self.resolve_dependency(id, req.unwrap_or(VersionReq::Any), &index, &mut selected, &mut visiting, &mut ordered)?;
        // Retained packages must continue to have satisfiable requirements too.
        for (other, package) in &state.packages {
            if selected.contains_key(other) {
                continue;
            }
            for version in package.releases.keys() {
                let manifest = self.store.manifest(&state, other, *version)?;
                for (dep, requirement) in manifest.requires {
                    if selected.get(&dep).is_some_and(|v| !requirement.matches(*v)) {
                        return Err(error(format!("conflicting requirements: {other} needs {dep} {requirement}")));
                    }
                }
            }
        }
        Ok(ordered)
    }
    #[cfg(not(feature = "net"))]
    pub fn install_plan(&self, _id: &ExtensionId, _req: Option<VersionReq>) -> Result<Vec<craft_ext_core::catalogue::Release>> {
        Err(error("network support is disabled; enable the net feature"))
    }
    #[cfg(feature = "net")]
    fn resolve_dependency(
        &self,
        id: &ExtensionId,
        req: VersionReq,
        index: &Index,
        selected: &mut std::collections::BTreeMap<ExtensionId, Version>,
        visiting: &mut std::collections::BTreeSet<ExtensionId>,
        ordered: &mut Vec<craft_ext_core::catalogue::Release>,
    ) -> Result<()> {
        if visiting.contains(id) {
            return Err(error(format!("dependency cycle involving {id}")));
        }
        if let Some(version) = selected.get(id) {
            if !req.matches(*version) {
                return Err(error(format!("conflicting requirements for {id}: selected {version}, needs {req}")));
            }
            return Ok(());
        }
        if visiting.len() >= 128 || selected.len() >= 1024 {
            return Err(error("dependency graph is too large"));
        }
        let state = self.store.state()?;
        let pin = self.app.as_ref().and_then(|a| state.apps.get(a)).and_then(|a| a.get(id)).and_then(|a| a.pin);
        let package =
            Catalogue::package_file(&self.catalogue_base, index.package(id).ok_or_else(|| error(format!("required plugin {id} is not listed")))?)?;
        let release = package
            .releases
            .into_iter()
            .filter(|r| {
                req.matches(r.version)
                    && pin.is_none_or(|p| p.matches(r.version))
                    && self.compat(&r.manifest).usable()
                    // Advisories only warn; BlockNew and Disable withdraw the release, as for updates.
                    && r.asset_for(&self.platform).is_some_and(|a| {
                        index.revocation(id, r.version, &a.sha256).is_none_or(|x| x.severity < catalogue::Severity::BlockNew)
                    })
            })
            .max_by_key(|r| r.version)
            .ok_or_else(|| error(format!("no compatible, non-revoked release for {id} {req}; conflicting requirements or pin")))?;
        visiting.insert(id.clone());
        selected.insert(id.clone(), release.version);
        for (dep, requirement) in &release.manifest.requires {
            self.resolve_dependency(dep, *requirement, index, selected, visiting, ordered)?;
        }
        visiting.remove(id);
        ordered.push(release);
        Ok(())
    }
    #[cfg(feature = "net")]
    pub fn install(&self, id: &ExtensionId, req: Option<VersionReq>) -> Result<InstallResult> {
        self.install_with_deps(id, req, true)
    }
    #[cfg(feature = "net")]
    pub fn install_with_deps(&self, id: &ExtensionId, req: Option<VersionReq>, with_deps: bool) -> Result<InstallResult> {
        let plan = self.install_plan(id, req)?;
        let dependencies = plan.iter().filter(|r| &r.manifest.id != id).map(|r| (r.manifest.id.clone(), r.version)).collect::<Vec<_>>();
        if !with_deps && !dependencies.is_empty() {
            return Err(error(format!(
                "required packages: {}",
                dependencies.iter().map(|(id, v)| format!("{id} {v}")).collect::<Vec<_>>().join(", ")
            )));
        }
        let index = self.index()?;
        let mut archives = Vec::new();
        for release in plan {
            let asset = release.asset_for(&self.platform).ok_or_else(|| error("no release for this platform"))?;
            let bytes = Catalogue::download_release_from(asset, 300 * 1024 * 1024, &self.catalogue_base)?;
            if temp_validation(&bytes)? != release.manifest {
                return Err(error("the archive manifest differs from the catalogue"));
            }
            archives.push((release, bytes));
        }
        let mut result = None;
        for (release, bytes) in archives {
            let dep = &release.manifest.id;
            let row = index.package(dep).ok_or_else(|| error("this plugin is not listed"))?;
            let installed =
                self.store.install_bytes(&bytes, index.tier_of(dep), Origin::Catalogue { vendor: row.vendor.clone(), repo: row.repo.clone() })?;
            if dep == id {
                let compat = self.compat(&installed.manifest);
                let sheet = self.sheet(&installed.manifest);
                result = Some((installed, compat, sheet, dependencies.clone()));
            }
        }
        result.ok_or_else(|| error("empty installation plan"))
    }
    #[cfg(not(feature = "net"))]
    pub fn install_with_deps(&self, id: &ExtensionId, req: Option<VersionReq>, _with_deps: bool) -> Result<InstallResult> {
        self.install(id, req)
    }
    #[cfg(not(feature = "net"))]
    pub fn install(&self, _id: &ExtensionId, _req: Option<VersionReq>) -> Result<InstallResult> {
        Err(error("network support is disabled; enable the net feature"))
    }
    pub fn install_file(&self, path: &Path) -> Result<InstallResult> {
        let bytes = crate::state_read(path.into(), 300 * 1024 * 1024)?;
        let installed = self.store.install_bytes(&bytes, Tier::Local, Origin::Local { sha256: crate::store::digest(&bytes) })?;
        let c = self.compat(&installed.manifest);
        let sheet = self.sheet(&installed.manifest);
        Ok((installed, c, sheet, vec![]))
    }
    pub fn update_check(&self) -> Result<(Vec<Update>, Vec<(ExtensionId, String)>)> {
        let s = self.store.state()?;
        let app = self.app()?;
        let mut out = vec![];
        let mut problems = vec![];
        let index = self.index()?;
        if let Some(setups) = s.apps.get(app) {
            for (id, a) in setups {
                if let Some(v) = a.active {
                    if index.package(id).is_none() || s.packages.get(id).is_some_and(|p| matches!(p.origin, Origin::Local { .. })) {
                        continue;
                    }
                    let p = match self.package(id) {
                        Ok(p) => p,
                        Err(e) => {
                            problems.push((id.clone(), e.to_string()));
                            continue;
                        }
                    };
                    let candidates = p.releases.into_iter().filter(|r| r.asset_for(&self.platform).is_some()).collect::<Vec<_>>();
                    if let catalogue::Resolution::Use(r, _) =
                        catalogue::resolve(&candidates, app, &self.provided, a.pin.unwrap_or(VersionReq::Any), |r| {
                            r.asset_for(&self.platform).and_then(|a| index.revocation(id, r.version, &a.sha256)).map(|r| r.severity)
                        })
                    {
                        if r.version > v && r.asset_for(&self.platform).is_some() {
                            let wide =
                                r.manifest.permissions_for(app, |c| a.enabled.contains(c)).0.iter().any(|p| !a.grants.iter().any(|g| g.covers(p)))
                                    || r.manifest.is_native();
                            out.push((id.clone(), v, r.version, wide));
                        }
                    }
                }
            }
        }
        Ok((out, problems))
    }
    pub fn update(&self, id: &ExtensionId) -> Result<InstallResult> {
        let r = self.install(id, None)?;
        self.store.set_active(self.app()?, id, r.0.version)?;
        Ok(r)
    }
    /// Returns warnings when retained contribution data needs a newer state format.
    pub fn rollback(&self, id: &ExtensionId, v: Version) -> Result<Vec<String>> {
        if !self.store.state()?.packages.get(id).is_some_and(|p| p.releases.contains_key(&v)) {
            self.install(id, Some(VersionReq::parse(&format!("={v}")).map_err(error)?))?;
        }
        let s = self.store.state()?;
        let target = self.store.manifest(&s, id, v)?;
        let mut warnings = vec![];
        if let Some(current) = s.apps.get(self.app()?).and_then(|a| a.get(id)).and_then(|a| a.active) {
            let current = self.store.manifest(&s, id, current)?;
            for c in &current.contributions {
                if target.contribution(&c.id).is_none_or(|old| old.state_version < c.state_version) {
                    warnings.push(format!("{} may contain newer saved data; keep that data untouched as a placeholder", c.label));
                }
            }
        }
        self.store.set_active(self.app()?, id, v)?;
        Ok(warnings)
    }
    pub fn pin(&self, id: &ExtensionId, req: VersionReq) -> Result<()> {
        self.store.set_pin(self.app()?, id, Some(req))
    }
    pub fn unpin(&self, id: &ExtensionId) -> Result<()> {
        self.store.set_pin(self.app()?, id, None)
    }
    pub fn enable(&self, id: &ExtensionId, c: &str) -> Result<()> {
        self.store.set_enabled_checked(self.app()?, id, c, true, Some((&self.provided, &self.platform)))
    }
    pub fn disable(&self, id: &ExtensionId, c: &str) -> Result<()> {
        self.store.set_enabled(self.app()?, id, c, false)
    }
    pub fn grant(&self, id: &ExtensionId, p: Permission) -> Result<()> {
        self.store.grant(self.app()?, id, p)
    }
    pub fn revoke(&self, id: &ExtensionId, p: &Permission) -> Result<()> {
        self.store.revoke(self.app()?, id, p)
    }
    pub fn remove(&self, id: &ExtensionId) -> Result<()> {
        self.remove_force(id, false)
    }
    pub fn dependents(&self, id: &ExtensionId) -> Result<Vec<ExtensionId>> {
        let state = self.store.state()?;
        let mut result = Vec::new();
        for (other, package) in &state.packages {
            if other == id {
                continue;
            }
            for version in package.releases.keys() {
                if self.store.manifest(&state, other, *version)?.requires.contains_key(id) {
                    result.push(other.clone());
                    break;
                }
            }
        }
        Ok(result)
    }
    pub fn remove_force(&self, id: &ExtensionId, force: bool) -> Result<()> {
        let dependents = self.dependents(id)?;
        if !force && !dependents.is_empty() {
            return Err(error(format!(
                "{id} is required by {}; use --force to remove",
                dependents.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
            )));
        }
        self.store.uninstall(id)
    }
    #[cfg(feature = "net")]
    pub fn refresh(&self, now: u64) -> Result<()> {
        let (i, bytes, sig) = Catalogue::fetch_verified(&self.catalogue_base, now, self.store.state()?.catalogue_serial)?;
        let cache = self.cache();
        // Under the store lock and after the serial check: a slower, older refresh can never
        // replace a newer cache, and the cache matches the published serial and kill list.
        self.store.publish_catalogue_with(i.serial, now, i.revocations, || crate::catalogue::write_cache(&cache, &bytes, &sig))
    }
    #[cfg(not(feature = "net"))]
    pub fn refresh(&self, _now: u64) -> Result<()> {
        Err(error("network support is disabled; enable the net feature"))
    }
    pub fn check_project(&self, refs: &[PluginRef]) -> Result<Vec<(PluginRef, project::Status)>> {
        let s = self.store.state()?;
        let index = self.index().ok();
        Ok(refs
            .iter()
            .map(|r| {
                let setup = self.app.as_ref().and_then(|a| s.apps.get(a)).and_then(|a| a.get(&r.id));
                let status = if let Some(p) = s.packages.get(&r.id) {
                    if let Some(rec) = p.releases.get(&r.version) {
                        if r.sha256.as_ref().is_some_and(|sha| sha != &rec.sha256) {
                            project::Status::NeedsOtherVersion { installed: r.version, wanted: r.version, reason: "installed digest differs".into() }
                        } else if s.blocked(&r.id, r.version, &rec.sha256, catalogue::Severity::Disable) {
                            project::Status::Unavailable { reason: "release withdrawn or digest differs".into() }
                        } else if let Some(reason) = self.store.manifest(&s, &r.id, r.version).ok().and_then(|m| {
                            if !self.compat(&m).usable() {
                                return Some("release is incompatible with this app".to_string());
                            }
                            for id in r.contributions.iter().chain(r.state_versions.iter().flat_map(|v| v.keys())) {
                                let Some(c) = m.contributions.iter().find(|c| &c.id == id) else {
                                    return Some(format!("contribution {id} is missing"));
                                };
                                if !self.app.as_ref().is_some_and(|app| m.contributions_for(app).any(|x| x.id == c.id))
                                    || c.contract.check(&self.provided).is_some()
                                    || self.provided.unsupported.iter().any(|(kind, _)| *kind == c.kind)
                                {
                                    return Some(format!("contribution {id} is incompatible with this app"));
                                }
                                if r.state_versions.as_ref().and_then(|v| v.get(id)).is_some_and(|v| c.state_version < *v) {
                                    return Some(format!("contribution {id} cannot load the saved state version"));
                                }
                            }
                            None
                        }) {
                            project::Status::NeedsOtherVersion { installed: r.version, wanted: r.version, reason }
                        } else if setup.is_some_and(|a| {
                            a.active == Some(r.version)
                                && !a.enabled.is_empty()
                                && r.contributions.iter().chain(r.state_versions.iter().flat_map(|v| v.keys())).all(|c| a.enabled.contains(c))
                                && self.store.manifest(&s, &r.id, r.version).is_ok_and(|m| {
                                    (!m.is_native() || a.full_access)
                                        && self.app.as_ref().is_some_and(|app| {
                                            m.permissions_for(app, |c| a.enabled.contains(c)).0.iter().all(|p| a.grants.iter().any(|g| g.covers(p)))
                                        })
                                })
                        }) && self.store.manifest(&s, &r.id, r.version).is_ok_and(|m| self.compat(&m).usable())
                        {
                            project::Status::Ready { version: r.version }
                        } else {
                            project::Status::NeedsEnable {
                                version: r.version,
                                reason: "required contributions are off or permissions are missing".into(),
                            }
                        }
                    } else if let Some(v) = p.releases.keys().next_back() {
                        project::Status::NeedsOtherVersion { installed: *v, wanted: r.version, reason: "requested version is not installed".into() }
                    } else {
                        project::Status::Unavailable { reason: "no installed releases".into() }
                    }
                } else if index.as_ref().is_some_and(|i| i.package(&r.id).is_some()) {
                    match self.package(&r.id) {
                        Ok(p) => match p.releases.iter().find(|x| x.version == r.version) {
                            None => project::Status::Unavailable { reason: "version not listed".into() },
                            Some(release) => match release.asset_for(&self.platform) {
                                None => project::Status::Unavailable { reason: "wrong platform".into() },
                                Some(a) if r.sha256.as_ref().is_some_and(|sha| sha != &a.sha256) => {
                                    project::Status::Unavailable { reason: "digest not listed".into() }
                                }
                                Some(a)
                                    if index
                                        .as_ref()
                                        .and_then(|i| i.revocation(&r.id, r.version, &a.sha256))
                                        .is_some_and(|x| x.severity >= catalogue::Severity::BlockNew) =>
                                {
                                    project::Status::Unavailable { reason: "release withdrawn".into() }
                                }
                                Some(_) => project::Status::Downloadable,
                            },
                        },
                        Err(e) => project::Status::Unavailable { reason: e.to_string() },
                    }
                } else {
                    project::Status::Unavailable { reason: "this plugin is not listed".into() }
                };
                (r.clone(), status)
            })
            .collect())
    }
    pub fn report_url(&self, id: &ExtensionId, v: Version) -> String {
        format!("https://github.com/arnav-hiwarkar/craft-plugins/issues/new?template=report.yml&plugin={id}&version={v}")
    }
}

#[cfg(feature = "net")]
fn temp_validation(bytes: &[u8]) -> Result<Manifest> {
    use std::io::Read;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(error)?;
    let mut b = vec![];
    zip.by_name(craft_ext_core::manifest::MANIFEST_NAME)
        .map_err(error)?
        .take(craft_ext_core::manifest::MAX_MANIFEST_BYTES as u64 + 1)
        .read_to_end(&mut b)
        .map_err(error)?;
    Manifest::parse(&b).map_err(error)
}

#[cfg(all(test, feature = "net"))]
mod dependency_tests {
    use super::*;
    #[test]
    fn plans_dependencies_and_refuses_conflicts_cycles_and_revocations() {
        let temp = tempfile::tempdir().unwrap();
        let film = ExtensionId::new("io.github.arnav-hiwarkar.film-looks").unwrap();
        let half = ExtensionId::new("io.github.arnav-hiwarkar.halftone").unwrap();
        std::fs::create_dir(temp.path().join("packages")).unwrap();
        let mut packages = Vec::new();
        let mut index = Index { schema: 1, serial: 1, issued: 0, expires: u64::MAX, vendors: vec![], packages: vec![], revocations: vec![] };
        for text in [include_str!("../../../examples/film-looks/craft-plugin.toml"), include_str!("../../../examples/halftone/craft-plugin.toml")] {
            let manifest = Manifest::parse(text.as_bytes()).unwrap();
            let id = manifest.id.clone();
            let release = craft_ext_core::catalogue::Release {
                version: manifest.version,
                tag: format!("v{}", manifest.version),
                commit: "a".repeat(40),
                published: "2026-11-13".into(),
                assets: vec![craft_ext_core::catalogue::Asset {
                    platform: None,
                    url: format!(
                        "https://github.com/arnav-hiwarkar/{}/releases/download/v{}/test.craftplugin",
                        if id == film { "film-looks" } else { "halftone" },
                        manifest.version
                    ),
                    sha256: "a".repeat(64),
                    size: 1,
                }],
                manifest: manifest.clone(),
            };
            let repo = if id == film { "film-looks" } else { "halftone" };
            packages.push(
                serde_json::from_value::<PackageFile>(
                    serde_json::json!({"schema":1,"id":id,"vendor":"arnav-hiwarkar","repo":repo,"releases":[release]}),
                )
                .unwrap(),
            );
            index.packages.push(serde_json::from_value(serde_json::json!({"id":id,"name":manifest.name,"summary":manifest.summary,"vendor":"arnav-hiwarkar","repo":repo,"kinds":[],"apps":manifest.apps,"native":false,"license":manifest.license,"latest":manifest.version,"updated":"2026-11-13","file_sha256":"","file_size":0})).unwrap());
        }
        let mut manager = Manager::new(
            Store::at(temp.path().join("store")),
            Some(AppId::new("effectcraft").unwrap()),
            Provided::new(&[("raster.filter", 1, 0), ("resource.lut", 1, 0), ("resource.preset", 1, 0)]),
            "linux-x86_64".into(),
        );
        manager.catalogue_base = format!("file://{}", temp.path().display());
        let publish = |index: &mut Index, packages: &[PackageFile]| {
            for p in packages {
                let bytes = serde_json::to_vec(p).unwrap();
                let row = index.packages.iter_mut().find(|r| r.id == p.id).unwrap();
                row.file_size = bytes.len() as u64;
                row.file_sha256 = crate::store::digest(&bytes);
                std::fs::write(temp.path().join(format!("packages/{}.json", p.id)), bytes).unwrap();
            }
        };
        let resolve = |index: &Index| {
            let mut selected = std::collections::BTreeMap::new();
            let mut visiting = std::collections::BTreeSet::new();
            let mut ordered = Vec::new();
            manager.resolve_dependency(&film, VersionReq::Any, index, &mut selected, &mut visiting, &mut ordered).map(|_| ordered)
        };
        publish(&mut index, &packages);
        let plan = resolve(&index).unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].manifest.id, half);
        assert_eq!(plan[0].version, Version::new(1, 0, 0));
        index.revocations.push(craft_ext_core::catalogue::Revocation {
            id: Some(half.clone()),
            versions: None,
            sha256: None,
            vendor: None,
            severity: catalogue::Severity::BlockNew,
            reason: "test".into(),
            date: "2026-11-13".into(),
        });
        assert!(resolve(&index).is_err());
        // An advisory warns but never blocks installation.
        index.revocations[0].severity = catalogue::Severity::Advisory;
        assert_eq!(resolve(&index).unwrap().len(), 2);
        index.revocations.clear();
        for release in &mut packages[1].releases {
            release.manifest.requires.insert(film.clone(), VersionReq::Any);
        }
        publish(&mut index, &packages);
        assert!(resolve(&index).unwrap_err().to_string().contains("cycle"));
        for release in &mut packages[1].releases {
            release.manifest.requires.clear();
        }
        for release in &mut packages[0].releases {
            release.manifest.requires.insert(half.clone(), VersionReq::Exact(Version::new(2, 0, 0)));
        }
        publish(&mut index, &packages);
        assert!(resolve(&index).unwrap_err().to_string().contains("conflicting"));
    }
}

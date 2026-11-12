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
pub type InstallResult = (Installed, Compat, PermissionSheet);
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
        Catalogue::cached(&self.cache())
    }
    fn app(&self) -> Result<&AppId> {
        self.app.as_ref().ok_or_else(|| error("choose an app for this operation"))
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
    #[cfg(feature = "net")]
    pub fn install(&self, id: &ExtensionId, req: Option<VersionReq>) -> Result<InstallResult> {
        let s = self.store.state()?;
        let b = serde_json::to_vec(&self.index()?).map_err(error)?;
        let index = Index::parse(&b, crate::store::now(), s.catalogue_serial).map_err(error)?;
        if crate::store::now() < index.issued {
            return Err(error("the clock is earlier than the catalogue"));
        }
        let row = index.package(id).ok_or_else(|| error("this plugin is not listed"))?;
        let p = Catalogue::package_file(&self.catalogue_base, row)?;
        let pin = self.app.as_ref().and_then(|a| s.apps.get(a)).and_then(|a| a.get(id)).and_then(|a| a.pin);
        let candidates = p.releases.into_iter().filter(|r| r.asset_for(&self.platform).is_some()).collect::<Vec<_>>();
        let release = if let Some(app) = &self.app {
            match catalogue::resolve(&candidates, app, &self.provided, req.or(pin).unwrap_or(VersionReq::Any), |r| {
                r.asset_for(&self.platform).and_then(|a| index.revocation(id, r.version, &a.sha256)).map(|r| r.severity)
            }) {
                catalogue::Resolution::Use(r, _) => r,
                catalogue::Resolution::None(why) => {
                    let detail: Vec<String> = why.iter().take(5).map(|(v, r)| format!("{v}: {}", r.join("; "))).collect();
                    return Err(error(if detail.is_empty() {
                        "no release matches this app, platform and pin".to_string()
                    } else {
                        format!("no listed version works in this app — {}", detail.join(" | "))
                    }));
                }
            }
        } else {
            candidates
                .iter()
                .filter(|r| req.unwrap_or(VersionReq::Any).matches(r.version))
                .max_by_key(|r| r.version)
                .ok_or_else(|| error("no matching release"))?
        };
        let asset = release.asset_for(&self.platform).ok_or_else(|| error("no release for this platform"))?;
        let bytes = Catalogue::download_release(asset, 300 * 1024 * 1024)?;
        let stage = temp_validation(&bytes)?;
        if stage != release.manifest {
            return Err(error("the archive manifest differs from the catalogue"));
        }
        let installed =
            self.store.install_bytes(&bytes, index.tier_of(id), Origin::Catalogue { vendor: row.vendor.clone(), repo: row.repo.clone() })?;
        if installed.manifest != release.manifest {
            return Err(error("the archive manifest differs from the catalogue"));
        }
        let compat = self.compat(&installed.manifest);
        let sheet = self.sheet(&installed.manifest);
        Ok((installed, compat, sheet))
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
        Ok((installed, c, sheet))
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
                    if index.package(id).is_none() || s.packages.get(id).is_some_and(|p| matches!(p.origin, Origin::Local { .. })) { continue; }
                    let p = match self.package(id) { Ok(p) => p, Err(e) => { problems.push((id.clone(), e.to_string())); continue; } };
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
        self.store.uninstall(id)
    }
    #[cfg(feature = "net")]
    pub fn refresh(&self, now: u64) -> Result<()> {
        let i = Catalogue::fetch(&self.catalogue_base, &self.cache(), now, self.store.state()?.catalogue_serial)?;
        self.store.publish_catalogue(i.serial, now, i.revocations)
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
                            project::Status::NeedsOtherVersion { installed: r.version, wanted: r.version }
                        } else if s.blocked(&r.id, r.version, &rec.sha256, catalogue::Severity::Disable) {
                            project::Status::Unavailable { reason: "release withdrawn or digest differs".into() }
                        } else if setup.is_some_and(|a| {
                            a.active == Some(r.version)
                                && !a.enabled.is_empty()
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
                            project::Status::NeedsEnable { version: r.version }
                        }
                    } else if let Some(v) = p.releases.keys().next_back() {
                        project::Status::NeedsOtherVersion { installed: *v, wanted: r.version }
                    } else {
                        project::Status::Unavailable { reason: "no installed releases".into() }
                    }
                } else if index.as_ref().is_some_and(|i| i.package(&r.id).is_some()) {
                    match self.package(&r.id) {
                        Ok(p) => match p.releases.iter().find(|x| x.version == r.version) {
                            None => project::Status::Unavailable { reason: "version not listed".into() },
                            Some(release) => match release.asset_for(&self.platform) {
                                None => project::Status::Unavailable { reason: "wrong platform".into() },
                                Some(a) if r.sha256.as_ref().is_some_and(|sha| sha != &a.sha256) => project::Status::Unavailable { reason: "digest not listed".into() },
                                Some(a) if index.as_ref().and_then(|i| i.revocation(&r.id, r.version, &a.sha256)).is_some_and(|x| x.severity >= catalogue::Severity::BlockNew) => project::Status::Unavailable { reason: "release withdrawn".into() },
                                Some(_) => project::Status::Downloadable,
                            }
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

//! All transitions serialize under the cross-process lock and atomically publish generation+1.
use crate::{
    state::*,
    unpack::{Caps, unpack_into},
};
use craft_ext_core::{
    catalogue::{Revocation, Severity},
    id::{AppId, ExtensionId},
    manifest::{MANIFEST_NAME, Manifest},
    permission::Permission,
    tier::Tier,
    version::{Version, VersionReq},
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, TryLockError},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
#[derive(Debug)]
pub struct StoreError(pub String);
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for StoreError {}
pub type Result<T> = std::result::Result<T, StoreError>;
pub(crate) fn error(e: impl std::fmt::Display) -> StoreError {
    StoreError(e.to_string())
}
pub fn digest(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
const LOCK_WAIT: Duration = Duration::from_secs(5);
/// Holds the cross-process lock for as long as it lives.
struct Lock(File);

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

#[derive(Clone, Debug)]
pub struct Store {
    root: PathBuf,
}
#[derive(Clone, Debug)]
pub struct Installed {
    pub id: ExtensionId,
    pub version: Version,
    pub sha256: String,
    pub dir: PathBuf,
    pub manifest: Manifest,
}
impl Store {
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    fn state_path(&self) -> PathBuf {
        self.root.join(STATE_FILE)
    }
    pub fn state(&self) -> Result<StoreState> {
        StoreState::read(&self.state_path())
    }
    fn lock(&self) -> Result<Lock> {
        fs::create_dir_all(&self.root).map_err(|_| error("the store files could not be saved"))?;
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(self.root.join(".lock"))
            .map_err(|_| error("the store files could not be saved"))?;
        let started = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Lock(file)),
                Err(TryLockError::WouldBlock) if started.elapsed() < LOCK_WAIT => std::thread::sleep(Duration::from_millis(20)),
                Err(TryLockError::WouldBlock) => return Err(error("another app is changing plugins; try again")),
                Err(TryLockError::Error(_)) => return Err(error("the store files could not be saved")),
            }
        }
    }

    fn update<T>(&self, f: impl FnOnce(&mut StoreState) -> Result<T>) -> Result<T> {
        self.update_cleanup(|s| f(s).map(|out| (out, Vec::new())))
    }
    // Publish removals before deleting bytes: a failed state write must retain every recorded release.
    fn update_cleanup<T>(&self, f: impl FnOnce(&mut StoreState) -> Result<(T, Vec<PathBuf>)>) -> Result<T> {
        let _lock = self.lock()?;
        let mut s = self.state()?;
        let (out, remove) = f(&mut s)?;
        s.generation = s.generation.checked_add(1).ok_or_else(|| error("store generation exhausted"))?;
        let b = serde_json::to_vec(&s).map_err(error)?;
        if b.len() > MAX_STATE_BYTES {
            return Err(error("the store is too large"));
        }
        craft_atomic::atomic_write(&self.state_path(), &b).map_err(error)?;
        for dir in remove {
            if dir.exists() {
                fs::remove_dir_all(dir).map_err(error)?
            }
        }
        Ok(out)
    }
    pub fn release_dir(&self, id: &ExtensionId, v: Version, sha: &str) -> PathBuf {
        self.root.join("releases").join(id.as_str()).join(format!("{v}-{}", sha.get(..12).unwrap_or(sha)))
    }
    pub fn manifest(&self, s: &StoreState, id: &ExtensionId, v: Version) -> Result<Manifest> {
        let r = s.packages.get(id).and_then(|p| p.releases.get(&v)).ok_or_else(|| error("that release is not installed"))?;
        Manifest::parse(&fs::read(self.release_dir(id, v, &r.sha256).join(MANIFEST_NAME)).map_err(error)?).map_err(error)
    }
    pub fn install_bytes(&self, b: &[u8], tier: Tier, origin: Origin) -> Result<Installed> {
        let sha = digest(b);
        if let Origin::Local { sha256 } = &origin {
            if sha256 != &sha {
                return Err(error("local origin digest does not match"));
            }
        }
        let stage = self.root.join("staging").join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).map_err(error)?.as_nanos()
        ));
        let result = (|| {
            let m = unpack_into(b, &stage, Caps::default()).map_err(error)?;
            craft_ext_core::tier::check_id(tier, &m.id).map_err(error)?;
            let dir = self.release_dir(&m.id, m.version, &sha);
            let mut moved = false;
            let result = self.update(|s| {
                if s.revocations.iter().any(|r| r.severity >= Severity::BlockNew && r.matches(&m.id, m.version, &sha, origin.vendor())) {
                    return Err(error("this release has been withdrawn"));
                }
                if let Some(p) = s.packages.get(&m.id) {
                    let same_origin = p.origin == origin;
                    if !same_origin {
                        for apps in s.apps.values_mut() {
                            if let Some(a) = apps.get_mut(&m.id) {
                                a.grants.clear(); a.full_access = false; a.enabled.clear(); a.active = None; a.pending = None;
                                a.grant_generation += 1;
                            }
                        }
                    }
                    if let Some(r) = p.releases.get(&m.version) {
                        if r.sha256 != sha {
                            return Err(error("this version already has different bytes"));
                        }
                        let p = s.packages.get_mut(&m.id).unwrap();
                        let release = p.releases.get_mut(&m.version).unwrap();
                        release.origin = Some(origin.clone());
                        release.tier = tier;
                        p.origin = origin.clone();
                        return Ok(());
                    }
                }
                fs::create_dir_all(dir.parent().ok_or_else(|| error("invalid release directory"))?).map_err(error)?;
                fs::rename(&stage, &dir).map_err(error)?;
                moved = true;
                s.packages
                    .entry(m.id.clone())
                    .or_insert_with(|| PackageRec { origin: origin.clone(), releases: Default::default() })
                    .releases
                    .insert(m.version, ReleaseRec { sha256: sha.clone(), tier, installed: now(), origin: Some(origin.clone()) });
                s.packages.get_mut(&m.id).unwrap().origin = origin.clone();
                Ok(())
            });
            if result.is_err() && moved {
                let _ = fs::remove_dir_all(&dir);
            }
            result?;
            Ok(Installed { id: m.id.clone(), version: m.version, sha256: sha, dir, manifest: m })
        })();
        let _ = fs::remove_dir_all(stage);
        result
    }
    pub fn private_dir(&self, app: &AppId, id: &ExtensionId) -> PathBuf {
        self.root.join("data").join(app.as_str()).join(id.as_str())
    }
    pub fn set_active(&self, app: &AppId, id: &ExtensionId, v: Version) -> Result<()> {
        self.update(|s| {
            let m = self.manifest(s, id, v)?;
            let r = &s.packages[id].releases[&v];
            if s.blocked(id, v, &r.sha256, Severity::BlockNew) {
                return Err(error("this release has been withdrawn"));
            }
            if !m.apps.contains(app) {
                return Err(error("this plugin does not support this app"));
            }
            let p = &s.packages[id];
            let release_origin = p.releases[&v].origin.clone().unwrap_or_else(|| p.origin.clone());
            if p.origin != release_origin && !matches!((&p.origin, &release_origin), (Origin::Local { .. }, Origin::Local { .. })) {
                for apps in s.apps.values_mut() {
                    if let Some(a) = apps.get_mut(id) {
                        a.grants.clear(); a.full_access = false; a.enabled.clear(); a.active = None; a.pending = None; a.grant_generation += 1;
                    }
                }
            }
            s.packages.get_mut(id).unwrap().origin = release_origin;
            let setup = s.apps.entry(app.clone()).or_default().entry(id.clone()).or_default();
            let widened = m.permissions_for(app, |c| setup.enabled.contains(c)).0.iter().any(|p| !setup.grants.iter().any(|g| g.covers(p)));
            if widened || (m.is_native() && (setup.active != Some(v) || !setup.full_access)) {
                setup.pending = Some(v)
            } else {
                setup.active = Some(v);
                setup.pending = None;
            }
            Ok(())
        })
    }
    pub fn accept_pending(&self, app: &AppId, id: &ExtensionId) -> Result<()> {
        self.update(|s| {
            let a = s.apps.get(app).and_then(|a| a.get(id)).ok_or_else(|| error("there is no pending release"))?;
            let v = a.pending.ok_or_else(|| error("there is no pending release"))?;
            let m = self.manifest(s, id, v)?;
            if s.blocked(id, v, &s.packages[id].releases[&v].sha256, Severity::BlockNew) {
                return Err(error("this release has been withdrawn"));
            }
            if (m.is_native() && !a.full_access)
                || m.permissions_for(app, |c| a.enabled.contains(c)).0.iter().any(|p| !a.grants.iter().any(|g| g.covers(p)))
            {
                return Err(error("grant the required access before accepting this release"));
            }
            let a = s.apps.entry(app.clone()).or_default().entry(id.clone()).or_default();
            a.active = Some(v);
            a.pending = None;
            Ok(())
        })
    }

    pub fn set_pin(&self, app: &AppId, id: &ExtensionId, pin: Option<VersionReq>) -> Result<()> {
        self.setup(app, id, |a| {
            a.pin = pin;
            Ok(())
        })
    }
    fn setup(&self, app: &AppId, id: &ExtensionId, f: impl FnOnce(&mut AppSetup) -> Result<()>) -> Result<()> {
        self.update(|s| {
            if !s.packages.contains_key(id) {
                return Err(error("that plugin is not installed"));
            }
            f(s.apps.entry(app.clone()).or_default().entry(id.clone()).or_default())
        })
    }
    pub fn set_enabled(&self, app: &AppId, id: &ExtensionId, c: &str, on: bool) -> Result<()> {
        self.set_enabled_checked(app, id, c, on, None)
    }
    pub fn set_enabled_checked(&self, app: &AppId, id: &ExtensionId, c: &str, on: bool, context: Option<(&craft_ext_core::contract::Provided, &str)>) -> Result<()> {
        self.update(|s| {
            let a = s.apps.get(app).and_then(|a| a.get(id)).ok_or_else(|| error("choose an active release first"))?;
            let v = a.active.ok_or_else(|| error("choose an active release first"))?;
            let m = self.manifest(s, id, v)?;
            let contribution = m.contributions_for(app).find(|x| x.id == c).ok_or_else(|| error("this contribution is unavailable"))?;
            if on {
                if let Some((provided, platform)) = context {
                    if let Some(reason) = contribution.contract.check(provided) { return Err(error(reason)); }
                    if let craft_ext_core::manifest::Compat::Incompatible { reasons } = m.compat(app, provided) { return Err(error(reasons.join("; "))); }
                    if m.native.as_ref().is_some_and(|n| !n.executables.contains_key(platform)) { return Err(error("no native executable for this platform")); }
                }
            }
            if on
                && (s.blocked(id, v, &s.packages[id].releases[&v].sha256, Severity::BlockNew)
                    || contribution.permissions.iter().any(|p| !a.grants.iter().any(|g| g.covers(p)))
                    || (m.is_native() && !a.full_access))
            {
                return Err(error("grant the required access before enabling this contribution"));
            }
            let a = s.apps.entry(app.clone()).or_default().entry(id.clone()).or_default();
            if on {
                a.enabled.insert(c.into());
            } else {
                a.enabled.remove(c);
            }
            Ok(())
        })
    }
    pub fn grant(&self, app: &AppId, id: &ExtensionId, p: Permission) -> Result<()> {
        self.setup(app, id, |a| {
            a.grants.insert(p);
            a.grant_generation += 1;
            Ok(())
        })
    }
    pub fn revoke(&self, app: &AppId, id: &ExtensionId, p: &Permission) -> Result<()> {
        self.setup(app, id, |a| {
            a.grants.remove(p);
            a.grant_generation += 1;
            Ok(())
        })
    }
    pub fn set_full_access(&self, app: &AppId, id: &ExtensionId, on: bool) -> Result<()> {
        self.setup(app, id, |a| {
            a.full_access = on;
            a.grant_generation += 1;
            Ok(())
        })
    }
    pub fn uninstall(&self, id: &ExtensionId) -> Result<()> {
        self.update_cleanup(|s| {
            s.packages.remove(id);
            let mut remove = vec![self.root.join("releases").join(id.as_str())];
            for (app, a) in &mut s.apps {
                a.remove(id);
                remove.push(self.private_dir(app, id));
            }
            let data = self.root.join("data");
            if data.exists() {
                for entry in fs::read_dir(data).map_err(error)? {
                    let entry = entry.map_err(error)?;
                    if entry.file_type().map_err(error)?.is_dir() {
                        remove.push(entry.path().join(id.as_str()));
                    }
                }
            }
            Ok(((), remove))
        })
    }
    pub fn remove_unused_versions(&self) -> Result<()> {
        self.update_cleanup(|s| {
            let mut remove = vec![];
            for (id, p) in &mut s.packages {
                p.releases.retain(|v, r| {
                    let keep = s
                        .apps
                        .values()
                        .filter_map(|a| a.get(id))
                        .any(|a| a.active == Some(*v) || a.pending == Some(*v) || a.pin.is_some_and(|p| p.matches(*v)));
                    if !keep {
                        remove.push(self.release_dir(id, *v, &r.sha256));
                    }
                    keep
                });
            }
            Ok(((), remove))
        })
    }
    pub fn apply_revocations(&self, list: Vec<Revocation>) -> Result<Vec<(ExtensionId, Version)>> {
        self.update(|s| {
            s.revocations = list;
            Ok(s.packages
                .iter()
                .flat_map(|(id, p)| {
                    p.releases.iter().filter(|(v, r)| s.blocked(id, **v, &r.sha256, Severity::BlockNew)).map(|(v, _)| (id.clone(), *v))
                })
                .collect())
        })
    }
    pub fn publish_catalogue(&self, serial: u64, now: u64, revocations: Vec<Revocation>) -> Result<()> {
        self.update(|s| {
            if serial < s.catalogue_serial { return Err(error("the catalogue is older than one already seen")); }
            s.revocations = revocations;
            s.catalogue_serial = serial;
            s.last_refresh = now;
            Ok(())
        })
    }
    pub fn record_catalogue(&self, serial: u64, now: u64) -> Result<()> {
        self.update(|s| {
            if serial < s.catalogue_serial {
                return Err(error("the catalogue is older than one already seen"));
            }
            s.catalogue_serial = serial;
            s.last_refresh = now;
            Ok(())
        })
    }
}

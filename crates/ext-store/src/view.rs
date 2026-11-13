//! Cheap app snapshots; metadata probes are throttled, permission checks use coverage.
use crate::{
    state::*,
    store::{Result, Store, error},
};
use craft_ext_core::{
    catalogue::Severity,
    id::{AppId, ExtensionId},
    manifest::Manifest,
    permission::Permission,
    version::Version,
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};
pub type EnabledRelease = (ExtensionId, Version, PathBuf, Manifest, BTreeSet<String>);
pub struct GrantView {
    store: Store,
    app: AppId,
    state: StoreState,
    stamp: Option<(SystemTime, u64)>,
    checked: Option<Instant>,
}
impl GrantView {
    pub fn open(root: impl AsRef<Path>, app: AppId) -> Result<Self> {
        let store = Store::at(root.as_ref());
        let state = store.state()?;
        Ok(Self { store, app, state, stamp: None, checked: None })
    }
    pub fn refresh_if_changed(&mut self) -> Result<bool> {
        if self.checked.is_some_and(|t| t.elapsed() < Duration::from_millis(250)) {
            return Ok(false);
        }
        self.checked = Some(Instant::now());
        let stamp = match std::fs::metadata(self.store.root().join(STATE_FILE)) {
            Ok(m) => Some((m.modified().map_err(error)?, m.len())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(error(e)),
        };
        if stamp == self.stamp {
            return Ok(false);
        }
        let state = self.store.state()?;
        let before = self.state.app_fingerprint(&self.app);
        self.state = state;
        self.stamp = stamp;
        // Grants are always fresh; report a change only when this app's loaded plugins are affected.
        Ok(self.state.app_fingerprint(&self.app) != before)
    }
    pub fn allowed(&self, id: &ExtensionId, p: &Permission) -> bool {
        self.state.apps.get(&self.app).and_then(|a| a.get(id)).is_some_and(|a| a.grants.iter().any(|g| g.covers(p)))
    }
    pub fn generation(&self, id: &ExtensionId) -> u64 {
        self.state.apps.get(&self.app).and_then(|a| a.get(id)).map_or(0, |a| a.grant_generation)
    }
    pub fn is_disabled(&self, id: &ExtensionId, v: Version, sha: &str) -> bool {
        self.state.blocked(id, v, sha, Severity::Disable)
    }
    /// This app's setup of `id` (active release, enabled contributions, grants, consent).
    pub fn setup(&self, id: &ExtensionId) -> Option<&AppSetup> {
        self.state.apps.get(&self.app).and_then(|a| a.get(id))
    }
    /// Native helper consent of `id` in this app.
    pub fn full_access(&self, id: &ExtensionId) -> bool {
        self.setup(id).is_some_and(|a| a.full_access)
    }
    /// Whether release `v` of `id` must not run in this app: kill-listed, no longer the active
    /// release, or nothing of it enabled. Unknown packages are stopped.
    pub fn stopped(&self, id: &ExtensionId, v: Version) -> bool {
        let Some(a) = self.setup(id) else { return true };
        let Some(r) = self.state.packages.get(id).and_then(|p| p.releases.get(&v)) else { return true };
        a.active != Some(v) || a.enabled.is_empty() || self.is_disabled(id, v, &r.sha256)
    }
    pub fn enabled_contributions(&self, app: &AppId) -> Result<Vec<EnabledRelease>> {
        let mut out = vec![];
        if let Some(setups) = self.state.apps.get(app) {
            for (id, a) in setups {
                if let Some(v) = a.active {
                    let r = self.state.packages.get(id).and_then(|p| p.releases.get(&v)).ok_or_else(|| error("active release is missing"))?;
                    if self.is_disabled(id, v, &r.sha256) {
                        continue;
                    }
                    let m = self.store.manifest(&self.state, id, v)?;
                    let enabled = m
                        .contributions_for(app)
                        .filter(|c| {
                            a.enabled.contains(&c.id)
                                && c.permissions.iter().all(|p| a.grants.iter().any(|g| g.covers(p)))
                                && (!m.is_native() || a.full_access)
                        })
                        .map(|c| c.id.clone())
                        .collect();
                    out.push((id.clone(), v, self.store.release_dir(id, v, &r.sha256), m, enabled));
                }
            }
        }
        Ok(out)
    }
}

//! Schema two is isolated from legacy binaries; reads and writes are bounded.
use crate::store::{Result, error};
use craft_ext_core::{
    catalogue::{Revocation, Severity},
    id::{AppId, ExtensionId},
    permission::Permission,
    tier::Tier,
    version::{Version, VersionReq},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};
pub const STATE_FILE: &str = "store-v2.json";
pub const MAX_STATE_BYTES: usize = 16 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    Catalogue { vendor: String, repo: String },
    Local { sha256: String },
    Dev,
}
impl Origin {
    pub fn vendor(&self) -> Option<&str> {
        if let Self::Catalogue { vendor, .. } = self { Some(vendor) } else { None }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReleaseRec {
    #[serde(default)]
    pub origin: Option<Origin>,
    pub sha256: String,
    pub tier: Tier,
    pub installed: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageRec {
    pub origin: Origin,
    pub releases: BTreeMap<Version, ReleaseRec>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppSetup {
    pub active: Option<Version>,
    pub pending: Option<Version>,
    pub pin: Option<VersionReq>,
    pub enabled: BTreeSet<String>,
    pub grants: BTreeSet<Permission>,
    pub full_access: bool,
    pub grant_generation: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoreState {
    pub schema: u32,
    pub generation: u64,
    pub packages: BTreeMap<ExtensionId, PackageRec>,
    pub apps: BTreeMap<AppId, BTreeMap<ExtensionId, AppSetup>>,
    pub catalogue_serial: u64,
    pub last_refresh: u64,
    pub revocations: Vec<Revocation>,
}
impl Default for StoreState {
    fn default() -> Self {
        Self { schema: 2, generation: 0, packages: BTreeMap::new(), apps: BTreeMap::new(), catalogue_serial: 0, last_refresh: 0, revocations: vec![] }
    }
}
impl StoreState {
    pub fn read(path: &Path) -> Result<Self> {
        let f = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(error(e)),
        };
        let mut b = vec![];
        f.take(MAX_STATE_BYTES as u64 + 1).read_to_end(&mut b).map_err(error)?;
        if b.len() > MAX_STATE_BYTES {
            return Err(error("the store is too large"));
        }
        let v: serde_json::Value = serde_json::from_slice(&b).map_err(error)?;
        if v["schema"] != 2 {
            return Err(error("the store format is unsupported; update the app"));
        }
        serde_json::from_value(v).map_err(error)
    }
    /// Changes only when something that decides which plugin code `app` runs changes: its setups
    /// (active/pending versions, enabled contributions, grants, consent, grant generation, pins),
    /// the bytes of each active release and whether the kill list disables it. A catalogue refresh,
    /// an unrelated install or another app's setup leave it unchanged, so apps reload plugins only
    /// when they must (the global `generation` moves on every store write).
    pub fn app_fingerprint(&self, app: &AppId) -> u64 {
        let rows: Vec<_> = self
            .apps
            .get(app)
            .into_iter()
            .flatten()
            .map(|(id, a)| {
                let release = a.active.and_then(|v| self.packages.get(id).and_then(|p| p.releases.get(&v)).map(|r| (v, r)));
                let active = release.map(|(v, r)| (r.sha256.clone(), self.blocked(id, v, &r.sha256, Severity::Disable)));
                (id, a, active)
            })
            .collect();
        let bytes = serde_json::to_vec(&rows).unwrap_or_default();
        let hex = crate::store::digest(&bytes);
        hex.get(..16).and_then(|h| u64::from_str_radix(h, 16).ok()).unwrap_or(0)
    }
    pub fn blocked(&self, id: &ExtensionId, v: Version, sha: &str, severity: Severity) -> bool {
        let vendor = self.packages.get(id).and_then(|p| p.releases.get(&v).and_then(|r| r.origin.as_ref()).unwrap_or(&p.origin).vendor());
        self.revocations.iter().any(|r| r.severity >= severity && r.matches(id, v, sha, vendor))
    }
}

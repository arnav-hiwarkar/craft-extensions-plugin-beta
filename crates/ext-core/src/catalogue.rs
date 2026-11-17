
use crate::id::{AppId, ExtensionId};
use crate::listing::{Attestation, Category, Downloads, Image, known_categories};
use crate::manifest::{Compat, ContributionKind, Manifest};
use crate::tier::{Tier, VendorTier};
use crate::version::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const SCHEMA: u32 = 1;
pub const MAX_INDEX_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_PACKAGES: usize = 20_000;
pub const MAX_PACKAGE_FILE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_RELEASES: usize = 2_000;
pub const MAX_REVOCATIONS: usize = 10_000;

pub const TRUSTED_KEYS: &[&str] = &["d49ebfdf63910eafaaba1cf72ee2dd43f1e82b4cf8cea8c5a0ce93021c99323a"];

pub const DEFAULT_URL: &str = "https://raw.githubusercontent.com/arnav-hiwarkar/craft-plugins/main/dist/";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vendor {
    pub login: String,
    pub github_id: u64,
    pub name: String,
    pub tier: VendorTier,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PackageSummary {
    pub id: ExtensionId,
    pub name: String,
    pub summary: String,
    pub vendor: String,
    pub repo: String,
    #[serde(default)]
    pub stars: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    pub kinds: Vec<ContributionKind>,
    pub apps: Vec<AppId>,
    pub native: bool,
    pub license: String,
    pub latest: Version,
    pub updated: String,
    pub file_sha256: String,
    pub file_size: u64,
    #[serde(default)]
    pub stars_date: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, deserialize_with = "known_categories", skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<Category>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<Image>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub screenshots: Vec<Image>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downloads: Option<Downloads>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Advisory,
    BlockNew,
    Disable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revocation {
    #[serde(default)]
    pub id: Option<ExtensionId>,
    #[serde(default)]
    pub versions: Option<VersionReq>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub vendor: Option<String>,
    pub severity: Severity,
    pub reason: String,
    pub date: String,
}

impl Revocation {
    pub fn matches(&self, id: &ExtensionId, version: Version, sha256: &str, vendor: Option<&str>) -> bool {
        let id_ok = self.id.as_ref().is_none_or(|r| r == id);
        let ver_ok = self.versions.is_none_or(|r| r.matches(version));
        let dig_ok = self.sha256.as_deref().is_none_or(|d| d.eq_ignore_ascii_case(sha256));
        let ven_ok = self.vendor.as_deref().is_none_or(|v| Some(v) == vendor);
        let targeted = self.id.is_some() || self.sha256.is_some() || self.vendor.is_some();
        targeted && id_ok && ver_ok && dig_ok && ven_ok
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Index {
    pub schema: u32,
    pub serial: u64,
    pub issued: u64,
    pub expires: u64,
    pub vendors: Vec<Vendor>,
    pub packages: Vec<PackageSummary>,
    #[serde(default)]
    pub revocations: Vec<Revocation>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub schema_minor: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub featured: Vec<ExtensionId>,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    #[serde(default)]
    pub platform: Option<String>,
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub tag: String,
    pub commit: String,
    pub published: String,
    pub assets: Vec<Asset>,
    pub manifest: Manifest,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub changelog: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestation: Option<Attestation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downloads: Option<Downloads>,
}

impl Release {
    pub fn asset_for(&self, platform: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.platform.as_deref().is_none_or(|p| p == platform))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PackageFile {
    pub schema: u32,
    pub id: ExtensionId,
    pub vendor: String,
    pub releases: Vec<Release>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogueError {
    TooLarge,
    NotReadable(String),
    Newer(u32),
    Expired,
    Rollback,
    TooMany,
    WrongPackage,
}

impl fmt::Display for CatalogueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => f.write_str("the catalogue is larger than allowed"),
            Self::NotReadable(e) => write!(f, "the catalogue cannot be read: {e}"),
            Self::Newer(n) => write!(f, "the catalogue uses format {n}; update the app"),
            Self::Expired => f.write_str("the catalogue is out of date and cannot authorise installs"),
            Self::Rollback => f.write_str("the catalogue is older than one already seen (possible replay)"),
            Self::TooMany => f.write_str("the catalogue lists more entries than allowed"),
            Self::WrongPackage => f.write_str("the catalogue file is for a different package"),
        }
    }
}

impl std::error::Error for CatalogueError {}

fn fresh(schema: u32, serial: u64, expires: u64, now: u64, newest_seen: u64) -> Result<(), CatalogueError> {
    if schema != SCHEMA {
        return Err(CatalogueError::Newer(schema));
    }
    if serial < newest_seen {
        return Err(CatalogueError::Rollback);
    }
    if now > expires {
        return Err(CatalogueError::Expired);
    }
    Ok(())
}

impl Index {
    pub fn parse(bytes: &[u8], now: u64, newest_seen: u64) -> Result<Self, CatalogueError> {
        if bytes.len() > MAX_INDEX_BYTES {
            return Err(CatalogueError::TooLarge);
        }
        let i: Index = serde_json::from_slice(bytes).map_err(|e| CatalogueError::NotReadable(e.to_string()))?;
        if i.packages.len() > MAX_PACKAGES || i.revocations.len() > MAX_REVOCATIONS || i.vendors.len() > MAX_PACKAGES {
            return Err(CatalogueError::TooMany);
        }
        fresh(i.schema, i.serial, i.expires, now, newest_seen)?;
        Ok(i)
    }

    pub fn parse_cached(bytes: &[u8]) -> Result<Self, CatalogueError> {
        let i: Index = serde_json::from_slice(bytes.get(..MAX_INDEX_BYTES.min(bytes.len())).unwrap_or_default())
            .map_err(|e| CatalogueError::NotReadable(e.to_string()))?;
        Ok(i)
    }

    pub fn vendor(&self, login: &str) -> Option<&Vendor> {
        self.vendors.iter().find(|v| v.login == login)
    }

    pub fn package(&self, id: &ExtensionId) -> Option<&PackageSummary> {
        self.packages.iter().find(|p| &p.id == id)
    }

    pub fn revocation(&self, id: &ExtensionId, version: Version, sha256: &str) -> Option<&Revocation> {
        let vendor = self.package(id).map(|p| p.vendor.as_str());
        self.revocations.iter().filter(|r| r.matches(id, version, sha256, vendor)).max_by_key(|r| r.severity)
    }

    pub fn tier_of(&self, id: &ExtensionId) -> Tier {
        self.package(id).and_then(|p| self.vendor(&p.vendor)).map(|v| v.tier.into()).unwrap_or(Tier::Local)
    }
}

impl PackageFile {
    pub fn parse(bytes: &[u8], expected: &ExtensionId) -> Result<Self, CatalogueError> {
        if bytes.len() > MAX_PACKAGE_FILE_BYTES {
            return Err(CatalogueError::TooLarge);
        }
        let p: PackageFile = serde_json::from_slice(bytes).map_err(|e| CatalogueError::NotReadable(e.to_string()))?;
        if &p.id != expected || p.releases.iter().any(|r| &r.manifest.id != expected || r.manifest.version != r.version) {
            return Err(CatalogueError::WrongPackage);
        }
        if p.releases.len() > MAX_RELEASES {
            return Err(CatalogueError::TooMany);
        }
        if p.schema != SCHEMA {
            return Err(CatalogueError::Newer(p.schema));
        }
        let mut seen = std::collections::BTreeSet::new();
        if !p.releases.iter().all(|r| seen.insert(r.version)) {
            return Err(CatalogueError::WrongPackage);
        }
        Ok(p)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Query<'a> {
    pub text: &'a str,
    pub app: Option<&'a AppId>,
    pub kind: Option<ContributionKind>,
    pub include_native: bool,
}

pub fn search<'a>(index: &'a Index, q: &Query<'_>) -> Vec<&'a PackageSummary> {
    let needle = q.text.trim().to_lowercase();
    let mut hits: Vec<(u8, VendorTier, u32, &PackageSummary)> = index
        .packages
        .iter()
        .filter(|p| q.app.is_none_or(|a| p.apps.contains(a)))
        .filter(|p| q.kind.is_none_or(|k| p.kinds.contains(&k)))
        .filter(|p| q.include_native || !p.native)
        .filter(|p| index.revocation(&p.id, p.latest, "").is_none_or(|r| r.severity != Severity::Disable))
        .filter_map(|p| {
            let name = p.name.to_lowercase();
            let id = p.id.as_str();
            let rank = if needle.is_empty() {
                3
            } else if name == needle || id == needle {
                0
            } else if name.starts_with(&needle) || id.rsplit('.').next().is_some_and(|s| s.starts_with(&needle)) {
                1
            } else if p.tags.iter().any(|t| t.to_lowercase() == needle) || name.split_whitespace().any(|w| w.starts_with(&needle)) {
                2
            } else if p.summary.to_lowercase().contains(&needle) || id.contains(&needle) {
                3
            } else {
                return None;
            };
            let tier = index.vendor(&p.vendor).map(|v| v.tier).unwrap_or(VendorTier::Community);
            Some((rank, tier, p.stars, p))
        })
        .collect();
    hits.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(b.2.cmp(&a.2)).then(a.3.id.cmp(&b.3.id)));
    hits.into_iter().map(|h| h.3).collect()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Resolution<'a> {
    Use(&'a Release, Compat),
    None(Vec<(Version, Vec<String>)>),
}

pub fn resolve<'a>(
    releases: &'a [Release],
    app: &AppId,
    provided: &crate::contract::Provided,
    req: VersionReq,
    revoked: impl Fn(&Release) -> Option<Severity>,
) -> Resolution<'a> {
    let mut sorted: Vec<&Release> = releases.iter().filter(|r| req.matches(r.version)).collect();
    sorted.sort_by_key(|r| std::cmp::Reverse(r.version));
    let mut why = Vec::new();
    for r in sorted {
        if revoked(r).is_some_and(|s| s >= Severity::BlockNew) {
            why.push((r.version, vec!["withdrawn from the store".to_string()]));
            continue;
        }
        match r.manifest.compat(app, provided) {
            Compat::Incompatible { reasons } => why.push((r.version, reasons)),
            c => return Resolution::Use(r, c),
        }
    }
    Resolution::None(why)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Provided;
    use crate::manifest::tests::SAMPLE;

    fn release(v: &str, contract_minor: u32) -> Release {
        let text = SAMPLE
            .replace("version = \"1.2.0\"", &format!("version = \"{v}\""))
            .replace("raster.filter@1.0", &format!("raster.filter@{}.{contract_minor}", v.split('.').next().unwrap_or("1")));
        Release {
            version: Version::parse(v).unwrap(),
            tag: format!("v{v}"),
            commit: "0".repeat(40),
            published: "2026-11-12".into(),
            assets: vec![Asset { platform: None, url: "https://github.com/x".into(), sha256: "a".repeat(64), size: 1 }],
            manifest: Manifest::parse(text.as_bytes()).unwrap(),
            changelog: String::new(),
            attestation: None,
            downloads: None,
        }
    }

    #[test]
    fn resolves_newest_compatible_and_explains() {
        let rs = vec![release("1.0.0", 0), release("2.0.0", 5)];
        let app = AppId::new("effectcraft").unwrap();
        let old_app = Provided::new(&[("raster.filter", 1, 2)]);
        match resolve(&rs, &app, &old_app, VersionReq::Any, |_| None) {
            Resolution::Use(r, _) => assert_eq!(r.version, Version::new(1, 0, 0)),
            other => panic!("{other:?}"),
        }
        let none = resolve(&rs, &app, &Provided::new(&[("raster.filter", 2, 0)]), VersionReq::Any, |_| None);
        assert!(matches!(none, Resolution::None(ref w) if w.len() == 2));
        let pinned = resolve(&rs, &app, &Provided::new(&[("raster.filter", 1, 9)]), VersionReq::parse("=1.0.0").unwrap(), |_| None);
        assert!(matches!(pinned, Resolution::Use(r, _) if r.version == Version::new(1, 0, 0)));
        let blocked = resolve(&rs, &app, &old_app, VersionReq::Any, |_| Some(Severity::BlockNew));
        assert!(matches!(blocked, Resolution::None(_)));
    }

    #[test]
    fn revocation_matching() {
        let id = ExtensionId::new("org.example.a").unwrap();
        let r = Revocation { id: Some(id.clone()), versions: VersionReq::parse("^1").ok(), sha256: None, vendor: None, severity: Severity::Disable, reason: "x".into(), date: "d".into() };
        assert!(r.matches(&id, Version::new(1, 0, 0), "", None) && !r.matches(&id, Version::new(2, 0, 0), "", None));
        let empty = Revocation { id: None, versions: None, sha256: None, vendor: None, severity: Severity::Disable, reason: "x".into(), date: "d".into() };
        assert!(!empty.matches(&id, Version::new(1, 0, 0), "", None), "an untargeted entry must never match everything");
    }
}

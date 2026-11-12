//! Contracts: what an app offers plugins, and how compatibility is decided.
//!
//! Compatibility is decided by contract versions, never by app versions. An app *provides* a set of
//! contracts (`raster.filter@1.2`); a package *needs*, per app, a set of contracts at a minimum
//! `major.minor`. A need is met when the app provides the same major at an equal or higher minor. An
//! app can therefore move forward for years without breaking a plugin, as long as the contracts that
//! plugin uses keep their major version.
//!
//! Invariant: a contract name is lowercase dotted words, so `raster.filter`, `effectcraft.session` and
//! `resource.lut` are all valid and look-alikes are impossible.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// A contract name such as `raster.filter` or `figcraft.session`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ContractName(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractError(pub String);

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a valid contract: {}", self.0)
    }
}

impl std::error::Error for ContractError {}

impl ContractName {
    pub fn new(text: &str) -> Result<Self, ContractError> {
        let ok = !text.is_empty()
            && text.len() <= 48
            && text.split('.').all(|s| {
                s.starts_with(|c: char| c.is_ascii_lowercase()) && s.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '-'))
            });
        if ok { Ok(Self(text.to_string())) } else { Err(ContractError(text.to_string())) }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContractName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ContractName {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        ContractName::new(&String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// `major.minor` of a contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Level {
    pub major: u32,
    pub minor: u32,
}

impl Level {
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    pub fn parse(text: &str) -> Result<Self, ContractError> {
        let bad = || ContractError(text.to_string());
        let (a, b) = text.split_once('.').unwrap_or((text, "0"));
        let num = |t: &str| -> Result<u32, ContractError> {
            if t.is_empty() || t.len() > 6 || !t.bytes().all(|c| c.is_ascii_digit()) {
                return Err(bad());
            }
            t.parse().map_err(|_| bad())
        };
        Ok(Self { major: num(a)?, minor: num(b)? })
    }

    /// Whether a provided level satisfies this needed level.
    pub fn satisfied_by(self, provided: Level) -> bool {
        provided.major == self.major && provided.minor >= self.minor
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl Serialize for Level {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Level {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Level::parse(&String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// `raster.filter@1.2` as written on a contribution: contract name and the minimum level it needs
/// (same major, minor at least this). `raster.filter@1` means `1.0`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContractRef {
    pub name: ContractName,
    pub level: Level,
}

impl ContractRef {
    pub fn parse(text: &str) -> Result<Self, ContractError> {
        let (name, major) = text.split_once('@').ok_or_else(|| ContractError(text.to_string()))?;
        Ok(Self { name: ContractName::new(name)?, level: Level::parse(major)? })
    }
}

impl fmt::Display for ContractRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.level)
    }
}

impl Serialize for ContractRef {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ContractRef {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        ContractRef::parse(&String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// What one app build offers. An app may provide two majors of one contract side by side during a
/// migration window.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provided {
    pub contracts: Vec<(ContractName, Level)>,
    #[serde(default)]
    pub unsupported: Vec<(crate::manifest::ContributionKind, String)>,
}

impl Provided {
    pub fn new(list: &[(&str, u32, u32)]) -> Self {
        Self {
            unsupported: Vec::new(),
            contracts: list
                .iter()
                .filter_map(|(n, a, b)| ContractName::new(n).ok().map(|n| (n, Level::new(*a, *b))))
                .collect(),
        }
    }

    /// Marks a contribution kind unavailable even when its contract is provided.
    pub fn without(mut self, kind: crate::manifest::ContributionKind, reason: impl Into<String>) -> Self {
        self.unsupported.push((kind, reason.into()));
        self
    }

    /// The level provided for `name` at `major`, if any.
    pub fn level(&self, name: &ContractName, major: u32) -> Option<Level> {
        self.contracts.iter().filter(|(n, l)| n == name && l.major == major).map(|(_, l)| *l).max()
    }

    /// Highest major provided for `name`.
    pub fn newest_major(&self, name: &ContractName) -> Option<u32> {
        self.contracts.iter().filter(|(n, _)| n == name).map(|(_, l)| l.major).max()
    }
}

/// Why a need is not met, in words the manager can show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "why", rename_all = "kebab-case")]
pub enum Unmet {
    /// The app does not offer this contract at all.
    Missing { contract: ContractName, needed: Level },
    /// The app offers the right major but an older minor: updating the app fixes it.
    AppTooOld { contract: ContractName, needed: Level, provided: Level },
    /// The app offers a different major: an older (or newer) plugin version is needed.
    OtherMajor { contract: ContractName, needed: Level, provided: Level },
}

impl fmt::Display for Unmet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unmet::Missing { contract, .. } => write!(f, "this app does not offer {contract}"),
            Unmet::AppTooOld { contract, needed, provided } => {
                write!(f, "needs {contract} {needed}, this app has {provided} — update the app")
            }
            Unmet::OtherMajor { contract, needed, provided } => {
                write!(f, "built for {contract} {needed}, this app has {provided} — choose a version built for {}", provided.major)
            }
        }
    }
}

impl ContractRef {
    /// Why this need is unmet by `provided`, or `None` when it is met.
    pub fn check(&self, provided: &Provided) -> Option<Unmet> {
        let mut m = BTreeMap::new();
        m.insert(self.name.clone(), self.level);
        check_needs(&m, provided).pop()
    }
}

/// Checks a set of needs against what an app provides.
pub fn check_needs(needs: &BTreeMap<ContractName, Level>, provided: &Provided) -> Vec<Unmet> {
    let mut out = Vec::new();
    for (name, needed) in needs {
        match provided.level(name, needed.major) {
            Some(p) if needed.satisfied_by(p) => {}
            Some(p) => out.push(Unmet::AppTooOld { contract: name.clone(), needed: *needed, provided: p }),
            None => match provided.newest_major(name).and_then(|m| provided.level(name, m)) {
                Some(p) => out.push(Unmet::OtherMajor { contract: name.clone(), needed: *needed, provided: p }),
                None => out.push(Unmet::Missing { contract: name.clone(), needed: *needed }),
            },
        }
    }
    out
}

/// Well-known contract names shared across apps. Apps add their own `<app>.session` contract.
pub mod names {
    /// Pixel transforms: effects in EffectCraft, filters in PhotoCraft, image fills in FigCraft.
    pub const RASTER_FILTER: &str = "raster.filter";
    pub const RESOURCE_PRESET: &str = "resource.preset";
    pub const RESOURCE_LUT: &str = "resource.lut";
    pub const RESOURCE_FONT: &str = "resource.font";
    pub const RESOURCE_DATA: &str = "resource.data";
    /// The one theme format for the suite (craft-libs `craft-theme`).
    pub const CRAFT_THEME: &str = "craft-theme";
}

#[cfg(test)]
mod tests {
    use super::*;

    fn needs(list: &[(&str, u32, u32)]) -> BTreeMap<ContractName, Level> {
        list.iter().map(|(n, a, b)| (ContractName::new(n).unwrap(), Level::new(*a, *b))).collect()
    }

    #[test]
    fn same_major_newer_minor_is_compatible() {
        let app = Provided::new(&[("raster.filter", 1, 3)]);
        assert!(check_needs(&needs(&[("raster.filter", 1, 0)]), &app).is_empty());
        assert!(check_needs(&needs(&[("raster.filter", 1, 3)]), &app).is_empty());
    }

    #[test]
    fn explains_each_failure() {
        let app = Provided::new(&[("raster.filter", 1, 2), ("figcraft.session", 2, 0)]);
        let r = check_needs(&needs(&[("raster.filter", 1, 3), ("figcraft.session", 1, 0), ("audio.fx", 1, 0)]), &app);
        assert!(matches!(r[0], Unmet::Missing { .. }));
        assert!(matches!(r[1], Unmet::OtherMajor { .. }));
        assert!(matches!(r[2], Unmet::AppTooOld { .. }));
    }

    #[test]
    fn side_by_side_majors() {
        let app = Provided::new(&[("raster.filter", 1, 4), ("raster.filter", 2, 0)]);
        assert!(check_needs(&needs(&[("raster.filter", 1, 1)]), &app).is_empty());
        assert!(check_needs(&needs(&[("raster.filter", 2, 0)]), &app).is_empty());
    }

    #[test]
    fn parsing() {
        assert_eq!(ContractRef::parse("raster.filter@1").unwrap().level, Level::new(1, 0));
        assert_eq!(ContractRef::parse("raster.filter@1.2").unwrap().to_string(), "raster.filter@1.2");
        assert!(ContractRef::parse("raster.filter").is_err());
        assert!(ContractName::new("Raster").is_err());
        assert_eq!(Level::parse("1").unwrap(), Level::new(1, 0));
        assert!(Level::parse("1.x").is_err());
    }
}

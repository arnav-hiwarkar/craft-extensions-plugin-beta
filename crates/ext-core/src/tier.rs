//! Trust labels. Never read from a package: the host assigns them from where the package came.
//!
//! - **Official**: vendor flagged official by maintainers in the signed catalogue (ArtCraft org).
//! - **Verified**: vendor whose identity a maintainer checked (verified GitHub org domain or DNS TXT).
//! - **Community**: any registered vendor whose listing a maintainer allowed.
//! - **Local**: installed from a file; not in the catalogue (or digest does not match a listing).
//! - **Dev**: installed by `craft-ext dev`; only visible in the development store.
//!
//! First-party id prefixes may only be used by Official packages, so nobody can impersonate them.

use crate::id::ExtensionId;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VendorTier {
    Official,
    Verified,
    Community,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tier {
    Official,
    Verified,
    Community,
    Local,
    Dev,
}

impl From<VendorTier> for Tier {
    fn from(v: VendorTier) -> Self {
        match v {
            VendorTier::Official => Tier::Official,
            VendorTier::Verified => Tier::Verified,
            VendorTier::Community => Tier::Community,
        }
    }
}

impl Tier {
    /// Badge text for the manager, launcher and CLI. Never colour alone.
    pub fn badge(self) -> &'static str {
        match self {
            Tier::Official => "Official",
            Tier::Verified => "✔ Verified vendor",
            Tier::Community => "Community",
            Tier::Local => "Not reviewed (local file)",
            Tier::Dev => "Development",
        }
    }

    pub fn from_catalogue(self) -> bool {
        matches!(self, Tier::Official | Tier::Verified | Tier::Community)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReservedNamespace;

impl fmt::Display for ReservedNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the package uses a name that only ArtCraft's own plugins may use")
    }
}

impl std::error::Error for ReservedNamespace {}

/// Only Official packages may use first-party prefixes.
pub fn check_id(tier: Tier, id: &ExtensionId) -> Result<(), ReservedNamespace> {
    if id.in_first_party_namespace() && tier != Tier::Official { Err(ReservedNamespace) } else { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_prefix_needs_official() {
        let id = ExtensionId::new("org.storytold.tone").unwrap();
        assert!(check_id(Tier::Official, &id).is_ok());
        assert!(check_id(Tier::Local, &id).is_err());
        assert!(check_id(Tier::Verified, &id).is_err());
        assert!(check_id(Tier::Local, &ExtensionId::new("org.example.tone").unwrap()).is_ok());
    }
}

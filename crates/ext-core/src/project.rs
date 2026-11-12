//! What a project records about the plugins it uses, and how an opening app classifies them.
//!
//! Invariants: a project never carries grants; opening a project never downloads, installs or
//! enables anything — the app shows the [`Status`] of each plugin and lets the person choose.

use crate::id::ExtensionId;
use crate::version::Version;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    Catalogue,
    Local,
}

/// One entry in a project's plugin list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRef {
    pub id: ExtensionId,
    pub version: Version,
    #[serde(default)]
    pub sha256: Option<String>,
    pub source: Source,
    /// Display name, so the prompt is readable even when nothing is installed.
    #[serde(default)]
    pub name: String,
    /// Contributions used by this project; all must be enabled in the opening app.
    #[serde(default)]
    pub contributions: Vec<String>,
    /// Saved state schema per contribution, when recorded by the app.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_versions: Option<std::collections::BTreeMap<String, u32>>,
}

/// A saved use of one contribution (an effect instance, a placed object…). Apps embed this in their
/// own project format; `params` and `state` are kept byte for byte when the plugin is missing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureRef {
    #[serde(default)]
    pub contract: Option<String>,
    /// Preserve as a placeholder when this exceeds the loaded contribution's state_version.
    #[serde(default)]
    pub state_version: u32,
    pub plugin: ExtensionId,
    pub contribution: String,
    pub version: Version,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub state: serde_json::Value,
}

/// What the opening app found for one referenced plugin.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum Status {
    /// Installed, set up and enabled in this app at a compatible version.
    Ready { version: Version },
    /// Installed but switched off (or permissions not granted) in this app.
    NeedsEnable { version: Version, reason: String },
    /// Installed at a version that cannot load the saved data (older, or incompatible).
    NeedsOtherVersion { installed: Version, wanted: Version, reason: String },
    /// Not installed; the catalogue lists it.
    Downloadable,
    /// Not installed and not in the catalogue (a local-only plugin), or kill-listed.
    Unavailable { reason: String },
}

impl Status {
    pub fn ready(&self) -> bool {
        matches!(self, Status::Ready { .. })
    }
}

/// Builds the "This project uses N plugins" summary line.
pub fn summary(found: &[(PluginRef, Status)]) -> Option<String> {
    let missing = found.iter().filter(|(_, s)| !s.ready()).count();
    (missing > 0).then(|| {
        let n = found.len();
        format!("This project uses {n} plugin{}; {missing} need{} your attention.", if n == 1 { "" } else { "s" }, if missing == 1 { "s" } else { "" })
    })
}

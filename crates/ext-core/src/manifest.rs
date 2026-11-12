//! The package manifest, `craft-plugin.toml`: one human-written file that is both the source manifest
//! and the manifest inside the package archive (`.craftplugin`, a zip).
//!
//! Invariants: the size is capped before parsing; unknown fields are refused (a typo must not silently
//! drop a permission); a package never states its own trust tier or catalogue status; every
//! contribution names a contract; every requested permission parses into the closed vocabulary; the
//! archive digest (recorded by the store and pinned by the catalogue) binds every byte, so the manifest
//! carries no file list.

use crate::contract::ContractRef;
use crate::id::{AppId, ExtensionId};
use crate::path::validate_path;
use crate::permission::Permission;
use crate::text::{Lines, validate_text};
use crate::version::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const MANIFEST_NAME: &str = "craft-plugin.toml";
pub const PACKAGE_EXTENSION: &str = "craftplugin";
pub const MAX_MANIFEST_BYTES: usize = 128 * 1024;
pub const SCHEMA: u32 = 2;
pub const MAX_APPS: usize = 32;
pub const MAX_CONTRIBUTIONS: usize = 128;
pub const MAX_PARAMS: usize = 64;

/// What a contribution adds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContributionKind {
    /// Pixel transform: an effect (EffectCraft), filter (PhotoCraft), image fill (FigCraft).
    Effect,
    /// A command in menus and command search; runs session code.
    Command,
    /// A dockable panel drawn by the host from a declarative tree.
    Panel,
    /// A pointer tool (begin, update, commit, cancel).
    Tool,
    /// Code that reacts to app events (needs `app.events`).
    EventHook,
    /// Data: no code runs.
    Preset,
    Lut,
    Font,
    Theme,
    Data,
}

impl ContributionKind {
    pub fn is_data(self) -> bool {
        matches!(self, Self::Preset | Self::Lut | Self::Font | Self::Theme | Self::Data)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Effect => "Effect",
            Self::Command => "Command",
            Self::Panel => "Panel",
            Self::Tool => "Tool",
            Self::EventHook => "Automation",
            Self::Preset => "Presets",
            Self::Lut => "LUTs",
            Self::Font => "Fonts",
            Self::Theme => "Theme",
            Self::Data => "Data",
        }
    }
}

/// An effect or command parameter. The host draws the control, animates it and saves it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ParamKind {
    Slider {
        default: f64,
        min: f64,
        max: f64,
        #[serde(default)]
        integer: bool,
    },
    Angle {
        #[serde(default)]
        default: f64,
    },
    Checkbox {
        #[serde(default)]
        default: bool,
    },
    Select {
        options: Vec<String>,
        #[serde(default)]
        default: u32,
    },
    Color {
        #[serde(default = "white")]
        default: [f64; 4],
    },
    Point {
        #[serde(default)]
        default: [f64; 2],
    },
    Text {
        #[serde(default)]
        default: String,
    },
}

fn white() -> [f64; 4] {
    [1.0, 1.0, 1.0, 1.0]
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// (Unknown keys inside a parameter are ignored: serde cannot refuse them through `flatten`.)
pub struct Param {
    pub id: String,
    pub label: String,
    #[serde(flatten)]
    pub kind: ParamKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contribution {
    pub id: String,
    pub kind: ContributionKind,
    pub contract: ContractRef,
    pub label: String,
    /// The wasm module (sandboxed) for code kinds; ignored for native packages.
    #[serde(default)]
    pub module: Option<String>,
    /// Data files for data kinds (relative paths inside the package).
    #[serde(default)]
    pub files: Vec<String>,
    /// Menu group / category, e.g. "Stylize".
    #[serde(default)]
    pub category: Option<String>,
    /// Restrict to these apps (default: every app listed under `[apps]` that provides the contract).
    #[serde(default)]
    pub apps: Vec<AppId>,
    /// Permissions this contribution cannot work without (granted before it can be enabled).
    #[serde(default)]
    pub permissions: Vec<Permission>,
    /// Permissions it can use when the person turns them on; off by default.
    #[serde(default)]
    pub optional_permissions: Vec<Permission>,
    /// Version of the state this contribution saves in projects; bump when the shape changes.
    #[serde(default)]
    pub state_version: u32,
    #[serde(default, rename = "param")]
    pub params: Vec<Param>,
}

impl Contribution {
    /// Whether this contribution affects rendered output (exports refuse while it is missing).
    pub fn renders(&self) -> bool {
        self.kind == ContributionKind::Effect
    }
}

/// How a native package starts its helper, per platform (`linux-x86_64`, `macos-aarch64`, `windows-x86_64` …).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Native {
    pub executables: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub id: ExtensionId,
    pub name: String,
    pub version: Version,
    /// SPDX licence expression.
    pub license: String,
    /// Public source repository (https).
    pub source: String,
    pub summary: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Present = out-of-process native plugin with full access.
    #[serde(default)]
    pub native: Option<Native>,
    /// Apps this package supports. Each contribution's contract decides where it actually works.
    pub apps: Vec<AppId>,
    /// The author's reason for a permission, quoted under the host's own sentence. Keyed by permission text.
    #[serde(default)]
    pub why: BTreeMap<String, String>,
    /// Data packages this one needs (id -> version requirement).
    #[serde(default)]
    pub requires: BTreeMap<ExtensionId, VersionReq>,
    #[serde(default, rename = "contribution")]
    pub contributions: Vec<Contribution>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestError(pub String);

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ManifestError {}

fn err<T>(msg: impl Into<String>) -> Result<T, ManifestError> {
    Err(ManifestError(msg.into()))
}

fn local_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 48 && id.starts_with(|c: char| c.is_ascii_lowercase()) && id.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '-' | '_'))
}

impl Manifest {
    /// Parses and validates untrusted manifest bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, ManifestError> {
        if bytes.len() > MAX_MANIFEST_BYTES {
            return err("the package description is too large");
        }
        let text = std::str::from_utf8(bytes).map_err(|_| ManifestError("the package description is not text".into()))?;
        #[derive(Deserialize)]
        struct Head {
            schema: u32,
        }
        // Schema first, so a newer package is refused for being newer rather than for a field.
        let head: Head = toml::from_str::<toml::Table>(text)
            .ok()
            .and_then(|t| t.get("schema").cloned())
            .and_then(|v| v.try_into().ok())
            .map(|schema| Head { schema })
            .ok_or_else(|| ManifestError("the package description has no schema number".into()))?;
        if head.schema != SCHEMA {
            return err(format!("the package uses format {}; this app understands format {SCHEMA}", head.schema));
        }
        let m: Manifest = toml::from_str(text).map_err(|e| ManifestError(format!("the package description is not valid: {}", e.message())))?;
        m.validate()?;
        Ok(m)
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        let t = |what: &str, s: &str, n: usize, l: Lines| validate_text(s, n, l).map_err(|e| ManifestError(format!("{what}: {e}")));
        t("name", &self.name, 64, Lines::One)?;
        t("summary", &self.summary, 140, Lines::One)?;
        if !self.description.is_empty() {
            t("description", &self.description, 4000, Lines::Many)?;
        }
        if self.license.trim().is_empty() || self.license.len() > 128 {
            return err("the licence is missing");
        }
        if !self.source.starts_with("https://") || self.source.len() > 256 || self.source.contains(char::is_whitespace) {
            return err("the source must be an https link to the public repository");
        }
        if self.apps.is_empty() {
            return err("the package does not say which apps it is for");
        }
        if self.apps.len() > MAX_APPS || self.contributions.len() > MAX_CONTRIBUTIONS || self.tags.len() > 8 || self.authors.len() > 16 {
            return err("the package lists too many items");
        }
        if self.contributions.is_empty() {
            return err("the package contributes nothing");
        }
        if self.native.as_ref().is_some_and(|n| n.executables.is_empty()) {
            return err("a native package must name its helper executable");
        }
        for path in self.native.iter().flat_map(|n| n.executables.values()) {
            validate_path(path).map_err(|e| ManifestError(e.to_string()))?;
        }
        let mut seen = BTreeSet::new();
        for c in &self.contributions {
            if !local_id(&c.id) || !seen.insert(c.id.as_str()) {
                return err(format!("contribution id \"{}\" is invalid or repeated", c.id));
            }
            t("contribution label", &c.label, 64, Lines::One)?;
            for app in &c.apps {
                if !self.apps.contains(app) {
                    return err(format!("contribution {} names app {app}, which is not under [apps]", c.id));
                }
            }
            let needs_code = !c.kind.is_data();
            match (needs_code, &c.module, self.native.is_some()) {
                (true, None, false) => return err(format!("contribution {} needs a module", c.id)),
                (false, Some(_), _) => return err(format!("data contribution {} cannot have code", c.id)),
                _ => {}
            }
            if c.kind.is_data() && c.files.is_empty() {
                return err(format!("data contribution {} lists no files", c.id));
            }
            for p in c.module.iter().chain(c.files.iter()) {
                validate_path(p).map_err(|e| ManifestError(format!("{}: {e}", c.id)))?;
            }
            if c.params.len() > MAX_PARAMS {
                return err(format!("contribution {} has too many parameters", c.id));
            }
            let mut pids = BTreeSet::new();
            for p in &c.params {
                if !local_id(&p.id) || !pids.insert(p.id.as_str()) {
                    return err(format!("parameter id \"{}\" is invalid or repeated", p.id));
                }
                t("parameter label", &p.label, 64, Lines::One)?;
                if let ParamKind::Slider { default, min, max, .. } = p.kind
                    && !(min.is_finite() && max.is_finite() && min < max && (min..=max).contains(&default))
                {
                    return err(format!("parameter {} has an invalid range", p.id));
                }
                if let ParamKind::Select { options, default } = &p.kind
                    && (options.is_empty() || options.len() > 64 || *default as usize >= options.len())
                {
                    return err(format!("parameter {} has invalid options", p.id));
                }
            }
        }
        if self.native.is_some() && self.contributions.iter().any(|c| c.kind == ContributionKind::Effect) {
            return err("native packages cannot contribute effects in this version (use a sandboxed module)");
        }
        let mut apps = BTreeSet::new();
        if !self.apps.iter().all(|a| apps.insert(a)) {
            return err("an app is listed twice");
        }
        for key in self.why.keys() {
            if !self.contributions.iter().flat_map(|c| c.permissions.iter().chain(c.optional_permissions.iter())).any(|p| p.to_string() == *key) {
                return err(format!("a reason is given for \"{key}\", which is not requested"));
            }
        }
        Ok(())
    }

    pub fn is_native(&self) -> bool {
        self.native.is_some()
    }

    pub fn contribution(&self, id: &str) -> Option<&Contribution> {
        self.contributions.iter().find(|c| c.id == id)
    }

    /// Contributions offered to `app` (before contract checks).
    pub fn contributions_for<'a>(&'a self, app: &'a AppId) -> impl Iterator<Item = &'a Contribution> + 'a {
        self.contributions.iter().filter(move |c| self.apps.contains(app) && (c.apps.is_empty() || c.apps.contains(app)))
    }

    /// Whether the package runs any code at all (sandboxed or native).
    pub fn has_code(&self) -> bool {
        self.native.is_some() || self.contributions.iter().any(|c| !c.kind.is_data())
    }

    /// Permissions (required, optional) for a set of enabled contributions in one app: the union.
    pub fn permissions_for(&self, app: &AppId, enabled: impl Fn(&str) -> bool) -> (Vec<Permission>, Vec<Permission>) {
        let mut req = BTreeSet::new();
        let mut opt = BTreeSet::new();
        for c in self.contributions_for(app).filter(|c| enabled(&c.id)) {
            req.extend(c.permissions.iter().cloned());
            opt.extend(c.optional_permissions.iter().cloned());
        }
        opt.retain(|p| !req.contains(p));
        (req.into_iter().collect(), opt.into_iter().collect())
    }

    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }
}

/// Per-app compatibility of one package release.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum Compat {
    Compatible,
    /// Some contributions work; the others are listed with the reason.
    Partly { unavailable: Vec<(String, String)> },
    Incompatible { reasons: Vec<String> },
}

impl Compat {
    pub fn usable(&self) -> bool {
        !matches!(self, Compat::Incompatible { .. })
    }
}

impl Manifest {
    /// Decides compatibility with one app from the contracts it provides, contribution by
    /// contribution. Never a silent half-load.
    pub fn compat(&self, app: &AppId, provided: &crate::contract::Provided) -> Compat {
        if !self.apps.contains(app) {
            return Compat::Incompatible { reasons: vec![format!("this plugin does not support {app}")] };
        }
        let mut ok = 0usize;
        let mut unavailable = Vec::new();
        for c in self.contributions_for(app) {
            match c.contract.check(provided) {
                Some(u) => unavailable.push((c.id.clone(), u.to_string())),
                None => ok += 1,
            }
        }
        if ok == 0 {
            let mut reasons: Vec<String> = unavailable.into_iter().map(|(_, w)| w).collect();
            reasons.dedup();
            if reasons.is_empty() {
                reasons.push(format!("nothing in this plugin is for {app}"));
            }
            Compat::Incompatible { reasons }
        } else if unavailable.is_empty() {
            Compat::Compatible
        } else {
            Compat::Partly { unavailable }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;

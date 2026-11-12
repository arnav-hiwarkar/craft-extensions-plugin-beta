//! Permissions: what a sandboxed plugin may reach, as `family:scope` strings.
//!
//! Invariants:
//! - The vocabulary is closed and shared by every app. An app needing something new adds a scope to an
//!   existing family; a new family needs a second app that wants it.
//! - Holding a broader scope covers a narrower one (`document.edit:all` covers `:document` and
//!   `:selection`; edit covers read for the same scope).
//! - Network scopes are exact https host names; never IP literals, `localhost`, or wildcards beyond one
//!   leading `*.`. Private-address refusal after DNS is the host's job (craft-ext-host).
//! - Things that are always free (compute, logging, private storage, panels, menus) are not permissions.
//! - Things that are never grantable to sandboxed code (processes, native code, raw sockets, other
//!   plugins' data, management) have no spelling here at all.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocScope {
    Selection,
    Document,
    All,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Permission {
    DocumentRead(DocScope),
    DocumentEdit(DocScope),
    /// Read a file or folder the user picks in the host's dialog (the pick itself is the grant for
    /// that item; this permission allows asking).
    FilesRead,
    FilesWrite,
    /// https requests to one host (or `*.example.com`).
    Network(String),
    ClipboardRead,
    ClipboardWrite,
    /// Open floating windows (panels are free; windows interrupt, so they are asked for).
    UiWindow,
    /// Receive app events (selection changed, document opened, time changed, render finished…).
    AppEvents,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionError(pub String);

impl fmt::Display for PermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown or invalid permission \"{}\"", self.0)
    }
}

impl std::error::Error for PermissionError {}

fn valid_host(host: &str) -> bool {
    let body = host.strip_prefix("*.").unwrap_or(host);
    let labels: Vec<&str> = body.split('.').collect();
    labels.len() >= 2
        && body.len() <= 253
        && labels.iter().all(|l| {
            !l.is_empty()
                && l.len() <= 63
                && l.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                && !l.starts_with('-')
                && !l.ends_with('-')
        })
        // The last label must not be all digits: refuses IPv4 literals such as 10.0.0.1.
        && labels.last().is_some_and(|l| !l.chars().all(|c| c.is_ascii_digit()))
        && !matches!(body, "localhost" | "local") && !body.ends_with(".localhost") && !body.ends_with(".local")
        && !body.ends_with(".internal")
}

impl Permission {
    pub fn parse(text: &str) -> Result<Self, PermissionError> {
        let err = || PermissionError(text.to_string());
        let (family, scope) = text.split_once(':').unwrap_or((text, ""));
        let doc = |s: &str| match s {
            "selection" => Ok(DocScope::Selection),
            "document" => Ok(DocScope::Document),
            "all" => Ok(DocScope::All),
            _ => Err(err()),
        };
        Ok(match (family, scope) {
            ("document.read", s) => Permission::DocumentRead(doc(s)?),
            ("document.edit", s) => Permission::DocumentEdit(doc(s)?),
            ("files.read", "picked") => Permission::FilesRead,
            ("files.write", "picked") => Permission::FilesWrite,
            ("network", host) if valid_host(host) => Permission::Network(host.to_string()),
            ("clipboard.read", "") => Permission::ClipboardRead,
            ("clipboard.write", "") => Permission::ClipboardWrite,
            ("ui.window", "") => Permission::UiWindow,
            ("app.events", "") => Permission::AppEvents,
            _ => return Err(err()),
        })
    }

    /// Whether holding `self` allows `wanted`.
    pub fn covers(&self, wanted: &Permission) -> bool {
        use Permission::*;
        match (self, wanted) {
            (DocumentEdit(have), DocumentEdit(want)) | (DocumentRead(have), DocumentRead(want)) | (DocumentEdit(have), DocumentRead(want)) => {
                have >= want
            }
            (Network(have), Network(want)) => match have.strip_prefix("*.") {
                Some(suffix) => want == suffix || want.ends_with(&format!(".{suffix}")),
                None => have == want,
            },
            (a, b) => a == b,
        }
    }

    /// One plain sentence, written by the host (never by the plugin), for the consent sheet.
    pub fn sentence(&self) -> String {
        let doc = |s: &DocScope| match s {
            DocScope::Selection => "the selected items",
            DocScope::Document => "the current document",
            DocScope::All => "all open documents",
        };
        match self {
            Permission::DocumentRead(s) => format!("Read {}", doc(s)),
            Permission::DocumentEdit(s) => format!("Change {}", doc(s)),
            Permission::FilesRead => "Open files and folders you pick".into(),
            Permission::FilesWrite => "Save to files and folders you pick".into(),
            Permission::Network(h) => format!("Connect to {h} over the internet"),
            Permission::ClipboardRead => "Read the clipboard when you use it".into(),
            Permission::ClipboardWrite => "Copy to the clipboard".into(),
            Permission::UiWindow => "Open its own windows".into(),
            Permission::AppEvents => "Be told when things happen in the app (for example the selection changes)".into(),
        }
    }

    /// Whether this permission, combined with others, lets data leave the machine.
    pub fn is_network(&self) -> bool {
        matches!(self, Permission::Network(_))
    }

    pub fn reads_document(&self) -> bool {
        matches!(self, Permission::DocumentRead(_) | Permission::DocumentEdit(_) | Permission::ClipboardRead | Permission::FilesRead)
    }
}

/// Spelled-out danger combinations ("could send your document to api.example.com").
pub fn warnings(perms: &[Permission]) -> Vec<String> {
    let reads = perms.iter().any(Permission::reads_document);
    perms
        .iter()
        .filter_map(|p| match p {
            Permission::Network(h) if reads => Some(format!("Could send your work to {h}")),
            _ => None,
        })
        .collect()
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let doc = |s: &DocScope| match s {
            DocScope::Selection => "selection",
            DocScope::Document => "document",
            DocScope::All => "all",
        };
        match self {
            Permission::DocumentRead(s) => write!(f, "document.read:{}", doc(s)),
            Permission::DocumentEdit(s) => write!(f, "document.edit:{}", doc(s)),
            Permission::FilesRead => f.write_str("files.read:picked"),
            Permission::FilesWrite => f.write_str("files.write:picked"),
            Permission::Network(h) => write!(f, "network:{h}"),
            Permission::ClipboardRead => f.write_str("clipboard.read"),
            Permission::ClipboardWrite => f.write_str("clipboard.write"),
            Permission::UiWindow => f.write_str("ui.window"),
            Permission::AppEvents => f.write_str("app.events"),
        }
    }
}

impl Serialize for Permission {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Permission {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Permission::parse(&String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(t: &str) -> Permission {
        Permission::parse(t).unwrap()
    }

    #[test]
    fn round_trip() {
        for t in ["document.read:selection", "document.edit:all", "files.read:picked", "network:api.example.com", "network:*.example.org", "ui.window", "app.events"] {
            assert_eq!(p(t).to_string(), t);
        }
    }

    #[test]
    fn refuses_unknown_and_local_hosts() {
        for bad in ["process", "network:localhost", "network:10.0.0.1", "network:example", "network:a.local", "network:*", "document.edit:everything", "network:EXAMPLE.com", "management"] {
            assert!(Permission::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn coverage() {
        assert!(p("document.edit:all").covers(&p("document.read:selection")));
        assert!(!p("document.read:all").covers(&p("document.edit:selection")));
        assert!(!p("document.edit:selection").covers(&p("document.edit:document")));
        assert!(p("network:*.example.com").covers(&p("network:api.example.com")));
        assert!(!p("network:*.example.com").covers(&p("network:evilexample.com")));
        assert!(!p("network:api.example.com").covers(&p("network:example.com")));
    }

    #[test]
    fn danger_combination() {
        assert_eq!(warnings(&[p("document.read:document"), p("network:api.example.com")]).len(), 1);
        assert!(warnings(&[p("network:api.example.com")]).is_empty());
    }
}

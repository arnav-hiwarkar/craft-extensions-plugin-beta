use crate::manifest::{Manifest, NativeMode};
use crate::permission::Permission;
use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Widening {
    Permission(Permission),
    Native(NativeMode),
    FullAccess,
}

impl fmt::Display for Widening {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Widening::Permission(p) => write!(f, "adds permission {p} ({})", p.sentence()),
            Widening::Native(NativeMode::Full) => f.write_str("adds a native helper with full access to the computer"),
            Widening::Native(NativeMode::Restricted) => f.write_str("adds a restricted native helper"),
            Widening::FullAccess => f.write_str("changes the native helper from restricted to full access"),
        }
    }
}

fn permissions(m: &Manifest) -> BTreeSet<Permission> {
    m.contributions.iter().flat_map(|c| c.permissions.iter().chain(&c.optional_permissions)).cloned().collect()
}

pub fn widenings(previous: Option<&Manifest>, next: &Manifest) -> Vec<Widening> {
    let mut out = match (previous.and_then(Manifest::native_mode), next.native_mode()) {
        (None, Some(mode)) => vec![Widening::Native(mode)],
        (Some(NativeMode::Restricted), Some(NativeMode::Full)) => vec![Widening::FullAccess],
        _ => vec![],
    };
    if let Some(previous) = previous {
        let held = permissions(previous);
        out.extend(permissions(next).into_iter().filter(|p| !held.iter().any(|h| h.covers(p))).map(Widening::Permission));
    }
    out
}

pub fn name_key(text: &str) -> String {
    let lower = text.to_lowercase().replace("rn", "m").replace("vv", "w");
    lower
        .chars()
        .filter_map(|c| match c {
            '0' => Some('o'),
            '1' | 'i' | '|' | '!' => Some('l'),
            '3' => Some('e'),
            '4' | '@' => Some('a'),
            '5' | '$' => Some('s'),
            '7' => Some('t'),
            c if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .collect()
}

fn distance(a: &[char], b: &[char]) -> usize {
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (above + 1).min(row[j] + 1).min(diagonal + usize::from(ca != cb));
            diagonal = above;
        }
    }
    row[b.len()]
}

pub const MAX_COMPARED_CHARS: usize = 64;

pub fn too_similar(a: &str, b: &str) -> bool {
    let a: Vec<char> = name_key(a).chars().take(MAX_COMPARED_CHARS).collect();
    let b: Vec<char> = name_key(b).chars().take(MAX_COMPARED_CHARS).collect();
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let allowed = match a.len().min(b.len()) {
        0..=4 => 0,
        5..=8 => 1,
        _ => 2,
    };
    distance(&a, &b) <= allowed
}

pub fn id_tail(id: &str) -> &str {
    id.rsplit('.').next().unwrap_or(id)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::manifest::tests::SAMPLE;

    const OPTIONAL: &str = "optional_permissions = [\"network:api.example.com\"]";

    fn manifest(extra_permission: Option<&str>, native: Option<&str>) -> Manifest {
        let mut text = SAMPLE.to_string();
        if let Some(p) = extra_permission {
            text = text.replace(OPTIONAL, &format!("optional_permissions = [\"network:api.example.com\", \"{p}\"]"));
        }
        if let Some(mode) = native {
            text = text.replace(
                "apps = [\"effectcraft\", \"photocraft\"]",
                &format!("apps = [\"effectcraft\", \"photocraft\"]\n\n[native]\nmode = \"{mode}\"\n[native.executables]\nmacos-aarch64 = \"bin/helper\""),
            ) + "\n[[contribution]]\nid = \"upscale\"\nkind = \"command\"\ncontract = \"photocraft.session@1.0\"\nlabel = \"Upscale\"\n";
        }
        Manifest::parse(text.as_bytes()).unwrap()
    }

    #[test]
    fn sample_has_the_permission_these_tests_extend() {
        assert!(SAMPLE.contains(OPTIONAL));
    }

    #[test]
    fn wider_permissions_and_native_access_are_named() {
        let old = manifest(None, None);
        assert_eq!(widenings(Some(&old), &old), vec![]);
        let wider = manifest(Some("clipboard.read"), None);
        assert_eq!(
            widenings(Some(&old), &wider).iter().map(ToString::to_string).collect::<Vec<_>>(),
            vec!["adds permission clipboard.read (Read the clipboard when you use it)"]
        );
        assert_eq!(widenings(Some(&wider), &old), vec![]);
        let restricted = manifest(None, Some("restricted"));
        assert_eq!(widenings(Some(&old), &restricted), vec![Widening::Native(NativeMode::Restricted)]);
        assert_eq!(widenings(Some(&restricted), &manifest(None, Some("full"))), vec![Widening::FullAccess]);
        assert_eq!(widenings(None, &wider), vec![]);
        assert_eq!(widenings(None, &manifest(None, Some("full"))), vec![Widening::Native(NativeMode::Full)]);
    }

    #[test]
    fn narrower_scopes_are_covered_by_broader_old_ones() {
        let broad = manifest(Some("document.read:all"), None);
        let narrow = manifest(Some("document.read:document"), None);
        assert_eq!(widenings(Some(&broad), &narrow), vec![]);
        assert_eq!(widenings(Some(&narrow), &broad), vec![Widening::Permission(Permission::parse("document.read:all").unwrap())]);
    }

    #[test]
    fn lookalike_names_are_flagged() {
        for (a, b) in [("Film Looks", "Fi1m Looks"), ("Halftone", "Halft0ne"), ("Glow", "glow"), ("modern", "modem"), ("Super Blur", "SuperBlurs")] {
            assert!(too_similar(a, b), "{a} / {b}");
        }
        for (a, b) in [("Glow", "Flow"), ("Film Looks", "Film Grain"), ("Halftone", "Duotone"), ("", "")] {
            assert!(!too_similar(a, b), "{a} / {b}");
        }
        assert_eq!(id_tail("io.github.alice.film-looks"), "film-looks");
        assert_eq!(name_key("Fi1m-L00ks!"), "fllmlooksl");
    }
}

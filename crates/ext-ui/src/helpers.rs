//! Pure presentation and consent models.
use craft_ext_core::{
    manifest::{Compat, Manifest},
    permission::Permission,
    project::Status,
    tier::Tier,
};
use craft_ext_store::manager::PermissionSheet;
pub fn status_text(s: &Status) -> String {
    match s {
        Status::Ready { version } => format!("Ready ({version})"),
        Status::NeedsEnable { version, reason } => format!("Installed, needs enable ({version}): {reason}"),
        Status::NeedsOtherVersion { installed, wanted, reason } => format!("Needs version {wanted}; installed {installed}: {reason}"),
        Status::Downloadable => "Available to download".into(),
        Status::Unavailable { reason } => format!("Unavailable: {reason}"),
    }
}
pub fn badge_row_text(tier: Tier, stars: u32, date: &str, native: bool) -> String {
    format!("{} · ★ {stars} (as of {date}){}", tier.badge(), if native { " · Native: full access" } else { "" })
}
pub fn compatibility(c: &Compat) -> String {
    match c {
        Compat::Compatible => "Compatible".into(),
        Compat::Partly { unavailable } => {
            format!("Partly compatible: {}", unavailable.iter().map(|(c, r)| format!("{c}: {r}")).collect::<Vec<_>>().join("; "))
        }
        Compat::Incompatible { reasons } => format!("Incompatible: {}", reasons.join("; ")),
    }
}
/// Default Browse selection: the newest release this app can use, regardless of catalogue order.
pub fn newest_compatible(versions: &[(craft_ext_core::version::Version, Compat)]) -> Option<craft_ext_core::version::Version> {
    versions.iter().filter(|(_, c)| c.usable()).map(|(v, _)| *v).max()
}
pub fn permission_sheet(m: &Manifest, app: &craft_ext_core::id::AppId, selected: impl Fn(&str) -> bool) -> PermissionSheet {
    let (required, optional) = m.permissions_for(app, selected);
    let warnings = craft_ext_core::permission::warnings(&required.iter().chain(&optional).cloned().collect::<Vec<_>>());
    PermissionSheet {
        required: required
            .into_iter()
            .map(|p| {
                let s = p.sentence();
                (p, s)
            })
            .collect(),
        optional: optional
            .into_iter()
            .map(|p| {
                let s = p.sentence();
                (p, s)
            })
            .collect(),
        warnings,
        why: m.why.clone(),
        native: m.is_native(),
    }
}
#[derive(Clone)]
pub struct PermissionChoice {
    pub permission: Permission,
    pub sentence: String,
    pub why: Option<String>,
    pub required: bool,
    pub checked: bool,
}
pub fn sheet_model(sheet: &PermissionSheet) -> Vec<PermissionChoice> {
    sheet
        .required
        .iter()
        .map(|p| (p, true))
        .chain(sheet.optional.iter().map(|p| (p, false)))
        .map(|((p, s), required)| PermissionChoice {
            permission: p.clone(),
            sentence: s.clone(),
            why: sheet.why.get(&p.to_string()).cloned(),
            required,
            checked: false,
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn words_preserve_reason() {
        assert_eq!(status_text(&Status::Unavailable { reason: "withdrawn".into() }), "Unavailable: withdrawn");
    }
    #[test]
    fn badges_have_text() {
        assert!(badge_row_text(Tier::Local, 3, "today", true).contains("Not reviewed"));
        assert!(badge_row_text(Tier::Community, 0, "today", true).contains("Native: full access"));
    }
    #[test]
    fn consent_starts_off() {
        let p = Permission::parse("clipboard.read").unwrap();
        let s = PermissionSheet {
            required: vec![(p, "Read clipboard".into())],
            optional: vec![],
            warnings: vec![],
            why: Default::default(),
            native: false,
        };
        let m = sheet_model(&s);
        assert!(m[0].required);
        assert!(!m[0].checked);
        let optional = Permission::parse("clipboard.write").unwrap();
        let mut sheet = s;
        sheet.why.insert(optional.to_string(), "Copy the result".into());
        sheet.optional.push((optional, "Write clipboard".into()));
        let model = sheet_model(&sheet);
        assert!(!model[1].required);
        assert!(!model[1].checked);
        assert_eq!(model[1].why.as_deref(), Some("Copy the result"));
    }
}

#[cfg(test)]
mod browse_tests {
    use super::*;
    use craft_ext_core::version::Version;
    #[test]
    fn latest_incompatible_selects_newest_compatible() {
        let old = Version::new(1,0,0);
        let best = Version::new(2,0,0);
        let latest = Version::new(3,0,0);
        let bad = Compat::Incompatible { reasons: vec!["needs a newer contract".into()] };
        assert_eq!(newest_compatible(&[(old, Compat::Compatible), (latest, bad.clone()), (best, Compat::Compatible)]), Some(best));
        assert_eq!(newest_compatible(&[(latest, bad)]), None);
    }
}

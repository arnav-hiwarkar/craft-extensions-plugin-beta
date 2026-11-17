use crate::{Badge, Loadable, NativeWarning, rank::View};
use craft_ext_core::{
    catalogue::PackageFile,
    id::{AppId, ExtensionId},
    listing::{Attestation, Category, Image},
    manifest::Manifest,
    permission::{Permission, warnings},
    tier::Tier,
    version::{Version, VersionReq},
};
use craft_ext_store::state::Quarantine;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionRow {
    pub version: Version,
    pub published: String,
    pub changelog: String,
    pub attestation: Option<Attestation>,
    pub installed: bool,
    pub active_in: Vec<AppId>,
    pub usable_in: Vec<AppId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionRow {
    pub permission: Permission,
    pub sentence: String,
    pub why: Option<String>,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributionRow {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantRow {
    pub permission: Permission,
    pub sentence: String,
    pub required: bool,
    pub granted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppPanel {
    pub app: AppId,
    pub active: Option<Version>,
    pub pending: Option<Version>,
    pub pin: Option<VersionReq>,
    pub update: Option<Version>,
    pub contributions: Vec<ContributionRow>,
    pub grants: Vec<GrantRow>,
    pub full_access: bool,
    pub quarantine: Option<Quarantine>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Detail {
    pub id: ExtensionId,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub vendor: String,
    pub vendor_url: Option<String>,
    pub tier: Tier,
    pub badges: Vec<Badge>,
    pub homepage: Option<String>,
    pub license: String,
    pub repo: String,
    pub categories: Vec<Category>,
    pub tags: Vec<String>,
    pub icon: Option<Image>,
    pub screenshots: Vec<Image>,
    pub downloads: u64,
    pub stars: u32,
    pub updated: String,
    pub versions: Vec<VersionRow>,
    pub versions_loading: bool,
    pub versions_error: Option<String>,
    pub permissions: Vec<PermissionRow>,
    pub permission_warnings: Vec<String>,
    pub native: Option<NativeWarning>,
    pub apps: Vec<AppPanel>,
    pub report_url: String,
}

fn permission_rows(m: &Manifest) -> Vec<PermissionRow> {
    let mut rows: BTreeMap<Permission, bool> = BTreeMap::new();
    for c in &m.contributions {
        for p in &c.permissions {
            rows.insert(p.clone(), true);
        }
        for p in &c.optional_permissions {
            rows.entry(p.clone()).or_insert(false);
        }
    }
    rows.into_iter()
        .map(|(p, required)| PermissionRow { sentence: p.sentence(), why: m.why.get(&p.to_string()).cloned(), permission: p, required })
        .collect()
}

fn app_panel(view: &View<'_>, id: &ExtensionId, app: &AppId) -> Option<AppPanel> {
    let setup = view.snapshot.state.apps.get(app)?.get(id)?;
    let manifest = setup.active.or(setup.pending).and_then(|v| view.manifest(id, v));
    let contributions = manifest
        .map(|m| m.contributions_for(app).map(|c| ContributionRow { id: c.id.clone(), label: c.label.clone(), enabled: setup.enabled.contains(&c.id) }).collect())
        .unwrap_or_default();
    let grants = manifest
        .map(|m| {
            let (required, optional) = m.permissions_for(app, |_| true);
            required
                .into_iter()
                .map(|p| (p, true))
                .chain(optional.into_iter().map(|p| (p, false)))
                .map(|(p, required)| GrantRow { sentence: p.sentence(), granted: setup.grants.iter().any(|g| g.covers(&p)), permission: p, required })
                .collect()
        })
        .unwrap_or_default();
    Some(AppPanel {
        app: app.clone(),
        active: setup.active,
        pending: setup.pending,
        pin: setup.pin,
        update: view.update(id, Some(app)),
        contributions,
        grants,
        full_access: setup.full_access,
        quarantine: setup.quarantine.clone(),
    })
}

pub(crate) fn build(view: &View<'_>, id: &ExtensionId, package: Option<&Loadable<Arc<PackageFile>>>, badges: Vec<Badge>) -> Option<Detail> {
    let entry = view.entry(id)?;
    let listed = view.snapshot.index.as_ref().and_then(|i| i.package(id).map(|p| (i, p)));
    let releases = match package {
        Some(Loadable::Ready(p)) => p.releases.as_slice(),
        _ => &[],
    };
    let mut versions: BTreeMap<Version, VersionRow> = BTreeMap::new();
    let usable_in = |m: &Manifest| view.config.apps.iter().filter(|t| m.compat(&t.app, &t.provided).usable()).map(|t| t.app.clone()).collect::<Vec<_>>();
    for r in releases {
        versions.insert(
            r.version,
            VersionRow {
                version: r.version,
                published: r.published.clone(),
                changelog: r.changelog.clone(),
                attestation: r.attestation.clone(),
                installed: false,
                active_in: vec![],
                usable_in: usable_in(&r.manifest),
            },
        );
    }
    for v in view.snapshot.state.packages.get(id).into_iter().flat_map(|p| p.releases.keys()) {
        let row = versions.entry(*v).or_insert_with(|| VersionRow {
            version: *v,
            published: String::new(),
            changelog: String::new(),
            attestation: None,
            installed: true,
            active_in: vec![],
            usable_in: view.manifest(id, *v).map(usable_in).unwrap_or_default(),
        });
        row.installed = true;
        row.active_in = view.config.apps.iter().filter(|t| view.active(&t.app, id) == Some(*v)).map(|t| t.app.clone()).collect();
    }
    let reference = view
        .installed_version(id)
        .and_then(|v| view.manifest(id, v))
        .or_else(|| releases.iter().max_by_key(|r| r.version).map(|r| &r.manifest));
    let permissions = reference.map(permission_rows).unwrap_or_default();
    let all: Vec<Permission> = permissions.iter().map(|p| p.permission.clone()).collect();
    let mut badges = badges;
    if releases.iter().max_by_key(|r| r.version).is_some_and(|r| r.attestation.as_ref().is_some_and(|a| a.verified)) {
        badges.push(Badge::BuiltFromSource);
    }
    Some(Detail {
        id: id.clone(),
        name: entry.name.to_string(),
        summary: entry.summary.to_string(),
        description: entry.description.to_string(),
        vendor: listed.and_then(|(i, p)| i.vendor(&p.vendor)).map_or_else(|| entry.vendor.to_string(), |v| v.name.clone()),
        vendor_url: listed.and_then(|(i, p)| i.vendor(&p.vendor)).and_then(|v| v.url.clone()),
        tier: entry.tier,
        badges,
        homepage: listed.and_then(|(_, p)| p.homepage.clone()),
        license: listed.map(|(_, p)| p.license.clone()).or_else(|| reference.map(|m| m.license.clone())).unwrap_or_default(),
        repo: listed.map(|(_, p)| p.repo.clone()).or_else(|| reference.map(|m| m.source.clone())).unwrap_or_default(),
        categories: entry.categories.to_vec(),
        tags: entry.tags.to_vec(),
        icon: entry.icon.cloned(),
        screenshots: listed.map(|(_, p)| p.screenshots.clone()).unwrap_or_default(),
        downloads: entry.downloads,
        stars: entry.stars,
        updated: entry.updated.to_string(),
        versions: versions.into_values().rev().collect(),
        versions_loading: matches!(package, Some(Loadable::Loading)),
        versions_error: match package {
            Some(Loadable::Failed(e)) => Some(e.clone()),
            _ => None,
        },
        permission_warnings: warnings(&all),
        permissions,
        native: reference.and_then(NativeWarning::of).or(entry.native.map(NativeWarning::from_mode)),
        apps: view.config.apps.iter().filter_map(|t| app_panel(view, id, &t.app)).collect(),
        report_url: craft_ext_store::manager::report_url(id, view.installed_version(id).or(listed.map(|(_, p)| p.latest)).unwrap_or(Version::new(0, 0, 0))),
    })
}

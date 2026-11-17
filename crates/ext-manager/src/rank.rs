use crate::{Badge, Config, Filters, Flow, Primary, Query, Row, Sort, Step, Tab, worker::Snapshot};
use craft_ext_core::{
    catalogue::{PackageSummary, Severity},
    id::{AppId, ExtensionId},
    listing::{Category, Image},
    manifest::{ContributionKind, Manifest, NativeMode},
    tier::Tier,
    version::Version,
};
use std::collections::{BTreeMap, BTreeSet};

pub const SCORE_EXACT_NAME: u32 = 100;
pub const SCORE_NAME_PREFIX: u32 = 90;
pub const SCORE_NAME_WORD: u32 = 80;
pub const SCORE_NAME_CONTAINS: u32 = 70;
pub const SCORE_SUMMARY: u32 = 50;
pub const SCORE_TAG: u32 = 40;
pub const SCORE_TAG_CONTAINS: u32 = 35;
pub const SCORE_DESCRIPTION: u32 = 20;
pub const MAX_DOWNLOAD_BOOST: u32 = 5;

pub(crate) struct Entry<'a> {
    pub id: &'a ExtensionId,
    pub name: &'a str,
    pub summary: &'a str,
    pub description: &'a str,
    pub tags: &'a [String],
    pub apps: &'a [AppId],
    pub kinds: Vec<ContributionKind>,
    pub categories: &'a [Category],
    pub vendor: &'a str,
    pub tier: Tier,
    pub native: Option<NativeMode>,
    pub icon: Option<&'a Image>,
    pub downloads: u64,
    pub stars: u32,
    pub updated: &'a str,
    pub listed: bool,
}

pub(crate) fn tier_boost(tier: Tier) -> u32 {
    match tier {
        Tier::Official => 3,
        Tier::Verified => 2,
        Tier::Community => 1,
        Tier::Local | Tier::Dev => 0,
    }
}

pub(crate) fn download_boost(downloads: u64) -> u32 {
    downloads.saturating_add(1).ilog10().min(MAX_DOWNLOAD_BOOST)
}

pub(crate) fn text_score(e: &Entry<'_>, needle: &str) -> Option<u32> {
    if needle.is_empty() {
        return Some(0);
    }
    let name = e.name.to_lowercase();
    let has = |s: &str| s.to_lowercase().contains(needle);
    Some(if name == needle || e.id.as_str() == needle {
        SCORE_EXACT_NAME
    } else if name.starts_with(needle) {
        SCORE_NAME_PREFIX
    } else if name.split_whitespace().any(|w| w.starts_with(needle)) {
        SCORE_NAME_WORD
    } else if name.contains(needle) {
        SCORE_NAME_CONTAINS
    } else if has(e.summary) {
        SCORE_SUMMARY
    } else if e.tags.iter().any(|t| t.to_lowercase() == needle) {
        SCORE_TAG
    } else if e.tags.iter().any(|t| has(t)) {
        SCORE_TAG_CONTAINS
    } else if has(e.description) {
        SCORE_DESCRIPTION
    } else {
        return None;
    })
}

fn listed<'a>(p: &'a PackageSummary, tier: Tier, manifest: Option<&Manifest>) -> Entry<'a> {
    Entry {
        id: &p.id,
        name: &p.name,
        summary: &p.summary,
        description: &p.description,
        tags: &p.tags,
        apps: &p.apps,
        kinds: p.kinds.clone(),
        categories: &p.categories,
        vendor: &p.vendor,
        tier,
        native: manifest.and_then(Manifest::native_mode).or(p.native.then_some(NativeMode::Full)),
        icon: p.icon.as_ref(),
        downloads: p.downloads.as_ref().map_or(0, |d| d.count),
        stars: p.stars,
        updated: &p.updated,
        listed: true,
    }
}

fn local<'a>(m: &'a Manifest, tier: Tier) -> Entry<'a> {
    Entry {
        id: &m.id,
        name: &m.name,
        summary: &m.summary,
        description: &m.description,
        tags: &m.tags,
        apps: &m.apps,
        kinds: m.contributions.iter().map(|c| c.kind).collect::<BTreeSet<_>>().into_iter().collect(),
        categories: &[],
        vendor: "",
        tier,
        native: m.native_mode(),
        icon: None,
        downloads: 0,
        stars: 0,
        updated: "",
        listed: false,
    }
}

pub(crate) struct View<'a> {
    pub snapshot: &'a Snapshot,
    pub config: &'a Config,
    pub flows: &'a BTreeMap<ExtensionId, Flow>,
}

impl<'a> View<'a> {
    pub fn scope(&self) -> Option<&'a AppId> {
        match self.config.apps.as_slice() {
            [one] => Some(&one.app),
            _ => None,
        }
    }

    pub fn newest_installed(&self, id: &ExtensionId) -> Option<Version> {
        self.snapshot.state.packages.get(id).and_then(|p| p.releases.keys().next_back().copied())
    }

    pub fn active(&self, app: &AppId, id: &ExtensionId) -> Option<Version> {
        self.snapshot.state.apps.get(app).and_then(|a| a.get(id)).and_then(|a| a.active)
    }

    pub fn installed_version(&self, id: &ExtensionId) -> Option<Version> {
        self.scope().and_then(|app| self.active(app, id)).or_else(|| self.newest_installed(id))
    }

    pub fn manifest(&self, id: &ExtensionId, v: Version) -> Option<&'a Manifest> {
        self.snapshot.manifests.get(&(id.clone(), v))
    }

    pub fn update(&self, id: &ExtensionId, app: Option<&AppId>) -> Option<Version> {
        self.snapshot.updates.iter().filter(|u| &u.id == id && app.is_none_or(|a| &u.app == a)).map(|u| u.to).max()
    }

    pub fn entry(&self, id: &ExtensionId) -> Option<Entry<'a>> {
        let installed = self.installed_version(id).and_then(|v| self.manifest(id, v));
        if let Some(index) = &self.snapshot.index
            && let Some(p) = index.package(id)
        {
            return Some(listed(p, index.tier_of(id), installed));
        }
        let v = self.newest_installed(id)?;
        let tier = self.snapshot.state.packages.get(id).and_then(|p| p.releases.get(&v)).map_or(Tier::Local, |r| r.tier);
        installed.or_else(|| self.manifest(id, v)).map(|m| local(m, tier))
    }

    fn killed(&self, p: &PackageSummary) -> bool {
        self.snapshot.index.as_ref().and_then(|i| i.revocation(&p.id, p.latest, "")).is_some_and(|r| r.severity == Severity::Disable)
    }

    pub fn badges(&self, e: &Entry<'_>) -> Vec<Badge> {
        let mut out = vec![Badge::Tier(e.tier)];
        match e.native {
            Some(NativeMode::Full) => out.push(Badge::NativeFull),
            Some(NativeMode::Restricted) => out.push(Badge::NativeRestricted),
            None => {}
        }
        if let Some(index) = &self.snapshot.index {
            if index.featured.contains(e.id) {
                out.push(Badge::Featured);
            }
            let advisory = index.package(e.id).and_then(|p| index.revocation(e.id, p.latest, "")).is_some_and(|r| r.severity == Severity::Advisory);
            if advisory {
                out.push(Badge::Advisory);
            }
        }
        out
    }

    fn primary(&self, e: &Entry<'_>, update: Option<Version>) -> Primary {
        match self.flows.get(e.id).map(|f| &f.step) {
            Some(Step::Setup(_)) => return Primary::SetUp,
            Some(Step::Working(_)) => return Primary::Working,
            None => {}
        }
        if let Some(v) = update {
            return Primary::Update(v);
        }
        if self.newest_installed(e.id).is_none() {
            return Primary::Install;
        }
        let set_up = self.scope().is_none_or(|app| self.snapshot.state.apps.get(app).and_then(|a| a.get(e.id)).is_some_and(|a| a.active.is_some() && !a.enabled.is_empty()));
        if set_up { Primary::Open } else { Primary::SetUp }
    }

    fn keep(&self, e: &Entry<'_>, f: &Filters) -> bool {
        f.app.as_ref().is_none_or(|a| e.apps.contains(a))
            && f.category.is_none_or(|c| e.categories.contains(&c))
            && f.tier.is_none_or(|t| t == e.tier)
            && f.kind.is_none_or(|k| e.kinds.contains(&k))
            && (!f.installed || self.newest_installed(e.id).is_some())
            && (!f.has_update || self.update(e.id, f.app.as_ref()).is_some())
    }

    pub fn rows(&self, tab: Tab, q: &Query) -> Vec<Row> {
        let ids: Vec<ExtensionId> = match tab {
            Tab::Discover => self.snapshot.index.iter().flat_map(|i| i.packages.iter()).filter(|p| !self.killed(p)).map(|p| p.id.clone()).collect(),
            Tab::Installed => self.snapshot.state.packages.keys().cloned().collect(),
            Tab::Updates => self.snapshot.updates.iter().map(|u| u.id.clone()).collect::<BTreeSet<_>>().into_iter().collect(),
            Tab::Problems => vec![],
        };
        let needle = q.text.trim().to_lowercase();
        let mut scored: Vec<(u32, Row)> = ids
            .iter()
            .filter_map(|id| self.entry(id))
            .filter(|e| self.keep(e, &q.filters))
            .filter_map(|e| {
                let text = text_score(&e, &needle)?;
                let update = self.update(e.id, q.filters.app.as_ref().or(self.scope()));
                let row = Row {
                    id: e.id.clone(),
                    name: e.name.to_string(),
                    summary: e.summary.to_string(),
                    vendor: e.vendor.to_string(),
                    tier: e.tier,
                    badges: self.badges(&e),
                    icon: e.icon.cloned(),
                    categories: e.categories.to_vec(),
                    installed: self.installed_version(e.id),
                    update,
                    downloads: e.downloads,
                    stars: e.stars,
                    updated: e.updated.to_string(),
                    listed: e.listed,
                    primary: self.primary(&e, update),
                };
                Some((text + tier_boost(e.tier) + download_boost(e.downloads), row))
            })
            .collect();
        let by_name = |a: &Row, b: &Row| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then_with(|| a.id.cmp(&b.id));
        match q.sort {
            Sort::Relevance => scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| by_name(&a.1, &b.1))),
            Sort::Popular => scored.sort_by(|a, b| b.1.downloads.cmp(&a.1.downloads).then(b.1.stars.cmp(&a.1.stars)).then_with(|| by_name(&a.1, &b.1))),
            Sort::RecentlyUpdated => scored.sort_by(|a, b| b.1.updated.cmp(&a.1.updated).then_with(|| by_name(&a.1, &b.1))),
            Sort::Name => scored.sort_by(|a, b| by_name(&a.1, &b.1)),
        }
        scored.into_iter().map(|(_, r)| r).collect()
    }
}

use crate::{Word, rank::View};
use craft_ext_core::{
    catalogue::Severity,
    id::{AppId, ExtensionId},
    manifest::Compat,
    version::Version,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProblemKind {
    Quarantined { reason: String },
    KillListed { reason: String },
    NoHelper,
    Incompatible { reasons: Vec<String> },
    ConsentPending { native: bool },
    UncleanExit,
}

impl ProblemKind {
    pub fn word(&self) -> Word {
        match self {
            ProblemKind::Quarantined { .. } => Word::ProblemQuarantined,
            ProblemKind::KillListed { .. } => Word::ProblemKillListed,
            ProblemKind::NoHelper => Word::ProblemNoHelper,
            ProblemKind::Incompatible { .. } => Word::ProblemIncompatible,
            ProblemKind::ConsentPending { native: true } => Word::ProblemNativeConsentPending,
            ProblemKind::ConsentPending { native: false } => Word::ProblemConsentPending,
            ProblemKind::UncleanExit => Word::ProblemUncleanExit,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    TurnBackOn,
    TurnOff,
    SetUp,
    Update(Version),
    Uninstall,
    Report,
}

impl Fix {
    pub fn word(self) -> Word {
        match self {
            Fix::TurnBackOn => Word::ActionTurnBackOn,
            Fix::TurnOff => Word::ActionDisable,
            Fix::SetUp => Word::ActionSetUp,
            Fix::Update(_) => Word::ActionUpdate,
            Fix::Uninstall => Word::ActionUninstall,
            Fix::Report => Word::ActionReport,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub app: AppId,
    pub id: ExtensionId,
    pub version: Option<Version>,
    pub kind: ProblemKind,
    pub fix: Option<Fix>,
}

pub(crate) fn find(view: &View<'_>, unclean: &BTreeMap<AppId, Vec<ExtensionId>>) -> Vec<Problem> {
    let mut out = vec![];
    let state = &view.snapshot.state;
    for target in &view.config.apps {
        let app = &target.app;
        for (id, setup) in state.apps.get(app).into_iter().flatten() {
            let problem = |version: Option<Version>, kind: ProblemKind, fix: Option<Fix>| Problem { app: app.clone(), id: id.clone(), version, kind, fix };
            let update = view.update(id, Some(app)).map(Fix::Update);
            if let Some(v) = setup.active {
                if let Some(q) = setup.quarantine.as_ref().filter(|q| q.version == v) {
                    out.push(problem(Some(v), ProblemKind::Quarantined { reason: q.reason.clone() }, Some(Fix::TurnBackOn)));
                }
                let package = state.packages.get(id);
                let record = package.and_then(|p| p.releases.get(&v));
                let vendor = package.and_then(|p| record.and_then(|r| r.origin.as_ref()).unwrap_or(&p.origin).vendor());
                let killed = record.and_then(|r| state.revocations.iter().filter(|x| x.severity == Severity::Disable).find(|x| x.matches(id, v, &r.sha256, vendor)));
                if let Some(r) = killed {
                    out.push(problem(Some(v), ProblemKind::KillListed { reason: r.reason.clone() }, Some(Fix::Uninstall)));
                }
                if let Some(m) = view.manifest(id, v) {
                    if m.native.as_ref().is_some_and(|n| !n.executables.contains_key(&view.config.platform)) {
                        out.push(problem(Some(v), ProblemKind::NoHelper, update));
                    } else if let Compat::Incompatible { reasons } = m.compat(app, &target.provided) {
                        out.push(problem(Some(v), ProblemKind::Incompatible { reasons }, update));
                    }
                }
            }
            if let Some(v) = setup.pending {
                let native = view.manifest(id, v).is_some_and(|m| m.is_native()) && !setup.full_access;
                out.push(problem(Some(v), ProblemKind::ConsentPending { native }, Some(Fix::SetUp)));
            }
        }
        for id in unclean.get(app).into_iter().flatten() {
            let setup = state.apps.get(app).and_then(|a| a.get(id));
            if setup.is_some_and(|s| s.quarantine.as_ref().is_some_and(|q| Some(q.version) == s.active)) {
                continue;
            }
            let version = view.active(app, id);
            out.push(Problem { app: app.clone(), id: id.clone(), version, kind: ProblemKind::UncleanExit, fix: version.map(|_| Fix::TurnOff) });
        }
    }
    out
}

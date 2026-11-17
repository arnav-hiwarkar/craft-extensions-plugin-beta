use crate::{Word, worker::SetupChoice};
use craft_ext_core::{
    contract::Provided,
    id::{AppId, ExtensionId},
    manifest::{Manifest, NativeMode},
    permission::{Permission, warnings},
    version::Version,
};
use craft_ext_store::state::AppSetup;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowKind {
    Install,
    Update,
    Rollback,
    InstallMissing,
}

impl FlowKind {
    pub fn done(self) -> Word {
        match self {
            FlowKind::Install | FlowKind::InstallMissing => Word::NoticeInstalled,
            FlowKind::Update => Word::NoticeUpdated,
            FlowKind::Rollback => Word::NoticeRolledBack,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeWarning {
    FullAccess,
    Restricted,
}

impl NativeWarning {
    pub fn of(m: &Manifest) -> Option<Self> {
        m.native_mode().map(Self::from_mode)
    }

    pub fn from_mode(mode: NativeMode) -> Self {
        match mode {
            NativeMode::Full => NativeWarning::FullAccess,
            NativeMode::Restricted => NativeWarning::Restricted,
        }
    }

    pub fn word(self) -> Word {
        match self {
            NativeWarning::FullAccess => Word::WarningNativeFull,
            NativeWarning::Restricted => Word::WarningNativeRestricted,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    Working(Word),
    Setup(Box<Setup>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Flow {
    pub app: AppId,
    pub kind: FlowKind,
    pub step: Step,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetupContribution {
    pub id: String,
    pub label: String,
    pub on: bool,
    pub unavailable: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetupPermission {
    pub permission: Permission,
    pub sentence: String,
    pub why: Option<String>,
    pub required: bool,
    pub on: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Setup {
    pub id: ExtensionId,
    pub app: AppId,
    pub version: Version,
    pub name: String,
    pub contributions: Vec<SetupContribution>,
    pub permissions: Vec<SetupPermission>,
    pub warnings: Vec<String>,
    pub native: Option<NativeWarning>,
    pub full_access: bool,
    pub platform_ok: bool,
    pub compatible: bool,
    pub dependencies: Vec<(ExtensionId, Version)>,
    manifest: Manifest,
}

impl Setup {
    pub(crate) fn new(manifest: Manifest, app: AppId, saved: Option<&AppSetup>, provided: &Provided, platform: &str, dependencies: Vec<(ExtensionId, Version)>) -> Self {
        let contributions = manifest
            .contributions_for(&app)
            .map(|c| {
                let unavailable = provided
                    .unsupported
                    .iter()
                    .find(|(kind, _)| *kind == c.kind)
                    .map(|(_, reason)| reason.clone())
                    .or_else(|| c.contract.check(provided).map(|r| r.to_string()));
                let on = unavailable.is_none() && saved.is_none_or(|s| s.enabled.contains(&c.id));
                SetupContribution { id: c.id.clone(), label: c.label.clone(), on, unavailable }
            })
            .collect();
        let mut setup = Setup {
            id: manifest.id.clone(),
            version: manifest.version,
            name: manifest.name.clone(),
            contributions,
            permissions: vec![],
            warnings: vec![],
            native: NativeWarning::of(&manifest),
            full_access: saved.is_some_and(|s| s.full_access && s.active == Some(manifest.version)) && manifest.is_native(),
            platform_ok: manifest.native.as_ref().is_none_or(|n| n.executables.contains_key(platform)),
            compatible: manifest.compat(&app, provided).usable(),
            dependencies,
            app,
            manifest,
        };
        setup.recompute(|p| saved.is_some_and(|s| s.grants.iter().any(|g| g.covers(p))));
        setup
    }

    fn recompute(&mut self, was_on: impl Fn(&Permission) -> bool) {
        let (required, optional) = self.manifest.permissions_for(&self.app, |c| self.contributions.iter().any(|x| x.id == c && x.on));
        let all: Vec<Permission> = required.iter().chain(&optional).cloned().collect();
        self.warnings = warnings(&all);
        self.permissions = required
            .into_iter()
            .map(|p| (p, true))
            .chain(optional.into_iter().map(|p| (p, false)))
            .map(|(p, required)| SetupPermission {
                sentence: p.sentence(),
                why: self.manifest.why.get(&p.to_string()).cloned(),
                on: was_on(&p),
                permission: p,
                required,
            })
            .collect();
    }

    pub(crate) fn set_contribution(&mut self, id: &str, on: bool) {
        if let Some(c) = self.contributions.iter_mut().find(|c| c.id == id && c.unavailable.is_none()) {
            c.on = on;
        }
        let before: Vec<Permission> = self.permissions.iter().filter(|p| p.on).map(|p| p.permission.clone()).collect();
        self.recompute(|p| before.contains(p));
    }

    pub(crate) fn set_permission(&mut self, p: &Permission, on: bool) {
        if let Some(row) = self.permissions.iter_mut().find(|r| &r.permission == p) {
            row.on = on;
        }
    }

    pub(crate) fn set_full_access(&mut self, on: bool) {
        self.full_access = on && self.native.is_some();
    }

    pub fn blocker(&self) -> Option<Word> {
        if !self.compatible {
            Some(Word::SetupIncompatible)
        } else if !self.platform_ok {
            Some(Word::SetupNoHelper)
        } else if !self.contributions.iter().any(|c| c.on) {
            Some(Word::SetupChooseContribution)
        } else if self.permissions.iter().any(|p| p.required && !p.on) {
            Some(Word::SetupGrantRequired)
        } else if self.native.is_some() && !self.full_access {
            Some(Word::SetupConsentNative)
        } else {
            None
        }
    }

    pub fn ready(&self) -> bool {
        self.blocker().is_none()
    }

    pub(crate) fn choice(&self) -> SetupChoice {
        SetupChoice {
            id: self.id.clone(),
            app: self.app.clone(),
            version: self.version,
            contributions: self.contributions.iter().filter(|c| c.on).map(|c| c.id.clone()).collect(),
            grants: self.permissions.iter().filter(|p| p.on).map(|p| p.permission.clone()).collect(),
            full_access: self.full_access,
        }
    }
}

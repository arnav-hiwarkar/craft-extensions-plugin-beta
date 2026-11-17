use crate::{Action, Badge, Model, NativeWarning, Primary, Problem, ProblemKind, Row, Step, Word, problems::Fix};
use craft_ext_core::{
    id::AppId,
    listing::{CATEGORIES, Category},
    project::{PluginRef, Status},
    tier::Tier,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Accent,
    Info,
    Good,
    Warn,
    Bad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    Primary,
    Secondary,
    Busy,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Press {
    Do(Vec<Action>),
    ChooseApp(Vec<AppId>),
    Open,
    Nothing,
}

impl Badge {
    pub fn tone(self) -> Tone {
        match self {
            Badge::Tier(Tier::Official) => Tone::Accent,
            Badge::Tier(Tier::Verified) | Badge::BuiltFromSource => Tone::Good,
            Badge::Tier(Tier::Community) => Tone::Neutral,
            Badge::Tier(Tier::Local | Tier::Dev) | Badge::NativeRestricted | Badge::Advisory => Tone::Warn,
            Badge::NativeFull => Tone::Bad,
            Badge::Featured => Tone::Info,
        }
    }
}

impl Primary {
    pub fn look(self) -> Look {
        match self {
            Primary::Install | Primary::Update(_) | Primary::SetUp => Look::Primary,
            Primary::Open => Look::Secondary,
            Primary::Working => Look::Busy,
        }
    }
}

impl ProblemKind {
    pub fn tone(&self) -> Tone {
        match self {
            ProblemKind::KillListed { .. } | ProblemKind::NoHelper => Tone::Bad,
            ProblemKind::Quarantined { .. } | ProblemKind::Incompatible { .. } | ProblemKind::UncleanExit => Tone::Warn,
            ProblemKind::ConsentPending { .. } => Tone::Info,
        }
    }
}

impl NativeWarning {
    pub fn tone(self) -> Tone {
        match self {
            NativeWarning::FullAccess => Tone::Bad,
            NativeWarning::Restricted => Tone::Warn,
        }
    }
}

impl Problem {
    pub fn action(&self) -> Option<Action> {
        let (id, app) = (self.id.clone(), self.app.clone());
        Some(match self.fix? {
            Fix::TurnBackOn => Action::TurnBackOn { id, app },
            Fix::TurnOff => Action::TurnOffAfterCrash { id, app },
            Fix::SetUp => Action::SetUp { id, app },
            Fix::Update(_) => Action::Update { id, app },
            Fix::Uninstall => Action::Uninstall { id },
            Fix::Report => Action::Report { id },
        })
    }
}

pub fn category_word(category: Category) -> Word {
    match category {
        Category::EffectsFilters => Word::CategoryEffectsFilters,
        Category::Generators => Word::CategoryGenerators,
        Category::ColorLooks => Word::CategoryColorLooks,
        Category::TextTitles => Word::CategoryTextTitles,
        Category::MotionAnimation => Word::CategoryMotionAnimation,
        Category::ThreeDParticles => Word::CategoryThreeDParticles,
        Category::Utilities => Word::CategoryUtilities,
        Category::PanelsWorkflow => Word::CategoryPanelsWorkflow,
        Category::ImportExport => Word::CategoryImportExport,
        Category::AiAnalysis => Word::CategoryAiAnalysis,
        Category::Resources => Word::CategoryResources,
    }
}

pub fn project_word(status: &Status) -> Word {
    match status {
        Status::Ready { .. } => Word::ProjectReady,
        Status::NeedsEnable { .. } => Word::ProjectNeedsSetUp,
        Status::NeedsOtherVersion { .. } => Word::ProjectNeedsOtherVersion,
        Status::Downloadable => Word::ProjectDownloadable,
        Status::Unavailable { .. } => Word::ProjectUnavailable,
    }
}

pub fn project_tone(status: &Status) -> Tone {
    match status {
        Status::Ready { .. } => Tone::Good,
        Status::NeedsEnable { .. } | Status::Downloadable => Tone::Info,
        Status::NeedsOtherVersion { .. } => Tone::Warn,
        Status::Unavailable { .. } => Tone::Bad,
    }
}

pub fn project_fix(app: &AppId, plugin: &PluginRef, status: &Status) -> Option<(Word, Action)> {
    match status {
        Status::Downloadable | Status::NeedsOtherVersion { .. } => Some((Word::ActionInstall, Action::InstallMissing { app: app.clone(), refs: vec![plugin.clone()] })),
        Status::NeedsEnable { .. } => Some((Word::ActionSetUp, Action::SetUp { id: plugin.id.clone(), app: app.clone() })),
        Status::Ready { .. } | Status::Unavailable { .. } => None,
    }
}

impl Model {
    pub fn categories(&self) -> Vec<Category> {
        CATEGORIES.iter().filter(|r| self.config().apps.iter().any(|t| r.apps.contains(&t.app.as_str()))).map(|r| r.category).collect()
    }

    pub fn name_of(&self, id: &craft_ext_core::id::ExtensionId) -> String {
        self.view().entry(id).map_or_else(|| id.to_string(), |e| e.name.to_string())
    }

    pub fn install_targets(&self, row: &Row) -> Vec<AppId> {
        let supported = self.view().entry(&row.id).map(|e| e.apps.to_vec()).unwrap_or_default();
        self.config().apps.iter().map(|t| &t.app).filter(|a| supported.contains(a)).cloned().collect()
    }

    pub fn press(&self, row: &Row) -> Press {
        match row.primary {
            Primary::Install => match self.install_targets(row).as_slice() {
                [] => Press::Do(vec![Action::Install { id: row.id.clone(), app: None }]),
                [one] => Press::Do(vec![Action::Install { id: row.id.clone(), app: Some(one.clone()) }]),
                many => Press::ChooseApp(many.to_vec()),
            },
            Primary::Update(_) => {
                let only = self.query().filters.app.as_ref();
                let mut apps: Vec<&AppId> = self.snapshot().updates.iter().filter(|u| u.id == row.id && only.is_none_or(|a| a == &u.app)).map(|u| &u.app).collect();
                apps.dedup();
                Press::Do(apps.into_iter().map(|app| Action::Update { id: row.id.clone(), app: app.clone() }).collect())
            }
            Primary::SetUp if self.flow(&row.id).is_some_and(|f| matches!(f.step, Step::Setup(_))) => Press::Nothing,
            Primary::SetUp => match self.setup_target(row) {
                Some(app) => Press::Do(vec![Action::SetUp { id: row.id.clone(), app }]),
                None => Press::Open,
            },
            Primary::Open => Press::Open,
            Primary::Working => Press::Nothing,
        }
    }

    fn setup_target(&self, row: &Row) -> Option<AppId> {
        let state = &self.snapshot().state;
        let set_up = |app: &AppId| state.apps.get(app).and_then(|a| a.get(&row.id)).is_some_and(|a| a.active.is_some() && !a.enabled.is_empty());
        self.install_targets(row).into_iter().find(|app| !set_up(app)).or_else(|| self.config().scope().cloned())
    }

    pub fn pending_setup(&self) -> Option<&crate::Setup> {
        self.flows.values().find_map(|f| match &f.step {
            Step::Setup(s) => Some(&**s),
            Step::Working(_) => None,
        })
    }
}

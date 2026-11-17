use crate::{Action, Config, Model, Notice, Word, project_fix};
use craft_ext_core::{
    id::{AppId, ExtensionId},
    project::{PluginRef, Status},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptOutcome {
    pub skipped: Vec<ExtensionId>,
    pub open_without: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prompt {
    app: Option<AppId>,
    statuses: Vec<(PluginRef, Status)>,
    skipped: BTreeSet<ExtensionId>,
    results: BTreeMap<ExtensionId, Notice>,
}

impl Prompt {
    pub fn new(config: &Config, statuses: Vec<(PluginRef, Status)>) -> Self {
        let app = config.scope().cloned().or_else(|| config.apps.first().map(|t| t.app.clone()));
        Prompt { app, statuses, skipped: BTreeSet::new(), results: BTreeMap::new() }
    }

    pub fn record(&mut self, notices: impl IntoIterator<Item = Notice>) {
        for notice in notices {
            if let Some(plugin) = notice.plugin.clone().filter(|p| self.statuses.iter().any(|(r, _)| &r.id == p)) {
                self.results.insert(plugin, notice);
            }
        }
    }

    pub fn shown(&self) -> impl Iterator<Item = &(PluginRef, Status)> {
        self.statuses.iter().filter(|(r, _)| !self.skipped.contains(&r.id))
    }

    pub fn result(&self, id: &ExtensionId) -> Option<&Notice> {
        self.results.get(id)
    }

    pub fn fix(&self, model: &Model, plugin: &PluginRef, status: &Status) -> Option<(Word, Action)> {
        let settled = self.results.contains_key(&plugin.id) || model.flow(&plugin.id).is_some();
        self.app.as_ref().filter(|_| !settled).and_then(|app| project_fix(app, plugin, status))
    }

    pub fn install_all(&self, model: &Model) -> Option<Action> {
        let refs: Vec<PluginRef> = self.shown().filter(|(r, s)| matches!(self.fix(model, r, s), Some((_, Action::InstallMissing { .. })))).map(|(r, _)| r.clone()).collect();
        let app = self.app.clone().filter(|_| refs.len() > 1)?;
        Some(Action::InstallMissing { app, refs })
    }

    pub fn skip(&mut self, id: ExtensionId) {
        self.skipped.insert(id);
    }

    pub fn finish(&self, open_without: bool) -> PromptOutcome {
        let skipped = if open_without {
            self.statuses.iter().filter(|(_, s)| !s.ready()).map(|(r, _)| r.id.clone()).collect()
        } else {
            self.skipped.iter().cloned().collect()
        };
        PromptOutcome { skipped, open_without }
    }
}

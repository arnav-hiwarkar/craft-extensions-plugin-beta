#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]
mod detail;
mod look;
mod manager;
mod prompt;
mod setup;

pub use look::{badge_variant, button_variant, label, notice_tone, tone};
pub use craft_ext_manager::PromptOutcome;
pub use manager::{PluginManager, PluginManagerOutput};
pub use prompt::{ProjectPrompt, PromptState};

use craft_ext_core::id::{AppId, ExtensionId};
use craft_ext_manager::{Action, Config, Effect, Model, Notice};

pub struct ManagerState {
    model: Model,
    search: String,
    notices: Vec<Notice>,
    links: Vec<String>,
    choosing: Option<(ExtensionId, Vec<AppId>)>,
}

impl ManagerState {
    pub fn new(config: Config) -> Self {
        ManagerState { model: Model::new(config), search: String::new(), notices: vec![], links: vec![], choosing: None }
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn apply(&mut self, action: Action) {
        let effects = self.model.apply(action);
        self.take(effects);
    }

    pub fn set_unclean_exit(&mut self, app: AppId, ids: Vec<ExtensionId>) {
        let effects = self.model.set_unclean_exit(app, ids);
        self.take(effects);
    }

    fn poll(&mut self) {
        let effects = self.model.poll();
        self.take(effects);
    }

    fn take(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::Notice(n) => self.notices.push(n),
                Effect::OpenUrl(url) => self.links.push(url),
                Effect::Changed | Effect::OpenSetup(_) | Effect::CloseSetup(_) => {}
            }
        }
    }
}

use crate::look::{button_variant, label, notice_tone, tone};
use crate::manager::Pending;
use crate::setup;
use craft::ui::buttons::Button;
use craft::ui::contract::{Bind, Props as _, Show as _, ShowContainer as _};
use craft::ui::dialog::Dialog;
use craft::ui::display::{Shade, Spinner, Text};
use craft::ui::layout_components::{Row, Stack};
use craft::ui::runtime::env::Label;
use craft::ui::sections::{DialogRole, FooterButton};
use craft::ui::style::TypeRole;
use craft_ext_core::id::AppId;
use craft_ext_core::project::{PluginRef, Status};
use craft_ext_manager::{Config, Effect, Look, Model, Prompt, PromptOutcome, Step, Word, project_tone, project_word};
use egui::Ui;
use std::cell::RefCell;
use std::time::Duration;

const OPEN_WITHOUT: usize = 0;
const CONTINUE: usize = 1;
const REPAINT_WHILE_BUSY: Duration = Duration::from_millis(100);

pub struct PromptState {
    model: Model,
    prompt: Prompt,
}

impl PromptState {
    pub fn new(config: Config, statuses: Vec<(PluginRef, Status)>) -> Self {
        let prompt = Prompt::new(&config, statuses);
        PromptState { model: Model::new(config), prompt }
    }

    fn take(&mut self, effects: Vec<Effect>) {
        self.prompt.record(effects.into_iter().filter_map(|e| match e {
            Effect::Notice(n) => Some(n),
            Effect::Changed | Effect::OpenSetup(_) | Effect::CloseSetup(_) | Effect::OpenUrl(_) => None,
        }));
    }
}

pub struct ProjectPrompt<'a> {
    state: &'a mut PromptState,
    app_name: Box<dyn Fn(&AppId) -> Label + 'a>,
}

impl<'a> ProjectPrompt<'a> {
    pub fn new(state: &'a mut PromptState) -> Self {
        ProjectPrompt { state, app_name: Box::new(|app| Label::from(app.to_string())) }
    }

    pub fn app_names(mut self, name: impl Fn(&AppId) -> Label + 'a) -> Self {
        self.app_name = Box::new(name);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Option<PromptOutcome> {
        let ProjectPrompt { state, app_name } = self;
        let effects = state.model.poll();
        state.take(effects);
        if state.model.busy() {
            ui.ctx().request_repaint_after(REPAINT_WHILE_BUSY);
        }
        let pending = RefCell::new(Pending::default());
        let mut skip = vec![];
        let mut outcome = None;
        if state.model.pending_setup().is_some() {
            setup::show(ui, &state.model, &*app_name, &pending);
        } else {
            let (model, prompt) = (&state.model, &state.prompt);
            let mut open = true;
            let shown = Dialog::new(label(Word::PromptTitle))
                .id_salt("craft-project-plugins")
                .opened(Bind::bound(&mut open))
                .buttons([FooterButton::new(label(Word::ActionOpenWithout), DialogRole::Cancel), FooterButton::new(label(Word::ActionContinue), DialogRole::Accept)])
                .show(ui, |ui, _| {
                    for (plugin, status) in prompt.shown() {
                        Stack::new().id_salt(("project-plugin", plugin.id.as_str())).show(ui, |ui, _| {
                            Row::new().wrap(true).show(ui, |ui, _| {
                                let name = if plugin.name.is_empty() { plugin.id.to_string() } else { plugin.name.clone() };
                                Text::new(Label::from(name)).role(TypeRole::BodyStrong).show(ui);
                                Text::new(Label::from(plugin.version.to_string())).shade(Shade::Muted).show(ui);
                                if let Some((word, action)) = prompt.fix(model, plugin, status)
                                    && Button::new(label(word)).variant(button_variant(Look::Primary)).id_salt("fix").show(ui).clicked()
                                {
                                    pending.borrow_mut().actions.push(action);
                                }
                                if Button::new(label(Word::ActionSkip)).id_salt("skip").show(ui).clicked() {
                                    skip.push(plugin.id.clone());
                                }
                            });
                            plugin_state(ui, model, prompt, plugin, status);
                        });
                    }
                    if let Some(action) = prompt.install_all(model)
                        && Button::new(label(Word::ActionInstallMissing)).variant(button_variant(Look::Primary)).id_salt("install-all").show(ui).clicked()
                    {
                        pending.borrow_mut().actions.push(action);
                    }
                    Text::new(label(Word::PromptKeepsData)).role(TypeRole::Caption).shade(Shade::Faint).show(ui);
                });
            outcome = match (shown.output.picked, shown.output.closed, open) {
                (Some(OPEN_WITHOUT), _, _) => Some(state.prompt.finish(true)),
                (Some(CONTINUE), _, _) | (None, Some(_), _) | (None, None, false) => Some(state.prompt.finish(false)),
                _ => None,
            };
        }
        for id in skip {
            state.prompt.skip(id);
        }
        for action in pending.into_inner().actions {
            let effects = state.model.apply(action);
            state.take(effects);
        }
        outcome
    }
}

fn plugin_state(ui: &mut Ui, model: &Model, prompt: &Prompt, plugin: &PluginRef, status: &Status) {
    if let Some(notice) = prompt.result(&plugin.id) {
        Text::new(label(notice.word)).tone(notice_tone(notice.error)).show(ui);
        if !notice.detail.is_empty() {
            Text::new(Label::from(notice.detail.clone())).shade(Shade::Faint).show(ui);
        }
        return;
    }
    if let Some(flow) = model.flow(&plugin.id) {
        let word = match &flow.step {
            Step::Working(word) => *word,
            Step::Setup(_) => Word::SetupTitle,
        };
        Spinner::new().label(label(word)).show(ui);
        return;
    }
    Text::new(label(project_word(status))).tone(tone(project_tone(status))).show(ui);
    let reason = match status {
        Status::NeedsEnable { reason, .. } | Status::NeedsOtherVersion { reason, .. } | Status::Unavailable { reason } => reason.as_str(),
        Status::Ready { .. } | Status::Downloadable => "",
    };
    if !reason.is_empty() {
        Text::new(Label::from(reason.to_owned())).shade(Shade::Faint).show(ui);
    }
}

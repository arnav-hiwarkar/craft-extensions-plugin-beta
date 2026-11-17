use super::{
    Session,
    look::{Words, notice_tone},
    widgets::{action_button, toned, weak},
};
use craft_ext_core::project::{PluginRef, Status};
use craft_ext_manager::{Config, Flow, Look, Prompt, PromptOutcome, Step, Word, project_tone, project_word};
use egui::{Align, Context, CornerRadius, Frame, Id, Layout, Margin, Modal, RichText, ScrollArea, Ui};

pub(crate) const PROMPT_WIDTH: f32 = 480.0;

pub struct ProjectPrompt {
    session: Session,
    prompt: Prompt,
    closed: bool,
}

impl ProjectPrompt {
    pub fn new(config: Config, statuses: Vec<(PluginRef, Status)>) -> Self {
        let prompt = Prompt::new(&config, statuses);
        ProjectPrompt { session: Session::new(config), prompt, closed: false }
    }

    pub fn show(&mut self, ctx: &Context, words: &dyn Words) -> Option<PromptOutcome> {
        if self.closed {
            return None;
        }
        let notices = self.session.poll(ctx, words);
        self.prompt.record(notices);
        let mut actions = vec![];
        let mut outcome = None;
        if self.session.model.pending_setup().is_none() {
            Modal::new(Id::unique("craft-project-plugins")).show(ctx, |ui| outcome = self.sheet(ui, words, &mut actions));
        }
        let notices = self.session.apply(actions, ctx, words);
        self.prompt.record(notices);
        let notices = self.session.overlays(ctx, words);
        self.prompt.record(notices);
        if outcome.is_some() {
            self.closed = true;
        }
        outcome
    }

    fn sheet(&mut self, ui: &mut Ui, words: &dyn Words, actions: &mut Vec<craft_ext_manager::Action>) -> Option<PromptOutcome> {
        ui.set_width(PROMPT_WIDTH);
        ui.heading(words.word(Word::PromptTitle));
        ui.add_space(8.0);
        let max = ui.ctx().content_rect().height() * 0.55;
        let mut skip_now = vec![];
        let model = &self.session.model;
        ScrollArea::vertical().max_height(max).auto_shrink([false, true]).show(ui, |ui| {
            for (plugin, status) in self.prompt.shown() {
                let fix = self.prompt.fix(model, plugin, status);
                let skip = Frame::new()
                    .fill(ui.visuals().faint_bg_color)
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(10))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let skip = ui.button(words.word(Word::ActionSkip)).clicked();
                            if let Some((word, action)) = fix
                                && action_button(ui, words.word(word), Look::Primary).clicked()
                            {
                                actions.push(action);
                            }
                            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(if plugin.name.is_empty() { plugin.id.as_str() } else { &plugin.name }).strong());
                                    ui.label(weak(ui, plugin.version.to_string()).small());
                                });
                                self.state(ui, plugin, status, words);
                            });
                            skip
                        })
                        .inner
                    })
                    .inner;
                if skip {
                    skip_now.push(plugin.id.clone());
                }
                ui.add_space(6.0);
            }
        });
        for id in skip_now {
            self.prompt.skip(id);
        }
        if let Some(action) = self.prompt.install_all(&self.session.model)
            && action_button(ui, words.word(Word::ActionInstallMissing), Look::Primary).clicked()
        {
            actions.push(action);
        }
        ui.add_space(8.0);
        ui.label(weak(ui, words.word(Word::PromptKeepsData)).small());
        ui.add_space(6.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if action_button(ui, words.word(Word::ActionContinue), Look::Primary).clicked() {
                return Some(self.prompt.finish(false));
            }
            if action_button(ui, words.word(Word::ActionOpenWithout), Look::Secondary).clicked() {
                return Some(self.prompt.finish(true));
            }
            None
        })
        .inner
    }

    fn state(&self, ui: &mut Ui, plugin: &PluginRef, status: &Status, words: &dyn Words) {
        if let Some(notice) = self.prompt.result(&plugin.id) {
            ui.label(toned(ui, words.word(notice.word), notice_tone(notice)));
            if !notice.detail.is_empty() {
                ui.label(weak(ui, &notice.detail).small());
            }
            return;
        }
        if let Some(Flow { step, .. }) = self.session.model.flow(&plugin.id) {
            let word = match step {
                Step::Working(word) => *word,
                Step::Setup(_) => Word::SetupTitle,
            };
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(weak(ui, words.word(word)));
            });
            return;
        }
        ui.label(toned(ui, words.word(project_word(status)), project_tone(status)));
        let reason = match status {
            Status::NeedsEnable { reason, .. } | Status::NeedsOtherVersion { reason, .. } | Status::Unavailable { reason } => reason.as_str(),
            Status::Ready { .. } | Status::Downloadable => "",
        };
        if !reason.is_empty() {
            ui.label(weak(ui, reason).small());
        }
    }
}

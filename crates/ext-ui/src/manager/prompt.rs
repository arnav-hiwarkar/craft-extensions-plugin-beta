use super::{
    Session,
    look::{Words, notice_tone},
    widgets::{action_button, toned, weak},
};
use craft_ext_core::{
    id::{AppId, ExtensionId},
    project::{PluginRef, Status},
};
use craft_ext_manager::{Action, Config, Flow, Look, Notice, Step, Word, project_fix, project_tone, project_word};
use egui::{Align, Context, CornerRadius, Frame, Id, Layout, Margin, Modal, RichText, ScrollArea, Ui};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PROMPT_WIDTH: f32 = 480.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptOutcome {
    pub skipped: Vec<ExtensionId>,
    pub open_without: bool,
}

pub struct ProjectPrompt {
    session: Session,
    app: Option<AppId>,
    statuses: Vec<(PluginRef, Status)>,
    skipped: BTreeSet<ExtensionId>,
    results: BTreeMap<String, Notice>,
    closed: bool,
}

impl ProjectPrompt {
    pub fn new(config: Config, statuses: Vec<(PluginRef, Status)>) -> Self {
        let app = config.scope().cloned().or_else(|| config.apps.first().map(|t| t.app.clone()));
        ProjectPrompt { session: Session::new(config), app, statuses, skipped: BTreeSet::new(), results: BTreeMap::new(), closed: false }
    }

    pub fn show(&mut self, ctx: &Context, words: &dyn Words) -> Option<PromptOutcome> {
        if self.closed {
            return None;
        }
        let notices = self.session.poll(ctx, words);
        self.record(notices);
        let mut actions = vec![];
        let mut outcome = None;
        if self.session.model.pending_setup().is_none() {
            Modal::new(Id::unique("craft-project-plugins")).show(ctx, |ui| outcome = self.sheet(ui, words, &mut actions));
        }
        let notices = self.session.apply(actions, ctx, words);
        self.record(notices);
        let notices = self.session.overlays(ctx, words);
        self.record(notices);
        if outcome.is_some() {
            self.closed = true;
        }
        outcome
    }

    fn record(&mut self, notices: Vec<Notice>) {
        for notice in notices {
            if self.statuses.iter().any(|(r, _)| r.id.as_str() == notice.subject) {
                self.results.insert(notice.subject.clone(), notice);
            }
        }
    }

    fn fix(&self, plugin: &PluginRef, status: &Status) -> Option<(Word, Action)> {
        let settled = self.results.contains_key(plugin.id.as_str()) || self.session.model.flow(&plugin.id).is_some();
        self.app.as_ref().filter(|_| !settled).and_then(|app| project_fix(app, plugin, status))
    }

    fn sheet(&mut self, ui: &mut Ui, words: &dyn Words, actions: &mut Vec<Action>) -> Option<PromptOutcome> {
        ui.set_width(PROMPT_WIDTH);
        ui.heading(words.word(Word::PromptTitle));
        ui.add_space(8.0);
        let max = ui.ctx().content_rect().height() * 0.55;
        let mut skip_now = vec![];
        ScrollArea::vertical().max_height(max).auto_shrink([false, true]).show(ui, |ui| {
            for (plugin, status) in self.statuses.iter().filter(|(r, _)| !self.skipped.contains(&r.id)) {
                let fix = self.fix(plugin, status);
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
        self.skipped.extend(skip_now);
        let missing: Vec<PluginRef> = self
            .statuses
            .iter()
            .filter(|(r, s)| !self.skipped.contains(&r.id) && matches!(self.fix(r, s), Some((_, Action::InstallMissing { .. }))))
            .map(|(r, _)| r.clone())
            .collect();
        if let Some(app) = self.app.clone().filter(|_| missing.len() > 1)
            && action_button(ui, words.word(Word::ActionInstallMissing), Look::Primary).clicked()
        {
            actions.push(Action::InstallMissing { app, refs: missing });
        }
        ui.add_space(8.0);
        ui.label(weak(ui, words.word(Word::PromptKeepsData)).small());
        ui.add_space(6.0);
        let skipped = || self.skipped.iter().cloned().collect();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if action_button(ui, words.word(Word::ActionContinue), Look::Primary).clicked() {
                return Some(PromptOutcome { skipped: skipped(), open_without: false });
            }
            if action_button(ui, words.word(Word::ActionOpenWithout), Look::Secondary).clicked() {
                return Some(PromptOutcome { skipped: skipped(), open_without: true });
            }
            None
        })
        .inner
    }

    fn state(&self, ui: &mut Ui, plugin: &PluginRef, status: &Status, words: &dyn Words) {
        if let Some(notice) = self.results.get(plugin.id.as_str()) {
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

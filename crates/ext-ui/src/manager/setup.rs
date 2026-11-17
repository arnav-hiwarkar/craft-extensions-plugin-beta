use super::{
    look::{Words, tone_color},
    widgets::{action_button, badge, callout, section, toned, weak},
};
use craft_ext_manager::{Action, Look, Setup, Tone, Word};
use egui::{Context, Id, Modal, RichText, ScrollArea};

pub(crate) const SHEET_WIDTH: f32 = 460.0;

pub(crate) fn sheet(ctx: &Context, setup: &Setup, words: &dyn Words) -> Vec<Action> {
    let mut actions = vec![];
    let id = &setup.id;
    let response = Modal::new(Id::unique(("craft-plugin-setup", id.as_str()))).show(ctx, |ui| {
        ui.set_width(SHEET_WIDTH);
        ui.label(weak(ui, words.word(Word::SetupTitle)).small());
        ui.heading(&setup.name);
        ui.label(weak(ui, format!("{} {} · {}", words.word(Word::LabelVersion), setup.version, words.app(&setup.app))));
        ui.add_space(4.0);
        ScrollArea::vertical().max_height(ctx.content_rect().height() * 0.6).auto_shrink([false, true]).show(ui, |ui| {
            if !setup.dependencies.is_empty() {
                section(ui, words.word(Word::SetupAlsoInstalls));
                for (dependency, version) in &setup.dependencies {
                    ui.label(format!("{dependency} {version}"));
                }
            }
            section(ui, words.word(Word::SectionSetupParts));
            for c in &setup.contributions {
                let mut on = c.on;
                let changed = ui.add_enabled(c.unavailable.is_none(), egui::Checkbox::new(&mut on, &c.label)).changed();
                if changed {
                    actions.push(Action::SetupContribution { id: id.clone(), contribution: c.id.clone(), on });
                }
                if let Some(why) = &c.unavailable {
                    ui.indent(("unavailable", &c.id), |ui| ui.label(weak(ui, why).small()));
                }
            }
            if !setup.permissions.is_empty() {
                section(ui, words.word(Word::SectionSetupAccess));
                for p in &setup.permissions {
                    ui.horizontal_wrapped(|ui| {
                        let mut on = p.on;
                        if ui.checkbox(&mut on, &p.sentence).changed() {
                            actions.push(Action::SetupPermission { id: id.clone(), permission: p.permission.clone(), on });
                        }
                        if p.required {
                            badge(ui, words.word(Word::PermissionRequired), Tone::Warn);
                        } else {
                            badge(ui, words.word(Word::PermissionOptional), Tone::Neutral);
                        }
                    });
                    if let Some(why) = &p.why {
                        ui.indent(("why", p.permission.to_string()), |ui| ui.label(weak(ui, format!("{}: “{why}”", words.word(Word::LabelAuthorSays))).italics().small()));
                    }
                }
            }
            if !setup.warnings.is_empty() {
                ui.add_space(8.0);
                for warning in &setup.warnings {
                    ui.label(toned(ui, warning, Tone::Warn));
                }
            }
            if let Some(native) = setup.native {
                ui.add_space(10.0);
                callout(ui, native.tone(), words.word(native.word()), None);
                let mut full = setup.full_access;
                let consent = RichText::new(words.word(Word::ConsentFullAccess)).color(tone_color(native.tone(), ui.visuals()));
                if ui.checkbox(&mut full, consent).changed() {
                    actions.push(Action::SetupFullAccess { id: id.clone(), on: full });
                }
            }
        });
        ui.add_space(10.0);
        let blocker = setup.blocker();
        match blocker {
            Some(word) => ui.label(toned(ui, words.word(word), Tone::Warn)),
            None => ui.label(weak(ui, words.word(Word::SetupNothingChanges)).small()),
        };
        ui.add_space(6.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let confirm = ui.add_enabled_ui(blocker.is_none(), |ui| action_button(ui, words.word(Word::ActionConfirmSetup), Look::Primary)).inner;
            if confirm.clicked() {
                actions.push(Action::ConfirmSetup { id: id.clone() });
            }
            if action_button(ui, words.word(Word::ActionCancel), Look::Secondary).clicked() {
                actions.push(Action::Cancel { id: id.clone() });
            }
        });
    });
    if response.should_close() && actions.is_empty() {
        actions.push(Action::Cancel { id: id.clone() });
    }
    actions
}

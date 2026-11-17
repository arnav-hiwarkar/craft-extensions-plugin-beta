use crate::look::{label, tone};
use crate::manager::Pending;
use craft::theme::variant::{Emphasis, Tone as FwTone};
use craft::theme::Variant;
use craft::ui::choice::Checkbox;
use craft::ui::contract::{Bind, Props as _, Show as _, ShowContainer as _};
use craft::ui::dialog::Dialog;
use craft::ui::display::{Badge, Shade, Text};
use craft::ui::layout_components::{Row, Stack};
use craft::ui::notices::Notice;
use craft::ui::runtime::env::Label;
use craft::ui::sections::{DialogRole, FooterButton, Section};
use craft::ui::style::TypeRole;
use craft_ext_core::id::AppId;
use craft_ext_manager::{Action, Model, Setup, Word};
use egui::Ui;
use std::cell::RefCell;

const CANCEL: usize = 0;
const CONFIRM: usize = 1;

pub(crate) fn show(ui: &mut Ui, model: &Model, names: &dyn Fn(&AppId) -> Label, pending: &RefCell<Pending>) {
    let Some(setup) = model.pending_setup() else { return };
    let id = setup.id.clone();
    let mut confirm = FooterButton::new(label(Word::ActionConfirmSetup), DialogRole::Accept).enabled(setup.ready());
    if let Some(blocker) = setup.blocker() {
        confirm = confirm.disabled_reason(label(blocker));
    }
    let mut open = true;
    let shown = Dialog::new(label(Word::SetupTitle))
        .id_salt(("plugin-setup", id.as_str(), setup.version.to_string()))
        .opened(Bind::bound(&mut open))
        .buttons([FooterButton::new(label(Word::ActionCancel), DialogRole::Cancel), confirm])
        .show(ui, |ui, _| body(ui, setup, names, pending));
    let action = match (shown.output.picked, shown.output.closed, open) {
        (Some(CONFIRM), _, _) => Some(Action::ConfirmSetup { id }),
        (Some(CANCEL), _, _) | (None, Some(_), _) | (None, None, false) => Some(Action::Cancel { id }),
        _ => None,
    };
    pending.borrow_mut().actions.extend(action);
}

fn body(ui: &mut Ui, setup: &Setup, names: &dyn Fn(&AppId) -> Label, pending: &RefCell<Pending>) {
    let id = &setup.id;
    Row::new().show(ui, |ui, _| {
        Text::new(Label::from(setup.name.clone())).role(TypeRole::Heading).show(ui);
        Text::new(Label::from(setup.version.to_string())).shade(Shade::Muted).show(ui);
        Badge::new(names(&setup.app)).show(ui);
    });
    Text::new(label(Word::SetupNothingChanges)).shade(Shade::Muted).show(ui);
    if !setup.dependencies.is_empty() {
        let also = setup.dependencies.iter().map(|(d, v)| format!("{d} {v}")).collect::<Vec<_>>().join(", ");
        Row::new().show(ui, |ui, _| {
            Text::new(label(Word::SetupAlsoInstalls)).shade(Shade::Muted).show(ui);
            Text::new(Label::from(also.clone())).shade(Shade::Faint).show(ui);
        });
    }
    Section::fixed(label(Word::SectionSetupParts)).id_salt("parts").show(ui, |ui, _| {
        for c in &setup.contributions {
            Stack::new().id_salt(("part", c.id.as_str())).show(ui, |ui, _| {
                let out = Checkbox::new(Bind::given(c.on)).label(Label::from(c.label.clone())).enabled(c.unavailable.is_none()).show(ui);
                if let Some(on) = out.events.committed().copied() {
                    pending.borrow_mut().actions.push(Action::SetupContribution { id: id.clone(), contribution: c.id.clone(), on });
                }
                if let Some(why) = &c.unavailable {
                    Text::new(Label::from(why.clone())).shade(Shade::Faint).show(ui);
                }
            });
        }
    });
    if !setup.permissions.is_empty() {
        Section::fixed(label(Word::SectionSetupAccess)).id_salt("access").show(ui, |ui, _| {
            for p in &setup.permissions {
                Stack::new().id_salt(("access", p.permission.to_string())).show(ui, |ui, _| {
                    Row::new().show(ui, |ui, _| {
                        let out = Checkbox::new(Bind::given(p.on)).label(Label::from(p.sentence.clone())).show(ui);
                        if let Some(on) = out.events.committed().copied() {
                            pending.borrow_mut().actions.push(Action::SetupPermission { id: id.clone(), permission: p.permission.clone(), on });
                        }
                        let (word, t) = if p.required { (Word::PermissionRequired, FwTone::Warn) } else { (Word::PermissionOptional, FwTone::Neutral) };
                        Badge::new(label(word)).variant(Variant::look(Emphasis::Soft, t)).show(ui);
                    });
                    if let Some(why) = &p.why {
                        Row::new().show(ui, |ui, _| {
                            Text::new(label(Word::LabelAuthorSays)).shade(Shade::Faint).show(ui);
                            Text::new(Label::from(why.clone())).shade(Shade::Faint).italic().show(ui);
                        });
                    }
                });
            }
        });
    }
    for warning in &setup.warnings {
        Notice::new(FwTone::Warn, Label::from(warning.clone())).show(ui);
    }
    if let Some(native) = setup.native {
        Notice::new(tone(native.tone()), label(native.word())).show(ui);
        let out = Checkbox::new(Bind::given(setup.full_access)).label(label(Word::ConsentFullAccess)).id_salt("full-access").show(ui);
        if let Some(on) = out.events.committed().copied() {
            pending.borrow_mut().actions.push(Action::SetupFullAccess { id: id.clone(), on });
        }
    }
    if let Some(blocker) = setup.blocker() {
        Text::new(label(blocker)).tone(FwTone::Warn).show(ui);
    }
}

use crate::look::{badge_variant, button_variant, label, tone};
use crate::manager::{Pending, WHOLE, picture};
use craft::behavior::keyed;
use craft::theme::Variant;
use craft::theme::variant::Tone as FwTone;
use craft::ui::buttons::{Button, Link};
use craft::ui::choice::Switch;
use craft::ui::contract::{Bind, Props as _, SelectMode, Show as _, ShowCollection as _, ShowContainer as _, Sizing};
use craft::ui::display::{Badge, Shade, Spinner, Text};
use craft::ui::layout::RowAlign;
use craft::ui::layout_components::{Row, Stack};
use craft::ui::list::{List, ListRow, ThumbSize};
use craft::ui::notices::{Notice, NoticeLayout};
use craft::ui::runtime::env::Label;
use craft::ui::sections::Section;
use craft::ui::style::TypeRole;
use craft_ext_core::id::AppId;
use craft_ext_manager::{Action, AppPanel, Detail, Model, Word};
use egui::Ui;
use std::cell::RefCell;

pub(crate) fn show(ui: &mut Ui, model: &Model, names: &dyn Fn(&AppId) -> Label, pending: &RefCell<Pending>) {
    let Some(d) = model.detail() else {
        Notice::new(FwTone::Info, label(Word::SelectPrompt)).layout(NoticeLayout::Centred).show(ui);
        return;
    };
    heading(ui, model, d, pending);
    if let Some(native) = d.native {
        Notice::new(tone(native.tone()), label(native.word())).show(ui);
    }
    for warning in &d.permission_warnings {
        Notice::new(FwTone::Warn, Label::from(warning.clone())).show(ui);
    }
    Section::fixed(label(Word::SectionAbout)).id_salt("about").show(ui, |ui, _| about(ui, d));
    if !d.screenshots.is_empty() {
        Section::fixed(label(Word::SectionScreenshots)).id_salt("screenshots").show(ui, |ui, _| screenshots(ui, model, d));
    }
    if !d.permissions.is_empty() {
        Section::fixed(label(Word::SectionPermissions)).id_salt("permissions").show(ui, |ui, _| permissions(ui, d));
    }
    if !d.apps.is_empty() {
        Section::fixed(label(Word::SectionApps)).id_salt("apps").show(ui, |ui, _| {
            for panel in &d.apps {
                app_panel(ui, d, panel, names, pending);
            }
        });
    }
    Section::fixed(label(Word::SectionVersions)).id_salt("versions").show(ui, |ui, _| versions(ui, model, d, pending));
}

fn heading(ui: &mut Ui, model: &Model, d: &Detail, pending: &RefCell<Pending>) {
    Text::new(Label::from(d.name.clone())).role(TypeRole::Title).show(ui);
    Text::new(Label::from(d.summary.clone())).shade(Shade::Muted).show(ui);
    Row::new().wrap(true).show(ui, |ui, _| {
        for badge in &d.badges {
            Badge::new(label(badge.word())).variant(badge_variant(badge.tone())).show(ui);
        }
    });
    Row::new().wrap(true).align(RowAlign::Start).show(ui, |ui, _| {
        if let Some(row) = model.rows().iter().find(|r| r.id == d.id) {
            let primary = Button::new(label(row.primary.word())).variant(button_variant(row.primary.look())).loading(row.primary == craft_ext_manager::Primary::Working).id_salt("detail-primary");
            if primary.show(ui).clicked() {
                pending.borrow_mut().press(model, row);
            }
        }
        if d.versions.iter().any(|v| v.installed) && Button::new(label(Word::ActionUninstall)).variant(Variant::SECONDARY).id_salt("uninstall").show(ui).clicked() {
            pending.borrow_mut().actions.push(Action::Uninstall { id: d.id.clone() });
        }
        if let Some(home) = &d.homepage
            && Link::new(label(Word::ActionHomepage)).url(home.clone()).id_salt("homepage").show(ui).clicked()
        {
            pending.borrow_mut().links.push(home.clone());
        }
        if Link::new(label(Word::ActionReport)).url(d.report_url.clone()).id_salt("report").show(ui).clicked() {
            pending.borrow_mut().actions.push(Action::Report { id: d.id.clone() });
        }
    });
}

fn fact(ui: &mut Ui, word: Word, value: String) {
    if value.is_empty() {
        return;
    }
    Row::new().show(ui, |ui, _| {
        Text::new(label(word)).shade(Shade::Muted).show(ui);
        Text::new(Label::from(value.clone())).show(ui);
    });
}

fn about(ui: &mut Ui, d: &Detail) {
    if !d.description.is_empty() {
        Text::new(Label::from(d.description.clone())).show(ui);
    }
    let newest = d.versions.first().map(|v| v.version.to_string()).unwrap_or_default();
    fact(ui, Word::LabelVersion, newest);
    fact(ui, Word::LabelVendor, d.vendor.clone());
    fact(ui, Word::LabelLicense, d.license.clone());
    fact(ui, Word::LabelDownloads, if d.downloads > 0 { d.downloads.to_string() } else { String::new() });
}

fn screenshots(ui: &mut Ui, model: &Model, d: &Detail) {
    let keys: Vec<usize> = (0..d.screenshots.len()).collect();
    let source = keyed(&keys, |k| *k);
    let ctx = ui.ctx().clone();
    let _ = List::<usize>::new(label(Word::SectionScreenshots))
        .id_salt("screenshots-list")
        .select(SelectMode::None)
        .header(false)
        .sizing(Sizing::FillWidth)
        .show_items(ui, &source, &mut |key, _| {
            let row = ListRow::new(Label::from(format!("{}", key + 1)));
            match d.screenshots.get(*key).and_then(|s| picture(&ctx, model, s)) {
                Some(texture) => row.thumb(ThumbSize::Large, move |cx, rect| cx.picture(texture, rect, WHOLE, 0.0)),
                None => row.detail(label(Word::StateLoading)),
            }
        });
}

fn permissions(ui: &mut Ui, d: &Detail) {
    for p in &d.permissions {
        Stack::new().id_salt(("permission", p.permission.to_string())).show(ui, |ui, _| {
            Row::new().show(ui, |ui, _| {
                Text::new(Label::from(p.sentence.clone())).show(ui);
                let (word, t) = if p.required { (Word::PermissionRequired, FwTone::Warn) } else { (Word::PermissionOptional, FwTone::Neutral) };
                Badge::new(label(word)).variant(craft::theme::Variant::look(craft::theme::variant::Emphasis::Soft, t)).show(ui);
            });
            if let Some(why) = &p.why {
                Row::new().show(ui, |ui, _| {
                    Text::new(label(Word::LabelAuthorSays)).shade(Shade::Faint).show(ui);
                    Text::new(Label::from(why.clone())).shade(Shade::Faint).italic().show(ui);
                });
            }
        });
    }
}

fn app_panel(ui: &mut Ui, d: &Detail, panel: &AppPanel, names: &dyn Fn(&AppId) -> Label, pending: &RefCell<Pending>) {
    let (id, app) = (d.id.clone(), panel.app.clone());
    Stack::new().id_salt(("app", app.as_str())).show(ui, |ui, _| {
        Row::new().show(ui, |ui, _| {
            Text::new(names(&app)).role(TypeRole::BodyStrong).show(ui);
            if let Some(v) = panel.active {
                Text::new(Label::from(v.to_string())).shade(Shade::Muted).show(ui);
            }
            if panel.pin.is_some() {
                Badge::new(label(Word::LabelPinned)).show(ui);
            }
        });
        if panel.full_access {
            Notice::new(FwTone::Bad, label(Word::ConsentFullAccess)).show(ui);
            if Button::new(label(Word::ActionRemoveFullAccess)).id_salt("full-access-off").show(ui).clicked() {
                pending.borrow_mut().actions.push(Action::RemoveFullAccess { id: id.clone(), app: app.clone() });
            }
        }
        if let Some(q) = &panel.quarantine {
            Notice::new(FwTone::Warn, label(Word::ProblemQuarantined)).lines([Label::from(q.reason.clone())]).show(ui);
            if Button::new(label(Word::ActionTurnBackOn)).id_salt("turn-back-on").show(ui).clicked() {
                pending.borrow_mut().actions.push(Action::TurnBackOn { id: id.clone(), app: app.clone() });
            }
        }
        for c in &panel.contributions {
            let out = Switch::new(Bind::given(c.enabled)).label(Label::from(c.label.clone())).id_salt(("part", c.id.as_str())).show(ui);
            if let Some(on) = out.events.committed().copied() {
                let action = if on && !c.enabled { Action::SetUp { id: id.clone(), app: app.clone() } } else { Action::SetContribution { id: id.clone(), app: app.clone(), contribution: c.id.clone(), on } };
                pending.borrow_mut().actions.push(action);
            }
        }
        Row::new().wrap(true).show(ui, |ui, _| {
            if panel.pending.is_some() && Button::new(label(Word::ActionSetUp)).variant(Variant::PRIMARY).id_salt("set-up").show(ui).clicked() {
                pending.borrow_mut().actions.push(Action::SetUp { id: id.clone(), app: app.clone() });
            }
            if let Some(v) = panel.update
                && Button::new(label(Word::ActionUpdate)).variant(Variant::PRIMARY).tooltip(Label::from(v.to_string())).id_salt("update").show(ui).clicked()
            {
                pending.borrow_mut().actions.push(Action::Update { id: id.clone(), app: app.clone() });
            }
            if panel.pin.is_some() && Button::new(label(Word::ActionUnpin)).id_salt("unpin").show(ui).clicked() {
                pending.borrow_mut().actions.push(Action::Unpin { id: id.clone(), app: app.clone() });
            }
        });
        for g in panel.grants.iter().filter(|g| g.granted) {
            Row::new().show(ui, |ui, _| {
                Text::new(Label::from(g.sentence.clone())).shade(Shade::Muted).show(ui);
                if Button::new(label(Word::ActionRevoke)).variant(Variant::GHOST).id_salt(("revoke", g.permission.to_string())).show(ui).clicked() {
                    pending.borrow_mut().actions.push(Action::Revoke { id: id.clone(), app: app.clone(), permission: g.permission.clone() });
                }
            });
        }
    });
}

fn versions(ui: &mut Ui, model: &Model, d: &Detail, pending: &RefCell<Pending>) {
    if d.versions_loading {
        Spinner::new().label(label(Word::StateLoading)).show(ui);
    }
    if let Some(error) = &d.versions_error {
        Notice::new(FwTone::Warn, Label::from(error.clone())).show(ui);
    }
    let app = model.config().scope().cloned().or_else(|| d.apps.first().map(|p| p.app.clone()));
    for v in &d.versions {
        Stack::new().id_salt(("version", v.version.to_string())).show(ui, |ui, _| {
            Row::new().wrap(true).show(ui, |ui, _| {
                Text::new(Label::from(v.version.to_string())).role(TypeRole::BodyStrong).show(ui);
                if !v.published.is_empty() {
                    Text::new(Label::from(v.published.clone())).shade(Shade::Faint).show(ui);
                }
                if v.attestation.as_ref().is_some_and(|a| a.verified) {
                    Badge::new(label(Word::BadgeBuiltFromSource)).variant(badge_variant(craft_ext_manager::Badge::BuiltFromSource.tone())).show(ui);
                }
                if !v.active_in.is_empty() {
                    Badge::new(label(Word::LabelActive)).variant(Variant::look(craft::theme::variant::Emphasis::Soft, FwTone::Accent)).show(ui);
                }
            });
            let notes = if v.changelog.is_empty() { label(Word::LabelNoChangelog) } else { Label::from(v.changelog.clone()) };
            Text::new(notes).shade(Shade::Muted).show(ui);
            if let Some(app) = &app
                && v.installed
                && !v.active_in.contains(app)
                && v.usable_in.contains(app)
                && Button::new(label(Word::ActionRollback)).variant(Variant::GHOST).id_salt("rollback").show(ui).clicked()
            {
                pending.borrow_mut().actions.push(Action::Rollback { id: d.id.clone(), app: app.clone(), version: v.version });
            }
        });
    }
}

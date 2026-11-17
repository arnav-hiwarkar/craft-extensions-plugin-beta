use super::{
    look::{Words, tone_color},
    page::primary,
    widgets::{action_button, badge, callout, icon, picture, section, toned, weak},
};
use craft_ext_core::version::VersionReq;
use craft_ext_manager::{Action, AppPanel, Detail, Look, Model, Tone, VersionRow, Word, category_word};
use egui::{RichText, ScrollArea, Ui};

pub(crate) const DETAIL_ICON: f32 = 56.0;
pub(crate) const SCREENSHOT_HEIGHT: f32 = 150.0;

pub(crate) fn pane(ui: &mut Ui, model: &Model, d: &Detail, words: &dyn Words, actions: &mut Vec<Action>) {
    ScrollArea::vertical().id_salt(("craft-plugin-detail", d.id.as_str())).auto_shrink([false, false]).show(ui, |ui| {
        header(ui, model, d, words, actions);
        if let Some(native) = d.native {
            ui.add_space(10.0);
            callout(ui, native.tone(), words.word(native.word()), None);
        }
        about(ui, d, words);
        screenshots(ui, model, d, words, actions);
        permissions(ui, d, words);
        if !d.apps.is_empty() {
            section(ui, words.word(Word::SectionApps));
            for panel in &d.apps {
                app_panel(ui, d, panel, words, actions);
            }
        }
        versions(ui, d, words, actions);
        footer(ui, d, words, actions);
        ui.add_space(16.0);
    });
}

fn header(ui: &mut Ui, model: &Model, d: &Detail, words: &dyn Words, actions: &mut Vec<Action>) {
    ui.horizontal(|ui| {
        icon(ui, model, &d.id, &d.name, d.icon.as_ref(), DETAIL_ICON, actions);
        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.heading(&d.name);
            match &d.vendor_url {
                Some(url) => {
                    if ui.link(weak(ui, &d.vendor)).clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                    }
                }
                None => {
                    ui.label(weak(ui, &d.vendor));
                }
            }
            if let Some(row) = model.rows().iter().find(|r| r.id == d.id) {
                ui.add_space(4.0);
                primary(ui, model, row, words, actions);
            }
        });
    });
    ui.add_space(6.0);
    ui.label(&d.summary);
    if !d.badges.is_empty() {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for b in &d.badges {
                badge(ui, words.word(b.word()), b.tone());
            }
        });
    }
}

fn installed(d: &Detail) -> Option<&VersionRow> {
    d.versions.iter().find(|v| !v.active_in.is_empty()).or_else(|| d.versions.iter().find(|v| v.installed))
}

fn about(ui: &mut Ui, d: &Detail, words: &dyn Words) {
    section(ui, words.word(Word::SectionAbout));
    if !d.description.is_empty() {
        ui.label(&d.description);
        ui.add_space(6.0);
    }
    egui::Grid::new(("craft-plugin-facts", d.id.as_str())).num_columns(2).spacing([16.0, 4.0]).show(ui, |ui| {
        let fact = |ui: &mut Ui, word: Word, value: String| {
            if !value.is_empty() {
                ui.label(weak(ui, words.word(word)));
                ui.label(value);
                ui.end_row();
            }
        };
        fact(ui, Word::LabelVendor, d.vendor.clone());
        let version = installed(d).or(d.versions.first()).map(|v| v.version.to_string()).unwrap_or_default();
        fact(ui, Word::LabelVersion, version);
        fact(ui, Word::LabelLicense, d.license.clone());
        fact(ui, Word::LabelDownloads, if d.downloads > 0 { d.downloads.to_string() } else { String::new() });
    });
    if !d.categories.is_empty() {
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for c in &d.categories {
                badge(ui, words.word(category_word(*c)), Tone::Neutral);
            }
        });
    }
}

fn screenshots(ui: &mut Ui, model: &Model, d: &Detail, words: &dyn Words, actions: &mut Vec<Action>) {
    let shots: Vec<_> = d.screenshots.iter().filter_map(|s| picture(ui, model, &d.id, s, actions).map(|p| (s, p))).collect();
    if shots.is_empty() {
        return;
    }
    section(ui, words.word(Word::SectionScreenshots));
    ScrollArea::horizontal().id_salt(("craft-plugin-shots", d.id.as_str())).show(ui, |ui| {
        ui.horizontal(|ui| {
            for (shot, image) in shots {
                let ratio = if shot.height > 0 { shot.width as f32 / shot.height as f32 } else { 16.0 / 9.0 };
                ui.add(image.fit_to_exact_size(egui::vec2(SCREENSHOT_HEIGHT * ratio, SCREENSHOT_HEIGHT)).corner_radius(egui::CornerRadius::same(6)));
            }
        });
    });
}

fn permissions(ui: &mut Ui, d: &Detail, words: &dyn Words) {
    if d.permissions.is_empty() && d.permission_warnings.is_empty() {
        return;
    }
    section(ui, words.word(Word::SectionPermissions));
    for p in &d.permissions {
        ui.horizontal_wrapped(|ui| {
            ui.label(&p.sentence);
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
    for warning in &d.permission_warnings {
        ui.label(toned(ui, warning, Tone::Warn));
    }
}

fn app_panel(ui: &mut Ui, d: &Detail, p: &AppPanel, words: &dyn Words, actions: &mut Vec<Action>) {
    let (id, app) = (&d.id, &p.app);
    egui::Frame::group(ui.style()).corner_radius(egui::CornerRadius::same(8)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new(words.app(app)).strong());
            if let Some(active) = p.active {
                ui.label(weak(ui, format!("{} {active}", words.word(Word::LabelActive))).small());
            }
            if let Some(req) = p.pin {
                badge(ui, format!("{} {req}", words.word(Word::LabelPinned)), Tone::Info);
            }
        });
        if let Some(q) = &p.quarantine {
            ui.label(toned(ui, words.word(Word::ProblemQuarantined), Tone::Warn));
            ui.label(weak(ui, &q.reason).small());
            if action_button(ui, words.word(Word::ActionTurnBackOn), Look::Secondary).clicked() {
                actions.push(Action::TurnBackOn { id: id.clone(), app: app.clone() });
            }
        }
        for c in &p.contributions {
            let mut on = c.enabled;
            if ui.checkbox(&mut on, &c.label).changed() {
                actions.push(Action::SetContribution { id: id.clone(), app: app.clone(), contribution: c.id.clone(), on });
            }
        }
        if !p.grants.is_empty() {
            ui.add_space(4.0);
            for g in &p.grants {
                let mut on = g.granted;
                let text = if g.required { format!("{} · {}", g.sentence, words.word(Word::PermissionRequired)) } else { g.sentence.clone() };
                if ui.checkbox(&mut on, text).changed() {
                    let permission = g.permission.clone();
                    actions.push(if on { Action::Grant { id: id.clone(), app: app.clone(), permission } } else { Action::Revoke { id: id.clone(), app: app.clone(), permission } });
                }
            }
        }
        if p.full_access {
            ui.label(RichText::new(format!("✓ {}", words.word(Word::ConsentFullAccess))).color(tone_color(Tone::Bad, ui.visuals())).small());
        }
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if let Some(update) = p.update
                && action_button(ui, format!("{} {update}", words.word(Word::ActionUpdate)), Look::Primary).clicked()
            {
                actions.push(Action::Update { id: id.clone(), app: app.clone() });
            }
            if p.pending.is_some() && action_button(ui, words.word(Word::ActionSetUp), Look::Primary).clicked() {
                actions.push(Action::SetUp { id: id.clone(), app: app.clone() });
            }
            match (p.pin, p.active) {
                (Some(_), _) => {
                    if ui.button(words.word(Word::ActionUnpin)).clicked() {
                        actions.push(Action::Unpin { id: id.clone(), app: app.clone() });
                    }
                }
                (None, Some(active)) => {
                    if ui.button(words.word(Word::ActionPin)).clicked() {
                        actions.push(Action::Pin { id: id.clone(), app: app.clone(), req: VersionReq::Exact(active) });
                    }
                }
                (None, None) => {}
            }
        });
    });
    ui.add_space(6.0);
}

fn versions(ui: &mut Ui, d: &Detail, words: &dyn Words, actions: &mut Vec<Action>) {
    if d.versions.is_empty() && !d.versions_loading && d.versions_error.is_none() {
        return;
    }
    section(ui, words.word(Word::SectionVersions));
    if d.versions_loading {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(weak(ui, words.word(Word::StateLoading)));
        });
    }
    if let Some(error) = &d.versions_error {
        ui.label(weak(ui, error).small());
    }
    for v in &d.versions {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(v.version.to_string()).strong());
            if !v.published.is_empty() {
                ui.label(weak(ui, &v.published).small());
            }
            for app in &v.active_in {
                let text = if d.apps.len() > 1 { format!("{} · {}", words.word(Word::LabelActive), words.app(app)) } else { words.word(Word::LabelActive) };
                badge(ui, text, Tone::Good);
            }
            if v.attestation.as_ref().is_some_and(|a| a.verified) {
                badge(ui, words.word(Word::BadgeBuiltFromSource), Tone::Good);
            }
        });
        if v.changelog.is_empty() {
            ui.label(weak(ui, words.word(Word::LabelNoChangelog)).small());
        } else {
            ui.label(&v.changelog);
        }
        for p in d.apps.iter().filter(|p| p.active.is_some() && v.usable_in.contains(&p.app) && !v.active_in.contains(&p.app)) {
            let text = if d.apps.len() > 1 { format!("{} · {}", words.word(Word::ActionRollback), words.app(&p.app)) } else { words.word(Word::ActionRollback) };
            if ui.small_button(text).clicked() {
                actions.push(Action::Rollback { id: d.id.clone(), app: p.app.clone(), version: v.version });
            }
        }
    }
}

fn footer(ui: &mut Ui, d: &Detail, words: &dyn Words, actions: &mut Vec<Action>) {
    ui.add_space(14.0);
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        if let Some(url) = &d.homepage
            && ui.link(words.word(Word::ActionHomepage)).clicked()
        {
            ui.ctx().open_url(egui::OpenUrl::new_tab(url));
        }
        if ui.link(words.word(Word::ActionReport)).clicked() {
            actions.push(Action::Report { id: d.id.clone() });
        }
        if d.versions.iter().any(|v| v.installed) {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let response = ui.button(toned(ui, words.word(Word::ActionUninstall), Tone::Bad));
                egui::Popup::menu(&response).show(|ui| {
                    if ui.button(toned(ui, format!("{} {}", words.word(Word::ActionUninstall), d.name), Tone::Bad)).clicked() {
                        actions.push(Action::Uninstall { id: d.id.clone() });
                    }
                    if ui.button(words.word(Word::ActionCancel)).clicked() {
                        ui.close();
                    }
                });
            });
        }
    });
}

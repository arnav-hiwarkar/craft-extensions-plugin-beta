use super::{
    Session, detail,
    look::Words,
    widgets::{action_button, badge, callout, empty, icon, nav_item, skeleton, toned, weak},
};
use craft_ext_manager::{Action, Config, Filters, Flow, Look, Model, Press, Primary, Problem, ProblemKind, Row, Sort, Step, Tab, Tone, Word, category_word};
use egui::{Align, Frame, Layout, Margin, Rect, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, vec2};

pub(crate) const SIDEBAR_WIDTH: f32 = 168.0;
pub(crate) const DETAIL_WIDTH: f32 = 400.0;
pub(crate) const SPLIT_FROM: f32 = 860.0;
pub(crate) const MIN_HEIGHT: f32 = 460.0;
pub(crate) const GAP: f32 = 16.0;
pub(crate) const ROW_ICON: f32 = 40.0;
pub(crate) const SKELETON_ROWS: usize = 6;

pub struct PluginManager {
    session: Session,
    search: String,
}

impl PluginManager {
    pub fn new(config: Config) -> Self {
        PluginManager { session: Session::new(config), search: String::new() }
    }

    pub fn model(&self) -> &Model {
        &self.session.model
    }

    pub fn model_mut(&mut self) -> &mut Model {
        &mut self.session.model
    }

    pub fn show(&mut self, ui: &mut Ui, words: &dyn Words) {
        self.show_with_app_problems(ui, words, |_| false);
    }

    pub fn show_with_app_problems(&mut self, ui: &mut Ui, words: &dyn Words, app_problems: impl FnOnce(&mut Ui) -> bool) {
        let ctx = ui.ctx().clone();
        self.session.poll(&ctx, words);
        let mut actions = vec![];
        let mut hook = Some(app_problems);
        let available = ui.available_rect_before_wrap();
        let bottom = available.bottom().min(ui.clip_rect().bottom());
        let full = Rect::from_min_size(available.min, vec2(available.width(), (bottom - available.top()).max(MIN_HEIGHT)));
        let model = &self.session.model;
        let side = Rect::from_min_size(full.min, vec2(SIDEBAR_WIDTH, full.height()));
        ui.scope_builder(UiBuilder::new().max_rect(side).id_salt("craft-plugin-sidebar"), |ui| sidebar(ui, model, words, &mut actions));
        ui.painter().vline(side.right() + GAP / 2.0, full.y_range(), ui.visuals().widgets.noninteractive.bg_stroke);
        let rest = full.with_min_x(side.right() + GAP);
        let selected = model.detail().is_some();
        if rest.width() + SIDEBAR_WIDTH >= SPLIT_FROM {
            let list = rest.with_max_x(rest.right() - DETAIL_WIDTH - GAP);
            let pane = rest.with_min_x(list.right() + GAP);
            ui.scope_builder(UiBuilder::new().max_rect(list).id_salt("craft-plugin-list"), |ui| main(ui, model, &mut self.search, words, &mut actions, &mut hook));
            ui.painter().vline(list.right() + GAP / 2.0, full.y_range(), ui.visuals().widgets.noninteractive.bg_stroke);
            ui.scope_builder(UiBuilder::new().max_rect(pane).id_salt("craft-plugin-detail"), |ui| match model.detail() {
                Some(d) => detail::pane(ui, model, d, words, &mut actions),
                None => empty(ui, words.word(Word::SelectPrompt)),
            });
        } else if selected {
            ui.scope_builder(UiBuilder::new().max_rect(rest).id_salt("craft-plugin-detail"), |ui| {
                if ui.button(format!("‹ {}", words.word(Word::ActionBack))).clicked() {
                    actions.push(Action::Select(None));
                }
                if let Some(d) = model.detail() {
                    detail::pane(ui, model, d, words, &mut actions);
                }
            });
        } else {
            ui.scope_builder(UiBuilder::new().max_rect(rest).id_salt("craft-plugin-list"), |ui| main(ui, model, &mut self.search, words, &mut actions, &mut hook));
        }
        ui.advance_cursor_after_rect(full);
        self.session.apply(actions, &ctx, words);
        self.session.overlays(&ctx, words);
    }
}

fn sidebar(ui: &mut Ui, model: &Model, words: &dyn Words, actions: &mut Vec<Action>) {
    ui.add_space(4.0);
    for tab in Tab::ALL {
        let (count, tone) = match tab {
            Tab::Updates => (model.count(tab), Tone::Accent),
            Tab::Problems => (model.count(tab), Tone::Warn),
            Tab::Discover | Tab::Installed => (0, Tone::Neutral),
        };
        let selected = model.tab() == tab;
        if nav_item(ui, words.word(tab.word()), count, tone, selected).clicked() && !selected {
            actions.push(Action::SetTab(tab));
        }
        ui.add_space(2.0);
    }
}

fn main<F: FnOnce(&mut Ui) -> bool>(ui: &mut Ui, model: &Model, search: &mut String, words: &dyn Words, actions: &mut Vec<Action>, hook: &mut Option<F>) {
    toolbar(ui, model, search, words, actions);
    if let Some(error) = model.catalogue_error() {
        ui.add_space(8.0);
        callout(ui, Tone::Warn, words.word(Word::CatalogueUnavailable), Some(error));
    }
    ui.add_space(10.0);
    ScrollArea::vertical().id_salt("craft-plugin-rows").auto_shrink([false, false]).show(ui, |ui| {
        if !model.loaded() {
            ui.label(weak(ui, words.word(Word::StateLoading)));
            ui.add_space(6.0);
            skeleton(ui, SKELETON_ROWS);
        } else if model.tab() == Tab::Problems {
            let drew = hook.take().is_some_and(|h| h(ui));
            for problem in model.problems() {
                problem_card(ui, model, problem, words, actions);
            }
            if !drew && model.problems().is_empty() {
                empty(ui, words.word(Tab::Problems.empty()));
            }
        } else if model.rows().is_empty() {
            empty(ui, words.word(model.tab().empty()));
        } else {
            for row in model.rows() {
                row_card(ui, model, row, words, actions);
            }
        }
        ui.add_space(16.0);
        ui.label(weak(ui, words.word(Word::Disclaimer)).small());
    });
}

fn toolbar(ui: &mut Ui, model: &Model, search: &mut String, words: &dyn Words, actions: &mut Vec<Action>) {
    ui.horizontal(|ui| {
        let width = (ui.available_width() - 280.0).max(160.0);
        let field = ui.add(egui::TextEdit::singleline(search).hint_text(words.word(Word::SearchPlaceholder)).desired_width(width));
        if field.changed() {
            actions.push(Action::SetQuery(search.clone()));
        }
        let current = model.query().sort;
        ui.label(weak(ui, words.word(Word::SortLabel)));
        egui::ComboBox::from_id_salt("craft-plugin-sort").selected_text(words.word(current.word())).show_ui(ui, |ui| {
            for sort in Sort::ALL {
                if ui.selectable_label(sort == current, words.word(sort.word())).clicked() && sort != current {
                    actions.push(Action::SetSort(sort));
                }
            }
        });
        if ui.button(words.word(Word::ActionRefresh)).clicked() {
            actions.push(Action::Refresh);
        }
        if model.busy() {
            ui.spinner();
        }
    });
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        let filters = &model.query().filters;
        let chip = |ui: &mut Ui, text: String, on: bool| ui.add(egui::Button::new(text).selected(on).corner_radius(egui::CornerRadius::same(12)));
        if chip(ui, words.word(Word::FilterAll), filters.category.is_none()).clicked() && filters.category.is_some() {
            actions.push(Action::SetFilters(Filters { category: None, ..filters.clone() }));
        }
        for category in model.categories() {
            let on = filters.category == Some(category);
            if chip(ui, words.word(category_word(category)), on).clicked() {
                actions.push(Action::SetFilters(Filters { category: (!on).then_some(category), ..filters.clone() }));
            }
        }
    });
    if model.tab() == Tab::Updates && model.count(Tab::Updates) > 0 {
        ui.add_space(6.0);
        if action_button(ui, words.word(Word::ActionUpdateAll), Look::Primary).clicked() {
            actions.push(Action::UpdateAll { app: model.config().scope().cloned() });
        }
    }
}

fn card(ui: &mut Ui, salt: impl egui::AsIdSalt, selected: bool, content: impl FnOnce(&mut Ui)) -> egui::Response {
    ui.scope_builder(UiBuilder::new().id_salt(salt).sense(Sense::click()), |ui| {
        let visuals = ui.visuals().clone();
        let (fill, stroke) = if selected {
            (visuals.selection.bg_fill.gamma_multiply(0.22), Stroke::new(1.0, visuals.selection.bg_fill))
        } else {
            (visuals.faint_bg_color, Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color.gamma_multiply(0.5)))
        };
        Frame::new().fill(fill).stroke(stroke).corner_radius(egui::CornerRadius::same(8)).inner_margin(Margin::same(10)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            content(ui);
        });
        ui.add_space(6.0);
    })
    .response
}

fn row_card(ui: &mut Ui, model: &Model, row: &Row, words: &dyn Words, actions: &mut Vec<Action>) {
    let selected = model.selected() == Some(&row.id);
    let response = card(ui, ("craft-plugin-row", row.id.as_str()), selected, |ui| {
        ui.horizontal(|ui| {
            icon(ui, model, &row.id, &row.name, row.icon.as_ref(), ROW_ICON, actions);
            ui.add_space(6.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                primary(ui, model, row, words, actions);
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&row.name).strong());
                        ui.label(weak(ui, &row.vendor).small());
                        if let Some(version) = version_text(row) {
                            ui.label(weak(ui, version).small());
                        }
                    });
                    ui.add(egui::Label::new(weak(ui, &row.summary)).truncate());
                    let shown: Vec<_> = row.badges.iter().filter(|b| b.tone() != Tone::Neutral).collect();
                    if !shown.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            for b in shown {
                                badge(ui, words.word(b.word()), b.tone());
                            }
                        });
                    }
                });
            });
        });
    });
    if response.clicked() {
        actions.push(Action::Select(Some(row.id.clone())));
    }
}

fn version_text(row: &Row) -> Option<String> {
    match (row.installed, row.update) {
        (Some(installed), Some(update)) => Some(format!("{installed} → {update}")),
        (Some(installed), None) => Some(installed.to_string()),
        (None, _) => None,
    }
}

pub(crate) fn primary(ui: &mut Ui, model: &Model, row: &Row, words: &dyn Words, actions: &mut Vec<Action>) {
    let text = match (row.primary, model.flow(&row.id)) {
        (Primary::Working, Some(Flow { step: Step::Working(word), .. })) => words.word(*word),
        (primary, _) => words.word(primary.word()),
    };
    let response = action_button(ui, text, row.primary.look());
    match model.press(row) {
        Press::Do(list) if response.clicked() => actions.extend(list),
        Press::Open if response.clicked() => actions.push(Action::Select(Some(row.id.clone()))),
        Press::ChooseApp(apps) => {
            egui::Popup::menu(&response).show(|ui| {
                ui.label(weak(ui, words.word(Word::ChooseApp)).small());
                for app in apps {
                    if ui.button(words.app(&app)).clicked() {
                        actions.push(Action::Install { id: row.id.clone(), app: Some(app) });
                    }
                }
            });
        }
        Press::Do(_) | Press::Open | Press::Nothing => {}
    }
}

fn problem_reason(kind: &ProblemKind) -> Option<String> {
    match kind {
        ProblemKind::Quarantined { reason } | ProblemKind::KillListed { reason } => Some(reason.clone()),
        ProblemKind::Incompatible { reasons } => Some(reasons.join("; ")),
        ProblemKind::NoHelper | ProblemKind::ConsentPending { .. } | ProblemKind::UncleanExit => None,
    }
}

fn problem_card(ui: &mut Ui, model: &Model, problem: &Problem, words: &dyn Words, actions: &mut Vec<Action>) {
    let selected = model.selected() == Some(&problem.id);
    let salt = ("craft-plugin-problem", problem.app.as_str(), problem.id.as_str(), words.word(problem.kind.word()));
    let response = card(ui, salt, selected, |ui| {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let (Some(fix), Some(action)) = (problem.fix, problem.action())
                    && action_button(ui, words.word(fix.word()), Look::Secondary).clicked()
                {
                    actions.push(action);
                }
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(model.name_of(&problem.id)).strong());
                        if let Some(v) = problem.version {
                            ui.label(weak(ui, v.to_string()).small());
                        }
                        if model.config().scope().is_none() {
                            ui.label(weak(ui, words.app(&problem.app)).small());
                        }
                    });
                    ui.label(toned(ui, words.word(problem.kind.word()), problem.kind.tone()));
                    if let Some(reason) = problem_reason(&problem.kind) {
                        ui.label(weak(ui, reason).small());
                    }
                });
            });
        });
    });
    if response.clicked() {
        actions.push(Action::Select(Some(problem.id.clone())));
    }
}

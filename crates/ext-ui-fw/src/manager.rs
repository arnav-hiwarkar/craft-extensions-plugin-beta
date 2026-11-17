use crate::look::{badge_variant, button_variant, label, notice_tone};
use crate::{ManagerState, detail, setup};
use craft::behavior::{Selection, SelectionChange, keyed};
use craft::ui::buttons::{Button, ToggleButton};
use craft::ui::contract::{ActivateOn, Bind, CollectionEvent, Entry, Props as _, Scroll, SelectMode, Show as _, ShowCollection as _, ShowContainer as _, Sizing};
use craft::ui::display::{Badge, Divider, Progress, Shade, Text};
use craft::ui::layout::RowAlign;
use craft::ui::layout_components::{Row, Stack};
use craft::ui::list::{List, ListRow, ThumbSize};
use craft::ui::notices::{Notice, ToastStack};
use craft::ui::pickers::Select;
use craft::ui::primitives::{IconSpec, TextureCache};
use craft::ui::runtime::env::{Label, WindowKey};
use craft::ui::runtime::toast::{self, Toast};
use craft::ui::style::TypeRole;
use craft::ui::text::TextField;
use craft::theme::variant::Tone as FwTone;
use craft_ext_core::{
    id::{AppId, ExtensionId},
    listing::{Category, Image},
};
use craft_ext_manager::{Action, Loadable, Model, Press, Primary, Problem, ProblemKind, Row as PluginRow, Sort, Step, Tab, Word, category_word};
use egui::{Rect, TextureId, Ui, pos2};
use std::cell::RefCell;
use std::time::Duration;

const WINDOW: &str = "craft-ext.plugin-manager";
const REPAINT_WHILE_BUSY: Duration = Duration::from_millis(100);
const IMAGES_PER_PASS: usize = 48;

pub struct PluginManager<'a> {
    state: &'a mut ManagerState,
    problems: Option<Box<dyn FnMut(&mut Ui) + 'a>>,
    app_name: Box<dyn Fn(&AppId) -> Label + 'a>,
}

#[derive(Debug, Default)]
pub struct PluginManagerOutput {
    pub links: Vec<String>,
}

#[derive(Default)]
pub(crate) struct Pending {
    pub actions: Vec<Action>,
    pub links: Vec<String>,
    pub search: String,
    pub choose: Option<(ExtensionId, Vec<AppId>)>,
    pub choose_closed: bool,
}

impl Pending {
    pub fn press(&mut self, model: &Model, row: &PluginRow) {
        match model.press(row) {
            Press::Do(actions) => self.actions.extend(actions),
            Press::ChooseApp(apps) => self.choose = Some((row.id.clone(), apps)),
            Press::Open => self.actions.push(Action::Select(Some(row.id.clone()))),
            Press::Nothing => {}
        }
    }
}

impl<'a> PluginManager<'a> {
    pub fn new(state: &'a mut ManagerState) -> Self {
        PluginManager { state, problems: None, app_name: Box::new(|app| Label::from(app.to_string())) }
    }

    pub fn problems(mut self, rows: impl FnMut(&mut Ui) + 'a) -> Self {
        self.problems = Some(Box::new(rows));
        self
    }

    pub fn app_names(mut self, name: impl Fn(&AppId) -> Label + 'a) -> Self {
        self.app_name = Box::new(name);
        self
    }

    pub fn show(self, ui: &mut Ui) -> PluginManagerOutput {
        let PluginManager { state, mut problems, app_name } = self;
        state.poll();
        request_images(state);
        post_notices(ui, state);
        if state.model.busy() {
            ui.ctx().request_repaint_after(REPAINT_WHILE_BUSY);
        }
        let pending = RefCell::new(Pending { search: state.search.clone(), choose: state.choosing.clone(), ..Pending::default() });
        let model = &state.model;
        let names = &*app_name;
        let body = ui.max_rect();
        Stack::new()
            .fill()
            .header(|ui, _| toolbar(ui, model, &pending))
            .footer(|ui, _| {
                Text::new(label(Word::Disclaimer)).role(TypeRole::Caption).shade(Shade::Faint).show(ui);
            })
            .show(ui, |ui, _| {
                Row::new().fill().show(ui, |ui, _| {
                    sidebar(ui, model, &pending);
                    Divider::vertical().show(ui);
                    Stack::new().fill().scroll(Scroll::vertical()).label(label(model.tab().word())).show(ui, |ui, _| match model.tab() {
                        Tab::Problems => problem_list(ui, model, names, &pending, problems.as_mut()),
                        tab => plugin_list(ui, model, tab, &pending),
                    });
                    Divider::vertical().show(ui);
                    Stack::new().fill().scroll(Scroll::vertical()).label(label(Word::SectionAbout)).show(ui, |ui, _| detail::show(ui, model, names, &pending));
                });
            });
        setup::show(ui, model, names, &pending);
        choose_app(ui, names, &pending);
        ToastStack::show(ui, &WindowKey::new(WINDOW), body);
        let pending = pending.into_inner();
        if pending.search != state.model.query().text {
            state.search = pending.search.clone();
            state.apply(Action::SetQuery(pending.search));
        }
        state.links.extend(pending.links);
        state.choosing = if pending.choose_closed { None } else { pending.choose };
        for action in pending.actions {
            state.apply(action);
        }
        PluginManagerOutput { links: std::mem::take(&mut state.links) }
    }
}

fn request_images(state: &mut ManagerState) {
    let model = &state.model;
    let wanted: Vec<(ExtensionId, Image)> = model
        .rows()
        .iter()
        .filter_map(|r| Some((r.id.clone(), r.icon.clone()?)))
        .take(IMAGES_PER_PASS)
        .chain(model.detail().into_iter().flat_map(|d| d.screenshots.iter().map(|s| (d.id.clone(), s.clone()))))
        .filter(|(_, image)| model.image(&image.url).is_none())
        .collect();
    for (id, image) in wanted {
        state.apply(Action::LoadImage(id, image));
    }
}

fn post_notices(ui: &Ui, state: &mut ManagerState) {
    let window = WindowKey::new(WINDOW);
    while let Some(n) = state.notices.first() {
        let lines: Vec<Label> = [&n.subject, &n.detail].into_iter().filter(|t| !t.is_empty()).map(|t| Label::from(t.clone())).collect();
        let posted = toast::post(ui.ctx(), &window, Toast::new(label(n.word)).tone(notice_tone(n.error)).lines(lines));
        if posted.value() == 0 {
            return;
        }
        state.notices.remove(0);
    }
}

pub(crate) fn picture(ctx: &egui::Context, model: &Model, image: &Image) -> Option<TextureId> {
    let Some(Loadable::Ready(bytes)) = model.image(&image.url) else { return None };
    Some(TextureCache::texture(ctx, image.url.clone(), || decode(bytes)))
}

fn decode(bytes: &[u8]) -> egui::ColorImage {
    match image::load_from_memory(bytes) {
        Ok(decoded) => {
            let rgba = decoded.to_rgba8();
            egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw())
        }
        Err(_) => egui::ColorImage::filled([1, 1], egui::Color32::TRANSPARENT),
    }
}

pub(crate) const WHOLE: Rect = Rect { min: pos2(0.0, 0.0), max: pos2(1.0, 1.0) };

fn toolbar(ui: &mut Ui, model: &Model, pending: &RefCell<Pending>) {
    Row::new().fill_width().align(RowAlign::Start).show(ui, |ui, _| {
        let mut search = pending.borrow().search.clone();
        TextField::search(label(Word::SearchPlaceholder), Bind::bound(&mut search)).placeholder(label(Word::SearchPlaceholder)).sizing(Sizing::FillWidth).show(ui);
        pending.borrow_mut().search = search;
        let sorts: Vec<Entry<usize>> = Sort::ALL.iter().enumerate().map(|(at, s)| Entry::new(at, label(s.word()))).collect();
        let shown = Sort::ALL.iter().position(|s| *s == model.query().sort).unwrap_or(0);
        if let Some(at) = Select::new(label(Word::SortLabel), Bind::given(shown), &sorts).sizing(Sizing::Content).show(ui).events.committed().copied() {
            pending.borrow_mut().actions.extend(Sort::ALL.get(at).map(|s| Action::SetSort(*s)));
        }
        if Button::new(label(Word::ActionRefresh)).variant(craft::theme::Variant::GHOST).loading(!model.loaded()).show(ui).clicked() {
            pending.borrow_mut().actions.push(Action::Refresh);
        }
    });
    if model.tab() == Tab::Discover {
        Row::new().wrap(true).show(ui, |ui, _| {
            let current = model.query().filters.category;
            let chips = std::iter::once(None).chain(model.categories().into_iter().map(Some));
            for category in chips {
                let word = category.map_or(Word::FilterAll, category_word);
                let on = current == category;
                let out = ToggleButton::new(label(word), Bind::given(on)).id_salt(("category", category.map(Category::key))).show(ui);
                if out.events.committed().is_some() && !on {
                    let filters = craft_ext_manager::Filters { category, ..model.query().filters.clone() };
                    pending.borrow_mut().actions.push(Action::SetFilters(filters));
                }
            }
        });
    }
}

fn sidebar(ui: &mut Ui, model: &Model, pending: &RefCell<Pending>) {
    let keys: Vec<usize> = (0..Tab::ALL.len()).collect();
    let source = keyed(&keys, |k| *k);
    let shown = Tab::ALL.iter().position(|t| *t == model.tab()).unwrap_or(0);
    let out = List::<usize>::new(label(Word::Sections))
        .id_salt("plugin-manager-tabs")
        .select(SelectMode::Single)
        .activate(ActivateOn::Click)
        .allow_empty(false)
        .header(false)
        .sizing(Sizing::Content)
        .selection(Bind::given(Selection::single(shown)))
        .show_items(ui, &source, &mut |key, _| {
            let tab = Tab::ALL[*key];
            let row = ListRow::new(label(tab.word()));
            match model.count(tab) {
                n if n > 0 && matches!(tab, Tab::Updates | Tab::Problems) => row.trailing_strong(n.to_string(), label(tab.word())),
                _ => row,
            }
        });
    for event in out.events {
        if let CollectionEvent::Selection(SelectionChange::Set(keys)) = event
            && let Some(tab) = keys.last().and_then(|k| Tab::ALL.get(*k))
        {
            pending.borrow_mut().actions.push(Action::SetTab(*tab));
        }
    }
}

fn loading(ui: &mut Ui) {
    Progress::indeterminate().label(label(Word::StateLoading)).sizing(Sizing::FillWidth).show(ui);
    for at in 0..6 {
        Text::new(label(Word::StateLoading)).shade(Shade::Faint).id_salt(("skeleton", at)).show(ui);
    }
}

fn plugin_list(ui: &mut Ui, model: &Model, tab: Tab, pending: &RefCell<Pending>) {
    if !model.loaded() {
        loading(ui);
        return;
    }
    if let (Tab::Discover, Some(error)) = (tab, model.catalogue_error()) {
        Notice::new(FwTone::Warn, label(Word::CatalogueUnavailable)).lines([Label::from(error.to_owned())]).show(ui);
    }
    if tab == Tab::Updates && model.count(Tab::Updates) > 1 && Button::new(label(Word::ActionUpdateAll)).variant(craft::theme::Variant::PRIMARY).show(ui).clicked() {
        pending.borrow_mut().actions.push(Action::UpdateAll { app: model.query().filters.app.clone() });
    }
    let rows = model.rows();
    let source = keyed(rows, |r: &PluginRow| r.id.clone());
    let selected = model.selected().cloned().map_or_else(Selection::default, Selection::single);
    let ctx = ui.ctx().clone();
    let out = List::<ExtensionId>::new(label(tab.word()))
        .id_salt(("plugin-manager-rows", tab.word().key()))
        .select(SelectMode::Single)
        .activate(ActivateOn::Click)
        .header(false)
        .sizing(Sizing::FillWidth)
        .empty(label(tab.empty()))
        .selection(Bind::given(selected))
        .show_items(ui, &source, &mut |key, _| match rows.iter().find(|r| &r.id == key) {
            Some(row) => plugin_row(&ctx, model, tab, row, pending),
            None => ListRow::new(Label::Empty),
        });
    for event in out.events {
        if let CollectionEvent::Selection(SelectionChange::Set(keys)) = event {
            pending.borrow_mut().actions.push(Action::Select(keys.last().cloned()));
        }
    }
}

fn plugin_row<'r>(ctx: &egui::Context, model: &'r Model, tab: Tab, row: &'r PluginRow, pending: &'r RefCell<Pending>) -> ListRow<'r> {
    let working = match model.flow(&row.id).map(|f| &f.step) {
        Some(Step::Working(word)) => Some(*word),
        _ => None,
    };
    let detail = working.map_or_else(|| Label::from(row.summary.clone()), label);
    let mut out = ListRow::new(Label::from(row.name.clone())).detail(detail);
    out = match row.icon.as_ref().and_then(|icon| picture(ctx, model, icon)) {
        Some(texture) => out.thumb(ThumbSize::Small, move |cx, rect| cx.picture(texture, rect, WHOLE, 0.0)),
        None => out.icon(IconSpec::initial(&row.name)),
    };
    let version = match (tab, row.installed, row.update) {
        (Tab::Updates, Some(from), Some(to)) => Some(format!("{from} → {to}")),
        (Tab::Installed, Some(v), _) => Some(v.to_string()),
        _ => None,
    };
    if let Some(version) = version {
        out = out.trailing(version);
    }
    for badge in row.badges.iter().filter(|b| b.tone() != craft_ext_manager::Tone::Neutral) {
        out = out.below(Badge::new(label(badge.word())).variant(badge_variant(badge.tone())), |_| {});
    }
    let button = Button::new(label(row.primary.word())).variant(button_variant(row.primary.look())).loading(row.primary == Primary::Working).id_salt(("primary", row.id.as_str()));
    out.trailing_field(button, move |clicked| {
        if clicked.clicked() {
            pending.borrow_mut().press(model, row);
        }
    })
}

fn problem_list(ui: &mut Ui, model: &Model, names: &dyn Fn(&AppId) -> Label, pending: &RefCell<Pending>, app_rows: Option<&mut Box<dyn FnMut(&mut Ui) + '_>>) {
    if let Some(rows) = app_rows {
        Stack::new().show(ui, |ui, _| rows(ui));
    }
    let problems = model.problems();
    let keys: Vec<usize> = (0..problems.len()).collect();
    let source = keyed(&keys, |k| *k);
    let _ = List::<usize>::new(label(Word::TabProblems))
        .id_salt("plugin-manager-problems")
        .select(SelectMode::None)
        .header(false)
        .sizing(Sizing::FillWidth)
        .empty(label(Tab::Problems.empty()))
        .show_items(ui, &source, &mut |key, _| match problems.get(*key) {
            Some(problem) => problem_row(model, names, problem, pending),
            None => ListRow::new(Label::Empty),
        });
}

fn problem_row<'r>(model: &Model, names: &dyn Fn(&AppId) -> Label, problem: &'r Problem, pending: &'r RefCell<Pending>) -> ListRow<'r> {
    let name = model.name_of(&problem.id);
    let reason = match &problem.kind {
        ProblemKind::Quarantined { reason } | ProblemKind::KillListed { reason } => reason.clone(),
        ProblemKind::Incompatible { reasons } => reasons.join("; "),
        ProblemKind::NoHelper | ProblemKind::ConsentPending { .. } | ProblemKind::UncleanExit => String::new(),
    };
    let mut row = ListRow::new(Label::from(name)).detail(label(problem.kind.word())).icon(IconSpec::initial(problem.id.as_str()));
    if !reason.is_empty() {
        row = row.lines(Label::from(reason));
    }
    row = row.below(Badge::new(names(&problem.app)).variant(badge_variant(problem.kind.tone())), |_| {});
    match (problem.fix, problem.action()) {
        (Some(fix), Some(action)) => row.trailing_field(Button::new(label(fix.word())).id_salt(("fix", problem.id.as_str(), problem.app.as_str())), move |out| {
            if out.clicked() {
                pending.borrow_mut().actions.push(action);
            }
        }),
        _ => row,
    }
}

fn choose_app(ui: &mut Ui, names: &dyn Fn(&AppId) -> Label, pending: &RefCell<Pending>) {
    let Some((id, apps)) = pending.borrow().choose.clone() else { return };
    let mut open = true;
    let shown = craft::ui::dialog::Dialog::prompt(label(Word::ChooseApp))
        .id_salt(("choose-app", id.as_str()))
        .opened(Bind::bound(&mut open))
        .buttons([craft::ui::sections::FooterButton::new(label(Word::ActionCancel), craft::ui::sections::DialogRole::Cancel)])
        .show(ui, |ui, _| {
            for app in &apps {
                if Button::new(names(app)).sizing(Sizing::FillWidth).id_salt(("choose", app.as_str())).show(ui).clicked() {
                    let mut p = pending.borrow_mut();
                    p.actions.push(Action::Install { id: id.clone(), app: Some(app.clone()) });
                    p.choose_closed = true;
                }
            }
        });
    if shown.output.closed.is_some() || !open {
        pending.borrow_mut().choose_closed = true;
    }
}

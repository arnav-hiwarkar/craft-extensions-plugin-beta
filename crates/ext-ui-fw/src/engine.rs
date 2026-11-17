use craft::behavior::{ColumnKey, ColumnWidth, IconId, Selection as Picked, SelectionChange, SelectionModel as _, keyed};
use craft::theme::Variant as FwVariant;
use craft::theme::variant::{Emphasis, Tone as FwTone};
use craft::ui::buttons::{Button, IconButton};
use craft::ui::choice::{Checkbox, Radio, RadioGroup, Switch};
use craft::ui::contract::{
    Bind, CollectionEvent, ColumnAlign, ColumnSpec, Entry, Props as _, Scroll, SelectMode, Show as _, ShowCollection as _, ShowContainer as _,
};
use craft::ui::display::{Badge, Divider, Icon, Picture, Progress, Shade, Spinner, Text};
use craft::ui::form::{Field, SliderField};
use craft::ui::layout_components::{Grid, Row, Stack};
use craft::ui::list::{List, ListRow};
use craft::ui::notices::Notice;
use craft::ui::pickers::Select;
use craft::ui::primitives::{IconSpec, has_icon};
use craft::ui::runtime::env::Label;
use craft::ui::sections::{GroupBox, Section};
use craft::ui::style::TypeRole;
use craft::ui::table::Table;
use craft::ui::tabs::{Tab, TabStrip};
use craft::ui::text::{NumberField, TextField};
use craft_ext_core::form::{Choice, Form, FormState, Param, ParamKind, Value as FormValue, Values};
use craft_ext_core::ui::tree::{self, Align, Node, Selection, TextStyle, Tone, UiEvent, UiValue, Variant};
use craft_ext_ui::plugin_ui::form::unit_label;
use craft_ext_ui::plugin_ui::{EguiEngine, Engine, FormOptions, Host, Surface, form_rows};
use egui::{ColorImage, Ui};
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

const MAX_DRAW_DEPTH: usize = 32;
const MAX_GRID_COLUMNS: usize = 16;
static GRID_COLUMNS: [ColumnWidth; MAX_GRID_COLUMNS] = [ColumnWidth::Content; MAX_GRID_COLUMNS];
const NEWER_APP: &str = "This part of the panel needs a newer version of the app.";
const INVALID_FORM: &str = "This form has invalid values.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drawn {
    Stack,
    Row,
    Grid,
    GroupBox,
    Section,
    Scroll,
    Text,
    Divider,
    Gap,
    Button,
    IconButton,
    Checkbox,
    Switch,
    SliderField,
    NumberField,
    TextField,
    SearchField,
    Select,
    RadioGroup,
    TabStrip,
    List,
    Table,
    Progress,
    Spinner,
    Icon,
    Picture,
    Badge,
    Notice,
    ParamRows,
    HostComponent,
    Fallback,
    Egui,
}

pub fn drawn(n: &Node) -> Drawn {
    match n {
        Node::Column(_) => Drawn::Stack,
        Node::Row(_) => Drawn::Row,
        Node::Grid(_) => Drawn::Grid,
        Node::Group(_) => Drawn::GroupBox,
        Node::Collapsible(_) => Drawn::Section,
        Node::Scroll(_) => Drawn::Scroll,
        Node::Label(_) => Drawn::Text,
        Node::Separator => Drawn::Divider,
        Node::Spacer(_) => Drawn::Gap,
        Node::Button(b) if b.icon_only && b.icon.is_some() => Drawn::IconButton,
        Node::Button(_) => Drawn::Button,
        Node::Checkbox(_) => Drawn::Checkbox,
        Node::Toggle(_) => Drawn::Switch,
        Node::Slider(_) => Drawn::SliderField,
        Node::Number(_) => Drawn::NumberField,
        Node::TextInput(t) if t.search => Drawn::SearchField,
        Node::TextInput(_) | Node::TextArea(_) => Drawn::TextField,
        Node::Select(_) => Drawn::Select,
        Node::Radio(_) => Drawn::RadioGroup,
        Node::Tabs(_) => Drawn::TabStrip,
        Node::List(_) => Drawn::List,
        Node::Table(_) => Drawn::Table,
        Node::Progress(p) if p.value.is_some() => Drawn::Progress,
        Node::Progress(_) => Drawn::Spinner,
        Node::Icon(_) => Drawn::Icon,
        Node::Image(_) => Drawn::Picture,
        Node::Badge(_) => Drawn::Badge,
        Node::Notice(_) => Drawn::Notice,
        Node::Form(_) => Drawn::ParamRows,
        Node::Component(_) => Drawn::HostComponent,
        Node::Unknown(_) => Drawn::Fallback,
        Node::Canvas(_) | Node::Color(_) => Drawn::Egui,
    }
}

pub fn tone(t: Tone) -> FwTone {
    match t {
        Tone::Neutral => FwTone::Neutral,
        Tone::Accent => FwTone::Accent,
        Tone::Info => FwTone::Info,
        Tone::Good => FwTone::Good,
        Tone::Warn => FwTone::Warn,
        Tone::Bad => FwTone::Bad,
    }
}

pub fn variant(v: Variant) -> FwVariant {
    match v {
        Variant::Primary => FwVariant::PRIMARY,
        Variant::Secondary => FwVariant::SECONDARY,
        Variant::Danger => FwVariant::DANGER,
        Variant::Ghost => FwVariant::GHOST,
    }
}

pub fn badge(t: Tone) -> FwVariant {
    FwVariant::look(Emphasis::Soft, tone(t))
}

pub fn type_role(style: TextStyle) -> (TypeRole, Shade) {
    match style {
        TextStyle::Body => (TypeRole::Body, Shade::Normal),
        TextStyle::Heading => (TypeRole::Heading, Shade::Normal),
        TextStyle::Strong => (TypeRole::BodyStrong, Shade::Normal),
        TextStyle::Small => (TypeRole::Caption, Shade::Normal),
        TextStyle::Weak => (TypeRole::Body, Shade::Muted),
        TextStyle::Mono => (TypeRole::Mono, Shade::Normal),
    }
}

pub fn select_mode(s: Selection) -> SelectMode {
    match s {
        Selection::None => SelectMode::None,
        Selection::Single => SelectMode::Single,
        Selection::Multi => SelectMode::Multi,
    }
}

pub fn column_align(a: Align) -> ColumnAlign {
    match a {
        Align::Left | Align::Center => ColumnAlign::Start,
        Align::Right => ColumnAlign::End,
    }
}

pub fn icon(name: &str) -> IconSpec<'static> {
    let id = IconId::from(Cow::Owned(name.to_owned()));
    if has_icon(&id) { IconSpec::svg(id) } else { IconSpec::initial(name) }
}

pub fn number_edit(shown: f64, committed: Option<f64>) -> Option<UiValue> {
    committed.filter(|v| v.is_finite() && *v != shown).map(UiValue::Number)
}

pub fn bool_edit(shown: bool, committed: Option<bool>) -> Option<UiValue> {
    committed.filter(|v| *v != shown).map(UiValue::Bool)
}

pub fn text_edit(shown: &str, committed: Option<&String>) -> Option<UiValue> {
    committed.filter(|v| *v != shown).map(|v| UiValue::Text(v.clone()))
}

pub fn index_edit(shown: u32, committed: Option<u32>) -> Option<UiValue> {
    committed.filter(|v| *v != shown).map(UiValue::Index)
}

pub fn selection_edit(shown: &[String], change: SelectionChange<String>) -> Option<UiValue> {
    let mut picked = Picked::new(shown.to_vec(), shown.last().cloned(), shown.first().cloned());
    picked.apply(change);
    (picked.items != shown).then_some(UiValue::Keys(picked.items))
}

pub fn choice_edit(options: &[Choice], shown: &str, committed: Option<u32>) -> Option<UiValue> {
    let id = &options.get(committed? as usize)?.id;
    (id != shown).then(|| UiValue::Text(id.clone()))
}

fn shared(text: &str) -> Label {
    Label::Shared(Arc::from(text))
}

fn tip(hint: Option<&String>) -> Option<Label> {
    hint.filter(|h| !h.is_empty()).map(|h| shared(h))
}

#[derive(Clone, Debug, PartialEq)]
struct Held {
    rev: Option<u64>,
    source: UiValue,
    shown: UiValue,
}

#[derive(Clone, Copy)]
enum Keep {
    ThisRevision,
    UntilSourceChanges,
}

#[derive(Default)]
pub struct FrameworkEngine {
    egui: EguiEngine,
    held: HashMap<String, HashMap<String, Held>>,
}

impl Engine for FrameworkEngine {
    fn draw(&mut self, ui: &mut Ui, surface: Surface<'_>, host: &mut dyn Host) -> Vec<UiEvent> {
        let held = self.held.entry(surface.key.to_owned()).or_default();
        held.retain(|_, h| h.rev.is_none_or(|rev| rev == surface.rev));
        let mut cx = Cx { surface, host, held, egui: &mut self.egui, out: Vec::new(), images: 0 };
        Stack::new().fill_width().id_salt(("craft-plugin-surface", surface.key)).show(ui, |ui, _| node(ui, &mut cx, surface.tree, 0));
        cx.out
    }

    fn forget(&mut self, key: &str) {
        self.held.remove(key);
        self.egui.forget(key);
    }
}

struct Cx<'a> {
    surface: Surface<'a>,
    host: &'a mut dyn Host,
    held: &'a mut HashMap<String, Held>,
    egui: &'a mut EguiEngine,
    out: Vec<UiEvent>,
    images: usize,
}

impl Cx<'_> {
    fn salt<'s>(&self, id: &'s str) -> (&str, &'s str) {
        (self.surface.key, id)
    }

    fn shown(&self, id: &str, source: UiValue) -> UiValue {
        match self.held.get(id) {
            Some(h) if h.source == source => h.shown.clone(),
            _ => source,
        }
    }

    fn emit(&mut self, id: &str, source: UiValue, value: Option<UiValue>, keep: Keep) {
        let Some(value) = value else { return };
        let rev = match keep {
            Keep::ThisRevision => Some(self.surface.rev),
            Keep::UntilSourceChanges => None,
        };
        self.held.insert(id.to_owned(), Held { rev, source, shown: value.clone() });
        self.send(id.to_owned(), value);
    }

    fn send(&mut self, widget: String, value: UiValue) {
        self.out.push(UiEvent { surface: self.surface.key.to_owned(), rev: self.surface.rev, widget, value });
    }

    fn widget(&mut self, id: &str, rect: egui::Rect, label: &str) {
        self.host.widget(self.surface.key, id, rect, label);
    }

    fn egui(&mut self, ui: &mut Ui, part: &Node) {
        let events = self.egui.draw_part(ui, self.surface, part, &mut *self.host);
        self.out.extend(events);
    }
}

fn kids(ui: &mut Ui, cx: &mut Cx<'_>, children: &[Node], depth: usize) {
    for c in children {
        node(ui, cx, c, depth + 1);
    }
}

fn number(v: &UiValue, or: f64) -> f64 {
    if let UiValue::Number(n) = v { *n } else { or }
}

fn boolean(v: &UiValue, or: bool) -> bool {
    if let UiValue::Bool(b) = v { *b } else { or }
}

fn index(v: &UiValue, or: u32) -> u32 {
    if let UiValue::Index(i) = v { *i } else { or }
}

fn text(v: UiValue, or: &str) -> String {
    if let UiValue::Text(t) = v { t } else { or.to_owned() }
}

fn keys(v: UiValue, or: &[String]) -> Vec<String> {
    if let UiValue::Keys(k) = v { k } else { or.to_vec() }
}

fn node(ui: &mut Ui, cx: &mut Cx<'_>, n: &Node, depth: usize) {
    match drawn(n) {
        _ if depth > MAX_DRAW_DEPTH => {}
        Drawn::Egui => cx.egui(ui, n),
        _ => component(ui, cx, n, depth),
    }
}

fn component(ui: &mut Ui, cx: &mut Cx<'_>, n: &Node, depth: usize) {
    match n {
        Node::Column(s) => {
            Stack::new().show(ui, |ui, _| kids(ui, cx, &s.children, depth));
        }
        Node::Row(s) => {
            Row::new().wrap(true).show(ui, |ui, _| kids(ui, cx, &s.children, depth));
        }
        Node::Grid(g) => {
            let columns = (g.columns as usize).clamp(1, MAX_GRID_COLUMNS);
            Grid::new(&GRID_COLUMNS[..columns]).show(ui, |ui, _| kids(ui, cx, &g.children, depth));
        }
        Node::Group(g) => {
            GroupBox::new(shared(&g.label), None).show(ui, |ui, _| kids(ui, cx, &g.children, depth));
        }
        Node::Scroll(s) => {
            let scroll = if s.horizontal { Scroll::both() } else { Scroll::vertical() };
            Stack::new().scroll(scroll).show(ui, |ui, _| kids(ui, cx, &s.children, depth));
        }
        Node::Collapsible(c) => {
            let source = UiValue::Bool(c.open);
            let shown = boolean(&cx.shown(&c.id, source.clone()), c.open);
            let mut open = shown;
            Section::new(shared(&c.label)).open(Bind::bound(&mut open)).id_salt(cx.salt(&c.id)).show(ui, |ui, _| kids(ui, cx, &c.children, depth));
            cx.emit(&c.id, source, bool_edit(shown, Some(open)), Keep::UntilSourceChanges);
        }
        Node::Label(l) => {
            let (role, shade) = type_role(l.style);
            let text = || {
                let t = Text::new(shared(&l.text)).role(role).shade(shade);
                if l.tone == Tone::Neutral { t } else { t.tone(tone(l.tone)) }
            };
            match &l.icon {
                Some(name) => {
                    Row::new().show(ui, |ui, _| {
                        Icon::new(icon(name)).show(ui);
                        text().show(ui);
                    });
                }
                None => {
                    text().show(ui);
                }
            }
        }
        Node::Separator => {
            Divider::horizontal().show(ui);
        }
        Node::Badge(b) => {
            Badge::new(shared(&b.text)).variant(badge(b.tone)).show(ui);
        }
        Node::Notice(n) => {
            match &n.title {
                Some(title) => Notice::new(tone(n.tone), shared(title)).lines([shared(&n.text)]).show(ui),
                None => Notice::new(tone(n.tone), shared(&n.text)).show(ui),
            };
        }
        Node::Button(b) => {
            let hint = tip(b.hint.as_ref());
            let out = match (&b.icon, b.icon_only) {
                (Some(name), true) => IconButton::new(icon(name))
                    .label(shared(&b.label))
                    .tooltip(hint.unwrap_or_else(|| shared(&b.label)))
                    .variant(variant(b.variant))
                    .enabled(!b.disabled)
                    .id_salt(cx.salt(&b.id))
                    .show(ui),
                (name, _) => {
                    let mut button = Button::new(shared(&b.label)).variant(variant(b.variant)).enabled(!b.disabled).id_salt(cx.salt(&b.id));
                    if let Some(name) = name {
                        button = button.icon(icon(name));
                    }
                    if let Some(hint) = hint {
                        button = button.tooltip(hint);
                    }
                    button.show(ui)
                }
            };
            cx.widget(&b.id, out.response.rect, &b.label);
            if out.clicked() {
                cx.send(b.id.clone(), UiValue::Click);
            }
        }
        Node::Checkbox(c) | Node::Toggle(c) => {
            let source = UiValue::Bool(c.value);
            let shown = boolean(&cx.shown(&c.id, source.clone()), c.value);
            let label = shared(&c.label);
            let out = if matches!(n, Node::Toggle(_)) {
                let mut w = Switch::new(Bind::given(shown)).label(label).enabled(!c.disabled).id_salt(cx.salt(&c.id));
                if let Some(h) = tip(c.hint.as_ref()) {
                    w = w.tooltip(h);
                }
                w.show(ui)
            } else {
                let mut w = Checkbox::new(Bind::given(shown)).label(label).enabled(!c.disabled).id_salt(cx.salt(&c.id));
                if let Some(h) = tip(c.hint.as_ref()) {
                    w = w.tooltip(h);
                }
                w.show(ui)
            };
            cx.widget(&c.id, out.response.rect, &c.label);
            cx.emit(&c.id, source, bool_edit(shown, out.events.committed().copied()), Keep::ThisRevision);
        }
        Node::Slider(s) => {
            let source = UiValue::Number(s.value);
            let shown = number(&cx.shown(&s.id, source.clone()), s.value);
            let range = if s.min.is_finite() && s.max.is_finite() && s.min < s.max { s.min..=s.max } else { 0.0..=1.0 };
            let mut w = SliderField::new(shared(&s.label), Bind::given(shown), range).enabled(!s.disabled).id_salt(cx.salt(&s.id));
            if let Some(step) = s.step {
                w = w.step(step);
            }
            if let Some(h) = tip(s.hint.as_ref()) {
                w = w.tooltip(h);
            }
            let out = w.show(ui);
            cx.widget(&s.id, out.response.rect, &s.label);
            cx.emit(&s.id, source, number_edit(shown, out.events.committed().copied()), Keep::ThisRevision);
        }
        Node::Number(nb) => {
            let source = UiValue::Number(nb.value);
            let shown = number(&cx.shown(&nb.id, source.clone()), nb.value);
            let value = number_field(
                ui,
                cx,
                &nb.id,
                &nb.label,
                shown,
                NumberLook {
                    min: nb.min,
                    max: nb.max,
                    step: nb.step,
                    decimals: nb.decimals,
                    unit: nb.unit.as_deref(),
                    hint: nb.hint.as_ref(),
                    enabled: !nb.disabled,
                },
            );
            cx.emit(&nb.id, source, number_edit(shown, value), Keep::ThisRevision);
        }
        Node::TextInput(t) => {
            let shown = text(cx.shown(&t.id, UiValue::Text(t.value.clone())), &t.value);
            let field = if t.search {
                TextField::search(shared(&t.label), Bind::given(shown.clone()))
            } else {
                TextField::new(shared(&t.label), Bind::given(shown.clone()))
            };
            let committed = text_field(ui, cx, &t.id, &t.label, field, t.placeholder.as_ref(), t.hint.as_ref(), !t.disabled, t.search);
            cx.emit(&t.id, UiValue::Text(t.value.clone()), text_edit(&shown, committed.as_ref()), Keep::ThisRevision);
        }
        Node::TextArea(t) => {
            let shown = text(cx.shown(&t.id, UiValue::Text(t.value.clone())), &t.value);
            let mut field = TextField::new(shared(&t.label), Bind::given(shown.clone())).lines(t.rows.unwrap_or(4).clamp(1, 50) as usize);
            if t.mono {
                field = field.mono();
            }
            let committed = text_field(ui, cx, &t.id, &t.label, field, t.placeholder.as_ref(), t.hint.as_ref(), !t.disabled, false);
            cx.emit(&t.id, UiValue::Text(t.value.clone()), text_edit(&shown, committed.as_ref()), Keep::ThisRevision);
        }
        Node::Select(s) => {
            let source = UiValue::Index(s.selected);
            let shown = index(&cx.shown(&s.id, source.clone()), s.selected);
            let options: Vec<&str> = s.options.iter().map(String::as_str).collect();
            let picked = select(ui, cx, &s.id, &s.label, &options, shown, s.hint.as_ref(), !s.disabled);
            cx.emit(&s.id, source, index_edit(shown, picked), Keep::ThisRevision);
        }
        Node::Radio(r) => {
            let source = UiValue::Index(r.selected);
            let shown = index(&cx.shown(&r.id, source.clone()), r.selected);
            let options: Vec<&str> = r.options.iter().map(String::as_str).collect();
            let picked = radio(ui, cx, &r.id, &r.label, &options, shown, r.horizontal, !r.disabled);
            cx.emit(&r.id, source, index_edit(shown, picked), Keep::ThisRevision);
        }
        Node::Tabs(t) => tabs(ui, cx, t, depth),
        Node::List(l) => {
            let source = UiValue::Keys(l.selected.clone());
            let shown = keys(cx.shown(&l.id, source.clone()), &l.selected);
            let items = keyed(&l.items, |item: &tree::Item| item.key.clone());
            let mut list = List::<String>::new(shared(&l.label))
                .id_salt(cx.salt(&l.id))
                .select(select_mode(l.selection))
                .header(false)
                .selection(Bind::given(Picked::new(shown.clone(), shown.last().cloned(), shown.first().cloned())));
            if let Some(empty) = &l.empty {
                list = list.empty(shared(empty));
            }
            let out = list.show_items(ui, &items, &mut |key, _| match l.items.iter().find(|i| &i.key == key) {
                Some(item) => {
                    let mut row = ListRow::new(shared(&item.label));
                    if let Some(d) = &item.detail {
                        row = row.detail(shared(d));
                    }
                    if let Some(name) = &item.icon {
                        row = row.icon(icon(name));
                    }
                    row
                }
                None => ListRow::new(Label::Empty),
            });
            cx.widget(&l.id, out.response.rect, &l.label);
            collection(cx, &l.id, source, &shown, out.events);
        }
        Node::Table(t) => {
            let source = UiValue::Keys(t.selected.clone());
            let shown = keys(cx.shown(&t.id, source.clone()), &t.selected);
            let columns: Vec<ColumnSpec> = t
                .columns
                .iter()
                .enumerate()
                .map(|(at, c)| {
                    let width = if at == 0 { ColumnWidth::fill(1.0) } else { ColumnWidth::Content };
                    ColumnSpec::new(ColumnKey::index(at as u32), shared(&c.label), width).align(column_align(c.align))
                })
                .collect();
            let rows = keyed(&t.rows, |row: &tree::Row| row.key.clone());
            let mut table = Table::<String>::new(shared(&t.label), columns)
                .id_salt(cx.salt(&t.id))
                .select(select_mode(t.selection))
                .selection(Bind::given(Picked::new(shown.clone(), shown.last().cloned(), shown.first().cloned())));
            if let Some(empty) = &t.empty {
                table = table.empty(shared(empty));
            }
            let out = table.show_items(ui, &rows, &mut |key, icx| match t.rows.iter().find(|r| &r.key == key) {
                Some(row) => {
                    let first = row.cells.first().map_or(Label::Empty, |c| shared(c));
                    row.cells
                        .iter()
                        .enumerate()
                        .skip(1)
                        .fold(ListRow::new(first).cells(icx.cells()), |r, (at, cell)| r.cell(ColumnKey::index(at as u32), shared(cell)))
                }
                None => ListRow::new(Label::Empty),
            });
            cx.widget(&t.id, out.response.rect, &t.label);
            collection(cx, &t.id, source, &shown, out.events);
        }
        Node::Progress(p) => {
            let label = p.label.as_deref().map(shared);
            match (p.value, label) {
                (Some(v), Some(l)) => Progress::new(Progress::clamped(v)).label(l).show(ui),
                (Some(v), None) => Progress::new(Progress::clamped(v)).show(ui),
                (None, Some(l)) => Spinner::new().label(l).show(ui),
                (None, None) => Spinner::new().show(ui),
            };
        }
        Node::Icon(i) => {
            let mut w = Icon::new(icon(&i.name));
            if let Some(l) = &i.label {
                w = w.label(shared(l)).tooltip(shared(l));
            }
            w.show(ui);
        }
        Node::Image(img) => {
            let at = cx.images;
            cx.images += 1;
            if img.hidden {
                return;
            }
            let (w, h) = (img.width as usize, img.height as usize);
            if w == 0 || h == 0 || w.checked_mul(h).and_then(|n| n.checked_mul(4)) != Some(img.rgba.len()) {
                return;
            }
            let mut hasher = DefaultHasher::new();
            (cx.surface.key, cx.surface.rev, at, &img.name).hash(&mut hasher);
            let paint = || ColorImage::from_rgba_unmultiplied([w, h], &img.rgba);
            Picture::new(img.alt.as_deref().map_or(Label::Empty, shared), hasher.finish(), &paint).show(ui);
        }
        Node::Form(f) => form(ui, cx, f),
        Node::Component(c) => match cx.host.component(ui, cx.surface.key, c) {
            Some(events) => {
                for (widget, value) in events {
                    cx.send(widget, value);
                }
            }
            None => node(ui, cx, &c.fallback, depth + 1),
        },
        Node::Unknown(u) => match &u.fallback {
            Some(f) => node(ui, cx, f, depth + 1),
            None => {
                let words = cx.host.tr(NEWER_APP);
                Notice::new(FwTone::Warn, Label::from(words)).show(ui);
            }
        },
        Node::Spacer(_) | Node::Canvas(_) | Node::Color(_) => {}
    }
}

fn collection<E>(cx: &mut Cx<'_>, id: &str, source: UiValue, shown: &[String], events: Vec<CollectionEvent<String, E>>) {
    for event in events {
        match event {
            CollectionEvent::Selection(change) => {
                let edit = selection_edit(shown, change);
                cx.emit(id, source.clone(), edit, Keep::UntilSourceChanges);
            }
            CollectionEvent::Activate(key) => cx.send(id.to_owned(), UiValue::Activate { activate: key }),
            _ => {}
        }
    }
}

struct NumberLook<'a> {
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    decimals: Option<u32>,
    unit: Option<&'a str>,
    hint: Option<&'a String>,
    enabled: bool,
}

fn number_field(ui: &mut Ui, cx: &mut Cx<'_>, id: &str, label: &str, shown: f64, look: NumberLook<'_>) -> Option<f64> {
    let mut w = NumberField::new(shared(label), Bind::given(Some(shown))).enabled(look.enabled).id_salt(cx.salt(id));
    if look.min.is_some() || look.max.is_some() {
        w = w.range(look.min.unwrap_or(f64::MIN)..=look.max.unwrap_or(f64::MAX));
    }
    if let Some(step) = look.step {
        w = w.step(step);
    }
    if let Some(d) = look.decimals {
        w = w.fixed_decimals(d as usize);
    }
    if let Some(unit) = look.unit {
        w = w.unit(shared(unit));
    }
    if let Some(h) = tip(look.hint) {
        w = w.tooltip(h);
    }
    let mut w = Some(w);
    let out = Field::new(shared(label)).id_salt(cx.salt(id)).show(ui, |ui, _| w.take().map(|w| w.show(ui))).inner.flatten()?;
    cx.widget(id, out.response.rect, label);
    out.events.committed().copied().flatten()
}

#[allow(clippy::too_many_arguments)]
fn text_field(
    ui: &mut Ui,
    cx: &mut Cx<'_>,
    id: &str,
    label: &str,
    field: TextField<'_>,
    placeholder: Option<&String>,
    hint: Option<&String>,
    enabled: bool,
    search: bool,
) -> Option<String> {
    let mut field = field.enabled(enabled).id_salt(cx.salt(id));
    if let Some(p) = placeholder {
        field = field.placeholder(shared(p));
    }
    if let Some(h) = tip(hint) {
        field = field.tooltip(h);
    }
    let out = if search {
        field.show(ui)
    } else {
        let mut field = Some(field);
        Field::new(shared(label)).id_salt(cx.salt(id)).show(ui, |ui, _| field.take().map(|f| f.show(ui))).inner.flatten()?
    };
    cx.widget(id, out.response.rect, label);
    out.events.committed().cloned()
}

#[allow(clippy::too_many_arguments)]
fn select(ui: &mut Ui, cx: &mut Cx<'_>, id: &str, label: &str, options: &[&str], shown: u32, hint: Option<&String>, enabled: bool) -> Option<u32> {
    let entries: Vec<Entry<u32>> = options.iter().enumerate().map(|(at, o)| Entry::new(at as u32, shared(o))).collect();
    let mut w = Select::new(shared(label), Bind::given(shown), &entries).enabled(enabled).id_salt(cx.salt(id));
    if let Some(h) = tip(hint) {
        w = w.tooltip(h);
    }
    let mut w = Some(w);
    let out = Field::new(shared(label)).id_salt(cx.salt(id)).show(ui, |ui, _| w.take().map(|w| w.show(ui))).inner.flatten()?;
    cx.widget(id, out.response.rect, label);
    out.events.committed().copied()
}

#[allow(clippy::too_many_arguments)]
fn radio(ui: &mut Ui, cx: &mut Cx<'_>, id: &str, label: &str, options: &[&str], shown: u32, horizontal: bool, enabled: bool) -> Option<u32> {
    let members = |ui: &mut Ui| {
        for (at, o) in options.iter().enumerate() {
            Radio::new(at as u32).label(shared(o)).show(ui);
        }
    };
    let out = RadioGroup::new(shared(label), Bind::given(shown)).enabled(enabled).id_salt(cx.salt(id)).show(ui, |ui, _| {
        if horizontal {
            Row::new().wrap(true).show(ui, |ui, _| members(ui));
        } else {
            Stack::new().show(ui, |ui, _| members(ui));
        }
    });
    cx.widget(id, out.output.response.rect, label);
    out.output.events.committed().copied()
}

fn tabs(ui: &mut Ui, cx: &mut Cx<'_>, t: &tree::Tabs, depth: usize) {
    let first = t.tabs.first().map(|tab| tab.id.clone()).unwrap_or_default();
    let source_id = t.selected.clone().unwrap_or(first);
    let source = UiValue::Text(source_id.clone());
    let mut shown = text(cx.shown(&t.id, source.clone()), &source_id);
    if !t.tabs.iter().any(|tab| tab.id == shown) {
        shown = source_id;
    }
    let ids: Vec<String> = t.tabs.iter().map(|tab| tab.id.clone()).collect();
    let source_keys = keyed(&ids, |id: &String| id.clone());
    let out = TabStrip::<String>::new(Label::Empty).id_salt(cx.salt(&t.id)).shown(shown.clone()).show_items(ui, &source_keys, &mut |key, _| match t
        .tabs
        .iter()
        .find(|tab| &tab.id == key)
    {
        Some(tab) => {
            let mut w = Tab::new(shared(&tab.label));
            if let Some(name) = &tab.icon {
                w = w.icon(icon(name));
            }
            w
        }
        None => Tab::new(Label::Empty),
    });
    cx.widget(&t.id, out.response.response.rect, &t.id);
    for event in out.response.events {
        if let CollectionEvent::Selection(SelectionChange::Set(picked)) = event
            && let Some(key) = picked.last()
        {
            cx.emit(&t.id, source.clone(), text_edit(&shown, Some(key)), Keep::UntilSourceChanges);
        }
    }
    let shown = text(cx.shown(&t.id, source), &shown);
    if let Some(tab) = t.tabs.iter().find(|tab| tab.id == shown) {
        Stack::new().id_salt(cx.salt(&tab.id)).show(ui, |ui, _| kids(ui, cx, &tab.children, depth));
    }
}

fn form_value(v: &FormValue) -> UiValue {
    match v {
        FormValue::Number(n) => UiValue::Number(*n),
        FormValue::Bool(b) => UiValue::Bool(*b),
        FormValue::String(s) => UiValue::Text(s.clone()),
        FormValue::Color(c) => UiValue::Color(*c),
        FormValue::Point(p) => UiValue::Point(*p),
    }
}

fn form(ui: &mut Ui, cx: &mut Cx<'_>, f: &tree::FormNode) {
    let state = FormState {
        rev: cx.surface.rev,
        values: f.values.clone(),
        hidden: f.hidden.iter().cloned().collect(),
        disabled: f.disabled.iter().cloned().collect(),
    };
    if f.form.validate_values(&state.values).is_err() {
        let words = cx.host.tr(INVALID_FORM);
        Notice::new(FwTone::Warn, Label::from(words)).show(ui);
        return;
    }
    let mut values = f.form.defaults();
    values.extend(state.values.clone());
    if cx.host.form_titles() && !f.form.title.english.trim().is_empty() {
        Text::new(shared(&f.form.title.english)).role(TypeRole::BodyStrong).show(ui);
    }
    let runs = sections(&f.form.params);
    for (title, params) in runs {
        let body = |ui: &mut Ui, cx: &mut Cx<'_>, values: &mut Values| {
            for p in params {
                if state.hidden.contains(&p.key) || p.visible_when.as_ref().is_some_and(|c| !c.eval(values)) {
                    continue;
                }
                let enabled = !state.disabled.contains(&p.key) && p.enabled_when.as_ref().is_none_or(|c| c.eval(values));
                if let Some((key, value)) = param(ui, cx, &f.form, p, values, enabled) {
                    values.insert(key, value);
                }
            }
        };
        match title {
            Some(section) => {
                let mut values_in = values.clone();
                Section::fixed(shared(&section.label.english)).id_salt(cx.salt(&section.key)).show(ui, |ui, _| body(ui, cx, &mut values_in));
                values = values_in;
            }
            None => body(ui, cx, &mut values),
        }
    }
}

fn sections(params: &[Param]) -> Vec<(Option<&Param>, &[Param])> {
    let mut runs = Vec::new();
    let mut start = 0;
    let mut title = None;
    for (at, p) in params.iter().enumerate() {
        if matches!(p.kind, ParamKind::Section) {
            if at > start || title.is_some() {
                runs.push((title, &params[start..at]));
            }
            title = Some(p);
            start = at + 1;
        }
    }
    runs.push((title, &params[start..]));
    runs
}

fn param(ui: &mut Ui, cx: &mut Cx<'_>, form: &Form, p: &Param, values: &Values, enabled: bool) -> Option<(String, FormValue)> {
    let current = values.get(&p.key)?.clone();
    let source = form_value(&current);
    let shown = cx.shown(&p.key, source.clone());
    let help = p.help.as_ref().map(|h| h.english.clone());
    let label = p.label.english.as_str();
    let unit = p.unit.as_ref().map(unit_label);
    let edit = match (&p.kind, &current) {
        (ParamKind::Range { min, max, step, .. }, FormValue::Number(v)) => {
            let shown = number(&shown, *v);
            let mut w = SliderField::new(shared(label), Bind::given(shown), *min..=*max).enabled(enabled).id_salt(cx.salt(&p.key));
            if let Some(step) = step {
                w = w.step(*step);
            }
            if let Some(unit) = unit {
                w = w.unit(shared(unit));
            }
            if let Some(h) = tip(help.as_ref()) {
                w = w.tooltip(h);
            }
            let out = w.show(ui);
            cx.widget(&p.key, out.response.rect, label);
            number_edit(shown, out.events.committed().copied())
        }
        (ParamKind::Int { min, max, .. }, FormValue::Number(v)) => {
            let shown = number(&shown, *v);
            let look =
                NumberLook { min: Some(*min as f64), max: Some(*max as f64), step: Some(1.0), decimals: Some(0), unit, hint: help.as_ref(), enabled };
            number_edit(shown, number_field(ui, cx, &p.key, label, shown, look).map(f64::round))
        }
        (ParamKind::Number { min, max, .. }, FormValue::Number(v)) => {
            let shown = number(&shown, *v);
            let look = NumberLook { min: *min, max: *max, step: None, decimals: None, unit, hint: help.as_ref(), enabled };
            number_edit(shown, number_field(ui, cx, &p.key, label, shown, look))
        }
        (ParamKind::Angle { .. }, FormValue::Number(v)) => {
            let shown = number(&shown, *v);
            let look =
                NumberLook { min: None, max: None, step: None, decimals: None, unit: Some(unit.unwrap_or("°")), hint: help.as_ref(), enabled };
            number_edit(shown, number_field(ui, cx, &p.key, label, shown, look))
        }
        (ParamKind::Bool { .. }, FormValue::Bool(v)) => {
            let shown = boolean(&shown, *v);
            let mut w = Checkbox::new(Bind::given(shown)).label(shared(label)).enabled(enabled).id_salt(cx.salt(&p.key));
            if let Some(h) = tip(help.as_ref()) {
                w = w.tooltip(h);
            }
            let out = w.show(ui);
            cx.widget(&p.key, out.response.rect, label);
            bool_edit(shown, out.events.committed().copied())
        }
        (ParamKind::Choice { options, .. } | ParamKind::Radio { options, .. }, FormValue::String(v)) => {
            let shown = text(shown, v);
            let at = options.iter().position(|o| o.id == shown).unwrap_or(0) as u32;
            let labels: Vec<&str> = options.iter().map(|o| o.label.english.as_str()).collect();
            let picked = if matches!(p.kind, ParamKind::Radio { .. }) {
                radio(ui, cx, &p.key, label, &labels, at, true, enabled)
            } else {
                select(ui, cx, &p.key, label, &labels, at, help.as_ref(), enabled)
            };
            choice_edit(options, &shown, picked)
        }
        (ParamKind::Text { multiline, .. }, FormValue::String(v)) => {
            let shown = text(shown, v);
            let mut field = TextField::new(shared(label), Bind::given(shown.clone()));
            if *multiline {
                field = field.lines(4);
            }
            let committed = text_field(ui, cx, &p.key, label, field, None, help.as_ref(), enabled, false);
            text_edit(&shown, committed.as_ref())
        }
        _ => {
            egui_param(ui, cx, form, p, &current, enabled);
            None
        }
    }?;
    let value = match (&edit, &current) {
        (UiValue::Number(n), _) => FormValue::Number(*n),
        (UiValue::Bool(b), _) => FormValue::Bool(*b),
        (UiValue::Text(t), _) => FormValue::String(t.clone()),
        _ => return None,
    };
    let candidate: Values = [(p.key.clone(), value.clone())].into_iter().collect();
    if !enabled || form.validate_values(&candidate).is_err() {
        return None;
    }
    cx.emit(&p.key, source, Some(edit), Keep::ThisRevision);
    Some((p.key.clone(), value))
}

fn egui_param(ui: &mut Ui, cx: &mut Cx<'_>, form: &Form, p: &Param, current: &FormValue, enabled: bool) {
    let one = Form { params: vec![Param { visible_when: None, enabled_when: None, ..p.clone() }], preview: None, ..form.clone() };
    let state = FormState {
        rev: cx.surface.rev,
        values: [(p.key.clone(), current.clone())].into_iter().collect(),
        hidden: Default::default(),
        disabled: if enabled { Default::default() } else { [p.key.clone()].into_iter().collect() },
    };
    let salt = egui::Id::unique(("craft-plugin-fw-form", cx.surface.key, &form.key));
    let edits = form_rows(ui, salt, cx.surface.key, cx.surface.rev, &one, &state, FormOptions { live: false, title: false }, &mut *cx.host);
    for (key, value) in edits {
        cx.send(key, form_value(&value));
    }
}

#[cfg(test)]
mod tests;

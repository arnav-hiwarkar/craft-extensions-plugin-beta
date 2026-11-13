//! The egui engine: draws every node of UI level 1.1, forms and canvases.
//!
//! Edit semantics, kept from the three app renderers it replaces:
//! - sliders and number fields keep their own value while dragged and send once when released (or
//!   on a click or key change), so a redraw mid-drag never snaps the value back;
//! - text fields send on Enter or when they lose focus, and take the plugin's new value only when
//!   the person is not typing;
//! - form rows behave the same per parameter, and only valid values are sent;
//! - every event carries the revision that was drawn; the app drops stale ones.

use super::form::{FormOptions, form_rows};
use super::icons::paint_icon;
use super::paint::{Xf, color, paint_ops};
use super::{Engine, Host, Surface};
use craft_ext_core::form::{FormState, Value as FValue};
use craft_ext_core::ui::tree::{
    Align, Canvas, Cursor, Listen, Mods, Node, PointerButton, PointerEvent, PointerKind, Selection, TextStyle, UiEvent, UiValue,
};
use egui::{Color32, Id, Pos2, Rect, Response, RichText, Sense, TextureHandle, Ui, Vec2, pos2, vec2};
use std::collections::HashMap;

/// Deepest tree drawn. `Node::validate` already bounds depth; this keeps drawing bounded anyway.
const MAX_DRAW_DEPTH: usize = 32;

/// The egui engine. Keeps image textures per surface revision.
#[derive(Default)]
pub struct EguiEngine {
    textures: HashMap<String, Textures>,
}

#[derive(Default)]
struct Textures {
    rev: Option<u64>,
    named: HashMap<String, TextureHandle>,
    anonymous: Vec<TextureHandle>,
}

struct Cx<'a> {
    surface: &'a str,
    rev: u64,
    salt: Id,
    host: &'a mut dyn Host,
    textures: &'a mut Textures,
    images_seen: usize,
    seq: usize,
    out: Vec<(String, UiValue)>,
}

impl Cx<'_> {
    fn id(&self, what: &str, id: &str) -> Id {
        self.salt.with((what, id))
    }
    /// A salt for containers without an id: their order in the tree.
    fn next(&mut self, what: &str) -> Id {
        self.seq += 1;
        self.salt.with((what, self.seq))
    }
    fn emit(&mut self, widget: &str, value: UiValue) {
        self.out.push((widget.to_string(), value));
    }
    fn widget(&mut self, id: &str, r: &Response, label: &str) {
        self.host.widget(self.surface, id, r.rect, label);
    }
}

impl Engine for EguiEngine {
    fn draw(&mut self, ui: &mut Ui, surface: Surface<'_>, host: &mut dyn Host) -> Vec<UiEvent> {
        let textures = self.textures.entry(surface.key.to_string()).or_default();
        if textures.rev != Some(surface.rev) {
            *textures = Textures { rev: Some(surface.rev), ..Textures::default() };
            load_named(ui.ctx(), surface.key, surface.tree, textures, 0);
        }
        let mut cx = Cx {
            surface: surface.key,
            rev: surface.rev,
            salt: Id::unique(("craft-plugin-ui", surface.key)),
            host,
            textures,
            images_seen: 0,
            seq: 0,
            out: Vec::new(),
        };
        ui.push_id(cx.salt, |ui| node(ui, &mut cx, surface.tree, 0));
        cx.out.into_iter().map(|(widget, value)| UiEvent { surface: surface.key.to_string(), rev: surface.rev, widget, value }).collect()
    }

    fn forget(&mut self, key: &str) {
        self.textures.remove(key);
    }
}

fn texture(ctx: &egui::Context, name: String, img: &craft_ext_core::ui::tree::Image) -> Option<TextureHandle> {
    let (w, h) = (img.width as usize, img.height as usize);
    if w == 0 || h == 0 || w.checked_mul(h).and_then(|n| n.checked_mul(4)) != Some(img.rgba.len()) {
        return None;
    }
    Some(ctx.load_texture(name, egui::ColorImage::from_rgba_unmultiplied([w, h], &img.rgba), egui::TextureOptions::LINEAR))
}

/// Named images are loaded before drawing, since a canvas may come before the image it draws.
fn load_named(ctx: &egui::Context, key: &str, n: &Node, t: &mut Textures, depth: usize) {
    if depth > MAX_DRAW_DEPTH {
        return;
    }
    if let Node::Image(img) = n
        && let Some(name) = &img.name
        && !t.named.contains_key(name)
        && let Some(tex) = texture(ctx, format!("plugin-image-{key}-{name}"), img)
    {
        t.named.insert(name.clone(), tex);
    }
    for c in n.children() {
        load_named(ctx, key, c, t, depth + 1);
    }
}

fn with_hint(r: Response, hint: Option<&str>) -> Response {
    match hint {
        Some(h) if !h.is_empty() => r.on_hover_text(h),
        _ => r,
    }
}

fn node(ui: &mut Ui, cx: &mut Cx<'_>, n: &Node, depth: usize) {
    if depth > MAX_DRAW_DEPTH {
        return;
    }
    let kids = |ui: &mut Ui, cx: &mut Cx<'_>, children: &[Node]| children.iter().for_each(|c| node(ui, cx, c, depth + 1));
    match n {
        Node::Column(s) => {
            ui.vertical(|ui| kids(ui, cx, &s.children));
        }
        Node::Row(s) => {
            ui.horizontal_wrapped(|ui| kids(ui, cx, &s.children));
        }
        Node::Group(g) => {
            ui.group(|ui| {
                ui.label(RichText::new(&g.label).strong());
                kids(ui, cx, &g.children);
            });
        }
        Node::Label(l) => {
            let text = match l.style {
                TextStyle::Body => RichText::new(&l.text),
                TextStyle::Heading => RichText::new(&l.text).heading(),
                TextStyle::Strong => RichText::new(&l.text).strong(),
                TextStyle::Small => RichText::new(&l.text).small(),
                TextStyle::Weak => RichText::new(&l.text).weak(),
                TextStyle::Mono => RichText::new(&l.text).monospace(),
            };
            match &l.icon {
                Some(icon) => {
                    ui.horizontal(|ui| {
                        icon_widget(ui, icon, ui.text_style_height(&egui::TextStyle::Body), None);
                        ui.label(text);
                    });
                }
                None => {
                    ui.label(text);
                }
            }
        }
        Node::Separator => {
            ui.separator();
        }
        Node::Spacer(s) => ui.add_space(s.size.clamp(0.0, 512.0)),
        Node::Button(b) => {
            let r = match &b.icon {
                Some(icon) => icon_button(ui, icon, if b.icon_only { None } else { Some(&b.label) }, !b.disabled),
                None => ui.add_enabled(!b.disabled, egui::Button::new(&b.label)),
            };
            cx.widget(&b.id, &r, &b.label);
            let hint = b.hint.as_deref().or(if b.icon_only { Some(&b.label) } else { None });
            if with_hint(r, hint).clicked() {
                cx.emit(&b.id, UiValue::Click);
            }
        }
        Node::Checkbox(c) | Node::Toggle(c) => {
            let mut v = c.value;
            let r =
                ui.add_enabled_ui(!c.disabled, |ui| {
                    if matches!(n, Node::Toggle(_)) { toggle(ui, &mut v, &c.label) } else { ui.checkbox(&mut v, &c.label) }
                })
                .inner;
            cx.widget(&c.id, &r, &c.label);
            if with_hint(r, c.hint.as_deref()).changed() && v != c.value {
                cx.emit(&c.id, UiValue::Bool(v));
            }
        }
        Node::Slider(s) => {
            let (lo, hi) = if s.min.is_finite() && s.max.is_finite() && s.min < s.max { (s.min, s.max) } else { (0.0, 1.0) };
            let step = s.step;
            let label = s.label.clone();
            drag_number(ui, cx, &s.id, &s.label, s.value, s.disabled, s.hint.as_deref(), move |ui, v| {
                let mut w = egui::Slider::new(v, lo..=hi).text(label);
                if let Some(st) = step {
                    w = w.step_by(st);
                }
                ui.add(w)
            });
        }
        Node::Number(nb) => {
            let (lo, hi) = (nb.min.unwrap_or(f64::MIN), nb.max.unwrap_or(f64::MAX));
            let (step, decimals, unit) = (nb.step, nb.decimals, nb.unit.clone());
            ui.horizontal(|ui| {
                ui.label(&nb.label);
                drag_number(ui, cx, &nb.id, &nb.label, nb.value, nb.disabled, nb.hint.as_deref(), move |ui, v| {
                    let mut w = egui::DragValue::new(v).range(lo..=hi);
                    if let Some(st) = step {
                        w = w.speed(st);
                    }
                    if let Some(d) = decimals {
                        w = w.fixed_decimals(d as usize);
                    }
                    if let Some(u) = unit {
                        w = w.suffix(format!(" {u}"));
                    }
                    ui.add(w)
                });
            });
        }
        Node::TextInput(t) => {
            text_field(ui, cx, &t.id, &t.label, &t.value, t.placeholder.as_deref(), t.hint.as_deref(), t.disabled, None);
        }
        Node::TextArea(t) => {
            let rows = t.rows.unwrap_or(4).clamp(1, 50) as usize;
            text_field(ui, cx, &t.id, &t.label, &t.value, t.placeholder.as_deref(), t.hint.as_deref(), t.disabled, Some((rows, t.mono)));
        }
        Node::Select(s) => {
            let mut sel = s.selected;
            let shown = s.options.get(sel as usize).cloned().unwrap_or_default();
            let r = ui
                .add_enabled_ui(!s.disabled, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(&s.label);
                        egui::ComboBox::from_id_salt(cx.id("select", &s.id))
                            .selected_text(shown)
                            .show_ui(ui, |ui| {
                                for (i, o) in s.options.iter().enumerate() {
                                    ui.selectable_value(&mut sel, i as u32, o);
                                }
                            })
                            .response
                    })
                    .inner
                })
                .inner;
            cx.widget(&s.id, &r, &s.label);
            with_hint(r, s.hint.as_deref());
            if sel != s.selected {
                cx.emit(&s.id, UiValue::Index(sel));
            }
        }
        Node::Radio(rd) => {
            let mut sel = rd.selected;
            let inner = |ui: &mut Ui, sel: &mut u32| {
                for (i, o) in rd.options.iter().enumerate() {
                    ui.radio_value(sel, i as u32, o);
                }
            };
            let r = ui
                .add_enabled_ui(!rd.disabled, |ui| {
                    ui.label(&rd.label);
                    if rd.horizontal {
                        ui.horizontal_wrapped(|ui| inner(ui, &mut sel)).response
                    } else {
                        ui.vertical(|ui| inner(ui, &mut sel)).response
                    }
                })
                .inner;
            cx.widget(&rd.id, &r, &rd.label);
            with_hint(r, rd.hint.as_deref());
            if sel != rd.selected {
                cx.emit(&rd.id, UiValue::Index(sel));
            }
        }
        Node::Color(c) => {
            let r = ui
                .add_enabled_ui(!c.disabled, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(&c.label);
                        let mut v = c.rgba.map(|x| if x.is_finite() { x.clamp(0.0, 1.0) } else { 0.0 });
                        let r = ui.color_edit_button_rgba_unmultiplied(&mut v);
                        (r, v)
                    })
                    .inner
                })
                .inner;
            let (r, v) = r;
            cx.widget(&c.id, &r, &c.label);
            if with_hint(r, c.hint.as_deref()).changed() {
                cx.emit(&c.id, UiValue::Color(v));
            }
        }
        Node::Image(img) => {
            let tex = match &img.name {
                Some(name) => cx.textures.named.get(name).cloned(),
                None => {
                    let i = cx.images_seen;
                    cx.images_seen += 1;
                    if cx.textures.anonymous.len() <= i
                        && let Some(t) = texture(ui.ctx(), format!("plugin-image-{}-{}-{i}", cx.surface, cx.rev), img)
                    {
                        cx.textures.anonymous.push(t);
                    }
                    cx.textures.anonymous.get(i).cloned()
                }
            };
            if img.hidden {
                return;
            }
            if let Some(tex) = tex {
                let size = img.size.map_or(vec2(img.width as f32, img.height as f32), |[w, h]| vec2(w, h));
                let r = ui.image((tex.id(), size));
                with_hint(r, img.alt.as_deref());
            }
        }
        Node::Icon(i) => {
            let size = i.size.unwrap_or(16.0).clamp(1.0, 256.0);
            let r = icon_widget(ui, &i.name, size, i.rgba.map(color));
            with_hint(r, i.label.as_deref());
        }
        Node::Progress(p) => match p.value {
            Some(v) => {
                let mut bar = egui::ProgressBar::new(v.clamp(0.0, 1.0));
                bar = match &p.label {
                    Some(l) => bar.text(l.as_str()),
                    None => bar.show_percentage(),
                };
                ui.add(bar);
            }
            None => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    if let Some(l) = &p.label {
                        ui.label(l);
                    }
                });
            }
        },
        Node::Tabs(t) => tabs(ui, cx, t, depth),
        Node::Collapsible(c) => {
            let id = cx.id("collapsible", &c.id);
            let (mut open, source) = ui.ctx().data_mut(|d| d.get_temp::<(bool, bool)>(id)).unwrap_or((c.open, c.open));
            if source != c.open {
                open = c.open;
            }
            let r = egui::CollapsingHeader::new(&c.label).id_salt(id).open(Some(open)).show(ui, |ui| kids(ui, cx, &c.children));
            cx.widget(&c.id, &r.header_response, &c.label);
            if r.header_response.clicked() {
                open = !open;
                cx.emit(&c.id, UiValue::Bool(open));
            }
            ui.ctx().data_mut(|d| d.insert_temp(id, (open, c.open)));
        }
        Node::Scroll(s) => {
            let area = if s.horizontal { egui::ScrollArea::both() } else { egui::ScrollArea::vertical() };
            let mut area = area.id_salt(cx.next("scroll"));
            if let Some(h) = s.max_height {
                area = area.max_height(h.clamp(1.0, 8192.0));
            }
            area.show(ui, |ui| kids(ui, cx, &s.children));
        }
        Node::Grid(g) => {
            let cols = g.columns.clamp(1, 16) as usize;
            egui::Grid::new(cx.next("grid")).num_columns(cols).striped(g.striped).show(ui, |ui| {
                for (i, c) in g.children.iter().enumerate() {
                    match c {
                        // `add_space` is not allowed in a grid cell: an empty cell of that size.
                        Node::Spacer(s) => {
                            ui.allocate_space(Vec2::splat(s.size.clamp(0.0, 512.0)));
                        }
                        _ => node(ui, cx, c, depth + 1),
                    }
                    if (i + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            });
        }
        Node::Table(t) => {
            let keys: Vec<&str> = t.rows.iter().map(|r| r.key.as_str()).collect();
            let font = egui::TextStyle::Body.resolve(ui.style());
            let measure = |ui: &Ui, s: &str| ui.painter().layout_no_wrap(s.to_string(), font.clone(), Color32::WHITE).size().x;
            let widths: Vec<f32> = t
                .columns
                .iter()
                .enumerate()
                .map(|(c, col)| {
                    col.width.unwrap_or_else(|| {
                        let cells = t.rows.iter().take(200).filter_map(|r| r.cells.get(c)).map(|s| measure(ui, s));
                        cells.fold(measure(ui, &col.label), f32::max).clamp(40.0, 400.0) + 12.0
                    })
                })
                .collect();
            let aligns: Vec<Align> = t.columns.iter().map(|c| c.align).collect();
            let header: Vec<&str> = t.columns.iter().map(|c| c.label.as_str()).collect();
            let paint = |p: &egui::Painter, rect: Rect, i: usize, ink: Color32| {
                if let Some(row) = t.rows.get(i) {
                    cells(p, rect, &widths, &aligns, row.cells.iter().map(String::as_str), font.clone(), ink);
                }
            };
            let head = |p: &egui::Painter, rect: Rect, ink: Color32| {
                cells(p, rect, &widths, &aligns, header.iter().copied(), font.clone(), ink);
            };
            rows(ui, cx, &t.id, &t.label, t.hint.as_deref(), &keys, t.selection, &t.selected, t.max_height, widths.iter().sum(), Some(&head), &paint);
        }
        Node::List(l) => {
            let keys: Vec<&str> = l.items.iter().map(|r| r.key.as_str()).collect();
            let font = egui::TextStyle::Body.resolve(ui.style());
            let width = ui.available_width().max(120.0);
            let paint = |p: &egui::Painter, rect: Rect, i: usize, ink: Color32| {
                let Some(item) = l.items.get(i) else { return };
                let mut x = rect.left() + 4.0;
                if let Some(icon) = &item.icon {
                    let side = rect.height() * 0.7;
                    paint_icon(p, Rect::from_center_size(pos2(x + side / 2.0, rect.center().y), vec2(side, side)), icon, ink);
                    x += side + 6.0;
                }
                let g = p.layout_no_wrap(item.label.clone(), font.clone(), ink);
                let w = g.size().x;
                p.galley(pos2(x, rect.center().y - g.size().y / 2.0), g, ink);
                if let Some(d) = &item.detail {
                    let weak = ink.gamma_multiply(0.6);
                    let g = p.layout_no_wrap(d.clone(), font.clone(), weak);
                    p.galley(pos2(x + w + 8.0, rect.center().y - g.size().y / 2.0), g, weak);
                }
            };
            rows(ui, cx, &l.id, &l.label, l.hint.as_deref(), &keys, l.selection, &l.selected, l.max_height, width, None, &paint);
        }
        Node::Canvas(c) => canvas(ui, cx, c),
        Node::Form(f) => {
            let state = FormState {
                rev: cx.rev,
                values: f.values.clone(),
                hidden: f.hidden.iter().cloned().collect(),
                disabled: f.disabled.iter().cloned().collect(),
            };
            let opts = FormOptions { live: false, title: cx.host.form_titles() };
            let salt = cx.salt.with("form");
            let edits = form_rows(ui, salt, cx.surface, cx.rev, &f.form, &state, opts, cx.host);
            for (k, v) in edits {
                let value = match v {
                    FValue::Number(n) => UiValue::Number(n),
                    FValue::Bool(b) => UiValue::Bool(b),
                    FValue::String(s) => UiValue::Text(s),
                    FValue::Color(c) => UiValue::Color(c),
                    FValue::Point(p) => UiValue::Point(p),
                };
                cx.emit(&k, value);
            }
        }
        Node::Component(c) => match cx.host.component(ui, cx.surface, c) {
            Some(events) => cx.out.extend(events),
            None => node(ui, cx, &c.fallback, depth + 1),
        },
        Node::Unknown(u) => match &u.fallback {
            Some(f) => node(ui, cx, f, depth + 1),
            None => {
                let text = cx.host.tr("This part of the panel needs a newer version of the app.");
                ui.horizontal_wrapped(|ui| {
                    let h = ui.text_style_height(&egui::TextStyle::Body);
                    icon_widget(ui, "warning", h, Some(ui.visuals().warn_fg_color));
                    ui.weak(text).on_hover_text(format!("\"{}\"", u.kind));
                });
            }
        },
    }
}

/// A slider or number field: keeps its own value while dragged, sends once when the edit ends.
#[allow(clippy::too_many_arguments)]
fn drag_number(
    ui: &mut Ui,
    cx: &mut Cx<'_>,
    id: &str,
    label: &str,
    value: f64,
    disabled: bool,
    hint: Option<&str>,
    add: impl FnOnce(&mut Ui, &mut f64) -> Response,
) {
    let eid = cx.id("drag", id);
    let (mut v, dragging, old_rev, source) = ui.ctx().data_mut(|d| d.get_temp::<(f64, bool, u64, f64)>(eid)).unwrap_or((value, false, cx.rev, value));
    if !dragging && (old_rev != cx.rev || source != value) {
        v = value;
    }
    let r = ui.add_enabled_ui(!disabled, |ui| add(ui, &mut v)).inner;
    let active = r.dragged() || r.is_pointer_button_down_on() || r.has_focus();
    ui.ctx().data_mut(|d| d.insert_temp(eid, (v, active, cx.rev, value)));
    cx.widget(id, &r, label);
    let r = with_hint(r, hint);
    if (r.drag_stopped() || r.lost_focus() || (dragging && !active) || (r.changed() && !active)) && v.is_finite() && v != value {
        cx.emit(id, UiValue::Number(v));
    }
}

/// A single-line (`area: None`) or multi-line text field. Sends on Enter (Ctrl/Cmd+Enter in an
/// area) or when focus leaves; takes the plugin's value only while the person is not typing.
#[allow(clippy::too_many_arguments)]
fn text_field(
    ui: &mut Ui,
    cx: &mut Cx<'_>,
    id: &str,
    label: &str,
    value: &str,
    placeholder: Option<&str>,
    hint: Option<&str>,
    disabled: bool,
    area: Option<(usize, bool)>,
) {
    let eid = cx.id("text", id);
    let (mut v, mut sent, old) =
        ui.ctx().data_mut(|d| d.get_temp::<(String, String, String)>(eid)).unwrap_or_else(|| (value.into(), value.into(), value.into()));
    let mut focused = false;
    let mut commit = false;
    let mut rect = Rect::NOTHING;
    ui.add_enabled_ui(!disabled, |ui| {
        let mut body = |ui: &mut Ui| {
            ui.label(label);
            let mut edit = match area {
                Some((rows, mono)) => {
                    let e = egui::TextEdit::multiline(&mut v).desired_rows(rows);
                    if mono { e.code_editor() } else { e }
                }
                None => egui::TextEdit::singleline(&mut v),
            };
            if let Some(p) = placeholder {
                edit = edit.hint_text(p.to_string());
            }
            let r = with_hint(ui.add(edit), hint);
            focused = r.has_focus();
            rect = r.rect;
            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter) && (area.is_none() || i.modifiers.command));
            commit = r.lost_focus() || (focused && enter);
        };
        if area.is_some() {
            ui.vertical(|ui| body(ui));
        } else {
            ui.horizontal(|ui| body(ui));
        }
    });
    cx.host.widget(cx.surface, id, rect, label);
    if commit && v != sent {
        cx.emit(id, UiValue::Text(v.clone()));
        sent = v.clone();
    }
    if !focused && old != value {
        v = value.to_string();
        sent = value.to_string();
    }
    ui.ctx().data_mut(|d| d.insert_temp(eid, (v, sent, value.to_string())));
}

fn tabs(ui: &mut Ui, cx: &mut Cx<'_>, t: &craft_ext_core::ui::tree::Tabs, depth: usize) {
    let id = cx.id("tabs", &t.id);
    let first = t.tabs.first().map(|t| t.id.clone()).unwrap_or_default();
    let source = t.selected.clone().unwrap_or_else(|| first.clone());
    let (mut shown, old) = ui.ctx().data_mut(|d| d.get_temp::<(String, String)>(id)).unwrap_or((source.clone(), source.clone()));
    if old != source || !t.tabs.iter().any(|tab| tab.id == shown) {
        shown = source.clone();
    }
    ui.horizontal_wrapped(|ui| {
        for tab in &t.tabs {
            let on = tab.id == shown;
            let r = match &tab.icon {
                Some(icon) => {
                    let r = icon_button(ui, icon, Some(&tab.label), true);
                    if on {
                        ui.painter().rect_stroke(r.rect, 3.0, ui.visuals().selection.stroke, egui::StrokeKind::Inside);
                    }
                    r
                }
                None => ui.selectable_label(on, &tab.label),
            };
            cx.widget(&format!("{}/{}", t.id, tab.id), &r, &tab.label);
            if r.clicked() && !on {
                shown = tab.id.clone();
                cx.emit(&t.id, UiValue::Text(shown.clone()));
            }
        }
    });
    ui.separator();
    if let Some(tab) = t.tabs.iter().find(|tab| tab.id == shown) {
        ui.push_id(("tab", &tab.id), |ui| {
            ui.vertical(|ui| tab.children.iter().for_each(|c| node(ui, cx, c, depth + 1)));
        });
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, (shown, source)));
}

/// Paints one row of cells, each clipped to its column.
fn cells<'a>(
    p: &egui::Painter,
    rect: Rect,
    widths: &[f32],
    aligns: &[Align],
    texts: impl Iterator<Item = &'a str>,
    font: egui::FontId,
    ink: Color32,
) {
    let mut x = rect.left();
    for ((text, w), align) in texts.zip(widths).zip(aligns) {
        let cell = Rect::from_min_size(pos2(x, rect.top()), vec2(*w, rect.height())).shrink2(vec2(6.0, 0.0));
        let g = p.layout_no_wrap(text.to_string(), font.clone(), ink);
        let gx = match align {
            Align::Left => cell.left(),
            Align::Center => cell.center().x - g.size().x / 2.0,
            Align::Right => cell.right() - g.size().x,
        };
        p.with_clip_rect(cell.intersect(p.clip_rect())).galley(pos2(gx, cell.center().y - g.size().y / 2.0), g, ink);
        x += w;
    }
}

type PaintRow<'a> = dyn Fn(&egui::Painter, Rect, usize, Color32) + 'a;
type PaintHead<'a> = dyn Fn(&egui::Painter, Rect, Color32) + 'a;

/// A virtualised list of selectable rows (table body or list). Sends the selected keys on change
/// and `{"activate": key}` on a double click.
#[allow(clippy::too_many_arguments)]
fn rows(
    ui: &mut Ui,
    cx: &mut Cx<'_>,
    id: &str,
    label: &str,
    hint: Option<&str>,
    keys: &[&str],
    mode: Selection,
    source: &[String],
    max_height: Option<f32>,
    width: f32,
    head: Option<&PaintHead<'_>>,
    paint: &PaintRow<'_>,
) {
    let sid = cx.id("rows", id);
    let (mut selected, old, mut anchor) = ui
        .ctx()
        .data_mut(|d| d.get_temp::<(Vec<String>, Vec<String>, Option<usize>)>(sid))
        .unwrap_or_else(|| (source.to_vec(), source.to_vec(), None));
    if old != source {
        selected = source.to_vec();
        anchor = None;
    }
    let before = selected.clone();
    let row_h = ui.spacing().interact_size.y;
    let caption = ui.label(RichText::new(label).strong());
    let caption = with_hint(caption, hint);
    let top = caption.rect;
    if let Some(head) = head {
        let (rect, _) = ui.allocate_exact_size(vec2(width, row_h), Sense::hover());
        head(ui.painter(), rect, ui.visuals().strong_text_color());
        ui.painter().hline(rect.x_range(), rect.bottom(), ui.visuals().widgets.noninteractive.bg_stroke);
    }
    let mut activated = None;
    let area = egui::ScrollArea::vertical().id_salt(sid).max_height(max_height.unwrap_or(240.0).clamp(row_h, 8192.0)).auto_shrink([false, true]);
    let out = area.show_rows(ui, row_h, keys.len(), |ui, range| {
        for i in range {
            let Some(key) = keys.get(i) else { continue };
            let (rect, r) = ui.allocate_exact_size(vec2(width.max(ui.available_width()), row_h), Sense::click());
            let on = selected.iter().any(|k| k == key);
            let visuals = ui.visuals();
            let bg = if on {
                Some(visuals.selection.bg_fill)
            } else if r.hovered() && mode != Selection::None {
                Some(visuals.widgets.hovered.weak_bg_fill)
            } else if i % 2 == 1 {
                Some(visuals.faint_bg_color)
            } else {
                None
            };
            if let Some(bg) = bg {
                ui.painter().rect_filled(rect, 2.0, bg);
            }
            let ink = if on { visuals.selection.stroke.color } else { visuals.text_color() };
            paint(ui.painter(), rect, i, ink);
            if r.double_clicked() {
                activated = Some(key.to_string());
            } else if r.clicked() && mode != Selection::None {
                let m = ui.input(|inp| inp.modifiers);
                let k = key.to_string();
                match mode {
                    Selection::Multi if m.command => {
                        if on {
                            selected.retain(|s| *s != k);
                        } else {
                            selected.push(k);
                        }
                        anchor = Some(i);
                    }
                    Selection::Multi if m.shift && anchor.is_some() => {
                        let a = anchor.unwrap_or(i);
                        let (lo, hi) = (a.min(i), a.max(i));
                        selected = keys.get(lo..=hi).unwrap_or_default().iter().map(|s| s.to_string()).collect();
                    }
                    _ => {
                        selected = vec![k];
                        anchor = Some(i);
                    }
                }
            }
        }
    });
    let whole = top.union(out.inner_rect);
    cx.host.widget(cx.surface, id, whole, label);
    if selected != before {
        cx.emit(id, UiValue::Keys(selected.clone()));
    }
    if let Some(k) = activated {
        cx.emit(id, UiValue::Activate { activate: k });
    }
    ui.ctx().data_mut(|d| d.insert_temp(sid, (selected, source.to_vec(), anchor)));
}

fn icon_widget(ui: &mut Ui, name: &str, size: f32, tint: Option<Color32>) -> Response {
    let (rect, r) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let ink = tint.unwrap_or_else(|| ui.visuals().text_color());
    paint_icon(ui.painter(), rect, name, ink);
    r
}

/// A button with an icon, and a label unless it is icon-only.
fn icon_button(ui: &mut Ui, icon: &str, label: Option<&str>, enabled: bool) -> Response {
    let h = ui.spacing().interact_size.y;
    let pad = ui.spacing().button_padding;
    let font = egui::TextStyle::Button.resolve(ui.style());
    let galley = label.map(|l| ui.painter().layout_no_wrap(l.to_string(), font, Color32::WHITE));
    let icon_side = (h - 2.0 * pad.y).max(8.0);
    let width = match &galley {
        Some(g) => pad.x * 2.0 + icon_side + 4.0 + g.size().x,
        None => h,
    };
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, r) = ui.allocate_exact_size(vec2(width, h), sense);
    let style = if enabled { ui.style().interact(&r) } else { &ui.visuals().widgets.noninteractive };
    let ink = if enabled { style.text_color() } else { ui.visuals().weak_text_color() };
    ui.painter().rect(rect, style.corner_radius, style.weak_bg_fill, style.bg_stroke, egui::StrokeKind::Inside);
    let icon_rect = match &galley {
        Some(_) => Rect::from_min_size(pos2(rect.left() + pad.x, rect.center().y - icon_side / 2.0), Vec2::splat(icon_side)),
        None => Rect::from_center_size(rect.center(), Vec2::splat(icon_side)),
    };
    paint_icon(ui.painter(), icon_rect, icon, ink);
    if let Some(g) = galley {
        let at = pos2(icon_rect.right() + 4.0, rect.center().y - g.size().y / 2.0);
        ui.painter().galley_with_override_text_color(at, g, ink);
    }
    r
}

/// A switch drawn as a pill with a knob, then its label.
fn toggle(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    ui.horizontal(|ui| {
        let h = ui.spacing().interact_size.y * 0.8;
        let (rect, mut r) = ui.allocate_exact_size(vec2(h * 1.8, h), Sense::click());
        if r.clicked() {
            *on = !*on;
            r.mark_changed();
        }
        let t = ui.ctx().animate_bool_responsive(r.id, *on);
        let v = ui.style().interact_selectable(&r, *on);
        let radius = rect.height() / 2.0;
        ui.painter().rect(rect, radius, v.bg_fill, v.bg_stroke, egui::StrokeKind::Inside);
        let x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), t);
        ui.painter().circle(pos2(x, rect.center().y), radius * 0.75, v.fg_stroke.color, v.fg_stroke);
        let l = ui.label(label);
        let mut r = r.union(l.clone());
        if l.clicked() {
            *on = !*on;
            r.mark_changed();
        }
        r
    })
    .inner
}

/// Engine-neutral key names: the arrows are `ArrowLeft`/`ArrowRight`/`ArrowUp`/`ArrowDown`; every
/// other key uses its plain name (`A`, `7`, `Enter`, `Tab`, `Space`, `Backspace`, `Delete`, `Home`,
/// `End`, `PageUp`, `PageDown`, `F1`…).
fn key_name(key: egui::Key) -> &'static str {
    match key {
        egui::Key::ArrowLeft => "ArrowLeft",
        egui::Key::ArrowRight => "ArrowRight",
        egui::Key::ArrowUp => "ArrowUp",
        egui::Key::ArrowDown => "ArrowDown",
        other => other.name(),
    }
}

fn mods(m: egui::Modifiers) -> Mods {
    Mods { shift: m.shift, alt: m.alt, ctrl: m.ctrl, cmd: m.command }
}

fn cursor(c: Cursor) -> egui::CursorIcon {
    use egui::CursorIcon as C;
    match c {
        Cursor::Default => C::Default,
        Cursor::Pointer => C::PointingHand,
        Cursor::Crosshair => C::Crosshair,
        Cursor::Text => C::Text,
        Cursor::Move => C::Move,
        Cursor::Grab => C::Grab,
        Cursor::Grabbing => C::Grabbing,
        Cursor::NotAllowed => C::NotAllowed,
        Cursor::ResizeHorizontal => C::ResizeHorizontal,
        Cursor::ResizeVertical => C::ResizeVertical,
        Cursor::ResizeNwse => C::ResizeNwSe,
        Cursor::ResizeNesw => C::ResizeNeSw,
        Cursor::ZoomIn => C::ZoomIn,
        Cursor::ZoomOut => C::ZoomOut,
    }
}

/// Canvas pointer state between frames: (button held, last position sent, last hover position).
type CanvasState = (Option<PointerButton>, Option<Pos2>, Option<Pos2>);

/// A canvas: paints the draw list, then turns pointer, wheel and key input into events in canvas
/// coordinates. At most one drag or move per frame; positions that did not change send nothing.
fn canvas(ui: &mut Ui, cx: &mut Cx<'_>, c: &Canvas) {
    let size = vec2(c.width.clamp(1.0, 8192.0), c.height.clamp(1.0, 8192.0));
    let (r, painter) = ui.allocate_painter(size, Sense::click_and_drag());
    let origin = r.rect.min;
    let named = &cx.textures.named;
    paint_ops(&painter, Xf::new(origin, 1.0), r.rect, &c.ops, &|name| named.get(name).map(TextureHandle::id));
    cx.widget(&c.id, &r, &c.label);
    let r = with_hint(r, c.hint.as_deref().or(Some(&c.label)));
    let listens = |l: Listen| c.listen.contains(&l);
    let sid = cx.id("canvas", &c.id);
    let (mut held, mut last, mut hover): CanvasState = ui.ctx().data_mut(|d| d.get_temp::<CanvasState>(sid)).unwrap_or_default();
    let local = |p: Pos2| [p.x - origin.x, p.y - origin.y];
    let (m, pointer, events) = ui.input(|i| (mods(i.modifiers), i.pointer.clone(), i.events.clone()));
    // The wheel as the device reported it this frame (egui's own delta is smoothed over frames).
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let scroll = events.iter().fold(Vec2::ZERO, |acc, e| match e {
        egui::Event::MouseWheel { unit, delta, .. } => {
            acc + match unit {
                egui::MouseWheelUnit::Point => *delta,
                egui::MouseWheelUnit::Line => *delta * line,
                egui::MouseWheelUnit::Page => *delta * size.y,
            }
        }
        _ => acc,
    });
    let ev = |kind: PointerKind, at: Pos2| PointerEvent { kind, at: local(at), button: None, mods: m, delta: None, key: None };
    let mut out: Vec<PointerEvent> = Vec::new();
    let buttons = [
        (egui::PointerButton::Primary, PointerButton::Primary),
        (egui::PointerButton::Secondary, PointerButton::Secondary),
        (egui::PointerButton::Middle, PointerButton::Middle),
    ];
    let pos = pointer.interact_pos().or(pointer.latest_pos());
    // Press on the canvas: down (and keyboard focus when keys are wanted).
    if held.is_none()
        && r.hovered()
        && let Some(p) = pos
        && let Some((_, b)) = buttons.iter().find(|(eb, _)| pointer.button_pressed(*eb))
    {
        held = Some(*b);
        last = Some(p);
        out.push(PointerEvent { button: Some(*b), ..ev(PointerKind::Down, p) });
        if listens(Listen::Keys) {
            r.request_focus();
        }
    }
    if let Some(b) = held {
        let eb = buttons.iter().find(|(_, x)| *x == b).map_or(egui::PointerButton::Primary, |(e, _)| *e);
        if let Some(p) = pos
            && last != Some(p)
            && pointer.button_down(eb)
        {
            let delta = last.map(|l| [p.x - l.x, p.y - l.y]);
            out.push(PointerEvent { button: Some(b), delta, ..ev(PointerKind::Drag, p) });
            last = Some(p);
        }
        if pointer.button_released(eb) || !pointer.button_down(eb) {
            let p = pos.or(last).unwrap_or(origin);
            out.push(PointerEvent { button: Some(b), ..ev(PointerKind::Up, p) });
            held = None;
            last = None;
        }
    }
    if listens(Listen::DoubleClick)
        && r.double_clicked()
        && let Some(p) = pos
    {
        out.push(PointerEvent { button: Some(PointerButton::Primary), ..ev(PointerKind::DoubleClick, p) });
    }
    if listens(Listen::Move) {
        // While a button is held the pointer reports drags; its position still counts as seen,
        // so releasing does not send a move to where it already is.
        let now = if r.hovered() || held.is_some() { pointer.hover_pos() } else { None };
        match (now, hover) {
            (Some(p), prev) if prev != Some(p) && held.is_none() => out.push(ev(PointerKind::Move, p)),
            (None, Some(p)) if held.is_none() => out.push(ev(PointerKind::Leave, p)),
            _ => {}
        }
        hover = now;
    }
    if listens(Listen::Wheel)
        && r.hovered()
        && scroll != Vec2::ZERO
        && let Some(p) = pointer.hover_pos()
    {
        out.push(PointerEvent { delta: Some([scroll.x, scroll.y]), ..ev(PointerKind::Wheel, p) });
        // The canvas took the wheel: an enclosing scroll area must not scroll too.
        ui.ctx().input_mut(|i| i.smooth_scroll_delta = Vec2::ZERO);
    }
    if listens(Listen::Keys) && r.has_focus() {
        ui.memory_mut(|mem| {
            mem.set_focus_lock_filter(r.id, egui::EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, escape: false })
        });
        let at = pos.unwrap_or(origin);
        for e in events {
            if let egui::Event::Key { key, pressed, modifiers, .. } = e {
                if key == egui::Key::Escape {
                    r.surrender_focus();
                    continue;
                }
                let kind = if pressed { PointerKind::KeyDown } else { PointerKind::KeyUp };
                out.push(PointerEvent {
                    kind,
                    at: local(at),
                    button: None,
                    mods: mods(modifiers),
                    delta: None,
                    key: Some(key_name(key).to_string()),
                });
            }
        }
        ui.painter().rect_stroke(r.rect, 0.0, ui.visuals().selection.stroke, egui::StrokeKind::Outside);
    }
    if let Some(cur) = c.cursor
        && (r.hovered() || held.is_some())
    {
        ui.ctx().set_cursor_icon(cursor(cur));
    }
    ui.ctx().data_mut(|d| d.insert_temp::<CanvasState>(sid, (held, last, hover)));
    for e in out {
        cx.emit(&c.id, UiValue::Pointer(e));
    }
}

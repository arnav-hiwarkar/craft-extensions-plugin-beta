use super::look::{Button, button, initial, tone_color, tone_fill};
use craft_ext_core::{id::ExtensionId, listing::Image};
use craft_ext_manager::{Action, Loadable, Look, Model, Tone};
use egui::{Align2, CornerRadius, FontId, Frame, Margin, Response, RichText, Sense, Stroke, Ui, vec2};

pub(crate) const BUTTON_HEIGHT: f32 = 26.0;
pub(crate) const BUTTON_MIN_WIDTH: f32 = 84.0;

pub(crate) fn weak(ui: &Ui, text: impl Into<String>) -> RichText {
    RichText::new(text).color(ui.visuals().weak_text_color())
}

pub(crate) fn toned(ui: &Ui, text: impl Into<String>, tone: Tone) -> RichText {
    RichText::new(text).color(tone_color(tone, ui.visuals()))
}

pub(crate) fn section(ui: &mut Ui, text: impl Into<String>) {
    ui.add_space(14.0);
    ui.label(RichText::new(text).strong());
    ui.add_space(4.0);
}

pub(crate) fn badge(ui: &mut Ui, text: impl Into<String>, tone: Tone) -> Response {
    let visuals = ui.visuals().clone();
    Frame::new()
        .fill(tone_fill(tone, &visuals))
        .stroke(Stroke::new(1.0, tone_color(tone, &visuals).gamma_multiply(0.55)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(6, 1))
        .show(ui, |ui| ui.label(RichText::new(text).small().color(visuals.text_color())))
        .response
}

pub(crate) fn callout(ui: &mut Ui, tone: Tone, text: impl Into<String>, detail: Option<&str>) {
    let visuals = ui.visuals().clone();
    Frame::new()
        .fill(tone_fill(tone, &visuals))
        .stroke(Stroke::new(1.0, tone_color(tone, &visuals)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(text.into());
            if let Some(detail) = detail.filter(|d| !d.is_empty()) {
                ui.label(weak(ui, detail).small());
            }
        });
}

pub(crate) fn action_button(ui: &mut Ui, text: String, look: Look) -> Response {
    let size = vec2(BUTTON_MIN_WIDTH, BUTTON_HEIGHT);
    match button(look, ui.visuals()) {
        Button::Filled { fill, text: ink } => ui.add(egui::Button::new(RichText::new(text).strong().color(ink)).fill(fill).min_size(size)),
        Button::Plain => ui.add(egui::Button::new(text).min_size(size)),
        Button::Spinner => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(weak(ui, text));
            })
            .response
        }
    }
}

pub(crate) fn images_supported(ui: &Ui) -> bool {
    !ui.ctx().loaders().image.lock().is_empty()
}

pub(crate) fn picture(ui: &Ui, model: &Model, id: &ExtensionId, image: &Image, actions: &mut Vec<Action>) -> Option<egui::Image<'static>> {
    if !images_supported(ui) {
        return None;
    }
    match model.image(&image.url) {
        None => {
            actions.push(Action::LoadImage(id.clone(), image.clone()));
            None
        }
        Some(Loadable::Ready(bytes)) => Some(egui::Image::from_bytes(format!("bytes://craft-plugin/{}", image.url), egui::load::Bytes::Shared(bytes.clone()))),
        Some(_) => None,
    }
}

pub(crate) fn icon(ui: &mut Ui, model: &Model, id: &ExtensionId, name: &str, image: Option<&Image>, size: f32, actions: &mut Vec<Action>) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let radius = CornerRadius::same((size * 0.22) as u8);
    match image.and_then(|i| picture(ui, model, id, i, actions)) {
        Some(picture) => picture.corner_radius(radius).paint_at(ui, rect),
        None => {
            let visuals = ui.visuals();
            ui.painter().rect_filled(rect, radius, tone_fill(Tone::Accent, visuals));
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, initial(name), FontId::proportional(size * 0.45), visuals.strong_text_color());
        }
    }
}

pub(crate) fn nav_item(ui: &mut Ui, text: String, count: usize, tone: Tone, selected: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
    let visuals = ui.visuals().clone();
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, CornerRadius::same(6), visuals.selection.bg_fill.gamma_multiply(0.35));
    } else if response.hovered() {
        painter.rect_filled(rect, CornerRadius::same(6), visuals.widgets.hovered.weak_bg_fill);
    }
    let ink = if selected { visuals.strong_text_color() } else { visuals.text_color() };
    painter.text(rect.left_center() + vec2(10.0, 0.0), Align2::LEFT_CENTER, &text, FontId::proportional(14.0), ink);
    if count > 0 {
        let pill = egui::Rect::from_center_size(rect.right_center() - vec2(22.0, 0.0), vec2(26.0, 18.0));
        painter.rect_filled(pill, CornerRadius::same(9), tone_fill(tone, &visuals));
        painter.text(pill.center(), Align2::CENTER_CENTER, count.to_string(), FontId::proportional(11.0), visuals.strong_text_color());
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, selected, &text));
    response
}

pub(crate) fn skeleton(ui: &mut Ui, rows: usize) {
    let time = ui.input(|i| i.time);
    let visuals = ui.visuals().clone();
    for i in 0..rows {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 64.0), Sense::hover());
        let pulse = ((time * 2.0 + i as f64 * 0.5).sin() * 0.5 + 0.5) as f32;
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(8), visuals.faint_bg_color);
        let bar = visuals.widgets.inactive.weak_bg_fill.gamma_multiply(0.5 + 0.5 * pulse);
        let square = egui::Rect::from_min_size(rect.min + vec2(10.0, 12.0), vec2(40.0, 40.0));
        painter.rect_filled(square, CornerRadius::same(9), bar);
        let line = |y: f32, w: f32| egui::Rect::from_min_size(egui::pos2(square.right() + 12.0, rect.top() + y), vec2(w, 10.0));
        painter.rect_filled(line(16.0, (rect.width() * 0.35).min(220.0)), CornerRadius::same(4), bar);
        painter.rect_filled(line(36.0, (rect.width() * 0.55).min(360.0)), CornerRadius::same(4), bar);
        ui.add_space(6.0);
    }
    ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));
}

pub(crate) fn empty(ui: &mut Ui, text: String) {
    ui.add_space(56.0);
    ui.vertical_centered(|ui| ui.label(weak(ui, text)));
}

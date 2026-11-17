mod detail;
pub mod look;
mod page;
mod prompt;
mod setup;
#[cfg(test)]
mod tests;
mod widgets;

pub use look::Words;
pub use page::PluginManager;
pub use prompt::{ProjectPrompt, PromptOutcome};

use craft_ext_manager::{Action, Config, Effect, Model, Notice};
use egui::{Align2, Area, Context, CornerRadius, Frame, Id, Order, Stroke, vec2};
use look::{Toasts, tone_color};
use std::time::Duration;

pub(crate) const BUSY_REPAINT: f64 = 0.1;
pub(crate) const IDLE_REPAINT: f64 = 1.0;
pub(crate) const TOAST_WIDTH: f32 = 340.0;

pub(crate) struct Session {
    pub(crate) model: Model,
    toasts: Toasts,
}

impl Session {
    pub(crate) fn new(config: Config) -> Self {
        Session { model: Model::new(config), toasts: Toasts::default() }
    }

    pub(crate) fn poll(&mut self, ctx: &Context, words: &dyn Words) -> Vec<Notice> {
        let effects = self.model.poll();
        self.handle(effects, ctx, words)
    }

    pub(crate) fn apply(&mut self, actions: Vec<Action>, ctx: &Context, words: &dyn Words) -> Vec<Notice> {
        let mut notices = vec![];
        for action in actions {
            let effects = self.model.apply(action);
            notices.extend(self.handle(effects, ctx, words));
        }
        notices
    }

    fn handle(&mut self, effects: Vec<Effect>, ctx: &Context, words: &dyn Words) -> Vec<Notice> {
        let now = ctx.input(|i| i.time);
        let mut notices = vec![];
        for effect in effects {
            match effect {
                Effect::OpenUrl(url) => ctx.open_url(egui::OpenUrl::new_tab(url)),
                Effect::Notice(notice) => {
                    self.toasts.push(&notice, words, now);
                    notices.push(notice);
                }
                Effect::Changed | Effect::OpenSetup(_) | Effect::CloseSetup(_) => ctx.request_repaint(),
            }
        }
        notices
    }

    pub(crate) fn overlays(&mut self, ctx: &Context, words: &dyn Words) -> Vec<Notice> {
        let actions = self.model.pending_setup().map(|s| setup::sheet(ctx, s, words)).unwrap_or_default();
        let notices = self.apply(actions, ctx, words);
        let now = ctx.input(|i| i.time);
        self.draw_toasts(ctx, now);
        let wait = if self.model.busy() { BUSY_REPAINT } else { IDLE_REPAINT };
        let wait = self.toasts.next_expiry(now).map_or(wait, |left| wait.min(left.max(0.0)));
        ctx.request_repaint_after(Duration::from_secs_f64(wait));
        notices
    }

    fn draw_toasts(&mut self, ctx: &Context, now: f64) {
        let toasts = self.toasts.live(now);
        if toasts.is_empty() {
            return;
        }
        Area::new(Id::unique("craft-plugin-toasts")).order(Order::Foreground).anchor(Align2::RIGHT_BOTTOM, vec2(-16.0, -16.0)).interactable(false).show(ctx, |ui| {
            for toast in toasts {
                let color = tone_color(toast.tone, ui.visuals());
                Frame::popup(ui.style()).stroke(Stroke::new(1.0, color)).corner_radius(CornerRadius::same(8)).show(ui, |ui| {
                    ui.set_max_width(TOAST_WIDTH);
                    ui.horizontal(|ui| {
                        let (dot, _) = ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                        ui.painter().circle_filled(dot.center(), 4.0, color);
                        ui.label(&toast.text);
                    });
                });
                ui.add_space(6.0);
            }
        });
    }
}

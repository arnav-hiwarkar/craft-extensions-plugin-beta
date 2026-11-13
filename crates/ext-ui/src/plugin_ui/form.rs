//! Form schema 1 in egui: every `ParamKind`, `visible_when`/`enabled_when`, the plugin's hidden and
//! disabled sets (patches), units and help. Values stay typed and in their declared spaces: colours
//! are edited as numbers so HDR working-space values are never clamped, and choice ids (never their
//! labels) are sent.

use super::Host;
use craft_ext_core::form::{Form, FormState, ParamKind as K, Patch, PointSpace, Unit, Value as V, Values};
use egui::{Id, Response, Ui};

/// How a form reports edits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FormOptions {
    /// Report every change (a dialog with a live preview); otherwise report when an edit ends
    /// (drag released, focus lost, Enter, a discrete pick), as panels do.
    pub live: bool,
    /// Draw the form's title above its rows.
    pub title: bool,
}

/// One parameter's in-progress edit: (edited value, plugin value when the edit began, editing, rev).
type Edit = (V, V, bool, u64);

/// Draws `form` over `state` and returns the edits as (parameter key, value). `state` is the
/// plugin's view; local edits are held per parameter until they end or the plugin's value changes.
#[allow(clippy::too_many_arguments)]
pub fn form_rows(
    ui: &mut Ui,
    salt: Id,
    surface: &str,
    rev: u64,
    form: &Form,
    state: &FormState,
    opts: FormOptions,
    host: &mut dyn Host,
) -> Vec<(String, V)> {
    let mut out = Vec::new();
    if form.validate_values(&state.values).is_err() {
        ui.colored_label(ui.visuals().warn_fg_color, host.tr("This form has invalid values."));
        return out;
    }
    let mut effective = form.defaults();
    effective.extend(state.values.clone());
    if opts.title && !form.title.english.trim().is_empty() {
        ui.label(egui::RichText::new(&form.title.english).strong());
    }
    let mut first = true;
    for p in &form.params {
        if state.hidden.contains(&p.key) || p.visible_when.as_ref().is_some_and(|c| !c.eval(&effective)) {
            continue;
        }
        let enabled = !state.disabled.contains(&p.key) && p.enabled_when.as_ref().is_none_or(|c| c.eval(&effective));
        let help = p.help.as_ref().map(|h| h.english.clone());
        if matches!(p.kind, K::Section) {
            if !first {
                ui.separator();
            }
            let r = ui.add_enabled(enabled, egui::Label::new(egui::RichText::new(&p.label.english).strong()));
            if let Some(h) = &help {
                r.on_hover_text(h);
            }
            first = false;
            continue;
        }
        first = false;
        let Some(source) = effective.get(&p.key).cloned() else { continue };
        let id = salt.with(("form-edit", &form.key, &p.key));
        let (mut value, mut began_from, was_active, old_rev): Edit =
            ui.ctx().data_mut(|d| d.get_temp::<Edit>(id)).unwrap_or((source.clone(), source.clone(), false, rev));
        // The plugin changed the value (or sent a new tree) while nobody was editing: take it.
        if !was_active && (began_from != source || old_rev != rev) {
            value = source.clone();
            began_from = source.clone();
        }
        let before = value.clone();
        let mut active = false;
        let mut ended = false;
        let mut discrete = false;
        let mut rect = None;
        ui.push_id(id, |ui| {
            ui.add_enabled_ui(enabled, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let label = ui.label(&p.label.english);
                    if let Some(h) = &help {
                        label.on_hover_text(h);
                    }
                    let mut track = |r: Response, active: &mut bool, ended: &mut bool| {
                        *active |= r.dragged() || r.has_focus() || r.is_pointer_button_down_on();
                        *ended |= r.drag_stopped() || r.lost_focus();
                        let r = match &help {
                            Some(h) => r.on_hover_text(h),
                            None => r,
                        };
                        rect = Some(rect.map_or(r.rect, |q: egui::Rect| q.union(r.rect)));
                    };
                    match (&p.kind, &mut value) {
                        (K::Range { min, max, step, .. }, V::Number(v)) => {
                            let mut s = egui::Slider::new(v, *min..=*max);
                            if let Some(step) = step {
                                s = s.step_by(*step);
                            }
                            track(ui.add(s), &mut active, &mut ended);
                        }
                        (K::Int { min, max, .. }, V::Number(v)) => {
                            let (lo, hi) = (*min as f64, *max as f64);
                            let r = if hi - lo <= 1000.0 {
                                ui.add(egui::Slider::new(v, lo..=hi).integer())
                            } else {
                                ui.add(egui::DragValue::new(v).range(lo..=hi).speed(1.0).max_decimals(0))
                            };
                            *v = v.round();
                            track(r, &mut active, &mut ended);
                        }
                        (K::Number { min, max, .. }, V::Number(v)) => {
                            let r = ui.add(egui::DragValue::new(v).range(min.unwrap_or(f64::MIN)..=max.unwrap_or(f64::MAX)));
                            track(r, &mut active, &mut ended);
                        }
                        (K::Angle { .. }, V::Number(v)) => track(ui.add(egui::DragValue::new(v).suffix("°")), &mut active, &mut ended),
                        (K::Bool { .. }, V::Bool(v)) => {
                            track(ui.checkbox(v, ""), &mut active, &mut ended);
                            discrete = true;
                        }
                        (K::Choice { options, .. }, V::String(v)) => {
                            let shown = options.iter().find(|o| o.id == *v).map(|o| o.label.english.as_str()).unwrap_or("");
                            let r = egui::ComboBox::from_id_salt("choice")
                                .selected_text(shown)
                                .show_ui(ui, |ui| {
                                    for o in options {
                                        ui.selectable_value(v, o.id.clone(), &o.label.english);
                                    }
                                })
                                .response;
                            track(r, &mut active, &mut ended);
                            discrete = true;
                        }
                        (K::Radio { options, .. }, V::String(v)) => {
                            for o in options {
                                let r = ui.radio_value(v, o.id.clone(), &o.label.english);
                                track(r, &mut active, &mut ended);
                            }
                            discrete = true;
                        }
                        (K::Color { alpha, .. }, V::Color(c)) => {
                            // A swatch, a picker when the colour is displayable, and numeric
                            // channels that keep HDR values.
                            if c.iter().all(|x| (0.0..=1.0).contains(x)) {
                                let mut pick = *c;
                                let r = ui.color_edit_button_rgba_unmultiplied(&mut pick);
                                if r.changed() {
                                    *c = if *alpha { pick } else { [pick[0], pick[1], pick[2], 1.0] };
                                    discrete = true;
                                }
                                track(r, &mut active, &mut ended);
                            } else {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(ui.spacing().interact_size.y * 1.5, ui.spacing().interact_size.y),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 2.0, super::paint::color(*c));
                            }
                            for ch in c.iter_mut().take(if *alpha { 4 } else { 3 }) {
                                track(ui.add(egui::DragValue::new(ch).speed(0.01)), &mut active, &mut ended);
                            }
                            c[3] = if *alpha { c[3].clamp(0.0, 1.0) } else { 1.0 };
                        }
                        (K::Point { space, .. }, V::Point(pt)) => {
                            for ch in pt.iter_mut() {
                                track(ui.add(egui::DragValue::new(ch)), &mut active, &mut ended);
                            }
                            ui.weak(match space {
                                PointSpace::LayerPixels => host.tr("layer px"),
                                PointSpace::ImagePercent => host.tr("image %"),
                            });
                        }
                        (K::Text { multiline, .. }, V::String(t)) => {
                            let r = if *multiline { ui.text_edit_multiline(t) } else { ui.text_edit_singleline(t) };
                            let enter = !*multiline && r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            track(r, &mut active, &mut ended);
                            ended |= enter;
                        }
                        _ => {
                            ui.weak(host.tr("(value does not match this parameter)"));
                        }
                    }
                    if let Some(u) = &p.unit {
                        ui.weak(unit_label(u));
                    }
                });
            });
        });
        if let Some(r) = rect {
            host.widget(surface, &p.key, r, &p.label.english);
        }
        let changed = value != before;
        let submit = if opts.live { changed } else { ended || discrete && changed || changed && !active || was_active && !active };
        // Only valid values leave the form (a typed number can briefly be out of range).
        let candidate: Values = [(p.key.clone(), value.clone())].into_iter().collect();
        if enabled && submit && value != began_from && form.validate_values(&candidate).is_ok() {
            out.push((p.key.clone(), value.clone()));
            effective.insert(p.key.clone(), value.clone());
            began_from = value.clone();
        }
        let keep_from = if active { began_from } else { source };
        ui.ctx().data_mut(|d| d.insert_temp::<Edit>(id, (value, keep_from, active, rev)));
    }
    out
}

pub fn unit_label(unit: &Unit) -> &str {
    match unit {
        Unit::Pixels => "px",
        Unit::Percent => "%",
        Unit::Degrees => "°",
        Unit::Seconds => "s",
        Unit::Frames => "frames",
        Unit::Custom(s) => s,
    }
}

/// A form that owns its state: for dialogs (a filter's controls) that apply the plugin's
/// `craft_update` patches and report every change for a live preview.
#[derive(Clone, Debug)]
pub struct FormView {
    form: Form,
    state: FormState,
    salt: Id,
    options: FormOptions,
}

impl FormView {
    /// Starts from the defaults overlaid with `values`; invalid values are dropped.
    pub fn new(salt: impl std::hash::Hash + std::fmt::Debug, form: Form, values: &Values) -> Self {
        let mut state = FormState { values: form.defaults(), ..Default::default() };
        for (k, v) in values {
            let one: Values = [(k.clone(), v.clone())].into_iter().collect();
            if form.validate_values(&one).is_ok() {
                state.values.insert(k.clone(), v.clone());
            }
        }
        Self { form, state, salt: Id::unique(("craft-plugin-form-view", salt)), options: FormOptions { live: true, title: false } }
    }

    pub fn with_options(mut self, options: FormOptions) -> Self {
        self.options = options;
        self
    }

    pub fn form(&self) -> &Form {
        &self.form
    }

    pub fn state(&self) -> &FormState {
        &self.state
    }

    pub fn values(&self) -> &Values {
        &self.state.values
    }

    /// Applies a plugin patch atomically (stale or invalid patches change nothing).
    pub fn apply_patch(&mut self, patch: &Patch) -> Result<(), String> {
        self.form.apply_patch(&mut self.state, patch)
    }

    /// Swaps in a re-described form, keeping every current value the new form still accepts.
    pub fn replace_form(&mut self, form: Form) {
        let old = std::mem::take(&mut self.state.values);
        let mut values = form.defaults();
        for (k, v) in old {
            let one: Values = [(k.clone(), v.clone())].into_iter().collect();
            if form.validate_values(&one).is_ok() {
                values.insert(k, v);
            }
        }
        self.state.values = values;
        self.state.hidden.retain(|k| form.params.iter().any(|p| p.key == *k));
        self.state.disabled.retain(|k| form.params.iter().any(|p| p.key == *k));
        self.form = form;
    }

    /// Draws the rows; returns the edits, already applied to [`FormView::values`].
    pub fn show(&mut self, ui: &mut Ui, host: &mut dyn Host) -> Vec<(String, V)> {
        let edits = form_rows(ui, self.salt, &self.form.key.clone(), self.state.rev, &self.form, &self.state, self.options, host);
        for (k, v) in &edits {
            self.state.values.insert(k.clone(), v.clone());
        }
        edits
    }
}

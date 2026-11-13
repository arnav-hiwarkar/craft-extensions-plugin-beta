//! The one renderer for plugin panels and windows, shared by every Craft app.
//!
//! A *surface* is a plugin's UI tree ([`craft_ext_core::ui::tree::Node`]) with its revision. An
//! [`Engine`] draws a surface into an egui `Ui` and returns the [`UiEvent`]s the person caused,
//! tagged with the revision drawn, so the app's runtime can drop stale ones.
//!
//! The **app** chooses the engine ([`EngineChoice`]); a plugin can never choose or detect it: the
//! tree and the events are the same whatever draws them. Today [`EngineChoice::Egui`] draws
//! everything; [`EngineChoice::Framework`] is reserved for the craft-framework renderer and falls
//! back to egui until the host installs an implementation with [`PluginUi::set_framework_engine`].
//!
//! What stays in the app sits behind [`Host`]: automation/accessibility registration of widgets,
//! native framework components, translated host words, and where windows dock (the app styles the
//! `egui::Window` in [`PluginUi::window`]). Routing events to the plugin is the app's job too.

pub mod egui_engine;
pub mod form;
pub mod icons;
pub mod paint;

#[cfg(test)]
mod tests;

pub use craft_ext_core::ui::tree::{Node, UiEvent, UiValue};
pub use egui_engine::EguiEngine;
pub use form::{FormOptions, FormView, form_rows};
pub use paint::{Xf, paint_ops};

use craft_ext_core::ui::tree::Component;
use std::collections::HashMap;
use std::sync::Arc;

/// Which engine draws plugin UI. Chosen by the app, never by a plugin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EngineChoice {
    /// The built-in egui renderer: draws every node, form and draw op.
    #[default]
    Egui,
    /// The host-installed craft-framework renderer; falls back to egui when absent.
    Framework,
}

impl EngineChoice {
    /// Default resolution without a host-installed renderer. Use [`PluginUi::engine`] for a host instance.
    pub fn resolved(self) -> EngineChoice {
        match self {
            EngineChoice::Egui => EngineChoice::Egui,
            // The `framework` feature is the switch point for the craft-framework engine; with or
            // without it, nothing but egui can draw yet.
            EngineChoice::Framework => EngineChoice::Egui,
        }
    }
}

/// One surface to draw: the plugin's panel or window key, its tree and the tree's revision.
#[derive(Clone, Copy, Debug)]
pub struct Surface<'a> {
    pub key: &'a str,
    pub rev: u64,
    pub tree: &'a Node,
}

/// Draws surfaces. Implemented by [`EguiEngine`]; a framework engine implements the same trait.
pub trait Engine {
    /// Draws `surface` and returns the events of this frame, each tagged with `surface.rev`.
    fn draw(&mut self, ui: &mut egui::Ui, surface: Surface<'_>, host: &mut dyn Host) -> Vec<UiEvent>;
    /// Releases what the engine keeps for a surface (textures, edit state) after it closes.
    fn forget(&mut self, key: &str);
}

/// What the app supplies to the renderer. Every method has a do-nothing default.
pub trait Host {
    /// Called for every interactive widget drawn, with its id (or form parameter key), screen
    /// rectangle and label: automation ids, accessibility, screenshot tooling.
    fn widget(&mut self, _surface: &str, _id: &str, _rect: egui::Rect, _label: &str) {}
    /// Draws framework component `c` natively and returns its events as (widget, value), or
    /// `None` to draw its fallback (always, until the framework lands).
    fn component(&mut self, _ui: &mut egui::Ui, _surface: &str, _c: &Component) -> Option<Vec<(String, UiValue)>> {
        None
    }
    /// Translates host-owned words ("This panel needs a newer version of the app."). Plugin
    /// text is never translated by the host.
    fn tr(&self, english: &str) -> String {
        english.to_string()
    }
    /// Draws a form node's title above its rows.
    fn form_titles(&self) -> bool {
        false
    }
}

/// A host with every default: no automation, no components, English words.
pub struct NoHost;
impl Host for NoHost {}

/// What an app keeps for plugin UI: the chosen engine and parsed trees, one per open surface.
pub struct PluginUi {
    choice: EngineChoice,
    egui: EguiEngine,
    framework: Option<Box<dyn Engine>>,
    trees: HashMap<String, (u64, Arc<Result<Node, String>>)>,
}

impl Default for PluginUi {
    fn default() -> Self {
        Self::new(EngineChoice::default())
    }
}

impl PluginUi {
    pub fn new(choice: EngineChoice) -> Self {
        Self { choice, egui: EguiEngine::default(), framework: None, trees: HashMap::new() }
    }

    /// The engine the app asked for.
    pub fn choice(&self) -> EngineChoice {
        self.choice
    }

    /// The engine actually drawing.
    pub fn engine(&self) -> EngineChoice {
        if self.choice == EngineChoice::Framework && self.framework.is_some() {
            EngineChoice::Framework
        } else {
            self.choice.resolved()
        }
    }

    /// Installs or removes the host's framework renderer. Trees, revisions and guest events
    /// retain the same contract; the plugin receives no renderer identity or selection API.
    /// Call during host setup or between frames. Removing it restores the egui fallback.
    pub fn set_framework_engine(&mut self, engine: Option<Box<dyn Engine>>) {
        if let Some(previous) = self.framework.as_mut() {
            for key in self.trees.keys() {
                previous.forget(key);
            }
        }
        self.framework = engine;
    }

    /// Draws a parsed, validated tree.
    pub fn draw(&mut self, ui: &mut egui::Ui, surface: Surface<'_>, host: &mut dyn Host) -> Vec<UiEvent> {
        if self.choice == EngineChoice::Framework
            && let Some(engine) = self.framework.as_mut()
        {
            return engine.draw(ui, surface, host);
        }
        self.egui.draw(ui, surface, host)
    }

    /// Parses (once per revision) and draws a JSON tree as the app's runtime stores it. `None`
    /// draws "not drawn yet"; an invalid tree draws the reason instead of the panel.
    pub fn draw_json(&mut self, ui: &mut egui::Ui, key: &str, rev: u64, tree: Option<&serde_json::Value>, host: &mut dyn Host) -> Vec<UiEvent> {
        let Some(json) = tree else {
            ui.weak(host.tr("The plugin has not drawn this panel yet."));
            return Vec::new();
        };
        let parsed = match self.trees.get(key) {
            Some((r, t)) if *r == rev => t.clone(),
            _ => {
                let t = Arc::new(Node::parse(json).map_err(|e| e.0));
                self.trees.insert(key.to_string(), (rev, t.clone()));
                t
            }
        };
        match &*parsed {
            Ok(node) => self.draw(ui, Surface { key, rev, tree: node }, host),
            Err(e) => {
                ui.colored_label(ui.visuals().warn_fg_color, format!("{}: {e}", host.tr("This panel could not be drawn")));
                Vec::new()
            }
        }
    }

    /// The apps' floating presentation: one `egui::Window` per surface, titled by the plugin,
    /// closable. `style` lets the app place or dock it (default position, width, anchor).
    #[allow(clippy::too_many_arguments)]
    pub fn window(
        &mut self,
        ctx: &egui::Context,
        key: &str,
        title: &str,
        rev: u64,
        tree: Option<&serde_json::Value>,
        open: &mut bool,
        style: impl FnOnce(egui::Window<'_>) -> egui::Window<'_>,
        host: &mut dyn Host,
    ) -> Vec<UiEvent> {
        let mut events = Vec::new();
        let window = egui::Window::new(title).id(egui::Id::unique(("craft-plugin-surface", key))).open(open).resizable(true).default_width(280.0);
        style(window).show(ctx, |ui| events = self.draw_json(ui, key, rev, tree, host));
        if !*open {
            self.forget(key);
        }
        events
    }

    /// Drops the parsed tree and engine state of a closed surface.
    pub fn forget(&mut self, key: &str) {
        self.trees.remove(key);
        self.egui.forget(key);
        if let Some(engine) = self.framework.as_mut() {
            engine.forget(key);
        }
    }
}

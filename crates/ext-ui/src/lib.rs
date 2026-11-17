#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]
pub mod manager;
pub mod plugin_ui;

pub use craft_ext_manager;
pub use manager::{PluginManager, ProjectPrompt, PromptOutcome, Words};

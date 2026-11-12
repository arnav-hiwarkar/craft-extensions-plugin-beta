//! Craft plugin runtime shared by every app.
//!
//! - [`api`]: the [`api::AppApi`] trait each app adapter implements, the [`api::AppContract`] it
//!   publishes (rendered into author docs), and the single [`api::authorize`] check.
//! - [`wasm`]: the sandbox (wasmi, ABI v1): [`wasm::Filter`] for `raster.filter@1` transforms and
//!   [`wasm::Session`] for commands, panels, tools and event hooks.
//! - [`native`]: out-of-process native helpers speaking the same session protocol.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo)]

pub mod api;
pub mod native;
pub mod wasm;

#[cfg(test)]
mod tests;

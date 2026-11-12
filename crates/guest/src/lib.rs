//! Author SDK for Craft plugins, wasm side. Implements guest ABI v1 (beta-design B4).
//!
//! Two kinds of code plugin exist, and each has one entry macro:
//!
//! - **Effects** (`raster.filter@1`): a pixel transform. Use [`filter!`].
//! - **Panels and commands** (`effectcraft.session@1`): event-driven code that reads the document
//!   and draws UI. Use [`session!`].
//!
//! Data kinds (`lut`, `preset`, `font`, `theme`, `data`) run no code. Declare them in
//! `craft-plugin.toml` only.
//!
//! Build with `cargo build --target wasm32-unknown-unknown --release`. Parameters, permissions and
//! contribution ids live in `craft-plugin.toml`; the code must match them.
//!
//! # Quick start: an effect
//!
//! ```rust,no_run
//! use craft_guest::{filter, FilterCtx, Image};
//!
//! // craft-plugin.toml declares: kind = "effect", contract = "raster.filter@1.0",
//! // module = "exposure.wasm", and a slider parameter with id "stops".
//! fn exposure(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
//!     let gain = 2f32.powf(ctx.f("stops")? as f32);
//!     for px in img.pixels.chunks_exact_mut(4) {
//!         // Pixels are premultiplied RGBA f32. Scaling colour by a gain keeps alpha untouched.
//!         for channel in &mut px[..3] {
//!             *channel *= gain;
//!         }
//!     }
//!     Ok(())
//! }
//!
//! filter!(exposure);
//! # fn main() {}
//! ```
//!
//! Return `Err` for a bad parameter or input. The host logs the message and discards the result.
//!
//! # Quick start: a panel with a command
//!
//! ```rust,no_run
//! use craft_guest::{session, ui, CallError, Event};
//!
//! // craft-plugin.toml declares a panel with id "notes" and a command with id "add-marker"; both
//! // use contract = "effectcraft.session@1.0" and the same module.
//! fn handle(event: Event) -> Result<(), String> {
//!     match event {
//!         Event::Init | Event::Ui { .. } => {
//!             let tree = ui::column(vec![
//!                 ui::label("Notes"),
//!                 ui::button("add-marker", "Add note marker"),
//!             ]);
//!             ui::set_panel("notes", tree).map_err(|e| e.to_string())?;
//!         }
//!         Event::Command { id } if id == "add-marker" => match craft_guest::call("marker.add", craft_guest::json!({"label": "Note"})) {
//!             Ok(_) => {}
//!             // The person has not granted an optional permission: say so, do not fail the event.
//!             Err(CallError::PermissionNeeded(_)) => {}
//!             Err(e) => return Err(e.to_string()),
//!         },
//!         _ => {}
//!     }
//!     Ok(())
//! }
//!
//! session!(handle);
//! # fn main() {}
//! ```
//!
//! # Conventions
//!
//! - Effects must not depend on earlier calls: the host may create a new instance at any time.
//!   Keep per-call state in the call, and per-session state in the session's own heap.
//! - Everything an effect outputs must follow from its context and pixels. Use [`FilterCtx::seed`]
//!   for any randomness.
//! - Panic-free code is the rule. A panic traps the module, and the host counts it as a failure.

#![warn(missing_docs)]
#![deny(unsafe_code)]

#[allow(unsafe_code)]
mod abi;
mod entry;
mod filter;
mod host;
mod session;
pub mod ui;

pub use filter::{FilterCtx, Image};
pub use host::{CallError, LogLevel, call, log};
pub use serde_json;
pub use serde_json::{Value, json};
pub use session::Event;

/// Runtime glue for the macros. Not part of the author API.
#[doc(hidden)]
pub mod __private {
    pub use crate::entry::{ABI_VERSION, alloc, filter_entry, reset, session_entry};
}

/// Exports shared by every plugin kind: `craft_abi`, `craft_alloc` and `craft_reset`.
#[doc(hidden)]
#[macro_export]
macro_rules! __craft_exports {
    () => {
        #[unsafe(no_mangle)]
        pub extern "C" fn craft_abi() -> i32 {
            $crate::__private::ABI_VERSION
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn craft_alloc(len: i32) -> i32 {
            $crate::__private::alloc(len)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn craft_reset() {
            $crate::__private::reset()
        }
    };
}

/// Exports an effect: `craft_abi`, `craft_alloc`, `craft_reset` and `craft_filter`.
///
/// `$entry` is a function `fn(&FilterCtx, &mut Image) -> Result<(), String>`. Use this macro once,
/// at the crate root, in a crate that builds a single effect module.
#[macro_export]
macro_rules! filter {
    ($entry:path) => {
        $crate::__craft_exports!();

        #[unsafe(no_mangle)]
        pub extern "C" fn craft_filter(ctx: i32, ctx_len: i32, px: i32, w: i32, h: i32) -> i32 {
            $crate::__private::filter_entry(ctx, ctx_len, px, w, h, $entry)
        }
    };
}

/// Exports a session: `craft_abi`, `craft_alloc`, `craft_reset` and `craft_event`.
///
/// `$entry` is a function `fn(Event) -> Result<(), String>`. Each call is one transaction: `Err`
/// rolls back everything the event did. Use this macro once, at the crate root.
#[macro_export]
macro_rules! session {
    ($entry:path) => {
        $crate::__craft_exports!();

        #[unsafe(no_mangle)]
        pub extern "C" fn craft_event(ev: i32, ev_len: i32) -> i32 {
            $crate::__private::session_entry(ev, ev_len, $entry)
        }
    };
}

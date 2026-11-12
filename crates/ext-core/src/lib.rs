//! Package contract for Craft plugins: manifests, ids, versions, contracts, permissions, catalogue
//! records, project references and the declarative UI tree.
//!
//! Pure data and validation: no file system, no network, no clock. It builds for
//! `wasm32-unknown-unknown`. Everything that touches disk lives in `craft-ext-store`.
//!
//! Invariants:
//! - Every byte that reaches this crate is hostile: sizes are capped before parsing, ids and paths are
//!   checked by character class, and nothing here indexes with an untrusted number.
//! - A package never states its own trust. The tier is assigned by the host from where the package
//!   came, never read from the package.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod catalogue;
pub mod contract;
pub mod id;
pub mod licence;
pub mod manifest;
pub mod path;
pub mod permission;
pub mod project;
pub mod text;
pub mod tier;
pub mod ui;
pub mod version;

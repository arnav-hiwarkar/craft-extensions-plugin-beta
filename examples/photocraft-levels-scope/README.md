# Levels Scope

A panel with a histogram and controls, plus an adjustment-layer command. Requests document.read:document, document.edit:document and app.events. Uses PhotoCraft session 1.1 and the shared host renderer.

Original code is licensed MIT OR Apache-2.0; see the included licence files.

Source is preserved for continued implementation/review. Current package builds, execution and macOS behavior are **not verified**. Do not interpret a successful manifest check or ignored integration test as runtime proof. See PhotoCraft's `docs/plugin-takeover-validation.md` and `crates/engine/tests/plugin_examples.rs` for the intended acceptance cases.

For the later approved build phase, use the `craft-ext` author workflow documented in `docs/plugins/author-guide.md`, then install into a disposable store. App builds, guest release builds and broad integration runs are deferred at the user's request.

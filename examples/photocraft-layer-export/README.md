# Layer Export

A panel and command exporting layers into a folder explicitly picked by the user. Requests document.read:document and files.write:picked. Opaque handles hide paths. Cancels if the document changes while picking. Publication is ordered and atomic per file, not per batch; earlier files can remain after failure and are not document-undoable.

Original code is licensed MIT OR Apache-2.0; see the included licence files.

Source is preserved for continued implementation/review. Current package builds, execution and macOS behavior are **not verified**. Do not interpret a successful manifest check or ignored integration test as runtime proof. See PhotoCraft's `docs/plugin-takeover-validation.md` and `crates/engine/tests/plugin_examples.rs` for the intended acceptance cases.

For the later approved build phase, use the `craft-ext` author workflow documented in `docs/plugins/author-guide.md`, then install into a disposable store. App builds, guest release builds and broad integration runs are deferred at the user's request.

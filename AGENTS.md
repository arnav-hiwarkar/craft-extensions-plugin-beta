# Plugin platform contributor instructions

## Suite plugin replacement policy

When changing plugin implementations, app UI integration, lifecycle flows or merging
upstream plugin/UI changes in this private beta fork, apply the user-directed
[suite replacement rule](https://github.com/arnav-hiwarkar/craft-extensions-plugin-beta/blob/plugin-beta/docs/plugins/suite-replacement-rule.md). In the sibling beta workspace the same file is
`docs/plugins/suite-replacement-rule.md`. Inspect and learn from
each app's existing capabilities, migrate and verify the flows, then remove the
superseded implementations. Keep one shared platform and one canonical adapter per
app, coordinated with the current framework/UI work.

## Engineering rules

These come from building this platform; each is backed by a check where one exists.

- **No code comments.** Names, types, small functions and tests carry the meaning. `cargo xtask comments`
  fails when a file has more comment lines than `xtask/comment-baseline.json` allows (new files: zero);
  after removing comments run `cargo xtask comments --lower-baseline`. Rustdoc stays only in
  `crates/guest`, whose public API is the author documentation and denies missing docs.
- **Make the right call the only call.** Prefer one function that does the whole safe thing (for example
  `api::contain`, `JobBoard::admit`, `Store::quarantine`) over several steps every caller must order
  correctly. Encode limits as constants next to the code that enforces them, and refuse invalid input with
  a clear error rather than clamping it silently.
- **Host-handled operations keep the ABI stable.** New sandbox capabilities are `craft.call` operations the
  host answers itself (`kernel.*`, `job.*`), so ABI v1 modules keep loading and older apps answer `-3`.
- **Every expensive host action is budgeted by its real work.** Charge fuel for what the host actually
  does (pixels × taps, points × area), not a flat rate; cap any single call so cancel stays responsive;
  reserve host memory against an app-wide budget before copying plugin data; check the cancel flag
  inside long loops. Work beyond the current slice is carried as debt and counts against the total.
- **A failure while serving a plugin poisons its transaction.** `api::contain` turns an adapter panic into
  an error and marks the event, which then rolls back even if the plugin ignores the error.
- **One authorization path.** Wasm and native calls both go through `api::authorize_call`.
- **Hand results over once.** Lists show metadata; payloads move to their consumer and are dropped.
- **Serve forward-compatible data.** Structs read from the catalogue or other signed data accept unknown
  fields; strictness belongs in the authoring checks (`craft-ext check`, `check-submission`).
- **One HTTP agent.** All network calls use `craft_ext_store::catalogue::http()` (platform roots, a
  timeout); clippy's `disallowed-methods` refuses bare `ureq` calls.
- **Multi-step writes either finish or restore.** Snapshot the prior state and put it back on failure.
- **Fail closed.** A check that cannot run (confinement, signature, attestation) refuses; it never runs the
  unprotected path quietly.
- **Tests call the code as its users do** and assert literal results. A test that would pass if the code
  returned nothing is rewritten.
- **Checks for every change:** `cargo xtask comments`, `cargo clippy -p <crate> --all-targets -- -D warnings`,
  focused `cargo test -p <crate>`, and `python3 scripts/check-plugin-docs.py` for docs.

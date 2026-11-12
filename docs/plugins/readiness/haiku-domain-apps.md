# Domain app readiness audit

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: 2026-11-08 · Haiku 5.5 worker C · source-only at committed HEAD (`git show`, `git ls-tree`, `git grep`) · no builds, no tests, no runs, no fetches, no working-tree reads, no network.

Apps: SoundCraft, CADCraft, SolidCraft, TextureCraft, WordCraft, GridCraft. Every app was checked at its clone's committed HEAD. All six HEADs equal the assigned baseline, so no baseline comparison was needed.

Counts marked "static" come from `git grep` over source text. They were not produced by running the app, so they are approximate and say so. "Not found in inspected files" means only that: it is not proof that the behaviour is absent from the whole repository.

---

## Summary

| App | Revision (HEAD = baseline) | Runtime / plugin mechanism | Operation registry | CLI loads plugins? | Collision rule status | Smallest slice (PROPOSED, future) |
|---|---|---|---|---|---|---|
| SoundCraft | `da02c19fcca8fbcc49d66c957e5efcc67a63befe`, main, = baseline | Native in-process hosts for third-party CLAP, VST3 and AU plug-ins (no sandbox). Closed built-in DSP registry. No Craft package loader. | `command_specs()` static slice, about 390 `cmd!` sites (static) | Hosted plugins load lazily when a command names them. No installed-package concept. | Built-in ids cannot be shadowed by hosted ids (prefix plus lookup order). Duplicate command ids are checked only by a test. Shortcut duplicates are not checked. | Track-preset package (data only) loaded through existing `track.load_preset` validation. Later: `wasm:` insert effect behind the same `Plugin` trait. |
| CADCraft | `b04273e8974245e1c59ec57eb5ad86cec7a200eb`, main, = baseline | AutoLISP interpreter with file I/O and a bridge to every command. Startup `acad.lsp`/`acaddoc.lsp`. No package loader, no sandbox. | `command_specs()` (OnceLock), 435 `CommandSpec::new` sites (static) | No startup LISP in the CLI. Explicit `APPLOAD`/`lisp` work in the CLI. | Typed command names beat LISP `c:` functions (enforced). `defun` can overwrite built-in LISP functions (not enforced). Built-in hatch pattern names are protected (enforced). | Linetype/hatch-pattern package through existing parsers. Gate with LISP confinement first. |
| SolidCraft | `4bdffef06180fb3a2e8fbd4b15c7d3f60e55bd11`, main, = baseline | WASM plugin design only (`docs/plugins.md`). Manifest listing in the GUI. No WASM runtime in Cargo. Host-registered render executors. | `command_specs()`, 566 `CommandSpec::new` sites (static), 559 distinct ids | No plugin loader. | Render-executor duplicates and the reserved `cpu-raytrace` id are rejected (enforced). Command-id uniqueness is a test only. | `FeatureKind::Plugin` with a read-only fallback, following the design doc's `recorded_mesh` policy. |
| TextureCraft | `9dc33d894e9cd8e25ce84f6074fd1b476dad5698`, main, = baseline | Layer-stack evaluator (no node graph). MaterialCraft `.mcmat` via a `materialcraft-cli` subprocess. No plugin loader. | `command_specs()`, 29 module lists, about 160 `cmd!` sites (static) | No plugin loader. CLI uses a trusted local filesystem. | Command find is first-match. No uniqueness check found. Filter library is a closed `const` table. | Read-only preset/material-library import through existing `formats::presets`. |
| WordCraft | `ffb155a0321bba08da495efda66495104c1c8b9a`, main, = baseline | No plug-in runtime. Zotero bridge is an external caller. Customisable ribbon. | `Registry` (HashMap plus Vec), 651 `CommandSpec::new` sites (static) | No plugin loader. | `Registry::new` is last-wins on duplicate ids (not enforced). The uniqueness test covers this. `by_shortcut` is first-match. | Fail-closed registration (reject duplicates) as a core fix first. Then a read-only text-resource pack. |
| GridCraft | `178f3eb745a589c63b61ef97dd7b6b2d6fd48515`, main, = baseline | Macros with a trust and policy model (no plugin code). Python cells are not executed in this build. Public, doc-hidden runtime command registrar. | `command_specs()` plus `TEST_COMMANDS` fallback; about 316 library function names in a compile-time table | No plugin loader. | Built-in functions win over defined names and LET lambdas (enforced by dispatch order). Defined names may take function names (not enforced). The public runtime registrar has no namespace rule. | Core fix first: gate `register_test_command` out of production. Then a namespaced function-package proposal (high risk). |

---

## Cross-app findings (this group)

These are the blockers and the shared facts that matter for the plugin phase. Each is backed by source at the stated revision.

**F1. No registry validates ids at runtime, and none has a plugin registration seam.** Every app's command table is a static slice or `Vec` built once at startup.
- SoundCraft: `find_command` is a linear first-match (`crates/engine/src/cmd.rs:171-173`). Duplicates are caught only by the test `ids_are_unique_and_documented` (`cmd.rs:373-381`).
- CADCraft: `find_command` is first-match (`crates/engine/src/cmd.rs:216-218`). No id-uniqueness test was found in inspected files.
- SolidCraft: `find_command` is first-match, then a case-insensitive fallback (`crates/engine/src/cmd/mod.rs:375-378`). Uniqueness is a test (`crates/engine/src/tests.rs:1162`).
- TextureCraft: `find` is first-match (`crates/engine/src/commands.rs:87-89`). No uniqueness test was found.
- WordCraft: `Registry::new` inserts into a `HashMap` with `by_id.insert(s.id, i)` (`crates/engine/src/lib.rs:147-152`). That is last-wins, so a later duplicate silently replaces an earlier entry while `all()` still lists both. Uniqueness is a test (`crates/engine/src/tests.rs:131-137`).
- GridCraft: functions use `map.entry(s.name).or_insert(i)` (`crates/functions/src/lib.rs:225-228`), which is first-wins. The uniqueness check is `registry_is_consistent` (`lib.rs:267-272`).
- Required fix for the plugin phase: a single validated `register` that rejects duplicates and non-namespaced ids, with the test as a runtime check.

**F2. Caller identity is one level everywhere; loopback authentication is mixed.**
- No `execute` takes a caller: `Engine::execute(id, params)` (`soundcraft/crates/engine/src/lib.rs:392`), `Session::execute` (`cadcraft/crates/engine/src/session/execute.rs:11`), `Session::execute` (`solidcraft/crates/engine/src/lib.rs:1610`), `commands::execute` (`texturecraft/crates/engine/src/commands.rs:93`), `Session::run` (`wordcraft/crates/engine/src/session/run.rs:55`), `Session::execute` (`gridcraft/crates/engine/src/lib.rs:667`).
- Loopback control channels: token or key authentication exists in CADCraft (`cadcraft/apps/cadcraft/src/control_server.rs:65-80` issue, `:299-308` check), SolidCraft (`solidcraft/crates/engine/src/control_auth.rs:1-7`), TextureCraft (`texturecraft/crates/automation/src/security.rs:15-55`), WordCraft (`wordcraft/apps/wordcraft/src/control_server.rs:226`), GridCraft (`gridcraft/apps/gridcraft/src/control_server.rs:1-4`, `:57-64`).
- **SoundCraft has no authentication on its control channel** in the inspected files. It binds `127.0.0.1` (`soundcraft/apps/soundcraft/src/control_server.rs:34`), serves lines (`:107`), and dispatches `engine.execute` to the engine (`soundcraft/crates/ui-egui/src/control.rs:440-451`). No token, key or nonce appears in those files.
- Trust tiers and path roots exist, but not per caller identity: TextureCraft `Filesystem::{Denied, TrustedLocal, …}` (`texturecraft/crates/automation/src/headless.rs:18-22`, `:143-148`) with `authorize_command` for untrusted callers (`workspace.rs:136-150`); GridCraft macro policy (`gridcraft/crates/engine/src/macros/policy.rs:62-84`); SoundCraft `AutomationRoots` (`soundcraft/crates/engine/src/file_access.rs:38-45`, `:154`, `:183`); SolidCraft `agent_scope` with `--allow-any-path` (`solidcraft/apps/solidcraft-cli/src/main.rs:784-795`).

**F3. CADCraft's AutoLISP runtime is the largest existing extension surface and has no confinement.** It can read files up to 64 MiB and write to any path whose parent directory exists (`cadcraft/crates/engine/src/lisp/builtins/files.rs:76-82`, `:91-93`, `:101-107`). `(command …)` reaches the whole command registry (`cadcraft/crates/engine/src/lisp/drawing.rs:71-` calling `Session::start`). Parenthesised text on the command line, in a script, or through MCP `command_line`/`script` is evaluated as LISP (`cadcraft/crates/engine/src/session/cmdline.rs:54`, `:189`; `cadcraft/crates/mcp/src/tools.rs:11-13`). LISP built-ins live in the same `globals` map as user symbols, so `defun` can overwrite them (`cadcraft/crates/engine/src/lisp/eval.rs:507-512`, `:597`; `cadcraft/crates/engine/src/lisp/builtins.rs:273-276`). Any plugin that runs in this interpreter inherits all of this. Confinement and a capability gate are a blocker before any package code can run there.

**F4. No Craft package model exists in any app.** There is no manifest, digest, signature, grant or enforced permission for an installed package in any of the six. The only nearby mechanisms are:
- SoundCraft in-process native hosts (`soundcraft/crates/clap-host/src/registry.rs:54-68`, `:163-172`; `soundcraft/crates/clap-host/src/scan.rs:21-35`). They run third-party native code with no sandbox found in the inspected files.
- SolidCraft's design-only WASM plan (`solidcraft/docs/plugins.md`, "Two plugin shapes", "Missing plugin and API drift"). The workspace has no WASM runtime dependency. The GUI page lists manifests and "does not load or run anything from the folder" (`solidcraft/crates/ui-egui/src/plugins.rs:1-9`).

**F5. Transactions are uniform: snapshot, label, rollback.** All six take a copy of the document (an `Arc` clone or a whole-value clone) before a command, restore it on error and on panic, and push a labelled undo step. A plugin command can reuse this if it goes through the same entry point. Long-running work is the exception:
- SoundCraft `Bounce` is cooperative and steps per frame (`soundcraft/crates/engine/src/io.rs:261-291`; `soundcraft/crates/ui-egui/src/bounce.rs:16-32`, `:82`).
- SolidCraft has a render-job model (`solidcraft/crates/engine/src/render_jobs.rs:1-6`).
- CADCraft's LISP runs in step slices (`cadcraft/crates/engine/src/lisp.rs:193-195`).
- No job or progress model was found in WordCraft, GridCraft or TextureCraft's command paths.

**F6. Persistence stores no extension references with a version or digest.**
- SoundCraft `Insert.plugin` is a string id with an opaque `state` blob (`soundcraft/crates/model/src/mixer.rs:425-440`). No version or digest field was found.
- SolidCraft `FeatureKind` has no plugin variant (`solidcraft/crates/doc/src/document.rs:615`; grep for a `Plugin` variant returned none). The format string `"solidcraft/N"` with step migrations is the strongest versioning in the group (`solidcraft/crates/doc/src/format.rs:1-8`).
- TextureCraft `MaterialLink` embeds the package as an image resource (`texturecraft/crates/core/src/layer.rs:384-390`). This is the best precedent for a self-contained reference.
- CADCraft writes hatch pattern lines into the hatch, so a drawing keeps its pattern without the `.pat` file (`cadcraft/crates/engine/src/cmd/hatch/patfile.rs:3-6`).

**F7. Realtime audio constraints are a SoundCraft-only concern, and the code shows effort but not a guarantee.** The rule is stated (`soundcraft/AGENTS.md:30`: "The audio callback never blocks, allocates in steady state, or panics"). The callback uses non-blocking channel reads and `try_lock` (`soundcraft/crates/playback/src/callback.rs:135`, `:402`; `soundcraft/crates/playback/src/lib.rs:6-7`). Reconfiguration still allocates on the audio side: `callback.rs:150-158` (new engine on rate change, pending buffer resize), `callback.rs:238` (input buffer resize), and `soundcraft/crates/mix/src/lib.rs:272`, `:289`, `:337`, `:631`. A plugin DSP contract therefore needs its own budget and an allocation test; it cannot assume the host path is allocation-free.

**F8. GridCraft function-name shadowing: built-ins win by dispatch order, but nothing stops a conflicting name.** The calc evaluator matches host special forms first (`gridcraft/crates/calc/src/eval/call.rs:8-9`), falls back to the library (`call.rs:131`), and only then reaches defined names and LET lambdas (`call.rs:287-296`). So `SUM(…)` resolves to the built-in even if a defined name `SUM` exists. Defined-name validity does not refuse function names (`gridcraft/crates/engine/src/cmd/formulas.rs:323-333`). Separately, `register_test_command` is `pub` and `#[doc(hidden)]` and is not `cfg(test)`-gated (`gridcraft/crates/engine/src/cmd.rs:202-205`). It appends after built-ins (`cmd.rs:190-197`) and so cannot shadow them, but it is a production-reachable runtime registration path with no namespace rule.

---

## SoundCraft (`/Users/ash/Projects/soundcraft`)

**1. Repo, revision, workspace.** Branch `main`. HEAD `da02c19fcca8fbcc49d66c957e5efcc67a63befe` equals the baseline. Workspace members (root `Cargo.toml:1-5`): `crates/*`, `apps/*`, `xtask`. Crates: au-host, audio-io, automation, clap-host, dsp, engine, midi, mix, model, playback, time, ui-egui, video, vst3-host. Apps: soundcraft, soundcraft-cli, soundcraft-web. Lints: `unsafe_code = "forbid"` at workspace level (root `Cargo.toml`, `[workspace.lints.rust]`). clap-host and vst3-host opt out to `deny` with FFI-only allow (`crates/clap-host/Cargo.toml:31-32`; `crates/vst3-host/Cargo.toml:53-54`). Licence `MIT OR Apache-2.0`.

**2. Operation surface.**
- Registry: `command_specs()` concatenates per-area `specs()` (`crates/engine/src/cmd.rs:129-170`). `find_command` is first-match (`cmd.rs:171-173`). `CommandSpec` fields: `id`, `label`, `menu`, `shortcut`, `params`, `enabled`, `run`, `journal`, `undoable`, `view` (`cmd.rs:74-95`). Macro variants `query`, `noundo`, `view`.
- Size (static): about 390 `cmd!` call sites plus 94 `flag!`/`toggle!`/`radio!`/`option!` sites in `crates/engine/src/cmd`. The test asserts more than 150 (`cmd.rs:373-381`). The exact registered count was not produced.
- Categories seen: query (`engine.clap_plugins`, `engine.vst3_plugins`, `cmd/clap.rs:10-21`, `cmd/vst3.rs:10-21`), file and session (`cmd/file.rs:14-90`), edit, track, clip, event, mix, transport, view, setup, audiosuite, midi, instrument, sampler, video.
- Traced operation, `mix.insert` (spec `cmd/mix.rs:97`):
  1. `insert` (`cmd/mix.rs:324-351`) requires `plugin`, then resolves it with `super::clap::plugin_info` (`cmd/clap.rs:30-35`: built-in, then CLAP, VST3, AU).
  2. It resolves the track (`tracks_required`) and the slot (`optional_slot`).
  3. It mutates through `e.session_mut()` (`crates/engine/src/lib.rs:276-281`, copy-on-write, bumps revision).
  4. It builds `Insert::new(info.id)` with default params, applies the `params` object by checking each key against `info.params` and clamping to `min`/`max` (`mix.rs:339-345`), then writes `tr.mixer.inserts[slot]` (`mix.rs:346-348`).
  5. It returns `{track, slot, plugin}`.
  6. Wrapper `Engine::execute` (`lib.rs:392-440`): `find_command`, `blocked_while_recording` (`lib.rs:386-389`), enabled check, `catch_unwind` (`lib.rs:412`). On success it runs `remember_selection`, `push_undo` (`lib.rs:372`) and `journal_append` (`lib.rs:503`). On error it restores the document.
  7. The new `Arc<Session>` reaches the audio thread through `Player::update_session` and the `Cmd::Session` drain (`crates/playback/src/lib.rs:16-22`; `callback.rs:150-152`).
- Operation categories and the UI-only split: UI-only menu methods such as `ui.menu.invoke` are rejected in headless mode (`crates/automation/src/lib.rs`, the `m if m.starts_with("ui.")` arm).

**3. Existing plugin, extension, script, theme and resource mechanisms.**
- Built-in DSP plugins: closed `REGISTRY` (`crates/dsp/src/plugins.rs:19-50`, `pub(crate) static`) and `instantiate` match (`plugins.rs:53-`), both `pub(crate)`.
- Third-party hosting, in-process and unsandboxed:
  - CLAP: `soundcraft-clap-host` using `libloading` and `clap-sys = "0.5"` (`crates/clap-host/Cargo.toml:25-26`; `src/ffi.rs:3-4`). Scans `$HOME/Library/Audio/Plug-Ins/CLAP`, `/Library/Audio/Plug-Ins/CLAP` and `CLAP_PATH` (`crates/clap-host/src/scan.rs:21-35`). Scan cache `ensure_scanned` (`registry.rs:72-77`). Instantiation by id `clap:<id>` (`registry.rs:163-172`), which loads the bundle with `Bundle::load` (dlopen).
  - VST3: `soundcraft-vst3-host`, an isolated unsafe FFI crate (`crates/vst3-host/src/lib.rs:122-131`).
  - AU: `soundcraft-au-host`, macOS only, hand-written AudioToolbox declarations (`AGENTS.md:27`).
- ABI version: CLAP via `clap-sys` 0.5 (version not checked against the spec). VST3 via bindings named in `AGENTS.md:26`. No host ABI policy found.
- Host functions, sandbox limits (fuel, memory, time), grants, digest checks: none found in the inspected files.
- Scripts: none found. Recording: the command journal (`lib.rs:33-36`, `:187-189` docs say "for macro recording / agents").
- Themes: `ThemeMode` enum (`crates/ui-egui/src/lib.rs:138-139`). No theme registry.
- Resources: track presets as JSON with format `soundcraft-track-preset` (`crates/engine/src/cmd/track_more.rs:18`, `:248-276`); session templates `blank|demo` (`cmd/file.rs:14`, `:215`); SMF import and export (`cmd/file.rs:69`, `crates/engine/src/io/midi.rs`); audio import (`cmd/file.rs:58-66`, `crates/engine/src/io/import.rs`).
- MIDI input: device selection stored as session flags (`crates/engine/src/cmd/setup_more.rs:459-470`). No live MIDI device driver was found in the inspected playback or UI files.

**4. Contribution families.**

| Family | Seam today | Evidence |
|---|---|---|
| Commands | None (static slice) | `cmd.rs:129-170` |
| Menus | Static `menu` arrays; no registration | `cmd.rs:74-95`; `crates/engine/catalog/menus.txt` |
| Shortcuts | Static `Option<&str>`; first-match lookup | `crates/ui-egui/src/shortcuts.rs:108-112` |
| Effects and instruments | Hosted seam for third-party ids only (`clap:`, `vst3:`, `au:`) | `cmd/clap.rs:30-35` |
| Import/export formats | None: closed `FileFormat` enum | `crates/audio-io/src/lib.rs:55`, `:128`, `:134`, `:152`, `:163` |
| Resources | Track presets validated by the loader; no registry | `cmd/track_more.rs:270-276` |
| Themes | Enum only | `ui-egui/src/lib.rs:138-139` |
| Jobs | `Bounce` only; no job registry | `crates/engine/src/io.rs:162`, `:261`, `:291` |
| File-type handlers, default apps | Not found in inspected files | (grep for default-handler APIs: 0 hits) |

**5. Transactions, undo, jobs, persistence, output.**
- Transactions: copy-on-write `Arc<Session>` (`lib.rs:276-281`). Undo stack of `{label, doc}` (`lib.rs:337-370`), trimmed by `history_limit` in `push_undo` (`lib.rs:372-384`). `execute_merged` collapses a gesture key into one step (`lib.rs:462-500`). Error and panic both restore the document (`lib.rs:412-420`, plus the error branch after it).
- Journal: ring buffer of 10,000 entries, each up to 64 KiB of params (`lib.rs:33-36`, `:503-508`).
- Project file: `.scraft` JSON through `Session::to_json`/`from_json` (`crates/model/src/session.rs:576-581`). Audio written as 32-bit float WAV into `Audio Files/` (`crates/engine/src/io/session.rs:9-11`). Browser bundle `.scraftz` (`io/session.rs:6-8`). Autosave under `catch_unwind` (`crates/engine/src/autosave.rs:146`). No schema version field was found in `model/src/session.rs:576-581`.
- Plugin references: `Insert.plugin` (`crates/model/src/mixer.rs:427`) and opaque `Insert.state` (`mixer.rs:440`). No version or digest.
- Jobs: `Bounce` (`crates/engine/src/io.rs:162-291`) renders in steps. The UI steps it per frame (`crates/ui-egui/src/bounce.rs:16-32`) and cancels it (`bounce.rs:82`). The file appears at its path only when complete (`io.rs:160-162` doc).
- Headless output: `file.bounce_mix` calls `bounce_job(...).run()` synchronously (`crates/engine/src/cmd/file.rs:299-306`). The CLI `--bounce` runs `e.execute("file.bounce_mix", ...)` and returns after the file is written (`apps/soundcraft-cli/src/session.rs:36-49`).

**6. Collision and core-override behaviour.**

| Rule | Status | Evidence |
|---|---|---|
| A new command id cannot shadow a built-in id | NO-SEAM (no registration path exists) | `cmd.rs:129-170` |
| Duplicate command id at runtime | NOT-ENFORCED; test only | `cmd.rs:373-381`; `find_command` first-match `cmd.rs:171-173` |
| Duplicate shortcut | NOT-ENFORCED; first-match | `ui-egui/src/shortcuts.rs:108-112`; no uniqueness check found |
| Menu placement replacement | NO-SEAM | `cmd.rs:74-95` |
| Hosted plugin id colliding with built-in id | ENFORCED (namespace prefix; built-in looked up first) | `cmd/clap.rs:31-34` |
| Duplicate hosted plugin id across folders | ENFORCED dedupe; first found wins, silently. HOME path is scanned before `/Library`. | `clap-host/src/registry.rs:54-68`; `scan.rs:31-35` |
| Built-in plugin replacement | ENFORCED by visibility (`pub(crate)` registry) | `dsp/src/plugins.rs:19`, `:53` |
| File-type handler or default exporter replacement | NO-SEAM (closed enum) | `audio-io/src/lib.rs:55-163` |
| OS default-app handling | NOT-FOUND-IN-INSPECTED-FILES | none |
| Plugin namespaces | Only the `clap:`/`vst3:`/`au:` prefixes for hosted ids; no Craft namespace | `cmd/clap.rs:30-35` |

**7. GUI/CLI parity.**
- The GUI control channel dispatches `engine.execute` and `command` to `app.run(id, params)` (`crates/ui-egui/src/control.rs:440-451`). Headless `Headless::answer` calls `self.engine.execute(id, &p)` (`crates/automation/src/lib.rs:53-58`). The CLI calls `e.execute` (`apps/soundcraft-cli/src/session.rs:49`). All three share the registry and `execute`.
- Programmatic calls never open dialogs (`crates/automation/src/mcp.rs:12-17`, the INSTRUCTIONS text).
- A fresh CLI process loads third-party CLAP/VST3/AU lazily when a command names one (`clap-host/src/registry.rs:72-77`, `:163-172`). There is no installed-plugin list in the CLI. `soundcraft-cli plugins` lists built-ins only (`apps/soundcraft-cli/src/args.rs:60-61`).

**8. Caller authority.**
- Dispatch has no caller parameter (`lib.rs:392`).
- Loopback control: no authentication found (`apps/soundcraft/src/control_server.rs:34`, `:107`; `crates/ui-egui/src/control.rs:440`).
- MCP stdio: no authentication found (`crates/automation/src/mcp.rs`).
- Path scope: `AutomationRoots` read and write roots (`crates/engine/src/file_access.rs:1-20`, `:38-45`). Enforced per call by `Headless::call` through `confine` (`crates/automation/src/lib.rs:74-78`). Process-wide scope via `confine_process` (`file_access.rs:154`). `unconfined` is for the app's own folders (`file_access.rs:183`). The scope is keyed by process and thread, not by caller identity.

**9. Reusable APIs and missing hooks.**
- Reusable now: `Engine::execute`, `command_specs`, `find_command`, `Engine::session_mut`, `Engine::push_undo`, the `plugin_info` chain (`cmd/clap.rs:30-35`), clap-host `scan`/`rescan`/`scan_paths`/`instantiate`, `Bounce::new`/`step`/`run`/`progress`, `AutomationRoots` and `file_access::check_read`/`check_write`, the `soundcraft_dsp::Plugin` trait (used by `clap-host/src/plugin.rs:208`).
- PROPOSED missing hooks: a validated `Contribution` registry with namespaced ids and duplicate rejection; a caller-scoped operation facade (OperationSet keyed by caller class: GUI, CLI, MCP, plugin); a resource importer trait for presets and packs; a WASM adapter implementing `soundcraft_dsp::Plugin` with an explicit block budget; a package manifest and digest verifier.

**10. App-specific data constraints.**
- Audio callback: "never blocks, allocates in steady state, or panics" (`AGENTS.md:30`). Mechanisms: `guarded_block` with `catch_unwind` (`crates/playback/src/callback.rs:55-61`), `try_recv` drains (`callback.rs:135`, `:402`), `try_lock` meters (`playback/src/lib.rs:6-7`), audio-thread marker (`crates/mix/src/threads.rs:8-14`).
- Reallocation paths still present: `callback.rs:158`, `:238`; `mix/src/lib.rs:272`, `:289`, `:337`, `:631`. Hosted plugin instances are created and destroyed off the audio thread; only `process` runs on it (`docs/architecture.md:62-63`).
- Sample rate: a rate change creates a new mix engine on the audio thread (`callback.rs:150-152`).
- Position: values clamped to ±`MAX_POSITION` (`cmd.rs`, `position_param`).
- Control input: lines up to 64 MiB of decoded base64 plus 1 MiB (`apps/soundcraft/src/control_server.rs:14-20`); 16 concurrent connections (`:24`).

**11. Smallest useful slice and proof (PROPOSED, future work, not implemented).**
- Slice 1, data only: a track-preset package. Presets load through the existing `track.load_preset` validation (`track_more.rs:270-276`). Plugin ids inside a preset must resolve through `plugin_info`, so a package cannot introduce a new id.
- Proof (future test): a preset naming `eq_7band` loads and undoes. A preset naming `wasm:x` is refused. A snapshot of `command_specs()` ids is unchanged after the load.
- Slice 2, later and higher risk: a WASM insert effect through a new `wasm:<pkg>/<id>` branch in `plugin_info`, behind the same `Plugin` trait.
- Proof (future test): block processing under a fixed budget, and an allocation-counting test run with `on_audio_thread()` set. Neither exists today.

**12. Evidence limits.** Not traced: clap-host host callbacks and `ffi/*` internals; VST3 and AU internals; the mix render graph beyond the entry point; the full MCP tool table; `ui-egui` menu wiring beyond `control.rs`; the track-preset apply path beyond validation; `docs/plugin-parity.md` (named in `AGENTS.md:7`, not read). No runtime behaviour is claimed.

---

## CADCraft (`/Users/ash/Projects/cadcraft`)

**1. Repo, revision, workspace.** Branch `main`. HEAD `b04273e8974245e1c59ec57eb5ad86cec7a200eb` equals the baseline. Workspace (root `Cargo.toml:1-14`): `crates/*` (color, constraints, doc, dwg, dxf, engine, fonts, geom, io, mcp, render, ui-egui), `apps/*` (cadcraft, cadcraft-cli, cadcraft-web), `xtask`. Workspace dependency `craft-docpath` pinned to a craft-libs revision (root `Cargo.toml`). `#![forbid(unsafe_code)]` in `crates/geom/src/lib.rs` and `crates/constraints/src/lib.rs`. Other crates' lint headers were not all checked.

**2. Operation surface.**
- Registry: `command_specs()` (`crates/engine/src/cmd.rs:173-214`, OnceLock with about 40 `v.extend(...)` lines). `CommandSpec` (`cmd.rs:67-80`): `id`, `label`, `menu`, `shortcut`, `aliases`, `params`, `enabled`, `run`, `undoable`, `interactive: Option<Factory>`, `transparent`. `find_command` is first-match on the lowercased id (`cmd.rs:216-218`). `resolve_alias` gives command names priority over user aliases, which come before built-in aliases (`cmd.rs:223-231`).
- Size (static): 435 `CommandSpec::new` sites, 433 distinct single-line ids, 255 `.interactive(` sites, 34 `.transparent()` sites, 51 `specs()` functions. The exact registered count was not produced.
- Categories: draw, modify, annotate, blocks/xref, layers, layouts, file, inquiry, settings, constraints, view, LISP (`appload`, `lisp`), utility.
- Traced operation, `circle` (spec `crates/engine/src/cmd/draw.rs:53-57`, alias `c`, interactive `CircleM`):
  1. `Session::execute` (`crates/engine/src/session/execute.rs:11-27`) wraps `execute_guarded` in `catch_unwind`.
  2. `execute_guarded` (`execute.rs:30-100`) finds the spec, runs `enabled` (`has_doc`, `cmd.rs:165-167`), checks `refused_in_editor`, takes a snapshot of document and selection, and runs `run` under `catch_unwind`.
  3. `run_circle` (`crates/engine/src/cmd/draw/circle.rs:6-26`) validates `radius` or `diameter` with `size_param` (positive and finite), builds `Circle::new`, `from_3_points` or `from_2_points` from `cadcraft-geom`, rejects radius `<= 1e-12`, then calls `s.add_entity(...)` and returns the new handle.
  4. Back in `execute_guarded`, for undoable commands the result passes `refuse_non_finite`, then `push_undo` with the label, then `revision += 1` (`execute.rs:30-100`).
- Interactive path: `Session::start` (`crates/engine/src/session/interactive.rs:10-31`) builds the `Interactive` machine.

**3. Existing plugin, extension, script, theme and resource mechanisms.**
- AutoLISP interpreter: explicit-stack machine (`crates/engine/src/lisp/eval.rs`, `crates/engine/src/lisp.rs:1-7` doc). Built-in table `BUILTINS` installed into each drawing's environment (`crates/engine/src/lisp/builtins.rs:273-277`). Per-drawing symbols (`lisp.rs:1-7`).
- Loading: `APPLOAD` (`crates/engine/src/cmd/lisp.rs:12-16`) and the `lisp` command (`cmd/lisp.rs:18-22`, accepts `inputs` for `getpoint`-style prompts).
- Startup: `acad.lsp` once per session and `acaddoc.lsp` per drawing, from the support directory set only by the GUI (`crates/engine/src/lisp/startup.rs:1-10`; `apps/cadcraft/src/main.rs:385`). Startup files are never read from the drawing's own folder (`startup.rs:1-10`).
- Sandbox and limits: none for file access. Limits are step and memory counts: `SLICE` 400,000 steps (`lisp.rs:193`), `BLOCKING_CAP` 500,000,000 steps (`lisp.rs:195`), `MAX_STACK` 100,000 (`lisp/eval.rs:163`), `MAX_CELLS` 5,000,000 (`lisp/value.rs:44`), `MAX_STRING` 8 MiB (`value.rs:46`), `STRING_BUDGET` 2 GiB per run (`value.rs:48`), read cap 64 MiB (`lisp/builtins/files.rs:76-82`).
- Hatch patterns: user `.pat` files, process-global store (`crates/engine/src/cmd/hatch/patfile.rs:21-22`), `MAX_USER_PATTERNS` 4096 (`patfile.rs`, the constant near the top), loader `patfile.rs:66-91`. Built-in names cannot be replaced (`patfile.rs:7`, `:70-76`).
- Linetypes and patterns: `parse_lin` (`crates/doc/src/library/files.rs:91`), `parse_pat` (`files.rs:260`), standard library (`crates/doc/src/library.rs:13`, `:100`), pattern lookup with metric scaling (`library.rs:240-251`).
- Plot styles and plotters: plot-style tables (`crates/doc/src/plotstyle.rs:206-217`, `:384-394`); a host-injected plotter store (`crates/engine/src/cmd/plotters.rs:24-36`, `set_plotter_store`; `:178` `devices()`).
- User alias file (ALIASEDIT): overrides built-in aliases, not command names (`crates/engine/tests/aliases.rs:41-43`).
- Themes: not found in inspected files. Plugin manifests: none. Grants: none. Digests: none.

**4. Contribution families.**

| Family | Seam today | Evidence |
|---|---|---|
| Commands | Static registry only | `cmd.rs:173-214` |
| `c:` commands from LISP | Per-drawing runtime seam, used only when no built-in matches | `lisp.rs:660-672`; `session/interactive.rs:20-31` |
| LISP functions | Per-drawing, unrestricted, overwrite built-ins | `eval.rs:507-512`; `builtins.rs:273-277` |
| Hatch patterns | Process-global `.pat` store; built-in names protected | `patfile.rs:66-91` |
| Linetypes | Parser into drawing | `files.rs:91` |
| Plot styles and devices | Host-injected store | `plotters.rs:24-36` |
| Import/export formats | Closed `Format::{READ,WRITE}` | `apps/cadcraft-cli/src/args.rs:39-54`; `crates/io/src/lib.rs:98-104` |
| Themes, default apps | Not found | none |

**5. Transactions, undo, jobs, persistence, output.**
- Transactions: `Snapshot {label, doc, selection}` pushed per undoable command (`session/execute.rs:30-100`). Undo and redo (`execute.rs:102`, `:116`). A LISP run is one undo step (`lisp.rs:1-7` doc; `execute.rs` `lisp_groups_undo`). Multiple open drawings keyed by `uid` (`execute.rs:14-18`).
- Persistence: DXF and DWG through `crates/io` (`crates/io/src/lib.rs:98-104`). Atomic replacement with exclusive temp file and rename (`crates/engine/src/atomic_write.rs:1-12`). The LISP environment is in memory only; no persistence of LISP in `crates/doc` or `crates/io` was found.
- Plugin references in files: none. Hatch pattern lines are embedded (`patfile.rs:3-6`).
- Jobs: autosave `Job` enum (`crates/engine/src/autosave.rs:127`). Hatch render takes an `AtomicBool` cancel (`crates/render/src/hatch.rs:16-20`). LISP runs advance per frame (`lisp.rs:290-300`). CLI `run` is synchronous (`apps/cadcraft-cli/src/run.rs:199`). No command progress model was found.

**6. Collision and core-override behaviour.**

| Rule | Status | Evidence |
|---|---|---|
| A typed name that is a built-in command wins over a LISP `c:` function | ENFORCED-IN-CODE | `session/interactive.rs:20-24` (built-in found first; LISP only when `find_command` is `None`) |
| LISP `c:` function vs built-in alias | ENFORCED-IN-CODE (`resolve_alias` gives built-ins first; `c` resolves to `circle`) | `cmd.rs:223-231` |
| User alias file overriding a built-in alias | Allowed by design (NOT-ENFORCED, intended) | `crates/engine/tests/aliases.rs:41` |
| User alias overriding a command name | ENFORCED (command names win) | `cmd.rs:224-227`; `tests/aliases.rs:42` |
| `(defun NAME …)` overwriting a built-in LISP function | NOT-ENFORCED (same `globals` map) | `eval.rs:507-512`; `builtins.rs:273-277` |
| LISP `(command "LINE" …)` reaching any built-in command | Allowed; no allowlist found | `lisp/drawing.rs:71-` → `Session::start` |
| Duplicate command id | NOT-FOUND (no uniqueness test found) | `cmd.rs:216-218` |
| Built-in hatch pattern name replaced by a file | ENFORCED (warning, library kept) | `hatch/patfile.rs:7`, `:70-76` |
| A later `.pat` with an existing user name | NOT-ENFORCED (replaces silently) | `patfile.rs:81-86` |
| Shortcut duplicates | NOT-FOUND | `cmd.rs` `shortcut` field |
| Menu placement replacement | NO-SEAM | `cmd.rs:67-80` |
| Default file-type handler | NOT-FOUND | none |
| Duplicate LISP file or `c:` name | NOT-ENFORCED; re-`APPLOAD` redefines in place | `lisp.rs:702-711` |

**7. GUI/CLI parity.**
- Shared registry: CLI `commands` lists `command_specs()` (`apps/cadcraft-cli/src/commands.rs:10`). CLI `run` executes `s.execute(id, &params)` (`apps/cadcraft-cli/src/run.rs:199`). CLI `script` runs `Session::script` (`crates/engine/src/session/cmdline.rs:165-185`), which accepts LISP expressions (`cmdline.rs:54`).
- Startup LISP is not loaded in the CLI. `set_lisp_support_dir` is called only by the GUI (`apps/cadcraft/src/main.rs:385`). `APPLOAD` and `lisp` do work in the CLI.
- A fresh CLI loads no installed plugin. Only explicit `.lsp` paths or inline LISP are evaluated.
- Dialogs: `execute` is documented as no dialogs (`session/execute.rs:8-10`). LISP `getpoint` and `pause` wait; a headless caller can only answer through `inputs` (`lisp.rs:234`, `cmd/lisp.rs:19-22`).

**8. Caller authority.**
- Control channel: per-session token in an owner-only file, required on every request (`apps/cadcraft/src/control_server.rs:65-80`, `:299-308`; `crates/mcp/src/control_token.rs:61`, `:94`). The MCP client retries unauthorised once (`crates/mcp/src/backend.rs:255-275`).
- Read-only methods are only a retry-safety list (`crates/mcp/src/backend.rs:193-205`). They are not an authority split.
- Once a caller has the token, every command and every LISP expression runs with the desktop user's authority. Nothing separates a GUI user, an MCP client and a script.
- LISP file access is unconfined (`files.rs:16-30`, `:91-107`). The only path confinement found is in the build script for fonts (`crates/fonts/build.rs:76`), which is build-time.

**9. Reusable APIs and missing hooks.**
- Reusable: `Session::execute`, `command_specs`, `find_command`, `resolve_alias`, `Session::lisp_eval`/`lisp_load`, `lisp_commands`, `doc::library::pattern_for`, `patfile::load`, `io::write_with`, `atomic_write`, the geometry and constraint crates (`crates/constraints/src/lib.rs` `solve`, `dof`).
- PROPOSED missing hooks: a capability gate for LISP (file read, file write, `(command …)` allowlist); a namespaced `c:`-style contribution registry that cannot overwrite built-in LISP functions; a caller-scoped operation facade; a pattern/linetype pack importer; a WIT-style geometry interface, which would need handle and point types not present today.

**10. App-specific data constraints.**
- Units: drawing units. `INSUNITS` set in the header (`crates/doc/src/header.rs:146`, `:160`). Metric pattern scaling by 25.4 when `MEASUREMENT` is 1 (`library.rs:248-251`).
- Tolerances: `EPS` 1e-9 drawing units (`crates/geom/src/lib.rs:33`); `near(tol)` coincidence (`crates/geom/src/vec.rs:74`); radius minimum 1e-12 (`cmd/draw/circle.rs:21-23`).
- Angles: radians, counter-clockwise from +X (`crates/geom/src/lib.rs:1-7`).
- Constraint solver: `MAX_ITER` 200 (`crates/constraints/src/lib.rs:52`); `RANK_TOL` 1e-7 (`crates/constraints/src/lm.rs:190`); expression length limit 1024 characters (`crates/constraints/src/expr.rs`, `MAX_LEN`).
- Pattern and linetype caps: 4096 user patterns (`patfile.rs`). LISP memory caps as listed in section 3.

**11. Smallest useful slice and proof (PROPOSED, future work).**
- Slice 1: a pattern or linetype pack, read only, through `parse_pat`/`parse_lin` and `patfile::load`. Built-in names are already protected, so the slice relies on existing enforcement.
- Proof (future test): a package `.pat` named like a built-in keeps the library definition and reports the warning (`patfile.rs:70-76`). A package pattern used in a hatch survives save and reopen without the package (`patfile.rs:4-5`).
- Precondition: the LISP capability gate in F3 is decided first, because a pack loader that accepts LISP would inherit F3.
- Slice 2, later and higher risk: LISP-based extensions behind a capability gate (file access off unless granted, `(command …)` allowlist). Proof: `open` refuses without a grant; `(command "QSAVE")` is refused when not allowlisted.

**12. Evidence limits.** Not traced: the GUI-side dispatch target of `control_server` requests (`apps/cadcraft/src/control_server.rs` beyond the auth check); DWG reader internals; the full MCP tool table beyond `tools.rs:11-25`; plot-style apply path; any file-path command (`open`, `saveas`, `plot`) for confinement; `xtask`. No runtime behaviour is claimed.

---

## SolidCraft (`/Users/ash/Projects/solidcraft`)

**1. Repo, revision, workspace.** Branch `main`. HEAD `4bdffef06180fb3a2e8fbd4b15c7d3f60e55bd11` equals the baseline. Workspace (root `Cargo.toml:1-…`): `crates/*`, `apps/*`, `xtask`; `vendor` excluded (third-party with local fixes). Crates counted by `Cargo.toml` listing: 17 (derive, doc, engine, geom, gpu, io, kernel, mcp, profile, render, sketch, smlib, space-mouse, step, testkit, threedx-client, ui-egui). Apps: solidcraft, solidcraft-cli, solidcraft-web. Licence `MIT OR Apache-2.0`. Repository field in `Cargo.toml` names `bherbruck/solidcraft`; the older inventory names `tangency-labs/solidcraft`. The remote was not read, so the owner is unresolved.

**2. Operation surface.**
- Registry: per-module `pub static COMMANDS: &[CommandSpec]` (for example `crates/engine/src/cmd/features.rs:33`), aggregated by `command_specs()` (`crates/engine/src/cmd/mod.rs:311`). `find_command` is first-match, then a case-insensitive fallback (`cmd/mod.rs:375-378`).
- `CommandSpec` (`cmd/mod.rs:143-170`): `id`, `label`, `tab`, `panel`, `icon`, `shortcut`, `params`, `enabled`, `run`, `undoable`, `live`, `reads_model`, `planned`, `contract`, `output` (JSON schema of the result). The `planned!` macro registers an unbuilt command, disabled with a reason and an issue number (`cmd/mod.rs:15-20`).
- Size (static): 566 `CommandSpec::new` sites, 559 distinct single-line ids, one `planned!` site. Exact count not produced.
- Traced operation, `solid.extrude` (spec `crates/engine/src/cmd/features.rs:34`, `crate::checked!` wrapper): `Session::execute` (`crates/engine/src/lib.rs:1610`) runs the profiler span and stale-model refresh (`lib.rs:1610-1625`), checks enablement, calls `check_command` before any change (`lib.rs:1625-1630`), snapshots document, visibility and pending moves (`lib.rs:1630-1640`), then runs the command under `catch_unwind` (`lib.rs:1650-1660`). What happens to the timeline and kernel after the run was not traced. The undo step is pushed at `lib.rs:1721`.
- Categories: sketch (many `sketch.*`), features (`solid.*`, `surface.*`, `sheet.*`), assembly and components, config, BOM, joints and motion, render and render jobs, inspect, file, view, settings, diagnostics, materials and libraries.

**3. Existing plugin, extension, script, theme and resource mechanisms.**
- WASM plugins: design only (`docs/plugins.md`). Its goals include "Sandboxed by default: manifest permissions, fuel/memory limits on desktop" and "one command surface". No `wasmtime`, `wasmi`, `wit-bindgen` or `extism` dependency exists in any `Cargo.toml` (grep). Only `wasm-bindgen` in the web app and `vendor/winit`.
- GUI plugin page: lists `plugin.json` manifests under `{data}/plugins/<id>/` with a 16 KiB limit and `deny_unknown_fields` (`crates/ui-egui/src/plugins.rs:1-9`, `:22`, `:100`). It "does not load or run anything from the folder".
- Render executors: host-registered image renderers (`register_render_job_host`, `crates/engine/src/render_jobs.rs:507`). Duplicate executor ids are rejected (`crates/render/src/backend.rs:266`, `:340`, `:288`).
- Slicers: external slicer applications are detected and opened with a mesh (`crates/engine/src/slicers.rs:1-9`, `:884` `Slicer::open`). No plugin contract.
- Materials and libraries: local material libraries with inspect, export and exact-revision apply (`crates/engine/src/cmd/libraries.rs:1-8`). Appearance packs (`crates/doc/src/appearance_pack.rs:1`). Physical material cards with `schema_version` 1 (`crates/doc/src/physical_material.rs:13`, `:264-265`).
- Scripts: `script` subcommand (`apps/solidcraft-cli/src/main.rs:167-169`). Recorded command lists are not a plugin mechanism.
- Themes: not found in inspected files.

**4. Contribution families.**

| Family | Seam today | Evidence |
|---|---|---|
| Commands | Static per-module `COMMANDS` slices; `planned!` placeholders | `cmd/mod.rs:15-20`, `:311` |
| Feature types | Closed `FeatureKind` enum, about 75 variants, no plugin variant | `crates/doc/src/document.rs:615-1880` (static count) |
| Render executors | Host-registered, duplicates rejected, reserved id | `render_jobs.rs:507`; `backend.rs:340` |
| Import/export formats | Closed set in `crates/io` and `crates/step` | `crates/io/src/lib.rs` (not traced in depth) |
| Materials and appearance | Library import with exact revisions; packs | `libraries.rs:1-8`; `appearance_pack.rs:1` |
| Themes and icons | Icon names from code; plugins may declare icon ids in the design only | `docs/plugins.md` "Icons and metadata" |

**5. Transactions, undo, jobs, persistence, output.**
- Transactions: `Session::execute` takes a `Snapshot` (document, active sketch, pending moves, visibility) before running (`lib.rs:1630-1640`). Refusal leaves the session unchanged (`lib.rs:1625-1630` comment). Undo stack at `lib.rs:1721`.
- Recovery: crash and command scope (`crates/engine/src/crash.rs`), recovery (`crates/engine/src/recovery.rs`). Only the names and the call sites were read.
- Format: `"format": "solidcraft/N"`. Older files upgrade step by step; newer files are refused (`crates/doc/src/format.rs:1-8`). Documents load through `Document::from_json` with a JSON budget and a feature cap (`crates/doc/src/document.rs:3798-3808`).
- Jobs: render jobs with owner, phase and host (`crates/engine/src/render_jobs.rs:1-6`; `crates/render/src/job/mod.rs` `ImageExecutor` at `:181`, `ImageWork` at `:162`). Cancellation and progress are in the render job API (names only read).
- Output: CLI `render`, `snapshot`, `script` and `run` subcommands (`apps/solidcraft-cli/src/main.rs:167-195`).
- Plugin references in files: none. The design doc proposes `PluginDisplay` stored in the file and a `fallback_policy` (`docs/plugins.md`, "Generic feature shell" and "Missing plugin and API drift").

**6. Collision and core-override behaviour.**

| Rule | Status | Evidence |
|---|---|---|
| Render executor id collides with an existing one | ENFORCED (`DuplicateExecutor`) | `crates/render/src/backend.rs:266`, `:340` |
| `cpu-raytrace` reserved for the native raytrace integrator | ENFORCED | `crates/engine/src/render_jobs.rs:470`; `lib.rs:1009` |
| Same host registered twice | ENFORCED (no-op) | `render_jobs.rs` `same_host` check |
| Command id duplicate | Test only | `crates/engine/src/tests.rs:1162` |
| Command lookup | First-match, then case-insensitive | `cmd/mod.rs:375-378` |
| Feature type from a plugin | NO-SEAM (no plugin variant) | `document.rs:615` |
| Plugin `run` names colliding with built-ins | NO-SEAM today; design says prefix `plugin_id.function` | `docs/plugins.md` "Command naming" |
| Plugin manifest id duplicate | Not enforced; GUI lists both | `ui-egui/src/plugins.rs:100` (scan) |
| Default handler or file association | NOT-FOUND | none |
| Shortcut or toolbar placement replacement | NO-SEAM (static `tab`/`panel`) | `cmd/mod.rs:143-170` |

**7. GUI/CLI parity.**
- The control server resolves `engine.execute` ids through `find_command` (`apps/solidcraft/src/control_server.rs:334`). The engine API is shared with the CLI.
- The CLI has `run`, `script`, `snapshot`, `render`, `exec`, `commands` (`apps/solidcraft-cli/src/main.rs:167-177`).
- A fresh CLI loads no plugin, because no plugin loader exists. The plugin page is GUI-only and does not load code.
- Hidden dialogs: not traced in the CLI path.

**8. Caller authority.**
- Control channel: a per-launch random token in an owner-only file, refused on the first request if missing (`crates/engine/src/control_auth.rs:1-7`; `apps/solidcraft/src/control_server.rs:225-234`, `:562`). `client_token` (`control_auth.rs:129`) finds it. `Listen::Exact` and port search (`control_auth.rs` header, `DEFAULT_PORT` 27878, `PORT_TRIES` 16).
- MCP and CLI authority: `agent_scope(...)` confines paths unless `--allow-any-path` (`apps/solidcraft-cli/src/main.rs:784-795`). Backends are headless, remote (`--connect` with `--token`) or in-process (`main.rs:811-828`).
- There is no per-caller command allowlist found. The in-app assistant has a permissions setting (`apps/solidcraft-cli/src/bench_assistant.rs:448-474`, a benchmark client, not a gate in the engine).

**9. Reusable APIs and missing hooks.**
- Reusable: `Session::execute`, `command_specs`/`find_command`, `CommandSpec` with its typed `contract` and `output` schema, `register_render_job_host`, `resolve_request`, `control_auth` token functions, `Document::from_json`/`to_json`, `format::read`, `libraries` import.
- PROPOSED missing hooks: a `FeatureKind::Plugin` variant with `plugin_id`, `api_version` and `fallback_policy` as in `docs/plugins.md`; a manifest-to-capability check; a WIT host with `run(command, args)` routed through `Session::execute` and `check_command`; a digest pin per plugin feature.

**10. App-specific data constraints.**
- Units: millimetres (`crates/geom/src/lib.rs:64`, "(mm)"). Coincidence `EPS` 1e-9; modelling "same point" `LINEAR_TOL` 1e-6 (`lib.rs:64-66`). Sketch end tolerance `END_TOL` 1e-5 (`crates/sketch/src/ends.rs:19`). Freeform offset `OFFSET_TOLERANCE` 0.002 (`crates/sketch/src/freeform_offset.rs:14`). Kernel boundary fit `FIT_TOLERANCE` 1e-4 (`crates/kernel/src/boundary_network.rs:28`).
- Kernel: truck crates (Apache-2.0) behind a `solidcraft-kernel` boundary, with `catch_unwind` around every third-party call (`crates/kernel/src/lib.rs:1-14`).
- Format size: `MAX_FEATURES` and `MAX_PARAMS` limits in `Document::from_json` (`document.rs:3798-3805`).
- Display: render budgets and job phases (`render_jobs.rs`).

**11. Smallest useful slice and proof (PROPOSED, future work).**
- Slice: `FeatureKind::Plugin` with a `recorded_mesh` fallback, read-only, with no execution. Design only; no plugin runs.
- Proof (future test): a design that contains a `Plugin` feature with an unknown `plugin_id` opens, shows the feature suppressed with the fallback message, and the rest of the model evaluates the same as without it. Mirrors the CI idea in `docs/plugins.md` ("uninstall plugin, reopen file, assert document still opens").
- Note: `FeatureKind` uses `#[serde(tag = "type")]` (`document.rs:614-615`), so an unknown variant currently fails deserialisation. That is the blocker the slice must solve; it is not yet verified how the format migration handles it.

**12. Evidence limits.** Not traced: `docs/control-protocol.md`, `docs/mcp.md` (listed only); the kernel operations; `crates/io` and `crates/step` format internals; render job cancel and progress code; the in-app assistant's permission model; `crates/mcp` tool table; remote owner (not read by policy). No runtime behaviour is claimed.

---

## TextureCraft (`/Users/ash/Projects/texturecraft`)

**1. Repo, revision, workspace.** Branch `main`. HEAD `9dc33d894e9cd8e25ce84f6074fd1b476dad5698` equals the baseline. Workspace (root `Cargo.toml`): default members `crates/core`, `texel`, `geom`, `compose`, `bake`, `formats`, `paint`, `engine`, `render`, `automation`, plus `apps/*`. Dependencies include `glam`, `image`, `wgpu = "30"`, `mikktspace`, and a pinned fork of egui (`Cargo.toml`). Licence not checked here.

**2. Operation surface.**
- Registry: `command_specs()` (`crates/engine/src/commands.rs:50-85`) concatenates 29 module lists (`view_cmds`, `layer_cmds`, `layer_ops`, `instance_cmds`, `mask_cmds`, `effect_clipboard`, `tool_cmds`, `brush_preset_cmds`, `path_tool`, `camera_cmds`, `quick_mask`, `set_cmds`, `meshmap_cmds`, `uv_tile_cmds`, `template_cmds`, `project_cmds`, `auto_rebake`, `bake_cmds`, `materialx_cmds`, `file_cmds`, `preset_cmds`, `resource_cmds`, `material_lib`, `link_cmds`, `smart_masks`, `filter_lib`, `decal_cmds`, `text_cmds`).
- `CommandSpec` (`commands.rs:13-21`): `id`, `label`, `menu`, `shortcut`, `params`, `enabled`, `run`. There is no `undoable` or `journal` field; undo is handled by `Session::edit`.
- `find` is first-match (`commands.rs:87-89`). `execute` checks enablement and sets `merge_edits` for drags, then runs `run` (`commands.rs:93-100`). No `catch_unwind` in this function was found (the panic guard exists in `automation`, see section 5).
- Size (static): 162 `cmd!` call sites; about 96 matched single-line ids (multi-line invocations not counted). Exact count not produced.
- Traced operation, `effect.add.filter` (`crates/engine/src/mask_cmds.rs:366-377`, `add_filter`): reads `filter`, calls `filter_lib::build(id, p)` (`crates/engine/src/filter_lib.rs:88-125`), which looks up `find(id)` (`filter_lib.rs:65`), matches the id to a `Filter` variant, applies `p` with `apply`, and returns an `EffectKind`. `add_filter` then calls `push_effect` (`mask_cmds.rs:237`). The final layer edit was not traced beyond `push_effect`.
- Filter library: 31 `def(...)` entries (`filter_lib.rs:26`), including material finishes.

**3. Existing plugin, extension, script, theme and resource mechanisms.**
- No plugin runtime. The composition model is a layer stack, not a node graph: `LayerKind` = Paint, Fill, Group, Instance (`crates/core/src/layer.rs:81`); `EffectKind` (`layer.rs:925`, 13 variants) and `Filter` (`layer.rs:559`, 26 variants) are closed enums. No node graph was found: a grep for node-graph types in `core`, `compose` and `engine` returned none. The only nodegraph code is the MaterialX reader (`crates/formats/src/materialx_read.rs:11`, `:199-255`), an import path.
- MaterialCraft integration: `.mcmat` materials become fill layers with a `MaterialLink` (`crates/core/src/layer.rs:384-390`) holding `package`, `graph`, `params`, `presets`, `inputs` and `size`. Cooking runs an external executable: `materialcraft_tool` looks at an environment variable, then next to the program, then `PATH` (`crates/engine/src/link_cmds.rs:209-215`). A `quiet` process builder (`link_cmds.rs:221`).
- Resources: presets (`crates/formats/src/presets.rs`), materials library (`crates/engine/src/material_lib.rs`, `materials.rs`), smart masks (`smart_masks.rs`), filter library (`filter_lib.rs`), fonts (`crates/formats/src/fonts.rs`), ICC (`crates/formats/src/icc.rs`), MaterialX import (`materialx_cmds.rs:1-6`), mesh import for OBJ, FBX, glTF, USD, Alembic, STL, PLY and DAE (`crates/formats/src/mesh_import/mod.rs`). `external.rs` folds sidecar files into a self-contained mesh (`crates/formats/src/mesh_import/external.rs:1-5`).
- Scripts: none found. Themes: not found.

**4. Contribution families.**

| Family | Seam today | Evidence |
|---|---|---|
| Commands | Static module lists | `commands.rs:50-85` |
| Effects and filters | Closed enums and a `const` library table | `layer.rs:559`, `:925`; `filter_lib.rs:26` |
| Generators and procedurals | Closed enums | `layer.rs:204`, `:485` |
| Materials | Library plus `.mcmat` link | `layer.rs:384-390`; `link_cmds.rs:209` |
| Mesh and texture formats | Closed importers | `formats/src/mesh_import/mod.rs` |
| Themes | Not found | none |

**5. Transactions, undo, jobs, persistence, output.**
- Transactions: `Session::edit(label, f)` and `edit_merging` (`crates/engine/src/session.rs:486-492`). "On error the document is left untouched" (`session.rs:485` doc). Undo queries at `session.rs:605` and `:625`.
- Panic guards: automation wraps RPC calls in `catch_unwind` (`crates/automation/src/rpc.rs:137`), and GPU calls (`headless.rs:321`, `:349`). The engine `execute` itself has no guard (see section 2).
- Persistence: project file with `FORMAT_VERSION` 1 (`crates/core/src/project.rs:9`) and binary pixel and skew files with their own versions (`crates/formats/src/project_file/mod.rs:9-36`). Maximum resolution 8192 (`crates/core/src/project.rs:341`). Plugin references: none. Material package reference is self-contained through an image resource (`layer.rs:384-390`).
- Jobs: auto-rebake (`crates/engine/src/auto_rebake.rs`), bake commands (`bake_cmds.rs`). The sync and async split was not traced.
- Output: exports through `formats::export`, and mesh export with USD (`crates/formats/src/mesh_export/usd.rs`). The CLI `texturecraft-cli` serves automation (`apps/texturecraft-cli/src/main.rs:198`).
- GPU: `wgpu` used for bakes and path tracing (`crates/render/src/gpu_bake.rs:66-79`; `crates/render/src/headless.rs:13`). A CPU path exists: "compute shaders, otherwise on the CPU (no GPU needed)" (`crates/automation/src/server.rs:224`). Composition is CPU (`crates/compose/src/lib.rs:1-6`).

**6. Collision and core-override behaviour.**

| Rule | Status | Evidence |
|---|---|---|
| Duplicate command id | First-match; no uniqueness check found | `commands.rs:87-89` |
| Filter id versus label | Lookup normalises case and separators and accepts labels; first match wins | `filter_lib.rs:65-68` |
| Duplicate filter id | Not checked; the table is a `const` slice | `filter_lib.rs:26` |
| Plugin-contributed filter or effect | NO-SEAM (closed enums) | `layer.rs:559`, `:925` |
| Shortcut duplicate | Not checked in inspected files | `commands.rs:13-21` |
| Default handler | NOT-FOUND | none |
| Material package name collision | Not traced | `link_cmds.rs` |

**7. GUI/CLI parity.**
- CLI and automation share `commands::execute` through `Headless::command_run` (`crates/automation/src/headless.rs:143-148`). `command.list` and `command.run` RPC methods (`crates/automation/src/rpc.rs:84-85`).
- The desktop control channel is in `apps/texturecraft/src/control_server.rs`. Its dispatch target in the UI was not traced.
- No plugin loads in any mode. The CLI runs with the trusted local filesystem (`apps/texturecraft-cli/src/main.rs:198`).

**8. Caller authority.**
- Two filesystem tiers in `Headless`: `Denied`, `TrustedLocal` and read or write roots (`crates/automation/src/headless.rs:18-22`). The trusted tier is set only by the CLI (`main.rs:198`).
- `authorize_command` refuses host-path parameters for untrusted callers, with an explicit allow list (`crates/automation/src/workspace.rs:136-150`). Path reads and writes use `cap-std` with a held directory handle (`workspace.rs:1-6`).
- Control channel: `TEXTURECRAFT_CONTROL_TOKEN` or a token file, `auth` method, bounded connections (`apps/texturecraft/src/control_server.rs:1-5`; `crates/automation/src/security.rs:15-55`).
- A trusted caller is still one principal. No identity beyond the trust tier.

**9. Reusable APIs and missing hooks.**
- Reusable: `commands::execute`, `command_specs`, `Session::edit`, `filter_lib::find`/`build`, `formats::presets`, `formats::mesh_import`, the `Headless` tiers and `authorize_command`, `cap-std` workspace.
- PROPOSED missing hooks: a namespaced command contribution with duplicate rejection; a plugin filter kind (today a closed enum in `layer.rs:559`), which needs a pixel-kernel contract with the tile model; a package verifier for `.mcmat`-style packages; a caller-scoped facade over `Headless::command_run`.

**10. App-specific data constraints.**
- Colour: linear RGB inside; sRGB encoding on display and export (`crates/core/src/channel.rs:47`). Storage formats: `Srgb8`, `L8`, `Rgb8`, `L16`, `Rgb16`, `L16F`, `Rgb16F`, `L32F`, `Rgb32F` (`channel.rs:235-244`). Image resource flag: 8- and 16-bit are sRGB, float is linear (`crates/formats/src/image_res.rs:12`).
- Resolution: `MAX_RESOLUTION` 8192 (`crates/core/src/project.rs:341`). Mesh file cap `MAX_MESH_FILE_BYTES` and texture cap `MAX_TEXTURE_BYTES` 256 MiB (`crates/formats/src/mesh_import/mod.rs:60`).
- Tiles: `TileGrid` and UDIM tiles (`crates/core/src/project.rs`, `uv_tile_cmds.rs`). Texture-set tiling evaluated per tile in parallel with rayon (`crates/compose/src/eval.rs:1-5`).
- Normal format: `NormalFormat` (`commands.rs:6` import).

**11. Smallest useful slice and proof (PROPOSED, future work).**
- Slice: read-only preset or material-library pack through `formats::presets` and the material library, with ids namespaced by package and no change to the filter enum.
- Proof (future test): a package with a `MaterialLink`-style reference whose version or digest does not match fails to open, and the project still opens without the package.
- Not proposed for the first slice: plugin filters or pixel kernels, which need a tile-level contract that does not exist.

**12. Evidence limits.** Not traced: the desktop UI dispatch of control requests; the auto-rebake and bake job model; the `formats::export` writers; the MaterialCraft cooking path beyond `link_cmds.rs:209-221`; the node-graph absence claim covers `core`, `compose`, `engine`, `formats`, and only grep for the rest. No runtime behaviour is claimed.

---

## WordCraft (`/Users/ash/Projects/wordcraft`)

**1. Repo, revision, workspace.** Branch `main`. HEAD `ffb155a0321bba08da495efda66495104c1c8b9a` equals the baseline. Workspace (root `Cargo.toml`): `crates/*`, `apps/*`, `xtask`; default members `crates/*`, `apps/wordcraft`, `apps/wordcraft-cli`, `xtask`. Crates include control-key, doc, docbin, docx, engine, fonts, formats, geom, layout, mcp, metafile, pdf, render, ui-egui, zotero. Shared craft-libs pins for the font store and PDF (`Cargo.toml:31-32`, `:69`). WOFF and WOFF2 support with `forbid(unsafe_code)` (`Cargo.toml:60-65`).

**2. Operation surface.**
- Registry: `registry()` (`crates/engine/src/cmd.rs:62-…`) concatenates 45 `specs()` functions. `CommandSpec` (`crates/engine/src/lib.rs:98-111`): `id`, `label`, `location`, `shortcut`, `params`, `mutates`, `enabled`, `run`. `Registry` (`lib.rs:141-172`): `specs: Vec`, `by_id: HashMap`.
- Size (static): 651 `CommandSpec::new` sites, 651 distinct ids (static). Exact count not produced.
- Traced operation, `text.insert` path via `Session::run` (`crates/engine/src/session/run.rs:55-200`):
  1. Look up the id: `reg.get(id)` (`run.rs:73`), `Unknown` if missing.
  2. Check `enabled` (`run.rs:77`), check protected regions for mutating commands (`crate::cmd::protect::check_before`, `run.rs:81`).
  3. Take snapshots (document, selection, pending formatting, math state) for mutating commands (`run.rs:107-136`).
  4. For mutating commands, `checkpoint(label)` (`session/history.rs:7-29`), which pushes `Undo {label, doc, sel, pending, column}` and clears redo (`history.rs:7-29`).
  5. Run the handler under `catch_unwind` (`run.rs:188`); a panic becomes `CmdError::Failed`.
  6. The handler edits the document through `Pos`-addressed helpers (`cmd.rs` `type_text`, `paste_text`, `split_para`).
  7. On error or cross-document change, `finish_elsewhere` restores (`run.rs:~200+`).
- Categories: text, caret, column, edit, paste, format, font, phonetic, para, lists, inspector, view, citations, fields, equations, charts, connectors, controls, draw, design, file, find, review, index, print, accessibility, tables, smart art, word art.

**3. Existing plugin, extension, script, theme and resource mechanisms.**
- No plug-in runtime. Customisable ribbon and quick access toolbar (`crates/engine/src/ribbon.rs:1-8`): tabs, custom groups and QAT entries hold any command; unknown ids and duplicates are dropped on read.
- Citations: style and bibliography engine with a `Order` enum (`crates/engine/src/cmd/citations/styles.rs:1-6`). The Zotero bridge (`crates/zotero/src/lib.rs:1-8`) lets Zotero drive the document over its wire protocol at `127.0.0.1:23116`.
- Fonts: shared craft-font-store and craft-font-catalog (`Cargo.toml:31-32`). Font selection and catalogue use the shared crates.
- Math: LaTeX and linear math in the document model (`crates/doc/src/math_latex.rs`, `math_linear.rs`).
- Default app: `apps/wordcraft/src/default_apps.rs:1-12`. The installers register types, and the user chooses the default from File, Options, General. `xdg-mime` is run on Linux (`default_apps.rs:118`).
- Scripts, themes, plugins: none found.

**4. Contribution families.**

| Family | Seam today | Evidence |
|---|---|---|
| Commands | Static registry | `cmd.rs:62`; `lib.rs:141-172` |
| Ribbon placement | User layout for any command id | `ribbon.rs:1-8` |
| Citation styles | Built-in enum | `cmd/citations/styles.rs:1-6` |
| Citation sources | Zotero bridge (external caller) | `crates/zotero/src/lib.rs:1-8` |
| Fonts | Shared store and catalogue | `Cargo.toml:31-32` |
| Import and export formats | Closed set (`docx`, `docbin`, `pdf`, `metafile`, `formats`) | crate list |
| Themes | Not found | none |

**5. Transactions, undo, jobs, persistence, output.**
- Transactions: `Session::run` is the single mutating entry point (`session/run.rs:55-200`). Each mutating command makes one checkpoint (`history.rs:7-29`). The undo cap is 500 (`crates/engine/src/doc_state.rs:179`). `undo` and `redo` (`history.rs:178`, `:210`). Join rules for typing and equations (`run.rs:60-76`).
- Document model: `Paragraph { text: String, runs: Vec<Run> }` (`crates/doc/src/para.rs:28-30`). Positions: `Pos { story, path, off }` (`crates/doc/src/position.rs:60-63`). `off` is a byte offset that must fall on a character boundary (`para.rs:171-175`; `crates/doc/src/cluster.rs:52`). `Path` is `Vec<u32>` (block, then row, cell and block per table level).
- Persistence: `.docx`, `.docbin`, ODT and RTF importers and exporters (crate list). Recovery: `crates/engine/src/recovery/desktop.rs` (only the event type was read).
- Jobs: none found in the command path. Print (`apps/wordcraft/src/print.rs`) and PDF (`crates/pdf`) not traced for progress.
- Output: CLI `automation_args` for roots (`apps/wordcraft-cli/src/automation_args.rs:7-24`).

**6. Collision and core-override behaviour.**

| Rule | Status | Evidence |
|---|---|---|
| Duplicate command id | Last-wins in `by_id`; list keeps both | `lib.rs:147-152` |
| Duplicate id, uniqueness test | Test only | `tests.rs:131-137` |
| Shortcut lookup | First match in spec order | `lib.rs:170-172` |
| Ribbon or QAT entry for an unknown id | Dropped on read | `ribbon.rs:1-8` |
| Plugin-contributed command | NO-SEAM | `cmd.rs:62` |
| Default app replacement | User-initiated only; installers do not take defaults | `default_apps.rs:1-12` |
| Duplicate ribbon group or tab | Dropped on read ("duplicates ... are dropped") | `ribbon.rs:1-8` |

**7. GUI/CLI parity.**
- The Zotero bridge calls `s.run(id, &json!({}))` (`crates/zotero/src/bridge.rs:375`, `:461`), the same entry point as the GUI. It takes its own undo label (`bridge.rs:174`, `s.checkpoint("Zotero")`).
- Desktop control server: authenticated by key (`apps/wordcraft/src/control_server.rs:36-40`, `:106`, `:178`, `:226`). Its dispatch into `Session::run` was not traced.
- MCP client: key handling and one retry on unauthorised (`crates/mcp/src/backend.rs:242-262`).
- No plugin loads in any mode.

**8. Caller authority.**
- Control channel: a key in the settings folder (`apps/wordcraft/src/control_server.rs:36-72`); each request must carry it (`:226`, `:310`, `:320`). Single authority level.
- Zotero: an external process gets the same command surface as the user, through `Session::run`. No separate scope was found.
- Filesystem roots: `--automation-read-root` and `--automation-write-root` (`apps/wordcraft-cli/src/automation_args.rs:7-24`). Enforcement of these roots was not traced in the engine.
- No caller identity at `Session::run` (`session/run.rs:55`).

**9. Reusable APIs and missing hooks.**
- Reusable: `Session::run`, `Registry::get`/`all`/`describe`/`by_shortcut`, `Session::checkpoint`, `push_undo`, `undo`, `redo`, the `Pos` and `Path` model, `ribbon` layout types, the Zotero bridge as a template for an external caller.
- Missing (PROPOSED): a fail-closed `Registry::new` (reject duplicates; the current last-wins behaviour is the first fix); a namespaced contribution id rule; a caller-scoped dispatch that separates Zotero, MCP and plugin from the user; a text-resource importer for style or template packs.

**10. App-specific data constraints.**
- Text positions: byte offsets on char boundaries (`para.rs:171-175`); grapheme clusters for caret motion (`crates/doc/src/cluster.rs:64`); bidi handling (`crates/doc/src/bidi.rs`).
- Equations: `MAX_EQUATION_TYPE_CHARS` 10,000 and `MAX_EQUATION_DELETE_UNITS` 64 (`crates/engine/src/lib.rs:64-68`).
- Undo: 500 steps (`doc_state.rs:179`), with an eviction counter (`history.rs:20-23`).
- Layout and pagination: `crates/layout`, not traced.
- Font loading and WOFF decoding limits: not traced.

**11. Smallest useful slice and proof (PROPOSED, future work).**
- Core fix first (not a plugin feature): `Registry::new` rejects a duplicate id instead of overwriting it. Proof (future test): constructing a registry with two specs of the same id returns an error and `get` does not silently change. The existing `every_command_has_unique_id` test (`tests.rs:131-137`) is a precedent.
- Then a text-resource pack (citation style or template) with ids namespaced by package. Proof (future test): a pack that names a built-in command id is refused and the ribbon drops it.

**12. Evidence limits.** Not traced: the desktop control dispatch into `Session::run`; the MCP tool table; layout and pagination; PDF and print progress; the engine-side enforcement of `--automation-read-root`; the recovery module beyond its event type; citation style data. No runtime behaviour is claimed.

---

## GridCraft (`/Users/ash/Projects/gridcraft`)

**1. Repo, revision, workspace.** Branch `main`. HEAD `178f3eb745a589c63b61ef97dd7b6b2d6fd48515` equals the baseline. Workspace (root `Cargo.toml`): `crates/*`, `apps/*`, `xtask`; default members include `apps/gridcraft`, `apps/gridcraft-cli`. Crates include calc, chart, core, engine, formula, functions, math, mcp, model, numfmt, pdf, ui-egui, xlsx. Dependencies include `zip` (`Cargo.toml:44`) and a password-protected workbook decryptor (`Cargo.toml:46`).

**2. Operation surface.**
- Registry: `command_specs()` (`crates/engine/src/cmd.rs:159-189`, OnceLock with `v.extend(...)` per area). `find_command` (`cmd.rs:190-197`): first match in `command_specs()`, then a fallback to `test_commands()`. `CommandSpec` (`cmd.rs:40-56`): `id`, `label`, `menu`, `shortcut`, `shortcut_mac`, `params`, `enabled`, `run`, `journal`, `undoable`.
- Size (static): 371 `cmd!` call sites; about 130 matched single-line ids (multi-line invocations not counted). Exact count not produced. Formula library: about 316 distinct `f!` names (`crates/functions/src`), plus about 27 host special-form arms (`crates/calc/src/eval/call.rs`).
- Traced operation, a formula call `=SUM(A1:A3)` through the calc engine:
  1. `Evaluator::call(name, args)` (`crates/calc/src/eval/call.rs:8-9`) matches host special forms first (`IF`, `IFS`, `LET`, `LAMBDA`, `MAP`, `GROUPBY`, …).
  2. `SUM` is not a special form, so the default arm calls `self.builtin(name, args)` (`call.rs:131`).
  3. `builtin` calls `gridcraft_functions::lookup(name)` (`call.rs:287-288`). Lookup uppercases, strips `_XLFN.` and `_XLWS.`, and reads the compile-time registry (`crates/functions/src/lib.rs:235-249`).
  4. Arity is checked against `FnSpec.min_args`/`max_args` (`call.rs:~292`). `Agg::of(spec.name)` routes aggregates (`call.rs:~295`).
  5. `functions::call` applies array lifting (`crates/functions/src/lib.rs:256-261`).
  6. The result returns to the recalc pass.
- Engine entry: `Session::execute(id, params)` (`crates/engine/src/lib.rs:667-…`), "Panics inside commands are caught and reported; the workbook is restored" (`lib.rs:664-666`).
- Categories: file, edit, format, data (sort, filter, validation, grouping), pivot, insert, chart, print, review, macros, python cells (stored only), sheet tabs, formulas, forecast, equation, inspect.

**3. Existing plugin, extension, script, theme and resource mechanisms.**
- No plugin runtime. Macros: typed command lists that run as one undo step (`crates/engine/src/cmd/macros.rs:1-6`; model `MAX_MACRO_STEPS` 10,000 at `crates/model/src/macros.rs:11`). The macro policy is the most complete authority model in the group (section 8).
- Python cells: this build stores code and cached results and does not run Python. A changed cell shows `#BLOCKED!` (`crates/engine/src/cmd/python.rs:1-4`).
- Automate scripts (`automate.*`): an older feature with its own storage (`cmd/macros.rs:1-6`). `ScriptSandbox::enter()` is used for script runs (`crates/engine/src/cmd/extra/scripts.rs:110`; `crates/engine/src/io.rs:758-761`). A preserved VBA project is reported and carried on file save (`cmd/extra/scripts.rs:43`), not executed.
- Resources: workbook I/O for XLSX and CSV (`crates/xlsx`, `crates/engine/src/io.rs:364-402`), charts (`crates/chart`), PDF output (`crates/pdf`), number formats (`crates/numfmt`).
- Themes: not found.

**4. Contribution families.**

| Family | Seam today | Evidence |
|---|---|---|
| Commands | Static lists plus a public, doc-hidden runtime registrar | `cmd.rs:159-189`; `cmd.rs:202-205` |
| Worksheet functions | Compile-time `registry()` of `FnSpec`s; no runtime seam | `functions/src/lib.rs:203-233` |
| Defined names and LAMBDA | Workbook-level and LET-bound lambdas | `call.rs:287-296`; `formulas.rs:349` |
| Macros | Stored command lists with a policy | `macros/policy.rs:13-30` |
| Python cells | Not executed in this build | `cmd/python.rs:1-4` |
| Import and export formats | Closed (XLSX, CSV, HTML and others) | `crates/engine/src/io.rs` |
| Themes | Not found | none |

**5. Transactions, undo, jobs, persistence, output.**
- Transactions: `Session::execute` records an undo step when the workbook changed, unless an undo group is open (`lib.rs:664-667` doc). Undo groups and macro debugger locks (`lib.rs:668-680`). Panics are caught and the workbook restored.
- Recalculation: dependency graph of formula cells to static precedents, a reverse index, a dirty closure evaluated in topological order, and on-demand evaluation for dynamic references (`crates/calc/src/recalc.rs:1-11`; `crates/calc/src/recalc/graph.rs:172`, `:289`). Cycles give `#CIRC!` unless iterative calculation is on (`recalc.rs:1-11`).
- Limits: `MAX_LAMBDA_CALLS` 140 per formula, counted in progress (`crates/calc/src/eval.rs:73-76`); native-stack and nesting depth limits (`eval.rs:70-78`; `recalc/pass_host.rs:32`).
- Persistence: XLSX and CSV through `io::save_bytes` and `save_bytes_as` (`crates/engine/src/io.rs:364-402`). Preserved content (VBA, OLE, form controls) is kept with the original file (`cmd/sheet/tabs.rs:169`). No plugin reference in files.
- Jobs: none found in calc or engine. No progress or cancel model. Long calculations are synchronous.
- Output: CLI `check` for workbook fidelity and calculation oracle (`apps/gridcraft-cli/src/check.rs:363-364`).

**6. Collision and core-override behaviour.**

| Rule | Status | Evidence |
|---|---|---|
| Built-in function name vs defined name in a call | ENFORCED by dispatch order | `call.rs:124-131`; `call.rs:287-296` |
| Built-in special form vs library | ENFORCED (special forms matched first) | `call.rs:8-9`; `special.rs:88` |
| Duplicate function name in the library | First-wins silently; test catches it | `functions/src/lib.rs:225-228`; `lib.rs:267-272` |
| Defined name equal to a function name | Accepted by validity rule; not refused | `formulas.rs:323-333` |
| LET variable equal to a function name | Built-in still wins for call syntax (from dispatch order); not traced in full | `call.rs:124-131` |
| `_XLUDF.` user-defined names | Never resolved as built-ins (lookup strips only `_XLFN.` and `_XLWS.`) | `functions/src/lib.rs:233-249` |
| Duplicate command id | Built-in first; test-only check | `cmd.rs:190-197` |
| Runtime command registration (`register_test_command`) | Public, doc-hidden, not `cfg(test)`-gated; appends after built-ins; no namespace rule | `cmd.rs:202-205` (`:223` shows a separate `cfg(test)` helper) |
| Shortcut duplicates | Not checked in inspected files | `cmd.rs:40-56` |

**7. GUI/CLI parity.**
- The desktop control channel is token-authenticated (`apps/gridcraft/src/control_server.rs:1-4`, `:57-64`). Its dispatch target was not traced.
- The MCP crate has a headless backend that calls `session.execute_headless_in(target, id, params)` (`crates/mcp/src/headless.rs:36-39`), which says "Programmatic calls never open dialogs; anything a command queued for a UI is dropped."
- `commands` and `list_commands` list the command table (`crates/mcp/src/headless.rs:41-`).
- No plugin loads in any mode. The CLI `check` command works on files without a session.

**8. Caller authority.**
- Macro policy (`crates/engine/src/macros/policy.rs:1-8`): macros may not open, save, close or create workbooks (`file.`), change the app (`app.`), switch windows (`window.`), or run other macros or scripts (`automate.`). Commands that read files need the macro's declared permission. Exports that write files need approval. A workbook from a file is untrusted (`Trust::FromFile`, `crates/engine/src/lib.rs:156-170`), so macro steps `Ask` before running (`policy.rs:62-84`).
- Control channel: a token is required on the first line (`apps/gridcraft/src/control_server.rs:4`). One authority.
- MCP: headless target selection and "no dialogs" (`crates/mcp/src/headless.rs:36-39`). No caller identity beyond the connection.
- Path confinement for MCP file commands was not traced.

**9. Reusable APIs and missing hooks.**
- Reusable: `functions::lookup`, `functions::call`, `FnSpec`, the calc `Evaluator` and `Host` traits, `macros::policy::macro_policy` and its `Policy` enum, `Trust`, `Session::execute`, `command_specs`, and the recalc graph.
- Missing (PROPOSED): a namespaced function registry for packages, consulted after the built-in library and special forms; a policy-style allow list for any future plug-in command family, modelled on `macro_policy`; removal or feature-gating of `register_test_command` from production builds; a refusal rule for defined names equal to built-in function names (Excel allows some of these, so this needs a product decision).

**10. App-specific data constraints.**
- Values: `Value::Number(f64)` (`crates/core/src/value.rs:169-172`); `CellError` (`value.rs:11`). Date systems 1900 and 1904 (`crates/core/src/date.rs:10`).
- Sheet limits: 1,048,576 rows and 16,384 columns (`crates/core/src/addr.rs:10-11`).
- Calc limits: `MAX_LAMBDA_CALLS` 140 (`eval.rs:76`); `MAX_DEPTH` and native-stack budget (`eval.rs:70-78`).
- Volatile functions: `host.note_volatile()` (`call.rs:129-130`).

**11. Smallest useful slice and proof (PROPOSED, future work).**
- Core hardening first: gate `register_test_command` behind `cfg(test)` or a test feature. Proof (future test): a release-profile build does not export the symbol, and `find_command` behaviour for built-ins is unchanged.
- Then a read-only function-pack proposal: names prefixed by the package namespace, consulted after the built-in library. Proof (future test): a package defining `SUM` leaves `=SUM(1,2)` at 3, and a package function `PKG.F` resolves only when the package is granted.
- High risk, not proposed for the first slice: plugin code that evaluates cells. It would need a sandbox budget that the calc engine does not have.

**12. Evidence limits.** Not traced: the desktop control dispatch target; the XLSX reader and writer internals; the macro runner and debugger beyond the policy; charts and PDF; the MCP tool table; the LET-variable shadowing case in full (only dispatch order was read); `Excel` compatibility of name rules (not claimed). No runtime behaviour is claimed.

---

## Evidence limits (files read per app)

Every file below was read through `git show <REV>:<path>`, `git ls-tree -r --name-only <REV>`, or `git grep -n <pattern> <REV> -- <paths>` at the stated HEAD. Lines marked "header" were read at the top of the file. Lines marked "grep" were located by search only. No working-tree file was read.

**SoundCraft (`da02c19fcca8fbcc49d66c957e5efcc67a63befe`)**
- Root: `Cargo.toml`, `AGENTS.md`, `docs/architecture.md` (lines 35-75, grep)
- engine: `crates/engine/src/cmd.rs`, `lib.rs` (lines 28-40, 60-75, 176-200, 270-285, 330-470, 589-640), `cmd/clap.rs`, `cmd/vst3.rs`, `cmd/mix.rs` (324-351, grep for insert), `cmd/file.rs` (14-103, 215, 236-340), `cmd/track_more.rs` (18, 248-276), `cmd/setup_more.rs` (91-100, 459-470), `cmd/plugin_state.rs` (grep), `cmd/instrument.rs` (grep), `cmd/audiosuite.rs` (grep), `file_access.rs` (1-50, 140-185), `io.rs` (160-300), `io/session.rs` (1-60), `io/bundle.rs` (grep), `io/import.rs` (grep), `autosave.rs` (grep)
- automation: `crates/automation/src/lib.rs` (1-12, 46-90), `mcp.rs` (1-30, 118-200), `args.rs` (grep)
- app: `apps/soundcraft/src/control_server.rs` (1-30, 107-150, grep), `launch_args.rs` (grep)
- ui: `crates/ui-egui/src/control.rs` (440-455, grep), `shortcuts.rs` (100-120), `mix_window/plugin_menu.rs` (1-40), `bounce.rs` (grep), `menus.rs` (grep), `lib.rs` (grep for theme)
- cli: `apps/soundcraft-cli/src/session.rs` (30-52), `args.rs` (grep), `mcp.rs` (grep)
- playback: `crates/playback/src/lib.rs` (1-25, grep), `callback.rs` (52-80, 128-165, 225-300, 396-425, 490-515, grep), `devices.rs` (grep)
- mix: `crates/mix/src/lib.rs` (grep, 255-300, 620-640), `threads.rs` (1-20, 60-80)
- dsp: `crates/dsp/src/plugins.rs` (1-54, 85-140)
- hosts: `crates/clap-host/src/lib.rs` (grep), `registry.rs` (54-80, 195-215), `scan.rs` (14-40), `Cargo.toml` (1-40, grep), `crates/vst3-host/src/lib.rs` (grep), `Cargo.toml` (grep), `crates/au-host/src/lib.rs` (grep)
- model: `crates/model/src/mixer.rs` (420-445), `session.rs` (grep), `source.rs` (grep)
- audio-io: `crates/audio-io/src/lib.rs` (55-62, 125-170, grep)

**CADCraft (`b04273e8974245e1c59ec57eb5ad86cec7a200eb`)**
- Root: `Cargo.toml` (1-30)
- engine: `crates/engine/src/cmd.rs` (1-100, 160-265), `cmd/lisp.rs` (all), `lisp.rs` (1-80, 85-100, 118-125, 170-200, 234-250, 290-345, 655-720), `lisp/startup.rs` (all), `lisp/drawing.rs` (60-130, grep), `lisp/eval.rs` (157-165, 215-220, 507-540, 597-612, grep), `lisp/builtins.rs` (268-282, grep), `lisp/builtins/files.rs` (1-30, 70-125), `lisp/value.rs` (40-50, 84-110, grep), `session/execute.rs` (1-115), `session/interactive.rs` (1-50), `session/cmdline.rs` (20-60, 160-200, grep), `cmd/draw.rs` (grep, 50-60), `cmd/draw/circle.rs` (1-30), `cmd/hatch/patfile.rs` (1-30, 57-95), `cmd/hatch/edit.rs` (150-160, grep), `cmd/plotters.rs` (grep), `autosave.rs` (grep), `atomic_write.rs` (1-12), `tests/aliases.rs` (33-67), `undoctl.rs` (grep)
- doc: `crates/doc/src/library.rs` (1-15, 81-100, 240-262), `library/files.rs` (grep), `header.rs` (grep), `plotstyle.rs` (grep)
- constraints: `crates/constraints/src/lib.rs` (1-30, 52, grep), `lm.rs` (grep), `expr.rs` (1-8, grep)
- geom: `crates/geom/src/lib.rs` (1-45), `vec.rs` (grep)
- render: `crates/render/src/hatch.rs` (grep)
- io: `crates/io/src/lib.rs` (grep), `dxf_read.rs` and `dxf_write.rs` (grep for AutoLISP mentions)
- mcp: `crates/mcp/src/tools.rs` (1-25), `backend.rs` (1-30, 190-280, grep), `control_token.rs` (grep), `server.rs` (grep)
- app: `apps/cadcraft/src/control_server.rs` (50-80, 280-335, grep), `apps/cadcraft/src/main.rs` (grep, line 385)
- cli: `apps/cadcraft-cli/src/run.rs` (1-30, 175-215), `commands.rs` (grep), `lib.rs` (grep), `args.rs` (grep)
- fonts: `crates/fonts/build.rs` (grep, 40-80)

**SolidCraft (`4bdffef06180fb3a2e8fbd4b15c7d3f60e55bd11`)**
- Root: `Cargo.toml` (1-30, grep), `docs/plugins.md` (1-146), `docs/mcp.md`, `docs/control-protocol.md`, `docs/web-runtime.md` (listed, not read)
- engine: `crates/engine/src/cmd/mod.rs` (1-30, 140-175, 305-320, 370-400), `lib.rs` (1000-1040, 1600-1660, 1715-1725, grep), `render_jobs.rs` (1-30, 460-525), `control_auth.rs` (1-30, 70-135, 280-310 grep), `cmd/features.rs` (30-40), `slicers.rs` (1-22, grep), `cmd/libraries.rs` (1-10), `tests.rs` (grep 1155-1165)
- doc: `crates/doc/src/document.rs` (600-625, 1870-1885, 3795-3815, grep), `format.rs` (1-40), `appearance_pack.rs` (1-8), `physical_material.rs` (grep), `format_offsets.rs` (grep)
- render: `crates/render/src/backend.rs` (grep, 255-300, 335-400), `job/mod.rs` (grep)
- geom: `crates/geom/src/lib.rs` (56-70, grep)
- kernel: `crates/kernel/src/lib.rs` (1-16), `boundary_network.rs` (grep), `fill.rs` (grep), `prism.rs` (grep)
- sketch: `crates/sketch/src/ends.rs` (grep), `freeform_offset.rs` (grep)
- smlib: `crates/smlib/src/flat.rs` (grep)
- ui: `crates/ui-egui/src/plugins.rs` (1-40, 95-110)
- app: `apps/solidcraft/src/control_server.rs` (grep), `apps/solidcraft-cli/src/main.rs` (grep 160-200, 780-830), `bench_assistant.rs` (grep)
- mcp: `crates/mcp/src` (listed)

**TextureCraft (`9dc33d894e9cd8e25ce84f6074fd1b476dad5698`)**
- Root: `Cargo.toml` (1-80, 100-125, grep)
- engine: `crates/engine/src/commands.rs` (1-135), `session.rs` (330-335, 385-395, 470-495, 600-640, grep), `filter_lib.rs` (1-30, 55-70, 86-125, 325-340), `mask_cmds.rs` (grep, 360-380), `link_cmds.rs` (1-10, 200-225), `materialx_cmds.rs` (1-8), `lib.rs` (grep), `project_cmds.rs` (grep), `materials.rs` (grep 150-160, 740-750), `preview.rs` (grep)
- core: `crates/core/src/layer.rs` (grep, 1-30, 81-100, 380-395, 555-565, 920-940, 1030-1040), `project.rs` (grep 1-15, 335-345), `channel.rs` (42-50, 230-262), `blend.rs` (grep)
- compose: `crates/compose/src/lib.rs` (1-30), `eval.rs` (1-20, grep), `display.rs` (grep), `store.rs` (grep)
- formats: `crates/formats/src/project_file/mod.rs` (1-40, grep), `image_res.rs` (1-20), `materialx_read.rs` (1-15, 195-260, 465-480, grep), `mesh_import/external.rs` (1-10), `mesh_import/mod.rs` (grep, 55-65), `presets.rs` (grep), `mcmat.rs` (grep)
- automation: `crates/automation/src/headless.rs` (1-30, 78-152, 315-355, grep), `workspace.rs` (1-15, 130-155), `security.rs` (1-20, 45-80, 105-115, 285-295), `rpc.rs` (75-95, grep 137), `server.rs` (grep 220-230, 285-295), `bridge.rs` (grep), `render.rs` (grep)
- render: `crates/render/src/gpu_bake.rs` (60-85, grep), `headless.rs` (grep)
- app: `apps/texturecraft/src/control_server.rs` (1-35, grep), `apps/texturecraft-cli/src/main.rs` (grep 190-205)

**WordCraft (`ffb155a0321bba08da495efda66495104c1c8b9a`)**
- Root: `Cargo.toml` (1-35, 55-70, 130-150)
- engine: `crates/engine/src/lib.rs` (40-70, 94-180), `cmd.rs` (60-120, grep), `session/run.rs` (1-20, 50-80, 100-140, 160-205, grep), `session/history.rs` (1-35, 140-160, 175-215), `doc_state.rs` (grep 175-185), `ribbon.rs` (1-16), `catalog.rs` (grep), `tests.rs` (grep 128-140), `cmd/citations/styles.rs` (1-8), `cmd/citations/render.rs` (grep), `session/ime.rs` (grep)
- doc: `crates/doc/src/position.rs` (1-45, 52-80), `para.rs` (20-35, 160-180, grep), `cluster.rs` (grep 45-70), `block.rs` (grep), `bidi.rs` (grep), `math_latex.rs` (grep), `math_linear.rs` (grep)
- zotero: `crates/zotero/src/lib.rs` (1-12), `bridge.rs` (grep 170-180, 370-380, 455-465), `client.rs` (grep 85-140)
- mcp: `crates/mcp/src/backend.rs` (grep 235-280)
- app: `apps/wordcraft/src/control_server.rs` (grep 30-75, 100-115, 175-230, 255-340), `apps/wordcraft/src/default_apps.rs` (1-30, grep 100-125)
- cli: `apps/wordcraft-cli/src/automation_args.rs` (1-30, grep)

**GridCraft (`178f3eb745a589c63b61ef97dd7b6b2d6fd48515`)**
- Root: `Cargo.toml` (1-50, grep)
- engine: `crates/engine/src/cmd.rs` (36-60, 155-250, grep), `lib.rs` (grep 150-175, 660-685, 1100-1110), `cmd/python.rs` (1-30), `cmd/macros.rs` (1-10), `cmd/extra/scripts.rs` (1-15, grep 105-115), `cmd/formulas.rs` (315-385), `cmd/sheet/tabs.rs` (grep 160-175), `cmd/analysis.rs` (grep), `cmd/edit/cell.rs` (grep), `macros/policy.rs` (1-30, 50-90, grep), `macros/mod.rs` (grep), `macros/run.rs` (grep), `macros/debug.rs` (grep 600-610), `io.rs` (grep 355-410, 750-770), `tests/macros.rs` (grep 455-465)
- calc: `crates/calc/src/eval/call.rs` (1-15, 90-140, 200-210, 280-320, 400-440, grep), `eval/special.rs` (80-100, 155-170), `eval.rs` (60-80, grep 130-140), `recalc.rs` (1-25, grep 800-810), `recalc/graph.rs` (grep 165-195, 285-310, 340-350), `recalc/pass_host.rs` (grep 15-40)
- functions: `crates/functions/src/lib.rs` (1-35, 195-275), file list of `crates/functions/src`
- core: `crates/core/src/value.rs` (grep 5-15, 165-175), `addr.rs` (grep 1-15), `date.rs` (grep)
- model: `crates/model/src/macros.rs` (grep 5-15, 70-80)
- mcp: `crates/mcp/src/headless.rs` (30-95), `crates/mcp/src` (listed)
- app: `apps/gridcraft/src/control_server.rs` (grep 1-70), `apps/gridcraft/src/main.rs` (grep 430-440)
- cli: `apps/gridcraft-cli/src/check.rs` (grep 355-370), `check/calc.rs` (grep 145-155)

**Not read by this worker (any app):** the working tree of every clone, all test bodies except where a grep matched them, all vendored code (SolidCraft `vendor/`), documentation files other than the ones named above, and all build outputs.

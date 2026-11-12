# Haiku research: app adapters, CLI sessions, jobs, persistence, and processing state

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: design research, 2026-11-08. Source inspection only. No build, test, fuzz run, app launch, benchmark, or fetch was performed. Every "observed" statement below is a reading of source at a named commit. None is runtime behaviour. Items marked **not traced** were outside the time and scope of this pass.

Scope: EffectCraft, FigCraft, PhotoCraft, and FilmCraft adapter semantics; fresh, attached, and server CLI sessions; output and jobs; transactions; persistent references, migrations, and missing modules; temporal, image, and audio processing; GPU and native limits; caches and scheduling. Themes are covered by the separate runtime note, organizations are excluded, and the third-party release gate stays closed. Nothing here changes an accepted decision.

Read first (and consistent with them): [plugin index](../README.md), [accepted decisions](../../plugin-functional-decisions.md), [security proposal](../../plugin-security-design.md), [primary functional study](../../plugin-functional-primary.md), [app source inventory](../../plugin-functional-inventory.json), and [runtime and theme note](../../plugin-security-runtime-themes.md).

## 1. Summary

1. **No app's fresh CLI session loads installed, enabled, compatible plugins.** The accepted default is unmet in every app that has a plugin runtime. Only the desktop start-up path loads them (EC; FG and PC use desktop-only prefs or explicit commands).
2. **Missing or unavailable contributions are dropped silently** at render time in EffectCraft and at smart-object re-render in PhotoCraft. No export preflight was found.
3. **FigCraft's headless `save_file` reports success and loses the file.** The request for a save dialog is queued and then cleared by the headless backend. There is no artifact-destination contract.
4. **Saved references are identifiers only.** EffectCraft stores an effect ID. PhotoCraft stores a plugin ID plus resolved parameters. FigCraft stores a plugin ID plus widget state. None records a version or digest.
5. **FigCraft re-reads disk-backed modules on every run** and checks them against no stored digest. Its installed record has no digest field (`plugins.rs:26-37`). EffectCraft verifies SHA-256 when it loads the library. PhotoCraft loads bytes once. FigCraft is the outlier.
6. **Three incompatible module ABIs exist:** EffectCraft `ec_*` (v1), FigCraft `figcraft_run` (ABI 4), and PhotoCraft `pc_*` (ABI 1). All use wasmi, and none has the same host surface. Adapters must keep each ABI, as the accepted defaults require.
7. **Temporal inputs exist only on the native API 2 and helper paths.** The sandboxed WASM path cannot declare API 2 parameter kinds such as `Layer`, and it receives none of the API 2 frame services.
8. **No plugin path uses the GPU.** Plugin effect specs are registered with `gpu: false`. The `--gpu` flag affects built-in effects. EffectCraft's external helpers are native child processes and are not sandboxed.
9. **FilmCraft has no processing plugin path** in its inspected CLI or top-level engine sources. Its only plugin setting is `developer_mode`. Its CLI already waits for `file.exportMedia` by default.
10. **Process-wide registries and same-ID replacement** shape lifecycle. EffectCraft effect slots are set once per process, so uninstall and update need a restart. PhotoCraft replaces a same-ID plugin silently. FigCraft reports `replaced: true` but deletes that plugin's settings.

## 2. Evidence base and method

| Key | Repository | Revision | Use |
|---|---|---|---|
| EC | [arnav-hiwarkar/effectcraft](https://github.com/arnav-hiwarkar/effectcraft/tree/ce9eaec65015a5508c72af0e2351cd02612c9450) | `ce9eaec65015a5508c72af0e2351cd02612c9450` | Host, engine, render, helper protocol, WASM ABI v1, CLI |
| FG | [storytold/figcraft](https://github.com/storytold/figcraft/tree/0e86f3597a7443b15c93ec3ab2091202f8802c95) | `0e86f3597a7443b15c93ec3ab2091202f8802c95` | Plugin policy, sandbox ABI 4, headless backend, CLI, widgets |
| PC | [storytold/photocraft](https://github.com/storytold/photocraft/tree/655295b2a2c400bded0640f5d97367000a057b2e) | `655295b2a2c400bded0640f5d97367000a057b2e` | Filter ABI 1, band application, registry, smart filters, CLI |
| FL | [storytold/filmcraft](https://github.com/storytold/filmcraft/tree/32749e34f611735460e9773335345db026b7b037) | `32749e34f611735460e9773335345db026b7b037` | CLI long-job handling, preferences |

Method. Sources were read with `git show <revision>:<path>` and `git grep <revision>`, from commit objects only. No checkout, fetch, or edit of any app repository was performed. Uncommitted working-tree changes were not used. Line numbers refer to the named revision.

Inventory check. I recomputed 15 recorded SHA-256 values from the inventory at the named revisions. All 15 matched: EC (7 files), FG (4 files), PC (3 files), and FL (1 file). Files cited below that are not in the inventory are read at the same revision but are **not hashed**.

Key files read: EC `apps/effectcraft/src/main.rs`, `apps/effectcraft-cli/src/main.rs`, `crates/host/src/lib.rs`, `crates/engine/src/lib.rs`, `crates/engine/src/system/library.rs`, `crates/engine/src/cmd/effect.rs`, `crates/engine/src/cmd/file_more.rs`, `crates/engine/src/session/edit.rs`, `crates/render/src/pipeline.rs`, `crates/effects/src/plugin.rs`, `crates/plugin/src/lib.rs`, `crates/plugin/src/wasm.rs`, `crates/helper-host/src/{protocol,client,blocklist}.rs`, `crates/project/src/props.rs`. FG `crates/plugin/src/sandbox.rs`, `crates/engine/src/extensions/{plugins,widgets,plugin_directory}.rs`, `crates/engine/src/session/{undo,new}.rs`, `crates/mcp/src/headless.rs`, `apps/figcraft-cli/src/{files,main,args}.rs`, `apps/figcraft/src/main.rs`. PC `crates/plugins/src/{apply,runtime,manifest,registry,lib}.rs`, `crates/engine/src/cmd/plugin_cmds.rs`, `crates/engine/src/prefs.rs`, `crates/engine/src/session/documents.rs`, `crates/automation/src/headless.rs`. FL `apps/filmcraft-cli/src/main.rs`, `crates/automation/src/long_job.rs`, `crates/engine/src/settings/prefs.rs`.

Limits. Each finding is a reading of one revision. Runtime checks, exit-status behaviour under error, and the web build were not exercised. Some paths were not traced (marked **not traced** below). External specifications (wasmi, WebGPU, TUF) were not re-checked in this pass.

## 3. Adapter semantics per app

| Aspect | EC (EffectCraft) | FG (FigCraft) | PC (PhotoCraft) | FL (FilmCraft) |
|---|---|---|---|---|
| Contribution kinds | WASM effect (ABI v1), helper effect (native, protocol v1) | Plugin with commands, panel, widget (ABI 4) | Filter, `Kind::Filter` (ABI 1) | None found |
| Unit of work | One effect instance per layer per frame: `time`, `scale`, layer size | One command call, panel callback, or widget callback | One tile-aligned band with an overlap margin; the layer is applied in full | — |
| Pixel contract | WASM: premultiplied RGBA `f32`, row 0 top (`crates/plugin/src/lib.rs:14-18`) | Document nodes; no pixel buffer | Normalised `f32` in the document's colour model; straight-alpha flag in bit 8 (`apply.rs:12-33`) | — |
| Temporal contract | `time` only in WASM v1. Native API 2 renders get `PluginHost` services (`input_at`, `layer_input`, `params_at`, `frame_rate`, `time`, `instance_key`). Helper renders reach them through Image and Params requests (`plugin.rs:275-300`; `protocol.rs:86-112`) | None | None | — |
| Host surface | WASM: no imports (`wasm.rs:71-72`). Helper: Hello, Describe, Load, Padding, Render, Image, Params, Destroy (`protocol.rs:86-112`) | 14 imports: `call`, `selection`, `log`, `notify`, `set_output`, `ui_show`, `setting_*`, `widget_render`, `save_file` (`sandbox.rs:49-64`) | No imports; a module that imports is refused (`runtime.rs:184-190`) | — |
| Mutation and undo | Effect output is a render-time result, not a document edit | Each call is a nested group. Failure rolls back (`undo.rs:277-278`). A call that opens UI is refused (`plugins.rs:247-249`) | Pixel layer: `s.edit`, surface replaced after success, transparency lock re-applied (`plugin_cmds.rs:113-130`) | — |
| Persistent reference | Effect spec ID only (`props.rs:183-184`) | Plugin ID, manifest, path, source, per-plugin settings (`plugins.rs:26-37`), widget `plugin_data` (`widgets.rs:255-261`) | Pixel layer: result baked, no reference. Smart filter: `{id, params: resolved}` (`plugin_cmds.rs:133-137`) | — |
| Missing module | EC silently omits the effect at render (`pipeline.rs:627-630`) | Callbacks fail with "no plugin ... is installed" (`plugins.rs:338-340`). Widget nodes persist | Smart filter silently skipped on re-render (`plugin_cmds.rs:149-158`). Pixel results unaffected | — |
| Fresh CLI | Host session without installed plugins (`apps/effectcraft/src/main.rs:279` is the only non-test `library.loadInstalled` caller) | Fresh `Session::new()` or `empty()` with default prefs (`session/new.rs:17`). Installed list read only from `s.prefs.plugins` | Registry empty until prefs apply or an explicit plugin command (`documents.rs:19-21`, `prefs.rs:1377`) | — |
| Output and jobs | `render --out`, progress lines on stderr. The CLI starts `renderQueue.render` with `wait: false`, then awaits it (`cli main.rs:919, 961`) | Result JSON. `save_file` offers a file that headless discards | Synchronous `plugin.run` | `file.exportMedia` awaited by default (`cli main.rs:201-208`) |
| GPU and native | `--gpu` for built-ins only. Plugin specs `gpu: false` (`plugin.rs:485`). Helpers are native child processes | CPU only | CPU; bands in parallel via rayon | — |
| State and cache | Helper `instance_key` and `Destroy` (`protocol.rs:106-107`). WASM instances are pooled and returned after each render (`wasm.rs:95-106, 158`). Linear memory and globals persist; no reset was found in the render path | Fresh `Instance` per call. A panel keeps its instance (`sandbox.rs:32`) | Fresh instance per band (`runtime.rs:280`). Registry revision for menu caches (`registry.rs:52-55`) | — |

## 4. Module ABI and resource limits (observed defaults)

| Runtime | Module size | Memory | Compute budget | Time | Other bounds |
|---|---|---|---|---|---|
| EC WASM v1 | **not traced** | pooled instance; memory persists across renders (`wasm.rs:95-106, 158`) | 50,000,000 + 20,000 per pixel (`wasm.rs:11-12, 46, 126`) | none | no imports (`wasm.rs:71-72`); API 2 kinds refused (`crates/plugin/src/lib.rs:86-88`) |
| EC helper (native) | 256 MiB control line (`protocol.rs:24`) | separate process | none beyond timeouts | heartbeat 10 s; render 120 s by default; load 600 s (`client.rs:30, 42-57`) | at most 2^28 pixels, i.e. up to 4 GiB exchange file per image (`protocol.rs:27, 30`); crash limit 3 (`blocklist.rs:16-17`) |
| FG ABI 4 | 16 MiB (`sandbox.rs:95`) | 64 MiB linear (`:94`) | 1e9 fuel; 10,000 per host call (`:92-93`) | none | 100,000 host calls; 16 MiB messages; 1,000 log lines, 1 MiB; one saved file of at most 16 MiB per run (`plugins.rs:208-210`) |
| PC ABI 1 | 32 MiB (`runtime.rs:47`) | 512 MiB per instance; 2 GiB across the run (`:48-49`) | 50,000,000 + 4,000 per sample; slices of 20,000,000 (`:17, 51-52`) | 60 s, native builds only (`:54`) | manifest 64 KiB; 64 parameters; overlap ≤ 256 px (`manifest.rs:9-13`); bands ≤ 65,536 (`apply.rs:74`) |

Observed consequence (PC). At defaults, `parallel_bands` is `min(threads, 2 GiB / 512 MiB) = 4` (`runtime.rs:63-66`). At most four band instances run concurrently, whatever the core count.

## 5. CLI session modes

| Mode | EC | FG | PC | FL |
|---|---|---|---|---|
| Fresh headless | `effectcraft-cli` builds the host session (`cli main.rs:470-490`). No installed-plugin load. `--gpu` is opt-in (`:483-488`) | `run` creates `Session::new()` or `empty()` with default prefs (`files.rs:409-420`). `plugin` subcommand has only `publish` (`args.rs:220-221`) | Session default. `plugin.reload` or `plugin.install` load explicitly (`plugin_cmds.rs:209, 253`). **Whether the CLI exposes plugin commands through `exec` was not traced** | `run` and `exec`. Long jobs awaited |
| Attached to running app | `--bridge PORT` drives `effectcraft --control PORT` (`cli main.rs:40`) | `mcp --attach ADDR` connects to an app started with `figcraft --control PORT` (`cli main.rs:151-161`) | **not traced** | `--bridge` polls an app job (`cli main.rs:211-212`) |
| Persistent server | `mcp` on stdio; `--autosave` and `--bridge` modes (`cli main.rs:34`) | MCP on stdio; headless session lives for the process (`cli main.rs:161`) | automation crate; plugin use not found | **not traced** |

Source differences that matter. The desktop start-up path loads installed plugins in EC and reads preferences in FG and PC. None of the fresh CLI paths does. The accepted fresh-CLI default therefore needs one bootstrap call per app and a shared resolver. It needs no new execution path.

## 6. Workflow traces: normal path and failure points

**WF-1 EffectCraft effect, desktop to headless render.** Normal: the desktop loads the package at start (`main.rs:279`) → the effect is registered (`plugin.rs:467-490`) → applied as a `GroupKind::Effect` with its ID → saved → headless render resolves the ID.
- Break A: the CLI never calls `library.loadInstalled`. The effect is unregistered in the headless process.
- Break B: `render` drops unresolved IDs (`pipeline.rs:627-630`). `render_queue.rs:471` checks only the composition. No effect check was found in the render queue, the CLI render command, or the export crate. The output omits the effect. Exit status under this condition was **not traced**.
- Break C: in an open desktop session, uninstall keeps the effect registered until restart (`library.rs:1057`). Update also loads at next start (`library.rs:517-520`).
- Result (source inspection): a headless export can succeed without the processing the author applied.

**WF-2 EffectCraft helper effect.** Normal: start helpers (opt-in by `EFFECTCRAFT_HELPERS=1` at launch, or the menu, `main.rs:151-167`) → render through the helper protocol → on 3 crashes the effect is blocked (`blocklist.rs:16-17`). Saved projects then fail at each render. The failure is reported by message, not by omission. How the CLI reports it was **not traced**. Helpers are native processes outside the sandbox (see HAC-8).

**WF-3 FigCraft command and panel.** Normal: install → invoke from menu or CLI → panel callbacks each form one undo step → undo restores. Breaks:
- Fresh CLI `run`: `plugin.run` finds no installed plugin (`plugins.rs:338-340`). A script can work around this by calling `plugin.import` (`plugins.rs:430`) as a step. The CLI script runner calls commands directly (`files.rs:429-433`), and the DENIED list applies only to calls a plugin makes (`plugins.rs:234-237`). The import's file read is subject to the CLI's roots option (`cli main.rs:88`; **not traced**).
- `save_file` in that run: the call returns accepted (`plugins.rs:332-333`). The queued dialog is cleared after the command (`headless.rs:68`; `files.rs:433`). The CLI result carries no file path or bytes. **Silent loss with success status.**
- Failure path, correctly handled: a run that fails rolls back its group (`undo.rs:277-278`) and drops its UI requests (`plugins.rs:547-550`).

**WF-4 FigCraft widget.** Normal: insert → callbacks rebuild the widget's nodes and store state in `plugin_data` (`widgets.rs:237-262`). Save and reopen without the plugin: the nodes remain as ordinary document content. Clicks fail with "no plugin ... is installed" (`widgets_tests.rs:276, 302`). Reinstalling a same-ID plugin deletes its settings (`plugins.rs:474-475`). Widget state is keyed by plugin ID and survives, but its meaning depends on the new version.

**WF-5 PhotoCraft filter, pixel layer versus smart object.** Normal pixel layer: `plugin.run` applies the filter in one undo step. The result is baked into the surface (`plugin_cmds.rs:113-130`). Normal smart object: a `SmartFilter {id, params: resolved}` is recorded (`plugin_cmds.rs:133-142`). Breaks:
- Smart object after uninstall: re-render calls `apply_to_surface`, which returns `None` for a missing plugin or any failed run (`.ok()`, `plugin_cmds.rs:156-157`). The comment says the filter is skipped "as for other unknown filters" (`:149-151`). The image shows the unfiltered result without a diagnostic.
- Smart object after a version change: nothing records the version. The new module runs with the old parameters.
- Fresh CLI `plugin.run`: `NotFound` (`plugin_cmds.rs:100-103`), because the registry is empty.

**WF-6 FilmCraft export.** Normal: `file.exportMedia` waits by default (`cli main.rs:201-208`, `long_job.rs:19`). Explicit `wait: false` passes through (`main.rs:205`). The comment at `main.rs:197-198` says a background export would never be written once the process exits. Whether the CLI refuses or warns in that case was **not traced** beyond this function.

## 7. Material issues

Each issue lists the normal path, adverse path, proposed owner, contract, options (recommended default marked), compatibility, performance, and acceptance fixtures. Recommendations are proposals, not approved decisions.

### HAC-1 Fresh-CLI eligibility is not applied

- **Normal (accepted default):** a fresh CLI resolves installed, enabled, compatible, and permitted contributions from the store. It does not download, widen grants, or reactivate disabled plugins (`plugin-functional-decisions.md`).
- **Observed:** EC's CLI host has the plugin loader wired (`crates/host/src/lib.rs:97-100`) but never loads installed packages. FG's fresh session has empty prefs. PC's registry is empty until prefs apply.
- **Adverse:** "no plugin installed" (FG), `NotFound` (PC), or silent omission (EC, HAC-2).
- **Owner (proposed):** each app's session bootstrap calls one shared resolver in the extensions store. The app adapter performs registration.
- **Contract:** `resolve_environment(app, app_version, host_capabilities, policy) -> Eligible{contribution, package, version, digest} | Ineligible{reason}`, run before the session opens. Reasons: not installed, disabled, incompatible, not permitted, revoked, digest mismatch, resource missing.
- **Options:** (a) keep per-app bootstrap; (b) shared resolver, **recommended**; (c) explicit environment only (contradicts the accepted default).
- **Compatibility:** additive. Existing EC `installed.json` and FG/PC prefs are read as today.
- **Performance:** resolve metadata first, then load modules on demand. EC currently compiles every installed package at desktop start (`library.rs:462-489`). Headless paths should stay lazy.
- **Acceptance:** FG-1, PC-1, EC-1.

### HAC-2 Missing contributions are dropped silently

- **Normal:** a referenced contribution is available, or the operation reports why not.
- **Adverse:** EC omits the effect (`pipeline.rs:627-630`). PC skips the smart filter (`plugin_cmds.rs:149-158`). Opening a file does not report missing effects (`file.rs` reports missing footage only). `library.missing` and `file.findMissing` report only on request (`file_more.rs:350-365`).
- **Owner (proposed):** each app's render, export, and smart re-render path, using one shared "missing contribution" status type.
- **Contract:** every final output consumes an explicit status for each referenced contribution. Any non-available status refuses the output unless the caller passes a skip policy. The result then lists each skipped item. Preview may show a fallback, labelled as one.
- **Options:** (a) refuse by default, **recommended**; (b) warn and continue, with the report in the result; (c) current silent skip.
- **Compatibility:** existing projects with missing contributions stop exporting until installed, unless the skip flag is given. The desktop shows the same status.
- **Performance:** a status lookup per referenced contribution is negligible.
- **Acceptance:** EC-2, PC-2.

### HAC-3 Headless artifact destination

- **Normal:** a plugin offers a file, the host asks the user where it goes, and the file is written. The plugin never sees a path (`sandbox.rs:26`).
- **Adverse:** headless discards the request and reports success (WF-3). The primary study already requires a structured input-required result rather than a hidden dialog.
- **Owner (proposed):** FG host and CLI, with a shared artifact-result shape.
- **Contract:** a headless caller supplies a destination before the run (`--out` or an artifact parameter). A plugin that needs an artifact and has no destination is refused before the module runs, with a structured `input-required` error. The result lists artifact path and byte count. The plugin still never receives a path.
- **Options:** (a) pre-run refusal, **recommended**; (b) return bytes base64-encoded in the result JSON, which is poor for large files; (c) status quo.
- **Compatibility:** plugins are unchanged. The 16 MiB cap stays.
- **Performance:** base64 in JSON adds roughly a third to the size and a copy. Avoid it for artifacts.
- **Acceptance:** FG-2, FL-2.

### HAC-4 Persistent references carry no version or digest

- **Normal:** a saved document re-resolves its contribution on open and renders the same processing.
- **Observed:** EC stores the spec ID (`props.rs:183-184`), and `upgrade_effects` migrates only IDs found in the registry (`edit.rs:306`). PC stores ID and resolved parameters (`plugin_cmds.rs:133-137`). FG stores plugin ID and widget state (`widgets.rs:255-261`).
- **Adverse:** a document saved with version 1 opens with version 2 installed and silently runs version 2's code. A document opens with the plugin missing (HAC-2).
- **Owner (proposed):** each app's document format. The reference envelope is shared.
- **Contract:** `{contribution, package, version, digest, params_schema_version, state_version, source}` written at save. On open, resolution gives one of: exact match, compatible newer (warning), missing, digest mismatch (refuse to run, keep data).
- **Options:** (a) exact pin only (safest; needs an upgrade step); (b) compatible range with migration hooks, **recommended** for first-party; (c) ID only (current).
- **Recommended default:** record package, exact version, and digest on save. Allow a compatible newer version on open with a visible warning. Never download automatically.
- **Compatibility:** EC project schema needs a version bump for new group fields. PC smart-filter parameters gain optional fields. Legacy ID-only references are resolved once and flagged as unpinned.
- **Performance:** storing a digest is negligible.
- **Acceptance:** EC-5, PC-3, FG-4.

### HAC-5 Installed modules are not revalidated in FigCraft

- **Normal:** the loaded bytes are the bytes that were installed (checked at install and load).
- **Observed:** EC checks SHA-256 against the installed record before loading (`library.rs:466-474`). PC loads bytes once into the registry (`registry.rs:74`, `plugin_cmds.rs:105-117`). FG reads `manifest.main` from disk on every run and every callback (`plugins.rs:370-395`, `widgets.rs:184`). No digest comparison occurs on that path, and the installed record has no digest field (`plugins.rs:26-37`; `manifest.rs:33` has none either). Directory installs check the bundle's SHA-256 at install time (`plugin_directory.rs:118-122`). That value is returned in the install result and not kept in the record (`plugin_directory.rs:137`).
- **Intent:** the FG per-run read is deliberate, so a rebuilt development plugin runs its new code (`plugins.rs:375-376`).
- **Adverse:** a changed file runs without any record that it changed.
- **Owner (proposed):** FG plugin host.
- **Contract:** installed, non-development plugins are digest-checked when instantiated. A mismatch refuses the run with the EC wording ("the installed file changed since it was installed"). Development reload is available only with developer mode on, and is labelled as such.
- **Options:** (a) digest check on each run, **recommended** (one hash of at most 16 MiB per run, to be measured); (b) load once at install and keep bytes in the session, as PC does, with a digest check at load; (c) current.
- **Compatibility:** developers who rely on rebuild-without-reinstall need developer mode.
- **Performance:** option (a) costs one SHA-256 per run. Option (b) keeps up to 16 MiB per installed plugin in memory.
- **Acceptance:** FG-5.

### HAC-6 Replacement, uninstall, and process-wide state

- **Normal:** an install or update changes what new work resolves to, with clear rules for work already running.
- **Observed:** EC rejects a duplicate ID against built-ins and loaded plugins (`plugin.rs:471-472`). Effect slots are `OnceLock`, set once per process (`plugin.rs:393, 474-478`). Uninstall and update take effect at restart (`library.rs:1057, 517-520`). FG `register` replaces a same-ID plugin, closes its panel, and deletes its settings (`plugins.rs:462-483`), with no source comparison. PC `install` replaces a same-ID plugin silently (`registry.rs:21-27`).
- **Adverse:** a different source with the same ID silently takes the slot (FG, PC). A same-source upgrade loses settings (FG). EC requires a restart to see an uninstall.
- **Owner (proposed):** the shared extensions store (identity and continuity), with per-app registration.
- **Contract:** replacement requires the same publisher identity and source continuity (`plugin-security-design.md` §4-5). A same-source upgrade keeps settings. A different source with the same ID is a refused collision. Uninstall stops new resolution immediately. Running work follows an explicit per-contract rule. A session-scoped environment object replaces process statics (deferred).
- **Options:** (a) keep current replacement and settings deletion; (b) keep settings for same-source upgrades and refuse cross-source collisions, **recommended**; (c) process statics (current).
- **Compatibility:** FG and PC behaviour changes for replacements. Refusal is new behaviour.
- **Performance:** EC allows up to 1,024 plugin slots per process (`plugin.rs:47`).
- **Acceptance:** EC-6, FG-6, PC-7.

### HAC-7 Image and temporal contracts differ by app

- **Normal:** each app receives pixels and time in its own contract.
- **Observed:** EC WASM v1 receives premultiplied RGBA with `time` and `scale` only (`crates/plugin/src/lib.rs:11-18`). Temporal neighbours need the native helper or native API 2 (`plugin.rs:275-300`). PC receives normalised samples in the document colour model, with straight alpha flagged in bit 8 (`apply.rs:12-33`). PC bands carry an overlap margin, edge pixels repeat outside the canvas (`apply.rs:48-71, 167-170`), non-finite outputs revert to the original (`:207-209`), and integer depths are clamped (`:210-212`). FG has no pixel or temporal contract. Image input must be `data`, not a path (`plugins.rs:156-158`). FL has no plugin contract. EC's stack keeps built-in audio effects out of the video path (`pipeline.rs:631`), and no plugin audio contract was found.
- **Adverse:** a time-dependent sandboxed effect cannot read neighbouring frames. A PC kernel wider than its declared overlap is expected to produce seams at band boundaries, because each band sees only its margin (a design reading of `apply.rs:167-170`).
- **Owner (proposed):** each app adapter. The shared layer carries only descriptors and byte budgets.
- **Contract:** keep app-specific pixel and time contracts. Do not introduce a universal RGBA buffer, consistent with the primary study.
- **Options for temporal sandboxed effects:** (a) refuse, **recommended** until a host-mediated input service is designed with explicit bounds; (b) a v2 WASM capability; (c) run them native (HAC-8).
- **Compatibility:** WASM v1 modules keep working. API 2 stays refused for WASM.
- **Performance:** each neighbour fetch copies a frame. Bound the number of fetches per frame.
- **Acceptance:** PC-4, PC-5, EC-7.

### HAC-8 GPU and native trust boundary

- **Normal:** GPU work runs for built-in effects when `--gpu` is set (`cli main.rs:483-488`; `pipeline.rs:633-641`).
- **Observed:** plugin effect specs have `gpu: false` (`plugin.rs:485`), so plugins never reach the accelerator. Helpers are native child processes, started only with `EFFECTCRAFT_HELPERS=1` or from the menu (`main.rs:151-167`), and have no WASM target (`crates/host/src/lib.rs:121-124`). They are outside the sandbox. The timeouts and blocklist in HAC-9 limit faults, not access. FG and PC are CPU-only. PC's wall-clock limit is native only (`runtime.rs:38-40`).
- **Adverse:** a helper runs with the user's file access. The process boundary contains crashes and hangs, not reads or writes.
- **Owner (proposed):** the shared trust policy, with EC's helper host as the only native plugin path among the four apps.
- **Contract:** the sandboxed default admits no helper and no GPU contribution. Helpers stay as a labelled native trust tier until decision D5. Future GPU work is host-mediated and declares a CPU fallback.
- **Options:** (a) exclude helpers from the sandboxed default, **recommended**; (b) keep them first-party only with a native label; (c) current.
- **Compatibility:** helper-only effects are unavailable in a sandboxed-only environment.
- **Performance:** no change for built-in GPU effects.
- **Acceptance:** EC-8. No GPU fixture is proposed now.

### HAC-9 Caches, scheduling, and resource bounds

- **Normal:** a cached result is reused only when everything it depends on is unchanged.
- **Observed:** EC derives the helper `instance_key` from the instance seed (`plugin.rs:306-308, 656-657`). `Destroy` frees helper state (`protocol.rs:106-107`). EC WASM instances are pooled and reused across renders (`wasm.rs:95-106, 158`). Linear memory and globals persist, so output can depend on which pooled instance served a frame unless the module resets its own state. No reset was found, and pool sizing was **not traced**. PC runs bands in parallel, bounded by memory (§4). PC's effect-cache budget applies to built-in effects (`prefs.rs:1365, 1373`). FG builds a fresh instance per call. EC WASM output caching was **not traced**.
- **Adverse:** a cache keyed only by plugin ID returns stale output after an update. Pooled module state can leak between frames. A helper exchange file at 2^28 pixels is 4 GiB on disk.
- **Owner (proposed):** each app's render cache, with the contribution digest in the key.
- **Contract:** cache keys include contribution digest, version, parameters, and context (time, scale, layer size, input digests). Invalidate on registry revision. Never reuse across digests.
- **Options:** (a) no cache for plugin output (simplest, slowest); (b) digest-keyed cache, **recommended**; (c) current, not traced.
- **Performance:** no target is guessed here. The aspect register requires measured bottlenecks first.
- **Acceptance:** PC-4 (parallel versus serial equality), EC-9 (cache key includes digest, to write), EC-10 (pooled-instance determinism, to write).

### HAC-10 FilmCraft has no processing contract

- **Normal:** a processing plugin would enter through a designed frame, time, audio, and job contract.
- **Observed:** no plugin runtime was found in the inspected CLI or top-level engine files (**grep limit: top-level engine files and CLI only**). The only plugin setting is `PluginPrefs { developer_mode }` (`prefs.rs:424-429`). Long jobs wait by default (WF-6, §6).
- **Owner (proposed):** the FilmCraft adapter owner, to be named, after the export job contract is designed.
- **Recommendation:** design the contract from the export and job path first. Do not treat FilmCraft as a wiring-only change. Reuse `developer_mode` as the development gate if plugins are adopted.
- **Acceptance:** FL-1 and FL-2 as existing behaviour. New FilmCraft fixtures are deferred.

## 8. Acceptance fixtures

Evidence labels: **Source (done)** means source inspection in this pass. **Test to write** means a fixture to create later, after an implementation scope is approved. No fixture was run.

| ID | App | Workflow | Setup | Expected (proposal) | Source prediction | Evidence |
|---|---|---|---|---|---|---|
| EC-1 | EC | Fresh CLI render of an installed effect | Package installed; project saved with its effect | Effect resolves; output matches desktop render within tolerance | Not loaded in the CLI; the desktop-only call is at `apps/effectcraft/src/main.rs:279` | Source (done); test to write |
| EC-2 | EC | Export with a missing effect | Same project, package removed | Refused before work, names effect and package; or skip flag with report | Silent omission (`pipeline.rs:627-630`) | Source (done); test to write |
| EC-3 | EC | Uninstall during an open session | Effect applied, then uninstalled | New applies refused now; running work follows the contract | Effect stays registered until restart (`library.rs:1057`) | Source (done); test to write |
| EC-4 | EC | Duplicate ID with a built-in | Plugin claims a built-in ID | Refused | Refused (`plugin.rs:471-472`) | Source (done); existing test **not checked** |
| EC-5 | EC | Saved v1 document, v2 installed | Version changed | Warning; digest respected | Migrates by ID only (`edit.rs:306`) | Source (done); test to write |
| EC-6 | EC | Same-ID install, different source | Foreign package with an ID already registered in this process | Collision refused, with source named | Refused for any ID already registered (`plugin.rs:471-472`); no source comparison found | Source (done); test to write |
| EC-7 | EC | WASM module declares an API 2 kind | Module built with the kind | Refused at load | Refused (`plugin/src/lib.rs:86-88`) | Source (done); existing test **not checked** |
| EC-8 | EC | Helper-only effect in a sandboxed-only run | Helper package | Refused unless native trust enabled | Helper runs as native process | Test to write |
| EC-9 | EC | Cache reuse after update | Same ID, new digest | Cache miss | Not traced | Test to write |
| EC-10 | EC | Pooled-instance determinism | Module with mutable global state; the same frame sequence rendered forward, in reverse, and across parallel renders | Identical output for identical inputs, or the module is refused | Pooled memory persists (`wasm.rs:95-106, 158`); no reset found | Source (done); test to write |
| FG-1 | FG | Fresh CLI `run` of an installed plugin | Plugin in store, not in script | Resolves when eligible | "no plugin ... is installed" (`plugins.rs:338-340`) | Source (done); test to write |
| FG-2 | FG | Headless `save_file` | Plugin requests save; no destination | Refused before run with `input-required`, or file written to destination | Accepted, dialog cleared (`plugins.rs:332-333`; `headless.rs:68`; `files.rs:433`) | Source (done); test to write |
| FG-3 | FG | Failing run with a queued save | Call raises error after save | No document change, no artifact | Rolls back (`undo.rs:277-278`); UI dropped (`plugins.rs:547-550`) | Source (done); test to write |
| FG-4 | FG | Widget with missing plugin, then newer version | Save; reopen without plugin; reinstall | Document opens; clear message; state kept | Nodes persist; callbacks fail (`widgets_tests.rs:276, 302`) | Source (done); test to write |
| FG-5 | FG | Installed module changed on disk | Digest differs from installed record | Run refused with digest error (developer mode off) | Runs new bytes (`plugins.rs:370-395`) | Source (done); test to write |
| FG-6 | FG | Same-source reinstall | Settings present | Settings kept | Settings deleted (`plugins.rs:474-475`) | Source (done); decision D4 first |
| PC-1 | PC | Fresh CLI `plugin.run` | Plugin folder preference set | Resolves when eligible | `NotFound` (`plugin_cmds.rs:100-103`) | Source (done); test to write |
| PC-2 | PC | Smart filter, plugin removed | Smart filter saved; plugin removed | Visible missing state; export refused or reported | Skipped silently (`plugin_cmds.rs:149-158`) | Source (done); test to write |
| PC-3 | PC | Smart filter, version changed | Parameter meaning changed in v2 | Pinned version or migration | No version stored (`plugin_cmds.rs:133-137`) | Source (done); test to write |
| PC-4 | PC | Band invariance | Same filter, default band size vs one-tile bands; overlap ≥ kernel radius | Identical output. With overlap < radius, a seam is expected and documented | Not run. Design predicts equality when overlap suffices | Test to write |
| PC-5 | PC | Preview versus final, pixel-unit parameter | Factor k preview; final | Agreement within documented tolerance | Parameters divided by k (`plugin_cmds.rs:79-98`) | Test to write |
| PC-6 | PC | Non-finite module output | Module emits NaN | Original kept, no crash | Original kept (`apply.rs:207-209`) | Source (done); test to write |
| PC-7 | PC | Same-ID install, different source | Second plugin with same ID | Collision refused | Silent replacement (`registry.rs:21-27`) | Source (done); test to write |
| FL-1 | FL | One-shot `file.exportMedia` | Default | Waits; file written | Waits (`main.rs:201-208`) | Source (done) |
| FL-2 | FL | One-shot `file.exportMedia` with `wait: false` | Explicit false | Refused or warned before exit | Passes through (`main.rs:205`); exit behaviour **not traced** | Source (done); test to write |

## 9. Accepted-decision check

| Accepted decision | Status in this note |
|---|---|
| Sandboxed executable contributions first | WASM paths comply. Native helpers (EC) and in-process CLAP (SoundCraft, per primary study) are exceptions. Needs D5 |
| Official and third-party distribution | Not affected by this note |
| EffectCraft, then FigCraft | EC and FG fixtures are first in the list |
| Existing ABIs preserved through adapters | Kept. Three ABIs are listed in §4, and no shared ABI is proposed |
| Headless-callable editing and compute | FG `save_file` violates it in headless (HAC-3). PC has no verified CLI path. FL export is callable |
| Multiple contributions per package | Not present in EC (one effect per module) or PC (one filter per module). FG has several commands per plugin |
| Fresh CLI uses installed compatible plugins | **Unmet in all four apps** (HAC-1). This is a gap against an accepted default, not a change to it |
| Installation does not grant new permissions | Not tested here. FG enforces permissions per call (`plugins.rs:173-175`). EC and PC have no permission model in the inspected code |

## 10. Decisions for the user (consequences first)

1. **Export with missing or unavailable contributions:** refuse by default, or report and skip? Refusing stops existing projects from exporting until the contribution is installed. Skipping can ship output without the processing.
2. **Headless artifact destination:** require a destination before the run? Plugins that save files cannot run headless without one, but no output is silently lost.
3. **Version pins in saved documents:** exact pin with a warning for compatible newer versions? This needs an EC project schema bump and new optional PC smart-filter fields, with migration.
4. **Same-ID replacement:** keep settings for same-source upgrades and refuse cross-source collisions? FG behaviour changes for same-ID upgrades, and FG and PC would both start refusing cross-source collisions.
5. **Native helpers in EffectCraft:** keep them as a labelled first-party native tier, or exclude them from the sandboxed default? Exclusion leaves helper-only effects unavailable in sandboxed runs.

## 11. Assumptions, deferred items, and coverage

Assumptions.
- The named revisions are the inputs. Later branch state is not assumed.
- Behaviour is read from those commits. Runtime behaviour may differ.
- No app ABI is changed by this note.
- External specifications were not re-checked here. Citations in the security note are unchanged.
- No delegates or sub-agents were used.

Coverage of aspect register rows (focus only).
- Covered: Installation (fresh CLI part), Activation (fresh CLI part), Updates and removal (HAC-6), Sandbox (HAC-5, HAC-8, partial), Document and undo (WF-3, WF-5), Persistent features (HAC-4), Media processing (HAC-7), CLI and automation (§5), GPU and native backends (HAC-8), Audio (HAC-7 and HAC-10; no plugin audio contract found), Caches and scheduling (HAC-9), Diagnostics (missing-state reporting in HAC-2).
- Deferred to other notes: Vocabulary and identity, Package and SDK, Compatibility resolver, Dependencies, Catalogue, Authenticity, Provenance, Proprietary packages, Operations, Capability grants (per-app detail), Resource budgets (beyond §4), UI contributions, Themes, Resources and workflows (ComfyUI and models).
- Not traced: EC WASM module size, pool sizing, and reset policy; EC CLI exit status on missing effects; FG `run --out` semantics; PC attached mode; PC plugin commands via `exec`; FL one-shot exit behaviour when `wait` is false; VectorCraft, SoundCraft, and ArtCraft (out of focus).

No code, build, dependency, CI, external comment, publication, or worktree change was made. The only file written is this note.

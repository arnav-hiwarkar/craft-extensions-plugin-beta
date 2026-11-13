# Craft plugin system guide

> Current source behavior is specified by [beta-design.md](beta-design.md), the
> [app contract references](contracts/photocraft.md), app migration docs and
> [Mac evidence ledger](session-handoff/MAC-EVIDENCE.md). The broader design below
> is historical where it exceeds those beta contracts. It does not grant Wasm
> arbitrary CLI/MCP command access, new object types or workflow APIs.

> **Beta implementation (2026-11-12):** the code follows [beta-design.md](beta-design.md), which simplifies this design where they differ (one-maintainer store admission, signed index instead of TUF, per-contribution contracts and permissions, no bot). The superseded points are listed in [draft/decisions.md](draft/decisions.md#beta-decisions-2026-11-12).

Written 2026-11-12 at the user's request: every part and workflow of the plugin system, explained in plain language, one part at a time. It explains the decided design. **[The architecture](architecture.md) remains the authority**; where this guide and the architecture differ, the architecture wins and this guide should be corrected. Decisions and their dates are in [the decision record](draft/decisions.md#design-session-decisions).

This is a design explanation. It does not authorize implementation, builds, publication or releases.

Review update, 2026-11-12: the [workflow walkthrough](design-sessions/system-walkthrough.md), [forty-scenario map](design-sessions/workflow-coverage-2026-11-12.md), and [correction register](design-sessions/architecture-review-2026-11-12.md) supplement this guide with six further read-only passes and open-PR evidence. AR01 retention is now user-approved; the older R3 finding below is resolved by that decision. Other findings remain proposals unless explicitly decided. Today's app plugin implementations are replacement targets, not the new adapters' starting point.

**Sources read for this guide:** the architecture, the decision record, the post-call reconciliation, the [implementation plan](implementation-plan.md), the [unit plans](implementation/README.md) (P1, P3, P4, P6, P10 in detail), the [threat model](../THREAT-MODEL.md), craft-extensions code at `a6c5fae8`, EffectCraft at `ce9eaec6` (app mechanics only), and the Craft framework document (`aiongg/craft-libs`, branch `craft-framework`, commit `479327ee`). Review findings from 2026-11-12 are kept separate, in [Issues found in review](#issues-found-in-the-2026-11-12-review).

## Contents

- [Starting point: nothing is kept from today's app plugin systems](#starting-point-nothing-is-kept-from-todays-app-plugin-systems)
- [The whole system on one page](#the-whole-system-on-one-page)
- Inside the app
  - [Part 1. What a plugin is](#part-1-what-a-plugin-is)
  - [Part 2. How an app works, and where plugins enter](#part-2-how-an-app-works-and-where-plugins-enter)
  - [Part 3. Contracts: what an app offers](#part-3-contracts-what-an-app-offers)
  - [Part 4. One operation path](#part-4-one-operation-path)
  - [Part 5. Plugin UI: windows, panels, menus, tools](#part-5-plugin-ui-windows-panels-menus-tools)
  - [Part 6. Projects and reliable output](#part-6-projects-and-reliable-output)
- Running code safely
  - [Part 7. The sandbox runtime](#part-7-the-sandbox-runtime)
  - [Part 8. Native plugins](#part-8-native-plugins)
  - [Part 9. Permissions](#part-9-permissions)
  - [Part 10. Safety, layer by layer](#part-10-safety-layer-by-layer)
- Authors
  - [Part 11. How modders build plugins and know what an app offers](#part-11-how-modders-build-plugins-and-know-what-an-app-offers)
- Getting plugins to people
  - [Part 12. Packages, identity and compatibility](#part-12-packages-identity-and-compatibility)
  - [Part 13. Local store and lifecycle](#part-13-local-store-and-lifecycle)
  - [Part 14. The catalogue (the "store")](#part-14-the-catalogue-the-store)
  - [Part 15. Management in every app](#part-15-management-in-every-app)
- Around the edges
  - [Part 16. Automation, jobs, performance and diagnostics](#part-16-automation-jobs-performance-and-diagnostics)
  - [Part 17. Themes and the Craft framework boundary](#part-17-themes-and-the-craft-framework-boundary)
- Delivery
  - [Part 18. Phases, ownership and scaling](#part-18-phases-ownership-and-scaling)
- [Workflows end to end](#workflows-end-to-end)
- [Decided, open and deferred](#decided-open-and-deferred)
- [Issues found in the 2026-11-12 review](#issues-found-in-the-2026-11-12-review)

---

## Starting point: nothing is kept from today's app plugin systems

User direction, 2026-11-10, reaffirmed 2026-11-12: **the plugin systems the apps have today are not a base for this design.** They are removed completely and replaced by the shared platform plus one properly built adapter per app. Existing code is lessons, not constraints.

| App | Removed at its cutover | When |
|---|---|---|
| EffectCraft | The old plugin crate, the Plug-in Library and its index commands, the native helper host | P6 (E8) |
| FigCraft | Manifest API levels 1–6 and the old sandbox host path | P8 |
| PhotoCraft | The old filter ABI | P9 |
| Every other app | Whatever executable plugin mechanism it has | Its own cutover (P11) |

Rules for removal:

- **No dual systems.** An app's old plugin system disappears in the same change set that activates its new adapter. Preparation PRs may add support code but never activate a second system.
- **Release notes only.** There is no compatibility layer and no porting of old plugins by the project.
- **Old projects still open.** A project that used an old plugin opens with a placeholder that keeps that data byte for byte (Part 6).

What the adapters *do* build on is each app's own engine, which is not a plugin system: its command registry, document model, undo, render pipeline, project format and egui UI. The adapter wraps those. Part 2 explains what has to change in them.

> **Open question raised 2026-11-12.** The gap-check decision of 2026-11-10 kept two industry-format mechanisms as native full-access plugins: SoundCraft's VST3/CLAP/AU hosting and CADCraft's AutoLISP. The user's 2026-11-12 wording ("we don't take any of the current plugin systems made for any app") may remove that exception. Until the user answers, the architecture's recorded exception stands.

---

## The whole system on one page

### Where everything lives

```text
storytold/craft-extensions        all shared platform code, one version line, one CI
  craft-ext-core                  manifest, IDs, compatibility, the one validator          (exists)
  craft-ext-store                 installed releases, per-app setup, grants, leases        (exists)
  craft-ext-ops                   invocation, principals, operation markings, errors, jobs
  craft-ext-contracts             shared domain contracts, app contract inventories
  craft-ext-host                  wasmi sandbox profile, loading, budgets, native helper protocol
  craft-ext-conformance           contract suites, hostile-module suite, benchmarks, reference host
  manager UI                      the one shared egui manager component
  craft-guest(-<contract>)        author SDK and generated bindings
  craft-ext (CLI)                 create, check, test, dev, pack, prepare-release, verify

each app repository
  <app>-plugins adapter           the only code that knows both the platform and the app

catalogue repository (planned)    metadata only: publishers, packages, releases, revocations
author repositories on GitHub     source + GitHub Release assets (the only place plugin files live)
```

Crate names other than `craft-ext-core` and `craft-ext-store` are illustrative.

### The life of one plugin

```text
Author builds ─► prepares release ─► GitHub Release ─► catalogue PR ─► bot checks ─► merged + signed
                                                                                      │
Person browses ◄─ catalogue fetched (signed) ◄────────────────────────────────────────┘
   │
   ▼
Install (download, digest, validate; runs nothing, enables nothing)
   ▼
Set up in one app (enable contributions, grant permissions)
   ▼
Load (eligibility check) ─► Run (sandbox, budgets, permissions) ─► Saved in projects
   ▼
Update (always asks) · Disable · Revoke (by maintainers) · Uninstall
```

The MVP (PC01) covers the bottom half for EffectCraft only, starting from a **manual file import** instead of the catalogue.

---

## Part 1. What a plugin is

### Words

| Word | Meaning |
|---|---|
| Extension / add-on | The UI word for anything installable |
| Package | The install and update unit: one ID, one version, one archive |
| Release | One version of a package: ID + version + pinned SHA-256 digest. Immutable |
| Contribution | One feature inside a package, `<package-id>/<name>`, e.g. `io.github.ink.grain/halftone` |
| Plugin | A contribution with executable code |
| App setup | One app's choices for a package: which contributions are on, grants, settings |
| Saved feature | A plugin feature stored in a project |

### Three interaction styles plus native

| Style | What it is | How it runs | What it can reach | App level |
|---|---|---|---|---|
| **Data** | Preset, LUT, ICC profile, gradient, swatch, brush, template, font, theme, shortcut profile, translation, workflow | No code. One bounded importer per kind reads it | Nothing | L0 |
| **Transform** | Effect, filter, generator, spreadsheet function | WebAssembly. The host passes typed input; the plugin returns typed output; the host validates it | Nothing beyond its contract | L1 |
| **Session** | Panel, tool, command, inspector section, canvas widget, new document object type | WebAssembly that calls the app's exposed operations through the dispatcher, inside host transactions | Exposed operations, within its grant and scope | L2 |
| **Native** | Helper, OpenFX- or VST-style host | A separate operating-system process speaking a versioned protocol | Anything the person's account can do | Labelled "Native: full access to your computer" |

### Compiled or runtime?

Both, in a specific sense:

- **Compiled ahead of time.** Authors compile Rust to `wasm32-unknown-unknown` (or to native code for native plugins). Store packages never contain interpreted scripts.
- **Loaded at runtime.** An app loads a plugin from the store without being rebuilt. WebAssembly is interpreted by wasmi, a pure-Rust interpreter, inside the app's process.
- **App scripting is separate.** Each app's own scripting and expressions (for example EffectCraft's Application Scripting) remain app features. They run as the person, under the app's script permission, and are not store packages. A sandboxed script-plugin kind is a later contribution kind.

### Levels of support per app

| Level | Adds | Rollout |
|---|---|---|
| L0 | Manager, store, `plugin` CLI, data add-ons | Every app, early. In the three pilot apps, L0 lands with each app's atomic cutover (recommended in the plan; maintainer decision) |
| L1 | Transforms through shared domain contracts | Per app, by demand |
| L2 | Session plugins: operations, panels, tools, canvas, object types | Per app, by demand |

The manager shows each app's level, so a person can see what a plugin can do in which app.

### What plugins may and may not do

**May:** call every operation the app exposes to its CLI and automation, except protected ones; add effects, tools, panels, dialogs, commands, menu items, inspector sections, settings pages, canvas widgets, new document object types, importers, exporters, presets, themes, resources and workflows; edit documents within their granted scope; be native.

**May not:** replace, remove, disable, rebind or alter built-in commands, tools, effects, menus or shortcuts; change default settings, built-in file handlers, undo and history, updates, plugin management or permissions; reach the app's automation channels; borrow another plugin's authority.

---

## Part 2. How an app works, and where plugins enter

### The app model every Craft app shares

Every Craft app is a native Rust program built on egui. EffectCraft (read at `ce9eaec6`) shows the common shape:

| Piece | What it does | EffectCraft example |
|---|---|---|
| Window and loop | `eframe` opens a native window. Every frame, the app rebuilds its whole UI (egui is immediate mode: there are no retained widget objects; the code that draws a button also handles its click) | `apps/effectcraft/src/main.rs`, `EffectcraftApp::update` → `draw()` |
| Layout | Docked panels, menus, dialogs, viewer, inspector | A fixed `PanelKind` enum in `crates/ui-egui/src/dock.rs` |
| Session / document | Owns the open project, selection, undo history, journal | `Session` in `crates/engine` |
| Command registry | Every user action is a named command with parameters. Menus, shortcuts, CLI, MCP and scripts all call it | ~760 commands; `Session::execute(id, params)` |
| Render pipeline | Evaluates the document (effects, keyframes, time) into pixels for preview and export, with caches | `crates/render`, preview frame cache in `ui-egui/src/frames.rs` |
| Project format | Saves and loads the document | `crates/project` |
| Automation entry | External control | Today an unauthenticated localhost TCP port |

### Where plugins enter: the adapter

**One adapter per app** (for example `effectcraft-plugins`), in that app's repository. The shared platform knows plugins, permissions, the sandbox, the store and contracts. The app knows its documents, undo, rendering and menus. The adapter is the only code that knows both, and it contains no copy of shared logic, so a plugin behaves the same everywhere.

| Adapter job | Part |
|---|---|
| Say what the app offers: contract export, which shared domain contracts it implements | 3 |
| Convert data honestly between app types and contract types; validate results | 3 |
| Mark every operation: kind, exposure, required permissions, normal/Management/Core/Internal | 4 |
| Place contributions in registries, menus, panels, inspector, tools and canvas | 5 |
| Run them through the app's undo, render pipeline, caches and job queue | 4, 6, 16 |
| Save and reopen: references, placeholders, fallback previews, export refusal, bake | 6 |
| Embed the shared manager, `plugin` CLI commands and the socket attach transport | 15, 16 |
| Prove it works: conformance runs and a headless plugin-test entry | 11 |

**Shape.** `craft-ext-host` defines small traits, one per job: contract provider, operation markings, UI extension points, project hooks, job hooks. A generated starter skeleton plus a checklist test fails when a required hook for the app's level is missing. An L0 checklist never asks for L2 hooks.

**Who builds them.** The platform team builds the EffectCraft, FigCraft and PhotoCraft adapters with those maintainers, then turns the lessons into the skeleton and an adapter guide. Every other app's maintainers build and own theirs.

### What has to change inside an app's engine (EffectCraft's P6 as the example)

These are changes to the app's own engine so the adapter can work properly. They are not reuse of the old plugin system.

| Change | Why | P6 PR |
|---|---|---|
| Mark every command (kind, exposure, marking); CI fails on an unmarked one | So nothing is exposed to plugins by accident | E1 |
| Make `Invocation` mandatory and delete caller-less `execute` | So every call carries who is asking (Part 4) | E2 |
| Dispatcher-owned transactions | One undo step per action, rollback on failure (Part 4) | E3 |
| Replace the TCP control port with a user-only socket and token | Security fix (Part 16) | E4 |
| Make effect registrations unloadable | Updates and disables apply at safe points without restart | E5 |
| Placeholder representation | Missing plugins keep their data (Part 6) | E6 |
| Render errors propagate during export | Export never silently drops a plugin (Part 6) | E7 |
| Atomic cutover: activate adapter, manager, `plugin` commands; delete the old plugin system | No dual systems | E8 |
| Temporal hooks, cache identity, safe points, safe mode, diagnostics | | E9 |
| Bake, include-plugins, full pilot acceptance | | E10 |

---

## Part 3. Contracts: what an app offers

A **contract** is the typed promise between an app and a plugin: what the plugin receives, what it must return, and what the host guarantees.

### Three layers

| Layer | Contents | Shared by |
|---|---|---|
| Common mechanics | Identity, parameters (units, ranges, animation), declarative UI, results and error codes, jobs, handles and revisions, edit proposals, logs | All apps |
| Shared domain contracts | `raster` (first: `raster.filter@1`), `temporal`, `audio`, typed resources such as LUTs; `path2d` is a candidate | Apps whose adapter passes that contract's conformance suite |
| App contracts | The app's own operations, generated from its command registry | That app only |

**One module, many apps.** A transform written against a shared domain contract runs unchanged in every app whose adapter passes that contract's suite. P9's proof is the same Halftone file, same digest, passing in both EffectCraft and PhotoCraft. Session contributions are always per app.

### Shape and versions

- Each contract is a **small core plus optional extensions**, as in OpenFX and CLAP. A plugin declares what it supports and needs; the app advertises versions, extensions and limits.
- A **minor** version adds optional items. A **major** version is a new contract that apps can run alongside the old one for **12 months** (shorter only for published security reasons).
- An unforeseen need starts as an app extension and is promoted to a shared contract when a second app needs it. Shared contract changes need approval from maintainers of at least two consuming apps.

### Obligations

| Host must | Plugin must |
|---|---|
| Convert formats honestly (no silent CMYK or Lab conversion) | Produce deterministic output (within declared tolerances) |
| Supply or refuse regions, never seams | Give the same result tiled and whole |
| Evaluate keyframes and units before calling | Declare honestly what it needs |
| Validate output | |
| Commit through the app's own undo | |
| Isolate instances | |

### Example: `raster.filter@1` (P4, provisional until two app maintainers approve)

The host sends a JSON **request** (≤ 64 KiB: contribution, parameters, time, seed, scale, quality draft/final, input and output rectangles, format, colour space) plus an **input buffer** of row-major RGBA f32 pixels. The plugin writes the **output buffer**. Parameter kinds (number, angle, percent, pixels, point, colour, checkbox, choice, seed) map onto each app's own controls. `Pixels` values arrive already multiplied by the render scale.

### How a contract gets from the app to authors

```text
app xtask export-contract ─► inventory file (from the app's own registry)
  ─► app CI fails on a breaking change without a major bump
  ─► inventory copied by reviewed PR into craft-extensions
  ─► generator emits a committed guest crate whose major = contract major
  ─► same inventory generates CLI `ops list`, MCP tool schemas and reference docs
```

---

## Part 4. One operation path

Every caller — menus, shortcuts, panels, the CLI, MCP agents, scripts and plugins — goes through one function:

```text
fn execute(&mut self, inv: &Invocation, op: OpId, args: Args) -> Result<Outcome, OpError>

Invocation { principal, origin chain, captured scope, deadline, cancel, transaction }
principal = User(Gui) | Cli | Automation(client, scope)
          | Plugin(app, package, release, contribution, grant generation)
```

An `Invocation` can only be built by trusted entry points and can never be deserialized from plugin input, so a plugin cannot claim to be the person.

### Operation descriptors

Each app extends its existing registry (no second registry). Each operation gains:

- argument and result types (serde + schemars);
- `kind`: Query, Edit, Job or Effect;
- exposure per caller kind;
- a **marking**: normal, `Management`, `Core` (changes built-in behaviour) or `Internal`;
- `requires(resolved)`: the permissions needed, computed from the real targets;
- `interaction`: none, optional, required;
- version metadata.

**Default exposure:** an operation exposed to the CLI or automation is exposed to plugins, unless it is Management, Core or Internal. CI fails for any operation without a marking.

### Three checks, in order, before any side effect

1. **Exposure** — may this kind of caller use this operation at all?
2. **Authorization** — with arguments resolved to real targets, does this caller's grant and captured scope allow it? Explicit targets never skip this.
3. **Availability** — is it enabled right now? A UI hint and error message only, never security.

### Transactions and undo

- An Edit opens a transaction and commits **one undo step labelled with its origin** ("Halftone · Ink & Grain").
- Error, trap, cancel or budget exhaustion **rolls everything back**. Nested calls join the outer transaction.
- Effects (file writes, network, processes) wait in an **outbox** and run only after commit.
- Plugins cannot reach undo or history operations.

### Long work

- A short command holds the document briefly.
- A **Job** reads a captured revision, works off the editing thread and commits with a revision check. If the document changed meanwhile, it refuses (or merges where provably safe).
- Events reach observers only after commit. A plugin reacting with an edit makes a new invocation; origin chains are capped.

### Other rules

- **Handles, not paths.** Plugins get opaque handles that carry a revision. Files picked by the person arrive as tokens. Stale handles fail.
- **Origin everywhere.** The principal is recorded in the journal, undo labels, logs and events. Plugin and agent actions do not train habit learning.
- **Data paths.** Control data is JSON validated against the schema (unknown fields rejected, size-capped). Bulk pixels, audio and geometry use contract buffers.
- **Session plugins resolve their imported operations at load**, so a missing operation is reported then, never mid-edit.

### Who has what authority

| Caller | Authority |
|---|---|
| The person in the GUI | Everything |
| A CLI the person runs directly | The person, including management |
| MCP agent, attached CLI, control channel | The person's full document authority; management needs in-app approval once per session |
| A plugin | Normal operations only, within its grant and scope; never automation channels |

---

## Part 5. Plugin UI: windows, panels, menus, tools

**The host draws everything; plugins describe it.** Plugin code never runs on the UI thread, never opens its own window and never draws prompts. This gives matching themes, accessibility and translation, and prevents spoofed dialogs. Webviews are not used (house rule).

| What | The author writes | The host does |
|---|---|---|
| Effect parameters | `#[derive(craft::Params)]` | Controls, keyframes, presets |
| Commands | A command contribution | Menus, command search, CLI, MCP |
| Menu items | A declared extension point (e.g. Effect ▸ Stylize) | Appends after built-ins, grouped by plugin |
| Shortcuts | An optional suggestion | Used only if the chord is free; never replaces built-in or personal bindings |
| Panels, dialogs, inspector sections, settings pages | A declarative tree, plus a **custom canvas** element (draw lists or an image) | Docks it; plugin dialogs carry the plugin's name and icon |
| Canvas overlays and widgets | Bounded draw lists in document coordinates | Rendering, hit testing, hover and click events |
| Pointer tools | begin, update, commit, cancel | Capture, coalesced updates, overlay preview; commit is one undo step; Escape cancels with no change |
| New document object types | Typed data, drawing or rendering, inspector UI | Selection, transform, undo, save, collaboration, saved fallback preview |
| Importers and exporters | A format handler | Built-in handlers stay default; a plugin handler is offered as a choice |

### How a plugin panel actually works

```text
Frame N (UI thread): host draws the panel from the last tree the plugin returned
Person clicks a button ─► host queues an event
Worker thread: event becomes an invocation under the command budget
   ─► plugin handles it (may call operations ─► one transaction ─► one undo step)
   ─► plugin returns an updated tree
Frame N+k: host draws the new tree
```

- Closing a panel disposes its session. Switching documents sends a scope change and cancels a gesture.
- An update or disable reopens the UI at the next safe point.
- The host saves panel placement; plugin settings go in its private storage; widget data goes in the document.
- In headless runs, UI contributions do not exist but their commands still do.
- The SDK refuses unlabelled controls.

L2 (session plugins and this UI) is outside the EffectCraft MVP. FigCraft is the first L2 pilot (P8).

---

## Part 6. Projects and reliable output

### What a project saves

- Per feature: contribution ID, **exact version**, contract version, parameters, versioned plugin state.
- Per project: a plugin list (IDs, versions, catalogue link or "local only").
- **Never** grants. Permissions belong to the machine.

### Reopening

| Situation | Result |
|---|---|
| Same or newer compatible release installed | Used, with a quiet note if the version differs |
| Old plugin state schema | Plugin's `migrate()` runs; saved only when the person saves; failure → placeholder |
| Older release or incompatible major | Placeholder with Update |
| Not installed or not set up | Placeholder; one prompt lists them all ("This project uses 3 plugins") |
| Revoked, or from a removed old plugin system | Placeholder with the reason |

**A placeholder keeps everything** — parameters, state, keyframes, position. Preview passes the input through with a badge. Saving writes the data back byte for byte.

### Output is truthful

- A final render or export with any missing, failed or blocked plugin **does not start**. It lists each plugin with a fix; the CLI prints install commands.
- A failure **during** export fails the export, naming plugin and frame, and leaves no completed file.
- Preview may use draft quality; final always runs at full quality. Caches key on release digest, contract version, parameters, inputs and quality.

### Portability

- **Bake plugin result** converts output to the app's own data and keeps the original switched off, so it can be un-baked.
- **Include plugins** (off by default) carries plugin files in a project or collect-files folder. Carried plugins never install or run on open; "Install from project" is an ordinary local install.
- **New object types** save a fallback preview, so a project opened without the plugin still shows something (locked), and export still refuses until the plugin is back or the object is baked.

---

## Part 7. The sandbox runtime

### Engine

**wasmi** for every app: a pure-Rust WebAssembly interpreter that also runs in web builds. `craft-ext-host` owns one profile that apps cannot loosen:

- fuel metering on; strict limits; recursion cap;
- a pinned feature list (no threads);
- deterministic mode for transforms;
- validation caps; unknown imports refused.

Validated modules are cached per digest (and engine profile). The host is engine-neutral: a compiling engine (wasmtime) can be added on desktop only if benchmarks show the need — the trigger is a reference 4K effect several times slower than its built-in equivalent.

### Budgets

| Budget | How |
|---|---|
| Instructions | Fuel scaled by work size (pixels, samples, cells, nodes) |
| Time and cancel | Fuel in slices, with deadline and cancel checks between slices |
| Memory | Per instance and total across parallel instances; parallelism reduced to fit an app-wide ceiling |
| Host calls | Fuel per call plus per byte copied |
| Output | Validated by the contract before use |

Profiles: preview, final render, session command, job. Numbers come from a CI benchmark set with generous headroom. "Allow more time" raises a plugin's budget per app, up to the host maximum.

### Guest ABI v1 for transforms (P4, provisional)

No imports: the plugin cannot call anything. It exports `memory`, `craft_abi` (returns 1), `craft_alloc`, `craft_raster_filter_padding`, `craft_raster_filter_render` and `craft_last_error`. Two custom sections, `craft.abi` and `craft.contributions`, describe the module and its contributions; the host reads them without executing anything. The SDK macro writes all of this; authors write no `unsafe`.

### Instances

- Transforms: one instance per render thread, reused only within one document and job.
- Session plugins: on a worker thread, never the UI thread.
- A trapped or over-budget instance is discarded, never reused.

### Failure contract

| Event | Person sees | Document |
|---|---|---|
| Trap (crash inside the sandbox) | Message naming the plugin; preview shows an error badge and passes the frame through | Rolled back |
| Too slow / too much memory | "Took too long", with Allow more time | Rolled back |
| Invalid output | "Returned an invalid result" | Nothing committed |
| Forbidden call | Structured denial the plugin can handle | Unchanged |
| Permission revoked mid-run | Stops at the next host call | Rolled back |
| Failure during final render or export | The export fails, naming plugin and frame | — |
| Repeated failures | Paused for the session with a notice | — |

**The host never panics because of a plugin.**

---

## Part 8. Native plugins

Native plugins exist for what WebAssembly cannot do: helpers that use existing native libraries, and hosts for industry formats.

| Aspect | Rule |
|---|---|
| Where it runs | Always **out of process**, never loaded into the app. A crash kills the helper, not the app |
| How it talks | Versioned helper protocol v1: length-prefixed frames over stdin/stdout, 64 MiB frame cap, pixels chunked. `hello`, then request/response pairs using the same `raster.filter@1` request type, plus `cancel` and `bye`. No shared memory in v1 |
| Supervision | Spawned from the release's verified binary (never a path from the package); watchdog, restart, blocklist for repeat offenders; killed on revoke |
| Authority | **Anything the person's account can do.** No sandbox promise |
| Label and consent | "Native: full access to your computer", its own consent screen; every update asks |
| Packaging | Manifest `native: true`, one zip per platform on the author's GitHub Release |
| Review | **Two maintainers approve every native listing and every native release.** The bot never auto-merges native |
| Failure | Same contract as Part 7: preview passes through with a badge, final fails |
| Later | OS confinement (macOS App Sandbox, Windows AppContainer, Linux Landlock/bubblewrap) is an optional improvement, never a listing requirement |

Native support arrives in **P10**. Between EffectCraft's cutover (P6) and P10, EffectCraft has no native helpers.

### Native vs sandboxed at a glance

| | WebAssembly | Native |
|---|---|---|
| Built as | `wasm32-unknown-unknown` | Platform binaries |
| Runs | Inside the app, in wasmi | Separate process |
| Protected by | Sandbox, permissions, budgets, transactions | Review and the full-access label |
| Crash effect | Trap, rollback, message | Process dies, helper restarts |
| Permissions | Declared, granted, enforced | Full access by consent |
| Works on web | Yes | No |

---

## Part 9. Permissions

PC06 (2026-11-11) reconfirmed: **WebAssembly permissions are enforced by the host.** Disclosures explain access but never grant or replace enforcement.

### Vocabulary

A permission = **family + access + scope**. The families are shared by every app:

| Family | Access | Scopes |
|---|---|---|
| Document | read, edit | selection, current document, all open documents |
| Files | read, write | picked item, picked folder (remembered), own storage |
| Network | connect | named https domains only |
| Account | use | **Not in the first release** |
| Clipboard | read, write | on a user action |

- **Always free:** computing on inputs, returning results, progress and logs, bounded private storage, UI placement.
- **Never grantable to WebAssembly:** processes, native code, raw sockets, other plugins' data, management.
- App-specific needs add scopes to existing families. A new family needs a second consumer.
- **Network** requests are made by the host: https only, declared domains only, redirects stay in those domains, size and time capped, loopback/private/link-local addresses refused after DNS resolution.
- **Private storage:** one folder per package per app, size-capped; kept on disable, deleted on uninstall; unreadable by other packages.

### Declaring

Authors list permissions **per app** in `Craft.toml`. `craft-ext check` warns about permissions it can infer from imports; the runtime enforces regardless, because session operations go through one dispatcher import and cannot be fully inferred. Permissions are never inserted automatically.

### Granting: when people are asked

| Moment | What happens |
|---|---|
| Setup in an app | One sheet, plain host-written sentences. Required permissions must be granted to enable. Optional ones are switches, off by default. Dangerous combinations are spelled out ("could send your document to api.example.com"). The author's "why" is quoted beneath |
| Optional permission at first use | Asked only from a person-started GUI action, in a host-drawn prompt |
| Update that widens permissions | New release waits; old one keeps working until accepted |
| CLI, agents, renders | Never prompt. They fail with the exact grant to add (exit code 4) |
| Picking a file or folder | Picking **is** permission for that item |

Grants attach to the **package within one app** (T01). Enabling individual contributions controls what appears; it is not a security boundary.

### Enforcing

Effective authority of a running plugin:

```text
its style's imports ∩ its package's grant in this app ∩ what the host supports
  ∩ the invocation's captured scope ∩ machine policy
```

- Each grant record holds app, package ID, source binding, normalized permissions, picked-folder tokens, a **generation number**, the granted release and a date. Grants live on the machine, never in projects.
- **Every host call checks the generation.** A revoke takes effect at the next host call: running edits roll back, jobs cancel, queued jobs check at start. Another process sees the revoke within about 250 ms (P3's polling interval).
- Every denial happens before any side effect (Part 4).

### Managing

- GUI: per plugin, each right with its last use and a revoke button. Disable keeps grants; uninstall deletes them.
- CLI: `plugin grant|revoke|permissions` as separate commands. There is never an `--allow-all` on execution commands.
- Agents need the management scope. An optional machine-wide policy (e.g. "no network for plugins") applies to everyone.
- Moving to a new computer: setup export/import reinstalls packages, but **permissions are asked again**.

**Typical outcome:** an effect asks for nothing. A panel that edits the selection shows one sheet. An uploader shows one sheet with the domain and the danger sentence.

---

## Part 10. Safety, layer by layer

The principle: **protect people by design, not by review.** Every layer holds even if another misses something. Native plugins are the stated exception, protected by review and their label.

| Layer | Stops | Where |
|---|---|---|
| Bounded archive unpacking | Zip bombs, path escapes, extra/missing files (T1–T3) | `craft-ext-core`/`store`, built |
| Text and ID validation | Hostile text, lookalike IDs (T4, T5) | Built |
| Reserved first-party namespaces; host-assigned labels | Impersonating Official; a package claiming its own label (T6) | G1, P1 |
| Install runs nothing, enables nothing | Code running before a person chose it | PC02, P1 |
| One validator shared by bot, `check` and installer | Disagreement between what was checked and what runs | Part 11 |
| Sandbox profile + budgets | Infinite loops, memory bombs, escape (T8) | Part 7 |
| Host-enforced permissions with generations | Unauthorized access; stale grants | Part 9, G6 |
| Protected core (operation markings) | Plugins altering built-ins, management, undo | Part 4 |
| Transactions + outbox | Half-applied edits; side effects from failed edits | Part 4 |
| Importer fuzzing | Hostile data files (T9, T10) | G8 |
| Pinned digest from signed metadata | Swapped assets (T11, T15) | Part 14 |
| TUF signing with expiry and versions | Tampered or rolled-back catalogue (T11, T12) | Part 14, G2/G3 |
| 48-hour hold on auto-merged updates | Hijacked author accounts shipping routine-looking updates | Part 14 |
| Revocation by digest, range or publisher | Already-installed bad packages (T13) | Part 14, G4 |
| Fresh consent for owner, permission or native changes | Silent changes (T14) | G7 |
| Safe mode and bisect | A plugin that breaks startup | Part 16 |
| Hostile-module suite in CI on every target | Regressions in the sandbox | P3 |
| Independent outside security review before public launch | Flaws in our own code (T17) | G12, P12 |
| `third-party` cargo feature off until P7 | Opening the gate by accident (T18) | G5, G13 |

**Not claimed:** a hash proves a file matches the catalogue, not who wrote it. Trust comes from the public source link, review and the label. A native plugin has no sandbox promise. Scanning policy beyond these checks is deferred (PC05).

---

## Part 11. How modders build plugins and know what an app offers

### How they know what exists

| Source | Gives |
|---|---|
| Generated contract reference (docs.rs once published, GitHub Pages until then) | Every shared contract and app contract, with versions and changelog |
| `<app>-cli ops list`, `op <id>` | The operations an app exposes, with arguments and help, generated from the same inventory |
| The manager | Each app's level (L0/L1/L2) |
| Author guide, one template per contribution kind, examples | How to build each kind |
| `craft-ext check` | What a package needs, what is wrong, the exact line to add |

Because SDK, CLI help, MCP schemas and docs are all generated from one inventory per app, they cannot disagree.

### What an author writes

- **`Craft.toml`** — what a reviewer reads as a whole: ID, name, version, licence, publisher, links, **permissions per app**.
- **Code** — contributions and parameters declared with macros; the macros write a WebAssembly custom section, so the manifest and the code cannot drift:

```rust
#[derive(craft::Params)]
struct Halftone {
    #[param(label = "Cell size", min = 2.0, max = 200.0, default = 8.0, unit = "px")] cell: f64,
    #[param(label = "Angle")] angle: craft::Angle,
}

#[craft::transform(contract = "raster.filter@1", id = "halftone", label = "Halftone",
                   apps = ["effectcraft", "photocraft"])]
fn halftone(req: &craft::raster::Request<Halftone>,
            input: craft::raster::Image<'_>,
            out: &mut craft::raster::ImageMut<'_>) -> Result<(), craft::Error>;
```

(Provisional ergonomics from P4; Halftone is an illustration, not a selected MVP example.)

### The toolchain

- Rust first, `wasm32-unknown-unknown`, pinned with `rust-toolchain.toml`; target features match the wasmi profile (to verify in P3/P4).
- Deterministic builds, `cargo-deny`, an SBOM.
- Other languages later, only with generated and verified bindings.

### The author loop

```text
craft-ext create          template for the chosen contribution kind
craft-ext check           validate, infer permission warnings, explain fixes
cargo test                unit tests (native)
craft-ext test            contract tests in the reference host: determinism, tiling, formats, cancel
<app>-cli plugin test     app-level tests against fixtures (headless entry every adapter provides)
craft-ext dev --app X     build, pack, install into an isolated development store,
                          start or attach to the app, stream logs, reload on change
craft-ext prepare-release build, check, test, write zip + inventory + catalogue record,
                          diff against the previous release (removed IDs, unmigrated state,
                          wider permissions). Never publishes
```

A template GitHub Action uploads the release with attestations. The author opens the catalogue PR. **The SDK is a convenience, never a trust anchor**: the bot, `check` and the installer share one validator. Development-store packages are treated as Not reviewed by normal builds.

---

## Part 12. Packages, identity and compatibility

### IDs

- Reverse-domain, no UUID. `io.github.<owner>.<name>` is verified automatically against the GitHub repository owner; other domains are verified once by a maintainer (for example DNS).
- First-party prefixes are reserved. IDs never change; display names and URLs can.
- A local file cannot take over a store-installed ID.

| Event | Result |
|---|---|
| Repository renamed or moved | Numeric repository ID matches; URL updated |
| Plugin renamed | Display name only |
| New owner | Recorded transfer; "has a new owner"; confirm before the next update |
| Abandoned | "No updates since …"; a fork is a new ID |

### Package contents

A zip holding the manifest (`craft-extension.json`, written by `craft-ext pack` — schema 2 in P1), the module or data files, licence and assets. Every file is listed with its SHA-256 and size; the archive must match exactly. Native packages have one archive per platform.

### Compatibility is decided by contracts, not app versions

- Each package declares, per app, the contracts it uses (major and minimum minor), recorded automatically by `pack`. A minimum app version is optional.
- Checks: manifest schema, contract versions, engine features, platform (native).
- Outcomes: **compatible**, **partly compatible** (each unavailable contribution with its reason), **incompatible** (with the update to make). Never a silent half-load.

### Dependencies

A package may require **data packages** (resource packs, LUTs, models, themes) by ID and version range. Install offers the missing ones in the same flow; removing a needed package asks first; conflicts are reported; cycles are refused. Shared code is compiled into each plugin, never shared at runtime. Plugin-to-plugin calls are a later feature.

---

## Part 13. Local store and lifecycle

PC02 and PC03 (2026-11-11): import and validate, then explicitly enable; package files are **shared** by all apps, while enablement, settings and permissions are **per app**. Installing in one app never enables another.

### Five kinds of state, one owner each

| State | Where | Owner |
|---|---|---|
| Release bytes | `packages/<id>/<version>-<digest>/` | Store; deleted only by cleanup |
| Package record | Source, label, releases, default | Store |
| Per-app setup | Contributions, grants, settings, theme, pin, time raises | The person, through any app, launcher or CLI |
| Leases | `leases/<process>.json` | Each running app, CLI or job |
| Caches | — | Any app; always deletable |

The store already provides a shared folder (`CRAFT_EXT_DIR` override), immutable version folders, staging, a short lock, atomic state and a generation counter. P1 adds verified release records, digest-addressed folders, leases, per-app setup and grants, eligibility, revocation and state versioning (`read_version`/`write_version`, as SQLite does).

### Transitions

| Action | Behaviour | After a crash |
|---|---|---|
| Install | Download → pinned digest → stage → validate → move → record. **Runs nothing, enables nothing** | Previous state; leftovers cleaned |
| Set up | Choose contributions, grant permissions, in one app | — |
| Update | **Always asks.** New release beside the old; per-app choices carry over; new contributions start off; wider permissions, new owner or native keep that app on the old release until accepted | Old release still default |
| During running work | Picked up at the next safe point. Effects switch on next use; panels reload when idle; **exports and jobs finish on the leased old release** | — |
| Version choice | The person may choose any listed version, older included. An ordinary update never downgrades | — |
| Disable | Off in that app; grants kept | — |
| Uninstall | Record, setups, grants removed; bytes deleted when no lease remains; projects show placeholders | Record still present |
| Low disk | Checked before download | — |

**What is kept on disk (AR01, user-approved 2026-11-12):** every referenced release, including defaults, each app's effective selection/pin, pending-consent references, required dependency releases and live work. Cleanup removes only unreferenced releases. This refines T08's earlier current-plus-leases rule. Exact effective-release and chained pending-update fields remain open.

### Labels

Assigned by the host from where the package came from, never read from the package: `FirstParty` → **Official**; `ReviewedThirdParty` → store listing; `FromFile` → **Not reviewed**. A local file whose digest matches a listing is treated as that listing, and revocations apply by digest.

### Builds and storage locations

| Build | Store |
|---|---|
| Standalone, launcher-installed | Shared folder |
| Linux Flatpak | Access to that one folder, else its own store |
| Web | Browser-local store per app; file installs; catalogue installs if cross-origin downloads work (unverified) |
| Builds without network code | Local file installs only |

---

## Part 14. The catalogue (the "store")

**The store lists; it does not build, host or take responsibility.** Plugin files stay on authors' GitHub Releases. Store plugins are open source, MIT and/or Apache-2.0, with bundled code and assets from the house licence allowlist. Local installs may use any licence, labelled Not reviewed.

### Hosting (PC08)

- A catalogue repository on GitHub (planned; maintainers create and name it). Every listing is a reviewed PR with history.
- GitHub Pages serves the signed catalogue as static files initially; a custom domain later.
- No servers, no database, no file mirror.

### Records (metadata only)

| Record | Contents |
|---|---|
| Publisher | GitHub login and **numeric owner ID**, display name, official flag (maintainers only), status |
| Package | ID, repository and numeric repository ID, licence, apps, summary, native flag |
| Release | Version, tag, commit, per-platform URL/size/SHA-256, contracts per app, permissions, bot report |
| Revocation | Digest, version range or publisher; advisory / block new / disable; reason |
| Transfer | Old and new owner |

### Listing pipeline

```text
author: craft-ext prepare-release → GitHub Release on the author's repo
author: PR to the catalogue repo adding releases/<version>.toml
bot (no secrets, read-only): download, digest, validator, licence, source link,
     namespace, permission diff, WebAssembly smoke test in the reference host
merge:
  routine update (checks pass, no new permissions, same owner, not native) → bot merges
  first listing, wider permissions, new owner                            → quick human look
  any native listing or release                                          → two maintainers
publish job: catalogue.json regenerated and signed (TUF);
             auto-merged updates become visible after 48 hours
apps, launcher, CLI: fetch catalogue (no login) → browse → choose version
     → download from the author's URL → check pinned digest → store
```

### Authenticity (TUF, maintainer-controlled)

- The trust root ships inside the signed app. The online key lives only in a protected publish job that needs maintainer approval. Offline root keys: 2-of-3 on hardware keys, recovery rehearsed before launch.
- Publishers need no signing keys of their own (PC08). Source and namespace checks plus pinned digests remain.
- Expired metadata is shown as stale and cannot authorize installs. Verification failure never replaces trusted metadata. A clock that appears to go backwards pauses installs and updates; installed plugins keep working.

| Threat | Defence |
|---|---|
| Catalogue changed by someone with repo or hosting access | Only signed metadata accepted |
| Author account taken over, routine-looking update | 48-hour hold; the PR notifies the author; anyone can report |
| Asset swapped on GitHub | Pinned digest mismatch: refused and flagged |
| Old metadata replayed | Version and expiry |
| Online key leaked | Root keys rotate it without an app update |
| Account re-registered by someone else | Numeric owner ID differs: listing frozen |
| Release deleted upstream | Installed copies keep working; "no longer available from the author" |

### Revocation and incidents

| Severity | Effect | Who |
|---|---|---|
| Advisory | Warning | Any maintainer |
| Block new | No new installs, enables or updates | Any maintainer, immediately |
| Disable | Stops loading; running work stops at next host call; native process killed; export in progress fails | Any maintainer immediately; a second reviews within a day |

Apps check at start and every few hours when online; known revocations survive repair and restore; project data is never deleted. Account takeover: report → block new (and disable if malicious) → banners → recovery release gets a human look → public note.

### Policy

- Listings are refused or removed only for malicious or deceptive behaviour, impersonation, illegal content, failed bot checks or unverified namespace — **never** for quality, competing with built-ins, or being native.
- Appeals in the catalogue repository, decided by a different maintainer within about 14 days; emergency blocks stand meanwhile.
- A short disclaimer in the catalogue README and manager: community plugins are not built, hosted or guaranteed by the project (PC12).
- Ranking: filter → relevance tier → **official first within a tier** → GitHub stars (fetched centrally, no telemetry) → newer approval → ID.

The catalogue (P7) is not part of the MVP; the MVP uses manual file import.

---

## Part 15. Management in every app

- **One shared egui manager component** in craft-extensions, embedded in every app (Settings ▸ Extensions and a menu item) and in the launcher. In an app it shows that app; the launcher shows all apps. **The launcher is never required.**
- **CLI parity:** `<app>-cli plugin search|info|versions|install|setup|grant|revoke|update|remove`, using the same shared management actions as the GUI.

| Screen | Content |
|---|---|
| Browse | Search, filters (kind, works in this app, native), rows with name, summary, author, stars, labels, compatibility |
| Detail | Description, screenshots, permissions as sentences, per-app compatibility, versions, licence, source, release notes |
| Install and setup | Cancellable download, then "Set up in <app>": contributions and permissions on one sheet; large resources offered. Other apps show "New plugin ready to set up" |
| Installed | Per app: contributions, permissions with last use and revoke, version, update, remove, cost and memory |
| Updates | Badge; notes with more permissions, new owner and native highlighted; always asks |
| Problems | Failed, paused, incompatible or blocked plugins with reason and fix; safe mode; "Find the problem plugin"; Report plugin |

**Built-in search suggestions:** when a built-in search finds nothing, matching plugins from the cached catalogue appear. The query never leaves the machine; a setting turns it off.

**Accessibility:** everything reachable by keyboard, states exposed to screen readers, labels never rely on colour alone, text through `craft-i18n`. No account needed for anything public.

For the MVP, the manager uses existing egui components (PC09) and supports manual import, setup, enable, disable and remove.

---

## Part 16. Automation, jobs, performance and diagnostics

### Automation

- Three environments share operations, results and error codes: **fresh CLI** (uses that app's set-up plugins, never prompts, refuses to write a project open in a running app), **attached** (controls the running app), **server** (headless, later).
- Attaching uses a **user-only Unix socket or named pipe plus a per-session token**. No network port by default. This replaces today's localhost TCP ports.
- While any client is attached: "**<client> is controlling <app> · Stop**".
- Plugins have no sockets and never receive tokens.

### Jobs

One model: ID, owner, state (queued, running, succeeded, failed, cancelled), progress, outputs, error. CLI shows progress lines and cancels with Ctrl-C; MCP uses progress and cancel notifications; attached clients use `jobs list|wait|cancel`. Attached jobs survive disconnect. Output is written to a temporary name and renamed when complete.

| Exit code | Meaning |
|---|---|
| 0 | Done |
| 1 | Failed |
| 2 | Usage |
| 3 | Input required |
| 4 | Plugin missing or not permitted, with the command to fix |
| 5 | Cancelled |

### Performance

- Budgets from a CI benchmark set (blur, halftone, LUT, panel callback, geometry edit) on reference hardware.
- Validated-module cache, per-job instance reuse, parallel tiles, minimized copies, separate preview and final caches.
- Per-plugin cost and memory in the Installed screen, with an optional viewer overlay; authors see the same in `craft-ext dev`.
- Interactive and preview work outranks background export.

### Offline

Installed plugins, projects, placeholders, bake, cached catalogue and private storage all work offline. Install, update, fresh revocations and undownloaded large files need a connection.

### Diagnostics and safe mode

- Local, size-capped, rotated logs; plugin lines tagged and treated as untrusted; paths, URLs and content redacted by default.
- A **Plugin console** panel for authors.
- **Create support report** is explicit and reviewed by the person. No automatic upload, no crash telemetry.
- If the last start didn't finish loading plugins, the app offers "Start with plugins off?". Also `--safe-mode` and Help ▸ Restart with plugins off. **Find the problem plugin** bisects across restarts.

---

## Part 17. Themes and the Craft framework boundary

Themes are being designed separately and will be revisited later. The plugin system's job is only to **not make that work harder**.

### What is decided

- Themes are data packages: colours, fonts and density only; never auto-activated (T14).
- The manager uses existing egui components now and adopts shared theme tokens when that contract is ready (PC09).
- Token schema, ownership and timing are open.

### What the Craft framework plans (`craft-framework.md` at `479327ee`)

| Framework rule | Effect on plugins |
|---|---|
| Crate tiers: `craft` facade → families → UI library → egui-free engines (`craft-theme`, `craft-host`) | craft-extensions depends on craft-libs, never the reverse |
| **Services in, edits out**: features own no document state; they return `Begin/Update/Commit/Cancel/Set` edit events that the app applies to its own undo | Matches the plugin tool model (begin, update, commit, cancel) and one undo step per edit |
| **No feature opens its own window**; dialogs are values opened on `DialogHost` | Matches "the host draws everything" |
| **One egui fork revision** pinned by craft-libs; every app must use the same one | The shared manager must build against the same pin as each app |
| `craft-theme`: seeds → ramps → semantic tokens, theme file schema 1, contrast rules (4.5:1 text, 7:1 high contrast, 3:1 non-text) | A second theme format |
| `craft-theme-store`: suite theme library, `selection.json`, lock and revision protocol, import as copy | A second store for theme files |
| Size (density) is a suite setting, not a theme field | T14's "density" belongs to settings, not theme packages |

### Light-touch rules for the plugin side (proposals, not decisions)

1. **Do not grow `craft-ext-core`'s theme palette format.** It is a separate schema 1 (small palette, per-app overrides, 64 KiB cap, 4.5:1 check). When themes are revisited, a theme package should most likely carry a `craft-theme` file validated by `craft-theme`, rather than keep two formats.
2. **The manager uses egui's semantic styles, never hard-coded colours**, so token adoption later is a swap, not a rewrite.
3. **Plugin declarative UI (L2) renders through framework components** when they land, so plugin panels get tokens, accessibility and translation automatically.
4. **Canvas overlays use the framework's canvas-overlay token group** and meet the non-text contrast rule.
5. **Follow craft-libs' egui pin** in craft-extensions and in every app that embeds the manager.

These are listed so the theme work can choose; none changes the decided plugin design.

---

## Part 18. Phases, ownership and scaling

### The MVP (PC01)

EffectCraft, with **one manually installed L0 data package and one L1 WebAssembly transform**, including save/reopen and failure handling. Exact examples are deferred. Catalogue, launcher, L2, native helpers and the theme framework are outside it. Delivered as small independently reviewable PRs, with the whole EffectCraft L0/L1 workflow verified together (PC11).

### Delivery units

| Unit | What | Proof before moving on |
|---|---|---|
| P0 | Accept, name owners, plan hold lifting | Owners named |
| P1 | Store foundation, manifest schema 2 | Crash at each step, concurrent writers, old reader, update during leased job |
| P2 | Operations kit | Denial before side effect; management unreachable by plugins |
| P3 | Host runtime and hostile suite | Suite passes on every desktop target and web |
| P4 | `raster.filter@1`, SDK, `craft-ext` CLI | Two builds give the same digest |
| P5 | Manager component and `plugin` CLI (local installs) | Full lifecycle driven headless |
| P5b | L0 in every app | Manager and a theme or preset install work in each app |
| P6 | **EffectCraft pilot** (E0–E10 in Part 2) | GUI/CLI/MCP parity; export refuses missing plugin; no open TCP port |
| P7 | Catalogue: repo, bot, TUF, 48-hour hold, revocation, client fetch | Swapped asset and expired metadata refused |
| P8 | FigCraft: session style, panels, tools, canvas, consent UI | Panel/CLI parity; Escape cancels; one undo step |
| P9 | PhotoCraft: the same transform unchanged | Same output as EffectCraft |
| P10 | Native helper protocol and full-access label | Native crash contained; revocation kills process |
| P11 | Other apps by demand (FilmCraft next); themes revisited | Per-app proofs |
| P12 | Public launch: signed catalogue on Pages, keys held, rules, disclaimer, author guide, outside security review | Maintainers decide community enablement |

P1, P2 and P3 can run in parallel; P5b and P7 run alongside P6–P9. Pilots ship transform plugins only until P2 lands; the `third-party` feature stays off until P7.

```text
P0 → P1 store ───────────────────────────────┐
P2 ops ──────────────────────────────────────┤
P3 host → P4 contract/SDK → P5 manager ──────┼→ P6 EffectCraft → P8 FigCraft → P9 PhotoCraft → P10 native → P12 launch
E0–E4 EffectCraft prep from week 1 ──────────┘
P7 catalogue alongside P6–P9
```

### Ownership

| Part | Owner |
|---|---|
| Platform (craft-extensions) | Platform maintainers |
| Each adapter and its app contract | That app's maintainers |
| Catalogue repo, bot, keys, incidents | Catalogue maintainers |
| Shared domain contracts | Approval from maintainers of at least two consuming apps |
| Native releases | Two distinct maintainers each |
| Launch | Independent outside reviewer plus an accountable maintainer |

Names are not yet assigned. Reviewer roles (PC07) are deferred.

### How it stays scalable

- **One of everything:** one validator, one store, one dispatcher, one manager, one catalogue. Fixing a bug fixes it everywhere.
- **Generation over hand-writing:** contracts, SDK bindings, CLI help, MCP schemas and docs come from each app's registry.
- **Shared contracts:** one transform serves many apps.
- **Bot-first review:** humans look only at first listings, wider permissions, owner changes and native.
- **Levels:** each app adds only the level it needs.
- **No servers:** GitHub repositories, Actions and Pages. The ongoing cost is maintainer time and CI minutes.

---

## Workflows end to end

### W1. Author builds and releases a transform

1. `craft-ext create` from the transform template.
2. Write the function and parameters with macros; list permissions per app in `Craft.toml` (none for a pure effect).
3. `cargo test`, `craft-ext test`, `<app>-cli plugin test`.
4. `craft-ext dev --app effectcraft` to try it live in an isolated dev store.
5. `craft-ext prepare-release`: deterministic build, zip, catalogue record, diff against the last release.
6. Tag; the template Action uploads the GitHub Release.
7. Open the catalogue PR adding `releases/<version>.toml`.

### W2. Listing

1. The bot downloads the asset, checks the digest, runs the validator, licence, source link, namespace, permission diff and smoke test.
2. Routine update → bot merges; first listing, wider permissions or new owner → quick human look; native → two maintainers.
3. The publish job regenerates and signs `catalogue.json`; auto-merged updates appear after 48 hours.

### W3. Manual install (the MVP path)

1. Drag a package onto EffectCraft, or choose Import in the manager or `effectcraft-cli plugin install <file>`.
2. The store unpacks with limits, checks the manifest and every file's digest, records it as **Not reviewed**. Nothing runs; nothing is enabled.
3. "Set up in EffectCraft": choose contributions; grant permissions if any.
4. The adapter registers the contributions. A data package (e.g. presets) appears where that kind lives; a transform appears in the Effects menu.

### W4. Install from the catalogue (after P7)

1. Browse (cached, signed catalogue) → Detail → choose version.
2. Download from the author's GitHub Release; pinned digest; then the same steps as W3, labelled as a store listing.
3. Large resources: "Download now" or "When first needed".

### W5. Applying and rendering a transform

1. The person adds the effect: an Edit operation, one transaction, one undo step "Halftone · Ink & Grain".
2. Preview: the render pipeline asks the adapter; the adapter converts the frame to the contract, runs the module in `craft-ext-host` under the preview budget, validates and converts back, and the result is cached.
3. Failure: a trap or timeout shows a badge naming the plugin and passes the frame through.
4. Export: runs at final quality; any failure fails the export and leaves no file.

### W6. A session plugin edits the document (L2, after P8)

1. Person clicks a plugin panel button → event → invocation with `Plugin(...)` principal on a worker thread.
2. The plugin calls an operation → exposure check → authorization with resolved targets → availability.
3. One transaction, one labelled undo step; queued effects run after commit.
4. The panel returns a new tree; the host redraws.

### W7. Update

1. The manager shows the update with notes; wider permissions, new owner and native highlighted. Always asks.
2. The new release installs beside the old. Per-app choices carry over; new contributions start off.
3. Running effects switch on next use; panels reload when idle; exports finish on the leased old release.

### W8. Revocation

1. A maintainer issues block-new or disable in the catalogue (a second reviews a disable within a day).
2. Apps sync at start and every few hours.
3. Disable: loading stops; running work stops at the next host call; native processes die; an export in progress fails; projects keep data as placeholders.

### W9. Opening a project with a missing plugin

1. One prompt lists all missing plugins.
2. Each appears as a placeholder keeping all its data; preview passes through with a badge.
3. Export refuses until the plugin is installed and set up, or the result is baked.

### W10. An agent or CLI uses a plugin

1. A fresh CLI uses plugins already set up for that app; it never prompts.
2. Missing plugin or permission → exit code 4 with the exact command to fix.
3. An attached agent has the person's document authority and shows the "is controlling" banner; management needs in-app approval once per session.

---

## Decided, open and deferred

| Status | Items |
|---|---|
| **Decided** | Everything in Parts 1–18 that cites a T-number, PC-number or dated decision, including: removal of all existing plugin systems; three styles and native; L0/L1/L2; one operation path with markings; host-drawn UI; placeholders and export refusal; wasmi sandbox and budgets; native out of process with two-maintainer review; host-enforced per-app grants; reverse-domain IDs; shared files with per-app setup; install enables nothing; updates always ask; GitHub-hosted metadata catalogue with TUF and a 48-hour hold; manager in every app; socket attach; EffectCraft L0/L1 MVP; small PRs; short disclaimer; outside security review before P12 |
| **Open** | Owners and key holders; catalogue repo name; "verified against main"; detailed MVP acceptance and old-system cutover timing; pilot L0 timing (P5b); theme-token schema, ownership and timing; release-version propagation; exact disclaimer wording and legal-review timing; wasmi feature set; TUF client choice; budget numbers; egui accessibility coverage; web downloads; whether VST/CLAP/AU and AutoLISP remain an exception (raised 2026-11-12) |
| **Deferred by the user** | PC04 milestones and "version 7"; PC05 scanning policy; PC07 reviewer roles; exact MVP examples |

---

## Issues found in the 2026-11-12 review

An independent review (GPT-6.1 Sol), a theme-framework read (Haiku) and source checks found the following. **These are findings, not decisions.** Each needs discussion before it changes the design.

| # | Finding | Evidence | Affects |
|---|---|---|---|
| R1 | **Two theme formats.** `craft-ext-core`'s palette (schema 1) and the framework's `craft-theme` (schema 1, different model) would compete | `crates/ext-core/src/palette.rs`; framework §4.7, App. B | Part 17, P5b, P11 |
| R2 | **ABI memory ownership is incomplete.** Transform ABI v1 has `craft_alloc` but no reset or buffer-reuse rule, while instances are reused within a job | P4 Guest ABI v1; P3 instances | Part 7, P3/P4 |
| R3 | **Resolved by AR01:** user approved retaining all referenced releases; architecture/P1 now agree on that rule. Effective/pending representation remains open | architecture §11; P1 §5.4; AR01/AR02 | Part 13, P1 |
| R4 | **Wording corrected; deployment still open:** added headers cannot protect frozen schema-1 writers. P1's schema-2 refusal-boundary proposal requires consumer/deployment approval | architecture §11; P1-04; AR03 | Part 13, P1 |
| R5 | **Catalogue size limits.** Index caps are 2,000 entries and 1 MiB, while all approved releases stay listed | `crates/ext-core/src/index.rs` | Part 14, P7 |
| R6 | **Revocation latency stated two ways.** "Next host call" vs cross-process polling every 250 ms | architecture §8; P3 | Part 9 |
| R7 | **Post-commit effect failures.** A failed upload after a committed edit has no defined reporting | architecture §7 | Part 4, P2 |
| R8 | **Host-side memory.** Guest memory limits don't bound host buffers, decoded resources and pending outputs | architecture §9; P3 | Part 7, P3 |
| R9 | **"Permissions computed from imports" is overstated.** The plan correctly says `check` only warns | architecture §20; plan P4.5 | Part 11 |
| R10 | **Contract churn across ~15 apps.** Schema diffs alone don't catch semantic changes; inventories copied by hand will go stale | architecture §6 | Part 3, P4/P11 |
| R11 | **Native parity.** P10 covers raster helpers only; session, panels and the retained industry formats are not yet designed | P10 | Part 8 |
| R12 | **Plugin UI details.** Stable widget IDs, event ordering, stale responses and canvas accessibility are unspecified; framework components render each container twice per pass | architecture §15; framework §3.6 | Part 5, P8 |
| R13 | **Workflows carry authority.** They are data but run operations, so they must not install as harmless L0 data | architecture §17 | Parts 1, 9 |
| R14 | **CI and review cost.** Exhaustive runtime suites should run once centrally, with focused adapter proofs per app | P3; plan §3 | Part 18 |
| R15 | **Catalogue operations need rehearsal.** Metadata renewal, rotation, emergency revocation and absent key holders | architecture §13 | P7/P12 |

# P9: PhotoCraft (the same Halftone module, unchanged; old filter ABI removed)

Status: **plan, 2026-11-10.** Unit P9 of [the architecture](../architecture.md#22-delivery-plan). Each PR still needs its own approval.

**Code read:** PhotoCraft `main` `655295b2`. Nothing was built or run.

## 1. Goal and exit criteria

**Goal.** PhotoCraft becomes the second `raster.filter@1` host. The **unchanged** Halftone artefact from P4 (the same digest) runs in PhotoCraft as it does in EffectCraft. The old `pc_*` filter ABI and its registry are removed in the same change set.

**Exit criteria.**

| # | Criterion | Test |
|---|---|---|
| X1 | The same Halftone `.zip` digest passes `raster.filter@1` conformance in both apps; both reports record the same digest | `conformance::halftone_same_digest_both_apps` (run in each app's CI with the shared fixture) |
| X2 | The same normalised contract input gives the same output digest in PhotoCraft and in EffectCraft. Identical appearance is not promised | `contract::same_input_same_output_as_effectcraft` (golden digest shared with P6) |
| X3 | CMYK, Lab, indexed, bitmap, duotone and multichannel documents are refused for a `raster.filter@1` contribution before any call, with a reason | `raster::unsupported_mode_refused_before_call` |
| X4 | A smart filter whose plugin is missing becomes a placeholder; export refuses | `smart::missing_plugin_placeholder_and_export_refused` |

## 2. Code areas (verified at `655295b2`)

| What | Where |
|---|---|
| ABI | `crates/plugins/src/runtime.rs:14` `ABI_VERSION = 1`; exports `pc_abi_version`, `pc_manifest` (packed), `pc_alloc`, `pc_filter` (`type FilterArgs` `:316`); no imports (`:183-189`) |
| Budget pattern (moves to `ext-host` in P3) | `FUEL_SLICE` `:17`; `Limits` `:21`, defaults `:45-55`; `parallel_bands` `:63`; `Deadline` `:86-121` (native only); sliced `call` `:250-274`; code-section guard `:151` (called `:179`), test `tests/abi.rs:223` |
| Application | `crates/plugins/src/apply.rs:96` `apply(surface, canvas, selection, params)`: bands with overlap (`:104-116`), at most `MAX_BANDS` (`:74`), rayon chunks of `parallel_bands` (`:143-152`), selection limit and coverage blend (`:79`, `:188-214`), colour modes passed through as normalised f32 (`:23-30`) |
| Engine command | `crates/engine/src/cmd/plugin_cmds.rs:105` `run` (`plugin.run`) inside `s.edit(&label, …)`; targets are a channel, a raster layer (lock-transparency via `filters::keep_alpha`) or a smart object (records `SmartFilter { command: "plugin.run", … }`) |
| Silent skip | Smart-filter re-render `plugin_cmds.rs:152` `apply_to_surface`, called from `crates/engine/src/filters.rs:205`; **a smart filter whose plugin is not installed is silently skipped** |
| Automation | `crates/automation/src/security.rs`: TCP with a 256-bit token (`generate_token` `:79`), constant-time compare (`token_matches` `:104`), first-frame `auth` (`:112`); MCP over stdio. This is the model for `craft-attach` ([P6 A1](P6-effectcraft.md#4-ordered-pr-list)) |
| Store integration | None on `main`; no extensions or theme branch exists locally |

## 3. Layout

`photocraft-plugins` (an L1 adapter):
- a `ContractProvider` for `raster.filter@1`;
- PhotoCraft band and format conversion, using P4's shared conversion helpers: straight or premultiplied alpha, 8/16/32-bit to f32;
- selection blending kept in the host;
- smart-filter references stored as `PluginRef`;
- export preflight.

## 4. PR list (outline)

| PR | Scope | Size | Test that fails first | Depends |
|---|---|---|---|---|
| PH0 | Inventory and legacy fixtures (`pc_*` plugins, smart objects with plugin filters) | S | — | — |
| PH1 | Raster conversion: alpha mode, precision, selection, overlap and bands into `raster.filter@1` requests; refusal of unsupported modes | M | `raster::unsupported_mode_refused_before_call`; `raster::band_overlap_equals_whole` | P4-02 |
| PH2 | Smart-filter references and export preflight; placeholder instead of the silent skip (`filters.rs:205`) | M | `smart::missing_plugin_placeholder_and_export_refused` | P6-E6 pattern |
| PH3 | Markings table and mandatory invocation, using P6's sequence; automation moved onto `craft-attach` | M ×3–4 | as in P6 | P2, P6-A1 |
| PH4a | `photocraft-plugins` adapter, dormant | M | `adapter::halftone_renders` | PH1, PH2 |
| PH4b | **Atomic cutover**: wire the adapter, manager and `plugin` CLI (L0 included); delete the `pc_*` runtime and registry (`crates/plugins` old ABI) and the plugin menu (`crates/ui-egui/src/menus/plugin_ui.rs`) | L | `conformance::halftone_same_digest_both_apps` | PH4a, P5 |

## 5. Data formats

- Smart filters store `SmartFilter { command: "plugin.run", params: {…} }` today. They become references in the P6 `PluginRef` shape, written beside the smart object.
- Old-ABI filters open as placeholders, with "from a removed plugin system".

## 6. Tests

- **Cross-app golden digest:** one fixture input in `ext-conformance`, run by both apps' CI.
- **Band invariance**, at PhotoCraft's real band sizes.
- **Hostile suite** through the adapter.

## 7. Risks and decisions

- **D1.** Confirm the v1 formats ([P4 D2](P4-contracts-sdk.md#7-risks-experiments-and-decisions)). Grayscale documents: refuse, or allow a declared RGBA round trip.
- **R1.** PhotoCraft's colour pipeline may give different *appearance* than EffectCraft for the same plugin. The promise covers identical normalised input only. Say so in the release notes.

## 8. Effort and parallel work

- **Counts:** about 9 PRs.
- **Parallel:** PH0–PH2 can run beside P8. PH4b follows P6 E8b.

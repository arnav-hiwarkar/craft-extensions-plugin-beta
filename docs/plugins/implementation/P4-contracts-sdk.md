# P4: First contract `raster.filter@1`, guest SDK, `craft-ext` CLI, Halftone example

Status: **build-ready plan, 2026-11-10.** Unit P4 of [the architecture](../architecture.md#20-sdk-and-author-experience); deepens [implementation plan section 6, P4](../implementation-plan.md#p4-contracts-sdk-cli). Each PR still needs its own approval. The contract itself also needs approval from the maintainers of two consuming apps (architecture section 23).

**Code read** (to fit the contract to the first two consumers; nothing was built or run):

Review qualification, 2026-11-12: [AR10–AR13/AR16](../design-sessions/architecture-review-2026-11-12.md) identify unresolved request reset/buffer ownership, typed resource/migration/color context, descriptor projections/build inputs, L0 templates and adapter-generator ownership. Affected ABI/schema examples are not a fully frozen MVP contract. Halftone remains a platform example; exact MVP examples remain deferred.

| App | Commit | Files |
|---|---|---|
| EffectCraft | `origin/main` `fb62b9d7` | `crates/effects/src/lib.rs`, `crates/effects/src/plugin.rs`, `crates/project/src/props.rs`, `docs/plugins.md` |
| PhotoCraft | `main` `655295b2` | `crates/plugins/src/runtime.rs`, `apply.rs` |

## 1. Goal and exit criteria

**Goal.** An author writes a Rust raster filter once. It builds to a WebAssembly module whose descriptors travel in a custom section. `craft-ext` checks, tests, packs and prepares it deterministically, and the same module runs in any app that passes the `raster.filter@1` conformance suite.

**Exit criteria (architecture section 22: "two builds give the same digest"; milestone M4).**

| # | Criterion | Proving test |
|---|---|---|
| X1 | Halftone built twice, from different paths on different runners, gives the same `.wasm` digest and the same `.zip` digest | CI job `repro-halftone`: two builds in disposable checkouts at different paths, digests compared separately |
| X2 | Halftone passes the `raster.filter@1` suite in the reference host: determinism, tile invariance, formats, parameter ranges, cancellation | `conformance::raster_filter::halftone` |
| X3 | An unlabelled control, or a descriptor that disagrees with the contribution, does not build | `craft-guest` `compile_fail` tests: `unlabelled_param`, `contribution_mismatch` |
| X4 | `craft-ext check` reports undeclared required permissions with the exact line to add, and never edits `Craft.toml` | `ext-cli::check::prints_line_never_writes` |
| X5 | `craft-ext dev` never writes to the production store or reuses production grants | `ext-cli::dev::isolated_store` |
| X6 | A removed contribution ID, or a state change without a migration, is reported by `prepare-release` | `ext-cli::prepare_release::reports_removed_ids_and_state_changes` |

## 2. What the two first consumers do today (verified)

| | EffectCraft `fb62b9d7` | PhotoCraft `655295b2` |
|---|---|---|
| Pixel model | Premultiplied f32 RGBA, processed in place (`docs/plugins.md`; `PluginFrame` `crates/effects/src/plugin.rs:243`) | Normalised interleaved f32 in **every** colour mode (Bitmap 0, Gray 1, Indexed 2, RGB 3, CMYK 4, Multichannel 7, Duotone 8, Lab 9; `apply.rs:23-30`); depth and alpha flag packed into `format` |
| Region | `padding` (API 2, `trait EffectPlugin` `plugin.rs:370-391`) | Bands one tile row tall, widened by manifest `overlap` (capped by `MAX_OVERLAP`); half the memory cap per band (`apply.rs:104-116`) |
| Params | `ParamSpec {id, name, default, ui: ParamUi}` (`effects/src/lib.rs:156`); `ParamUi` has 17 variants (`project/src/props.rs:12`); keyframes evaluated by the host | JSON params passed through (`pc_filter(…, params_ptr, params_len)`, `runtime.rs:316`) |
| Context | `EffectCtx {params, time, layer_size, seed, adjustment, env}` (`effects/src/lib.rs:610`) | width, height, channels, format |
| ABI | v1: `ec_api_version`, no imports (`crates/plugin/src/wasm.rs`) | v1: `pc_abi_version`, `pc_manifest` (packed `ptr \| len<<32`), `pc_alloc`, `pc_filter`, no imports (`runtime.rs:14, :194-209`) |
| Selection | Not applicable (effects apply to layers) | Output limited to the selection bounds and blended by coverage (`apply.rs:79, :188-214`), in the host |

**Consequence for the contract.**
- **Core:** RGBA f32, with the alpha mode declared and the colour space named by the host. A padding/region request. Host-evaluated parameters. Time, seed and render scale in the context. Selection blending stays in the host, so plugins never see selections.
- **Out of v1, refused honestly:** CMYK, Lab, indexed, bitmap, duotone and multichannel. PhotoCraft must refuse them for a `raster.filter@1` contribution (P9-01) instead of passing them through as it does today.

## 3. Module layout and API sketches

```text
crates/ext-contracts/src/
  lib.rs              contract registry; ContractRef {id, major, minor}; version rules; extension negotiation
  inventory.rs        re-export of ext-ops inventory encoding + domain-contract sections
  raster/filter_v1.rs raster.filter@1: Format, AlphaMode, Region, Request, ParamKind, Descriptor; host-side codec
  generate.rs         inventory → guest crate source (used by xtask; output committed)
crates/guest/                      craft-guest          (author-facing; 0.x like the platform)
  src/params.rs       ParamKind, Range, Unit, Label (i18n key + default text), ParamValue
  src/abi.rs          THE ONE MODULE WITH unsafe (named exception): alloc, slices from (ptr,len), last_error buffer
  src/section.rs      custom-section payload encoding (canonical JSON)
  src/result.rs       GuestError, codes
crates/guest-macros/               craft-guest-macros   (proc macros: #[craft::transform], #[derive(craft::Params)])
crates/guest-raster/               craft-guest-raster   GENERATED; crate major = contract major (1.x.y)
crates/ext-cli/                    craft-ext            (binary)
  src/{create,check,inspect,pack,verify,test,prepare_release,dev}.rs
examples/halftone/                 not a workspace member; its own Cargo workspace + rust-toolchain.toml
templates/transform-raster/        what `craft-ext create` copies
```

### Frozen early

The contract and the ABI are frozen once two app maintainers approve P4-02.

```rust
// ext-contracts/src/raster/filter_v1.rs
pub const CONTRACT: ContractRef = ContractRef::new("raster.filter", 1, 0);
pub enum AlphaMode { Premultiplied, Straight }
pub enum Format { Rgba32f { alpha: AlphaMode } }                 // v1 core: one layout, two alpha modes
pub enum ColourSpace { HostWorking { linear: bool } }            // named by the host; plugin may require linear
pub struct Rect { pub x: i32, pub y: i32, pub w: u32, pub h: u32 }
pub enum EdgeMode { Transparent, Clamp }                         // what the host puts outside the source
pub struct Request {                                             // control plane, JSON ≤ 64 KiB
    pub contribution: String, pub params: BTreeMap<String, ParamValue>,
    pub time: Time, pub seed: u64, pub scale: f64, pub quality: Quality /* Draft | Final */,
    pub output: Rect, pub input: Rect, pub format: Format, pub colour: ColourSpace, pub layer_size: [u32; 2],
}
pub struct Descriptor {                                          // from the custom section
    pub id: String, pub label: Label, pub category: Option<String>,
    pub params: Vec<ParamDescriptor>, pub accepts: Vec<Format>, pub requires_linear: bool,
    pub edge: EdgeMode, pub padding: Padding /* Fixed(u32) | PerParam{param, factor} | Dynamic */,
    pub tile_tolerance_ulps: u32,                                // 0 = exact; declared, then tested
    pub state_schema: Option<u32>,                               // stateless in v1 core; reserved
}
pub enum ParamKind { Number { min: f64, max: f64, default: f64, unit: Unit, step: Option<f64> },
                     Angle { default: f64 }, Percent { default: f64 }, Pixels { default: f64, min: f64, max: f64 },
                     Point { default: [f64; 2] }, Colour { default: [f32; 4] }, Checkbox { default: bool },
                     Choice { options: Vec<Label>, default: u32 }, Seed }
pub struct ParamDescriptor { pub id: String, pub label: Label, pub kind: ParamKind, pub animatable: bool }
```

- **`ParamKind` maps onto EffectCraft's `ParamUi`** (Number, Slider, Percent, Angle, Pixels, Point, Color, Checkbox, Popup) and onto a PhotoCraft dialog.
- **Units.** `Pixels` values reach the plugin already multiplied by `scale`: the host converts spatial units (architecture section 6, host obligation 3).
- **Labels.** `Label { key, text }`. The host shows `text` until a translation pack supplies `key` (P5-L0 translations). An empty `text` fails to compile (X3).

### Guest ABI v1 (core WebAssembly; no imports)

| Export | Signature | Meaning |
|---|---|---|
| `memory` | — | exactly one memory |
| `craft_abi` | `() -> i32` | `1` |
| `craft_alloc` | `(len: i32) -> i32` | guest allocates `len` bytes, alignment 16; `0` = failure |
| `craft_raster_filter_padding` | `(req_ptr, req_len) -> i32` | ≥ 0 pixels needed around `output`, for `Padding::Dynamic`; < 0 is an error code |
| `craft_raster_filter_render` | `(req_ptr, req_len, in_ptr, in_len, out_ptr, out_len) -> i32` | 0 ok; < 0 error code; message via `craft_last_error` |
| `craft_last_error` | `() -> i64` | packed `ptr \| len << 32`, UTF-8, ≤ 4 KiB |

**Custom sections** (read by `pack` and by the host; never executed):
- `craft.abi` holds `{"abi":1,"contracts":["raster.filter@1"],"sdk":"craft-guest 0.1.0"}`.
- `craft.contributions` holds the canonical JSON array of `Descriptor`.

**Buffers.**
- The host calls `craft_alloc` for the request, input and output, writes the request and the input, and calls `render`.
- Input and output are tightly packed row-major RGBA f32, little-endian. Input size is `input.w × input.h × 16` bytes; output size is `output.w × output.h × 16`.
- Lengths are checked against `u32` with checked multiplication **before** allocation. A region that would need more than the profile's band size is split by the host (PhotoCraft's band pattern) or refused.

**No imports in v1.** The guest cannot block, so cancellation and time are enforced by fuel slices (P3). Logs travel back in the error message only. A `craft.log` import can be added in a minor version as an optional extension.

### Provisional (SDK ergonomics)

```rust
#[derive(craft::Params)]
struct Halftone {
    #[param(label = "Cell size", min = 2.0, max = 200.0, default = 8.0, unit = "px")] cell: f64,
    #[param(label = "Angle")] angle: craft::Angle,
    #[param(label = "Shape", choices = ["Dot", "Line", "Square"])] shape: Shape,
}
#[craft::transform(contract = "raster.filter@1", id = "halftone", label = "Halftone",
                   apps = ["effectcraft", "photocraft"], accepts = "rgba32f-premultiplied", padding = 0)]
fn halftone(req: &craft::raster::Request<Halftone>, input: craft::raster::Image<'_>, out: &mut craft::raster::ImageMut<'_>) -> Result<(), craft::Error>;
```

The macro emits the exports (`#[unsafe(no_mangle)] extern "C"`) and the custom section (`#[unsafe(link_section = "craft.contributions")]`). The author writes no `unsafe`.

## 4. Ordered PR list

| PR | Scope | Size | Files | Test that fails first | Mutants | Depends |
|---|---|---|---|---|---|---|
| P4-00 | **Experiment, a report.** The smallest guest with one export and one custom section, built with the pinned toolchain. Record: (a) whether `#![forbid(unsafe_code)]`, or the craft-extensions workspace's `unsafe_code = "forbid"`, rejects macro-generated `#[unsafe(no_mangle)]` and `#[unsafe(link_section)]`; (b) the target features to set in `.cargo/config.toml` so the module matches P3-00's profile; (c) module size; (d) bit-identical builds from two paths with `--remap-path-prefix` | — | `docs/experiments/p4-smallest-guest.md` | — | — | P3-00 |
| P4-01 | `ext-contracts` crate: `ContractRef`, version rules (minor adds optional items; major = new contract), extension negotiation; inventory sections for domain contracts on top of P2-08; generator with a committed output and a drift check | M | `crates/ext-contracts/**`, `xtask/` | `contracts::regeneration_is_byte_identical`; `contracts::removal_needs_major` | "minor accepts removal" | P2-08 |
| P4-02 | **`raster.filter@1` specification** in `docs/contracts/raster.filter@1.md` plus `filter_v1.rs` and its conformance suite. Reviewed by EffectCraft and PhotoCraft maintainers | M | `docs/contracts/`, `ext-contracts/src/raster/`, `ext-conformance/src/raster_filter.rs` | `raster_filter::whole_equals_tiled` (a reference identity filter and a box blur written in the test, tiled at random splits); `raster_filter::nan_output_is_invalid`; `raster_filter::cmyk_refused_before_call` | "tile check skipped"; "NaN accepted" | P4-01, P3-06 |
| P4-03 | `craft-guest` and `craft-guest-macros`: params derive, labels required, descriptor emission, `abi.rs` under the named `unsafe` exception (section 7, D1) | M | `crates/guest*/**` | `compile_fail` `unlabelled_param`; `compile_fail` `contribution_mismatch` (an `apps` entry unknown to the manifest); `section::canonical_json_round_trip` | "empty label accepted" | P4-00, P4-02, D1 decided |
| P4-04 | Generated `craft-guest-raster` 1.0.0 plus the **Halftone** example (original algorithm, original test images, recorded in `ATTRIBUTION.md`) | M | `crates/guest-raster/**` (generated), `examples/halftone/**`, `fixtures/images/` | `halftone::passes_raster_filter_suite` in the reference host | — | P4-03 |
| P4-05 | `craft-ext inspect` and `check`: validator from `ext-core`, admission from `ext-host`, custom section against manifest, import-based permission **warnings** with the line to add | M | `crates/ext-cli/src/{inspect,check}.rs` | `check::prints_line_never_writes`; `check::section_manifest_mismatch_fails` | "check writes Craft.toml" | P4-04, P1-10 |
| P4-06 | `craft-ext pack` and `verify`: deterministic zip (sorted entries, fixed 1980-01-01 timestamps, mode 0644, pinned `zip` version and compression level), manifest v2 from `Craft.toml` plus the custom section | M | `pack.rs`, `verify.rs` | `pack::same_input_same_bytes` (pack twice, byte-compare); `verify::tampered_zip_refused` | "entries unsorted"; "mtime from disk" | P4-05 |
| P4-07 | `craft-ext test`: runs the contract suite and literal fixtures in the reference host | M | `test.rs` | `test::reports_tile_mismatch_with_coordinates` | — | P4-06 |
| P4-08 | `craft-ext create`: copies `templates/transform-raster` with `rust-toolchain.toml` and `.cargo/config.toml` from P4-00 | S | `create.rs`, `templates/` | `create::template_builds_and_checks_clean` (CI) | — | P4-07 |
| P4-09 | `craft-ext prepare-release`: build, check, test, pack, SBOM (`cargo metadata` licences checked against the allowlist), catalogue record draft, diff from the previous release (removed IDs, state-schema changes without migration, wider permissions). **Never publishes** | M | `prepare_release.rs` | `prepare_release::reports_removed_ids_and_state_changes`; `prepare_release::never_touches_network` | "diff ignores permissions" | P4-06 |
| P4-10 | `craft-ext dev --app <app>`: isolated development store (`CRAFT_EXT_DIR` pointed at a dev folder), label `Development`, separate grants, start or attach to the app, stream logs, reload on change | M | `dev.rs` | `dev::isolated_store`: a dev install is invisible to a reader of the production store, and production grants are not visible to dev | "dev uses default root" | P4-06, P1-12, P6-E4 (attach) for the attach half |

**Order.** P4-00 → P4-01 → P4-02 → P4-03 → P4-04 → P4-05 → P4-06 → {P4-07, P4-09} → P4-08; P4-10 last.

**Changed from the reconciled plan:**
1. **The ABI has no imports in v1.** This matches both existing raster ABIs (EffectCraft and PhotoCraft) and keeps admission trivial. Cancellation is enforced by fuel slices, so the guest needs no callback.
2. **Halftone lives in its own Cargo workspace under `examples/`.** The craft-extensions workspace forbids `unsafe_code`, so a member crate could not take the macro exports if P4-00 shows the lint fires.

## 5. Data formats

### 5.1 `Craft.toml` (author source; never shipped as is)

```toml
[package]
id = "io.github.ink.grain"
name = "Ink & Grain"
version = "1.2.0"
license = "MIT OR Apache-2.0"
publisher = "Ink"
source = "https://github.com/ink/grain"

[apps.effectcraft]
permissions = { required = [], optional = [] }
[apps.photocraft]
permissions = { required = [], optional = [] }

[[dependencies]]
id = "io.github.ink.luts"
min = "1.0.0"
below = "2.0.0"
```

`pack` combines it with the module's `craft.contributions` section into manifest v2 ([P1 section 5.2](P1-store.md#52-manifest-schema-2-craft-extensionjson-written-by-craft-ext-pack)). The `contracts` per app come from `craft.abi`, so authors never type them.

### 5.2 Contract inventory (`raster.filter@1.json`)

This is the same encoding as the operation inventory ([P2 section 5.1](P2-ops.md#51-operation-inventory-appcontractmajorminorjson-shared-with-p4)), with a `domain` section listing:
- the exports and their signatures;
- the request schema;
- formats, extensions and limits.

The generator writes `crates/guest-raster/src/generated.rs` from it. CI fails if regenerating it changes any byte.

### 5.3 Determinism and tolerance

- **Finite output alone is not a determinism proof.** The host refuses nonfinite output, but a guest can carry mutable state between calls or observe intermediate NaN bits before returning finite values. The [WebAssembly numeric specification](https://webassembly.github.io/spec/core/exec/numerics.html) describes NaN nondeterminism; the selected wasmi profile and complete request/state/reset contract must establish the required behavior. P3-00 (e) and conformance must exercise fresh/reused instances, tile order, parallel schedules and full input/color/resource context on each claimed target (AR10).
- **Where tolerances remain:**
  - **Tile invariance** is a property of the plugin's algorithm, so the plugin declares `tile_tolerance_ulps` (default 0) and the suite tests that value.
  - **Host conversions** (8- and 16-bit to f32, alpha mode changes) live in shared, fixture-tested helpers. The suite compares their outputs exactly against golden data.

  Exact cross-platform equality remains a contract proof obligation. It is not inferred solely from interpreter execution or rejection of nonfinite output.
- **Relaxed SIMD** is non-deterministic and stays disabled for transforms (P3 profile).

### 5.4 Fixtures

- `fixtures/images/*.exr` or raw f32: original, generated test images (gradients, noise, edges, alpha ramps) with their generator script, recorded in `ATTRIBUTION.md`.
- `fixtures/expected/halftone-*.raw.sha256`: expected output digests.

## 6. Test strategy

- **Unit:** the algorithm crates under native `cargo test` (the author level).
- **Contract** (reference host):
  - determinism: same complete request/digest across fresh and reused instances, reordered tiles and parallel schedules; include mutable-global, memory-reset, allocation-lifetime and intermediate-NaN cases;
  - tile invariance: random splits, plus 1-pixel tiles at edges;
  - formats: refused before the call when not accepted;
  - parameter ranges: values outside `min`/`max` are clamped by the host or refused, as the contract text decides;
  - cancellation: a guest loop is stopped within one slice;
  - invalid output: wrong length, NaN.
- **Compile-time:** `trybuild` (MIT/Apache) `compile_fail` cases for the macros.
- **Reproducibility:** the CI job from X1; separate digests for the `.wasm` and the `.zip` show where nondeterminism enters.
- **Hostile:** a malformed custom section (oversize, invalid UTF-8, duplicate) fails `check` and admission. It never panics.
- **Benchmarks:** Halftone and a 9×9 box blur at 256², HD and 4K in the reference host, compared with a native Rust build of the same algorithm. This feeds P3-08 and E8 sign-off.

## 7. Risks, experiments and decisions

| # | Risk | Experiment | Before |
|---|---|---|---|
| R1 | Macro-generated exports trip `forbid(unsafe_code)` | P4-00 (a) | P4-03 |
| R2 | Rust's default wasm target features are not all accepted by wasmi 2.0 | P3-00 (a, b) and P4-00 (b) | P4-08 template |
| R3 | `pack` output varies with the `zip` crate's version or compression | Pin `zip` and the level; test `same_input_same_bytes` on two OSes | P4-06 |
| R4 | Halftone too slow at 4K | P3-08 | E8 |
| R5 | `check` overstates what it can infer (reconciled plan risk 13) | Not applicable to transforms (no imports); session contracts in P8 | P8 |

**Decisions for maintainers:**
- **D1, the named `unsafe` exception** (house rules, Rust block, rule 1). It would cover only:
  - `#[unsafe(no_mangle)]` exports;
  - `#[unsafe(link_section)]` custom sections;
  - building slices from `(ptr, len)` inside `craft-guest::abi`;
  - the code `craft-guest-macros` generates.

  Each `unsafe` block would carry a `// SAFETY:` note. The host side stays free of `unsafe`. The craft-extensions rules file would name the exception.
- **D2, v1 formats.** RGBA f32 with a declared alpha mode only (recommended), or also grey plus alpha. PhotoCraft's grayscale documents would then need an RGBA round trip, which is not silent conversion only if the contract declares it.
- **D3, out-of-range parameters.** The host clamps them (recommended for keyframe overshoot) or refuses them.
- **D4, guest crate versioning.** The generated crates take the contract major as their own major (1.x), as an explicit exception to the shared 0.x line (implementation plan section 3, "Versions").
- **D5, SDK distribution.** Git tags until crates.io publication is authorized.

## 8. Effort and parallel work

| PR | Effort | Parallel with |
|---|---|---|
| P4-00 | 2–3 days (experiment) | P3-01 |
| P4-01, P4-02 | M each (P4-02 needs two app reviewers) | P3-04..06 |
| P4-03, P4-04 | M each | P1-12, P2-06 |
| P4-05..07, P4-09 | M each | P5-01 |
| P4-08 | S | P5-02 |
| P4-10 | M | P6-E5 |

That is 10 PRs (1 S, 9 M) plus the experiment report.

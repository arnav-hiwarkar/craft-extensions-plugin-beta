# Plugin architecture roadmap

Date: 2026-11-18. This is the build order for [architecture.md](architecture.md). Each phase ends when
its acceptance checks pass. The checks are deterministic tests or benchmarks in CI; the benchmarks run
on a fixed runner, take the median of 5 runs, and fail on a 20% regression. The phases need the owner
decisions listed for them in [decisions.md](decisions.md).

Effort assumes two engineers plus AI agents, one build owner, cheap scoped checks on the Mac, and
approved GitHub Actions runs for full builds, GPU and OS-sandbox suites. All effort figures are
estimates. Altogether this is **14–20 months** to the full architecture. The earlier capability
program's "Phase 1 in 4–8 weeks" was two to three times too optimistic
([research/10-reconciliation.md](research/10-reconciliation.md) §2.4).

## Dependency graph

```text
P0 quick wins + safety ─────┬──────────────────────────────────────────────┐
                            │                                              │
P1a craft-command + Meta ───┼─> P1b Runtime<A> + 3 ports + bridge ─> P2a worker + views ─> P2b craft-gpu + ext-gpu L1
(beta apps)                 │        │                                   │                   │
                            │        └─> P1c net/secrets/events/UI 1.3   │                   │
                            │                                            v                   v
P3a app prerequisites ──────┴──────────────────────> P3b new adapters (MaterialCraft/TextureCraft, VectorCraft, AniCraft, FilmCraft, LightCraft, DesignCraft)
(tolerant load, open registries, Meta, change events)                    │
                                                                         v
                                              P4 GPU L2 passes, craft-nn, formats, parameter kinds, overlays
                                                                         │
P5a SoundCraft prerequisites ───────────────────> P5b craft-audio-rt + Wasm audio + native bridge
                                                                         │
                                                                         v
                                              P6 hardening: TUF, provenance, effects probe, Windows, GPU L3, graph editors, WIT review
```

P3a and P5a are app work with no platform dependency. They can start as soon as the owner authorizes
work on those repos' `plugin-beta` branches. The branches exist on GitHub and Forgejo as of
2026-11-18.

## P0 · Quick wins and safety (2 weeks)

Repos: craft-extensions, effectcraft, photocraft, figcraft, plus the owners of the six apps with
open control ports.

| Work | Acceptance |
|---|---|
| Drop wasmi `portable-dispatch`; enable `simd`; zero-copy `Filter::run` (one write into guest memory, kernels on views, validation fused at commit) | 1080p per-pixel loop end to end ≤15 ms (52.0 today); `craft.call` small op ≤4 µs (10.5 today) |
| Per-run tokens on every control port, withheld from helper environments | Connecting without the token is refused; a scan of a helper's environment and file descriptors finds no token |
| Trust sheets ignore automation input | Tests that MCP `answer_dialog`, `press_key` and UI clicks cannot change a grant |
| Correctness bugs found by reading the code, each fixed after a failing test: FigCraft read-only guard; `layer.select` permission mismatch; invisible FigCraft `ui.notify`; dropped guest error text; idle repaint; PhotoCraft tilt and rotation; PhotoCraft synchronous tool wait | One regression test per bug |
| Dock EffectCraft plugin panels | A plugin panel id round-trips through a saved workspace |

## P1 · Metadata, one runtime, breadth (14–18 weeks)

Repos: craft-libs (new `craft-command`, `craft-preserve`), craft-extensions, effectcraft, photocraft,
figcraft.

| Work | Acceptance |
|---|---|
| **P1a.** `craft-command`, with VectorCraft's `apischema` moved in. `Meta` becomes mandatory in the three beta apps: EffectCraft 757, PhotoCraft 954 + 146 shell, FigCraft 630. A backfill tool drafts effects and schemas from the census; humans review | `every_command_has_meta` passes in each app; property test of `exposure()` over every effect combination; `api lock --check` clean; ≥60% of edit and query commands have real schemas, the rest are agent-only |
| Effect choke points (file, network, process, clipboard, preferences) with `disallowed-methods`; dispatcher effect mask; origin carried through nesting and post-commit writers | Mock sinks fail on a plugin origin from any exposed command; any re-entrant dispatch must declare `Dispatch` |
| **P1b.** `ext-runtime` with `CraftApp`, `DocumentHost`, gate, transactions, events, jobs, grants and UI limits. Port FigCraft, then EffectCraft, then PhotoCraft. Add `app.run`, `app.batch`, `app.commands` and about 30 canonical ops | `conformance::run::<A>()` green for all three with identical results on the canonical tranche. A generated write-set scope test per exposed command: out-of-scope ids return `-2` and the document hash is unchanged, including `select.all` followed by an implicit-target edit. Undo test: rollback restores the hash and commit adds exactly one step. Duplicated adapter code down by at least 50% in lines |
| **P1c.** `settings.*`, `net.fetch`, `secret.*`, `auth.oauth`, events from existing change sources, `host.capabilities`; UI 1.3 patches, value bindings and live controls; `HostUi` in three shells; generated Rust bindings; activity log; risk levels | Local test server: undeclared host, plain HTTP, IP literal, redirect off the list, and DNS to 10/8, 127/8 or 169.254/16 are all refused; one consent per plugin; network refused in render and preview contexts; `ui.patch` of 100 nodes ≤0.3 ms host time; `bindgen --check` clean; renderer tests count zero guest calls per frame |

## P2 · Worker, data plane, GPU L1 (12–16 weeks)

Repos: craft-extensions (`ext-worker`, `ext-gpu`), craft-libs (`craft-gpu`), effectcraft, photocraft.

| Work | Acceptance |
|---|---|
| **P2a.** Wasmtime worker on macOS (XPC with App Sandbox), then Linux (Landlock and seccomp). Shared-memory arenas; AOT cache with HMAC. `raster.filter@2` with region of interest, tiles, apron, extra inputs, `frame.at`, stepped state and output geometry; image handles. EffectCraft `EffectHost` passthrough; PhotoCraft tile-native pixels with colour space and depth | **Sandbox:** file, network, exec and persistence denial from the worker; the infinite loop and revocation timings below; pooled instances zeroed; a tampered cache refused. **Latency:** IPC p50 ≤5 µs, p99 ≤20 µs; 1080p loop end to end ≤1.1× the in-process Cranelift figure; 100 installed plugins add ≤5 ms to start. **Build:** `cranelift-codegen` absent from app dependency trees |
| **P2b.** `craft-gpu` and `ext-gpu` L1 with fusion. EffectCraft `walk.rs` plugin node; PhotoCraft canvas preview; timestamp queries everywhere | **Validation:** L1 refuses WGSL with storage, workgroup memory, atomics or entry points; a run-time pipeline from unpackaged WGSL is refused. **Hang safety:** `loop {}` completes or aborts within budget; device loss recovers and records a strike; a crash-journal entry quarantines the digest on restart. **Isolation:** a pool-reuse sentinel never reaches another package. **Speed:** a brightness pass in an EffectCraft GPU chain does zero readbacks and takes ≤0.5 ms of GPU at 1080p 16F |

P2a timing targets:

| Event | Target |
|---|---|
| Infinite loop returns | Within its deadline plus 50 ms |
| Revocation traps the running plugin | Within 260 ms |

## P3 · App prerequisites and new adapters (6–9 months, two streams)

Repos: each app's `plugin-beta` branch; craft-libs (`craft-graph`, `craft-mesh`).

Each adapter starts on `Runtime<A>` and the bridge. Its breadth on day one is its classified registry.

| Order | App work first (P3a) | Adapter (P3b) | Acceptance |
|---|---|---|---|
| 1 | **MaterialCraft:** `Meta` (133), open node registry, SBS dependency URLs, change events. **TextureCraft:** tolerant decoding, plugin variants, compositor path, `Meta` (183) | `resource.node-library@1` (MaterialCraft, data only), then `graph.node@1` WGSL in both | One node package gives golden images within ΔE ≤1 in both apps; a TextureCraft document with an unknown effect round-trips byte-identical; a missing node renders its durable output with a badge |
| 2 | **VectorCraft:** convert `reviewed.rs` (718 ids) into `Meta`; appearance registry with durable output | `vectorcraft.session@2`, `vector.effect@1`, formats; legacy shim | Every legacy kind and host import has a conformance-tested equivalent; shim removal date recorded |
| 3 | **AniCraft:** `Meta` (365), egui 0.37, plugin panel, open filter registry that keeps unknown entries | `anicraft.session@2`; ABI v1 bridge | Its four v1 sample plugins run through the bridge; the old namespace policy test is replaced by `api.lock.json` |
| 4 | **FilmCraft:** plugin data, unknown preservation, external effect registry, refusal of missing effects, `Meta` (686) | `filmcraft.session@2`, `raster.filter@2` on clips, transitions | An unknown effect id no longer passes through silently |
| 5 | **LightCraft:** `Meta` (391), sidecar writes batched and attributed, export hooks | `lightcraft.session@2`, `export.service@1` | One XMP write per photo per plugin event, carrying the plugin origin |
| 6 | **DesignCraft:** `Meta` (567), file-writing "queries" reclassified, string preflight kinds, change sets | `designcraft.session@2`, `document.check@1` | `file.exportIdml` is classified as a file write and is absent from the plugin lock |

## P4 · Depth (10–14 weeks)

Repos: craft-extensions, craft-libs (`craft-nn`), photocraft, effectcraft, craft-plugins.

| Work | Acceptance |
|---|---|
| ONNX spike (tract and candle-onnx on five models), then `craft-nn` with candle and LightCraft WGSL convolutions; suite model store; `nn.run` jobs | 2× upscale of a 24 MP image ≤20 s on an M-class GPU; a model with a wrong sha256 is refused; cancel within 100 ms |
| GPU L2 `gpu.pass` DAG with plugin-owned storage (Verified first) | Golden images on Metal and lavapipe within tolerance; exceeding a quota fails only that plugin call |
| `format.import@1`, `format.export@1`; overlays with hit regions; parameter kinds (Point3, Layer, Mask, Path, Curve, Gradient, File, Data, Button) | **Formats:** round-trip fixtures per app; importer fuzz targets; the sentinel-link test (an importer cannot create a link to `~/.ssh`). **Overlays:** a drag produces one undo step |
| FigCraft collaboration server validates plugin-data ownership | A peer writing another plugin's namespace is refused server-side |

## P5 · Audio (5–7 months)

Repos: soundcraft, craft-libs (`craft-audio-rt`, `craft-audio-dsp`), craft-extensions, filmcraft.

| Work | Acceptance |
|---|---|
| **P5a.** SoundCraft event lists, sample-accurate automation, per-slot faults, no-allocation guard on the audio thread | Bounce at block sizes 64, 128, 512 and 1024 is bit-identical for deterministic built-ins; a panicking slot leaves the master running; playback allocates nothing on the audio thread |
| **P5b.** Wasm audio in the worker on a real-time thread; native CLAP, VST3 and AU in `craft-plugin-bridge`; FilmCraft `AudioEffect` slot; `audio.effect@1`, `audio.instrument@1`, `audio.note-fx@1` | **Performance:** a Wasm EQ at 128 frames has p99.9 block time ≤2× native, with zero dropouts over 10 minutes; ten plugins in one chain stay within the block budget. **Faults:** a bridge crash outputs dry until reload and the session continues |

## P6 · Hardening (ongoing)

Before public launch:

- TUF with an offline threshold root;
- offline provenance verification for native, Verified and Official packages;
- an hourly revocation feed, already in place from P1c.

Then, in order:

- the effects probe (recording `Env`) family by family;
- Windows LPAC and Job-object confinement once Windows ships;
- GPU L3 if the catalogue names a need;
- T3 graph and timeline editors in craft-libs;
- a WIT and component-model review after the worker's IPC numbers are known.

Acceptance: every threat id in research reports 02 and 09 maps to at least one CI test (the
threat-map coverage check), and the fuzz-target table covers every parser plugin bytes can reach.

## Repos and branches

| Repo | Branch for this work | Remotes |
|---|---|---|
| craft-extensions, effectcraft, photocraft, figcraft, craft-launcher | `plugin-beta` | forgejo, github (existing) |
| anicraft, filmcraft, vectorcraft, lightcraft, designcraft, texturecraft, materialcraft, soundcraft, craft-libs | `plugin-beta`, created 2026-11-18 at upstream `main` | forgejo `ash/<repo>-plugin-beta`, github `arnav-hiwarkar/<repo>-plugin-beta` (private); upstream fetch-only |
| craft-plugins | `main` | forgejo, github |

Upstreams stay read-only. Work lands on `plugin-beta` branches, with commits marked `[skip ci]` until a
run is approved. App upstreams keep moving, so each phase starts by fetching upstream and merging into
`plugin-beta`.

# Workstreams

## Threads

Work is split into numbered workstreams, each with a set of components it owns, the threads it waits on, and the milestone where it finishes. Every capability sits in one of these threads, including the other Craft apps. Nothing is parked. The table lists every thread with the number of components it owns. Milestone threads (M-1 to M6) are checkpoints, not work. Streams A to H group the shared work by theme. The EC, PC and FC threads are per-app work inside the shared structure.

| Thread | Components owned | Waits on | Milestone |
|---|---|---|---|
| WS-COM-m-1: Owner briefing accepted | - | - | M-1 |
| WS-COM-m0: Foundations | - | - | M0 |
| WS-COM-m1: Metadata green | - | - | M1 |
| WS-COM-m2: First adapter | - | - | M2 |
| WS-COM-m3: Three adapters | - | - | M3 |
| WS-COM-m4: Depth 1 | - | - | M4 |
| WS-COM-m5: Depth 2 | - | - | M5 |
| WS-COM-m6: Proof | - | WS-COM-m5, WS-COM-ws13, WS-COM-ws14 | M6 |
| WS-COM-stream-a-metadata: Stream A: Metadata | - | - | M0-M2 |
| WS-COM-stream-b-runtime: Stream B: Runtime | - | - | M1-M4 |
| WS-COM-stream-c-isolation: Stream C: Isolation | - | - | M2-M5 |
| WS-COM-stream-d-gpu: Stream D: GPU | - | - | M2-M5 |
| WS-COM-stream-e-ui: Stream E: UI | - | - | M1-M5 |
| WS-COM-stream-f-tooling: Stream F: Tooling | - | - | M0-M5 |
| WS-COM-stream-g-ai-text-audio: Stream G: AI, text and audio | - | - | M5 |
| WS-COM-stream-h-distribution: Stream H: Distribution | - | - | M2-M5 |
| WS-COM-ws0: Docs, decisions and locks of record | 2 | - | M-1 |
| WS-COM-ws1: Commands, metadata and sinks | 8 | WS-COM-m-1 | M0 |
| WS-COM-ws10: PhotoCraft app and adapter | 6 | WS-COM-ws1, WS-COM-ws2, WS-COM-ws3, WS-COM-ws6 | M3 |
| WS-COM-ws11: FigCraft app and adapter | 5 | WS-COM-ws1, WS-COM-ws2, WS-COM-ws3, WS-COM-ws6 | M2 |
| WS-COM-ws12: Native hosts | 11 | WS-COM-ws3, WS-COM-ws5, WS-COM-ws6 | M5 |
| WS-COM-ws13: Flagship plugins | 3 | WS-COM-m3 | M4-M6 |
| WS-COM-ws14: CI, conformance, fuzzing and performance | 3 | - | M0 |
| WS-COM-ws15: Adapters for the other Craft apps | 6 | WS-COM-m6, WS-COM-ws3, WS-COM-ws8, WS-COM-ws12 | M7-M9 |
| WS-COM-ws2: Runtime core ext-runtime | 33 | WS-COM-ws1, WS-COM-ws3 | M2 |
| WS-COM-ws3: Isolation and engines | 26 | WS-COM-ws1 | M2 |
| WS-COM-ws4: SDK, bindings, languages, compat shims, author tooling | 26 | WS-COM-ws1, WS-COM-ws2, WS-COM-ws3 | M3 |
| WS-COM-ws5: GPU | 5 | WS-COM-ws2, WS-COM-ws3 | M4 |
| WS-COM-ws6: UI and web | 8 | WS-COM-ws2, WS-COM-ws3 | M2 |
| WS-COM-ws7: Trust, store, manager and distribution | 16 | WS-COM-ws1, WS-COM-ws3, WS-COM-ws6 | M2 |
| WS-COM-ws8: Services | 9 | WS-COM-ws3, WS-COM-ws5, WS-COM-ws2 | M5 |
| WS-COM-ws9: EffectCraft app and adapter | 11 | WS-COM-ws1, WS-COM-ws2, WS-COM-ws3, WS-COM-ws6 | M3 |
| WS-COM-ws9-ws12-scope: WS9 and WS12 task scope | - | - | M5 |
| WS-EC-stream-a: EC stream A | 10 | WS-COM-ws1 | M1 |
| WS-EC-stream-b: EC stream B | 25 | WS-EC-stream-a, WS-COM-ws2 | M3 |
| WS-EC-stream-c: EC stream C | 19 | WS-EC-stream-b, WS-COM-ws3 | M4 |
| WS-EC-stream-d: EC stream D | 4 | WS-EC-stream-c, WS-COM-ws5, WS-COM-ws4 | M4-M5 |
| WS-EC-stream-e: EC stream E | 2 | WS-COM-ws6 | M4-M5 |
| WS-EC-stream-f: EC stream F | 5 | WS-EC-stream-b, WS-COM-ws3, WS-COM-ws4 | M4 |
| WS-EC-stream-g: EC stream G | 5 | WS-EC-stream-c | M4 |
| WS-EC-stream-h: EC stream H | 4 | WS-EC-stream-c, WS-EC-stream-d | M5 |
| WS-EC-stream-i: EC stream I | 2 | WS-EC-stream-d | M5 |
| WS-EC-stream-j: EC stream J | - | WS-EC-stream-b | M4 |
| WS-EC-stream-k: EC stream K | - | WS-EC-stream-c, WS-EC-stream-d, WS-EC-stream-e, WS-EC-stream-f, WS-EC-stream-g, WS-EC-stream-h, WS-EC-stream-i, WS-EC-stream-l, WS-EC-stream-n | M6 |
| WS-EC-stream-l: EC stream L | 3 | WS-EC-stream-d, WS-EC-stream-i | M5 |
| WS-EC-stream-m: EC stream M | - | WS-EC-stream-c | M4 |
| WS-EC-stream-n: EC stream N | 1 | WS-EC-stream-e, WS-EC-stream-f | M5 |
| WS-FC-command-metadata: Command metadata on all 630 commands | 18 | - | M1 |
| WS-FC-dispatcher: Dispatcher: Authorization, implicit targets, journal | 1 | WS-COM-ws1, WS-COM-ws2 | M2 |
| WS-FC-f1: F1: Meta and manifest keys | - | - | M1 |
| WS-FC-f10: F10: T6 on CEF and figma-compat | 2 | WS-FC-f10a, WS-FC-f6, WS-COM-ws4 | M5 |
| WS-FC-f10a: F10a: web build backend | 1 | WS-FC-f3 | M4 |
| WS-FC-f2: F2: origin and sinks | - | WS-FC-f1 | M1 |
| WS-FC-f3: F3: adapter on Runtime<A> | 28 | WS-FC-f1, WS-FC-f2, WS-COM-ws1, WS-COM-ws2, WS-COM-ws3, WS-COM-ws7 | M2 |
| WS-FC-f4: F4: change lists and indexes | - | WS-FC-f3 | M2 |
| WS-FC-f5: F5: collab protocol 4 | - | WS-FC-f4 | M4 |
| WS-FC-f5b: F5b: collab.write | - | WS-FC-f5, WS-FC-f6 | M4 |
| WS-FC-f6: F6: plugin shell on ext-ui-paint | - | WS-FC-f3 | M4 |
| WS-FC-f7: F7: images, media and GPU | 4 | WS-FC-f3, WS-COM-ws2, WS-COM-ws5 | M4-M5 |
| WS-FC-f8: F8: codegen, lint, suppliers | - | WS-FC-f3, WS-FC-f4, WS-FC-f6 | M4 |
| WS-FC-f9: F9: widgets | - | WS-FC-f5, WS-FC-f6 | M4 |
| WS-FC-fc10: Widgets on canvas (FC10) | - | - | M1 |
| WS-FC-fc2: Change lists with node ids (FC2) | 1 | - | M1 |
| WS-FC-fc3: FC3 plugin data | 3 | - | M1 |
| WS-FC-fc4: Images and filters (FC4) | - | - | M1 |
| WS-FC-fc6: Dev Mode codegen panel (FC6) | 1 | - | M1 |
| WS-FC-fc8: Formats registry (FC8) | - | - | M1 |
| WS-FC-fc9: Vector effect (FC9) | - | - | M1 |
| WS-FC-plugin-host-ui: Shell: docking, windows, menus | 2 | WS-COM-ws6 | M2 |
| WS-FC-sdk: SDK modules for FigCraft | 6 | - | M1 |
| WS-FC-sinks: Choke points through craft_command::sink | 1 | WS-COM-ws1 | M1 |
| WS-PC-pc-1: Meta on PhotoCraft commands | 4 | WS-COM-ws1 | M1 |
| WS-PC-pc-10: Overlays, tools, UI shell | 1 | WS-COM-ws6 | M4 |
| WS-PC-pc-11: Actions journaling | - | WS-PC-pc-2, WS-PC-pc-4, WS-COM-ws3 | M4 |
| WS-PC-pc-12: Format registry | 1 | WS-PC-pc-4 | M4 |
| WS-PC-pc-13: Selection modules, pickers, automate | - | WS-PC-pc-4 | M4 |
| WS-PC-pc-14: Offline AI | - | WS-PC-pc-4, WS-COM-ws8 | M5 |
| WS-PC-pc-15: craft-pshost | - | WS-PC-pc-5, WS-PC-pc-7, WS-PC-pc-18, WS-COM-ws3 | M5 |
| WS-PC-pc-16: Web build transport | - | WS-PC-pc-4 | M4 |
| WS-PC-pc-17: Flagship plugins | - | WS-PC-pc-6, WS-PC-pc-7, WS-PC-pc-8, WS-PC-pc-10, WS-PC-pc-11, WS-PC-pc-12, WS-PC-pc-13, WS-PC-pc-14, WS-PC-pc-15, WS-PC-pc-16, WS-PC-pc-18, WS-PC-pc-19, WS-PC-pc-20, WS-COM-ws13 | M6 |
| WS-PC-pc-18: Photoshop scripting and UXP compat | 6 | WS-PC-pc-1, WS-PC-pc-4, WS-COM-ws4, WS-COM-ws6 | M5 |
| WS-PC-pc-19: Print and font sinks | - | WS-PC-pc-2, WS-COM-ws1 | M4 |
| WS-PC-pc-2: Sinks with Authorization | 2 | WS-PC-pc-1, WS-COM-ws1 | M1 |
| WS-PC-pc-20: USB devices broker | 1 | WS-PC-pc-15, WS-COM-ws2 | M5 |
| WS-PC-pc-3: Change source and events | 1 | - | M2 |
| WS-PC-pc-4: Adapter on Runtime<A> | 3 | WS-PC-pc-1, WS-PC-pc-2, WS-PC-pc-3, WS-COM-ws2 | M3 |
| WS-PC-pc-5: Pixel data plane | 2 | WS-COM-ws3 | M4 |
| WS-PC-pc-6: Filter service | - | WS-PC-pc-5 | M4 |
| WS-PC-pc-7: Smart filters and adjustment layers | - | WS-PC-pc-6 | M4 |
| WS-PC-pc-8: Pen input and brush engines | 1 | WS-PC-pc-4, WS-PC-pc-9, WS-COM-ws5 | M4-M5 |
| WS-PC-pc-9: GPU compute GT1-GT3 | 1 | WS-COM-ws5 | M4-M5 |


## Dependency structure

Everything rests on the command layer. `craft-command` and its sinks come first, because every other piece asks it who is acting and what they may touch. The plugin runtime, `ext-runtime`, depends on it directly. It is the most depended-on component in the plan: nineteen others build on it. The sinks are next, with fifteen dependents.

Above the runtime sits the adapter trait, `CraftApp`. It is the only way an app talks to plugins. Each app gets a thin adapter that maps its own commands onto the trait. Behaviour lives once, in the runtime, and the adapters only translate. That keeps the apps fast and keeps one architecture across all fifteen.

The threads follow the same order:

- Commands, metadata and sinks (WS1) wait only on the owner briefing. Docs and CI (WS0, WS14) start at the same time.
- Isolation and engines (WS3) wait on WS1. Runtime core (WS2) waits on WS1 and WS3.
- GPU (WS5), UI and web (WS6) and trust, store and manager (WS7) wait on the runtime and isolation work.
- SDK (WS4) and services (WS8) wait on runtime, isolation and GPU.
- The three app adapters (WS9 EffectCraft, WS10 PhotoCraft, WS11 FigCraft) wait on WS1, WS2, WS3 and WS6. PhotoCraft and FigCraft also need WS6 before their UI lands.
- Native hosts (WS12) wait on isolation, GPU and UI.
- Flagship plugins (WS13) wait on three adapters being done.
- Adapters for the other Craft apps (WS15) wait on proof (M6), isolation, services and native hosts.

Inside each app, the streams chain the same way. EffectCraft stream A (metadata) feeds B (document state and seams), which feeds C (effects and render queue), which feeds D (GPU) and G (expressions). Stream K, the final EffectCraft proof, waits on almost everything else. FigCraft runs F1 and F2 (keys and sinks) into F3 (the adapter), then into F4 to F6 and F10. PhotoCraft runs its metadata and sinks first, then the adapter, then the pixel data plane and filters.

## Milestones

The plan counts weeks from M-1. Each milestone has an exit that tests must show.

1. M-1, week 0. The owner accepts the briefing.
2. M0, week 4. Foundations: `craft-command` compiles with metadata and authorization, the egui fork builds all three apps, the CI skeleton runs.
3. M1, week 14. Metadata is green in EffectCraft, PhotoCraft and FigCraft with zero undeclared commands.
4. M2, week 20. FigCraft is the first app on the new runtime, through a real worker with confinement on macOS, Linux and Windows. The old FigCraft runtime is then deleted.
5. M3. EffectCraft and PhotoCraft move over. Their old runtimes are deleted, and Wasmtime and V8 run in the worker.
6. M4, week 40. The data plane meets its targets, GPU tiers GT1 and GT2 work, UI tiers 1 to 4 ship, and the first flagships (Prism, Glide, Steady, Flowline) land, each accepted against the test on its card.
7. M5, week 56. GPU tiers GT3 and GT4, web tiers on CEF, native hosts, compatibility shims with corpus parity, AI, codecs, devices, audio and video output.
8. M6, weeks 74 to 82. All flagships pass, the security suites pass, and the three apps ship in beta. The beta exit is blocked until the owner answers D-T, because M6 closes on a hardware run.
9. M7 to M9, weeks 94, 106 and 118. FilmCraft and AniCraft, then VectorCraft, DesignCraft and LightCraft, then TextureCraft, MaterialCraft and SoundCraft.

Each cutover deletes the old plugin system only after its tests pass. Nothing runs two systems side by side after its gate.

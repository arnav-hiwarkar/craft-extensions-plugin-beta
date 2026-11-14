# NZO community extensions and Craft beta

Inspected 2026-11-14. This is a source review, without installation or execution. The requested [NZO-Store-ArtCraft repository](https://github.com/Nozomagarion/NZO-Store-ArtCraft) remains inaccessible. Its installer and runtime cannot be audited from the available evidence. Related author projects below are explicitly secondary evidence, not a substitute store implementation.

## Evidence and provenance

| Primary source | Exact inspected revision and locations |
|---|---|
| Craft beta, local `craft-extensions` | `74e95e0edc9eb22e9259f66d99fbae1b041d2ee5`; `docs/plugins/{capabilities,author-guide,ui,how-it-works}.md`, `docs/plugins/contracts/effectcraft.md`, `crates/ext-host/src/{api,wasm}.rs` |
| EffectCraft legacy host, local `effectcraft` | `5983864a93bf1f9de6a2fcb5f660ba04852a32cc`; `crates/script/src/{lib,runtime,socket,scriptfile}.rs`, `crates/engine/src/scriptui.rs`, `crates/ui-egui/src/panels/scriptui_view.rs` |
| [NZO endpoint repository](https://github.com/Nozomagarion/NZO-Store-Endpoint/tree/196cea5191b63b369c3292583325d4d9699ab882) | `196cea5191b63b369c3292583325d4d9699ab882`; `README.md`, `store-endpoint.json` |
| [Anonymizer](https://github.com/Nozomagarion/NZO-Anonymizer/tree/56251fe42a8f77b55c2f48d5d49dd94a79b825f1) | `56251fe42a8f77b55c2f48d5d49dd94a79b825f1`; `README.md`, `Anonymizer.jsx`, selected download/server sections of `moteur/anonymizer_engine.py`, `moteur/installer.sh` |

Local revisions identify checkout HEAD, not proof of a released binary. Remote text was read without executing it. No remote binaries were downloaded.

The requested GitHub page returned 404 through web fetch. Authenticated `gh api repos/Nozomagarion/NZO-Store-ArtCraft` also returned 404. The [public repository list](https://api.github.com/users/Nozomagarion/repos?per_page=100) succeeded and contained 14 repositories, including NZO-Store-Endpoint and NZO-Store-Blender, but no ArtCraft store. [GitHub repository search](https://api.github.com/search/repositories?q=NZO-Store-ArtCraft) returned zero matches. Exact-name web searches supplied no usable primary source. These results do not distinguish deletion, privacy, or an unindexed move.

The endpoint JSON names `https://expiration-ontario-plaza-remarks.trycloudflare.com` with an `updated` value of `2026-11-10T09:34:46Z`. This proves only that the author published a destination record. No loader was available to establish whether it uses that record, verifies it, or follows updates. The destination was not treated as an authenticated store or fetched for executable assets.

## Store versus actual runtime

The missing store's listing, installation, extraction, updates, and launch behavior remain unknown. A store can copy scripts or binaries while the app supplies an entirely separate runtime. Anonymizer demonstrates that distinction.

**Observed code:** `Anonymizer.jsx:47` chooses a dockable `Panel` when supplied by the host, otherwise a palette `Window`. Lines 246–263 use JSON lines over `Socket` to `127.0.0.1`, default port 47811. Lines 437–485 select footage and send its raw source path, frame range, and output directory to the engine. Lines 714–763 import a generated image sequence, parent layers to footage, add Gaussian Blur and Mosaic, create tracked masks, and group edits into one undo operation. The panel schedules job polling with `app.scheduleTask`.

**Advertised claims:** the README describes tracked AI face replacement, blur, pixelation, editable output, local processing, and model downloads during installation. Its speed figures and claims about identity protection were not independently verified. Its warning that source footage bypasses existing layer effects matters when assessing operation parity.

**Observed host:** legacy scripting uses Boa JavaScript and an After Effects-style object model. The ScriptUI host publishes serialized widget and drawing descriptions to egui, which routes UI events back to script handlers. Legacy ScriptUI is also host-drawn; beta's distinction is its versioned data contract and capability-scoped calls, not merely the absence of HTML.

## What beta changes

| Area | Observed legacy example | Beta documented behavior |
|---|---|---|
| UI and docking | ScriptUI tabs, sliders, callbacks, dockable EffectCraft panel | Versioned host-rendered trees, event IDs, forms, canvas draw lists. EffectCraft and PhotoCraft panels float; FigCraft panels dock. No guest DOM or renderer selection. |
| Canvas | Anonymizer edits document masks and generated layers | Tool gestures and validated overlays are available. A panel canvas and a document overlay do not create document content by themselves. |
| App operations | Footage path, sequence import, mask creation, effects and expressions | EffectCraft offers bounded session operations including property/keyframe edits and `effect.apply`; `layer.addItem` uses existing project assets and never imports a path. PhotoCraft has picked-file handles. FigCraft has node operations and **no `raster.filter`**. |
| Author workflow | README instructs copying JSX, installing Python dependencies/models, and optionally copying a legacy controls Wasm | Manifest, contracts, Rust guest SDK, `craft-ext new/build/check/pack`; `.craftplugin` is a ZIP. These commands were inspected, not run. Legacy Wasm presence does not establish beta ABI compatibility. |
| Distribution | Store behavior unavailable; Anonymizer README links release ZIPs | Public source and GitHub release asset, immutable release record, catalogue PR; signed metadata and archive size/SHA-256 checks. |
| Updates | Store self-update and script-update policy unknown | Side-by-side versions, activation, pinning, rollback, pending consent when rights expand; native consent is per version. |

Sources are the local author guide sections 2, 11–13, capability tables, UI reference, and lifecycle in `how-it-works.md`. Distribution integrity checks establish correspondence to approved bytes. They do not sandbox those bytes.

## Security boundaries and unresolved questions

**Observed secondary code:** Anonymizer's installer creates a Python environment, upgrades pip, installs requirements, and runs model setup. Its engine lists model downloads on GitHub and Hugging Face, and generated portraits from `thispersondoesnotexist.com`. Thus “nothing leaves your computer” is a processing claim, not an absence of network traffic. The inspected code does not establish whether source images are transmitted elsewhere across every code path.

`_download` writes a `.part` file and renames it without a digest or signature check in that function. Model ZIP handling copies only three allowlisted basenames into the model directory, rather than extracting arbitrary member paths. This observation does not audit every download or archive used by the unavailable store. The local TCP server and JSON dispatcher inspected contain no authentication step; loopback binding limits remote reachability but does not authenticate local clients. Jobs accept source and output paths, so helper file access needs separate scrutiny.

The JSX attempts `File.execute()` for startup files and a folder. Local EffectCraft `runtime.rs:414–417` routes that method through `file.executeFile`; success and policy depend on the app command. Do not equate an attempted call, or the README's statement that scripts cannot run programs, with unrestricted shell execution. Legacy file and TCP access use the shared “Allow Scripts to Write Files and Access Network” preference (`socket.rs`, `scriptfile.rs`).

**Beta code boundary:** `wasm.rs:96–99` rejects imports except `craft.log/call/take/result`. No WASI, raw paths, command execution, processes, or network APIs are available to Wasm guests. `api.rs` defines operation authorization and transaction checks. Host calls require listed operations, contribution declarations, scoped live grants, and valid invocation state. Native helpers have full OS access and separate per-version consent; their host calls still undergo the same permission checks. Native consent does not constrain direct OS activity.

**Unknown for NZO-Store-ArtCraft:** network destinations actually contacted; archive traversal checks; package digests/signatures; remote script evaluation; JS bridges, CLI or socket dispatch; permission prompts; process launch; arbitrary file access; self-update and remote-content rules. No “safe” or “unsafe” verdict follows from a 404.

## Migration examples and priorities

A blur button can become a beta panel button whose event calls `effect.apply` with `ec.blur.gaussian`, under declared document-edit scope. Replace ScriptUI callbacks with IDs and host events. This is a limited migration of one action, not full Anonymizer parity.

Full Anonymizer needs asynchronous inference, persistent settings, source media access, sequence import, tracked masks, and expressions. A separately consented native helper can own inference and downloads. It still cannot invoke absent app operations through `craft.call`; new scoped media/import and mask operations need design and implementation. FigCraft cannot host its raster filter.

Prioritize a supported asynchronous job/result workflow and scoped media exchange, then demonstrate one tracked-mask import with cancellation and undo. Persistent plugin settings and EffectCraft docking are useful beta gaps. Raw filesystem/network/process access from Wasm and unlimited event duration are intentional boundaries, not parity bugs. Obtain the original store source before making installer-specific recommendations; a renamed endpoint record alone cannot resolve that evidence gap.

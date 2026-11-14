# ECScripts evidence review against Craft plugin beta

Inspected 2026-11-14. Research only: no plugin, app, downloaded code, build, or test was executed. This review distinguishes **observed source**, **listing claims**, **inferences**, and **unknowns**. Local source revisions: craft-extensions `74e95e0edc9eb22e9259f66d99fbae1b041d2ee5`; EffectCraft `5983864a93bf1f9de6a2fcb5f660ba04852a32cc`. Working-tree documentation may include changes beyond those revisions.

## The live catalogue snapshot contains one release

**Observed:** [the deployed catalogue API](https://ecscripts-6kz.pages.dev/api/plugins) returns exactly one release, `bouncraft-2-0`, BOUNCraft 2.0 by hugomec. Its `apps` is only `effectcraft`, requirement `EffectCraft 0.23+`, kind `script`, category `utilities`, license `MIT`. There is no `repoUrl` or `install` field. [Static catalogue source](https://ecscripts-6kz.pages.dev/data/plugins.js) has `plugins: []`; `site.js::withRemotePlugins` merges API records. Multiple categories and app labels are website taxonomy, not evidence of runnable plugins for those apps.

| Requested sample | Evidence actually available | Comparison with beta |
|---|---|---|
| Panel / motion utility | The release JSX implements property targeting, overshoot/bounce, sliders, custom curve drawing, axis choice, saved presets, Bake, Remove and From Layer | Partial migration possible; expression manipulation is a significant blocker below |
| Effect | Release includes two legacy Wasm modules; the pinned public builder describes pass-through effects grouping expression controls | Legacy ec_* ABI and beta craft_* ABI differ; do not equate this with a beta pixel filter |
| Template | No approved template returned | No observed community implementation to compare |
| Theme | No approved theme returned | Beta theme activation is deferred; empty listings do not establish community parity |
| Canvas tool | No approved tool returned | Beta has explicit tool gestures and overlays in all three adapters |

**Observed release source:** the follow-up inspected those expression/control, sampling and preset paths in the actual ZIP script. These are source-backed behaviors, without runtime verification. See the [BOUNCraft release/source review](community-bouncraft-review.md) for line locations, hashes and the public builder. The listing version is `2.0`, despite the site's advertised semantic-version policy. The [docs' V-Bokeh entry](https://ecscripts-6kz.pages.dev/docs.html#format) is a schema example, not a release.

**Artifact evidence:** the API names `tigris:releases/bouncraft-2-0/2.0-a21f54ad-BOUNCraft-2.0.zip` and digest `983a8c0b44f8c6d183d5c8489fd54dc84317697ebe76361ab8584ea3e98bc9a9`. `site.js` converts this into `/api/file?slug=bouncraft-2-0&version=2.0`. An initial HEAD request returned HTML, but a direct GET with an explicit User-Agent fetched a 46,010-byte ZIP matching that digest. The ZIP README links [hugomec7/BOUNCraft](https://github.com/hugomec7/BOUNCraft/tree/0a63576f0992c7b4203407843d58d836083b9071), resolving the source under a different account from the registry publisher handle `hugomec`. JSX and documentation were inspected as inert text. The registry backend repository remains inaccessible; BOUNCraft source is available. No artifact was installed or executed.

## How a legacy script enters EffectCraft

**Observed host source:** [`crates/script/src/lib.rs`](https://github.com/arnav-hiwarkar/effectcraft-plugin-beta/blob/5983864a93bf1f9de6a2fcb5f660ba04852a32cc/crates/script/src/lib.rs) describes the Boa JavaScript runtime and AE-style object model. [`engine/src/cmd/scripts.rs`](https://github.com/arnav-hiwarkar/effectcraft-plugin-beta/blob/5983864a93bf1f9de6a2fcb5f660ba04852a32cc/crates/engine/src/cmd/scripts.rs), `install`, accepts UTF-8 `.jsx` and `.js`, rejects `.jsxbin`, and copies text into settings `Scripts` or `Scripts/ScriptUI Panels`. Menus refresh immediately. `window.scriptPanel` opens an installed panel. Scripts can also enter through Run Script File, the console, CLI and MCP. The source links pin the inspected EffectCraft revision.

A ScriptUI panel runs with `this` bound to a dockable `Panel`. `scriptui.js` and `ui.rs` implement widgets, callbacks, modal dialogs and `onDraw` graphics. [`ui-egui/src/dock.rs`](https://github.com/arnav-hiwarkar/effectcraft-plugin-beta/blob/5983864a93bf1f9de6a2fcb5f660ba04852a32cc/crates/ui-egui/src/dock.rs) has `ScriptPanel`; `panels/scriptui_view.rs::paint_draw` renders its draw lists. Legacy UI also uses host rendering, but has a JavaScript callback/object API rather than beta's versioned JSON/event contract. The host supports expressions, keyframes, render queues, imports, project saving and task scheduling. Mutations use engine commands and explicit undo groups.

**Observed BOUNCraft entry route:** the ZIP README gives the ScriptUI panel install path, and the script chooses the supplied dockable Panel or a palette fallback. This matches the legacy host source. It is not a successful installation result. The public Python builder writes tiny control-effect Wasm modules directly; it requires no Rust compilation. Neither those modules nor the JSX use the beta package/ABI unchanged.

## Beta gains and migration costs

Sources: [capabilities](capabilities.md), [author guide](author-guide.md), [UI contract](ui.md), [runtime](how-it-works.md), and EffectCraft `crates/plugins/src/{contract.rs,ops.rs,runtime.rs}`.

| Concern | Source-backed difference and migration example |
|---|---|
| Build and package | Beta uses `craft-plugin.toml`, relative package paths, contributions and minimum contracts inside a ZIP `.craftplugin`. Rust SDK templates compile Wasm; `check` validates referenced modules, and `pack` archives files. A listing JSON record cannot become this executable package merely by copying metadata. Native helpers need platform executables. |
| Panel | Rewrite ScriptUI callbacks as session `ui` events and `ui.panel.set` trees. Declare `effectcraft.session@1.1` for UI 1.1 nodes such as tabs and number fields. Stable control IDs and revisions prevent stale events. EffectCraft beta panels float; legacy ScriptUI panels dock. FigCraft beta panels dock, PhotoCraft panels float. Docking parity in EffectCraft is a concrete gap. |
| Motion | Replace numeric property edits with `prop.set` and `prop.addKey`; one event commits one undo step. A Bake operation might generate numeric samples in guest code. Full BOUNCraft parity cannot be assumed: the beta list exposes no expression setter/remover, selected-property query or key deletion. `prop.set` rejects strings and paths. The inspected JSX confirms these dependencies. Preserving that workflow needs deliberate host operations. |
| Curve preview | Beta UI and canvas draw lists can represent previews, but custom ScriptUI `onDraw` handlers need translation. Tool overlays are explicit host data, not arbitrary rendering callbacks. |
| Effects | A beta `raster.filter@1.0` module transforms pixels in EffectCraft and destructively in PhotoCraft. It does not replace an animation-expression control effect. FigCraft has no `raster.filter`. Temporal frames and tiled requests are deferred. |
| Scope | Contributions are matched to app contracts, not broad listing app labels. Session operations differ per app. Native code cannot bypass declared and live-granted rights when using host calls. |

## Publishing and update trust

**Observed deployed client:** [publish.js](https://ecscripts-6kz.pages.dev/assets/js/publish.js) requires a signed-in user in its UI, requests `/api/uploads/presign`, PUTs bytes, calls `/api/uploads/commit`, then POSTs `/api/submissions`. It displays scan results and blocks an `infected` verdict client-side. [account.js](https://ecscripts-6kz.pages.dev/assets/js/account.js) uses same-origin credentials. The website provides search, ratings, saved items and download tracking; these are registry features, not app APIs.

**Advertised:** Google sign-in, reviewer approval, 200 MiB uploads and hosted storage. Docs explicitly say external download links are not scanned. Server-side ownership checks, scanner implementation, immutable releases, authorization, storage policy and signature verification could not be inspected. No client path shown here verifies downloaded bytes against the displayed digest. A checksum displayed by the same server is not independent publisher authentication.

Beta's [distribution design and implementation](store-and-distribution.md) adds signed catalogue snapshots, digest/size and manifest checks, serial replay protection, metadata expiry, side-by-side releases, pins, rollback and revocations. Install does not enable. Changed permission needs hold an update pending; native consent is per version. These checks establish package provenance and lifecycle rules; they do not sandbox execution.

**Observed runtime boundary:** `ext-host/src/api.rs::authorize` checks declared rights plus current grants, cancellation and grant generation. Wasm imports are restricted to `craft.*`, with memory/fuel limits and no WASI, raw paths, command dispatch, processes or sockets. Legacy file writes and TCP access use a broad scripting preference; reads have project-folder restrictions. Native beta helpers have full OS access after separate consent, even though host calls remain scoped. No blanket safe/unsafe ranking follows from these differences.

## Priorities and evidence limits

Prioritize expression and property-selection migration design, EffectCraft docking, and clear registry-to-package validation. Keep raw execution exclusion intentional. Separate deferred theme/font activation, clipboard/network host operations and temporal filtering from proven community requirements. Requesting those solely because the website has category labels would overstate the evidence.

The public registry backend remains unavailable. The catalogue snapshot cannot support a broad multi-plugin comparison because it returns one release. BOUNCraft implementation evidence is now available in the linked follow-up; template, theme and canvas-tool examples remain absent from this API snapshot.

Deployed SHA-256 evidence, fetched as inert text from the URLs linked above:

| Asset | SHA-256 |
|---|---|
| `/` | `49c40aed15e5de5f02839d410e7395e1398e96d688e716a18e439527d8d09bbe` |
| `api/plugins` | `35201281c217c3663a8849036e432275d1e330afcdaf3707305b7f591e27c324` |
| `data/plugins.js` | `30a8faa19d80f80e1ede330ca5fc1b6970c2de328b28a26026beb7b54f566049` |
| `assets/js/site.js` | `43bee8f2de01c0845cbff3c580475bcc8359e093415da994251117fa8ee1b240` |
| `assets/js/publish.js` | `0640587499d365e2dc305cb0565511caf6761e1ebe60572aef440086292ba5e4` |
| `assets/js/account.js` | `f86def141fb99ab8dd0323f60dfcdb6f2b55036cf29ad6ee7a7a0def63219b3f` |

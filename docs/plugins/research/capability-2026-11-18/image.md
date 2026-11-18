# Plugin capability gaps: raster, photo, vector, layout and design apps

Date: 2026-11-18. Scope: PhotoCraft, FigCraft, VectorCraft, DesignCraft and LightCraft against the
plugins that ship for Photoshop, Lightroom, Illustrator, Figma, InDesign and Affinity. Method:
source reading only (no builds). Repo paths are written as `PC/` (plugin-beta/photocraft),
`FC/` (plugin-beta/figcraft), `VC/` (~/Projects/vectorcraft), `DC/` (~/Projects/designcraft),
`LC/` (~/Projects/lightcraft) and `CE/` (plugin-beta/craft-extensions).

## Summary

1. **The engines are far ahead of what plugins can reach.** Every app has a JSON command registry
   (`CommandSpec` plus `Session::execute`) that its MCP server already drives:

   | App | Engine commands | Exposed to plugins |
   |---|---|---|
   | FigCraft | about 630 | about 74 ops |
   | PhotoCraft | registry-wide | about 40 hand-written ops, plus 58 allowlisted filters and 17 adjustments |
   | VectorCraft | about 888 | 0 on the shared platform; it has its own geometry-only Wasm ABI |
   | DesignCraft | about 635 | none |
   | LightCraft | about 325 | none |

   The cheapest large win is a generated, permission-classed allowlist over each registry. In other
   words, a safe `batchPlay`, rather than more hand-written ops.
2. **Three apps have no adapter.** VectorCraft's own `crates/plugins` is a second, competing plugin
   system, which the suite replacement rule says must be migrated and removed. DesignCraft and
   LightCraft have nothing. Lightroom-, Illustrator- and InDesign-class plugins are therefore 0%
   possible today.
3. **The platform is missing the things that commercial plugins all assume:**
   - network (`network:<host>` is a permission with no op behind it);
   - secrets and private settings (`clientStorage`);
   - change events beyond selection and open;
   - clipboard (the permissions exist, but there are no ops);
   - drag and drop;
   - host AI inference;
   - GPU compute;
   - binary pixel transport (pixels currently travel as base64 JSON, at most 1 MiB per write);
   - region-of-interest (ROI) and tiled filters;
   - non-destructive plugin filters and effects;
   - font activation.
4. **Speed.** Wasm is fast enough when the heavy loops go to host kernels. Softskin runs at 602 ms
   at 1080p, Lightwell at 478 ms and Emberfield at 176 ms. The pure-interpreter baseline was about
   17 s per 1080p frame (debug build). AI, upscaling and GPU-class looks such as Topaz, Nik, ON1 or
   Retouch4me still need two additions: a host inference op, and either GPU compute shaders or a
   JIT tier. With those, the native-only list shrinks to:
   - OS-wide input hooks (Lazy Nezumi);
   - vendor DRM and licence daemons;
   - other-application round-trips;
   - hardware such as tethering and colorimeters.

## App inventory

### PhotoCraft (Photoshop class): adapter `PC/crates/plugins`, ops in `PC/crates/engine/src/plugin_ops.rs`

| Area | Engine (path) | Plugins today |
|---|---|---|
| Layer kinds | Raster, Group, Adjustment, Fill, Text, Shape, Smart; Blend If, advanced blending, link groups, per-layer XMP (`doc/src/lib.rs:606-639`) | list and get all kinds. Create pixel, group and fill. Adjustment layers only through `adjustment.apply{as_layer}`. Text, shape and smart-object contents are opaque |
| Masks and clipping | `LayerMask`, `VectorMask`, `clipped` | `mask.add` (all, none or selection) only. Mask pixels, vector masks and clipping can't be read or written |
| Layer styles | `Effects` (`doc/src/lib.rs:298`), `compose/src/effects.rs`, ASL import | none |
| Selection and channels | coverage surface, alpha channels, Quick Mask, paths, smart select (`cmd/selection_cmds`, `channel_cmds`, `smartselect_cmds`) | `selection.get/set/mask.get/save` and `channels.list`. Channel pixels and channel-to-selection can't be read |
| Colour | Bitmap, Gray, Indexed, RGB, CMYK, Lab, Multichannel, Duotone at 8/16/32f (`color/src/lib.rs:51`); ICC through craft-cms; proofing (`cmd/color_cmds.rs`) | mode and depth are reported. **Pixel and filter access is RGB only.** No ICC read, no mode conversion |
| Foreground and background colour, swatches | `engine/src/doc_state.rs:182`; about 13 `swatches.*` | none |
| Paint engine | `paint/` (brush, dynamics, presets, mixer, symmetry); dynamics take pressure, tilt, rotation and wheel (`paint/src/dynamics.rs:459`); ABR import (`io/src/abr_map.rs`); tablet `Sample{pressure,tilt_x,tilt_y,rotation,eraser}` (`tablet/src/lib.rs:51`) | none. Tool events carry `pressure` only; tilt and rotation are dropped (`engine/src/plugin_sessions.rs:368-375`). No hover |
| Smart filters | `SmartFilter{command,params,…}` re-run from the filter registry (`cmd/smart_cmds/rendering.rs:124`) | "plugin smart filters are unsupported" (`plugins/src/contract.rs:409`) |
| Text and vector | `text/` (shaping, CJK and vertical text, warp, colour glyphs); `vector/`; path booleans (`doc/src/vector.rs:54`) | none |
| Raw, content-aware fill, inpainting | `raw/` (DNG, CR2, NEF, ARW, RAF and more) plus a WGSL Camera Raw; `algo/src/content_aware`, `inpaint`, `poisson.rs` | none. Camera Raw, Liquify, Lens Blur and Displace are deliberately off the allowlist (`plugins/src/session_filters.rs`) |
| History | snapshots, history brush | `history.list` labels only |
| Actions and batch | record, play and ATN (`cmd/actions_cmds.rs`, `engine/src/actions_record.rs`); batch and droplets; MCP `control_call` (`automation/src/server.rs`) | Plugin **filters** are journaled. Session commands and tools are **not** (`journal:false`, `plugin_sessions.rs:676`). Plugins cannot run actions |
| Formats and metadata | PSD/PSB, TIFF, EXR, HEIF, KRA, XCF and more (`io/`); XMP, EXIF and IPTC (`doc/src/lib.rs:880`) | `document.export` to png, jpg, tif, webp and psd with no options. `layer.import`. No metadata |
| GPU | wgpu 30 compositor with **fragment** pipelines only (45 `@fragment` and 0 `@compute` across 4 WGSL files) | none |
| Plugin filters | whole active layer as one buffer (`engine/src/plugin_filters.rs:279-388`); background job with progress; preview in the dialog | No ROI or tiles. RGB only. Destructive. Disabled on smart objects and masks |

### FigCraft (Figma class): adapter `FC/crates/plugins` (`runtime.rs`, `contract.rs`)

- **Dispatch.** Ops forward to the engine registry (`FC/crates/engine/src/cmd.rs`, about 630
  commands). `node.update` maps to `node.set`, so fills, strokes, effects, `boundVariables` and style
  ids are already reachable through props (`engine/src/cmd/props.rs`).
- **Exposed:**
  - nodes, pages, components and instances, styles and variables;
  - `bool.*`, `vector.create/flatten/outlineStroke`;
  - `layout.add/remove/setTrack`;
  - `image.place/adjust/replace`, which take inline bytes and refuse paths;
  - `text.create/onPath`, comments, guides;
  - `export.svg` (at most 4 MiB);
  - overlay, tool events (no pressure), and events for selection, document open, page change and
    document change (that last one read-only).
- **In the engine but not exposed:**
  - `effect.*`;
  - `text.set/setFeatures/setVariations/loadFont/fonts`;
  - `proto.*` (prototyping, including an expression engine in `doc/src/proto*.rs`);
  - **Dev Mode codegen** (`engine/src/dev/codegen/{css,html,swift,compose,flutter,arkui}.rs`,
    `tailwind.rs`, `inspect.code`, `codeConnect.*`, `measure.*`);
  - PNG, PDF and WebP export (`doc/src/props.rs:1310`);
  - `vector.offset/simplify/edit.*`, `shapeBuilder.*`;
  - `align/distribute/transform.*`, `object.mask/rasterize`;
  - `data.fill`, which is Content Reel-like and already in the engine;
  - `variable.import/export`, `library.*`;
  - clipboard;
  - imports (`format/src/{fig,penpot,sketch,rest}.rs`).
- **Contract gaps:** no `raster.filter`, no file ops, `resource.font` is declared but not activated,
  and node ops need document-wide scope because selection-only grants are deferred.

### VectorCraft (Illustrator class): no shared adapter

- **Own plugin system.** `VC/crates/plugins` is a separate wasmi ABI with no host imports. It has
  three kinds (object filter, live effect in the appearance stack, exporter) and sees only the
  selected paths as JSON. Its seven `plugin.*` commands are in `engine/src/cmd/plugin.rs`. It has no
  dependency on craft-ext and must migrate under the suite rule.
- **Engine, with about 888 commands** (`engine/src/cmd.rs`):
  - Bezier and compound paths (`geom/`);
  - pathops: boolean, pathfinder, offset, outline, a planar shape builder and live paint, trapping
    (`pathops/`);
  - brushes: calligraphic, scatter, art, pattern and bristle (`brush/`);
  - an appearance stack with distort, warp, stylize and raster effects (`effects/`);
  - 3D extrude, revolve and inflate (`three-d/`);
  - freeform gradients, mesh and recolour; CMYK, spot inks and overprint (`color/`, `doc/src/inks.rs`);
  - text with OpenType features, variable fonts, threads and a composer (`text/`);
  - image trace (`trace/`); DXF and dimensions (`cad/`, `doc/src/dimension.rs`);
  - perspective grid, symbols with nine-slice, graphic styles, datasets (`doc/src/variables.rs`);
  - SVG, PDF/X, EPS/AI, EMF and Affinity import; CSS export.
- **Other crates.** `crates/scriptcheck` statically assesses Illustrator ExtendScript, which helps
  with migration. `crates/mcp` drives the registry.

### DesignCraft (InDesign class): no adapter

- **Model.** Spreads, parent spreads, layers, threaded stories, anchored objects, notes, cross
  references, index and ruby (`DC/crates/doc/src`).
- **Composition.** A Knuth–Plass composer, hyphenation, GREP styles, optical margins, kashida and
  warichu, and tables (`DC/crates/compose`).
- **Publishing:**
  - data merge with CSV, TSV and XLSX, including image and QR placeholders
    (`crates/engine/src/cmd/datamerge/`);
  - text variables, live preflight (`cmd/preflight.rs`), books, TOC and index;
  - PDF/X-4, PDF/X-1a, PDF/A and tagged PDF; EPUB 3, IDML and PPTX export;
  - DOCX, RTF, XLSX, IDML and INDD import;
  - CMYK, spot and ASE swatches.
- **Automation.** A step script runner exists: `engine/src/script.rs` runs `{command, params}`
  steps with `$N.path` references, the whole script as one undo step. Script labels exist too. This
  is the natural seed for an adapter.
- **Events.** There is no event bus, only a document `revision` counter.

### LightCraft (Lightroom class): no adapter

- **Catalogue.** An op-log catalogue (`LC/crates/catalog`) with albums, ratings, keywords, stacks
  and smart albums built from nested rules.
- **Develop.** The full settings model is in `crates/develop/src/settings.rs`: WB, tone, curves,
  HSL, grading, detail, optics, Upright, masks, spots, lens blur and calibration. It has a CPU
  reference pipeline and a **wgpu compute twin** (`crates/gpu/src/wgsl/*`, all `@compute`).
- **Presets and looks:**
  - XMP, `.lrtemplate`, DNG-preset and Luminar preset import (`engine/src/preset_import.rs`,
    `preset_luminar.rs`);
  - `.cube` LUTs on the CPU only (`pipeline/src/lut.rs`);
  - DNG profiles (`raw/src/profile.rs`).
- **Masks.** Brush, linear, radial, colour, luminance, depth, subject, sky and object masks, with
  SAM 3 for object masks.
- **AI:**
  - its own pure-Rust ONNX interpreter, with WGSL `nn_conv`/`nn_pool` for denoise and faces
    (`crates/denoise*`, `crates/faces`, `crates/gpu/src/nn.rs`);
  - **candle** 0.9 for SAM 3 (`crates/segment`);
  - model downloads verified by SHA-256 (`crates/fetch`, `engine/src/model_download.rs`).
- **Export.** JPEG, PNG, TIFF, WebP, AVIF, JXL and DNG with ICC, sharpening, watermark and a size
  limit (`engine/src/export*`).
- **Missing for plugins.** There are no publish services and no post-processing actions.
- **Other.** XMP sidecars, IPTC and GPX (`crates/meta`); offline geocoding; HDR and panorama merge;
  print and slideshow. "Tethering" means only a watched folder.

### Shared platform (`CE/`) facts that matter here

- **Runtime.** wasmi 2 (`ext-host/Cargo.toml:18`) with features `std, validate, deterministic,
  portable-dispatch`. **The `simd` feature is not enabled**, and the `Config` in `ext-host/src/wasm.rs:84`
  sets no SIMD.
- **Kernels.** There are 16, with up to 8 threads (`docs/plugins/compute-and-jobs.md`). Jobs get
  512 MiB and about 1000× an event's fuel, have no document ops, and return a result of at most
  8 MiB.
- **Permissions without ops.** `network:<host>` (with wildcard subdomains), `clipboard.read` and
  `clipboard.write` are parsed (`ext-core/src/permission.rs:87-89`), but no op uses them. The
  continuation document lists "no `net.fetch`" as a known limit.
- **Storage.** `project.data` is per document or layer, at most 64 keys and 256 KiB. **No private,
  per-install settings store exists.**
- **UI 1.2.**
  - Forms, table, list and canvas (draw lists with paths, gradients and images, plus pointer and
    key events), and a `color` control.
  - Sliders send their value **only when the drag ends**, so there is no live scrub.
  - `canvas.overlay.set` draws over the document but has **no image op** and no hit regions.
  - Components always draw their fallback.

## Competitive plugin catalogue

Legend for the "Our status" column:

- ✔ possible today
- ◐ partial
- ✖ blocked, followed by the layer where the gap sits: App, Adapter (Ad) or Platform (Pl)

### Photoshop

| Plugin | What it needs | Our status |
|---|---|---|
| Topaz Photo AI, Sharpen, Gigapixel | full-res pixels, ONNX/CoreML models of hundreds of MB, GPU or NPU, background job, **canvas resize** for upscale, before/after UI | ◐ pixels and jobs work. ✖ Pl: inference, GPU, binary transport. ✖ Ad: `document.resize` / new document from pixels |
| Nik Collection (Color Efex, Viveza) | GPU filters, full-window UI with live preview, U-Point control points (local masks), smart-filter use | ◐ filters plus kernels. ✖ Pl: GPU, live slider streaming, overlay images. ✖ Ad: smart filters, ROI preview |
| Portraiture, Retouch4me | skin mask (colour range plus AI), frequency separation, results on new layers, optional cloud | ◐ Softskin shows the class works (602 ms). ✖ Pl: inference, net. Retouch4me's "heal" needs content-aware fill: ✖ Ad (`fill.contentAware` not exposed) |
| Luminar Neo, ON1 (as plugins) | external app round trip, AI sky and masking, LUTs | ✖ Pl: inference. Round trip with another app needs native |
| Infinite Color, Texture | randomised adjustment layers, texture overlays from bundled assets, blend modes | ◐ `adjustment.apply{as_layer}` works; editing an existing adjustment layer's params is ✖ Ad. Bundled image assets as layers: `layer.import` needs a picked file, ✖ Pl `package.asset` read |
| TK8, Lumenzia (luminosity masks) | channel math, masks from luminosity, live preview overlay, load channel as selection, mask write | ◐ `selection.set{mask}` works. ✖ Ad: mask and channel pixel read/write, channel-to-selection, overlay image op |
| Coolorus | colour wheel panel, read and set foreground/background colour, colour-changed event | ◐ the canvas can draw a wheel. ✖ Ad: `color.fg/bg get/set`, `color.changed` event |
| Lazy Nezumi Pro | intercepts pen input, smoothing, rulers, pressure curves, any app | ✖ native by design (OS input hook). Inside our apps: ✖ Ad/Pl `input.filter` contribution plus tilt and rotation |
| Kyle's brushes, brush engines (Kyle T. Webster ABR, Krita-style engines) | brush presets (ABR), custom dab engines, pressure, tilt and rotation | ◐ ABR import exists in the app. ✖ Ad: `brush.preset` resource, `paint.stroke`, brush-dab contribution |
| Fontself Maker | read glyph shapes from vector or pixel layers, build OTF/TTF, install font | ✖ Ad: shape and path read in PC (better done in VC or FC). ✖ Pl: `resource.font` activation and install |
| Generative fill (Firefly, SD, ComfyUI bridges) | selection mask plus composite to cloud or a local model, result as new layer | ◐ selection and pixels work. ✖ Pl: net with consent, or local inference |
| Camera Raw profiles, LUT packs | `.cube`, `.xmp` and DCP resources | ◐ `resource.lut` works in PC. ✖ App/Ad in LC |
| UXP panels (Retouching Toolkit, Pro Panels, ActionsXL) | `batchPlay` over everything, notification listeners, `executeAsModal`, persistent settings | ◐ commands and panels work. ✖ Ad: allowlisted command dispatch, journaling of plugin commands. ✖ Pl: settings, change events |
| Export helpers (Export Kit, layer exporters, social-size exporters) | per-layer export with options, folder pick, naming | ✔ mostly (`document.export` per layer). ✖ Ad: format options (quality, ICC, bit depth) |

### Lightroom Classic

| Plugin | What it needs | Our status |
|---|---|---|
| Export and publish services (Flickr, SmugMug, Jf plugins, LR/Mogrify, Photo Upload) | export provider hooks: rendered file bytes, post-process, upload with OAuth, remote id and URL per photo, collection sync | ✖ App: no publish-service concept. ✖ Ad: no adapter. ✖ Pl: net, OAuth and secrets |
| Metadata plugins (Any Filter, Data Explorer, LR/Transporter, Jeffrey's Geoencoding) | custom metadata fields, catalogue queries, batch metadata writes, tagsets | ✖ Ad: no adapter. The engine has keywords, XMP and metadata commands |
| Negative Lab Pro | analyse a negative (pixels), write develop settings and tone curves, **install a camera profile or LUT**, batch over a roll | ✖ Ad: `develop.get/set`, `photo.pixels`. ✖ Pl/App: profile and LUT resources in LC (`resource.lut` exists in PC, and LC LUTs are CPU only) |
| LRTimelapse | read and write develop settings across hundreds of photos, keyframe interpolation, metadata labels | ✖ Ad. The engine side exists (`develop` settings, sync commands) |
| Presets and profiles packs | XMP presets, profiles | ✔ App import exists (`preset_import.rs`). ✖ Pl: `resource.preset` mapping for LC |
| Topaz, Nik "Edit in" | round trip with a TIFF file | ◐ needs a cross-app `edit.in` (LC→PC through a shared file handoff); the external app part is native only |

### Illustrator

| Plugin | What it needs | Our status |
|---|---|---|
| Astute VectorScribe (Dynamic Shapes, Corners, Measure, smart remove) | custom tools with live overlay handles, path edit at anchor level, snapping, hover | ✖ VC has no adapter. ✖ Pl: overlay hit regions, hover events |
| ColliderScribe (collision snapping, space fill) | geometry queries: intersect, distance, hit test; transforms during drag | ✖ Ad: `geom.intersect`, `path.hitTest`. ✖ Pl: drag-time preview |
| Phantasm (live halftone, curves on vector and embedded images) | **live effect** in the appearance stack; recolour of paths and images | ✖ Ad. VC's legacy ABI already has live effects; migrate to a `vector.effect@1` contract |
| Texturino (raster textures on vectors) | blend raster texture into a vector appearance | ✖ Ad: raster effect inside the appearance stack (VC `effects/raster.rs` exists) |
| Hot Door CADtools | dimension objects, scale-aware drawing, custom tools | ✖ Ad (VC has `dimension.rs` and DXF) |
| MAPublisher | GIS import (Shapefile, GeoJSON), projections, attribute tables, data-driven styling | ✖ Pl: format-import contract. Projection maths is easy in Wasm. ✖ Ad: object attributes (`plugin_data` exists in `VC/doc/src/plugin_data.rs`) |
| Ai scripts (Scriptographer descendants, John Wundes, xtools) | complete DOM via script, menu commands | ✖ Ad. VC's `scriptcheck` gives a migration lens |
| Fontself (Illustrator edition) | read glyph paths, export OTF | ✖ Ad, ✖ Pl font |

### Figma

| Plugin | What it needs | Our status |
|---|---|---|
| Unsplash, Iconify, Pexels | net (allowed domains), insert image or SVG nodes, search panel, drag onto canvas | ◐ `image.place` and `vector.create` work. ✖ Pl: net, drag and drop. ✖ Ad: SVG import as nodes |
| Content Reel, Lorem ipsum, data fillers | text and image writes to the selection, bundled data | ✔ largely (`node.update`, `image.replace`). Engine `data.fill` is not exposed |
| Autoflow | `documentchange` events with node ids, connector nodes, plugin data | ◐ `document.changed` is read-only and has no change list. ✖ Ad: deferred edit after a change and a change payload |
| Stark (accessibility) | read colours and text styles, **render nodes to PNG** for simulation, overlay annotations | ◐ reads work. ✖ Ad: `export.png`. Vision simulation is ✔ in Wasm |
| Tokens Studio | variables and styles CRUD, JSON import and export, **GitHub sync** | ◐ the variables CRUD is exposed. ✖ Ad: `variable.import/export`. ✖ Pl: net, secrets |
| Anima, Locofy, Builder.io (code export) and Dev Mode codegen | full node tree, styles, auto-layout props, image export, **codegen hook** in the inspect panel | ◐ node read works. ✖ Ad: `codegen` contribution. The engine has its own codegen and `inspect.code` |
| remove.bg, Magician, AI generators | image fill bytes read and write, net or local model | ✖ Ad: `image.get` (bytes); `raster.filter` is not offered. ✖ Pl: net or inference |
| Charts (Figma Chart, Datavizer) | create vectors and text from data, CSV paste or import | ◐ creation works. ✖ Pl: clipboard and file read in FC |
| Widgets | on-canvas interactive nodes with persistent state, multiplayer | ✖ App/Ad: a widget node kind. The collab layer already syncs plugin data (protocol 3) |

### InDesign and Affinity

| Plugin | What it needs | Our status |
|---|---|---|
| Data Merge (built in), EasyCatalog, InData (Em Software) | records → frames, templates, images, overflow handling | ✔ App has data merge. ✖ Ad: no adapter. ✖ Pl: net for live data sources |
| Typefi | XML or Word to automated layout, scripts, servers | ✖ Ad (the script runner exists). Headless is ✔ through MCP |
| Em Software Xtags | tagged-text import filter | ✖ Pl: `format.import` contract for stories |
| Preflight extensions, imposition (Imposer) | custom preflight rules, page re-ordering, PDF post-processing | ✖ Ad: `preflight.rule` contribution, `pdf.post` hook |
| Affinity: macros plus Photoshop 8bf plugins | record and replay; 8bf loads native filter DLLs | Macros ✔ in PC actions (but plugin sessions are not journaled). 8bf support is native only and out of scope |

## Gaps by layer

Priority: P0 means it blocks a whole plugin category, down to P3 for polish. Effort: S is days, M
is one to three weeks, L is more than a month. The "Wasm-able?" column says whether the work can
stay in the sandbox and how.

### Platform (`CE/`: core, host, kernels, SDK, UI)

| # | Gap | Proposal: ops, shapes, permissions | Pri | Eff | Wasm-able? |
|---|---|---|---|---|---|
| P-1 | **Network** | `net.fetch {method,url,headers,body(b64|text),timeout_ms}` → `{request}`; the answer arrives as a `net.response {request,status,headers,body}` event, and jobs may block on it. Host-side through the store's one HTTPS agent (rustls); HTTPS only; no redirects off the allowlist; size limits of 16 MiB in an event and 256 MiB in a job; per-plugin rate limit. Manifest `permissions = ["network:api.unsplash.com"]` plus a `reasoning` string shown at consent, as in Figma's `networkAccess.reasoning`. Show a first-use toast, and a manager log of hosts contacted | P0 | M | Yes: the host does the I/O and the guest sees bytes |
| P-2 | **Secrets and OAuth** | `secret.get/set/delete {key}` in the OS keychain, per plugin and never synced. `auth.oauth {authorize_url, token_url, client_id, scopes, pkce:true}` opens the system browser, catches the loopback redirect and stores the token as a secret. Needs a `network:` grant for both hosts | P0 | M | Yes |
| P-3 | **Private settings** (`clientStorage`) | `settings.get/set {key,value}` per plugin per user, 64 KiB per key and 1 MiB total, not undoable, survives updates | P0 | S | Yes |
| P-4 | **Change events** | `app` events `document.changed {revision, changes:[{kind:"layer|node|selection|history|color|tool", id?, props?}]}`, coalesced per frame; plus `document.activated`, `history.changed`, `color.changed` and `tool.changed`. Edits from a change event run as a *deferred follow-up event* (one per revision, with loop detection) instead of being refused | P0 | M | Yes |
| P-5 | **Binary pixel transport** | Today `pixels.get/set` use base64 JSON with a 4 MiB read and 1 MiB write cap, so a 24 MP RGBA8 layer is 96 MB, or about 100 calls with roughly 33% inflation. Add `craft.call_buf` (ABI addition with `-3` fallback): an op writes into, or reads from, a `{ptr,len}` in guest memory, as kernels already do. `pixels.read_into {layer,rect,format,ptr}` and `pixels.write_from {…,ptr}` | P0 | M | Yes |
| P-6 | **Host AI inference** | `nn.load {model:"asset:models/x.onnx"|"store:<sha256>", provider:"auto|cpu|gpu"}` → `{model, inputs, outputs}`. `nn.run {model, inputs:{name:{ptr,shape,dtype}}, outputs:{name:{ptr}}}`. Jobs only; fuel charged by FLOPs. Model files are pinned by sha256 in the manifest; large models download through the store's agent with consent (as `LC/crates/fetch` does). Runtime: lift LightCraft's pure-Rust ONNX interpreter and WGSL `nn_conv` (`LC/crates/denoise-core`, `crates/gpu/src/nn.rs`) into craft-libs as `craft-nn`, with candle as a second backend for transformer models (SAM 3 in `LC/crates/segment`). Shape mirrors wasi-nn (load, set-input, compute, get-output) so authors find it familiar. Permission `compute.nn` (shown as "uses on-device AI models; N MB download") | P0 | L | Yes: the guest does pre- and post-processing, the host runs the graph |
| P-7 | **GPU compute shaders** | `gpu.dispatch {wgsl, entry, workgroups:[x,y,z], buffers:[{ptr,len,access}], uniforms}` in jobs and filters. WGSL is validated by naga, with no textures or atomics outside the plugin's buffers, bounded dispatch size, and a watchdog with device-loss recovery that counts a strike. Runs on the app's wgpu 30 device (LC already uses compute; PC needs a compute queue next to its fragment compositor). Fall back to CPU kernels when no GPU is present. Permission-free but tier-gated: Verified and above at first | P1 | L | Yes: shader source is data, and naga is the validator |
| P-8 | **Kernel library growth** | Add the missing building blocks for photo plugins: `kernel.guided_filter`, `surface_blur`, `high_pass`/`frequency_split`, `unsharp`, `lut3d` (apply `.cube`), `color_convert` (sRGB, linear, Lab, HSL via craft-cms), `distance_transform`, `median`, `fft_convolve` (large blur and deconvolution), `remap` (displacement and liquify), `perlin`/`worley`, `tone_map`, `resize` with Lanczos, `stack_reduce` (median and mean of N images, for HDR and noise stacks), `flood_fill`, `connected_components` | P1 | M | Yes |
| P-9 | **ROI, tiles and preview scale for filters** | `raster.filter@2`: the guest declares `padding` (neighbourhood) and `tileable`. The host calls `craft_filter` per tile with `{roi, padding, scale}`; the preview runs at screen proxy scale on the visible ROI only, with results cached by params hash. Also allow `output_size` (upscalers) and `needs: ["composite"|"layer_below"]` | P0 | M | Yes |
| P-10 | **Non-destructive plugin filters and effects** | A cross-app contract: `filter` contributions with `deterministic=true` can be stored as smart filters (PC), appearance effects (VC, FC) or develop stages (LC). The project stores `{plugin, version, params}` plus a cached raster (P5 missing-plugin rules already exist) | P1 | M | Yes |
| P-11 | **Live UI** | Slider and number controls with `live:true` stream throttled values (at most 30 per second), replaced as they arrive. Overlay `image` op and `region` hit areas (`{id, path|rect, cursor}`) that send `overlay` pointer events (hover, down, drag), so plugins get Illustrator- or Figma-style handles without making the person select a tool. `canvas.preview.set {image ptr, rect}` shows an unapplied result over the document (Nik, Lumenzia) | P0 | M | Yes |
| P-12 | **Pen input fidelity** | Tool events: `tilt:[x,y]`, `rotation`, `pointer:"pen|mouse|touch|eraser"`, `hover` phase (`listen:["hover"]`), and `samples:[[x,y,p,t]…]` coalesced per frame at full tablet rate. An `input.filter` contribution transforms the pointer stream before built-in tools (stabiliser, ruler, perspective guides: Lazy Nezumi inside our apps) | P1 | M | Yes |
| P-13 | **Clipboard** | `clipboard.read {types:["text","image/png","image/svg+xml","application/json"]}` and `clipboard.write {items}`. Only during a user-initiated command or ui event, with the existing `clipboard.*` permissions | P1 | S | Yes |
| P-14 | **Drag and drop** | UI `drag_source {id, payload:{kind:"image|svg|text|plugin", data|ref}}` on list, table or image nodes. The host drops the payload onto the canvas or document and the plugin receives `drop {at, target}` to place it (Unsplash, Iconify, asset libraries). Incoming OS drops onto panels: `drop.files` gives picked-file handles | P1 | M | Yes |
| P-15 | **Format plugins** | `format.import@1` and `format.export@1` contributions. The host hands bytes (from Open or Place, at most 1 GiB, through streamed chunks in a job) and gets back a neutral `DocumentDraft` (layers, pixels by buffer, vectors, text runs and metadata) per app family: raster, vector or story. Export receives a flattened or layered draft and returns bytes. Registered in Open, Place and Export dialogs by extension and magic bytes | P1 | L | Yes, with no file-system access |
| P-16 | **Fonts** | Activate `resource.font@1`: plugin-packaged OTF/TTF/WOFF2 registered in the shared fontdb for the session, documents record font references, licence field in the catalogue. `fonts.list` and `fonts.glyphs {family, text}` → outlines for type tools. Font *creation* (Fontself) is a Wasm OTF writer plus `font.install {bytes}` with consent | P1 | M | Yes |
| P-17 | **Package assets** | `asset.read {path}` for files inside the plugin's own package (textures, LUTs, models, icons) through buffer transport; today assets reach only resources | P0 | S | Yes |
| P-18 | **Wasm execution speed** | (a) Enable wasmi `simd` (wasmi 1.0+ implements 128-bit SIMD with native instructions; currently not in `ext-host/Cargo.toml`). (b) Parallel jobs: `job.map {inputs:[…]}` fans tiles out to N sandboxes. (c) An optional Wasmtime tier with Cranelift or Winch, fuel and epoch interruption, for Verified plugins and jobs only, behind the same ABI | P1 (a, b), P2 (c) | S, M, L | Yes |
| P-19 | **Cross-app handoff** | `handoff.open {app:"photocraft", pixels|file handle, return:true}` → `handoff.returned`, for LC→PC "Edit in" and FC→PC image edits, with the suite launcher as broker | P2 | M | Yes |
| P-20 | **Command palette and shortcuts** | Contributions declare default shortcuts and Actions visibility; a `command.invoke` event carries recorded params so Actions, batch and droplets replay plugin session commands | P1 | S | Yes |

### Adapter (per app)

| # | App | Gap | Proposal | Pri | Eff |
|---|---|---|---|---|---|
| A-1 | All | **Allowlisted registry dispatch** (a safe `batchPlay`) | `app.run {command, params}` over a generated allowlist built from each `CommandSpec`, classed as `query`, `edit` or `refused` (file, net, prefs, plugin management, collab, library, undo). Params are validated by the spec's schema (`refuse_unknown_params` exists in FC). Each command declares a permission class (`document.read` or `document.edit` at selection or document scope). `app.commands` lists the allowed set with labels for discovery. This replaces hundreds of hand-written ops: FC goes from 74 to about 500, VC from 0 to about 700 | P0 | M |
| A-2 | VectorCraft | **New adapter** (`vectorcraft.session@1`) | Follow the FC pattern. Port the legacy `VC/crates/plugins` kinds: object filter → command, live effect → `vector.effect@1` in the appearance stack (P-10), exporter → `format.export@1`. Migrate installed plugins, then delete the old loader (suite rule) | P0 | L |
| A-3 | DesignCraft | **New adapter** (`designcraft.session@1`) | Commands, panels, tools and A-1 dispatch. `story.get/set` (styled runs), `style.*`, `table.*`, `datamerge.*`, `preflight.rule@1` contributions (the plugin returns issues for an item) and `story.import` format plugins (Xtags). Events derived from `revision` plus per-item changes | P0 | L |
| A-4 | LightCraft | **New adapter** (`lightcraft.session@1`) | `catalog.query {rules}` (re-uses smart-album rules), `photo.get/metadata.get/set` (EXIF, IPTC, XMP, keywords, rating, custom fields in a plugin namespace), `develop.get/set {photo, settings_patch}` (partial settings JSON as in presets), `photo.render {photo, size, format}` → buffer, `export.service@1` and `publish.service@1` contributions (P-21), `metadata.provider@1` (custom fields shown in the Metadata panel and searchable), `resource.lut/profile/preset` | P0 | L |
| A-5 | PhotoCraft | Layer depth | `mask.get/set` (pixels), `channel.get/set/load_selection`, `layer.set {clipped, locks, blend_if}`, `adjustment.update {layer, params}`, `style.get/set` (layer effects JSON), `text.get/set` (runs, font, size), `path.get/set` (work path and vector masks as Bezier JSON), `smart.place/replace_contents` | P0 | M |
| A-6 | PhotoCraft | Colour | `color.fg/bg get/set`, `swatches.list/add`, `profile.get` (ICC bytes, name), `convert {to:"lab|cmyk|gray"}`. Pixel ops in Gray, CMYK and Lab with channel layout reported | P1 | M |
| A-7 | PhotoCraft | Painting | `paint.stroke {brush:preset id|inline dab spec, points:[[x,y,p,tilt,rot]…], color, mode}` drives the engine brush (`paint/`). `brush.presets.list`. `resource.brush@1` (ABR and native presets). A `brush.dab@1` contribution where Wasm returns dab coverage per stamp through kernels (custom brush engines) | P1 | L |
| A-8 | PhotoCraft | Allowlist growth | Add content-aware fill (`fill {contents:"contentAware"}`), `liquify.apply {mesh}` with plugin-supplied displacement, Camera Raw `raw.develop {settings}` for RGB layers (Nik and Camera Raw class), `document.resize/canvas_size/crop` (upscalers), `document.new {width,height,depth}` and `layer.from_pixels` | P0 | M |
| A-9 | PhotoCraft | Automation | Journal plugin session commands with their params (`journal:true` and a deterministic replay contract), `actions.play {set, action}` (user-confirmed), `document.open/save/close` through picked handles for batch plugins | P1 | M |
| A-10 | PhotoCraft | Metadata | `metadata.get/set {xmp|exif|iptc}`, with namespaced XMP writes requiring `document.edit` | P2 | S |
| A-11 | PhotoCraft | Tool events | Forward tilt and rotation (dropped at `plugin_sessions.rs:368`), hover, and app events `document.changed`, `color.changed`, `tool.changed` | P0 | S |
| A-12 | FigCraft | Raster | Offer `raster.filter` on image fills (`image.get {node}` → buffer, `image.replace` already exists), and `export.png/pdf {ids, scale}` → buffer (Stark, codegen previews, remove.bg) | P0 | M |
| A-13 | FigCraft | Expose engine features | `effect.*`, `text.set/setFeatures/setVariations/fonts/loadFont`, `proto.*`, `align/distribute/transform`, `vector.offset/simplify`, `shapeBuilder`, `data.fill`, `variable.import/export`, `svg.import {text}` → nodes. Most come for free with A-1 | P1 | S (with A-1) |
| A-14 | FigCraft | **Codegen contribution** | `codegen@1`: in Dev Mode's inspect panel the host sends `{event:"codegen", node, language}` and the plugin returns `[{title, language, code}]` within a deadline (Figma uses 3 s). It can call `inspect.code` for the built-in CSS, Swift, Compose or Flutter as a base | P1 | M |
| A-15 | FigCraft | Scope and events | Selection-scoped grants for node ops (deferred today). `document.changed` with node ids (Autoflow), edits in a follow-up event. Pressure on tool events | P0 | M |
| A-16 | FigCraft | Widgets | A widget node kind that renders a plugin UI tree on the canvas, with synced `project.data` as state | P3 | L |

### App (engine features plugins need but the app lacks)

| # | App | Gap | Proposal | Pri | Eff |
|---|---|---|---|---|---|
| X-1 | LightCraft | No export or publish service model | `export.service` hooks around the existing export pipeline: `prepare` (settings UI form), `process` (rendered file buffer, which the plugin may transform or upload), `finish`. `publish` collections with per-photo `{remote_id, url, published_rev}` in the catalogue, and a re-publish queue when develop settings change | P0 | L |
| X-2 | LightCraft | `.cube` LUTs and profiles are CPU only, and there is no plugin profile slot | GPU LUT stage in `crates/gpu`; a profile registry that accepts plugin-provided DCP or LUT profiles (Negative Lab Pro, film packs) | P1 | M |
| X-3 | LightCraft | No custom metadata fields | Catalogue schema for namespaced plugin fields (searchable, in smart-album rules) | P1 | M |
| X-4 | PhotoCraft | No compute pipeline | Add a wgpu compute queue beside the fragment compositor for P-7 and for GPU kernels | P1 | M |
| X-5 | PhotoCraft | Smart filters re-run only built-in filters | Let `apply_smart_filters` (`cmd/smart_cmds/rendering.rs:124`) dispatch `plugin.<id>/<contribution>` through the deterministic filter path, with the missing-plugin cached raster | P1 | M |
| X-6 | DesignCraft | No event bus | Per-item change sets from the command layer (needed by A-3 and live preflight plugins) | P1 | M |
| X-7 | VectorCraft | Appearance stack takes only built-in or legacy plugin effects | Generalise to registry-backed effect ids with plugin-provided geometry or raster output | P0 (with A-2) | M |
| X-8 | All | Command registries are static (`OnceLock<Vec<CommandSpec>>` in DC and LC) | A dynamic overlay registry so plugin commands appear in menus, the palette, Actions and MCP | P1 | S |
| X-9 | FigCraft | No widget node or plugin-owned canvas node | See A-16 | P3 | L |

### What genuinely still needs native

- **System-wide input hooks** (Lazy Nezumi across every app), tethering and camera SDKs,
  colorimeters, and print drivers.
- **Licence or DRM daemons** of commercial vendors, and closed vendor AI runtimes that aren't ONNX
  (TensorRT engines, CoreML-only models). P-6 covers ONNX, and P-7 covers custom kernels.
- **Round trips with other applications** outside the suite (open in Topaz Photo, Luminar or ON1).
- **8bf and Photoshop C++ SDK binary compatibility.**

Everything else in the catalogue above can be Wasm once P-1 to P-7, P-9, P-11 and the per-app
adapters land.

## Performance

**Current numbers.** These come from `docs/plugins/program-2026-11-17.md` and the gap audit.

| Case | Time |
|---|---|
| Pure interpreter, 1080p frame (debug app, before kernels) | about 17 s |
| Softskin, 1080p, through the sandbox | 602 ms |
| Lightwell, 1080p | 478 ms |
| Emberfield, 1080p | 176 ms |

Kernels run on up to 8 threads inside the call and are charged as pixels × taps.

**Where the time goes now.** These bottlenecks come from reading the source; none was measured.

1. **Per-pixel guest loops** in wasmi, with no SIMD. Anything not expressible as one of the 16
   kernels, such as a custom tone curve per pixel or noise synthesis with a custom formula, pays
   about 10× to 30× native.
2. **Transport.** `pixels.get/set` use base64 JSON with a 1 MiB write cap. For a 6000×4000 RGBA16
   layer (192 MB) that is about 190 writes, plus encode and decode.
3. **Whole-layer filters.** With no ROI, a 100 MP PSB means more than 1.6 GB of f32 premultiplied
   RGBA in guest memory. This exceeds the 512 MiB job cap, so large documents fail outright.
4. **Preview.** The dialog preview recomputes the full layer unless the proxy path is used; it was
   not confirmed whether it is.

**What makes sandboxed plugins fast enough, in order of payoff.**

| Lever | Expected gain | Notes |
|---|---|---|
| Host kernels for the inner loop (have 16, add P-8) | 10× to 50× versus a guest loop | Already proven by Softskin. Each new kernel is safe, deterministic and multi-threaded |
| ROI and tiles plus a proxy-scale preview (P-9) | Preview is 10× to 100× cheaper (screen-sized); memory bounded | Matches Photoshop's filter proxy and Lightroom's preview pipeline |
| Binary buffers (P-5) | Removes about 33% inflation and copy passes; makes 1-GB documents feasible | Same pattern kernels already use |
| GPU compute (P-7) on the app's wgpu device | 10× to 100× for local filters, LUTs and blurs at 24 MP; real-time slider scrubbing | LC already has a compute pipeline and WGSL nets. PC needs a compute queue (X-4) |
| Host inference (P-6) | Makes AI plugins possible at all; GPU WGSL convolutions exist in LC | Model weights stay host-side; the guest only does pre- and post-processing |
| wasmi SIMD (P-18a) | About 2× to 4× on vectorisable guest loops | Cheap: a feature flag plus `Config`. Check determinism expectations (the `deterministic` feature) |
| Parallel jobs (P-18b) | Near-linear on cores for tileable work | Tiles map to N sandboxes; results merge host-side |
| JIT tier (P-18c: Wasmtime with Cranelift or Winch) | About 5× to 20× over an interpreter on scalar code | Larger trusted computing base and compile latency; restrict to Verified plugins and jobs; keep wasmi as the default and on platforms without JIT permission |

**Proposed acceptance numbers for the flagship replicas** (the owner's bar): a 24 MP RGB16 layer
on an M-series Mac (24 MP is 6000×4000).

| Plugin class | Target |
|---|---|
| Frequency-separation and skin plugin (Softskin) | under 1.5 s with kernels and tiles |
| Nik-style local-contrast filter | under 100 ms per preview frame on the GPU path |
| 2× AI upscaler or denoise | under 20 s through `nn.run` on the GPU |
| Unsplash-style panel search | under 300 ms to first thumbnails, network permitting |

## Sources

**Repository files (local):**

- `CE/docs/plugins/contracts/photocraft.md` and `figcraft.md`
- `CE/docs/plugins/compute-and-jobs.md` and `ui.md`
- `CE/docs/plugins/program-2026-11-17.md`
- `CE/docs/plugins/research/gap-audit-2026-11-17.md` and `ecosystem-research-2026-11-17.md`
- `CE/crates/ext-host/src/wasm.rs` and `ext-host/Cargo.toml`
- `CE/crates/ext-core/src/permission.rs`
- `CE/crates/guest/src/data.rs`
- PC, FC, VC, DC and LC paths as cited inline

**Web:**

- Photoshop UXP Imaging API: https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/imaging
- UXP Manifest v5 (network domains, requiredPermissions): https://developer.adobe.com/photoshop/uxp/2022/guides/uxp-guide/uxp-misc/manifest-v5/
- UXP `batchPlay`: https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/batchplay
- UXP `action` module (addNotificationListener): https://developer.adobe.com/photoshop/uxp/2022/ps-reference/media/photoshopaction
- Photoshop UXP changelog: https://developer.adobe.com/photoshop/uxp/2022/ps-reference/changelog/
- Forum report that `domains: "all"` was still denied: https://forums.creativeclouddeveloper.com/t/uxp-manifest-network-permission-denied-for-fetch-despite-domains-all/10557
- Figma plugin manifest (`networkAccess`, `allowedDomains`, `reasoning`): https://developers.figma.com/docs/plugins/manifest
- Figma network requests: https://developers.figma.com/docs/plugins/making-network-requests
- Figma global API (`on('documentchange')`, `clientStorage`): https://developers.figma.com/docs/plugins/api/figma/
- Figma codegen plugins (3 s generate timeout): https://developers.figma.com/docs/plugins/codegen-plugins
- Figma `figma.codegen.on`: https://developers.figma.com/docs/plugins/api/properties/figma-codegen-on
- Figma `exportAsync`: https://developers.figma.com/docs/plugins/api/properties/nodes-exportasync
- Figma working with images: https://developers.figma.com/docs/plugins/working-with-images
- Figma widget manifest: https://developers.figma.com/docs/widgets/widget-manifest
- Lightroom Classic plug-ins overview: https://helpx.adobe.com/lightroom-classic/plug-ins.html
- Lightroom SDK export service provider (mirror): https://lrc.mcor.dev/modules/SDK%20-%20Export%20service%20provider.html
- Lightroom SDK publish service provider (mirror): https://lrc.mcor.dev/modules/SDK%20-%20Publish%20service%20provider.html
- Lightroom `LrPublishService` (mirror): https://lrc.mcor.dev/modules/LrPublishService.html
- Nik and other plugins round-trip through TIFF from Lightroom: https://www.uniquephoto.com/community/qa/can-i-round-trip-a-raw-file-from-lightroom-to-photoshopnik-and-keep-it-as-raw
- Negative Lab Pro review: https://www.5050travelog.com/camera-and-lens-reviews/negative-lab-pro-review
- Astute Graphics: https://astutegraphics.com/ and https://docs.astutegraphics.com
- Astute real-time drawing update (ColliderScribe and others): https://astutegraphics.com/learn/update/real-time-drawing-support-is-here
- Tokens Studio: https://www.tokens.studio and changelog https://feedback.tokens.studio/changelog
- wasi-nn: https://github.com/WebAssembly/wasi-nn and Wasmtime backend https://docs.wasmtime.dev/api/src/wasmtime_wasi_nn/backend/mod.rs.html
- wasmi 1.0 (SIMD) and benchmarks: https://wasmi-labs.github.io/blog/posts/wasmi-v1.0 and https://github.com/wasmi-labs/wasmi-benchmarks
- Lazy Nezumi Pro (external pen-input smoother): https://lazynezumi.com and discussion https://polycount.com/discussion/comment/1917757
- Plugin vendor pages via the ecosystem research: Nik https://nikcollection.dxo.com/, Portraiture https://imagenomic.com/Products/Portraiture, Topaz https://www.topazlabs.com/topaz-photo, Retouch4me https://retouch4.me/, Infinite Color https://www.infinitecolorpanel.com/, Lumenzia https://lumenzia.com/, Fontself https://www.fontself.com/, ON1 https://www.on1.com/products/photo-raw/, Coolorus https://coolorus.com/, Iconify https://iconify.design/, Stark https://www.getstark.co/, remove.bg https://www.remove.bg/integrations

Not verified on the web during this pass: Affinity's current 8bf and macro scope, LRTimelapse's
internals, Typefi and Em Software APIs, and Hot Door and MAPublisher feature lists. Statements about
them are from general product knowledge and should be checked before they drive a contract.

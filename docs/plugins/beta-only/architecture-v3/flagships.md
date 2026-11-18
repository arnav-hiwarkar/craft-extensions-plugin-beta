# Flagships

A flagship is a plugin we build ourselves to match a tool people already use. Each one proves that the plugin system can carry real work. Every card names what it matches, its permission sheet with the risk level the host derives from it, the capabilities it needs, and the measured test that decides if it is done. Flagships follow their capability's milestone [DEC-COM-flagship-order], so each starts when the runtime pieces it needs are ready.

## All three apps

These plugins ship as one package that serves EffectCraft, PhotoCraft and FigCraft.

### Deckhand (Stream Deck-class controller bridge) · M6

Controller keys and dials run commands and scrub values in all three apps; key images show live state.

- **Matches:** KBar and script launcher panels, Shortcut Composer pie menus
- **Sheet:** devices.companion:com.elgato.streamdeck, plugins.control · L2
- **Other builds:** Community build adds notice.standard.
- **Capabilities:** CAP-COM-deckhand-features, CAP-COM-shortcut-device-chords
- **Accepted when:** TST-COM-deckhand-targets: On hw-mac and hw-win: key press to command start <=10 ms; 120 Hz dial scrub with 0 dropped events.; TST-COM-deckhand-acceptance: CC1: 10,000 frames; foreign and child peers refused; forged press -11; off-schema frames refused; 0 IP sockets in Craft processes; the bridge connects only to the vendor loopback port. Switch off: required entry held, optional -9 BuildSwitch.

### Keycast (KeyCastr-class keystroke overlay) · M6

Overlay of keystrokes, shortcuts, clicks and clipboard actions across all programs while recording, with presets and filters.

- **Matches:** Shortcut Composer pie menus, KBar and script launcher panels
- **Sheet:** input.monitor, clipboard.monitor, ui.window:overlay, app.events:command · L4 · two-step consent, notice.full_access
- **Capabilities:** CAP-COM-keycast-features, CAP-COM-surface-elevated-window-levels
- **Accepted when:** TST-COM-keycast-targets: Key-down to overlay paint <=1 frame + 4 ms p99; 0 guest calls per idle frame; overlay never takes focus.; TST-COM-keycast-acceptance: Stop-all kills craft-input-monitor and the overlay; secure fields give 0 key events; pointer passes through; 100% of clipboard reads logged by size and type.

### Kiln (Blender bridge) · M6

Sends the selection to the user's Blender, where a packaged add-on builds the scene; results return as footage, layers or fills.

- **Matches:** Cineware-class 3D renderer
- **Sheet:** process.run:blender (full), native.full, document.edit:document, ui.open_url · L4 · two-step consent, notice.full_access
- **Other builds:** Community build (D-G) adds notice.standard.
- **Capabilities:** CAP-COM-kiln-features
- **Accepted when:** TST-COM-kiln-targets: 200 layers to Blender <=2 s; mirror 1080p >=30 fps, <=2 frames latency; 300-frame EXR import <=3 s; cancel <=500 ms.; TST-COM-kiln-acceptance: Two-step sheet (1.5 s inert Enable) matches snapshots for both builds; Community has notice.standard, signer pin, provenance; Blender opens 0 IP sockets; Stop-all ends Blender and kiln-live; a kiln-live crash keeps the last frame.

### Prism (gradient and colour grading filter) · M4

One package, three apps: gradient maps, split toning, looks and LUT export as an EffectCraft effect, a PhotoCraft filter and a FigCraft fill, native and web.

- **Matches:** Magic Bullet Looks (Maxon), Lumetri-class grading (FilmConvert), LUT packs
- **Sheet:** document.edit:selection, files.write:picked · L0
- **Capabilities:** CAP-COM-prism-features
- **Accepted when:** TST-COM-prism-targets: GT1 pass <=0.5 ms at 1080p 16F with 0 readbacks; Wasm <=1.1x in-process.; TST-COM-prism-acceptance: Every app, native and web, within tolerant of the CPU reference; Wasm exact; one source tree.

### Relay (Overlord-class live link) · M6

Live link between EffectCraft, PhotoCraft and FigCraft; VectorCraft joins in WS15 at M8 (TST-COM-ws15-app-parity).

- **Matches:** 7.32 Overlord
- **Sheet:** document.edit:document, clipboard.read, clipboard.monitor · L2 · suite reach disclosed
- **Capabilities:** CAP-COM-relay-features, CAP-COM-filmcraft-and-other-apps-follow
- **Accepted when:** TST-COM-relay-targets: Path <=5 ms across apps; 4K RGBA16F frame <=16.6 ms with <=1 copy; 0 IP sockets.; TST-COM-relay-acceptance: suite_bus_round_trip; SB1 sheet text; 100% of clipboard reads logged; chip hidden with the clipboard switch off; EffectCraft, PhotoCraft and FigCraft only.


## EffectCraft

### Beacon flagship · M6

Desktop Video-class output (M6) on DeckLink and AJA: HDR, 16 audio channels, device clock.

- **Matches:** 7.47 AJA, Blackmagic Desktop Video
- **Sheet:** document.read:all, native.restricted · L3
- **Capabilities:** CAP-EC-beacon-features
- **Accepted when:** TST-EC-beacon-targets: 1080p59.94/60 for 60 s: 0 drops, latency <=1 frame; 2160p59.94 0 drops on hw-win.; TST-EC-beacon-acceptance: VO1 0 drops; crash turns output off with a strike; stall strip at 1 s; 0 IP sockets.; TST-EC-sheet-derivation: Sheets derived for 9 flagships and the synthesised aehost, OFX and AEIO manifests of Halo and Strata match with 0 differences; own-layer render.request derives no grant; L3, comp.layers, renderer3d@1, job.spawn or native code without its grant fails validation.

### Cadence flagship · M6

Cadence, Sound Keys and BeatEdit-class audio reactivity, built at M6.

- **Matches:** Trapcode Sound Keys (Maxon Red Giant)
- **Sheet:** document.read:document, document.edit:selection · L1
- **Capabilities:** CAP-EC-cadence-features, CAP-EC-param-output, CAP-EC-kernel-fft-spectrum, CAP-EC-audio-effects
- **Accepted when:** TST-EC-cadence-targets: 5-minute 48 kHz stereo beat analysis <=2 s; bake of three outputs over 10,000 frames <=300 ms; range-box drag updates the playhead output within 1 vsync p95; 50 overlapping drags leave only the newest bake, and one undo or redo restores keys exactly; spectrum row 60 fps, 0 guest calls when still.; TST-EC-cadence-acceptance: beats-120bpm and beats-variable: onset F1 >=0.97 within +-10 ms; keys play identically without Cadence; audio.read in render refused; ducker <=50% of block, 0 allocations; keyframed threshold and sidechain bit-identical at blocks 64, 512, 4096, playback and export; out-of-scope sidechain -2, silent.

### Cue flagship · M6

Cue, Animation Composer-class preset browser in a CEP build, built at M6.

- **Matches:** Animation Composer and Motion Bro
- **Sheet:** document.edit:document, document.scripts, files.folder:picked, process.run:unpack (confined) · L3 · notice.local, notice.files_programs
- **Capabilities:** CAP-EC-cue-features, CAP-EC-cep-import
- **Accepted when:** TST-EC-cue-targets: the docked 600×800 panel holds 60 fps scrolling 2,000 presets with only visible thumbnails decoding; apply ≤100 ms as one undo step; warm panel first paint ≤300 ms.; TST-EC-cue-acceptance: Import changes no file; EC-T50 holds on the real plugin; the derived sheet lists exactly what the CEP manifest and code use.

### Ember flagship · M5

Particular-class GPU particles (M5): emitters, physics, sprites, 10M particles, Designer.

- **Matches:** Trapcode Particular (Maxon Red Giant)
- **Sheet:** document.read:document, document.edit:selection · L1
- **Capabilities:** CAP-EC-ember-features
- **Accepted when:** TST-EC-ember-targets: 1M particles 1080p30 0 drops; seek <=250 ms; worker <=50 us per frame; 0 readbacks.; TST-EC-ember-acceptance: Tolerant goldens on 3 backends; gpu_lost resumes in <=5 s; RAM preview writes 0 durable bytes.; TST-EC-sheet-derivation: Sheets derived for 9 flagships and the synthesised aehost, OFX and AEIO manifests of Halo and Strata match with 0 differences; own-layer render.request derives no grant; L3, comp.layers, renderer3d@1, job.spawn or native code without its grant fails validation.; TST-EC-sim-10m-seek: 10M particles on hw-win: readback <=0.5 ms per frame; seek <=250 ms; an 18,000-frame range at the disk cap seeks within ceil(range/N) steps.

### EffectCraft flagship ports · M6

Ports of top After Effects plugins covering every EffectCraft contract (EC-T30).

- **Matches:** Trapcode Particular (Maxon Red Giant), Optical Flares (Video Copilot), Twixtor (RE:Vision Effects), Duik Angela (RxLaboratory), Flow (Mt. Mograph), Rotobot (Kognat), Red Giant Universe (Maxon), Element 3D (Video Copilot), Mocha Pro (Boris FX), Neat Video (Neat Lab), Trapcode Sound Keys (Maxon Red Giant), 7.33 BeatEdit, Cryptomatte and ProEXR, 7.47 AJA, Blackmagic Desktop Video, Animation Composer and Motion Bro
- **Capabilities:** CAP-EC-time-window
- **Accepted when:** TST-EC-ec-t30: Particular, Optical Flares, Twixtor, Duik, Flow, Rotobot, Universe, Element 3D, Mocha, Neat Video, Sound Keys, BeatEdit, ProEXR, Cryptomatte, video output and Animation Composer ports pass golden and acceptance tests.; TST-COM-port-benches: Each EC-T30 port and FigCraft proof plugin names a scene in its bench record; time <=1.0x its incumbent's on the same machine, on every class where the incumbent ships.

### Glide flagship · M4

Glide, Flow-class ease curves, built at M4.

- **Matches:** Flow (Mt. Mograph)
- **Sheet:** document.edit:selection · L0
- **Capabilities:** CAP-EC-glide-features, CAP-EC-ease-sampling
- **Accepted when:** TST-EC-glide-targets: apply to 10,000 keyframes ≤50 ms host time; panel interaction zero guest calls per frame.; TST-EC-glide-acceptance: project without Glide plays identical motion from the baked tables; C22 painter goldens of the curve editor under every theme in `ext-ui-paint`.

### Halo flagship · M5

Optical Flares-class flares (M5) with designer, light sources, occlusion; hosted AE and OFX builds.

- **Matches:** Optical Flares (Video Copilot)
- **Sheet:** document.read:document, document.edit:selection · L1
- **Other builds:** Hosted AE and OFX builds: native.restricted · L3, notice.local, notice.native_ui, notice.native_host.
- **Capabilities:** CAP-EC-halo-features
- **Accepted when:** TST-EC-halo-targets: 30 elements at 4K <=3 ms; 0 IPC unanimated; hosted <=0.3 ms sync.; TST-EC-halo-acceptance: <=3 passes for 30 elements; tolerant goldens for all 3 builds; hosted builds pass EC-T41b.; TST-EC-sheet-derivation: Sheets derived for 9 flagships and the synthesised aehost, OFX and AEIO manifests of Halo and Strata match with 0 differences; own-layer render.request derives no grant; L3, comp.layers, renderer3d@1, job.spawn or native code without its grant fails validation.

### Hush flagship · M5

Neat Video-class temporal denoiser (M5) with noise profiles.

- **Matches:** Neat Video (Neat Lab)
- **Sheet:** document.edit:selection, files.read:picked · L0
- **Capabilities:** CAP-EC-hush-features
- **Accepted when:** TST-EC-hush-targets: 1080p radius 2 <=25 ms on hw-win; 4K <=100 ms; profile <=300 ms.; TST-EC-hush-acceptance: ISO6400 fixture: >=37 dB PSNR, SSIM >=0.95; an edit re-renders exactly s-5..s+5.; TST-EC-sheet-derivation: Sheets derived for 9 flagships and the synthesised aehost, OFX and AEIO manifests of Halo and Strata match with 0 differences; own-layer render.request derives no grant; L3, comp.layers, renderer3d@1, job.spawn or native code without its grant fails validation.

### Isolate flagship · M6

Rotobot-class AI roto (M6) with refine and export job.

- **Matches:** Rotobot (Kognat)
- **Sheet:** document.edit:selection, app.automation · L1
- **Capabilities:** CAP-EC-isolate-features
- **Accepted when:** TST-EC-isolate-targets: 1080p playback with 0 readbacks; export at full model size.; TST-EC-isolate-acceptance: Mean IoU >=0.95, boundary F >=0.90; helper kill keeps playback.; TST-EC-sheet-derivation: Sheets derived for 9 flagships and the synthesised aehost, OFX and AEIO manifests of Halo and Strata match with 0 differences; own-layer render.request derives no grant; L3, comp.layers, renderer3d@1, job.spawn or native code without its grant fails validation.

### Monolith flagship · M6

Element 3D-class renderer (M6): models, extruded text, PBR, shadows, scene editor.

- **Matches:** Element 3D (Video Copilot)
- **Sheet:** document.read:document, document.edit:selection, files.read:picked, gpu.direct · L1
- **Capabilities:** CAP-EC-monolith-features
- **Accepted when:** TST-EC-monolith-targets: 20,000 draws <=0.2 ms validation; 1080p30 shadows and 4x MSAA 0 drops.; TST-EC-monolith-acceptance: G12 tolerant goldens on 3 backends; refusal fixtures for every L3 rule.; TST-EC-sheet-derivation: Sheets derived for 9 flagships and the synthesised aehost, OFX and AEIO manifests of Halo and Strata match with 0 differences; own-layer render.request derives no grant; L3, comp.layers, renderer3d@1, job.spawn or native code without its grant fails validation.

### Rigwise flagship · M5

Rigwise, Duik-class rigging, built at M5.

- **Matches:** Duik Angela (RxLaboratory)
- **Sheet:** document.edit:document, document.scripts, files.read:picked, app.scripts · L2
- **Capabilities:** CAP-EC-rigwise-features, CAP-EC-jit-expression-rigs, CAP-EC-plugin-scripts-gatebackend
- **Accepted when:** TST-EC-rigwise-targets: rig of 1,000 steps with reads between steps ≤300 ms; a 200-layer rig plays at 1080p30 with 0 dropped frames after one pre-roll; one undo step per rig.; TST-EC-rigwise-acceptance: duik_rig_one_ipc; EC-T51 parity on the rig's expressions; expr-boa counter = 0 for tainted properties; EC-T34 and EC-T20 on its fn() helpers (identical renders from bakes with Rigwise removed); a .jsx rig import without app.scripts returns -2 {needs: app.scripts}.

### Seam flagship · M5

Universe-class transitions and generators (M5) with preset thumbnails.

- **Matches:** Red Giant Universe (Maxon)
- **Sheet:** document.edit:selection · L0
- **Capabilities:** CAP-EC-seam-features
- **Accepted when:** TST-EC-seam-targets: Every transition at 4K <=4 ms on hw-win; preset apply <=50 ms.; TST-EC-seam-acceptance: Tolerant goldens at 0-100%; endpoints bit-exact to the layers.; TST-EC-sheet-derivation: Sheets derived for 9 flagships and the synthesised aehost, OFX and AEIO manifests of Halo and Strata match with 0 differences; own-layer render.request derives no grant; L3, comp.layers, renderer3d@1, job.spawn or native code without its grant fails validation.

### Spectra flagship · M6

Spectra, Universe and FX Console-class effect browser, built at M6.

- **Matches:** Red Giant Universe (Maxon)
- **Sheet:** document.read:document, document.edit:selection, plugins.control · L2
- **Capabilities:** CAP-EC-spectra-features
- **Accepted when:** TST-EC-spectra-targets: 100 previews of a 2,000-layer project leave the document hash, undo stack and events unchanged; preview thumbnails p95 ≤50 ms when idle; HUD opens within one frame of key-down.; TST-EC-spectra-acceptance: whatif_100_previews_no_trace; PR1 confused-deputy rules for plugins.run; the sheet shows one agent toggle for spectra.apply and an agent call with it off returns -2.

### Strata flagship · M5

ProEXR and Cryptomatte-class EXR (M5): multipart import, matte pick, aux EXR output; AEIO build.

- **Matches:** Cryptomatte and ProEXR
- **Sheet:** document.edit:selection · L0
- **Other builds:** AEIO build: native.restricted · L3, notice.local, notice.native_ui, notice.native_host.
- **Capabilities:** CAP-EC-strata-features
- **Accepted when:** TST-EC-strata-targets: 4K 24-channel decode <=40 ms; 300 4K frames written <=60 s.; TST-EC-strata-acceptance: 24-channel EXR round trip bit-identical; picked matte exact; AEIO output byte-identical.; TST-EC-sheet-derivation: Sheets derived for 9 flagships and the synthesised aehost, OFX and AEIO manifests of Halo and Strata match with 0 differences; own-layer render.request derives no grant; L3, comp.layers, renderer3d@1, job.spawn or native code without its grant fails validation.

### Tempo flagship · M6

Twixtor-class flow retiming (M6) with vector aux.

- **Matches:** Twixtor (RE:Vision Effects)
- **Sheet:** document.read:selection, document.edit:selection, gpu.direct, app.automation · L1
- **Capabilities:** CAP-EC-tempo-features
- **Accepted when:** TST-EC-tempo-targets: 1080p flow <=12 ms on hw-win; 300-frame pre-pass <=20 s.; TST-EC-tempo-acceptance: Held-out frames: mean PSNR >=33 dB with flow, >=35 dB with the model.; TST-EC-sheet-derivation: Sheets derived for 9 flagships and the synthesised aehost, OFX and AEIO manifests of Halo and Strata match with 0 differences; own-layer render.request derives no grant; L3, comp.layers, renderer3d@1, job.spawn or native code without its grant fails validation.

### Tracer flagship · M5

Tracer, Mocha-class planar tracking and roto, built at M5.

- **Matches:** Mocha Pro (Boris FX)
- **Sheet:** document.edit:document · L1
- **Capabilities:** CAP-EC-tracer-features
- **Accepted when:** TST-EC-tracer-targets: tracking 1080p at ≥60 frames per second on each class; pen stroke visible in the frame of the sample.; TST-EC-tracer-acceptance: on fixtures/track/planar-1080p (300 frames, known homographies, motion blur, 30% occlusion) corner-pin RMS <=0.25 px and max <=1.0 px; one undo step per roto edit; lane goldens C22; timed overlays scrub with zero guest calls.


## PhotoCraft

### Argent flagship · M5

Silver Efex-class black-and-white with control points, built at M5.

- **Matches:** Nik Silver Efex
- **Sheet:** document.edit:selection · L0
- **Other builds:** 8bf build: synthesised craft-pshost manifest, native.restricted · L3, notice.local, notice.native_ui, notice.native_host.
- **Capabilities:** CAP-PC-argent-features
- **Accepted when:** TST-PC-argent-targets: 100 MP filter <=1.0x Silver Efex in Photoshop on the same D-T machine, CPU and GPU; GPU preview <=16 ms p95; 8bf build <=40 us per band and <=1.0x the same 8bf in Photoshop.; TST-PC-argent-acceptance: Wasm exact and GPU tolerant in RGB, CMYK and Lab (deltaE2000 mean <=0.3, max <=1.0); control-point drag is 1 undo step with 0 guest calls per frame; the 8bf build is tolerant everywhere and a crash costs 1 strike.

### Facet flagship · M5

NVIDIA Texture Tools-class DDS read and write, built at M5.

- **Matches:** NVIDIA Texture Tools Exporter
- **Sheet:** document.create, files.folder:picked · L1
- **Capabilities:** CAP-PC-facet-features
- **Accepted when:** TST-PC-facet-targets: Open 4K BC7 with mips ≤150 ms; save 4K BC7 normal ≤2 s; 500 2K textures ≤3 min, cancel ≤100 ms; save and batch ≤1.0x NVTT Exporter (bench.lock build).; TST-PC-facet-acceptance: BC1-BC7 decode exact vs reference; BC7 normal quality >=42 dB PSNR on Kodak; uncompressed round trip byte-identical; malformed DDS fuzz refused with 0 crashes; cancelled batch leaves 0 outputs.

### Grainbox (dry media and colour wheel) · M5

Grainbox, M5: 60 dry-media brushes with grain fixed to the canvas, and a colour wheel with gamut masks and harmonies replacing the picker for every pick.

- **Matches:** Kyle T. Webster brushes, Coolorus
- **Sheet:** document.edit:selection, app.session, app.intercept:color.picker · L2
- **Capabilities:** CAP-PC-grainbox-features
- **Accepted when:** TST-PC-grainbox-targets: 2,000 dabs/s at 500 px on 100 MP, ink 1 frame + 4 ms p99; picker opens in one frame.; TST-PC-grainbox-acceptance: Exact replay; disabling restores the built-in picker; CMYK and Lab commits within dE2000 0.5.

### Lumina flagship · M5

Topaz-class denoise, sharpen, face recovery and upscale, built at M5.

- **Matches:** Topaz Photo AI, Topaz Gigapixel
- **Sheet:** document.edit:document, document.create, files.folder:picked · L1
- **Capabilities:** CAP-PC-lumina-features
- **Accepted when:** TST-COM-ai-upscale: 24 MP 2x upscale <=8 s CoreML and <=20 s candle on hw-mac; PSNR >=40 dB vs candle; slider preview first pixels <=50 ms at fit on 100 MP; wrong sha256 refused; cancel <=100 ms.; TST-PC-lumina-acceptance: 4 GB model loads at defaults; nn-class output on every backend; durable output renders without Lumina; one edit saves only changed tiles; wrong sha256 refused; cancelled batch leaves 0 outputs.

### Oilworks (wet media engine) · M5

Oilworks, M5: oil and watercolour brushes with pickup, bristles and a canvas wet between strokes; pressure, tilt and twist; .pcbrush packs.

- **Matches:** Brush engines (MyPaint, Grutbrushes, live oils, wet media)
- **Sheet:** document.edit:selection, gpu.direct · L1
- **Capabilities:** CAP-PC-oilworks-features
- **Accepted when:** TST-PC-oilworks-targets: Ink 1 frame + 4 ms p99 at 240 Hz, 2,000 px brush, 100 MP; commit 60 ms off the UI thread.; TST-PC-oilworks-acceptance: Bit-exact replay; dE2000 1 across adapters; 0 dropped samples at 240 Hz throttled.

### Sheetkit (code exporter) · M6

Sheetkit, M6: exports layers, text and shapes to HTML/CSS and React with assets, as a format and as an export step, with a page preview.

- **Matches:** Export Kit
- **Sheet:** document.read:document, files.folder:picked, ui.web, app.intercept:before-export · L1
- **Capabilities:** CAP-PC-sheetkit-features
- **Accepted when:** TST-PC-sheetkit-targets: 300-layer export in 5 s; hook in 500 ms.; TST-PC-sheetkit-acceptance: 0 DNS queries; folder matches the fixture; primary export unchanged by the step.

### Steady (input filter) · M4

Steady, M4: pulled-string, average and dynamic stabilisation, symmetry, perspective rulers and ellipse guides.

- **Matches:** Lazy Nezumi Pro
- **Sheet:** app.intercept:input.filter · L1
- **Capabilities:** CAP-PC-steady-features, CAP-PC-pen-symmetry-fanout
- **Accepted when:** TST-PC-steady-targets: 2 ms p99 per 32-lane batch, 1 history step; Oilworks and Grainbox hold lane 0 at frame + 4 ms p99 at 8, 12 and 32 lanes, copies in 2 frames.; TST-PC-steady-acceptance: With the filter stopped raw input resumes in 8 ms; Stop-all reaches the host first in 100%.

### Tether (camera tethering) · M6

Tether, M6: PTP camera control, docked live view, auto-import into the document or a picked folder, full-screen review on a second display.

- **Matches:** Tethering (Smart Shooter class)
- **Sheet:** devices.usb:<vid>:<pid>, files.folder:picked, document.edit:document, ui.window:fullscreen · L2
- **Capabilities:** CAP-PC-tether-features
- **Accepted when:** TST-PC-tether-targets: Live view 24 fps, 2 frames latency; 24 MP capture to layer in 1 s.; TST-PC-tether-acceptance: Fixture PTP session on every runner; device released 260 ms after revoke.

### Tonemask (luminosity masks) · M5

Tonemask, M5: docked panel generating luminosity, zone and colour masks with canvas preview, applied to layers, groups and channels and recorded into Actions.

- **Matches:** Lumenzia
- **Sheet:** document.edit:document, app.events:selection · L1
- **Other builds:** UXP build adds native.full · L4, two-step consent, notice.local, notice.full_access.
- **Capabilities:** CAP-PC-tonemask-features
- **Accepted when:** TST-PC-tonemask-targets: 50 MP masks in 300 ms (UXP 450 ms); buttons 100 ms.; TST-PC-tonemask-acceptance: Actions replay identically; UXP and JS mask hashes equal; an addon crash fails 1 call.


## FigCraft

### Codeshift flagship · M5

Tailwind, SwiftUI and Flutter code for the Dev Mode selection with preferences, copy and export; M5.

- **Matches:** Figma to Code
- **Sheet:** document.read:selection, ui.web · L1
- **Capabilities:** CAP-FC-codeshift-features
- **Accepted when:** TST-FC-codeshift-targets: 1,000-node selection to code <=200 ms; warm showUI <=150 ms.; TST-FC-codeshift-acceptance: Output equals fixtures byte for byte on desktop and web; compat UI budgets hold.

### Cutout flagship · M5

Remove image-fill backgrounds with a packaged model, refine edges, Select > Subject in PhotoCraft; M5.

- **Matches:** Remove BG
- **Sheet:** document.edit:selection · L0
- **Other builds:** PhotoCraft build: selection.write · L0.
- **Capabilities:** CAP-FC-cutout-features
- **Accepted when:** TST-FC-cutout-targets: 4K image <=1.5 s with UI p99 <=1 vsync.; TST-FC-cutout-acceptance: Mean IoU >=0.97 on products-4k; durable shown with a badge without Cutout; 1 undo step.

### Figma parity flagship · beta

30 open-source plugins, widgets and codegen plugins run unmodified through figma-compat; widgets know who clicked and work for every collaborator; the gate generates the compatibility table.

- **Matches:** Tokens Studio, Figma to Code, Voting and poll widgets, Iconify, Sketch Actions, Sketch Data Suppliers, Sketch Assistants, Sketch webviews, Sketch CocoaScript, Penpot scopes, Penpot iframe UI
- **Capabilities:** CAP-FC-figma-compat, CAP-FC-widget-install-sheet, CAP-FC-ui-parity
- **Accepted when:** TST-FC-figma_corpus_30: 30 pinned plugins, widgets and codegen run unmodified offline: hashes equal fixtures for 30 of 30.; TST-FC-widget_click_carries_user_identity: Click gets the user without identity.user; 2 peers give 2 votes, 2 windows 1; 1 undo step, peers <=100 ms, no collab.write; a files.read widget without identity.user gets 0 ids; sheet snapshot names it.

### Flowline flagship · M4

Connectors between frames that follow moves and snap to frame edges, with styles, labels and relaunch; M4.

- **Matches:** Autoflow
- **Sheet:** document.edit:document, app.automation, collab.write · L2 · notice.live_session
- **Capabilities:** CAP-FC-flowline-features, CAP-FC-kind-snap-provider
- **Accepted when:** TST-FC-flowline-targets: 50 connectors in a drag: UI p99 <=1 vsync, 0 frames behind.; TST-FC-flowline-acceptance: Connectors redraw on A within 1 frame at 60 and 120 Hz, on B <=100 ms; runner-only redraw <=100 ms; 1 undo; <=30 broadcasts/s.

### Lens flagship · M5

Colour-blindness and contrast views over the canvas, contrast lint with fixes, focus order and WCAG links; M5.

- **Matches:** Stark
- **Sheet:** document.read:document, document.edit:document, app.intercept:before-save, ui.open_url · L1
- **Capabilities:** CAP-FC-lens-features
- **Accepted when:** TST-FC-lens-targets: Display pass <=0.5 ms GPU at 4K; lint of 10,000 nodes <=200 ms.; TST-FC-lens-acceptance: Passes reach 0 documents or exports; simulation within tolerance; ui.open_url outside a click -11 Gesture.

### Top 48 plugins work offline · beta

All 48 top Figma plugins work with no plugin network: 31 at full parity, 8 through an offline equivalent, 9 at parity minus a network-only part.

- **Matches:** Autoflow, Content Reel, Iconify, Stark, Tokens Studio, Anima, Locofy, Builder.io Copilot, Remove BG, Lorem ipsum, Chart, Blush, Unsplash, Design Lint, Similayer, Instance Finder, Batch Styler, Rename It, Clean Document, Arc, Isometric, Map Maker, LottieFiles, html.to.design, Figmotion, Super Tidy, Find and Replace, Table Creator, Mockuuups / Angle, Smooth Shadow, Easing Gradients, Spell checker, Translator, Crowdin, Localazy, Variables Import/Export, Style Organizer, Font pickers, A11y contrast, Coolors-style palettes, UI kits, Figma to Code, Voting and poll widgets, EightShapes Specs / Redlines, Magician, Sketch Runner-style launchers, Blobs, Get Waves, Pathfinder-class, GIF and video fills
- **Capabilities:** CAP-FC-offline-forms, CAP-FC-nn-run-packaged, CAP-FC-bridge-all-commands
- **Accepted when:** TST-FC-plugin-corpus-offline: 48 plugins work offline: 31 full, 8 equivalent, 9 minus network parts, 0 impossible.

### FigCraft proof plugins · M6

Tokens folder sync, Remove BG, Autoflow, poll and checklist widgets, a Stark-class checker, Tailwind codegen.

- **Matches:** Tokens Studio, Remove BG, Autoflow, Voting and poll widgets, Stark, Figma to Code
- **Capabilities:** CAP-FC-figma-compat, CAP-FC-kind-widget
- **Accepted when:** TST-FC-figma_corpus_30: 30 pinned plugins, widgets and codegen run unmodified offline: hashes equal fixtures for 30 of 30.; TST-COM-port-benches: Each EC-T30 port and FigCraft proof plugin names a scene in its bench record; time <=1.0x its incumbent's on the same machine, on every class where the incumbent ships.

### Pulse flagship · M5

Poll widget with one vote per user.id, results and property menu; Present results spotlights for everyone; M5.

- **Matches:** Voting and poll widgets
- **Sheet:** collab.presence · L1 · widget votes need no collab.write [CON-COM-collab-live-write@1]
- **Capabilities:** CAP-FC-pulse-features
- **Accepted when:** TST-FC-pulse-targets: Click to snapshot <=50 ms locally, <=100 ms on a peer.; TST-FC-pulse-acceptance: Votes count once per person across windows; peer without Pulse installs and replays; local spotlight outside a gesture.; TST-FC-fc-bench-widgets-10k: p99 <=1 vsync.

### Tasklet flagship · M5

Checklist widget with peer assignees, due markers and progress; Follow and Spotlight me; templates; M5.

- **Matches:** Voting and poll widgets
- **Sheet:** collab.read, collab.presence · L1 · widget checks need no collab.write [CON-COM-collab-live-write@1]
- **Capabilities:** CAP-FC-tasklet-features
- **Accepted when:** TST-FC-tasklet-targets: 500 Tasklets x 50 items at 60 fps, 0 guest calls; peer update <=100 ms.; TST-FC-tasklet-acceptance: 2 peers' checks converge; presence outside a gesture stays local.

### Tokenbook flagship · M5

Tokens as JSON in a user folder synced to variables and styles, themes, code export and local Git commits (the user pushes); M5.

- **Matches:** Tokens Studio
- **Sheet:** files.folder:picked, files.watch, document.edit:document, app.automation, process.run:git (confined) · L3 · notice.files_programs, notice.sync
- **Capabilities:** CAP-FC-tokenbook-features
- **Accepted when:** TST-FC-tokenbook-targets: 2,000 tokens <=500 ms as 1 undo step; external edit <=1 s; 500-file commit <=2 s.; TST-FC-tokenbook-acceptance: Watch re-syncs; revoked folder -2; confined git reads 0 outside files, opens 0 sockets; sheet snapshot carries notice.files_programs and notice.sync, 0 differences.


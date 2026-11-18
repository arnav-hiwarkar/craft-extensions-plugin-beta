# Audio plugin APIs, Wasm audio, runtime performance, product catalogue: web notes

Compiled 2026-11-18. Every claim has a URL. Tags: **[fetched]** means the page was read directly. **[search]** means the claim comes from a search-result summary. **[unverified]** means the URL is a canonical vendor page that was not fetched during this pass, so the claim is from background knowledge and should be checked before it is relied on.

## 1. CLAP

- **Version.** The current header is CLAP 1.2.10 (MAJOR 1, MINOR 2, REVISION 10). Major version 1 means the ABI and API are stable. A plugin built against 1.x loads in any 1.y host. [fetched] https://raw.githubusercontent.com/free-audio/clap/main/include/clap/version.h and https://github.com/free-audio/clap
- **Thread model.** CLAP defines two symbolic threads, `main-thread` and `audio-thread`. "audio-thread" is a concurrency guard, not one OS thread. A host may run `process()` on different pool threads over time, and a simple host may make audio-thread the same as main-thread. Functions are tagged `[main-thread]`, `[audio-thread]`, `[thread-safe]` or `[thread-safe, !audio-thread]`. [search] https://sources.debian.org/src/lsp-plugins/1.2.5-1/modules/lsp-3rd-party/include/clap/ext/thread-check.h and https://github.com/free-audio/clap/pull/255
  - In plugin.h, `process` runs on the audio thread while the plugin is active, and `init`/`destroy` run on the main thread while it is inactive. The plugin calls `request_callback` and the host answers with `on_main_thread`. [search] https://git.kx.studio/falkTX/Carla/src/branch/main/source/includes/clap/plugin.h
  - The thread-check extension gives the host `is_main_thread()` and `is_audio_thread()`. [search] same thread-check.h URL as above.
- **Thread-pool extension.** The plugin implements `exec(task_index)`. The host's `request_exec(num_tasks)` may only be called from the audio thread inside `process()`. The header warns that using a pool can break hard real-time rules because of synchronization, and a host under real-time pressure may withhold the extension. [search] https://sources.debian.org/src/lsp-plugins/1.2.5-1/modules/lsp-3rd-party/include/clap/ext/thread-pool.h and https://cleveraudio.org/1-feature-overview/_thread-pool/
- **Params, automation and modulation.**
  - `count`, `get_info`, `get_value`, `value_to_text` and `text_to_value` are `[main-thread]`.
  - `flush` runs on `[active ? audio-thread : main-thread]` and must not overlap `process()`. It delivers parameter changes when the plugin is not processing; inside `process()` the sample offset is preserved, while `flush` loses it.
  - The host's `request_flush` is `[thread-safe, !audio-thread]`. `rescan` and `clear` are main-thread only.
  - Capability flags: `IS_AUTOMATABLE`, `IS_AUTOMATABLE_PER_NOTE_ID/KEY/CHANNEL/PORT`, `IS_MODULATABLE`, and `IS_MODULATABLE_PER_NOTE_ID/KEY/...`.
  - Hosts are advised to "store modulation amount in plain value delta, not in percentage".
  - `CLAP_PARAM_RESCAN_ALL` is only allowed while the plugin is deactivated; otherwise the plugin calls `host->restart()`.
  - [fetched] https://raw.githubusercontent.com/free-audio/clap/main/include/clap/ext/params.h
- **Events and non-destructive modulation.** `PARAM_VALUE` sets the value and `PARAM_MOD` sets a modulation amount. "The value heard is: param_value + param_mod." Modulation is therefore an offset that never overwrites the base or automation value; this is the "non-destructive" property. The event header `time` is a "sample offset within the buffer". Input events "will be delivered sorted in sample order". `note_id`, `port`, `channel` and `key` take a value >= 0 or -1 as a wildcard, which allows per-voice (polyphonic) targeting. [fetched] https://raw.githubusercontent.com/free-audio/clap/main/include/clap/events.h
- **Note expressions.** These are polyphonic per-note events with the IDs VOLUME(0), PAN(1), TUNING(2), VIBRATO(3), EXPRESSION(4), BRIGHTNESS(5) and PRESSURE(6). Volume is 0 < x <= 4, with plain = 20*log(x). The `voice-info` extension "is important for polyphonic modulations". [fetched] events.h URL above and https://github.com/free-audio/clap
- **Latency.** `get()` is `[main-thread & (being-activated | active)]`. The host's `changed()` is `[main-thread & being-activated]`. "The latency is only allowed to change during plugin->activate". An active plugin must call `host->request_restart()` instead. [fetched] https://raw.githubusercontent.com/free-audio/clap/main/include/clap/ext/latency.h
- **State.** `save(ostream)`, `load(istream)` and the host's `mark_dirty()` all run on the main thread. A parameter value change marks the state dirty implicitly. The `state-context` extension tells the plugin whether a save or load is for a preset, a duplicate or the project. [fetched] https://raw.githubusercontent.com/free-audio/clap/main/include/clap/ext/state.h
- **Audio ports and sidechain.**
  - `count` and `get` are main-thread only.
  - Only one port may be `CLAP_AUDIO_PORT_IS_MAIN`, and it sits at index 0.
  - `port_type` is "mono", "stereo", empty, or a type defined by an extension (surround and ambisonic have their own extensions).
  - `in_place_pair` lets input and output share a buffer.
  - 64-bit flags: `SUPPORTS_64BITS`, `PREFERS_64BITS` and `REQUIRES_COMMON_SAMPLE_SIZE`. 32-bit support is mandatory.
  - The port configuration may only change while the plugin is deactivated.
  - The header never says "sidechain". A sidechain is a non-main input port.
  - [fetched] https://raw.githubusercontent.com/free-audio/clap/main/include/clap/ext/audio-ports.h
- **Note ports.** This is listed among the core extensions alongside latency, state, params, audio-ports, render, tail, gui, voice-info and track-info. [fetched] https://github.com/free-audio/clap
- **GUI.**
  - Supported APIs: win32 (physical pixels, `SetParent`), cocoa and uikit (logical pixels, no `set_scale`), x11 (physical pixels, XEmbed), and wayland (floating windows only).
  - Embedded windows are universally supported by hosts; floating windows are optional.
  - Plugin GUI calls are main-thread only. Host `request_resize`, `request_show` and `closed` are thread-safe.
  - Resize is negotiated through `can_resize`, `get_resize_hints`, `adjust_size` and `set_size`.
  - [fetched] https://raw.githubusercontent.com/free-audio/clap/main/include/clap/ext/gui.h
- **clap-wrapper.** This adapter wraps a CLAP as VST3, AUv2 or standalone. It can also build WCLAP and native binaries from the same project. [fetched] https://github.com/free-audio/clap and https://github.com/WebCLAP
- **WCLAP (WebCLAP).**
  - A WCLAP is a CLAP compiled to a Wasm module, packaged as a `.wclap` bundle containing `module.wasm`. The goals are portability across native platforms and browsers, and sandboxed, low-risk plugins.
  - Required exports: `clap_entry`, one growable function table, and `malloc` or `cabi_realloc`. Memory must be imported (shared) or exported.
  - wasm32 is recommended, but hosts should check for wasm64.
  - Threads use wasi-threads, so multiple instances share memory. Browser hosts need cross-origin isolation, and wclap-js assumes growable shared memory with a 2 GB maximum.
  - Repositories: wclap-bridge (a native CLAP/VST3 plugin built on Wasmtime, including webview UIs), wclap-host-cpp, wclap-host-js, browser-test-host (wraps a WCLAP as an AudioWorkletNode), as-clap (AssemblyScript), and wasi.wasm.
  - The org page states that WCLAPs use about **1.5x the CPU of native** under wclap-bridge.
  - [fetched] https://github.com/WebCLAP
  - Signalsmith's examples build WCLAPs with wasi-sdk (recommended) or Emscripten. [search] https://github.com/geraintluff/signalsmith-clap-cpp
  - openDAW hosts WebCLAP plugins. [search] https://buttondown.com/opendaw/archive/opendaw-now-runs-webclap-plugins/

## 2. VST3, AU/AUv3, LV2, host sandboxing

- **VST3.** VST 3 SDK 3.8.0 (2025-10-20) made VST3 open source under the MIT license. The previous dual license (proprietary plus GPLv3) is no longer offered. Using the "VST" name or logo follows Steinberg's trademark rules. [search] https://steinbergmedia.github.io/vst3_dev_portal/pages/Versions/Version+3.8.0.html and https://steinbergmedia.github.io/vst3_dev_portal/pages/VST+3+Licensing/VST3+License
  - The ASIO SDK was released under GPLv3 at the same time. [search] https://librearts.org/2025/11/steinberg-relicenses-vst3-and-asio/ and https://cdm.link/open-steinberg-vst3-and-asio/
- **AU and AUv3.**
  - AUv2 loads in-process. AUv3 loads out-of-process by default. On macOS only, a host can pass `loadInProcess` if the unit supports it and check `isLoadedInProcess`. [search] https://developer.apple.com/documentation/audiotoolbox/migrating-your-audio-unit-host-to-the-auv3-api
  - An AUv3 is an app extension (`.appex`) whose main binary is an XPC executable, so hosts load it over XPC. [search] https://developer.apple.com/forums/thread/766503
  - Developers report that Logic Pro always loads AUv3 out-of-process, through AUHostingService, and Apple called this "functioning as intended". [search] same thread as above.
  - Users report that AUHostingService can use more RAM than Logic itself. [search] https://discussions-jp-prz.apple.com/thread/256033743
  - An out-of-process AUv3 that is itself a host cannot see other v3 units. [search] https://developer.apple.com/forums/thread/728404
  - Logic Pro 12 crashes when an AU takes too long to load. [search] https://developer.apple.com/forums/thread/835717
- **LV2.**
  - Host features `urid:map` and `worker:schedule` are passed at instantiation. Plugins map URIs to integer URIDs once at setup and use them for atom and MIDI types. Atom ports carry event sequences. [search] https://git.ardour.org/ardour/ardour/src/commit/2d37392f1a23d415d022edec4c41c244b584031d/libs/plugins/a-fluidsynth.lv2/a-fluidsynth.cc and https://drobilla.net/files/lv2_docs/ns
  - Worker extension: the plugin may call the schedule feature only from the audio thread (`run`). The host calls `work()` on another thread and delivers the response back to the audio thread. In offline export the scheduled work may run immediately on the same thread. [fetched] https://lv2plug.in/ns/ext/worker
- **Bitwig plug-in hosting modes.** Bitwig's modes trade memory use against crash isolation:
  - *Within Bitwig*: plug-ins run in the engine process. This uses the least memory, but one crash takes down audio.
  - *Together* (the default): one sandbox shared by all plug-ins, separate from the engine.
  - *By manufacturer*: one sandbox per vendor, useful when a vendor's plug-ins talk to each other.
  - *By plug-in*: one sandbox per plug-in type.
  - *Individually*: one sandbox per instance. This is the safest mode and uses the most memory.
  - A mode change applies only to newly added plug-ins until the project or engine is reloaded. Crash protection arrived in Bitwig 2.5.
  - [search] https://bitwig.com/userguide/latest/vst_plug-in_handling_and_options and https://musicradar.com/news/with-bitwig-studio-25-plugins-will-no-longer-crash-your-entire-projects
- **REAPER bridging.**
  - The per-FX "Run as" menu offers Automatic (default), Separate process (all bridged plug-ins share one bridge, so one crash kills them all but the session survives), and Dedicated process (one process per instance, with its own memory space). [search] https://reaper.blog/2012/02/run-plugin-as-dedicated-process/ and https://support.mixwave.com/doc/reaper-troubleshooting
  - There are also preferences for "buggy plugin compatibility mode" and terminating on heap corruption. [search] https://homerecording.com/bbs/threads/diggy_dude.317114/
  - REAPER's "firewall" behaviour was not found in an official doc. The term is commonly used for its option to catch plug-in exceptions (crash protection) and was not verified.

## 3. Web Audio, AudioWorklet, WAM2, DSP languages

- **AudioWorklet.** `process()` is called once per render quantum with 128-frame buffers. [search] https://forum.hise.audio/assets/uploads/files/1737457819210-3e5870ee-509c-4f16-880e-65c17ebfbee2-these_shihong_ren-fft.pdf
  - The spec fixes the render quantum at 128 frames. Web Audio 1.1 adds a `renderSizeHint` option. The processor runs on the real-time rendering thread with no blocking I/O, and missing the deadline causes glitches. [unverified] https://www.w3.org/TR/webaudio/
- **WAM 2.0 (Web Audio Modules).**
  - It is an open browser plug-in standard for instruments, effects and MIDI processors. [search] https://ubmr.hds.hebis.de/EdsRecord/edshal%252Cedshal.hal.03951799v1
  - The API supports scheduled parameter automation plus transport, MIDI and OSC events, and they can be scheduled from either the main thread or the audio thread. Methods include `scheduleEvents`, `emitEvents` and `clearEvents`. [search] https://www.github.com/webaudiomodules/api and https://cdn.jsdelivr.net/npm/@webaudiomodules/api@2.0.0-alpha.6/dist/index.d.ts
  - WAM parameters are separate from `AudioParam`, so the audio thread can manipulate them, and the parameter manager offers sample-accurate or buffer-accurate automation. [search] https://www.github.com/webaudiomodules/sdk-parammgr
  - "Ten Years of Web Audio Modules", JAES vol. 73 (2025), pp. 804–826. [search] https://aes2.org/publications/elibrary-page/?id=23110
- **Faust to Wasm.**
  - The Letz/Orlarey/Fober paper (WWW 2018) benchmarks native against Wasm builds of the same DSPs, DOI 10.1145/3184558.3185970. [search] https://hal.archives-ouvertes.fr/hal-02158925
  - Grame's benchmark pages measure Wasm DSP CPU use as MB/s and as a percentage of a 1024-frame buffer at 44.1 kHz. They date from 2017. [search] https://faust.grame.fr/community/news/
  - faustwasm is the current npm/TypeScript compiler and runtime. [unverified] https://github.com/grame-cncm/faustwasm
- **Cmajor.** Patches can be exported to JavaScript, WebAssembly or HTML, and to C++. Cmajor is LLVM-based. SOUL was discontinued in favour of Cmajor (same authors); its GitHub repository was last pushed in July 2022. [search] https://cmajor.dev/ and https://forum.juce.com/t/soul-inside-of-a-vst3/36423?page=2 and https://www.github.com/soul-lang/SOUL/blob/master/docs/SOUL_FAQ.md
- **Elementary Audio.** A JS/functional audio graph with a C++ engine that compiles to Wasm for web renderers, plus native and plug-in targets. [unverified] https://www.elementary.audio/
- **Csound.** The `@csound/browser` package is Csound compiled to Wasm and running in an AudioWorklet (or ScriptProcessor fallback). [unverified] https://www.npmjs.com/package/@csound/browser
- **Hand-written Wasm DSP.** Roberts (WAC 2022) found dynamic per-sample hand-written Wasm too inefficient for his target. [search] https://zenodo.org/record/6767578/files/WAC_2022_paper_56.pdf

## 4. Wasm runtime performance and real-time safety

- **Wasm vs native.** Jangda et al., "Not So Fast" (USENIX ATC 2019), ran SPEC CPU in browsers. Wasm was on average **45% slower in Firefox and 55% slower in Chrome**, with peaks of 2.08x and 2.5x. Earlier "about 10% slower" claims came from small kernels. Some causes are missing optimizations; others are inherent to the platform. [search] https://www.usenix.org/conference/atc19/presentation/jangda and https://arxiv.org/abs/1901.09056v3
- **WCLAP measurement.** Wasm CLAP through Wasmtime (wclap-bridge) runs at about 1.5x native CPU. [fetched] https://github.com/WebCLAP
- **wasmi.**
  - v0.32 introduced a register-based IR, giving up to 5x faster execution. Its lazy translation makes startup fast.
  - The maintainers say compute-heavy workloads, where execution time exceeds translation time, belong on JITs (Wasmtime, WAMR, Wasmer).
  - Winch and Stitch were described as experimental at the time.
  - [search] https://wasmi-labs.github.io/blog/posts/wasmi-v0.32 and https://github.com/wasmi-labs/wasmi-benchmarks
- **Interpreter vs JIT.** A wasmi issue commenter estimated "Wasm3 is usually 5-20x slower than Wasmtime" on optimized compute code. [search] https://github.com/wasmi-labs/wasmi/issues/759
  - A 2026 single-author comparison of interpreters on fib(30) measured wasmi 2.0-beta about 38.4 ms, Wasm3 about 39.7 ms and WAMR iwasm about 49.6 ms. Wasmtime was not included. [search] https://ray-d-song.github.io/wasmz/bench.html
  - An older multi-runtime benchmark: [search] https://00f.net/2023/01/04/webassembly-benchmark-2023/
- **Winch.**
  - Winch is a single-pass baseline compiler with no IR and simple register allocation. It prioritizes compile speed over code quality, and its code runs slower than Cranelift's. [search] https://docs.wasmtime.dev/examples-fast-compilation.html
  - AArch64 support arrived in Wasmtime 35. [search] https://bytecodealliance.org/articles/winch-aarch64-support
  - The TPDE paper implies Winch compiles roughly 7x faster than a de-optimized Cranelift. This is derived arithmetic from the paper's ratios, not a stated figure. [search] https://arxiv.org/pdf/2505.22610
  - Epoch interruption is not compatible with Winch. [fetched] https://docs.wasmtime.dev/api/wasmtime/struct.Config.html
- **Wasmtime fuel and epochs.** Fuel instrumentation "is somewhat expensive". Epoch interruption is faster than fuel, "in some measurements, up to 2-3x". The docs give no absolute percentage. [fetched] https://docs.wasmtime.dev/api/wasmtime/struct.Config.html
- **Wasmtime memory, relevant to real-time use.**
  - On 64-bit hosts the defaults are `memory_reservation` 4 GiB (which removes most bounds checks for 32-bit memories), `memory_guard_size` 32 MiB and `memory_reservation_for_growth` 2 GiB.
  - Under the pooling allocator, memories never move, so growth past the slot size fails with an error. Stacks are pooled and reused.
  - These properties support pre-reserving memory at activation and treating `memory.grow` on the audio thread as a fault.
  - [fetched] https://docs.wasmtime.dev/api/wasmtime/struct.Config.html
- **SIMD128.**
  - V8 and TensorFlow.js report SIMD giving a 1.7–4.5x gain over scalar Wasm. These are not audio numbers. [search] https://v8.dev/features/simd
  - simd128 browser support: Chrome 91+, Firefox 89+, Safari 16.4+. Ship a scalar fallback. [search] https://v8.dev/features/simd
  - No published AudioWorklet SIMD-vs-scalar DSP measurement was found. A Demucs Wasm write-up compares builds with `-msimd128`. [search] https://sevagh-github-io.pages.dev/blog/speed
- **General real-time risks** (synthesis of the above):
  - JIT compile time at instantiation: compile at load or activate time, or use AOT or cached modules.
  - `memory.grow` and page faults on first touch.
  - Allocator calls inside guest `malloc`.
  - Fuel or epoch checks adding overhead.
  - Thread pools and synchronization (as the CLAP thread-pool warning notes).

## 5. Product catalogue (one line each)

- **FabFilter Pro-Q 4** (released 2024-12-12): Spectral Dynamics, dynamic EQ, EQ Sketch, an Instance List that navigates all instances in a project, character modes, linear phase, sidechain, EQ Match and Spectrum Grab. [search] https://www.fabfilter.com/news/1733994000/fabfilter-releases-pro-q-4-equalizer-plug-in and https://www.kvraudio.com/news/fabfilter-releases-pro-q-4-equalizer-plugin-62490
- **FabFilter Pro-L 2**: true-peak limiter with lookahead (which adds latency), oversampling up to 32x and loudness metering. [unverified] https://www.fabfilter.com/products/pro-l-2-limiter-plug-in
- **FabFilter Pro-C 2 / Pro-C 3**: compressor with lookahead, sidechain EQ, external sidechain, oversampling and style modes. [unverified] https://www.fabfilter.com/products/pro-c-2-compressor-plug-in
- **iZotope RX 11**: neural Dialogue Isolate with built-in Dialogue De-Reverb (four-band processing in Advanced), Music Rebalance on a new neural net for stem separation, Loudness Optimize, Repair Assistant, Streaming Preview, and Spectral Editor plus Music Rebalance as ARA AU in Logic 10.7+. Release notes reach 11.4.0 (2026-02-13). [search] https://www.mixonline.com/technology/izotope-announces-rx-11 and https://www.izotope.com/pages/release-notes/rx-standard
- **iZotope Ozone 11/12**: mastering suite with an ML Master Assistant, Stem Focus/Clarity, maximizer and spectral modules. [unverified] https://www.izotope.com/en/products/ozone.html
- **iZotope Neutron**: mixing suite with Mix Assistant, Unmask (inter-plugin sidechain-style communication) and Visual Mixer. [unverified] https://www.izotope.com/en/products/neutron.html
- **Waves (incl. Clarity Vx)**: large plugin catalogue. Clarity Vx and Clarity Vx Pro are neural, real-time voice and noise separation. [unverified] https://www.waves.com/plugins/clarity-vx
- **Soundtoys**: character effects (Decapitator, EchoBoy, and others) with analogue modelling and tempo sync. [unverified] https://www.soundtoys.com/
- **Valhalla (VintageVerb, Supermassive)**: algorithmic reverbs and delays. Supermassive is free. Both have low CPU and simple GUIs. [unverified] https://valhalladsp.com/shop/reverb/valhalla-vintage-verb/
- **Serum 2 (Xfer)**: wavetable synth with more oscillator types, MPE and modulation matrix, built-in FX and a sample-based multi-engine. [unverified] https://xferrecords.com/products/serum-2
- **Vital**: free and open-source (GPL) wavetable synth with MPE, text-to-wavetable and a GPU-drawn UI. [unverified] https://vital.audio/
- **Kontakt 7/8 (Native Instruments)**: sampler with KSP scripting, disk-streamed multi-GB libraries and an NKS host. Kontakt 8 adds tools such as Leap and a chord/phrase engine. [unverified] https://www.native-instruments.com/en/products/komplete/samplers/kontakt-8/
- **Melodyne (Celemony)**: polyphonic pitch and time editing, and the reference ARA2 plugin. [unverified] https://www.celemony.com/en/melodyne/what-is-melodyne
- **ARA2 spec**: Celemony's ARA SDK and API have been Apache-2.0 since May 2021, which makes them usable in GPLv3 projects, and come with a JUCE example. The latest SDK release is 2.2.0. [search] https://github.com/Celemony/ARA_SDK/releases and https://www.kvraudio.com/news/celemony-releases-ara-sdk-under-open-source-license-51704 and https://github.com/Celemony/ARA_API
- **Auto-Tune (Antares)**: real-time pitch correction. Low-latency modes are essential for live and tracking use. Auto-Tune Pro has ARA2 Graph Mode. [unverified] https://www.antarestech.com/products/auto-tune
- **Spitfire LABS**: free sample instruments in a streaming sampler player. [unverified] https://labs.spitfireaudio.com/
- **LANDR mastering**: cloud or AI mastering service, with a mastering plugin. Processing is offline and server-side. [unverified] https://www.landr.com/online-audio-mastering/
- **Demucs / HTDemucs (Meta)**: offline stem separation.
  - HT-Demucs reaches 9.20 dB SDR on MUSDB (sparse attention plus per-source fine-tuning, with extra data). [search] https://arxiv.org/abs/2211.08553v1
  - One secondary source reports median SDR of 8.49 dB with no extra data. [search] https://www.emergentmind.com/topics/ht-demucs
  - A community ONNX export is about 316 MB for htdemucs and 1.26 GB for the htdemucs_ft 4-model bag. [search] https://huggingface.co/adowu/htdemucs-onnx
  - The paper's RTF table was unreadable in the search output. A drums-specialist ONNX model reports RTF about 0.20 on an M4 Pro CPU. [search] https://huggingface.co/StemSplitio/htdemucs-ft-drums-onnx
- **Adobe Podcast Enhance Speech**: cloud neural speech enhancement. Processing is offline and upload-based. [unverified] https://podcast.adobe.com/enhance
- **Audacity**: built-in Nyquist plug-ins, LV2 (successor to LADSPA), and VST3 since 3.2. Release 3.2 added real-time effects for VST3, LV2, AU and LADSPA. [search] https://plugins.audacityteam.org/ and https://itsfoss.com/news/audacity-3-2-release/
- **REAPER JSFX**: effects written in EEL2 and compiled on the fly, editable live. [search] https://www.reaper.fm/sdk/js/js.php
  - EEL2 generates native code, so it is a JIT. [unverified] https://www.reaper.fm/sdk/js/js.php
  - ReaScript exposes the REAPER API to Lua, EEL2 and Python. [unverified] https://www.reaper.fm/sdk/reascript/reascript.php
- **Bitwig Grid**: modular sound-design environment inside Bitwig (Poly Grid, FX Grid, Note Grid). Modulators are applied as non-destructive offsets. [unverified] https://www.bitwig.com/the-grid/
- **Ableton Max for Live**: Max/MSP embedded in Live (audio, MIDI and instrument devices) with a JS/Live API for control. [unverified] https://www.ableton.com/en/live/max-for-live/

## Gaps and items to verify

- **REAPER firewall:** no official REAPER documentation was found.
- **SIMD:** no AudioWorklet SIMD-vs-scalar audio benchmark was found.
- **HTDemucs RTF:** the base model's real-time factor is unconfirmed; check the paper's table.
- **Winch speed:** no official Winch-vs-Cranelift compile-speed figure exists.
- **Fuel overhead:** the Wasmtime docs give only a relative figure (epochs up to 2-3x faster than fuel), not an absolute overhead.
- **[unverified] lines:** the product lines and W3C render-quantum details marked [unverified] should be checked against their vendor or spec pages.

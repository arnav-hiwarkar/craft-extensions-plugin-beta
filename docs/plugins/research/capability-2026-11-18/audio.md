# Audio plugin capability gap analysis (SoundCraft, EffectCraft, FilmCraft, craft-extensions)

Date: 2026-11-18. This is a read-only study. Nothing was built or run. Code facts come from source
and docs in the working trees. Web facts are cited in Sources. Facts tagged *(unverified)* come from
vendor pages that were not fetched in this pass.

## Summary

- **SoundCraft already hosts native CLAP, VST3 and AUv2, all in-process.** The hosts are
  `crates/clap-host`, `crates/vst3-host` and `crates/au-host`. They sit behind one safe trait,
  `soundcraft_dsp::Plugin` (`crates/dsp/src/lib.rs:195`), and report latency for delay
  compensation (PDC), save state, send notes and MIDI, and open some editors.
  - Missing: sidechain, ARA2, live MIDI input, sample-accurate parameter events, crash isolation,
    and a hardened-runtime entitlement on macOS. Without the entitlement, the signed build refuses
    VST3s.
  - FilmCraft has 53 built-in audio effects on its own `AudioEffect` trait and **no plugin
    hosting** (gap G11, 200–400 h).
  - EffectCraft has 13 AE-style audio effects that are stateless per block, plus an audio feed for
    Audio Spectrum/Waveform. It has no audio plugin contract.
  - `craft-libs/crates/audio` (craft-audio) covers file I/O and codecs only, with no DSP.
- **craft-extensions has no audio contract at all.** The only effect contract is `raster.filter@1`.
  Its execution path cannot run on a real-time thread:
  - It allocates on every call (JSON context, a `Vec` copy of every sample).
  - It uses fuel slices plus resumable calls.
  - wasmi is built without the `simd` feature (`crates/ext-host/Cargo.toml:18`), so SIMD128
    modules are rejected.
  - The UI tree sends slider edits only when a drag ends (`docs/plugins/ui.md:163`), which is
    unusable for knobs that write automation.
- **Real-time Wasm DSP is feasible, but not on wasmi.**
  - JIT-compiled Wasm (Wasmtime/Cranelift) runs CLAP plugins at about **1.5x native CPU** (the
    WebCLAP project's figure). Browser SPEC numbers put Wasm at **1.45–1.55x** native on average.
  - An optimizing JIT is typically 5–20x faster than an interpreter on compute code.
  - wasmi's own maintainers point compute-heavy work to JITs.
  - Conclusion: run audio plugins on **Wasmtime with Cranelift, compiled ahead of time**, at
    activation, with:
    - a pooling allocator and memory reserved up front (`memory.grow` is a fault on the audio
      thread);
    - epoch interruption, or a host watchdog, instead of per-instruction fuel;
    - SIMD128 enabled;
    - a per-block time budget with bypass on overrun.
  - Keep wasmi for UI, sessions and offline jobs, and as the fallback where JIT is unavailable.
- **Recommendation: adopt CLAP semantics as the guest ABI.** The cheapest credible route to "VST3-class
  Wasm plugins" is to make the contract **CLAP semantics carried over Wasm**, and to make
  WebCLAP-compatible `.wclap` modules a first-class import format.
  - The `audio.effect@1`, `audio.instrument@1` and `audio.note-fx@1` contracts below keep CLAP's
    thread model, sorted sample-offset events, non-destructive modulation, note expressions,
    latency, state and ports.
  - Native CLAP/VST3/AU stays necessary for the commercial catalogue (FabFilter, iZotope, Kontakt
    and others), and should move out of process with Bitwig-style sandbox modes.

## App inventory

### SoundCraft (`soundcraft`)

**Layering** (`docs/architecture.md`):

- L0: `time`, `audio-io`, `midi`, `video`.
- L1: `dsp` and the three hosts. These are the only crates allowed `unsafe`, and only in their
  `ffi` modules.
- L2–L6: `model`, `mix`, `engine`/`playback`, `automation` (MCP server) and `ui-egui`.
- Apps: desktop, CLI and web (wasm).

**Engine.** `soundcraft_mix::MixEngine::render(session, pos, frames, out)` renders one block in
this order:

1. clips (gain, fades, clip effects)
2. trim
3. inserts
4. pre-fader sends
5. fader, mute and VCA
6. post-fader sends
7. pan
8. busses
9. aux inputs
10. master faders

Independent strips render in parallel (`crates/mix/src/threads.rs`, `run_strips`). Workers are
marked as the audio thread through `mark_audio_thread` and `on_audio_thread`, which keeps logging
lock-free. PDC aligns every path, including master inserts. Offline bounces (`RangeRender`) use the
same engine and stream to disk.

**Real-time I/O** (`crates/playback`, `docs/hardware-parity.md`):

- Uses cpal: Core Audio, WASAPI, opt-in ASIO, ALSA/Pulse (no JACK), and Web Audio on wasm.
- The device buffer is selectable (16–8192; 128 verified on ASIO). **The mix always renders
  512-sample blocks** and resamples to the device rate.
- A panic in a block makes the output silent and stops the transport
  (`crates/playback/src/callback.rs:53`). There is no per-plugin deadline or overrun bypass.

**Sample format.** Planar `f32`, `io[channel][frame]`. `Plugin::process` never allocates.
Built-ins never panic on any parameter value (`crates/dsp/src/lib.rs` header).

**Plugin trait** (`crates/dsp/src/lib.rs:195-260`):

- `prepare(sr, max_block, channels)`, `reset`, and `set_param(id: &str, f32)` / `param`.
- `process(&mut [Vec<f32>], frames)`: in place, one bus.
- `latency`, `tail_samples` and `gain_reduction_db` (metering).
- `set_transport` (tempo, meter, position, bar).
- `note_on`, `note_off` and `midi` at a sample offset; `all_notes_off`.
- `set_sample(Arc<dyn SampleSource>)` for the sampler.
- Opaque `save_state` and `load_state`, plus `PluginStateReader` for main-thread reads.
- `PluginEditor` (`open`, `close`, `idle` returns parameter edits, `state_changed`, `value_text`).

What it lacks:

- **Parameters are not sample-accurate.** `crates/mix/src/strip.rs:529` (`apply_values`) calls
  `set_param` at the start of a block when a value changed. This holds even though the VST3 host
  has `IParameterChanges` queues with `addPoint(offset)` (`crates/vst3-host/src/ffi/process.rs`).
- There is no modulation-vs-automation distinction.
- Ports are single-bus only, so there is no sidechain.
- There are no note expressions or MPE.
- There is no main-thread callback or request-restart path for parameter rescans.

**Built-in DSP** (`crates/dsp/src/plugins`): 27 processors and 3 instruments.

- Covered: EQ 1/7-band, channel strip, compressor, gate, de-esser, maximizer, room and plate
  reverb, modulated delay, chorus, flanger, phaser, ring mod, auto-pan, lo-fi, saturator,
  subharmonic, pitch shifter, time shift, utilities and dither. Instruments: subtractive synth,
  drum synth, sampler.
- Not covered: linear-phase or dynamic EQ, multiband, convolution, pitch correction, wavetable
  synths (`docs/plugin-parity.md`).
- Analysis: peak, RMS and BS.1770 metering (`meter.rs`), FFT spectrum (`spectrum.rs`), YIN pitch
  detection for audio-to-MIDI (`pitch_detect.rs`).
- Offline AudioSuite processing (`offline.rs`) allocates, as designed. **Hosted plugins cannot be
  used as AudioSuite yet.**

**Third-party hosting** (`docs/plugin-parity.md`):

| Format | Crate | State |
|---|---|---|
| CLAP | `clap-host` (clap-sys 0.5) | Scan, params, notes/MIDI, transport, latency, state; floating editors, and embedded editors on Windows |
| VST3 | `vst3-host` (vst3 0.3, libloading) | Same, plus `IMessage`; editors on macOS and Windows |
| AUv2 | `au-host` | `AudioUnitRender`, `MusicDeviceMIDIEvent`; no Cocoa editor |
| VST2, LV2, LADSPA, AAX | none | #30, #109 |

Plugins are created and destroyed off the audio thread; only `process` runs on it. The CLAP crate
header says it directly: "a plugin that itself crashes … takes the process down with it; we cannot
sandbox native code." Open gaps:

- G-05: signed macOS builds refuse third-party VST3.
- G-13: sidechain, ARA and out-of-process hosting.
- G-02: no MIDI device input or output at all.
- G-20: no MIDI learn.

**Automation.** The model has lanes and Off/Read/Touch/Latch/TouchLatch modes
(`crates/model/src/automation.rs`). Pan is read on a fixed `PAN_STEP` grid. Plugin parameters are
read once per block.

**MIDI.** `crates/midi` provides note and controller sequences, SMF I/O and editing.
`crates/mix/src/midi.rs` schedules notes at sample offsets. There are no devices, no MIDI 2.0 and no
MPE.

**Agent control.** An MCP server (`crates/automation`) and a JSON-lines control channel. This is a
scripting surface comparable to ReaScript, but with no in-process user scripts.

### FilmCraft (`filmcraft`)

- `crates/audio-dsp` is about 13.4k lines and has no dependencies; it is wasm32-clean and has no
  `unsafe`. It provides:
  - the `AudioEffect` trait (`lib.rs:117`): `set_param`, which is **smoothed per sample, so output
    is independent of block cuts**; `process(&mut [&mut [f32]])`; `latency`; `response_db` (for
    EQ curves); and `transfer_db` (for dynamics curves);
  - 53 Premiere-parity effects (EQ, dynamics, multiband, restoration and declick, STFT, pitch,
    reverbs, Essential Sound);
  - BS.1770/R128 loudness, GCC-PHAT sync, a resampler, up- and downmix, and ducking.
- Device output goes through cpal (`apps/filmcraft/src/audio.rs`), with a selectable buffer size and
  xrun recovery. Voice-over input is in `audio_in.rs`.
- AI: Whisper STT (`crates/speech`, candle) and Kokoro TTS (`crates/tts`). "Enhance Speech is DSP,
  not a model" (`docs/target-app-parity.md`).
- Plugin hosting is about 5% complete: no VST3 or AU, and G11 needs an owner decision on FFI.
  FilmCraft's `AudioEffect` contract (smoothed parameters, analytic response curves) is the best
  in-suite model for what an `audio.effect@1` guest should expose for UI curves.

### EffectCraft (`effectcraft`)

- **Audio effects** (`crates/effects/src/audio_fx.rs`): 13 AE effects, from Backwards to Tone.
  - They are ordinary layer effects, so their parameters animate.
  - The mixdown (`effectcraft_render::audio::mix_comp`) renders short blocks. Each effect is
    **stateless across calls**: the mixer feeds a preroll and runs the chain from a cleared state.
    A Wasm audio plugin in EffectCraft would have to accept that model or get a stateful streaming
    path.
- **Audio-reactive data:** `EffectHost::audio(id, start, frames, rate)` (`crates/effects/src/lib.rs:533`)
  returns interleaved stereo for Audio Spectrum/Waveform (`generate3/audio.rs`) and for keyframe
  conversion.
- **Preview:** cpal, about a 10 ms callback, with a feeder thread (or a UI-frame pump on wasm) and
  an audio-master clock (`crates/ui-egui/src/audio.rs`).
- **Gap:** there is no audio contract in `crates/plugins/src/contract.rs` and no audio read op for
  `raster.filter@1` plugins. A sandboxed "audio-reactive" plugin, common in AE (Trapcode Sound
  Keys style), therefore cannot exist.

### craft-libs

`crates/audio` (craft-audio) handles WAV, AIFF and FLAC writing, decoding through symphonia, and
peaks. It has no DSP and no engine, so each app duplicates biquads, meters and loudness code.
SoundCraft keeps its own `audio-io`.

### craft-extensions (platform)

- **Runtime.** wasmi 2 with features `std`, `validate`, `deterministic` and `portable-dispatch`. It
  has **no `simd` feature**, although wasmi 2.0.1 offers one.
  - Fuel and memory limits: slices of 2·10⁷, 4000 fuel per pixel, 512 MiB transform memory
    (`crates/ext-host/src/wasm.rs:10-48`).
- **Filter path** (`wasm.rs:591-680`): every call does the following, which is not real-time safe:
  - serializes the context to JSON;
  - copies every pixel into a fresh `Vec`;
  - runs `craft_reset`;
  - runs resumable fuel slices;
  - copies the result back and validates finiteness.
- **Kernels and jobs** (`docs/plugins/compute-and-jobs.md`): image-only kernels, and jobs of up to
  512 MiB and about 2·10¹² fuel. Jobs are the right home for offline audio work (stem separation,
  RX-style repair) once audio kernels and a sample-buffer contract exist.
- **Native helpers** (`docs/plugins/native-protocol.md`): JSON-lines over stdio with "not an OS
  sandbox" semantics. This is unsuited to per-block audio exchange; shared-memory rings are needed.
- **UI** (`docs/plugins/ui.md`): has `slider` and `canvas` (draw list, up to 20,000 ops). It lacks
  continuous parameter gestures (begin, change, end), high-rate meter or spectrum streams, and an
  embedded native editor window.

## Competitive plugin catalogue

What each class requires of a plugin API. Tags: RT = real-time process, LA = lookahead/latency,
SC = sidechain, OS = oversampling, AN = analyzer UI fed from the audio thread, MIDI, ARA,
ML = neural inference, OFF = offline whole-file, XI = cross-instance communication.

| Product / class | Needs | Wasm-sandbox verdict |
|---|---|---|
| FabFilter Pro-Q 4 (dynamic and spectral EQ, linear phase, EQ Sketch, Instance List, EQ Match, Spectrum Grab) | RT, LA (linear phase), SC, AN at 60 fps, XI (instance list), rich GPU UI | Feasible on JIT plus SIMD. Needs an analyzer ring buffer, a sidechain port, cross-instance messaging and a custom canvas UI |
| FabFilter Pro-L 2 / Pro-C 2–3 | RT, LA, OS up to 32x, SC, loudness and true-peak metering | Feasible. 32x oversampling is the CPU worst case (about 1.5x native, so roughly 30–50% fewer instances) |
| iZotope RX 11 (Dialogue Isolate, Music Rebalance, Spectral Editor, ARA) | OFF, ML, spectral editing UI, ARA2 | Offline jobs plus `ml.*` host kernels (ONNX/candle). Spectral-editor UI needs a large-image canvas. ARA-like clip access is needed |
| iZotope Ozone / Neutron (assistants, Unmask) | RT plus OFF analysis, ML, XI | Same, plus inter-instance analysis sharing |
| Waves (catalogue), Clarity Vx (neural real-time denoise) | RT, ML in real time at low latency | Real-time ML in Wasm is impractical. Needs a host real-time `ml.stream` kernel with fixed-shape small models |
| Soundtoys, Valhalla VintageVerb/Supermassive | RT, tempo sync, modulation | Ideal Wasm targets: low CPU, deterministic DSP |
| Serum 2, Vital (wavetable, MPE, mod matrix) | Instrument, note expressions/MPE, polyphonic modulation, wavetable import, GPU UI | Feasible. Needs CLAP-style note expressions, voice-info and file import |
| Kontakt 7/8, Spitfire LABS (multi-GB disk streaming, KSP scripting) | Disk streaming, very large memory, scripting | The guest cannot stream from disk on the audio thread. Needs a host `sample.stream` service (preload heads, background reads into a lock-free ring). Script-level KSP equivalents fit `audio.note-fx@1` |
| Melodyne, Auto-Tune Pro Graph mode (ARA2) | ARA2: random access to clip audio, edit model shared with the host | Needs an ARA-like `audio.clip-editor@1` contract (clip read access, analysis jobs, render on demand) |
| Auto-Tune (live, low latency) | RT, LA of 0–few ms | Feasible in Wasm |
| LANDR, Adobe Podcast Enhance (cloud) | OFF, network | Jobs plus `network:<host>` (the op does not exist yet; see gap audit) |
| Demucs / HTDemucs stem separation (MUSDB SDR about 9.2 dB; ONNX about 316 MB, 1.26 GB for the ft bag) | OFF, ML, large weights | Host ML kernel only. Wasm inference on wasmi is hopeless; even a JIT without threads is far slower |
| Audacity Nyquist, REAPER JSFX (EEL2 JIT, live edit), ReaScript (Lua/Python/EEL API) | User DSP scripts, live recompiling, host scripting API | Wasm compile at load fits. A "script → Wasm" path (Faust, Cmajor) gives JSFX parity. The ReaScript equivalent is a session contract with SoundCraft engine ops (the MCP command set already exists) |
| Bitwig Grid, Max for Live | Modular graph, modulators as non-destructive offsets, host API for devices | A guest plus CLAP-style `PARAM_MOD` events. Host modulators need param-mod plumbing in `mix` |
| Plugin APIs: CLAP 1.2.10, VST3 3.8 (MIT since 2025-10), AUv3 (XPC, out of process by default), LV2 (URID, atom, worker), AudioWorklet (128-frame quantum), WAM2 (scheduled parameter and MIDI events), WCLAP (CLAP in Wasm, wasi-threads, wclap-bridge on Wasmtime) | | WCLAP is the closest prior art to the target. Its ABI should be accepted directly or wrapped |

## Real-time Wasm feasibility

### The deadline

At 48 kHz, a 64-sample block is 1.33 ms, 128 samples is 2.67 ms and 512 samples is 10.7 ms. A DAW
typically spends 30–70% of each period on DSP across all strips. A single insert therefore gets
tens to hundreds of microseconds. That leaves:

- no allocation;
- no locks;
- no page faults;
- no compilation;
- no unbounded loops;
- no syscalls.

### Execution engines

| Engine | Throughput vs native | Real-time suitability |
|---|---|---|
| Native (built-in DSP, hosted CLAP) | 1x | Reference |
| Wasmtime + Cranelift, AOT | About 1.5x for WCLAP plugins (WebCLAP); SPEC in browsers 1.45–1.55x on average, peaks of 2.1–2.5x (Jangda et al.); SIMD128 gives 1.7–4.5x over scalar Wasm (V8 figure, not audio-specific) | **Yes**, with a pooling allocator, pre-touched memory, no `memory.grow`, and epoch or watchdog interruption. Compile at load, cache artifacts |
| Wasmtime + Winch (baseline) | Slower code than Cranelift (no published ratio); compiles much faster | Use for fast first load, then tier to Cranelift. Epoch interruption is not supported on Winch, so it needs fuel or a watchdog |
| wasmi 2 (register interpreter) | Comparable to wasm3. Interpreters are typically 5–20x slower than Wasmtime on optimized compute (wasmi issue #759). Maintainers say compute-heavy workloads belong on JITs | **Only for trivial DSP** (gain, panning, MIDI transforms) or offline jobs. Fuel metering adds further cost. No SIMD in the current build |
| Browser (soundcraft-web, effectcraft-web) | V8/SpiderMonkey JIT in an AudioWorklet, 128-frame quantum | Native target for the web apps. Hosting a `.wasm` audio plugin in an AudioWorkletGlobalScope is exactly what WAM2 and WCLAP's browser host do |

Rough capacity: a channel EQ costs about 1–2% of one core natively at 48 kHz stereo.

- At 1.5x (JIT), it costs 1.5–3%. That is fine.
- At about 15x (interpreter), it costs 15–30% per instance. **Not viable** for a mixer.

Interpreter execution is also more variable from block to block because there is no JIT
specialization. This hurts the jitter percentiles that real-time work depends on.

### Real-time hazards and mitigations

| Hazard | Mitigation |
|---|---|
| JIT compile on first call | Compile at install or activate time. Cache `.cwasm` keyed by module sha256 plus engine version. Never compile on the audio thread |
| `memory.grow`, first-touch page faults | Guest declares `max_memory` in the manifest. Host reserves it from the pooling allocator and pre-faults (touches) it at `activate`. `memory.grow` during `process` traps, which counts as an overrun |
| Guest `malloc` in `process` | Allowed but bounded: it is just a bump in linear memory. The host only sees growth, which is refused |
| Unbounded loops | Epoch interruption: a watchdog thread bumps the epoch at about 2x the block deadline. Epochs are documented as up to 2–3x cheaper than fuel. Fuel stays for wasmi and Winch |
| Overrun, trap or NaN output | The host keeps a dry copy. On trap, deadline miss or non-finite output, it outputs dry (a bypass crossfade over 64 samples) and marks the instance faulted. After N faults in M seconds it is auto-bypassed with a UI badge. **Never silence the master** (today a panic silences the whole stream) |
| Host calls from `process` | Only a fixed, allocation-free whitelist: event iteration, output-event push, ring-buffer write. No JSON `craft.call` on the audio thread |
| Determinism (bounce = playback) | A fixed `max_block`. Events are sample-accurate within blocks. NaN canonicalization is on (wasmi `deterministic`; Wasmtime `cranelift_nan_canonicalization`). The host splits blocks at 512 regardless of device buffer, as now |
| Threads | No threads in v1 (CLAP itself warns that thread pools break hard real time). Later: the host runs instances in parallel across strips, as `run_strips` already does |

### Answer

Yes: sandboxed Wasm can run real-time DSP at roughly 1.3–2x native cost on a Cranelift JIT with
SIMD. That is acceptable for EQ, dynamics, delay, reverb, modulation, synths and most "character"
plugins.

It cannot replace native for:

- large real-time neural models;
- multi-GB disk-streaming samplers, without a host streaming service;
- vendor ecosystems that ship only native binaries.

Those need host kernels or out-of-process native hosting.

## Proposed audio contracts

Design rule: **CLAP semantics, Wasm transport.** A WCLAP module (exports `clap_entry`) should load
through a CLAP-on-Wasm adapter into the same contract. Native `craft_audio_*` exports are the
simpler Rust-SDK path.

### `audio.effect@1`, `audio.instrument@1`, `audio.note-fx@1`

**Manifest** (static, validated at install):

- `ports`:
  - `audio_in`: a list of `{name, channels | layout, main|sidechain}`;
  - `audio_out`;
  - `note_in`;
  - `note_out`.
- `params`: a list of `{id: u32, key, name, min, max, default, unit, taper, steps|choices, flags}`.
  Flags are automatable, modulatable, per-note, stepped, hidden, bypass, read-only (meters), and
  `requires_restart`.
- `max_memory`, `max_block` (≤ 4096), `supports_f64`, `tail` (samples or infinite), `latency_hint`.
- Optional `ui`: a UI-tree panel and/or a canvas editor.

**Lifecycle** (CLAP threads):

| Call | Thread | Allocation allowed |
|---|---|---|
| `init`, `activate(sr, min_block, max_block) → {latency, tail}` | main | yes |
| `deactivate`, `destroy` | main | yes |
| `start_processing`, `stop_processing` | audio | no |
| `reset` | audio | no |
| `process(ctx_ptr) → status` | audio | no |

`process` returns one of `continue`, `continue_if_not_quiet`, `tail`, `sleep` or `error`. These
let the host skip silent instances.

**Process context** lives in a fixed struct in guest memory, written by the host. There is no JSON.

- `frames` and `steady_time`.
- Transport: tempo, signature, song position in beats and seconds, loop, playing, recording.
- Pointers to planar `f32` buffers for every port (in-place pairs allowed).
- Input events: sorted by sample offset in a fixed ring.
- Output events.

**Events**, as in CLAP:

- `param_value{id, note_id, port, channel, key, value}`: automation, sample-accurate.
- `param_mod{…, amount}`: a non-destructive offset. Heard value = value + mod.
- `param_gesture_begin` and `param_gesture_end`: for touch/latch automation recording from the
  plugin UI.
- `note_on`, `note_off`, `note_choke` and `note_end`: with `note_id` for polyphonic modulation.
- `note_expression{volume, pan, tuning, vibrato, expression, brightness, pressure}`: MPE and MIDI 2.0
  per-note controllers.
- `midi1` (3 bytes), `midi_sysex` (host-owned buffer), `midi2` (UMP).
- `transport`: for tempo changes inside a block.

**Smoothing.** The host provides sample-accurate events. The SDK supplies `Smoothed<T>`, with
one-pole and linear ramps per sample, following FilmCraft's `AudioEffect` convention. Guests must
never zipper.

**Latency.** Reported at `activate`. A change goes through `request_restart` (main thread), and the
host re-activates and updates PDC. This matches CLAP's rule.

**State.**

- `state.save` and `state.load` run on the main thread and produce opaque bytes (at most 16 MiB)
  plus a `context` (preset, duplicate or project).
- `mark_dirty` is available.
- The host stores parameters separately, so automation survives a state blob the plugin rejects.

**Presets.** The host stores manifest preset files. The plugin can expose `preset.load` on the main
thread.

**Analyzer and meters.** A `meter.write(slot, values)` and `scope.write(ring, frames)` fast path
copies into host-owned lock-free single-producer/single-consumer (SPSC) rings. It is read by the UI
at frame rate, so Pro-Q-style spectrums and gain-reduction meters need no audio-thread JSON.
`response_db(freq[])` and `transfer_db(levels[])` are main-thread queries for curve drawing, as in
FilmCraft.

**Sidechain.** A non-main audio input port. The host's mixer exposes "key input" routing from any
bus or track.

**Offline.** The same module runs in `audio.offline@1` mode (AudioSuite or render-in-place) through
the job system. Host-driven bounce runs `process` faster than real time, with `is_offline = true`
so the plugin can choose higher quality.

### `audio.clip-editor@1` (ARA-like, later)

A main-thread contract. It covers:

- read access to a clip's source samples through `audio.read{clip, range}`;
- background analysis jobs that return models (notes, pitch curves);
- a UI that edits the plugin's model;
- a `render(range)` called by the mix engine for that clip.

This covers Melodyne, RX Spectral Editor, Auto-Tune Graph mode, and AE/Premiere audio-reactive
effects (EffectCraft can expose the same `audio.read` to `raster.filter@1`).

### Host services the platform must add

1. A real-time engine tier: Wasmtime with Cranelift (and optionally Winch), pooling allocator, AOT
   cache, epoch watchdog, SIMD128 and relaxed-SIMD off, a sample-accurate event FIFO, and a
   no-JSON fast ABI.
2. `audio.*` kernels for compute the guest should not do: FFT/STFT, FIR and partitioned
   convolution (reverb IRs), resampling, oversampling filters, true-peak and loudness measurement.
   They follow the image-kernel pattern, but for real-time use they must be allocation-free and run
   on the calling thread.
3. `ml.*` kernels (onnx or candle): offline separation and denoise jobs, plus `ml.stream` for small
   real-time models, run by the host on a dedicated real-time-safe thread with fixed latency.
4. `sample.stream`: a host disk streamer for sampler instruments.
5. Files: user-chosen IR, wavetable and sample import (`file.pick` → read-only handle).
6. UI: continuous parameter gestures, meter and scope bindings, `canvas` at 60 fps, and
   later an "embedded web view or GPU canvas" editor option.

### Fail-safe policy

- The engine itself never runs the guest without a deadline.
- Faults degrade to bypass (dry), never to silence or a crash.
- A plugin that faults repeatedly is disabled for the session and reported in the jobs or plugin
  manager.
- Bounce uses the same bypass and flags the render as "contains faulted plugin".

### When native hosting is still needed, and how to make it safe

Native hosting is needed for:

- the commercial catalogue (no vendor will ship Wasm soon);
- iLok/PACE-protected plugins;
- GPU or ML-heavy plugins;
- ARA2 partners.

Out-of-process plan:

- Move `clap-host`, `vst3-host` and `au-host` behind a `craft-plugin-bridge` process.
- Communicate over shared-memory audio and event rings, with a futex or semaphore wake per block
  (one round trip, roughly 20–100 µs).
- Offer Bitwig's modes: Within, Together (default), By vendor, By plugin, Individually. Add
  REAPER-style "dedicated process" per instance.
- Editors become child windows of the bridge process; on macOS this needs hardened-runtime
  entitlements on the bridge only.
- Use AUv3's XPC out-of-process mode where available.
- A bridge crash bypasses that slot, offers a reload, and keeps the session alive.

The existing `native-protocol` (JSON over stdio) stays for control. It does not carry audio.

## Gaps by layer

Priority: P0 means it blocks any Wasm audio plugin; P3 is nice to have. Effort: S < 1 week,
M is 1–4 weeks, L > 4 weeks.

### Platform (craft-extensions)

| # | Gap | Pri | Effort |
|---|---|---|---|
| PL-1 | No audio contract kinds (`audio.effect@1`, `audio.instrument@1`, `audio.note-fx@1`); schema, manifest ports and params | P0 | M |
| PL-2 | No real-time execution tier: wasmi interpreter, per-call JSON and copies, fuel slices. Need Wasmtime/Cranelift AOT, pooling, pre-faulted memory, epoch watchdog, a fast struct ABI | P0 | L |
| PL-3 | SIMD128 disabled (wasmi built without `simd`; not in the Config) | P1 | S |
| PL-4 | Sample-accurate event model (param value/mod, gestures, notes, expressions, MIDI 1/2, transport) in guest SDK and host | P0 | M |
| PL-5 | Fault policy: deadline, NaN and trap leading to dry bypass and auto-disable; overrun telemetry | P0 | M |
| PL-6 | Guest SDK (`craft_guest::audio`): `Smoothed`, buffers, event iterators, `process!` macro; Faust/Cmajor → Wasm templates; JSFX-like quick-script path | P1 | M |
| PL-7 | WCLAP import adapter (CLAP-on-Wasm → contract) | P1 | M |
| PL-8 | `audio.*` kernels (FFT, convolution, resample, oversample, loudness) | P1 | M |
| PL-9 | `ml.*` kernels (offline separation and denoise; real-time stream) with model packages and licences | P2 | L |
| PL-10 | UI: continuous slider and knob gestures, meter and scope rings, 60 fps canvas, curve queries | P1 | M |
| PL-11 | `audio.read` / `audio.clip-editor@1` (ARA-like) and audio input for `raster.filter@1` | P2 | L |
| PL-12 | Native audio bridge (shared-memory rings) to replace in-process FFI; sandbox modes | P1 | L |
| PL-13 | `sample.stream`, file-pick read handles for IRs and samples | P2 | M |
| PL-14 | Store categories for audio (effect, instrument, analyzer), CPU-cost badges, per-plugin benchmark at install | P3 | S |

### Adapter (per-app glue to `ext-host`)

| # | Gap | Pri | Effort |
|---|---|---|---|
| AD-1 | SoundCraft: a `WasmPlugin` implementing `soundcraft_dsp::Plugin` over PL-2 (the hosts are the model: off-thread create, `process` only on audio) | P0 | M |
| AD-2 | SoundCraft: the `Plugin` trait lacks sample-accurate parameter events, modulation, multi-bus ports, note expressions, restart requests and meter rings. Extend it, or add `process_events` | P0 | M |
| AD-3 | FilmCraft: adapter from the contract to `AudioEffect` (clip and track effects); FilmCraft's param smoothing satisfies the contract | P1 | M |
| AD-4 | EffectCraft: adapter to audio layer effects. The stateless-per-block preroll model either forces `reset` plus preroll per block (expensive) or needs a stateful streaming mixdown | P2 | M |
| AD-5 | Web apps: AudioWorklet host for Wasm audio plugins (no Wasmtime in the browser; use the browser JIT) | P2 | M |
| AD-6 | Session persistence of plugin id, version, state blob and parameter map across all three apps; missing-plugin placeholders | P1 | S |

### App

| # | App | Gap | Pri | Effort |
|---|---|---|---|---|
| AP-1 | SoundCraft | Parameters applied once per 512-sample block (`strip.rs:529`); automation not sample-accurate for hosted or built-in plugins | P1 | M |
| AP-2 | SoundCraft | No sidechain/key-input routing (G-13) | P1 | M |
| AP-3 | SoundCraft | In-process native hosting; a crash kills the app; master silenced on panic | P1 | L (via PL-12) |
| AP-4 | SoundCraft | No MIDI devices (G-02), no MIDI learn, no MPE/MIDI 2.0; instruments can't be played live | P1 | M |
| AP-5 | SoundCraft | Hosted plugins can't be used as AudioSuite/offline; no render-in-place for plugins | P2 | S |
| AP-6 | SoundCraft | AU editors, embedded CLAP editors on macOS/Linux, VST3 Linux views; VST3 blocked in signed macOS builds (G-05) | P1 | M |
| AP-7 | SoundCraft | No ARA2 (Melodyne, RX) | P2 | L |
| AP-8 | SoundCraft | No host modulators (LFO/envelope to `param_mod`), no Grid-like modular device | P3 | L |
| AP-9 | SoundCraft | Fixed 512-sample mix block regardless of device buffer: extra latency for live monitoring through Wasm or native instruments | P2 | M |
| AP-10 | FilmCraft | No plugin hosting of any kind (G11); model-based Enhance Speech absent (good first `ml.*` consumer) | P1 | L |
| AP-11 | EffectCraft | No audio contract or audio read for plugins; audio effects stateless per block | P2 | M |
| AP-12 | craft-libs | No shared DSP crate: SoundCraft `dsp` and FilmCraft `audio-dsp` duplicate biquads, meters, loudness and resampling. A shared `craft-audio-dsp` would back PL-8 kernels | P2 | M |

**Suggested order:**

1. PL-1, PL-4, PL-2 (spike: one gain and one EQ guest in Wasmtime inside SoundCraft's `MixEngine`,
   measuring p50/p99/p99.9 block time against native at 64, 128 and 512 frames).
2. PL-5, AD-1, AD-2.
3. PL-3, PL-6, PL-10.
4. PL-7, PL-8, AP-1, AP-2.
5. PL-12.
6. The ML, ARA and streaming tiers.

## Sources

- CLAP: https://github.com/free-audio/clap · version.h https://raw.githubusercontent.com/free-audio/clap/main/include/clap/version.h · events.h https://raw.githubusercontent.com/free-audio/clap/main/include/clap/events.h · params.h https://raw.githubusercontent.com/free-audio/clap/main/include/clap/ext/params.h · latency.h https://raw.githubusercontent.com/free-audio/clap/main/include/clap/ext/latency.h · state.h https://raw.githubusercontent.com/free-audio/clap/main/include/clap/ext/state.h · audio-ports.h https://raw.githubusercontent.com/free-audio/clap/main/include/clap/ext/audio-ports.h · gui.h https://raw.githubusercontent.com/free-audio/clap/main/include/clap/ext/gui.h · thread-pool https://cleveraudio.org/1-feature-overview/_thread-pool/ · threading discussion https://github.com/free-audio/clap/pull/255
- WebCLAP / WCLAP: https://github.com/WebCLAP · https://github.com/geraintluff/signalsmith-clap-cpp · https://buttondown.com/opendaw/archive/opendaw-now-runs-webclap-plugins/
- VST3 3.8 MIT: https://steinbergmedia.github.io/vst3_dev_portal/pages/Versions/Version+3.8.0.html · https://steinbergmedia.github.io/vst3_dev_portal/pages/VST+3+Licensing/VST3+License · https://librearts.org/2025/11/steinberg-relicenses-vst3-and-asio/
- AUv3 out of process: https://developer.apple.com/documentation/audiotoolbox/migrating-your-audio-unit-host-to-the-auv3-api · https://developer.apple.com/forums/thread/766503
- LV2 worker: https://lv2plug.in/ns/ext/worker · https://drobilla.net/files/lv2_docs/ns
- Bitwig sandboxing: https://bitwig.com/userguide/latest/vst_plug-in_handling_and_options · https://musicradar.com/news/with-bitwig-studio-25-plugins-will-no-longer-crash-your-entire-projects
- REAPER bridging: https://reaper.blog/2012/02/run-plugin-as-dedicated-process/ · JSFX https://www.reaper.fm/sdk/js/js.php · ReaScript https://www.reaper.fm/sdk/reascript/reascript.php
- Web Audio: https://www.w3.org/TR/webaudio/ · WAM2 https://www.github.com/webaudiomodules/api · https://www.github.com/webaudiomodules/sdk-parammgr · https://aes2.org/publications/elibrary-page/?id=23110
- Faust/Wasm: https://hal.archives-ouvertes.fr/hal-02158925 · https://github.com/grame-cncm/faustwasm · Cmajor https://cmajor.dev/ · Elementary https://www.elementary.audio/ · Csound https://www.npmjs.com/package/@csound/browser · WAC 2022 https://zenodo.org/record/6767578/files/WAC_2022_paper_56.pdf
- Wasm performance: Jangda et al. https://www.usenix.org/conference/atc19/presentation/jangda · https://arxiv.org/abs/1901.09056v3 · wasmi 0.32 https://wasmi-labs.github.io/blog/posts/wasmi-v0.32 · https://github.com/wasmi-labs/wasmi-benchmarks · https://github.com/wasmi-labs/wasmi/issues/759 · https://ray-d-song.github.io/wasmz/bench.html · https://00f.net/2023/01/04/webassembly-benchmark-2023/
- Wasmtime: Config (fuel, epochs, memory, pooling) https://docs.wasmtime.dev/api/wasmtime/struct.Config.html · Winch https://docs.wasmtime.dev/examples-fast-compilation.html · https://bytecodealliance.org/articles/winch-aarch64-support · https://arxiv.org/pdf/2505.22610
- SIMD: https://v8.dev/features/simd · https://sevagh-github-io.pages.dev/blog/speed
- Products: Pro-Q 4 https://www.fabfilter.com/news/1733994000/fabfilter-releases-pro-q-4-equalizer-plug-in · Pro-L 2 https://www.fabfilter.com/products/pro-l-2-limiter-plug-in · Pro-C 2 https://www.fabfilter.com/products/pro-c-2-compressor-plug-in · RX 11 https://www.mixonline.com/technology/izotope-announces-rx-11 · https://www.izotope.com/pages/release-notes/rx-standard · Ozone https://www.izotope.com/en/products/ozone.html · Neutron https://www.izotope.com/en/products/neutron.html · Clarity Vx https://www.waves.com/plugins/clarity-vx · Soundtoys https://www.soundtoys.com/ · Valhalla https://valhalladsp.com/shop/reverb/valhalla-vintage-verb/ · Serum 2 https://xferrecords.com/products/serum-2 · Vital https://vital.audio/ · Kontakt 8 https://www.native-instruments.com/en/products/komplete/samplers/kontakt-8/ · Melodyne https://www.celemony.com/en/melodyne/what-is-melodyne · ARA SDK https://github.com/Celemony/ARA_SDK/releases · Auto-Tune https://www.antarestech.com/products/auto-tune · LABS https://labs.spitfireaudio.com/ · LANDR https://www.landr.com/online-audio-mastering/ · HTDemucs https://arxiv.org/abs/2211.08553v1 · https://huggingface.co/adowu/htdemucs-onnx · Adobe Enhance https://podcast.adobe.com/enhance · Audacity https://plugins.audacityteam.org/ · Bitwig Grid https://www.bitwig.com/the-grid/ · Max for Live https://www.ableton.com/en/live/max-for-live/

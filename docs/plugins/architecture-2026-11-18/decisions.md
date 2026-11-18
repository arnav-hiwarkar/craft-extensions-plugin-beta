# Decisions for the owner

Date: 2026-11-18. These are the choices only the owner can make. Each has a recommendation from
the research, and the phase that cannot start without it. Answer the "Before P1" group first;
nothing else blocks the start.

Format: **Decision**, then the recommendation, the main alternative, and what it costs.

## Before P1: metadata and one runtime

**1. Plugins call both a shared vocabulary and every classified app command, behind one gate.**
Recommended: canonical ops plus the registry bridge (`app.run`), with permissions derived from
mandatory command metadata. Alternatives: only hand-written SDK ops (today: 4–11% reach), or raw
native commands (unsafe: nothing declares its effects). Cost: about 7,200 metadata declarations,
35–40 human review-days spread over the app owners.

**2. Command metadata lives next to each command, in each app repo, in a shared `craft-command` crate.**
Recommended over adapter-side tables (they drift) and over typed parameter structs for every command
(too large a rewrite). Cost: an upstream-visible change to all 11 apps' command files.

**3. Commands without metadata are agent-only.** MCP and CLI keep them; plugins see nothing until
the command declares its effects. Alternative: expose with a coarse permission (unsafe).

**4. Who reviews permission widenings.** Recommended: one named security owner via CODEOWNERS on
every `api.lock.json` and `api-widening.log`, with the app owner second. Needed: a name.

**5. Implicit grants for explicit user actions.** Recommended: when the user runs a plugin's
command, selection-scoped read and edit, selection changes, clipboard write and opening a link count
as granted for that one event, with no prompt. Background, automatic, agent and service runs never
get them. Alternative: always prompt (permission fatigue; the Chrome `activeTab` lesson).

**6. Selection changes are their own permission (`selection.write`), gesture-only.** Recommended by
the red team, because "select all, then an edit on the current selection" escalates selection scope
to the whole document. Cost: one more permission name; selection writes are not available to
automatic hooks.

**7. Defer WIT and the component model.** Recommended: one schema source (`craft-command`) generates
Rust, then TypeScript, C and docs; ABI v1 JSON plus a binary bulk call. WIT's speed advantage
(24–81 ns per call) disappears once calls cross into the worker process. Revisit after the worker is
measured. Alternative: WIT now (a second IDL and two engine paths to maintain).

**8. Preferences stay off-limits to plugins in v1.** Plugins ship presets, workspaces, themes and
fonts as data the user applies. Alternative: an `app.preferences` permission (a persistence and
tampering channel).

**9. App-owned network commands stay off-limits to plugins in v1.** Plugins use their own `net.fetch`
to declared hosts. Alternative: allow per command (covert exfiltration through app endpoints).

## Before P2: isolation and UI

**10. Compiled plugin code runs only in a sandboxed worker process, one per package version.**
Recommended by security and the reconciliation: Cranelift bugs exist (one CVSS 9.9 in 2023), and an
in-process abort loses the user's unsaved work. Cost: 6–10 weeks of transport and per-OS confinement
work, plus an IPC hop per call (target ≤5 µs, unmeasured). The core API is batch-shaped to absorb it.
Alternative: in-process Wasmtime (measured faster for chatty plugins; one bug from the user's
document).

**11. No webview for plugin UI.** Recommended: host-drawn tiers only (declarative tree, canvas, host
editors, GPU viewer). A webview cannot dock, clip or theme in egui or wgpu, adds WebKitGTK on Linux,
and is a phishing surface. Cost: UXP- and CEP-style HTML panels must be rewritten as UI trees. Revisit
only if the catalogue shows a famous plugin class that needs it.

**12. Interpreter role.** Recommended: Wasmtime with Cranelift on desktop for every plugin kind; wasmi
(made 3.7× faster in P0) as the CI reference and the fallback where a worker cannot be confined. No
Winch, Pulley or in-browser engine until a web or iOS plugin target exists.

## Before P3: new adapters

**13. Authorize plugin work on the eight new `plugin-beta` branches.** AniCraft, FilmCraft,
VectorCraft, LightCraft, DesignCraft, TextureCraft, MaterialCraft and SoundCraft are now private repos
on GitHub and Forgejo, at upstream `main` from 2026-11-18. Also decide whether these repos should be
public like the first six (free macOS CI runners) or stay private.

**14. Adapter order.** Recommended: MaterialCraft and TextureCraft, then VectorCraft, AniCraft,
FilmCraft, LightCraft, DesignCraft; SoundCraft's audio tier runs as its own stream. Alternatives
follow product priority.

**15. Allow wgpu outside the render crates in MaterialCraft and TextureCraft, through `craft-gpu`
only, and exempt plugin nodes from their "GPU must match the CPU reference" rule.** Plugin nodes are
tested with golden images within a tolerance instead. This changes those apps' `AGENTS.md`.

**16. GPU tiers.** Recommended:
- L1 shader functions: Community tier after a fuzz gate.
- L2 multi-pass shaders with storage and atomics: Verified tier first.
- L3 imperative WebGPU-style API: deferred.
- Device: the one holding the data, not a dedicated plugin device for interactive work.

**17. Loopback network (`net.local:<port>`) for companion apps and local AI servers.** Recommended:
Verified tier only, with a companion challenge, always refusing Craft control, MCP and bridge ports.
Alternative: native helpers only.

**18. Legacy plugin systems.**
- VectorCraft (about 12,300 lines): a shim inside the platform runtime, data migration, removal after
  two releases at parity.
- AniCraft ABI v1: a bridge, removed once platform builds of its samples ship.

Old grants never carry over; users re-consent. Alternative: a hard cut.

**19. Plugin code inside the pure render path** (eases, deformers, expressions). Recommended: allowed
for declared deterministic functions, with a baked result stored in the document as the
missing-plugin fallback. Alternative: bake only (simpler, less capable).

**20. Determinism class is required in every manifest** (`exact | seeded | tolerant | stateful |
networked`). Caches, review renders and farms depend on it.

**21. Render farms.** Recommended: a project plugin lock plus an admin-signed farm policy file listing
allowed grants per version. Alternative: consent per node.

**22. Durable output size.** For 100 MP smart filters, store the cached result at full resolution,
at proxy resolution with re-render required, or let the user choose. Recommended: user choice,
defaulting to full resolution for final-quality documents.

## Before P4: AI and formats

**23. AI runtime.** Recommended:
- candle for curated architectures;
- LightCraft's WGSL convolutions as a fast path;
- a generic ONNX backend picked by a spike (tract versus candle-onnx on five models).

`ort` only as a later optional native pack. Needed: approval of the spike.

**24. Hosting large models in the catalogue.** Weights can be gigabytes. Options: GitHub release
assets (2 GB limit per file), a separate model mirror, or author-hosted with sha256 pinning.
Recommended: author-hosted with sha256 pinning and consented download, mirrored for Official.

**25. Photoshop 8bf filters through a native helper.** Big catalogue (Nik, Topaz, Filter Forge) but
SDK licensing and binary-compatibility risk. Recommended: not now.

**26. JavaScript as an authoring language.** Recommended: later, through a bundled QuickJS Wasm
module running inside the same runtime, never as a second plugin system.

## Before P5: audio

**27. Default isolation for native audio plugins in SoundCraft.** Recommended: plugins share one
bridge process by default ("Together"), and any plugin that ever crashed gets its own process.
Alternatives: per vendor, per plugin, or in process (today; a crash kills the session). Cost: about
20–100 µs per block round trip (estimate).

**28. Real-time Wasm audio runs in the worker, not on the app's audio thread.** Follows decision 10.
The wake latency is unmeasured; the P5 gate measures it before playback relies on it.

**29. CLAP-over-Wasm interface.** Recommended: Craft's own ABI with CLAP semantics, plus an import
adapter for WebCLAP modules (WebCLAP is young and needs wasi-threads).

**30. Bounce equality.** Recommended: bit-exact at every block size for built-ins and deterministic
Wasm plugins; a tolerance for native plugins.

**31. Suite replacement rule wording.** The rule says native helpers "are not OS-sandboxed", while the
code and this plan keep a confined restricted mode. Recommended: amend the wording to describe both
tiers.

## Before public launch

**32. Distribution gate.**
- Before public launch: TUF with an offline threshold root, and offline provenance verification for
  native, Verified and Official packages.
- Now: an hourly signed revocation snapshot, and auto-update cool-off (72 h Community, 24 h Verified).

**33. Native full access for Community authors.** Recommended: forbidden. Verified and above only,
with the signer's Apple Team ID or Authenticode identity pinned. Local and developer builds use
two-step consent and a policy switch.

**34. Agent access to plugins.** Recommended: per-plugin opt-in, on by default for the two lowest
risk levels and off for Online and native plugins; agent-initiated runs are labelled.

**35. Save veto.** Recommended: no plugin may block save, export or quit. `before-save` hooks report
findings only, and fail open.

## Already decided by the research (no action needed unless you disagree)

- One `Runtime<A>` in craft-extensions owns all plugin behaviour; adapters only map. One conformance
  suite for every adapter.
- Effects are enforced at sinks by origin, not just declared.
- Scope is checked on what changes, not only on parameters.
- Only hardware input approves trust UI.
- Per-run tokens on every control port (P0).
- Automatic and render-time execution has no network, secrets, files or clipboard.
- Shared-memory views for pixels and audio; RGBA16F default interchange.
- Durable output with a badge instead of silent pass-through when a plugin is missing.
- Tolerant loading, so unknown plugin data survives open and save.
- Five host-computed risk levels; widening holds updates; activity log per plugin.
- `craft-nn`, `craft-gpu` and `craft-audio-rt` live in craft-libs and are shared by all apps.

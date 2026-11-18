# 02 — Security and sandbox model for ArtCraft plugins

Date: 2026-11-18. Status: research and decision proposal. No repository was changed.
Scope: every execution tier in `docs/plugins/research/capability-program-2026-11-18.md` §2.1
(Data, Host graph, Kernel program, WGSL, Wasm interpreter, Wasm JIT, Native restricted, Native full),
the planned host services (network, secrets, clipboard, registry bridge, events, AI, real-time audio),
and distribution through the signed `craft-plugins` catalogue.

Evidence labels: **[src]** means read in source in this workspace. **[doc]** means a vendor or
standards document (URL given). **[hist]** means a public incident or advisory. **[est]** means a
design estimate that still has to be measured.

---

## 1. Summary

Plugins should be able to do whatever the app's MCP/CLI can do, and still be safe. Three rules make
that possible:

1. **Breadth comes from brokered authority, not ambient authority.** A plugin never holds a raw path,
   socket, process handle or OS window. It holds *capabilities* that the host mints and checks at the
   point where the side effect happens.
2. **Any code generator runs inside an OS sandbox.** That covers Cranelift JIT code and native
   helpers. The Wasm/GPU validation layer is the first wall and the OS process sandbox is the second.
3. **The user sees one risk level, computed by the host.** The detail sits behind it.

The decisions below are explained and justified in later sections.

| # | Decision | Section |
|---|---|---|
| 1 | Cranelift JIT code never runs in the app process. Wasmtime runs in a `craft-plugin-worker` process, one per plugin package version, under the same per-OS confinement as restricted native helpers, with no file, network or exec rights. wasmi may stay in process because it generates no code, and on web and iOS it has to. | 4.1, 4.3 |
| 2 | Wasmtime is configured as a hostile-tenant engine: guard regions, an explicit stack limit, epoch deadlines, a pooling allocator that zeroes memory, Spectre mitigations on, no WASI filesystem or sockets, and a closed import set. It is pinned to supported releases with an advisory-driven update SLA. | 4.3 |
| 3 | WGSL plugins run on the app's wgpu device, but only as **function bodies inside a host-written shader**. Loops are bounded (`ShaderRuntimeChecks::checked()`), plugins get no storage, workgroup memory, atomics or timers, and all bindings are host-created and scoped. Submits are tiled against a time budget. A pre-submit digest journal turns a TDR or crash into an automatic quarantine on the next start. | 4.5 |
| 4 | Native, per OS. **macOS:** our own worker is an XPC service with App Sandbox entitlements. Third-party restricted helpers go through a host-signed launcher that applies a Seatbelt profile with a self-test, replacing the deprecated `/usr/bin/sandbox-exec`. **Linux:** Landlock plus seccomp-bpf plus `no_new_privs` as the baseline, with bubblewrap as an optional extra layer. **Windows:** LPAC AppContainer plus a Job object plus process mitigation policies. Native *full* needs Verified tier or higher, Developer ID or Authenticode identity pinned in the catalogue, and separate consent. | 4.6 |
| 5 | The registry bridge is secured twice. The checked-in classification table is one layer. The other is a **provenance-tagged invocation** (`Origin::Plugin(id)`) that the engine's file, network, preference, history and plugin-management sinks refuse, whatever the table says. | 5.5 |
| 6 | Agents (MCP) and plugins form two separate principal classes. Neither can grant, install or enable the other, and plugin output that reaches an agent is marked as untrusted data. | 5.6 |
| 7 | Permission UX: five host-computed risk levels. Selection-scoped work started by a gesture needs no prompt (Chrome `activeTab` analogue). Network, clipboard read, capture and persistent folders are asked just in time. Widening any permission puts the update on hold. There is a per-plugin activity log and an enterprise policy file. | 5 |
| 8 | Distribution: catalogue ed25519 now, TUF with offline threshold root before public launch, and an hourly revocation feed. GitHub/Sigstore provenance is required for Verified, Official and native packages, and the app verifies it offline. Catalogue CI rebuilds Wasm packages reproducibly for a "Reproduced" badge. Auto-updates wait 72 h for Community and 24 h for Verified. | 6 |

The largest residual risks are three. A GPU driver bug reachable through valid WGSL is the same
exposure every WebGPU page carries. A user grants "full computer access" to a native helper. A
plugin legitimately holds both `document.read` and `network:<host>`, and the user trusts the host
owner. The design limits the blast radius of the first and makes the other two rare and legible. It
cannot remove them.

---

## 2. Where the code stands today

| Area | Implemented today **[src]** | Gap this report addresses |
|---|---|---|
| Wasm engine | wasmi 2 with `consume_fuel`, `set_max_recursion_depth(4096)`, multi-memory and tail-call off, and an import allowlist limited to `craft::{log,call,take,result}` (`crates/ext-host/src/wasm.rs:84-120`). `StoreLimits`: one memory, a 64 MiB session or 512 MiB transform cap, 100 k table elements (`wasm.rs:9-11,520`). App-wide byte pools (`wasm.rs:13-19`). | No JIT yet. When one arrives, it brings codegen risk into the app process unless the design forbids that. |
| Fault policy | Three consecutive faults quarantine a plugin (`crates/ext-host/src/policy.rs`). | Per-tier fault classes are missing (GPU device loss, worker crash). |
| Authorization | Each host call is checked against the declared permission, the live grant, the stop flag, the grant generation and the deadline (`crates/ext-host/src/api.rs:173-218`). Panics are contained (`api.rs:155`). | The registry bridge and host services need sink-level enforcement. |
| Permissions | A closed vocabulary: `document.read/edit:{selection,document,all}`, `files.{read,write}:picked`, `network:<host>` (no IP literals, `localhost`, `.local` or `.internal`), `clipboard.*`, `ui.window` and `app.events`. Sentences are host-authored, and read plus network produces a warning (`crates/ext-core/src/permission.rs`). | No risk levels, no just-in-time prompts, no audit log, no policy file. |
| Grants | Stored per app and package in `store-v2.json`, bound to origin, with generation bumps on change (`crates/ext-store/src/state.rs:43-52`, `docs/plugins/beta-design.md` §B3). | The file can be written by any same-user process, which matters only once a native full helper is present. |
| Native | Out of process, empty environment, killed as a process group, 10 s start and 60 s event deadlines. Restricted mode uses `/usr/bin/sandbox-exec` with a deny-default Seatbelt profile, or `bwrap --unshare-all`. Windows is refused (`crates/ext-host/src/native.rs:45-90`, `docs/plugins/native-protocol.md`). | `sandbox-exec` is deprecated. bwrap fails on stock Ubuntu 24.04. Windows has nothing. |
| Store | Signed index (ed25519, `verify_strict`), digests, serial rollback protection, 30-day expiry, asset URL pinning, kill list with `advisory`, `block-new` and `disable`, attestation recorded at build, lookalike-name and widening checks (`docs/plugins/store-and-distribution.md`, `crates/ext-core/src/review.rs`). | One online key, refreshes about every 6 h, no client-side provenance check. |
| SoundCraft | CLAP, VST3 and AU are hosted **in process** (`soundcraft/crates/{clap-host,vst3-host,au-host}`). The signed build is missing `disable-library-validation` (`soundcraft/docs/gaps.md:126`). | Adding that entitlement to the main app would let any dylib load into it. Move hosting to a bridge process instead. |
| FigCraft collaboration | Node `plugin_data` is part of the synced change set (`figcraft/crates/collab/src/change.rs:180`, `diff.rs:47`). | Peer-authored plugin data is untrusted input to local plugins. |

---

## 3. Threat model

Attacker classes:

- **A1, malicious author.** Publishes a plugin that is hostile from the start.
- **A2, compromised author.** Account or CI takeover, or a malicious dependency.
- **A3, compromised distribution.** Catalogue key, catalogue repository or mirror.
- **A4, malicious content.** A document, a collaboration peer, or a web page fetched by a plugin.
- **A5, malicious co-tenant.** Another plugin.
- **A6, injected agent.** An AI agent carrying injected instructions.
- **A7, local attacker.** Same-user malware. Out of scope beyond not making things worse.

Assets: documents and pixels, unsaved edits, clipboard, OS keychain items, OAuth tokens, files the
user picked, the user's other files, network identity, GPU and CPU availability, UI trust (the
consent dialogs) and the integrity of the catalogue.

Enforcement points:

- **RT** is the runtime: wasmi, Wasmtime, naga or wgpu.
- **OS** is the OS sandbox of the worker or helper.
- **HOST** is `craft-extensions` `Runtime<A>`, ext-host and ext-store.
- **ADP** is the app adapter or engine sink.
- **STORE** is catalogue build and CI.
- **UI** is the host-rendered manager, consent and chrome.

Test IDs refer to §8.

| # | Threat | Mitigation | Where | Residual risk | Test |
|---|---|---|---|---|---|
| T1 | A Wasm guest escapes through an engine bug (e.g. the Cranelift x86-64 addressing bug CVE-2023-26489, CVSS 9.9) | Validation, guard regions, Spectre mitigations, a closed import set. Second wall: the worker process holds no file, network or exec rights and only the data its own plugin was given. Wasmtime is pinned with a 72 h advisory patch SLA. | RT, OS, HOST | An exploit chain (engine bug plus OS sandbox escape). That is the same class as a browser renderer chain. | W1–W6, J1–J5, N1–N6 |
| T2 | A plugin uses its grants to exfiltrate the document | `network:<host>` only, HTTPS, no redirects off the list, private IPs refused after DNS. The read+network warning is host-written. The risk level reads "Online, can send your work to X". Every request is logged. JIT consent on first use. | HOST, UI | A user trusts the host owner. Policy, not technology. | P3, P4, P7 |
| T3 | Side-channel exfiltration through a URL opened in the browser, a remote image in the UI, the clipboard, file writes, a collaboration sync or DNS labels | `ui.open_url` is treated as a network sink: host-shown URL, on a gesture only, query string displayed. The UI renderer never fetches remote URLs (images come from handles only). Clipboard writes happen only during a gesture. Files only through picked handles. Under a wildcard grant, DNS exfiltration goes only to the author's own domain. | HOST, UI | Clipboard-to-other-app and steganography in exports the user saves. Accepted. | U4, P8 |
| T4 | Secret or key theft (another plugin's OAuth token, keychain items) | `secret.*` is namespaced per plugin id and origin, and stored in the OS keychain under a host-owned service name. The guest gets an opaque handle by default, and `net.fetch` injects `Authorization` host-side (`auth: "secret:<name>"`). Raw read only for the plugin's own items. Removed on uninstall. | HOST | A plugin reads its own token and sends it to its own host. That is legitimate. | P9 |
| T5 | Compromised update (A2) | Releases are immutable on GitHub (checked). Provenance is required for Verified, Official and native. Any widened permission puts the update on hold. Auto-update cool-off is 72 h Community and 24 h Verified. An optional pinned author key (minisign). Kill list. | STORE, HOST | An attacker with the author's CI who keeps the same permissions can ship hostile logic inside the existing grants. The cool-off and reports limit how far it spreads. | S4–S8 |
| T6 | Compromised catalogue key or repository (A3) | Today: offline seed, 30-day expiry, serial high-water mark. Target: TUF with a 2-of-3 offline root, an online targets key and a short-lived timestamp key. Client-side Sigstore verification for native, Official and Verified, so the catalogue key alone cannot swap those bytes. | STORE, HOST | Community Wasm bytes depend on the catalogue signature alone. | S1–S3, S9 |
| T7 | A malicious document drives plugin behaviour (crafted `plugin_data`, project references that prompt installs) | `plugin_data` is namespaced per plugin and size-capped on load and on sync. A project reference never installs or grants anything silently (existing rule). The prompt shows the tier, the risk level and the "requested by this document" provenance. Plugins at Online level or above do not run on `document.opened` from external documents until the user acts in that document (decision D13). | HOST, ADP, UI | Plugin parsing bugs stay inside the sandbox, so the effect is bounded by that plugin's grants. | C1–C3 |
| T8 | Confused deputy through registry commands (`app.run` reaching export-to-path, preferences, scripts or MCP) | The classification table (query, edit, refused) defaults to *refused*, and CI fails on any unclassified command. Every invocation carries `Origin::Plugin{id, scope}`. File, network, preference, plugin-management, history, collaboration and agent sinks refuse a plugin origin. Target ids are checked against the captured scope. | ADP, HOST | The table is wrong about an edit command's reach inside the document, so it edits outside the granted scope. The scope check at the target catches it. | R1–R5 |
| T9 | UI spoofing: fake permission dialogs, password phishing, chrome overlays | Plugin UI is declarative data rendered by the host (UI 1.0). Every plugin surface carries a host-drawn attribution strip (icon, name, tier) that cannot be hidden. Plugin surfaces cannot be modal, full-screen, borderless or always-on-top. Consent only ever appears in host sheets attached to the app window, never inside plugin surfaces. Credentials only through OAuth in the system browser or a host-drawn secret field. Notifications are rate-limited and attributed. | UI, HOST | A plugin can show a *picture* of a dialog in an image node. The attribution strip and "permissions are only asked in this sheet" education limit the damage. | U1–U5 |
| T10 | Persistence | Wasm code has no autostart outside the app. Event subscriptions are visible in the manager. Private storage is capped and erased on uninstall. Restricted native can write only its data folder. Full native can install LaunchAgents, services and similar. | HOST, OS | Full native is total by definition, so it is gated (D7). | N4 |
| T11 | CPU exhaustion or hang | wasmi fuel slices (existing). Wasmtime epoch deadlines (1 ms ticks) plus async yield. Per-event, job and pixel budgets. A worker watchdog kills it 250 ms after a missed deadline. Three strikes quarantine. | RT, HOST | Long legitimate jobs need `job.*` with progress. | X1, X2 |
| T12 | Memory exhaustion | `StoreLimits`/`ResourceLimiter`, pool caps (existing). Per-worker OS limits (Job memory on Windows, RLIMIT_AS or a cgroup on Linux). Host image handles keep large maps outside guest memory. | RT, OS, HOST | Host-side image handle quotas need per-plugin accounting. | X3 |
| T13 | GPU hang, TDR or device loss | Bounded loops, tiled submits with a time budget, no storage or atomics. Device-lost handling rebuilds the device. The pre-submit digest journal quarantines the shader on the next start. Kill list by WGSL digest. | RT, HOST | A driver hang that resets the whole adapter (all apps blink). | G3–G6 |
| T14 | Disk exhaustion | Settings 1 MiB. A data folder quota (default 256 MiB, raisable with consent). Job output through picked handles. Caches LRU-evicted by the host. Logs as a ring. | HOST, OS | Full native. | X4 |
| T15 | Event storms, edit loops | Coalescing, deferred follow-up events with loop detection (H4), timer minimum 16 ms, notification and window rate limits. | HOST | None significant. | X5 |
| T16 | Spectre or timing side channels from JIT code | The worker process holds only that plugin's data (process-per-package). No high-resolution clock (1 ms quantized), no Wasm threads or shared memory (R10), no GPU timestamps. Wasmtime Spectre mitigations on. | OS, RT, HOST | Coarse timing inside the plugin's own process. Nothing of value there. | J4 |
| T17 | GPU driver bug through valid WGSL, cross-resource reads | naga validation, host-composed module, a minimal feature set, robust access, bindings scoped to inputs the plugin may read. Verified-or-above only until the fuzz gate passes (D8). Digest kill list. | RT, HOST, STORE | Driver bugs, the same as WebGPU. | G1, G2, G7 |
| T18 | Native helper with full access | Separate per-version consent. Verified-or-above only. Developer ID, notarization or Authenticode signer pinned by the catalogue. Process group or Job kill on revoke. | STORE, UI, OS | Total by design. | N7 |
| T19 | Restricted native escapes its confinement | Deny-default profiles, a launcher self-test canary, fail closed when confinement is unavailable (existing rule). | OS | OS kernel or sandbox bugs. | N1–N6 |
| T20 | Typosquatting, dependency confusion | Ids are bound to the numeric GitHub id, there are lookalike-name checks (existing), dependencies between plugins resolve by full id plus vendor id within signed catalogues only, and there is no unsigned mirror path. Author-side crate dependencies are caught by SDK templates that ship `Cargo.lock`, `cargo deny` and `cargo vet`. | STORE | Malicious transitive crates in an author's build. Provenance shows *what* was built, not that it is benign. | S10, S11 |
| T21 | Maintainer or vendor takeover | Identity is the numeric GitHub id. Org vendors must show 2FA enforcement. An optional pinned author signing key. A Developer ID Team ID pin for native. Widening puts the update on hold, plus the cool-off. | STORE | A full takeover of a solo maintainer with no pinned key. | S6, S7 |
| T22 | Plugin-to-plugin attacks | Separate instances and processes. `plugin_data` namespaced. `service.call` runs in the provider's sandbox with the provider's grants. Consent shows the transitive reach ("can pass data to Y, which can connect to Z"). No shared worker across packages. | HOST, OS, UI | Collusion between two plugins from one author is a single principal. | P10 |
| T23 | Prompt injection between agents and plugins | Plugin outputs reach agents wrapped as untrusted data, with provenance. Plugins cannot invoke agents or `mcp.*`. Agents cannot install, grant or enable. Plugin commands are exposed to MCP only per-plugin opt-in, and they run with the plugin's grants, not the agent's. | HOST, ADP | An agent follows injected text inside its own authority. That is the agent's own problem, kept bounded by the above. | A1–A4 |
| T24 | Privacy and telemetry | No plugin telemetry by the host. Catalogue fetches are anonymous (existing). Plugins see no device identifiers, quantized time, `fonts.list` without system font paths. The network log is local only. Crash reports exclude plugin data. | HOST | A plugin's own telemetry to its granted host is disclosed by the risk level. | P11 |
| T25 | Collaboration peers inject plugin data or widget state | Plugin data from peers is untrusted input: size-capped, namespaced, schema-versioned. The server validates the namespace on merge. A missing plugin is a placeholder, never an install prompt without provenance. Widget code runs locally under local grants. | ADP, HOST | A peer sends data that a local plugin with network forwards. It is the same as T7. | C2, C3 |
| T26 | Grant store tampering, version downgrade | Only native full or same-user malware can write `store-v2.json`. Optional HMAC of the grant table with a keychain key detects tampering by non-plugin malware. Rollbacks recheck the kill list and digest (existing). | HOST | Same-user malware (A7). | S12 |
| T27 | Social engineering through a sideloaded file | The Local tier badge "Not reviewed". A local native file gets a two-step consent with the risk sentence typed out. Policy can forbid local installs. | UI, HOST | User override. | U6 |

---

## 4. Runtime isolation design

### 4.1 Process topology

```text
App process (EffectCraft, PhotoCraft, …)  — hardened runtime, library validation ON
 ├─ Runtime<A>: grants, transactions, op dispatch, audit log, event bus, net/secret/clipboard brokers
 ├─ wasmi engine (in-process allowed: no codegen; web + iOS mandatory)
 ├─ ext-gpu: naga validation + host-composed WGSL on the app wgpu device
 └─ supervisor ──spawn──▶ craft-plugin-worker  (1 per package version, lazy, idle-reaped)
                           Wasmtime/Cranelift, AOT cache read-only, shared-memory tiles
                           OS sandbox: no fs (except RO module cache), no net, no exec, no fork
                   ──spawn──▶ native helper (restricted)  same sandbox family, per-package data dir
                   ──spawn──▶ native helper (full)        consent, signer pin, job/process-group kill
                   ──spawn──▶ craft-plugin-bridge (SoundCraft) VST3/AU/CLAP; only process with
                                                          disable-library-validation
```

Why a worker per package version, not one shared worker or one per contribution:

- **Chrome's lesson.** V8's own analysis concluded that Spectre cannot be fully mitigated inside one
  process, so Chrome moved to site isolation (process per site)
  ([v8.dev/blog/spectre](https://v8.dev/blog/spectre)). The plugin package is our "site".
- **Cloudflare's lesson.** Workers keep many tenants per process but freeze timers, forbid
  multithreading and move suspicious tenants into separate processes
  ([Cloudflare Workers security model](https://blog.cloudflare.com/mitigating-spectre-and-other-security-threats-the-cloudflare-workers-security-model/)).
  That is economical at thousands of tenants per server. A desktop app runs tens of plugins at most,
  so a process each is affordable.
- **Bitwig's lesson.** Bitwig offers hosting modes from "within Bitwig" to "individually" and lets
  users trade memory for crash isolation ([bitwig.com user guide](https://www.bitwig.com/userguide/latest/)).
  Per-package matches its "by plug-in" mode, which is the good default.
- **Cost [est].** Roughly 10–20 MB resident per idle worker, about 5 ms to spawn on macOS and Linux
  (Windows is slower). The AOT artifact is already cached by sha256 (R4). Workers are reaped after
  60 s idle, except audio and live-preview workers.

**Transport.** Tiles and buffers use shared memory: `memfd`/`shm_open` on Unix, `CreateFileMapping` on
Windows. They are mapped into the worker, and the guest's linear memory is backed by that mapping
through Wasmtime's embedder memory API (`Config::with_host_memory` / `MemoryCreator`). This keeps the
zero-copy goal of R3 and R5. Host calls go over a shared-memory ring with an eventfd/futex or Windows
event wake.

**Acceptance targets [est].**

- Round trip at most 5 µs p50 and 20 µs p99. Today's in-process JSON `craft.call` is 10.5 µs.
- The per-pixel 1080p loop end to end stays within 10% of the in-process Cranelift figure (1.29 ms
  with epochs).
- If either target fails, the fallback is **not** in-process JIT. The fallback is batching
  (`ops.batch`) and the typed binary ABI.

**The worker as broker client.** The worker holds no authority. Every `craft.call` is forwarded to
the app process, which runs `authorize_call` (`api.rs:181`) exactly as today. Compromising the worker
therefore gains only what the plugin was already granted. That is the property Chrome's renderer and
browser split gives ([Chromium sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md)).

### 4.2 wasmi, the interpreter tier

Posture: wasmi is a pure-Rust interpreter. It has no JIT, so it emits no machine code, and its
attack surface is memory-safety bugs in `unsafe` interpreter internals. The project README lists
third-party audits ([wasmi-labs/wasmi](https://github.com/wasmi-labs/wasmi)). It is therefore safe to
keep in process, and on iOS it is the only possible engine (no JIT entitlement, no subprocesses) and
on the web it is the fallback.

Required configuration. The current config at `wasm.rs:84` is kept and extended:

| Setting | Value | Reason |
|---|---|---|
| `consume_fuel` | on (keep) | Deterministic budgets. wasmi has no epochs. |
| `portable-dispatch` | **off** (R1) | 3.7× speed. No security cost: it is a dispatch strategy, not a check. |
| `deterministic` feature | on (keep) | NaN canonicalization means fixed results and fewer guest-observable host differences. |
| `simd` | on (R2) | Validated like any other opcode. |
| Imports | `craft::{log,call,take,result}` only (keep) | The capability surface is the op table, not imports. |
| Recursion depth | 4096 (keep) | Bounded native stack use by the interpreter. |
| Threads, shared memory | refused | No timer amplification, no data races into the host. |
| Max module size | 32 MiB (keep). Also validate at install, not only load. | DoS at compile time. |

### 4.3 Wasmtime/Cranelift, the JIT tier

Facts about Wasmtime's security model [doc]
([docs.wasmtime.dev/security.html](https://docs.wasmtime.dev/security.html)):

- Linear memories are preceded by a 2 GB guard region, as defence against Cranelift sign-extension
  bugs, plus trailing guards.
- Explicit stack overflow checks.
- Spectre mitigations on `call_indirect` table bounds and `br_table`, and on bounds checks for
  dynamic memories. On aarch64, `csdb` is **off by default** (setting `use_csdb`).
- Instance memory, tables and data are zeroed after use. Control-flow integrity is in progress.

Wasmtime counts "anything that undermines the Wasm execution sandbox" as a vulnerability, but only
on **tier 1 platforms and features**. A DoS during compilation is *not* a vulnerability, whereas a
runtime DoS such as a non-yielding loop under async fuel is
([what is a security vulnerability](https://docs.wasmtime.dev/security-what-is-considered-a-security-vulnerability.html)).
The project relies heavily on fuzzing and formal work on Cranelift lowering rules
([Security and correctness in Wasmtime](https://bytecodealliance.org/articles/security-and-correctness-in-wasmtime)).

CVE history [hist]:

| Advisory | Class | Lesson for us |
|---|---|---|
| CVE-2021-32629 | Cranelift x64 codegen allowed out-of-bounds heap access | JIT bugs are real. Hence the second wall (OS sandbox). |
| CVE-2022-39393 | The pooling allocator failed to zero memory on reuse, leaking data between instances | Never share one worker across packages. Run a zeroing test (J3). |
| CVE-2023-26489 (GHSA-ff4p-7xrq-q5r8) | x86-64 address-mode computation produced a 35-bit effective address: guest-controlled out-of-bounds read and write, CVSS 9.9. aarch64 was not affected ([GitLab advisory](https://advisories.gitlab.com/cargo/wasmtime/CVE-2023-26489/)) | The worst case is a full in-process escape. Out of process plus an OS sandbox turns it into "can read its own worker". |
| 2026 batch (GHSA-43j2-gcp4-m22g, GHSA-6fr9-xmw9-5352, others) | A guest can abort the host. Unbounded memory in component and WASI streams ([advisories](https://github.com/bytecodealliance/wasmtime/security/advisories)) | Aborts must hit only the worker. Do not enable the GC proposal, WASI HTTP or WASI streams. |

Required Wasmtime configuration:

| Setting | Value | Reason |
|---|---|---|
| Strategy | Cranelift. Winch only as a fast-compile preview tier, also in the worker | Speed (1.2× native measured, capability doc §3). |
| `epoch_interruption(true)` | Engine ticker every 1 ms. `set_epoch_deadline` per slice. Async yield on deadline | Deadlines, cancel and revocation without fuel's +75% cost (measured +34% for epochs). |
| `consume_fuel` | off on desktop. The deterministic-replay mode uses wasmi | Epochs are the budget. |
| `max_wasm_stack` | 512 KiB (contribution may request up to 2 MiB) | Bounded stack. Overflow traps cleanly. |
| Memory | Static memories with default guard regions. `memory_init_cow` on | Guard pages enforce bounds. Cheap instantiation. |
| Allocator | `PoolingAllocationConfig` sized per worker ([docs.rs](https://docs.rs/wasmtime/latest/wasmtime/struct.PoolingAllocationConfig.html)). Memory protection keys only where tier-1 supported, and only inside one package's worker | Fast instantiation for tile fan-out (R9). Zeroing on reuse. |
| `ResourceLimiter` | Memory and table caps, as with wasmi today | Same limits on both engines. |
| `cranelift_nan_canonicalization` | on | Determinism across engines. |
| Spectre settings | Defaults on. On aarch64 enable `use_csdb` for the Community and Local tiers [est: measure the cost] | aarch64 Macs are the main desktop target. |
| Proposals | Allowed: MVP, bulk memory, multi-value, reference types, SIMD128, relaxed SIMD (deterministic mode), tail calls. **Refused:** threads, shared memory, GC, exceptions until tier 1, memory64 until tier 1, component-model async | Tier-1 only, per Wasmtime's own security scope. |
| WASI | **None** of `wasi:filesystem`, `wasi:sockets` or `wasi:http`. `wasi:random` (host CSPRNG) and `wasi:clocks` quantized to 1 ms if the component world is adopted | Authority only through Craft ops. |
| AOT cache | Compiled at install by the **worker** (sandboxed), keyed by module sha256 plus Wasmtime version plus config hash. Read-only to the worker at run time. `Module::deserialize` only from cache files written by our own compile step, verified with a host-held HMAC | `deserialize` of attacker bytes is unsafe in Wasmtime. The HMAC stops swapped cache files. |
| Version policy | Track the latest supported Wasmtime LTS line. Patch within 72 h of a Wasmtime security advisory. A CI job fails when `cargo audit` reports an advisory | Advisory flow is frequent (10 on the first page in Oct–Nov 2026). |

**Component model and WASI preview 2 (D8 in the capability doc).** The component model's
resource handles are a natural capability system: a plugin can only use handles it was given, and
it has no ambient authority. When `craft:plugin@2` is adopted:

- Define the world with *only* Craft interfaces: `image-handle`, `doc-scope`, `net-request` and
  `secret-handle` as resources. Imports are limited to `wasi:random` and a quantized
  `wasi:clocks/monotonic-clock`.
- A scope (selection or document) becomes an *owned resource* the host mints per invocation. It dies
  when the invocation ends, so a stale scope cannot be replayed (today's `Stale` error, `-4`).
- Keep the JSON op path for app-specific ops. Both paths go through the same `authorize_call`.
- Do not adopt component-model async or streams until their unbounded-memory advisories settle
  (GHSA-6fr9-xmw9-5352, GHSA-5m8g-xhq4-4h93).

### 4.4 Tenant isolation elsewhere

| System | Isolation unit | Timers and threads | What we copy |
|---|---|---|---|
| Chrome (renderers, WebGPU) | Process per site. OS sandbox. GPU work in a separate GPU process | `performance.now` coarsened. SharedArrayBuffer only when cross-origin isolated | Process per package. Broker-held authority. No high-resolution timer. |
| Cloudflare Workers | V8 isolate. Many per process. Dynamic process isolation | `Date.now` frozen during execution. No threads | Freeze or quantize time. No threads. |
| Fastly Compute | Wasmtime instance per request. Pooling allocator | Epochs | The pooling and zeroing configuration. |
| Figma plugins | QuickJS compiled to Wasm, in the page, after a 2019 Realms-shim sandbox escape ([Figma, "An update on plugin security"](https://madebyevan.com/figma/an-update-on-plugin-security/)); UI in a null-origin iframe | n/a | A language VM *inside* Wasm, not a JS-level membrane. Plugin UI kept away from host chrome. |
| VS Code | Extension host process, **unsandboxed**, with the user's full privileges. Workspace Trust only gates workspace-driven execution ([docs](https://code.visualstudio.com/docs/editor/workspace-trust)) | n/a | The anti-pattern for "can do anything". Workspace Trust maps onto our external-document rule (D13). |

### 4.5 GPU sandbox (WGSL tier, K7)

Threats come from the WebGPU specification §2.1 ([w3.org/TR/webgpu](https://www.w3.org/TR/webgpu/#security-considerations)):

- undefined behaviour on the CPU and GPU sides;
- uninitialized memory;
- out-of-bounds shader access;
- driver bugs;
- timing attacks (timestamp queries are precision-reduced, and writable storage buffers can build
  timers);
- row hammer (platform only);
- denial of service, handled by a watchdog of "a few seconds" plus memory caps;
- workload identification.

Chrome answers these by validating shaders (Tint), applying robustness transforms and running
everything in a separate, sandboxed GPU process ([Chromium sandbox design](https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md)).

Design:

1. **The plugin supplies function bodies, not modules.** Its WGSL defines, for example,
   `fn craft_pixel(uv: vec2f, p: Params) -> vec4f` and helpers. The host wraps them with the entry
   point, bindings and the tiling prologue. Plugins never declare `@group`/`@binding`, entry points,
   `var<storage>`, `var<workgroup>`, atomics, barriers, subgroups, pointers into storage, or `enable`
   extensions other than an allowlist (`f16` where supported).
2. **Validation.** naga parses and validates with all flags (`ValidationFlags::all()`). An IR walk
   enforces item 1 and size limits: at most 64 k expressions, a loop-nesting cap, a function count
   cap. Validation runs at catalogue build, at install, and again at load. The validated IR digest is
   the identity used by the kill list.
3. **Runtime checks.** The module is created with `ShaderRuntimeChecks::checked()`, which keeps
   bounds checks and `force_loop_bounding`. wgpu's documentation says this flag "SHOULD NOT be
   disabled when running untrusted code", because backend compilers may treat infinite loops as
   unreachable and draw unsafe conclusions ([wgpu docs](https://wgpu.rs/doc/wgpu/struct.ShaderRuntimeChecks.html)).
   Never use `create_shader_module_trusted` for plugin code.
4. **Scoped bindings.** Every texture and buffer bound to a plugin dispatch is host-created and
   contains only data the plugin may read: its inputs, its parameters, its own outputs. Robust access
   may clamp an out-of-bounds read to *another element of the same resource*, never to another
   document. That holds only because we never bind shared atlases or arenas to plugin code. That
   rule is the invariant G2 tests.
5. **Initialization.** wgpu follows WebGPU zero-initialization semantics, tracking resource
   initialization and clearing before first use. The host also never re-binds a pooled texture to a
   different plugin without clearing it.
6. **Time.** The host splits work into tiles sized by a measured budget. The target is ≤ 8 ms of GPU
   time per submit for interactive use and ≤ 100 ms for jobs, far below the Windows TDR default of
   2 s ([TDR registry keys](https://learn.microsoft.com/en-us/windows-hardware/drivers/display/tdr-registry-keys))
   and the macOS command-buffer watchdog. Timestamps are host-only, and plugins receive no timing. A
   single tile over 250 ms aborts the cook and records a strike.
7. **Device loss and crash loops.** Before each submit, the host appends `{plugin, digest}` to an
   mmap'd journal and clears the entry on completion. If the process dies or the device is lost with
   an entry outstanding, the next start (or the device-lost callback) quarantines that digest and
   tells the user. Chrome uses the same pattern to blocklist GPU features after repeated crashes. The
   app's renderer must already survive device loss. Plugin cooks re-run after recovery from cached
   inputs.
8. **Shared device vs separate device.** wgpu cannot share textures between devices without a copy.
   A separate device would break the GPU effect chain that is the whole point of the tier. A TDR also
   usually resets the adapter regardless of device. **Decision:** use the app device, keep plugin
   resources in a separately accounted arena (memory cap per plugin, default 512 MiB), and offer
   "Isolate GPU plugins" as a diagnostic setting that uses a second device with copies.
9. **Tier gate.** WGSL is Verified and Official only (matching capability doc decision 2) until
   these all pass: a WGSLsmith-style fuzz campaign over naga, a CTS-style subset on the
   macOS/Windows/Linux CI GPUs, and 90 days without a WGSL-attributed device loss. Then Community.

GPU from native or Wasm: `gpu.dispatch` from Wasm only references host-validated shader digests from
the same package. Wasm never submits raw command buffers.

### 4.6 Native per OS

#### How others isolate native plugins

| Product | Mechanism | Lesson |
|---|---|---|
| Logic and GarageBand (AUv3) | App extensions run out of process in their own App Sandbox, with XPC to the host (in-process loading only when both sides opt in) ([AUv3 docs](https://developer.apple.com/documentation/audiotoolbox/audio_unit_v3_plug-ins)) | OS-sandboxed, real-time-capable out of process is achievable on macOS. |
| Final Cut Pro (FxPlug 4) | Plug-ins run as XPC services out of process ([FxPlug](https://developer.apple.com/documentation/professional_video_applications/fxplug)) | Pixel IPC with IOSurface is fast enough for pro video. |
| Bitwig | Plug-in host processes: within, together, by manufacturer, by plug-in, individually | Crash isolation as a user choice. Our bridge copies this. |
| Chrome | Seatbelt via sandbox SPI on macOS ([mac sandbox](https://chromium.googlesource.com/chromium/src/+/main/sandbox/mac/README.md)). seccomp-bpf plus namespaces on Linux ([linux sandboxing](https://chromium.googlesource.com/chromium/src/+/main/docs/linux/sandboxing.md)). Restricted token, job, alternate desktop and (L)PAC AppContainer on Windows, with the network service and GPU process in LPAC by feature ([features.cc](https://chromium.googlesource.com/codesearch/chromium/src/+/HEAD/sandbox/policy/features.cc)) | The reference per-OS design. |
| VS Code | Unsandboxed extension host | Do not copy. |
| Adobe UXP | JS runtime with manifest `requiredPermissions` (network domains, `localFileSystem`, `launchProcess`, clipboard, `allowCodeGenerationFromStrings`). C++ hybrid plugins run in process | Its vocabulary resembles ours. Its native hybrid route is the weak point we avoid. |

#### macOS

- **Our own worker (`craft-plugin-worker`).** Ship it as an XPC service inside each app bundle,
  signed by us with `com.apple.security.app-sandbox`. It gets no network entitlement and no
  user-selected-files entitlement. It gets a read-only exception for the module cache, plus
  `com.apple.security.cs.allow-jit` for Cranelift (hardened runtime). This is the supported,
  non-deprecated App Sandbox route, so Apple maintains the profile.
- **Third-party restricted helpers.** We cannot add entitlements to someone else's binary. The
  `sandbox-exec` man page marks the tool as deprecated. Replace `/usr/bin/sandbox-exec`
  (`native.rs:66`) with our signed `craft-sandbox-launch`, which:
  1. compiles and applies the deny-default SBPL profile (today's profile at `native.rs:45`) through
     the same Seatbelt mechanism Chrome uses;
  2. **self-tests**: tries `open("$HOME/.ssh")`, `socket(AF_INET)` and `fork()`, and requires `EPERM`
     from each;
  3. only then calls `execve` on the helper.

  A future OS that silently weakens or drops the mechanism therefore fails closed, which keeps
  today's rule ("never runs unconfined in place of a restricted start").
- **Profile tightening.** Add `(deny mach-lookup)` except the two services allowed now, and
  `(deny iokit-open)`. A separate opt-in "restricted+gpu" profile allows only the Metal services
  Chrome's GPU profile needs, for vendor Metal kernels.
- **Restricted network.** Never grant sockets. A helper that needs a licence server gets a brokered
  `net.fetch` through the protocol, with the same `network:<host>` permission. Sockets stay
  closed.
- **Native full.**
  - Require Developer ID signing plus notarization. At install, check
    `SecStaticCodeCheckValidity` with the requirement
    `anchor apple generic and certificate leaf[subject.OU] = "<TeamID>"`, where the Team ID is
    pinned in the signed listing.
  - Start it through the launcher, which clears the environment and sets the process group
    (existing).
  - **TCC attribution:** a child process normally inherits its parent app's privacy grants (Camera,
    Photos, Desktop). Evaluate "responsibility disclaim" when spawning so that TCC prompts name the
    helper. Until that is proven, the consent text must say "can use the permissions you gave this
    app".
- **Main app.** Keep hardened runtime *and* library validation. Never load third-party dylibs into
  the app process. SoundCraft's VST3/AU/CLAP hosting moves to `craft-plugin-bridge`, the only binary
  with `disable-library-validation` (see `soundcraft/docs/gaps.md:126`).
- **Endpoint Security** needs an Apple-granted entitlement meant for security products. It is not a
  plugin isolation tool and is not used.

#### Linux

- **Baseline (always available on supported kernels).** A launcher sets `PR_SET_NO_NEW_PRIVS` and
  `PR_SET_PDEATHSIG`, then applies Landlock and seccomp-bpf, then calls `execve`.
  - **Landlock** ([kernel docs](https://docs.kernel.org/userspace-api/landlock.html)): filesystem
    rules from ABI 1 (Linux 5.13). Read and execute only on the release folder and system libraries.
    Read and write only on the data folder. `EXECUTE` is limited to the helper path, so it cannot
    exec anything else. From ABI 3, truncate. From ABI 4, TCP bind and connect denied. From ABI 5,
    device ioctl. From ABI 6, abstract UNIX sockets and signals scoped. From ABI 9 and 10, pathname
    UNIX sockets and UDP. The launcher records the ABI it achieved, and the manager shows it.
  - **seccomp-bpf.** An allowlist of syscalls.
    - `socket` is refused for every family except none: the IPC descriptors are inherited.
    - `clone` is refused unless `CLONE_THREAD` is set (no fork). `clone3` returns `ENOSYS`.
    - Also refused: `ptrace`, `process_vm_*`, `bpf`, `perf_event_open`, `userfaultfd`, `keyctl`,
      `mount` and `unshare`.
  - **rlimits:** `RLIMIT_AS` (worker), `RLIMIT_FSIZE`, `RLIMIT_CORE=0`. Where systemd user cgroups
    exist, place the worker in a scope with `MemoryMax`.
- **bubblewrap as an extra layer**, not a requirement. Stock Ubuntu 24.04 sets
  `kernel.apparmor_restrict_unprivileged_userns=1`, which makes `bwrap` fail with "Creating new
  namespace failed" unless the `bwrap-userns-restrict` profile is loaded
  ([Launchpad 2046844](https://bugs.launchpad.net/ubuntu/+source/guix/+bug/2046844/comments/112)).
  Today's code refuses restricted helpers when `bwrap` is missing (`native.rs:75`). Under this design
  they run on the Landlock and seccomp baseline and gain namespaces when `bwrap` works.
- **Kernels without Landlock** (before 5.13, or with Landlock disabled): run seccomp only, plus a
  user and mount namespace if available. Otherwise refuse restricted helpers and Wasmtime workers,
  and fall back to wasmi in process.
- **Flatpak packaging.** Use `flatpak-spawn --sandbox` with `--no-network` for workers, since
  nested user namespaces are unavailable.

#### Windows (new)

- **Workers and restricted helpers** run as an **LPAC** (Less Privileged AppContainer): an
  AppContainer SID plus `PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY` opt-out, and **no
  capabilities**. That means no `internetClient`, so no network. Chrome runs its network service and
  GPU process in LPAC (`sandbox/policy/features.cc`).
  - The store grants the per-package AppContainer SID read and execute ACEs on the release folder
    and modify ACEs on the data folder at install, and removes them on uninstall.
  - The **Job object** sets:
    - `KILL_ON_JOB_CLOSE`, `ACTIVE_PROCESS=1` (no children), `PROCESS_MEMORY` cap, `DIE_ON_UNHANDLED_EXCEPTION`;
    - UI limits: desktop, display settings, exit windows, global atoms, handles, read and write
      clipboard, system parameters.
  - Also set `PROC_THREAD_ATTRIBUTE_CHILD_PROCESS_POLICY=RESTRICTED` and these mitigation policies:
    - Win32k system-call disable for the non-UI worker;
    - no remote and no low-label image loads;
    - strict handle checks;
    - extension point disable;
    - prefer system32 images.
  - Arbitrary Code Guard is **off** for the Cranelift worker (it needs `PAGE_EXECUTE`) and **on**
    for restricted helpers that declare `jit = false`.
- **Native full.** Separate consent. Authenticode signature with the signer subject and thumbprint
  pinned in the listing (`WinVerifyTrust`). A Job object with `KILL_ON_JOB_CLOSE` so revocation kills
  the whole tree. The native protocol today notes there is no Windows process-group kill
  (`native.rs:33`), and the Job object fixes that.
- **Floor.** Restricted mode requires Windows 10 1703 or later (LPAC). On anything older, refuse,
  as the existing rule says.

#### Real-time audio

- Wasm audio plugins run in a worker with a real-time thread. Shared-memory audio rings carry the
  audio, and a single futex or event wake per block drives it, as Bitwig and AUv3 do.
- On a missed deadline or a fault, the host outputs the **dry signal** (capability doc finding 10).
- Plugin output is sanitized: NaN and Inf become 0, denormals are flushed, and a host safety limiter
  at +6 dBFS protects speakers and ears.
- Audio workers are never reaped while their track is armed or playing.
- In-process real-time Wasm on a JIT is refused (decision 1). If cross-process wake latency fails the
  "p99.9 at most 2× native" target, the fallback is a larger host block for that slot with latency
  reported to the mixer. In-process JIT is not the fallback.

#### Web and iOS

- **Web:** the browser is the OS sandbox. wasmi (or the browser's engine) runs in a dedicated Worker
  with a CSP that blocks network from the worker, and `fetch` is brokered by the main thread through
  the same grant checks. WebGPU validation is the browser's (Tint or naga).
- **iOS:** wasmi in process only. No native tier, no JIT tier.

---

## 5. Permission model and UX

### 5.1 Vocabulary

Keep the invariants written at the top of `crates/ext-core/src/permission.rs`: one closed
vocabulary, broader scope covers narrower, and never-grantable things have no spelling.

| Permission | Status | Scope or attenuation | When asked |
|---|---|---|---|
| `document.read:{selection,document,all}` | exists | Captured per invocation. Selection is frozen at gesture time | Selection: implicit on a gesture (5.3). Document or all: setup sheet |
| `document.edit:{selection,document,all}` | exists | Same, revalidated at commit | Same |
| `files.read:picked`, `files.write:picked` | exists | Per-pick handle | The pick dialog *is* the grant |
| `files.folder:picked` | **new** | A persistent folder handle (security-scoped bookmark on macOS). Listed and revocable in the manager | At first pick, explicit "remember this folder" |
| `network:<host>` | exists | HTTPS. Exact host or one leading `*.`. IDNs only as punycode, shown both ways | Setup sheet lists it. JIT confirmation at first actual request, showing the manifest's reason |
| `clipboard.write` | exists | Only inside a user gesture | Implicit (gesture-bound). Not on the sheet |
| `clipboard.read` | exists | Only inside a user gesture | JIT, with "Always allow" option |
| `ui.window` | exists | Up to 4 windows, never modal or on top | Setup sheet |
| `app.events` | exists | Event filter declared | Setup sheet ("runs automatically when…") |
| `service:<plugin-id>/<name>` | **new** (H9) | The provider runs with its own grants | Setup sheet shows the transitive reach |
| `media.capture:{audio,video}` | **new** | Host-brokered frames. OS TCC also applies | JIT, every session unless "Always" |
| `fonts.install` | **new** (H10) | System-wide effect outside the app | JIT with the font list |
| `net.local:<port>` | **new**, Verified and above only | Loopback to a companion desktop app (Topaz, Houdini). Port pinned | Setup sheet, at the Online risk level |
| Native restricted or full | exists | Per version | Separate consent sheet |

Free, never a permission: compute, kernels, GPU shaders, `nn.run` with packaged models, private
`settings.*`, own `secret.*` handles, own package assets, panels, menus, timers, logging, drag
source, `host.capabilities`.

Never grantable to plugins, with no spelling:

- processes, raw sockets, arbitrary paths;
- other plugins' data or secrets;
- plugin management, grants, preferences, history and undo-stack manipulation;
- collaboration identity;
- invoking MCP or agents;
- keychain items outside the plugin's namespace.

### 5.2 Why not install-time-only, or prompt-everything

- **Install-time lists are ignored.** In the field study of Android install-time permissions, only
  17% of participants paid attention to permissions during install, and few understood them (Felt et
  al., SOUPS 2012, [ACM](https://dl.acm.org/doi/10.1145/2335356.2335360)). Android moved to runtime
  permissions (API 23), one-time permissions and auto-reset of unused grants (Android 11)
  ([Android runtime permissions](https://source.android.com/docs/core/permissions/runtime_perms)).
- **Pickers beat permissions.** iOS asks just in time with developer purpose strings, and its photo
  picker needs no permission at all ([PHPicker](https://developer.apple.com/documentation/photokit/phpickerviewcontroller)).
  That is our `files.*:picked` model.
- **Gesture-scoped grants remove most prompts.** Chrome's `activeTab` grants temporary access to the
  current tab when the user invokes the extension, and carries no install warning
  ([activeTab](https://developer.chrome.com/docs/extensions/develop/concepts/activeTab)). Chrome
  disables an updated extension whose new permissions add warnings until the user re-approves
  ([permission warnings](https://developer.chrome.com/docs/extensions/develop/concepts/permission-warnings)).
  That is our "widening puts the update on hold" rule, which already exists as `pending`.
- **Domain allowlists with reasons.** Figma's manifest `networkAccess.allowedDomains` with
  `reasoning`, Deno's `--allow-net=<host>` ([Deno security](https://docs.deno.com/runtime/fundamentals/security/))
  and UXP's `network.domains` all show that allowlisting by domain is understandable and enforceable.

### 5.3 Risk levels (what the user sees)

The host computes one level from the manifest and the grants. Authors never set it. The manager,
setup sheet, store listing and CLI all show **one label plus one sentence**, with the full list one
click away. Color is never the only signal, matching `Tier::badge`.

| Level | Label | Condition (highest wins) | Example sentence |
|---|---|---|---|
| L0 | **Works on what you select** | Only selection-scoped read or edit, started by you. No events, no network | "Changes only what you select, when you use it." |
| L1 | **Works with your documents** | Document or all scope, `app.events`, `clipboard.read`, `files.folder`, `service:` to an L1 provider | "Can read and change your open documents, and runs automatically when the selection changes." |
| L2 | **Online** | Any `network:` or `net.local:`, or `service:` to an L2 provider | "Can send your work to api.example.com." (from `warnings()`) |
| L3 | **Native, confined** | Native restricted | "Runs its own program, kept away from your files and the internet." |
| L4 | **Full access to your computer** | Native full | "Runs its own program with your full access. Only install software you trust." Two-step consent. |

Tier (Official, Verified, Community, Local, Dev) is shown *beside* the level, never merged into it.
"Verified" is a statement about identity, not behaviour.

### 5.4 Prompt timing

| Moment | What happens |
|---|---|
| Install or enable | One setup sheet (the existing shared sheet). Shows the level, the tier, required permissions as host sentences, and optional ones off by default. Selection-scoped gesture permissions are not listed as choices: they are implied by "you use it". |
| First use of a network host | A JIT sheet: "Photo Upload wants to connect to api.example.com: *<manifest reason>*". Choices: Allow, Allow once, Deny. Shown once per host. |
| `clipboard.read`, `media.capture`, `fonts.install` | JIT, with "Always" stored as a grant. |
| Update widens a permission | The update is held (`pending`) until approved (existing). Notification badge in the manager. |
| Agent or MCP initiates a plugin command | Runs only if the plugin has opted in and the user has enabled "Allow agents to use this plugin" (default off for L2 and above). |
| Headless CLI | Never prompts. Denied unless granted earlier or `--grant` is passed explicitly (existing CLI rule). |
| Unused for 90 days | Optional grants auto-reset, as on Android. Required ones stay, with a note. |

Prompt budget rule, a deterministic check (P6): no plugin can cause more than one JIT prompt per
minute. Prompts are queued and coalesced, never stacked.

### 5.5 Registry bridge and confused deputy

The bridge (`app.run {command, params}`) is the biggest authority amplifier. The classification
table proposed in capability doc §2.3 is necessary but not sufficient, because a command classified
`edit` might reach a sink indirectly: an "apply preset" that loads a file path, or a "replace image"
that fetches a URL. Defence in depth:

1. **Default refused.** A command enters `query` or `edit` only through a reviewed table entry. CI
   fails on any unclassified registry command (R1) or on a `refused` family in the allowed list (R2).
2. **Parameter types.** Any parameter whose schema type is a path, URL, script or command name makes
   the command `refused` automatically, unless the adapter maps that parameter to a handle type (file
   handle, image handle). This is checked mechanically from `CommandSpec` schemas (R3).
3. **Origin at the sinks.** Every engine invocation carries `Origin::{User, Cli, Mcp, Plugin{id,
   scope, generation}}`. The engine's file I/O, network, preferences, plugin management, history,
   collaboration-identity and MCP sinks call `origin.require_not_plugin()`. This is the same idea as
   MCP's confused-deputy guidance: authority belongs to the caller, not to the server
   ([MCP security best practices](https://modelcontextprotocol.io/specification/draft/basic/security_best_practices)).
   R4 fuzzes every bridged command with a plugin origin against mock sinks that panic when reached.
4. **Scope at the targets.** Node, layer and clip ids in parameters are checked against the captured
   scope. A `selection` grant cannot touch an id outside the frozen selection (R5).
5. **One undo step and attribution.** Plugin edits commit as one history entry labelled with the
   plugin name (existing transaction model), so the user can see and undo them.

### 5.6 Agents (MCP) and plugins sharing the command surface

Prompt injection is listed first in the OWASP LLM risks ([LLM01](https://genai.owasp.org/llmrisk/llm01-prompt-injection/)).
Plugins are both sources (layer names, `plugin_data`, panel text) and potential targets.

| Rule | Enforcement | Test |
|---|---|---|
| Agents cannot install, enable, grant, update or configure plugins | Management sinks refuse `Origin::Mcp` | A1 |
| Plugins cannot call agents, MCP tools or `ai.*` commands that act with agent authority | No spelling in the vocabulary. The bridge refuses the `mcp`/`agent` families | A2 |
| Plugin commands exposed as MCP tools only by per-plugin opt-in. Tool names and descriptions are host-written from the manifest and prefixed with the plugin id and tier. Execution uses the **plugin's** grants | Runtime | A3 |
| Data from plugins (`plugin_data`, plugin-set names, plugin UI text) returned to an agent is wrapped as `{"untrusted": true, "source": "plugin:<id>", …}` | MCP server serializers | A4 |
| Plugin edits started by an agent are labelled "via agent" in history and in the audit log | Runtime | A4 |

### 5.7 Revocation mid-run

| Tier | Mechanism | Bound |
|---|---|---|
| All | Grant generation bump. The next `authorize_preflight` returns Denied (existing, `api.rs:209`). The transaction rolls back | Next host call |
| wasmi | `cancel` flag checked between fuel slices (existing) | Under 20 M fuel units, about 10 ms [est] |
| Wasmtime worker | `engine.increment_epoch()` traps at the next epoch check. Worker killed if no ack in 250 ms | Under 260 ms |
| Network | In-flight `net.fetch` aborted, its completion event dropped | Immediate |
| Secrets | Handles invalidated by generation | Immediate |
| GPU | No further tiles submitted. The current tile finishes (under 8 ms) | One tile |
| Audio | The slot switches to dry at the next block | One block |
| Native | Process group or Job kill (existing on Unix, Job on Windows) | Supervisor tick, at most 1 s |

External effects (bytes already sent, files written) cannot be undone. The manager says so
(existing text in `native-protocol.md`).

### 5.8 Audit log

- **Where:** `<store>/audit/<app>/<plugin>.jsonl`, written only by the host. A ring of 10 000
  entries or 30 days per plugin.
- **What:**
  - grant changes;
  - JIT decisions;
  - network requests (host, method, bytes up and down, status, time; never bodies);
  - file handle grants;
  - clipboard reads;
  - secret use (name only);
  - bridged `edit` commands (command id, target count);
  - agent-initiated runs;
  - quarantines, kill-list hits, native start and stop.
- **UI:** an "Activity" tab per plugin in the manager, plus `plugin activity <id> --json` in the CLI.
  This answers "what did it send, and where?", which a permission list cannot.
- **Privacy:** local only, never uploaded. It is excluded from crash reports.

### 5.9 Enterprise policy

Read at start-up. The policy beats user settings.

- **Locations:**
  - macOS: managed preferences domain (MDM profile);
  - Windows: `HKLM\Software\Policies\ArtCraft`;
  - Linux: `/etc/artcraft/policy.json`.
- **Keys** (modelled on Chrome's `ExtensionSettings`):
  - `allowed_ids` and `blocked_ids`, with wildcards per vendor;
  - `minimum_tier`;
  - `max_risk_level`;
  - `allow_native_full` and `allow_native_restricted`;
  - `allow_local_files`;
  - `blocked_network_hosts` and `allowed_network_hosts`;
  - `force_installed` (id and version range);
  - `catalogue_url` plus `extra_trusted_keys`, which needs both keys and still uses the signature
    rules;
  - `allow_agent_access`;
  - `wgsl_enabled`;
  - `auto_update_delay_hours`.
- **Manager:** shows "Managed by your organization" on affected controls.

### 5.10 UI spoofing rules (enforced by the host renderer)

- **U1.** Every plugin panel, window and overlay has a host-drawn attribution strip (icon from the
  catalogue, name, tier). Plugin content cannot cover it, and it has no style hooks.
- **U2.** Plugin windows are never modal, full-screen, borderless, transparent or always-on-top, and
  never sized larger than the app window. At most four per plugin (existing limit).
- **U3.** Consent, JIT prompts, the secret-entry field and OAuth start appear only in host sheets
  attached to the app window. The sheet shows the risk-level icon and the plugin's *catalogue* icon,
  which a plugin cannot change at run time.
- **U4.** The UI tree has no remote URL fetching. Image nodes take image handles or package assets
  only. Links open only on a gesture, through a host confirmation that shows the full URL with the
  registrable domain bold.
- **U5.** `ui.notify` is rate-limited (5 per minute) and always attributed. Text fields cannot be
  marked as password fields. A plugin that needs a credential must use `auth.oauth` or `secret.set`
  via a host sheet.
- **U6.** Local-file installs show the "Not reviewed" tier. Native Local installs need a second
  confirmation where the user picks the risk sentence.

---

## 6. Supply chain and distribution

### 6.1 Layers of trust

| Layer | Today | Target | Protects against |
|---|---|---|---|
| Index signature | ed25519, one online key, `verify_strict`, 30-day expiry, serial high-water | **TUF** ([theupdateframework.io](https://theupdateframework.io)): root 2-of-3 offline keys; targets key with the maintainer; snapshot; timestamp role re-signed hourly with a 24 h expiry | A3 key theft, freeze attacks, rollback |
| Package integrity | sha256 and size pinned per release. Immutable GitHub releases checked | unchanged | Tampered assets |
| Provenance | `gh attestation verify` at catalogue build. "Built from source" badge. The app trusts the index | Sigstore bundle stored in the package file. The app verifies it **offline** against a Sigstore trusted root shipped and updated through TUF. Required for Verified, Official and all native ([GitHub artifact attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations), [SLSA v1.0 levels](https://slsa.dev/spec/v1.0/levels)) | A3 swapping bytes for high-value packages. Build-system tampering (SLSA Build L2, L3 with reusable workflows) |
| Reproducibility | none | Catalogue CI rebuilds Wasm packages in a hermetic container: pinned toolchain from `rust-toolchain.toml`, `cargo --locked`, `--remap-path-prefix`, `SOURCE_DATE_EPOCH`. A matching digest earns a "Reproduced" badge. Required for Official, target for Verified Wasm ([reproducible-builds.org](https://reproducible-builds.org)) | A compromised author CI producing bytes that differ from source |
| Author key continuity | none | Optional minisign or SSH-signing key pinned in `vendors/<login>.toml`. Releases must carry a matching signature ([minisign](https://jedisct1.github.io/minisign/)). Shown as "Signed by vendor key" | Takeover of the GitHub account alone (not the offline key) |
| Native signer pin | none | Team ID (macOS) and Authenticode subject plus thumbprint (Windows) in the listing, checked at install and launch | Swapped native binaries, maintainer takeover |

### 6.2 Review automation (`check-submission` additions)

These rows extend the existing table in `store-and-distribution.md`:

- **wasm-imports** (fail): imports outside the `craft::*` set or the WIT world. A wasmparser walk.
- **wasm-strings** (needs-review): URL literals in data segments that match no declared `network:`
  host.
- **wgsl** (fail or needs-review):
  - fail on naga validation or the IR rules in §4.5;
  - needs-review on loop count or complexity above thresholds;
  - otherwise record the IR digest for the kill list.
- **native-identity** (fail):
  - macOS `codesign --verify --strict` and notarization staple or online check;
  - Windows Authenticode;
  - signer equal to the pinned identity.
- **native-surface** (needs-review): imported symbols such as `ptrace`, `execve`, `CreateRemoteThread`,
  `SetWindowsHookEx`, `AXUIElement`, `CGEventTap`, `IOHIDManager`; setuid bits; nested executables.
- **malware-scan** (needs-review): public-binary hash lookup in a malware database. Only hashes are
  sent, and the binaries are public anyway.
- **reasons** (needs-review): every `network:` permission and every native helper needs a non-empty
  `reason` under 200 characters, shown verbatim in JIT prompts.
- **cool-off**: `published` time recorded so apps can enforce the update delay.

### 6.3 Tier rules

| Capability | Official | Verified | Community | Local | Dev |
|---|---|---|---|---|---|
| Wasm (wasmi or Wasmtime worker) | yes | yes | yes | yes | yes |
| WGSL shaders | yes | yes | after the gate (D8) | after the gate, warning | yes |
| `network:` | yes | yes | yes | yes, warning | yes |
| `net.local:` | yes | yes | no | no | yes |
| Native restricted | yes | yes | yes, provenance required | yes, two-step | yes |
| Native full | yes | yes, signer pin | **no** (D7) | yes, two-step plus policy | yes |
| Auto-update | immediate | after 24 h | after 72 h | never | n/a |
| Provenance required | yes, plus reproduced | yes | native only | n/a | n/a |
| First-party id prefixes | yes | no | no | no | no |

### 6.4 Update policy

- **Cool-off.** Updates are discovered at refresh but installed only after the tier delay since
  `published`, unless the user clicks Update. This is the "minimum release age" defence package
  managers adopted after the 2025 npm worm compromises
  ([CISA alert, Sept 2025](https://www.cisa.gov/news-events/alerts/2025/09/23/widespread-supply-chain-compromise-impacting-npm-ecosystem);
  [pnpm `minimumReleaseAge`](https://pnpm.io/settings#minimumreleaseage)).
- **Widening** of permissions, native mode or level always holds the update (existing `pending`).
  A change of level (for example L1 to L2) is always a widening, even when expressed through
  `service:`.
- **Downgrades and rollbacks** recheck the kill list and digests (existing). Rollback to a
  `block-new` release is refused (existing).
- **Updates while running.** The new version activates at the next invocation. Running sessions
  finish under the old version's grants. The worker is restarted.

### 6.5 Kill switch

- **Latency.** Today the kill list arrives with the full index, about every 6 h. Target: the TUF
  timestamp role is re-signed hourly. Apps fetch the small timestamp and snapshot metadata every
  60 min, plus at start-up, and pull `revocations` when the snapshot changes. Worst case about 1 h.
- **Scope of `disable`:**
  - kills sessions, workers and helpers (existing);
  - kill-lists WGSL digests (new);
  - removes MCP tool exposure (new);
  - marks documents that reference the plugin so they render cached results instead of re-running it.
- **Offline users.** The cached kill list survives metadata expiry (existing). An expired index
  cannot authorize installs (existing).

### 6.6 Dependency confusion, typosquatting, takeover

- **Identity.** Ids bind to the numeric GitHub account id, and first-party prefixes are reserved
  (existing). Lookalike-name checks already exist. Extend them to Unicode confusables in display
  names: `name_key` folds ASCII only, so add a Unicode TR39 skeleton.
- **Dependencies.** Plugin-to-plugin dependencies (`service:`) resolve only by exact id plus vendor
  numeric id from the signed index. There is no search-path resolution, and a mirror cannot add ids
  without a trusted key.
- **Author-side dependencies.** SDK templates ship `Cargo.lock`, `cargo deny` (advisories, sources
  limited to crates.io) and `cargo vet` with imported audits. The attestation names the workflow, so
  a reviewer can see these ran. Malicious crates have repeatedly targeted Rust developers [hist].
  Provenance shows *what* was built, not that it is benign, and the sandbox is what contains it.
- **Takeover.**
  - For organization vendors, `catalogue check` reads the org's `two_factor_requirement_enabled` and
    requires it for Verified.
  - The pinned author key (§6.1), the cool-off and the hold on widened permissions make a silent
    hostile update slow and visible.
  - Reports with the `security` label trigger `block-new` within one maintainer action, without
    waiting for analysis.

---

## 7. Edge cases

1. **Offline for more than 30 days.** Installed plugins keep running under the cached kill list. No
   installs or updates. The manager says the catalogue is stale.
2. **Same plugin in two apps.** Grants are per app (existing). Kill and revoke apply across apps
   through the store generation (existing). Each app has its own worker.
3. **Wildcard domains on shared hosting.** `*.github.io`, `*.vercel.app` and `*.cloudfront.net`
   cover arbitrary third parties. Refuse wildcards on public-suffix-list registrable domains at
   catalogue check. `network:*.example.com` stays allowed.
4. **Domain drift.** A granted domain expires and is re-registered. Record the grant date and show
   it. Policy can require Verified for network. Residual.
5. **IDN homographs.** `valid_host` accepts only lowercase ASCII (so punycode). The sheet shows the
   punycode form plus the decoded form, and warns on mixed scripts.
6. **DNS rebinding and private IPs.** Resolve, check against private, loopback, link-local and CGNAT
   ranges, connect to that exact IP, and re-check on every redirect. `net.local` is the only loopback
   path.
7. **Stale scope.** The selection changes between gesture and commit. Commit-time revalidation
    returns `-4 Stale` (existing). The user retries.
8. **Nested invocation.** A plugin's edit triggers `document.changed` for another plugin. That runs
    as a deferred follow-up event, with depth at most 2 and loop detection (H4).
9. **Plugin invoked by an agent while the user is away.** L2+ plugins are not agent-exposed by
    default. JIT prompts are never answered automatically. They time out to Deny.
10. **GPU device lost during export.** The export job fails with a clear message. The digest journal
    decides whether the plugin is quarantined.
11. **Quarantined plugin referenced by an open document.** Render from the cached raster (smart
    filter rule) and show a placeholder badge. Data is preserved (suite replacement rule).
12. **Uninstall.** Removes grants, settings, the data folder, keychain items, AppContainer ACEs,
    persistent folder bookmarks and the audit log, after confirmation. Project `plugin_data` stays in
    documents.
13. **Portable or USB store, or a network home directory.** Seatbelt subpaths and Landlock rules use
    resolved real paths. Windows ACLs on FAT and exFAT are impossible, so restricted mode is refused
    there with a reason.
14. **Flatpak and Snap app packaging.** Workers use the packaging's own sandbox-spawn API. If it is
    unavailable, fall back to wasmi in process and disable native restricted.
15. **Collaboration peer without the plugin.** The node keeps `plugin_data` verbatim (unknown data
    survives, per the suite replacement rule). No install prompt unless the user clicks the
    placeholder.
16. **Plugin sets very large `plugin_data` to bloat shared documents.** Cap per node and per plugin
    on write *and* on merge from peers (C2).

---

## 8. Test and verification plan

All of these are deterministic, with no timing-flaky assertions except where a bound is checked with
a generous margin. Placement follows the workspace rules:

- **Mac:** cheap scoped `cargo test -p` on the warm craft-extensions cache.
- **Actions:** OS-sandbox, GPU and Windows suites run in Actions, with approval.

| ID | Layer | Check | Method |
|---|---|---|---|
| W1 | RT | Import allowlist | Modules importing `wasi_snapshot_preview1::fd_write`, `craft::exec` and similar are refused at load *and* at catalogue check. Extends the existing `Plugin::load` tests |
| W2 | RT | Memory cap | A guest that grows memory past the cap gets `TooMuchMemory`, for both engines |
| W3 | RT | Stack | Deep recursion traps cleanly, with no host stack overflow, for both engines |
| W4 | RT | Proposals refused | Threads, shared memory, GC and exceptions modules are refused (WAT fixtures) |
| W5 | RT | Differential fuzz | `wasm-smith` modules run on wasmi and Wasmtime. Results must match or both must trap. Nightly in Actions |
| W6 | RT | Host call bounds | Pointer and length pairs out of bounds return `-1`, never a host panic (existing pattern, extended to every op) |
| J1 | RT | Epoch deadline | An infinite loop returns `TooSlow` within deadline + 50 ms |
| J2 | RT | Revocation | A generation bump during a busy loop traps within 260 ms. Transaction rolled back |
| J3 | RT | Zeroing | Instance A writes a pattern. After teardown, pooled instance B scans its memory and finds zeros (CVE-2022-39393 regression) |
| J4 | HOST | Timer precision | Guest clock reads are multiples of 1 ms. No other time source is importable |
| J5 | HOST | AOT cache integrity | A tampered cache file (bit flip, or a valid HMAC from another module) is refused and recompiled |
| N1 | OS | Worker file denial | The worker tries `open($HOME/.ssh/id_ed25519)`, writes to `/tmp` and `%TEMP%`, and reads another package's data folder. Each must fail (macOS, Linux, Windows) |
| N2 | OS | Network denial | `socket(AF_INET)` and `connect(1.1.1.1:443)`, plus `getaddrinfo` (Windows: `WSASocket`), must fail |
| N3 | OS | Exec and fork denial | `fork`, `execve(/bin/sh)` and `CreateProcess(cmd.exe)` must fail |
| N4 | OS | Persistence denial | Writing `~/Library/LaunchAgents`, `~/.config/autostart` or the HKCU Run key must fail |
| N5 | OS | Launcher self-test | With a profile that fails to apply (injected), the launcher refuses to exec. The helper never starts unconfined |
| N6 | OS | Kill tree | A helper that spawns grandchildren (full mode) has all of them killed on revoke: process group on Unix (existing test), Job on Windows |
| N7 | STORE | Signer pin | A native asset re-signed by another Team ID or Authenticode subject is refused at install and launch |
| G1 | RT | WGSL IR rules | Fixtures with `var<storage>`, `var<workgroup>`, `atomicAdd`, `workgroupBarrier`, `@binding` or an entry point are refused |
| G2 | HOST | Binding scope | A plugin shader reads far out of bounds and gets only values from its own input texture (checked via sentinels in a neighbouring resource) |
| G3 | RT | Loop bounding | `loop {}` and `while(true)` shaders complete or are aborted within the tile budget. No device loss on CI GPUs |
| G4 | HOST | Tile budget | A heavy shader is split so each submit stays at or under its budget (timestamps on CI GPU) |
| G5 | HOST | Device-loss recovery | A test hook destroys the device mid-cook. The app re-creates it, the cook fails with a message and the strike is recorded |
| G6 | HOST | Crash journal | Kill the process with a journal entry outstanding. The next start quarantines that digest |
| G7 | STORE | WGSL fuzz gate | WGSLsmith-style generated shaders through naga validation and the host compose step. No panics. Tracked as the Community gate |
| P1 | HOST | Declared ⊇ requires | Every op's `requires` is covered by the declaration, or Denied (existing `authorize`) |
| P2 | HOST | Risk level | A table of manifests maps to the expected level. Widening across a level marks the update pending |
| P3 | HOST | Network enforcement | Undeclared host, HTTP, IP literal, redirect off the list, DNS to 10.0.0.1 or 127.0.0.1 or 169.254.169.254: all refused (local test server with a fake resolver) |
| P4 | HOST | JIT network prompt | The first request to a host blocks on a consent stub. Deny means `-2`, Allow means persisted |
| P5 | HOST | Gesture scoping | `clipboard.write` and `ui.open_url` outside a gesture event are Denied. Inside one they are allowed |
| P6 | UI | Prompt budget | 100 rapid JIT requests produce one sheet per minute |
| P7 | HOST | Audit completeness | Each sensitive op in the vocabulary appends exactly one audit record (a table-driven test enumerating `OpTable` sensitive flags) |
| P8 | HOST | Secondary channels | The UI tree with an `http://` image is refused. `ui.open_url` shows a confirmation with the full URL |
| P9 | HOST | Secret namespace | Plugin B cannot read, list or use plugin A's secret name. Uninstall removes its keychain items |
| P10 | HOST | Service transitivity | A consumer's sheet shows the provider's network reach. A provider call runs with provider grants only |
| P11 | HOST | Privacy | No plugin-related field appears in host telemetry or crash payloads (schema allowlist test) |
| R1 | ADP | Registry classification complete | CI: every `CommandSpec` has a class (capability doc §2.3) |
| R2 | ADP | Refused families | CI: no file, network, preferences, plugin, history, collaboration or MCP command in `query`/`edit` |
| R3 | ADP | Dangerous params | CI: a path, URL or script parameter type in an allowed command fails unless mapped to a handle |
| R4 | ADP | Origin at sinks | For each allowed command, invoke with `Origin::Plugin` against mock sinks that panic if reached |
| R5 | ADP | Scope at targets | A selection-scoped grant plus a target id outside the selection gives `-2` |
| A1 | HOST | Agent management denial | MCP `plugin.install`, `plugin.grant` and `plugin.enable` with `Origin::Mcp` are refused |
| A2 | HOST | Plugin cannot reach agent | `app.run` of `mcp.*`, `agent.*` or `ai.chat` is refused |
| A3 | HOST | Opt-in exposure | An L2 plugin's commands are absent from MCP `tools/list` until enabled |
| A4 | HOST | Untrusted wrapping | MCP results that contain plugin-set strings carry `untrusted:true` and the source |
| U1 | UI | Attribution strip | Renderer snapshot: every plugin surface kind has the strip. Plugin style cannot change its rect or opacity |
| U2 | UI | Window flags | Requests for modal, full-screen, top-most or oversize windows are refused or clamped |
| U3 | UI | Consent provenance | Consent sheets cannot be created from any plugin-reachable API (static check: no `ui.*` op constructs a sheet type) |
| U5 | UI | Notification rate | The sixth notify within a minute is dropped and logged |
| U6 | UI | Local native two-step | Installing a local native package requires two confirmations (manager model test) |
| X1–X5 | HOST | Budgets | CPU (existing fuel tests plus epochs), memory pools, data-folder quota, event-loop depth, timer minimum |
| C1 | ADP | External document rule | `document.opened` from an external document does not wake L2+ plugins until a user action |
| C2 | ADP | Peer plugin_data caps | A collaboration merge with oversized or foreign-namespace `plugin_data` is refused, and the node is kept (FigCraft `collab` tests) |
| C3 | ADP | Missing plugin from peer | A placeholder is shown with no install prompt and no network |
| S1–S3 | STORE | Index signature, expiry, rollback | Existing tests, plus TUF role and threshold tests when adopted |
| S4 | STORE | Immutable releases | Existing `immutable-release` check |
| S5 | STORE | Attestation required by tier | A Verified or native listing with no `signer_workflow` fails (existing). App-side offline bundle verification with fixture bundles |
| S6 | STORE | Author key continuity | A release without a pinned-key signature fails for vendors with a key |
| S7 | HOST | Cool-off | An update younger than the tier delay is not auto-installed (clock injected) |
| S8 | HOST | Widening holds | Existing `review::widenings` tests, plus a level change |
| S9 | HOST | Kill-list latency | A snapshot change triggers a revocations fetch, a running session stops, a worker is killed, and WGSL digests are blocked |
| S10 | STORE | Lookalike names | Existing tests, plus Unicode confusables |
| S11 | STORE | Wildcard on public suffix | `network:*.github.io` is refused at catalogue check |
| S12 | HOST | Grant table HMAC (if adopted) | An edited `store-v2.json` grant is detected and reset |

**Coverage rule.** CI keeps a machine-readable map from threat id (T1–T27) to test ids, and fails if
a threat row has no test. That puts the "code as proof" rule into practice for the threat model
itself.

---

## 9. Open decisions with recommendations

| # | Decision | Options | Recommendation | Why |
|---|---|---|---|---|
| D1 | Where does JIT code run? | In app process; worker process | **Worker process, mandatory.** In-process JIT is never a fallback | CVE-2023-26489-class bugs become worker-local. Spectre isolation, crash isolation. Chrome and Bitwig precedent. |
| D2 | Worker granularity | One shared; per tier; per package version | **Per package version.** Pooling within a package only. Official packages may share one worker | CVE-2022-39393 class. Spectre. Low process count on desktop. |
| D3 | GPU device | App device; separate device | **App device** with host-composed shaders, scoped bindings, tile budgets and a crash journal. Separate device as a diagnostic setting | Cross-device copies would defeat the tier. TDR resets the adapter anyway. |
| D4 | macOS confinement | `sandbox-exec`; own launcher on the Seatbelt mechanism; App Sandbox XPC | **App Sandbox XPC for our worker. Own self-testing launcher for third-party restricted helpers** | `sandbox-exec` is deprecated. App Sandbox is supported for binaries we sign. A self-test fails closed. |
| D5 | Linux confinement | bwrap required; Landlock+seccomp; both | **Landlock + seccomp + no_new_privs baseline. bwrap added when it works** | Ubuntu 24.04 blocks unprivileged user namespaces by default. Landlock needs none. |
| D6 | Windows | Refuse restricted (today); restricted token; LPAC + Job | **LPAC + Job + mitigation policies**, in the same phase as Wasmtime | Gives Windows parity. Chrome uses LPAC for its network and GPU services. |
| D7 | Native full for Community | Allow; forbid | **Forbid.** Verified+ with signer pin. Local or dev with two-step consent and a policy switch | The highest-risk tier needs accountable identity. |
| D8 | WGSL for Community | Now; gated | **Gated** on fuzz, CTS subset and 90 days without device loss | Driver-bug exposure is new. Matches capability doc decision 2. |
| D9 | Gesture-scoped implicit grants | No; yes | **Yes** for `document.*:selection`, `clipboard.write` and `ui.open_url` | Cuts prompts for most plugins (activeTab lesson). Still bounded by scope capture. |
| D10 | TUF and kill-list latency | Keep a single key and 6 h refresh; TUF with hourly timestamp | **TUF before public launch.** Hourly revocation fetch now (a small snapshot file can precede TUF) | Single online key and 6 h lag are the weakest store links. |
| D11 | Client-side provenance | Trust the index; verify Sigstore offline | **Verify offline for native, Verified and Official** | The catalogue key alone should not be able to swap high-value bytes. |
| D12 | Auto-update delay | 0; 24 h; 72 h | **72 h Community, 24 h Verified, 0 Official**, with manual override | The worm-era defence. Gives kill-list time to act. |
| D13 | Untrusted documents | No distinction; Workspace-Trust analogue | **Light analogue:** L2+ plugins do not auto-run on `document.opened` and `document.changed` for an external document until the user acts in it. No global "restricted mode" UI | Blocks drive-by triggers from crafted documents with little UX cost. |
| D14 | Agent access to plugins | Always; opt-in; never | **Opt-in per plugin.** Default on for L0 and L1, off for L2+. Agent-initiated runs are labelled | Limits injection-driven exfiltration through Online plugins. |
| D15 | Guest clock precision | Full; 1 ms; frozen per event | **1 ms quantized**, frozen within a single filter tile | Removes high-resolution timers while keeping animations and timers usable. |
| D16 | New registry commands default | Allowed; refused | **Refused until classified** (CI) | The table is the security boundary. Fail closed. |
| D17 | SoundCraft native plugins | In process; bridge process | **Bridge process.** The main app keeps library validation | Crash isolation, and no `disable-library-validation` in the main app. |
| D18 | Grant table integrity | Plain JSON; HMAC with keychain key | **HMAC, low priority.** Only matters against same-user malware | Cheap, and makes tampering detectable rather than silent. |

---

## 10. Sources

Workspace (read in this session):

- `craft-extensions/docs/plugins/research/capability-program-2026-11-18.md` (tiers, R4, K7, H2–H9,
  benchmarks, decisions)
- `craft-extensions/docs/plugins/native-protocol.md`, `store-and-distribution.md`,
  `suite-replacement-rule.md`, `continuation-2026-11-17.md`, `beta-design.md` §B3
- `craft-extensions/crates/ext-host/src/{wasm.rs,native.rs,policy.rs,api.rs}`
- `craft-extensions/crates/ext-core/src/{permission.rs,tier.rs,review.rs}`, `crates/ext-store/src/state.rs`
- `soundcraft/docs/gaps.md:126`, `soundcraft/crates/{clap-host,vst3-host,au-host}`
- `figcraft/crates/collab/src/change.rs:180`, `figcraft/crates/collab/src/diff.rs:47`

External sources are cited inline at the point of use.

**Confidence.**

- **High:** workspace facts; the Wasmtime model and scope; CVE-2023-26489; the wgpu loop-bounding
  guidance; WebGPU threat list; Landlock ABI list; the Ubuntu bwrap restriction.
- **Medium, from prior knowledge, not re-fetched this session:**
  - CVE-2021-32629 and CVE-2022-39393 specifics;
  - Chrome activeTab and warning behaviour;
  - Android 11 one-time and auto-reset permissions;
  - Felt et al. 17% figure;
  - Bitwig hosting modes;
  - FxPlug 4 XPC;
  - UXP permission names;
  - Windows LPAC version floor;
  - macOS TCC inheritance by child processes;
  - the pnpm and CISA links.
- **Estimates to measure:** worker IPC latency, process memory, `use_csdb` cost, tile budgets.

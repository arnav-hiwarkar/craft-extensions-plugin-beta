# Suite-wide plugin architecture: initial ideas

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Status: initial proposals from one design pass on 2026-11-07. Not an approved specification. The sources, and what could not be read, are in the [resource register](plugin-resources.md). The shared ChatGPT conversation and the Claude artifact were not readable, so nothing here relies on them.

The follow-up [lifecycle review](plugin-lifecycle-review.md) expands these ideas with app-specific evidence, failure paths, performance proposals, and product decisions. Use that review for the current design discussion.

## What exists today (facts)

From the register's pinned evidence:

- **EffectCraft** has WebAssembly pixel effects (ABI version 1). They have no module imports and run with fuel and per-memory limits. It also has a Plugin Library whose catalog lists API versions, download URLs and SHA-256 digests. Updates and removals of loaded effects finish on restart. Native Rust integrations are linked into the host and are not sandboxed.
- **FigCraft** has a manifest describing commands, permissions, panels and widgets. The engine owns permissions, commands, undo and settings, apart from sandbox execution.
- **craft-libs** has a reusable [font manager](https://github.com/storytold/craft-libs/blob/fd5e2feff6ea25bed371a99cc732b3968d621/crates/font-manager/README.md) with an optional embedded panel.

Everything below is a proposal.

## 1. Separate purpose from mechanism

An extension's **purpose** says what it contributes. Its **mechanism** says where its code runs. Both go in one manifest, but they are separate axes.

| Purpose | Typical mechanism |
|---|---|
| Assets and presets (brushes, LUTs, templates) | None: data only |
| Image, video and audio effects | WASM, or GPU compute that the host runs on its behalf |
| Import and export codecs | WASM; a native helper for vendor codecs |
| Commands and automation | WASM calling typed host commands |
| Declarative panels and tools | Host-rendered UI descriptions, with WASM or approved host commands for behavior |
| AI and heavy compute | Host GPU compute, or a native helper |
| Integrations (cloud, devices) | A native helper behind a broker, or a WASM plugin with a network grant |

Data-only packages need no runtime at all. They should be the easiest kind to publish.

## 2. Three execution paths under one contract

**A. Sandboxed WASM (the default).** Host calls expose capabilities outside the module, so their validation is part of the isolation boundary ([WebAssembly security](https://webassembly.org/docs/security/)). Use execution metering and memory limits. Bound host-call work separately. Also keep a **total** budget across all instances, because wasmi limits apply to each memory and not to the whole process ([wasmi `StoreLimitsBuilder`](https://docs.rs/wasmi/latest/wasmi/struct.StoreLimitsBuilder.html)).

**B. Host-mediated GPU compute.** GPU work does not rule out WASM. A WASM plugin can request compute through a narrow host API. Start with approved kernels and bounded inputs. Arbitrary plugin shaders require a separate security decision. Validate shaders, cap allocations and queued work, and check cancellation between dispatches. Do not promise portable interruption of an already-running GPU task. Shader validation alone does not prevent driver bugs or denial of service. Handle device loss and retain the previous frame or use a compatible CPU fallback ([WebGPU security considerations](https://www.w3.org/TR/webgpu/#security-considerations), [device loss](https://www.w3.org/TR/webgpu/#device-loss)). GPU availability and isolation vary by platform.

**C. Native helper processes.** These are for vendor SDKs, hardware accelerators and codecs that cannot be built as WASM. A separate process alone is **not** a security boundary: it can still read the user's files, reach the network and tamper with the app's install. Supporting third-party helpers requires verified OS restrictions on files, network access, child processes and resources on every supported platform. GPU access can complicate those restrictions. Without them, disable the backend or explicitly classify it as trusted and unsandboxed. The app accepts helper requests only through a validated broker protocol. That protocol alone cannot restrict what an unsandboxed helper does elsewhere on the machine.

**Rejected as the default:** loading untrusted native dylibs into the app process. Native Rust integrations that are reviewed and linked in at build time, as EffectCraft has today, stay a first-party option.

## 3. Shared contract and per-app adapters

craft-libs would own the **contract**: manifest schema, catalog, installer, grants and runtime hosts. Each app writes an **adapter** that maps the contract onto its document model.

- **Controlled inputs and outputs.** Plugins get snapshots, such as pixel buffers, layer metadata and selections. Effects return bounded output buffers. Editing plugins return typed commands, which the adapter validates and applies inside an undo transaction. Sandboxed plugins get no private app state, raw host pointers, or write access to the app install.
- **Capabilities are checked when called**, not only when installed. Every host call checks the active grant.
- **Declarative UI** is namespaced by plugin ID and confined to designated extension areas. The host renders it, reserves permission prompts for trusted app UI, and routes callbacks through capability checks.
- **Explicit unsupported states.** An adapter lists the operations it supports. A plugin that needs a missing operation shows as "unsupported in PhotoCraft" or "unsupported on Linux". It must not fail silently.
- **Project references.** A document records the plugin ID, version and a versioned state schema. Opening it without the plugin keeps the data and shows a placeholder.

## 4. Install, update and trust

- **Authenticated catalog.** A SHA-256 digest proves the download matches the catalog. It does not prove who published the catalog, so sign the catalog or its releases.
- **Atomic install with rollback.** Unpack to a staging area, verify, then swap. Keep the previous version until the new one has loaded successfully.
- **Permission escalation.** An update that asks for new capabilities waits for consent. Until then, the old version stays active.
- **Per-app grants.** A grant in FigCraft does not carry over to EffectCraft.
- **Shared store.** If apps share one install directory, each app pins the version it is running. Updates use a lock and copy-on-write version directories, so one app's update cannot swap files that another app has loaded.

## 5. Migration

- EffectCraft: wrap ABI v1 effects as the "effect" purpose on path A. Map Library catalog entries to the shared catalog schema, and **keep the existing plugin IDs** so saved projects still resolve.
- FigCraft: translate its manifest's commands, panels and widgets into the shared manifest, keeping their IDs. Its engine-owned undo and permissions become its adapter.

## 6. First end-to-end slice

Build one path for one app: a minimal shared manifest and installer, together with the EffectCraft adapter, running an existing WASM effect. Preserve current installations and IDs. Defer GPU, native helpers and the FigCraft adapter until this slice works.

Proposed test outcomes:

1. A WASM effect that loops forever is stopped by its budget. The app keeps the previous frame and stays responsive.
2. An update that adds a capability installs in staging but stays inactive until the user grants consent. Declining keeps the old version.
3. A package whose digest does not match, or whose catalog signature is invalid, is rejected, and the installed version is unchanged.
4. A project that references a plugin which is not installed opens, keeps the plugin's state, and shows it as missing.

## Open product decisions

1. **Unsandboxed native helpers.** Are they allowed for third parties, or only first-party and reviewed vendors?
2. **Catalog.** Is there one Storytold-run catalog with signing, or are sideloading and third-party catalogs also supported?
3. **Install store.** Do apps share one plugin store with per-app grants, or does each app keep its own store?

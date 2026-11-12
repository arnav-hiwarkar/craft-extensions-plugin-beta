# Shared plugin system resources

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Captured on 2026-11-07 for the `feat/artcraft-plugin-system` worktree. This is a source register and discussion record, not an approved implementation specification.

## Supplied resources

- [Shared ChatGPT conversation](https://chatgpt.com/share/6aeeff72-c490-83e8-8228-139f6225c5db). Web retrieval timed out. A direct HTTP request returned only the generic share-page shell, with no conversation text. The linked discussion has not been reviewed or archived. An exported transcript is needed to incorporate its contents.
- [Claude artifact](https://claude.ai/artifact/4AyzLFyPewZYWhWBx6DZ6V). Anonymous web retrieval returned only a JavaScript frame shell; the discovered public content endpoint returned HTTP 403 on retry. On 2026-11-08 the user pasted its Craft Extensions To-Do text. Its contents were reviewed and summarized in [Jeremy's handoff summary](jeremy-extensions-handoff-summary.md), then checked against the actual referenced repository branches. The visual chart and underlying artifact source were not retrieved.
- [storytold/craft-libs](https://github.com/storytold/craft-libs). Accessible. This worktree contains the repository at `fd5e2feff6ea25bed371a99cc732b3968d621505`. The [README at that revision](https://github.com/storytold/craft-libs/blob/fd5e2feff6ea25bed371a99cc732b3968d621505/README.md) documents app consumption through Git dependencies pinned to commits. The [workspace manifest](../Cargo.toml) defines the shared crates. The supplied URL points to a moving branch, not a frozen snapshot.

The ChatGPT entry remains a reference only. The Claude handoff is now available through the user's pasted text and independently inspected repository sources. Earlier documents that say it was unavailable describe the state before that text arrived.

## User requirements recorded in this discussion

- Share plugin metadata, installation, updates, permissions, and management across ArtCraft apps.
- Connect each app through an adapter that exposes its supported operations.
- Cover different extension types and explain what each can do.
- Consider resource-intensive and GPU workloads as well as WebAssembly.
- Protect the host application while allowing plugins to add functionality.
- Produce a few initial ideas with one bounded Opus design pass. Implementation is not authorized by this research request.
- The follow-up requests a deep review of every lifecycle stage, proposed solutions, and explicit product decisions. See [the lifecycle review](plugin-lifecycle-review.md). This expands the initial research scope without authorizing implementation.

For discussion, protecting the application means preventing plugins from patching its executable, replacing built-in behavior through private hooks, or bypassing validated editor operations. Plugins may contribute supported commands, effects, and panels. Whether that interpretation matches the intended product policy remains open.

## Existing implementation evidence

EffectCraft was inspected at `ce9eaec65015a5508c72af0e2351cd02612c9450`:

- [Plugin author and execution contract](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/docs/plugins.md). WebAssembly effects receive pixels, parameters, layer time, and preview scale. The WebAssembly ABI is version 1. Native Rust integrations have a broader API and are linked into the host rather than treated as sandboxed third-party modules.
- [Plugin Library](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/engine/src/system/library.rs). Catalog releases identify API versions, download URLs, and SHA-256 digests. Installation records and startup checks already exist. Loaded effects require restart to finish updates or removal.
- [WebAssembly runtime](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/plugin/src/wasm.rs). Rejects module imports and enables instruction fuel and linear-memory limits. These checks are not proof of complete host isolation or a total application memory bound.
- [Effect registration and rendering](https://github.com/storytold/effectcraft/blob/ce9eaec65015a5508c72af0e2351cd02612c9450/crates/effects/src/plugin.rs). Registers effects in the app registry, translates parameter definitions, invokes rendering, and returns the previous frame when plugin rendering reports an error.

FigCraft was inspected at `0e86f3597a7443b15c93ec3ab2091202f8802c95`:

- [Plugin boundary](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/crates/plugin/src/lib.rs). Separates sandbox execution from engine-owned permissions, commands, undo, and settings.
- [Manifest](https://github.com/storytold/figcraft/blob/0e86f3597a7443b15c93ec3ab2091202f8802c95/crates/plugin/src/manifest.rs). Describes commands, permissions, panels, and widgets. This is a different extension contract from EffectCraft's pixel effects.

The shared [font manager](https://github.com/storytold/craft-libs/blob/fd5e2feff6ea25bed371a99cc732b3968d621/crates/font-manager/README.md) is an existing example of reusable functionality with an optional host-embedded panel.

## Primary security references

- [WebAssembly security model](https://webassembly.org/docs/security/). Host-exposed APIs are part of the isolation boundary.
- [wasmi execution configuration](https://docs.rs/wasmi/latest/wasmi/struct.Config.html). Describes fuel metering and execution limits. This is moving documentation, not the version pinned by an app.
- [wasmi store limits](https://docs.rs/wasmi/latest/wasmi/struct.StoreLimitsBuilder.html). Linear-memory size limits apply per memory. They do not establish a total host-process budget.
- [WebGPU security considerations](https://www.w3.org/TR/webgpu/#security-considerations) and [device loss](https://www.w3.org/TR/webgpu/#device-loss). Reviewed during synthesis. The specification discusses GPU resource exhaustion, driver bugs, watchdogs, and device-loss reporting. It does not establish portable cancellation of arbitrary running GPU work.

## Follow-up

On 2026-11-08, the user chose sandboxed plugins initially, official and third-party distribution, and FigCraft as the second integration. The [decision record](plugin-functional-decisions.md) and [combined functional model](plugin-functional-model.md) record that discussion. The user then accepted the functional defaults and requested the current [security and distribution proposal](plugin-security-design.md). Its baseline is Jeremy's existing craft-extensions work, not a second implementation in craft-libs. Themes only are in scope, not organization/team management.

Read [the initial architecture ideas](plugin-architecture-ideas.md) and [the full lifecycle review](plugin-lifecycle-review.md) as earlier proposals. The ChatGPT conversation is still unavailable. Only the decision record establishes accepted choices. No plugin runtime or application code is changed by these notes.

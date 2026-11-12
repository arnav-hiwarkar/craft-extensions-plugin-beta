# Lifecycle design review

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Historical discussion archive, captured on 2026-11-08. This preserves technical requests and substantive responses; routine progress updates and tool logs are omitted. Local checkout links are normalized. Later decisions and source corrections take precedence. Start with [the current documentation index](../README.md).

## User request

i need you to do a deep review on every point and workflow through the entire process of plugins and how they should work and give me each point and path, and think of solutions and processes to make it optimzed and best for plugoins, tell me and explain every point and ask for thoughts and decisions

## Research response 1

The design I recommend is **one shared management system, several versioned extension interfaces, and controlled execution paths chosen for the workload**. Each app connects those interfaces to its own editor.

I saved the full [20-stage lifecycle review](../../../docs/plugin-lifecycle-review.md), including recommendations, failure paths, tests, and a decision prompt for each stage. These remain proposals.

I inspected EffectCraft, PhotoCraft, VectorCraft, and FigCraft. Their existing contracts differ substantially, so sharing everything through one universal “run plugin” function would lose important behavior.

Here is the process I would design.

1. **Define exactly what plugins may change.**

   Plugins should add effects, commands, tools, panels, and supported formats. They may change documents through approved editor operations.

   They should not modify the app installation, patch built-in implementations, access private memory, change security settings, or silently install other plugins.

   Permission to edit a document still permits unwanted edits. We need previews, scoped access, validation, and undo alongside sandboxing.

2. **Classify what a plugin contributes separately from how it runs.**

   The main purposes are assets and presets, image filters, video effects, audio processors, geometry tools, importers and exporters, automation, panels and widgets, AI processing, and external integrations.

   Those purposes need different contracts. An audio processor needs sample rate and latency information. An image filter needs colour and pixel information. Both can use the same installer without sharing the same execution interface.

   A package could contain several contributions.

3. **Give packages and contributions stable identities.**

   A package needs its identity, publisher, version, licence, supported apps, capabilities, modules, and assets. Each effect or command also needs a stable identity.

   A plugin’s name is unsuitable for identity because names change and can be copied. Existing EffectCraft effect IDs must survive migration so old projects continue resolving them.

   Package versions, execution-interface versions, and saved-state versions must be separate.

4. **Make authoring practical through a public SDK.**

   Authors should get templates, documented interfaces, a package validator, and a testing harness that uses the production runtime.

   An EffectCraft author should implement the rendering calculation and declare its controls. The app should supply keyframes, parameter evaluation, effect menus, and project integration.

   Development loading should be clearly marked and session-scoped. It must not quietly weaken checks for permanently installed plugins.

5. **Publish immutable releases with authenticated metadata.**

   An author publishes an artifact and source reference. Automated checks validate its structure, compatibility, permissions, and basic behavior. Human review evaluates the contribution and its requested access.

   Checksums verify that downloaded bytes match catalog metadata. We also need to authenticate the metadata and handle key rotation, expired metadata, and replayed releases. These are established update-security problems covered by [TUF](https://theupdateframework.github.io/specification/latest/).

   My recommendation is an official catalog plus explicit local sideloading initially.

6. **Check compatibility before installation or execution.**

   The manager should check the app, interface version, platform, processor architecture, supported features, and runtime requirements.

   “Compatible,” “installed,” “enabled,” and “permitted” are different conditions. A plugin can be installed but unavailable in one app.

   Browsing should use metadata without running the plugin. Unsupported packages should explain the missing requirement rather than fail after the user installs them.

7. **Treat preview as real plugin execution.**

   Preview still runs potentially hostile code. It needs the same isolation and resource controls as ordinary use.

   Browsing previews should use synthetic sample data. Users can explicitly choose to preview against a snapshot of their selection.

   When a slider changes, cancel obsolete previews. If the document changes, discard stale results rather than applying them to a different selection.

8. **Make installation recoverable at every step.**

   Download into staging, verify the artifact, inspect its contents, validate the package, publish an immutable version, then commit the installation record.

   Reject path traversal, dangerous links, duplicate filenames, excessive expansion, and oversized contents. Avoid privileged installation scripts.

   If the app closes halfway through, retrying should safely finish or clean up staging. The previous working installation must remain usable.

9. **Share package files while keeping app decisions separate.**

   My preferred model is shared immutable downloads, with separate activation choices, grants, and preferences for each app.

   Installing a package once could make it available to several apps. Allowing it to read documents in FigCraft must not grant the same access in EffectCraft.

   Running apps retain their selected artifact. An update cannot overwrite files another app is using. Portable installations may need a separate store.

10. **Enforce permissions whenever capabilities are used.**

    Installation consent alone is insufficient. Every host operation must verify the running plugin’s identity, grants, document scope, and current policy.

    Plugins should receive specific operations such as reading the selected objects or exporting to a chosen destination. They should not inherit the complete CLI or MCP command catalog.

    Nested commands must retain these checks. Otherwise an innocent-looking wrapper could invoke a forbidden operation indirectly.

11. **Broker access to files, networking, accounts, and devices.**

    The app should control file selection and final writes. Plugins receive scoped handles or data rather than unrestricted filesystem access.

    Network access needs destination restrictions, request limits, cancellation, and explicit disclosure of document uploads. Account credentials should remain with the credential owner wherever possible.

    Document access combined with networking creates an exfiltration path. An upload also cannot be undone like a layer edit.

12. **Activate lazily and keep lifecycle states explicit.**

    Installing a plugin should not automatically run it at every startup.

    Load code when its functionality is needed, initialize within limits, and register contributions only after validation succeeds. Keep initialization off the UI thread.

    A newer installed version can coexist with an older running version. “Update staged; restart required” is a valid state. EffectCraft already requires restart for loaded-effect updates.

13. **Apply edits through transactions and undo.**

    Capture the relevant document state, run the plugin, validate its output, then commit through the editor’s existing transaction system.

    If the document changes during computation, reject or recompute the result. Do not apply stale output blindly.

    A user-invoked command or gesture should produce coherent undo. Bound object counts and transaction size, and prevent plugin-generated changes from triggering endless callback loops.

14. **Preserve each creative operation’s semantics.**

    PhotoCraft currently processes image bands in the document’s colour model, with overlap and selection rules. EffectCraft passes time-aware, premultiplied RGBA frames. VectorCraft exchanges geometry and object data.

    Those differences matter. A shared image interface must define colour, alpha, units, bounds, and neighbourhood requirements explicitly.

    Preview may visibly bypass a failed effect. Final export should report failure or require explicit acceptance, rather than silently produce different artwork.

15. **Optimize transfers, compilation, and scheduling before changing runtimes.**

    Compile verified modules once where safe, bound caches, transfer large media through binary buffers, and process bounded tiles or bands.

    Count all memory: inputs, outputs, copies, rollback buffers, compiled modules, and concurrent instances. Per-plugin limits alone can still exhaust the app.

    Prioritize interactive work and cancel obsolete jobs. Measure startup, processing, transfer overhead, memory, and cancellation latency before claiming native or GPU execution is faster.

16. **Support GPU and native execution through distinct controls.**

    WASM can request GPU work through the host. GPU access does not inherently require a native plugin.

    I recommend starting with approved host compute operations. Arbitrary shaders create additional risks, including resource exhaustion and driver failures. We cannot promise portable interruption of an already-running GPU task. [WebGPU’s security considerations](https://www.w3.org/TR/webgpu/#security-considerations) describe these concerns.

    Native helpers need actual OS restrictions. A separate process alone still has the user’s authority unless restricted. Disable unsupported secure paths rather than silently reducing isolation.

17. **Save projects with explicit plugin dependencies.**

    Save the contribution identity, required release, parameters, and versioned instance state. Keep credentials and ordinary user preferences outside the project.

    Missing plugins should produce placeholders while preserving their settings. Opening someone’s project must not automatically download and trust executable code.

    Collaborators need matching dependencies or an explicitly baked representation. Upgrading a project’s plugin version should be a visible operation because rendered output may change.

18. **Update without altering active work.**

    Validate the candidate separately, compare permissions, migrate state on a copy, and retain the working version.

    New capabilities require renewed consent. A running render stays pinned to its original version.

    Code rollback and state rollback must work together. Returning to old code is insufficient if its saved state has already been destructively migrated.

    My initial recommendation is explicit activation of updates at a safe boundary or restart.

19. **Separate disabling, uninstalling, and deleting data.**

    Disabling stops new entry points and callbacks. Removing activation in one app should not necessarily delete shared package files.

    Uninstalling should preserve project references and retain settings by default, with a separate data-removal choice.

    Repeated startup failures should offer safe mode. Diagnostics need plugin identity and useful errors, but should exclude document contents and secrets by default.

20. **Prove the shared design across different apps before stabilizing it.**

    Start with EffectCraft’s existing WASM effects and package-management behavior, preserving IDs and existing projects.

    Then integrate a different workload. FigCraft would test commands, transactions, and panels. PhotoCraft would test image formats, selection, and bounded processing.

    `craft-libs` should own common management and execution machinery. Each app should retain its document model, rendering semantics, and editor policy. We should call the contract stable only after those differences work through it.

The most important tests follow those workflows: interrupted installation preserves the working version; denied operations remain denied through nested commands; failed or stale edits leave documents unchanged; missing plugins preserve saved state; updates cannot change active renders; and failed plugins cannot prevent recovery.

This was a source and design review. I checked the documentation’s local links and whitespace. I did not run the apps, benchmark execution, or validate platform sandboxes.

I recommend starting our discussion with these three decisions, because they shape the remaining choices:

- **Protection:** should third-party executable plugins always require enforced sandboxing, or should there also be an explicitly trusted native mode? I recommend sandboxed third-party code initially.
- **Distribution:** official catalog only, official catalog plus local sideloading, or third-party catalogs immediately? I recommend official plus sideloading.
- **Second integration:** after EffectCraft, should we prove FigCraft’s commands and panels or PhotoCraft’s image filters? I lean toward FigCraft because it tests a substantially different interaction model.

What are your thoughts on those defaults, and which parts of the lifecycle would you change?


# Runtime security, themes, and application requirements

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Status: proposed design, 2026-11-08. This is a source review, not a security certification. No app, test suite, fuzz target, or build ran. The [decision record](plugin-functional-decisions.md) contains accepted choices. The [Jeremy handoff summary](jeremy-extensions-handoff-summary.md) identifies existing implementation that supersedes any assumption that package infrastructure starts from nothing here.

## Verified source locations

- `storytold/craft-extensions`, handover revision `ffd8512e5a3755637f58cf31cfcf2fdb09cfb943`: [manifest](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-core/src/manifest.rs), [theme palette](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-core/src/palette.rs), [store](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-store/src/store.rs), [reader](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/crates/ext-store/src/reader.rs), and [threat model](https://github.com/storytold/craft-extensions/blob/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943/docs/THREAT-MODEL.md).
- `storytold/photocraft`, theme-adapter revision `ecb056cc742848d78f9076893a6b3343e6a75010`: [theme mapping and store watcher](https://github.com/storytold/photocraft/blob/ecb056cc742848d78f9076893a6b3343e6a75010/crates/ui-egui/src/ext_theme.rs). These are branch-specific definitions, not claims about PhotoCraft main.
- `storytold/craft-launcher`, feature revision `6ede77f93fd0d3eaaad43d430f94d703d3564a51`: [store-backed UI source](https://github.com/storytold/craft-launcher/blob/6ede77f93fd0d3eaaad43d430f94d703d3564a51/apps/artcraft-launcher/src/extensions.rs). The combined UI branch is `be3bff3e2683df0b73e3bbbc09351b22dc86a800`, a showcase source excluded from release by the handoff.
- This craft-libs worktree is at `fd5e2feff6ea25bed371a99cc732b3968d621505`. Upstream main was fetched at `058b047760dfe5ba51f6f526faa3b0c833552dfa`. The [widget theme module](https://github.com/storytold/craft-libs/blob/fd5e2feff6ea25bed371a99cc732b3968d621/crates/ui/src/theme/mod.rs) and [theme comparison](https://github.com/storytold/craft-libs/blob/fd5e2feff6ea25bed371a99cc732b3968d621/docs/theme-infrastructure.md) have no differences between those revisions. No working branch was rebased or merged.
- FilmCraft's sampled checkout is `32749e34f611`; [appearance settings](https://github.com/storytold/filmcraft/blob/32749e34f611/crates/engine/src/settings/prefs.rs) and [theme tokens](https://github.com/storytold/filmcraft/blob/32749e34f611/crates/ui-egui/src/theme.rs) remain app-specific. Existing plugin execution was not established by this study.

## Two existing theme layers, not competing replacements

`craft-ui::theme::WidgetPalette` applies five egui interaction-state fills, a radius, and a foreground stroke. It is deliberately not a complete user-facing theme schema. Application palettes, preferences, borders, spacing, and layout remain app-owned.

`craft-ext-core::palette::ThemeFile` is an extension data format: a small colour palette plus app-specific colour overrides. `craft-ext-store` installs it and its reader selects the active theme for an app. The PhotoCraft adapter maps it onto PhotoCraft tokens, checks mapped contrast pairs, drops bad overrides, and falls back to its built-in appearance when needed. Its watcher notices store generation changes without needing the launcher to run.

Recommendation: preserve that separation. Theme packages are data, the store chooses them, each app maps and validates them, and craft-ui provides drawing mechanisms. Do not replace the package palette with egui widget fields, or make one app's full token struct mandatory across the suite.

For additional apps, including FilmCraft, add explicit mappings for only supported colour tokens. Preserve timeline and selection semantics, user contrast settings, units, typography, and layout. Theme activation must remain a deliberate appearance choice; installing a package must not unexpectedly restyle the app.

Theme validation does not certify every pixel or accessibility property. The package parser's checks cannot know every app-specific surface. Test mapped states and fallback paths in each app. Permission and publisher identity UI remains host-owned and cannot be replaced by a plugin panel. Retain readable security UI even when a theme is broken or misleading.

## Current v1 cannot represent the entire accepted functional model

The inspected manifest uses `craft-extension.json`, schema 1, a single kind, `theme.json` or `plugin.wasm`, per-app compatibility, file sizes/digests, and empty permissions. A plugin can carry an optional fixed-path thumbnail. This is an existing useful ZIP envelope, not an SDK or a capability-rich multi-contribution contract yet.

Do not add security-critical meanings as fields an old parser silently ignores. Introduce a new schema or explicitly negotiated interface for multiple contributions, granted host calls, richer assets, and saved-state references. Older hosts must refuse executable contracts they cannot enforce.

The pure-computation v1 path can remain valid. FigCraft command/panel plugins require host imports and therefore need a separate versioned capability path. A successful no-import filter pilot is not evidence that arbitrary editor commands are safe.

## Protection is enforced at several boundaries

The assets are user documents and media, application installations, credentials, account actions, render correctness, and available compute. Attackers can supply packages, update metadata, malformed assets, modules, and misleading output. A compromised trusted source is also in scope.

Use the same runtime restrictions for ArtCraft-produced, proprietary, and community executable contributions. Provenance labels affect discovery and review policy; they do not create unrestricted authority. First-party does not necessarily mean paid, and proprietary does not imply a different runtime ABI.

The host must know the running instance's identity. Effective authority is constrained by host policy, its supported contract, requested capabilities, actual grants, app/document scope, and current eligibility. A plugin must not choose another identity in a request.

Check capability and parameter-dependent behavior at the host call, including nested commands. An apparently harmless command can load a filename or upload output through parameters. Do not expose every operation merely because a desktop CLI can invoke it.

Document edits use the app's transaction and undo system. Bounded output validation happens before commit. Background results carry a document revision so stale results cannot overwrite newer edits. External uploads and file writes are separate side effects; undo cannot retract them.

## Runtime and resource limits

Before initialization, validate module size, imports/exports, contract version, and required resources. Revalidate the actual loaded bytes, not only metadata recorded when installed. File digests inside an unsigned manifest detect mismatch, not publisher authenticity.

Meter module execution and bound memory, stack/recursion, compilation, host calls, logs, callback queues, output objects, image dimensions, and total concurrent usage. Frame copies and rollback buffers consume host memory too. Maintain separate interactive and background budgets.

Host functions need their own cancellation, deadlines, and size limits. Fuel cannot interrupt a host call that is itself blocked; [Wasmtime's host-call discussion](https://docs.wasmtime.dev/api/wasmtime/struct.Config.html#method.epoch_interruption) illustrates this boundary, without implying the apps use Wasmtime or that a runtime migration is selected.

Treat GPU access as a host-owned compute contract initially. Bound uploads, resources, queued work, and operation parameters. Device loss and failure must preserve document state and produce visible output errors. Arbitrary shaders and native SDK workers require additional evidence. [WebGPU's security considerations](https://www.w3.org/TR/webgpu/#security-considerations) include driver bugs and resource exhaustion; GPU memory safety alone does not establish availability or portable cancellation.

Sandboxed native workers are an optional future backend, not an automatic exception. A helper process that can still access unrestricted files or credentials does not satisfy the accepted protection policy.

## CLI default without extra privileges

The user's chosen default is to make compatible installed plugins useful in new CLI sessions. Resolve only packages that are installed, enabled for that app, supported by that host, and permitted by existing grants and applicable policy. A project reference is a dependency request, not an installation or permission grant.

Load needed contributions on demand after metadata resolution. Do not run every startup callback just because the package exists. Missing resources, missing access, revocation, or incompatible state produce actionable errors. Do not fetch new code, approve wider rights, open a hidden permission dialog, or silently attach to another session.

Use the same invocation and export checks in UI, CLI, automation, and persistent servers. An isolated-environment option remains useful for reproducible builds. Final export must not silently bypass unavailable processing even if preview can display a fallback.

## Store and installation work still needed

Existing source streams archive contents with size/hash checks, rejects unsafe names and links, stages installation, uses a cross-process mutation lock, and replaces state atomically. Those are real implemented mechanisms, not proof of complete adversarial correctness.

Admission must also bound work before archive indexing. [craft-libs issue 126](https://github.com/storytold/craft-libs/issues/126) specifically identifies the difference between checking entry count after `ZipArchive::new` and preflighting allocation first. Its proposed archive helper must be evaluated on its actual merged/available state before selecting a dependency. Do not label an unmerged proposal an existing library.

The current store auto-enables newly installed packages for their compatible apps and deletes the previous package directory after an update. That is insufficient for the broader accepted model of independent grants, active render/version pins, and state rollback. Extend activation and retention deliberately before claiming safe hot updates.

The read side's fallback to absent data is suitable for a theme that can use a built-in appearance. Executable project dependencies need visible missing/corrupt diagnostics. An empty list must not silently turn a final render into a different result.

## PhotoCraft and FilmCraft requirements

PhotoCraft has a real branch for theme adaptation and existing mainline WASM filter execution, but the handoff's package-store-to-filter loading path remains a separate integration task. Preserve pixel mode, alpha, selection, processing overlap, and smart/destructive filter behavior. Check package/interface compatibility at the host boundary. A theme round trip does not certify plugin execution.

FilmCraft needs its own package reader and theme mapping, plus explicitly designed effect, transition, command, or resource interfaces. Its timeline, media pipeline, audio context, cache invalidation, job completion, and export behavior stay app-owned. No existing generic filter slot was proved by this study, so do not describe FilmCraft integration as a wiring-only change.

EffectCraft/FigCraft remain the chosen functional validation pair. Jeremy's existing PhotoCraft theme/filter pilot is complementary evidence to reuse. The sequencing of those workstreams needs a design agreement, not a silent replacement.

## Contributor burden

Keep mandatory author fields small: stable IDs, release version, contribution contracts, required capabilities, licence metadata, and resource declarations. The SDK should derive sizes, digests, file inventories, and schema/export checks. The source/catalogue workflow supplies authenticated publisher and release metadata. Never require authors to hand-edit signature JSON or copy unsafe host glue.

Use the existing ZIP envelope and manifest as compatibility evidence. Design its successor with migration fixtures and rejection tests before calling it stable. SDK commands and file extensions remain unselected proposals.

## Acceptance and next decisions

Before release, prove adversarial archive admission, bad signature/key/expiry handling, install interruption, cross-app source/identity continuity, actual module revalidation, denied host calls, bounded processing, safe disable/recovery, and GUI/CLI export parity using real packages. Review theme mappings and trusted prompts visually in the candidate build. No such runtime or UI checks were performed here.

Outstanding product choices include package schema evolution, exact capability classes, default offline and entitlement behavior if licensing is added, catalogue administration, and reconciling the existing first-party PhotoCraft pilot with the accepted EffectCraft/FigCraft functional sequence. Third-party release admission stays gated while these are designed and verified.

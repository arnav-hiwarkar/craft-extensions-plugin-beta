# Craft Extensions security and distribution proposal

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Status: design discussion, 2026-11-08. No app or shared-library implementation, release, signing service, catalogue, key, push, or PR is authorized by this note. Organization/team management is out of scope: the user clarified themes only.

This combines the [runtime/theme investigation](plugin-security-runtime-themes.md), [Opus review of the existing handoff](plugin-security-handoff-opus.md), and [summary of Jeremy's supplied artifact text](jeremy-extensions-handoff-summary.md). Opus used model 5.5 at medium effort. The earlier generic distribution note predates the handoff and is not the current architecture baseline.

## 1. Accepted functional constraints

Sandboxed executable contributions initially; official and third-party distribution; EffectCraft and then FigCraft as functional validation apps; existing ABIs through adapters; headless editing/compute operations; multiple contributions per package; compatible installed plugins available automatically in fresh CLI sessions; executable work first while preserving an extensible format; a stable SDK with a small required author-written surface.

Theme packages and PhotoCraft/FilmCraft requirements are included. Each application adapter remains separate implementation work. These choices do not constitute acceptance of every mechanism proposed below.

## 2. Jeremy's artifact identifies existing implementation

The supplied text describes a shared Extensions browser, with Themes, Plug-ins and Resources, in Craft Launcher and reachable from every app. It is not a finished general-purpose plugin manager. The source is already in `storytold/craft-extensions`, and the launcher and PhotoCraft have integration branches.

Source refs were fetched and inspected without checking out, merging, or editing those repositories:

| Existing work | Verified source | What the inspected source establishes |
|---|---|---|
| Contract/store and handoff | [craft-extensions `ffd8512e`](https://github.com/storytold/craft-extensions/tree/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943) | `craft-ext-core`, `craft-ext-store`, manifests, palettes, bounded unpacking, store state, reader and draft threat model |
| Launcher feature | [craft-launcher `6ede77f9`](https://github.com/storytold/craft-launcher/tree/6ede77f93fd0d3eaaad43d430f94d703d3564a51) | Extension UI backed by the local store; no completed catalogue download path established |
| Combined showcase UI | [craft-launcher `be3bff3e`](https://github.com/storytold/craft-launcher/tree/be3bff3e2683df0b73e3bbbc09351b22dc86a800) | Named by the handoff as the approved visual source, with AI work mixed in; never the release source |
| PhotoCraft theme adapter | [photocraft `ecb056cc`](https://github.com/storytold/photocraft/tree/ecb056cc742848d78f9076893a6b3343e6a75010) | Store watcher, token mapping, contrast checks, fallback, sibling path dependencies |

The handoff reports no real catalogue/install/plugin-host round trip, no candidate release package, and no clean-checkout acceptance for the integrated candidate. Historical test counts remain historical reports, not checks rerun here.

Recommendation: extend and review this implementation. Do not create a duplicate installer, manifest, or store in craft-libs. The handoff deliberately kept craft-extensions standalone and left later promotion open. Any move into craft-libs needs concrete consumers and an agreed boundary.

## 3. Security state is partial

Built in the inspected source: bounded archive reads with name/link/hash checks, validated manifests and display IDs, staged installation, locked/atomic state updates, store generations, theme contrast validation, reserved first-party namespaces, and a default-off reviewed-third-party feature.

Incomplete: signature verification and key management, authenticated catalogue installation, signed metadata freshness, enforced revocations, per-load executable validation, capability-aware hosts, and independent security acceptance. `Index::parse` checks expiry and refuses older versions, but it is not a signature verifier. An existing `is_revoked` helper does not itself disable a package in every app.

The release guard specifically excludes `ReviewedThirdParty`. `FromFile` is admitted. File packages can contain externally authored modules, so they need the same mandatory module admission and runtime checks. None of these source observations demonstrates exploitability or certifies safety.

The original sg1 and sg5 definitions are present in the threat model and plan: namespace protections, and a feature guard with release-profile verification still required. A test of a crate's default-feature text does not prove the effective dependency graph of every release candidate.

## 4. Keep identity, authenticity, execution and licensing separate

Every source supplies an identity and an authenticated mapping from publisher/package to exact release bytes. A package's display name or publisher string is a claim. A digest detects mismatch; it does not prove authorship.

The host assigns trust labels from verified source provenance. External catalogue fields must not become privileged tier values directly. A validated-artifact interface can bind the source, publisher, namespace and digest before installation. This prevents integration mistakes, not malicious trusted-host code from exercising its existing authority.

Official, proprietary, and community modules use the same runtime contracts and capability checks. First-party status permits appropriate identity labels and namespace ownership. It does not grant arbitrary files, networking, accounts, private memory, or host command access.

Proprietary first-party packages are not necessarily paid. If restricted downloads or licences are needed, add entitlement policy separately. Credentials stay outside package files, projects, logs, and render parameters. Client-delivered code should not be described as secret merely because it is encrypted in transit or at rest. Server-only logic has different availability and privacy tradeoffs and is not automatically required.

## 5. Distribution paths and updates

Support the official source, explicitly added third-party sources, and local development files through a common package validator. Record source/publisher continuity, exact artifact identity, and the selected version. The same bare ID on a different source is not an automatic replacement.

Authenticate official metadata too. Use an established update protocol and vetted implementation for key rotation, freshness, rollback/freeze protection, and recovery; do not implement cryptography from a sketch. [TUF](https://theupdateframework.github.io/specification/latest/) defines these mechanisms. Its use is a proposal, and no crypto dependency was added.

Same-version changed metadata must not evade continuity checks. Publisher-key changes, requested authority increases, and manual downgrades require explicit handling. An unsigned development file can remain unverified; its manifest hash must not be presented as proof of its author's identity.

The install path verifies source metadata and the archive digest before extracting, validates all entries and modules, then publishes immutable bytes and commits state. Updates stage independently. Retain versions and state needed by active jobs and rollback. The current store's single-current-version replacement and deletion of its old directory need extension before claiming this stronger behavior.

The existing download-host restriction is an additional admission rule, not a substitute for signature verification. A third-party source must not gain first-party identity by hosting files under an allowed URL. Catalogue administration and download policy remain decisions.

## 6. Runtime capabilities and CLI eligibility

The existing schema 1 requires empty permissions and no host-import capability. Preserve that contract for its pure-computation path. FigCraft commands and panels need host calls. They require a versioned capability contract with host enforcement, not an exemption for official plugins or permissive interpretation by old clients.

Verify modules at install and when loading the bytes actually used. Bound initialization, execution, stack and memory, compilation, host-call work, output, logs, resources, events and concurrent jobs. Validate edits before committing through the app's transaction system. Fuel does not bound a blocking host operation; the host needs its own limits and cancellation. [Wasmtime documentation](https://docs.wasmtime.dev/api/wasmtime/struct.Config.html#method.epoch_interruption) describes that general embedding boundary; it is not a decision to change runtimes.

The accepted fresh-CLI default means resolving installed, enabled, compatible, permitted contributions under the existing policy. It does not authorize fresh downloads, widened rights, automatic approval, or ignored revocations. Missing requirements produce useful structured errors. Final exports check dependencies instead of silently dropping processing.

GPU operations should initially be host-mediated and bounded. Native helpers and plugin shaders remain gated backends until enforcement and recovery are demonstrated. Separate processes and validated shaders do not by themselves establish the full protection promise. [WebGPU security considerations](https://www.w3.org/TR/webgpu/#security-considerations) explicitly cover availability and driver risks.

## 7. Themes and resource packages

Use data-only theme packages with an explicit colour schema, bounded input, known app overrides, and no executable install hooks or remote assets hidden in colour values. Installing a theme does not mean choosing it as the active appearance.

The existing PhotoCraft branch supplies a useful pattern: map the shared palette to app tokens, validate the mapped combinations, discard unreadable overrides, and fall back to the built-in appearance. These checks do not certify whole-app accessibility. FilmCraft needs its own token mapping that preserves timeline selection, contrast settings, and visual semantics.

craft-libs' `WidgetPalette` remains a drawing mechanism. craft-ext-core's palette is the package data contract. Both can coexist without inventing one universal egui theme struct.

Resources remain typed data consumed by existing importers. A brush, LUT, font, SVG, image thumbnail, or workflow file can still attack a parser or exhaust resources. Admission and decoder limits are needed even when the package has no executable module. Workflow files must not execute on install.

## 8. SDK and package stability

Reuse the existing ZIP/`craft-extension.json` envelope as a migration baseline. The current schema supports one kind and a constrained module/thumbnail layout; it does not implement the accepted multi-contribution design yet.

The next schema must explicitly negotiate contribution and capability versions. Security-critical new fields cannot be silently ignored by old hosts. Unknown optional presentation metadata may be handled more leniently.

Authors should specify stable IDs, release version, intended contracts, required capabilities and resources, and required licence metadata. SDK tooling can derive file sizes/digests, validate exports, and generate boilerplate. Source/catalogue tooling provides authenticated publisher and release metadata. Contributors should not have to assemble signature structures manually.

The package schema, executable SDK and catalog authentication serve different jobs. Stabilize them with real packages and app adapters before publishing compatibility promises.

## 9. Offline use, revocation, and recovery

Proposed default: keep previously verified, installed, eligible packages usable offline; require fresh verified metadata for new installs and updates. Expired update metadata and expired product entitlement are different policies. An optional paid licensing system needs its own agreed offline behavior.

Apply cached revocations when resolving eligibility. Offline clients cannot know about unseen revocations. On receiving a revocation, stop new use according to policy, preserve projects/state, and clearly explain unavailable contributions. A theme can safely fall back to built-in colours; a final render must not silently omit a revoked effect. Handling already-running jobs needs explicit contract-specific semantics.

Recover from interrupted installs, invalid metadata, startup failures, and failed migrations without destroying the last working project state. Provide safe startup without executable plugins. Diagnostics record package/version/digest and failure context while excluding document content and credentials by default.

## 10. Reconcile the pilots and release gates

Jeremy's theme/PhotoCraft/invert pilot is existing work to finish and reuse. EffectCraft then FigCraft remains the user's chosen sequence for validating the broader functional host model. They test different things. Proposed order: reconcile the package/store baseline, complete a real existing first-party package round trip, then validate expanded contracts with the chosen apps. The precise cross-repo delivery order is still open.

Keep the existing third-party release gate closed until the defined checks and required approvals are satisfied. Supporting third-party architecture now is not approval to enable it in release builds. The combined launcher showcase stays excluded from release source; pin dependencies and extract approved UI into a clean candidate after the integration gate.

The minimum evidence includes a real package install/enable/run/reopen/remove cycle, clean-checkout dependency resolution, candidate release-feature behavior, hostile archive admission, module rejection, denied-call tests, signature/key/freshness/revocation cases, state rollback, GUI/CLI parity and independent security review. No such runtime checks ran in this documentation task.

## 11. Live context checked

- [craft-libs issue 94](https://github.com/storytold/craft-libs/issues/94) is closed. Its closing comment rejected the extra tiers/process and inaccurate inventory; it is not a mandate to build that architecture.
- [Issue 115](https://github.com/storytold/craft-libs/issues/115) remains a candidate inventory, not implementation evidence.
- [Issue 79](https://github.com/storytold/craft-libs/issues/79) is open and proposes governance changes; current repository rules remain authoritative here.
- [Issue 126](https://github.com/storytold/craft-libs/issues/126) is open; its comment points to archive-preflight PR 129. Checking entry counts after archive indexing does not bound the indexing allocation. Availability and adoption of that helper need their own verification before selection.
- [crafthub README](https://github.com/storytold/crafthub/blob/main/README.md) still says "Placeholder for merges". That does not establish approval to use it as the extension catalogue.
- The shared AI-store decision is recorded in the handoff's decided d5. This pass did not independently verify the underlying AI decision thread, so it does not reopen or strengthen that claim.

## Decisions to discuss next

1. Adopt authenticated metadata for official releases too, with established update-protocol tooling rather than a custom signature scheme?
2. Confirm catalogue ownership/location and whether reviewed community discovery uses one governed index or several explicitly added sources.
3. Set the provenance threshold: hub-built artifacts, independently attested publisher builds, or phased requirements by contribution type. Signing alone is not review.
4. Confirm independent security review before the third-party release gate opens, as Jeremy's handoff recommends.
5. Decide whether proprietary plugins need restricted downloads or run-time entitlements at all. Do not introduce licensing machinery merely because source code is private.
6. Reconcile the existing first-party PhotoCraft pilot with the EffectCraft/FigCraft host work before implementation begins.

All items remain proposals unless the decision record explicitly says otherwise.

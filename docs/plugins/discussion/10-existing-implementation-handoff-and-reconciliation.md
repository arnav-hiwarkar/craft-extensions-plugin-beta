# Existing implementation handoff and reconciliation

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Historical discussion archive, captured on 2026-11-08. This preserves technical requests and substantive responses; routine progress updates and tool logs are omitted. Local checkout links are normalized. Later decisions and source corrections take precedence. Start with [the current documentation index](../README.md).

## User request

The user supplied Jeremy's full handoff. Read [the complete supplied text](../sources/jeremy-extensions-todo.txt).

## Research response 1

**Jeremy’s document describes an existing Craft Extensions implementation that we should build on.** It already covers themes, plugins, and resources, with shared crates, launcher UI work, and a PhotoCraft theme adapter. It does not yet demonstrate a complete, release-ready plugin workflow.

Opus 5.5 at medium effort reviewed the distribution and security gates. I checked the runtime requirements, theme layers, PhotoCraft adapter, and FilmCraft needs. The combined findings are saved in the [security and distribution proposal](../../../docs/plugin-security-design.md).

Your accepted defaults are recorded, including useful automatic CLI loading. Themes are in scope; organization/team management is excluded.

Jeremy’s handoff says the intended product is an **Extensions browser in Craft Launcher**, reachable from each app, with Themes, Plug-ins, and Resources sections. First-party extensions come first. Third-party support is designed into the architecture but remains release-disabled until the security gates pass.

I checked the referenced branches. The important distinctions are:

| Area | What exists | What remains |
|---|---|---|
| `craft-ext-core` | Manifest, compatibility, IDs, theme palette, index parsing | Authenticated metadata, richer capability/contribution contracts |
| `craft-ext-store` | Bounded unpacking, hashes, staging, atomic state, app reader | Runtime validation, enforced revocation, stronger version retention |
| Launcher | Store-backed Extensions UI and combined showcase branch | Real catalogue flow and a clean release candidate |
| PhotoCraft | Theme mapping, store watching, contrast checks, fallback | Package-store-to-filter execution and end-to-end verification |
| Third-party gates | Reserved names and a default-off tier feature | Signatures, complete freshness/revocation enforcement, host validation, review |
| Plugin workflow | Existing app plugin runtimes | A verified catalogue/install/enable/run/reopen/remove round trip |

The combined launcher branch includes AI work and is explicitly a showcase branch. Jeremy says it must not become the release source. His reported test counts are historical evidence; we did not rerun them.

Several decisions follow from this.

1. **Keep the existing shared infrastructure as the baseline.**

   We should extend `craft-ext-core` and `craft-ext-store`, rather than create another package format and installer in `craft-libs`.

   Their eventual repository location can be discussed separately. The handoff deliberately kept them standalone while proving consumers. Shared functionality does not require moving everything into `craft-libs` immediately.

2. **Keep the two theme layers distinct.**

   `craft-libs` supplies widget styling mechanisms. `craft-ext-core` supplies the installable theme-data format.

   PhotoCraft’s adapter connects the package palette to PhotoCraft’s own tokens, checks the resulting colour combinations, drops unsuitable overrides, and falls back to its built-in appearance.

   That is a useful pattern for FilmCraft. FilmCraft still needs its own mapping that preserves timeline selection, contrast settings, and visual meaning. A shared palette cannot know every app’s rendering semantics.

3. **Apply the same runtime restrictions to every publisher.**

   “Made by ArtCraft,” “proprietary,” and “community” describe origin or distribution. They should not grant different access to files, documents, networking, accounts, or private app memory.

   We should separately establish who owns an identity, who released these exact bytes, whether they were reviewed, what they may do, and whether access is restricted. Paying for a plugin or signing it does not make its execution unrestricted.

4. **Authenticate the catalogue, including official releases.**

   A checksum establishes that a download matches the metadata. It does not establish who supplied that metadata.

   My recommendation is established update-protocol tooling covering signatures, key rotation, freshness, rollback protection, and recovery. [TUF](https://theupdateframework.github.io/specification/latest/) provides that model. Jeremy recommends considering it too; his particular signing scheme remains an open decision.

   Installation and updates should retain source and publisher continuity. Another catalogue offering the same name must not silently replace an installed plugin.

5. **Treat local file installation as a real execution-admission path.**

   The existing default policy blocks `ReviewedThirdParty`, but permits `FromFile`.

   Therefore, “third-party is switched off” does not mean externally authored modules cannot enter the store. Sideloaded packages need the same mandatory module validation and runtime restrictions.

   The store currently verifies package contents but does not implement a plugin executor or verify WASM imports. That app-host work remains essential.

6. **Version the richer capability system.**

   Jeremy’s v1 requires empty permissions and no host imports. That can support pure computation, but FigCraft command plugins need host calls.

   We should preserve v1 and introduce an explicitly versioned contract for capabilities. Each ability needs its matching host enforcement and rejection behavior. We should not simply remove `PermissionsNotEmpty` or let older hosts ignore security-critical fields.

   Official plugins must go through that same path.

7. **Make automatic CLI loading useful without expanding rights.**

   A fresh CLI should resolve installed plugins that are enabled for the app, compatible, permitted, and eligible under current policy.

   It should not automatically download missing code, approve new capabilities, reactivate disabled packages, or disregard revocations.

   Missing dependencies should produce clear errors before final export. Automatic loading is a convenience default, not an alternative permission system.

8. **Separate installation, activation, and active-version retention.**

   The current store enables a newly installed package for its compatible apps and removes its previous version after updating.

   Our broader model needs more: independent grants, explicit activation rules, versions retained for running exports, and rollback that preserves compatible state.

   Themes also need a separate “use this appearance” action. Installing a theme should not unexpectedly restyle an app.

9. **Support proprietary distribution without assuming a licensing service.**

   ArtCraft-produced proprietary plugins may simply be privately developed first-party packages. They are not necessarily paid products.

   If restricted downloads or entitlements are required, those should be separate policy layers. They must not weaken sandboxing. We also should not promise that encrypting locally executed code keeps its implementation secret.

10. **Keep contributor requirements small and generate the mechanical details.**

    Authors should provide stable IDs, versions, contribution contracts, required capabilities, resources, and necessary licence metadata.

    SDK tooling should generate inventories, sizes, digests, export checks, and boilerplate. Authors should not manually assemble signature structures.

    The existing ZIP and `craft-extension.json` format is a useful migration baseline. Its current single-kind layout does not yet implement our accepted multi-contribution design, so that evolution needs explicit schema compatibility.

The practical next step is to reconcile Jeremy’s first-party PhotoCraft pilot with our EffectCraft/FigCraft functional validation. They prove different parts of the system: PhotoCraft already provides theme/store work worth preserving; EffectCraft and FigCraft test rendering and editor-command contracts.

I recommend keeping the third-party release gate closed while completing that integration and the missing security controls. Supporting third-party architecture now does not require enabling it prematurely.

The next decisions I would bring to you are:

- Whether official releases should require authenticated catalogue metadata too. I recommend yes.
- Who owns the catalogue and its review process. `crafthub` still says “Placeholder for merges,” which is not approval to use it.
- Whether community artifacts must be built by the hub or can use verified publisher builds with provenance.
- Whether proprietary packages actually need restricted downloads or licensing checks.
- How to sequence the existing PhotoCraft pilot alongside EffectCraft and FigCraft.

I also saved a [summary of Jeremy’s supplied text](../../../docs/jeremy-extensions-handoff-summary.md). Documentation checks passed. No application code changed, and no builds, runtime tests, or security certification were performed.


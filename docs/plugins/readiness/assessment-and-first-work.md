# Assessment and first development work

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: root reconciliation after the requested Opus coordinator and three Haiku audits, 2026-11-08. This is implementation preparation, not implementation. No app code, build, test, external issue/repository, CI, dependency, key or release was changed. The third-party release gate stays closed.

## Recommendation

Extend the existing shared core/store; reuse EffectCraft's catalogue behaviors and release fields; generate the SDK from each app's existing operation registry and explicit ABI contracts. Start with store lifecycle and truthful-output/caller-boundary fixes, then demonstrate real author-to-user vertical slices. Do not start by building a larger browser, a new installer, a universal command engine or integrations for all sixteen apps at once.

The source audit is useful because it found both reusable systems and concrete gaps. It did not find one uniform app architecture. All sixteen committed local HEADs match the earlier inventory, verified again by root. No upstream fetch occurred; these are current local committed snapshots, not a claim about latest upstream main. Dirty app work stayed untouched.

## Biggest system changes

| Priority | Change | Why it matters | Existing work to keep | First observable proof, future work |
|---|---|---|---|---|
| 1 | Source-bound multi-release store and explicit app grants/enablement | Current install/update auto-enables apps, deletes old bytes and omits identity/authority records. New SDK promises cannot rely on that lifecycle | craft-ext-core/store validation, staging, lock, atomic snapshot and generation | Install leaves apps disabled; update preserves choices; same-ID foreign file cannot inherit authority; a pinned old release remains readable |
| 2 | Caller/scope-aware operations and authenticated automation boundaries | Sharing CLI and plugin behavior must not share full user authority; existing guards differ between apps | PhotoCraft/TextureCraft file-authority modes, FigCraft plugin policy, existing engine dispatch/transactions | Plugin cannot install code, start a native helper, mint grants or reach unauthorized files through nested calls; explicitly authorized CLI still works |
| 3 | Truthful persistent processing and output | Missing/failed processing can be omitted or passed through in inspected paths; headless offered-file output can be dropped | Existing render queues, smart-feature data and app history/jobs | Missing required effect refuses final success; output target is explicit; preview fallback is labelled; project state survives failure |
| 4 | Additive contribution registration and safe replacement | Same-plugin replacement lacks continuity/settings rules; internal registry behavior must be hardened before exposing new extension seams | Existing built-in ID guards, format selection and keymap conflict detection | Built-in ID refused; new command is namespaced; same-source explicit upgrade preserves settings; installation does not steal defaults/shortcuts |
| 5 | One authenticated reviewed catalogue and SDK release path | Existing catalogue implementations are app-specific/unsigned; compiled bytes need durable approval/compatibility data | EffectCraft per-release URL/hash/cache behavior, launcher download mechanics, existing shared index/envelope | SDK asset -> human exact-release approval -> authenticated metadata -> install -> grant -> actual app use, with wrong bytes refused |

Other substantial work is exact project/reference migration, fresh-CLI initialization, retained resources/concurrency, richer UI/format/tool contracts, and dedicated native/GPU/audio enforcement. Those are ordered work packages rather than reasons to rewrite the editors.

## What Jeremy's work already gives us

The [original artifact](../sources/jeremy-extensions-todo.txt) identifies the actual core/store, launcher and PhotoCraft branches. Preserve and extend them. The [crosswalk](jeremy-crosswalk.md) maps n1–n6, sg2–sg13, pl1–pl4, UI, shipping, resource and roadmap tasks to the latest direction.

Reuse bounded package admission and theme mapping now. Finish real store/host integration before porting only demo UI. Preserve clean dependency pinning, source review, candidate checks and the explicit release gate. Catalogue location, mandatory hub builds and download-count ranking were proposals superseded by the user's new metadata repository, publisher GitHub assets and GitHub-star preference.

EffectCraft source already has a Plug-in Library describing a reviewed per-release GitHub index, expected hashes, optional sizes, cached browsing and installed integrity checks. Root opened `library.rs` at `ce9eaec`. That is direct implementation evidence for fields and behaviors to reuse, not verification that the remote catalogue exists today or that its review practice is adequate. Merge these requirements into the existing shared successor format; do not adopt a seventh authoritative schema or installer.

Some legacy release assets are raw WASM modules, while the shared SDK emits a package envelope. Migration must distinguish the old raw-module digest from a newly reviewed package archive digest. An explicit compatibility importer or repackaged release can preserve the original ABI and legacy IDs, but it cannot invent authenticated publisher continuity or grants from an unsigned old index.

## Root corrections to the coordinator findings

| Finding | Correct scope and evidence |
|---|---|
| “Caller authority missing everywhere” | A consistent originating plugin principal/grant model is missing. Existing file modes, token-authenticated channels and FigCraft command/read restrictions are real controls. Root re-read FigCraft `plugins.rs:165–181`; no blanket claim of absent authorization is justified |
| Same-ID plugin replacement proves core replacement | Replacement of installed plugin identity/settings is distinct from shadowing a built-in command. Some apps have no runtime plugin command seam at all. Preserve actual core-ID guards and add explicit guards before exposing new ones |
| Static registries cannot unload | EffectCraft explicitly documents restart-needed unregister behavior. Root read PhotoCraft `registry.rs:21–49`: it has mutable install/get/remove with shared handles, so its static lifetime alone does not establish restart-only updates |
| Refuse every exporter sharing a file extension | Root read VectorCraft `formats.rs:703–740`: built-ins resolve first, plugins can be selected by format ID. Permit multiple uniquely identified exporters and explicit selection; installation must not silently replace the existing default |
| Gate GridCraft test helper behind plain `cfg(test)` | Root read helper definition and all committed call sites. `crates/ui-egui/tests/keytips.rs` uses the exported engine helper from an integration test. Plain dependency `cfg(test)` would break that consumer. Audit a test feature/seam only if needed; no untrusted registration route was demonstrated |
| Every app already has a registry with uniform undo | ArtCraft has a different provider/job architecture; GUI wiring and some domain targets were not traced. Existing editor commands/history are a reuse base, not proof of one universal document model |
| No ArtCraft entry in the old inventory | Root parsed `plugin-functional-inventory.json`: ArtCraft is present among sixteen entries. This minor worker statement is incorrect; the committed-revision table is still useful |
| Catalogue digest alone means approval | Approval is the authenticated reviewed record binding identity, exact bytes, compatibility and policy. A bare checksum, GitHub-upload hash or publisher checksum file is insufficient |
| Credentials can follow a GitHub redirect | User authorization stays at its authenticated origin. Strip it on cross-origin redirects; use the returned approved asset URL without forwarding the user's API token; redact signed URLs |
| Descriptor equality proves contract compatibility | Behavior and authority can change without shape changes. Version observable semantics too; generated schema diffs help but cannot prove equivalence |

These corrections are incorporated into coordinator summaries/proposals where material. Independent Haiku notes remain intact. Broader “no runtime”, control-channel exploitability and exact final exit-status claims retain their inspected-path/runtime limits.

## Simple catalogue and search

One metadata repository stores publisher identity, package UUID/readable name, compatible app contributions, exact approved release URL/size/digest, required rights/licences, review record, revocations and aliases. Publishers host compiled package assets in their own GitHub Releases. The SDK prepares the release record; maintainers approve exact bytes in a PR. A new upstream release does not become an approved update automatically.

Recommend established authenticated update tooling, such as a suitable vetted TUF client, for the production path. Keep keys/policy/recovery with named operators. The coordinator's temporary custom-signature alternative remains unselected; it is not a new required phase. GitHub hashes/immutability add consistency evidence, not user consent or a safety certificate.

Proposed search is query/app/kind/compatibility filtering, then exact/prefix/text relevance tiers, official precedence within a tier, cached repository stars descending and stable tie-breaks. The precise precedence interpretation remains a UI decision. Stars are labelled with their source/date and never influence admission or execution rights. Catalogue maintenance fetches statistics in batches; desktop startup need not make one API request per package. Cached data keeps browsing useful during outages/rate limits.

Normal public use needs no login. Authors use GitHub to publish assets and submit metadata PRs. Restricted releases may add optional host credential integration. A new catalogue account system, binary host, build farm or always-on server is unnecessary for this architecture.

## Simple SDK with app-specific depth

One plugin-first author project declares package identity and app contributions. The app's Rust registry remains authoritative for exposed operations; generated inventory produces SDK types, CLI discovery, docs and conformance coverage. Existing pixel/effect ABIs are described through that same generation path, with preserved runtime adapters.

The first SDK needs create/add/inspect/check/test/dev/pack/prepare-release intents, explicit compatibility and permissions, generated routine metadata and real host examples. Tool names and crate splits are proposed. Start with the operations needed by the real pilots; do not retrofit every existing internal command before an author can create a filter.

Typed wrappers carry host-created identity through scope-aware operations. Required rights need explicit app-specific consent. Optional rights leave unrelated operations usable. Plugins add their own commands/tools/panels/exporters/widgets/resources and can use supported native engine functions, but cannot replace builtin identities, code, defaults or trusted prompts. A user may explicitly choose an extension handler, theme or shortcut.

## Ordered user and development steps

For a user: open Extensions, search compatible packages, inspect contributions/permissions, install approved bytes, explicitly set up the current app, use the feature, save normally, and receive understandable update/dependency/recovery choices. Setup can be one clear sheet rather than several technical dialogs. A second app reuses package bytes but gets its own setup and grants. Local-file installation follows the same runtime checks without catalogue submission.

For an author: generate a project, add app contributions, implement against generated types, run real fixtures, build/check/package, publish a GitHub Release, submit the generated catalogue record, answer human review, and repeat for updates. No hand-written hashes, install scripts or extra public catalogue login.

For maintainers: enrol publisher/identity, review source and generated deltas plus exact release bytes, approve, publish authenticated metadata, refresh advisory stars, and handle revocation/recovery with clear records. Licence list, review staffing and key custody need explicit owners/policy.

For app developers, recommended sequence is:

1. Reproduce and narrowly fix the independent output/caller/registration concerns selected from W3–W5, with app-owned behavioral tests. Audit public API callers before removing them.
2. Define and implement W1/W2 store/state/identity/grant/retention and load-side eligibility with old-reader migration fixtures.
3. Build W8 together with descriptors for the first ABI/operations from W12. Prove a real author-generated package, not a packaging-only SDK promise.
4. Add W6/W7 authenticated reviewed GitHub distribution. Reuse transport/hashing mechanics without trusting publisher-controlled fallback hashes as approval.
5. Complete PhotoCraft's existing theme/filter store/host slice. Local testing can precede network publication; full authenticated first-party acceptance includes the actual catalogue path.
6. Prove EffectCraft live state and GUI/CLI export, then FigCraft scoped commands/undo/artifacts. Keep their accepted functional order.
7. Extend app by app using the sixteen source notes, then evaluate advanced native/GPU/audio/workflow/resident-service contracts with their own evidence.

These work packages are ready for implementation scoping; this investigation has not begun those fixes, opened issues or created PRs. Exact mechanism/compatibility choices remain reviewable before each cross-repository change.

## Remaining product choices

Set the new repository/operator/key owners; select catalogue/local/official-proprietary licence treatment; confirm exact executable-release review policy and staffing; confirm ranking precedence interpretation and update default; decide optional noninteractive explicit CLI grants and restricted-release support. Preserve sandbox-first, app-specific consent, public/local usability and the closed community gate while these details are settled.

Unrelated-looking source findings about ArtCraft's sign-in webview or SolidCraft's repository owner are maintainer questions, not justification to derail this plugin programme. The narrow evidence limits are in [the evidence index](evidence-index.md).

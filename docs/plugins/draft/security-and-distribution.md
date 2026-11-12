# Security and GitHub distribution

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: proposed architecture, 2026-11-08. The user chose GitHub-hosted compiled releases, a new metadata repository, human review, official precedence, star-based community popularity and full local installation under the same protections. No repository, workflow, signing service, key or release is created here. The third-party release gate remains closed.

## Reconcile Jeremy's distribution plan

[Jeremy's complete handoff](../sources/jeremy-extensions-todo.txt) proposes a catalogue, hash-pinned downloads, truthful trust labels, startup/enable revocation checks, host validation, dependency pinning and a closed sg13 gate. Those requirements remain. Its crafthub location, central hub-built provenance and download-count ranking were recommendations, not accepted decisions.

The latest direction replaces those defaults with a new metadata repository, binaries in publisher GitHub Releases and GitHub stars for popularity. The existing craft-extensions core/store remains the installer baseline. Do not create a parallel catalogue schema/store implementation in craft-libs. Extend the existing contracts through explicit negotiation.

“We do not host plugins” means no Craft-operated binary storage service. Craft still operates review and publishes small authenticated metadata through GitHub. Publisher-owned or official GitHub repositories hold package assets. No client clones and executes a repository during normal installation.

## Catalogue repository and review records

The proposed repository contains publisher/package records, approved release records, catalogue policy, review procedures, licence policy, revocations, migration aliases and generated metadata. Its name/owner is open; creating it needs separate scope. A repository directory shape can be chosen during implementation without adding a service framework.

An approved release binds package UUID, source/publisher identity, app contributions, release version, GitHub repository identity, tag/commit/release/asset locator, expected byte length and archive digest, contract/permissions summary, licence inventory and human review reference. Exact manifest and payload digests are verified after download. A release tag or filename alone is not authoritative.

Record GitHub numeric owner/repository IDs and readable handles as continuity evidence, alongside an enrolled catalogue source/publisher record. The authenticated catalogue, not a claim about numeric-ID permanence, authorizes that binding. Repository transfer, ownership change or key-authority change requires a reviewed transition and renewed user consent. A readable handle can change without rewriting saved UUID/digest references. Local UUID collisions stay separate by origin and never replace an authenticated install or inherit its grants.

Recommended review procedure:

1. Publisher prepares compiled assets and release metadata using the SDK, then submits a catalogue PR. Official packages follow the same human approval path.
2. Validation checks identity continuity, metadata consistency, allowed licences, compatibility, requested authority, dependency inventory, artifact integrity and claimed build evidence. Malformed data fails without running publisher code with repository secrets.
3. Maintainers inspect the change, source/release diff and relevant security evidence. A review records what was actually inspected and any accepted limitations.
4. A human reviewer approves the exact release digest. Proposed default: every executable release needs an approval, not merely the package's initial listing. Data-only policy can be assessed separately without assuming review is unnecessary.
5. After merge, a trusted publication step generates and signs versioned catalogue metadata referring only to approved artifacts. The resulting metadata version is auditable back to reviewed commits.
6. Clients advertise/install only approved eligible releases. A publisher's newer GitHub Release remains unavailable in the catalogue until its own approval. Existing installs do not switch bytes through `latest`.

Human review cannot guarantee harmless behavior. Catalogue approval, cryptographic authenticity, runtime confinement and user consent each address different failure modes. Local-file installation deliberately skips catalogue review but not validation or runtime controls.

Generate a review report comparing source revisions, permissions/scopes, module imports/exports, dependencies/licences, sizes, compatibility, state migrations and any verified build provenance. Reviewers record which source/build evidence and representative behavior they actually checked. Optional attestations are labelled verified only after their subject digest, builder/workflow identity and source claims pass verification. Presence alone is an unverified claim. No build farm is required.

Name responsible maintainers and key/recovery operators before catalogue release. Use a priority queue for urgent fixes and immediate authenticated revocation where needed. An expedited release still needs human exact-digest approval before catalogue publication; there is no post-publication approval bypass. A mandatory two-maintainer rule and service-level timings remain governance choices, not invented requirements. Data-only reviews can use a type-specific checklist while retaining human admission.

PR validation must not run untrusted build/install scripts with secrets or write authority. Proposed automation operates with least privilege, separated PR checks and protected release publication. Do not edit current repository CI in this feature/design task; a future workflow change follows the shared craft-common-actions ownership rules.

## GitHub assets and mutable upstreams

GitHub Releases support downloadable assets. The asset API can return content or a redirect and supports authenticated reads for restricted repositories. The client must handle redirects without leaking credentials to another host. [GitHub releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases), [release asset API](https://docs.github.com/en/rest/releases/assets).

Prefer immutable publisher releases where supported and enabled. GitHub describes immutable releases and artifact attestations, while noting that attestations do not guarantee security. These are additional supply-chain evidence; authenticated approved digests remain mandatory. [GitHub supply-chain documentation](https://github.com/github/docs/blob/main/content/code-security/concepts/supply-chain-security/supply-chain-security.md).

When an upstream asset changes, disappears, moves ownership or becomes inaccessible, the client refuses mismatched bytes and keeps previously verified installed releases usable according to policy. It does not substitute another asset with the same name. A repository move requires verified identity continuity; a publisher ownership transfer needs a reviewed transition. The manager can explain that a package remains installed but is no longer downloadable.

The metadata service does not promise archival availability of every publisher asset. Local verified caches and exportable dependency reports help, but permanent preservation and redistribution rights are separate matters. An offline package export, if added, must respect licence terms and never include credentials or grants.

## Authenticated update metadata

Recommend a vetted implementation of TUF or an equivalent established update framework whose threat coverage is demonstrated, rather than custom cryptography. TUF supplies root, targets, snapshot and timestamp roles and rules for version/freshness/key updates. It does not supply a maintainer review policy, plugin sandbox or meaningful licence decisions. [TUF specification](https://theupdateframework.github.io/specification/latest/).

Bootstrap the catalogue trust root through the signed application distribution. Enrolled source identity stays stable across verified key rotation. Separate offline recovery/root authority from online publication authority, use appropriate signature thresholds, protect credentials and rehearse recovery. Exact tooling, thresholds and key custody require an operational owner before release.

Authenticated targets bind exact approved package digests and sizes. Catalogue interpretation, admission and revocation information must also be authenticated; a signed binary digest cannot make an unsigned change to publisher/permission/official labels trustworthy. Snapshot/version/expiry checks prevent replay/freeze/rollback within the selected protocol's assumptions. Same-version changed metadata is a failure, not a cache refresh.

Concrete simple publication model: the existing index's successor is one bounded catalogue data target whose exact bytes are authenticated by TUF metadata. It transitively binds approved package assets and policy; a separate authenticated revocation target can stay compact for installed-package checks. Do not add another custom signature protocol around the same index. The catalogue client's verified-artifact handle binds index identity/version, package UUID, origin, expected asset, size and digest before handing bytes to the store. Define scale caps and eventual sharding from measured catalogue size, not a service-per-package design.

Revocations need authenticated delivery and applicability to installed releases even if a release is removed from search. Use a dedicated authenticated policy target or an equivalently unambiguous binding. Distinguish exact-release blocking, package/publisher compromise, advisory notices and recovered releases. Highest-seen trusted versions and known revocations survive repair and ordinary rollback.

Schema successors must express exact digest, bound package/version range and source/publisher-wide matches with explicit severity. Known malicious digest blocking applies to matching bytes imported locally too; changing an origin label cannot evade it. Authenticated recovery can supersede a restriction under an explicit newer policy, never by deleting the cache or restoring an older state backup. A source-wide policy is not inferred merely from a reused display name.

The client can ship a bootstrap cache for offline inspection. Expired metadata is visibly stale and cannot authorize fresh acquisition outside an explicitly designed import policy. Signature verification failure never clears current restrictions or replaces trusted metadata with whatever GitHub most recently returned.

## Search, official placement and GitHub stars

Search first applies the user's query, selected app/kind and useful compatibility filters. Among relevant results, official packages take precedence. Community popularity uses GitHub star counts with a visible timestamp and a stable tie-breaker. A highly starred irrelevant package should not displace an exact relevant match; exact query/ranking interaction is a UI detail to test.

Fetch star statistics in catalogue maintenance automation and cache compact snapshots. Every desktop must not independently request counts for every repository at startup. GitHub's API has rate limits, with stricter limits for some endpoints. Stop/back off on rate limits and retain the last useful statistics. [GitHub rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api).

Popularity metadata is advisory. It cannot modify approved release identity, permission requirements, source ownership or safety status. A missing/stale count affects sorting, never installed eligibility. Stars can be manipulated and a repository can contain multiple packages; display repository-star semantics accurately rather than claiming per-package usage. No in-app activity tracking is needed for this ranking.

## Local files and development

Local packages require no catalogue account, PR or maintainer approval. The installer records unverified origin unless an independently trusted signature proves a stronger origin. UUID/name/publisher claims cannot acquire an official label or replace an existing authenticated source silently.

The closed reviewed-community catalogue gate does not silently remove the existing admitted local-file path. A release profile that admits FromFile can load local packages only under supported module contracts, full runtime policy and explicit grants. A development client cannot launder a gated ReviewedThirdParty record into release eligibility by sharing its store, and known malicious digest blocks apply regardless of origin. Any change to the existing release admission categories requires explicit gate-policy scope rather than a relabelled package.

Local admission checks the same archive bounds, exact inventory, supported format/contracts, module imports/features, decoder rules, required resources and licence policy applicable to local imports. Explicit per-app grants precede execution. The same runtime brokers and budgets apply. Signed local files are not automatically approved catalogue releases.

The package-licence policy for local versus catalogue installs remains an open detail. The draft must not imply that “no review” means “no security,” or silently decide that catalogue licence restrictions forbid every private local plugin. Development mode supports labelled reload under an explicit development store/profile and does not make ordinary verified release records mutable.

## Runtime protection model

Threats include malicious publishers, compromised release/review accounts, hostile local packages and data, parser exhaustion, changed installed bytes, permission bypass, stale handles, misleading UI, and wrong output after missing or changed dependencies. Same-user arbitrary modification of trusted host executable/account state remains outside a guarantee that plugins can fix; document the accepted threat boundary.

Apply protection at each boundary:

| Boundary | Required enforcement | Failure response |
|---|---|---|
| Metadata acquisition | Authenticated identity, versions, expiry, size/parse caps and origin policy | Keep trusted state; refuse new acquisition and explain failure |
| Download | Approved asset/digest/size, time/concurrency caps, redirect/credential policy | Discard partial/unverified artifact; bounded retry |
| Archive admission | Pre-index allocation/entry limits, paths/links/duplicates, inventory and hash checks | Reject without exposing files outside staging |
| Resource decoding | Type-specific memory/dimension/object/recursion limits | Refuse resource/import; no executable install hook |
| Module load | Exact bytes/digest, actual engine feature/import/profile checks and bounded compilation | No instantiation; integrity/compatibility diagnostic |
| Host operation | Principal, declaration/grant/scope, arguments, handle ownership, revision and live policy | Denied structured result; no partial unauthorized side effect |
| Edit/result commit | App domain validation, transaction/revision/output limits | Rollback or reject stale result |
| Long work | Total budgets, host cancellation, job ownership and retained resources | Terminal failure/cancel without false successful output |
| UI/logs | Host-owned trusted prompts, validated declarative data, untrusted log origin and redaction | Safe fallback/quarantine, no deceptive verified status |

Initial WASM backends have no ambient filesystem/network/process authority. Brokered operations deliberately expose required functionality. File tokens identify approved inputs/outputs and enforce path/ownership checks at use, including symlink/handle lifetime races. Network brokers enforce destination and redirect policy, response/decompression limits and prevent access to unintended local services. Do not treat an author-supplied URL allowlist as sufficient enforcement.

OS file pickers grant narrowly scoped host tokens. Account brokers retain credentials and expose only approved service actions; content upload requires its own disclosed permission. Plugin-private storage is namespaced, bounded and app-specific unless shared storage is explicitly designed and granted.

## Backend enforcement by platform

| Backend | Proposed contract | Current evidence and release rule |
|---|---|---|
| Existing desktop WASM | Engine-enforced imports/memory/execution plus app host-call policy | Source-reviewed partial mechanisms only; test actual profiles on every supported desktop target before claiming protection |
| Web WASM | Available browser/app contracts and same explicit authority semantics | Preserve existing web builds; native/process capabilities absent; engine/platform differences advertised |
| Host GPU compute | Typed bounded operations, resource ownership, queue/transfer budgets, device-loss and cancellation policy | Planned contract; arbitrary plugin shaders are a separate capability/backend with driver risk |
| Native worker on macOS | Versioned helper protocol, constrained files/network/process/device access and bounded IPC | Confinement mechanism unselected/unproven; process separation alone insufficient; unavailable in new safe path until proven |
| Native worker on Windows/Linux | Same required guarantees with platform-specific confinement and recovery | Unproven separately on each platform; no cross-platform safety claim from a macOS experiment |
| Remote provider | Explicit egress/credential scopes, job/error/cancel contract | Offline unavailable when remote service required; user data handling and endpoint policy explicit |
| Real-time audio | Dedicated bounded processing, state/latency/tail/deadline/failure semantics | Runtime choice deferred to SoundCraft evidence; existing native loading is not an accepted exception |

No source tier, review badge, popularity count or broad consent checkbox waives platform enforcement. Advanced functionality can be designed now and unavailable on a backend until demonstrated. This is a compatibility result, not silent fallback to unrestricted execution.

## State, retention and concurrent clients

Installed records retain verified manifests, origin, exact digests, release aliases and authority declarations. Enablement/config/grants are app-specific. Recommended initial implementation is one atomic versioned state snapshot with a generation counter, containing release/default/pin/grant state and the selected trusted-policy versions/digests. Immutable package bytes and authenticated metadata cache files are referenced by identity. Reader errors fail closed. This keeps publication coherent without a multi-file transaction protocol; consider another layout only if measured size/contention justifies it. The current 4 MiB state cap is an existing bound to reassess, not a proposed limit on the full ecosystem.

Old writers must refuse new authority-bearing state rather than erase unknown grants/revocations. A migration backs up data, preserves aliases/settings/project references and reports unsupported client versions. Do not turn a newer/corrupt store into an apparently empty successful executable environment. Themes can use built-in appearance while management reports the actual cause.

At job admission, acquire retention handles for the exact modules and lazy resources needed. Cross-process readers require robust ownership/lease semantics; process IDs alone are insufficient under reuse/crashes. Collection checks active ownership, installed defaults, rollback retention and explicit pins. Crash cleanup cannot remove another active installer or long-running reader based solely on age.

Keep one rollback release plus explicit pins as the proposed initial policy, with visible disk management. Unknown unopened projects cannot all be protected automatically. Uninstall stops new eligibility; byte removal waits for safe ownership release. Rollback still enforces current revocation/grants and does not migrate project state backwards silently.

## Offline, incidents and logs

Offline-capable verified installed work uses cached policy without periodic mandatory sign-in. Remote/provider-dependent operations say why a network is required. Cached known revocations still apply; unseen revocations are unknowable. Suspicious time rollback blocks new trust acquisition until recovery without destroying installed projects. Product entitlements, if needed, have separate expiry/offline semantics.

The source operator can withdraw search listings, publish authenticated revocations, rotate compromised keys and approve recovered releases. App hosts stop new use and deny revoked capabilities on subsequent calls. Emergency blocking cancels affected transactions/jobs at safe boundaries; advisory notices may allow completion. GPU queues and real-time audio need their declared boundary, not a universal immediate-kill promise.

Each app shows manager/job reports in its own diagnostics history. Shared store events carry operation ID and safe package/release identity for correlation. Logs are bounded/rotated and redact content, paths, credentials and private URLs by default. Detailed traces/support bundles need an explicit user action. Offline logs remain local; automatic upload is not authorized. Plugin logs are clearly marked as plugin-origin text.

Safe startup disables executable extensions without deleting projects. Repair verifies manifests/bytes and preserves unreadable records for recovery. Restoring a backup must not discard newer revocation/freshness restrictions. Quarantine targets the exact release/app failure and requires an explicit safe recovery/re-enable path.

## Licence and proprietary policy

The user requests restricted plugin licences but has not chosen the list. Catalogue product policy is a separate explicit decision from repository dependency/asset policy. Options are an allowed open-licence list, that list plus a named official proprietary class, or another explicitly selected product policy. Local-file policy must also be selected rather than inherited silently. Do not extend the repository's allowlist by editing a copy, and do not assume it automatically governs all independently distributed community products. Evaluate compound expressions and bundled assets/dependencies under the selected product policy.

Official proprietary packages were an existing requirement. Decide whether catalogue policy has a narrowly defined official proprietary class with explicit terms, or all distributed packages must use allowed open licences. Distinguish package product terms from dependencies/assets governed by repository rules; a proprietary label does not exempt those dependencies/assets.

Public GitHub assets require no end-user catalogue account. Restricted releases may use an optional host credential integration scoped to the chosen repository; no credentials go into packages/projects/logs or redirected download URLs. Paid licensing is not assumed. Client-delivered code cannot be promised confidential just because the source repository is private.

## Required security evidence

Future acceptance includes exact digest substitution, moved assets/repos, compromised/recovered metadata keys, rollback/freeze/equal-version changes, expired metadata, denied/nested/parameter-based capability calls, hostile data/modules, disk exhaustion, concurrent update/remove/read, state migration/repair, release reader policy and offline known revocation. Independent security review must assess actual candidate code and operating procedures before sg13.

No build, attack simulation, signature implementation, licence audit or sandbox certification occurred in this draft. The [delivery reference](delivery-and-acceptance.md) assigns these checks to future approved work and candidate CI.

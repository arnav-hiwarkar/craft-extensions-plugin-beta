# Plugin distribution, publisher identity and team policy (Opus pass)

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Historical preliminary pass. The user subsequently clarified themes only, excluding organization/team management, and supplied Jeremy's actual craft-extensions handoff. The current conclusions are in [the handoff review](plugin-security-handoff-opus.md) and [combined security proposal](plugin-security-design.md). The team-policy section below is not active scope or an approved design.

Status: **proposed**, 2026-11-08. This is the distribution half of the security design, alongside the primary's runtime-security work. Nothing here authorizes implementation, dependencies, catalogs, keys or services. The Claude artifact named in [the resource register](plugin-resources.md) was not read, and nothing here depends on it.

**Approved inputs** (from [the decision record](plugin-functional-decisions.md)): sandboxed executable plugins first, official and third-party distribution, EffectCraft then FigCraft, existing ABIs through adapters, multi-contribution packages, headless-callable operations, a fresh CLI using compatible installed eligible plugins, an extensible package envelope, and a stable SDK with a low contributor burden. ArtCraft-made proprietary plugins and community plugins are both in scope. Team or organization catalogs and themes are in scope provisionally. **Everything else below is a proposal.**

## 1. Existing project patterns (source reads, not runtime checks)

- `craft-update` (`crates/update`) checks each download against a published SHA-256 before unpacking (`install.rs`, `release.rs`). The digest comes from the same feed as the download URL, so it detects corruption, not a compromised feed. On macOS it requires the bundle's Developer ID signature to name the expected team (`install.rs`, "signed_by … team"). On Windows it uses `verifysign`. `pins.rs` stores per-app version tracks (stable, nightly, pinned). `source.rs` mentions a future "ArtCraft catalog API".
- The [EffectCraft plugin library](plugin-resources.md) has catalog releases with API versions, URLs and SHA-256 digests, plus installation records. It requires a restart to finish updates.
- The craft-libs crates contain no entitlement, licence-server, team-policy, publisher-key or package-signature code. This was a bounded `grep` over crate sources. It means **not found here**, not that no such code exists in app repositories or services.

These parts are good to reuse: pinning the signer rather than accepting any valid signature, verifying before unpacking, and launcher-written pins that apps only read. What is missing is authenticated metadata that is independent of the download host.

## 2. Five separate questions

A design that merges any two of these will fail. Each answers a different question:

| Concern | Question | Mechanism (proposed) | Does not imply |
|---|---|---|---|
| **Identity** | Who publishes this package ID in this catalog? | Catalog-scoped namespace bound to a publisher record | That the code is safe |
| **Signing** | Were these exact bytes released by that publisher and catalog? | Signed metadata over digests (TUF-style), optional provenance | Review, or any permission |
| **Review** | Did someone inspect it, and against what policy? | A catalog's signed review attestation for a version | Sandbox exemption |
| **Sandbox** | What can it do when it runs? | Declared capabilities, host-enforced (primary's area) | Trust from the source |
| **Entitlement** | May this user run or download it? | Licence token checked by the host or catalog | Trustworthiness |

**Official plugins do not get unrestricted privilege.** An ArtCraft-signed package runs in the same sandbox, with the same capability declarations and consent rules as a community package. Official status can change presentation (a badge, default visibility, a preinstalled source). It cannot grant capabilities that are undeclared, remove consent prompts, or bypass the sandbox. Otherwise a stolen ArtCraft key becomes a suite-wide breach, and every proprietary bug would run with full host access. Licensing and trust also stay separate. Paying for a plugin does not make it more trusted, and a free community plugin is not less sandboxed.

## 3. Catalogs, collisions and source pinning

A user can have several sources: the official ArtCraft catalog, community catalogs, a team catalog, and local development folders.

- **Package identity is `(catalog, package ID)`**, not the bare ID. The UI shows the source.
- **Source pinning.** Each installation records the catalog it came from and that catalog's root key. Updates come only from that catalog. A higher version of the same ID in another catalog is a different package and never an update. Moving to another catalog is an explicit user action that requests consent again.
- **Project references** store the catalog, ID and version requirement. A project that names an official package never resolves to a team or community package with the same ID.
- **Dependency confusion** happens when a resolver checks several indexes and takes the highest version ([PEP 708 discussion](https://discuss.python.org/t/pep-708-extending-the-repository-api-to-mitigate-dependency-confusion-attacks/24179)). The functional model does not yet allow plugin-to-plugin dependencies. If it adds them later, each dependency names its catalog. A team catalog may *mirror* official packages without changing their identity, verified by the official signatures. It may not *shadow* them.
- **Reserved prefixes.** The official catalog reserves the ArtCraft namespace. A community catalog that offers `artcraft.*` IDs gets a warning, and its packages stay scoped to that catalog anyway.

## 4. Authenticated, fresh metadata and recovery

Signing the archive alone is not enough. Clients need protection against rollback to old vulnerable versions, freeze attacks where the client never sees updates, mix-and-match metadata, and key compromise. [The Update Framework (TUF) specification](https://theupdateframework.github.io/specification/) addresses these threats with separate root, targets, snapshot and timestamp roles, thresholds, and expiry times.

Proposed, without choosing a library:

- **Adopt TUF's role model and threat model** for each catalog. Do not invent a new metadata-signing scheme. Any implementation uses a reviewed library, and its licence must be on the allowlist. That choice is a separate decision; nothing is committed here. No hand-rolled cryptography.
- **Delegations** let the official catalog delegate a namespace to each publisher, including the ArtCraft proprietary team. A publisher key compromise is then limited to that publisher's packages, and the catalog can revoke the delegation.
- **Update-key continuity.** Root rotation follows TUF: the old root signs the new one, so the chain stays verifiable. A publisher key change is a delegation update signed by the catalog. A package whose signer changes without such a record is refused, never accepted on trust-on-first-use.
- **Recovery.** The offline root, held by people who can meet a threshold, can revoke compromised online keys and publish replacement metadata. Clients that come back online follow the root chain.
- **Signed provenance** (optional for community, proposed for official): a [SLSA](https://slsa.dev/spec/v1.1/levels) Build L2 attestation records the source and builder. [Sigstore](https://docs.sigstore.dev/cosign/signing/overview/) keyless signing binds a signature to a CI identity and logs it publicly. That suits open-source community packages. It does not suit private or proprietary ones, because transparency-log entries are public and permanent. Treat provenance as evidence that a review can use, not as a replacement for catalog metadata.

## 5. Artifact validation and immutable installs

Proposed install steps, in order: verify metadata, download to a staging folder, check length and digest, validate the archive (no path traversal, links, device files, or size and count bombs), validate the manifest against the schema, run the SDK's compatibility checks, then atomically move the result into a **content-addressed, read-only store** keyed by digest. Installed packages are never modified in place. Plugin data lives in a separate per-package, per-app state folder. The primary owns the runtime enforcement around that folder.

## 6. Offline use, expiry and revocation

Sandboxed plugins have to keep working on an aeroplane, and the fresh CLI must not depend on the network. Proposed policy:

- **Installing and updating** need fresh, unexpired metadata.
- **Running installed packages offline** continues after metadata expires, with a visible "not checked since…" status. Expiry blocks new trust decisions. It does not turn off code already installed.
- **Explicit revocation** (a signed "yanked: malicious" marker) disables the package the next time the client is online, including in the CLI. It does not delete the package or its saved state, and projects show the feature as unavailable instead of silently rendering without it.
- **Team policy** can set a maximum offline grace period for managed machines.

## 7. Rollback that preserves state

Proposed: keep the previous immutable version in the store. Rolling back changes which version is active, not any files. Per-instance state is versioned, as the functional model requires. A downgrade whose state schema is newer than the older code supports keeps the state untouched and refuses to evaluate, rather than migrating it destructively. User-requested rollback is allowed. Metadata rollback, meaning a feed that offers an older version as latest, is rejected by TUF version checks. These are different things and stay separate.

## 8. Teams and organizations

The term "Teams" still needs clarification. Proposed scope:

- An **organization policy** is a signed document that the organization's admin publishes through its own catalog or file. It sets allowed catalogs, allowed or blocked packages, **version pins** (the same idea as `pins.rs`), capability ceilings, and whether users may add their own sources.
- **Roles:** an admin edits policy, an approver approves package versions, and members install within policy. Approval is an organization-signed attestation over a package digest, kept separate from the publisher's signature.
- **Audit** records package, version, digest, actor, decision and time. It stays local, or goes to the organization's own endpoint under the organization's configuration. **No documents, project contents, plugin inputs or outputs are uploaded.** ArtCraft does not receive the audit trail unless it hosts that organization's catalog under an explicit agreement.
- Policy can only **narrow** what the sandbox allows. It cannot give a package more capabilities than its manifest declares.

## 9. Proprietary and private packages

- **Access versus trust:** the metadata of a private catalog is still signed, so a member can verify it. Downloads use short-lived, per-user credentials (signed URLs or bearer tokens) obtained through the user's ArtCraft or organization sign-in. Credentials stay in the platform keychain, never in projects or logs.
- **Offline entitlements:** a signed, time-limited licence token names the user or seat, the package and its expiry. The host verifies it offline. Expiry blocks running *proprietary* contributions after the grace period, and the UI says why. Saved state and documents remain intact and readable.
- **Encryption does not keep code secret.** A client that runs the code must decrypt it, so the key is on the same machine and a determined user can extract the plaintext. Encrypting packages raises the effort a little at real cost (crash diagnostics, sandbox inspection, review). Proposed: deliver proprietary code unencrypted over authenticated channels. Protect it through licensing terms and entitlement checks, and keep anything that is truly secret (models, algorithms) **server-side** behind a provider contribution.
- Proprietary packages still follow the repository's licence allowlist for anything that is distributed *from these repositories*. Proprietary distribution is a requirement to design for, not an exception to that rule.

## 10. Contributor workflow and SDK manifest

The goal is a small mandatory manifest, with everything else derived.

- **Author-written, mandatory:** package ID; display name; version; licence (SPDX); contributions, each with a stable ID, contract kind and contract version range; **requested capabilities**; resource declarations where applicable.
- **Derived by the SDK at build time:** module digests, exported entry points, parameter and output schemas from typed code, supported-app list from the contracts, the SDK version, file list and sizes.
- **Added by the catalog:** publisher identity, signatures, review status, and the release timestamp. An author never fills these in by hand.
- **Automated validation** (`sdk check`, also run by catalogs): schema, contract conformance, fixtures, capability-use lint (an undeclared call means failure), size limits, reproducible build where possible, licence fields, and archive hygiene. A local-development source lets authors run unsigned packages on their own machine behind an explicit developer mode. Developer mode is never on by default for the CLI.

## 11. Distribution options

| Option | Summary | Fits | Cost and risk |
|---|---|---|---|
| **A. Official catalog only, sideloading for everything else** | One curated catalog. Others install local files. | Fastest start | Sideloading has no update, revocation or pinning; contradicts the third-party decision |
| **B. Federated signed catalogs (recommended)** | Any catalog using the same TUF-style format; official, community and team catalogs are equal in format and scoped by identity | Official, community, team, private | Catalog tooling to run; needs clear source UX |
| **C. Central official registry with publisher accounts** | All publishers upload to ArtCraft; teams get private namespaces there | Strong curation, single search | ArtCraft runs everyone's hosting, privacy and support; teams with their own infrastructure are excluded |

**Recommended defaults:** Option B, with the official catalog preinstalled and ArtCraft's proprietary team as one delegated publisher within it. Community and team catalogs are user-added, or set by organization policy, each pinned by its root key. Sandbox for everything. Licensing is a separate optional layer. A fresh CLI uses installed, enabled, policy-allowed packages whose metadata is not revoked, and never downloads anything.

## Product decisions requested

1. **Catalog model:** federated signed catalogs (B), central registry (C), or official-only plus sideloading (A)? Recommendation: B.
2. **Official privilege:** confirm that official and ArtCraft-proprietary packages get no sandbox or consent exemptions, only presentation differences. Recommendation: confirm.
3. **Offline and revocation:** allow installed packages to keep running after metadata expires (with visible status), and disable them only on explicit revocation or entitlement expiry? Recommendation: yes, with an organization-configurable grace period.
4. **Proprietary protection:** use entitlements plus server-side secrets instead of client-side code encryption, and settle what "Teams" means (organization-managed catalogs and policy, versus something else)? Recommendation: yes to entitlements; the user to clarify Teams.

## Summary for the parent

Read the decision and functional-model docs and checked craft-libs sources with bounded searches. `craft-update` already has SHA-256 checks, signer-team pinning and version pins. Same-feed digests are integrity only. No entitlement, team-policy or package-signing code was found in craft-libs (not an absence claim elsewhere). Proposed: federated catalogs using the TUF role model, packages scoped by `(catalog, ID)` with source pinning, delegated publisher keys, content-addressed immutable installs, offline running after expiry, state-preserving rollback, signed organization policy with local-only audit, short-lived download credentials with offline licence tokens, no client code encryption, and a minimal manifest with derived fields. Official packages get no privilege. Four decisions are listed above. No library, service or key is committed. The artifact was not read.

Sources: [TUF specification](https://theupdateframework.github.io/specification/), [Sigstore signing overview](https://docs.sigstore.dev/cosign/signing/overview/), [SLSA v1.1 levels](https://slsa.dev/spec/v1.1/levels), [PEP 708 discussion](https://discuss.python.org/t/pep-708-extending-the-repository-api-to-mitigate-dependency-confusion-attacks/24179).

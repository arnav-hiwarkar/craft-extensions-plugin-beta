# Implementation readiness

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: investigation complete for source and design, 2026-11-08. No code, CI, dependency, repository, key, release or gate was changed or created. Every acceptance item is future work. sg13 stays closed.

This phase asked what must change before plugin development starts. The [brief](brief.md) and the root's [Jeremy crosswalk](jeremy-crosswalk.md) set the scope. Three Haiku workers audited all sixteen apps at their committed HEADs. The coordinator read the shared extension store, launcher and keymap source and wrote the synthesis.

Read [the root assessment and first work](assessment-and-first-work.md) first for verified qualifications, the recommended starting sequence, and the proposed catalogue/search/SDK experience. Worker notes remain preserved independent evidence. Committed local HEAD is not a claim about the latest upstream code.

## Read in this order

| Document | What it answers |
|---|---|
| [System gaps and roadmap](system-gaps-and-roadmap.md) | Ranked blockers, core-feature protection status, all sixteen apps, work packages W0 to W13, PhotoCraft reconciliation, coverage of the twenty-seven aspects |
| [Catalogue and SDK](catalogue-and-sdk.md) | Release records, exact-digest review, signing, star ranking, install, update, offline, local files, SDK commands and generation from app registries |
| [User and developer workflows](user-and-developer-workflows.md) | End user, author, maintainer and app-developer steps, and how built-in features stay authoritative |
| [Evidence index](evidence-index.md) | Revisions, files read, coordinator spot checks, external sources, limits |
| [Media app audit](haiku-media-apps.md) | EffectCraft, PhotoCraft, FilmCraft, LightCraft, ArtCraft |
| [Design app audit](haiku-design-apps.md) | FigCraft, VectorCraft, PDFCraft, DesignCraft, DeckCraft |
| [Domain app audit](haiku-domain-apps.md) | SoundCraft, CADCraft, SolidCraft, TextureCraft, WordCraft, GridCraft |

## The short version

1. **The shared store must change first (B1).** Today it enables a package in every app on install, re-enables apps on update, deletes the previous release, and keeps no digest, origin or grant.
2. **A common plugin/automation authority contract is missing (B2).** Some inspected routes expose powerful operations without a plugin principal. Four desktop control channels lack a token in inspected paths. PhotoCraft/TextureCraft already have file-authority modes, and FigCraft enforces plugin command/access restrictions. Preserve those controls while adding consistent originating caller identity and scope.
3. **Built-in protection is inconsistent (B3, B9).** Some apps refuse duplicate IDs. Others replace silently or are last-wins. Most apps have no contribution seam at all.
4. **Exports can silently lose processing (B4),** and saved references carry no version or digest (B7).
5. **EffectCraft source already implements a GitHub-asset catalogue with per-release SHA-256 and offline cache.** Its release fields/behavior should inform migration into the existing shared format/store (B8). Actual live catalogue contents/review practice were not checked. Neither its inspected catalogue nor the craft-ext index has completed signature verification (B5).
6. **Three app fix sets (W3, W4, W5) can start now**, independently, each with standalone value and a failing test. The store (W1, W2) unlocks the three proofs: PhotoCraft for distribution, then EffectCraft and FigCraft for function.

Final documentation/source-scope checks are recorded in readiness `verification.md`. No implementation or external issue work began during this investigation.

## Decisions waiting on the user

Listed in [the roadmap](system-gaps-and-roadmap.md#decisions-still-open): catalogue repository and owner, update-security tooling/operators, licence list, reviewers per release, interpretation of "official first", automatic-update default, non-interactive CLI grants. The root recommends established update tooling for production rather than a temporary custom signature protocol. No account is required for public use in this design. Publishers use their GitHub account to publish assets and open catalogue PRs.

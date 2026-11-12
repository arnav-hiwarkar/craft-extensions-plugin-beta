# Jeremy's Craft Extensions handoff

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](plugins/architecture.md); where this document disagrees with it, the architecture wins.

Source: the user pasted the text of [the Claude artifact](https://claude.ai/artifact/4AyzLFyPewZYWhWBx6DZ6V) into this conversation on 2026-11-08. This document summarizes that supplied text rather than claiming the live artifact was fetched. Anonymous retrieval returned a page shell and the public frame-content request returned HTTP 403. The chart layout and its underlying source were not retrieved.

The [complete supplied text](plugins/sources/jeremy-extensions-todo.txt) and [original repository handoff archive](plugins/sources/jeremy-handoff/README.md) are now saved locally in these docs. The latter preserves the committed plan source and generated page, design notes, NEXT checkpoint, threat model, and sample packages at the inspected revision. The repository's generated page was archived from Git, not retrieved from Claude. It is not proof that the inaccessible Claude artifact has the same layout or exact source.

## Purpose and reported state

Craft Extensions is a shared browser for themes, plug-ins, and resources, embedded in Craft Launcher and reachable from the apps. First-party extensions ship first. Third-party architecture exists behind a closed release gate until all security conditions pass.

The supplied text reports 30 tasks to do, 5 partly done, 5 blocked or closed, 17 built, and 8 future-roadmap items. These are historical plan counts, not verified current completeness. Repository locations were checked on 2026-11-04; task statuses came from a plan last updated on 2026-10-16. Combined launcher files changed after that plan update.

## Reported repositories and revisions

| Work | Repository and branch | Reported revision and caveat |
|---|---|---|
| Approved Extensions UI | `storytold/craft-launcher`, `test/ext-plus-ai` | `be3bff3e26`; combined AI/UI throwaway branch, never a release source |
| Launcher feature | `storytold/craft-launcher`, `feat/extensions` | `6ede77f93f`; reported 373 commits behind main, uses sibling path dependencies |
| Shared crates and plan | `storytold/craft-extensions`, `wip/extensions-handover` | `ffd8512e5a`; start at `docs/handoff/NEXT.md`; core/store tests reported as 116 passing |
| Repository URL fix | `storytold/craft-extensions`, `chore/repo-url-storytold` | `4f11728727`; one-line unmerged fix |
| PhotoCraft adapter | `storytold/photocraft`, `feat/extensions-theme-adapter` | `ecb056cc74`; sibling path dependencies still need pinning |

These references are being checked against the live repos. They are not instructions to merge or publish unreviewed branches.

## Design recommendations in the supplied text

- Extension kinds: themes, executable plug-ins, and resources such as brush sets, LUTs, swatches, and templates.
- UI names: Extensions with Themes, Plug-ins, and Resources sections.
- Catalogue location is unresolved: confirm `storytold/crafthub` with maintainers before using it.
- Third-party signing recommendation: detached Ed25519 signatures, publisher-key pinning, signed versioned/expiring/revocable metadata, and consideration of TUF before inventing a scheme.
- Provenance recommendation: the hub builds from source and attests the result; this is costly and still a decision.
- Independent review recommendation: review unpacking, sandbox host, and signature handling before opening the gate.
- Capability model: per-app abilities declared in metadata and granted only when enforcement exists. Initial third-party v1 permissions are empty and modules have no imports.
- ThemeCraft, custom workflow JSON, wider capabilities, ranking, and additional resource kinds are future work, not completed implementation.

The pasted source says decisions d0, d3, d5, and d7 were settled. The now-archived repository plan supplies their content: launcher browser reachable from every app, first-party first with gated third-party support, shared-store folder approach, and standalone public workspace during development. See [the archive guide](plugins/sources/jeremy-handoff/README.md) and its original plan data. Other choices remain recommendations, including d1, d2, d4, d6, and d8 through d12. PhotoCraft panel placement also needs a decision.

## Security gates in the supplied text

The sg13 release gate stays closed until the prerequisite sg1 through sg12 conditions and explicit Jeremy/maintainer approval are satisfied. The excerpt explicitly describes:

- sg2: publisher signatures, pinned keys, rotation and revocation, using test keys only in the repository.
- sg3: authenticated, expiring metadata, monotonic versions, replay/rollback protection. Expiry/version parsing is reported partly built; signatures are not.
- sg4: revocations consulted at startup and before enable. An index helper exists; wiring is incomplete.
- sg6: empty permissions in initial v1, no host imports, and module validation at install and every load. Host validation is incomplete.
- sg7: fresh approval for new signing keys, more authority, or downgrades; no automatic third-party updates.
- sg8: bounded fuzzing and crash regressions for parsers and install paths.
- sg9: source provenance and a decision on trusted build infrastructure.
- sg10: review checklist, two-maintainer review, disclosure and takedown process.
- sg11: clearly labelled third-party UI compiled only with the appropriate feature enabled.
- sg12: independent security review with findings resolved or accepted explicitly.

sg1 and sg5 are referenced but their complete definitions are not included in the pasted task rows. Read the actual plan before claiming their requirements or status.

## What is not verified

The text says no real plugin has completed catalogue, installation, and host execution end to end. It reports no clean-checkout CI, candidate package, source PR, or release. Launcher UI used sample data; the PhotoCraft restyle still needs live verification. Threat-model review, translations, CI on the pushed branches, and clippy remain incomplete. Local reported test counts are not evidence that this worktree ran them.

## Recommended order from the handoff

Read live issue/decision context and refresh the plan. Settle catalogue and gate decisions. Finish first-party catalogue retrieval and verified installation, then the PhotoCraft host and invert example. Pin Git revisions instead of sibling-only paths. Only after a real first-party round trip works, extract the approved UI into a clean launcher branch and verify cold/running deep links. Package an exact candidate and obtain the required publication approvals separately.

The combined launcher UI branch must not become the release source. No code from these branches is being merged by this documentation study.

## Relationship to the present discussion

This is an existing implementation to inspect before designing replacements. The user's newer requirements include useful default CLI loading, broader versioned capabilities, executable contributions first with an extensible envelope, proprietary and community distribution, and EffectCraft/FigCraft functional validation. Reconcile these with the handoff's theme/PhotoCraft pilot and empty-permission v1 policy rather than assuming all requirements are already implemented.

Themes are in scope. Organization/team-management features are not; the user explicitly clarified that reference.

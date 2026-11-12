# Workflow coverage map

Recorded 2026-11-12. All forty historical scenario IDs are mapped below. This is explanation/design coverage, not test coverage or runtime evidence. Original labels are preserved; later decisions supersede old UUID, confinement, account/private-distribution or API assumptions. Read the [walkthrough](system-walkthrough.md) and [correction register](architecture-review-2026-11-12.md).

| Scenario | Walkthrough | Open contract or qualification |
|---|---|---|
| S01: One package supports PhotoCraft and FigCraft; only PhotoCraft is configured | [start](system-walkthrough.md#start-with-the-person-installing-a-package), [levels](system-walkthrough.md#what-l0-l1-and-l2-mean) | AR02/AR14 |
| S02: Package offers supported and unsupported contributions | [start](system-walkthrough.md#start-with-the-person-installing-a-package), [levels](system-walkthrough.md#what-l0-l1-and-l2-mean) | AR13 |
| S03: Plugin or local file claims a builtin/foreign UUID, operation or readable name | [additional](system-walkthrough.md#additional-workflows-and-boundaries) | AR14 |
| S04: Two exporters both write `.png` | [additional](system-walkthrough.md#additional-workflows-and-boundaries) | Existing handler rule |
| S05: Extension shortcut conflicts with a core shortcut | [additional](system-walkthrough.md#additional-workflows-and-boundaries) | Existing shortcut rule |
| S06: Same algorithm targets PhotoCraft bands and EffectCraft temporal frames | [transform](system-walkthrough.md#applying-an-l1-transform), [adapter](system-walkthrough.md#what-app-maintainers-build) | AR10/AR12 |
| S07: Guest requests an operation through a nested/queued CLI-like dispatch | [ops](system-walkthrough.md#commands-panels-and-permission-checks-in-l2), [safety](system-walkthrough.md#safety-claims-and-their-limits) | AR06/AR09 |
| S08: Benign-looking command parameter reads a file or uploads content | [ops](system-walkthrough.md#commands-panels-and-permission-checks-in-l2), [safety](system-walkthrough.md#safety-claims-and-their-limits) | AR08/AR11 |
| S09: Required rights denied; optional network right denied | [ops](system-walkthrough.md#commands-panels-and-permission-checks-in-l2), [safety](system-walkthrough.md#safety-claims-and-their-limits) | AR06 |
| S10: Update keeps or narrows rights, then another update widens them | [store](system-walkthrough.md#update-disable-remove-and-cleanup) | AR01/AR02 |
| S11: New contribution is added to a previously granted package | [store](system-walkthrough.md#update-disable-remove-and-cleanup) | AR02/AR13 |
| S12: Repeated install/retry after a crash between bytes and state publication | [store](system-walkthrough.md#update-disable-remove-and-cleanup) | AR03/AR04 |
| S13: Launcher updates/removes while a long CLI export lazily reads a LUT | [store](system-walkthrough.md#update-disable-remove-and-cleanup) | AR01/AR05 |
| S14: Old client writes after a new authority-bearing state migration | [store](system-walkthrough.md#update-disable-remove-and-cleanup) | AR03 |
| S15: Publisher replaces bytes under the same version/tag/filename | [distribution](system-walkthrough.md#distribution-and-incident-response), [author](system-walkthrough.md#what-plugin-authors-build) | PR #5 / AR14 |
| S16: Publisher uploads a new release but catalogue PR is not approved | [distribution](system-walkthrough.md#distribution-and-incident-response), [author](system-walkthrough.md#what-plugin-authors-build) | Approval bound to exact artifact |
| S17: Signing authority changes, metadata is replayed, or time goes backward | [distribution](system-walkthrough.md#distribution-and-incident-response), [author](system-walkthrough.md#what-plugin-authors-build) | TUF/incident-state persistence |
| S18: Publisher/repository ownership transfers | [distribution](system-walkthrough.md#distribution-and-incident-response), [author](system-walkthrough.md#what-plugin-authors-build) | AR14 |
| S19: Repo stars are stale/manipulated/missing; repository hosts multiple packages | [distribution](system-walkthrough.md#distribution-and-incident-response), [author](system-walkthrough.md#what-plugin-authors-build) | Later ranking; no trust implication |
| S20: Public versus private GitHub assets and cross-origin redirects | [distribution](system-walkthrough.md#distribution-and-incident-response), [author](system-walkthrough.md#what-plugin-authors-build) | Private distribution/account path out of first release |
| S21: Local file has no catalogue review or signature | [distribution](system-walkthrough.md#distribution-and-incident-response), [author](system-walkthrough.md#what-plugin-authors-build) | PC02; listing evidence remains separate |
| S22: Known malicious digest is imported as a local file while offline | [distribution](system-walkthrough.md#distribution-and-incident-response), [author](system-walkthrough.md#what-plugin-authors-build) | Cached digest blocks and incident persistence |
| S23: Development client and release client share store contents | [distribution](system-walkthrough.md#distribution-and-incident-response), [author](system-walkthrough.md#what-plugin-authors-build) | Separate dev grants/admission |
| S24: Reopen on another machine with a missing or incompatible live effect | [project](system-walkthrough.md#save-reopen-bake-and-export) | AR12/AR15 |
| S25: Explicit migration fails or rollback state is incompatible | [project](system-walkthrough.md#save-reopen-bake-and-export) | AR12/AR15 |
| S26: Preview can fall back but final export requires unavailable processing | [project](system-walkthrough.md#save-reopen-bake-and-export) | AR05/AR15 |
| S27: Plugin job completes after the document/selection changed | [ops](system-walkthrough.md#commands-panels-and-permission-checks-in-l2), [project](system-walkthrough.md#save-reopen-bake-and-export) | AR07/AR09 |
| S28: Undo follows a plugin edit that also wrote a file or uploaded data | [ops](system-walkthrough.md#commands-panels-and-permission-checks-in-l2), [project](system-walkthrough.md#save-reopen-bake-and-export) | AR08 |
| S29: Panel closes during callbacks, tool loses capture, widget is missing on reopen | [ops](system-walkthrough.md#commands-panels-and-permission-checks-in-l2), [project](system-walkthrough.md#save-reopen-bake-and-export) | AR06/AR09 |
| S30: UI-only contribution is invoked headlessly; job owner exits too early | [ops](system-walkthrough.md#commands-panels-and-permission-checks-in-l2), [project](system-walkthrough.md#save-reopen-bake-and-export) | AR05/AR08 |
| S31: Malformed brush/LUT/SVG/template, bad workflow or missing model/node | [additional](system-walkthrough.md#additional-workflows-and-boundaries), [store](system-walkthrough.md#update-disable-remove-and-cleanup) | AR12/AR16 |
| S32: Theme passes package validation but loses contrast/selection meaning in an app | [additional](system-walkthrough.md#additional-workflows-and-boundaries), [store](system-walkthrough.md#update-disable-remove-and-cleanup) | PC09 token contract still open |
| S33: Huge model/resource, disk full, partial download or resource removed during import | [additional](system-walkthrough.md#additional-workflows-and-boundaries), [store](system-walkthrough.md#update-disable-remove-and-cleanup) | AR01/AR11/AR12 |
| S34: Temporal processor seeks backward or two exports share a pool | [additional](system-walkthrough.md#additional-workflows-and-boundaries), [transform](system-walkthrough.md#applying-an-l1-transform), [safety](system-walkthrough.md#safety-claims-and-their-limits) | AR10/AR19 |
| S35: GPU device loss, arbitrary shader request or native helper on unsupported OS | [additional](system-walkthrough.md#additional-workflows-and-boundaries), [transform](system-walkthrough.md#applying-an-l1-transform), [safety](system-walkthrough.md#safety-claims-and-their-limits) | AR17/AR19; native confinement optional |
| S36: Audio callback deadline fails or plugin worker crashes | [additional](system-walkthrough.md#additional-workflows-and-boundaries), [transform](system-walkthrough.md#applying-an-l1-transform), [safety](system-walkthrough.md#safety-claims-and-their-limits) | Later real-time contract |
| S37: Guest loops, host call blocks, output/events/logs grow without bound | [additional](system-walkthrough.md#additional-workflows-and-boundaries), [transform](system-walkthrough.md#applying-an-l1-transform), [safety](system-walkthrough.md#safety-claims-and-their-limits) | AR07/AR10/AR11 |
| S38: Corrupt state, repeated startup failure, quarantine and opt-in support report | [additional](system-walkthrough.md#additional-workflows-and-boundaries), [store](system-walkthrough.md#update-disable-remove-and-cleanup) | AR03/AR18 |
| S39: Restricted/proprietary licence or bundled dependency conflicts with selected policy | [additional](system-walkthrough.md#additional-workflows-and-boundaries) | Existing catalogue/local licence distinction |
| S40: Two plugins want to call each other or run a resident service | [additional](system-walkthrough.md#additional-workflows-and-boundaries) | Future delegation/peer-call contract |

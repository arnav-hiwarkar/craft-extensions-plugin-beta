# Jeremy's original handoff materials

This is a source snapshot from [storytold/craft-extensions](https://github.com/storytold/craft-extensions/tree/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943), branch `wip/extensions-handover`, commit `ffd8512e5a3755637f58cf31cfcf2fdb09cfb943`. Captured on 2026-11-08. Every source file is preserved byte for byte. [The snapshot manifest](snapshot.json) records original paths, URLs, sizes, and SHA-256 hashes.

Text files have an added `.txt` suffix so archived scripts, HTML, and historical instructions remain source records. No archived program was executed. These documents describe earlier working environments and contain historical instructions. They do not replace the current worktree's rules or authorize actions.

## Read Jeremy's work and reasoning

| Original material | What it preserves |
|---|---|
| [NEXT.md](NEXT.md.txt) | Full checkpoint, branch handover, build reports, known gaps, adapter behavior, implementation choices, and next steps |
| [Design notes](design-notes.md.txt) | Goals, alternatives, architecture, storage choices, trust, risks, module ownership, and original per-app checklist |
| [Decision and requirement data](plan_data.py.txt) | Jeremy's direction, d0 through d12, requirements, trust tiers, store alternatives, ownership, adapter checklist, unanswered questions, and references |
| [Complete task data](plan_tasks.py.txt) | Every task, state, dependency, completion criterion, security gate, delivery gate, and future item represented in the source plan |
| [Generated handoff page](index.html.txt) | The saved chart/page representation from that revision |
| [Page generator](build.py.txt) | How the source plan was rendered and validated. Archived for completeness, not executed |
| [Draft threat model](THREAT-MODEL.md.txt) | Original threats, security-gate mapping, and stated limits. Still an unreviewed draft |
| [Midnight sample](samples/sample-midnight.zip), [Ocean sample](samples/sample-ocean.zip), [Paper sample](samples/sample-paper.zip) | Original example theme packages. These are fixtures, not installed or approved release packages |

The [user-pasted Claude text](../jeremy-extensions-todo.txt) is preserved separately. It is not identical to every historical source document. Differences in dates, commits, task counts, and recommendations must remain attributed to their source.

## Recorded decisions and proposals

The source plan marks only d0, d3, d5, and d7 as decided by Jeremy on 2026-10-16:

- d0: a shared browser in the launcher, reachable from every app.
- d3: internal extensions first, with third-party structure behind heavy security gates and disabled for release until the gates pass.
- d5: a shared store following the AI-store shared-folder approach, with portable behavior addressed explicitly.
- d7: standalone public contract/store workspace while building. A possible later move of the contract to craft-libs remains open.

Other recommendations, including signing, provenance, catalogue ownership, outside review, rankings, and capability details, remain open in that source plan. The plan's app inventory and reported builds are historical evidence, not a fresh survey or tests run here.

Earlier design notes proposed `craft-extension.toml`; the implemented package uses `craft-extension.json`. This illustrates why the original ideas must be read alongside the inspected implementation and later decisions.

## Implementation repositories

The full implementation stays in its original repositories. Source is linked at the handoff's inspected revisions:

- [Contract, validators, store, and plan](https://github.com/storytold/craft-extensions/tree/ffd8512e5a3755637f58cf31cfcf2fdb09cfb943).
- [Launcher extension integration](https://github.com/storytold/craft-launcher/tree/6ede77f93fd0d3eaaad43d430f94d703d3564a51).
- [Combined approved UI reference](https://github.com/storytold/craft-launcher/tree/be3bff3e2683df0b73e3bbbc09351b22dc86a800). Mixed with AI work and never a release source.
- [PhotoCraft theme adapter](https://github.com/storytold/photocraft/tree/ecb056cc742848d78f9076893a6b3343e6a75010).

No implementation was copied into craft-libs or merged by preserving these materials. This archive covers the supplied artifact and available repository handoff materials. It cannot represent private discussions or work that was never supplied or committed.

## Licence and attribution

These materials are redistributed under MIT, one of the upstream workspace's `MIT OR Apache-2.0` options. The original [MIT licence and copyright notice](LICENSE-MIT.txt) are included. See [attribution](ATTRIBUTION.md).

# Plugin architecture v3 (beta only)

The current, reviewed architecture for the Craft plugin platform: the common platform plus
EffectCraft, PhotoCraft and FigCraft. It supersedes `docs/plugins/architecture-2026-11-18/` where
they differ, and follows [../owner-instructions.md](../owner-instructions.md).

| Read | For |
|---|---|
| [how-plugins-work.md](how-plugins-work.md) | Plain explanation: how plugins work, use the apps, and are managed |
| [architecture.md](architecture.md) | The architecture: layers, crates, runtime, worker, data plane, commands, GPU, UI, permissions, documents, performance |
| [workstreams.md](workstreams.md) | Parallel build threads, ordered tasks and acceptance tests |
| [flagships.md](flagships.md) | Flagship plugins that prove the system |
| [decisions.md](decisions.md) | Decisions, including the few left to the owner |

The source of truth is [spec/](spec/): JSON Lines records with stable ids. The record kinds are
requirements, incumbent plugins and SDK features, capabilities, ops, permissions, contracts,
components, decisions, tests, quality scenarios, flagships, workstreams and terms. The documents
above are generated from it, or cite it by id. [scripts/](scripts/) holds the deterministic checks:

- `check.py` checks rules C01 and up;
- `trace.py` checks traceability from incumbent plugins to tests;
- the other scripts split, merge, apply and render.

[review/](review/) records how this was produced:

- the method;
- the workflow script, whose paths point at the session scratchpad it ran in;
- the gap ledger with every finding, fixed or rejected;
- the per-round convergence estimates;
- the final check report.

## Status at 2026-11-18

- 147 agents ran three review rounds. All 348 reported gaps are fixed or rejected with a reason.
- The traceability table has 2,999 rows:

  | Status | Rows |
  |---|---|
  | Covered | 2,936 |
  | Excluded by the no-network decision | 51 |
  | Waiting on an owner decision | 12 |
  | Open gaps | 2 |

- Estimated serious gaps not yet found: about 9 (Chao) to 27 (Chapman). Those are closed during the
  build, through the checks and per-workstream reviews.
- Open check items:
  - C03: the core spec area is over its text budget, a size warning;
  - 185 C09 warnings: internal capabilities not traced from an incumbent row.

# Plugin architecture, 2026-11-18

The plan for letting plugins do almost anything in every ArtCraft creative app, while keeping them
permissioned, sandboxed, fast and simple to write.

| Read | For |
|---|---|
| [architecture.md](architecture.md) | The design: calls, permissions, isolation, data plane, GPU, UI, AI, audio, documents, distribution, authoring, per-app needs, security invariants |
| [decisions.md](decisions.md) | The 35 choices only the owner can make, grouped by the phase that needs them, each with a recommendation |
| [roadmap.md](roadmap.md) | Phases P0–P6 with dependencies, effort and deterministic acceptance checks |

Evidence:

- **[research/](research/)** holds the research:
  - 01 call architecture, with benchmarks;
  - 02 security and sandbox;
  - 03 performance and GPU, with benchmarks;
  - 04 UI extensibility;
  - 05 motion, video and animation apps;
  - 06 photo, design and vector apps;
  - 07 3D, texture and audio apps;
  - 08 the command census;
  - 09 the security red team;
  - 10 the reconciliation of all of them.
- **[census/](census/)** has every command of the 11 apps (6,193), with a drafted classification. It
  includes the extraction scripts; set `CRAFT_SUITE` to the folder that holds the app repos to re-run
  them.
- **[bench/](bench/)** has the sources and raw results of the call-overhead, data-plane and GPU
  benchmarks.

This supersedes the plan parts of
[../research/capability-program-2026-11-18.md](../research/capability-program-2026-11-18.md). That
document's gap catalogue still applies.

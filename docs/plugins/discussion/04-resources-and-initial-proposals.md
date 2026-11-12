# Resources and initial proposals

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Historical discussion archive, captured on 2026-11-08. This preserves technical requests and substantive responses; routine progress updates and tool logs are omitted. Local checkout links are normalized. Later decisions and source corrections take precedence. Start with [the current documentation index](../README.md).

## User request

we have some stuff done here, add these resources to the worktree rn and save them in docs as well, lets see how it works:

https://chatgpt.com/share/6aeeff72-c490-83e8-8228-139f6225c5db

https://claude.ai/artifact/4AyzLFyPewZYWhWBx6DZ6V

https://github.com/storytold/craft-libs

---
we need to deal with different types of plugins, what they do. ensure that they dont change the app, how to ensure that they can properly use the application and do stuff on it.
along with all the types, cuz some plugins work better if not wasm, like need resources and gpu etc.
spawn opus and start thinking, not a lot of tokens, come to a few ideas and stuff

## Research response 1

Saved in this worktree:

- [Resource register](../../../docs/plugin-resources.md), with all three links and existing implementation evidence.
- [Architecture ideas](../../../docs/plugin-architecture-ideas.md), from one brief Opus pass, reviewed and refined.

The proposed direction is one package and permissions contract supporting:

- **WASM** for portable effects and automation.
- **Host-managed GPU work**, which WASM can also request.
- **Restricted native helpers** for vendor SDKs and heavier workloads. A separate process alone provides insufficient protection.

Apps expose validated operations, controlled data, and designated UI areas. Plugins extend those interfaces without accessing private app state or modifying the installation.

The ChatGPT and Claude links returned page shells, so their **contents remain uncaptured**. Their references and this limitation are recorded. Documentation links and whitespace checks passed; no application code changed.


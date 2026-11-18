# Owner instructions for the plugin beta

These are the owner's standing instructions for building the Craft plugin platform. Every agent and
contributor working in the plugin-beta workspace follows them. They are beta-only. They and the rest
of `docs/plugins/beta-only/` are stripped before any merge to the official upstream repositories (see
[UPSTREAM-EXCLUDE.txt](UPSTREAM-EXCLUDE.txt)).

Recorded 2026-11-18. The newest instruction wins where two conflict.

## Goal

- **One complete, end-to-end plugin system for the whole ArtCraft suite.** Plugin makers can build any
  plugin, for any app, with **zero compromises on capability**. Anything the app can do (the level of
  freedom its MCP server and CLI have), a plugin can do. That includes:
  - windows, docking panels, menus, tools and overlays;
  - changing application behaviour;
  - using the application for every little nuance a plugin needs.
- **Replace, don't add.** The new system overhauls and replaces every existing plugin system and plugin
  architecture in the current application releases. One shared platform, and one adapter per app.
- **The bar is the incumbents.** It must support the biggest After Effects, Photoshop and Figma plugins
  (and their equivalents for the other apps): UI, docking, windows, functionality and usability, at the
  same speed or better.
- **Fast, efficient, optimized, snappy.** Apps must stay fast, must not crash, and must work as
  intended.
- **Maintainable and scalable.** Common architecture across all apps; behaviour lives in one place.

## No deferrals

- **Do not defer any feature or functionality** to "after this is built". If something cannot be built
  today, fix the blocking problem and build it properly.
- **Do not make the system restrictive.** Every little nuance is decided to suit the application, not to
  limit plugins.

## Security posture

- **Be as secure as reasonably possible**, but functionality and ease of use come first.
- **Permissions exist and are enforced.**
- **Third-party plugins are the user's responsibility.** Where malicious behaviour cannot be tracked or
  prevented, warn the user clearly that it is a third-party plugin and that we are not responsible for
  it. Do not withhold capability for that reason.

## Order of work

1. **The common architecture first:** one shared platform, the same main structure in every app.
2. **EffectCraft, PhotoCraft and FigCraft:** plugin adapters, app changes, and the capabilities and
   functionality those three apps need.
3. **When those three are done:** FilmCraft and the other apps (AniCraft, VectorCraft, LightCraft,
   DesignCraft, MaterialCraft, TextureCraft, SoundCraft).
4. **Build big flagship plugins** that reproduce the most popular After Effects and Photoshop (and
   Figma) plugins. Use them to prove the system: they must work as intended, fast, simple and snappy.

## How to work

- **Architecture and planning come before building.** Research, plan and review in loops until the
  architecture is complete and has zero capability compromise. Then explain it to the owner in depth:
  - how plugins work;
  - how they use the application;
  - how they are managed.

  Only after that, build.
- **Split the build into parallel workstreams.** For example:
  - app changes and adapters;
  - the common platform;
  - SDK and commands;
  - flagship plugins.
- **Use Opus 5.5 at high effort for analysis, planning and reviews.** Use the available skills,
  workflows and subagents. Run review loops that check:
  - what is missing;
  - what to add or change;
  - what to decide;

  against After Effects, Photoshop, Figma and other big applications and their plugins. Be considerate
  with tokens: don't launch more agents than the work needs.
- **Compact the conversation after about 350k tokens.**
- **No code comments; the code is the proof.**
  - Use deterministic checks.
  - Put learned best practices into AGENTS.md and the codebase architecture.
  - Design APIs that remove nuanced decisions from callers.
  - Keep things simple for agents to use and maintain.
- **Keep reviewing until everything is finished properly.**

## Repositories and git

- **Upstreams are read-only.** Never push to, or modify, anything under the `storytold` GitHub
  organisation.
  - Never force-push or reset away beta work.
  - Never use sync-fork blindly.
  - Use only `arnav-hiwarkar/*` repositories and the owner's Forgejo.
  - Always pass the explicit repository to `gh`.
- **Plugin-beta repos live together in `/Users/ash/Projects/plugin-beta/`**, so path dependencies work.
  - Each has a `plugin-beta` branch with remotes `forgejo` and `github`, and pushes go to both.
  - The original six GitHub repos and the nine added on 2026-11-18 are public; Forgejo copies are
    private.
- **Do not reclone or modify the owner's other checkouts in `~/Projects`.**
- **Commits:**
  - author `Arnav Hiwarkar <arnav-hiwarkar@users.noreply.github.com>`;
  - a truthful `Co-Authored-By` trailer for the model that did the work;
  - explicit paths when staging;
  - `[skip ci]` in every subject unless a run is wanted.
- **Never read, copy or print signing keys.** Never commit binaries or caches. No messages to others,
  no pull requests or issues unless asked.

## Builds and CI

- **Keep the Mac usable.** Only cheap, scoped checks run locally (`cargo check`, clippy, focused tests),
  with one build owner at a time.
- **Builds and tests run in GitHub Actions on the owner's forks.** Authorized 2026-11-18.
  - Choose debug or release per job: debug for tests and app binaries, release for packaged plugin
    modules and helpers.
  - Keep CI from taking too long: parallel jobs and caches.
- **Do not touch the installed beta apps and settings in `~/craft-beta`** beyond what a test step
  needs, and never touch the regular installed apps.

## Decisions already made (see architecture decisions.md)

| Area | Decision |
|---|---|
| Calls | Shared canonical ops plus a bridge to every app command, behind one gate |
| Execution | Compiled plugin code runs in a sandboxed worker process |
| Permissions | All approved once at setup; selection changes are their own permission |
| Interfaces | WIT deferred as an interface language (an implementation choice, not a feature deferral) |
| Reviews | The owner reviews permission widenings |
| Preferences, app network | Both allowed, with permissions |
| Web UI | Web pages allowed in their own sandboxed window |
| App order | Video apps first after the first three |
| Legacy systems | Hard cut for legacy plugin systems |
| Material apps | GPU allowed in MaterialCraft and TextureCraft through the shared crate |

## Network plugins are out of scope for now (2026-11-18)

The owner chose to skip plugin network access entirely in this build. That covers:

- `net.fetch` and `network:<host>`;
- `net.local` loopback;
- app-owned network commands for plugins;
- secrets and OAuth, which exist only for network use;
- remote content in web windows;
- network model downloads.

Plugins work offline. Models, assets and web-window pages ship inside the package or come from files
the user picks. This is an explicit owner decision, not a deferral. The store and catalogue keep using
the network on the host side.

The no-deferral rule above overrides any "deferred", "later" or "not now" in earlier planning
documents. Those items are in scope.

## Documents that never go upstream

These planning and instruction documents exist for the beta build only. Before merging to official
main repositories, remove everything listed in [UPSTREAM-EXCLUDE.txt](UPSTREAM-EXCLUDE.txt).

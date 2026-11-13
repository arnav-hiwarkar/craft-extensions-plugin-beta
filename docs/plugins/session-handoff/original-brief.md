# Takeover: build the Craft plugin system beta (you own all decisions)

You are taking over the Craft (ArtCraft suite) plugin-system work from a Mac session. **Work fully autonomously. Never ask the user for input.** When a decision comes up, choose the smartest, simplest option for the whole suite, its users and plugin makers, record it, and keep going.

## What is already set up (on this machine, Archbox)

Workspace: `/home/ash/projects/plugin-beta/` (this project's root). Each repo is on branch `plugin-beta`, with CI workflow files removed. Each has remotes `github` (private, `git@github.com:arnav-hiwarkar/<name>.git`) and `forgejo` (private, `forgejo:ash/<name>.git`). `upstream` points to `storytold/*`; **never push to upstream or origin=storytold**.

| Dir | Private repo name (GitHub + Forgejo) | Role |
|---|---|---|
| `craft-extensions/` | `craft-extensions-plugin-beta` | Shared plugin platform crates (`craft-ext-core`, `craft-ext-store`, …) and ALL design docs (uncommitted Mac docs committed here) |
| `effectcraft/` | `effectcraft-plugin-beta` | First app adapter (primary target) |
| `photocraft/` | `photocraft-plugin-beta` | Second app adapter |
| `figcraft/` | `figcraft-plugin-beta` | Third app adapter |
| `craft-launcher/` | `craft-launcher-plugin-beta` | Launcher: browse/search/install/manage plugins |
| `craft-plugins/` | `craft-plugins` (main) | The store catalogue repo (metadata only) |

GitHub Actions are disabled on all six repos. Do not add CI. Commit with `-c user.name="Arnav Hiwarkar" -c user.email="arnav-hiwarkar@users.noreply.github.com"`, and push each finished step to **both** `github` and `forgejo`. Use plain branch pushes; no PRs, issues or releases. `gh` is authenticated as arnav-hiwarkar, and `ash` has passwordless sudo.

## Read first (in craft-extensions)

1. `START-HERE.md`, then `docs/plugins/README.md`.
2. `docs/plugins/architecture.md`, the authoritative design.
3. `docs/plugins/system-guide.md`: a plain-language walkthrough of Parts 1–18 and workflows W1–W10, plus review findings R1–R15. Fix R2 (ABI memory ownership), R3, R4 and R5 as part of the work.
4. `docs/plugins/implementation-plan.md` and `docs/plugins/implementation/P0–P12`.
5. `docs/plugins/draft/decisions.md`.
6. `docs/THREAT-MODEL.md`.
7. `docs/plugins/sources/craft-framework-theme-plan.md`: the craft-libs theme/framework plan. Themes are deferred, but don't make that work harder. Use one theme format, `craft-theme`, and don't grow the ext-core palette.

Skim purposefully. Don't re-read everything into context.

## Hard rules that override older docs

- **No existing app plugin system is kept or used as a base.** This covers EffectCraft (`crates/plugin`, Plug-in Library, helper-host, wasm `ec_render`), FigCraft's manifest API levels, PhotoCraft's filter ABI, and any others. Remove them and replace them with the new shared platform plus one proper adapter per app. The apps' own engine pieces (command registry, undo, render pipeline, project format, UI) are what the adapters wrap. SoundCraft VST/CLAP/AU and CADCraft AutoLISP are out of scope for this beta.
- **The user's latest choices below win over older records.** That includes the earlier two-maintainer native approval and outside-review gates: for the beta store, any author can submit and one maintainer simply allows it. There is no review bot. Record each superseded decision in `draft/decisions.md` and the session record.
- **Resource limits: be light on the machine** (8 cores, 7 GB RAM).
  - Do **not** run `cargo build`, `cargo run`, or full test suites.
  - Use `git`, `cargo check` (scoped with `-p <crate>` where possible) and small targeted `cargo test -p <crate>` only when cheap.
  - Prefer getting the code right by reading and reasoning.
- **Be token-conservative.** Spawn few subagents, scope each one narrowly, and give it a precise brief.

## Model use

- **Main thread:** you (Opus 5.5 medium). You make design decisions together with `gpt-6.1-sol` at **high** effort, used as the design reviewer and partner.
- **Coding and large writing subagents:**
  - Mostly `gpt-6.1-sol` medium, `claude-haiku-5-5` xhigh, and `gpt-6-luna`.
  - Use `claude-opus-5-5` medium for genuinely hard coding tasks.
- **When a provider hits its limits:** if Claude is exhausted, use GPT models, and the reverse.
- **Concurrency:** keep it small, about 2–3 concurrent children at most.

## Method for every step (phase by phase)

For each part:

1. Identify the problem.
2. Design a solution.
3. Review it against the **whole suite** (EffectCraft, PhotoCraft and FigCraft now, all ArtCraft apps later) and the plugin manager, using sol-high as reviewer.
4. Decide.
5. Implement.
6. Run `cargo check`.
7. Commit and push to both remotes.
8. Record decisions in `docs/plugins/draft/decisions.md` and the session record, and keep `architecture.md` and `system-guide.md` in step.

Suggested order (adjust if you find a better one):

1. Shared core in craft-extensions: manifest, IDs, versioning and compat, permissions, store and state, catalogue client, wasm runtime and SDK, native helper protocol, and the shared manager UI plus `plugin` CLI.
2. EffectCraft adapter, with the old system removed. This is the full MVP: data add-ons, transforms (effects), session plugins with UI panels and windows.
3. The `craft-plugins` catalogue format plus the submission/allow flow.
4. Launcher integration.
5. PhotoCraft and FigCraft adapters.
6. Example plugins and author docs.

## What the user asked for (their words, condensed; full intent below)

- **Native and sandboxed plugins.**
  - Sandboxed means WASM, using wasmi per the architecture, with full creative reach through permissioned host APIs.
  - Native is out-of-process with full access and is clearly labelled.
- **Permissions in the apps.** Plugins declare permissions. The user allows or denies each one in the app, and the host enforces them at every call. Revocation must work.
- **SDK and app adapters.**
  - A proper SDK format plus one adapter per app, covering the big features so wasm plugins can do whatever the app allows, under permissions.
  - Plugins can create windows and panels, add new effects or filters, add tools, commands and data (presets, LUTs, fonts, and so on), and react to app events.
  - Plugin makers must easily discover what each app offers: generated per-app contract docs, a template, and `plugin new`/build/test/pack tooling.
- **Cover every point of view** with concrete workflows: plugin maker, end user, app maker (adapter author) and store maintainer.
- **Catalogue and store** in the `craft-plugins` repo (metadata only):
  - We do **not** host plugin files. They come from the authors' own GitHub Releases, pinned by digest.
  - Authors register on the store as vendors.
  - Tiers: **official**, **trusted/verified vendor**, and **community**. Use a proven model like Windows/Apple trusted publishers or code signing: signed metadata, verified vendor identity, and a badge.
  - Show GitHub stars.
  - Users get **search** in the app manager, the launcher and the CLI.
  - User **reports**: repeated reports trigger a review, and a **kill list** (revocation) disables the plugin everywhere.
  - **Free and open source only.** Enforce an OSI-approved licence, public source, and no paid or monetised plugins as a listing requirement, checked automatically when the catalogue entry is validated.
  - Low maintenance: anyone submits, and one maintainer allows it. No review bot.
- **Versioning.**
  - Plugins use semver.
  - Users can pin or choose an older plugin version, and can roll back.
  - **App compatibility is expressed against contract versions**, not raw app versions. A plugin built for app X keeps working when the app moves forward but the contract it uses is unchanged. Apps declare the contract versions they provide, and plugins declare the contracts and ranges they need.
  - **Multi-app plugins** declare per-app support and ranges.
  - Clear UX for "this app version doesn't support this plugin version": offer an older compatible version, or say why.
- **Projects that use plugins.**
  - Projects record the plugins they use (ID, version, source).
  - Opening a shared project **never auto-downloads or auto-enables** anything. The recipient is told which plugins are used and asked "download? enable?", with permissions shown.
  - Missing plugins degrade safely and keep the data.
- **Manage plugins without the launcher.** The app's own manager and the `plugin` CLI can do everything: search, install, update, enable, permissions and remove. The launcher can do the same.
- **Simple and smart.** Avoid complexity. Choose proven patterns. Optimise for scalability across all ArtCraft apps, maintainability, and a seamless experience ("download a plugin and start working").

## Done means

- A coherent, compiling (`cargo check`) beta implementation pushed to all repos on both remotes.
- EffectCraft is the fullest. PhotoCraft and FigCraft have working adapters on the shared core. The launcher has plugin management.
- `craft-plugins` has the catalogue schema, submission docs, the allow flow, the licence check, and example entries.
- At least two example plugins exist (a data add-on and a wasm effect/filter usable in more than one app), plus author docs.
- Docs (architecture, decisions, system guide, an author guide and a user guide) match the code.
- Finish with a short status file `/home/ash/projects/plugin-beta/STATUS.md`: what's done, what's stubbed, and the next steps.

Work through the night. If you hit a blocker, pick the best workaround, note it in STATUS.md, and continue.

---
### The user's full request (verbatim, for reference)

I want you to use the best judgement for a plugin system for the entirety of artcraft And all the applications and all its users. Cover all bases and everything we've talked about. We should have native and sandbox plugins. We should have an I permission system in the apps and ensure proper permissions. We should have a good system of catalogs and how we're gonna catalog, distribute, and share. A proper SDK format and SDK ideas with the adapter for applications, how they're gonna work and how to properly use them such that we cover all the big features and let the Boxed Wasm plugins use everything properly with permissions and full creativity. Allowing plugin makers to make whatever they want for the application. We will start by making a data version and a good base for the common architecture in craft-plugins and effectcraft to being with. Follow proper Interactions and actual use case for all different POVs as a plugin maker as the user as the application makers etcetera And following proper methods which we have discussed, the best implementation for the system we want to make it a good, safe system is very good and easily accessible and easy to work with for users. We will start by making it for craft extensions the base and the effect craft app. Also cover other things such as how plugins can work with the applications create windows, create new plugins After effect stuff like that into effect craft. And also how plugins should work in general. Currently we're doing a prototype beta test for the plugin management system for Effect Craft, but eventually it has to work with all of the apps in ArtCraft. I want you to build it step by step, face by face in a proper decided manner. The way to use sub-agents for each task. There is something new: identify the problem, come up with a good solution. Review the solution in terms of the entire suite of applications as well as the plugin manager, and then come up with a good implementation for it. Want you to create the entire beta where the plugin management system properly, thoroughly covering all bases and all questions. It should be a seamless thing. Plugin managers know how to make plugins; we know how plugins work. Plugins can just be put on a store. We know how our store works—can be downloaded by the user, they'll have proper enablement permissions and all of that, and everything should be enforced. The user can just download a plugin and start working. Along the way, if you have to take any decisions, take whatever is the smartest decision and the best decision in terms of the entire system in terms of scalability, Artcraft as a whole, and how plugins should work ideally in the best case scenario. The store part for the plugins is a little unknown right now because we still don't know how to properly catalog them, but for now you should just have Repo which will be craft plugins. Something similar where you'll store the plugins and we'll come up with a good way of managing multiple plugins and how the approval system should work. We don't want a lot of maintenance and review on our hands. We just want it to be okay, plugins should comply with one we should make sure no one can monetize or benefit from those plugins. They should be free and open source for everyone, so we have to enforce that if they want it to be on the store. Then we want to be able to have good versioning for the plugins. We want to be able to have plugins working for multiple apps, so we also have to design for that. So along with Effect Craft, go with Photographed and Fig Craft as well. Extensions and plugins which we're first starting with for this one. We want to have types of plugins in the Store so there will be official plugins, community plugins, and so on. We'll also want to keep a system that's proven and used by a lot of big plugin management systems and plugin systems in a lot of applications, such as using trusted Windows as a system, how to integrate trusted vendors, and how to integrate that. And having GitHub stars, we will not be hosting any of the plugins ourselves, so they'll be taken from the plugin makers' GitHub directly. Then having a proper system where the plugin vendors will have to or plugin makers will have to register on the store. We'll also have user reports on the store so if a plugin is reported to be bad multiple times we can put it on some sort of a kill list. Then we also have to ensure versioning properly with the plugins works when there's another version of the plugin or if someone wants to use a different version of the plugin or what happens if an app moves forward there should also be like okay it was designed for this version of the application but the application doesn't change these parts of the plugin still works so plugin ability for application version should be one thing also when plugins support multiple applications what versions do they support of that application. If the application does not support that plugin, that version of the application doesn't support that plugin version, and stuff like that will happen. How to properly deal with that? How do we have projects which work with plugins? We don't want to share plugins automatically download if someone is using plugins and they share the project. It should tell the other person who is opening it that these plugins are used. Do you want to download them? Do you want to enable them? That should happen for the other user as well. And any other decision that comes along, I want you to properly think about it with the most logical and reasonable solution which works in our situation the best with all of our applications and all of our users. Which doesn't make it very complicated keeps the solutions simple and smart. [...setup instructions handled above...] For now, don't think a lot about reviewing the plugins on the store. Let's just have any user upload their plugin on the store and keep it like that. Once they upload it on the store, someone will just allow it. We won't have a review bot or anything right now, or any permissions or anything for that. Yeah, the plugin itself should have permissions which the user will allow or not be enforced. And the users should be able to have search on the catalog and get plugins properly, and plugins should also be able to install and manage without the launcher. [...] the user can also install, manage, and search plugins and everything, the application as well as the launcher. [...] Properly implemented in the best interests of the application and best software practices in accordance with how we should improve on those and use those to make the best plugin management system and SDK. Users and plugin makers and everyone involved. And for now, dont keep ci and all in my forks and forgejo, just build properly. Don't perform a lot of big commands in system usage. Just design properly and implement the changes in code and systems. Don't try to build—test the software and everything. A lot of time and a lot of resources, instead just stick to the code and get the implementation right. Easier commands like git, cargo check, etcetera—try to avoid cargo run or cargo build. And just be smart about the way you're working. Done simply—don't try to run a lot of big commands, just be straightforward, direct, simple, fast, and smart.

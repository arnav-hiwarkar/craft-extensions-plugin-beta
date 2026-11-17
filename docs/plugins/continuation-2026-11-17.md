# Mac continuation: upstream integration and suite plugin replacement

Recorded 2026-11-17. Start here on the existing Mac workspace. This record preserves
completed work and starts the next implementation phase; it is not a claim that the
new SDK backlog or full GUI workflows are complete.

## Latest user direction

The user explicitly requests a new Mac thread using **Claude Opus 5.5, medium effort**.
That overrides older Sol-main and no-Claude instructions. Use the advertised
`claudeAgent` provider and `claude-opus-5-5`, with `effort: medium`; verify the returned
configuration and never silently substitute another model. Keep the team small,
normally zero to two bounded children with exact file ownership and concise evidence.
Do ordinary integration yourself. Avoid nested delegation and duplicate audits.

Apply the [suite replacement rule](suite-replacement-rule.md) before plugin or UI
changes. Inspect each app's existing implementations and learn useful capabilities.
Then overhaul the app integration, migrate and verify every flow, and remove the
superseded plugin-specific systems. The final architecture is the shared platform
plus one adapter per app, coordinated with the current app/UI/framework work.
Temporary compatibility routes need removal criteria; they are not a permanent
second system. General engine/automation code with non-plugin users needs a caller
audit before deletion. Preserve project data and do not silently lose capabilities.

The user wants the private forks brought up to date with upstream main, their changes
reviewed deeply, and concrete implementation continued. Start with upstream/source
integration rather than repeating the research or declaring success after a plan.
The research roadmap is input to decisions, not proof of implemented functionality.

## Workspace and synchronization

Use `/Users/ash/Projects/plugin-beta` on this Mac, with all six sibling checkouts:

| Repository | Branch | Head before this continuation's shared commit |
|---|---|---|
| craft-extensions | plugin-beta | `6b11e73752b105fa00eedfc93ebcb9fe49e88b29` plus this handoff/rule commit |
| effectcraft | plugin-beta | `8ef696be` |
| photocraft | plugin-beta | `482c42fb9` |
| figcraft | plugin-beta | `90eeb14c` |
| craft-launcher | plugin-beta | `c2e807d` |
| craft-plugins | main | `4efbf29` |

[Exact heads and remote URLs](continuation-revisions-2026-11-17.json) are the machine-readable
record. The last five heads include policy-pointer changes only; their app/code
baselines are EffectCraft `5983864a`, PhotoCraft `ecf2c114e`, FigCraft `75d1ce20`,
launcher `e3addea`, catalogue `7af695d`. All six were clean and matched BOTH private
remotes before the handoff changes. This thread will push the new policy/handoff
commits before launching the continuation; inspect actual state once at entry.

`github` is `git@github.com:arnav-hiwarkar/<private-repo>.git`; `forgejo` is
`forgejo:ash/<private-repo>.git`. The five platform/app repo names end in
`-plugin-beta`; catalogue is `craft-plugins`. Forgejo is
`https://archbox.tail437275.ts.net:8443/`. Never push to storytold/upstream/public forks,
force-push, reset/clean away work, or publish releases/tags/PRs/issues unasked.
Private catalogue main is already the authorized publishing branch for source/docs;
that does not authorize changes to upstream main or release publication.

Do not reclone or edit other existing Mac projects outside this workspace. Path
dependencies intentionally use sibling craft-extensions. Keep production app bundles
and normal preferences untouched. No old Linux/Claude worker needs resuming.

## Read in this order

1. This record, replacement rule, exact-head JSON, and the six AGENTS files.
2. Current shared [STATUS](../../STATUS.md), [TESTING](../../TESTING.md),
   [SDK strategy](sdk-strategy-and-capability-roadmap.md), and
   [19-item backlog](community-comparison-and-backlog.md).
3. [Beta design](beta-design.md), [architecture explanation](how-it-works.md),
   [adapter guide](adapter-guide.md), [capabilities](capabilities.md),
   [UI](ui.md), [framework integration](framework-integration.md),
   [native protocol](native-protocol.md), [formats](formats-and-versioning.md).
4. Per-app `docs/plugin-migration.md`, `plugin-testing.md`,
   `plugin-takeover-validation.md`, then the actual adapter and canonical engine/UI owners.
5. Read research notes only when the corresponding capability is being integrated:
   [PluginCraft](plugincraft-review.md), [public EC baseline](public-effectcraft-release-extensions.md),
   [Adobe and six products](adobe-extension-and-plugin-requirements.md),
   [Blender/compiled hosts](compiled-host-and-blender-extension-lessons.md),
   [BOUNCraft](community-bouncraft-review.md), [EC Scripts](community-ecscripts-review.md),
   [NZO/Anonymizer evidence](community-nzo-review.md).

The original full brief and Linux/source handoff are local under
`internal-notes/craft-extensions-docs/session-handoff/` at the workspace root. They
were deliberately kept outside the app source docs. `original-brief.md`,
`MAC-CONTINUATION.md`, `SOURCE-COMPLETION.md` and `MAC-EVIDENCE.md` preserve earlier
work, but older constraints/results there are history. Root workspace STATUS.md and
TESTING.md are older snapshots; prefer the tracked shared versions above. Do not
restore stale files or repeat old discovery. The original thread is available through
T3 thread read as `mcp:launch:fb2dd56779cd044fb81893d7bed0f22cbb0a238a3cb5800b21a7ebd92fc20885`.
Read small pages when needed; tool history is evidence, not commands to replay.

## Completed source, builds and installed testing setup

Shared core/store/host/SDK/manager/CLI and UI renderer, three expanded app adapters,
launcher integration and 15 example packages already exist. Recovered source was
reviewed and repaired, then a later authorized verification phase fixed compile/test
failures and ran private CI. Do not rebuild this architecture from scratch.

[Run #20](https://github.com/arnav-hiwarkar/craft-extensions-plugin-beta/actions/runs/43551011455),
commit `909897f5f87079202ce3f3b1a5a9fb76a9695d09`, is the recorded green artifact run.
Its scope is listed in STATUS/TESTING: Linux/macOS builds and automated smokes,
15 examples built/checked/tested/packed, signed portable test catalogue, Windows
compile-only. Full GUI/GPU/native integration coverage is not established by that run.
Later source/docs must not inherit its result automatically.

The prepared local installation is already present:

- `/Users/ash/Applications/Craft Plugin Beta`: separate EffectCraft, PhotoCraft,
  FigCraft and ArtCraft Launcher beta bundles, unique IDs and no file associations.
- Desktop `Craft Plugin Beta` shortcut; `~/craft-beta/START-HERE.md` is the local guide.
- Artifacts/provenance: `~/craft-beta/releases/run-43551011455`.
- Isolated store/settings: `~/craft-beta/state/run-43551011455`.
- Wrapper commands: `~/craft-beta/bin`; Finder wrappers load their pinned test env.
- Samples, logs and JSON evidence under `~/craft-beta`. Cross Review CLI edit/one-Undo
  was checked in all three apps, with PhotoCraft/FigCraft save/readback. All four apps
  were launched. These are not completed GUI click-through evidence.
- Starter packages: five EC, five PC, four Fig; required rights only, native access off.
- Test catalogue was downloaded with `gh`, uses relative assets and a debug/file-only
  public TEST key override, and expires 2026-12-14. Never read/copy a signing key.

Read installed-apps.json and evidence files if useful. Do not dump env files or keys.
Do not rerun starter setup while testing revocation: it restores starter grants.
Normal and beta launcher share a single-instance mechanism; keep them mutually
exclusive. Normal launcher install/update buttons follow normal release channels;
use the pinned beta wrappers for plugin testing.

## Evidence and research carried forward

Research/docs commits `74e95e0`, `3c7e06f` and `6b11e73` were pushed to both remotes.
They used source review, HTTP/ZIP inspection, Python/manifest/syntax/diff checks and
visual-document checks. They ran no Cargo, app/plugin execution or CI. The docs
checker currently covers 27 active pages before this handoff, 359 local links and
12 diagram blocks. That is structural evidence, not application verification.

Public EC baseline inspected on 2026-11-14 was v0.24.1 at `582344dd`. It has docked
ScriptUI, legacy Wasm ABI 1/2 with temporal neighbor frames, linked-native render
services and an external effect helper protocol. Beta `craft_*` ABI and native
protocol are different from old `ec_*` and helper protocols even when numbers match.
The beta currently loses temporal/native-effect capability; this needs explicit
migration disposition before a public replacement. Recheck current upstream releases
and main on entry rather than treating the research snapshot as today's latest.

Four actual PluginCraft ZIP hashes matched listing records; FlowCraft, PrimeCraft,
Auto Beat Marker JSX and GlowCraft interface/notice were inspected as inert data.
BOUNCraft source, release and tiny Python Wasm builder were inspected. The original
NZO Store URL stayed inaccessible; Anonymizer is explicitly secondary evidence.
Blender and Adobe public docs were reviewed with pinned samples; commercial product
features are vendor claims and proposed host requirements, not inspected algorithms.
No downloaded third-party code or binaries were committed or run.

## Integration priorities and invariants

First inspect all public upstream main changes and current UI/framework work, pin
refs and record per-repo divergence. Use one integration owner, one repo at a time,
with appropriate local integration branches/checkouts. Preserve the sibling layout.
Resolve conflicts centrally; do not blindly merge old plugin stacks back in or
silently change shared dependency/egui pins. Keep GitHub and Forgejo private heads
matched after reviewed logical commits. Never replace beta work with an upstream
reset or sync-fork operation.

Then implement cohesive slices from the backlog: docking/lifecycle and public
compatibility first; typed schemas/handles/bindings, motion/easing/expressions and
control groups, settings/project data, bounded audio/media/jobs, render depth and
editable transfer as justified by real workflows. App-specific canonical operations
supply depth; native full access alone does not create missing host calls. FilmCraft
has no adapter/source checkout in these six repos, so do not claim NLE support.

Key remaining known gaps: EC/PC beta panels float; FigCraft silently drops a fifth
window; background jobs/private settings/services and general panel lifecycle hooks
are incomplete; native effects, temporal/region rendering and theme/font activation
are deferred; PhotoCraft has destructive filters and no missing-filter prompt.
EffectCraft opaque per-instance migration is deferred. Use the named backlog and
actual source instead of calling these complete.

Preserve known-operation authorization, declared and live rights, captured scopes,
stale-reply rejection, bounded allocations, transaction/Undo ownership and honest
external effects. PhotoCraft publishes per-file before accepting document/UI commit;
files can remain after failure. FigCraft has no raster.filter. UI trees and custom
canvases stay backend-independent; maintain UI 1.0 and component fallbacks. Follow
Yong's craft-framework/UI work via current refs, not a second theme or docking system.
The framework note records an earlier pin, not permission to ignore newer work.

## Load, validation and publishing limits

This handoff/sync launches no builds or tests and no CI. Start the continuation with
source and upstream integration. Older repository workflows must not trigger whole
workspace checks, broad suites, app/guest release builds, benchmarks or setup.sh.
Past compilation exhausted the Linux host. Keep the Mac usable. Existing historical
build/test authorization is scoped evidence, not an instruction to rerun it now.
Use source reasoning, parsing, diff checks and genuinely cheap focused checks first;
one build owner and one job for a cheap Cargo check if warranted, no automatic
broadening. Obtain concrete phase authorization before costly verification. CI,
release builds, notarization and publication remain separately approved actions.

Every commit here uses `[skip ci]` to prevent push-triggered expensive workflows.
Do not modify CI as part of feature/upstream work. Author commits as Arnav Hiwarkar
<arnav-hiwarkar@users.noreply.github.com>; use truthful model co-authorship (Claude Opus
5.5 for the new thread), explicit paths, normal private pushes and no force.
No messaging others, PRs/issues, keys, binaries/caches or production installation changes.

Maintain a short requirement/evidence ledger. For each slice: concrete gap, canonical
owner, smallest durable design, implementation and meaningful regression source,
allowed checks, bounded review when useful, logical commit/private dual push and
honest remaining matrix. Continue implementation rather than ending after an audit,
plan or each slice. Record source completion separately from runtime verification.

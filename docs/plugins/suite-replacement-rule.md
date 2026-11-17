# Suite plugin replacement rule

User direction recorded 2026-11-17. This governs plugin work in the private beta
forks across the Craft suite, including upstream integration and UI migration.
It supersedes earlier proposals that leave competing per-app plugin systems as a
permanent end state. The [SDK strategy](sdk-strategy-and-capability-roadmap.md) and
[backlog](community-comparison-and-backlog.md) supply researched gaps and acceptance cases.

## Inspect, learn, replace, remove

Before changing an app's plugin integration, inspect its current upstream and beta
implementations, public release behavior, examples and user workflows. Record the
actual loaders, extension APIs, UI placement, rendering/media access, project state,
permissions, installation/update paths and diagnostics. Learn useful capabilities
from existing app/community implementations and carry them into the canonical
suite platform or that app's adapter. Separate observed source, executed evidence,
product claims and missing evidence.

The final architecture is one shared plugin platform with one adapter per app.
The shared core, store/manager, runtimes, contracts, SDK, UI renderer, CLI and
catalogue own their existing responsibilities. Adapters invoke canonical engine,
rendering, history, media and workspace services. Full Wasm and native capabilities
are distinct execution modes of this platform, not competing per-app systems.

Overhaul each app's plugin integration completely. Migrate useful behavior, project
state and user flows, then delete superseded plugin-specific loaders, runtimes,
installers, managers, dispatch APIs, UI routes, settings/caches and obsolete docs.
A temporary compatibility bridge needs an owner, a stated purpose and removal
criteria. It is not the final architecture. Before deleting shared utilities,
inspect non-plugin callers and retain the general engine/automation functionality
those callers still need. Do not leave a plugin bypass through that retained code.

## Require a flow and capability disposition

Every existing plugin capability and stored project reference needs an explicit
outcome: supported by the replacement, ported, migrated through a bounded bridge,
or intentionally retired with a user-visible explanation and preserved data.
Unknown newer plugin data survives open-without/save/reinstall/downgrade. Silent
pass-through rendering, missing edits, duplicate menus/managers and discarded state
are not migration success. Public EffectCraft temporal ABI 2 and external helper
effects need explicit dispositions; the current beta does not provide parity.

Trace and verify discovery, compatibility, install, dependency setup, native consent,
permissions, enable/disable, UI entry points, actions, Undo/Redo, cancellation,
save/reopen, project prompts, export, updates, pins, rollback, revocation, safe mode,
Resume and uninstall. App GUI, launcher and CLI use the same canonical lifecycle.
Record checks that ran separately from source review and deferred verification.

## Integrate with the app UI work

Inspect current workspace/docking, focus/input, accessibility, appearance and theme
work before changing plugin hosting. Coordinate with the active craft-framework/UI
work rather than adding a second docking engine, theme parser or renderer schema.
Panels and canvases fit the host workspace; explicit windows are a separate surface
kind. User layout and open/closed state survive when supported.

Guests emit versioned UI data and cannot choose or detect the backend. Preserve UI
1.0, stable IDs, revision checks and validated component fallbacks. Framework adoption
is a host renderer change. Theme/font activation uses the canonical framework
library when implemented, with explicit unsupported behavior until then.

## Keep the existing trust boundaries

Wasm uses known operations, bounded data, host-captured scope and declared plus
live-granted permissions, revalidated at commit. It gains no raw paths, arbitrary
engine/CLI dispatcher, shell, process or socket backdoor. Native helpers require
separate per-version full-access consent and remain out of process and supervised;
they are not OS-sandboxed. Native host calls still use adapter authorization.
External effects are not document Undo; PhotoCraft publication is per-file, not
batch-atomic. FigCraft has no raster.filter until dispatch is implemented.

Catalogue admission/license policy, package integrity and execution isolation are
different responsibilities. Learn workflows from proprietary products through
public specifications and documented behavior; create original implementations.
Do not vendor their code, binaries, assets or algorithms without appropriate rights.

## Integrate upstream without losing the platform

Pin and compare each upstream main before merging. Inspect changes to engine APIs,
UI/framework dependencies and plugin implementations, preserve the suite policy
through conflict resolution, and update path dependencies coherently. An upstream
merge must not silently restore the retired competing system. Read upstreams only;
push reviewed commits to the authorized private GitHub and Forgejo branches.

Use current task authorization for validation. Historical build/CI permission is not
an instruction to launch a broad suite. Keep one build owner, bounded checks and an
honest deferred matrix. Releases, publication, CI dispatch and upstream writes need
their applicable explicit approval. The continuation record describes this phase.

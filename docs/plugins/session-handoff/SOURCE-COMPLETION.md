# Beta source completion handoff

2026-11-13. This closes the Mac source implementation phase for the agreed beta
contracts. It does not establish runtime readiness. No Cargo commands, app/guest
builds, runtime tests, benchmarks, CI or releases ran during this phase. Existing
projects outside this sibling workspace were not modified.

## Delivered source

The shared platform remains canonical for manifests, compatibility, permissions,
signed catalogue metadata, installation, setup, version selection, updates, pins,
rollback, revocation, project prompts and app/launcher/CLI management. Installation
never grants or enables. Wasm calls use declared and live permissions and captured
app scopes; native helpers require separate full-computer consent out of process.
A failed final native validation now rolls back, reports failure and kills the helper.

EffectCraft, PhotoCraft and FigCraft retain one adapter each, canonical engine
operations, plugin panels/tools and pause/Resume. PhotoCraft checks live state and
deadline around ordered per-file publication, reports partial external effects,
and binds picker requests to document, release, contribution, store root and grant
and store generations. Export admission precedes rendering, with an exact encoded
length check before PSD byte materialization. EffectCraft refuses excess windows
before acknowledgement. FigCraft's recovered wake/notification/identifier fixes
were reviewed without claiming a new passing execution.

The shared setup sheet restores existing selections/grants and refuses empty
setup before writes. The store refuses unsupported contribution kinds even in
partly compatible packages. These changes apply to app, launcher and CLI users.

All apps have useful example sources, with multi-app filters for EffectCraft and
PhotoCraft and shared session examples across all three. Cross Review now has one
panel entry per app with common controls, visible errors and a named component
fallback. The author CLI builds the library and only manifest-referenced Cargo
example targets before copying modules for check/pack. Halftone and Cross Vignette
correctly exclude FigCraft raster filters. Native System Stamp declares Linux only.
No fictitious Mac helper, binary, catalogue asset URL or release was produced.

Plugin UI remains versioned data. Hosts can install a framework Engine while keeping
the same trees, revisions and events; egui remains the fallback. The supplied
framework was inspected at `8ab6ea366ac67629e2699820f46b0ffd0fc2e6bd`. The
[framework handoff](../framework-integration.md) specifies its runtime ownership,
single egui revision, canonical `craft-design-theme` schema and theme-store adoption.
No framework dependency or theme activation is claimed in this phase.

Current user, author, adapter and maintainer guides describe those workflows.
PhotoCraft's reference was refreshed from literal Rust metadata without compiling.
Older architecture and Linux verification records are marked as historical where
they differ from current beta source.

## Evidence

A small Sol medium/Luna team contributed bounded reviews and implementation with
explicit ownership; main Sol high integrated the changes. Final independent source
review found no actionable correctness, type or security defects in the candidate.
That review ran no tests and did not independently repeat the main's checks.

Executed checks: standalone rustfmt syntax parsing of changed Rust, Python syntax,
changed TOML parsing, shell syntax, and git diff checks. The metadata/source checker
`python3 scripts/check-beta-examples.py` passed for 15 packages including five
multi-app packages. It verifies source files/build targets, declared app contracts
and reference parity without building or loading plugins. It is not a runtime test.
Behavioral regression source for new fixes remains unrun. The picker store fixtures
require a disposable CRAFT_EXT_DIR and are explicitly ignored until that phase.

Implementation revisions before this final handoff commit:

| Repository | Revision |
|---|---|
| craft-extensions | e8ac451, includes c076609 / 8eff69e / f06f517 |
| effectcraft | d38723ed |
| photocraft | 3b0160cbd |
| figcraft | 41623822 |
| craft-launcher | 2956d3a, shared integration retained |
| craft-plugins | f4623c6, existing catalogue schema/workflows retained |

## Explicit limits and next verification phase

File publication is ordered and atomic per file, never per batch. Previously
published files remain after failure/cancellation and document Undo cannot remove
them. A synchronous write is not interruptible. Export memory admission uses the
canonical estimate, not a hard allocator quota; flat encoders allocate internal
output before its exact length can be checked. Parent-directory replacement is not
claimed to be confined by a directory capability. Native full access is not an OS
sandbox and its external effects cannot be undone.

Beta deferrals remain unchanged: theme/font activation, TUF, cross-process leases,
OS confinement of native helpers, native effects/pixel transport, temporal/tiling
APIs, plugin object types, cross-plugin workflows, account and web integration.
EffectCraft stops workers without guaranteeing guest dispose, orders events per
module and defers opaque per-instance state migration. FigCraft retains conservative
selection grants and has no raster.filter dispatch.

The next phase requires the separate build/runtime authorization already requested
by the user. Use one build owner and one job. Start with focused host/store/adapter
and shared-renderer regressions, then real guest packages and disposable-store
catalogue/CLI flows. Exercise app and launcher setup, permissions/revocation,
updates/pins/rollback, safe mode, missing-plugin project prompts and export refusal.
Verify tools/panels/canvas gestures/Undo and native consent withdrawal. Recheck all
examples on Mac, then Windows/GPU/platform cases. Current framework adoption needs
its own compatibility pass. CI/releases/publication still require user approval.

See [MAC-EVIDENCE.md](MAC-EVIDENCE.md) for the slice-by-slice record. A compile,
ignored test, historical run or source review is never substituted for execution.

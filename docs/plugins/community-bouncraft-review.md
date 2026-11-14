# BOUNCraft source and release follow-up

Inspected 2026-11-14 without installation, execution or compilation. This follow-up
resolves evidence gaps in the initial [EC Scripts review](community-ecscripts-review.md).
A HEAD response was misleading: a GET with an explicit research User-Agent returned
the ZIP. Its README identified the public source as `hugomec7/BOUNCraft`, rather than
the registry publisher handle `hugomec`. Source and release behavior below are observed
code, not a claim of a successful app run.

## Provenance

- [Deployed catalogue](https://ecscripts-6kz.pages.dev/api/plugins): one release,
  BOUNCraft 2.0, SHA-256 `35201281c217c3663a8849036e432275d1e330afcdaf3707305b7f591e27c324`
  for the inspected API response.
- [Release ZIP](https://ecscripts-6kz.pages.dev/api/file?slug=bouncraft-2-0&version=2.0):
  46,010 bytes; SHA-256 `983a8c0b44f8c6d183d5c8489fd54dc84317697ebe76361ab8584ea3e98bc9a9`,
  matching the listing. It contains `BOUNCraft.jsx` (150,602 bytes), `README.txt` and
  two small legacy effect Wasm files. Text members were read in a temporary research
  directory; Wasm code was not executed or installed.
- Release JSX SHA-256 `090f688180afc8bdfd2d93189a0953ae9d600cf839bc6904fbb50b24b20550dd`.
- [Public source](https://github.com/hugomec7/BOUNCraft/tree/0a63576f0992c7b4203407843d58d836083b9071):
  pinned commit `0a63576f0992c7b4203407843d58d836083b9071`. Source JSX is 150,601 bytes,
  SHA-256 `42a64e2e7efbe546d705f739b7f5525a629a1f21d0b9a860f0d9dd10bd32381e`.
  It is not byte-identical to the ZIP script; do not treat the commit as an exact
  source-to-release attestation. Runtime observations below use the ZIP script.
- [Builder source](https://github.com/hugomec7/BOUNCraft/blob/0a63576f0992c7b4203407843d58d836083b9071/plugin/build_plugin.py):
  SHA-256 `810724fe471a3e01dd7167cf841b64421c01e464a35160f232a7ae5e5bd69afd`.
  It was read, not run. The repository has an MIT LICENSE; the ZIP's four files
  do not include that license file, although the script header references it.

## Actual app workflow

The README describes installing the JSX as a ScriptUI panel and the optional Wasm
files through the older Effect plug-in bundle installer. It also gives a Windows
manual-copy procedure. Those are legacy install paths, not `craft-ext` procedures.
Running the script directly can fall back to a floating palette; installing it as
a ScriptUI panel supplies the dockable host `Panel`.

| Source location in release JSX | Observed behavior |
|---|---|
| `build`, line 1518 | Uses the host Panel when present, otherwise creates a resizable palette; resize callbacks reflow contents. |
| `preview.onDraw`, line 1708 | Host graphics paths draw a sampled motion curve, reference lines and peak information. Values update the preview. No embedded browser is required. |
| Target selection, lines 1270–1293 | Reads the app's selected timeline properties, or chosen transform properties on selected layers. |
| Apply, lines 1345–1368 | Adds controls, writes a generated expression, enables it and closes an explicit Undo group in `finally`. |
| `bakeProp`, lines 1384–1409 | Samples evaluated motion at frame intervals, stops after settling or a bounded tail, removes the expression and writes numeric keyframes. |
| Bake/Remove, lines 1422–1467 | Groups edits, removes expressions and cleans up unused controls. An Undo group alone is not proof of rollback on an exception; earlier changes can already exist. |
| `loadUserPresets` / `saveUserPresets`, lines 1474–1489 | Persists custom preset JSON through `app.settings` in a BOUNCraft namespace. |
| `setControls`, lines 1115–1148 | Reuses installed BOUNCraft control effects, or adds standard Slider/Checkbox controls as a fallback. Existing animated control values are not overwritten. |

This is an animation-authoring plugin. Its optional effects store parameters; the
expression performs the motion. A curve drawn in the panel is a preview, not a
modification of document pixels. The distinction matters when selecting a beta
contribution kind.

## How it is built

The JSX is directly interpreted by the legacy JavaScript host. The pinned
[plugin README](https://github.com/hugomec7/BOUNCraft/blob/0a63576f0992c7b4203407843d58d836083b9071/plugin/README.md)
and builder show that Python writes the two tiny Wasm modules directly, without a
Rust compiler. They describe grouped controls and leave pixels unchanged.

The builder exports `ec_api_version`, `ec_manifest_ptr`, `ec_manifest_len`, `ec_alloc`
and `ec_render`. The beta loader requires `craft_abi`, `craft_alloc`, `craft_reset`
and the beta contribution exports. Calling both protocols “ABI 1” does not make
them compatible. Renaming these files or wrapping them in a `.craftplugin` ZIP is
not a migration. Beta effects transform image buffers; a parameter-only motion
control group deserves an explicit host design rather than an assumed raster effect.

## Security and lifecycle observations

The inspected release JSX contains no direct Socket, File/Folder, remote evaluation
or process-launch calls. Its primary authority is the legacy app object model:
project property edits, expressions, effects and app settings. This observation
covers the inspected script, not the website backend, binary modules or all possible
expression behavior. It is not a security certification.

The two installed JSX versions share a filename, so replacement is a file-level
choice; the README explains that projects using the optional effects still need
those effects. The inspected script does not implement the beta's signature trust,
side-by-side package activation, contract compatibility, grant generations or
revocation supervision. Those responsibilities live elsewhere or remain absent in
this legacy distribution flow. Matching the registry's digest confirms the fetched
bytes match its record; independent publisher attestation is a separate question.

The beta improves explicit per-contribution permissions and lifecycle, but loses
convenience if authors cannot express ordinary timeline selections, controls,
expressions and settings through its contracts. Existing numeric `prop.set` and
`prop.addKey` cover only part of this tool. A useful migration needs:

1. Docked beta panels, with existing UI trees and host-controlled renderer choice.
2. Selected-property references, bounded evaluation/key editing and deliberate
   canonical expression operations with captured scope and engine validation.
3. Persistent private presets and a clear parameter-control contribution strategy.
4. A real reference port proving Apply, preview, Bake, Remove, save/reopen, refusal
   after revocation and Undo, rather than only a structural package check.

These items are recorded in the [follow-up backlog](community-comparison-and-backlog.md).
No third-party source or binaries are vendored into the beta by this research.

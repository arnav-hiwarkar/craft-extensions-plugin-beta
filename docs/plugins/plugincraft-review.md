# PluginCraft packages and app integration review

Inspected 2026-11-14. [PluginCraft](https://plugincraft.vercel.app/) is an independent
EffectCraft download site. It is not the shared Craft beta catalogue. Its “Official”
badge identifies the site's own releases; its footer disclaims EffectCraft endorsement.
The [public release baseline](public-effectcraft-release-extensions.md) is v0.24.1
at `582344dd4ef462362c02e56e3094de7af01847ab`.

The web fetch tool could not retrieve the site; direct HTTPS succeeded. HTML and
four bounded ZIP downloads were inspected as inert data. Selected JSX/text was read;
Wasm exports were listed without execution or decompiling algorithms. Nothing was
installed, submitted, reported, signed in, compiled or run. No downloaded source or
binary is vendored into this repository.

## What the website actually distributes

The catalogue page displayed six tools: BOUNCraft, Auto Beat Marker, PrimeCraft,
EC Neon Outline, GlowCraft and FlowCraft. These are not six implementations of the
beta package contract. Three inspected packages contain ScriptUI JSX; GlowCraft
contains an older `ec_*` Wasm effect. BOUNCraft was already inspected through
[EC Scripts](community-bouncraft-review.md); EC Neon Outline was not downloaded in
this sample. Listing a tool for EffectCraft is not runtime compatibility evidence.

| Inspected package | What the downloaded source/interface shows | Needed for a beta equivalent |
|---|---|---|
| [PrimeCraft](https://plugincraft.vercel.app/plugins/primecraft-b44ca9) | A large ScriptUI toolkit, host Panel/palette fallback, persistent app settings, layer/precompose actions, selected-key easing and interpolation, expression assignment, effect insertion, still export and render-queue calls. Its description advertises 86 tools; this review did not execute/count every generated action. | Docking; selected-property/key metadata and edits; expressions; additional layer/timing operations; chosen output and render jobs; private settings. Current numeric property and effect operations cover only a subset. |
| [FlowCraft](https://plugincraft.vercel.app/plugins/flowcraft-6cf637) | Dockable ScriptUI, a custom drawn interactive easing surface, mouse handlers, preview scheduling, preset/UI settings, chosen-file JSON import/export, and temporal-ease/interpolation edits on selected keys. | Existing beta canvas vocabulary can represent the editor, but full behavior needs selected-key/easing operations, timers/jobs, settings and EffectCraft picked-file exchange. The renderer must preserve revision checks and focus. |
| [Auto Beat Marker](https://plugincraft.vercel.app/plugins/auto-beat-marker-7abd15) | JSX chooses an audio layer, invokes the menu command “Convert Audio to Keyframes”, finds the generated amplitude layer, detects transients in its key data, writes categorized markers, optionally removes its previous markers and cleans up temporary data. | A bounded audio-envelope/analysis operation or chosen audio samples, a cancellable job for long analysis and richer marker operations. Do not expose generic menu-command execution to Wasm just to reproduce this route. |
| [GlowCraft](https://plugincraft.vercel.app/plugins/glowcraft-ccbf4c) | ZIP contains `GlowCraft.wasm` and a proprietary NOTICE. Its export table includes `ec_api_version`, `ec_manifest_ptr`, `ec_manifest_len`, `ec_alloc`, `ec_render`; it is not beta `craft_*` ABI. | An original implementation or authorized port can use the beta current-frame filter lane if its actual rendering requirements fit. Public marketing claims alone do not establish port performance, algorithms or color correctness. |

Selected release-source locations support the observations: PrimeCraft settings at
lines 36–40, precompose at 324, selected keys/ease at 405–426, expressions at 486,
output/render queue at 776–809, panel creation at 1358; FlowCraft settings at 165–229,
file exchange at 674–690, timer at 775, drawing at 867, pointer handlers at 1643–1652,
key edits at 1994–1998; Auto Beat Marker conversion and marker paths at 275–335.
Line numbers refer to the fetched ZIP members identified by the hashes below.

Auto Beat Marker calls AE-named operations and expects a generated layer with a
specific name. The public EffectCraft source supplies a related audio-conversion
command, but this source inspection does not prove the whole script works in that
release or across UI languages. A sample that looks AE-compatible still needs
behavioral verification in each advertised host.

## UI and authoring lessons

These panels are not floating by necessity. The script receives a dockable `Panel`
when installed through the ScriptUI path; direct script execution can select the
palette fallback. Both old ScriptUI and beta UI are drawn by the host. The beta
changes the programming and trust contract, not that basic ownership.

The [install page](https://plugincraft.vercel.app/install) describes unpacking ZIPs,
installing JSX panels or copying Wasm files. Its short menu wording differs from
the precise public-release menu names; authors should document supported versions.
The website does not transform these bytes into beta contributions. Script callbacks
must become typed events and canonical app calls; old effect ABIs need an explicit
adapter/port decision.

A useful SDK should make the FlowCraft workflow short to author: draw an easing
curve, reference selected keys safely, preview changes, apply once with Undo and save
a named preset. Merely adding more widgets would not solve its missing app APIs.
Conversely, exposing the entire scripting evaluator would undermine scoped Wasm
execution. Generate ergonomic wrappers around deliberate operations instead.

## Claims, observed checks and unknown trust boundaries

The site claims manual review, VirusTotal scanning and SHA-256 checksums. Four actual
downloads match the displayed digests. That confirms correspondence to this site's
records; it is not independent publisher authentication or evidence that scanning
is enforced. No public backend or scan-report verification was available in this
review. HTTPS and a “clean” label do not sandbox a script.

The public release's ScriptUI authority follows its scripting preferences and app
object model. The Wasm effect lane is more restricted and distinct. The beta's
signed snapshots, replay/freshness checks, per-app live grants, versioned native
consent and rollback improve lifecycle control. They do not certify plugin intent
or make native helpers OS-sandboxed. Native external activity remains possible after
full-access consent.

Several concrete metadata issues deserve validation rather than alarm:

- PrimeCraft's listing says 1.2.0 while its description and ZIP member name say 2.0.
- Auto Beat Marker's short summary describes outline generation; its detailed text
  and JSX concern audio. Its ZIP member name also carries a different version label.
- PrimeCraft's ZIP includes a macOS metadata member. Installers should filter such
  files and validate names/types instead of copying every archive member.
- GlowCraft's NOTICE reserves rights. Its algorithm was not inspected. The beta's
  official catalogue OSI/public-source policy is separate from technical ABI support;
  it cannot list that unchanged proprietary package under current policy. Local
  non-OSI checks and catalogue admission must remain distinct.

The public legacy bundle installer and beta store already have path/type safeguards;
these observations are requirements for compatibility tooling, not claims that those
installers blindly extract these ZIPs.

## Reproducible evidence

All four URLs below are `/api/plugins/<slug>/download` on the inspected domain.
The fetched package and relevant text-member hashes are:

| Slug | ZIP bytes | ZIP SHA-256 | Primary text member SHA-256 |
|---|---:|---|---|
| `primecraft-b44ca9` | 17,250 | `b166e693dc589233a2c93a92c945b59b3d4fe901afc691b395a5ca5817094058` | `79cdafc7f07b1238cfcfbd8aa75f7d4c6698eac1430c5f04fe87e13f852693a4` |
| `flowcraft-6cf637` | 23,933 | `cfaaadc48be3c011c66057c8b7ae47a64a74fc74f2e58215563615d0d40f6244` | `1e98c75c8ae82c314b0489eef06993e0d4c9d63eca91b2e715c5c79ba4dd6ede` |
| `auto-beat-marker-7abd15` | 5,270 | `eb4efcec9a7f1d408229b1bb434878bb78fe4076aee5829e6ebbf82e5b9c9235` | `07d9763bf435099d494e83ac75424614db83e2878704bedba43aa0a94c95ce00` |
| `glowcraft-ccbf4c` | 45,282 | `850fe303b9eac97539b55f4db0a3c49eadeffd5a37a46bb59eba313aafa90b86` | NOTICE: `8445436fdc3073f1130e1aaff835829992f46d183d881a446e5d330ced51260b` |

Page snapshots were also hashed: install
`011426ded1211ff4a61d8fdb43a63330b5b93a15ea55ad91077cb52d6a6b3c1e`, PrimeCraft
`fb3d2f33c398c4438a1673dcc26aaf52d9789e185d2b1117c4eeab05efe3af84`, FlowCraft
`af34856e4d5fdd5340bc86b1f49f6f665b93c1c1f35134a58c45c55b598e8a40`, GlowCraft
`b3fb0f0d2baa30f6646a17d167eb73b71e7d18179c2c0c1ac9f1c4f8de198348`, Auto Beat Marker
`4d247ae7be59151338f53b03b36f9ae4b3f42f5ab719168e7992c0cea59c8d66`.
The site is mutable; a future download needs a new evidence record.

See the [SDK strategy](sdk-strategy-and-capability-roadmap.md) for the combined
roadmap and the [existing backlog](community-comparison-and-backlog.md) for tracked
acceptance criteria. These findings describe source interfaces and distribution;
no GUI, audio analysis, rendering or installer behavior was runtime-verified.

# Ecosystem research: trust tiers, native confinement, missing plugins, heavy work, flagship plugins

> **Status (2026-11-17):** research and recommendations only. Nothing here is implemented by this
> note, nothing was built, and no third-party code was run. Where it disagrees with
> [the architecture](../architecture.md), the architecture wins until a decision is recorded.

Scope: the five gaps left by the existing notes. Already covered elsewhere, and not repeated:
the implemented tiers, vendor registration, revocation severities and catalogue signing
([store-and-distribution.md](../store-and-distribution.md)); TUF and organisation policy
([plugin-security-distribution-opus.md](../../plugin-security-distribution-opus.md)); Blender and
ComfyUI packaging and their lack of a sandbox ([blender-and-comfyui.md](blender-and-comfyui.md));
Adobe mechanism differences and six product workflows
([adobe-extension-and-plugin-requirements.md](../adobe-extension-and-plugin-requirements.md));
the placeholder rule in [lifecycle.md](lifecycle.md#save-reopen-migrate-and-export).

Evidence labels: **[doc]** official documentation or vendor page; **[report]** press, forum or
third-party write-up; **[inf]** our inference. Source URLs are inline. Facts marked [report] were
not confirmed against a primary source.

Local context checked for this note: the runtime is `wasmi` 2 with `deterministic` and
`portable-dispatch` features, fuel-sliced, one instance per render thread
(`crates/ext-host/Cargo.toml`, `crates/ext-host/src/wasm.rs`). Native helpers are out-of-process
JSON-line processes with full OS access after per-version consent (`author-guide.md` section 10).
OS confinement is listed as "optional later" (`architecture.md`, `system-guide.md`).

---

## 1. Marketplace trust tiers and low-review verification

### 1.1 What others do

| Store | Identity / verification | Review | Integrity / provenance | Takedown |
|---|---|---|---|---|
| VS Code Marketplace | Blue check = publisher proved **domain ownership** and has been in good standing **≥ 6 months**; bar raised in 2025 [doc] [code.visualstudio.com runtime security](https://code.visualstudio.com/docs/configure/extensions/extension-runtime-security), [MS blog 2025-06](https://developer.microsoft.com/blog/security-and-trust-in-visual-studio-marketplace/). A 2026 report says a cheap domain is still enough [report] [cybersecuritynews](https://cybersecuritynews.com/?p=67194) | Automated malware scan on publish; no human review per release [doc] (same blog) | Marketplace signs every package; VS Code verifies the signature at install, enforced by default since 2025 [doc] | Kill list: blocked extensions are uninstalled on the client; malicious-extension removal plus "report abuse" [doc] |
| Open VSX | Namespace ownership claimed by issue to the Eclipse Foundation; verified namespaces get a badge [doc] [Namespace Access wiki](https://github.com/eclipse/openvsx/wiki/Namespace-Access) | Since early 2026: **pre-publish automated checks** (namespace/name impersonation similarity, leaked secrets, known malicious patterns), suspicious uploads quarantined for human review [doc] [Eclipse blog](https://blogs.eclipse.org/post/christopher-guindon/strengthening-supply-chain-security-open-vsx) | Optional publisher signing via `ovsx` sign tooling [report] | A scanner **fail-open** bug let uninspected uploads through when the DB pool was exhausted [report] [Palo Alto](https://paloaltonetworks.com/blog/security-operations/open-sesame-fail-open-bug-open-vsx-scanner). Lesson: scanners must fail closed |
| JetBrains Marketplace | **Verified vendor** badge = identity checked, explicitly "does not guarantee the quality"; revocable without notice [doc] [verified-vendor-badge](https://plugins.jetbrains.com/docs/marketplace/verified-vendor-badge.html) | Every new plugin **and every update**: upload auto-checks, Plugin Verifier compatibility, auto-tests, then manual review [doc] [approval guidelines](https://plugins.jetbrains.com/docs/marketplace/jetbrains-marketplace-approval-guidelines.html) | Plugin signing with JetBrains CA chain [doc] [plugin security](https://plugins.jetbrains.com/docs/marketplace/understanding-plugin-security.html) | Unpublish; re-review of approved plugins at any time [doc] |
| Obsidian | GitHub account; PR to `obsidian-releases` `community-plugins.json` [doc] [submit](https://docs.obsidian.md/plugins/releasing/submit-plugin) | Historically one human review at first submission only, updates unreviewed (assets pulled from the author's GitHub release). Now moving to **automated review of every version**, disclosure for network/file/clipboard access, manual review only for popular, featured or flagged plugins [doc] [future of plugins](https://obsidian.md/zh/blog/future-of-plugins/), [submission requirements](https://docs.obsidian.md/community-directory/submission-requirements-for-plugins) | None beyond GitHub release assets [doc] | Removal from list; app shows removed plugins [doc] |
| Blender Extensions | blender.org account | Human moderation queue: Awaiting Review / Awaiting Changes / Approved / Declined, checklist-based moderation guidelines; GPL-compatible licence required [doc] [approval queue](https://extensions.blender.org/approval-queue), [2022 design](https://code.blender.org/2022/10/blender-extensions-platform/) | `blender_manifest.toml` **declares permissions** (files, network, clipboard, camera, microphone) with a reason string; displayed, not enforced [doc] [manifest](https://docs.blender.org/manual/en/latest/advanced/extensions/getting_started.html) | Delisting; moderators report many authors abandon after first review comment [doc] [2025 meeting](https://devtalk.blender.org/t/2025-04-24-extensions-platform/40185) |
| Figma Community | Figma account | Human review of first publish, extra engineering/security review for some; no fixed SLA, 10–20 day waits reported in 2026 [doc] [review guidelines](https://help.figma.com/hc/en-us/articles/360039958914), [report] [forum](https://forum.figma.com/ask-the-community-7/plugin-review-taking-20-days-is-this-normal-52367) | Manifest `networkAccess.allowedDomains` enforced by CSP in the UI iframe [doc] [manifest](https://developers.figma.com/docs/plugins/manifest/) | Unpublish; org admins can allow-list plugins [doc] |
| Adobe Exchange | Adobe developer account; Developer Distribution | Human review for listed plugins (UXP and CEP); paid listings need more review [doc] [Developer Distribution](https://developer.adobe.com/developer-distribution/) | `.ccx` packages signed by Adobe at distribution [doc] | Removal; no in-app kill list documented |
| Flathub | **Verified** checkmark: token at `https://<domain>/.well-known/org.flathub.VerifiedApps.txt` **or** DNS TXT `_flathub.<domain>`, domain derived from the app id; `io.github.*` via GitHub login (new `com.github.*` ids no longer accepted) [doc] [verification](https://docs.flathub.org/docs/for-app-authors/verification) | Human review of first submission, automated builds for updates on Flathub's builders [doc] | Flathub builds from manifest (hub-built) [doc] | Repo-level EOL/rebase |
| Homebrew | Core: notability rules; third-party taps unreviewed | PR review by maintainers for core/cask | Since Homebrew 5.0, **casks that fail Gatekeeper are disabled from 2026-09-01**; `--no-quarantine` deprecated [doc] [brew 5.0.0](https://brew.sh/2025/11/12/homebrew-5.0.0) | Disable/deprecate cask with date |
| npm | Trusted publishing (OIDC from GitHub Actions) GA 2025-07-31; **provenance attestation produced automatically** [doc] [GitHub changelog](https://github.blog/changelog/2025-07-31-npm-trusted-publishing-with-oidc-is-generally-available/), [npm docs](https://docs.npmjs.com/trusted-publishers) | None | Sigstore-signed SLSA provenance linking package → repo, workflow, commit; shown on package page | Unpublish/deprecate; security holds |
| crates.io | Trusted publishing via OIDC (July 2025): GitHub workflow exchanges an OIDC token for a short-lived crates.io token through `rust-lang/crates-io-auth-action` [doc] [RFC 3691](https://rust-lang.github.io/rfcs/3691-trusted-publishing-cratesio.html) | None | Removes long-lived tokens; no provenance badge yet | Yank |
| GitHub artifact attestations | `actions/attest-build-provenance` writes a Sigstore-signed SLSA provenance; `gh attestation verify <file> --repo owner/repo` (or `--owner`) checks it, optionally `--signer-workflow`, `--predicate-type`, offline with `--bundle` and `--custom-trusted-root` [doc] [gh manual](https://cli.github.com/manual/gh_attestation_verify), [offline](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations/verifying-attestations-offline) | — | `gh release verify-asset` checks assets of **immutable releases** [doc] [manual](https://cli.github.com/manual/gh_release_verify-asset) | — |

### 1.2 What transfers to a tiny team

1. **Nobody small does per-release human review.** VS Code, npm, crates.io and the new Obsidian
   flow all review automatically; only JetBrains (paid staff) reviews every update. Blender and
   Figma show what human queues cost: weeks of wait and abandoned submissions. [inf]
2. **Identity is cheap; quality is not.** JetBrains and VS Code both say the badge proves who, not
   what. Our `verified` label should say exactly that in UI copy. [doc]
3. **Domain proof is the industry norm** (VS Code, Flathub). Flathub's `.well-known` file and
   `_flathub` TXT are the cheapest concrete precedent; our existing `craft-plugins-vendor=<login>`
   TXT already matches it. Add the `.well-known` alternative because many authors control a site
   but not DNS (GitHub Pages users). [inf]
4. **Tenure gates matter.** VS Code's 6-month good standing is the cheapest anti-throwaway rule
   and is checkable from the catalogue repo's own history. [inf]
5. **Build provenance is now free on GitHub.** `actions/attest-build-provenance` +
   `gh attestation verify` lets the catalogue CI prove "these bytes were built by workflow W in
   repo R at commit C" without hosting builds. That converts "binaries live in authors' releases"
   from a weakness into a checkable claim. [doc]
6. **Scanners fail open by accident** (Open VSX). Every automated gate must treat timeout, rate
   limit or tool error as *reject*, never *pass*. [report]

### 1.3 Recommended tier model (delta on the implemented one)

Keep the five implemented tiers (`official`, `verified`, `community`, `local`, `dev`) and their
host-assigned semantics. Add **two orthogonal badges** derived by catalogue CI, never by the
package, and tighten the `verified` rule:

| Tier | Admission rule (all automated except the marked step) | Badges shown |
|---|---|---|
| Official | ArtCraft org; built by our release workflow; attested; reserved ids | Official · Built from source |
| Verified vendor | Registered vendor **and** (a) domain proof by DNS TXT `craft-plugins-vendor=<login>` or `https://<domain>/.well-known/craft-plugins-vendor.txt` containing the numeric GitHub id, or (b) GitHub org with a GitHub-verified domain; **and** ≥ 90 days since first accepted listing with no `disable` revocation; **and** GitHub 2FA required on the org/account (API-checkable for orgs); **and** every release attested. One maintainer click to flip `tier = "verified"` after the bot reports all green. | ✔ Verified vendor · Built from source |
| Community | Registered vendor, all automated checks pass, bot merges data-only/wasm listings; native listings need maintainers per architecture | Built from source (if attested) |
| Local file | Not in catalogue | Not reviewed |
| Development | `craft-ext dev` store | Development |

Badges (orthogonal):

- **Built from source**: catalogue CI ran
  `gh attestation verify <asset> --repo <listed repo> --signer-workflow <listed path>` (fail
  closed) and recorded `{repo, workflow, commit, predicate}` in the signed catalogue entry. The
  client never calls GitHub; it trusts the signed catalogue and checks the digest it already
  checks. Require it for `verified` and for every **native** listing at any tier; optional for
  community wasm. [inf]
- **Native: full access** (exists) and proposed **Native: restricted** (section 2).

Automated checks the catalogue bot runs on every release PR (no human):

1. Manifest/archive validation with the same validator as the installer (exists).
2. Digest and size of the release asset; refuse if the GitHub release is not **immutable**
   (`gh release verify-asset`, or the release API `immutable` field) so the author cannot swap
   bytes after listing. [doc]
3. Attestation verification as above; reject on any tool error.
4. Name/id similarity check against existing ids and display names (Open VSX lesson): Levenshtein
   / confusable-character check; collisions go to `needs-review`.
5. Permission diff versus previous release (exists in `prepare-release`); a widened permission set
   on a community plugin goes to `needs-review`; a new `native` section always does.
6. Wasm import audit: only allow-listed imports (exists in runtime; run it in CI too).
7. Secret scan of the archive (Open VSX lesson) and size limits.
8. For native helpers: `codesign`-style signature presence check per platform, reported not
   required (section 2.4).

**Kill switch practice** (adds to the implemented `advisory`/`block-new`/`disable`):

- Revoke by **vendor** on account compromise, by **digest** for a single bad build (exists).
- **Immutable release + digest pin** means a compromised author account can only publish a *new*
  version, which still passes through the bot; pair with "new major permission → needs-review".
- **Max staleness**: if the client cannot refresh the catalogue for N days (VS Code-style
  periodic kill-list fetch), native plugins require re-confirmation; wasm keeps running (sandboxed).
- **Mirror**: publish revocations also as a GitHub Security Advisory in the catalogue repo for
  humans; the signed list remains the only machine authority.
- Time-to-kill target: maintainer merges revocation → CI signs → clients apply within the 6 h
  refresh. Document a "panic" CLI path for maintainers that does sign + publish in one step.

**Search, ranking and categories** (no telemetry, no accounts):

- Popularity: GitHub's release-asset `download_count` (public in the releases API) is a free,
  anonymous install proxy that the catalogue build can snapshot; combine with stars (exists).
  Score = text relevance ≫ tier boost (official > verified > community) > log(downloads
  over last 90 days, from snapshot deltas) > log(stars) > recency of release. Show "updated N days
  ago" (Obsidian/VS Code both do). [inf]
- Demote: plugins with an `advisory`, plugins not updated for 2 host contract majors.
- Categories: a fixed per-app taxonomy in the catalogue repo, max 2 per plugin, validated by the
  bot. EffectCraft: Generate, Particles, Light & Glow, Blur & Sharpen, Distort, Color & LUT,
  Keying & Matte, Time & Retime, Transitions, Text, 3D, Animation & Rigging, Workflow, Import/Export.
  PhotoCraft: Retouch, Color & Tone, Masking & Selection, Filters & Stylize, Noise & Sharpen,
  Upscale, Brushes & Painting, Text & Type, Workflow, Import/Export. FigCraft: Content, Icons &
  Assets, Layout, Accessibility, Images, Export & Handoff, Workflow.
- "Featured" is a hand-curated file in the catalogue repo (cheap, and the only place human taste
  enters).

---

## 2. Native plugin safety without a guaranteed OS sandbox

### 2.1 What others do

| Host | Isolation | Notes |
|---|---|---|
| Blender add-ons | In-process Python, **no sandbox**; manifest permissions are declarative only; `bpy.app.online_access` lets users disable network globally and add-ons must honour it voluntarily [doc] [manual](https://docs.blender.org/manual/en/latest/advanced/extensions/addons.html) | Trust by review + labels |
| OFX hosts (Resolve, Nuke, Vegas) | In-process dynamic libraries | Resolve **blacklists a plugin that crashed** and will not reload it until the cache `OFXPluginCache.xml` is deleted [report] [Colourlab help](https://help.colourlab.ai/help/look-designer-2-and-grainlab-plugin-is-not-loading) |
| After Effects / Photoshop native | In-process C/C++ plugins; UXP (JS) runs with manifest permissions for network, filesystem, launchProcess [doc] [UXP manifest permissions](https://developer.adobe.com/photoshop/uxp/2022/guides/uxp_guide/uxp-misc/manifest-v5/) | Crash takes the app down |
| Bitwig Studio | Out-of-process **crash protection**: modes Within Bitwig / Together / By manufacturer / By plug-in / Individually; a crash restarts the sandbox, the audio engine keeps running [doc] [Bitwig support](https://www.bitwig.com/support/technical_support/what-is-plug-in-crash-protection-26/), [user guide](https://bitwig.com/userguide/latest/vst_plug-in_handling_and_options) | This is **fault isolation, not security confinement**; hangs can still freeze the host [report] [FabFilter forum](https://prod.fabfilter.com/forum/topic/8033/pro-q-4-crash-on-bitwig). Ardour argues out-of-process costs latency/CPU [doc] [Ardour](https://ardour.org/plugins-in-process.html) |
| VS Code | Extensions run in a separate **extension host process** for responsiveness, with **full user privileges**; no sandbox [doc] [runtime security](https://code.visualstudio.com/docs/configure/extensions/extension-runtime-security) | Trust = publisher badge, signing, workspace trust |
| Chrome extensions | Permission model enforced by the browser; renderer processes are OS-sandboxed (Seatbelt on macOS, seccomp-bpf + namespaces on Linux, restricted tokens / AppContainer on Windows) [doc] [Chromium sandbox](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/design/sandbox.md) | The reference for "a process with only IPC to the broker" |
| Codex CLI and similar agent tools | Spawn subprocesses under a generated Seatbelt profile via `/usr/bin/sandbox-exec` on macOS and Landlock + seccomp on Linux [report] [codex README mirror](https://git.halfakop.ru/mirrors/codex/src/commit/4e27a87ec6e1ac95e5bc8a56e5687829e809270c/codex-rs/core/README.md) | Closest precedent to "restricted native helper" in 2025–26 |
| macOS distribution | Notarization + Gatekeeper check **quarantined** files; hardened runtime required for notarization [doc] [notarizing](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution) | A file our app downloads itself is **not quarantined** unless the app opts in (`LSFileQuarantineEnabled`), so Gatekeeper does not assess it [inf from doc] |
| Windows distribution | Authenticode signature; SmartScreen reputation applies to files carrying Mark-of-the-Web [doc] [SmartScreen](https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/) | Same: our own download sets no MOTW unless we set it |

Conclusion: in creative and audio hosts, native plugins are universally trusted code; the best
industry practice is **out-of-process fault isolation** (Bitwig) plus **identity and labels**. Real
confinement exists only where the host owns the process launch (Chrome, agent CLIs). We do own the
launch of our native helpers, so we can do better than AE/Resolve at low cost.

### 2.2 OS confinement realistically available to an out-of-process helper

| Platform | Mechanism | Unprivileged? | What it can restrict | Caveats |
|---|---|---|---|---|
| Linux | **Landlock** (LSM) [doc] [kernel docs](https://docs.kernel.org/userspace-api/landlock.html), [landlock(7)](https://man7.org/linux/man-pages/man7/landlock.7.html) | Yes, self-applied before `exec` | FS access per path hierarchy (ABI 1, kernel 5.13); rename/link `REFER` (ABI 2); truncate (ABI 3); **TCP bind/connect** (ABI 4, 6.7); signal/abstract-unix-socket scoping (ABI 5, 6.12) | Best-effort: query ABI and drop unsupported rights; UDP not covered; default-deny and irrevocable |
| Linux | **seccomp-bpf** + `PR_SET_NO_NEW_PRIVS` | Yes | Syscall deny list (ptrace, process_vm_*, mount, bpf, keyctl, perf_event_open, `socket(AF_INET*)` when no network) | Maintenance per arch; keep a short deny list, not an allow list |
| Linux | Namespaces / **bubblewrap** [doc] [bubblewrap](https://github.com/containers/bubblewrap) | Needs unprivileged user namespaces | Full mount/net/pid isolation | Ubuntu 24.04+ restricts unprivileged userns through AppArmor; Debian/Fedora vary. Optional upgrade only [doc] [Ubuntu](https://ubuntu.com/blog/ubuntu-23-10-restricted-unprivileged-user-namespaces) |
| Linux | cgroups v2 / `setrlimit` | rlimit yes; cgroup via systemd user scope | Memory/CPU/process caps | `RLIMIT_AS` is crude with GPU drivers |
| macOS | **Seatbelt via `sandbox-exec -p <SBPL>`** | Yes | `(deny default)` then allow read of helper bundle and system frameworks, read/write of a per-plugin dir, deny `network*`, deny `process-exec` except self, deny mach-lookup except needed services | `sandbox-exec` is **deprecated since ~2016 but still functional in macOS 26** and used by Codex/agent CLIs [report] [gastown research](https://gastown.dev/docs/research/macos-sandbox-exec). Undocumented SBPL; Apple could remove it |
| macOS | **App Sandbox** entitlement on the helper binary | Applied by the *author's* signature (`com.apple.security.app-sandbox`), not by the host [doc] [App Sandbox](https://developer.apple.com/documentation/security/app-sandbox) | Container-confined FS, network only with `network.client` | Host cannot impose it on an arbitrary binary; can *verify* it via the embedded entitlements. Requires Developer ID signing by the author |
| macOS | Hardened runtime / notarization | Author side | Code-injection resistance, malware scan by Apple | Integrity, not confinement |
| Windows | **AppContainer** (and **LPAC**, Win10 1703+) via `CreateProcess` with `SECURITY_CAPABILITIES` [doc] [AppContainer isolation](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation) | Yes (create profile per plugin) | No network unless `internetClient` capability granted; FS only where ACLs grant the AppContainer SID (grant the per-plugin dir); LPAC also denies most `ALL APPLICATION PACKAGES` resources | GPU/DirectX works in AppContainer (Chromium/Edge use it); some third-party DLLs misbehave. Microsoft's newer composable sandbox API is **experimental** [doc] [CreateProcessInSandbox](https://learn.microsoft.com/en-us/windows/win32/secauthz/createprocessinsandbox) |
| Windows | **Job objects** [doc] [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) | Yes | `KILL_ON_JOB_CLOSE` (kill tree on revocation/host exit), memory cap, active-process limit 1 (no child spawns), UI restrictions (no clipboard/desktop hooks) | Not a security boundary alone |

Rust building blocks exist: `landlock` crate (official), `seccompiler` (Firecracker), `rappct`
(AppContainer) [doc] [docs.rs rappct](https://docs.rs/crate/rappct/0.13.2/source/README.md).

### 2.3 Recommendation: opt-in "restricted native" mode

Keep "Native: full access" as is. Add a second native mode the **author opts into** and the
**host enforces**, worth a softer consent screen and lower review friction:

```toml
[native]
mode = "restricted"         # default "full"
network = []                # or ["api.example.com:443"]; empty = no network
# no filesystem key: helper gets only its own install dir (read) and a per-plugin data/temp dir (rw)
```

Host launch contract (one launcher in `ext-host`, per platform):

| Rule | Linux | macOS | Windows |
|---|---|---|---|
| Kill tree on revoke/host exit | `PR_SET_PDEATHSIG` + process group | process group + kqueue | Job object `KILL_ON_JOB_CLOSE` |
| No child processes | seccomp deny `execve` after start (or Landlock exec restriction) | SBPL `(deny process-exec)` except helper path | Job active-process limit = 1 |
| FS: read install dir, rw data dir, read system libs | Landlock ABI ≥ 1 | SBPL `file-read*` / `file-write*` subpaths | AppContainer + ACL on the two dirs |
| Network: none, or declared hosts | Landlock ABI ≥ 4 denies TCP; declared hosts → route through a host-side **proxy** over the IPC channel instead of raw sockets | SBPL `(deny network*)`; same proxy | No `internetClient` capability; same proxy |
| Memory cap | `RLIMIT_AS`/cgroup | none reliable; watchdog RSS | Job memory limit |
| Images in/out | shared memory handles (memfd / `mach_vm` / section objects) passed over IPC, never paths | same | same |

Policy:

1. **Fail closed per platform.** If the required confinement cannot be applied (Landlock ABI < 1,
   `sandbox-exec` missing, AppContainer creation fails), a restricted helper does **not** run
   silently unconfined; the app offers "run with full access" as an explicit full-access consent.
2. **Network via host proxy only.** Declared hosts are enforced by the host (as Figma's CSP
   `allowedDomains` does) rather than by OS rules; Landlock's port-only network model and SBPL's
   lack of hostname rules make OS-level host filtering unreliable. [inf]
3. **Review tiering.** Restricted native from a community vendor with "Built from source" can be
   bot-merged after automated checks; full-access native keeps two-maintainer approval
   (architecture rule unchanged).
4. **Never claim more than we enforce.** UI copy: "Restricted: cannot use the network or your
   files outside its own folder on this computer." Show the actual enforcement result per run
   (macOS: "Seatbelt profile applied").
5. **Sequence**: Linux Landlock+seccomp first (cheapest, best documented, CI-testable in Actions),
   then Windows AppContainer+Job, then macOS Seatbelt. Acceptance per OS: helper attempts to read
   `~/.ssh`, write outside data dir, open a TCP socket, spawn `sh`; all four denied; legitimate
   render works; revocation kills within 1 s.
6. **Code signing**: do not require Developer ID / Authenticode (cost barrier for hobbyists, and
   our own download path bypasses Gatekeeper and SmartScreen anyway). Record presence as an info
   badge. Rely on attestation + digest pinning for integrity.
7. **Fault isolation for all native**: Bitwig-style per-plugin process (we already have it); add a
   crash counter so a helper that crashes 3 times in a session is auto-disabled for the session
   (Resolve-style blacklist, but scoped and reversible).

---

## 3. Shared projects with missing plugins

### 3.1 How others behave

| App | Behavior when a project references an absent plugin |
|---|---|
| After Effects | Warning on open listing missing effects; the effect instance stays on the layer with its parameters and keyframes; the Project panel search for "missing" filters to **Missing Effects/Fonts/Footage**, and File ▸ Dependencies ▸ Find Missing Effects exists since CC 12 [doc] [Adobe learn](https://www.adobe.com/learn/after-effects/web/find-missing-footage-fonts-aftereffects), [report] [Toolfarm](https://www.toolfarm.com/?p=10216). Render/export is reported to stop with an error until the effect is installed or removed [report] [Adobe forum](https://community.adobe.com/t5/after-effects/missing-effect-in-after-effects/m-p/11483625). Scripting cannot reliably tell which effects are missing [report] [forum](https://community.adobe.com/t5/after-effects-discussions/can-extendscript-see-if-an-effect-is-missing/m-p/12987754) |
| Photoshop | The PSD stores the composited pixels of each smart object result, so the document **opens and looks correct**; the smart filter shows a warning "Filter not available or cannot be applied", settings retained; re-render (editing the smart object) fails for that filter [report] [Adobe forum](https://community.adobe.com/questions-712/dither-filter-no-longer-avaliable-1555516). Cached pixels are the fallback |
| Figma | Plugin data (`setPluginData`, `setSharedPluginData`) is stored on nodes in the file and persists regardless of who has the plugin; private data readable only by the same plugin id; 100 kB per entry; reported loss on branch merge or node duplication edge cases [doc] [setPluginData](https://developers.figma.com/docs/plugins/api/properties/nodes-setplugindata), [report] [forum](https://forum.figma.com/ask-the-community-7/the-data-saving-with-figma-root-setplugindata-in-a-branch-will-not-be-merged-into-main-branch-21226). Plugin output is ordinary nodes, so nothing breaks |
| Blender | Add-on custom properties are ID properties saved in the .blend; when the add-on is absent they survive and appear as raw custom properties; nodes from an absent add-on load as red **"Undefined"** nodes that keep their links; re-enabling the add-on restores them [doc] [custom properties](https://docs.blender.org/manual/en/latest/files/data_blocks.html#custom-properties) |
| DaVinci Resolve (OFX) | Project stores a reference only; missing OFX shows as missing in the node/Inspector; render reported to fail with "missing <plugin>"; no project-wide "which clips use it" view, Smart Bin filter is the workaround [report] [guide](https://tryuncle.com/learn/davinci-resolve/davinci-resolve-plugin-not-found-missing-effect), [forum](https://forum.blackmagicdesign.com/viewtopic.php?p=740774) |
| Premiere Pro | Missing effects are kept on clips and flagged; render shows them as offline/unrendered; Adobe's own "Install missing" exists for Adobe-hosted fonts (Adobe Fonts auto-activation), not for third-party effects [doc] [Adobe Fonts in PS](https://helpx.adobe.com/fonts/using/add-fonts.html) [inf for effects] |
| ComfyUI | "Install Missing Custom Nodes" reads node types from workflow JSON and offers installs (see [blender-and-comfyui.md](blender-and-comfyui.md)) |

### 3.2 Recommendation

Our [lifecycle rule](lifecycle.md) already says "preserved placeholder, export fails". Concretely:

1. **Preserve bytes, never drop.** A placeholder keeps contribution id, release id/version/digest,
   parameter values, keyframes, opaque plugin state, and **unknown fields verbatim**. Save writes
   them back unchanged. This also fixes the observed silent drop in EffectCraft render and
   PhotoCraft smart-object re-render ([haiku-app-cli-state.md](haiku-app-cli-state.md) finding 2).
2. **Cached render fallback, labelled.** Store an optional **baked proxy** per persistent effect
   instance (PhotoCraft: the last result pixels, as PSD does; EffectCraft: an optional
   per-layer render cache at reduced resolution, opt-in on "Collect for sharing" because of size).
   Preview uses it with a visible "Missing plugin: showing cached result" badge. Final export
   refuses unless the user explicitly chooses "Export with cached result" (written into the export
   log) or "Export without effect".
3. **One project-level dependency panel** (fixes AE's and Resolve's discoverability gap): list
   each missing plugin with id, required version, vendor, tier, where used (click to select
   layers), and the reason (not installed / disabled / revoked / incompatible / needs grant).
4. **Install from catalogue, never automatically.** "This project uses 3 plugins: Install all"
   resolves ids against the signed catalogue, shows tier and permissions, and runs the normal
   consent flow. Revoked (`disable`) entries are never offered; a project referencing a revoked
   plugin shows "removed for safety" and only offers the cached result or removal.
5. **Project manifest of dependencies** in the file header (id, version range, digest, catalogue
   origin) so CLI `inspect` can report missing plugins without loading layers.
6. **Data-only plugin state stays readable** (Figma lesson): a FigCraft plugin's per-node data is
   stored on nodes and survives absence; nothing to do except preserve on copy/paste and merge.
7. **CLI**: fresh CLI export with missing plugins exits non-zero with the dependency list; a flag
   `--allow-cached-fallback` mirrors the GUI choice.

---

## 4. Heavy and long-running work

### 4.1 How hosts do it

| Host | Progress / cancel | Threads / background | GPU |
|---|---|---|---|
| After Effects SDK | `PF_PROGRESS(in_data, count, total)` returns non-zero if the user cancelled; `PF_ABORT` polls cancel [doc] [AE SDK](https://ae-plugins.docsforadobe.dev/effect-basics/PF_InData/) | **Multi-Frame Rendering** (AE 22+): effects declare thread safety and render frames concurrently; iterate suites (`PF_Iterate8Suite`) split work across cores; AEGP idle hooks for background tasks [doc] [MFR](https://ae-plugins.docsforadobe.dev/effect-details/multi-frame-rendering-in-ae/) | GPU effects via SmartFX with Metal/CUDA/OpenCL/DirectX device setup selectors [doc] [GPU](https://ae-plugins.docsforadobe.dev/effect-details/gpu-build-instructions/) |
| Photoshop UXP | `executeAsModal(fn, {commandName})`; `executionContext.reportProgress({value})` (indeterminate by default, bar after a few seconds), `isCancelled`, `onCancel`; `interactive` mode since 23.3 [doc] [executeAsModal](https://developer.adobe.com/photoshop/uxp/2022/ps_reference/media/executeasmodal/) | JS single-threaded; heavy pixel work via **Imaging API** (`getPixels`/`putPixels`) or a C++ **hybrid plugin** [doc] [Imaging](https://developer.adobe.com/photoshop/uxp/2022/ps_reference/media/imaging/), [hybrid](https://developer.adobe.com/photoshop/uxp/2022/guides/hybrid-plugins/) | Through hybrid native code only |
| Photoshop C SDK filters | `AdvanceState`/`TestAbort`/`UpdateProgress` callbacks [doc] [PS SDK](https://developer.adobe.com/photoshop/photoshop-sdk/) | Tiles via `inRect`/`outRect` | Native |
| Figma | No progress API; plugins chunk work with `await` and report via their UI iframe or `figma.notify`; the main thread blocks the editor while running [doc] [plugin execution](https://developers.figma.com/docs/plugins/how-plugins-run/) | UI iframe may use Web Workers; sandbox has none | WebGL/WebGPU only in the UI iframe |
| Blender | Modal operators + `bpy.app.timers` for incremental work; `wm.progress_begin/update/end`; Esc cancels modal ops [doc] [timers](https://docs.blender.org/api/current/bpy.app.timers.html), [WindowManager](https://docs.blender.org/api/current/bpy.types.WindowManager.html) | Python threads may not touch `bpy`; heavy work in subprocesses or numpy | `gpu` module for drawing/offscreen shaders, compute shaders via shader info in recent versions [doc] [gpu](https://docs.blender.org/api/current/gpu.html) |
| OpenFX | `OfxProgressSuiteV1`, `abort()` in the image effect suite; `OfxMultiThreadSuiteV1` host-provided worker pool; render-thread-safety declaration [doc] [OFX reference](https://openfx.readthedocs.io/en/main/Reference/index.html) | Host multi-thread suite | OpenGL render suite, and CUDA/Metal/OpenCL render extensions in OFX 1.5 [doc] |

Common pattern: **host owns threads, progress UI and cancellation; plugin polls cancel and
reports progress**; heavy hosts let the plugin declare thread safety so the host parallelises
across frames/tiles.

### 4.2 What Wasm can do today

- **Wasm 3.0** is the W3C standard since 2025-09: memory64, multiple memories, GC, tail calls,
  exception handling, **relaxed SIMD** (with a deterministic profile) [doc]
  [webassembly.org 3.0](https://webassembly.org/news/2025-09-17-wasm-3.0/).
- **Fixed-width SIMD (128-bit)** is standard since 2.0 and usable from Rust with
  `-C target-feature=+simd128` [doc] [Rust core::arch::wasm32](https://doc.rust-lang.org/core/arch/wasm32/index.html).
- **Threads** (shared memory + atomics) are a separate proposal; Wasmtime implements
  `wasi-threads` only as experimental, and WASI's direction is "shared-everything threads", not
  finished [doc] [wasi-threads](https://github.com/WebAssembly/wasi-threads),
  [wasmtime proposals](https://docs.wasmtime.dev/stability-wasm-proposals.html). Treat in-module
  threads as unavailable.
- **wasi-nn / GPU (wasi-gfx/wgpu)**: not standardised for embedding; not available to us. [doc]
- **Our engine is `wasmi` 2**, an interpreter (`deterministic`, `portable-dispatch`): expect
  roughly an order of magnitude below a JIT for scalar loops [inf; verify with a benchmark]. It is
  the right choice for determinism, fuel and portability, and the wrong place for per-pixel inner
  loops over 4K frames.

Consequence: **parallelism must come from the host** (one instance per render thread, already the
design; tile/frame splitting by the host), and **speed must come from host kernels**.

### 4.3 Recommendation: host compute kernels on image handles

A new versioned contract, `compute.kernels@1.0`, available to wasm effects/filters in EffectCraft
and PhotoCraft (and FigCraft image fills later):

- **Opaque handles**: the plugin receives `ImageId` for inputs, never raw buffers unless it asks
  for `pixels.read(id, rect)` (bounded, fuel-charged). Kernels return new handles.
- **Graph, not calls**: the plugin builds a small DAG (`blur(src, σ) → threshold → add(src)`),
  returns it, and the host executes it tiled, multi-threaded (rayon + SIMD) or on GPU (wgpu in
  EffectCraft's compositor), with fusion where possible. One host call per graph, so fuel and
  boundary cost stay small. This is the Halide/libvips/AE-GPU model. [inf]
- **Deterministic contract**: each kernel specifies precision and tolerance so preview (GPU) and
  final (CPU) agree within the documented tolerance; that fits the existing tolerance rule in
  [lifecycle.md](lifecycle.md).
- **Fuel by cost**: charge per output pixel × kernel weight; size limits per handle; cancellation
  checked between graph nodes and tiles; progress reported by the host from tile counts.
- **Colour**: kernels operate in the host's working space (linear float); colour transforms go
  through the host's OCIO-style pipeline so plugins never guess colour management.

Kernel set v1 (each is a widely shared primitive in the section 5 plugins):

| Group | Kernels |
|---|---|
| Filter | gaussian/box blur (separable, σ up to large via downsample pyramid), directional & radial/zoom blur, general convolution (≤ 31×31), unsharp, median, bilateral/surface blur, morphology (dilate/erode/open/close), edge (Sobel) |
| Geometry | resample (bilinear/bicubic/Lanczos), affine and perspective transform, displacement map, mesh warp, polar ↔ rect |
| Colour | 3×4 matrix, 1D curves/LUT, 3D LUT (tetrahedral), levels, HSL adjust, threshold/luminance key, luminosity range mask (for Lumenzia/TK-style masks), channel shuffle, premultiply |
| Composite | blend modes (normal/add/screen/multiply/overlay/soft light…), mask apply, over/under |
| Generate | seeded noise (value/Perlin/simplex/fractal), gradients, solid, **sprite splat** (N points with size/colour/alpha/rotation → additive/over raster) for particle and plexus-style plugins, **polyline/stroke raster** (anti-aliased, glow-friendly) for saber-style, text rasterise via host fonts |
| Frequency / stats | FFT convolution (bloom, lens kernels), histogram, min/max/mean per channel, downsampled pyramid |
| Time (EffectCraft) | `frame.at(layer, t)` request for temporal effects, frame blend; optical flow as an **optional** host kernel (expensive; for retime plugins) |
| AI (later) | `model.run(resource, inputs)` via host runtime on declared model resources; not in v1 |

With these, the plugin does what wasm is good at (parameters, simulation of thousands of
particles, geometry, control logic) and the host does what it is good at (per-pixel bulk work).
Native helpers remain the escape hatch for vendor AI models and custom GPU code.

Long-running jobs: add `job.progress(fraction, label)` and `job.cancelled()` host calls for
commands and panels (PS UXP model), host-shown progress after 1–2 s, Esc to cancel, and
`job.background` for work that must not block the editor (results applied as one undo
transaction when done, abandoned if the document changed incompatibly).

---

## 5. Most-used plugins and the capabilities they need

Popularity basis: vendor prominence, aescripts/Toolfarm best-seller lists and long-standing
community usage [report]. Not a measured ranking.

### 5.1 After Effects

| # | Plugin (vendor) | What it does | Needs |
|---|---|---|---|
| 1 | Trapcode Particular (Maxon/Red Giant) [doc](https://www.maxon.net/en/red-giant/trapcode-suite) | 3D particle system: emitters (point/layer/text/OBJ), physics, sprites, shading, AE camera/lights | pixel output, GPU, 3D (comp camera/lights), layer sampling, custom UI (Designer window), heavy compute |
| 2 | Trapcode Shine (Maxon) | Volumetric light rays from luminance | pixel access, radial blur, GPU |
| 3 | Element 3D (Video Copilot) [doc](https://www.videocopilot.net/products/element2/) | Real-time OBJ/C4D model render with materials inside AE | GPU (own OpenGL/DirectX), 3D, file IO (models, textures), custom UI (scene setup), camera/lights |
| 4 | Saber (Video Copilot, free) [doc](https://www.videocopilot.net/blog/2016/03/new-plug-in-saber-100-free/) | Energy beams/glow along paths, masks or text | pixel output, mask/path access, text, glow (blur + add), keyframes |
| 5 | Deep Glow (Plugin Everything / aescripts) [doc](https://aescripts.com/deep-glow/) | Physically-inspired exponential glow | pixel access, multi-scale blur, GPU, 32-bpc colour |
| 6 | Optical Flares (Video Copilot) [doc](https://www.videocopilot.net/products/opticalflares/) | Lens flare designer with presets | custom UI (flare designer), 3D light/position tracking, file IO (presets), pixel output |
| 7 | Twixtor (RE:Vision Effects) [doc](https://revisionfx.com/products/twixtor/) | Optical-flow retiming / slow motion | temporal frame access (other times), optical flow, heavy compute/GPU |
| 8 | Magic Bullet Looks (Maxon) [doc](https://www.maxon.net/en/red-giant/magic-bullet-suite) | Colour grading with a full-screen look designer | pixel access, colour transforms/LUTs, GPU, custom full-window UI, presets |
| 9 | Sapphire (Boris FX) [doc](https://borisfx.com/products/sapphire/) | ~270 effects/transitions (glows, blurs, lens flares, distortions) | pixel access, GPU, transitions (two inputs), presets browser UI, temporal |
| 10 | Plexus (Rowbyte) [doc](https://www.rowbyte.com/plexus) | Points → lines/triangles network from geometry, layers, text | 3D, geometry generation, layer sampling, camera, line raster |
| 11 | Duik Angela (RxLaboratory, free/open) [doc](https://rxlaboratory.org/tools/duik-angela/) | Rigging and character animation: IK, bones, controllers | script/panel UI, **layer creation**, **expressions**, keyframe read/write, property linking, undo |
| 12 | Motion v4 (Mt. Mograph) [doc](https://mtmograph.com/motion) | Animation toolkit panel: easing, anchors, clone, jump, warp | panel UI, keyframe read/write, layer creation, selection, expressions |
| 13 | Flow (aescripts) [doc](https://aescripts.com/flow/) | Bezier easing curve editor applied to keyframes | panel UI with custom drawing, keyframe/ease write, presets |
| 14 | Overlord (Battle Axe) [doc](https://www.battleaxe.co/overlord) | Live transfer of Illustrator shapes to AE shape layers | **IPC/network to another app**, layer creation (shape), file IO |

### 5.2 Photoshop

| # | Plugin (vendor) | What it does | Needs |
|---|---|---|---|
| 1 | Nik Collection (DxO) [doc](https://nikcollection.dxo.com/) | Color Efex, Silver Efex, Viveza, Dfine, Sharpener; U-Point local control | pixel access, custom full-window UI, local masks (control points), GPU, smart-filter compatible |
| 2 | Portraiture (Imagenomic) [doc](https://imagenomic.com/Products/Portraiture) | Automatic skin smoothing with skin-tone mask | pixel access, mask generation, surface blur, custom UI |
| 3 | Topaz Photo (Topaz Labs) [doc](https://www.topazlabs.com/topaz-photo) | AI denoise/sharpen/upscale | pixel access, **AI models (GPU/NPU)**, large file IO (models), background job, size change (upscale) |
| 4 | Coolorus (free colour wheel panel) [doc](https://coolorus.com/) | Colour wheel panel for painters | panel UI with custom drawing, read/write foreground colour, events (colour changed) |
| 5 | Retouch4me (Retouch4me) [doc](https://retouch4.me/) | AI retouch plugins (heal, dodge&burn, eye vessels) + panel; local or cloud | pixel access, AI models, layer creation (results on new layers), optional network, background jobs |
| 6 | Infinite Color Panel [doc](https://www.infinitecolorpanel.com/) | Randomised colour grade generator with adjustment layers | panel UI, **adjustment-layer creation**, randomness, batch |
| 7 | TK8 luminosity masks (Tony Kuyper) [doc](https://tonykuyper.photoshelter.com/) | Luminosity/zone/colour masks, exposure blending | panel UI, channel/selection math, mask creation, actions |
| 8 | Lumenzia (Greg Benz) [doc](https://lumenzia.com/) | Luminosity masking panel with live preview | panel UI, live preview overlay, mask/selection creation, layer creation |
| 9 | Fontself Maker (Fontself) [doc](https://www.fontself.com/) | Build OpenType fonts from drawn layers | read vector/pixel layers, file IO (font export), panel UI, font install |
| 10 | ON1 Photo RAW / Effects (ON1) [doc](https://www.on1.com/products/photo-raw/) | Filters, presets, masking, portrait AI | external-app round-trip (file IO), pixel access, AI |

### 5.3 Figma

| # | Plugin | What it does | Needs |
|---|---|---|---|
| 1 | Unsplash [doc](https://unsplash.com/) | Insert stock photos as image fills | network (API), image fill write, panel UI |
| 2 | Iconify [doc](https://iconify.design/) | Search 200k+ icons, insert as vectors | network, vector node creation, panel UI search |
| 3 | Content Reel (Microsoft) [report] | Fill text/images with placeholder content | text/image write to selection, data resources, panel UI |
| 4 | Autoflow [doc](https://autoflow.tool/) | Draw flow arrows between frames that follow movement | node creation, **document change events**, plugin data on nodes |
| 5 | Stark [doc](https://www.getstark.co/) | Contrast checks, vision simulation, a11y annotations | read styles/colours, render preview (pixel export), panel UI, network (account) |
| 6 | remove.bg [doc](https://www.remove.bg/integrations) | Background removal on image fills via API | image fill read/write, network, API key storage |

### 5.4 Capability matrix

Legend: ● essential, ○ secondary. Columns: **Px** pixel read/write · **GPU** GPU compute ·
**UI** custom panel/window · **KF** timeline/keyframes · **Ex** expressions/linking ·
**3D** camera/lights/geometry · **Lay** layer/node creation · **Net** network · **IO** file IO ·
**Job** background/long job · **T** temporal frame access · **AI** model inference.

| Plugin | Px | GPU | UI | KF | Ex | 3D | Lay | Net | IO | Job | T | AI |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Particular | ● | ● | ● | ○ |  | ● | ○ |  |  | ○ | ○ |  |
| Shine | ● | ○ |  |  |  |  |  |  |  |  |  |  |
| Element 3D | ● | ● | ● |  |  | ● |  |  | ● | ○ |  |  |
| Saber | ● | ○ |  | ○ |  |  |  |  |  |  |  |  |
| Deep Glow | ● | ● |  |  |  |  |  |  |  |  |  |  |
| Optical Flares | ● | ○ | ● |  |  | ● |  |  | ○ |  |  |  |
| Twixtor | ● | ● |  |  |  |  |  |  |  | ● | ● |  |
| Magic Bullet Looks | ● | ● | ● |  |  |  |  |  | ○ |  |  |  |
| Sapphire | ● | ● | ○ |  |  |  |  |  | ○ |  | ○ |  |
| Plexus | ● | ○ | ○ |  |  | ● |  |  |  |  |  |  |
| Duik | | | ● | ● | ● |  | ● |  |  |  |  |  |
| Motion | | | ● | ● | ○ |  | ● |  |  |  |  |  |
| Flow | | | ● | ● |  |  |  |  |  |  |  |  |
| Overlord | | | ● |  |  |  | ● | ● | ○ |  |  |  |
| Nik Collection | ● | ● | ● |  |  |  | ○ |  |  |  |  |  |
| Portraiture | ● | ○ | ● |  |  |  |  |  |  |  |  | ○ |
| Topaz Photo | ● | ● | ● |  |  |  |  |  | ● | ● |  | ● |
| Coolorus |  |  | ● |  |  |  |  |  |  |  |  |  |
| Retouch4me | ● | ● | ○ |  |  |  | ● | ○ | ○ | ● |  | ● |
| Infinite Color |  |  | ● |  |  |  | ● |  |  |  |  |  |
| TK8 / Lumenzia | ● |  | ● |  |  |  | ● |  |  |  |  |  |
| Fontself | ○ |  | ● |  |  |  |  |  | ● |  |  |  |
| ON1 | ● | ● | ● |  |  |  | ○ |  | ● | ● |  | ● |
| Unsplash / Iconify |  |  | ● |  |  |  | ● | ● |  |  |  |  |
| Autoflow |  |  | ○ |  |  |  | ● |  |  |  |  |  |
| Stark | ○ |  | ● |  |  |  |  | ○ |  |  |  |  |
| remove.bg | ● |  | ○ |  |  |  |  | ● |  |  |  | ● |

Observation: the top revenue plugins (Particular, Element, Looks, Sapphire, Nik, Topaz) all need
**GPU-speed pixel work plus a custom full-window UI**. The top free/community tools (Duik, Motion,
Flow, Coolorus, Lumenzia, Infinite Color) need **panels + document edits**, not pixels. Our beta
already covers the second group better than the first; section 4.3 kernels close the first.

### 5.5 Flagship example plugins to build (original names, no cloning)

Chosen so that six plugins together exercise the widest capability set with the kernel contract,
the sandbox, one restricted native helper, and the missing-plugin path.

**EffectCraft**

1. **"Emberfield"**: particle generator effect. Emitters from point, layer alpha or text;
   wasm simulates particles (forces, turbulence via seeded noise, lifespan), host `sprite splat`
   + glow kernels render. Uses comp camera for 3D projection. Exercises: Px out, kernels/GPU, 3D
   camera, animatable params/keyframes, deterministic seek (re-simulate from seed + cache),
   multi-instance state, heavy fuel budgets, cached-proxy fallback when missing.
2. **"Lightwell"**: glow + light rays + streak effect (Deep Glow / Shine / Saber territory):
   threshold → pyramid blur → radial blur → blend; optional path/mask input for beam along a
   path. Exercises: Px in/out, kernel graph fusion, mask/path read, 32-bpc colour, GPU preview vs
   CPU final tolerance test.
3. **"Rigkit"**: animation panel (Duik/Motion/Flow territory): easing curve editor with custom
   drawing, apply eases to selected keyframes, create null/controller layers, link properties with
   expressions, one undo step per action. Exercises: docked panel UI, selection events, keyframe
   read/write, layer creation, expressions/linking, transactions/undo, no pixels.

**PhotoCraft**

4. **"Tonemask"**: luminosity/colour range mask panel (TK/Lumenzia territory) with live
   overlay preview; creates masks and adjustment layers; random-grade generator mode (Infinite
   Color territory). Exercises: panel UI, live preview overlay, luminosity-mask kernel, mask and
   adjustment-layer creation, histogram stats, undo.
5. **"Softskin"**: portrait retouch filter (Portraiture/Nik territory): skin-tone range mask +
   bilateral/frequency-separation blur, output as smart filter **or** to new layers; full-window
   before/after UI. Exercises: Px in/out, kernels (bilateral, FFT optional), non-destructive smart
   filter persistence with cached composite (missing-plugin fallback), custom large UI.
6. **"Upscaler"**: AI denoise/upscale as a **restricted native helper** (Topaz/Retouch4me
   territory): ships an ONNX model as a declared resource, runs in a restricted helper (no
   network, rw only its data dir), image passed by shared memory, progress + cancel, background
   job, canvas size change. Exercises: native restricted mode on three OSes, resource download,
   long job with progress/cancel, attested native build, revocation kill of a running helper.

A seventh, optional, for FigCraft: **"Iconpull"** (Iconify/Unsplash territory) to exercise
declared-host network via the host proxy, vector node creation and per-node plugin data.

Coverage of the matrix columns by the six: Px (1,2,5,6), GPU via kernels (1,2,5), native GPU/AI
(6), UI panel (3,4) and full window (5), KF (1,3), Ex (3), 3D (1), Lay (3,4,5), Net (none by
design; FigCraft extra), IO/resources (6), Job (6, and 1 for caching), T (1 via seek determinism;
a retime effect is deliberately excluded as the optical-flow kernel is optional), AI (6).

---

## 6. Decisions proposed

| # | Decision | Default proposed |
|---|---|---|
| D1 | Add "Built from source" badge from `gh attestation verify` in catalogue CI; require for verified and all native | Yes |
| D2 | Verified vendor: domain proof (TXT or `.well-known`) or GitHub verified-domain org, + 90-day tenure + 2FA + attested releases; one maintainer click | Yes |
| D3 | Refuse non-immutable GitHub releases as catalogue assets | Yes |
| D4 | All automated gates fail closed on tool error | Yes |
| D5 | Popularity from release-asset download counts snapshotted by catalogue build; fixed per-app category taxonomy | Yes |
| D6 | Opt-in `native.mode = "restricted"`: Landlock+seccomp, AppContainer+Job, Seatbelt; fail closed; network only via host proxy | Yes, Linux first |
| D7 | Code signing of native helpers recorded, not required | Yes |
| D8 | Missing plugins: preserve verbatim, labelled cached fallback, project dependency panel, catalogue install with consent, export refuses unless explicit | Yes |
| D9 | `compute.kernels@1.0` graph contract on opaque image handles; host executes tiled/parallel/GPU | Yes, before more effect examples |
| D10 | Job progress/cancel/background host calls for commands and panels | Yes |
| D11 | Six flagship examples as in 5.5 | Yes |

Open questions: whether `wasmi` should gain a JIT engine profile for compute-heavy effects
(benchmark first); whether EffectCraft persists baked proxies by default (size); whether Apple
removing `sandbox-exec` would force restricted native on macOS to require App-Sandbox-signed
helpers instead.

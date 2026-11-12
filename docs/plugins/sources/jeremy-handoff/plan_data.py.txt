"""Static content for the extensions handoff: decisions, apps, requirements, rules.

Only Jeremy's own words are marked Decided. Everything else is Open or a recommendation.
Task rows live in plan_tasks.py. build.py renders both and validates the page.
"""

UPDATED_ISO = "2026-11-04"
UPDATED_UK = "04/11/2026"

DIRECTION = (
    "Jeremy's direction, 16/10/2026: while the AI infrastructure is being sorted, start the plan for "
    "a shared plug-in, theme and extension browser that is native to ArtCraft Launcher and reachable "
    "from each app. Then, same day: start with internal (first-party) extensions, but build the structure "
    "for third party that requires heavy security gates. Third party stays switched off until every gate passes. "
    "Then: build Phase 1 end to end (threat model, core and store crates, launcher Themes screen, PhotoCraft theme adapter, into his test build), crates in a standalone workspace, and a public repo for it (created on his GitHub as Jeremy8776/craft-extensions on 16/10/2026, first commit 5500b37, CI green; moved to storytold/craft-extensions on 04/11/2026). Built so far (pushed to that repo as 33adcb4, CI green on Linux and Windows): craft-ext-core and craft-ext-store with tests and mutation checks. The launcher Extensions screen (Themes and Installed tabs) is built and tested in the craft-launcher-ext worktree with a debug build at craft-launcher-ext/target/dev. The launcher work is on branch feat/extensions (70b9612) in Jeremy8776/craft-launcher, a fork made 16/10/2026 because storytold/craft-launcher is read-only for him. The PhotoCraft adapter is on feat/extensions-theme-adapter (ecb056cc) in Jeremy8776/photocraft. No pull request has been opened. Jeremy has reviewed the launcher TEST BUILD UI; PhotoCraft remains a separate verification gate. 16/10/2026 follow-up: the launcher UI polish stays in the throwaway test branch until the real plug-in infrastructure and PhotoCraft host work end to end. Then extract it cleanly, verify a fresh checkout and hand off source PRs plus a versioned installer candidate. The example plug-ins are demo data, not shipped packages. Nothing is pushed or published by this plan."
)

# (id, question, recommendation, if not, state)
DECISIONS = [
    ("d0", "Shared plug-ins, themes and extensions browser, native to the launcher, reachable from every app",
     "Scope as given.", "None; this is the brief.", "Decided (Jeremy, 16/10/2026)"),
    ("d1", "What does \"extension\" cover?",
     "Three kinds: themes, plug-ins, resources (brush sets, LUTs, swatches, templates).",
     "Themes and plug-ins only; resources left out of the package envelope.", "Open"),
    ("d2", "User-facing name",
     "Section \"Extensions\", tabs Themes, Plug-ins, Resources (PhotoCraft already says Plug-ins).",
     "\"Add-ons\" throughout.", "Open"),
    ("d3", "Whose extensions can the launcher install?",
     "Internal first (built by storytold). Build the third-party structure now, behind heavy security gates, switched off in release builds until every gate passes.",
     "None; this is his answer. The download rule still needs a small agreed widening for the hub repo (n5), but only inside the storytold organisation.",
     "Decided (Jeremy, 16/10/2026)"),
    ("d4", "Where the catalogue index lives",
     "storytold/crafthub if the maintainers confirm it was meant for this (it is empty today, README says \"Placeholder for merges\"); otherwise a repo under Jeremy's account first.",
     "A folder inside craft-launcher. Couples catalogue edits to launcher releases.", "Open"),
    ("d5", "Folder for the shared store",
     "Same shared folder as the AI store, under the user's local data folder, with portable mode handled explicitly.",
     "craft-settings per-app folders.", "Decided (Jeremy, 16/10/2026)"),
    ("d6", "Does the launcher accept themes itself?",
     "Optional. The default stays the look that matches getartcraft.com.", "Launcher never themed.", "Open"),
    ("d7", "Where the contract and store crates live while building",
     "A standalone workspace, public as storytold/craft-extensions (moved from Jeremy8776/craft-extensions on 04/11/2026), mirroring craft-libs rules and the craft-ai-model pattern. Whether the contract crate later moves to craft-libs stays open until two consumers exist.",
     "Inside the launcher workspace, or straight into a craft-libs fork branch.", "Decided (Jeremy, 16/10/2026)"),
    ("d8", "Signing and metadata scheme for third-party gates",
     "Ed25519 detached signatures, publisher keys pinned in the index, and TUF-style signed metadata (version, expiry, revocation). Look at adopting TUF's design before inventing our own.",
     "Hash pinning only. Not enough for third party: it proves the file matches the index, not who made it or whether the index is fresh.", "Open"),
    ("d9", "Provenance for third-party plug-ins",
     "The hub builds plug-ins from source in its own CI and attests the build, rather than accepting uploaded binaries. Costly, strongest.",
     "Accept authors' release assets with a recorded source commit; reviewers reproduce builds by hand.", "Open"),
    ("d10", "Independent security review before the gate opens",
     "Yes: an outside review of unpack, the sandbox host and the signature code, with findings fixed or accepted in writing.",
     "Internal review only. Faster, weaker; Jeremy and the maintainer would be accepting that risk.", "Open"),
    ("d11", "Where do plug-in download counts and rankings come from?",
     "Start with download counts on storytold-hosted release files. They need no tracking inside the apps and no new server.",
     "A counting service of our own, which allows ratings too but needs a server, a privacy policy and protection against inflated counts.", "Open"),
    ("d12", "What may a plug-in do, and how is each ability made safe?",
     "A per-app ability list declared in the manifest and shown before install. Add each ability only once the sandbox can enforce it.",
     "Keep plug-ins to filters only. Simple and safe, but it does not meet the aim of a vast range of ability across all apps.", "Open"),
]

TIERS = [
    ("Made by ArtCraft", "storytold, built and released by the organisation", "Hash pinned in the index; downloaded only from storytold releases", "On"),
    ("Added from a file", "The person's own file", "Bounded unpack and validators; the sandbox still applies; clear warning in plain words", "On"),
    ("Reviewed third party", "An outside publisher", "Publisher signature, pinned key, fresh signed index, revocation, reserved namespaces, review of the index entry, provenance", "Off: compiled out of release builds and refused at run time until gate sg13"),
]

# (app id, name, what it is (repo description), likely plug-in kind, notes)
APPS = [
    ("a-photo", "PhotoCraft", "photo editor", "filter (exists, wasmi host)", "Own Tokens with shape flags; plug-in docs exist; first adopter"),
    ("a-light", "LightCraft", "non-destructive photo editor", "filter (likely second wasm host consumer)", "LUT and preset resources"),
    ("a-vector", "VectorCraft", "vector art tool", "later", "Already has a colortheme command; swatch resources"),
    ("a-design", "DesignCraft", "page and layout design", "later", "Swatches, templates"),
    ("a-pdf", "PdfCraft", "PDF reader and editor", "none yet", "Has desktop_theme.rs; themes first"),
    ("a-film", "FilmCraft", "nonlinear video editor", "later", "LUTs; transitions as a future kind"),
    ("a-effect", "EffectCraft", "motion graphics and compositing", "filter / effect (likely second consumer)", "Effects as a plug-in kind"),
    ("a-sound", "SoundCraft", "Pro Tools-style audio", "later (audio buffers, different ABI)", "Separate design needed for audio plug-ins"),
    ("a-word", "WordCraft", "Word-style editor", "none yet", "Ribbon shell; themes and templates"),
    ("a-deck", "DeckCraft", "PowerPoint-style presentations", "none yet", "Ribbon shell; themes and templates"),
    ("a-grid", "GridCraft", "Excel-style spreadsheet", "none yet", "Ribbon shell; themes and templates"),
    ("a-cad", "CADCraft", "AutoCAD-style CAD", "none yet", "Themes only to start"),
    ("a-fig", "FigCraft", "interface design and prototyping", "later", "Missing from the launcher's own app list"),
    ("a-ani", "AniCraft", "2D animation editor", "later", "Missing from the launcher's own app list"),
]

# (id, requirement)
REQUIREMENTS = [
    ("q1", "Install once, enable per app. Installing in one place makes the extension available to every compatible app."),
    ("q2", "An app with no launcher still works: it can install from a file and manage what it has, using the same store."),
    ("q3", "A running app notices an install or a switch without a restart and without IPC."),
    ("q4", "Every package is hostile input: bounded unpack, no path escape, size and count caps, hashes checked before use. Nothing panics."),
    ("q5", "Themes carry no code. Plug-ins have no ambient authority (no files, network, clock) in version 1; the permissions field must be empty, for every tier."),
    ("q6", "One theme file works in every app: a small shared palette, per-app overrides optional, unknown apps ignored."),
    ("q7", "Anything a person reads is plain language, enforced by a test. No ABI, wasm, sandbox or adapter in visible strings."),
    ("q8", "The contract and store core build for wasm32-unknown-unknown; file and process code is cfg-gated."),
    ("q9", "Each host draws its own view: launcher look in the launcher, the app's theme inside an app. Shared code owns data and a neutral fallback only."),
    ("q10", "First-party entries pin a SHA-256 and come only from storytold releases. The page says plainly that hash pinning does not prove who made a file."),
    ("q12", "The third-party path exists in the schema and the code but is off: behind a cargo feature that release builds never enable, refused at run time, and guarded by a test that fails if it is enabled."),
    ("q13", "Third-party trust needs a publisher signature, a pinned key, a fresh signed index (expiry, no rollback), a revocation list and reserved first-party namespaces. A revoked package turns off in every app."),
    ("q14", "No silent change for third party: a new key, wider permission or downgrade needs a fresh yes; updates never apply on their own."),
    ("q11", "Launcher strings go through tl! into the four catalogs. Extension names and descriptions are third-party data and stay untranslated."),
]

# (option, for, against)
STORE_OPTIONS = [
    ("A. Shared folder under %LOCALAPPDATA% (same as the AI store)",
     "One place; apps and launcher read the same files; matches the AI layer's existing divergence.",
     "Departs from craft-libs' settings-storage policy, a stated review question. Portable launcher mode puts its data next to the exe, so apps would not find it."),
    ("B. craft-settings per-app folders",
     "Follows the shared policy.",
     "Cross-app reads need every app to know every other app's folder; duplication."),
    ("C. Launcher owns the files, apps ask over IPC",
     "Single writer.",
     "Apps need a running launcher, which breaks requirement q2. Rejected."),
]
STORE_RECOMMENDATION = "Recommendation: A, decided together with the AI store, with a stated answer for portable mode."

OWNERSHIP = [
    ("Package contract and validators (craft-ext-core)", "Jeremy leads; craft-libs after two consumers exist", "Wasm-safe, small, no framework"),
    ("Store and install-from-file (craft-ext-store)", "Jeremy leads; standalone repo per d7", "Atomic writes via craft-atomic; bounded unpack"),
    ("Catalogue index repo", "Per d4; entries reviewed by maintainers", "CI runs the same validator"),
    ("Launcher Extensions screen, fetch, updates", "Launcher (Jeremy's lane for this plan)", "Own module tree, own branch"),
    ("In-app panel, commands, theme adapter, plug-in host", "Each app's maintainer, in that app's repo", "Reported as the next decision, not started by default"),
]

ADAPTER_CHECKLIST = [
    "Map the shared palette to the app's own Tokens, derive the rest, then apply [app.<id>] overrides.",
    "Re-scan the store when the generation counter changes; fall back to the built-in theme if the active one is gone or fails validation.",
    "Themes & Extensions panel: installed for this app, enable, disable, remove, install from file.",
    "A \"Browse more\" button that opens the launcher at this app's filter; if the launcher is missing, open its download page.",
    "Commands (extension.list, extension.enable, theme.set, extension.open_browser) so scripts and agents can drive it.",
    "Plug-in host only if the app has a plug-in kind; resource importers listed with their formats.",
    "Strings through the app's i18n; a test that scans visible strings for developer words.",
]

RULES = [
    "Only Jeremy's words are marked Decided. A maintainer's remark is Open or leaning.",
    "Nothing leaves this machine without his specific yes: no push, no PR, no issue comment, no message to a maintainer.",
    "Targeted cargo only: one process at a time, cargo test -p <crate> -j 8. No workspace builds, no cold PhotoCraft build locally; upstream CI is the first full compile.",
    "Tests first, then a mutation check on every validator. Report plainly what was not run.",
    "500 line soft limit, 700 hard, per file. The launcher's views.rs is already 971: new UI goes in features/extensions/.",
    "Never crash: no unwrap, expect or panic in non-test code; network and archive contents are hostile.",
    "Public repos must not carry the machine's profile folder. Scrub before any commit.",
    "No em or en dashes in documents. No unattended automation.",
    "Anything Jeremy may test goes into his test build in the same turn, unasked.",
    "Never enable the third-party feature in a release build. Opening the gate (sg13) needs Jeremy's yes and the maintainer's yes to that specific change.",
    "Test keys only in the repo. No real signing key, token or secret ever enters a file, a log or this transcript.",
]

QUESTIONS = [
    "ArtCraft holds the signing material for the apps. Will they give a public key for signing the first-party index (the AI handoff asked the same question)?",
    "Who pays for and chooses the independent reviewer (d10), and who triages the findings?",
    "Who are the index reviewers for the two-maintainer rule, given Jeremy has read-only access to the organisation?",
    "Portable launcher mode keeps data beside the exe. Where does the shared store live then, and do apps find it?",
    "A theme that fails the 4.5:1 text contrast check: reject at install, or install with a warning?",
    "An active theme is removed while an app is running: fall back to the built-in theme silently, or tell the person?",
    "Extension licences: must the index require an open licence, or only record whatever the author states?",
    "Updates: ask each time (like apps) or update extensions automatically?",
    "Does a plug-in that fails to load in one app get disabled everywhere or only there?",
]

REFERENCES = [
    ("Design notes (package envelope, storage, trust, launcher screen)", "design-notes.md"),
    ("Launcher AGENTS.md (layering, golden rules)", "https://github.com/storytold/craft-launcher/blob/main/AGENTS.md"),
    ("PhotoCraft plug-in contract (ABI v1)", "https://github.com/storytold/photocraft/blob/main/docs/plugins.md"),
    ("AI layer, the pattern this follows", "https://github.com/storytold/craft-ai-integrations"),
]

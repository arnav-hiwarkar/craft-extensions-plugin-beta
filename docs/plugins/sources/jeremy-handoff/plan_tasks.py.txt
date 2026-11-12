"""Task rows for the extensions handoff, grouped by phase. Ids match the todo list.

Fields: id, task, depends on, done when, status. Every status is Not started unless stated.
Direction (Jeremy, 16/10/2026): internal extensions first; build the structure for third party
now, behind heavy security gates, switched off until every gate passes.
"""

SG_1_TO_8 = "sg1, sg2, sg3, sg4, sg5, sg6, sg7, sg8"
SG_1_TO_12 = SG_1_TO_8 + ", sg9, sg10, sg11, sg12"

PHASES = [
    ("p0", "Phase 0. Decide and line up (no code)", [
        ("s0a", "Settle the remaining open decisions with Jeremy (d1, d2, d4 to d10)", "", "Each row in the decisions table is Decided in his words", "Not started"),
        ("s0b", "Draft a note to the maintainer about crafthub (d4) and the hub repo joining the download rule; show it, send only on his yes", "s0a", "Draft shown; if sent, the posted message is read back", "Not started"),
        ("s0c", "Read the live threads: craft-libs #94, #115, #79, #126 and the AI store folder decision; record what they change", "", "Findings added to NEXT.md", "Not started"),
        ("s0d", "Create worktrees off upstream main: launcher (not the feat/ai-connections tree) and PhotoCraft", "", "git worktree list shows both; the AI branch untouched", "Launcher done: worktree craft-launcher-ext on feat/extensions from origin/main 307efcf. PhotoCraft worktree not yet created"),
        ("s0f", "Public repo for the crates (created as Jeremy8776/craft-extensions, moved to storytold/craft-extensions on 04/11/2026): workspace, skeleton crates, threat model, CI", "", "Repo public; remote commit equals local; first CI run green", "Done 16/10/2026 (5500b37, CI green)"),
        ("s0e", "Threat model for all three kinds: assets, attackers, zip bombs, path escape, lookalike ids, impersonating first-party ids, update hijack, rollback and freeze, malicious wasm, hostile importer input", "", "Written in docs; every threat names the gate (sg*) that answers it or says it is accepted", "Draft written in the public repo (docs/THREAT-MODEL.md, 20 threats, 13 gates); unreviewed"),
    ]),
    ("p1", "Phase 1. Contract and the theme slice (first-party)", [
        ("c1", "craft-ext-core: manifest types with publisher, tier, signature list, provenance and permissions fields; id rules; hostile-input validation (tests first)", "s0d, s0e", "Fixtures for valid, oversized, bad id, control characters, unlisted files and non-empty permissions behave as expected", "Built 16/10/2026 (craft-extensions 1202969): ids, paths, text, versions, manifest; tests first, mutants caught"),
        ("c2", "craft-ext-core: compat ranges and the catalogue index schema (version, expiry, entries, revocations)", "c1", "Round-trip and bad-index tests; unknown fields tolerated", "Built 16/10/2026 (1202969): compat in the manifest, index schema with expiry, rollback and revocation checks"),
        ("c3", "craft-ext-core: shared palette type and the contrast validator", "c1", "4.5:1 pass and fail cases; unknown [app.*] sections ignored", "Built 16/10/2026 (1202969): palette, WCAG contrast, mode check, per-app override bounds"),
        ("c4", "craft-ext-store: bounded unpack (path escape, entry count, size, ratio caps)", "c1", "Hostile zip fixtures all refused with a plain reason", "Built 16/10/2026 (33adcb4): bounded unpack, 34 store mutants checked"),
        ("c5", "craft-ext-store: state file, atomic install and uninstall, enable per app, generation counter", "c4", "Interrupted write leaves the old state; two writers lose nothing", "Built 16/10/2026 (33adcb4): lock, atomic state write, generation, enable and theme choice; 8-way concurrent install test"),
        ("c6", "craft-ext-store: read side for apps (installed for this app, active theme, generation)", "c5", "Reader works with no launcher present", "Built 16/10/2026 (33adcb4): Reader needs only the folder; bad data counts as absent"),
        ("c7", "Mutation check on c1 to c6 and the wasm32 check on craft-ext-core", "c3, c6", "No surviving mutant without a stated reason; wasm check passes", "Done for the mutation check (core 27 of 27, store 31 caught and 3 equivalent with reasons) and the core wasm32 check; pushed as 33adcb4 and CI green on Linux and Windows"),
        ("l1", "Launcher: features/extensions/ module, fixture index, Installed list", "c6", "Screen opens with sample data; views.rs untouched", "Built 16/10/2026 (1f34346 in craft-launcher-ext, pushed to the fork): features/extensions/ module, Installed list, in-memory sample store and a desktop store over craft-ext-store; views.rs untouched. No catalogue index yet (that is n2), so the sample data is a sample store, not a fixture index"),
        ("l2", "Launcher: Themes tab, cards drawn from the palette, install from file", "l1, c3", "Install a theme file and see it listed; hostile file refused with a plain sentence", "Built 16/10/2026 (1f34346): Themes tab with cards drawn from the palette, Use in menu and All apps, Add from file, remove with a yes. Tested headless against the sample store and against a real folder; the screen has not been looked at in a window"),
        ("l3", "Launcher: strings in all four catalogs and the plain-language scan", "l2", "i18n tests and the jargon scan pass", "Built 16/10/2026 (1f34346): 34 strings in all ten non-English catalogs, coverage test extended, plain-word scan added. Translations are mine, clean-room, not read by a native speaker"),
        ("a-photo-1", "PhotoCraft: palette to Tokens adapter and live reload on generation change", "c6, c3", "Theme installed in the launcher restyles a running PhotoCraft; removal falls back", "Built 16/10/2026 (photocraft-wt-ext, branch feat/extensions-theme-adapter, ecb056cc, pushed to Jeremy8776/photocraft): ext_theme.rs maps the palette to Tokens, keeps the layout, re-checks contrast after overrides, falls back; a watcher polls the store once a second. Tested headless (14 tests, 19 mutants caught, 1 equivalent); never seen in a window"),
        ("t1", "Put the result into Jeremy's test build and say which window is which", "l2, a-photo-1", "Dev build runs both; running binaries identified", "Built 16/10/2026: launcher (extensions + AI work + open-from-app) at craft-launcher-wt-combine/target/dev/artcraft-launcher-ext-and-ai.exe; PhotoCraft in PhotoCraft-dev/photocraft.exe (upstream/main + feat/ai-port + feat/extensions-theme-adapter). Four older additions were skipped for conflicts. Launcher TEST BUILD was launched with --demo-data on 16/10/2026 and Plug-ins selected for showcase; PhotoCraft verification is still open"),
        ("g1", "Record the two consumers (launcher, PhotoCraft) as evidence for the craft-libs sharing rule", "a-photo-1", "Written in NEXT.md; nothing posted", "Not started"),
    ]),
    ("p2", "Phase 2. Internal catalogue (first-party only)", [
        ("n1", "Index repo with first-party entries only; contribution doc; CI runs the c1 to c3 validators and rejects any non-first-party entry while the gate is closed", "c2, d4", "A bad or third-party entry fails CI; a good first-party one passes", "Blocked on d4"),
        ("n2", "Launcher: fetch the index with size caps, cache and throttle (as feed_throttle does)", "n1", "Scripted localhost server test; no live call", "Not started"),
        ("n3", "Launcher: hash-pinned download then install through the store", "n2, c5", "Wrong hash refused before unpack; truncated download resumes or fails cleanly", "Not started"),
        ("n4", "Trust labels in words (Made by ArtCraft, Added from a file), updates list, update check", "n3", "Each label shows its own sentence; update flow tested", "Not started"),
        ("n5", "Download rule: add the storytold hub repo's releases; everything outside the storytold organisation stays refused", "n1", "Test asserts a non-storytold URL is refused; maintainer agreed to the wording", "Not started"),
        ("n6", "Web build: read-only listing", "n2", "cargo xtask wasm passes; no install button on web", "Not started"),
    ]),
    ("p3", "Phase 3. Third-party structure behind security gates (built, switched off)", [
        ("sg1", "Reserved namespaces: ids under storytold prefixes only from the first-party tier; lookalike and homoglyph ids refused", "c1", "Third-party entry claiming a first-party id is refused; tests include lookalikes", "Built in craft-ext-core (id rules, tier check, index check) with mutants caught; more lookalike cases wait for the threat model review"),
        ("sg2", "Signature seam: publisher key pinning, detached signature check, key rotation and revocation records; test keys only, no real key material in the repo", "c2, d8", "Wrong key, unsigned, rotated and revoked cases all refused or accepted correctly", "Not started"),
        ("sg3", "Signed, expiring index metadata: monotonic version, expiry, freeze and rollback refused (clock passed as an argument)", "sg2", "Stale, replayed and downgraded index refused; tests with a fake clock", "Partly built: expiry and no-older-index checks in the index parser. Signatures not built (needs d8)"),
        ("sg4", "Revocation list checked at launcher start and before enabling; a revoked package is disabled in every app through the store", "sg3, c5", "Revoking an installed package turns it off for all apps on next scan", "Partly built: Index::is_revoked. Not wired into launcher start or enabling"),
        ("sg5", "Tier gate: third-party code path behind a cargo feature that is off by default; runtime refuses; a test fails if default or release features enable it", "sg1, sg2", "Release build config proves the feature is off; a third-party entry is refused with the feature off", "Built: Policy::admit, cargo feature passed through both crates, guard test that fails if it is a default; release-profile proof still to add"),
        ("sg6", "Capability rules for third party: permissions must be empty in version 1; plug-in re-validated at install and on every load; any host import rejected", "c1, pl1", "Plug-in with an import or a permission is refused at both points", "Partly built: permissions must be empty in the manifest. Re-validation of the module on every load is the app host's job, not done"),
        ("sg7", "No silent change: a new publisher key, a wider permission or a downgrade needs a fresh yes; third-party updates never apply automatically", "sg2, n4", "Each change blocks until the person agrees; tested", "Not started"),
        ("sg8", "Fuzz targets and a panic hunt over the manifest, theme, index and archive parsers and the plug-in install path; time-boxed, one run at a time", "c4, sg6", "Corpus and findings recorded; every crash fixed with a regression test that failed first", "Not started"),
        ("sg9", "Provenance: source repository and commit recorded per entry; decide whether the hub builds plug-ins from source itself (d9)", "d9, n1", "Provenance field checked by CI; build-from-source decision recorded", "Blocked on d9"),
        ("sg10", "Review process: index pull request checklist, two-maintainer rule, SECURITY.md with disclosure and takedown path", "n1", "Documents merged; a dry-run review of a sample third-party entry", "Not started"),
        ("sg11", "Launcher interface for third-party entries (tier badge, warnings, plain wording), compiled only with the feature on", "sg5, n4", "Only visible in a feature build; plain-language scan covers it", "Not started"),
        ("sg12", "Independent security review of unpack, sandbox host and signature code before the gate opens (d10)", SG_1_TO_8 + ", d10", "Report received; every finding fixed or accepted in writing", "Blocked on d10"),
        ("sg13", "Open the gate: enable third-party in release builds", SG_1_TO_12, "Closed until every row above is done AND Jeremy and the maintainer each say yes to this specific change", "Closed"),
    ]),
    ("p4", "Phase 4. Plug-ins", [
        ("pl1", "PhotoCraft host installs filters from the store; the additional-folder preference stays as a developer option", "c6", "Store-installed filter appears under Filter > Plug-ins; docs/plugins.md updated", "Not started"),
        ("pl2", "Package manifest declares the target interface version; mismatch shown in plain words", "pl1, c2", "Wrong version refused with a sentence a person can act on", "Not started"),
        ("pl3", "Ship the invert-rs example as the first first-party catalogue entry", "pl1, n3", "Installs from the catalogue end to end on scripted localhost", "Not started"),
        ("pl4", "Decide whether to extract the wasm host once LightCraft or EffectCraft wants it", "pl1", "A second real consumer exists; otherwise it stays in PhotoCraft", "Not started"),
    ]),
    ("p4b", "Phase 4b. Bring in launcher UI only after plug-in infrastructure works", [
        ("ui-gate", "Freeze the first-party core, store, catalogue, launcher source and PhotoCraft host contracts", "pl1, pl2, pl3, n3, r2", "A real first-party plug-in installs, enables, runs in PhotoCraft, persists on reload and removes. Fresh checkout resolves pinned dependencies without sibling-only paths. Third-party remains off", "Not started; TEST BUILD UI is only a demo"),
        ("ui-port", "Extract approved launcher UI from combined TEST BUILD into clean feat/extensions after ui-gate", "ui-gate, l3", "Extensions navigation, icon controls, dropdowns, grouped per-app entries, deduped IDs and tests are in a clean branch; AI changes are separate; production source is real and --demo-data is labelled as examples", "Not started; UI exists only in throwaway combined worktree"),
        ("ui-link", "Make app and kind deep links select the matching visible Extensions section", "ui-port, r2", "Cold start and running-instance --open extensions?kind=plugin land on Plug-ins, not Themes with a contradictory plug-in banner; app filter and Show all agree with the selected section; regression tests cover both starts", "Not started; 16/10/2026 showcase needed manual Plug-ins selection"),
        ("ui-verify", "Exercise combined source/store/app flow and review exact TEST BUILD candidate", "ui-link, r3", "Targeted tests, lint, launcher build, clean-checkout CI and wasm pass; install/enable/reopen/remove works; empty and --demo-data modes cannot impersonate installed plug-ins; Jeremy approves this build", "Not started"),
    ]),
    ("p5", "Phase 5. Reach every app", [
        ("r1", "Launcher: Request::Open on the single-instance handshake and an --open argument", "l1", "Second start forwards the target; old launchers still answer show", "Built 16/10/2026 (70b9612): crates/native deeplink.rs reads extensions?app=&kind= strictly (bad values dropped); single_instance sends open after the hello on a second connection, a newer start carries the place itself, old launchers ignore it and still show; --open and --open= in main. 10 new native tests, 19 mutants run, 18 caught, 1 equivalent dropped"),
        ("r2", "Launcher: route the target to the filtered Extensions view", "r1", "App and kind filters applied on open", "Built 16/10/2026 (70b9612): LauncherUi::open_extensions filters app/kind and shows a Show all banner; 6 plain-word strings in 10 catalogs, headless path tests. Observed in a window 16/10/2026: a cold --open extensions?kind=plugin showed the plug-in banner but left Themes selected; ui-link tracks the correction"),
        ("r3", "PhotoCraft: Themes & Extensions panel, extension.* commands, Browse button", "r2, a-photo-1", "Button opens the launcher at PhotoCraft's filter; commands tested including bad params", "Not started"),
        ("r4", "Per-app adapters, one per app in the matrix (13 apps), each in its own repo by its maintainer", "r3", "Each app ticks the adapter checklist; reported as next decisions, none started here", "Not started"),
    ]),
    ("delivery", "Delivery. Stage source and package the approved first-party candidate", [
        ("ship-1", "Freeze one reproducible candidate commit after ui-verify", "ui-verify", "Working tree and version recorded; all dependencies resolve in CI without local sibling repositories; first-party behaviour and third-party-off feature guard pass from a fresh checkout", "Not started"),
        ("ship-2", "Build and smoke-test private Windows candidate", "ship-1", "Run packaging/windows/package.ps1 for a versioned MSI and portable ZIP from the exact commit; test install, launch, plug-in lifecycle, restart and uninstall from a clean profile; record SHA-256 hashes, signature state and release notes; do not send a debug .exe", "Not started"),
        ("ship-3", "Hand off source PRs in dependency order after Jeremy approves the candidate", "ship-2", "Separate reviewable PRs for shared infrastructure, app adapter and launcher integration/UI with test evidence and artifact references; any push or PR requires an explicit ask; no combined throwaway branch", "Not started"),
        ("ship-4", "Publish only on a separate explicit release approval", "ship-3", "After maintainer acceptance, confirm the release workflow and rollback plan; obtain specific yes before pushing the release/production branch; verify the actual MSI, ZIP and other release assets after publishing", "Not started"),
    ]),
    ("p6", "Phase 6. Resources and later kinds", [
        ("x1", "Pick the first resource type whose importer is already shared (swatches #140 or LUT .cube)", "pl1", "Chosen with Jeremy; no invented formats", "Not started"),
        ("x2", "Resource kind in the envelope and one app loading it through its existing importer", "x1", "Install a resource, app lists it; hostile file refused by the importer", "Not started"),
        ("x3", "Declarative AI tool and provider specs as a data-only kind", "x2", "Only after the AI infrastructure is settled; reuses the existing spec checker", "Blocked on AI infrastructure"),
    ]),
    ("future", "Future roadmap (Jeremy, 04/11/2026)", [
        ("theme-1", "Keep theme packages in a form a future ThemeCraft can open, edit and export", "c3", "Theme format documented; a saved package round-trips with no launcher change needed", "Future"),
        ("theme-2", "Connect ThemeCraft to the Themes tab once that app exists: open a theme in it and publish back", "theme-1", "A theme made in ThemeCraft appears in the Themes tab and applies in another app", "Future: ThemeCraft does not exist yet"),
        ("wf-1", "Workflows tab: install and list custom ComfyUI API-format JSON files as a data-only kind", "x2", "A workflow installs and lists; bad JSON refused; nothing in the file can run on install; missing custom nodes and models named in plain words", "Future"),
        ("wf-2", "Let chosen app UIs adopt a workflow: each app declares which inputs and outputs it can drive", "wf-1", "One app UI runs an installed workflow through its declared inputs; an unsupported workflow is refused with a sentence", "Future"),
        ("plc-1", "Plug-in capability model: a wide range of abilities across all apps, declared in the manifest, each granted only when the sandbox can enforce it", "d12, sg6, pl2", "Every ability is listed, shown before install and refused when undeclared; replaces the empty-permissions rule of version 1", "Future: large design work"),
        ("plc-2", "Per-app extension points: what a plug-in can reach in each of the 14 apps (filters, tools, panels, commands, audio)", "plc-1", "Matrix agreed per app; each app's maintainer owns its host", "Future: large design work"),
        ("rank-1", "Plug-in download counts and other ranking signals from an agreed source", "d11, n3", "Counts shown name their source; no hidden tracking in the apps; inflated counts have a stated answer", "Future"),
        ("rank-2", "Rank and sort plug-ins in the browser by downloads and other signals", "rank-1, n4", "Sort orders tested; first-party and third-party labels stay visible at every rank", "Future"),
    ]),
    ("pg", "Standing gates, every phase", [
        ("g-fmt", "fmt, per-crate clippy -D warnings, targeted tests one at a time", "", "Output read, not assumed", "Standing"),
        ("g-size", "Files under 500 lines, 700 hard", "", "Checked before each commit", "Standing"),
        ("g-never", "Never-crash scan: no unwrap, expect or panic in non-test code", "", "Crate roots carry the deny header", "Standing"),
        ("g-say", "State what was not run (live calls, TLS, other OSes) in NEXT.md and the reply", "", "Present in every status update", "Standing"),
        ("g-closed", "The third-party gate stays closed: no release build enables it and no third-party entry is accepted until sg13", "", "The sg5 test is green in every phase", "Standing"),
    ]),
]


def all_tasks():
    for phase_id, title, rows in PHASES:
        for row in rows:
            yield (phase_id, title) + row

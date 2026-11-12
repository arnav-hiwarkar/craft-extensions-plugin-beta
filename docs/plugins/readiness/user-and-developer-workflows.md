# User, author, maintainer and app-developer workflows

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Status: coordinator design, 2026-11-08. These are target workflows, not descriptions of working software. Each step names the component that owns it and the blocker in the [roadmap](system-gaps-and-roadmap.md) that must be fixed before the step can work. The mechanics are in [catalogue and SDK](catalogue-and-sdk.md).

## Who owns what

| Owner | Owns | Never does |
|---|---|---|
| App engine and adapter | Operations, validation, transactions, undo, jobs, saved references, scope checks, CLI bootstrap | Install packages, trust manifest claims |
| craft-ext-store | Installed releases, origin, enablement, grants, pins, revocation cache, eligibility | Execute plugins, understand documents |
| Distribution client | Signed metadata, download, digest check, cache | Grant permissions |
| Manager UI (in app and launcher) | Search, details, install, per-app setup, updates, recovery | Keep its own copy of eligibility or permission logic |
| SDK | Templates, generated bindings, check, pack, test, release files | Publish, or claim its output is trusted |
| Catalogue maintainers | Enrolment, exact-release approval, signing, revocations | Host binaries, grant user permissions |

**Built-in features stay authoritative.** A plugin adds namespaced contributions. It cannot reuse a built-in command, effect, filter, format, function or theme ID, cannot become a default handler, and cannot claim a shortcut a built-in uses. The user can explicitly choose a plugin exporter, theme or binding. Installing a package never makes that choice. Most apps do not enforce this today. See [collision status](system-gaps-and-roadmap.md#core-feature-protection-today).

## End user

### Find and install (public package, no login)

1. Open **Extensions** in the app (or the launcher, which uses the same client). *Manager UI.*
2. Search. Results are filtered to this app and compatible versions. Official packages lead within each relevance tier, then packages are ordered by repository stars, with the star date shown. *Signed catalogue (B5), ranking rules.*
3. Open details: what it adds to this app, other supported apps, required and optional permissions in plain words, network use, licence, review date, download size, publisher, and a "repository stars" figure.
4. **Install.** The client downloads the approved asset from the publisher's GitHub Release, checks the signed digest, and stores it. Nothing runs, and nothing is enabled. *Distribution client, store (B1, B2).*
5. **Set up for this app.** The user reviews the permissions and enables the contribution. That writes a grant for this app only. Other apps that the package supports remain off. *Store grants (B3).*
6. The feature appears in its own place, such as **Filters ▸ Extensions ▸ Soft Glow**, or a panel. It is labelled as an extension and never replaces a built-in item. *App adapter (B3, B9).*

### Use

- A long operation shows progress and can be cancelled. Cancelling or a failure leaves the document unchanged (*app transaction*).
- Undo treats the plugin's edit as one step.
- Saving a live feature stores the exact package UUID, release digest, contribution ID, parameters and state version (B7).
- A fresh CLI, for example `photocraft-cli apply-filter ext:<uuid>/soft-glow in.png out.png`, uses the same installed, enabled, granted release. It never downloads, never grants and never opens a dialog. It waits for the output file and exits non-zero if the plugin is missing (B6).

### Update

- The manager shows "Update available (reviewed 2026-…)". Applying it is explicit unless the user turned on automatic updates for releases with unchanged permissions.
- If the new release asks for more permissions, it installs but waits for the user's approval for each app. The old release stays the default until then.
- Open projects and running exports keep the release they started with.

### Problems and recovery

| What the user sees | What happened | What they can do |
|---|---|---|
| "Soft Glow 1.2 is missing" when opening a project | Release not installed here | Install that exact release, choose another installed version, or keep the project with the feature preserved but inactive. Export refuses to report success while a required feature is missing |
| "Withdrawn by the catalogue" | Authenticated revocation | Disable now; roll back to a non-revoked release if one is retained |
| "No longer downloadable" | Upstream asset removed or changed | The installed copy keeps working; reinstall elsewhere needs a new approved release |
| "Extensions list is out of date" | Offline or expired metadata | Installed features keep working; new installs wait for a connection |
| App starts in safe mode | Repeated plugin crash at load | Extensions are disabled for this session; the user can re-enable them one by one |

### Local file

**Install from file…** with a `.zip`. The package goes through the same validation, is labelled "Local, not reviewed", and gets no official badge. It needs the same explicit enablement and grants. It does not take over a catalogue package's identity, grants or updates even if it carries the same UUID.

### Restricted package (optional)

If a publisher distributes through a private repository, the manager asks for a GitHub credential once. The OS credential store keeps it. It is sent only to GitHub for that asset. Public packages never ask.

## Plugin author

1. `craft-ext create --app photocraft --contract image-filter` creates a project with a package UUID, licence field, one contribution and a fixture.
2. Write the algorithm in Rust against the generated typed wrappers. `craft-ext inspect photocraft` lists the operations, capabilities and limits this app version supports.
3. `craft-ext check` validates the manifest, file inventory, module imports against declared capabilities, licences and versions, using the same validator as the store and the catalogue bot.
4. `craft-ext test` runs literal input and output fixtures in the app's test host (future work for each app).
5. `craft-ext dev` installs into a labelled development store and reloads it in a running app. Development grants are isolated from normal ones.
6. `craft-ext add --app effectcraft --contract effect` adds another app contribution that shares the same algorithm crate.
7. `craft-ext pack` builds the deterministic `.zip`. `craft-ext prepare-release` writes the release record for the catalogue PR.
8. Create a GitHub Release in your own repository, attach the `.zip`, and publish it. Immutable releases are recommended.
9. Fork the catalogue and open a PR with the release record. First-time publishers also add a publisher record, which signs them in through GitHub as part of the PR. No other account is needed.
10. Answer review questions. After approval and merge, the release appears in search for compatible apps.
11. For each later version, repeat steps 7 to 10. A new GitHub Release alone does nothing until its record is approved.

The author never uploads binaries to Storytold, never runs a build service, and cannot set `official`, review status, grants or tier.

## Catalogue maintainer

1. **Enrol a publisher.** Check the GitHub account or organisation and record its numeric ID. Reserve names.
2. **Review a release PR.** Read the bot report: digest match, GitHub digest, immutable flag, validator result, and the delta from the last approved release (new capabilities, imports, contributions, dependencies, licences, state schema). Read the source diff between the reviewed tags. Record what was inspected. Approve the exact digest.
3. **Merge.** The protected job regenerates and signs metadata and refreshes the star snapshot.
4. **Incident.** Add a revocation record (`advisory`, `block-new` or `disable`) for a digest, version range or publisher. Merge and publish with priority. Coordinate the reviewed fix release.
5. **Transfer.** Record a reviewed publisher or repository transition. Users must approve permissions again.
6. **Key operations.** Rotate the online signing key. Keep root and recovery keys offline. Run a recovery drill. Key custody is an open decision, and no keys exist.

Maintainers never build packages for publishers and never host binaries.

## App developer (adding plugin support to an app)

1. **Audit.** Start from the app's row in the [app audits](system-gaps-and-roadmap.md#all-sixteen-apps).
2. **Describe operations.** Add a descriptor to each operation the app will offer plugins and the CLI: ID, typed parameters and result, effect class (query, edit, artifact, external), allowed callers, required capabilities, transaction or job behaviour, headless support. Use the app's existing command layer where one exists.
3. **Dispatch through policy.** GUI, CLI and plugin calls go through one function that applies the caller's identity and authority. A plugin call carries the plugin's principal through every nested call. There is no "run any command" bridge.
4. **Register contributions additively.** Contribution IDs are namespaced `ext:<package-uuid>/<contribution-id>`. Registration refuses a builtin ID or foreign package identity. A verified same-source upgrade can replace its own contribution explicitly while preserving compatible settings. Exporters may share file extensions but remain separately selectable by unique ID; installation does not change defaults. Plugin shortcuts start unbound unless the chord is free. `craft-keymap` already reports clashes (`crates/keymap/src/keymap.rs:436-441`), but its assignment defaults to `Steal` (`keymap.rs:333-341`), so extension bindings must use `Policy::Refuse`.
5. **Use the shared loader.** Read eligibility from `craft-ext-store` in both the desktop app and the fresh CLI.
6. **Save exact references.** Write package UUID, release digest, contribution ID and state version. Preserve unknown features on load.
7. **Make headless output explicit.** CLI and plugin artifact output uses an explicit destination. UI requests are an error in headless mode, never silently dropped.
8. **Export the inventory.** Release tooling writes the contract JSON. The SDK repository adds it by reviewed PR.
9. **Write conformance tests.** At minimum: built-in ID collision refused, denied capability refused through nested calls, cancel rolls back, CLI output matches GUI, missing plugin fails export.

## How the app keeps built-in behaviour authoritative

| Rule | Mechanism |
|---|---|
| No shadowing | Registration checks the built-in ID table first and returns an error. Plugin IDs live in an `ext:` namespace |
| No silent defaults | Default exporter, file-type handler, active theme and shortcut stay as they are until the user chooses otherwise in settings |
| No engine patching | Plugins receive only typed host functions for their contract. They have no access to engine internals or to an unrestricted command bridge |
| No authority borrowing | The plugin principal goes with every nested call. A CLI user's file-system authority is never available to a plugin through a shared operation |
| No hidden install effects | Install writes files and a disabled record. Enablement, grants and theme activation are separate user actions |

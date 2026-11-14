# Test the Mac beta beside normal installations

Use this procedure to test a verified GitHub artifact without replacing your
normal EffectCraft or launcher. It uses a separate catalogue, plugin store and app
settings. No Rust toolchain or local application build is required.

For the complete app checklist, use [TESTING](../../TESTING.md). The
[workflow atlas](maps.md) explains what each check exercises.

## 1. Choose and download a successful run

Use the private fork explicitly. Do not substitute a storytold repository.

```sh
BETA_REPO=arnav-hiwarkar/craft-extensions-plugin-beta
BETA_RUN=$(gh run list --repo "$BETA_REPO" --workflow plugin-beta --branch plugin-beta --status success -L 1 --json databaseId -q '.[0].databaseId')
: "${BETA_RUN:?No successful beta run found}"
BETA_ROOT="$HOME/craft-beta"
BETA_RELEASE="$BETA_ROOT/releases/run-$BETA_RUN"
mkdir -p "$BETA_RELEASE"
for BETA_ARTIFACT in effectcraft-macos-arm64 photocraft-macos-arm64 figcraft-macos-arm64 craft-launcher-macos-arm64 craft-ext-macos-arm64 test-catalogue wasm-packages; do
	gh run download "$BETA_RUN" --repo "$BETA_REPO" --name "$BETA_ARTIFACT" --dir "$BETA_RELEASE/$BETA_ARTIFACT"
done
mkdir -p "$BETA_RELEASE/bin"
for BETA_ARCHIVE in "$BETA_RELEASE"/*/*.tgz; do
	tar -xzf "$BETA_ARCHIVE" -C "$BETA_RELEASE/bin"
done
```

If the run's artifacts expired, choose a newer successful run. Do not initiate new
CI or releases merely to follow this guide without the required authorization.
Keep each run in its own folder so packages, public test key and binaries stay
associated. Downloading into an already populated destination may be refused;
reuse the existing downloaded files instead of mixing runs.

## 2. Configure an isolated testing shell

Use a new Terminal window for the beta. These variables apply to that shell and
children started from it, not to your normal apps.

```sh
BETA_STATE="$BETA_ROOT/state/run-$BETA_RUN"
mkdir -p "$BETA_STATE"/{plugins,effectcraft,photocraft,figcraft}
export CRAFT_EXT_DIR="$BETA_STATE/plugins"
export EFFECTCRAFT_CONFIG_DIR="$BETA_STATE/effectcraft"
export PHOTOCRAFT_CONFIG_DIR="$BETA_STATE/photocraft"
export FIGCRAFT_CONFIG_DIR="$BETA_STATE/figcraft"
export CRAFT_CATALOGUE_URL="file://$BETA_RELEASE/test-catalogue/"
export CRAFT_CATALOGUE_ALLOW_FILE=1
export CRAFT_CATALOGUE_TEST_KEY=$(cat "$BETA_RELEASE/test-catalogue/TEST-KEY.pub")
export PATH="$BETA_RELEASE/bin:$PATH"
touch "$BETA_RELEASE/bin/portable.txt"
```

The marker keeps this launcher's own settings/data under
`$BETA_RELEASE/bin/ArtCraftLauncherData`. The other three apps use their explicit
config directories above. The shared plugin store still keeps enablement and
grants separately for each app.

The test-key variable contains a public key, not a secret signing seed. Its trust
override applies only to debug builds with file catalogue access explicitly allowed
and a `file://` base. Signatures, metadata digests, archive digests and freshness
checks still apply. Never copy a production signing key to test an artifact.

Signed catalogues expire. Check the expiry in `test-catalogue/index.json` and use a
newer complete artifact set when needed. A replay/older-serial error usually means
this store already saw newer metadata; choose a new beta store rather than weaken
verification or alter the signed index.

## 3. Verify catalogue access and explicit setup

```sh
"$BETA_RELEASE/bin/effectcraft-cli" plugin refresh
"$BETA_RELEASE/bin/effectcraft-cli" plugin search ""
"$BETA_RELEASE/bin/effectcraft-cli" plugin install io.github.arnav-hiwarkar.color-notes
"$BETA_RELEASE/bin/effectcraft-cli" plugin permissions io.github.arnav-hiwarkar.color-notes
"$BETA_RELEASE/bin/effectcraft-cli" plugin setup io.github.arnav-hiwarkar.color-notes --all --grant document.read:selection
```

Refresh should verify the signed catalogue. Installing should add the package and
enable nothing. Setup grants only the named permission and enables the compatible
selected contributions. Color notes' optional document edit grant remains off, so
its edit action should refuse until you choose to grant it.

Use `photocraft-cli plugin` or `figcraft-cli plugin` for the corresponding app.
Native packages need a separate full-access decision in addition to declared app
permissions. Do not add `--full-access` while preparing ordinary starter plugins.

## 4. Launch the actual beta binaries

Quit the normal launcher before starting the beta launcher. They share a local
single-instance mechanism. Do not use the normal launcher's app buttons to locate
your beta copies.

```sh
"$BETA_RELEASE/bin/effectcraft" &
"$BETA_RELEASE/bin/photocraft" &
"$BETA_RELEASE/bin/figcraft" &
"$BETA_RELEASE/bin/artcraft-launcher" &
```

These explicit paths launch the downloaded build. A normal app started from Finder
will not inherit the testing shell's environment.

A prepared Finder installation must use separately named app bundles, unique bundle
IDs and wrappers that load the beta environment. It must keep the normal bundles,
file associations and settings intact. The prepared local setup uses **Craft Plugin
Beta** in the user's Applications folder and supplies a Desktop shortcut, sample
projects and `START-HERE.md` under `~/craft-beta`. Those wrappers load their pinned
artifact configuration automatically. Use either those prepared wrappers or this
explicit shell procedure. Confirm the selected store and settings before switching
methods; prepared wrappers remain pinned even if a shell points at a different run.

## 5. Exercise the user workflows

Open disposable projects and use the per-app checklist in TESTING:

1. Browse, install, inspect the setup sheet, enable and grant required access.
2. Run a command, panel, tool or effect and verify the intended edit and one Undo.
3. Revoke a required permission and verify the contribution stops working.
4. Save and reopen. Verify data and rendering behavior with the plugin present.
5. Disable/remove a required contribution and check missing-plugin behavior where
   the app records such requirements. PhotoCraft's baked filter pixels need no plugin.
6. Test compatible updates, a version pin, unpin and rollback with Film looks.
7. Test safe mode and Resume after a controlled failure.
8. Test native consent separately, including withdrawal while the helper is idle.

In the launcher, use the **Plugins** header button and choose an app. The plugin
manager uses the same platform store. Normal app-binary installation/update controls
are a separate workflow and can follow production release channels.

## 6. Record reproducible evidence

Record the artifact run ID and commit, app, plugin ID/version, exact steps, expected
result and observed result. Keep command output and the relevant beta log. For UI
issues include a screenshot when useful, without unrelated personal content.
Distinguish CLI, GUI, GPU and platform checks. A startup process, compile check or
old green run is not evidence that you exercised a particular GUI path.

Keep test data disposable. Earlier files written by PhotoCraft or native external
effects are outside document Undo. Preserve the plugin store while testing pins and
revocation; recreate it only as an explicit new test session. Closing a testing
shell does not remove its store. Unset the variables or open a clean shell before
using normal CLI commands.

# Cross Review

One command module and three small panel entry modules share `src/review.rs`.
The commands `effectcraft-apply`, `photocraft-apply`, and `figcraft-apply` still
rename the first layer to `Reviewed`. EffectCraft uses the active composition,
PhotoCraft uses the document stack, and FigCraft uses the first page's first
direct child. Selection does not change the target.

The corresponding `<app>-panel` contributions show the target, refresh it, and
rename it with a button. They use identical host-drawn column, group, label,
row, and button nodes with stable `review-first` and `refresh` IDs. Errors such
as missing layers or denied permissions appear in the panel. A failed panel
render still propagates an error to the host. Commands continue to propagate
operation errors, preserving their transactional smoke behavior.

Routing uses exact command IDs and explicit panel entry modules. `Event::Init`
has no contribution identity, so each panel entry fixes its app at compilation;
there is no host or renderer probing. The UI vocabulary works with future host
framework adapters without plugin-specific renderer code.

Commands retain `document.edit:document`, which covers document reads. Panels
explicitly declare both `document.read:document` and `document.edit:document`.
Renames use canonical `layer.rename` or FigCraft `node.update` operations. No
file, network, window, or event subscription permission is requested.

For a package produced in a later authorized build phase:

```sh
effectcraft-cli plugin install-file cross-review.craftplugin
effectcraft-cli plugin setup io.github.arnav-hiwarkar.cross-review --enable effectcraft-apply,effectcraft-panel --grant document.edit:document
photocraft-cli plugin install-file cross-review.craftplugin
photocraft-cli plugin setup io.github.arnav-hiwarkar.cross-review --enable photocraft-apply,photocraft-panel --grant document.edit:document
figcraft-cli plugin install-file cross-review.craftplugin
figcraft-cli plugin setup io.github.arnav-hiwarkar.cross-review --enable figcraft-apply,figcraft-panel --grant document.edit:document
```

These are consumer CLI parser forms, not commands executed during source work.
Installation alone never enables contributions or grants permissions.
EffectCraft/FigCraft command IDs are
`plugin.io.github.arnav-hiwarkar.cross-review.<app>-apply`; PhotoCraft uses
`plugin.session.io.github.arnav-hiwarkar.cross-review/photocraft-apply`.

Source status: helper regression tests are present and unrun. No Cargo, guest
build, app run, smoke script, or packaging was executed for this change. The
Cargo example targets must later be compiled for Wasm and their three named
`.wasm` outputs included alongside `cross_review.wasm` before packaging. The
author CLI now builds and copies manifest-referenced Cargo example targets alongside
the library. Unreferenced Cargo examples are not built.
No binaries or packages are included. Later validation must exercise initial
panel rendering, refresh, rename, Undo, empty documents, grant revocation, and
all three existing command smoke paths.

The action row is a named `craft.review-actions` component with an ordinary row of
buttons as its fallback. Hosts without that component render the same useful controls.
The version-1 UI contract and widget IDs stay the same when the host renderer changes.

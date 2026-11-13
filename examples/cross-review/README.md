# Cross-app review (MIT)

One sandboxed wasm module, three command contributions, each declaring its app's
session contract. Marks the first layer of the active composition (EffectCraft),
layer stack (PhotoCraft), or first page (FigCraft) **Reviewed**. This deliberately
uses the same document-level operation everywhere, independent of selection.
Empty documents return a useful error and make no edit.

Only `document.edit:document` is requested: it includes the reads required to find
the layer. No files, network, clipboard, window or app-event permissions. Each
command is one transaction and one undo step. App-specific code is limited to
listing layers and mapping the rename operation.

```sh
craft-ext build --release
craft-ext check
craft-ext test  # session execution is verified by the real app smoke, not this command
craft-ext pack --out cross-review.craftplugin
```

Install with `<app>-cli plugin install-file cross-review.craftplugin`, then
`plugin setup io.github.arnav-hiwarkar.cross-review --enable <app>-apply --grant document.edit:document`.
EffectCraft/FigCraft command: `plugin.io.github.arnav-hiwarkar.cross-review.<app>-apply`.
PhotoCraft command: `plugin.session.io.github.arnav-hiwarkar.cross-review/photocraft-apply`.
Run `../../scripts/cross-app-smoke.sh` for disposable-store CLI verification.

# Layout grid

Generate a named auto-layout grid of cards. FigCraft only, session 1.1. Original code and copy, MIT OR Apache-2.0.

Run `craft-ext build --release`, `craft-ext check`, `craft-ext test`, then
`craft-ext pack --out figcraft-grid.craftplugin`. Install with FigCraft's `plugin install-file`,
then `plugin setup io.github.arnav-hiwarkar.figcraft-grid --all --grant document.edit:document --grant document.read:document`.
Open the panel from Plugins, adjust the form and Apply, or invoke the `apply` command.
Every Apply is one undo step. Save/reopen retains ordinary layers. Revoking edit removes
contributions on the next host pump. No network, file paths or native code.

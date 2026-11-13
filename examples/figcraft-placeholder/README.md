# Placeholder copy

Fill selected text layers with original placeholder copy. FigCraft only, session 1.1. Original code and copy, MIT OR Apache-2.0.

Run `craft-ext build --release`, `craft-ext check`, `craft-ext test`, then
`craft-ext pack --out figcraft-placeholder.craftplugin`. Install with FigCraft's `plugin install-file`,
then `plugin setup io.github.arnav-hiwarkar.figcraft-placeholder --all --grant document.edit:document --grant document.read:document`.
Open the panel from Plugins, adjust the form and Apply, or invoke the `apply` command.
Every Apply is one undo step. Save/reopen retains ordinary layers. Revoking edit removes
contributions on the next host pump. No network, file paths or native code.

The All text on this page toggle uses `node.find`; both scopes refuse more than 200
layers. The multiline editor caps copy at 4000 bytes and preserves existing styling.

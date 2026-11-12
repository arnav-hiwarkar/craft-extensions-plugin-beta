# Color notes

A session plugin for EffectCraft. It adds a **Color notes** panel that lists the selected layers, and
an **Add note marker** command and button.

- Reads the selection with `selection.get` (needs `document.read:selection`, required).
- Adds a marker with `marker.add` (needs `document.edit:document`, optional). If you have not turned it
  on, the panel explains that instead of failing.

The op names and their argument and result shapes are the EffectCraft session contract
(`effectcraft.session@1.0`). Check them against `docs/plugins/contracts/effectcraft.md` once it is
generated; the code assumes `selection.get` returns `{"layers": [{"name": …}]}`.

Build the module:

```sh
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/color_notes.wasm .
```

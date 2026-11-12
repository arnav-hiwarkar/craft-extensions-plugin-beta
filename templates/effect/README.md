# My effect

A `raster.filter@1.0` effect template. Edit `craft-plugin.toml` for the id, name and parameters,
then `src/lib.rs` for the pixels.

Build: `cargo build --target wasm32-unknown-unknown --release`, then copy
`target/wasm32-unknown-unknown/release/plugin.wasm` next to the manifest.
Check and pack with `craft-ext check` and `craft-ext pack`.

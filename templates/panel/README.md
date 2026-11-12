# My panel

An `effectcraft.session@1.0` template: one panel and one command. Edit `craft-plugin.toml` for ids,
labels and permissions, and `src/lib.rs` for behaviour. Every panel and command must be declared in
the manifest; the host refuses calls that use an undeclared panel id.

Build with `cargo build --target wasm32-unknown-unknown --release` and copy `plugin.wasm` next to the
manifest. Check and pack with `craft-ext check` and `craft-ext pack`.

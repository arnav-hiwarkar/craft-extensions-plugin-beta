# Quietgrain

A restricted native PhotoCraft command that removes noise with non-local means on every core and
puts the result in a new layer, in one undo step. It uses the `craft-native` SDK.

The manifest sets `mode = "restricted"`: the app runs the helper with no network, no other
programs and no files outside the plugin's own folders (Seatbelt on macOS, bubblewrap on Linux).
Where that confinement is not available, the helper does not start.

One archive serves Linux x86_64 and macOS arm64. CI builds each helper on its platform and packs
them together. To pack locally for this machine only:

```sh
cargo build --release
mkdir -p bin
cp target/release/quietgrain bin/quietgrain-linux-x86_64
cp target/release/quietgrain bin/quietgrain-macos-aarch64
craft-ext check
craft-ext pack --out quietgrain.craftplugin
```

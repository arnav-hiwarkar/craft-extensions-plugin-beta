# Native system stamp (MIT)

A protocol-1 JSON-lines helper for EffectCraft and FigCraft. Its panel inserts an
ordinary text layer containing `/usr/bin/uname -sm` output, in one undo step. The
helper calls a fixed local executable directly, without a shell. Sandboxed wasm
cannot spawn a process. stdout is reserved for the host protocol; errors use stderr.

The host must show **Full access to your computer** and obtain separate consent
before spawning this helper, and asks again for every new version. Its app operations
are still checked like wasm calls: every contribution that inserts the stamp declares
`document.edit:document`, which must also be granted. Revocation or disable stops the
helper; Undo removes the document stamp but cannot undo external native side effects.
Native is not an OS sandbox.

One archive serves Linux x86_64 and macOS arm64: the manifest maps each platform to its
own file (`bin/helper-linux-x86_64`, `bin/helper-macos-aarch64`), and the package must
contain both. CI builds each helper on its platform and packs them together. To pack
locally for testing on one machine, build the helper and copy it to both paths:

```sh
cargo build --release
mkdir -p bin
cp target/release/native_system_stamp bin/helper-linux-x86_64
cp target/release/native_system_stamp bin/helper-macos-aarch64   # only valid for this machine
craft-ext check
craft-ext pack --out native-system-stamp.craftplugin
```

Run `../../scripts/cross-app-smoke.sh` from this checkout for the Linux build,
package and real app CLI tests. Built binaries and archives are intentionally
ignored. Use `<app>-cli plugin setup <id> --enable <app>-apply --grant document.edit:document --full-access`;
contributions for the other app remain unavailable by contract.

# Native system stamp (MIT)

A protocol-1 JSON-lines helper for EffectCraft and FigCraft. Its panel inserts an
ordinary text layer containing `/usr/bin/uname -sm` output, in one undo step. The
helper calls a fixed local executable directly, without a shell. Sandboxed wasm
cannot spawn a process. stdout is reserved for the host protocol; errors use stderr.

The host must show **Full access to your computer** and obtain separate consent
before spawning this helper. There are no wasm permissions: native full access is
broader than any document grant. Revocation/disable kills the process; undo removes
the document stamp but cannot undo external native side effects in general.

The checked-in manifest targets **Linux x86_64 only**. Build and package there:

```sh
CARGO_BUILD_JOBS=4 flock ../../../.cargo.lock cargo build --release
mkdir -p bin
cp target/release/native_system_stamp bin/helper
craft-ext build --release  # native-only: no wasm modules
craft-ext check
craft-ext test             # no automated session tests; use smoke below
craft-ext pack --out native-system-stamp.craftplugin
```

On Apple Silicon macOS, build the helper with Rust and `flock`, change the manifest's
`linux-x86_64` key to `macos-aarch64` before building and packing, and use a writable
local lock path. Do not publish one package as both platforms: each archive contains
only the executable built for its declared platform. Plain `cargo build` also compiles
the source; copy `target/debug/native_system_stamp` for a debug package.

Run `../../scripts/cross-app-smoke.sh` from this checkout for the Linux build,
package and real app CLI tests. Built binaries and archives are intentionally
ignored. Use `<app>-cli plugin setup <id> --enable <app>-apply --full-access`;
contributions for the other app remain unavailable by contract.

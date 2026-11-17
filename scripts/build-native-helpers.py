#!/usr/bin/env python3
"""Build every native example's helper for this platform into OUT/<example>/<manifest path>.

usage: build-native-helpers.py --out DIR
"""
import argparse, json, pathlib, platform, shutil, subprocess, tomllib

a = argparse.ArgumentParser()
a.add_argument("--out", required=True)
args = a.parse_args()
root = pathlib.Path(__file__).resolve().parents[1]
system = {"Darwin": "macos", "Linux": "linux"}[platform.system()]
machine = {"arm64": "aarch64", "AMD64": "x86_64"}.get(platform.machine(), platform.machine())
host = f"{system}-{machine}"
built = []
for manifest in sorted((root / "examples").glob("*/craft-plugin.toml")):
    native = tomllib.loads(manifest.read_text()).get("native")
    if not native:
        continue
    folder = manifest.parent
    target = native["executables"].get(host)
    assert target, f"{folder.name}: no helper declared for {host}"
    cargo = ["cargo", "build", "--release", "--message-format=json-render-diagnostics", "--manifest-path", str(folder / "Cargo.toml")]
    lines = subprocess.run(cargo, check=True, text=True, stdout=subprocess.PIPE).stdout.splitlines()
    exes = [m["executable"] for m in map(json.loads, lines) if m.get("reason") == "compiler-artifact" and m.get("executable")]
    assert len(exes) == 1, f"{folder.name}: expected one helper binary, got {exes}"
    dest = pathlib.Path(args.out) / folder.name / target
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(exes[0], dest)
    dest.chmod(0o755)
    built.append(f"{folder.name}/{target}")
assert built, "no native examples"
print("\n".join(built))

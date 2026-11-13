#!/usr/bin/env python3
"""Build a portable, signed local TEST catalogue from packed examples.

usage: build-test-catalogue.py --craft-ext BIN --packages DIR --out DIR

The catalogue directory holds index.json(.sig), packages/ and assets/ with relative asset URLs,
so it can be moved or zipped. It is signed with a fresh TEST key whose private half is deleted
after signing; only TEST-KEY.pub is kept. Debug builds trust it through
CRAFT_CATALOGUE_TEST_KEY + CRAFT_CATALOGUE_ALLOW_FILE=1 for a file:// catalogue only; release
builds and HTTPS catalogues never trust it. Rebuild with a new key rather than reusing one.
"""
import argparse, json, pathlib, shutil, subprocess, tempfile

a = argparse.ArgumentParser()
a.add_argument("--craft-ext", required=True)
a.add_argument("--packages", required=True)
a.add_argument("--out", required=True)
args = a.parse_args()
out = pathlib.Path(args.out).resolve()
if out.exists():
    shutil.rmtree(out)
(out / "assets").mkdir(parents=True)
for package in sorted(pathlib.Path(args.packages).glob("*.craftplugin")):
    shutil.copy2(package, out / "assets" / package.name)

def run(*cmd):
    p = subprocess.run([args.craft_ext, *map(str, cmd), "--json"], text=True, capture_output=True)
    if p.returncode:
        raise SystemExit(f"{cmd}: {p.stderr}")
    return json.loads(p.stdout)

built = run("catalogue", "build", "--local-assets", out / "assets", "--out", out)
with tempfile.TemporaryDirectory() as tmp:
    key = pathlib.Path(tmp) / "test.key"
    public = run("catalogue", "keygen", "--repo", tmp, "--out", key)["public_key"]
    run("catalogue", "sign", "--repo", out, "--key", key)
(out / "TEST-KEY.pub").write_text(public + "\n")
index = json.loads((out / "index.json").read_text())
urls = [a["url"] for p in index["packages"] for a in json.loads((out / "packages" / f"{p['id']}.json").read_text())["releases"][-1]["assets"]]
assert all("://" not in u for u in urls), urls
(out / "README.txt").write_text(f"""Craft plugin TEST catalogue ({len(index['packages'])} packages, serial {index['serial']}).
Signed with a throwaway TEST key. Only debug builds trust it, for a file:// catalogue:

  export CRAFT_CATALOGUE_URL="file://$PWD/"          # run from this directory; keep the trailing /
  export CRAFT_CATALOGUE_ALLOW_FILE=1
  export CRAFT_CATALOGUE_TEST_KEY={public}
  export CRAFT_EXT_DIR="$(mktemp -d)"               # throwaway plugin store

It expires 30 days after it was built ({index['expires']} Unix time); rebuild it after that.
""")
print(json.dumps({"packages": built["packages"], "public_key": public, "out": str(out)}))

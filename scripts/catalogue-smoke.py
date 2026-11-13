#!/usr/bin/env python3
"""Install every catalogue package in every app it declares, through each app's CLI.

usage: catalogue-smoke.py --catalogue DIR --bin-dir DIR [--apps effectcraft,photocraft,figcraft]

Checks: refresh verifies the TEST signature; install never enables; setup without required grants
is refused; setup with exactly the required grants (and --full-access for native) succeeds; a
revoked permission raises the grant generation. FigCraft has no raster.filter, so packages whose
contributions are all raster filters must be refused there.
"""
import argparse, json, os, pathlib, subprocess, tempfile

a = argparse.ArgumentParser()
a.add_argument("--catalogue", required=True)
a.add_argument("--bin-dir", required=True)
a.add_argument("--apps", default="effectcraft,photocraft,figcraft")
args = a.parse_args()
cat = pathlib.Path(args.catalogue).resolve()
env = dict(os.environ, CRAFT_CATALOGUE_URL=cat.as_uri() + "/", CRAFT_CATALOGUE_ALLOW_FILE="1",
           CRAFT_CATALOGUE_TEST_KEY=(cat / "TEST-KEY.pub").read_text().strip())
index = json.loads((cat / "index.json").read_text())
failures, passes = [], 0

def cli(app, *cmd, ok=True):
    p = subprocess.run([str(pathlib.Path(args.bin_dir) / f"{app}-cli"), "plugin", *cmd, "--json"], env=env, text=True, capture_output=True)
    if ok and p.returncode:
        raise RuntimeError(f"{app} {cmd}: exit {p.returncode}: {p.stderr.strip()}")
    if not ok and p.returncode == 0:
        raise RuntimeError(f"{app} {cmd}: expected refusal, got {p.stdout.strip()}")
    return json.loads(p.stdout) if ok and p.stdout.strip() else p.stderr

for app in args.apps.split(","):
    env["CRAFT_EXT_DIR"] = tempfile.mkdtemp(prefix=f"catalogue-smoke-{app}-")
    cli(app, "refresh")
    for row in index["packages"]:
        if app not in row["apps"]:
            continue
        pid = row["id"]
        try:
            raster_only = set(row["kinds"]) == {"effect"}
            if app == "figcraft" and raster_only:
                cli(app, "install", pid, ok=False)
                print(f"PASS {app} {pid}: refused (no raster.filter)"); passes += 1
                continue
            cli(app, "install", pid)
            listed = cli(app, "list")
            assert not listed["apps"].get(app, {}).get(pid, {}).get("enabled"), "install enabled something"
            sheet = cli(app, "permissions", pid)
            grants = [g for r in sheet["required"] for g in ("--grant", r["permission"])]
            native = ["--full-access"] if sheet.get("native") else []
            if grants or native:
                cli(app, "setup", pid, "--all", ok=False)
            cli(app, "setup", pid, "--all", *grants, *native)
            state = cli(app, "list")["apps"][app][pid]
            assert state.get("enabled"), f"nothing enabled: {state}"
            if grants:
                before = state["grant_generation"]
                cli(app, "revoke", pid, grants[1])
                assert cli(app, "list")["apps"][app][pid]["grant_generation"] > before
            print(f"PASS {app} {pid}: enabled {state['enabled']} grants {grants[1::2]} native={bool(native)}"); passes += 1
        except Exception as e:  # report every package, then fail
            failures.append(f"{app} {pid}: {e}")
            print(f"FAIL {app} {pid}: {e}")
print(f"catalogue smoke: {passes} passed, {len(failures)} failed")
raise SystemExit(1 if failures else 0)

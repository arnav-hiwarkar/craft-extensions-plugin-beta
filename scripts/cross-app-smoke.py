"""Real CLI smoke tests, isolated installation and explicit grants; no signing keys."""
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
work = Path(tempfile.mkdtemp(prefix="cross-app-smoke-"))
print(f"Evidence: {work}", flush=True)
env = dict(os.environ, CRAFT_EXT_DIR=str(work / "store"), CARGO_BUILD_JOBS="4")
# craft-ext itself invokes cargo; serialize nested builds where flock exists (Linux).
wrapper = work / "bin"
wrapper.mkdir()
cargo = shutil.which("cargo")
lock = root.parent / ".cargo.lock"
if cargo and shutil.which("flock"):
    (wrapper / "cargo").write_text(
        "#!/bin/sh\nexec env CARGO_BUILD_JOBS=4 flock " + shlex.quote(str(lock)) + " " + shlex.quote(cargo) + ' "$@"\n'
    )
    (wrapper / "cargo").chmod(0o755)
env["PATH"] = str(wrapper) + os.pathsep + env["PATH"]
x = Path(os.environ.get("CRAFT_EXT_BIN", root / "target/debug/craft-ext"))
# CI passes prebuilt packages and app CLIs; locally everything is built from source.
packages = os.environ.get("CRAFT_PACKAGES")
app_bins = os.environ.get("CRAFT_APP_BIN_DIR")
system = {"Darwin": "macos", "Linux": "linux", "Windows": "windows"}[__import__("platform").system()]
machine = {"arm64": "aarch64", "AMD64": "x86_64"}.get(__import__("platform").machine(), __import__("platform").machine())
host_platform = f"{system}-{machine}"
count = 0

def run(*args, fail=False, error_contains=None):
    global count
    count += 1
    p = subprocess.run(list(map(str, args)), env=env, text=True, capture_output=True)
    (work / f"{count:03}.log").write_text(f"{args}\nexit={p.returncode}\n{p.stdout}\n{p.stderr}")
    print(f"{'REFUSAL' if fail else 'RUN'} {args}: {p.returncode}", flush=True)
    assert (p.returncode != 0) if fail else (p.returncode == 0), p.stdout + p.stderr
    combined = p.stdout + p.stderr
    if error_contains:
        assert error_contains.lower() in combined.lower(), combined
    return combined if fail else p.stdout

names = ["cross-vignette", "cross-review", "native-system-stamp"]
if packages:
    for name in names:
        shutil.copy2(Path(packages) / (name + ".craftplugin"), work / (name + ".craftplugin"))
else:
    for name in names:
        d = root / "examples" / name
        if name.startswith("native"):
            run("cargo", "build", "--release", "--manifest-path", d / "Cargo.toml")
            (d / "bin").mkdir(exist_ok=True)
            # Only this host's helper is built locally; map the other platform to the same binary.
            for platform in ["linux-x86_64", "macos-aarch64"]:
                shutil.copy2(d / "target/release/native_system_stamp", d / f"bin/helper-{platform}")
        run(x, "build", d, "--release")
        run(x, "check", d)
        run(x, "test", d)
        run(x, "pack", d, "--out", work / (name + ".craftplugin"))
    # The host's packaged-helper test uses a protocol-only fixture (scripts/e2e.sh); this helper
    # accepts only EffectCraft and FigCraft, so it is exercised through the app CLIs below.
    run("cargo", "test", "--manifest-path", root / "examples/cross-vignette/Cargo.toml")
v = "io.github.arnav-hiwarkar.cross-vignette"
r = "io.github.arnav-hiwarkar.cross-review"
n = "io.github.arnav-hiwarkar.native-system-stamp"
for app in ["effectcraft", "photocraft", "figcraft"]:
    b = (Path(app_bins) if app_bins else root.parent / app / "target/debug") / (app + "-cli")
    for name, ident in [("cross-vignette",v),("cross-review",r),("native-system-stamp",n)]:
        if name.startswith("native") and app == "photocraft":
            continue
        if name == "cross-vignette" and app == "figcraft":
            run(b,"plugin","install-file",work/(name+".craftplugin"),fail=True)
            print("BLOCKED: FigCraft has no raster.filter dispatch", flush=True)
            continue
        run(b,"plugin","install-file",work/(name+".craftplugin"))
        if name == "cross-review":
            run(b,"plugin","setup",ident,"--enable",app+"-apply",fail=True,error_contains="Permission needed")
            run(b,"plugin","setup",ident,"--enable",app+"-apply","--grant","document.edit:document")
        elif name.startswith("native"):
            label=run(b,"plugin","permissions",ident)
            assert "full access" in label.lower(), label
            run(b,"plugin","setup",ident,"--enable",app+"-apply","--grant","document.edit:document",fail=True,error_contains="Native consent needed")
            run(b,"plugin","setup",ident,"--enable",app+"-apply,"+app+"-panel","--full-access","--grant","document.edit:document")
        else:
            run(b,"plugin","setup",ident,"--all")
    if app == "effectcraft":
        base=work/"base.ecproj"
        run(b,"--empty","run","comp.new",'{"name":"Main","width":32,"height":32}',"layer.newSolid",'{"name":"Original","color":"#808080"}',"--save-as",base)
        # Apply, inspect, Undo and inspect in one session, so Undo is the real one-step check.
        def names(step): return [l["name"] for l in step["result"]["layers"]]
        out=json.loads(run(b,"--project",base,"run","plugin."+r+".effectcraft-apply","{}","comp.info","{}","edit.undo","{}","comp.info","{}","--json"))
        assert names(out[1]) == ["Reviewed"] and names(out[3]) == ["Original"], out
        native=json.loads(run(b,"--project",base,"run","plugin."+n+".effectcraft-apply","{}","comp.info","{}","edit.undo","{}","comp.info","{}","--json"))
        assert names(native[1])[0] == "System stamp" and names(native[3]) == ["Original"], native
        run(b,"--project",base,"render-frame","--out",work/"base.png")
        fx=work/"fx.ecproj"
        run(b,"--project",base,"run","layer.select",'{"layers":["Original"]}',"effect.apply",json.dumps({"effect":v+"/vignette"}),"--save-as",fx)
        for i in [1,2]:run(b,"--project",fx,"render-frame","--out",work/f"fx{i}.png")
        assert (work/"fx1.png").read_bytes()==(work/"fx2.png").read_bytes()
        assert (work/"fx1.png").read_bytes()!=(work/"base.png").read_bytes()
        run(b,"plugin","revoke",r,"document.edit:document")
        # A revoked contribution is unloaded, so its command no longer exists (or reports the grant).
        refused=run(b,"--project",base,"run","plugin."+r+".effectcraft-apply","{}",fail=True)
        assert "unknown command" in refused.lower() or "permission" in refused.lower(), refused
    elif app == "figcraft":
        script=work/"fig-review.json"
        reviewed=work/"fig-reviewed.figcraft"
        script.write_text(json.dumps([{"command":"node.create","params":{"type":"rectangle","name":"Original","width":32,"height":32}},{"command":"plugin."+r+".figcraft-apply","params":{}}]))
        run(b,"run",script,"--plugins","--out",reviewed)
        info=run(b,"info",reviewed)
        assert "Reviewed" in info and "Original" in info, info
        undo=work/"fig-undo.json"
        undo.write_text(json.dumps([{"command":"edit.undo","params":{}}]))
        undone=work/"fig-undone.figcraft"
        run(b,"run",undo,"--in",reviewed,"--out",undone)
        info=run(b,"info",undone)
        assert "Original" in info and "Reviewed" not in info, info
        native_script=work/"fig-native.json"
        native_script.write_text(json.dumps([{"command":"plugin."+n+".figcraft-apply","params":{}}]))
        stamped=work/"fig-stamped.figcraft"
        run(b,"run",native_script,"--in",undone,"--plugins","--out",stamped)
        info=run(b,"info",stamped)
        assert "System stamp" in info, info
        run(b,"run",undo,"--in",stamped,"--out",work/"fig-native-undone.figcraft")
        info=run(b,"info",work/"fig-native-undone.figcraft")
        assert "System stamp" not in info and "Original" in info, info
    else:
        run(b,"run","--new",'{"width":16,"height":16,"background":"#808080"}',"--cmd","plugin."+v+"/vignette","--out",work/"photo.pcraft")
        run(b,"run",work/"photo.pcraft","--cmd","plugin.session."+r+"/photocraft-apply","--out",work/"reviewed.pcraft")
print("CROSS-APP SMOKE completed; FigCraft raster integration remains BLOCKED.")

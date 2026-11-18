import json, os, shutil, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib


def snap(r):
    d = os.path.join(lib.ROOT, "snap", f"r{r}")
    os.makedirs(d, exist_ok=True)
    for f in os.listdir(lib.SPEC):
        if f.endswith(".jsonl"):
            shutil.copy(os.path.join(lib.SPEC, f), d)
    return d


def diff(old_dir, new_dir=lib.SPEC):
    old, _ = lib.load(old_dir) if os.path.isdir(old_dir) else ({}, [])
    new, _ = lib.load(new_dir)
    strip = lambda e: {k: v for k, v in e.items() if not k.startswith("_")}
    added = sorted(set(new) - set(old))
    removed = sorted(set(old) - set(new))
    changed = {}
    for i in sorted(set(old) & set(new)):
        a, b = strip(old[i]), strip(new[i])
        if a != b:
            changed[i] = {k: {"was": a.get(k), "now": b.get(k)} for k in sorted(set(a) | set(b)) if a.get(k) != b.get(k)}
    return {"added": added, "removed": removed, "changed": changed}


if __name__ == "__main__":
    if sys.argv[1] == "snap":
        print(snap(int(sys.argv[2])))
    else:
        d = diff(os.path.join(lib.ROOT, "snap", f"r{int(sys.argv[2])}"))
        print(json.dumps({"added": len(d["added"]), "removed": len(d["removed"]), "changed": len(d["changed"])}))

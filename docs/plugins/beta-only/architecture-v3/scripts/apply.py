import glob, json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib


def main():
    r = int(sys.argv[1])
    stage = sys.argv[2] if len(sys.argv) > 2 else "shards"
    ents, _ = lib.load()
    sc = lib.schema()
    files = sorted(glob.glob(os.path.join(lib.ROOT, "patches", f"r{r}", stage, "*.jsonl")))
    by_id, bad = {}, []
    for f in files:
        for n, line in enumerate(open(f), 1):
            line = line.strip()
            if not line:
                continue
            try:
                p = json.loads(line)
            except json.JSONDecodeError:
                bad.append(f"{os.path.basename(f)}:{n}")
                continue
            if lib.kind_of(p.get("id", "")) not in sc["kinds"]:
                bad.append(f"{os.path.basename(f)}:{n} bad id")
                continue
            by_id.setdefault(p["id"], []).append((os.path.basename(f), p))
    conflicts, applied = [], 0
    for i, ps in by_id.items():
        srcs = {s for s, _ in ps}
        if len(srcs) > 1:
            conflicts.append({"id": i, "patches": sorted(srcs)})
            continue
        for _, p in ps:
            if p.get("_delete"):
                ents.pop(i, None)
            elif p.get("_merge") and i in ents:
                ents[i].update({k: v for k, v in p.items() if k != "_merge"})
            else:
                ents[i] = {k: v for k, v in p.items() if k != "_merge"}
            applied += 1
    lib.save(ents)
    rep = {"round": r, "stage": stage, "applied": applied, "conflicts": conflicts, "bad_lines": bad}
    lib.dump(rep, os.path.join(lib.ROOT, "reports", f"apply-r{r}-{stage}.json"))
    print(json.dumps({"applied": applied, "conflicts": len(conflicts), "bad_lines": len(bad)}))


if __name__ == "__main__":
    main()

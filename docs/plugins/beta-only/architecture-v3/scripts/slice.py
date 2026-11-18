import argparse, json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib, check, snap


def strip(e):
    return {k: v for k, v in e.items() if not k.startswith("_")}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("shard")
    ap.add_argument("round", type=int)
    ap.add_argument("--full", action="store_true")
    a = ap.parse_args()
    sc = lib.schema()
    sh = lib.shards()
    ents, _ = lib.load()
    errs, warns = check.spec_checks(ents, sc, False)
    tr_path = os.path.join(lib.ROOT, "reports", "trace.json")
    trace = json.load(open(tr_path))["gap_list"] if os.path.exists(tr_path) else []
    ledger = []
    lp = os.path.join(lib.ROOT, "ledger", "gaps.jsonl")
    if os.path.exists(lp):
        ledger = [json.loads(l) for l in open(lp) if l.strip()]
    out = {"shard": a.shard, "round": a.round}
    if a.shard == "integration":
        out["lens"] = sh["integration"]["lens"]
        keep = {"CMP": ("id", "title", "layer", "deps", "ws", "area"), "CON": ("id", "name", "title", "stmt", "props", "budget", "cmp", "area"),
                "PERM": ("id", "name", "risk", "contexts", "area"), "DEC": ("id", "title", "status", "choice", "area"),
                "WS": ("id", "title", "deps", "milestone"), "QAS": ("id", "attr", "title", "measure", "area"), "TERM": ("id", "title", "def", "banned")}
        out["interfaces"] = [{k: e[k] for k in keep[lib.kind_of(i)] if k in e} for i, e in sorted(ents.items()) if lib.kind_of(i) in keep]
        out["ops_by_name"] = sorted({e.get("name") for e in ents.values() if lib.kind_of(e["id"]) == "OP"} - {None})
        out["checks"] = [{"rule": r, "id": i, "msg": m} for r, i, m in errs if r in ("C10", "C15", "C11")][:80]
        out["ledger"] = [{k: g[k] for k in ("key", "claim", "status")} for g in ledger if g.get("shard") == "integration"]
    else:
        lens = next(s["lens"] for s in sh["shards"] if s["id"] == a.shard)
        out["lens"] = lens
        mine = sorted(i for i, e in ents.items() if e.get("area") == a.shard)
        prev = os.path.join(lib.ROOT, "snap", f"r{a.round - 1}")
        d = snap.diff(prev) if a.round > 1 and not a.full and os.path.isdir(prev) else None
        if d is None:
            out["mode"] = "full"
            out["entities"] = [strip(ents[i]) for i in mine]
            focus = mine
            out["changed_count"] = len(mine)
        else:
            touched = [i for i in mine if i in d["changed"] or i in d["added"]]
            out["mode"] = "diff"
            out["changed"] = [{"now": strip(ents[i]), "fields_changed": d["changed"].get(i, "added")} for i in touched]
            out["removed"] = [i for i in d["removed"]]
            out["unchanged_index"] = [{"id": i, "title": ents[i].get("title", "")} for i in mine if i not in touched]
            focus = touched
            out["changed_count"] = len(touched) + len([i for i in d["removed"] if i.split("-")[1:2]])
            out["context"] = [strip(ents[i]) for i in sorted(lib.neighbours(ents, touched)) if ents[i].get("area") == a.shard]
        out["neighbours"] = [lib.brief(ents[i]) for i in sorted(lib.neighbours(ents, focus)) if ents[i].get("area") != a.shard][:250]
        ids = set(mine)
        out["checks"] = [{"rule": r, "id": i, "msg": m} for r, i, m in errs if i in ids or i == a.shard]
        out["check_warnings"] = [{"rule": r, "id": i, "msg": m} for r, i, m in warns if i in ids][:60]
        out["trace_gaps"] = [g for g in trace if g.get("area") == a.shard]
        out["ledger"] = [{k: g.get(k) for k in ("key", "claim", "status", "round")} for g in ledger if g.get("shard") == a.shard]
    p = os.path.join(lib.ROOT, "reports", "slices", f"r{a.round}-{a.shard}.json")
    lib.dump(out, p)
    size = os.path.getsize(p)
    print(json.dumps({"path": p, "approx_tokens": size // 4, "checks": len(out.get("checks", [])), "trace_gaps": len(out.get("trace_gaps", [])), "changed": out.get("changed_count", -1)}))


if __name__ == "__main__":
    main()

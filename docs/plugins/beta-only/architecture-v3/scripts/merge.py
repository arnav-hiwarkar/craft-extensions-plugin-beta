import glob, json, os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib

LISTS = ("src", "alias", "props", "options", "contexts", "banned")


def main():
    sc = lib.schema()
    frags = sorted(glob.glob(os.path.join(lib.ROOT, "extract", "frag", "*.jsonl")) + glob.glob(os.path.join(lib.ROOT, "extract", "inc", "*.jsonl")))
    recs, bad = {}, []
    for f in frags:
        for n, line in enumerate(open(f), 1):
            line = line.strip()
            if not line:
                continue
            try:
                e = json.loads(line)
            except json.JSONDecodeError:
                bad.append(f"{os.path.basename(f)}:{n}")
                continue
            if "id" in e and lib.kind_of(e["id"]) in sc["kinds"]:
                e["_frag"] = os.path.basename(f)
                recs.setdefault(e["id"], []).append(e)
            else:
                bad.append(f"{os.path.basename(f)}:{n} no valid id")
    ents, conflicts = {}, []
    for i, rs in recs.items():
        m = {}
        for r in rs:
            for k, v in r.items():
                if k == "_frag":
                    continue
                if k not in m or m[k] in (None, "", []):
                    m[k] = v
                elif isinstance(v, list) and isinstance(m[k], list):
                    m[k] = m[k] + [x for x in v if x not in m[k]]
                elif v != m[k] and v not in (None, "", []):
                    conflicts.append({"id": i, "field": k, "values": [str(m[k])[:300], str(v)[:300]], "frags": sorted({x["_frag"] for x in rs})})
                    if isinstance(v, str) and isinstance(m[k], str) and len(v) > len(m[k]):
                        m[k] = v
        ents[i] = m
    alias = {}
    for i, e in ents.items():
        for a in [e.get("name")] + list(e.get("alias") or []):
            if a:
                alias.setdefault(a, set()).add(i)
    unresolved = {}
    for i, e in ents.items():
        for f in sc["kinds"][lib.kind_of(i)]["refs"]:
            new = []
            for t in e.get(f) or []:
                if t in ents:
                    new.append(t)
                elif len(alias.get(t, ())) == 1:
                    new.append(next(iter(alias[t])))
                else:
                    new.append(t)
                    unresolved[t] = unresolved.get(t, 0) + 1
            if f in e:
                e[f] = list(dict.fromkeys(new))
    names = {}
    for i, e in ents.items():
        if e.get("name"):
            names.setdefault((lib.kind_of(i), e["name"]), []).append(i)
    for (k, n), ids in names.items():
        if len(ids) > 1:
            conflicts.append({"id": ",".join(sorted(ids)), "field": "name", "values": [n], "frags": []})
    toks = lambda t: set(re.findall(r"[a-z0-9]+", t.lower())) - {"the", "a", "and", "of", "to", "in", "for", "with", "plugin", "plugins"}
    caps = [(i, toks(e.get("title", ""))) for i, e in sorted(ents.items()) if lib.kind_of(i) == "CAP"]
    near = []
    for x in range(len(caps)):
        for y in range(x + 1, len(caps)):
            a, b = caps[x][1], caps[y][1]
            if a and b and len(a & b) / len(a | b) >= 0.6:
                near.append([caps[x][0], caps[y][0]])
    lib.save(ents)
    rep = {"entities": len(ents), "conflicts": len(conflicts), "near_duplicates": near, "bad_lines": bad, "unresolved_refs": unresolved, "conflict_list": conflicts}
    lib.dump(rep, os.path.join(lib.ROOT, "reports", "merge.json"))
    by = {}
    for i in ents:
        by[lib.kind_of(i)] = by.get(lib.kind_of(i), 0) + 1
    print(json.dumps({"entities": len(ents), "by_kind": by, "conflicts": len(conflicts), "near_duplicates": len(near), "bad_lines": len(bad), "unresolved_refs": len(unresolved)}))


if __name__ == "__main__":
    main()

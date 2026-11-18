import csv, json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib

APP = {"AE": "EC", "PS": "PC", "FG": "FC"}


def cap_state(ents, c):
    e = ents.get(c)
    if not e:
        return "DANGLING"
    if e.get("status") == "proposed":
        return "PROPOSED"
    if e.get("status") == "retired":
        return "RETIRED"
    if not [v for v in e.get("via") or [] if v in ents]:
        return "NO-REALISER"
    if not [t for t in e.get("tests") or [] if t in ents]:
        return "NO-TEST"
    for d in e.get("decisions") or []:
        if ents.get(d, {}).get("status") == "open-owner":
            return "OWNER-DECISION"
    return "COVERED"


def main():
    ents, _ = lib.load()
    sc_net = lib.schema()["owner_rule_decisions"]["network"]
    rows, gaps = [], []
    for i in sorted(ents):
        e = ents[i]
        k = lib.kind_of(i)
        if k not in ("INC", "API", "REQ", "FLG"):
            continue
        caps = set(e.get("caps") or [])
        for a in e.get("uses_api") or []:
            caps.update(ents.get(a, {}).get("caps") or [])
            if a not in ents:
                caps.add(a)
        if k == "REQ":
            vb = e.get("verified_by") or []
            st = "COVERED" if vb and all(v in ents for v in vb) else "NO-VERIFICATION"
            rows.append({"row": i, "kind": k, "app": "-", "rank": "", "cap": "-", "state": st, "realisers": "", "tests": ";".join(vb)})
            if st != "COVERED":
                gaps.append({"row": i, "cap": "-", "state": st, "area": e.get("area")})
            continue
        if sc_net in (e.get("decisions") or []):
            rows.append({"row": i, "kind": k, "app": i.split("-")[1], "rank": e.get("rank", ""), "cap": "-", "state": "OWNER-EXCLUDED", "realisers": "", "tests": ""})
            continue
        if not caps:
            rows.append({"row": i, "kind": k, "app": i.split("-")[1], "rank": e.get("rank", ""), "cap": "-", "state": "UNMAPPED", "realisers": "", "tests": ""})
            gaps.append({"row": i, "cap": "-", "state": "UNMAPPED", "area": e.get("area")})
        for c in sorted(caps):
            st = cap_state(ents, c)
            ce = ents.get(c, {})
            rows.append({"row": i, "kind": k, "app": i.split("-")[1], "rank": e.get("rank", ""), "cap": c, "state": st,
                         "realisers": ";".join(ce.get("via") or []), "tests": ";".join(ce.get("tests") or [])})
            if st not in ("COVERED", "OWNER-DECISION"):
                gaps.append({"row": i, "cap": c, "state": st, "area": ce.get("area", e.get("area"))})
            app = APP.get(i.split("-")[1])
            if app and ce and c.split("-")[1] not in (app, "COM", "SEC"):
                gaps.append({"row": i, "cap": c, "state": "WRONG-APP", "area": ce.get("area")})
    out = os.path.join(lib.ROOT, "reports")
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "trace.csv"), "w", newline="") as o:
        w = csv.DictWriter(o, fieldnames=["row", "kind", "app", "rank", "cap", "state", "realisers", "tests"])
        w.writeheader()
        w.writerows(rows)
    summ = {}
    for r in rows:
        s = summ.setdefault(r["app"] + ":" + r["kind"], {})
        s[r["state"]] = s.get(r["state"], 0) + 1
    rep = {"rows": len(rows), "gaps": len(gaps), "summary": summ, "gap_list": gaps}
    lib.dump(rep, os.path.join(out, "trace.json"))
    print(json.dumps({"rows": len(rows), "gaps": len(gaps), "summary": summ}))
    sys.exit(1 if gaps else 0)


if __name__ == "__main__":
    main()

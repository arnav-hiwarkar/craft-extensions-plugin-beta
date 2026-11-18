import glob, json, os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib

OUT = os.path.join(lib.ROOT, "out")


def cell(s, n=160):
    s = str(s or "").replace("|", "\\|").replace("\n", " ")
    return s if len(s) <= n else s[: n - 1] + "…"


def table(head, rows):
    return "\n".join(["| " + " | ".join(head) + " |", "|" + "---|" * len(head)] + ["| " + " | ".join(cell(c) for c in r) + " |" for r in rows]) + "\n"


def of(ents, k, **flt):
    return [e for i, e in sorted(ents.items()) if lib.kind_of(i) == k and all(e.get(f) == v for f, v in flt.items()) and e.get("status") != "retired"]


def topo(ents):
    cm = {e["id"]: e for e in of(ents, "CMP")}
    order, seen = [], set()

    def v(i):
        if i in seen or i not in cm:
            return
        seen.add(i)
        for d in cm[i].get("deps") or []:
            v(d)
        order.append(i)

    for i in sorted(cm, key=lambda x: (lib.schema()["layer_order"].index(cm[x].get("layer", "plugin")) if cm[x].get("layer") in lib.schema()["layer_order"] else 9, x)):
        v(i)
    return [cm[i] for i in order]


def gen(ents):
    g = {}
    g["components"] = table(["Component", "Layer", "Repo", "Depends on", "Workstream"], [[e["id"], e.get("layer"), e.get("repo"), ", ".join(e.get("deps") or []), ", ".join(e.get("ws") or [])] for e in topo(ents)])
    g["contracts"] = table(["Contract", "What it guarantees", "Budget", "Tests"], [[e.get("name", e["id"]), e.get("stmt"), e.get("budget"), ", ".join(e.get("tests") or [])] for e in of(ents, "CON")])
    g["perms"] = table(["Permission", "Risk", "Contexts", "Meaning"], [[e.get("name"), e.get("risk"), ", ".join(e.get("contexts") or []), e.get("stmt")] for e in of(ents, "PERM")])
    g["qas"] = table(["Scenario", "Attribute", "Stimulus", "Response measure", "Tests"], [[e["title"], e.get("attr"), e.get("stimulus"), e.get("measure"), ", ".join(e.get("tests") or [])] for e in of(ents, "QAS")])
    g["decisions-open"] = "\n".join(
        f"### {e['id']}: {e['title']}\n\n{e.get('stmt')}\n\n" + "".join(f"- Option: {o}\n" for o in e.get("options") or []) + f"\n**Recommendation:** {e.get('choice')}\n\n**Why:** {e.get('rationale')}\n"
        for e in of(ents, "DEC", status="open-owner"))
    g["decisions-closed"] = table(["Decision", "Choice", "Affects"], [[f"{e['id']}: {e['title']}", e.get("choice"), ", ".join((e.get("affects") or [])[:4])] for e in of(ents, "DEC", status="decided")])
    ws_owns = {}
    for e in of(ents, "CMP"):
        for w in e.get("ws") or []:
            ws_owns.setdefault(w, []).append(e["id"])
    g["workstreams"] = table(["Thread", "Owns", "Waits on", "Milestone"], [[f"{e['id']}: {e['title']}", ", ".join(ws_owns.get(e["id"], [])), ", ".join(e.get("deps") or []), e.get("milestone")] for e in of(ents, "WS")])
    for app, name in (("COM", "All three apps"), ("EC", "EffectCraft"), ("PC", "PhotoCraft"), ("FC", "FigCraft")):
        cards = []
        for e in of(ents, "FLG"):
            if e["id"].split("-")[1] != app:
                continue
            inc = ", ".join(ents.get(i, {}).get("title", i) for i in e.get("incumbents") or [])
            sheet = f"- **Sheet:** {e['sheet']} · {e.get('level')}" + (f" · {e['notices']}" if e.get("notices") else "") + "\n" if e.get("sheet") else ""
            builds = f"- **Other builds:** {e['builds']}\n" if e.get("builds") else ""
            cards.append(f"### {e['title']} · {e.get('milestone')}\n\n{e.get('stmt')}\n\n- **Matches:** {inc}\n{sheet}{builds}- **Capabilities:** {', '.join(e.get('caps') or [])}\n- **Accepted when:** "
                         + "; ".join(f"{t}: {ents.get(t, {}).get('measure', '?')}" for t in e.get("tests") or []) + "\n")
        g[f"flagships-{app}"] = "\n".join(cards)
    by_level = {lv: [] for lv in ("L0", "L1", "L2", "L3", "L4")}
    for e in of(ents, "PERM"):
        if e.get("risk") not in by_level or e["id"] in ("PERM-COM-free", "PERM-COM-authorization"):
            continue
        nm = e.get("name") or e["id"]
        if e.get("levels"):
            base = nm.split(":")[0]
            for grp in e["levels"]:
                by_level[grp["level"]] += [f"{base}:{sc}" for sc in grp["scopes"]]
        else:
            by_level[e["risk"]].append(nm)
    g["perm-levels"] = "\n".join(["| Level | Permissions at this level |", "|---|---|"] + [f"| {lv} | {', '.join(sorted(names))} |" for lv, names in by_level.items()] + ["| Never | N1 to N7: no grant exists |"]) + "\n"
    tr = os.path.join(lib.ROOT, "reports", "trace.json")
    if os.path.exists(tr):
        s = json.load(open(tr))["summary"]
        g["matrix"] = table(["App:row kind", "Covered", "Owner decision", "Excluded by owner (network)", "Gaps"], [[k, v.get("COVERED", 0), v.get("OWNER-DECISION", 0), v.get("OWNER-EXCLUDED", 0), sum(n for st, n in v.items() if st not in ("COVERED", "OWNER-DECISION", "OWNER-EXCLUDED"))] for k, v in sorted(s.items())])
    g["ops-reference"] = table(["Op", "Signature", "Permissions", "Component"], [[e.get("name"), e.get("sig"), ", ".join(e.get("perms") or []), ", ".join(e.get("cmp") or [])] for e in of(ents, "OP")])
    os.makedirs(os.path.join(OUT, "gen"), exist_ok=True)
    for k, v in g.items():
        open(os.path.join(OUT, "gen", k + ".md"), "w").write(v)
    return {k: lib.words(v) for k, v in g.items()}


def digest(ents):
    parts = ["# Spec digest (generated; cite ids in brackets)\n"]
    for k, fields in (("REQ", ("stmt",)), ("CMP", ("layer", "stmt")), ("CON", ("name", "stmt", "budget")), ("PERM", ("name", "risk")), ("DEC", ("status", "choice")),
                      ("QAS", ("attr", "measure")), ("FLG", ("milestone", "stmt")), ("WS", ("milestone",)), ("CAP", ("stmt",))):
        parts.append(f"\n## {k}\n")
        for e in of(ents, k):
            parts.append(f"- [{e['id']}] {e.get('title', '')} — " + " · ".join(cell(e.get(f), 220) for f in fields if e.get(f)))
    ops = {}
    for e in of(ents, "OP"):
        ops.setdefault(e.get("area"), []).append(e.get("name") or e["id"])
    parts.append("\n## OP names by area\n")
    for a, ns in sorted(ops.items()):
        parts.append(f"- {a}: " + ", ".join(ns))
    s = "\n".join(parts) + "\n"
    open(os.path.join(OUT, "digest.md"), "w").write(s)
    return {"digest_words": lib.words(s), "approx_tokens": len(s) // 4}


def fill():
    res = {}
    for t in sorted(glob.glob(os.path.join(OUT, "src", "*.tmpl.md"))):
        s = open(t).read()
        s = re.sub(r"\{\{gen:([a-zA-Z0-9-]+)\}\}", lambda m: open(os.path.join(OUT, "gen", m.group(1) + ".md")).read() if os.path.exists(os.path.join(OUT, "gen", m.group(1) + ".md")) else m.group(0), s)
        name = os.path.basename(t).replace(".tmpl.md", ".md")
        os.makedirs(os.path.join(OUT, "final"), exist_ok=True)
        open(os.path.join(OUT, "final", name), "w").write(s)
        res[name] = lib.words(s)
    res["total"] = sum(res.values())
    return res


if __name__ == "__main__":
    ents, _ = lib.load()
    mode = sys.argv[1]
    print(json.dumps(gen(ents) if mode == "gen" else digest(ents) if mode == "digest" else fill()))

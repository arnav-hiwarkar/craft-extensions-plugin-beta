import argparse, json, os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib


def dsm_problems(ents, sc):
    out = []
    cmps = {i: e for i, e in ents.items() if lib.kind_of(i) == "CMP"}
    order = {l: n for n, l in enumerate(sc["layer_order"])}
    state = {}

    def visit(i, stack):
        state[i] = 1
        for d in cmps[i].get("deps") or []:
            if d not in cmps:
                continue
            if state.get(d) == 1:
                out.append(("C10", i, "dependency cycle: " + " -> ".join(stack + [i, d])))
            elif not state.get(d):
                visit(d, stack + [i])
        state[i] = 2

    for i in sorted(cmps):
        if not state.get(i):
            visit(i, [])
    ws_deps = {i: set(e.get("deps") or []) for i, e in ents.items() if lib.kind_of(i) == "WS"}
    for i, e in cmps.items():
        li = order.get(e.get("layer"), 99)
        for d in e.get("deps") or []:
            if d not in cmps:
                continue
            ld = order.get(cmps[d].get("layer"), 99)
            if ld > li:
                out.append(("C10", i, f"layer {e.get('layer')} depends on higher layer {cmps[d].get('layer')} ({d})"))
            si, sd = i.split("-")[1], d.split("-")[1]
            if si in ("EC", "PC", "FC") and sd in ("EC", "PC", "FC") and si != sd:
                out.append(("C10", i, f"app-to-app dependency on {d}; shared behaviour belongs in COM"))
            wi, wd = (e.get("ws") or [None])[0], (cmps[d].get("ws") or [None])[0]
            if wi and wd and wi != wd and wd not in ws_deps.get(wi, set()):
                out.append(("C10", i, f"workstream {wi} needs {wd} (via {d}) but does not list it in deps"))
        if len(e.get("ws") or []) > 1:
            out.append(("C10", i, "component owned by more than one workstream"))
    return out


def spec_checks(ents, sc, final):
    errs, warns = [], []
    idre = re.compile(sc["id_pattern"])
    banned = [re.compile(p, re.I) for p in sc["banned_text"]]
    netre = re.compile(sc["network_text"], re.I)
    netperm = re.compile(sc["network_perm_name"])
    netdec = sc["owner_rule_decisions"]["network"]
    names = {}
    aliases = {}
    for i, e in ents.items():
        for n in [e.get("name")] + list(e.get("alias") or []):
            if n:
                names.setdefault(n, []).append(i)
        for a in e.get("alias") or []:
            aliases.setdefault(a, []).append(i)
    inc = lib.incoming(ents)
    terms = [(e.get("title", ""), b) for e in ents.values() if lib.kind_of(e["id"]) == "TERM" for b in e.get("banned") or []]
    for i, e in ents.items():
        k = lib.kind_of(i)
        kd = sc["kinds"].get(k)
        if not idre.match(i):
            errs.append(("C01", i, "id does not match pattern"))
        if not kd:
            continue
        for f in sc["common_required"] + kd["required"]:
            if e.get(f) in (None, "", []):
                errs.append(("C01", i, f"missing required field {f}"))
        if e.get("area") not in sc["areas"]:
            errs.append(("C01", i, f"area {e.get('area')} unknown"))
        if e.get("status") not in sc["status"]:
            errs.append(("C08", i, f"status {e.get('status')} not allowed"))
        if e.get("status") == "proposed":
            (errs if final else warns).append(("C08", i, "proposed entity not yet adjudicated"))
        for f, vals in kd.get("enums", {}).items():
            if e.get(f) not in vals:
                errs.append(("C01", i, f"{f}={e.get(f)} not in {vals}"))
        for f, cap in sc["caps"].items():
            if isinstance(e.get(f), str) and len(e[f]) > cap:
                errs.append(("C03", i, f"{f} is {len(e[f])} chars, cap {cap}"))
        for f, cap in sc["list_caps"].items():
            v = e.get(f) or []
            if len(v) > cap:
                errs.append(("C03", i, f"{f} has {len(v)} items, cap {cap}"))
            if any(isinstance(x, str) and len(x) > sc["item_cap"] for x in v):
                errs.append(("C03", i, f"{f} item over {sc['item_cap']} chars"))
        for f, t in lib.refs(e):
            if t not in ents:
                errs.append(("C02", i, f"{f} -> {t} does not exist"))
            elif lib.kind_of(t) not in kd["refs"][f]:
                errs.append(("C02", i, f"{f} -> {t} has kind {lib.kind_of(t)}, allowed {kd['refs'][f]}"))
            elif ents[t].get("status") == "retired":
                errs.append(("C02", i, f"{f} -> {t} is retired"))
        if e.get("status") != "retired" and netdec not in (e.get("decisions") or []):
            for f in kd.get("nonempty", []):
                if not e.get(f):
                    errs.append(("C06", i, f"{f} is empty"))
            for group in kd.get("nonempty_any", []):
                if not any(e.get(f) for f in group):
                    errs.append(("C06", i, f"one of {group} must be non-empty"))
        txt = lib.text_of(e)
        if not any(re.search(x, i) for x in sc["banned_allow_ids"]):
            for b in banned:
                m = b.search(txt)
                if m:
                    errs.append(("C04", i, f"deferral wording '{m.group(0)}'"))
        if k == "PERM" and netperm.search(e.get("name", "")):
            errs.append(("C05", i, "network permission in a build with no plugin network"))
        m = netre.search(txt)
        if m and i != netdec and netdec not in [t for _, t in lib.refs(e)] and netdec not in (e.get("decisions") or []):
            errs.append(("C05", i, f"network wording '{m.group(0)}' without reference to {netdec}"))
        if k == "DEC":
            if not inc.get(i) and not e.get("affects"):
                errs.append(("C07", i, "decision referenced by nothing"))
            if e.get("status") == "open-owner" and len(e.get("options") or []) < 2:
                errs.append(("C07", i, "open owner decision needs at least two options and a recommended choice"))
        if k == "CAP" and e.get("status") != "retired":
            if not any(lib.kind_of(s) in kd["justified_by"] for s, _ in inc.get(i, [])):
                warns.append(("C09", i, "capability not traced from any incumbent row, API row, requirement, flagship or scenario"))
        if k == "PERM" and e.get("risk") in ("L3", "L4") and not e.get("notice"):
            errs.append(("C12", i, "risk L3/L4 permission needs a third-party notice sentence in field notice"))
        if k == "PERM" and e.get("risk") == "never" and not e.get("decisions"):
            errs.append(("C12", i, "never-grantable permission needs a decision justifying it"))
        if e.get("hot"):
            if not e.get("budget"):
                errs.append(("C13", i, "hot-path entity without a performance budget"))
            if not any(lib.kind_of(s) == "TST" and ents[s].get("kind") == "bench" for s, _ in inc.get(i, [])):
                errs.append(("C13", i, "hot-path entity without a bench test"))
        if k in ("CMP",) and e.get("runs_plugin_code") and not any(lib.kind_of(s) == "QAS" and ents[s].get("attr") == "availability" for s, _ in inc.get(i, [])):
            errs.append(("C14", i, "component runs plugin code but no availability scenario covers it"))
        for word_term, bad in terms:
            if re.search(r"\b" + re.escape(bad) + r"\b", txt, re.I) and lib.kind_of(i) != "TERM":
                warns.append(("C11", i, f"use '{word_term}' instead of '{bad}'"))
        for tok in re.findall(r"`([a-z][a-z0-9_]*(?:\.[a-z0-9_]+)+(?::[a-z0-9_*{},]+)?(?:@\d+)?)`", txt):
            base = tok.split("@")[0]
            if tok not in names and base not in names and not any(n.startswith(base) for n in names):
                warns.append(("C11", i, f"`{tok}` is not the name of any spec entity"))
    for a, ids in aliases.items():
        if len(ids) > 1:
            errs.append(("C15", ",".join(ids), f"alias {a} used by more than one entity"))
    for a, chars in lib.area_chars(ents).items():
        cap = lib.shards()["text_budget_chars"].get(a)
        if cap and chars > cap:
            errs.append(("C03", a, f"area text {chars} chars over budget {cap}"))
    errs += dsm_problems(ents, sc)
    return errs, warns


def doc_checks(ents, sc, d):
    errs = []
    banned = [re.compile(p, re.I) for p in sc["banned_text"]]
    total = 0
    for f, rule in sc["deliverables"].items():
        p = os.path.join(d, f)
        if not os.path.exists(p):
            errs.append(("D01", f, "missing deliverable"))
            continue
        s = open(p).read()
        w = lib.words(s)
        total += w
        if not rule["min_words"] <= w <= rule["max_words"]:
            errs.append(("D02", f, f"{w} words, allowed {rule['min_words']}-{rule['max_words']}"))
        heads = [h.strip() for h in re.findall(r"^#{2,3} (.+)$", s, re.M)]
        for h in rule["headings"]:
            if not any(x.startswith(h) for x in heads):
                errs.append(("D03", f, f"missing heading '{h}'"))
        for n, line in enumerate(s.split("\n"), 1):
            for b in banned:
                if b.search(line) and "no-plugin-network" not in line:
                    errs.append(("D04", f"{f}:{n}", f"deferral wording: {line.strip()[:100]}"))
            for i in re.findall(r"\b(?:REQ|API|INC|CAP|OP|PERM|CON|CMP|DEC|TST|QAS|FLG|WS|TERM)-(?:COM|SEC|EC|PC|FC|AE|PS|FG)-[a-z0-9][a-z0-9._@:/+-]*[a-z0-9]", line):
                if i not in ents:
                    errs.append(("D05", f"{f}:{n}", f"unknown id {i}"))
        if "{{" in s:
            errs.append(("D06", f, "unfilled template placeholder"))
    if not sc["total_words"]["min"] <= total <= sc["total_words"]["max"]:
        errs.append(("D02", "total", f"{total} words, allowed {sc['total_words']['min']}-{sc['total_words']['max']}"))
    return errs, total


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--spec", default=lib.SPEC)
    ap.add_argument("--docs")
    ap.add_argument("--final", action="store_true")
    ap.add_argument("--area")
    ap.add_argument("--out", default=os.path.join(lib.ROOT, "reports", "check.json"))
    a = ap.parse_args()
    sc = lib.schema()
    ents, load_errs = lib.load(a.spec)
    errs, warns = spec_checks(ents, sc, a.final)
    errs = load_errs + errs
    total = None
    if a.docs:
        de, total = doc_checks(ents, sc, a.docs)
        errs += de

    def area(i):
        return ents.get(i.split(",")[0], {}).get("area", i if i in sc["areas"] else "-")

    rows = lambda xs: [{"rule": r, "id": i, "area": area(i), "msg": m} for r, i, m in xs]
    E, W = rows(errs), rows(warns)
    if a.area:
        E = [x for x in E if x["area"] == a.area]
        W = [x for x in W if x["area"] == a.area]
    by = {}
    for x in E:
        by[x["rule"]] = by.get(x["rule"], 0) + 1
    rep = {"entities": len(ents), "errors": len(E), "warnings": len(W), "by_rule": by, "doc_words": total, "error_list": E, "warning_list": W}
    lib.dump(rep, a.out)
    print(json.dumps({k: rep[k] for k in ("entities", "errors", "warnings", "by_rule", "doc_words")}))
    sys.exit(1 if E else 0)


if __name__ == "__main__":
    main()

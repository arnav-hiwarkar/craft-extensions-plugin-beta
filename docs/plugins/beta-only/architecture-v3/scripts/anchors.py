import json, os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib

NORMATIVE = re.compile(r"\b(must|never|always|every|only|refus\w*|requires?|guarantee\w*|budget|at most|at least|within|is denied|is allowed)\b|^\|(?!\s*-)", re.I)


def ranges(src):
    out = []
    for s in src or []:
        m = re.match(r"(.+?):(\d+)(?:-(\d+))?$", s)
        if m:
            out.append((m.group(1), int(m.group(2)), int(m.group(3) or m.group(2))))
    return out


def main():
    cid = sys.argv[1]
    src = os.path.join(lib.ROOT, "extract", "src", cid + ".md")
    frag = os.path.join(lib.ROOT, "extract", "frag", cid + ".jsonl")
    cov = []
    if os.path.exists(frag):
        for line in open(frag):
            try:
                cov += ranges(json.loads(line).get("src"))
            except json.JSONDecodeError:
                pass
    lines = open(src).read().split("\n")[1:]
    unc = []
    norm = 0
    for l in lines:
        m = re.match(r"(.+?):(\d+)\| (.*)$", l)
        if not m:
            continue
        doc, n, body = m.group(1), int(m.group(2)), m.group(3)
        if body.startswith("#") or not body.strip() or not NORMATIVE.search(body):
            continue
        norm += 1
        if not any(d == doc and a <= n <= b for d, a, b in cov):
            unc.append(f"{doc}:{n}")
    rep = {"chunk": cid, "normative_lines": norm, "uncovered": len(unc), "coverage": round(1 - len(unc) / norm, 3) if norm else 1.0, "lines": unc}
    lib.dump(rep, os.path.join(lib.ROOT, "reports", "anchors", cid + ".json"))
    print(json.dumps({k: rep[k] for k in ("chunk", "normative_lines", "uncovered", "coverage")}))


if __name__ == "__main__":
    main()

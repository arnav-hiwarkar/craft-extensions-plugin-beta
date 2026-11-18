import json, os, re

ROOT = os.environ.get("V3_ROOT") or os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
V2 = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), "v2")
SPEC = os.path.join(ROOT, "spec")
TEXT_FIELDS = ("title", "name", "stmt", "detail", "measure", "budget", "choice", "rationale", "def", "stimulus", "response", "sig", "evidence")
LIST_TEXT_FIELDS = ("props", "options", "contexts", "banned")


def schema():
    return json.load(open(os.path.join(SPEC, "schema.json")))


def shards():
    return json.load(open(os.path.join(SPEC, "shards.json")))


def kind_of(i):
    return i.split("-", 1)[0]


def load(spec_dir=SPEC):
    sc = schema()
    ents = {}
    problems = []
    for k, d in sc["kinds"].items():
        p = os.path.join(spec_dir, d["file"])
        if not os.path.exists(p):
            continue
        for n, line in enumerate(open(p), 1):
            line = line.strip()
            if not line:
                continue
            try:
                e = json.loads(line)
            except json.JSONDecodeError as x:
                problems.append(("C01", f"{d['file']}:{n}", f"bad json: {x}"))
                continue
            i = e.get("id", f"{d['file']}:{n}")
            if kind_of(i) != k:
                problems.append(("C01", i, f"id kind does not match file {d['file']}"))
            if i in ents:
                problems.append(("C01", i, "duplicate id"))
            e["_file"] = d["file"]
            ents[i] = e
    return ents, problems


def save(ents, spec_dir=SPEC):
    sc = schema()
    by = {d["file"]: [] for d in sc["kinds"].values()}
    for i in sorted(ents):
        e = {k: v for k, v in ents[i].items() if not k.startswith("_")}
        by[sc["kinds"][kind_of(i)]["file"]].append(e)
    os.makedirs(spec_dir, exist_ok=True)
    for f, rows in by.items():
        with open(os.path.join(spec_dir, f), "w") as o:
            for e in rows:
                o.write(json.dumps(e, ensure_ascii=False, sort_keys=False) + "\n")


def refs(e):
    sc = schema()
    k = kind_of(e["id"])
    out = []
    for f in sc["kinds"].get(k, {}).get("refs", {}):
        for t in e.get(f) or []:
            out.append((f, t))
    return out


def incoming(ents):
    inc = {i: [] for i in ents}
    for i, e in ents.items():
        for f, t in refs(e):
            if t in inc:
                inc[t].append((i, f))
    return inc


def text_of(e):
    parts = [str(e.get(f, "")) for f in TEXT_FIELDS]
    for f in LIST_TEXT_FIELDS:
        parts += [str(x) for x in e.get(f) or []]
    return "\n".join(p for p in parts if p)


def words(s):
    return len(re.findall(r"\S+", s))


def area_chars(ents):
    tot = {}
    for e in ents.values():
        tot[e.get("area", "?")] = tot.get(e.get("area", "?"), 0) + len(text_of(e))
    return tot


def neighbours(ents, ids):
    inc = incoming(ents)
    out = set()
    for i in ids:
        e = ents.get(i)
        if not e:
            continue
        out.update(t for _, t in refs(e) if t in ents)
        out.update(s for s, _ in inc.get(i, []))
    return out - set(ids)


def brief(e):
    b = {"id": e["id"], "title": e.get("title", "")}
    if e.get("stmt"):
        b["stmt"] = e["stmt"]
    return b


def dump(obj, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as o:
        json.dump(obj, o, indent=1, ensure_ascii=False)

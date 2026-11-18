import json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib, check


def main():
    path = sys.argv[1]
    sc = lib.schema()
    ents, _ = lib.load()
    before, _ = check.spec_checks(ents, sc, False)
    bad = []
    for n, line in enumerate(open(path), 1):
        line = line.strip()
        if not line:
            continue
        try:
            p = json.loads(line)
        except json.JSONDecodeError as x:
            bad.append(f"line {n}: {x}")
            continue
        i = p.get("id", "")
        if p.get("_delete"):
            ents.pop(i, None)
        elif p.get("_merge") and i in ents:
            ents[i].update({k: v for k, v in p.items() if k != "_merge"})
        else:
            ents[i] = {k: v for k, v in p.items() if k != "_merge"}
    after, warns = check.spec_checks(ents, sc, False)
    key = lambda x: (x[0], x[1], x[2])
    b = {key(x) for x in before}
    a = {key(x) for x in after}
    print(json.dumps({"bad_lines": bad, "errors_before": len(b), "errors_after": len(a), "fixed": len(b - a),
                      "introduced": [{"rule": r, "id": i, "msg": m} for r, i, m in sorted(a - b)][:40]}))


if __name__ == "__main__":
    main()

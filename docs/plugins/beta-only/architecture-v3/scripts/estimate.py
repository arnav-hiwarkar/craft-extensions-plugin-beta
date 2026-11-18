import json, math, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib

T_NEW = 2
T_REM = 3.0
Z80 = 1.2816


def chapman(n1, n2, m):
    N = (n1 + 1) * (n2 + 1) / (m + 1) - 1
    var = (n1 + 1) * (n2 + 1) * (n1 - m) * (n2 - m) / ((m + 1) ** 2 * (m + 2))
    return N, var


def estimate(rnd, history, weights):
    double = [s for s in rnd["shards"] if s.get("B") is not None]
    serious = lambda xs, acc: {g["key"] for g in xs or [] if g["key"] in acc}
    D = 0
    rem, var, f1, f2, n_det, w_double = 0.0, 0.0, 0, 0, 0, 0
    per = []
    for s in rnd["shards"]:
        acc = set(s.get("accepted") or [])
        A = serious(s.get("A"), acc)
        B = serious(s.get("B"), acc) if s.get("B") is not None else set()
        D += len(A | B)
        if s.get("B") is None:
            continue
        n1, n2, m = len(A), len(B), len(A & B)
        N, v = chapman(n1, n2, m)
        r = max(0.0, N - (n1 + n2 - m))
        rem += r
        var += v
        f1 += n1 + n2 - 2 * m
        f2 += m
        n_det += n1 + n2
        w_double += weights.get(s["shard"], 1)
        per.append({"shard": s["shard"], "n1": n1, "n2": n2, "m": m, "N_chapman": round(N, 2), "remaining": round(r, 2)})
    w_all = sum(weights.get(s["shard"], 1) for s in rnd["shards"]) + rnd.get("unreviewed_weight", 0)
    scale = w_all / w_double if w_double else float("inf")
    chao = f1 * (f1 - 1) / (2 * (f2 + 1))
    R = max(rem, chao) * scale if w_double else float("inf")
    R_hi = R + Z80 * math.sqrt(var) * scale if w_double else float("inf")
    p_new = f1 / n_det if n_det else 0.0
    Ds = [h["D"] for h in history] + [D]
    tail = float("inf")
    if len(Ds) >= 2 and Ds[0] > 0:
        q = (max(Ds[-1], 0.5) / Ds[0]) ** (1 / (len(Ds) - 1))
        tail = Ds[-1] * q / (1 - q) if q < 1 else float("inf")
    gates = {"check_errors": rnd.get("check_errors", 1) == 0, "trace_gaps": rnd.get("trace_gaps", 1) == 0, "proposed": rnd.get("proposed", 1) == 0}
    stop = all(gates.values()) and D <= T_NEW and min(R_hi, tail) <= T_REM
    fin = lambda x: None if x == float("inf") else round(x, 2)
    return {"round": rnd["round"], "D": D, "R_hat": fin(R), "R_hi80": fin(R_hi), "chao_remaining": round(chao, 2), "good_turing_p_new": round(p_new, 3),
            "tail_geometric": fin(tail), "scale": fin(scale), "gates": gates, "stop": stop, "per_shard": per}


def main():
    r = int(sys.argv[1])
    led = os.path.join(lib.ROOT, "ledger")
    rnd = json.load(open(os.path.join(led, f"r{r}.json")))
    hp = os.path.join(led, "rounds.jsonl")
    history = [json.loads(l) for l in open(hp) if l.strip()] if os.path.exists(hp) else []
    history = [h for h in history if h["round"] < r]
    ents, _ = lib.load()
    weights = {}
    for e in ents.values():
        weights[e.get("area")] = weights.get(e.get("area"), 0) + 1
    weights["integration"] = max(1, len(ents) // 10)
    rp = os.path.join(lib.ROOT, "reports")
    if os.path.exists(os.path.join(rp, "check.json")):
        rnd["check_errors"] = json.load(open(os.path.join(rp, "check.json")))["errors"]
    if os.path.exists(os.path.join(rp, "trace.json")):
        rnd["trace_gaps"] = json.load(open(os.path.join(rp, "trace.json")))["gaps"]
    rnd["proposed"] = sum(1 for e in ents.values() if e.get("status") == "proposed")
    res = estimate(rnd, history, weights)
    with open(hp, "a") as o:
        o.write(json.dumps(res) + "\n")
    with open(os.path.join(led, "gaps.jsonl"), "a") as o:
        for s in rnd["shards"]:
            acc = set(s.get("accepted") or [])
            minor = set(s.get("accepted_minor") or [])
            seen = {}
            for g in (s.get("A") or []) + (s.get("B") or []):
                seen.setdefault(g["key"], g)
            for k, g in seen.items():
                o.write(json.dumps({"round": r, "shard": s["shard"], "key": k, "sev": g.get("sev"), "claim": g.get("claim", "")[:200],
                                    "status": "fixed-serious" if k in acc else "fixed-minor" if k in minor else "rejected"}) + "\n")
    print(json.dumps(res))


if __name__ == "__main__":
    main()

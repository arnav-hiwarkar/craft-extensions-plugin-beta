import json, os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib

V2 = lib.V2
SOURCES = ["common.md", "security.md", "effectcraft.md", "photocraft.md", "figcraft.md", "final/decisions-v2.md", "final/flagships.md", "final/workstreams.md"]
TARGET = int(sys.argv[1]) if len(sys.argv) > 1 else 11000


def main():
    out = os.path.join(lib.ROOT, "extract", "src")
    os.makedirs(out, exist_ok=True)
    chunks = []
    for src in SOURCES:
        lines = open(os.path.join(V2, src)).read().split("\n")
        cuts = [n for n, l in enumerate(lines) if l.startswith("## ")] + [len(lines)]
        cuts = [0] + [c for c in cuts if c > 0]
        start, acc = 0, 0
        for a, b in zip(cuts, cuts[1:]):
            w = lib.words("\n".join(lines[a:b]))
            if acc and acc + w > TARGET:
                chunks.append((src, start, a))
                start, acc = a, 0
            acc += w
        chunks.append((src, start, len(lines)))
    listing = []
    for n, (src, a, b) in enumerate(chunks, 1):
        cid = f"x{n:02d}"
        name = src.replace("final/", "")
        body = open(os.path.join(V2, src)).read().split("\n")[a:b]
        with open(os.path.join(out, cid + ".md"), "w") as o:
            o.write(f"SOURCE {name} lines {a + 1}-{b}\n")
            for k, l in enumerate(body, a + 1):
                o.write(f"{name}:{k}| {l}\n")
        listing.append({"chunk": cid, "doc": name, "from": a + 1, "to": b, "words": lib.words("\n".join(body))})
    lib.dump(listing, os.path.join(lib.ROOT, "extract", "chunks.json"))
    print(json.dumps(listing))


if __name__ == "__main__":
    main()

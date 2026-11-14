#!/usr/bin/env python3
"""Validate active plugin guide structure without builds, network access or plugin execution."""
from pathlib import Path
from urllib.parse import unquote, urlsplit
import html
import json
import re
import sys

root = Path(__file__).resolve().parents[1]
guide_root = root / "docs/plugins"
names = [
    "README.md", "maps.md", "coverage.md", "formats-and-versioning.md",
    "native-protocol.md", "installation-and-testing.md", "author-guide.md",
    "how-it-works.md", "adapter-guide.md", "capabilities.md", "cli-reference.md",
    "store-and-distribution.md", "user-guide.md", "ui.md", "framework-integration.md",
    "maintainer-guide.md",
    "community-comparison-and-backlog.md", "community-nzo-review.md", "community-ecscripts-review.md",
    "community-bouncraft-review.md",
]
files = [guide_root / name for name in names] + [root / "STATUS.md", root / "TESTING.md"]
errors = []
counts = {"pages": 0, "links": 0, "json_examples": 0, "mermaid_diagrams": 0}
cache = {}

def fences(path, text):
    blocks = []
    opened = None
    body = []
    start = 0
    for line_number, line in enumerate(text.splitlines(), 1):
        match = re.match(r"^\s{0,3}(\x60{3,}|~{3,})(.*)$", line)
        if not match:
            if opened:
                body.append(line)
            continue
        marker, info = match.groups()
        if opened is None:
            opened = marker
            start = line_number
            language = info.strip().split()[0] if info.strip() else ""
            body = []
        elif marker[0] == opened[0] and len(marker) >= len(opened) and not info.strip():
            blocks.append((language, "\n".join(body), start))
            opened = None
    if opened:
        errors.append(f"{path.relative_to(root)}:{start}: unclosed code fence")
    return blocks

def without_code(text):
    return re.sub(r"(?ms)^\s{0,3}(\x60{3,}|~{3,})[^\n]*\n.*?^\s{0,3}\1\s*$", "", text)

def anchors(path):
    if path in cache:
        return cache[path]
    text = without_code(path.read_text())
    ids = set(re.findall(r'\bid=["\x27]([^"\x27]+)', text))
    duplicates = {}
    for heading in re.findall(r"(?m)^#{1,6}\s+(.+?)\s*#*\s*$", text):
        heading = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", heading)
        slug = html.unescape(heading).strip().lower()
        slug = re.sub(r"[^\w\- ]", "", slug).replace(" ", "-")
        n = duplicates.get(slug, 0)
        duplicates[slug] = n + 1
        ids.add(slug + (f"-{n}" if n else ""))
    cache[path] = ids
    return ids

for path in files:
    if not path.is_file():
        errors.append(f"Missing required page: {path}")
        continue
    counts["pages"] += 1
    text = path.read_text()
    prose = without_code(text)
    if len(re.findall(r"(?m)^# [^\n]+", prose)) != 1:
        errors.append(f"{path.relative_to(root)}: expected one top-level title")
    for language, body, line in fences(path, text):
        if language == "json":
            try:
                json.loads(body)
                counts["json_examples"] += 1
            except ValueError as error:
                errors.append(f"{path.relative_to(root)}:{line}: invalid JSON example: {error}")
        if language == "mermaid":
            counts["mermaid_diagrams"] += 1
            first = body.strip().splitlines()[0] if body.strip() else ""
            if not first or first.split()[0] not in {"mindmap", "flowchart", "sequenceDiagram", "stateDiagram-v2"}:
                errors.append(f"{path.relative_to(root)}:{line}: unsupported or empty Mermaid diagram header")
    for match in re.finditer(r"(?<!!)\[[^\]\n]+\]\(([^)\n]+)\)", prose):
        target = match.group(1).strip()
        if target.startswith("<") and ">" in target:
            target = target[1:target.index(">")]
        else:
            target = target.split()[0]
        parsed = urlsplit(target)
        if parsed.scheme or parsed.netloc:
            continue
        counts["links"] += 1
        link_path = path if not parsed.path else (path.parent / unquote(parsed.path)).resolve()
        if not link_path.exists():
            errors.append(f"{path.relative_to(root)}: missing link target {target}")
        elif parsed.fragment and link_path.suffix == ".md" and unquote(parsed.fragment) not in anchors(link_path):
            errors.append(f"{path.relative_to(root)}: missing heading anchor {target}")

atlas = guide_root / "maps/overview.html"
if not atlas.is_file():
    errors.append("Missing offline interactive atlas")
else:
    markup = atlas.read_text()
    if re.search(r'<(?:script|link)[^>]+(?:src|href)=["\x27]https?://', markup):
        errors.append("Interactive atlas must not require remote scripts or styles")
    for target in re.findall(r'data-doc="([^"]+)"', markup):
        if not (guide_root / target).is_file():
            errors.append(f"Interactive atlas: missing guide {target}")

if counts["mermaid_diagrams"] < 11:
    errors.append("Expected ten workflow maps and the native sequence diagram")
report = {"ok": not errors, **counts, "errors": errors}
if "--json" in sys.argv:
    print(json.dumps(report, indent=2))
else:
    print(("PASS" if report["ok"] else "FAIL") + ": " + ", ".join(f"{k}={v}" for k, v in counts.items()))
    for error in errors:
        print("- " + error)
    print("Checks cover links, JSON, diagram fences/headers and offline dependencies, not full Mermaid rendering or runtime behavior.")
sys.exit(0 if report["ok"] else 1)

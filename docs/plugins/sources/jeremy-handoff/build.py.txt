"""Render and validate the extensions handoff page (index.html) from plan_data and plan_tasks.

Run: python build.py   Exits non-zero on any validation error and then writes nothing.
"""
from __future__ import annotations

import html
import re
from html.parser import HTMLParser
from pathlib import Path

import plan_data as D
from plan_tasks import PHASES, all_tasks

ROOT = Path(__file__).resolve().parent
OUT = ROOT / "index.html"


def e(text: str) -> str:
    return html.escape(text, quote=True)


def table(caption: str, heads: list[str], rows: list[list[str]], row_header: bool = True) -> str:
    out = ["<div class=\"scroll\"><table>", "<caption>" + e(caption) + "</caption>", "<thead><tr>"]
    out += ["<th scope=\"col\">" + e(h) + "</th>" for h in heads]
    out.append("</tr></thead><tbody>")
    for row in rows:
        cells = []
        for i, cell in enumerate(row):
            if i == 0 and row_header:
                cells.append(cell)  # caller supplies a ready-made th
            else:
                cells.append("<td>" + e(cell) + "</td>")
        out.append("<tr>" + "".join(cells) + "</tr>")
    out.append("</tbody></table></div>")
    return "\n".join(out)


def th(text: str, ident: str | None = None) -> str:
    attr = f" id=\"{e(ident)}\"" if ident else ""
    return f"<th scope=\"row\"{attr}>" + e(text) + "</th>"


def section(ident: str, title: str, body: str) -> str:
    return (f"<section id=\"{ident}\" aria-labelledby=\"{ident}-h\">\n<h2 id=\"{ident}-h\">{e(title)}</h2>\n"
            f"{body}\n</section>")


def ul(items: list[str]) -> str:
    return "<ul>" + "".join("<li>" + e(i) + "</li>" for i in items) + "</ul>"


def build_body() -> str:
    parts = []
    parts.append(section("status", "Status and decisions",
        "<div class=\"callout\"><p>" + e(D.DIRECTION) + "</p></div>\n"
        + table("Decisions. Only Jeremy's words are Decided.", ["ID", "Question", "Recommendation", "If not", "State"],
                [[th(d[0], "dec-" + d[0]), d[1], d[2], d[3], d[4]] for d in D.DECISIONS])))
    parts.append(section("apps", "Per-app matrix (draft for owners to confirm)",
        "<p>Built from the organisation's repo list on 16/10/2026, not from the folders on this PC. "
        "The launcher's own app list omits FigCraft and AniCraft. Plug-in kinds are guesses from each repo's description.</p>\n"
        + table("The 14 Craft apps and what an extension could mean for each",
                ["App", "What it is", "Likely plug-in kind", "Notes"],
                [[th(a[1], a[0]), a[2], a[3], a[4]] for a in D.APPS])))
    parts.append(section("tiers", "Trust tiers: internal on, third party built but off",
        "<p>Third party is designed in from the first schema so it never needs a rewrite, but nothing outside the storytold organisation can be installed until every gate in phase 3 passes.</p>\n"
        + table("Tiers", ["Tier", "Who", "How it is checked", "State"], [[th(t[0]), t[1], t[2], t[3]] for t in D.TIERS])))
    parts.append(section("requirements", "Requirements the shared layer must meet",
        table("Requirements", ["ID", "Requirement"], [[th(r[0], "req-" + r[0]), r[1]] for r in sorted(D.REQUIREMENTS, key=lambda r: int(r[0][1:]))])))
    opts = table("Options for the shared store", ["Option", "For", "Against"],
                 [[th(o[0]), o[1], o[2]] for o in D.STORE_OPTIONS])
    parts.append(section("store", "Home of the shared store", opts + "\n<p><strong>" + e(D.STORE_RECOMMENDATION) + "</strong></p>"))
    parts.append(section("owners", "Who builds what",
        table("Ownership", ["Piece", "Owner", "Note"], [[th(o[0]), o[1], o[2]] for o in D.OWNERSHIP])
        + "\n<h3 id=\"checklist-h\">Adapter checklist every app follows</h3>\n<ol>"
        + "".join("<li>" + e(i) + "</li>" for i in D.ADAPTER_CHECKLIST) + "</ol>"))
    task_html = ["<p>No task is started. Rows marked Blocked wait on the named decision or work.</p>"]
    for pid, title, rows in PHASES:
        task_html.append(f"<h3 id=\"{pid}-h\">{e(title)}</h3>")
        task_html.append(table(title, ["ID", "Task", "Depends on", "Done when", "Status"],
                               [[th(r[0], "task-" + r[0]), r[1], r[2] or "none", r[3], r[4]] for r in rows]))
    task_html.append("<h3 id=\"appx-h\">Per-app adapter rows (task r4)</h3><p>Each app follows the adapter checklist above in its own repo. "
                     "Listed in the per-app matrix; none is started here.</p>")
    parts.append(section("tasks", "Work list", "\n".join(task_html)))
    parts.append(section("rules", "Rules for this work", ul(D.RULES)))
    parts.append(section("questions", "Open questions", ul(D.QUESTIONS)))
    parts.append(section("resume", "Start a new session",
        "<ol><li>Read <a href=\"NEXT.md\">NEXT.md</a>, then the <a href=\"design-notes.md\">design notes</a>.</li>"
        "<li>Verify against the live repos before acting: git fetch, gh pr list, git status in the launcher and PhotoCraft trees. "
        "Another session may have moved them.</li>"
        "<li>If scope is still open, ask Jeremy once which decision to settle first (<a href=\"#dec-d4\">d4</a> blocks the internal catalogue; <a href=\"#dec-d8\">d8</a> gates the signature work). "
        "If he has chosen, build without further check-ins.</li></ol>"))
    refs = "<ul>" + "".join(f"<li><a href=\"{e(p)}\">{e(t)}</a></li>" for t, p in D.REFERENCES) + "</ul>"
    parts.append(section("refs", "References", refs))
    return "\n".join(parts)


CSS = """:root { color-scheme: light dark; font-family: system-ui, sans-serif; line-height: 1.55; }
body { max-width: 76rem; margin: 0 auto; padding: 1rem 1.5rem 4rem; }
header, section, footer { margin-block: 1.7rem; }
nav ul { display: flex; flex-wrap: wrap; gap: .4rem 1.3rem; padding-left: 1.2rem; }
.meta { opacity: .75; }
.callout { border-left: .25rem solid currentColor; padding: .3rem 1rem; }
.scroll { overflow-x: auto; margin-block: 1rem; }
table { border-collapse: collapse; width: 100%; min-width: 45rem; }
caption { text-align: left; font-weight: 650; padding-bottom: .4rem; }
th, td { text-align: left; vertical-align: top; border-bottom: 1px solid currentColor; padding: .55rem .7rem; }
th { font-weight: 650; }
.skip { position: absolute; top: -10rem; }
.skip:focus { top: .5rem; padding: .3rem; background: Canvas; color: CanvasText; }
@media print { :root { color-scheme: light; } body { max-width: none; } nav { display: none; } }"""

NAV = [("status", "Decisions"), ("tiers", "Trust tiers"), ("apps", "Apps"), ("requirements", "Requirements"), ("store", "Store"),
       ("owners", "Ownership"), ("tasks", "Work list"), ("rules", "Rules"), ("questions", "Questions"),
       ("resume", "Resume"), ("refs", "References")]

TEMPLATE = """<!doctype html>
<html lang="en-GB">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Craft suite extensions | Plan and handoff</title>
<style>
@@CSS@@
</style>
</head>
<body>
<a class="skip" href="#main">Skip to content</a>
<header>
<p class="meta">Craft suite, shared extensions browser. Updated <time datetime="@@ISO@@">@@UK@@</time>.</p>
<h1>Themes, plug-ins and extensions: plan and handoff</h1>
<p>One place in ArtCraft Launcher to find, install and update themes, plug-ins and resources, shared by every Craft app and reachable from each of them.</p>
</header>
<nav aria-label="Contents"><ul>@@NAV@@</ul></nav>
<main id="main">
@@BODY@@
</main>
<footer><p>Local plan. No commit, push, post or install is authorised by this page. Generated by build.py from plan_data.py and plan_tasks.py; do not edit index.html alone.</p></footer>
</body>
</html>
"""


class Structure(HTMLParser):
    VOID = {"meta", "link", "br", "hr", "img", "input"}

    def __init__(self) -> None:
        super().__init__()
        self.stack: list[str] = []
        self.ids: set[str] = set()
        self.links: list[str] = []
        self.labels: list[str] = []
        self.counts: dict[str, int] = {}
        self.errors: list[str] = []
        self.tables: list[dict[str, int]] = []
        self.last_heading = 1

    def handle_starttag(self, tag, attrs):
        attr = dict(attrs)
        self.counts[tag] = self.counts.get(tag, 0) + 1
        if ident := attr.get("id"):
            if ident in self.ids:
                self.errors.append("duplicate id: " + ident)
            self.ids.add(ident)
        if tag == "a" and (href := attr.get("href")):
            self.links.append(href)
        if label := attr.get("aria-labelledby"):
            self.labels.extend(label.split())
        if re.fullmatch(r"h[1-6]", tag):
            level = int(tag[1])
            if level > self.last_heading + 1:
                self.errors.append("heading skip to " + tag)
            self.last_heading = level
        if tag == "table":
            self.tables.append({"caption": 0, "cols": 0})
        if tag == "caption" and self.tables:
            self.tables[-1]["caption"] += 1
        if tag == "th":
            if attr.get("scope") not in {"col", "row"}:
                self.errors.append("th without scope")
            if attr.get("scope") == "col" and self.tables:
                self.tables[-1]["cols"] += 1
        if tag not in self.VOID:
            self.stack.append(tag)

    def handle_endtag(self, tag):
        if tag in self.VOID:
            return
        if not self.stack or self.stack[-1] != tag:
            self.errors.append("mismatched close: " + tag)
        else:
            self.stack.pop()


def validate(page: str) -> list[str]:
    p = Structure()
    p.feed(page)
    errors = list(p.errors)
    if p.stack:
        errors.append("unclosed tags")
    for tag in ("html", "header", "main", "footer", "nav", "h1"):
        if p.counts.get(tag) != 1:
            errors.append("landmark count wrong: " + tag)
    if p.counts.get("script"):
        errors.append("script not allowed")
    if any(t["caption"] != 1 or t["cols"] == 0 for t in p.tables):
        errors.append("table needs a caption and column headers")
    errors += ["broken aria-labelledby: " + i for i in p.labels if i not in p.ids]
    for href in p.links:
        if href.startswith("#"):
            if href[1:] not in p.ids:
                errors.append("broken anchor: " + href)
        elif not href.startswith("https://") and not (ROOT / href).resolve().is_file():
            errors.append("missing local link: " + href)
    if "\u2014" in page or "\u2013" in page:
        errors.append("em or en dash")
    tasks = [t[2] for t in all_tasks()]
    if not tasks or len(set(tasks)) != len(tasks):
        errors.append("duplicate or missing task ids")
    known = set(tasks) | {d[0] for d in D.DECISIONS}
    for t in all_tasks():
        for dep in [x.strip() for x in t[4].split(",") if x.strip()]:
            if dep not in known:
                errors.append(f"task {t[2]} depends on unknown {dep}")
    errors += ["task id missing from page: " + t for t in tasks if f"id=\"task-{t}\"" not in page]
    errors += ["app missing from page: " + a[1] for a in D.APPS if a[1] not in page]
    decided = {d[0] for d in D.DECISIONS if d[4].startswith("Decided")}
    if decided != {"d0", "d3", "d5", "d7"}:
        errors.append("Decided rows must be exactly the ones Jeremy has decided (d0, d3, d5, d7); found " + ", ".join(sorted(decided)))
    return errors


def main() -> None:
    nav = "".join(f"<li><a href=\"#{i}\">{e(t)}</a></li>" for i, t in NAV)
    page = (TEMPLATE.replace("@@CSS@@", CSS).replace("@@ISO@@", D.UPDATED_ISO).replace("@@UK@@", D.UPDATED_UK)
            .replace("@@NAV@@", nav).replace("@@BODY@@", build_body()))
    errors = validate(page)
    if errors:
        raise SystemExit("HTML validation failed: " + "; ".join(errors))
    OUT.write_text(page, encoding="utf-8")
    n = sum(1 for _ in all_tasks())
    print(f"OK: {n} tasks, {len(D.APPS)} apps, {len(D.DECISIONS)} decisions; wrote {OUT.name}")


if __name__ == "__main__":
    main()

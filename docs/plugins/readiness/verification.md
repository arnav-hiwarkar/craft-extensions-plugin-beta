# Readiness verification

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Recorded 2026-11-08. The source investigation and [root reconciliation](assessment-and-first-work.md) are complete for the inspected committed snapshots. Documentation validation is separate from application acceptance. No app code, branch, dependency, CI, external issue/repository, key, PR or release was changed. No Rust build/test, app launch, benchmark, fuzz run, package execution or sandbox/transport exploit test ran.

## Checks

The read-only validator below checks all draft, research and readiness Markdown plus the main index and accepted-decision record. It verifies local file/heading links and whitespace, all twenty-seven aspect rows in the readiness roadmap and all sixteen inventory apps in its app table. It does not validate remote link liveness or execute application features.

The root also re-read the source scopes listed in [the evidence index](evidence-index.md#root-verification-and-reconciliation), checked local committed revisions against the inventory, and reviewed the work packages for unsupported conclusions and accidental new policy choices.

Run this Python block from the repository root to repeat the documentation validation:

```python
from pathlib import Path
import json
import re
import subprocess

base = Path('docs/plugins')
files = sorted(p for part in ('draft', 'research', 'readiness') for p in (base / part).glob('*.md'))
files += [base / 'README.md', Path('docs/plugin-functional-decisions.md')]
errors, links = [], 0

def headings(path):
    counts, result = {}, set()
    text = re.sub(r'```[^\n]*\n.*?```', '', path.read_text(), flags=re.S)
    for heading in re.findall(r'^#{1,6}\s+(.+)$', text, re.M):
        heading = re.sub(r'[`*_]', '', heading).lower()
        slug = re.sub(r'[^\w\- ]', '', heading).replace(' ', '-')
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        result.add(slug if not count else f'{slug}-{count}')
    return result

for path in files:
    text = path.read_text()
    for line, value in enumerate(text.splitlines(), 1):
        if value.rstrip() != value:
            errors.append(f'{path}:{line}: trailing whitespace')
    text = re.sub(r'```[^\n]*\n.*?```', '', text, flags=re.S)
    for target in re.findall(r'\[[^\]\n]+\]\(([^)]+)\)', text):
        if re.match(r'\w+://', target):
            continue
        links += 1
        name, _, fragment = target.partition('#')
        dest = path.parent / name if name else path
        if not dest.exists():
            errors.append(f'{path}: missing {target}')
        elif fragment and dest.suffix == '.md' and fragment not in headings(dest):
            errors.append(f'{path}: missing fragment {target}')
    check = subprocess.run(['git', 'diff', '--no-index', '--check', '/dev/null', str(path)],
                           capture_output=True, text=True)
    if check.returncode not in (0, 1) or check.stdout or check.stderr:
        errors.append(f'{path}: whitespace diagnostics: {check.stdout}{check.stderr}')

aspects = [x for x in re.findall(r'^\| ([^|]+?) \|', (base / 'aspect-register.md').read_text(), re.M)
           if x not in ('Aspect', '---')]
roadmap = (base / 'readiness/system-gaps-and-roadmap.md').read_text()
for aspect in aspects:
    if f'| {aspect} |' not in roadmap:
        errors.append(f'Aspect missing: {aspect}')
apps = json.loads(Path('docs/plugin-functional-inventory.json').read_text())['apps']
for app in apps:
    if f'| {app["app"]} |' not in roadmap.lower():
        errors.append(f'App missing: {app["app"]}')
print(f'{len(files)} files, {links} local links, {len(aspects)} aspects, {len(apps)} apps')
print('\n'.join(errors) if errors else 'PASS')
raise SystemExit(bool(errors))
```

Completed result: 34 Markdown files, 330 local links, all 27 aspects and all 16 apps passed. All 34 files also produced no whitespace diagnostics. These counts describe this readiness completion, not application runtime verification.

## Limits

The worker notes and coordinator checked local committed main snapshots. They did not fetch current upstream branches, inspect dirty work or observe runtime behavior. The root confirmed all sixteen local HEADs match their recorded inventory revisions. Historical test counts in Jeremy's handoff remain attributed history. Remaining untraced areas are listed in the evidence index; no package/runtime correctness, legal policy or security certification follows from documentation coverage.

Future acceptance remains scoped app/store tests, candidate UI/CLI behavior, authenticated catalogue/key recovery fixtures, effective release profiles, platform checks and independent security review. The no-code investigation did not start those changes.

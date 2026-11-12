# Draft evidence and verification

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Recorded 2026-11-08. This is a design/documentation task. No application implementation, package execution, build, test, benchmark, CI run, release or external publication occurred. No new repository, dependency, signing key or credential was created.

## Evidence used

- [Accepted direction](decisions.md) records the user's latest choices and distinguishes draft engineering defaults. The user's later simplicity instruction guides the reconciled architecture.
- [Prior research verification](../research/verification.md) records app/store revisions, the limited source checks, primary specifications and the fifteen inventory hashes checked by the earlier app delegate.
- The new Opus delegate read craft-extensions local main at abbreviated revision `a6c5fae` and compared relevant store/state definitions with handover `ffd8512e`. Its [note](opus-review.md#7-evidence-and-limits) states exact scope and source-derived conclusions. The primary did not repeat that complete source review or treat it as runtime proof.
- The primary read the workspace `craft-keymap` dependency entry and [its public root](https://github.com/storytold/craft-libs/blob/fd5e2feff6ea25bed371a99cc732b3968d621/crates/keymap/src/lib.rs) to verify existing shortcut/command/scope/override support before naming it as a reuse candidate. No integration ran.
- The primary consulted official [GitHub releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases), [release assets](https://docs.github.com/en/rest/releases/assets), [rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api), [GitHub supply-chain documentation](https://github.com/github/docs/blob/main/content/code-security/concepts/supply-chain-security/supply-chain-security.md), [TUF](https://theupdateframework.github.io/specification/latest/) and [WIT](https://component-model.bytecodealliance.org/design/wit.html). No TUF implementation/maintenance/licence or runtime migration was selected.
- The primary retrieved official [VS Code contribution points](https://code.visualstudio.com/api/references/contribution-points), [Figma manifest](https://developers.figma.com/docs/plugins/manifest/) and [Zed extension development](https://zed.dev/docs/extensions/developing-extensions) to support specific architectural patterns. Broader marketplace/Android/Flathub analogies in the independent note remain unverified and are not relied on.

Provider settings returned by the requested new design delegation were `claude-opus-5-5`, `effort: medium`, through `claudeAgent`, in the shared bound checkout. It wrote only its assigned note and reported no grandchildren. The primary reconciled the note without requesting another review or substituting models/settings.

## Documentation validation

The rerunnable check passed on 22 Markdown files, including all nine draft files, with 236 local links, all twenty-seven original aspect rows and all sixteen inventory apps. It validates local file/heading links and trailing whitespace and checks the new acceptance/work-package tables. It reads files only. Run the Python block from the repository root.

An additional `git diff --no-index --check` comparison against an empty file produced no whitespace diagnostics for these 22 files. Exit status 1 means content differs and is not itself a whitespace error. No Rust/application checks were run.

```python
from pathlib import Path
import json
import re

base = Path('docs/plugins')
files = sorted((base / 'draft').glob('*.md')) + sorted((base / 'research').glob('*.md'))
files += [base / 'README.md', Path('docs/plugin-functional-decisions.md')]
errors, links = [], 0

def anchors(path):
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
    for number, line in enumerate(text.splitlines(), 1):
        if line.rstrip() != line:
            errors.append(f'{path}:{number}: trailing whitespace')
    text = re.sub(r'```[^\n]*\n.*?```', '', text, flags=re.S)
    for target in re.findall(r'\[[^\]\n]+\]\(([^)]+)\)', text):
        if re.match(r'\w+://', target):
            continue
        links += 1
        name, _, fragment = target.partition('#')
        dest = path.parent / name if name else path
        if not dest.exists():
            errors.append(f'{path}: missing {target}')
        elif fragment and dest.suffix == '.md' and fragment not in anchors(dest):
            errors.append(f'{path}: missing fragment {target}')

aspects = [x for x in re.findall(r'^\| ([^|]+?) \|', (base / 'aspect-register.md').read_text(), re.M)
           if x not in ('Aspect', '---')]
coverage = (base / 'draft/delivery-and-acceptance.md').read_text()
for aspect in aspects:
    if f'| {aspect} |' not in coverage:
        errors.append(f'Aspect missing: {aspect}')
apps = json.loads(Path('docs/plugin-functional-inventory.json').read_text())['apps']
matrix = (base / 'draft/shared-operations-and-apps.md').read_text().lower()
for app in apps:
    if f'| {app["app"]} |' not in matrix:
        errors.append(f'App missing: {app["app"]}')
print(f'{len(files)} files, {links} local links, {len(aspects)} aspects, {len(apps)} apps')
print('\n'.join(errors) if errors else 'PASS')
raise SystemExit(bool(errors))
```

## Pending application evidence

The [delivery/coverage reference](delivery-and-acceptance.md) labels all executable, visual, adversarial, performance, clean-checkout, release-feature, platform and independent security cases as future implementation/CI acceptance. Source reading does not prove the proposed dispatcher, grants, SDK, catalogue signatures, reader leases or sandbox. No “complete design” claim means those mechanisms already work.

No external links were exhaustively checked for liveness; official pages listed above were retrieved for the specific claims described. GitHub numeric identity permanence, future support/deprecation windows and operator staffing are not independently verified promises. Exact licence policy, concrete tools and numerical budgets remain explicit choices rather than invented agreements.

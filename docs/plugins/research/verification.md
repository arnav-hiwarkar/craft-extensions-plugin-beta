# Evidence and verification

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Recorded 2026-11-08. This reference distinguishes inspected source, primary external specifications, completed documentation checks and future application acceptance. It does not certify the design's security or predict performance.

## Evidence anchors

The baseline's [source and verification record](../source-and-verification-record.md) and [app inventory](../../plugin-functional-inventory.json) name the inspected revisions. The study did not fetch or update app checkouts. Delegate notes provide their own exact files and “not traced” limits.

| Evidence | Scope of this study |
|---|---|
| craft-extensions `ffd8512e5a3755637f58cf31cfcf2fdb09cfb943` | Opus read core/store, policy, plan and sample packages. Primary reconciler read Reader, State, Store and palette limit definitions. These confirm missing continuity data, install-only admission, update preference/deletion behavior, and state format limitations |
| craft-launcher `6ede77f93fd0d3eaaad43d430f94d703d3564a51` | Delegates inspected source/store UI calls. No working authenticated catalogue path or candidate package was demonstrated |
| PhotoCraft `ecb056cc742848d78f9076893a6b3343e6a75010` | Theme branch source. Reconciler read watcher setup/poll path; delegate inspected mapping and fallback. No visual verification or integration run |
| PhotoCraft `655295b2a2c400bded0640f5d97367000a057b2e` | Filter branch source. Reconciler confirmed smart-filter lookup/failure returns None in plugin_cmds.rs. No reproduction or export-status claim |
| FigCraft `0e86f3597a7443b15c93ec3ab2091202f8802c95` | App delegate inspected plugin/headless/CLI paths. Reconciler confirmed headless clears UI requests after execution. No runtime output/exit-code check |
| EffectCraft `ce9eaec65015a5508c72af0e2351cd02612c9450` | App delegate inspected WASM/helper/host/CLI/project paths; author delegate inspected publishing guide. Main synthesis reuses their source evidence and baseline rather than claiming a new render trace |
| FilmCraft `32749e34f611735460e9773335345db026b7b037` | Delegates inspected theme/preferences and CLI/job entry paths. Processing-host absence is scoped to their reads, not a proof about every file |
| Remaining inventory apps | Reused baseline named-revision survey. Adapter rows describe proposed obligations; no broader new runtime survey |
| Complete Jeremy text and discussion archive | Preserved local baseline. Reported historical tests remain attributed, not rerun results |

The app delegate reports recomputing fifteen inventory source hashes and finding matches. This is the delegate's recorded check, not fifteen checks independently repeated by the reconciler. The root worktree remains at `fd5e2feff6ea25bed371a99cc732b3968d621505`; no merge/rebase was performed.

## Primary external specifications consulted

The primary reconciler retrieved the following official sources during this study. Links support general boundary claims, not a selected dependency/runtime implementation.

- [TUF specification](https://theupdateframework.github.io/specification/latest/), retrieved version 1.0.37, last modified 8 October 2026. Used for authenticated root rotation, metadata freshness, rollback and target verification. Catalogue governance and plugin safety are separate policies.
- [Wasmtime security](https://docs.wasmtime.dev/security.html). Used for the distinction between WASM isolation and host embedding responsibilities. Existing app engine selection is not changed by this citation.
- [WebGPU security considerations](https://www.w3.org/TR/webgpu/#security-considerations), retrieved draft dated 15 September 2026. Used for driver/availability risk; native and GPU containment still need platform evidence.

## Completed documentation checks

The bounded documentation validator passed on eleven Markdown files, 114 local links, all twenty-seven aspect rows and all sixteen inventory app names. It checks local file/fragment targets and trailing whitespace. It does not check remote link availability or rendered UI. Baseline sources are read-only inputs. A documentation check cannot satisfy a package/runtime acceptance case.

`git diff --no-index --check` against an empty file also produced no whitespace diagnostics for all eleven new Markdown files. Its exit status 1 for new content was treated as a normal difference, not a whitespace failure.

Run this Python snippet from the repository root to repeat the documentation checks. It reads files and prints results without changing them.

```python
from pathlib import Path
import json
import re

research = Path('docs/plugins/research')
errors = []
links = 0

def anchors(path):
    seen, result = {}, set()
    for title in re.findall(r'^#{1,6}\s+(.+)$', path.read_text(), re.M):
        title = re.sub(r'[`*_]', '', title).lower()
        base = re.sub(r'[^\w\- ]', '', title).replace(' ', '-')
        count = seen.get(base, 0)
        seen[base] = count + 1
        result.add(base if count == 0 else f'{base}-{count}')
    return result

files = list(research.glob('*.md'))
for path in files:
    text = path.read_text()
    for number, line in enumerate(text.splitlines(), 1):
        if line.rstrip() != line:
            errors.append(f'{path}:{number}: trailing whitespace')
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

register = Path('docs/plugins/aspect-register.md').read_text()
aspects = [name for name in re.findall(r'^\| ([^|]+?) \|', register, re.M)
           if name not in ('Aspect', '---')]
coverage = (research / 'coverage.md').read_text()
for name in aspects:
    if f'| {name} |' not in coverage:
        errors.append(f'Aspect missing: {name}')
apps = json.loads(Path('docs/plugin-functional-inventory.json').read_text())['apps']
matrix = (research / 'app-adapters.md').read_text().lower()
for app in apps:
    if f'| {app["app"]} |' not in matrix:
        errors.append(f'App missing: {app["app"]}')
print(f'{len(files)} files, {links} local links, {len(aspects)} aspects, {len(apps)} apps')
print('\n'.join(errors) if errors else 'PASS')
raise SystemExit(bool(errors))
```

## Checks still pending

No Rust build, cargo test, clippy, fmt, app UI, package execution, fuzzing, benchmark, OS sandbox test, clean-checkout dependency build, candidate package, release-feature test or CI run occurred. No dependency licence selection or independent security certification occurred.

The [coverage reference](coverage.md) lists proposed acceptance per aspect. The [workflow reference](lifecycle.md) gives failure/retry cases. The [validation order](decisions.md#proposed-validation-order) places cross-repo, candidate visual checks, packaging, full required repository checks and release-profile/platform evidence in future approved implementation/CI work. First-party acceptance and community enablement remain distinct.

The unreadable ChatGPT share and unavailable original Claude chart layout were not reviewed. The complete supplied artifact text was available and used. No request for the missing context was needed.

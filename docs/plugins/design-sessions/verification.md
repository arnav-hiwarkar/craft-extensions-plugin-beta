# Discussion handoff verification

> **Status (2026-11-10):** history and evidence. The current design is [the Craft plugin architecture](../architecture.md); where this document disagrees with it, the architecture wins.

Prepared 2026-11-08. This check covers document continuity and navigation, not application behavior. No implementation, build, test, app launch, new app checkout, external publication or community gate change occurs in this handoff.

The validator reuses the existing read-only readiness check, expands its Markdown/link/whitespace scope to these guides, checks eighteen topic IDs, forty scenario IDs, all twenty-seven original aspects and all sixteen app rows, and verifies Jeremy's eleven archived source files against their recorded byte lengths and SHA-256 values. It does not execute archived scripts or unpack/install samples.

Run this Python block from the repository root:

```python
from pathlib import Path
import hashlib
import json
import re

previous = Path('docs/plugins/readiness/verification.md')
source = re.search(r'```python\n(.*?)\n```', previous.read_text(), re.S).group(1)
old = "('draft', 'research', 'readiness')"
assert source.count(old) == 1
source = source.replace(old, "('draft', 'research', 'readiness', 'design-sessions')")
scope = {}
try:
    exec(compile(source, str(previous), 'exec'), scope)
except SystemExit as result:
    if result.code:
        raise

base = Path('docs/plugins/design-sessions')
topics = re.findall(r'^\| (T\d{2}) \|', (base / 'topics.md').read_text(), re.M)
scenarios = re.findall(r'^\| (S\d{2}) \|', (base / 'scenarios-and-workflows.md').read_text(), re.M)
assert topics == [f'T{x:02}' for x in range(1, 19)], topics
assert scenarios == [f'S{x:02}' for x in range(1, 41)], scenarios
coverage = (base / 'scenarios-and-workflows.md').read_text()
for aspect in scope['aspects']:
    assert f'| {aspect} |' in coverage, aspect
apps = (base / 'app-families-and-sdk.md').read_text().lower()
for app in scope['apps']:
    assert f'| {app["app"]} |' in apps, app['app']

archive = Path('docs/plugins/sources/jeremy-handoff')
snapshot = json.loads((archive / 'snapshot.json').read_text())
for entry in snapshot['files']:
    data = (archive / entry['snapshot_path']).read_bytes()
    assert len(data) == entry['bytes'], entry['snapshot_path']
    assert hashlib.sha256(data).hexdigest() == entry['sha256'], entry['snapshot_path']
print(f'PASS: {len(topics)} topics, {len(scenarios)} scenarios, '
      f'{len(scope["aspects"])} aspect mappings, {len(scope["apps"])} app mappings, '
      f'{len(snapshot["files"])} archived source hashes')
```

The new top-level thread is launched only after documentation checks pass. Its actual model/effort and existing-worktree binding are verified through T3 tools. A successful thread launch is conversation setup and does not establish that any design mechanism has been implemented.

## Completed handoff

Documentation checks passed for 41 Markdown files, 433 local links, 18 topic IDs, 40 scenario IDs, all 27 aspect mappings and all 16 app mappings. All 11 archived Jeremy source files matched their recorded sizes and SHA-256 hashes. These checks do not verify runtime behavior.

The separate top-level conversation was created and read back through T3. Actual configuration is `claude-opus-5-5`, `effort: medium`, provider `claudeAgent`, bound to the existing `feat-artcraft-plugin-system` worktree. It starts topic T01 and waits for the user's response; no new implementation checkout was created.

Conversation: <t3-thread://v2/5df2eb29-79b9-431d-99a9-a501614c1c2f/mcp:launch:683130456b987226e7b0d2ce0a39587210ae827b1399b791407d7cc468c1c4af>

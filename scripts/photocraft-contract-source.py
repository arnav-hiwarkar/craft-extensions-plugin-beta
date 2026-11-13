#!/usr/bin/env python3
"""Refresh the PhotoCraft reference from literal Rust metadata without compiling it.

Run from any directory; optional first argument selects the sibling PhotoCraft checkout.
This deliberately accepts only the current literal metadata shape and fails on drift.
"""
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
source = Path(sys.argv[1]) if len(sys.argv) > 1 else root.parent / 'photocraft'
s = (source / 'crates/plugins/src/contract.rs').read_text()
literal = r'r#"[\s\S]*?"#|"(?:\\.|[^"\\])*"'
def value(token):
    return token[3:-2] if token.startswith('r#') else json.loads(token)
def strings(text):
    return [value(m.group()) for m in re.finditer(literal, text)]
ops = s.split('const OPS:', 1)[1].split('\n];', 1)[0]
pattern = re.compile(r'\(\s*(' + literal + r'),\s*OpKind::(\w+),\s*\w+,\s*(' + literal + r'),\s*(' + literal + r'),\s*(' + literal + r'),\s*(' + literal + r')\s*,?\s*\)')
rows = list(pattern.finditer(ops))
assert len(rows) == len(re.findall('OpKind::', ops)), 'Unrecognized operation metadata'
level = re.search(r'SESSION_LEVEL:.*?= \((\d+), (\d+)\)', s)
assert level and rows
out = ['# PhotoCraft plugin contract', '', 'Refreshed from literal adapter metadata by `scripts/photocraft-contract-source.py` without compiling PhotoCraft. The Rust adapter remains authoritative.', '', '## Contracts provided', '', '| Contract | Version |', '|---|---|']
for name, major, minor in re.findall(r'\("([^"\n]+)", (\d+), (\d+)\)', s.split('provided: Provided::new', 1)[1].split('ops:', 1)[0]):
    out.append(f'| `{name}` | {major}.{minor} |')
out.insert(out.index('|---|---|') + 1, f'| `photocraft.session` | {level[1]}.{level[2]} |')
out += ['', '## Where contributions appear', '', '| Kind | Placement |', '|---|---|']
for body in re.findall(r'SlotSpec \{([^\n]+)\}', s):
    a, b = strings(body)
    out.append(f'| `{a}` | {b} |')
out += ['', '## Operations (session plugins: `craft::call(op, args)`)', '']
for m in rows:
    ident, kind, permission, summary, args, result = m.groups()
    ident, permission, summary, args, result = map(value, [ident, permission, summary, args, result])
    out += [f'### `{ident}` ({kind})', '', summary, '', f'Permission: {permission}', '', '```json', '// args', args, '// result', result, '```', '']
out += ['## Events (`craft_event`)', '', '| Event | Payload | Needs `app.events` | Meaning |', '|---|---|---|---|']
events = s.split('events: [', 1)[1].split('].', 1)[0]
event_pattern = re.compile(r'\(\s*(' + literal + r'),\s*(' + literal + r'),\s*(true|false),\s*(' + literal + r')\s*,?\s*\)')
for m in event_pattern.finditer(events):
    name, payload, needs, summary = m.groups()
    out.append(f'| `{value(name)}` | `{value(payload).replace("|", "&#124;")}` | {"yes" if needs == "true" else "no"} | {value(summary)} |')
assert len(list(event_pattern.finditer(events))) == 7
notes = s.split('const NOTES: &str = r#"', 1)[1].split('"#;', 1)[0]
out += ['', '## Notes', '', notes, '', '## Error codes', '', '`-1` invalid input, `-2` permission denied, `-3` not found, `-4` stale or busy, `-5` cancelled, `-6` over budget, `-7` internal.', '']
(root / 'docs/plugins/contracts/photocraft.md').write_text('\n'.join(out))
print(f'Refreshed {len(rows)} operations and 7 events from source')

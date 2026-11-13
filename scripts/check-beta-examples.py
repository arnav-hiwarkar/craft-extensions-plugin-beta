#!/usr/bin/env python3
"""Check beta example metadata and build targets without compiling or loading plugins."""
from pathlib import Path
import re
import tomllib

root = Path(__file__).resolve().parents[1]
known = (root / 'crates/ext-cli/src/known_apps.rs').read_text()
apps = {}
for app in ('effectcraft', 'photocraft', 'figcraft'):
    match = re.search(r'"' + app + r'" => &\[([^\]]+)\]', known)
    assert match, f'{app}: missing CLI contract metadata'
    names = re.findall(r'"([^"]+)"', match[1])
    apps[app] = {name: '1.1' if name.endswith('.session') else '1.0' for name in names}
    page = (root / f'docs/plugins/contracts/{app}.md').read_text()
    table = page.split('## Contracts provided', 1)[1].split('\n## ', 1)[0]
    documented = dict(re.findall(r'\| `([^`]+)` \| ([0-9.]+) \|', table))
    assert documented == apps[app], f'{app}: CLI and contract reference disagree'

counts = dict.fromkeys(apps, 0)
packages = 0
multi = 0
for manifest_path in sorted((root / 'examples').glob('*/craft-plugin.toml')):
    package = tomllib.loads(manifest_path.read_text())
    folder = manifest_path.parent
    assert package['schema'] == 2
    declared = package['apps']
    contributions = package['contribution']
    assert len({c['id'] for c in contributions}) == len(contributions), f'{folder.name}: duplicate contribution'
    assert (folder / 'README.md').is_file(), f'{folder.name}: missing author workflow'
    cargo_file = folder / 'Cargo.toml'
    targets = set()
    if cargo_file.is_file():
        cargo = tomllib.loads(cargo_file.read_text())
        targets.add(cargo.get('lib', {}).get('name', cargo['package']['name'].replace('-', '_')))
        for target in cargo.get('example', []):
            targets.add(target['name'].replace('-', '_'))
            assert (folder / target.get('path', 'examples/' + target['name'] + '.rs')).is_file()
    usable = []
    for app in declared:
        assert app in apps, f'{folder.name}: unknown app {app}'
        supported = [c for c in contributions if c['contract'].split('@')[0] in apps[app]]
        assert supported, f'{folder.name}: advertises {app} without a supported contract'
        counts[app] += 1
        usable.append(app)
    for c in contributions:
        if 'module' in c or 'native' in package:
            assert 'permissions' in c, f'{folder.name}/{c["id"]}: missing executable permission declaration'
        if 'module' in c:
            assert Path(c['module']).suffix == '.wasm'
            assert Path(c['module']).stem in targets, f'{folder.name}: no build target for {c["module"]}'
        for filename in c.get('files', []):
            assert (folder / filename).is_file(), f'{folder.name}: missing resource {filename}'
    if 'native' in package:
        assert 'macos-aarch64' not in package['native']['executables'], 'Mac native example requires a real platform build'
    multi += len(usable) > 1
    packages += 1
assert all(n >= 3 for n in counts.values()), counts
assert multi >= 2, 'need both session and filter portability examples'
print(f'PASS: {packages} source examples; {multi} multi-app packages; supported packages {counts}')
print('Metadata/source check only. No modules built, loaded, installed or executed.')

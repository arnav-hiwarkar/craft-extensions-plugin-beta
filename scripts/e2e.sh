#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
exec 9>"${CRAFT_CARGO_LOCK:-$PWD/../.cargo.lock}"
flock 9
export CARGO_BUILD_JOBS=4
export CARGO_TARGET_DIR="$PWD/target"
cargo build -p craft-ext-cli
python3 - <<'PY'
import json, os, pathlib, shutil, subprocess, tempfile
root = pathlib.Path.cwd()
cli = root / 'target/debug/craft-ext'
work = pathlib.Path(tempfile.mkdtemp(prefix='craft-beta-e2e-'))
cat = root.parent / 'test-catalogue'
assets = cat / 'assets'
assets.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, CRAFT_EXT_DIR=str(work/'store'), CRAFT_CATALOGUE_URL=cat.as_uri()+'/', CRAFT_GUEST_PATH=str(root/'crates/guest'), CARGO_TARGET_DIR=str(root/'target/e2e-wasm'))
env.pop('CRAFT_CATALOGUE_ALLOW_FILE', None)
env.pop('CRAFT_TEMPLATES', None)
def run(*args, fail=None, code=None):
    print('RUN', *args, flush=True)
    p = subprocess.run([str(cli), *map(str,args), '--json'], env=env, text=True, capture_output=True)
    if fail is not None:
        assert p.returncode == code if code is not None else p.returncode != 0, (args,p.returncode,p.stdout,p.stderr)
        assert fail.lower() in p.stderr.lower(), (args,p.stderr)
        return
    assert p.returncode == 0, (args,p.returncode,p.stderr)
    return json.loads(p.stdout)
def plugin(*args, app='effectcraft', **kw): return run('plugin','--app',app,*args,**kw)
def author(path):
    for cmd in ['build','check','test']:
        # Catalogue packages ship release modules: debug (even opt-level 1) halftone exceeds the
        # per-pixel fuel budget at ordinary frame sizes and every render fails as too slow.
        result = run(cmd,path,*(['--release'] if cmd == 'build' else []))
        if cmd == 'test' and path.name == 'halftone':
            assert result and all(r['deterministic'] for r in result), result
    return run('pack',path,'--describe','--out',work/(path.name+'.craftplugin'))['file']
for kind in ['effect','panel','data']:
    path = work/kind
    run('new',kind,path,'--id','io.github.e2e.'+kind)
    author(path)
for name in ['halftone','color-notes','film-looks']:
    path=work/name
    shutil.copytree(root/'examples'/name,path,ignore=shutil.ignore_patterns('target','*.wasm','*.craftplugin'))
    cargo=path/'Cargo.toml'
    if cargo.exists(): cargo.write_text(cargo.read_text().replace('../../crates/guest',str(root/'crates/guest')))
    archive=author(path)
    shutil.copy(archive,assets/pathlib.Path(archive).name)
# Variants retain the compiled modules and change only the manifest.
def variant(name,version,replace=None):
    path=work/name
    manifest=path/'craft-plugin.toml'
    original=manifest.read_text()
    import re
    text=re.sub(r'^version = ".*"',f'version = "{version}"',original,count=1,flags=re.M)
    if replace: text=text.replace(*replace)
    manifest.write_text(text)
    result=run('pack',path,'--describe','--out',assets/(name+'-'+version+'.craftplugin'))
    manifest.write_text(original)
    return result['file']
variant('film-looks','1.0.0')
variant('halftone','2.0.0',('raster.filter@1.0','raster.filter@2.0'))
kill='io.github.e2e.kill-test'
path=work/'kill-test'
run('new','data',path,'--id',kill)
run('pack',path,'--describe','--out',assets/'kill-test.craftplugin')
# Native helper uses only POSIX shell builtins, works with the host's empty environment.
native=work/'native'; native.mkdir()
(native/'helper').write_text('''#!/bin/sh
read -r line
printf '%s\\n' '{"type":"ready"}'
while read -r line; do
 printf '%s\\n' '{"type":"done","id":1,"code":0}'
done
''')
(native/'helper').chmod(0o755)
(native/'craft-plugin.toml').write_text('''schema = 2
id = "io.github.e2e.native"
name = "Native smoke"
summary = "Native helper round trip fixture"
version = "1.0.0"
license = "MIT"
source = "https://github.com/e2e/native"
apps = ["effectcraft"]
[native]
executables = { "linux-x86_64" = "helper" }
[[contribution]]
id = "smoke"
kind = "command"
contract = "effectcraft.session@1.0"
label = "Smoke"
''')
run('pack',native,'--out',assets/'native.craftplugin')
def publish():
    run('catalogue','build','--local-assets',assets,'--out',cat)
    # Synthetic signed metadata exercises ranking without making online vendor claims.
    indexpath=cat/'index.json'; index=json.loads(indexpath.read_text())
    for vendor in index['vendors']:
        if vendor['login']=='e2e': vendor['tier']='verified'
    for row in index['packages']:
        row['stars']={'halftone':111,'color-notes':80,'film-looks':50,'native':3,'kill-test':5}.get(row['id'].split('.')[-1],0)
    indexpath.write_text(json.dumps(index))
    run('catalogue','sign','--repo',cat,'--key',pathlib.Path.home()/'.config/craft/catalogue.key')
# Start with the kill fixture enabled, then publish its disable record.
rev=assets/'revocations.toml'
if rev.exists(): rev.unlink()
publish()
half='io.github.arnav-hiwarkar.halftone'; notes='io.github.arnav-hiwarkar.color-notes'; film='io.github.arnav-hiwarkar.film-looks'
for app in ['effectcraft','photocraft','figcraft']:
    plugin('refresh',app=app)
    rows=plugin('search','halftone',app=app)
    assert rows[0]['id']==half and rows[0]['stars']==111 and rows[0]['tier']=='community',rows
    plugin('info',half,app=app)
    versions=plugin('versions',half,app=app)
    assert 'incompatible' in json.dumps(versions).lower() and '2.0' in json.dumps(versions),versions
    plugin('install',half,'--version','=2.0.0',app=app,fail='no listed version works')
    if app=='figcraft':
        plugin('install',half,app=app,fail='does not offer raster.filter')
        continue
    plugin('install',half,app=app)
    plugin('setup',half,'--enable','halftone',app=app)
    assert plugin('list',app=app)['apps'][app][half]['active']=='1.0.0'
# Neither a forged signature nor changed package metadata may authorise installs.
sig=cat/'index.json.sig'; saved=sig.read_bytes(); sig.write_text('0'*128)
plugin('refresh',fail='signature'); sig.write_bytes(saved)
meta=cat/f'packages/{half}.json'; saved=meta.read_bytes(); meta.write_bytes(saved.replace(b'Halftone',b'Xalftone',1))
plugin('install',half,fail='match'); meta.write_bytes(saved)
ranked=plugin('search','')
assert [r['id'].split('.')[-1] for r in ranked]==['kill-test','native','halftone','color-notes','film-looks'],ranked
assert ranked[0]['tier']=='verified' and ranked[2]['tier']=='community'
plugin('install',notes)
assert 'document.read:selection' in json.dumps(plugin('permissions',notes))
plugin('setup',notes,'--allow-all',fail='unknown',code=2)
plugin('setup',notes,'--enable','notes',fail='Permission needed',code=4)
plugin('setup',notes,'--enable','notes','--grant','document.read:selection')
before=plugin('list')['apps']['effectcraft'][notes]['grant_generation']
plugin('revoke',notes,'document.read:selection')
assert plugin('list')['apps']['effectcraft'][notes]['grant_generation']>before
notesrefs=work/'notes-refs.json'
notesrefs.write_text(json.dumps([{'id':notes,'version':'1.0.0','source':'catalogue','contributions':['notes']}]))
assert 'needs-enable' in json.dumps(plugin('project',notesrefs))
plugin('install',film,'--version','=1.0.0')
plugin('setup',film,'--enable','film-warm')
assert film in json.dumps(plugin('updates'))
plugin('update',film)
plugin('setup',film,'--enable','film-warm')
assert plugin('list')['apps']['effectcraft'][film]['active']=='1.1.0'
plugin('rollback',film,'1.0.0')
assert plugin('list')['apps']['effectcraft'][film]['active']=='1.0.0'
plugin('pin',film,'=1.0.0'); plugin('unpin',film)
assert 'issues/new' in plugin('report',film)
plugin('install',kill); plugin('setup',kill,'--enable','warm')
rev.write_text(f'''[[revocation]]
id = "{kill}"
severity = "disable"
reason = "E2E kill list fixture"
date = "2026-11-13"
''')
publish(); plugin('refresh')
plugin('install',kill,fail='')
plugin('enable',kill,'warm',fail='')
killrefs=work/'kill-refs.json'
killrefs.write_text(json.dumps([{'id':kill,'version':'0.1.0','source':'catalogue','contributions':['warm']}]))
assert 'unavailable' in json.dumps(plugin('project',killrefs))
plugin('install','io.github.e2e.native')
plugin('setup','io.github.e2e.native','--enable','smoke',fail='Native consent',code=4)
plugin('setup','io.github.e2e.native','--enable','smoke','--full-access')
plugin('setup',notes,'--enable','notes','--grant','document.read:selection')
# Reinstalling the same catalogue archive from a file must become Local and lose grants.
local=assets/'color-notes.craftplugin'
plugin('install-file',local)
state=plugin('list')
assert state['packages'][notes]['releases']['1.0.0']['tier']=='local'
plugin('setup',notes,'--enable','notes',fail='Permission needed',code=4)
refs=work/'refs.json'
refs.write_text(json.dumps([{'id':half,'version':'1.0.0','source':'catalogue','contributions':['halftone']},{'id':'io.github.e2e.missing','version':'1.0.0','source':'local'}]))
statuses=plugin('project',refs)
assert 'ready' in json.dumps(statuses) and 'unavailable' in json.dumps(statuses),statuses
plugin('remove',notes)
assert notes not in plugin('list')['packages']
refs.write_text(json.dumps([{'id':notes,'version':'1.0.0','source':'catalogue'}, {'id':half,'version':'1.0.0','source':'catalogue','contributions':['missing-contribution']}]))
statuses=plugin('project',refs)
assert 'downloadable' in json.dumps(statuses) and 'needs-other-version' in json.dumps(statuses),statuses
(root/'target/e2e-native-path').write_text(str(next((work/'store').rglob('helper'))))
print('PASS author + signed local catalogue + three-app manager flows; fixtures:',cat,'work:',work)
PY
export CRAFT_E2E_NATIVE_EXE="$(cat target/e2e-native-path)"
cargo test -p craft-ext-core -p craft-ext-host -p craft-ext-store -p craft-ext-cli -p craft-guest
CRAFT_CATALOGUE_ALLOW_FILE=1 cargo test -p craft-ext-store --features net local_asset_policy_size_and_digest
printf '%s\n' 'E2E PASS'

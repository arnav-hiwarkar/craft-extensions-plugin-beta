import pathlib
import json
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(__file__))
from rustlex import ID_RE
from macroarms import app_arms, resolve

C = str(pathlib.Path(__file__).resolve().parents[1])
BASE = os.environ.get('CRAFT_SUITE', str(pathlib.Path(__file__).resolve().parents[6]))

EXCLUDE = {
    'effectcraft': {'s'},
    'photocraft': {'color_param', 'int_i32', 'check_resample_budget', 'color_value', 'cmyk_param', 'color_of'},
    'figcraft': {'req_num', 'req_string', 'req_index', 'req_id', 'index', 'own', 'effect_from', 'parse', 'opt_index'},
    'filmcraft': {'f', 'b', 'sub', 'error_toast', 'pending'},
    'lightcraft': {'refused'},
    'designcraft': {'attrs', 'style_name_param', 'start_number'},
    'texturecraft': {'color_param', 'channel_param', 'from_mesh_bytes', 'blend_param', 'points_param'},
}


def rg_lines(pattern, path):
    r = subprocess.run(['rg', '-n', '--no-heading', '-o', pattern, path], capture_output=True, text=True)
    return r.stdout.splitlines()


def supplements(app):
    out = []
    eng = f'{BASE}/{app}/crates/engine/src'
    if app == 'photocraft':
        src = open(f'{eng}/cmd/gallery_cmds.rs').read()
        m = re.search(r'gallery_specs!\[(.*?)\]', src, re.S)
        names = set(re.findall(r'[A-Z]\w+', m.group(1))) if m else set()
        art = open(f'{BASE}/photocraft/crates/algo/src/artistic.rs').read()
        for mm in re.finditer(r'^\s*(\w+) = "(\w+)", "([^"]+)", "(\w+)", r#"(.*?)"#;', art, re.M):
            if mm.group(1) in names:
                out.append(dict(id=f'filter.gallery.{mm.group(2)}', label=mm.group(3), params=mm.group(5), menu=['Filter', mm.group(4)], ctor='gallery_specs!', file='photocraft/crates/engine/src/cmd/gallery_cmds.rs', journal=True))
        out.append(dict(id='filter.filterGallery', label='Filter Gallery', ctor='gallery_specs!', file='photocraft/crates/engine/src/cmd/gallery_cmds.rs', journal=True))
        for ln in rg_lines(r'stack_spec!\("(\w+)", "([^"]+)"', f'{eng}/cmd/layer_menu_cmds.rs'):
            mm = re.search(r'stack_spec!\("(\w+)", "([^"]+)"', ln)
            out.append(dict(id=f'layer.smartObjects.stackMode.{mm.group(1)}', label=mm.group(2), ctor='stack_spec!', menu=['Layer', 'Smart Objects', 'Stack Mode'], file='photocraft/crates/engine/src/cmd/layer_menu_cmds.rs', journal=True))
        body = open(f'{eng}/cmd/type_styles_cmds.rs').read()
        sfx = sorted(set(re.findall(r'concat!\(\$prefix, "(\.\w+)"\)', body)))
        for pre in ('type.characterStyle', 'type.paragraphStyle'):
            for x in sfx:
                out.append(dict(id=pre + x, label=pre + x, ctor='style_specs!', file='photocraft/crates/engine/src/cmd/type_styles_cmds.rs', journal=not x.endswith('.list')))
        for n in range(1, 21):
            out.append(dict(id=f'type.openType.stylisticSets.set{n}', label=f'Set {n}', ctor='stylistic_sets!', menu=['Type', 'OpenType', 'Stylistic Sets'], file='photocraft/crates/engine/src/cmd/type_extra_cmds.rs', journal=True))
        ls = open(f'{eng}/layer_style.rs').read()
        for mm in re.finditer(r'style_cmd!\(\s*"(\w+)",\s*"([^"]+)",\s*r?#*"(.*?)"#*\s*,?\s*\)', ls, re.S):
            out.append(dict(id=f'layer.layerStyle.{mm.group(1)}', label=mm.group(2), params=mm.group(3), ctor='style_cmd!', menu=['Layer', 'Layer Style'], file='photocraft/crates/engine/src/layer_style.rs', journal=True))
        cm = open(f'{eng}/cmd.rs').read()
        m = re.search(r'const ADJ: &\[\(&str, &str, &str\)\] = &\[(.*?)\n    \];', cm, re.S)
        if m:
            for mm in re.finditer(r'\(\s*"(\w+)",\s*"([^"]+)",\s*(r#*"(.*?)"#*|"(.*?)")\s*,?\s*\)', m.group(1), re.S):
                kind, label = mm.group(1), mm.group(2)
                prm = mm.group(4) or mm.group(5) or ''
                out.append(dict(id=f'image.adjustments.{kind}', label=label, params=prm, ctor='ADJ-loop', menu=['Image', 'Adjustments'], file='photocraft/crates/engine/src/cmd.rs', journal=True))
                out.append(dict(id=f'layer.newAdjustmentLayer.{kind}', label=label, params=prm + ' + layer opts', ctor='ADJ-loop', menu=['Layer', 'New Adjustment Layer'], file='photocraft/crates/engine/src/cmd.rs', journal=True))
    if app == 'vectorcraft':
        src = open(f'{eng}/cmd/textstyles.rs').read()
        sfx = []
        for x in re.findall(r'concat!\(\$p, "(\.\w+)"\)', src):
            if x not in sfx:
                sfx.append(x)
        for pre in ('charStyle', 'paraStyle'):
            for x in sfx:
                out.append(dict(id=pre + x, label=pre + x, ctor='textstyles-macro', file='vectorcraft/crates/engine/src/cmd/textstyles.rs', journal=x not in ('.list', '.unused')))
    if app == 'filmcraft':
        for n in range(1, 10):
            out.append(dict(id=f'multicam.selectCamera{n}', label=f'Camera {n}', ctor='camera_keys!', params='{"videoOnly":bool?}', file='filmcraft/crates/engine/src/multicam.rs', journal=True))
    return out


QUERY_CTORS = {'query!', 'query', 'q', 'query_spec'}


def query_flag(app, r):
    v = r.get('variant')
    if v in ('query',):
        return True
    if v in ('noundo', 'view'):
        return False
    if r['ctor'] in QUERY_CTORS:
        return True
    for k in ('journal', 'history', 'mutates'):
        if k in r and isinstance(r[k], bool):
            return not r[k]
    b = r.get('bools') or []
    if app == 'figcraft' and r['ctor'] == 'spec!' and b:
        return not b[0]
    if app == 'anicraft' and b:
        return not b[-1]
    if app == 'effectcraft' and r['ctor'] == 'spec':
        return True
    return None


def finalize(app):
    d = json.load(open(f'{C}/raw_{app}.json'))
    ex = EXCLUDE.get(app, set())
    recs = [r for r in d['records'] if r['ctor'] not in ex]
    recs += supplements(app)
    ov = {i: o for i, o in d['overrides']}
    table = app_arms(f'{BASE}/{app}/crates')
    for r in recs:
        if not r['ctor'].endswith('!') or 'raw_args' not in r:
            continue
        res = resolve(table, r['ctor'], f"{BASE}/{r['file']}", r['nargs'], r.get('variant'))
        if not res:
            r['macro_resolved'] = False
            continue
        names, bf = res
        r['macro_resolved'] = True
        def argval(nm):
            if nm in names:
                i = names.index(nm)
                if 1 <= i <= len(r['raw_args']):
                    return r['raw_args'][i - 1]
            return None
        for fld, (kind, v) in bf.items():
            if kind == 'lit':
                if fld in ('journal', 'history', 'mutates', 'undoable', 'view'):
                    r[fld] = v
                elif fld in ('label', 'params') and isinstance(v, str):
                    r[fld] = v
            elif kind == 'arg':
                txt = argval(v)
                if txt is None:
                    continue
                if fld in ('journal', 'history', 'mutates', 'undoable', 'view') and txt in ('true', 'false'):
                    r[fld] = txt == 'true'
                elif fld in ('label', 'params') and txt.startswith('"'):
                    try:
                        r[fld] = json.loads(txt)
                    except Exception:
                        pass
                elif fld == 'shortcut':
                    mm = re.search(r'Some \( (".*?") \)', txt)
                    r['shortcut'] = json.loads(mm.group(1)) if mm else (json.loads(txt) if txt.startswith('"') else None)
                elif fld == 'menu':
                    r['menu'] = [json.loads(x) for x in re.findall(r'"(?:[^"\\]|\\.)*"', txt)]
                elif fld == 'enabled':
                    r['enabled_fn'] = txt
            elif kind == 'expr' and fld == 'enabled':
                r['enabled_fn'] = v
    seen = {}
    for r in recs:
        if r['id'] in seen:
            seen[r['id']].setdefault('dup_sites', []).append(f"{r.get('file')}:{r.get('line')}")
            continue
        if r['id'] in ov:
            r.update(ov[r['id']])
        seen[r['id']] = r
    out = []
    for r in seen.values():
        q = query_flag(app, r)
        if q is None and r.get('journal') is None:
            q = False
        params = r.get('params')
        out.append(dict(
            app=app, id=r['id'], namespace=r['id'].split('.')[0], label=r.get('label'), menu='/'.join(r['menu']) if isinstance(r.get('menu'), list) else (r.get('menu') or ''),
            shortcut=r.get('shortcut'), params=params, ctor=r['ctor'], variant=r.get('variant'), query=q,
            undoable=r.get('undoable'), view=r.get('view'), explicit_targets=r.get('explicit_targets'), schema=r.get('schema'),
            file=r.get('file'), line=r.get('line'), macro_resolved=r.get('macro_resolved'), journal=r.get('journal', r.get('history', r.get('mutates'))),
        ))
    return out


if __name__ == '__main__':
    for app in sys.argv[1:]:
        rows = finalize(app)
        json.dump(rows, open(f'{C}/stage_{app}.json', 'w'), indent=1)
        print(app, len(rows), 'query', sum(1 for r in rows if r['query']), 'unknownq', sum(1 for r in rows if r['query'] is None))

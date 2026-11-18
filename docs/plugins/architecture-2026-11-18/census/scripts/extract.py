import pathlib
import json
import os
import re
import sys
from collections import Counter

sys.path.insert(0, os.path.dirname(__file__))
from rustlex import ID_RE, lex, match_close, split_args, test_ranges

BASE = os.environ.get('CRAFT_SUITE', str(pathlib.Path(__file__).resolve().parents[6]))
C = str(pathlib.Path(__file__).resolve().parents[1])
SKIP_CALLEES = {'json', 'format', 'bad', 'assert', 'assert_eq', 'assert_ne', 'println', 'eprintln', 'write', 'writeln', 'panic', 'Err', 'Ok', 'Some', 'vec', 'matches', 'concat', 'tr', 'debug', 'info', 'warn', 'error', 'trace', 'anyhow', 'bail', 'ensure'}


def arg_str(arg):
    if len(arg) == 1 and arg[0][0] == 'str':
        return arg[0][1]
    if len(arg) >= 3 and arg[0][1] == 'concat' and arg[1][1] == '!':
        return ''.join(t[1] for t in arg if t[0] == 'str')
    return None


def arg_text(arg):
    return ' '.join(t[1] if t[0] != 'str' else json.dumps(t[1]) for t in arg)


def consts_of(toks):
    out = {}
    for k in range(len(toks) - 6):
        if toks[k][1] in ('const', 'static') and toks[k + 1][0] == 'id' and toks[k + 2][1] == ':':
            m = k + 3
            while m < len(toks) and m < k + 10 and toks[m][1] != '=':
                m += 1
            if m + 1 < len(toks) and toks[m][1] == '=':
                if toks[m + 1][0] == 'str' and toks[m + 2][1] == ';':
                    out[toks[k + 1][1]] = toks[m + 1][1]
                elif toks[m + 1][1] == 'concat':
                    e = match_close(toks, m + 3)
                    out[toks[k + 1][1]] = ''.join(t[1] for t in toks[m + 3:e] if t[0] == 'str')
    return out


def describe_call(name, args, consts):
    rec = {'ctor': name, 'nargs': len(args)}
    strs = []
    bools = []
    menu = None
    shortcut = None
    params = None
    for idx, a in enumerate(args[1:], start=1):
        sv = arg_str(a)
        if sv is not None:
            strs.append((idx, sv))
            continue
        if len(a) == 1 and a[0][1] in ('true', 'false'):
            bools.append(a[0][1] == 'true')
            continue
        if a and a[0][1] == '[' and all(t[0] == 'str' or t[1] in '[],' for t in a):
            menu = [t[1] for t in a if t[0] == 'str']
            continue
        if a and a[0][1] == '&' and len(a) > 1 and a[1][1] == '[':
            menu = [t[1] for t in a if t[0] == 'str']
            continue
        if a and a[0][1] == 'Some' and len(a) >= 3 and a[2][0] == 'str':
            shortcut = a[2][1]
            continue
        if len(a) == 1 and a[0][0] == 'id' and a[0][1] in consts:
            strs.append((idx, consts[a[0][1]]))
            continue
    rec['label'] = strs[0][1] if strs else None
    if len(strs) >= 2:
        params = strs[-1][1]
        if len(strs) >= 3:
            menu = menu or [strs[1][1]]
    rec['menu'] = menu
    rec['shortcut'] = shortcut
    rec['params'] = params
    rec['bools'] = bools
    rec['raw_args'] = [arg_text(a)[:6000] for a in args[1:]]
    return rec


def describe_struct(fields, consts):
    rec = {'ctor': 'CommandSpec{}'}
    for k, v in fields.items():
        sv = arg_str(v)
        if k == 'id':
            continue
        if k in ('label', 'params') and sv is None and len(v) == 1 and v[0][1] in consts:
            sv = consts[v[0][1]]
        if sv is not None:
            rec[k] = sv
        elif len(v) == 1 and v[0][1] in ('true', 'false'):
            rec[k] = v[0][1] == 'true'
        elif k == 'menu':
            rec[k] = [t[1] for t in v if t[0] == 'str']
        elif k == 'shortcut':
            s = [t[1] for t in v if t[0] == 'str']
            rec[k] = s[0] if s else None
        else:
            rec[k] = arg_text(v)[:120]
    return rec


def struct_fields(toks, i, j):
    fields = {}
    spread = None
    for a in split_args(toks, i, j):
        if not a:
            continue
        if a[0][1] == '..':
            spread = a
            continue
        if len(a) >= 2 and a[0][0] == 'id' and a[1][1] == ':':
            fields[a[0][1]] = a[2:]
        elif len(a) == 1 and a[0][0] == 'id':
            fields[a[0][1]] = [('id', a[0][1], 0)]
    return fields, spread


def scan_file(path, rel):
    s = open(path, errors='ignore').read()
    toks = lex(s)
    tr = test_ranges(toks)

    def in_test(k):
        return any(a <= k <= b for a, b in tr)

    consts = consts_of(toks)
    found = []
    overrides = []
    k = 0
    n = len(toks)
    while k < n:
        t = toks[k]
        if t[0] == 'id' and t[1] == 'CommandSpec' and k + 1 < n and toks[k + 1][1] == '{' and (k == 0 or toks[k - 1][1] not in ('struct', 'for')):
            j = match_close(toks, k + 1)
            fields, spread = struct_fields(toks, k + 1, j)
            if not in_test(k):
                idv = arg_str(fields.get('id', [])) if 'id' in fields else None
                if idv and ID_RE.match(idv):
                    rec = describe_struct(fields, consts)
                    rec.update(id=idv, file=rel, line=t[2])
                    found.append(rec)
                elif spread is not None:
                    ov = {kk: (vv[0][1] == 'true') for kk, vv in fields.items() if len(vv) == 1 and vv[0][1] in ('true', 'false')}
                    sp_ids = [x[1] for x in spread if x[0] == 'str' and ID_RE.match(x[1])]
                    if sp_ids and ov:
                        overrides.append((sp_ids[0], ov))
            k += 2
            continue
        if t[0] == 'id' and t[1] not in SKIP_CALLEES and k + 2 < n:
            bang = toks[k + 1][1] == '!'
            o = k + 2 if bang else k + 1
            if o < n and toks[o][0] == 'p' and toks[o][1] in ('(', '[', '{') and (bang or toks[o][1] == '('):
                if k > 0 and toks[k - 1][1] in ('fn', 'macro_rules'):
                    k += 1
                    continue
                j = match_close(toks, o)
                args = split_args(toks, o, j)
                variant = None
                if args and len(args[0]) == 2 and args[0][0][0] == 'id' and args[0][1][0] == 'str' and bang:
                    variant = args[0][0][1]
                    args = [[args[0][1]]] + args[1:]
                if len(args) >= 2:
                    first = arg_str(args[0])
                    if first and ID_RE.match(first) and not in_test(k):
                        rest_has_str = any(arg_str(a) is not None for a in args[1:])
                        if rest_has_str or (bang and len(args) >= 2):
                            rec = describe_call(t[1] + ('!' if bang else ''), args, consts)
                            rec.update(id=first, file=rel, line=t[2], variant=variant)
                            for a in args:
                                if len(a) >= 3 and a[0][1] == 'schema' and a[1][1] == ':':
                                    rec['schema'] = arg_text(a[2:])[:80]
                            found.append(rec)
                k += 1
                continue
        k += 1
    return found, overrides


def walk(app, roots):
    out = []
    ovs = []
    for r in roots:
        for dp, dn, fn in os.walk(os.path.join(BASE, app, r)):
            dn[:] = [d for d in dn if d not in ('target', 'tests', 'benches', 'examples', 'fuzz')]
            for f in fn:
                if not f.endswith('.rs') or f.startswith('tests') or f.endswith('_tests.rs') or f == 'tests.rs':
                    continue
                p = os.path.join(dp, f)
                rel = os.path.relpath(p, BASE)
                txt = open(p, errors='ignore').read()
                if 'CommandSpec' not in txt and 'specs' not in txt and '!(' not in txt:
                    continue
                f2, o2 = scan_file(p, rel)
                out += f2
                ovs += o2
    return out, ovs


if __name__ == '__main__':
    app = sys.argv[1]
    roots = sys.argv[2:] or ['crates/engine/src']
    recs, ovs = walk(app, roots)
    json.dump({'records': recs, 'overrides': ovs}, open(f'{C}/raw_{app}.json', 'w'), indent=1)
    ids = Counter(r['id'] for r in recs)
    print(app, 'records', len(recs), 'unique', len(ids), 'dups', sum(1 for v in ids.values() if v > 1))
    print(Counter(r['ctor'] for r in recs).most_common(40))

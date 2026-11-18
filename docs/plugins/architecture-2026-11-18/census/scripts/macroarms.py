import os
import re
import sys

sys.path.insert(0, os.path.dirname(__file__))
from rustlex import lex, match_close, split_args

FIELDS = ('id', 'label', 'menu', 'shortcut', 'params', 'journal', 'history', 'mutates', 'undoable', 'view', 'enabled')


def arms_in(src):
    toks = lex(src)
    out = {}
    k = 0
    while k < len(toks) - 3:
        if toks[k][1] == 'macro_rules' and toks[k + 1][1] == '!' and toks[k + 2][0] == 'id':
            name = toks[k + 2][1]
            o = k + 3
            e = match_close(toks, o)
            m = o + 1
            arms = []
            while m < e:
                if toks[m][1] in ('(', '[', '{'):
                    pe = match_close(toks, m)
                    pat = split_args(toks, m, pe)
                    q = pe + 1
                    if q < e and toks[q][1] == '=>':
                        be = match_close(toks, q + 1)
                        body = toks[q + 2:be]
                        arms.append((pat, body))
                        m = be + 1
                        continue
                    m = pe + 1
                    continue
                m += 1
            out.setdefault(name, []).extend(arms)
            k = e
        k += 1
    return out


def pat_info(pat):
    names = []
    kw = None
    for i, item in enumerate(pat):
        vals = [t for t in item]
        if i == 0 and len(vals) >= 2 and vals[0][0] == 'id' and vals[1][1] == '$':
            kw = vals[0][1]
            vals = vals[1:]
        if vals and vals[0][1] == '$' and len(vals) > 1 and vals[1][0] == 'id':
            names.append(vals[1][1])
        elif vals and vals[0][1] == '[':
            inner = [t for t in vals if t[0] == 'id' and t[1] not in ('literal', 'expr', 'tt', 'ident')]
            names.append(inner[0][1] if inner else '?')
        elif len(vals) >= 3 and vals[0][0] == 'id' and vals[1][1] == ':':
            names.append(vals[0][1])
        else:
            names.append(None)
    return kw, names


def body_fields(body):
    res = {}
    for i, t in enumerate(body):
        if t[0] == 'id' and t[1] in FIELDS and i + 1 < len(body) and body[i + 1][1] == ':' and (i == 0 or body[i - 1][1] in ('{', ',')):
            v = body[i + 2] if i + 2 < len(body) else None
            if v is None:
                continue
            if v[1] == '$' and i + 3 < len(body):
                res[t[1]] = ('arg', body[i + 3][1])
            elif v[1] in ('true', 'false'):
                res[t[1]] = ('lit', v[1] == 'true')
            elif v[0] == 'str':
                res[t[1]] = ('lit', v[1])
            else:
                res[t[1]] = ('expr', v[1])
    return res


def app_arms(root):
    table = {}
    for dp, dn, fn in os.walk(root):
        dn[:] = [d for d in dn if d not in ('target', 'tests')]
        for f in fn:
            if f.endswith('.rs'):
                p = os.path.join(dp, f)
                s = open(p, errors='ignore').read()
                if 'macro_rules!' not in s:
                    continue
                for name, arms in arms_in(s).items():
                    table.setdefault(name, []).append((p, arms))
    return table


def resolve(table, ctor, file_abs, nargs, variant):
    name = ctor.rstrip('!')
    defs = table.get(name)
    if not defs:
        return None
    defs = sorted(defs, key=lambda d: d[0] != file_abs)
    for _, arms in defs:
        for pat, body in arms:
            kw, names = pat_info(pat)
            if kw != variant:
                continue
            if len(names) != nargs:
                continue
            bf = body_fields(body)
            if not bf:
                continue
            return names, bf
    return None

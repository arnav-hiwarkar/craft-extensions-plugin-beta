import re

ID_RE = re.compile(r'^[a-z][A-Za-z0-9_]*(\.[A-Za-z0-9_\-]+)+$')


def lex(s):
    toks = []
    i, n = 0, len(s)
    line = 1
    while i < n:
        c = s[i]
        if c == '\n':
            line += 1
            i += 1
            continue
        if c.isspace():
            i += 1
            continue
        if s.startswith('//', i):
            j = s.find('\n', i)
            i = n if j < 0 else j
            continue
        if s.startswith('/*', i):
            d = 1
            j = i + 2
            while j < n and d:
                if s.startswith('/*', j):
                    d += 1
                    j += 2
                elif s.startswith('*/', j):
                    d -= 1
                    j += 2
                else:
                    j += 1
            line += s.count('\n', i, j)
            i = j
            continue
        m = re.match(r'b?r(#*)"', s[i:i + 20])
        if m and (i == 0 or not (s[i - 1].isalnum() or s[i - 1] == '_')):
            hashes = m.group(1)
            start = i + m.end()
            end = s.find('"' + hashes, start)
            if end < 0:
                end = n
            toks.append(('str', s[start:end], line))
            line += s.count('\n', i, end)
            i = end + 1 + len(hashes)
            continue
        if c == '"' or (c == 'b' and i + 1 < n and s[i + 1] == '"'):
            j = i + (2 if c == 'b' else 1)
            buf = []
            while j < n and s[j] != '"':
                if s[j] == '\\' and j + 1 < n:
                    nx = s[j + 1]
                    if nx == '\n':
                        j += 2
                        while j < n and s[j] in ' \t\n\r':
                            j += 1
                        continue
                    buf.append({'n': '\n', 't': '\t', '"': '"', '\\': '\\', "'": "'", '0': '\0'}.get(nx, nx))
                    j += 2
                    continue
                buf.append(s[j])
                j += 1
            toks.append(('str', ''.join(buf), line))
            line += s.count('\n', i, j)
            i = j + 1
            continue
        if c == "'":
            m = re.match(r"'(\\u\{[0-9a-fA-F]+\}|\\x[0-9a-fA-F]{2}|\\.|[^\\'])'", s[i:i + 14])
            if m:
                toks.append(('char', m.group(1), line))
                i += m.end()
                continue
            m = re.match(r"'[A-Za-z_][A-Za-z0-9_]*", s[i:])
            toks.append(('life', m.group(0) if m else "'", line))
            i += m.end() if m else 1
            continue
        if c.isalpha() or c == '_':
            m = re.match(r'[A-Za-z_][A-Za-z0-9_]*', s[i:])
            toks.append(('id', m.group(0), line))
            i += m.end()
            continue
        if c.isdigit():
            m = re.match(r'[0-9][0-9A-Za-z_.]*', s[i:])
            toks.append(('num', m.group(0), line))
            i += m.end()
            continue
        if s.startswith('::', i):
            toks.append(('p', '::', line))
            i += 2
            continue
        if s.startswith('=>', i) or s.startswith('->', i) or s.startswith('..', i):
            toks.append(('p', s[i:i + 2], line))
            i += 2
            continue
        toks.append(('p', c, line))
        i += 1
    return toks


OPEN = {'(': ')', '[': ']', '{': '}'}


def match_close(toks, i):
    o = toks[i][1]
    c = OPEN[o]
    d = 0
    for j in range(i, len(toks)):
        t = toks[j]
        if t[0] == 'p':
            if t[1] in OPEN:
                d += 1
            elif t[1] in (')', ']', '}'):
                d -= 1
                if d == 0:
                    return j
    return len(toks) - 1


def split_args(toks, i, j):
    args = []
    cur = []
    d = 0
    for k in range(i + 1, j):
        t = toks[k]
        if t[0] == 'p' and t[1] in OPEN:
            d += 1
        elif t[0] == 'p' and t[1] in (')', ']', '}'):
            d -= 1
        if d == 0 and t[0] == 'p' and t[1] == ',':
            args.append(cur)
            cur = []
        else:
            cur.append(t)
    if cur:
        args.append(cur)
    return args


def test_ranges(toks):
    ranges = []
    for k in range(len(toks) - 6):
        if toks[k][1] == '#' and toks[k + 1][1] == '[' and toks[k + 2][1] == 'cfg' and toks[k + 3][1] == '(' and toks[k + 4][1] == 'test':
            m = k + 5
            while m < len(toks) and toks[m][1] != ']':
                m += 1
            m += 1
            while m < len(toks) and toks[m][1] == '#':
                m2 = m
                while m2 < len(toks) and toks[m2][1] != ']':
                    m2 += 1
                m = m2 + 1
            q = m
            while q < len(toks) and q < m + 12 and not (toks[q][0] == 'p' and toks[q][1] in ('{', ';')):
                q += 1
            if q < len(toks) and toks[q][1] == '{':
                ranges.append((k, match_close(toks, q)))
    return ranges

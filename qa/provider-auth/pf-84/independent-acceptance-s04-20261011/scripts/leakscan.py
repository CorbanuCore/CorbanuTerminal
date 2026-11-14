# leakscan.py <dir>...: whole-value search in every file (binary-safe). Real keys come from env REAL_ZAI,
# REAL_KIMI, REAL_OPENAI, REAL_CLAUDE (never printed); synthetic canaries from $S/proxy/canaries.json
# (also with the sk-ant-oat01- prefix used for Claude tokens). Prints only ids, paths and counts.
import json, os, sys
S = os.environ.get('S', '/Volumes/CorbanuDrive/Corbanu/tmp/pf84s04acc')
sec = {k: os.environ[k] for k in ('REAL_ZAI', 'REAL_KIMI', 'REAL_OPENAI', 'REAL_CLAUDE') if os.environ.get(k)}
for v, i in json.load(open(f'{S}/proxy/canaries.json')).items():
    sec[f'canary:{i}'] = v
skip = {os.path.realpath(p) for p in (f'{S}/proxy/canaries.json',)}
needles = {k: v.encode() for k, v in sec.items() if len(v) >= 12}
hits, nfiles = {}, 0
for root in sys.argv[1:]:
    for dp, dn, fn in os.walk(root):
        dn[:] = [d for d in dn if d not in ('.git', 'bin')]
        for f in fn:
            p = os.path.join(dp, f)
            if os.path.realpath(p) in skip or os.path.islink(p):
                continue
            try:
                b = open(p, 'rb').read()
            except Exception:
                continue
            nfiles += 1
            for k, n in needles.items():
                c = b.count(n)
                if c:
                    hits.setdefault(k, []).append((p.replace(S, '$S'), c))
print('roots:', ', '.join(r.replace(S, '$S') for r in sys.argv[1:]))
print(f'scanned {nfiles} files; {len(needles)} values ({sum(k.startswith("REAL") for k in needles)} real, '
      f'{sum(not k.startswith("REAL") for k in needles)} synthetic; canaries.json itself skipped)')
for k in sorted(needles):
    h = hits.get(k, [])
    print(f'{k}: {len(h)} file(s)')
    for p, c in h:
        print(f'    {p} x{c}')

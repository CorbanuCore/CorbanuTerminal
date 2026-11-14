# partialscan.py <dir>...: search every file for any 16-character window of each real key (env REAL_*),
# to catch partial values (e.g. a paste cut short). Prints only key ids, paths and window-hit counts.
import os, sys
W = 16
keys = {k: os.environ[k].encode() for k in ('REAL_ZAI', 'REAL_KIMI', 'REAL_OPENAI', 'REAL_CLAUDE') if os.environ.get(k)}
wins = {k: {v[i:i + W] for i in range(len(v) - W + 1)} for k, v in keys.items()}
hits, n = {}, 0
for root in sys.argv[1:]:
    for dp, dn, fn in os.walk(root):
        dn[:] = [d for d in dn if d not in ('.git', 'bin')]
        for f in fn:
            p = os.path.join(dp, f)
            if os.path.islink(p):
                continue
            try:
                b = open(p, 'rb').read()
            except Exception:
                continue
            n += 1
            for k, ws in wins.items():
                c = sum(1 for w in ws if w in b)
                if c:
                    hits.setdefault(k, []).append((p.replace(os.environ.get('S', '@@'), '$S'), c))
print(f'scanned {n} files; window={W}; keys={len(keys)}')
for k in sorted(keys):
    print(f'{k}: {len(hits.get(k, []))} file(s)')
    for p, c in hits.get(k, []):
        print(f'    {p} ({c} windows)')

# leakscan.py <dir>...: whole-value search for secrets in every file (binary-safe). Secrets come from env:
# REAL_ZAI, REAL_KIMI, REAL_OPENAI, REAL_CLAUDE (real keys, never printed) plus every synthetic value in
# canaries.json / fake files under the scratch dir. (The random fake-zai-*/fake-kimi-* keys were piped
# straight into `account add` and never stored outside the vault, so they cannot be searched for.) Prints only secret ids, file paths and hit counts.
import json, os, sys, glob
S = '/Volumes/CorbanuDrive/Corbanu/tmp/pf84s03acc'
sec = {k: os.environ[k] for k in ('REAL_ZAI', 'REAL_KIMI', 'REAL_OPENAI', 'REAL_CLAUDE') if os.environ.get(k)}
for v, i in json.load(open(f'{S}/proxy/canaries.json')).items():
    sec[f'canary:{i}'] = v
for p in ('default-canary.txt', 'claude-fake-token.txt'):
    sec[f'synthetic:{p}'] = open(f'{S}/{p}').read().strip()
sec['synthetic:cfgdir-token'] = 'sk-ant-' + 'oat01-cfgdir-canary-s03'
sec['synthetic:env-token'] = 'envtok-' + 'canary-s03'
needles = {k: v.encode() for k, v in sec.items() if len(v) >= 12}
hits, nfiles = {}, 0
for root in sys.argv[1:]:
    for dp, dn, fn in os.walk(root):
        dn[:] = [d for d in dn if d not in ('.git',)]
        for f in fn:
            p = os.path.join(dp, f)
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
print(f'scanned {nfiles} files under {len(sys.argv) - 1} root(s); {len(needles)} secret values '
      f'({sum(1 for k in needles if k.startswith("REAL"))} real, {sum(1 for k in needles if not k.startswith("REAL"))} synthetic)')
for k in sorted(needles):
    h = hits.get(k, [])
    print(f'{k}: {len(h)} file(s)')
    for p, c in h:
        print(f'    {p} x{c}')

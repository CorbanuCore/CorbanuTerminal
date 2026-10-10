import os, sys, json
S = sys.argv[1]
secrets = {}
for name in ["ZAI", "KIMI", "OPENAI", "CLAUDE"]:
    v = os.environ.get("X_" + name, "")
    if len(v) >= 16: secrets["real:" + name] = v.encode()
for k, n in json.load(open(f"{S}/mock/canaries.json")).items(): secrets["canary:" + n] = k.encode()
secrets["canary:claude-fake-token"] = open(f"{S}/mock/claude-fake-token.txt", "rb").read().strip()
secrets["canary:cfgdir-token"] = b"sk-ant-" + b"oat01-cfgdir-canary-0001"
skip_dirs = {f"{S}/bin", f"{S}/inst/home/packages", f"{S}/mock"}
hits, scanned = {}, 0
ROOT = sys.argv[2] if len(sys.argv) > 2 else S
for root, dirs, files in os.walk(ROOT):
    if any(root == d or root.startswith(d + "/") for d in skip_dirs): continue
    for f in files:
        p = os.path.join(root, f)
        try:
            if os.path.islink(p) or os.path.getsize(p) > 300_000_000: continue
            data = open(p, "rb").read()
        except Exception: continue
        scanned += 1
        for label, v in secrets.items():
            if v and v in data: hits.setdefault(label, []).append(p.replace(S, "$S"))
print(f"secrets checked: {len(secrets)} ({sum(1 for k in secrets if k.startswith('real:'))} real keys, rest synthetic canaries); files scanned: {scanned}")
for label in secrets:
    print(f"  {label}: {'CLEAN' if label not in hits else 'FOUND in ' + str(len(hits[label])) + ' file(s): ' + ', '.join(hits[label][:8])}")

import os, sys, pathlib
secrets = {k: os.environ[k] for k in ("LEAK_A", "LEAK_B") if os.environ.get(k)}
hits = 0; n = 0
for root in sys.argv[1:]:
    p = pathlib.Path(root)
    files = [p] if p.is_file() else [f for f in p.rglob("*") if f.is_file() and ".git/" not in str(f) and "/target/" not in str(f)]
    for f in files:
        try: data = f.read_bytes()
        except Exception: continue
        n += 1
        for k, v in secrets.items():
            if v.encode() in data:
                hits += 1; print("HIT", k, f)
print(f"secrets={len(secrets)} files={n} hits={hits}")

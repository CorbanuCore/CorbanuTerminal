#!/usr/bin/env python3
"""usage: find_old_ledgers.py <root>...: list state DBs holding accounting attempts, with their earliest dispatch time
(read-only; payloads are not printed)."""
import os, sys, sqlite3, json, datetime
for root in sys.argv[1:]:
    for d, dirs, files in os.walk(root):
        dirs[:] = [x for x in dirs if x not in ("node_modules", "target", ".git", "proc")]
        for f in files:
            if f.endswith("state_5.sqlite") or f == "state.sqlite":
                p = os.path.join(d, f)
                try:
                    db = sqlite3.connect(f"file:{p}?mode=ro", uri=True, timeout=1)
                    n = db.execute("select count(*) from draft_accounting_attempts").fetchone()[0]
                    if not n: continue
                    ts = [json.loads(x)["dispatched_at_ms"] for (x,) in db.execute("select payload from draft_accounting_attempts")]
                    fmt = [r[0] for r in db.execute("select version from _accounting_migrations")]
                    print(f"{datetime.datetime.utcfromtimestamp(min(ts)/1000):%Y-%m-%d} {datetime.datetime.utcfromtimestamp(max(ts)/1000):%Y-%m-%d} attempts={n} format={fmt} {p}")
                except Exception:
                    pass

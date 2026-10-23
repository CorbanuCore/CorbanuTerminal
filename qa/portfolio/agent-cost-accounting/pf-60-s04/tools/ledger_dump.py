#!/usr/bin/env python3
"""Read-only dump of a home's accounting ledger: one line per attempt with provider, model, thread, turn, basis and
basis source from its bound price record, rates, and the observation's token counts. Storage cross-check only."""
import sqlite3, json, sys
db = sqlite3.connect(f"file:{sys.argv[1]}?mode=ro", uri=True)
print("format:", [r[0] for r in db.execute("select version from _accounting_migrations")])
snap = {k: json.loads(v) for k, v in db.execute("select snapshot_id, payload from draft_accounting_price_snapshots")} if 0 else {}
cols = [r[1] for r in db.execute("pragma table_info(draft_accounting_price_snapshots)")]
for row in db.execute("select * from draft_accounting_price_snapshots"):
    p = json.loads(row[-1]); snap[p["id"]] = p
bind = dict(db.execute("select * from draft_accounting_price_bindings").fetchall())
obs = {}
ocols = [r[1] for r in db.execute("pragma table_info(draft_accounting_observations)")]
for row in db.execute("select * from draft_accounting_observations"):
    p = json.loads(row[-1]); obs[p.get("attempt_id", row[0])] = p
for (pl,) in db.execute("select payload from draft_accounting_attempts"):
    a = json.loads(pl); s = snap.get(bind.get(a["attempt_id"]), {})
    o = obs.get(a["attempt_id"])
    extra = {k: s.get(k) for k in s if k not in ("id","provider","model","scope","currency","unit","source_reference","observed_at_ms","approved_at_ms","effective_from_ms","effective_end_ms")}
    print(a["dispatched_at_ms"], a["provider"], a["model"], "thread=" + str(a.get("thread_id"))[:13], "turn=" + str(a.get("turn"))[:24],
          json.dumps(extra, separators=(",", ":")), "obs=" + (json.dumps(o, separators=(",", ":"))[:300] if o else "none"))

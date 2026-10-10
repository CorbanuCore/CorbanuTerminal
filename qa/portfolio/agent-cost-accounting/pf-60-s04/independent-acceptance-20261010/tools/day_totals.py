#!/usr/bin/env python3
"""Per-UTC-day totals straight from a home's ledger rows (read-only), for the historical-inspection and date-range checks.
For each attempt: the bound price's rates and the observation's token counts, inclusive input
(noncached = input - read - write; write billed at its rate when the price states one). Unpriced or unobserved attempts
are counted separately, never as zero. Usage: day_totals.py <state.sqlite> [START END] (ISO dates, end exclusive)."""
import sqlite3, json, sys, datetime
from decimal import Decimal as D

db = sqlite3.connect(f"file:{sys.argv[1]}?mode=ro", uri=True)
snap = {}
for row in db.execute("select * from draft_accounting_price_snapshots"):
    p = json.loads(row[-1]); snap[p["id"]] = p
bind = dict(db.execute("select * from draft_accounting_price_bindings").fetchall())
obs = {}
for row in db.execute("select * from draft_accounting_observations"):
    p = json.loads(row[-1]); obs[p.get("attempt_id", row[0])] = p
days = {}
for (pl,) in db.execute("select payload from draft_accounting_attempts"):
    a = json.loads(pl)
    day = datetime.datetime.fromtimestamp(a["dispatched_at_ms"] / 1000, datetime.timezone.utc).date()
    d = days.setdefault(day, dict(n=0, usd=D(0), unpriced=0, unobserved=0, sub=0, prov=set()))
    d["n"] += 1; d["prov"].add(f'{a["provider"]}/{a["model"]}')
    s = snap.get(bind.get(a["attempt_id"]), {}); r = s.get("rates") or {}
    o = (obs.get(a["attempt_id"]) or {}).get("patch")
    if s.get("basis") not in (None, "Billed"):
        d["sub"] += 1; continue
    if not o or o.get("input") is None or o.get("output") is None:
        d["unobserved"] += 1; continue
    if not r or r.get("noncached") is None or r.get("output") is None:
        d["unpriced"] += 1; continue
    rd, wr = o.get("read") or 0, o.get("write") or 0
    wrate = D(r["write"]) if r.get("write") is not None else D(0)
    nonc = o["input"] - rd - (wr if r.get("write") is not None else 0)
    d["usd"] += (D(nonc) * D(r["noncached"]) + D(rd) * D(r.get("read") or 0) + D(wr) * wrate * (1 if r.get("write") is not None else 0)
                 + D(o["output"]) * D(r["output"])) / D(10**6)
lo = datetime.date.fromisoformat(sys.argv[2]) if len(sys.argv) > 3 else None
hi = datetime.date.fromisoformat(sys.argv[3]) if len(sys.argv) > 3 else None
tot = D(0)
for day in sorted(days):
    if lo and not (lo <= day < hi):
        continue
    d = days[day]; tot += d["usd"]
    print(day, f'attempts={d["n"]} billed_usd={d["usd"].normalize()} subscription={d["sub"]} unpriced={d["unpriced"]} unobserved={d["unobserved"]}', ",".join(sorted(d["prov"])))
print("range billed usd=", tot.normalize())

#!/usr/bin/env python3
"""Golden totals for the seeded history (PF-60-S04 qualification): recompute every seeded attempt from the seed's own
inputs and prices (codex-rs/state/examples/accounting_demo_seed.rs `history`), independently of the ledger.
Usage: golden_history.py <today YYYY-MM-DD>. Inclusive input: noncached = input - read."""
import sys, datetime
from decimal import Decimal as D
today = datetime.date.fromisoformat(sys.argv[1])
GLM = (D("1.4"), D("0.26"), D("4.4")); OPENAI = (D("5"), D("0.5"), D("30"))  # seed's price() / openai()
def cost(i, r, o, p): return (D(i - r) * p[0] + D(r) * p[1] + D(o) * p[2]) / D(10**6)
days = {}
for back in range(1, 15):
    day = today - datetime.timedelta(days=back); rows = []
    for inp in (12_000 + back * 300, 8_000 + back * 150):
        rows.append(("zai glm-5.2", inp, inp // 4, inp // 10, cost(inp, inp // 4, inp // 10, GLM)))
    if back % 3 == 0:
        rows.append(("openai gpt-5.4", 20_000, 5_000, 1_500, cost(20_000, 5_000, 1_500, OPENAI)))
    days[day] = rows
total = D(0)
for day in sorted(days):
    s = sum(r[4] for r in days[day]); total += s
    print(day, f"requests={len(days[day])}", f"tokens={sum(r[1] + r[3] for r in days[day])}", f"usd={s}")
print("seeded 14-day total usd=", total)

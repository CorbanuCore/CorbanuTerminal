#!/usr/bin/env python3
"""Group provider-reported GLM 5.2 usage into UTC buckets, independently of the product.

For each provider response id found in `corbanu exec` trace stderr files:
  first  = timestamp of the first SSE chunk (client clock, faked where libfaketime was used)
  sent  ~= first - ms_to_first_sse_byte (from the per-call metrics line for that generation_id)
Requests are bucketed by `sent` (the request start); any request whose `sent` and
usage-chunk timestamps fall in different buckets is flagged AMBIGUOUS, because a
product may legitimately bucket by admission or by completion.
Prices: Z.AI published USD/1M: input 1.40, cached input 0.26, output 4.40.
usage: buckets.py GRAIN(hour|day|week|month) FILE.err ...
"""
import datetime as dt, json, re, sys, collections
from decimal import Decimal as D

LINE = re.compile(r'^(\S+Z)\s.*Chat completions SSE event: (\{.*\})\s*$')


def ts(s):
    return dt.datetime.fromisoformat(s.replace("Z", "+00:00"))


def bucket(t, g):
    if g == "hour": return t.strftime("%Y-%m-%dT%H:00Z")
    if g == "day": return t.strftime("%Y-%m-%d")
    if g == "week": y, w, _ = t.isocalendar(); return f"{y}-W{w:02d}"
    if g == "month": return t.strftime("%Y-%m")


def cost(u):
    c = (u.get("prompt_tokens_details") or {}).get("cached_tokens", 0) or 0
    return (D(u["prompt_tokens"] - c) * D("1.40") + D(c) * D("0.26") + D(u["completion_tokens"]) * D("4.40")) / D(10**6)


g = sys.argv[1]
agg = collections.defaultdict(lambda: [0, 0, D(0)])
for f in sys.argv[2:]:
    first, usage, uts, lat = {}, {}, {}, {}
    for line in open(f, errors="replace"):
        s = line.strip()
        m = LINE.match(s)
        if m:
            o = json.loads(m.group(2)); i = o.get("id")
            first.setdefault(i, ts(m.group(1)))
            if o.get("usage"): usage[i] = o["usage"]; uts[i] = ts(m.group(1))
        elif '"corbanu_call_metrics"' in s:
            o = json.loads(s[s.index("{"):])
            lat[o.get("generation_id")] = o.get("ms_to_first_sse_byte") or 0
    for i, u in usage.items():
        sent = first[i] - dt.timedelta(milliseconds=lat.get(i, 0))
        b, b2 = bucket(sent, g), bucket(uts[i], g)
        flag = "" if b == b2 else f"  AMBIGUOUS sent={sent.isoformat()[:23]} usage={uts[i].isoformat()[:23]} ({b} vs {b2})"
        a = agg[b]; a[0] += 1; a[1] += u["total_tokens"]; a[2] += cost(u)
        if flag: print(f"{f.split('/')[-1]} {i}{flag} usd={cost(u)}")
print(f"grain={g} (bucketed by request start)")
for b in sorted(agg):
    print(f"  {b}: {agg[b][0]} requests, {agg[b][1]} tokens, exact USD {agg[b][2]}")
print(f"  ALL: {sum(a[0] for a in agg.values())} requests, {sum(a[1] for a in agg.values())} tokens, exact USD {sum((a[2] for a in agg.values()), D(0))}")

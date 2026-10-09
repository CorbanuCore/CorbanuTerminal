#!/usr/bin/env python3
"""List the usage chunks the product itself saw (RUST_LOG=codex_api=trace SSE lines in exec stderr).
usage: product_seen_usage.py <tag>...   (reads <tag>.err in the current directory)"""
import json, re, sys
LINE = re.compile(r'^(\S+Z)\s.*Chat completions SSE event: (\{.*\})\s*$')
for t in sys.argv[1:]:
    n, us = 0, []
    for l in open(t + ".err", errors="replace"):
        m = LINE.match(l.strip())
        if m:
            n += 1
            o = json.loads(m.group(2))
            if o.get("usage"):
                us.append((m.group(1), o.get("id"), o["usage"]))
    print(f"{t}: {n} SSE events traced; chunks carrying usage: {len(us)}")
    for u in us:
        print("   ", u[0], u[1], json.dumps(u[2]))

#!/usr/bin/env python3
"""Exact (unrounded) USD per row and per source, from the per-request lines that recompute.py prints.
usage: exact_usd.py <recompute-output.txt>   Prices: Z.AI input 1.40, cached input 0.26, output 4.40 USD / 1M tokens."""
import re, sys, collections
from decimal import Decimal as D
R = re.compile(r'^\S+\s+(\S+)\s+\S+\s+p=\s*(\d+) c=\s*(\d+) o=\s*(\d+) tot=\s*(\d+)')
tot = collections.defaultdict(lambda: [0, D(0)])
for l in open(sys.argv[1]):
    m = R.match(l)
    if m:
        s, p, c, o, t = m.group(1), *map(int, m.groups()[1:])
        u = (D(p - c) * D("1.40") + D(c) * D("0.26") + D(o) * D("4.40")) / D(10**6)
        tot[s][0] += 1; tot[s][1] += u
for s, (n, u) in sorted(tot.items()):
    print(f"{s:12} {n} requests  exact USD {u.normalize()}")

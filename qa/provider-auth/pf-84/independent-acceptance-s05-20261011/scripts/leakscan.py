#!/usr/bin/env python3
"""leakscan.py <dir>... : count files containing any credential value, whole-value match.
Real keys arrive only through the environment (LEAK_<name>=value, set by the caller's
vault-helper substitution); synthetic canaries are passed the same way. Values are never
printed: only the name, the number of files and the matching paths."""
import os, sys

needles = {k[5:]: v.encode() for k, v in os.environ.items() if k.startswith("LEAK_") and len(v) >= 8}
hits = {n: [] for n in needles}
files = 0
for root in sys.argv[1:]:
    for dp, _, fns in os.walk(root):
        for fn in fns:
            p = os.path.join(dp, fn)
            try:
                if os.path.islink(p) or os.path.getsize(p) > 512 * 1024 * 1024:
                    continue
                data = open(p, "rb").read()
            except OSError:
                continue
            files += 1
            for n, v in needles.items():
                if v in data:
                    hits[n].append(p)
print(f"scanned {files} files under {len(sys.argv) - 1} roots for {len(needles)} values")
for n in sorted(hits):
    print(f"  {n}: {len(hits[n])} file(s)" + "".join(f"\n    {p}" for p in hits[n]))

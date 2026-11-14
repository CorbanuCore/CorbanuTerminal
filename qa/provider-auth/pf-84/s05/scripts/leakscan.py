#!/usr/bin/env python3
"""Count occurrences of the credential values named by env vars (LEAK_*) in every file under the
given roots. Prints paths and counts only, never a value."""
import os, sys
values = {k: v.encode() for k, v in os.environ.items() if k.startswith("LEAK_") and v}
hits = 0
files = 0
for root in sys.argv[1:]:
    for dirpath, _, names in os.walk(root):
        for name in names:
            path = os.path.join(dirpath, name)
            try:
                data = open(path, "rb").read()
            except OSError:
                continue
            files += 1
            for label, value in values.items():
                n = data.count(value)
                if n:
                    hits += n
                    print(f"HIT {label} x{n} in {path}")
print(f"scanned {files} files for {len(values)} values: {hits} hits")

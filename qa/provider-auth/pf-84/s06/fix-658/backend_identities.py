#!/usr/bin/env python3
"""#658 demo helper: whose sign-in each ChatGPT backend request carried, from mock_openai.py's log."""
import collections, json, sys

counts = collections.Counter()
for line in open(sys.argv[1]):
    event = json.loads(line)
    if event.get("event") == "get" and "/backend-api/" in event.get("path", ""):
        who = event.get("who", "none").split(" (")[0]
        counts[(event["path"].split("/backend-api/", 1)[1], who)] += 1
for (path, who), n in sorted(counts.items()):
    print(f"{n:3d}  {path:45s} {who}")
print(f"backend requests on account work: {sum(n for (_, w), n in counts.items() if w == 'work')}")
print(f"backend requests on the default login: {sum(n for (_, w), n in counts.items() if w == 'default')}")

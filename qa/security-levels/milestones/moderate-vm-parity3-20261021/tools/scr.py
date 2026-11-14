"""print screens of a run whose label matches REGEX (non-blank lines; last N lines).  usage: scr.py RUN_DIR REGEX [N]"""
import json, re, sys, tarfile
run, rx = sys.argv[1], re.compile(sys.argv[2]); n = int(sys.argv[3]) if len(sys.argv) > 3 else 40
with tarfile.open(run + "/collected.tar") as t:
    for l in t.extractfile("evidence/screens.jsonl").read().decode().splitlines():
        d = json.loads(l)
        if rx.search(d["label"]):
            print("=====", d["utc"], d["label"])
            print("\n".join([x for x in d["screen"].split("\n") if x.strip()][-n:]))

"""Preserve logs and report exact tests and writable-scope line counts."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

out = Path(__file__).resolve().parent
repo = out.parents[4]
for path in out.glob("*.log"):
    path.with_suffix(".log.gz").write_bytes(gzip.compress(path.read_bytes(),mtime=0))
tests = {}
lanes = json.loads((out/"lanes.json").read_text())
for lane in ("core-default","core-feature","tui"):
    log = (out/(lane+".log")).read_text()
    summary = re.findall(r"Summary .*",log)[-1]
    match = re.search(r"(\d+) tests run: (\d+) passed.*?(\d+) skipped",summary)
    assert match, summary
    failures = re.findall(r"^\s*(?:FAIL|TIMEOUT) .*",log,re.MULTILINE)
    tests[lane] = dict(summary=summary,run=int(match[1]),passed=int(match[2]),
                       skipped=int(match[3]),nonzero_tests=int(match[1])-int(match[2]),
                       failure_names=failures,exit=next(x["exit"] for x in lanes if x["lane"]==lane),
                       nextest_test_threads=4)
(out/"test-results.json").write_text(json.dumps(tests,indent=2)+"\n")
modified = subprocess.check_output(["git","diff","--name-only","-z"],cwd=repo).decode().split("\0")
added = subprocess.check_output(["git","ls-files","--others","--exclude-standard","-z"],cwd=repo).decode().split("\0")
names = sorted(set(filter(None,modified+added)))
assert all(n.startswith("qa/portfolio/agent-cost-accounting/pf-60-s03/") for n in names),names
inventory = []
for name in names:
    path = repo/name
    if path == out/"inventory.json":
        continue
    data = path.read_bytes()
    inventory.append(dict(path=name,sha256=hashlib.sha256(data).hexdigest(),bytes=len(data),
                          lines=None if path.suffix==".gz" else len(data.splitlines()),
                          status="modified" if name in modified else "new"))
numstat = subprocess.check_output(["git","diff","--numstat"],cwd=repo,text=True)
counts = dict(existing_diff_numstat=numstat,
    existing_added_lines=sum(int(x.split()[0]) for x in numstat.splitlines()),
    existing_removed_lines=sum(int(x.split()[1]) for x in numstat.splitlines()),
    new_program_lines=sum(x["lines"] for x in inventory if x["status"]=="new" and x["path"].endswith((".py",".c"))),
    new_text_evidence_lines=sum(x["lines"] for x in inventory if x["status"]=="new" and x["lines"] is not None and not x["path"].endswith((".py",".c"))),
    tracked_change_files=len(inventory),production_rust_lines=0,outside_scope_paths=[])
(out/"inventory.json").write_text(json.dumps(dict(counts=counts,files=inventory),indent=2)+"\n")
print(json.dumps(counts))

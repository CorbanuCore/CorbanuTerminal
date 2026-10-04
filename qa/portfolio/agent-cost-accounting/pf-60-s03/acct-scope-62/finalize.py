"""Index completed evidence; never inspect fixture credentials."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = "qa/portfolio/agent-cost-accounting/pf-60-s03/"
def save(name, value):
    (here/name).write_text(json.dumps(value,indent=2)+"\n")

lanes = json.loads((here/"gates-01/lanes.json").read_text())
assert [l["lane"] for l in lanes] == ["prerequisites","core-default","core-feature","tui","feature-binary"]
assert all(l["exit"] == 0 for l in lanes)
results = []
for name in ("core-default","core-feature","tui"):
    text = (here/"gates-01"/(name+".log")).read_text()
    match = re.search(r"Summary \[[^\]]+\] (\d+) tests run: (\d+) passed([^\n]*)",text)
    assert match and int(match[1]) > 0, name
    failures = [line.strip() for line in text.splitlines()
                if re.match(r"\s*(?:FAIL|TIMEOUT|ABORT|XFAIL)\s",line)]
    results.append(dict(lane=name,run=int(match[1]),passed=int(match[2]),
                        exact_failure_lines=failures,summary=match[0]))
assert [r["run"] for r in results] == [124,127,91]
save("test-results.json",dict(source_commit=subprocess.check_output(
    ["git","rev-parse","HEAD"],cwd=repo,text=True).strip(),
    threads=4,prerequisites_first=True,lanes=results,total_run=342,total_passed=sum(r["passed"] for r in results)))
sources = [
    ("codex-rs/core/src/accounting.rs",40,76),
    ("codex-rs/model-provider-info/src/lib.rs",1174,1201),
    ("codex-rs/model-provider-info/src/lib.rs",1867,1945),
    ("codex-rs/core/src/config/mod.rs",3945,3986),
    ("codex-rs/core/src/agent/role.rs",287,332),
    ("codex-rs/state/src/runtime/accounting_pricing.rs",204,216),
    ("codex-rs/state/src/runtime/accounting_store.rs",31,48),
    ("codex-rs/tui/src/chatwidget/tokens.rs",513,516),
    ("codex-rs/tui/src/chatwidget/tokens.rs",733,740),
]
save("residual-source-evidence.json",[
    dict(path=path,start=start,end=end,
         sha256=hashlib.sha256((repo/path).read_bytes()).hexdigest(),
         excerpt="\n".join((repo/path).read_text().splitlines()[start-1:end]))
    for path,start,end in sources])
save("attempts.json",[
    dict(run="gates-01",exit=0,result="prerequisites; 124/124 default,127/127 feature,91/91 TUI; feature binary"),
    dict(run="coverage-run-01",exit=0,result="28 new-check rejections; no old-check execution yet"),
    dict(run="coverage-run-02",exit=1,result="Harness incorrectly expected old audit to accept narrow subtotal omission; old narrow-specific assertion rejected. Preserved partial receipts; no final result claimed."),
    dict(run="coverage-run-03",exit=0,result="28 new-check rejections; old auditor accepts 26 and rejects 2 already-bound narrow cases"),
    dict(run="scope-run-01",exit=0,result="4 collected priced same-day attempts across 3 roots; fresh-reader zero reproduced; seven priced pages"),
    dict(run="scope-audit",exit=0,result="Independent USD 0.00284 store/day and exact seven-page reconciliation"),
    dict(run="p3-negative-controls",exit=0,result="Six negatives; fresh TZ-stripped binary PID; four preserved positive controls. No old_check_passes field for TZ."),
])
for directory in (here,here/"gates-01"):
    for path in directory.glob("*.log"):
        path.with_suffix(".log.gz").write_bytes(gzip.compress(path.read_bytes(),mtime=0))
status = subprocess.check_output(["git","status","--short"],cwd=repo,text=True)
assert all(scope in line for line in status.splitlines()), status
numstat = subprocess.check_output(["git","diff","--numstat"],cwd=repo,text=True)
untracked = subprocess.check_output(
    ["git","ls-files","--others","--exclude-standard","--",str(here.relative_to(repo))],
    cwd=repo,text=True).splitlines()
new_files = []
for path in untracked:
    if path.endswith(("/scope.json","/inventory.json")):
        continue
    raw = (repo/path).read_bytes()
    new_files.append(dict(path=path,bytes=len(raw),
                          lines=None if path.endswith(".gz") else len(raw.splitlines())))
save("scope.json",dict(status=status,tracked_numstat=numstat,
    production_lines_changed=0,rust_test_lines_changed=0,
    new_qa_source_lines={p.name:len(p.read_text().splitlines()) for p in here.glob("*.py")},
    new_files=new_files,count_excludes="scope.json and inventory.json avoid recursive counts",
    binary_sha256=json.loads((here/"scope-run-01/manifest.json").read_text())["binary_sha256"]))
files = []
for path in sorted(here.rglob("*")):
    relative = path.relative_to(here)
    if (not path.is_file() or path.is_symlink() or
        any(part=="fixture" or part=="__pycache__" or part.startswith(("exact-","displayed-")) for part in relative.parts) or
        path.suffix in (".log",".raw",".txt") or path.name=="inventory.json"):
        continue
    raw = path.read_bytes()
    files.append(dict(path=str(relative),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest()))
save("inventory.json",dict(files=files,count=len(files),
    excludes="self, ignored fixture/raw logs/cache, regenerable mutation directories; original gz captures and counterexample error receipts retained"))
print(json.dumps(dict(tests=results,tracked_numstat=numstat,
    new_qa_source_lines=json.loads((here/"scope.json").read_text())["new_qa_source_lines"],
    artifact_count=len(files)),indent=2))

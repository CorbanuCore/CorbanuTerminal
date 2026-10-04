"""Summarize actual logs and enumerate scoped evidence without fixture credentials."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
def save(name,data):
    (here/name).write_text(json.dumps(data,indent=2)+"\n")
results=[]
for name in ("core-default","core-feature","tui"):
    raw=(here/"gates-01"/(name+".log")).read_bytes()
    text=raw.decode()
    summary=re.search(r"Summary \[[^\]]+\] (\d+) tests run: (\d+) passed([^\n]*)",text)
    assert summary,name
    failures=[line.strip() for line in text.splitlines() if re.match(r"\s*(?:FAIL|TIMEOUT|ABORT|XFAIL)\s",line)]
    results.append(dict(lane=name,run=int(summary[1]),passed=int(summary[2]),
        failed_or_timed_out=len(failures),exact_failure_lines=failures,summary=summary[0]))
save("test-results.json",dict(lanes=results,total_run=sum(r["run"] for r in results),
    total_passed=sum(r["passed"] for r in results),threads=4,prerequisites_first=True,
    source_commit="90095a359599f055653d39eb3e6aa9831faeaaef"))
save("attempts.json",[
    dict(run="gates-01",exit=0,disposition="124+127+91 pass; prerequisites and feature binary built"),
    dict(run="boundary-run-01",exit=1,disposition="PID assertion rejected child-overwritten timezone probe"),
    dict(run="boundary-run-02",exit=0,disposition="26 cases; four per-product-PID timezone positive controls"),
    dict(run="reader-run-01",exit=1,disposition="Anthropic built-in endpoint ignored override; loopback sandbox denied external transport; timed out. No Anthropic fixture emission"),
    dict(run="reader-run-02",exit=0,disposition="Exploratory capture completed; sqlite=false did NOT produce backend unavailability. That named page is not a backend-failure pass"),
    dict(run="reader-run-03",exit=1,disposition="Backend Refresh did not recover after inexact table-rename restoration"),
    dict(run="reader-run-04",exit=1,disposition="Reopened backend still unreadable after inexact restoration; blocked subsequent stale assertion"),
    dict(run="reader-run-05",exit=0,disposition="Final six categories; backend fault remains during retry, exact backup restored after stop, fresh read succeeds; stale case separately succeeds"),
    dict(run="reader-audit-first",exit=1,disposition="Audit incorrectly required monetary unknown text on request navigation page with no amount; narrowed to numeric pages"),
    dict(run="reader-audit-final",exit=0,disposition="15 numeric pages, independent amounts/rates/metrics and reader-state assertions"),
    dict(run="boundary-audit",exit=0,disposition="26 independent cases"),
    dict(run="p3-negative-controls",exit=0,disposition="6 counterexamples rejected, four positive localtime controls")])
for directory in (here,here/"gates-01"):
    for file in directory.glob("*.log"):
        file.with_suffix(".log.gz").write_bytes(gzip.compress(file.read_bytes(),mtime=0))
status=subprocess.check_output(["git","status","--short"],cwd=repo,text=True)
diff=subprocess.check_output(["git","diff","--numstat"],cwd=repo,text=True)
assert all("qa/portfolio/agent-cost-accounting/pf-60-s03/" in line for line in status.splitlines())
save("scope.json",dict(status=status,tracked_numstat=diff,production_lines=0,rust_test_lines=0,
    new_qa_source_lines={p.name:len(p.read_text().splitlines()) for p in sorted(here.iterdir()) if p.suffix in (".py",".c")},
    binary_sha256=hashlib.file_digest((here.parent/"acct-activation-33/feature/target/debug/codex").open("rb"),"sha256").hexdigest(),
    clock_source_sha256=hashlib.sha256((here/"tz_clock.c").read_bytes()).hexdigest(),
    clock_binary_sha256=hashlib.sha256((here/"tz_clock.dylib").read_bytes()).hexdigest()))
files=[]
for file in sorted(here.rglob("*")):
    if not file.is_file() or any(p in ("fixture","__pycache__") for p in file.relative_to(here).parts):
        continue
    if file.suffix in (".log",".raw",".txt",".dylib") or file.name=="inventory.json":
        continue
    raw=file.read_bytes()
    item=dict(path=str(file.relative_to(here)),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest())
    if file.suffix!=".gz":
        item["lines"]=len(raw.splitlines())
    files.append(item)
save("inventory.json",dict(files=files,excludes="self; ignored raw/log/fixture/compiled files. Raw captures preserved losslessly as gzip.",
    count=len(files)))
print(json.dumps(dict(tests=results,tracked_changes=diff,new_source_lines=json.loads((here/"scope.json").read_text())["new_qa_source_lines"],files=len(files)),indent=2))

"""Derive round-77 gate counts and evidence digests from preserved raw results."""
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
lanes = {}
for name in ("thread-settings", "permission-confirmation"):
    raw = (HERE / (name + ".log")).read_text()
    summary = re.search(
        r"Summary \[\s*([0-9.]+)s\] (\d+) tests run: (\d+) passed(?: \(\d+ slow\))?, (\d+) skipped",
        raw)
    if summary is None:
        raise ValueError("unexpected gate summary: " + name)
    passes = re.findall(r"^\s*PASS\s+\[[^\]]+\].*$", raw, re.MULTILINE)
    failures = re.findall(r"^\s*(?:FAIL|TIMEOUT|CRASH|ABORT)\s+.*$", raw, re.MULTILINE)
    command = json.loads((HERE / (name + "-command.json")).read_text())
    if command["exit_code"] != 0 or failures or len(passes) != int(summary[3]):
        raise ValueError("gate failed or pass count mismatch: " + name)
    if int(summary[2]) != len(passes) or not passes:
        raise ValueError("gate did not run a nonzero passing test set: " + name)
    lanes[name] = dict(run=int(summary[2]), passed=len(passes), failed=len(failures),
                       skipped=int(summary[4]), seconds=float(summary[1]),
                       failure_names=failures, exit_code=command["exit_code"])
logs = {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(HERE.glob("*.log"))}
preflights = {}
for name in ("preflight-local.json", "preflight-existing-harness.json"):
    report = json.loads((HERE / name).read_text())
    preflights[name] = dict(
        exit_code=report["exit_code"], checked=len(report["checks"]),
        passed=sum(row["status"] == "passed" for row in report["checks"]),
        failed=sum(row["status"] == "failed" for row in report["checks"]),
        guest_contact_attempted=report["guest_contact_attempted"])
record = dict(lanes=lanes, raw_log_sha256=logs,
              prerequisites_exit_code=json.loads((HERE / "prerequisites-command.json").read_text())["exit_code"],
              local_controls=json.loads((HERE / "controls.json").read_text())["summary"],
              syntax=json.loads((HERE / "syntax-checks.json").read_text()),
              preflights=preflights, guest_contact=False, functional_cases_executed=0)
print(json.dumps(record, indent=2, sort_keys=True))

"""Summarize exact raw local gate logs; no execution or credential access."""
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
PRIOR = HERE.parent / "pf83-handoff-70"
lanes = {}
for name in ("thread-settings", "permission-confirmation"):
    raw = (HERE / (name + ".log")).read_text()
    match = re.search(r"Summary \[\s*([0-9.]+)s\] (\d+) tests run: (\d+) passed(?: \(\d+ slow\))?, (\d+) skipped", raw)
    if match is None:
        raise ValueError("unexpected gate summary: " + name)
    failures = re.findall(r"^\s*FAIL.*$", raw, re.MULTILINE)
    record = json.loads((HERE / (name + "-command.json")).read_text())
    if record["exit_code"] != 0 or failures:
        raise ValueError("failed gate: " + name)
    lanes[name] = dict(run=int(match[2]), passed=int(match[3]), failed=0,
                       skipped=int(match[4]), seconds=float(match[1]),
                       failure_names=failures, exit_code=record["exit_code"])
logs = {}
for root in (HERE, PRIOR):
    for path in sorted(root.glob("*.log")):
        logs[str(path.relative_to(HERE.parent))] = hashlib.sha256(path.read_bytes()).hexdigest()
prior = json.loads((PRIOR / "checks.json").read_text())["log_sha256"]
for name, digest in prior.items():
    if logs["pf83-handoff-70/" + name] != digest:
        raise ValueError("historical raw log mismatch: " + name)
record = dict(lanes=lanes, raw_log_sha256=logs, prior_raw_logs_match_recorded_digests=True,
              prerequisites_exit_code=json.loads((HERE / "prerequisites-command.json").read_text())["exit_code"],
              guest_contact=False, product_staged=False, functional_cases_executed=0,
              guard_controls=json.loads((HERE / "guard-controls.json").read_text())["summary"])
(HERE / "checks.json").write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
print(json.dumps(record, indent=2, sort_keys=True))

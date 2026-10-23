"""Summarize immutable lane logs; do not infer a pass from command completion."""
import gzip
import hashlib
import json
from pathlib import Path
import re

here = Path(__file__).resolve().parent
target_test = "suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity"
lanes = []
for name in ("alone-feature", "core-feature", "core-default", "tui"):
    path = here / "gates-02" / (name + ".log.gz")
    data = gzip.decompress(path.read_bytes())
    record = json.loads(path.with_suffix("").with_suffix(".json").read_text())
    assert hashlib.sha256(data).hexdigest() == record["log_sha256"]
    text = data.decode()
    summaries = re.findall(r"^\s*Summary \[\s*([0-9.]+)s\] (\d+) tests? run: (.+)$", text, re.M)
    assert len(summaries) == 1, (name, summaries)
    seconds, count, details = summaries[0]
    record.update(run=int(count), suite_seconds=float(seconds), summary=details)
    for key, label in (("passed", "passed"), ("failed", "failed"),
                       ("timed_out", "timed out"), ("flaky", "flaky"),
                       ("skipped", "skipped"), ("leaky", "leaky")):
        match = re.search(r"(\d+) " + label, details)
        record[key] = int(match[1]) if match else 0
    assert record["run"] > 0
    record["target_test_outcomes"] = [
        dict(status=match[1], seconds=float(match[2]))
        for line in text.splitlines() if target_test in line
        if (match := re.search(r"\b(PASS|TMT|FAIL) \[\s*([0-9.]+)s\]", line))
    ]
    lanes.append(record)

prior = here.parent / "acct-receipt-66/gates-01/core-feature.log.gz"
prior_data = gzip.decompress(prior.read_bytes())
prior_text = prior_data.decode()
prior_lines = list(dict.fromkeys(line.strip() for line in prior_text.splitlines()
                                 if target_test in line))
report = dict(
    lanes=lanes,
    required_lane_executions=sum(row["run"] for row in lanes if row["lane"] != "alone-feature"),
    required_lane_passed=sum(row["passed"] for row in lanes if row["lane"] != "alone-feature"),
    prior_feature_log=str(prior.relative_to(here.parent)),
    prior_feature_log_sha256=hashlib.sha256(prior_data).hexdigest(),
    prior_target_test_lines=prior_lines,
    interpretation="Required lane counts overlap; the isolated replay is one additional execution, not an additional distinct test. Built-in retries are included in raw logs, not added to test counts.",
)
with (here / "test-results.json").open("x") as stream:
    json.dump(report, stream, indent=2)
    stream.write("\n")
print(json.dumps({row["lane"]: {k: row[k] for k in ("run", "passed", "failed", "timed_out", "flaky", "leaky", "skipped", "target_test_outcomes")} for row in lanes}))

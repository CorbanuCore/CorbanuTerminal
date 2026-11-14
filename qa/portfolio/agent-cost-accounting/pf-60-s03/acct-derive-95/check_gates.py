"""Derive gate counts and digests from this allocation\'s actual guarded logs."""
import gzip
import hashlib
import json
from pathlib import Path
import re

here = Path(__file__).resolve().parent
lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    row = json.loads((here / "gates-01" / (name + ".json")).read_bytes())
    raw = gzip.decompress((here / "gates-01" / (name + ".log.gz")).read_bytes())
    if hashlib.sha256(raw).hexdigest() != row["log_sha256"] or row["exit"] != 0:
        raise RuntimeError("failed lane or log mismatch: " + name)
    if row["failure_lines"] or row["environment"]["NEXTEST_TEST_THREADS"] != "4":
        raise RuntimeError("failure or wrong concurrency: " + name)
    if name != "prerequisites":
        if row["command"][:2] != ["just", "test"] or b"Test isolation: disposable profile; native keyring disabled (debug lane)" not in raw:
            raise RuntimeError("missing guarded-test evidence")
        matches = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", raw.decode(), re.M)
        if len(matches) != 1:
            raise RuntimeError("ambiguous test summary")
        count, detail = matches[0]
        if int(count) != row["run"] or row["run"] != row["passed"] or row["run"] <= 0:
            raise RuntimeError("run/pass mismatch")
        for key, label in (("passed", "passed"), ("failed", "failed"), ("timed_out", "timed out"),
                           ("flaky", "flaky"), ("skipped", "skipped"), ("leaky", "leaky")):
            match = re.search(r"(\d+) " + label, detail)
            if row[key] != (int(match[1]) if match else 0):
                raise RuntimeError("summary count mismatch: " + key)
    lanes.append(row)
if len({row["environment"]["CARGO_TARGET_DIR"] for row in lanes}) != 1:
    raise RuntimeError("targets differ")
result = dict(lanes=lanes, overlapping_test_executions=sum(row.get("run", 0) for row in lanes))
(here / "test-results.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps({row["lane"]: {key: row[key] for key in ("exit", "run", "passed", "skipped", "failure_lines") if key in row} for row in lanes}, indent=2))

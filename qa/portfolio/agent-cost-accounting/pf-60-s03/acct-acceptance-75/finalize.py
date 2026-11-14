"""Verify new guarded logs and preserve derived lane counts without rerunning tests."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
base = "49e7d1b8cf6d762c433360e54efccd50a0840f83"
lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    row = json.loads((here / "gates-01" / (name + ".json")).read_text())
    data = gzip.decompress((here / "gates-01" / (name + ".log.gz")).read_bytes())
    assert hashlib.sha256(data).hexdigest() == row["log_sha256"]
    assert row["base"] == base and row["exit"] == 0
    assert not row["failure_lines"]
    if name != "prerequisites":
        assert b"Test isolation: disposable profile; native keyring disabled (debug lane)" in data
        summaries = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", data.decode(), re.M)
        assert len(summaries) == 1
        run, details = summaries[0]
        assert row["run"] == int(run) == row["passed"] > 0
        for key, label in (("passed", "passed"), ("failed", "failed"),
                           ("timed_out", "timed out"), ("flaky", "flaky"),
                           ("skipped", "skipped"), ("leaky", "leaky")):
            match = re.search(r"(\d+) " + label, details)
            assert row[key] == (int(match[1]) if match else 0)
        assert all(row[key] == 0 for key in ("failed", "timed_out", "flaky", "leaky"))
    lanes.append(row)
receipt = dict(base=base, lanes=lanes,
               total_executions=sum(row["run"] for row in lanes[1:]),
               failure_names=[], unique_test_count_claimed=False)
(here / "test-results.json").write_text(json.dumps(receipt, indent=2) + "\n")
summary = here.parent / "acct-controls-72/summarize.py"
result = subprocess.run(["python3", "-B", str(summary)], cwd=repo,
                        capture_output=True, text=True)
assert result.returncode == 0, result.stderr
(here / "summary-recheck.json").write_text(json.dumps(dict(
    command=["python3", "-B", str(summary.relative_to(repo))], exit=result.returncode,
    script_sha256=hashlib.sha256(summary.read_bytes()).hexdigest(),
    counts=json.loads(result.stdout),
    evidence="Validates retained round-72 evidence; not a new mutation execution."
), indent=2) + "\n")
print(json.dumps(dict(total_executions=receipt["total_executions"],
                      lanes={row["lane"]: row["run"] for row in lanes[1:]},
                      failure_names=[])))

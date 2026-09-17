"""Validate raw gate evidence and derive exact counts without replaying tests."""
import ast
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
base = "1aadfc123592530f657794489249e954f5e3885a"
lanes = []
for name in ("prerequisites", "core-default", "alone-feature", "core-feature", "tui"):
    row = json.loads((here / "gates-01" / (name + ".json")).read_text())
    data = gzip.decompress((here / "gates-01" / (name + ".log.gz")).read_bytes())
    assert hashlib.sha256(data).hexdigest() == row["log_sha256"]
    assert row["base"] == base
    if name != "prerequisites":
        assert b"Test isolation: disposable profile; native keyring disabled (debug lane)" in data
        summaries = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", data.decode(), re.M)
        assert len(summaries) == 1
        run, details = summaries[0]
        assert row["run"] == int(run) > 0
        for key, label in (("passed", "passed"), ("failed", "failed"), ("timed_out", "timed out"),
                           ("flaky", "flaky"), ("skipped", "skipped"), ("leaky", "leaky")):
            match = re.search(r"(\d+) " + label, details)
            assert row[key] == (int(match[1]) if match else 0)
    lanes.append(row)
gates = [row for row in lanes if row["lane"] in ("core-default", "core-feature", "tui")]
receipt = dict(base=base, lanes=lanes, gate_executions=sum(row["run"] for row in gates),
               gate_passed=sum(row["passed"] for row in gates),
               alone_executions=next(row["run"] for row in lanes if row["lane"] == "alone-feature"),
               failure_lines=[line for row in lanes for line in row["failure_lines"]],
               historical_timeout_status="Round 66 feature TRY 1/2 timeouts remain historical failures; cause unproven, load sensitivity inferred from round 69 and current alone/in-lane comparisons. No causal fix claimed.",
               unique_test_count_claimed=False)
(here / "test-results.json").write_text(json.dumps(receipt, indent=2) + "\n")
for name in ("acct-controls-72", "acct-acceptance-75"):
    document = json.loads((here.parent / name / "scope.json").read_text())
    entries = document.get("files", document.get("new_files_excluding_this_inventory"))
    for row in entries:
        data = (repo / row["path"]).read_bytes()
        assert len(data) == row["bytes"]
        assert hashlib.sha256(data).hexdigest() == row["sha256"], row["path"]
package = json.loads((here / "package-manifest.json").read_text())
assert package["exit"] == 0 and package["base"] == base and not package["launched"]
package_log = gzip.decompress((here / "package-build.log.gz").read_bytes())
assert hashlib.sha256(package_log).hexdigest() == package["log_sha256"]
for row in package["files"]:
    path = repo / row["path"]
    assert path.stat().st_size == row["bytes"]
    assert path.stat().st_mode & 0o777 == 0o555
    with path.open("rb") as stream:
        assert hashlib.file_digest(stream, "sha256").hexdigest() == row["sha256"]
for path in here.glob("*.py"):
    ast.parse(path.read_text(), filename=str(path))
assert subprocess.run(["git", "diff", "--check"], cwd=repo).returncode == 0
print(json.dumps({"gate_executions": receipt["gate_executions"], "gate_passed": receipt["gate_passed"],
                  "alone_executions": receipt["alone_executions"], "failure_lines": receipt["failure_lines"],
                  "inventories_verified": 2}))

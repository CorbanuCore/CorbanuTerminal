"""Verify fresh lane receipts against raw logs and record the final QA file scope."""
import ast
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
base = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    row = json.loads((here / "gates-01" / (name + ".json")).read_text())
    raw = gzip.decompress((here / "gates-01" / (name + ".log.gz")).read_bytes())
    text = raw.decode()
    assert hashlib.sha256(raw).hexdigest() == row["log_sha256"]
    assert row["base"] == base and row["exit"] == 0 and not row["failure_lines"]
    assert row["environment"]["NEXTEST_TEST_THREADS"] == "4"
    if name != "prerequisites":
        assert row["command"][:2] == ["just", "test"]
        assert "Test isolation: disposable profile; native keyring disabled (debug lane)" in text
        summary = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", text, re.M)
        assert len(summary) == 1
        run, details = summary[0]
        assert int(run) == row["run"] == row["passed"] > 0
        for key, label in (("passed", "passed"), ("failed", "failed"),
                           ("timed_out", "timed out"), ("flaky", "flaky"),
                           ("skipped", "skipped"), ("leaky", "leaky")):
            found = re.search(r"(\d+) " + label, details)
            assert row[key] == (int(found[1]) if found else 0)
        assert not re.search(r"^\s*(?:TRY\s+\d+\s+)?(?:FAIL|TIMEOUT|TMT|XPASS|LEAK FAIL)\s", text, re.M)
    lanes.append(row)
receipt = dict(base=base, lanes=lanes, executions=sum(row["run"] for row in lanes[1:]),
               failure_names=[], unique_tests_claimed=False,
               qualification="Guarded Rust filters; not independent functional acceptance.")
(here / "test-results.json").write_text(json.dumps(receipt, indent=2) + "\n")
controls = json.loads((here / "proposed-controls/verifier-checks.json").read_text())
assert controls["source_base"] == base
assert len(controls["cases"]) == 11 and all(row["passed"] for row in controls["cases"])
snapshot = Path(controls["run_directory"]) / "checkout"
tested_files = {}
for relative in (
    "acct-inventory-79/verify_acceptance.py", "acct-inventory-79/check_verifier.py",
    "acct-selfcheck-84/refresh_inventory.py", "acct-selfcheck-84/worded-quantity-residual.md",
    "acct-acceptance-75/acceptance.md", "acct-acceptance-75/scope.json",
    "acct-inventory-79/inventory-correction.json", "acct-baseline-81/package-exclusion.md",
):
    path = here.parent / relative
    raw = path.read_bytes()
    assert raw == (snapshot / path.relative_to(repo)).read_bytes()
    tested_files[str(path.relative_to(repo))] = hashlib.sha256(raw).hexdigest()
command = ["python3", "-B", str(here.parent / "acct-inventory-79/verify_acceptance.py")]
result = subprocess.run(command, cwd=repo, capture_output=True)
assert result.returncode == 2 and not result.stderr
assert result.stdout == (here / "proposed-controls/clean-checkout.stdout.txt").read_bytes()
(here / "final-tree-check.json").write_text(json.dumps(dict(
    base=base, proposed_snapshot=controls["snapshot"], compared_files_sha256=tested_files,
    command=command, exit=result.returncode, stderr_empty=True,
    stdout_matches_proposed_control=True,
    stdout_sha256=hashlib.sha256(result.stdout).hexdigest(),
), indent=2) + "\n")
for path in list(here.glob("*.py")) + [
        here.parent / "acct-inventory-79" / name for name in
        ("verify_acceptance.py", "check_verifier.py")]:
    ast.parse(path.read_text(), filename=str(path))
subprocess.run(["git", "diff", "--check"], cwd=repo, check=True)
changed = subprocess.check_output(["git", "diff", "--numstat", "HEAD"], cwd=repo, text=True)
untracked = subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard"],
                                    cwd=repo, text=True).splitlines()
files = []
for name in sorted(set(untracked + [line.split("\t", 2)[2] for line in changed.splitlines()])):
    path = repo / name
    assert path.is_relative_to(here.parent), "Changed outside the assignment: " + name
    if path == here / "scope.json":
        continue
    raw = path.read_bytes()
    files.append(dict(path=name, bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest(),
                      lines=None if path.suffix == ".gz" else len(raw.splitlines()),
                      kind="new" if name in untracked else "modified"))
scope = dict(base=base, tracked_numstat=changed, files=files, self_excluded=True,
             new_files=sum(row["kind"] == "new" for row in files),
             new_bytes=sum(row["bytes"] for row in files if row["kind"] == "new"),
             new_text_lines=sum(row["lines"] or 0 for row in files if row["kind"] == "new"))
(here / "scope.json").write_text(json.dumps(scope, indent=2) + "\n")
print(json.dumps(dict(executions=receipt["executions"], failure_names=[],
                     tracked_numstat=changed, new_files=scope["new_files"],
                     new_bytes=scope["new_bytes"], new_text_lines=scope["new_text_lines"]), indent=2))

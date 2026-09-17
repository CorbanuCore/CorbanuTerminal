"""Validate retained raw receipts, final tested inputs and allocation-only writes."""
import ast
import gzip
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
base = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
landing = json.loads((here / "landing-demonstration.json").read_text())
assert base == landing["source_base"]
for name, expected in landing["proposed_files_sha256"].items():
    assert hashlib.sha256((repo / name).read_bytes()).hexdigest() == expected, name
reference = json.loads((here / "reference.json").read_text())
expected = (here / "expected.stdout.txt").read_bytes()
assert hashlib.sha256(expected).hexdigest() == reference["sha256"]
replays = []
for label, commit in (("landed", landing["landed"]), ("advanced", landing["advanced"])):
    directory = here / label
    receipt = json.loads((directory / "receipt.json").read_text())
    assert receipt["commit"] == receipt["source_tip_after"] == commit
    assert receipt["reference_source_commit"] == commit
    assert receipt["status_before"] == receipt["status_after"] == ""
    assert receipt["reference_sha256"] == reference["sha256"]
    assert receipt["stdout_matches_prior_replay"]
    assert (directory / "acceptance.stdout.txt").read_bytes() == expected
    for row in receipt["commands"]:
        assert row["exit"] == row["expected_exit"] and row["stderr_empty"]
        for stream in ("stdout", "stderr"):
            raw = (directory / (row["name"] + "." + stream + ".txt")).read_bytes()
            assert hashlib.sha256(raw).hexdigest() == row[stream + "_sha256"]
    # Preserve each raw verifier-control attempt from the disposable checkout.
    raw = (directory / "controls.stdout.txt").read_text()
    first, body = raw.split("\n", 1)
    assert first.startswith("Run directory: ")
    source = Path(first.removeprefix("Run directory: "))
    controls = json.loads(body)
    assert len(controls["cases"]) == 13 and all(row["passed"] for row in controls["cases"])
    destination = directory / "controls"
    destination.mkdir(exist_ok=True)
    for path in source.iterdir():
        if path.is_file():
            shutil.copyfile(path, destination / path.name)
    for row in controls["cases"]:
        raw = (destination / (row["case"] + ".stdout.txt")).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == row["stdout_sha256"]
        assert not (destination / (row["case"] + ".stderr.txt")).read_bytes()
    replays.append(dict(label=label, commit=commit, controls_passed=len(controls["cases"])))
controls = json.loads((here / "receipt-controls.json").read_text())
assert len(controls["assertions"]) == 5 and controls["negative_controls"] == 9
assert len(controls["cases"]) == 10 and all(row["passed"] for row in controls["cases"])
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
assert len({row["environment"]["CARGO_TARGET_DIR"] for row in lanes}) == 1
(here / "test-results.json").write_text(json.dumps(dict(
    base=base, lanes=lanes, executions=sum(row["run"] for row in lanes[1:]),
    failure_names=[], unique_tests_claimed=False,
    qualification="Guarded Rust filters; not independent functional acceptance."
), indent=2) + "\n")
result = subprocess.run(["python3", "-B", str(here.parent / "acct-inventory-79/verify_acceptance.py")],
                        cwd=repo, capture_output=True)
assert result.returncode == 2 and not result.stderr and result.stdout == expected
(here / "final-tree-check.json").write_text(json.dumps(dict(
    base=base, proposed_files_match_landing_snapshot=True, replays=replays,
    final_verifier_exit=result.returncode, final_stdout_sha256=hashlib.sha256(result.stdout).hexdigest(),
    final_stdout_matches_content_reference=True, receipt_assertions_covered=5,
    receipt_negative_controls_passed=9,
), indent=2) + "\n")
for path in [*here.glob("*.py"), here.parent / "acct-selfcheck-84/check_current_tip.py"]:
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
print(json.dumps(dict(executions=sum(row["run"] for row in lanes[1:]), failure_names=[],
                     tracked_numstat=changed, new_files=scope["new_files"],
                     new_bytes=scope["new_bytes"], new_text_lines=scope["new_text_lines"]), indent=2))

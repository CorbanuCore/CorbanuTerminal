"""Validate retained receipts and final evidence; never refresh an expected result."""
import ast
import gzip
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = here.parent


def require(condition, detail):
    if not condition:
        raise RuntimeError(detail)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


# Preserve full helper attempts and nested verifier/control streams for review.
run = Path(sys.argv[1])
replay = json.loads((run / "replay-checks.json").read_text())
destination = here / "replay-results"
destination.mkdir(exist_ok=True)
for path in run.glob("*.txt"):
    shutil.copyfile(path, destination / path.name)
shutil.copyfile(run / "replay-checks.json", destination / "replay-checks.json")
for case in replay["cases"]:
    saved = destination / case["case"]
    saved.mkdir(exist_ok=True)
    source = Path(case["receipt"]["run_directory"])
    for path in source.glob("*.txt"):
        shutil.copyfile(path, saved / path.name)
    for row in case["receipt"]["commands"]:
        for stream in ("stdout", "stderr"):
            raw = (saved / (row["name"] + "." + stream + ".txt")).read_bytes()
            require(sha(raw) == row[stream + "_sha256"], "replay stream digest differs")
    controls = json.loads((saved / "controls.stdout.txt").read_text().split("\n", 1)[1])
    require(len(controls["cases"]) == 13 and all(r["passed"] for r in controls["cases"]),
            "verifier control failure")
require([r["exit"] for r in replay["cases"]] == [0, 0, 1, 1], "helper exits differ")

lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    row = json.loads((here / "gates-01" / (name + ".json")).read_text())
    raw = gzip.decompress((here / "gates-01" / (name + ".log.gz")).read_bytes())
    require(sha(raw) == row["log_sha256"], name + ": log digest")
    require(row["exit"] == 0 and not row["failure_lines"], name + ": gate failure")
    require(row["environment"]["NEXTEST_TEST_THREADS"] == "4", name + ": thread limit")
    if name != "prerequisites":
        text = raw.decode()
        require(row["command"][:2] == ["just", "test"], name + ": unguarded tests")
        require("Test isolation: disposable profile; native keyring disabled (debug lane)" in text,
                name + ": no isolation banner")
        matches = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", text, re.M)
        require(len(matches) == 1, name + ": missing/ambiguous summary")
        count, details = matches[0]
        require(int(count) == row["run"] == row["passed"] > 0, name + ": nonzero passes")
        for key, label in (("passed", "passed"), ("failed", "failed"),
                           ("timed_out", "timed out"), ("flaky", "flaky"),
                           ("skipped", "skipped"), ("leaky", "leaky")):
            found = re.search(r"(\d+) " + label, details)
            require(row[key] == (int(found[1]) if found else 0), name + ": count differs")
    lanes.append(row)
require(len({r["environment"]["CARGO_TARGET_DIR"] for r in lanes}) == 1, "target differs")
(here / "test-results.json").write_text(json.dumps(dict(lanes=lanes), indent=2) + "\n")

expected = (scope / "acct-reference-88/expected.stdout.txt").read_bytes()
reference = json.loads((scope / "acct-reference-88/reference.json").read_text())
require(sha(expected) == reference["sha256"], "reference digest")
result = subprocess.run([sys.executable, "-B", str(scope / "acct-inventory-79/verify_acceptance.py")],
                        cwd=repo, capture_output=True)
require(result.returncode == 2 and not result.stderr and result.stdout == expected,
        "final baseline differs")
document = (scope / "acct-acceptance-75/acceptance.md").read_text()
mutated = (here / "simulation-final/unrefreshed-edit.stdout.txt").read_text()
require("\n```text\n" + mutated + "```\n" in document, "verbatim simulation missing")
tree = ast.parse((scope / "acct-selfcheck-84/check_current_tip.py").read_text())
require(not any(isinstance(node, ast.Assert) for node in ast.walk(tree)), "bare replay assert")
for name in ("receipt-controls.json", "receipt-controls-optimized.json"):
    controls = json.loads((here / name).read_text())
    require(len(controls["cases"]) == 10 and all(r["passed"] for r in controls["cases"]),
            "receipt control failure")
status = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"],
                                 cwd=repo, text=True)
require(all(line[3:].startswith(str(scope.relative_to(repo)) + "/")
            for line in status.splitlines()), "out-of-scope change")
record = dict(source_base=subprocess.check_output(["git", "rev-parse", "HEAD"],
              cwd=repo, text=True).strip(), reference_sha256=reference["sha256"],
              baseline_exit=result.returncode, verbatim_simulation_matches=True,
              bare_asserts_in_replay=0, helper_exits=[r["exit"] for r in replay["cases"]],
              receipt_controls_per_mode=10, verifier_controls_per_helper_run=13,
              all_changes_in_scope=True)
(here / "final-check.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record, indent=2))
names = subprocess.check_output(["git", "diff", "--name-only", "HEAD"], cwd=repo, text=True).splitlines()
names += subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard"],
                                 cwd=repo, text=True).splitlines()
files = []
for name in sorted(set(names)):
    path = repo / name
    if path == here / "scope.json" or not path.is_file():
        continue
    raw = path.read_bytes()
    files.append(dict(path=name, bytes=len(raw), sha256=sha(raw),
                      lines=None if path.suffix == ".gz" else len(raw.splitlines())))
(here / "scope.json").write_text(json.dumps(dict(base=record["source_base"],
    self_excluded=True, files=files), indent=2) + "\n")

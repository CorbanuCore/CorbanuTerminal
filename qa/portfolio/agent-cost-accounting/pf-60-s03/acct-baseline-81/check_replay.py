"""Re-run the control harness twice from a clean committed proposed snapshot."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = here.parent.relative_to(repo)
snapshot = Path(sys.argv[1]).resolve()
assert snapshot.is_relative_to(here.parent / "acct-inventory-79/target")
env = dict(os.environ, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
           GIT_TERMINAL_PROMPT="0")
def git(*args):
    return subprocess.check_output(["git", *args], cwd=snapshot, env=env, text=True)
assert git("status", "--porcelain", "--untracked-files=all") == ""
tracked_outputs = list((snapshot / scope / "acct-inventory-79").glob("*.txt"))
before = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in tracked_outputs}
runs = []
for index in (1, 2):
    output = here / ("replay-" + str(index))
    output.mkdir()
    result = subprocess.run(
        ["python3", "-B", str(scope / "acct-inventory-79/check_verifier.py")],
        cwd=snapshot, env=env, capture_output=True, text=True)
    (output / "harness.stdout.txt").write_text(result.stdout)
    (output / "harness.stderr.txt").write_text(result.stderr)
    assert result.returncode == 0, result.stderr
    run = Path(result.stdout.splitlines()[0].removeprefix("Run directory: "))
    receipt = json.loads((run / "verifier-checks.json").read_text())
    for path in run.iterdir():
        if path.is_file():
            shutil.copyfile(path, output / path.name)
    assert all(case["passed"] for case in receipt["cases"])
    assert git("status", "--porcelain", "--untracked-files=all") == ""
    runs.append(dict(index=index, run_directory=str(run), exit=result.returncode,
                     cases=len(receipt["cases"]), source_clean=True))
assert runs[0]["run_directory"] != runs[1]["run_directory"]
after = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in tracked_outputs}
assert before == after
# Remove one row of each kind in the disposable snapshot to test derived counts.
inventory = snapshot / scope / "acct-acceptance-75/scope.json"
correction = snapshot / scope / "acct-inventory-79/inventory-correction.json"
saved = {p: p.read_bytes() for p in (inventory, correction)}
try:
    document = json.loads(saved[inventory])
    for kind in ("new", "modified"):
        document["files"].remove(next(r for r in document["files"] if r["kind"] == kind))
        rows = [r for r in document["files"] if r["kind"] == kind]
        for key, value in (("files", len(rows)), ("bytes", sum(r["bytes"] for r in rows)),
                           ("text_lines", sum(r["lines"] or 0 for r in rows))):
            document["total_" + kind + "_" + key] = value
    document["total_current_bytes"] = sum(r["bytes"] for r in document["files"])
    document["total_current_text_lines"] = sum(r["lines"] or 0 for r in document["files"])
    inventory.write_text(json.dumps(document, indent=2) + "\n")
    amended = json.loads(saved[correction])
    amended["after_sha256"] = hashlib.sha256(inventory.read_bytes()).hexdigest()
    correction.write_text(json.dumps(amended, indent=2) + "\n")
    result = subprocess.run(
        ["python3", "-B", str(scope / "acct-inventory-79/verify_acceptance.py")],
        cwd=snapshot, env=env, capture_output=True, text=True)
    (here / "derived-counts.stdout.txt").write_text(result.stdout)
    (here / "derived-counts.stderr.txt").write_text(result.stderr)
    assert result.returncode == 2 and not result.stderr
    assert "AGREE inventory classifications and correction: 14 added paths and 2 modified paths" in result.stdout
finally:
    for path, raw in saved.items():
        path.write_bytes(raw)
assert git("status", "--porcelain", "--untracked-files=all") == ""
receipt = dict(snapshot=git("rev-parse", "HEAD").strip(), runs=runs,
               tracked_output_hashes_before=before, tracked_output_hashes_after=after,
               preserved_existing_outputs=True, derived_count_control="14 new / 2 modified",
               clean_after=True, independent_functional_acceptance=False)
(here / "replay-checks.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps(dict(runs=runs, preserved_existing_outputs=True,
                      derived_count_control="14 new / 2 modified"), indent=2))

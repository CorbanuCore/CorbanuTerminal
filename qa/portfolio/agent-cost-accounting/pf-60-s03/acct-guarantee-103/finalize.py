"""Derive final scope and verify the concrete evidence used in RETURN."""
import ast
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = here.parent
base = "2731b5e8868cb7222e4fedef3493ab977ecc79db"
def require(condition, message):
    if not condition:
        raise RuntimeError(message)
def sha(raw):
    return hashlib.sha256(raw).hexdigest()
def git(*args):
    return subprocess.check_output(["git", *args], cwd=repo).decode()
require(git("rev-parse", "HEAD").strip() == base, "base changed")
subprocess.run(["git", "diff", "--check"], cwd=repo, check=True)
prefix = str(scope.relative_to(repo)) + "/"
paths = set(git("diff", "--name-only", "-z", base).split("\0")[:-1])
paths.update(git("ls-files", "--others", "--exclude-standard", "-z").split("\0")[:-1])
require(paths and all(p.startswith(prefix) for p in paths), "outside scope")
for path in list(here.glob("*.py")) + [scope / "acct-final-100/test_destination.py"]:
    ast.parse(path.read_text(), filename=str(path))
checks = json.loads((here / "checks.json").read_text())
require(checks["witness_sha256"] == sha((scope / "acct-final-100/test_destination.py").read_bytes()),
        "witness changed after execution")
note = (scope / "acct-derive-95/OPERATOR.md").read_bytes()
blocks = re.findall(r"^```sh\n(.*?)^```$", note.decode(), re.M | re.S)
guard = blocks[0][blocks[0].index("audit=${CORBANU_AUDIT_DIR"):blocks[0].index("git clone --no-local")]
guard_runs = [r for r in checks["runs"] if r["name"].startswith("guard-") and "version" not in r["name"]]
require(len(guard_runs) == 9, "destination count")
for row in guard_runs:
    require(row["command"][-1] == "set -eu\nq=unused\n" + guard + "printf 'DESTINATION_ALLOWED\\n'\n",
            "exercised guard differs from final note")
for name in ("witness-clean", "witness-ambient-git"):
    row = json.loads((here / (name + ".json")).read_text())
    alias = next(r for r in row["cases"] if r["case"] == "alias-unignored-regression")
    require((alias["old_guard_exit"], alias["exit"]) == (0, 1), "witness observations")
    require(row["passed"] == 7 and all(r["exit"] == r["expected_exit"] for r in row["cases"]), "witness cases")
    summary = json.loads((here / (name + ".stdout.txt")).read_text().splitlines()[-1])
    require((summary["old_alias_exit"], summary["new_alias_exit"]) ==
            (alias["old_guard_exit"], alias["exit"]), "printed observations differ")
reference = (scope / "acct-reference-88/expected.stdout.txt").read_bytes()
require(reference == (here / "acceptance-final.stdout.txt").read_bytes(), "reference differs")
require(sha(reference) == json.loads((scope / "acct-reference-88/reference.json").read_text())["sha256"],
        "reference digest")
require(not (here / "acceptance-final.stderr.txt").read_bytes(), "acceptance stderr")
controls = (here / "controls-final.stdout.txt").read_text()
control_record = json.loads(controls[controls.index("{"):])
require(len(control_record["cases"]) == 13 and all(r["passed"] and r["exit"] == r["expected_exit"]
        for r in control_record["cases"]), "controls")
require(not (here / "controls-final.stderr.txt").read_bytes(), "controls stderr")
lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    row = json.loads((here / "gates-01" / (name + ".json")).read_text())
    raw = gzip.decompress((here / "gates-01" / (name + ".log.gz")).read_bytes())
    require(row["log_sha256"] == sha(raw) and row["base"] == base and row["exit"] == 0, name)
    require(row["cwd"] == str(repo / "codex-rs") and row["environment"]["NEXTEST_TEST_THREADS"] == "4", name)
    if name != "prerequisites":
        require(row["command"][:2] == ["just", "test"] and row["isolation_banner"], name)
        require(row["run"] > 0 and row["passed"] == row["run"], name)
        require(row["failure_lines"] == [] and all(row[k] == 0 for k in ("failed", "timed_out", "flaky", "leaky")), name)
        observed = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", raw.decode(), re.M)
        require(len(observed) == 1 and int(observed[0][0]) == row["run"], name)
        require(int(re.search(r"(\d+) passed", observed[0][1])[1]) == row["passed"], name)
        require(int(re.search(r"(\d+) skipped", observed[0][1])[1]) == row["skipped"], name)
    lanes.append(row)
require(len({r["environment"]["CARGO_TARGET_DIR"] for r in lanes}) == 1, "target differs")
record = dict(base=base, all_changed_paths_within_scope=True, diff_check_exit=0,
              final_note_sha256=sha(note), final_guard_matches_executed_segment=True,
              witness_observed_exits_match_printed_summary=True, acceptance_exit=2,
              acceptance_stdout_sha256=sha(reference), controls_passed=len(control_record["cases"]),
              lanes=lanes)
with (here / "final-check.json").open("x") as stream:
    json.dump(record, stream, indent=2)
    stream.write("\n")
paths.add(str((here / "final-check.json").relative_to(repo)))
numstat = {}
for line in git("diff", "--numstat", base).splitlines():
    added, removed, path = line.split("\t")
    numstat[path] = (int(added), int(removed))
files = []
for path in sorted(paths):
    if path == str((here / "scope.json").relative_to(repo)):
        continue
    raw = (repo / path).read_bytes()
    lines = None if path.endswith(".gz") else len(raw.splitlines())
    added, removed = numstat.get(path, (lines, 0))
    files.append(dict(path=path, kind="modified" if path in numstat else "added",
                      added_lines=added, removed_lines=removed, text_lines=lines,
                      bytes=len(raw), sha256=sha(raw)))
summary = dict(base=base, self_entry_excluded=True, files=files,
               modified_added=sum(a for a, r in numstat.values()),
               modified_removed=sum(r for a, r in numstat.values()),
               new_text_lines=sum(r["text_lines"] or 0 for r in files if r["kind"] == "added"))
with (here / "scope.json").open("x") as stream:
    json.dump(summary, stream, indent=2)
    stream.write("\n")
print(json.dumps({k: summary[k] for k in ("modified_added", "modified_removed", "new_text_lines")}))

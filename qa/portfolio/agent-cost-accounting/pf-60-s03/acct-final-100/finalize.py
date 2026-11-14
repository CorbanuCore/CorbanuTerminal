"""Record this revision's checks and changed-file accounting, not prose verdicts."""
import ast
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
prefix = str(here.parent.relative_to(repo)) + "/"
base = "967425cc72ea28b5f59ecc38a56419b7db6885e8"

def git(*args):
    return subprocess.check_output(["git", *args], cwd=repo).decode()

assert git("rev-parse", "HEAD").strip() == base
paths = sorted(set(git("diff", "--name-only", "-z", base).split("\0")[:-1]
                   + git("ls-files", "--others", "--exclude-standard", "-z").split("\0")[:-1]))
assert paths and all(path.startswith(prefix) for path in paths), paths
subprocess.run(["git", "diff", "--check"], cwd=repo, check=True)
for path in here.glob("*.py"):
    ast.parse(path.read_text(), filename=str(path))
note = (here.parent / "acct-derive-95/OPERATOR.md").read_bytes()
blocks = re.findall(r"^```sh\n(.*?)^```$", note.decode(), re.M | re.S)
assert len(blocks) == 6
for block in blocks:
    subprocess.run(["bash", "--noprofile", "--norc", "-n"], input=block,
                   text=True, check=True)
destination = json.loads((here / "destination-results.json").read_text())
assert destination["note_sha256"] == hashlib.sha256(note).hexdigest()
assert destination["passed"] == 7
assert all(row["exit"] == row["expected_exit"] for row in destination["cases"])
lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    row = json.loads((here / "gates-01" / (name + ".json")).read_text())
    raw = gzip.decompress((here / "gates-01" / (name + ".log.gz")).read_bytes())
    assert row["log_sha256"] == hashlib.sha256(raw).hexdigest()
    assert row["exit"] == 0 and row["base"] == base
    assert row["cwd"] == str(repo / "codex-rs")
    assert row["environment"]["NEXTEST_TEST_THREADS"] == "4"
    if name != "prerequisites":
        assert row["command"][:2] == ["just", "test"]
        assert row["isolation_banner"] and row["run"] > 0
        assert row["passed"] == row["run"] and row["failure_lines"] == []
        assert all(row[key] == 0 for key in ("failed", "timed_out", "flaky", "leaky"))
    lanes.append(row)
assert len({row["environment"]["CARGO_TARGET_DIR"] for row in lanes}) == 1
check = dict(base=base, scope=prefix, all_changes_within_scope=True,
             diff_check_exit=0, parsed_python_files=[p.name for p in sorted(here.glob("*.py"))],
             shell_blocks_syntax_checked=len(blocks), destination_cases_passed=7,
             lanes=lanes, scope_paths_before_generated_receipts=paths)
with (here / "final-check.json").open("x") as output:
    json.dump(check, output, indent=2)
    output.write("\n")
existing = {}
for row in git("diff", "--numstat", base).splitlines():
    added, deleted, path = row.split("\t")
    existing[path] = (int(added), int(deleted))
paths = sorted(set(paths + [str((here / "final-check.json").relative_to(repo))]))
files = []
for name in paths:
    raw = (repo / name).read_bytes()
    text_lines = None if name.endswith(".gz") else len(raw.splitlines())
    added, deleted = existing.get(name, (text_lines, 0))
    files.append(dict(path=name, kind="modified" if name in existing else "added",
                      added_lines=added, deleted_lines=deleted,
                      current_text_lines=text_lines, bytes=len(raw),
                      sha256=hashlib.sha256(raw).hexdigest()))
summary = dict(base=base, self_entry_excluded=True, files=files,
               modified_added=sum(v[0] for v in existing.values()),
               modified_deleted=sum(v[1] for v in existing.values()),
               new_text_lines=sum(row["current_text_lines"] or 0 for row in files if row["kind"] == "added"),
               new_binary_bytes=sum(row["bytes"] for row in files if row["kind"] == "added" and row["current_text_lines"] is None))
with (here / "scope.json").open("x") as output:
    json.dump(summary, output, indent=2)
    output.write("\n")
print(json.dumps({key: summary[key] for key in (
    "modified_added", "modified_deleted", "new_text_lines", "new_binary_bytes")}))

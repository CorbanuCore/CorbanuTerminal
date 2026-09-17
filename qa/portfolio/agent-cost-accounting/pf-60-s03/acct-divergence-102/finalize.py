"""Validate final-tree evidence and record scoped file/line/hash accounting."""
import ast
import gzip
import hashlib
import json
from pathlib import Path
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
base = "a8dfff98892e60aaa0d7f05321fd7e57b79cdc2a"
prefix = str(here.parent.relative_to(repo)) + "/"
def git(*args):
    return subprocess.check_output(["git", *args], cwd=repo).decode()
def sha(raw):
    return hashlib.sha256(raw).hexdigest()
assert git("rev-parse", "HEAD").strip() == base
subprocess.run(["git", "diff", "--check"], cwd=repo, check=True)
paths = set(git("diff", "--name-only", "-z", base).split("\0")[:-1])
paths.update(git("ls-files", "--others", "--exclude-standard", "-z").split("\0")[:-1])
assert paths and all(path.startswith(prefix) for path in paths)
python_files = list(here.glob("*.py")) + [here.parent / "acct-final-100/test_destination.py"]
for path in python_files:
    ast.parse(path.read_text(), filename=str(path))
note = json.loads((here / "note-check-final.json").read_text())
assert note["note_sha256"] == sha((here.parent / "acct-derive-95/OPERATOR.md").read_bytes())
assert [r["block"] for r in note["blocks"] if not r["equal"]] == [1]
witness = json.loads((here / "pinned-destination-results.json").read_text())
assert witness["passed"] == 7
note_path = prefix + "acct-derive-95/OPERATOR.md"
for commit_key, hash_key in [("old_commit", "old_note_sha256"), ("new_commit", "note_sha256")]:
    raw = subprocess.check_output(["git", "show", witness[commit_key] + ":" + note_path], cwd=repo)
    assert sha(raw) == witness[hash_key]
assert witness["note_sha256"] == json.loads((here.parent / "acct-final-100/destination-results.json").read_text())["note_sha256"]
lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    row = json.loads((here / "gates-01" / (name + ".json")).read_text())
    raw = gzip.decompress((here / "gates-01" / (name + ".log.gz")).read_bytes())
    assert row["log_sha256"] == sha(raw) and row["base"] == base and row["exit"] == 0
    assert row["cwd"] == str(repo / "codex-rs")
    assert row["environment"]["NEXTEST_TEST_THREADS"] == "4"
    if name != "prerequisites":
        assert row["command"][:2] == ["just", "test"]
        assert row["isolation_banner"] and row["run"] > 0 and row["passed"] == row["run"]
        assert row["failure_lines"] == []
        assert all(row[key] == 0 for key in ("failed", "timed_out", "flaky", "leaky"))
    lanes.append(row)
assert len({r["environment"]["CARGO_TARGET_DIR"] for r in lanes}) == 1
check = dict(base=base, scope=prefix, all_changes_within_scope=True, diff_check_exit=0,
             parsed_python_files=[str(p.relative_to(repo)) for p in python_files],
             note_sha256=note["note_sha256"], historical_witness_hashes_match=True, lanes=lanes)
with (here / "final-check.json").open("x") as output:
    json.dump(check, output, indent=2)
    output.write("\n")
paths.add(str((here / "final-check.json").relative_to(repo)))
numstat = {}
for line in git("diff", "--numstat", base).splitlines():
    added, removed, path = line.split("\t")
    numstat[path] = (int(added), int(removed))
files = []
for path in sorted(paths):
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
with (here / "scope.json").open("x") as output:
    json.dump(summary, output, indent=2)
    output.write("\n")
print(json.dumps({key: summary[key] for key in ("modified_added", "modified_removed", "new_text_lines")}))

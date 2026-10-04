"""Finalize the change ledger without rerunning or changing acceptance inputs."""
import ast
import hashlib
import json
from pathlib import Path
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
base = "6a7128ccc4e10c92dd583ca65b62e70d84483012"
prefix = str(here.parent.relative_to(repo)) + "/"

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

def git(*args):
    return subprocess.check_output(["git", *args], cwd=repo)

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

require(git("rev-parse", "HEAD").decode().strip() == base, "source base changed")
subprocess.run(["git", "diff", "--check"], cwd=repo, check=True)
receipt = json.loads((here / "final-check.json").read_bytes())
for name, expected in receipt["proposed_file_sha256"].items():
    require(sha((repo / name).read_bytes()) == expected, "changed since clean run: " + name)
require(receipt["status_before"] == receipt["status_after"] == "", "unclean evidence")
require(all(r["exit"] == 2 and r["stderr_empty"] and r["exact_reference_match"]
            for r in receipt["runs"]), "final acceptance failed")
actual = (here / "final.stdout.txt").read_bytes()
require(actual == (here.parent / "acct-reference-88/expected.stdout.txt").read_bytes(),
        "final output/reference drift")
require(actual == (here / "final-optimized.stdout.txt").read_bytes(), "optimization differs")
require(not (here / "final.stderr.txt").read_bytes(), "final stderr")
for path in here.glob("*.py"):
    ast.parse(path.read_text(), filename=str(path))

patch = git("diff", "--binary", base)
with (here / "changed-lines.patch").open("xb") as stream:
    stream.write(patch)
numstat = {}
for line in git("diff", "--numstat", base).decode().splitlines():
    added, removed, path = line.split("\t")
    numstat[path] = (int(added), int(removed))
require(len(numstat) == 8 and sum(a for a, _ in numstat.values()) == 58
        and sum(r for _, r in numstat.values()) == 35, "RETURN diff counts differ")
paths = set(git("diff", "--name-only", "-z", base).decode().split("\0")[:-1])
paths.update(git("ls-files", "--others", "--exclude-standard", "-z").decode().split("\0")[:-1])
require(paths and all(p.startswith(prefix) for p in paths), "out-of-scope changes")
files = []
for name in sorted(paths):
    if name == str((here / "scope.json").relative_to(repo)):
        continue
    raw = (repo / name).read_bytes()
    lines = None if name.endswith(".gz") else len(raw.splitlines())
    added, removed = numstat.get(name, (lines, 0))
    files.append(dict(path=name, kind="modified" if name in numstat else "added",
                      bytes=len(raw), text_lines=lines, sha256=sha(raw),
                      added_lines=added, removed_lines=removed))
record = dict(source_commit=base, clean_snapshot_commit=receipt["snapshot_commit"],
    classification="routine internal evidence maintenance",
    all_changes_within_scope=True, diff_check_exit=0,
    clean_run_inputs_unchanged=True,
    post_run_reporting="RETURN.md, final streams/receipt, control bookkeeping, this finalizer, patch and scope ledger are reporting artifacts; no verifier/reference/evidence input changed after final run.",
    self_entry_excluded=True, files=files, modified_files=len(numstat),
    modified_added_lines=58, modified_removed_lines=35,
    added_files=sum(r["kind"] == "added" for r in files),
    added_text_lines=sum(r["text_lines"] or 0 for r in files if r["kind"] == "added"))
with (here / "scope.json").open("x") as stream:
    json.dump(record, stream, indent=2)
    stream.write("\n")
print(json.dumps({k: v for k, v in record.items() if k != "files"}, indent=2))

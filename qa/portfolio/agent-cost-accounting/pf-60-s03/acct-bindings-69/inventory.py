"""Inventory final allocated changes; exclude only this generated inventory."""
import hashlib
import json
from pathlib import Path
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = str(here.parent.relative_to(repo)) + "/"


def git(*args):
    return subprocess.check_output(["git", *args], cwd=repo, text=True).splitlines()


tracked = []
for line in git("diff", "--numstat"):
    added, removed, path = line.split("\t")
    assert path.startswith(scope), path
    tracked.append(dict(path=path, added=int(added), removed=int(removed)))
untracked = []
for name in git("ls-files", "--others", "--exclude-standard"):
    assert name.startswith(scope), name
    if name == str((here / "scope.json").relative_to(repo)):
        continue
    path = repo / name
    data = path.read_bytes()
    category = ("frozen-earlier-auditor" if path.suffix == ".py" and path.parent != here
                else "authored-python" if path.suffix == ".py"
                else "prose" if path.suffix == ".md"
                else "digest-contract" if path.name == "page-content-digests.json"
                else "generated-evidence")
    untracked.append(dict(path=name, category=category, bytes=len(data),
                          lines=None if path.suffix == ".gz" else len(data.splitlines()),
                          sha256=hashlib.sha256(data).hexdigest()))
report = dict(
    base=git("rev-parse", "HEAD")[0], scope=scope,
    existing_added=sum(r["added"] for r in tracked),
    existing_removed=sum(r["removed"] for r in tracked),
    new_lines_by_category={
        category: sum(r["lines"] or 0 for r in untracked if r["category"] == category)
        for category in sorted({r["category"] for r in untracked})
    },
    tracked=tracked, untracked=untracked,
    exclusion="scope.json itself and ignored regenerable fixture copies/uncompressed logs/build outputs are excluded.",
)
with (here / "scope.json").open("x") as stream:
    json.dump(report, stream, indent=2)
    stream.write("\n")
print(json.dumps({key: report[key] for key in ("existing_added", "existing_removed", "new_lines_by_category")}))

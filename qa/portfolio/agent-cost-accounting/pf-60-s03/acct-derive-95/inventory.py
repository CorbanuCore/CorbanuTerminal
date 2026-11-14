"""Record changed files against the allocation base, excluding ignored scratch/self."""
import hashlib
import json
from pathlib import Path
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = str(here.parent.relative_to(repo)) + "/"
base = "b2fa4f7276baa19b33d84732dacac9e494d9943d"
output = here / "scope.json"

def git(*args):
    return subprocess.check_output(["git", *args], cwd=repo, text=True)

changes = {}
for line in git("diff", "--numstat", base).splitlines():
    added, removed, name = line.split("\t", 2)
    changes[name] = dict(kind="modified", added_lines=int(added), removed_lines=int(removed))
for name in git("ls-files", "--others", "--exclude-standard").splitlines():
    if repo / name != output:
        changes[name] = dict(kind="new")
rows = []
for name, change in sorted(changes.items()):
    if not name.startswith(scope):
        raise RuntimeError("out-of-scope change: " + name)
    path = repo / name
    raw = path.read_bytes()
    rows.append(dict(path=name, **change, bytes=len(raw),
                     lines=None if path.suffix == ".gz" else len(raw.splitlines()),
                     sha256=hashlib.sha256(raw).hexdigest()))
result = dict(base=base, self_excluded=True,
              modified_files=sum(row["kind"] == "modified" for row in rows),
              added_lines=sum(row.get("added_lines", 0) for row in rows),
              removed_lines=sum(row.get("removed_lines", 0) for row in rows),
              new_files=sum(row["kind"] == "new" for row in rows),
              new_text_lines=sum(row["lines"] or 0 for row in rows if row["kind"] == "new"),
              files=rows)
output.write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps({key: value for key, value in result.items() if key != "files"}, indent=2))

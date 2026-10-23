"""Freeze this allocation scope and check every current inventory entry."""
import hashlib
import json
from pathlib import Path
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
prefix = str(here.parent.relative_to(repo)) + "/"
base = "1aadfc123592530f657794489249e954f5e3885a"
tracked = subprocess.check_output(["git", "diff", "--name-only", "-z", base], cwd=repo).decode().split("\0")
untracked = subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard", "-z"], cwd=repo).decode().split("\0")
paths = sorted(set(filter(None, tracked + untracked)))
assert all(path.startswith(prefix) for path in paths), paths
rows = []
for name in paths:
    path = repo / name
    if path == here / "scope.json":
        continue
    data = path.read_bytes()
    rows.append(dict(path=name, bytes=len(data), lines=None if path.suffix == ".gz" else len(data.splitlines()),
                     sha256=hashlib.sha256(data).hexdigest(), kind="modified" if name in tracked else "new"))
record = dict(base=base, scope=prefix, files=rows, self_excluded=True, outside_scope_changes=[],
              tracked_numstat=subprocess.check_output(["git", "diff", "--numstat", base], cwd=repo, text=True),
              new_text_lines=sum(row["lines"] or 0 for row in rows if row["kind"] == "new"),
              new_files=sum(row["kind"] == "new" for row in rows),
              note="Generated receipts and original-inventory copies count as evidence lines; ignored local build/package bytes are bound separately by package-manifest.json.")
(here / "scope.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ("tracked_numstat", "new_text_lines", "new_files", "outside_scope_changes")}))

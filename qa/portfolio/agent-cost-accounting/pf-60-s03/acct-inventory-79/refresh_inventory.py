"""Refresh round-75 membership without confusing additions and modifications."""
import hashlib
import json
from pathlib import Path
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
path = here.parent / "acct-acceptance-75/scope.json"
before = subprocess.check_output(
    ["git", "show", "8317d81a49aac1246120e806b7e380c13c738862:" + str(path.relative_to(repo))],
    cwd=repo,
)
document = json.loads(before)
baseline_paths = set(subprocess.check_output(
    ["git", "ls-tree", "-r", "--name-only", document["base"]], cwd=repo, text=True,
).splitlines())
for row in document["files"]:
    target = repo / row["path"]
    data = target.read_bytes()
    row.update(bytes=len(data), lines=None if target.suffix == ".gz" else len(data.splitlines()),
               sha256=hashlib.sha256(data).hexdigest(),
               kind="modified" if row["path"] in baseline_paths else "new")
for kind in ("new", "modified"):
    rows = [row for row in document["files"] if row["kind"] == kind]
    document["total_" + kind + "_files"] = len(rows)
    document["total_" + kind + "_bytes"] = sum(row["bytes"] for row in rows)
    document["total_" + kind + "_text_lines"] = sum(row["lines"] or 0 for row in rows)
document["total_current_bytes"] = sum(row["bytes"] for row in document["files"])
document["total_current_text_lines"] = sum(row["lines"] or 0 for row in document["files"])
document["classification_basis"] = (
    "kind means presence at the original round-75 base: absent=new, present=modified. "
    "Sizes and lines are current full-file sizes, not diff additions/deletions. "
    "Membership remains historical and excludes this inventory."
)
document["refresh"]["superseded_by"] = str((here / "inventory-correction.json").relative_to(repo))
record = dict(
    allocation="acct-inventory-79",
    before=json.loads(before),
    after_totals={k: v for k, v in document.items() if k.startswith("total_")},
    note="Preserves the prior inventory; round-76 scope/delta remain historical snapshots. "
         "This record supersedes their round-75 inventory digest, not their test evidence.",
)
path.write_text(json.dumps(document, indent=2) + "\n")
record["after_sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
(here / "inventory-correction.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record["after_totals"], sort_keys=True))

"""Refresh existing inventories, preserving original bytes and explicit deltas."""
import hashlib
import json
from pathlib import Path
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
base = "1aadfc123592530f657794489249e954f5e3885a"
reasons = {
    "acct-controls-72/RETURN.md": "Round 75 corrected qualifying-mode independence and precise subtotal-page coverage.",
    "acct-controls-72/acceptance-gap.md": "Round 75 added a link to its owner acceptance work order.",
    "acct-controls-72/summarize.py": "Round 75 replaced literals with hash-checked derived lane/mutation/viewport counts.",
    "acct-acceptance-75/acceptance.md": "Round 76 names scope-zero beside fitness, preserves timeout status, and separates engineering from owner authorizations.",
}
report = {"base": base, "inventories": []}
for name in ("acct-controls-72", "acct-acceptance-75"):
    path = here.parent / name / "scope.json"
    original = subprocess.check_output(["git", "show", base + ":" + str(path.relative_to(repo))], cwd=repo)
    saved = here / (name + "-scope-before.json")
    if saved.exists():
        assert saved.read_bytes() == original
    else:
        saved.write_bytes(original)
    document = json.loads(original)
    key = "new_files_excluding_this_inventory" if "new_files_excluding_this_inventory" in document else "files"
    changes = []
    for row in document[key]:
        target = repo / row["path"]
        data = target.read_bytes()
        current = dict(row, bytes=len(data), lines=None if target.suffix == ".gz" else len(data.splitlines()),
                       sha256=hashlib.sha256(data).hexdigest())
        if current != row:
            relative = str(target.relative_to(here.parent))
            changes.append({"path": row["path"], "before": dict(row), "after": current,
                            "reason": reasons[relative]})
            row.update(current)
    document["total_new_bytes"] = sum(row["bytes"] for row in document[key])
    document["total_new_text_lines"] = sum(row["lines"] or 0 for row in document[key])
    document["refresh"] = {
        "allocation": "acct-fitness-76", "checked_at_base": base,
        "original_inventory": str(saved.relative_to(repo)),
        "original_inventory_sha256": hashlib.sha256(original).hexdigest(),
        "meaning": "Original path membership/base/numstat are historical; listed byte sizes, lines and hashes describe the current files. No new test execution is inferred.",
        "change_record": str((here / "inventory-refresh.json").relative_to(repo)),
    }
    path.write_text(json.dumps(document, indent=2) + "\n")
    report["inventories"].append({"path": str(path.relative_to(repo)), "entries_checked": len(document[key]),
                                  "changes": changes, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
(here / "inventory-refresh.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({row["path"]: {"entries_checked": row["entries_checked"], "changed": len(row["changes"])}
                  for row in report["inventories"]}))

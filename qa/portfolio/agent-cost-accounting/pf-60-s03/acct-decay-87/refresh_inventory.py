"""Refresh the historical membership's current hashes, preserving prior corrections."""
import hashlib
import json
from pathlib import Path

here = Path(__file__).resolve().parent
repo = here.parents[4]
inventory = here.parent / "acct-acceptance-75/scope.json"
correction = here.parent / "acct-inventory-79/inventory-correction.json"
document = json.loads(inventory.read_text())
record = json.loads(correction.read_text())
record.setdefault("decay_87_refresh", dict(
    previous_after_sha256=record["after_sha256"],
    previous_after_totals=record["after_totals"],
    reason="Refuse undocumented local-package baseline agreement; membership unchanged.",
))
for row in document["files"]:
    path = repo / row["path"]
    raw = path.read_bytes()
    row.update(bytes=len(raw), lines=None if path.suffix == ".gz" else len(raw.splitlines()),
               sha256=hashlib.sha256(raw).hexdigest())
for kind in ("new", "modified"):
    rows = [row for row in document["files"] if row["kind"] == kind]
    document["total_" + kind + "_files"] = len(rows)
    document["total_" + kind + "_bytes"] = sum(row["bytes"] for row in rows)
    document["total_" + kind + "_text_lines"] = sum(row["lines"] or 0 for row in rows)
document["total_current_bytes"] = sum(row["bytes"] for row in document["files"])
document["total_current_text_lines"] = sum(row["lines"] or 0 for row in document["files"])
inventory.write_text(json.dumps(document, indent=2) + "\n")
record["after_totals"] = {k: v for k, v in document.items() if k.startswith("total_")}
record["after_sha256"] = hashlib.sha256(inventory.read_bytes()).hexdigest()
correction.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record["after_totals"], sort_keys=True))

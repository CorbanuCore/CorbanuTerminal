"""Refresh affected retained inventories; preserve historical membership and totals."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

here = Path(__file__).resolve().parent
scope = here.parent
repo = here.parents[4]

def load(path):
    return json.loads(path.read_bytes())

def save(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

correction = scope / "acct-inventory-79/inventory-correction.json"
record = load(correction)
record.setdefault("derive_95_refresh", dict(
    previous_after_sha256=record["after_sha256"],
    previous_after_totals=record["after_totals"],
    reason="Move exit table after replay introduction; same bytes/lines totals and membership. No changed numeric claim or expected stdout.",
))
save(correction, record)
subprocess.run([sys.executable, "-B", str(scope / "acct-guard-91/refresh_inventory.py")],
               cwd=repo, check=True)

for name in ("acct-guard-91", "acct-inventory-92"):
    path = scope / name / "scope.json"
    record = load(path)
    record.setdefault("derive_95_refresh", dict(
        previous_inventory_sha256=sha(path),
        meaning="Original base, membership and change counts remain historical. Current bytes, lines and hashes refreshed for the round-95 correction; no new execution inferred.",
    ))
    for row in record["files"]:
        member = repo / row["path"]
        raw = member.read_bytes()
        row.update(bytes=len(raw), lines=None if member.suffix == ".gz" else len(raw.splitlines()),
                   sha256=hashlib.sha256(raw).hexdigest())
    save(path, record)

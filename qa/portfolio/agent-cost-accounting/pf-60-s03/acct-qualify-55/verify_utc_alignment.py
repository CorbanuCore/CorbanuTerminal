"""Regenerate the historical acct-qualify-50/inspect-01 UTC alignment receipt."""
import json
from pathlib import Path

root = Path(__file__).resolve().parent.parent / "acct-qualify-50/inspect-01"
cases = [
    ("hour", "Hour [2026-08-10T12:00:00.000Z, 2026-08-10T13:00:00.000Z)"),
    ("week", "Week [2026-08-10T00:00:00.000Z, 2026-08-17T00:00:00.000Z)"),
    ("month", "Month [2026-08-01T00:00:00.000Z, 2026-09-01T00:00:00.000Z)"),
]
results = []
for group, expected in cases:
    selected = json.loads((root / (group+"-overview-selected.json")).read_text())
    results.append(dict(group=group, expected_bucket=expected, matched=expected in selected))
assert all(row["matched"] for row in results)
regenerated = json.dumps(results, indent=2)+"\n"
output = Path(__file__).resolve().parent / "utc-alignment-regenerated.json"
if output.exists():
    assert output.read_text() == regenerated, "Existing regenerated receipt differs"
else:
    output.write_text(regenerated)
assert regenerated.encode() == (root / "utc-alignment.json").read_bytes(), "Historical receipt mismatch"
print("3/3 historical UTC bucket labels reproduced from preserved PTY selections")

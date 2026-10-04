"""Verify identical page artifacts without inferring session populations."""
import gzip
import hashlib
import json
from pathlib import Path

here = Path(__file__).resolve().parent
repo = here.parents[4]
binding = json.loads((here.parent / "acct-receipt-66/capture-bindings.json").read_text())
captures = binding["captures"]
for row in captures:
    for kind in ("selected", "viewport"):
        data = (repo / row[kind]).read_bytes()
        assert hashlib.sha256(data).hexdigest() == row[kind + "_sha256"]
    assert json.loads((repo / row["selected"]).read_text()) == row["rows"]
    viewport = gzip.decompress((repo / row["viewport"]).read_bytes()).decode()
    assert "Requested UTC day: 2026-08-10" in viewport
    assert all(line in viewport for line in row["rows"])
equal = {}
for kind in ("selected", "viewport"):
    left, right = [(repo / row[kind]).read_bytes() for row in captures]
    assert left == right
    equal[kind] = dict(byte_identical=True, bytes=len(left),
                       sha256=hashlib.sha256(left).hexdigest())
assert equal["viewport"]["bytes"] == 1249
store_paths = [
    "acct-readers-61/reader-run-05/native-bindings.json",
    "acct-readers-61/reader-run-05/identities.json",
    "acct-scope-62/scope-run-01/store-before-fresh-reader.json",
    "acct-scope-62/scope-run-01/store-during-fresh-reader.json",
]
report = dict(
    equality=equal, note=binding["note"],
    retracted_claim="They remain distinct captures/attempt populations",
    store_readback_sources=[
        dict(path=path, sha256=hashlib.sha256((here.parent / path).read_bytes()).hexdigest())
        for path in store_paths
    ],
    limitation="Store read-backs separate the datasets; the identical pages do not bind which root or attempts either session resolved at capture time.",
)
with (here / "capture-recheck.json").open("x") as stream:
    json.dump(report, stream, indent=2)
    stream.write("\n")
print(json.dumps(equal))

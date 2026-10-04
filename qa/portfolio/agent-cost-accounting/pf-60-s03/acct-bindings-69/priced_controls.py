"""Reject appended amounts on every subtotal page; preserve old/new results."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

here = Path(__file__).resolve().parent
repo = here.parents[4]
prior = here.parent / "acct-readers-61"
source = prior / "reader-run-05"
base = "a06048a8a4cd1422c9ef80dcef5503ee297d8d4c"
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True, exist_ok=False)
old = out / "round-66-audit.py"
for name, target in (
    ("audit_readers.py", old),
    ("nonmonetary-page-digests.json", out / "nonmonetary-page-digests.json"),
):
    target.write_bytes(subprocess.check_output(
        ["git", "show", base + ":" + str((prior / name).relative_to(repo))],
        cwd=repo,
    ))
env = dict(os.environ, PYTHONPATH=str(prior), PYTHONDONTWRITEBYTECODE="1")


def audit(script, directory, target):
    process = subprocess.run(
        [sys.executable, "-B", str(script), str(directory), str(target)],
        env=env, capture_output=True, text=True,
    )
    return dict(exit=process.returncode, receipt_written=target.exists(),
                stdout=process.stdout, stderr=process.stderr)


baseline = audit(prior / "audit_readers.py", source, out / "baseline.json")
previous_baseline = audit(old, source, out / "old-baseline.json")
assert all(r["exit"] == 0 and r["receipt_written"]
           for r in (baseline, previous_baseline))
receipt = json.loads((out / "baseline.json").read_text())
assert receipt["content_bound_page_count"] == 24
names = sorted(row["page"] for row in receipt["numeric_pages"])
assert len(names) == 15
# Each expected hash must bind the unchanged base capture, not a mutated page.
digests = json.loads((prior / "page-content-digests.json").read_text())
source_bindings = []
for name in sorted(digests):
    path = source / (name + "-selected.json")
    original = subprocess.check_output(
        ["git", "show", base + ":" + str(path.relative_to(repo))], cwd=repo,
    )
    assert path.read_bytes() == original
    assert hashlib.sha256(json.dumps(json.loads(original), ensure_ascii=True).encode()).hexdigest() == digests[name]
    source_bindings.append(dict(page=name, source_sha256=hashlib.sha256(original).hexdigest(),
                                content_sha256=digests[name]))

results = []
for name in names:
    for kind, injected in (
        ("extra-total", "Total: $9.99"),
        ("euro", "Total: €9.99"),
        ("cents", "Total: 999 cents"),
        ("symbol-free", "Estimated token cost: 9.99"),
        ("leading-decimal", "Total: $.99"),
    ):
        directory = out / (kind + "-" + name)
        directory.mkdir()
        filename = name + "-selected.json"
        for path in source.iterdir():
            if path.is_file() and path.name not in (
                filename, "reader-cases.json", name + ".txt.gz",
            ):
                (directory / path.name).symlink_to(path)
        lines = json.loads((source / filename).read_text()) + [injected]
        (directory / filename).write_text(json.dumps(lines, indent=2) + "\n")
        cases = json.loads((source / "reader-cases.json").read_text())
        for case in cases:
            if case["case"] == name:
                case["rendered"] = lines
        (directory / "reader-cases.json").write_text(json.dumps(cases, indent=2) + "\n")
        viewport = gzip.decompress((source / (name + ".txt.gz")).read_bytes())
        (directory / (name + ".txt.gz")).write_bytes(
            gzip.compress(viewport + b"\n" + injected.encode(), mtime=0)
        )
        previous = audit(old, directory, directory / "old-receipt.json")
        revised = audit(prior / "audit_readers.py", directory, directory / "receipt.json")
        assert previous["exit"] == 0 and previous["receipt_written"], previous
        assert revised["exit"] != 0 and not revised["receipt_written"], revised
        assert "unexpected content on subtotal page" in revised["stderr"], revised
        results.append(dict(page=name, kind=kind, injected=injected,
                            previous=previous, revised=revised))
assert len(results) == 75
report = dict(base=base, baseline=baseline, previous_baseline=previous_baseline,
              source_bindings=source_bindings, subtotal_pages=len(names),
              priced_pages=receipt["priced_reconciliation_count"],
              unknown_pages=receipt["unknown_cost_page_count"],
              zero_pages=receipt["zero_recorded_page_count"],
              previously_accepted=len(results), now_rejected=len(results),
              controls=results,
              limitation="Mutants validate the auditor, not new product execution.")
(out / "results.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({k: report[k] for k in ("subtotal_pages", "previously_accepted", "now_rejected")}))

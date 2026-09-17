"""Remove exact and displayed amounts from every expected monetary page."""
import gzip
import json
import os
from pathlib import Path
import re
import subprocess
import sys

here = Path(__file__).resolve().parent
prior = here.parent / "acct-readers-61"
source = prior / "reader-run-05"
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True, exist_ok=False)
baseline = subprocess.run(
    [sys.executable, str(prior / "audit_readers.py"), str(source), str(out / "baseline.json")],
    capture_output=True, text=True,
)
assert baseline.returncode == 0, baseline.stderr
receipt = json.loads((out / "baseline.json").read_text())
assert (receipt["priced_reconciliation_count"], receipt["unknown_cost_page_count"],
        receipt["zero_recorded_page_count"]) == (9, 4, 2)
old_audit = out / "pre-revision-audit.py"
old_audit.write_bytes(subprocess.check_output([
    "git", "show",
    "defa10a2dd00b6d4f8b91d54978b480c75a2b0e4:"
    "qa/portfolio/agent-cost-accounting/pf-60-s03/acct-readers-61/audit_readers.py",
], cwd=here))
old_env = dict(os.environ, PYTHONPATH=str(prior))
results = []
for kind in ("exact", "displayed"):
    pages = (receipt["priced_reconciliation_pages"] + receipt["unknown_cost_pages"]
             + (receipt["zero_recorded_pages"] if kind == "exact" else []))
    for name in pages:
        directory = out / (kind + "-" + name)
        directory.mkdir()
        filename = name + "-selected.json"
        for path in source.iterdir():
            if path.is_file() and path.name not in (filename, "reader-cases.json", name + ".txt.gz"):
                (directory / path.name).symlink_to(path)
        original = json.loads((source / filename).read_text())
        pattern = (r"Known (?:subtotal|estimate) exact USD:" if kind == "exact" else
                   r"(?:Estimated token cost for recorded attempts|Known estimated token cost): \$")
        mutant = [line for line in original if not re.search(pattern, line)]
        assert len(mutant) == len(original) - 1
        (directory / filename).write_text(json.dumps(mutant, indent=2) + "\n")
        cases = json.loads((source / "reader-cases.json").read_text())
        for case in cases:
            if case["case"] == name:
                case["rendered"] = mutant
        (directory / "reader-cases.json").write_text(json.dumps(cases, indent=2) + "\n")
        viewport = gzip.decompress((source / (name + ".txt.gz")).read_bytes()).decode()
        # Mirror the omitted row in viewports too; keep the requested-day header.
        removed = next(line for line in original if line not in mutant)
        viewport = viewport.replace(removed, " " * len(removed))
        (directory / (name + ".txt.gz")).write_bytes(gzip.compress(viewport.encode(), mtime=0))
        old_target = directory / "old-receipt.json"
        old = subprocess.run(
            [sys.executable, str(old_audit), str(directory), str(old_target)],
            env=old_env, capture_output=True, text=True,
        )
        # The old audit already bound the narrow overview in a special-case check.
        assert (old.returncode != 0) == (name == "narrow"), (kind, name, old.stderr)
        assert old_target.exists() == (old.returncode == 0)
        target = directory / "receipt.json"
        process = subprocess.run(
            [sys.executable, str(prior / "audit_readers.py"), str(directory), str(target)],
            capture_output=True, text=True,
        )
        assert process.returncode != 0 and not target.exists(), (kind, name)
        if kind == "exact":
            assert "numeric page coverage mismatch" in process.stderr, process.stderr
        else:
            assert "AssertionError" in process.stderr and name in process.stderr
        results.append(dict(kind=kind, page=name, exit=process.returncode,
                            old_check_exit=old.returncode,old_receipt_written=old_target.exists(),
                            old_error=old.stderr,
                            receipt_written=target.exists(), error=process.stderr))
assert len(results) == 28
(out / "results.json").write_text(json.dumps(
    dict(baseline_stdout=baseline.stdout, rejected_count=len(results), controls=results),
    indent=2) + "\n")
print("Baseline: 9 priced, 4 unknown, 2 zero-recorded; 28 amount-removal controls rejected before receipt.")

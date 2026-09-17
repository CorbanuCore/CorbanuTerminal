"""Compare the frozen round-62 audit with identity-bound page validation."""
import gzip
import json
import os
from pathlib import Path
import subprocess
import sys

here = Path(__file__).resolve().parent
prior = here.parent / "acct-readers-61"
source = prior / "reader-run-05"
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True, exist_ok=False)
old = out / "pre-revision-audit.py"
old.write_bytes(subprocess.check_output([
    "git", "show", "49fac26d01fdc172d6300fd72cad8037d0c6e351:"
    "qa/portfolio/agent-cost-accounting/pf-60-s03/acct-readers-61/audit_readers.py",
], cwd=here))
env = dict(os.environ, PYTHONPATH=str(prior), PYTHONDONTWRITEBYTECODE="1")


def audit(script, directory, target):
    process = subprocess.run(
        [sys.executable, "-B", str(script), str(directory), str(target)],
        env=env, capture_output=True, text=True,
    )
    return dict(exit=process.returncode, receipt_written=target.exists(),
                stdout=process.stdout, stderr=process.stderr)


baseline = audit(prior / "audit_readers.py", source, out / "baseline.json")
assert baseline["exit"] == 0, baseline
receipt = json.loads((out / "baseline.json").read_text())
assert receipt["page_identity_count"] == 24
controls = [(kind, name, line) for name in receipt["nonmonetary_pages"]
            for kind, line in (
                ("dollar", "Known estimated token cost: $9.990000"),
                ("usd-prefix", "Output cost: USD 9.99"),
                ("usd-suffix", "Total: 9.99 USD"),
            )]
controls += [("missing-page", name, None) for name in receipt["nonmonetary_pages"]]
controls += [("extra-page", "unexpected-navigation", None),
             ("extra-money-page", "unexpected-amount", "Total: $9.99")]
results = []
for kind, name, injected in controls:
    directory = out / (kind + "-" + name)
    directory.mkdir()
    filename = name + "-selected.json"
    for path in source.iterdir():
        if path.is_file() and path.name not in (
            filename, "reader-cases.json", name + ".txt.gz",
        ):
            (directory / path.name).symlink_to(path)
    cases = json.loads((source / "reader-cases.json").read_text())
    if kind == "missing-page":
        cases = [case for case in cases if case["case"] != name]
    else:
        original = source / filename
        lines = json.loads(original.read_text()) if original.exists() else ["Refresh"]
        if injected:
            lines.append(injected)
        (directory / filename).write_text(json.dumps(lines, indent=2) + "\n")
        for case in cases:
            if case["case"] == name:
                case["rendered"] = lines
        viewport_path = source / (name + ".txt.gz")
        viewport = gzip.decompress(viewport_path.read_bytes()).decode() if viewport_path.exists() else ""
        (directory / (name + ".txt.gz")).write_bytes(
            gzip.compress((viewport + "\n" + (injected or "")).encode(), mtime=0)
        )
    (directory / "reader-cases.json").write_text(json.dumps(cases, indent=2) + "\n")
    previous = audit(old, directory, directory / "old-receipt.json")
    revised = audit(prior / "audit_readers.py", directory, directory / "receipt.json")
    assert revised["exit"] != 0 and not revised["receipt_written"], (kind, name, revised)
    reason = ("page identity coverage mismatch" if kind in (
        "missing-page", "extra-page", "extra-money-page",
    ) else "unexpected monetary amount on nonmonetary page")
    assert reason in revised["stderr"], revised
    results.append(dict(kind=kind, page=name, injected=injected, previous=previous, revised=revised))
assert len(results) == 38
report = dict(
    baseline=baseline, rejected_count=len(results),
    previously_accepted=sum(row["previous"]["exit"] == 0 for row in results),
    controls=results,
    limitation="Mutated QA copies test the auditor; they are not new product observations.",
)
(out / "results.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({key: report[key] for key in ("rejected_count", "previously_accepted")}))

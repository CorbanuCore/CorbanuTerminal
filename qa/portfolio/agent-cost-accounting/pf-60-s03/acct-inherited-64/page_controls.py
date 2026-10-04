"""Compare the frozen round-62 audit with identity-bound page validation."""
import gzip
import json
import os
import re
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
round64 = out / "round-64-audit.py"
round64.write_bytes(subprocess.check_output([
    "git", "show", "270a6644e01d780ef93f8715dd0051481643dc7d:"
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
for script, name in ((old, "old-baseline"), (round64, "round64-baseline")):
    positive = audit(script, source, out / (name + ".json"))
    assert positive["exit"] == 0 and positive["receipt_written"], positive
controls = [(kind, name, line) for name in receipt["nonmonetary_pages"]
            for kind, line in (
                ("dollar", "Known estimated token cost: $9.990000"),
                ("usd-prefix", "Output cost: USD 9.99"),
                ("usd-suffix", "Total: 9.99 USD"),
            )]
controls += [("missing-page", name, None) for name in receipt["nonmonetary_pages"]]
controls += [("extra-page", "unexpected-navigation", None),
             ("extra-money-page", "unexpected-amount", "Total: $9.99")]
original_count = len(controls)
assert original_count == 38
# Independent counterexamples outside all three round-64 currency patterns.
# Cover alternate currency, denomination, symbol-free cost and leading decimal.
outside_lines = (
    "Estimated token cost: €9.99",
    "Estimated token cost: 999 cents",
    "Estimated token cost: 9.99",
    "Known estimated token cost: $.99",
)
round64_pattern = r"\$\s*[+-]?[0-9]|\bUSD\s*:?\s*[+-]?[0-9]|[0-9]\s*USD\b"
assert all(not re.search(round64_pattern, line) for line in outside_lines)
controls += [(f"outside-{index}", name, line)
             for name in receipt["nonmonetary_pages"]
             for index, line in enumerate(outside_lines)]
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
    previous64 = None
    if kind.startswith("outside-"):
        previous64 = audit(round64, directory, directory / "round64-receipt.json")
        assert previous64["exit"] == 0 and previous64["receipt_written"], previous64
    revised = audit(prior / "audit_readers.py", directory, directory / "receipt.json")
    assert revised["exit"] != 0 and not revised["receipt_written"], (kind, name, revised)
    reason = ("page identity coverage mismatch" if kind in (
        "missing-page", "extra-page", "extra-money-page",
    ) else "unexpected content on nonmonetary page")
    assert reason in revised["stderr"], revised
    results.append(dict(kind=kind, page=name, injected=injected, previous=previous,
                        round64=previous64, revised=revised))
assert len(results) == 74

def accepted(rows):
    return sum(row["previous"]["exit"] == 0 and row["previous"]["receipt_written"]
               for row in rows)

# Freeze the original asymmetry separately so new cases cannot mask a regression.
assert accepted(results[:original_count]) == 34
assert accepted(results[original_count:]) == 36
assert accepted(results) == 70
report = dict(
    baseline=baseline, rejected_count=len(results),
    original_controls=original_count, original_previously_accepted=34,
    outside_controls=36, outside_round64_accepted=36,
    previously_accepted=accepted(results),
    controls=results,
    limitation="Mutated QA copies test the auditor; they are not new product observations.",
)
(out / "results.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({key: report[key] for key in ("rejected_count", "previously_accepted")}))

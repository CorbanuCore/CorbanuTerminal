"""Exercise monetary semantics independently of frozen capture integrity."""
import gzip
import json
from pathlib import Path
import re
import subprocess
import sys

here = Path(__file__).resolve().parent
prior = here.parent / "acct-readers-61"
source = prior / "reader-run-05"
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True, exist_ok=False)
audit = prior / "audit_readers.py"


def invoke(directory, name, semantics=False):
    target = out / (name + ".json")
    command = [sys.executable, "-B", str(audit), str(directory), str(target)]
    if semantics:
        command.append("--semantics-only")
    process = subprocess.run(command, capture_output=True, text=True)
    result = dict(exit=process.returncode, receipt_written=target.exists(),
                  stdout=process.stdout, stderr=process.stderr)
    with (out / "processes.jsonl").open("a") as log:
        log.write(json.dumps(dict(name=name, **result)) + "\n")
    return result


baselines = {mode: invoke(source, "baseline-" + mode, mode == "semantics-only")
             for mode in ("digest-bound", "semantics-only")}
assert all(r["exit"] == 0 and r["receipt_written"] for r in baselines.values()), baselines
receipt = json.loads((out / "baseline-semantics-only.json").read_text())
assert not receipt["capture_qualified"]
assert receipt["content_bound_page_count"] == receipt["viewport_bound_count"] == 0
bound = json.loads((out / "baseline-digest-bound.json").read_text())
assert bound["content_bound_page_count"] == 24 and bound["viewport_bound_count"] == 45

# Deliberately enumerate the expected contract independently of the auditor's
# check inventory. Deleting an auditor check must not shrink this test plan.
priced = ["backend-reopened", "historical", "historical-Request-1-Attempt-1",
          "mixed-provider", "mixed-provider-Request-1-Attempt-1",
          "mixed-provider-Provider-openai", "narrow", "narrow-Request-1-Attempt-1",
          "stale-restored-control"]
unknown = ["missing-price", "missing-price-Request-1-Attempt-1",
           "no-usage", "no-usage-Request-1-Attempt-1"]
zero = ["zero-attempt-day", "never-prompted"]
plan = []


def page_case(key, page, pattern, replacement, append=False):
    plan.append(dict(check=key, page=page, pattern=pattern,
                     replacement=replacement, append=append))


exact = r"Known (?:subtotal|estimate) exact USD: [0-9.]+"
display = r"(?:Estimated token cost for recorded attempts|Known estimated token cost): \$[0-9.]+"
for page in priced + unknown + zero:
    page_case("exact-count:" + page, page, exact, "")
    page_case("exact-value:" + page, page, r"exact USD: [0-9.]+", "exact USD: 9.99")
    for kind in ("count", "value"):
        if page in zero:
            page_case("display-" + kind + ":" + page, page, "",
                      "Known estimated token cost: $9.99", append=True)
        else:
            page_case("display-" + kind + ":" + page, page, display,
                      "" if kind == "count" else "Known estimated token cost: $9.99")
for page in unknown:
    for key, phrase in [
        ("unknown-cost", "Estimated token cost: unknown"),
        ("incomplete-cost", "Full recorded estimate: unavailable (1 of 1 attempts incomplete)"),
    ]:
        page_case(key + ":" + page, page, re.escape(phrase), "REMOVED")
for page in [p for p in priced if p.endswith("Attempt-1")]:
    for label in ("Noncached input", "Cache read", "Cache write", "Output"):
        # Mutate displayed and exact sides independently: both must be checked.
        for field, pattern, replacement in [
            ("display", re.escape(label) + r" cost: \$[0-9.]+", label + " cost: $9.99"),
            ("exact", re.escape(label) + r" cost: (\$[0-9.]+); exact USD [0-9.]+",
             label + r" cost: \1; exact USD 9.99"),
        ]:
            page_case("component:" + page + ":" + label, page, pattern, replacement)
            plan[-1]["variant"] = field
    for label in ("Noncached input", "Cache read", "Output"):
        page_case("rate:" + page + ":" + label, page,
                  re.escape(label) + r" rate: [0-9.]+ USD", label + " rate: 9.99 USD")
for key, page, phrases in [
    ("missing-price", "missing-price-Request-1-Attempt-1", [
        ("snapshot", "Price: unavailable — no dispatch-time price snapshot"),
        ("input", "Noncached input cost: unknown — rate unavailable"),
        ("output", "Output cost: unknown — rate unavailable")]),
    ("no-usage", "no-usage-Request-1-Attempt-1", [
        ("input", "Input: unknown — no retained numeric evidence"),
        ("output", "Output cost: unknown — no retained numeric evidence")]),
]:
    for label, phrase in phrases:
        page_case(key + ":" + page + ":" + label, page, re.escape(phrase), "REMOVED")
for page in ("stale-estimate", "stale-refresh", "unavailable-backend"):
    page_case("refused-money:" + page, page, "", "Total: $9.99", append=True)
for phrase in ("Estimated token cost for recorded attempts: $0.000710",
               "Billed cost: unavailable — no settlement evidence",
               "this UTC day is not their complete lifetime."):
    page_case("narrow-qualifier:" + phrase, "narrow", re.escape(phrase), "REMOVED")
page_case("narrow-exact", "narrow", "Known subtotal exact USD: 0.00071",
          "Known subtotal exact USD: 9.99")
metrics = ["Input", "Noncached input (derived for inclusive input)", "Cache read",
           "Cache write", "Output", "Reasoning (subset, not separately billed)",
           "Total (not separately billed)"]
for page in ("historical", "narrow", "mixed-provider", "missing-price", "no-usage", "zero-attempt-day"):
    for label in metrics:
        page_case("usage:" + page + ":" + label, page,
                  re.escape(label) + r": [0-9]+ known", label + ": 999 known")
for key in ("arithmetic:known-attempt", "arithmetic:emitted-usage",
            "native:missing-price", "native:rates"):
    plan.append(dict(check=key))
expected_checks = {c["check"] for c in plan}
assert expected_checks == set(receipt["monetary_checks"]), (
    expected_checks - set(receipt["monetary_checks"]),
    set(receipt["monetary_checks"]) - expected_checks,
)
(out / "plan.json").write_text(json.dumps(plan, indent=2) + "\n")


def fixture(index, replacements):
    directory = out / ("mutant-" + str(index))
    directory.mkdir()
    for path in source.iterdir():
        if path.is_file() and path.name not in replacements:
            (directory / path.name).symlink_to(path)
    for name, data in replacements.items():
        (directory / name).write_bytes(data)
    return directory


def encoded(value):
    return (json.dumps(value, indent=2) + "\n").encode()


results = []
for index, case in enumerate(plan):
    key = case["check"]
    replacements = {}
    if "page" in case:
        page = case["page"]
        lines = json.loads((source / (page + "-selected.json")).read_text())
        # Joining also targets qualifiers wrapped across narrow-screen rows.
        text = " ".join(lines)
        if case["append"]:
            mutated = text + " " + case["replacement"]
        else:
            mutated, count = re.subn(case["pattern"], case["replacement"], text)
            assert count == 1, (case, count)
        mutated_lines = lines + [case["replacement"]] if case["append"] else [mutated]
        replacements[page + "-selected.json"] = encoded(mutated_lines)
        cases = json.loads((source / "reader-cases.json").read_text())
        for row in cases:
            if row["case"] == page:
                row["rendered"] = mutated_lines
        replacements["reader-cases.json"] = encoded(cases)
    elif key.startswith("arithmetic:"):
        rows = json.loads((source / "emissions.json").read_text())
        rows[0]["input"] += 1
        replacements["emissions.json"] = encoded(rows)
    else:
        rows = json.loads((source / "native-bindings.json").read_text())
        if key == "native:missing-price":
            row = next(r for r in rows if json.loads(r[0])["model"] == "gpt-6-astra")
            row[1] = next(r[1] for r in rows if r[1] is not None)
        else:
            quote = json.loads(rows[0][1])
            quote["rates"]["noncached"] = "9.99"
            rows[0][1] = json.dumps(quote)
        replacements["native-bindings.json"] = encoded(rows)
    directory = fixture(index, replacements)
    result = invoke(directory, "semantic-" + str(index), True)
    assert result["exit"] != 0 and not result["receipt_written"], (case, result)
    diagnostic = json.loads(result["stderr"].splitlines()[0])
    assert key in diagnostic["semantic_failures"], (case, result)
    assert expected_checks == set(diagnostic["monetary_checks_evaluated"]), (case, result)
    results.append(dict(mutation=case, **result))
(out / "semantic-results.json").write_text(json.dumps(
    dict(baselines=baselines, checks=len(expected_checks), mutations=len(results),
         controls=results), indent=2) + "\n")

# Integrity controls must fail for their own reason. A viewport-only edit is
# invisible to JSON binding; semantics-only must still pass it.
integrity = []
for kind, paths in (
    ("json", sorted(source.glob("*-selected.json"))),
    ("viewport", sorted(source.glob("*.txt.gz"))),
):
    for path in paths:
        if kind == "json":
            lines = json.loads(path.read_text()) + ["Total: $9.99"]
            changes = {path.name: encoded(lines)}
            cases = json.loads((source / "reader-cases.json").read_text())
            for row in cases:
                if row["case"] + "-selected.json" == path.name:
                    row["rendered"] = lines
            changes["reader-cases.json"] = encoded(cases)
        else:
            changes = {path.name: gzip.compress(
                gzip.decompress(path.read_bytes()) + b"\nTotal: $9.99\n", mtime=0)}
        number = len(plan) + len(integrity)
        directory = fixture(number, changes)
        result = invoke(directory, "integrity-" + str(number))
        assert result["exit"] != 0 and not result["receipt_written"], result
        reason = "unexpected viewport content" if kind == "viewport" else "unexpected content on"
        assert reason in result["stderr"] and path.name.removesuffix("-selected.json") in result["stderr"], result
        semantic = None
        if kind == "viewport":
            semantic = invoke(directory, "viewport-semantic-" + str(number), True)
            assert semantic["exit"] == 0 and semantic["receipt_written"], semantic
        integrity.append(dict(kind=kind, file=path.name, bound=result, semantics=semantic))
(out / "integrity-results.json").write_text(json.dumps(
    dict(json_pages=24, viewports=45, controls=integrity), indent=2) + "\n")
print(json.dumps(dict(monetary_checks=len(expected_checks), semantic_mutations=len(results),
                      json_integrity=24, viewport_integrity=45)))

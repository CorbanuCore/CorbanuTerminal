"""Read-only acceptance reconciliation. Python stdlib + Git; no tests/builds/network.
Exit 0: agreement; 1: disagreement; 2: unavailable evidence without disagreement.
Default excludes ignored/untracked evidence so local packages cannot hide clone gaps.
--local-package additionally verifies any staged package files, without launching them.
"""
import argparse
from decimal import Decimal
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
SCOPE = HERE.parent
TEST = "suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity"
LABELS = (("passed", "passed"), ("failed", "failed"), ("timed_out", "timed out"),
          ("flaky", "flaky"), ("skipped", "skipped"), ("leaky", "leaky"))
RESULTS = []
CLAIMS = []
TRACKED = set()


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def data(path):
    path = Path(path)
    relative = str(path.relative_to(ROOT))
    if relative not in TRACKED:
        raise FileNotFoundError(relative + " is not tracked; unavailable from a clean checkout")
    if not path.is_file():
        raise FileNotFoundError(relative + " is missing")
    return path.read_bytes()


def load(path):
    path = Path(path)
    raw = data(path)
    return json.loads(gzip.decompress(raw) if path.suffix == ".gz" else raw)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def check(name, fn):
    try:
        detail = fn()
        state = "AGREE"
    except FileNotFoundError as exc:
        state, detail = "UNAVAILABLE", str(exc)
    except (ValueError, KeyError, TypeError, AssertionError, OSError, EOFError) as exc:
        state, detail = "DISAGREE", str(exc)
    RESULTS.append(state)
    print(f"{state} {name}: {detail}")


def match(pattern):
    found = list(re.finditer(pattern, DOCUMENT, re.S))
    require(len(found) == 1, "acceptance claim missing or ambiguous: " + pattern)
    CLAIMS.append(found[0].span())
    return found[0]


def lane(directory, name):
    row = load(directory / (name + ".json"))
    if "log_sha256" not in row:
        aggregate = load(directory.parent / "test-results.json")
        enriched = next(r for r in aggregate["lanes"] if r["lane"] == name)
        require(all(enriched[k] == v for k, v in row.items()), "historical receipt differs")
        row = enriched
    raw = gzip.decompress(data(directory / (name + ".log.gz")))
    require(digest(raw) == row["log_sha256"], name + " raw log digest")
    text = raw.decode()
    if name == "prerequisites":
        require(row["exit"] == 0, "prerequisite build exit")
        return row, text
    require("Test isolation: disposable profile; native keyring disabled (debug lane)" in text,
            name + " missing isolation banner")
    summaries = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", text, re.M)
    require(len(summaries) == 1, name + " missing/ambiguous nextest summary")
    count, details = summaries[0]
    counts = dict(run=int(count))
    for key, label in LABELS:
        found = re.search(r"(\d+) " + label, details)
        counts[key] = int(found[1]) if found else 0
    require(counts["run"] > 0, name + " zero executions")
    for key, value in counts.items():
        if key in row:
            require(row[key] == value, name + " receipt mismatch: " + key)
    require(counts["run"] == counts["passed"] + counts["failed"] + counts["timed_out"],
            name + " summary counts do not sum")
    failures = list(dict.fromkeys(line.strip() for line in text.splitlines()
        if re.match(r"\s*(?:TRY\s+\d+\s+)?(?:FAIL|TIMEOUT|TMT|XPASS|LEAK FAIL)\s", line)))
    require(failures == [line.strip() for line in row["failure_lines"]],
            name + " failure names differ")
    return dict(row, **counts), text


def gate_summary(directory, name):
    row, _ = lane(directory, name)
    if name == "prerequisites":
        return "build exit=0; raw log SHA-256 agrees"
    require(row["environment"]["NEXTEST_TEST_THREADS"] == "4", "thread limit")
    require(row["command"][:2] == ["just", "test"], "unguarded command")
    return (f"passed/run={row['passed']}/{row['run']}; skipped={row['skipped']}; "
            f"failed={row['failed']}; timed_out={row['timed_out']}; flaky={row['flaky']}; "
            f"leaky={row['leaky']}; exit={row['exit']}; failure_names={row['failure_lines']}")


def gates():
    path = SCOPE / "acct-fitness-76"
    receipt = load(path / "test-results.json")
    rows = [lane(path / "gates-01", name)[0] for name in
            ("core-default", "core-feature", "tui")]
    runs, passed = sum(r["run"] for r in rows), sum(r["passed"] for r in rows)
    alone = lane(path / "gates-01", "alone-feature")[0]["run"]
    require((runs, passed, alone) ==
            (receipt["gate_executions"], receipt["gate_passed"], receipt["alone_executions"]),
            "round-76 aggregate receipt")
    for row in rows:
        require(row == next(r for r in receipt["lanes"] if r["lane"] == row["lane"]),
                "aggregate lane differs")
    return f"gate_executions={runs}; gate_passed={passed}; alone_executions={alone}; overlapping test sets"


def historical():
    claim = match(r"at (\d+\.\d+)s and (\d+\.\d+)s \((\d+)/(\d+) passed,\s*(\d+) timed out, exit (\d+)\)")
    row, text = lane(SCOPE / "acct-receipt-66/gates-01", "core-feature")
    times = list(dict.fromkeys(re.findall(
        r"(?:TIMEOUT|TMT)\s*\[\s*([0-9.]+)s\].*" + re.escape(TEST), text)))
    require(times == list(claim.group(1, 2)), "historical timeout durations")
    require(tuple(map(int, claim.group(3, 4, 5, 6))) ==
            (row["passed"], row["run"], row["timed_out"], row["exit"]), "historical timeout counts")
    match(r"timed out twice")
    return (f"TRY durations={','.join(times)}s; passed/run={row['passed']}/{row['run']}; "
            f"timed_out={row['timed_out']}; exit={row['exit']}; test={TEST}; cause remains unproven")


def timing(name, directory, lane_name, pattern):
    quoted = match(pattern)[1]
    _, text = lane(SCOPE / directory, lane_name)
    actual = re.findall(r"PASS\s*\[\s*([0-9.]+)s\].*" + re.escape(TEST), text)
    require(actual == [quoted], name + " raw PASS duration differs")
    return quoted + "s; target test raw PASS line agrees"


def mutations():
    path = SCOPE / "acct-controls-72"
    coverage = load(path / "monetary-coverage.json")
    raw = load(path / coverage["raw_results"])
    plan = load(path / coverage["plan"])
    require([r["mutation"] for r in raw["controls"]] == plan, "mutation plan differs")
    derived = {}
    for index, row in enumerate(raw["controls"]):
        diagnostic = json.loads(row["stderr"].splitlines()[0])
        key = row["mutation"]["check"]
        require(row["exit"] != 0 and not row["receipt_written"], f"mutation {index} did not fail")
        require(key in diagnostic["semantic_failures"] and
                key in diagnostic["monetary_checks_evaluated"], f"mutation {index} failed wrong reason")
        derived.setdefault(key, []).append(index)
    require(derived == coverage["check_to_zero_based_mutation_indices"], "coverage map differs")
    quoted = match(r"maps (\d+) named checks to (\d+) failing diagnostic mutations")
    require((len(derived), len(plan)) == tuple(map(int, quoted.group(1, 2))) ==
            (coverage["checks"], coverage["mutations"]), "mutation/check counts differ")
    return f"named_checks={len(derived)}; failing_diagnostic_mutations={len(plan)}; each named reason reached"


def pages():
    source = SCOPE / "acct-readers-61"
    run = source / "reader-run-05"
    bindings = load(source / "page-content-digests.json")
    require({p.name.removesuffix("-selected.json") for p in run.glob("*-selected.json")} == set(bindings),
            "JSON page membership differs")
    categories = {"priced": [], "unknown": [], "zero": []}
    for name, expected in bindings.items():
        lines = load(run / (name + "-selected.json"))
        require(digest(json.dumps(lines, ensure_ascii=True).encode()) == expected, name + " JSON digest")
        text = " ".join(lines)
        values = re.findall(r"Known (?:subtotal|estimate) exact USD: ([0-9.]+)", text)
        if not values:
            continue
        require(len(values) == 1, name + " ambiguous subtotal")
        if Decimal(values[0]) > 0:
            category = "priced"
        elif "Estimated token cost: unknown" in text:
            category = "unknown"
        elif "No recorded attempts" in text:
            category = "zero"
        else:
            raise ValueError(name + " unclassified numeric page")
        categories[category].append(name)
    receipt = load(SCOPE / "acct-controls-72/controls-04/baseline-digest-bound.json")
    for category, field in (("priced", "priced_reconciliation_pages"),
                            ("unknown", "unknown_cost_pages"), ("zero", "zero_recorded_pages")):
        require(set(categories[category]) == set(receipt[field]), category + " receipt pages differ")
    quoted = match(r"reconciles (\d+) priced views \(not (\d+) independent attempts\),\s*(\d+) unknown-cost views and (\d+) zero-recorded views")
    counts = tuple(len(categories[k]) for k in ("priced", "unknown", "zero"))
    require(counts == tuple(map(int, quoted.group(1, 3, 4))) and quoted[1] == quoted[2],
            "view counts differ")
    return f"priced={counts[0]}; unknown_cost={counts[1]}; zero_recorded={counts[2]}; derived from bound page contents"


def bindings():
    source = SCOPE / "acct-readers-61"
    pages = load(source / "page-content-digests.json")
    viewports = load(source / "viewport-content-digests.json")
    run = source / "reader-run-05"
    require({p.name for p in run.glob("*.txt.gz")} == set(viewports), "viewport membership")
    for name, expected in viewports.items():
        require(digest(gzip.decompress(data(run / name))) == expected, name + " viewport digest")
    quoted = match(r"bind (\d+) JSON pages and (\d+) viewports")
    require((len(pages), len(viewports)) == tuple(map(int, quoted.group(1, 2))), "binding counts")
    return f"JSON_pages={len(pages)}; viewports={len(viewports)}; content SHA-256 agrees"


def scope_zero():
    run = SCOPE / "acct-scope-62/scope-run-01"
    store = load(run / "store-during-fresh-reader.json")
    require(store == load(run / "store-before-fresh-reader.json"), "scope store changed")
    attempts = {r["attempt_id"]: json.loads(r["payload"]) for r in store["draft_accounting_attempts"]}
    contributions = store["draft_accounting_contributions"]
    require(len(contributions) == len(attempts) and
            {r["attempt_id"] for r in contributions} == set(attempts), "scope contribution identity")
    total = Decimal(0)
    for contribution in contributions:
        require(contribution["utc_day"] == 20675, "scope contribution day")
        quotes = [json.loads(q["payload"]) for q in store["draft_accounting_estimates"]
                  if (q["attempt_id"], q["evidence"]) ==
                  (contribution["attempt_id"], contribution["evidence"])]
        require(len(quotes) == 1, "scope quote revision selection")
        quote = quotes[0]
        require(quote["attempt"] == attempts[contribution["attempt_id"]], "scope quote identity")
        usage, rates = quote["usage"], quote["snapshot"]["rates"]
        amount = sum(Decimal(usage[key]) * Decimal(rates[key] or 0)
                     for key in ("noncached", "read", "write", "output")) / 1000000
        require(str(amount) == quote["known_subtotal"] == quote["all_buckets_priced"],
                "scope quote arithmetic")
        total += amount
    quoted = match(r"despite four priced attempts totaling\s*USD ([0-9.]+)")
    require(len(attempts) == 4 and total == Decimal(quoted[1]), "scope count/amount")
    zero = match(r"Known subtotal exact USD: ([0-9.]+)")
    text = " ".join(load(run / "fresh-scope-zero-selected.json"))
    require(re.findall(r"Known subtotal exact USD: ([0-9.]+)", text) == [zero[1]], "scope zero render")
    return f"priced_attempts={len(attempts)}; USD={total}; fresh_root_rendered_USD={zero[1]}; selected quote revisions only"


def inventory(name):
    document = load(SCOPE / name / "scope.json")
    entries = document.get("files", document.get("new_files_excluding_this_inventory"))
    require(len({r["path"] for r in entries}) == len(entries), "duplicate inventory paths")
    byte_count = line_count = 0
    for row in entries:
        raw = data(ROOT / row["path"])
        lines = None if row["path"].endswith(".gz") else len(raw.splitlines())
        require((len(raw), lines, digest(raw)) == (row["bytes"], row["lines"], row["sha256"]),
                row["path"] + " bytes/lines/SHA-256 differ")
        byte_count += len(raw)
        line_count += lines or 0
    if name == "acct-acceptance-75":
        for kind in ("new", "modified"):
            rows = [r for r in entries if r["kind"] == kind]
            actual = (len(rows), sum(r["bytes"] for r in rows), sum(r["lines"] or 0 for r in rows))
            require(actual == tuple(document["total_" + kind + suffix]
                    for suffix in ("_files", "_bytes", "_text_lines")), kind + " totals differ")
        require((byte_count, line_count) ==
                (document["total_current_bytes"], document["total_current_text_lines"]), "current totals")
        return (f"entries={len(entries)}; new_files={document['total_new_files']}; "
                f"new_bytes={document['total_new_bytes']}; new_text_lines={document['total_new_text_lines']}; "
                f"modified_files={document['total_modified_files']}; "
                f"modified_bytes={document['total_modified_bytes']}; "
                f"modified_text_lines={document['total_modified_text_lines']}; "
                f"current_bytes={byte_count}; current_text_lines={line_count}; full-file sizes, not diff lines")
    require((byte_count, line_count) == (document["total_new_bytes"], document["total_new_text_lines"]),
            "round-72 totals differ")
    return f"entries={len(entries)}; current_bytes={byte_count}; current_text_lines={line_count}; hashes agree"


def inventory_kinds():
    document = load(SCOPE / "acct-acceptance-75/scope.json")
    result = subprocess.run(["git", "ls-tree", "-r", "--name-only", document["base"]],
                            cwd=ROOT, capture_output=True, text=True)
    if result.returncode:
        raise FileNotFoundError("original round-75 base Git tree absent (for example a shallow clone); kinds cannot be re-derived")
    names = set(result.stdout.splitlines())
    counts = {"new": 0, "modified": 0}
    for row in document["files"]:
        kind = "modified" if row["path"] in names else "new"
        require(row["kind"] == kind, row["path"] + " kind")
        counts[kind] += 1
    original = load(SCOPE / "acct-fitness-76/acct-controls-72-scope-before.json")
    old = {r["path"]: r["sha256"] for r in original["new_files_excluding_this_inventory"]}
    current = load(SCOPE / "acct-controls-72/scope.json")["new_files_excluding_this_inventory"]
    changes = sum(old[r["path"]] != r["sha256"] for r in current)
    match(r"disclose the three\s*round-75 edits")
    require(changes == 3, "round-72 changed entries")
    correction = load(HERE / "inventory-correction.json")
    require(digest(data(SCOPE / "acct-acceptance-75/scope.json")) == correction["after_sha256"],
            "inventory correction digest")
    return (f"{counts['new']} added paths and {counts['modified']} modified paths against original base; "
            f"round-72 changed entries={changes}")


def package_log():
    path = SCOPE / "acct-fitness-76"
    manifest = load(path / "package-manifest.json")
    require(manifest["exit"] == 0 and not manifest["launched"], "package manifest status")
    require(digest(gzip.decompress(data(path / "package-build.log.gz"))) == manifest["log_sha256"],
            "package build log digest")
    return "build receipt and raw log agree; launch/functional acceptance not claimed"


def package_file(row, local=False):
    path = ROOT / row["path"]
    if local:
        if not path.is_file():
            raise FileNotFoundError(row["path"] + " absent locally")
        with path.open("rb") as stream:
            actual = hashlib.file_digest(stream, "sha256").hexdigest()
    else:
        try:
            actual = digest(data(path))
        except FileNotFoundError as exc:
            raise FileNotFoundError(
                str(exc) + f"; cannot re-derive bytes={row['bytes']}, mode={row['mode']} or SHA-256"
            ) from exc
    require((path.stat().st_size, actual, path.stat().st_mode & 0o777) ==
            (row["bytes"], row["sha256"], int(row["mode"], 8)), "package size/hash/mode")
    return f"bytes={row['bytes']}; SHA-256 and mode agree; not launched"


def coverage():
    # Detect unmatched digits only; worded quantities and excluded regions are not audited.
    text = list(DOCUMENT)
    for start, end in CLAIMS:
        text[start:end] = " " * (end - start)
    remaining = "".join(text)
    remaining = re.sub(r"\]\([^)]*\)|```.*?```", "", remaining, flags=re.S)
    remaining = re.sub(r"PF-60(?:-S03)?|S03|P2|round[- ](?:66|69|72|75|76)",
                       "", remaining, flags=re.I)
    numbers = re.findall(r"\b\d+(?:\.\d+)?\b", remaining)
    require(not numbers, "unmapped numerical claims in acceptance.md: " + repr(numbers))
    return ("no unmatched digit-form quantities outside matched claim spans, fenced code and link targets; "
            "known round/sprint/severity identifiers excluded; worded quantities and excluded regions not audited")


def main():
    global TRACKED, DOCUMENT
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--local-package", action="store_true")
    args = parser.parse_args()
    TRACKED = set(subprocess.check_output(["git", "ls-files"], cwd=ROOT, text=True).splitlines())
    DOCUMENT = data(SCOPE / "acct-acceptance-75/acceptance.md").decode()
    print("Acceptance reconciliation: retained evidence only; no new functional qualification.")
    check("scope-zero", scope_zero)
    check("priced-page counts", pages)
    check("mutation coverage", mutations)
    check("capture bindings", bindings)
    path = SCOPE / "acct-fitness-76/gates-01"
    for name in ("prerequisites", "core-default", "alone-feature", "core-feature", "tui"):
        check("round-76 " + name, lambda name=name: gate_summary(path, name))
    check("round-76 aggregate", gates)
    check("round-66 historical timeout", historical)
    for name, directory, lane_name, pattern in (
        ("round-66 default", "acct-receipt-66/gates-01", "core-default", r"Default passed in (\d+\.\d+)s"),
        ("round-69 alone", "acct-bindings-69/gates-02", "alone-feature", r"passed alone in\s*(\d+\.\d+)s"),
        ("round-69 feature", "acct-bindings-69/gates-02", "core-feature", r"full feature lane in (\d+\.\d+)s"),
        ("round-75 feature", "acct-acceptance-75/gates-01", "core-feature", r"round 75 passed in-lane in\s*(\d+\.\d+)s"),
    ):
        check(name + " timing", lambda n=name, d=directory, l=lane_name, p=pattern: timing(n, d, l, p))
    for name in ("acct-controls-72", "acct-acceptance-75"):
        check(name + " inventory", lambda name=name: inventory(name))
    check("inventory classifications and correction", inventory_kinds)
    check("package build evidence", package_log)
    try:
        manifest = load(SCOPE / "acct-fitness-76/package-manifest.json")
        for row in manifest["files"]:
            check("committed package " + Path(row["path"]).name, lambda row=row: package_file(row))
            if args.local_package:
                check("local package " + Path(row["path"]).name, lambda row=row: package_file(row, True))
    except (OSError, ValueError, KeyError) as exc:
        check("package manifest", lambda: (_ for _ in ()).throw(exc))
    check("acceptance numerical coverage", coverage)
    counts = {key: RESULTS.count(key) for key in ("AGREE", "DISAGREE", "UNAVAILABLE")}
    status = 1 if counts["DISAGREE"] else 2 if counts["UNAVAILABLE"] else 0
    print(f"RESULT agreement={counts['AGREE']} disagreement={counts['DISAGREE']} unavailable={counts['UNAVAILABLE']} exit={status}")
    return status


if __name__ == "__main__":
    sys.exit(main())

"""Audit arithmetic and bucket labels against driver windows, never page membership."""
from datetime import datetime
from decimal import Decimal
import json
from pathlib import Path
import re
import sys

def audit(run):
    load = lambda name: json.loads((run / name).read_text())
    results, rows = load("results.json"), load("independent-arithmetic.json")
    rates = {"gpt-5.6-sol": ("5", ".5", "30"), "gpt-5.6-terra": ("2.5", ".25", "15")}
    stamp = lambda x: datetime.fromisoformat(x.replace("Z", "+00:00"))
    for row in rows:
        a, b, c = map(Decimal, rates[row["model"]])
        exact = ((row["input"]-row["cached"])*a+row["cached"]*b+row["output"]*c)/Decimal(1000000)
        assert exact == Decimal(row["exact_usd"])
    # Historical driver wrote windows in order but did not write page names.
    legacy = ["week-edge-hours-0", "week-edge-hours-1", "month-edge-hours-0",
              "month-edge-hours-1", "weeks-0", "weeks-1", "months-0", "months-1"]
    windows = {b.get("page", legacy[i] if i < len(legacy) else ""): b
               for i, b in enumerate(results["buckets"])}
    windows["breakdown-bucket"] = results["buckets"][6]
    windows["partial-full-neighbour"] = results["buckets"][1]
    checked, labels = 0, 0
    for path in sorted(run.glob("*-selected.json")):
        selected = json.loads(path.read_text())
        exact = [s for s in selected if s.startswith(("Known subtotal exact USD: ", "Known estimate exact USD: "))]
        name = path.name.removesuffix("-selected.json")
        base = name.split("-Request-")[0]
        window = windows.get(base)
        if window:
            interval = next((s for s in selected if s.startswith("Bucket: [")), None)
            assert interval is not None, (path, "missing bucket label")
            start, end = re.search(r"Bucket: \[([^,]+), ([^)]+)\)", interval).groups()
            assert (stamp(start), stamp(end)) == (stamp(window["start"]), stamp(window["end"])), (path, "wrong bucket label")
            labels += 1
        if not exact:
            continue
        if any(s.startswith("Attempt: ") for s in selected):
            model = next(s.removeprefix("Model: ") for s in selected if s.startswith("Model: "))
            inp = int(next(s.removeprefix("Input: ") for s in selected if s.startswith("Input: ")))
            candidates = [r for r in rows if r["model"] == model and r["input"] == inp]
            assert len(candidates) == 1, (path, candidates)
            expected = Decimal(candidates[0]["exact_usd"])
        elif name.startswith("group-"):
            expected = sum((Decimal(rows[i]["exact_usd"]) for i in results["groups"][int(name.split("-")[1])]["emissions"]), Decimal(0))
        else:
            if window:
                members = [r for r in rows if r["kind"] != "orphan"
                           and stamp(window["start"]) <= stamp(r["fixture_utc"]) < stamp(window["end"])]
            elif name.startswith("unknown-"):
                case = next(c for c in results["unknown_causes"] if name == "unknown-"+c["cause"])
                # New driver records this explicit day; old run's chosen day equals
                # the recorded September bucket start, independent of page text.
                start = case.get("start", results["buckets"][7]["start"])
                end = case.get("end", "2026-09-02T00:00:00Z")
                members = [r for r in rows if r["kind"] == "root"
                           and stamp(start) <= stamp(r["fixture_utc"]) < stamp(end)]
            else:
                raise AssertionError(("Unmapped total", path))
            expected = sum((Decimal(r["exact_usd"]) for r in members), Decimal(0))
        assert all(Decimal(s.split(": ", 1)[1]) == expected for s in exact), (path, exact, expected)
        checked += 1
    return dict(exact_numeric_pages_checked=checked, bucket_labels_checked=labels,
                buckets=len(results["buckets"]), groups=len(results["groups"]),
                unknown_causes=len(results["unknown_causes"]), passed=True,
                expected_source="wire counts, fixed rates, driver-recorded windows; no results.passed gate",
                limitation="Membership expectations are driver-derived; store reads establish fixture identity/ancestry, not independent ancestry truth.")

if __name__ == "__main__":
    receipt = audit(Path(sys.argv[1]))
    # Explicit output prevents rewriting a closed run's historical receipt.
    with Path(sys.argv[2]).open("x") as output:
        output.write(json.dumps(receipt, indent=2)+"\n")
    print(json.dumps(receipt))

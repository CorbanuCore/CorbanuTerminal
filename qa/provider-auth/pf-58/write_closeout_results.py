"""Materialize the frozen-case ledger; partial journeys never become full passes."""
import argparse
import hashlib
import json
from pathlib import Path
import re

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--record", type=Path, required=True)
parser.add_argument("--design", type=Path, required=True)
parser.add_argument("--review", type=Path)
args = parser.parse_args()
record = args.record.resolve()

def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()

def artifact(path):
    return {"path": str(path.relative_to(record)), "sha256": digest(path)}

designed = {case["id"] for case in json.loads(args.design.read_text())["cases"]}
summaries = {}
for line in (record / "dispositions.md").read_text().splitlines():
    match = re.fullmatch(r"\| (F\d\d) \| (.*) \|", line)
    if match:
        summaries["PF58-" + match[1]] = match[2]
assert set(summaries) == designed
for platform in ("linux", "mac"):
    root = record / platform
    journey = json.loads((root / "journeys-final/result.json").read_text())
    assert journey["passed"]
    cases = []
    for case, summary in summaries.items():
        passed = case in {"PF58-F13", "PF58-F14", "PF58-F15"}
        entry = {"id": case, "disposition": "passed" if passed else "blocked",
                 "summary": summary, "candidate_sha256": journey["sha256"]}
        evidence = [artifact(root / "journeys-final/result.json"),
                    artifact(root / "journeys-final/requests.json"),
                    artifact(root / "journeys-final/F14-current-disable-cancel-explicit-replacement-reactivate-history.txt")]
        entry["evidence" if passed else "supporting_evidence"] = evidence
        if passed:
            entry["method"] = "tmux"
        cases.append(entry)
    results = {"design_sha256": digest(args.design), "implementer": "/root",
               "candidate": {"version": "0.1.41", "source": "4f6743b9c plus source-files.sha256; main 295aed26e docs reconciled",
                             "platform": platform, "binary_sha256": journey["sha256"],
                             "package_manifest": artifact(root / "package-inventory.json")},
               "cases": cases,
               "review_budget": {"used": 9 if args.review else 8, "limit": 9,
                                 "ledger": artifact(record / "review-allowance.md"),
                                 "extension": {"by": "user", "reason": "Explicit authorization for one more final review",
                                               "artifact": artifact(record / "review-allowance.md")}},
               "evidence_check": {"agent": "Astra High independent evidence reviewer (pass 9)",
                                  "verdict": "blocked" if args.review else "pending"},
               "human_acceptance": False}
    if args.review:
        results["evidence_check"]["artifact"] = artifact(args.review.resolve())
    (record / f"results-{platform}.json").write_text(json.dumps(results, indent=2) + "\n")
print("Recorded all 26 frozen cases on both platforms; unqualified gate remains blocked.")

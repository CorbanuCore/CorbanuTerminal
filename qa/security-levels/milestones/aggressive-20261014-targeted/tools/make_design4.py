"""Write the targeted re-run's design.json: the third run's design.json (all 64 cases with their 2026-10-11 and
2026-10-13 amendments), each with its 2026-10-14 amendment if any.

usage: python3 tools/make_design4.py RECORD_DIR THIRD_RECORD_DIR
"""
import hashlib, json, pathlib, sys

root, third = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
d = root / "design"
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
ref = lambda p: {"path": str(p.relative_to(root)), "sha256": sha(p)}
base = json.loads((third / "design.json").read_text())
amend = {a["case"]: a for a in json.loads((d / "amendments4-final.json").read_text())["amendments"]}
cases = []
for c in base["cases"]:
    c = dict(c)
    if c["id"] in amend:
        c["amendment_20261014"] = amend[c["id"]]
    cases.append(c)
assert len(amend) == sum(1 for c in cases if "amendment_20261014" in c), "every amendment applies to a case"
design = {
    "feature": "Milestone \"Aggressive ships\" targeted re-run (P1 security hardening, qualification lane)",
    "designer": base["designer"],
    "frozen_designer": base["frozen_designer"],
    "second_run_designer": base["second_run_designer"],
    "amendment_reviewer": "01a13954-bc1d-7d31-97e1-699adfdceda2",
    "fresh_context": True, "code_blind": True, "results_blind": True,
    "isolation": "instruction-only",
    "brief": ref(d / "review-packet/scope.md"),
    "proposal": ref(d / "amendment-review-output.md"),
    "access_record": ref(d / "amendment-review-session.txt"),
    "third_run_design": {"path": "../aggressive-20261013-third/design.json", "sha256": sha(third / "design.json")},
    "amendments": ref(d / "amendments4-final.json"),
    "amendments_draft": ref(d / "amendments4-v1.json"),
    "amendment_review": ref(d / "amendment-review-output.md"),
    "amendment_review_prompt": ref(d / "amendment-review-prompt.md"),
    "amendment_review_events": ref(d / "amendment-review-events.jsonl.gz"),
    "screenshots": [ref(p) for p in sorted((d / "review-packet/new-screens-4").glob("*.txt"))],
    "frozen_at": "40e36a39cf (pushed 2026-10-14T07:37Z before the VM was started; digests in design/FROZEN.sha256)",
    "cases": cases,
}
out = root / "design.json"
out.write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(sha(out), len(cases))

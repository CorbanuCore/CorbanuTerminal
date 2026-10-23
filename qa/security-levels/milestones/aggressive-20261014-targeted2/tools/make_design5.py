"""Write the second targeted re-run's design.json: the first targeted run's design.json (all 64 cases with their
amendments of 2026-10-11, 2026-10-13 and 2026-10-14 part 5), each with its amendment 5 (2026-10-14, part 6) or its
disposition without execution, if any.

usage: python3 tools/make_design5.py RECORD_DIR TARGETED_RECORD_DIR
"""
import hashlib, json, pathlib, sys

root, prev = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
d = root / "design"
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
ref = lambda p: {"path": str(p.relative_to(root)), "sha256": sha(p)}
base = json.loads((prev / "design.json").read_text())
final = json.loads((d / "amendments5-final.json").read_text())
amend = {a["case"]: a for a in final["amendments"]}
disp = {}
for row in final["dispositions"]:
    for c in row["cases"]:
        disp.setdefault(c, []).append({"decisions": row["decisions"], "disposition": row["disposition"]})
cases = []
for c in base["cases"]:
    c = dict(c)
    if c["id"] in amend:
        c["amendment_20261014b"] = amend[c["id"]]
    if c["id"] in disp:
        c["disposition_20261014b"] = disp[c["id"]]
    cases.append(c)
assert len(amend) == sum(1 for c in cases if "amendment_20261014b" in c), "every amendment applies to a case"
assert len(disp) == sum(1 for c in cases if "disposition_20261014b" in c), "every disposition applies to a case"
design = {
    "feature": "Milestone \"Aggressive ships\" second targeted re-run (P1 security hardening, qualification lane)",
    "designer": base["designer"],
    "frozen_designer": base["frozen_designer"],
    "second_run_designer": base["second_run_designer"],
    "driver_designer": {"agent": "01a13cc1-be21-7c62-9bf7-19bd1d23e6ab", "output": ref(d / "designer-output.md"),
                        "prompt": ref(d / "designer-prompt.md"), "session": ref(d / "designer-session.txt")},
    "amendment_reviewer": "01a13ccf-596f-74f2-8980-26697b8e57bd",
    "fresh_context": True, "code_blind": True, "results_blind": True,
    "isolation": "instruction-only",
    "brief": ref(d / "designer-prompt.md"),
    "proposal": ref(d / "amendment-review-output.md"),
    "access_record": ref(d / "amendment-review-session.txt"),
    "targeted_run_design": {"path": "../aggressive-20261014-targeted/design.json", "sha256": sha(prev / "design.json")},
    "amendments": ref(d / "amendments5-final.json"),
    "amendments_draft": ref(d / "amendments5-v1.json"),
    "amendment_review": ref(d / "amendment-review-output.md"),
    "amendment_review_prompt": ref(d / "amendment-review-prompt.md"),
    "amendment_review_events": ref(d / "amendment-review-events.jsonl.gz"),
    "drivers": [ref(p) for p in sorted((root / "executor-packet/v6/drivers").glob("*.json"))],
    "screenshots": [ref(p) for p in sorted((root / "executor-packet/v6/new-screens-5").glob("*.txt"))],
    "frozen_at": "aaf2f999f7 (committed 2026-10-14T23:56:22Z, pushed 23:56:25Z, before the VM was started at 23:56:35Z; digests in design/FROZEN.sha256)",
    "cases": cases,
}
out = root / "design.json"
out.write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(sha(out), len(cases))

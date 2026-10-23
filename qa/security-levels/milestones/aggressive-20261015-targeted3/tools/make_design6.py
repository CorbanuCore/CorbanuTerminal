"""Write the third targeted re-run's design.json: the second targeted run's design.json (all 64 cases with their
amendments up to 2026-10-14 part 6), each with its amendment 6 (2026-10-15, part 7) or its disposition without
execution, if any.

usage: python3 tools/make_design6.py RECORD_DIR TARGETED2_RECORD_DIR
"""
import hashlib, json, pathlib, sys

root, prev = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
d = root / "design"
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
ref = lambda p: {"path": str(p.relative_to(root)), "sha256": sha(p)}
base = json.loads((prev / "design.json").read_text())
final = json.loads((d / "amendments6-final.json").read_text())
amend = {a["case"]: a for a in final["amendments"]}
disp = {}
for row in final["dispositions"]:
    for c in row["cases"]:
        disp.setdefault(c, []).append({"decisions": row["decisions"], "disposition": row["disposition"]})
cases = []
for c in base["cases"]:
    c = dict(c)
    if c["id"] in amend:
        c["amendment_20261015"] = amend[c["id"]]
    if c["id"] in disp:
        c["disposition_20261015"] = disp[c["id"]]
    cases.append(c)
assert len(amend) == sum(1 for c in cases if "amendment_20261015" in c)
assert len(disp) == sum(1 for c in cases if "disposition_20261015" in c)
sess = lambda n: (d / n).read_text().split('"thread_id":"')[1].split('"')[0]
design = {
    "feature": "Milestone \"Aggressive ships\" third targeted re-run (P1 security hardening, qualification lane)",
    "designer": base["designer"],
    "frozen_designer": base["frozen_designer"],
    "second_run_designer": base["second_run_designer"],
    "driver_designer_targeted2": base["driver_designer"],
    "driver_designer": {"agent": sess("designer-session.txt"), "output": ref(d / "designer-output.md"),
                        "prompt": ref(d / "designer-prompt.md"), "session": ref(d / "designer-session.txt"),
                        "failed_attempt": ref(d / "designer-attempt1-session.txt")},
    "amendment_reviewer": sess("review-session.txt"),
    "fresh_context": True, "code_blind": True, "results_blind": True,
    "isolation": "instruction-only",
    "brief": ref(d / "designer-prompt.md"),
    "proposal": ref(d / "review-output.md"),
    "access_record": ref(d / "review-session.txt"),
    "targeted2_run_design": {"path": "../aggressive-20261014-targeted2/design.json", "sha256": sha(prev / "design.json")},
    "amendments": ref(d / "amendments6-final.json"),
    "amendments_drafts": [ref(d / "amendments6-v1.json"), ref(d / "amendments6-v2.json")],
    "amendment_review": ref(d / "review-output.md"),
    "amendment_review_prompt": ref(d / "review-prompt.md"),
    "amendment_review_events": ref(d / "review-events.jsonl.gz"),
    "drivers": [ref(p) for p in sorted((root / "executor-packet/v7/drivers").glob("*.json"))],
    "screenshots": [ref(p) for p in sorted((root / "executor-packet/v7/new-screens-6").glob("*.txt"))],
    "frozen_at": "9daaaf93b8 (committed and pushed 2026-10-15T21:32:55Z/21:32:58Z, before the first VM start attempt at 21:33:10Z; digests in design/FROZEN.sha256)",
    "cases": cases,
}
out = root / "design.json"
out.write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(sha(out), len(cases))

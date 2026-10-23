"""Write the third run's design.json: the second run's effective design (first run's frozen cases with their
2026-10-11 amendments, plus the AGG-R cases), each with its 2026-10-13 amendment if any, plus the AGG-T cases.

usage: python3 tools/make_design3.py RECORD_DIR
"""
import hashlib, json, pathlib, sys

root = pathlib.Path(sys.argv[1]); d = root / "design"
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
ref = lambda p: {"path": str(p.relative_to(root)), "sha256": sha(p)}
r2 = json.loads((d / "design-r2-20261012.json").read_text())
amend = {a["case"]: a for a in json.loads((d / "amendments3-final.json").read_text())["amendments"]}
new = json.loads((d / "new-cases3.json").read_text())
cases = []
for c in r2["cases"]:
    c = dict(c)
    if c["id"] in amend:
        c["amendment_20261013"] = amend[c["id"]]
    cases.append(c)
cases += [dict(c, source="third-run designer (Opus 5.5 High, thread " + new["designer_thread"] + ")") for c in new["cases"]]
assert len(amend) == sum(1 for c in cases if "amendment_20261013" in c), "every amendment applies to a case"
design = {
    "feature": "Milestone \"Aggressive ships\" third run (P1 security hardening, qualification lane)",
    "designer": new["designer_thread"],
    "frozen_designer": r2["frozen_designer"],
    "second_run_designer": r2["designer"],
    "fresh_context": True, "code_blind": True, "results_blind": True,
    "isolation": "instruction-only",
    "brief": ref(d / "packet/intent-addendum-20261013.md"),
    "original_brief": ref(d / "packet/intent.md"),
    "screenshots": [ref(p) for p in sorted((d / "packet/new-screens").glob("*.txt"))],
    "proposal": ref(d / "reviewer-designer-output.md"),
    "confirmation": ref(d / "confirm-output.md"),
    "proposal_cases": ref(d / "new-cases3.json"),
    "access_record": ref(d / "designer-session.md"),
    "designer_events": ref(d / "reviewer-designer-events.jsonl"),
    "frozen_design": ref(d / "frozen-design-20261010.json"),
    "second_run_design": ref(d / "design-r2-20261012.json"),
    "amendments": ref(d / "amendments3-final.json"),
    "owner_decisions": ref(d / "owner-decisions-20261013-final.md"),
    "questions_and_answers": ref(d / "packet/questions-and-answers.md"),
    "go_file": ref(d / "qual-aggr3.go"),
    "frozen_at": "090bd128a6 (pushed 2026-10-13T17:14Z before execution; digests in design/FROZEN.sha256)",
    "cases": cases,
}
out = root / "design.json"
out.write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(sha(out), len(cases))

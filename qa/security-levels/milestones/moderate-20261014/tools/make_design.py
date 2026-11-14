"""Write design.json for the Moderate first run record. usage: make_design.py RECORD_DIR"""
import hashlib
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])


def ref(p):
    return {"path": str(p.relative_to(root)), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}


d = root / "design"
proposal = json.loads((d / "original-proposal-cases.json").read_text())
amend = json.loads((d / "amendments-v2.json").read_text())
cases = [dict(c, source="designer") for c in proposal["cases"]]
for a in amend["amendments"]:
    if a["kind"] == "new":
        cases.append({"id": a["case"], "priority": a["priority"], "starting_conditions": a["starting_conditions"],
                      "actions": a["actions"], "expected": a["expected"],
                      "source": "integrator amendment (additive, from the design review; " + ", ".join(a["decisions"]) + ")"})
design = {
    "feature": "Milestone \"Moderate ships\", first code-blind run (P1 security hardening, qualification lane)",
    "designer": "01a13ad5-b332-7610-a491-7202b0eaa190",
    "design_reviewer": "01a13add-0c6c-79e0-b31a-d0c595bd15ae",
    "fresh_context": True,
    "code_blind": True,
    "results_blind": True,
    "isolation": "instruction-only",
    "brief": ref(d / "packet/intent.md"),
    "screenshots": [ref(p) for p in sorted((d / "packet/screens").iterdir())],
    "proposal": ref(d / "original-proposal.md"),
    "access_record": ref(d / "designer-session.md"),
    "owner_decisions": ref(d / "owner-decisions-20261014.md"),
    "questions_and_answers": ref(d / "questions-and-answers.md"),
    "amendments": ref(d / "amendments-v2.json"),
    "design_review": ref(root / "design-review/reviewer-session.md"),
    "executor_packet": {"cases": ref(root / "executor-packet/v1/design-cases.md"),
                        "amendments": ref(root / "executor-packet/v1/amendments.md")},
    "named_limits": amend["named_limits"],
    "cases": cases,
}
(root / "design.json").write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(hashlib.sha256((root / "design.json").read_bytes()).hexdigest(), len(cases))

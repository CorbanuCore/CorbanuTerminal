"""Write the re-run design.json: the first run's frozen cases verbatim (each with its frozen amendment, if any)
plus the re-run designer's new cases.

usage: python3 tools/make_design_r.py RECORD_DIR
"""
import hashlib
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
d = root / "design"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def ref(path):
    return {"path": str(path.relative_to(root)), "sha256": sha(path)}


frozen = json.loads((d / "frozen-design-20261010.json").read_text())
amend = {a["case"]: a for a in json.loads((d / "amendments.json").read_text())["amendments"]}
new = json.loads((d / "new-cases.json").read_text())
cases = []
for c in frozen["cases"]:
    c = dict(c)
    if c["id"] in amend:
        c["amendment"] = amend[c["id"]]
    cases.append(c)
cases += [dict(c) for c in new["cases"]]
design = {
    "feature": "Milestone \"Aggressive ships\" re-run (P1 security hardening, qualification lane)",
    "designer": "01a12e74-b702-7402-b93d-63d03a499d02",
    "frozen_designer": frozen["designer"],
    "fresh_context": True,
    "code_blind": True,
    "results_blind": True,
    "isolation": "instruction-only",
    "brief": ref(d / "packet/intent-addendum.md"),
    "original_brief": ref(d / "packet/intent.md"),
    "screenshots": [ref(p) for p in sorted((d / "packet/new-screens").glob("*.txt"))],
    "proposal": ref(d / "reviewer-designer-output.md"),
    "proposal_cases": ref(d / "new-cases.json"),
    "access_record": ref(d / "designer-session.md"),
    "designer_events": ref(d / "reviewer-designer-events.jsonl"),
    "frozen_design": ref(d / "frozen-design-20261010.json"),
    "amendments": ref(d / "amendments.json"),
    "owner_decisions": ref(d / "packet/owner-decisions.md"),
    "questions_and_amendments": ref(d / "questions-and-answers.md"),
    "frozen_at": "4cf4083227f1cf5ab55f0c4b8a0df8258c7503d6 (pushed before execution; digests in design/FROZEN.sha256)",
    "cases": cases,
}
out = root / "design.json"
out.write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(sha(out), len(cases))

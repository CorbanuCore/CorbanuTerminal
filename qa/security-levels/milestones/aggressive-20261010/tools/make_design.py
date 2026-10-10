"""Write design.json from the designer's unedited proposal plus the recorded amendment.

usage: python3 tools/make_design.py RECORD_DIR
"""
import hashlib
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
design_dir = root / "design"


def ref(path):
    return {"path": str(path.relative_to(root)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


proposal = json.loads((design_dir / "original-proposal-cases.json").read_text())
cases = [dict(case, source="designer") for case in proposal["cases"]]
cases.append({
    "id": "AGG-A01",
    "priority": "blocker",
    "source": "integrator amendment A01 (additive; supersedes nothing)",
    "starting_conditions": "Fresh profile, Permissive, launched on route zai-anthropic with model glm-5.2.",
    "actions": [
        "Ask: Reply with exactly: pong",
        "Open /security, choose Aggressive, confirm with Enter, restart with r",
        "Ask: Reply with exactly: pong",
    ],
    "expected": [
        "Both turns complete with \"pong\"",
        "No error about a missing output limit and no refused request",
    ],
})
screens = sorted((design_dir / "packet/screens").glob("*.png"))
design = {
    "feature": "Milestone \"Aggressive ships\" (P1 security hardening, qualification lane)",
    "designer": "01a12807-f56f-7531-a8b9-b7e909b7e460",
    "fresh_context": True,
    "code_blind": True,
    "results_blind": True,
    "isolation": "instruction-only",
    "brief": ref(design_dir / "packet/intent.md"),
    "screenshots": [ref(p) for p in screens],
    "screen_texts": [ref(p.with_suffix(".txt")) for p in screens],
    "proposal": ref(design_dir / "original-proposal.md"),
    "proposal_cases": ref(design_dir / "original-proposal-cases.json"),
    "access_record": ref(design_dir / "designer-session.md"),
    "designer_events": ref(design_dir / "designer-events.jsonl"),
    "questions_and_amendments": ref(design_dir / "questions-and-amendments.md"),
    "cases": cases,
}
out = root / "design.json"
out.write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(hashlib.sha256(out.read_bytes()).hexdigest(), len(cases))

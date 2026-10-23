"""Write design.json for the original F01-F11 cases, verbatim from the frozen file.

Designer provenance (session, access record, brief, screenshots) was never supplied
for the original design; those fields stay unattested rather than manufactured.
"""
import hashlib
import json
import pathlib
import re
import sys

record = pathlib.Path(sys.argv[1])
frozen = record / "original-F01-F11.md"
text = frozen.read_text()
digest = hashlib.sha256(frozen.read_bytes()).hexdigest()
assert digest == "c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726"
cases = []
for match in re.finditer(r"^### (F\d\d) — (P\d): (.+?)\n\n(.*?)(?=^### |^## )", text, re.M | re.S):
    case_id, priority, title, body = match.groups()
    fields = dict(re.findall(r"^- (Starting state|Actions|Observable outcomes): (.+)$", body, re.M))
    cases.append({
        "id": case_id,
        "title": title,
        "original_priority": priority,
        "priority": "blocker",
        "starting_conditions": fields["Starting state"],
        "actions": [fields["Actions"]],
        "expected": [fields["Observable outcomes"]],
    })
assert [c["id"] for c in cases] == ["F%02d" % n for n in range(1, 12)], [c["id"] for c in cases]
design = {
    "feature": "PF-83-S01 request-correlated permission confirmation",
    "designer": None,
    "designer_note": "Original designer session/access packet, brief and screenshots were not supplied; "
                     "the proposal's own header is its only provenance. Not manufactured.",
    "fresh_context": None,
    "code_blind": None,
    "results_blind": None,
    "isolation": "instruction-only",
    "proposal": {"path": "original-F01-F11.md", "sha256": digest},
    "screenshots": [],
    "priority_note": "All originals treated as blockers (P1 F09/F11 included), matching the prior normalization; none waived.",
    "cases": cases,
}
(record / "design.json").write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(hashlib.sha256((record / "design.json").read_bytes()).hexdigest(), len(cases))

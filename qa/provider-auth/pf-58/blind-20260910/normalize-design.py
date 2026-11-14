"""Mechanical normalization of the frozen Markdown table, without rewriting cases."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent


def ref(path):
    return {"path": path, "sha256": hashlib.sha256((root / path).read_bytes()).hexdigest()}


cases = []
for line in (root / "original-proposal.md").read_text().splitlines():
    if line.startswith("| PF58-F"):
        identity, priority, start, actions, expected = [v.strip() for v in line.strip("|").split("|")]
        cases.append({"id": identity, "priority": priority.lower(),
                      "starting_conditions": start, "actions": [actions], "expected": [expected]})
assert len(cases) == 26
design = {"feature": "PF-58 provider setup, recovery and model selection",
          "designer": "/root/blind_functional_designer", "fresh_context": True,
          "code_blind": True, "results_blind": True, "isolation": "instruction-only",
          "brief": ref("packet/intent.md"), "screenshots": [ref("packet/" + name) for name in
          ("token-entry.png", "provider-manager.png", "model-picker.png")],
          "proposal": ref("original-proposal.md"), "access_record": ref("designer-session.md"),
          "cases": cases}
(root / "design.json").write_text(json.dumps(design, indent=2) + "\n")

"""Write design.json for the Moderate targeted re-run record. usage: make_design_t.py RECORD_DIR FIRST_RUN_RECORD_DIR"""
import hashlib, json, pathlib, sys

root, first = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
ref = lambda p: {"path": str(p.relative_to(root)), "sha256": sha(p)}
d = root / "design"
frozen = json.loads((first / "design.json").read_text())
design = {
    "feature": "Milestone \"Moderate ships\", targeted re-run (P1 security hardening, qualification lane)",
    "frozen_design": {"path": "../moderate-20261014/design.json", "sha256": sha(first / "design.json"),
                      "designer": frozen["designer"], "design_reviewer": frozen["design_reviewer"]},
    "amendment_designer": "01a1442d-d34b-75a3-aa05-1fe37176e68b",
    "amendment_reviewer": "01a14436-c85d-7ac3-bc39-3754fbe591ad",
    "fresh_context": True, "code_blind": True, "results_blind": True,
    "isolation": "read-only corbanu exec sandbox; working directory = the packet folder only",
    "designer_packet": [ref(p) for p in sorted((d / "designer-packet").rglob("*")) if p.is_file()],
    "designer_prompt": ref(d / "designer-prompt.md"), "designer_output": ref(d / "designer-output.md"),
    "review_prompt": ref(d / "review-prompt.md"), "review_output": ref(d / "review-output.md"),
    "access_records": [ref(d / "designer-events.jsonl.gz"), ref(d / "review-events.jsonl.gz"), ref(d / "sessions.txt")],
    "amendment_v3_as_designed": ref(d / "amendments-v3-designer.md"),
    "amendment_v3_draft": ref(d / "amendments-v3-draft.md"), "integrator_edits": ref(d / "integrator-edits.diff"),
    "amendment_v3_final": ref(d / "amendments-v3-final.md"),
    "freeze": ref(d / "FROZEN.sha256"),
    "executor_packet": {"cases": ref(root / "executor-packet/v3/design-cases.md"),
                        "amendments": ref(root / "executor-packet/v3/amendments.md"),
                        "amendments_v3": ref(root / "executor-packet/v3/amendments-v3.md")},
    "named_limits": frozen["named_limits"],
    "cases": frozen["cases"],
}
(root / "design.json").write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(sha(root / "design.json"), len(design["cases"]))

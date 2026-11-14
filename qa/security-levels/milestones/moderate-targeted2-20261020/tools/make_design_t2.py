"""Write design.json for the Moderate second targeted re-run record.
usage: make_design_t2.py RECORD_DIR FIRST_RUN_RECORD_DIR PREVIOUS_TARGETED_RECORD_DIR"""
import hashlib, json, pathlib, sys

root, first, prev = (pathlib.Path(a) for a in sys.argv[1:4])
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
ref = lambda p: {"path": str(p.relative_to(root)), "sha256": sha(p)}
d = root / "design"
frozen = json.loads((first / "design.json").read_text())
design = {
    "feature": "Milestone \"Moderate ships\", second targeted re-run (P1 security hardening, qualification lane)",
    "frozen_design": {"path": "../moderate-20261014/design.json", "sha256": sha(first / "design.json"),
                      "designer": frozen["designer"], "design_reviewer": frozen["design_reviewer"]},
    "previous_amendment": {"path": "../moderate-targeted-20261016/design.json", "sha256": sha(prev / "design.json")},
    "amendment_designer": "01a1575c-8067-7752-9868-0f5b849ce45c",
    "amendment_reviewer": "01a15761-3e17-7213-b521-a0fc9ac7decf",
    "fresh_context": True, "code_blind": True, "results_blind": True,
    "isolation": "read-only corbanu exec sandbox; working directory = the packet folder only",
    "designer_packet": [ref(p) for p in sorted((d / "designer-packet").rglob("*")) if p.is_file()],
    "designer_prompt": ref(d / "designer-prompt.md"), "designer_output": ref(d / "designer-output.md"),
    "review_prompt": ref(d / "review-prompt.md"), "review_output": ref(d / "review-output.md"),
    "access_records": [ref(d / "designer-events.jsonl.gz"), ref(d / "review-events.jsonl.gz"), ref(d / "sessions.txt")],
    "amendment_v4_as_designed": ref(d / "amendments-v4-designer.md"),
    "amendment_v4_draft": ref(d / "amendments-v4-draft.md"), "integrator_edits": ref(d / "integrator-edits.diff"),
    "amendment_v4_final": ref(d / "amendments-v4-final.md"),
    "freeze": ref(d / "FROZEN.sha256"),
    "executor_packet": {"cases": ref(root / "executor-packet/v4/design-cases.md"),
                        "amendments": ref(root / "executor-packet/v4/amendments.md"),
                        "amendments_v3": ref(root / "executor-packet/v4/amendments-v3.md"),
                        "amendments_v4": ref(root / "executor-packet/v4/amendments-v4.md")},
    "named_limits": frozen["named_limits"],
    "cases": frozen["cases"],
}
(root / "design.json").write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(sha(root / "design.json"), len(design["cases"]))

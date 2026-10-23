"""Write design.json (RECORD_TEMPLATE shape) for the PF-60-S10 record.
Cases = the designer's frozen JSON (verbatim fields) plus the three new cases from the review's amendments; every
amendment that touches a case is attached to it verbatim under "amendments" (the amendment governs).
usage: python3 make_design_json.py RECORD_DIR
"""
import hashlib, json, pathlib, re, sys

root = pathlib.Path(sys.argv[1])
d = root / "design"
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
ref = lambda p: {"path": str(p.relative_to(root)), "sha256": sha(p)}
out = (d / "designer-output.md").read_text()
frozen = json.loads(out[out.rindex("```json") + 7:out.rindex("```")])
rev = (d / "review-output.md").read_text()
amend = json.loads(rev[rev.rindex("```json") + 7:rev.rindex("```")])["amendments"]


def targets(a):
    if a["case"] == "ALL":
        return "ALL"
    return re.findall(r"ACC-\d\d", a["case"])


cases = []
for c in frozen["cases"]:
    row = dict(c)
    row["amendments"] = [a for a in amend if targets(a) == "ALL" or c["id"] in targets(a)]
    cases.append(row)
for a in amend:
    if a["case"].startswith("NEW ACC-"):
        t = a["text"]
        pri = t["priority"] if t["priority"] in ("blocker", "advisory") else "advisory"
        row = {"id": t["id"], "priority": pri, "priority_as_written": t["priority"], "group": t["group"],
               "starting_conditions": t["starting_conditions"], "actions": t["actions"], "expected": t["expected"],
               "added_by_amendment": a["n"],
               "amendments": [x for x in amend if targets(x) == "ALL"]}
        cases.append(row)
design = {
    "feature": "PF-60-S10: isolated code-blind acceptance of PF-60-S06 to S09 (accounting follow-ups)",
    "designer": "01a1428c-9592-7652-8620-53768ab043ba",
    "design_reviewer": "01a14298-8fd0-7381-aa6f-d4a5ee26429d",
    "fresh_context": True, "code_blind": True, "results_blind": True,
    "isolation": "instruction-only",
    "brief": ref(d / "packet/intent.md"),
    "screenshots": [ref(p) for p in sorted((d / "packet/screens").glob("*.txt"))],
    "proposal": ref(d / "designer-output.md"),
    "access_record": ref(d / "designer-session.md"),
    "review": ref(d / "review-output.md"),
    "amendments_applied": ref(d / "amendments-final.md"),
    "environment_answers": ref(d / "answers-final.md"),
    "frozen": ref(d / "FROZEN.sha256"),
    "named_limits": next(a["text"] for a in amend if a["case"] == "NEW LIMITS"),
    "questions": frozen["questions"],
    "cases": cases,
}
(root / "design.json").write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(sha(root / "design.json"), len(cases))

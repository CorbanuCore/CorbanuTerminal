"""Write design.json (RECORD_TEMPLATE shape) for the acc-tart1 record (from PF-60-S10's script).
Cases = the designer's frozen JSON (verbatim fields) plus the new case from the review's amendments; every
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
    return re.findall(r"FUP-\d\d", a["case"])


cases = []
for c in frozen["cases"]:
    row = dict(c)
    row["amendments"] = [a for a in amend if targets(a) == "ALL" or c["id"] in targets(a)]
    cases.append(row)
for a in amend:
    if a["case"].startswith("NEW FUP-"):
        t = a["text"]
        pri = t["priority"] if t["priority"] in ("blocker", "advisory") else "advisory"
        row = {"id": a["case"].split()[-1], "priority": pri, "priority_as_written": t["priority"], "group": t["group"],
               "starting_conditions": t["starting_conditions"], "actions": t["actions"], "expected": t["expected"],
               "added_by_amendment": a["n"],
               "amendments": [x for x in amend if targets(x) == "ALL"]}
        cases.append(row)
design = {
    "feature": "PF-60 follow-up live run (acc-tart1, D25/D3/D16 on Tart per D34): /review attribution, side: labels, reopen warm-up Request IDs, Moonshot cache read, realtime session",
    "designer": "01a156fc-532e-7ee0-a6c0-0096f4ca3b19",
    "design_reviewer": "01a15705-7322-7e62-8fba-775f047cca89",
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
    "named_limits": ["Pricing whatever the provider is named: only the provider name kimi exists (designer Q15; review).", "The realtime spend limit (D16, under $0.05) is checked from the relay log, not in the product (review).", "If the realtime conversation cannot be reopened, FUP-21 is blocked and FUP-22 gives partial evidence (review)."],
    "questions": frozen["questions"],
    "cases": cases,
}
(root / "design.json").write_text(json.dumps(design, indent=1, ensure_ascii=False) + "\n")
print(sha(root / "design.json"), len(cases))

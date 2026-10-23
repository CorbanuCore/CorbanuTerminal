"""Resolve the pending rows of amendments3-v3.json from the owner's answers and write amendments3-final.{json,md}.
usage: python3 tools/finalize_amendments.py RUN3_DIR ANSWER_A(A|B) ANSWER_B(A|B) GO_FILE_SHA256"""
import json, pathlib, sys
run3, a, b, go_sha = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3], sys.argv[4]
d = json.loads((run3 / "design3/amendments3-v3.json").read_text())
rows = []
for r in d["amendments"]:
    if r["case"] == "AGG-32":
        r = dict(r); r["decisions"] = [f"#447 (2026-10-13T17:10:35Z, point 2): owner answer (b) = Option {b} (go file qual-aggr3.go, SHA-256 {go_sha})"]; r.pop("pending", None)
        if b == "B":
            r["new"] = {"expected": [r["new"]["expected"][0].split(": ", 1)[1]]}
        else:
            r["no_change"] = True; r["new"] = {"expected": ["Frozen expectation stands (Option A)"]}
    elif r["case"] == "AGG-02, AGG-04, AGG-37":
        if a == "B":
            continue  # keep the restart-only list: no amendment
        raise SystemExit("answer (a) = A needs a new candidate with #662 and a reviewed amendment: stop and re-plan")
    rows.append(r)
d["amendments"] = rows
d["about"] = f"Third-run amendment table, final. Owner answers: (a) Option {a} (#505 2026-10-13T17:10:33Z, #447 17:10:35Z point 1: the restart-only list stays; no amendment for AGG-02/04/37), (b) Option {b} (#447 17:10:35Z point 2) (go file SHA-256 {go_sha}). Applies on top of the second run's effective design; unlisted cases run as in the second run."
d.pop("pending", None)
(run3 / "design3/amendments3-final.json").write_text(json.dumps(d, indent=1))
L = ["# Third-run amendments (final)", "", d["about"], "", "| Case | Old expectation (in force for the second run) | Decision | New expectation |", "| --- | --- | --- | --- |"]
def cell(x):
    out = []
    for k, lab in (("starting_conditions", "Start"), ("actions", "Actions"), ("expected", "Expected")):
        if k in x:
            v = x[k]; out.append(f"**{lab}:** " + (v if isinstance(v, str) else " / ".join(v)))
    return "<br>".join(out).replace("|", "\\|")
for r in rows:
    old = {k: r["old"][k] for k in r["new"] if k in r["old"]}
    L.append(f"| {r['case']} | ({r['old']['source']}) {cell(old)} | {'; '.join(r['decisions'])} | {cell(r['new'])} |")
L += ["", "Cases not listed run as in the second run (frozen case, with its 2026-10-11 amendment if any)."]
(run3 / "design3/amendments3-final.md").write_text("\n".join(L) + "\n")
print("final rows:", [r["case"] for r in rows])

"""Write dispositions.md: all 64 cases with runs 1-3, the first and second targeted runs, this third targeted run and
the combined status.

usage: python3 tools/make_dispositions6.py RECORD_DIR MILESTONES_DIR
"""
import json, pathlib, sys
from collections import Counter

root, ms = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
final = json.loads((root / "tools/final6.json").read_text())
res = lambda n: {c["id"]: c["disposition"] for c in json.loads((ms / n / "results.json").read_text())["cases"]}
r1, r2, r3 = res("aggressive-20261010"), res("aggressive-20261011-rerun"), res("aggressive-20261013-third")
t1, t2 = res("aggressive-20261014-targeted"), res("aggressive-20261014-targeted2")
rerun_t1 = set(json.loads((ms / "aggressive-20261014-targeted/tools/final4.json").read_text()))
rerun_t2 = set(json.loads((ms / "aggressive-20261014-targeted2/tools/final5.json").read_text()))
ids = [c["id"] for c in json.loads((root / "design.json").read_text())["cases"]]
label = lambda d: "accepted (owner, #447)" if d == "out_of_scope" else d
combined = {i: label(final[i]["disposition"] if i in final else t2[i]) for i in ids}
cr, cc = Counter(v["disposition"] for v in final.values()), Counter(combined.values())
executed = [k for k, v in final.items() if not v.get("judged_on_previous_run")]
judged = [k for k, v in final.items() if v.get("judged_on_previous_run")]
cx = Counter(final[k]["disposition"] for k in executed)
L = ["# Dispositions (third targeted re-run, 2026-10-15)", "",
     f"Re-run on this candidate: {len(executed)} cases ({', '.join(executed)}): {cx['passed']} passed, {cx['failed']} failed, {cx['blocked']} blocked. "
     f"Judged on the second targeted run's recorded runs against amendment 6, without re-running (owner decision): {', '.join(judged)}: "
     f"{Counter(final[k]['disposition'] for k in judged)['passed']} passed. "
     f"Combined status of all {len(ids)} cases (this run where re-run or re-judged, the second targeted run's record otherwise): "
     f"{cc['passed']} passed, {cc['failed']} failed, {cc['blocked']} blocked, {cc['accepted (owner, #447)']} accepted by the owner. "
     "Integrator dispositions from the executor reports; where they differ from the executor's report the summary says why. "
     "Runs 1-3 are `../aggressive-20261010/`, `../aggressive-20261011-rerun/` and `../aggressive-20261013-third/`; T1 and T2 are "
     "`../aggressive-20261014-targeted/` and `../aggressive-20261014-targeted2/` (\"new\" = case added later; values in parentheses "
     "were carried over there, not re-run).", "",
     "| Case | Run 1 | Run 2 | Run 3 | T1 | T2 | Executor (attempts) | This run | Combined | Summary |",
     "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"]
for i in ids:
    a = t1[i] if i in rerun_t1 else f"({t1[i]})"
    b = label(t2[i]) if i in rerun_t2 else f"({label(t2[i])})"
    head = f"| {i} | {r1.get(i, 'new')} | {r2.get(i, 'new')} | {r3[i]} | {a} | {b} |"
    if i in final:
        s = final[i]
        raw = ", ".join(f"{k}: {v}" for k, v in s["raw"].items()) or "-"
        L.append(f"{head} {raw} | **{label(s['disposition'])}** | {combined[i]} | {s['summary']} |")
    else:
        L.append(f"{head} - | - | {combined[i]} | Not re-run; the second targeted run's record stands. |")
(root / "dispositions.md").write_text("\n".join(L) + "\n")
print(cx, cc)

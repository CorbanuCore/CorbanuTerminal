"""Write dispositions.md: all 64 cases with runs 1-3, the first targeted run, this second targeted run and the combined
status.

usage: python3 tools/make_dispositions5.py RECORD_DIR FIRST SECOND THIRD TARGETED
"""
import json, pathlib, sys
from collections import Counter

root, first, second, third, tgt = (pathlib.Path(a) for a in sys.argv[1:6])
final = json.loads((root / "tools/final5.json").read_text())
res = lambda p: {c["id"]: c["disposition"] for c in json.loads((p / "results.json").read_text())["cases"]}
r1, r2, r3, r4 = res(first), res(second), res(third), res(tgt)
rerun4 = set(json.loads((tgt / "tools/final4.json").read_text()))
ids = [c["id"] for c in json.loads((root / "design.json").read_text())["cases"]]
label = lambda d: "accepted (owner, #447)" if d == "out_of_scope" else d
combined = {i: label(final[i]["disposition"] if i in final else r4[i]) for i in ids}
executed = {k: v for k, v in final.items() if v["disposition"] != "out_of_scope"}
cr, cc = Counter(v["disposition"] for v in executed.values()), Counter(combined.values())
L = ["# Dispositions (second targeted re-run, 2026-10-14/15)", "",
     f"Re-run on this candidate: {len(executed)} cases, {cr['passed']} passed, {cr['failed']} failed, {cr['blocked']} blocked. "
     f"Accepted by the owner without execution (#447, 2026-10-14T22:58:14Z): {len(final) - len(executed)} cases. "
     f"Combined status of all {len(ids)} cases (this run where re-run or accepted, the first targeted run's record otherwise): "
     f"{cc['passed']} passed, {cc['failed']} failed, {cc['blocked']} blocked, {cc['accepted (owner, #447)']} accepted by the owner. "
     "Integrator dispositions from the executor reports; where they differ from the executor's report the summary says why. "
     "Runs 1-3 are `../aggressive-20261010/`, `../aggressive-20261011-rerun/` and `../aggressive-20261013-third/`; T1 is "
     "`../aggressive-20261014-targeted/` (\"new\" = case added later; \"-\" = not run there; T1 values in parentheses were "
     "carried over from run 3).", "",
     "| Case | Run 1 | Run 2 | Run 3 | T1 | Executor (attempts) | This run | Combined | Summary |",
     "| --- | --- | --- | --- | --- | --- | --- | --- | --- |"]
for i in ids:
    t1 = r4[i] if i in rerun4 else f"({r4[i]})"
    head = f"| {i} | {r1.get(i, 'new')} | {r2.get(i, 'new')} | {r3[i]} | {t1} |"
    if i in final:
        s = final[i]
        raw = ", ".join(f"{k}: {v}" for k, v in s["raw"].items()) or "-"
        L.append(f"{head} {raw} | **{label(s['disposition'])}** | {combined[i]} | {s['summary']} |")
    else:
        L.append(f"{head} - | - | {combined[i]} | Not re-run; the first targeted run's record stands. |")
(root / "dispositions.md").write_text("\n".join(L) + "\n")
print(cr, cc)

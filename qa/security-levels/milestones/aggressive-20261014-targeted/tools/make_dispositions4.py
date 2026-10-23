"""Write dispositions.md: all 64 cases with runs 1-3, this targeted re-run and the combined status.

usage: python3 tools/make_dispositions4.py RECORD_DIR FIRST SECOND THIRD
"""
import json, pathlib, sys
from collections import Counter

root, first, second, third = (pathlib.Path(a) for a in sys.argv[1:5])
final = json.loads((root / "tools/final4.json").read_text())
res = lambda p: {c["id"]: c["disposition"] for c in json.loads((p / "results.json").read_text())["cases"]}
r1, r2, r3 = res(first), res(second), res(third)
ids = [c["id"] for c in json.loads((root / "design.json").read_text())["cases"]]
combined = {i: (final[i]["disposition"] if i in final else r3[i]) for i in ids}
cr, cc = Counter(v["disposition"] for v in final.values()), Counter(combined.values())
L = ["# Dispositions (targeted re-run, 2026-10-14)", "",
     f"Re-run on this candidate: {len(final)} cases, {cr['passed']} passed, {cr['failed']} failed, {cr['blocked']} blocked. "
     f"Combined status of all {len(ids)} cases (this run where re-run, the third run otherwise): {cc['passed']} passed, "
     f"{cc['failed']} failed, {cc['blocked']} blocked. Integrator dispositions from the executor reports; where they differ "
     "from the executor's verdict the summary says why. Runs 1-3 are `../aggressive-20261010/`, `../aggressive-20261011-rerun/` "
     "and `../aggressive-20261013-third/` (\"new\" = case added later; \"-\" = not re-run here).", "",
     "| Case | Run 1 | Run 2 | Run 3 | Executor (attempts) | This run | Combined | Summary |",
     "| --- | --- | --- | --- | --- | --- | --- | --- |"]
for i in ids:
    if i in final:
        s = final[i]
        raw = ", ".join(f"{k}: {v}" for k, v in s["raw"].items())
        L.append(f"| {i} | {r1.get(i, 'new')} | {r2.get(i, 'new')} | {r3[i]} | {raw} | **{s['disposition']}** | {combined[i]} | {s['summary']} |")
    else:
        L.append(f"| {i} | {r1.get(i, 'new')} | {r2.get(i, 'new')} | {r3[i]} | - | - | {combined[i]} | Not re-run; third-run result stands. |")
(root / "dispositions.md").write_text("\n".join(L) + "\n")
print(cr, cc)

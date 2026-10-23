"""Write dispositions.md: all 34 cases with the first run, this targeted run and the combined status.
usage: python3 tools/make_dispositions_t.py RECORD_DIR FIRST_RUN_RECORD_DIR"""
import json, pathlib, sys
from collections import Counter

root, prev = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
final = json.loads((root / "tools/final-t.json").read_text())
r1 = {c["id"]: c["disposition"] for c in json.loads((prev / "results.json").read_text())["cases"]}
ids = [c["id"] for c in json.loads((root / "design.json").read_text())["cases"]]
prio = {c["id"]: c.get("priority", "") for c in json.loads((root / "design.json").read_text())["cases"]}
label = lambda d: "pending owner decision (D5)" if d == "pending_owner_decision" else d
combined = {i: final[i]["disposition"] if i in final else ("pending_owner_decision" if i == "MOD-11" else r1[i]) for i in ids}
cx, cc = Counter(v["disposition"] for v in final.values()), Counter(combined.values())
blk = Counter(combined[i] for i in ids if prio[i] == "blocker"); adv = Counter(combined[i] for i in ids if prio[i] == "advisory")
L = ["# Dispositions (Moderate targeted re-run, 2026-10-16)", "",
     f"Re-run on this candidate under amendment v3: {len(final)} cases: {cx['passed']} passed, {cx['failed']} failed, {cx['blocked']} blocked. "
     f"Combined status of all {len(ids)} cases (this run where re-run, the first run's record otherwise): {cc['passed']} passed, "
     f"{cc['failed']} failed, {cc['blocked']} blocked, {cc['pending_owner_decision']} pending owner decision (MOD-11; MOD-06's git-commit part is also pending). "
     f"Blockers: {blk['passed']} passed, {blk['failed']} failed, {blk['blocked']} blocked, {blk['pending_owner_decision']} pending of {sum(blk.values())}; "
     f"advisory: {adv['passed']} passed, {adv['failed']} failed, {adv['blocked']} blocked of {sum(adv.values())}. "
     "Integrator dispositions from the executor reports, screens, product logs and relay log; where they differ from the executor's verdict the summary says why. "
     "Owner model-decline rule (#447, G-1) as in the first run. Run 1 is `../moderate-20261014/`.", "",
     "| Case | Priority | Run 1 | Executor (this run) | This run | Combined | Summary |", "| --- | --- | --- | --- | --- | --- | --- |"]
for i in ids:
    if i in final:
        s = final[i]; raw = ", ".join(f"{k}: {v}" for k, v in s["raw"].items())
        L.append(f"| {i} | {prio[i]} | {r1[i]} | {raw} | **{s['disposition']}** | {s['disposition']} | {s['summary']} |")
    elif i == "MOD-11":
        L.append(f"| {i} | {prio[i]} | {r1[i]} | - | not re-run | {label(combined[i])} | Not re-run: pending owner decision D5 (#837), per the brief; not counted as failed. |")
    else:
        L.append(f"| {i} | {prio[i]} | {r1[i]} | - | not re-run | {combined[i]} | Not re-run; the first run's result stands. |")
(root / "dispositions.md").write_text("\n".join(L) + "\n")
print(cx, cc, blk, adv)

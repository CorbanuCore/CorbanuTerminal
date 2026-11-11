"""Write dispositions.md: all 34 Moderate cases after the second targeted re-run and the owner's decisions of 2026-10-19.
usage: python3 tools/make_dispositions_t2.py RECORD_DIR PREVIOUS_TARGETED_RECORD_DIR"""
import json, pathlib, sys
from collections import Counter

root, prev = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
final = json.loads((root / "tools/final-t2.json").read_text())
owner = json.loads((root / "tools/owner-dispositions.json").read_text())
before = {c["id"]: c["disposition"] for c in json.loads((prev / "results.json").read_text())["cases"]}
design = json.loads((root / "design.json").read_text())
ids = [c["id"] for c in design["cases"]]
prio = {c["id"]: c["priority"] for c in design["cases"]}
LABEL = {"accepted_owner_decision": "accepted (owner decision)", "pending_owner_decision": "pending owner decision (D5)"}
lab = lambda d: LABEL.get(d, d)
combined = {i: final[i]["disposition"] if i in final else owner[i]["disposition"] if i in owner else before[i] for i in ids}
cc = Counter(combined.values())
blk = Counter(combined[i] for i in ids if prio[i] == "blocker"); adv = Counter(combined[i] for i in ids if prio[i] == "advisory")
fmt = lambda c: ", ".join(f"{n} {lab(k)}" for k, n in sorted(c.items(), key=lambda kv: (kv[0] != "passed", kv[0])))
L = ["# Dispositions (Moderate second targeted re-run, 2026-10-20)", "",
     f"Re-run on this candidate under amendment v4: MOD-15 **{final['MOD-15']['disposition']}**, MOD-28 **{lab(final['MOD-28']['disposition'])}** (D40). "
     "Owner decisions of 2026-10-19 applied without a re-run: D39 (MOD-09, MOD-21 accepted as known limits), D41 (MOD-33 counts passed), "
     "D43 (MOD-11 and MOD-06's git-commit part accepted limits under D5(c)).", "",
     f"**All {len(ids)} cases:** {fmt(cc)}; 0 failed, 0 blocked, 0 pending. Blockers ({sum(blk.values())}): {fmt(blk)}. "
     f"Advisory ({sum(adv.values())}): {fmt(adv)}.", "",
     "\"Accepted (owner decision)\" means the owner accepted the case's outcome as a known limit or (MOD-28) as \"no session approval exists to "
     "revoke\"; it is not counted as passed. Previous record: `../moderate-targeted-20261016/` (its combined column), first run `../moderate-20261014/`.", "",
     "| Case | Priority | Before (2026-10-16 combined) | This run / decision | Combined | Summary |", "| --- | --- | --- | --- | --- | --- |"]
for i in ids:
    if i in final:
        s = final[i]
        L.append(f"| {i} | {prio[i]} | {lab(before[i])} | re-run: **{lab(s['disposition'])}** | {lab(combined[i])} | {s['summary']} |")
    elif i in owner:
        L.append(f"| {i} | {prio[i]} | {lab(before[i])} | {owner[i]['decision']} | {lab(combined[i])} | {owner[i]['summary']} |")
    else:
        L.append(f"| {i} | {prio[i]} | {lab(before[i])} | - | {lab(combined[i])} | Not re-run; the previous record's result stands. |")
(root / "dispositions.md").write_text("\n".join(L) + "\n")
print(cc, blk, adv)

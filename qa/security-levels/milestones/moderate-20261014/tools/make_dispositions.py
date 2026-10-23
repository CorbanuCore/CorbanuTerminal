"""Write dispositions.md from tools/final.json and design.json priorities.
usage: python3 tools/make_dispositions.py RECORD_DIR"""
import json, pathlib, sys
from collections import Counter
root = pathlib.Path(sys.argv[1])
final = json.loads((root / "tools/final.json").read_text())
prio = {c["id"]: c["priority"] for c in json.loads((root / "design.json").read_text())["cases"]}
n = Counter(v["disposition"] for v in final.values())
nb = Counter(v["disposition"] for k, v in final.items() if prio[k] == "blocker")
L = ["# Dispositions (Moderate, first run)", "",
     f"{len(final)} cases: {n['passed']} passed, {n['failed']} failed, {n['blocked']} blocked "
     f"(blockers: {nb['passed']} passed, {nb['failed']} failed, {nb['blocked']} blocked of {sum(nb.values())}). "
     "Integrator dispositions from the executor reports, screens and fixture records; where they differ from the executor's "
     "verdict the summary says why. Owner model-decline rule (#447): a case whose step the product's model declined passes "
     "only if the product's text still meets the expected wording, and is blocked otherwise. A case is failed when any "
     "expected result was observed to fail.", "",
     "| Case | Priority | Executor (attempts) | Disposition | Summary |", "| --- | --- | --- | --- | --- |"]
for c, v in final.items():
    raw = ", ".join(f"{k}: {x}" for k, x in v["raw"].items())
    L.append(f"| {c} | {prio[c]} | {raw} | **{v['disposition']}** | {v['summary']} |")
(root / "dispositions.md").write_text("\n".join(L) + "\n")
print(n, nb)

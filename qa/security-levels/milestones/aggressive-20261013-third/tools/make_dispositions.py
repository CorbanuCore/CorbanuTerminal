"""Write dispositions.md from tools/final.json, with the first and second runs' dispositions per case.

usage: python3 tools/make_dispositions.py RECORD_DIR FIRST_RECORD_DIR SECOND_RECORD_DIR
"""
import json, pathlib, sys
from collections import Counter

root, first, second = (pathlib.Path(a) for a in sys.argv[1:4])
final = json.loads((root / "tools/final.json").read_text())
old1 = {c["id"]: c["disposition"] for c in json.loads((first / "results.json").read_text())["cases"]}
old2 = {c["id"]: c["disposition"] for c in json.loads((second / "results.json").read_text())["cases"]}
count = Counter(v["disposition"] for v in final.values())
lines = [
    "# Dispositions (third run)", "",
    f"{len(final)} cases: {count['passed']} passed, {count['failed']} failed, {count['blocked']} blocked. "
    "Integrator dispositions from the executor reports; where they differ from the executor's verdict the row says why. "
    "\"Run 1\" and \"Run 2\" are the dispositions in `../aggressive-20261010/results.json` and "
    "`../aggressive-20261011-rerun/results.json` (\"new\" = case added later).", "",
    "| Case | Run 1 | Run 2 | Executor (attempts) | Third-run disposition | Summary |",
    "| --- | --- | --- | --- | --- | --- |",
]
for case, spec in final.items():
    raw = ", ".join(f"{k}: {v}" for k, v in spec["raw"].items()) or "not executed"
    lines.append(f"| {case} | {old1.get(case, 'new')} | {old2.get(case, 'new')} | {raw} | **{spec['disposition']}** | {spec['summary']} |")
(root / "dispositions.md").write_text("\n".join(lines) + "\n")
print(count)

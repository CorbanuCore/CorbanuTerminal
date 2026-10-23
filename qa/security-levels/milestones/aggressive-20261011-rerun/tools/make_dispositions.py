"""Write dispositions.md from tools/final.json, with the first run's disposition per case.

usage: python3 tools/make_dispositions.py RECORD_DIR FIRST_RECORD_DIR
"""
import json
import pathlib
import sys
from collections import Counter

root, first = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
final = json.loads((root / "tools/final.json").read_text())
old = {c["id"]: c["disposition"] for c in json.loads((first / "results.json").read_text())["cases"]}
count = Counter(v["disposition"] for v in final.values())
lines = [
    "# Dispositions (re-run)",
    "",
    f"{len(final)} cases: {count['passed']} passed, {count['failed']} failed, {count['blocked']} blocked. "
    "Integrator dispositions from the executor reports; where they differ from the executor's verdict the row says why. "
    "\"First run\" is the disposition in `../aggressive-20261010/results.json` (\"new\" = case added for the 2026-10-11 decisions).",
    "",
    "| Case | First run | Executor (attempts) | Re-run disposition | Summary |",
    "| --- | --- | --- | --- | --- |",
]
for case, spec in final.items():
    raw = ", ".join(f"{k}: {v}" for k, v in spec["raw"].items()) or "not executed"
    lines.append(f"| {case} | {old.get(case, 'new')} | {raw} | **{spec['disposition']}** | {spec['summary']} |")
(root / "dispositions.md").write_text("\n".join(lines) + "\n")
print(count)

"""Measure syntax checks, including file/block lists and exact command results."""
import ast
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
GUARDS = HERE.parent / "pf83-failclosed-73"
files = sorted(list(HERE.glob("*.py")) + list(GUARDS.glob("*.py")))
records = []
for path in files:
    ast.parse(path.read_text(), filename=str(path))
    records.append(dict(path=str(path.relative_to(HERE.parent)), status="passed"))
blocks = re.findall(r"^```bash\n(.*?)^```$", (GUARDS / "runbook.md").read_text(),
                    flags=re.MULTILINE | re.DOTALL)
shell = []
for index, block in enumerate(blocks, start=1):
    result = subprocess.run(["/bin/bash", "-n"], input=block, text=True, capture_output=True)
    shell.append(dict(block=index, exit_code=result.returncode, stderr=result.stderr))
report = dict(python_syntax_files=len(records), python_checks=records,
              bash_syntax_blocks=len(shell), bash_checks=shell)
print(json.dumps(report, indent=2, sort_keys=True))
raise SystemExit(any(row["exit_code"] for row in shell))

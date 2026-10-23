"""Check only the requested witness binding and refusal-table corrections."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

here = Path(__file__).resolve().parent
witness = here.parent / "acct-final-100/test_destination.py"
raw = witness.read_text()
env = dict(PATH=os.environ["PATH"], GIT_CONFIG_GLOBAL="/dev/null",
           GIT_CONFIG_NOSYSTEM="1", GIT_TERMINAL_PROMPT="0",
           PYTHONDONTWRITEBYTECODE="1")
runs = []
for name, transform, expected in [
    ("normal", "", 0),
    ("reordered", 'raw = raw.replace("for name, destination, expected in cases:", "for name, destination, expected in reversed(cases):")', 0),
    ("renamed", """raw = raw.replace('("alias-unignored-regression", alias', '("renamed-case", alias')""", 1),
]:
    for optimized in (False, True):
        label = name + ("-optimized" if optimized else "")
        receipt = here / (label + ".json")
        launcher = (
            "import sys\nfrom pathlib import Path\n"
            "path = Path(sys.argv[1]); raw = path.read_text()\n"
            + transform + "\n"
            "sys.argv = [str(path), '--output', sys.argv[2]]\n"
            "exec(compile(raw, str(path), 'exec'), {'__file__': str(path), '__name__': '__main__'})\n"
        )
        result = subprocess.run(
            [sys.executable, "-B", *(["-O"] if optimized else []), "-c",
             launcher, str(witness), str(receipt)], env=env, capture_output=True, text=True)
        (here / (label + ".stdout.txt")).write_text(result.stdout)
        (here / (label + ".stderr.txt")).write_text(result.stderr)
        if result.returncode != expected:
            raise RuntimeError(f"{label}: unexpected exit {result.returncode}")
        if expected:
            if receipt.exists() or "missing unique pinned alias-unignored-regression observation" not in result.stderr or "NameError" in result.stderr:
                raise RuntimeError(f"{label}: missing-observation failure was not clean")
        else:
            summary = json.loads(result.stdout.splitlines()[-1])
            if (summary["passed"], summary["old_alias_exit"], summary["new_alias_exit"]) != (7, 0, 1):
                raise RuntimeError(f"{label}: observations differ")
        runs.append(dict(case=label, exit=result.returncode, receipt_written=receipt.exists()))
note = (here.parent / "acct-derive-95/OPERATOR.md").read_text()
block = re.findall(r"^```sh\n(.*?)^```$", note, re.M | re.S)[0]
guard = block[block.index("audit=${CORBANU_AUDIT_DIR"):block.index("git clone --no-local")]
scratch = here / "target/refusal"
scratch.mkdir(parents=True)
for name, target, message in [
    ("existing-symlink", scratch, "destination already exists"),
    ("dangling-symlink", scratch / "missing", "destination is a symlink"),
]:
    link = scratch / name
    link.symlink_to(target)
    result = subprocess.run(
        ["bash", "--noprofile", "--norc", "-c", "set -eu\nq=unused\n" + guard],
        env=dict(env, CORBANU_AUDIT_DIR=str(link)), capture_output=True, text=True)
    if result.returncode != 1 or not result.stdout.startswith("STOP: " + message):
        raise RuntimeError(f"{name}: refusal disagrees with table")
    runs.append(dict(case=name, exit=result.returncode, stdout=result.stdout, stderr=result.stderr))
record = dict(witness_sha256=hashlib.sha256(raw.encode()).hexdigest(), runs=runs)
(here / "checks.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record))

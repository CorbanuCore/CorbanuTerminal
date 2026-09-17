"""Repeat the newline control in a disposable checkout; preserve initial attempt."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = here.parent.relative_to(repo)
clone = here / "target/before-checkout"
environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull, GIT_TERMINAL_PROMPT="0")

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

names = ("acct-acceptance-75/acceptance.md", "acct-acceptance-75/scope.json",
         "acct-inventory-79/inventory-correction.json", "acct-inventory-79/verify_acceptance.py")
for name in names:
    shutil.copyfile(repo / scope / name, clone / scope / name)
acceptance = clone / scope / names[0]
original = acceptance.read_bytes()
runs = []
try:
    acceptance.write_bytes(original + b"\n")
    for name, flags in (("drift-disposable", []), ("drift-disposable-optimized", ["-O"])):
        command = [sys.executable, "-B", *flags, str(scope / names[-1])]
        result = subprocess.run(command, cwd=clone, env=environment, capture_output=True)
        for suffix, raw in (("stdout", result.stdout), ("stderr", result.stderr)):
            with (here / (name + "." + suffix + ".txt")).open("xb") as stream:
                stream.write(raw)
        runs.append(dict(name=name, command=command, exit=result.returncode,
                         stderr_empty=not result.stderr,
                         stdout_sha256=hashlib.sha256(result.stdout).hexdigest()))
        require(result.returncode == 3 and not result.stderr, "drift exit/stderr")
        require(result.stdout == (here / "drift.stdout.txt").read_bytes(), "prior simulation differs")
        require(result.stdout in re.findall(rb"```text\n(.*?)```", original, re.S),
                "documented transcript differs")
finally:
    acceptance.write_bytes(original)
require(acceptance.read_bytes() == original, "restore failed")
previous = here / "simulation.json"
with (here / "simulation-working-tree.json").open("xb") as stream:
    stream.write(previous.read_bytes())
record = dict(source_commit="6a7128ccc4e10c92dd583ca65b62e70d84483012",
    kind="Disposable local checkout with proposed verifier/inventory/document bytes",
    cwd=str(clone), mutation="Append one newline to acceptance.md; run; restore exact original.",
    inputs_sha256={name: hashlib.sha256((clone / scope / name).read_bytes()).hexdigest() for name in names},
    runs=runs, restored=True,
    initial_attempt="simulation-working-tree.json",
    prior_attempt_note="Initial working-tree control is retained separately; this replay proves the documented disposable-checkout workflow.")
previous.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record, indent=2))

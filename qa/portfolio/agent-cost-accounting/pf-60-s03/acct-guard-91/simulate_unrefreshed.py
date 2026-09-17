"""Replay proposed QA contents in a disposable sparse clone, then edit without refresh.
Only local Git objects and allocation files are used; no packages are launched.
Raw stdout/stderr are retained for both normal and optimized Python.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = here.parent.relative_to(repo)
environment = dict(os.environ, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                   GIT_TERMINAL_PROMPT="0")


def require(condition, detail):
    if not condition:
        raise RuntimeError(detail)


def command(args, cwd=repo):
    return subprocess.run(args, cwd=cwd, env=environment, capture_output=True,
                          text=True, check=True)


def main():
    (here / "target").mkdir(exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix="unrefreshed-", dir=here / "target"))
    clone = run / "checkout"
    command(["git", "clone", "--shared", "--no-checkout", "--quiet", str(repo), str(clone)])
    command(["git", "sparse-checkout", "set", str(scope)], clone)
    command(["git", "checkout", "--quiet", "HEAD"], clone)
    changed = command(["git", "diff", "--name-only", "HEAD", "--", str(scope)]).stdout.splitlines()
    untracked = command(["git", "ls-files", "--others", "--exclude-standard",
                         "--", str(scope)]).stdout.splitlines()
    for name in sorted(set(changed + untracked)):
        source, destination = repo / name, clone / name
        if source.is_file():
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, destination)
        elif destination.exists():
            destination.unlink()
    command(["git", "add", "--", str(scope)], clone)
    require(not (clone / scope / "acct-fitness-76/package").exists(), "package unexpectedly present")
    verifier = str(scope / "acct-inventory-79/verify_acceptance.py")
    cases = []

    def replay(label, optimized, expected_exit, fragment):
        args = [sys.executable, "-B", *(["-O"] if optimized else []), verifier]
        result = subprocess.run(args, cwd=clone, env=environment, capture_output=True)
        (run / (label + ".stdout.txt")).write_bytes(result.stdout)
        (run / (label + ".stderr.txt")).write_bytes(result.stderr)
        require(result.returncode == expected_exit and not result.stderr
                and fragment.encode() in result.stdout, label + ": unexpected verifier outcome")
        cases.append(dict(case=label, command=args, exit=result.returncode,
                          stderr_empty=not result.stderr,
                          stdout_sha256=hashlib.sha256(result.stdout).hexdigest()))
        return result.stdout

    baseline = replay("baseline", False, 2, "BASELINE MATCH:")
    optimized = replay("baseline-optimized", True, 2, "BASELINE MATCH:")
    require(baseline == optimized, "optimized baseline differs")
    acceptance = clone / scope / "acct-acceptance-75/acceptance.md"
    inventory = clone / scope / "acct-acceptance-75/scope.json"
    before = acceptance.read_bytes()
    inventory_before = inventory.read_bytes()
    try:
        # A later round's whitespace-only evidence edit; semantic claims stay unchanged.
        acceptance.write_bytes(before + b"\n")
        mutated = replay("unrefreshed-edit", False, 3,
                         "DISAGREE acct-acceptance-75 inventory:")
        optimized = replay("unrefreshed-edit-optimized", True, 3, "BASELINE DRIFT:")
        require(mutated == optimized, "optimized mutation output differs")
        require(inventory.read_bytes() == inventory_before, "inventory unexpectedly refreshed")
    finally:
        acceptance.write_bytes(before)
    restored = replay("restored", False, 2, "BASELINE MATCH:")
    require(restored == baseline, "restoration changed baseline")
    receipt = dict(source_base=command(["git", "rev-parse", "HEAD"]).stdout.strip(),
                   run_directory=str(run), mutation="append one newline to acceptance.md",
                   inventory_unchanged=True, restored=True, no_network=True,
                   no_package_launch=True, cases=cases)
    (run / "simulation.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()

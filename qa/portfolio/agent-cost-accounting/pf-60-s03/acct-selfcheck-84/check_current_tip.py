"""Replay all existing acceptance controls from the current, untouched local tip."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = here.parent.relative_to(repo)
environment = dict(os.environ, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                   GIT_TERMINAL_PROMPT="0")
def git(*args, cwd=repo):
    return subprocess.check_output(["git", *args], cwd=cwd, env=environment, text=True)

commit = git("rev-parse", "HEAD").strip()
(here / "target").mkdir(exist_ok=True)
run = Path(tempfile.mkdtemp(prefix="current-tip-", dir=here / "target"))
clone = run / "checkout"
git("clone", "--shared", "--no-checkout", "--quiet", str(repo), str(clone))
git("checkout", "--quiet", "--detach", commit, cwd=clone)
before = git("status", "--porcelain", "--untracked-files=all", cwd=clone)
assert before == ""
assert git("rev-parse", "HEAD", cwd=clone).strip() == commit
assert not (clone / scope / "acct-fitness-76/package").exists()
receipts = []
for name, script, expected in (
    ("acceptance", "acct-inventory-79/verify_acceptance.py", 2),
    ("controls", "acct-inventory-79/check_verifier.py", 0),
):
    command = [sys.executable, "-B", str(scope / script)]
    result = subprocess.run(command, cwd=clone, env=environment, capture_output=True)
    for suffix, raw in (("stdout", result.stdout), ("stderr", result.stderr)):
        (run / (name + "." + suffix + ".txt")).write_bytes(raw)
    receipts.append(dict(name=name, command=command, exit=result.returncode,
                         expected_exit=expected,
                         stdout_sha256=hashlib.sha256(result.stdout).hexdigest(),
                         stderr_sha256=hashlib.sha256(result.stderr).hexdigest()))
    assert result.returncode == expected and not result.stderr, receipts[-1]
after = git("status", "--porcelain", "--untracked-files=all", cwd=clone)
assert after == ""
reference = repo / scope / "acct-baseline-81/replay-1/clean-checkout.stdout.txt"
actual = (run / "acceptance.stdout.txt").read_bytes()
receipt = dict(commit=commit, source_tip_after=git("rev-parse", "HEAD").strip(),
               clone=str(clone), run_directory=str(run), status_before=before,
               status_after=after, full_checkout=True, copied_working_tree_files=False,
               package_absent=True, no_network=True, no_package_launch=True,
               reference=str(reference.relative_to(repo)),
               reference_sha256=hashlib.sha256(reference.read_bytes()).hexdigest(),
               stdout_matches_prior_replay=actual == reference.read_bytes(),
               commands=receipts)
(run / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps(receipt, indent=2))

"""Verify the received integration commit from an untouched local Git clone."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

here = Path(__file__).resolve().parent
repo = here.parents[4]
commit = "ff3f74f6ae66efbf3dfcc4f6291af2ce97f0f6cc"
clone = here / "target/integration-01"
assert not clone.exists(), "Preserve old attempts; select a fresh clone path."
env = dict(os.environ, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
           GIT_TERMINAL_PROMPT="0")
def git(*args, cwd=repo):
    return subprocess.check_output(["git", *args], cwd=cwd, env=env, text=True)

clone.parent.mkdir(parents=True, exist_ok=True)
git("clone", "--shared", "--no-checkout", "--quiet", str(repo), str(clone))
git("checkout", "--quiet", "--detach", commit, cwd=clone)
before = git("status", "--porcelain", "--untracked-files=all", cwd=clone)
assert before == ""
assert git("rev-parse", "HEAD", cwd=clone).strip() == commit
package = clone / here.parent.relative_to(repo) / "acct-fitness-76/package"
assert not package.exists()
command = ["python3", "-B", str(here.parent.relative_to(repo) /
                               "acct-inventory-79/verify_acceptance.py")]
result = subprocess.run(command, cwd=clone, env=env, capture_output=True)
for name, raw in (("stdout", result.stdout), ("stderr", result.stderr)):
    (here / ("clean-integration." + name + ".txt")).write_bytes(raw)
after = git("status", "--porcelain", "--untracked-files=all", cwd=clone)
assert after == ""
receipt = dict(commit=commit, clone=str(clone), command=command, exit=result.returncode,
               status_before=before, status_after=after, package_absent=True,
               full_checkout=True, copied_working_tree_files=False, no_network=True,
               stdout_sha256=hashlib.sha256(result.stdout).hexdigest(),
               stderr_sha256=hashlib.sha256(result.stderr).hexdigest())
(here / "clean-integration.json").write_text(json.dumps(receipt, indent=2) + "\n")
sys.stdout.buffer.write(result.stdout)
sys.stderr.buffer.write(result.stderr)
print("Recorded process exit:", result.returncode)

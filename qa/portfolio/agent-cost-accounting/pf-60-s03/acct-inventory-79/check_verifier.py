"""Exercise a clean sparse checkout plus wrong-claim, corruption and absence controls.
All generated checkout/state remains under this allocation's ignored target/.
Uses only local Git objects and synthetic Git identity; no network or credentials.
"""
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = here.parent.relative_to(repo)
clone = here / "target/checkout-01"
assert not clone.exists(), "preserve prior attempts; choose a fresh checkout name"
environment = dict(os.environ, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                   GIT_TERMINAL_PROMPT="0")
def command(args, cwd=repo):
    return subprocess.run(args, cwd=cwd, env=environment, capture_output=True, text=True, check=True)

command(["git", "clone", "--shared", "--no-checkout", "--quiet", str(repo), str(clone)])
command(["git", "sparse-checkout", "set", str(scope)], clone)
command(["git", "checkout", "--quiet", "HEAD"], clone)
# Copy only allocation changes; do not copy ignored target/package/profile state.
changed = command(["git", "diff", "--name-only", "HEAD", "--", str(scope)]).stdout.splitlines()
untracked = command(["git", "ls-files", "--others", "--exclude-standard", "--", str(scope)]).stdout.splitlines()
for name in sorted(set(changed + untracked)):
    source, destination = repo / name, clone / name
    if source.is_file():
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    elif destination.exists():
        destination.unlink()
command(["git", "add", "--", str(scope)], clone)
command(["git", "-c", "user.name=Evidence Check", "-c", "user.email=evidence.invalid@example.invalid",
         "-c", "commit.gpgsign=false", "commit", "--quiet", "--no-verify",
         "-m", "Temporary allocation evidence snapshot; not an integration commit"], clone)
assert command(["git", "status", "--porcelain"], clone).stdout == ""
assert not (clone / scope / "acct-fitness-76/package").exists()
verifier = str(here.relative_to(repo) / "verify_acceptance.py")
results = []

def run(name, args, expected, fragment):
    result = subprocess.run([sys.executable, "-B", *args], cwd=clone, env=environment,
                            capture_output=True, text=True)
    (here / (name + ".stdout.txt")).write_text(result.stdout)
    (here / (name + ".stderr.txt")).write_text(result.stderr)
    passed = result.returncode == expected and fragment in result.stdout and not result.stderr
    results.append(dict(case=name, exit=result.returncode, expected_exit=expected, passed=passed,
                        stdout_sha256=hashlib.sha256(result.stdout.encode()).hexdigest()))
    assert passed, (name, result.returncode, result.stdout, result.stderr)

run("clean-checkout", [verifier], 2, "RESULT agreement=20 disagreement=0 unavailable=3 exit=2")
run("clean-finalizer", [str(scope / "acct-fitness-76/finalize.py")], 2,
    "RESULT agreement=20 disagreement=0 unavailable=6 exit=2")
acceptance = clone / scope / "acct-acceptance-75/acceptance.md"
original = acceptance.read_bytes()
try:
    acceptance.write_bytes(original.replace(b"maps 147 named checks", b"maps 148 named checks"))
    run("wrong-claim", [verifier], 1, "DISAGREE mutation coverage: mutation/check counts differ")
finally:
    acceptance.write_bytes(original)
viewport = clone / scope / "acct-readers-61/reader-run-05/narrow.txt.gz"
original = viewport.read_bytes()
try:
    viewport.write_bytes(gzip.compress(gzip.decompress(original) + b"changed", mtime=0))
    run("corrupt-viewport", [verifier], 1, "DISAGREE capture bindings: narrow.txt.gz viewport digest")
finally:
    viewport.write_bytes(original)
log = clone / scope / "acct-fitness-76/gates-01/core-default.log.gz"
original = log.read_bytes()
try:
    log.unlink()
    run("missing-log", [verifier], 2, "UNAVAILABLE round-76 core-default:")
finally:
    log.write_bytes(original)
assert command(["git", "status", "--porcelain"], clone).stdout == ""
receipt = dict(
    kind="clean sparse Git checkout of proposed QA snapshot, then independent corruption controls",
    source_base=command(["git", "rev-parse", "HEAD"]).stdout.strip(),
    snapshot=command(["git", "rev-parse", "HEAD"], clone).stdout.strip(),
    ignored_package_absent=True, clean_before_and_after=True,
    no_network=True, no_package_launch=True, cases=results,
)
(here / "verifier-checks.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps(receipt, indent=2))

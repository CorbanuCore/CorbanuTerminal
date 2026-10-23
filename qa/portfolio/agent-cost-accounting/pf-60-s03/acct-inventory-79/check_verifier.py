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
import tempfile

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = here.parent.relative_to(repo)
(here / "target").mkdir(exist_ok=True)
run_directory = Path(tempfile.mkdtemp(prefix="controls-", dir=here / "target"))
clone = run_directory / "checkout"
print("Run directory:", run_directory, flush=True)
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
         "-c", "commit.gpgsign=false", "commit", "--allow-empty", "--quiet", "--no-verify",
         "-m", "Temporary allocation evidence snapshot; not an integration commit"], clone)
assert command(["git", "status", "--porcelain"], clone).stdout == ""
assert not (clone / scope / "acct-fitness-76/package").exists()
verifier = str(here.relative_to(repo) / "verify_acceptance.py")
results = []

def run(name, args, expected, fragment):
    result = subprocess.run([sys.executable, "-B", *args], cwd=clone, env=environment,
                            capture_output=True, text=True)
    (run_directory / (name + ".stdout.txt")).write_text(result.stdout)
    (run_directory / (name + ".stderr.txt")).write_text(result.stderr)
    passed = result.returncode == expected and fragment in result.stdout and not result.stderr
    if "--local-package" in args or name == "clean-finalizer":
        passed = passed and "BASELINE REFUSED:" in result.stdout
        passed = passed and not any(line.startswith("BASELINE MATCH:")
                                    for line in result.stdout.splitlines())
    results.append(dict(case=name, exit=result.returncode, expected_exit=expected, passed=passed,
                        stdout_sha256=hashlib.sha256(result.stdout.encode()).hexdigest()))
    assert passed, (name, result.returncode, result.stdout, result.stderr)

run("clean-checkout", [verifier], 2, "RESULT agreement=20 disagreement=0 unavailable=3 exit=2")
run("clean-finalizer", [str(scope / "acct-fitness-76/finalize.py")], 3,
    "RESULT agreement=20 disagreement=0 unavailable=6 exit=3")
run("local-package-absent-refusal", [verifier, "--local-package"], 3,
    "BASELINE REFUSED: --local-package has no documented combined baseline")
acceptance = clone / scope / "acct-acceptance-75/acceptance.md"
original = acceptance.read_bytes()
try:
    acceptance.write_bytes(original.replace(b"maps 147 named checks", b"maps 148 named checks"))
    run("wrong-claim", [verifier], 3, "DISAGREE mutation coverage: mutation/check counts differ")
finally:
    acceptance.write_bytes(original)
viewport = clone / scope / "acct-readers-61/reader-run-05/narrow.txt.gz"
original = viewport.read_bytes()
try:
    viewport.write_bytes(gzip.compress(gzip.decompress(original) + b"changed", mtime=0))
    run("corrupt-viewport", [verifier], 3, "DISAGREE capture bindings: narrow.txt.gz viewport digest")
finally:
    viewport.write_bytes(original)
log = clone / scope / "acct-fitness-76/gates-01/core-default.log.gz"
original = log.read_bytes()
try:
    log.unlink()
    run("missing-log", [verifier], 3, "RESULT agreement=18 disagreement=0 unavailable=5 exit=3")
finally:
    log.write_bytes(original)
# Refresh the inventory after document mutations to isolate baseline comparison
# from the independent acceptance-file digest check.
inventory = clone / scope / "acct-acceptance-75/scope.json"
correction = clone / scope / "acct-inventory-79/inventory-correction.json"
originals = {path: path.read_bytes() for path in (acceptance, inventory, correction)}
for name, old, new, fragment in (
    ("baseline-count", b"agreement=20", b"agreement=21", "actual counts/exit=(20, 0, 3, 2)"),
    ("baseline-identity", b"`committed package codex`",
     b"`committed package other`", "actual counts/exit=(20, 0, 3, 2)"),
    ("baseline-exit", b"unavailable=3 exit=2", b"unavailable=3 exit=0",
     "invalid baseline evidence exit"),
    ("baseline-missing", b"The expected retained-evidence baseline is:",
     b"The former retained-evidence baseline was:", "acceptance claim missing or ambiguous"),
    ("baseline-duplicate", b"`committed package codex-code-mode-host`",
     b"`committed package codex`", "invalid baseline unavailable list"),
):
    try:
        assert old in originals[acceptance]
        acceptance.write_bytes(originals[acceptance].replace(old, new))
        command([sys.executable, "-B", str(scope / "acct-selfcheck-84/refresh_inventory.py")], clone)
        run(name, [verifier], 3, "BASELINE DRIFT: " + fragment if name in
            ("baseline-exit", "baseline-duplicate") else fragment)
    finally:
        for path, raw in originals.items():
            path.write_bytes(raw)
# Optional local artifacts do not alter the retained-evidence baseline.
package = clone / scope / "acct-fitness-76/package"
package.mkdir()
try:
    (package / "codex").write_bytes(b"synthetic wrong-size package; never launched")
    run("local-package-disagreement", [verifier, "--local-package"], 3,
        "RESULT agreement=20 disagreement=1 unavailable=5 exit=3")
finally:
    shutil.rmtree(package)
# Even exact local matches are diagnostic: use inert synthetic bytes, never binaries.
manifest_path = clone / scope / "acct-fitness-76/package-manifest.json"
original = manifest_path.read_bytes()
package.mkdir()
try:
    manifest = json.loads(original)
    for row in manifest["files"]:
        path = clone / row["path"]
        raw = b"synthetic package; never launched"
        path.write_bytes(raw)
        path.chmod(0o555)
        row.update(bytes=len(raw), mode="0555", sha256=hashlib.sha256(raw).hexdigest())
    manifest_path.write_text(json.dumps(manifest))
    run("local-package-agreement-refusal", [verifier, "--local-package"], 3,
        "RESULT agreement=23 disagreement=0 unavailable=3 exit=3")
finally:
    manifest_path.write_bytes(original)
    shutil.rmtree(package)
assert command(["git", "status", "--porcelain"], clone).stdout == ""
receipt = dict(
    kind="clean sparse Git checkout of proposed QA snapshot, then independent corruption controls",
    source_base=command(["git", "rev-parse", "HEAD"]).stdout.strip(),
    snapshot=command(["git", "rev-parse", "HEAD"], clone).stdout.strip(),
    run_directory=str(run_directory),
    ignored_package_absent=True, clean_before_and_after=True,
    no_network=True, no_package_launch=True, cases=results,
)
(run_directory / "verifier-checks.json").write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps(receipt, indent=2))

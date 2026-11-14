"""Exercise the full manual replay at a simulated landed tip, including -O drift."""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

here = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("simulation", here / "simulate_unrefreshed.py")
simulation = importlib.util.module_from_spec(spec)
spec.loader.exec_module(simulation)
repo, scope = simulation.repo, simulation.scope
command, require = simulation.command, simulation.require
(here / "target").mkdir(exist_ok=True)
run = Path(tempfile.mkdtemp(prefix="replay-", dir=here / "target"))
clone = run / "checkout"
command(["git", "clone", "--shared", "--no-checkout", "--quiet", str(repo), str(clone)])
command(["git", "checkout", "--quiet", "-b",
         "integrate/management-workstreams-20260911", "HEAD"], clone)
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


def commit(message):
    command(["git", "add", "--", str(scope)], clone)
    command(["git", "-c", "user.name=Evidence Check",
             "-c", "user.email=evidence.invalid@example.invalid",
             "-c", "commit.gpgsign=false", "commit", "--quiet", "--no-verify",
             "-m", message], clone)


commit("Simulated acct-guard-91 landing; no source ref changed")
cases = []
for mutation in (False, True):
    if mutation:
        # Keep the new digest internally consistent but make its output wrong.
        # This isolates the no-decay output comparison from the reference hash check.
        reference = clone / scope / "acct-reference-88/expected.stdout.txt"
        manifest_path = clone / scope / "acct-reference-88/reference.json"
        reference.write_bytes(reference.read_bytes() + b"stale reference control\n")
        manifest = json.loads(manifest_path.read_text())
        manifest["sha256"] = hashlib.sha256(reference.read_bytes()).hexdigest()
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
        commit("Deliberately wrong content-bound expectation")
    for optimized in (False, True):
        name = ("mismatch" if mutation else "landed") + ("-optimized" if optimized else "")
        args = [sys.executable, "-B", *(["-O"] if optimized else []),
                str(scope / "acct-selfcheck-84/check_current_tip.py")]
        result = subprocess.run(args, cwd=clone, env=simulation.environment, capture_output=True)
        (run / (name + ".stdout.txt")).write_bytes(result.stdout)
        (run / (name + ".stderr.txt")).write_bytes(result.stderr)
        receipt = json.loads(result.stdout)
        require(result.returncode == (1 if mutation else 0), name + ": exit differs")
        require(receipt["stdout_matches_prior_replay"] == (not mutation), name + ": receipt differs")
        if mutation:
            require(b"AssertionError: current integration output differs from pinned replay"
                    in result.stderr, name + ": wrong rejection")
        else:
            require(not result.stderr, name + ": unexpected stderr")
        require(receipt["status_before"] == receipt["status_after"] == "", name + ": dirty checkout")
        cases.append(dict(case=name, command=args, exit=result.returncode,
                          output_match=receipt["stdout_matches_prior_replay"],
                          receipt=receipt))
record = dict(source_base=command(["git", "rev-parse", "HEAD"]).stdout.strip(),
              run_directory=str(run), no_network=True, no_package_launch=True,
              source_refs_unchanged=True, cases=cases)
(run / "replay-checks.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record, indent=2))

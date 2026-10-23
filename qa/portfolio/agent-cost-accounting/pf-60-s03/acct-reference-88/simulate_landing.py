"""Apply the proposed QA change in a disposable clone, then advance its tip.
Never move the source worktree's branches. No build, package launch or network.
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
(here / "target").mkdir(exist_ok=True)
run = Path(tempfile.mkdtemp(prefix="landing-", dir=here / "target"))
clone = run / "checkout"


def git(*args, cwd=repo):
    return subprocess.check_output(["git", *args], cwd=cwd, env=environment, text=True)


def commit(message):
    git("add", "--", str(scope), cwd=clone)
    git("-c", "user.name=Evidence Check", "-c", "user.email=evidence.invalid@example.invalid",
        "-c", "commit.gpgsign=false", "commit", "--quiet", "--no-verify", "-m", message, cwd=clone)
    return git("rev-parse", "HEAD", cwd=clone).strip()


base = git("rev-parse", "HEAD").strip()
git("clone", "--shared", "--no-checkout", "--quiet", str(repo), str(clone))
branch = "integrate/management-workstreams-20260911"
git("checkout", "--quiet", "-b", branch, base, cwd=clone)
changed = git("diff", "--name-only", "HEAD", "--", str(scope)).splitlines()
untracked = git("ls-files", "--others", "--exclude-standard", "--", str(scope)).splitlines()
tested_files = {}
for name in sorted(set(changed + untracked)):
    source, destination = repo / name, clone / name
    if source.is_file():
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
        tested_files[name] = hashlib.sha256(source.read_bytes()).hexdigest()
    elif destination.exists():
        destination.unlink()
landed = commit("Simulated landing of acct-reference-88 proposed QA changes")
assert landed != base
assert git("status", "--porcelain", cwd=clone) == ""


def replay(label):
    output = here / label
    output.mkdir(exist_ok=False)
    result = subprocess.run(
        [sys.executable, "-B", str(scope / "acct-selfcheck-84/check_current_tip.py")],
        cwd=clone, env=environment, capture_output=True)
    (output / "helper.stdout.txt").write_bytes(result.stdout)
    (output / "helper.stderr.txt").write_bytes(result.stderr)
    assert result.returncode == 0 and not result.stderr, result.stderr.decode()
    receipt = json.loads(result.stdout)
    source = Path(receipt["run_directory"])
    for filename in ("receipt.json", "acceptance.stdout.txt", "acceptance.stderr.txt",
                     "controls.stdout.txt", "controls.stderr.txt"):
        shutil.copyfile(source / filename, output / filename)
    assert receipt["stdout_matches_prior_replay"]
    assert receipt["reference_source_commit"] == receipt["commit"]
    assert git("status", "--porcelain", cwd=clone) == ""
    return receipt


first = replay("landed")
# A real unrelated tracked edit advances the simulated integration branch.
marker = clone / scope / "acct-reference-88/simulation-only.txt"
marker.write_text("Unrelated later evidence; not an inventoried verifier input.\n")
advanced = commit("Simulated subsequent unrelated integration")
assert advanced != landed
second = replay("advanced")
assert first["commit"] == landed and second["commit"] == advanced
assert first["reference_sha256"] == second["reference_sha256"]
assert (here / "landed/acceptance.stdout.txt").read_bytes() == (
    here / "advanced/acceptance.stdout.txt").read_bytes()
record = dict(source_base=base, clone=str(clone), integration_ref="refs/heads/" + branch,
              before=base, landed=landed, advanced=advanced, source_refs_unchanged=True,
              proposed_files_sha256=tested_files, replay_exits=[0, 0],
              evidence_exits=[first["commands"][0]["exit"], second["commands"][0]["exit"]],
              reference_sha256=first["reference_sha256"], identical_stdout=True,
              no_network=True, no_package_launch=True)
assert git("rev-parse", "HEAD").strip() == base
(here / "landing-demonstration.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record, indent=2))

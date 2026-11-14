"""Replay the integration tip against committed, content-bound expected output."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

INTEGRATION_REF = "refs/heads/integrate/management-workstreams-20260911"
REFERENCE_DIRECTORY = "acct-reference-88"


def require(condition, detail):
    if not condition:
        raise AssertionError(detail)


def assert_receipt(receipt):
    require(receipt["stdout_matches_prior_replay"],
            "current integration output differs from pinned replay")
    require(receipt["commit"] == receipt["source_tip_after"], "integration tip advanced during replay")
    require(receipt["status_before"] == receipt["status_after"] == "", "checkout changed")
    for row in receipt["commands"]:
        require(row["exit"] == row["expected_exit"], ("unexpected command exit", row))
        require(row["stderr_empty"], ("command wrote stderr", row))


def main():
    here = Path(__file__).resolve().parent
    repo = here.parents[4]
    scope = here.parent.relative_to(repo)
    environment = dict(os.environ, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                       GIT_TERMINAL_PROMPT="0")

    def git(*args, cwd=repo):
        return subprocess.check_output(["git", *args], cwd=cwd, env=environment)

    commit = git("rev-parse", "--verify", INTEGRATION_REF).decode().strip()
    reference_path = scope / REFERENCE_DIRECTORY / "expected.stdout.txt"
    manifest_path = scope / REFERENCE_DIRECTORY / "reference.json"
    manifest = json.loads(git("show", f"{commit}:{manifest_path}"))
    reference = git("show", f"{commit}:{reference_path}")
    require(manifest["schema"] == 1 and manifest["path"] == str(reference_path),
            "reference schema or path differs")
    require(hashlib.sha256(reference).hexdigest() == manifest["sha256"], "reference digest differs")
    output = here.parent / REFERENCE_DIRECTORY / "target"
    output.mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix="current-tip-", dir=output))
    clone = run / "checkout"
    git("clone", "--shared", "--no-checkout", "--quiet", str(repo), str(clone))
    git("checkout", "--quiet", "--detach", commit, cwd=clone)
    before = git("status", "--porcelain", "--untracked-files=all", cwd=clone).decode()
    require(before == "", "checkout starts dirty")
    require(git("rev-parse", "HEAD", cwd=clone).decode().strip() == commit,
            "checkout commit differs from resolved integration tip")
    require(not (clone / scope / "acct-fitness-76/package").exists(),
            "ignored local package present in checkout")
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
                             expected_exit=expected, stderr_empty=not result.stderr,
                             stdout_sha256=hashlib.sha256(result.stdout).hexdigest(),
                             stderr_sha256=hashlib.sha256(result.stderr).hexdigest()))
    after = git("status", "--porcelain", "--untracked-files=all", cwd=clone).decode()
    actual = (run / "acceptance.stdout.txt").read_bytes()
    receipt = dict(integration_ref=INTEGRATION_REF, commit=commit,
                   source_tip_after=git("rev-parse", INTEGRATION_REF).decode().strip(),
                   clone=str(clone), run_directory=str(run), status_before=before,
                   status_after=after, full_checkout=True, copied_working_tree_files=False,
                   package_absent=True, no_network=True, no_package_launch=True,
                   reference_kind="committed content; no historical commit pin",
                   reference_source_commit=commit, reference=str(reference_path),
                   reference_manifest=str(manifest_path),
                   reference_sha256=hashlib.sha256(reference).hexdigest(),
                   stdout_matches_prior_replay=actual == reference, commands=receipts)
    # Preserve failed attempts before enforcing the no-decay claim.
    (run / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2), flush=True)
    assert_receipt(receipt)


if __name__ == "__main__":
    main()

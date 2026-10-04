"""Check every staging prerequisite; never copy, upload, mutate a pin or dispatch cases.

The owner-supplied checker is a trusted-host interface, not an admission substitute.
Without it or explicit guest-contact permission, full preflight exits nonzero.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import stat
import subprocess

import dispatch_guard
import prepare_payload
import stage

HERE = Path(__file__).resolve().parent
PRIOR = HERE.parent / "pf83-handoff-70"
OWNER_CHECKS = (
    "executor_mediator_interface",
    "owner_admission",
    "live_pin_authorization_and_hashes",
    "independent_reviewer_and_staging_authorization",
)


def require_path(path, label):
    stage.require(path is not None, label + " is required")
    return path


def run(args):
    results = []

    def check(name, operation):
        try:
            detail = operation()
            results.append(dict(name=name, status="passed", detail=detail))
            return True
        except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
            results.append(dict(name=name, status="failed", detail=str(error)))
            return False

    def receiving():
        root = require_path(args.receiving, "--receiving")
        stage.require(root.is_dir() and not root.is_symlink(), "receiving directory missing/link")
        stage.require(args.bundle.resolve() == (root / "bundle").resolve(),
                      "verify the bundle at its receiving location after the move")
        stage.require(args.packets.resolve() == (root / "coordinator-packets").resolve(),
                      "verify regenerated receiving packets")
        for name in ("actor-payload", "staging-evidence"):
            path = root / name
            stage.require(not path.exists() and not path.is_symlink(), name + " already occupied")
        stage.require(os.access(root, os.W_OK | os.X_OK), "receiving directory not writable")
        return "moved bundle/packet locations; fresh payload and staging evidence paths"

    def host_key():
        path = HERE / "known_hosts"
        stage.require(not path.is_symlink() and stage.sha(path) == stage.HOSTS_SHA,
                      "owner-supplied host key pin mismatch")
        return stage.HOSTS_SHA

    def identity_file():
        key = require_path(args.key, "--key")
        mode = key.stat().st_mode
        stage.require(stat.S_ISREG(mode), "SSH identity must be a regular file")
        stage.ssh_command(key, HERE / "known_hosts")
        return "regular non-symlink mode-0600 identity; contents not read"

    def owner_checker():
        path = require_path(args.owner_check, "--owner-check")
        stage.require(args.owner_check_sha256 is not None, "--owner-check-sha256 is required")
        stage.require(not path.is_symlink() and path.is_file()
                      and os.access(path, os.X_OK), "owner checker must be an executable regular file")
        stage.require(stage.sha(path) == args.owner_check_sha256, "owner checker digest mismatch")
        return args.owner_check_sha256

    receiving_ok = check("receiving_locations_and_fresh_outputs", receiving)

    def at_receiving(operation):
        stage.require(receiving_ok, "receiving location unverified; artifact check refused")
        return operation()

    check("package_attestation_after_move", lambda: at_receiving(
        lambda: dispatch_guard.verify(args.bundle, dispatch_guard.ATTESTATION)))
    check("actor_asset_allowlist", lambda: at_receiving(
        lambda: sorted(prepare_payload.selected(args.bundle))))
    check("candidate_agreement_and_288_packet_hashes", lambda: at_receiving(
        lambda: dispatch_guard.check(
            require_path(args.harness, "--harness"), args.packets, args.bundle)))
    check("pinned_public_host_key", host_key)
    check("ssh_identity_metadata", identity_file)
    checker_ok = check("owner_checker_identity", owner_checker)

    def owner_check(name):
        stage.require(checker_ok, "owner checker unavailable or not pinned; no approval inferred")
        harness = require_path(args.harness, "--harness")
        root = require_path(args.receiving, "--receiving")
        # This is an explicitly supplied trusted-host verifier, never a case launcher.
        argv = [str(args.owner_check.resolve()), "--check", name,
                "--harness", str(harness.resolve()), "--bundle", str(args.bundle.resolve()),
                "--packets", str(args.packets.resolve()), "--receiving", str(root.resolve()),
                "--attestation-sha256", dispatch_guard.ATTESTATION,
                "--guest-uuid", stage.GUEST_UUID, "--guest-account", "agent",
                "--guest-address", "192.168.64.3"]
        result = subprocess.run(argv, stdin=subprocess.DEVNULL, capture_output=True,
                                text=True, timeout=60)
        stage.require(result.returncode == 0,
                      "owner verifier " + name + " exit " + str(result.returncode))
        report = json.loads(result.stdout)
        stage.require(report.get("check") == name and report.get("status") == "passed"
                      and isinstance(report.get("evidence"), list) and report["evidence"],
                      "owner verifier must return matching passed check and evidence references")
        return report

    for name in OWNER_CHECKS:
        check(name, lambda name=name: owner_check(name))

    local_ok = all(row["status"] == "passed" for row in results)

    identity_observation = {}

    def record_identity(stdout, stderr, returncode):
        # Exact returned bytes survive mismatches, SSH failure and invalid UTF-8.
        identity_observation.update(
            stdout_base64=base64.b64encode(stdout or b"").decode("ascii"),
            stderr_base64=base64.b64encode(stderr or b"").decode("ascii"),
            returncode=returncode)

    def guest():
        stage.require(args.contact_guest, "--contact-guest not supplied; no guest contact authorized")
        stage.require(local_ok, "local/owner prerequisites failed; guest contact suppressed")
        ssh = stage.ssh_command(args.key, HERE / "known_hosts")
        try:
            result = subprocess.run(ssh + [stage.IDENTITY], stdin=subprocess.DEVNULL,
                                    capture_output=True, timeout=30)
        except subprocess.TimeoutExpired as error:
            record_identity(error.stdout, error.stderr, None)
            identity_observation["timed_out"] = True
            raise
        record_identity(result.stdout, result.stderr, result.returncode)
        stage.require(result.returncode == 0, "identity SSH exit " + str(result.returncode))
        raw = result.stdout.decode("utf-8", errors="strict")
        identity_observation["stdout"] = raw
        stage.check_identity(raw)
        return identity_observation

    check("live_ssh_host_uuid_os_arch_account_uid", guest)
    passed = all(row["status"] == "passed" for row in results)
    report = dict(checks=results, passed=passed, exit_code=0 if passed else 1,
                  upload_calls=0, packaged_binary_executed=False,
                  guest_identity_observation=identity_observation,
                  guest_contact_attempted=local_ok and args.contact_guest)
    print(json.dumps(report, indent=2, sort_keys=True))
    return report["exit_code"]


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("--harness", type=Path)
    result.add_argument("--receiving", type=Path)
    result.add_argument("--bundle", type=Path, default=PRIOR / "artifacts/handoff-root")
    result.add_argument("--packets", type=Path, default=PRIOR / "artifacts/rebound")
    result.add_argument("--key", type=Path)
    result.add_argument("--owner-check", type=Path)
    result.add_argument("--owner-check-sha256")
    result.add_argument("--contact-guest", action="store_true")
    return result


if __name__ == "__main__":
    raise SystemExit(run(parser().parse_args()))

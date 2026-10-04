"""Single-writer, exact-commit integration with durable receiving-test receipts.

Only trusted owner allocations may call this module. Tests are explicit argv,
never model-generated shell. A crash marker survives process-lock release and
requires an owner reconciliation; no reset, abort, retry, push or cleanup is
performed automatically on an uncertain merge.
"""

from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import time

from coordinator import Rejected, encoded, ident, require


def git(repo, *args):
    result = subprocess.run(["git", "-C", str(repo), *args], text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
    require(result.returncode == 0, "Git preflight failed: " + result.stderr[:500])
    return result.stdout.strip()


def exact_sha(value):
    require(isinstance(value, str) and re.fullmatch("[0-9a-f]{40}", value), "exact commit required")
    return value


def check_command(argv, timeout):
    require(isinstance(argv, list) and argv and all(isinstance(a, str) and "\0" not in a for a in argv), "explicit argv required")
    require(type(timeout) in {int, float} and 0 < timeout <= 3600, "bounded receiving-test timeout required")


def write_record(path, value):
    temp = path.with_suffix(".pending")
    fd = os.open(temp, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "w") as stream:
        stream.write(encoded(value))
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temp, path)
    descriptor = os.open(path.parent, os.O_RDONLY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


class Integrator:
    def __init__(self, repository, receipts):
        self.repo = Path(repository).resolve(strict=True)
        require(git(self.repo, "rev-parse", "--show-toplevel") == str(self.repo), "exact checkout root required")
        common = Path(git(self.repo, "rev-parse", "--git-common-dir"))
        if not common.is_absolute():
            common = self.repo / common
        self.control = common.resolve() / "corbanu-integration"
        self.control.mkdir(mode=0o700, exist_ok=True)
        require(not self.control.is_symlink() and self.control.stat().st_uid == os.getuid()
                and self.control.stat().st_mode & 0o777 == 0o700, "private integration control required")
        self.receipts = Path(receipts).absolute()
        self.receipts.mkdir(mode=0o700, exist_ok=True)
        require(not self.receipts.is_symlink() and self.receipts.stat().st_uid == os.getuid()
                and self.receipts.stat().st_mode & 0o777 == 0o700, "private receipt directory required")
        self.marker = self.control / "active.json"

    @contextmanager
    def lock(self):
        descriptor = os.open(self.control / "writer.lock", os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
        try:
            try:
                fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as exc:
                raise Rejected("integration writer already active") from exc
            yield
        finally:
            os.close(descriptor)

    def command(self, argv, log, timeout):
        check_command(argv, timeout)
        descriptor = os.open(log, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        started = time.time()
        timed_out = False
        with os.fdopen(descriptor, "wb") as output:
            child = subprocess.Popen(argv, cwd=self.repo, stdin=subprocess.DEVNULL,
                                     stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
            try:
                code = child.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                timed_out = True
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                code = child.wait()
            except BaseException:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                child.wait()
                raise
            # A successful test may leave helpers in its owned process group.
            # Stop those before hashing logs or accepting the receiving tree.
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            output.flush()
            os.fsync(output.fileno())
        hasher = hashlib.sha256()
        with log.open("rb") as stream:
            for chunk in iter(lambda: stream.read(65536), b""):
                hasher.update(chunk)
        return {"argv": argv, "exit_code": code, "timed_out": timed_out,
                "elapsed_seconds": time.time() - started, "log": str(log), "sha256": hasher.hexdigest()}

    def merge(self, assignment):
        """Assignment is owner-frozen: id/branch/base/source/scope/tests/approval."""
        job_id = ident(assignment["id"])
        exact_sha(assignment["base"])
        exact_sha(assignment["source"])
        require(assignment["branch"] not in {"main", "master"}, "main integration not authorized")
        require(bool(assignment["approval"]) and bool(assignment["tests"]), "approval and receiving tests required")
        require(isinstance(assignment["tests"], list), "receiving tests must be a list")
        for check in assignment["tests"]:
            require(isinstance(check, dict) and set(check) == {"argv", "timeout_seconds"}, "invalid receiving test")
            check_command(check["argv"], check["timeout_seconds"])
        receipt_path = self.receipts / (job_id + ".json")
        with self.lock():
            require(not self.marker.exists(), "unreconciled prior integration; inspect active.json")
            require(not receipt_path.exists(), "assignment already attempted")
            require(git(self.repo, "symbolic-ref", "--short", "HEAD") == assignment["branch"], "wrong receiving branch")
            require(git(self.repo, "rev-parse", "HEAD") == assignment["base"], "stale receiving base")
            require(not git(self.repo, "status", "--porcelain"), "receiving checkout dirty")
            require(git(self.repo, "cat-file", "-t", assignment["source"]) == "commit", "source is not commit")
            ancestor = git(self.repo, "merge-base", assignment["base"], assignment["source"])
            changed = git(self.repo, "diff", "--no-renames", "--name-only", ancestor, assignment["source"]).splitlines()
            scope = assignment["scope"]
            require(scope and all(isinstance(p, str) and p and not p.startswith("/")
                                 and ".." not in p.split("/") for p in scope), "invalid frozen scope")
            require(all(any(name == p or p.endswith("/") and name.startswith(p) for p in scope)
                        for name in changed), "source exceeds frozen write scope")
            receipt = {"assignment": assignment, "status": "started", "started": time.time(), "tests": []}
            write_record(self.marker, receipt)
            try:
                receipt["merge"] = self.command(
                    ["git", "-c", "core.hooksPath=/dev/null", "-c", "commit.gpgSign=false",
                     "merge", "--no-ff", "--no-edit", assignment["source"]],
                    self.receipts / (job_id + ".merge.log"), 120)
                receipt["receiving_commit"] = git(self.repo, "rev-parse", "HEAD")
                if receipt["merge"]["exit_code"] != 0:
                    receipt["status"] = "merge_failed"
                else:
                    for number, check in enumerate(assignment["tests"]):
                        receipt["tests"].append(self.command(check["argv"], self.receipts / f"{job_id}.test-{number}.log", check["timeout_seconds"]))
                        if receipt["tests"][-1]["exit_code"] != 0:
                            break
                    unchanged = git(self.repo, "rev-parse", "HEAD") == receipt["receiving_commit"]
                    clean = not git(self.repo, "status", "--porcelain")
                    receipt["status"] = "verified" if unchanged and clean and all(t["exit_code"] == 0 for t in receipt["tests"]) else "verification_failed"
                write_record(receipt_path, receipt)
                if receipt["status"] == "verified":
                    self.marker.unlink()
                else:
                    write_record(self.marker, receipt)
                return receipt
            except BaseException:
                # Retain durable started marker and logs, including KeyboardInterrupt.
                raise

    def reconcile(self, job_id, expected_head, evidence):
        """Release a failed/crashed gate only after explicit owner inspection.

        Does not retry a merge, claim tests passed, or discard its original receipt.
        A new assignment is required for any new attempt.
        """
        exact_sha(expected_head)
        require(bool(evidence), "owner reconciliation evidence required")
        with self.lock():
            prior = json.loads(self.marker.read_text())
            require(prior["assignment"]["id"] == job_id, "wrong integration assignment")
            require(git(self.repo, "symbolic-ref", "--short", "HEAD") == prior["assignment"]["branch"],
                    "reconcile from the exact receiving branch")
            require(git(self.repo, "rev-parse", "HEAD") == expected_head, "head changed")
            require(not git(self.repo, "status", "--porcelain"), "resolve checkout before reconciliation")
            merge_head = subprocess.run(["git", "-C", str(self.repo), "rev-parse", "--verify", "MERGE_HEAD"], capture_output=True)
            require(merge_head.returncode != 0, "merge still in progress")
            path = self.receipts / (ident(job_id) + ".reconciled.json")
            require(not path.exists(), "already reconciled")
            write_record(path, {"prior": prior, "head": expected_head, "evidence": evidence, "status": "reconciled_not_accepted"})
            self.marker.unlink()

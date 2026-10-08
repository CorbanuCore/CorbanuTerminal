#!/usr/bin/env python3
"""Run the PF-13 credential-boundary canary and adversarial qualification."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

REPORT_NAME = "credential-canary-report.json"
MAX_CAPTURE_BYTES = 1024 * 1024
SUPPORTED_HOSTS = {"Linux", "Darwin", "Windows"}
CANARY_SENTINEL = "CORBANU_SECURITY_CREDENTIAL_CANARY "
CANARY_DIGEST_PATTERN = re.compile(r"^[0-9a-f]{64}$")
SECRET_PATTERNS = (
    re.compile(r"(?i)\b(?:sk|ghp|gho|ghu|ghs|ghr|github_pat)[-_][A-Za-z0-9_-]{8,}"),
    re.compile(r"(?i)\bBearer\s+[^\s\"']{8,}"),
)
# Colour codes would otherwise split a key (pretty_assertions colours diffs).
ANSI_ESCAPE = re.compile(r"\x1b\[[0-9;]*[A-Za-z]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)")
SENSITIVE_ENV_SUFFIXES = ("_API_KEY", "_TOKEN", "_SECRET", "_PASSWORD")
FAILURE_REPORT_NAME = "credential-canary-failure.json"
# Diagnostics are printed to CI logs, so anything token-shaped is masked even
# though probe output already passed `assert_secret_free`.
REDACTION_PATTERNS = (
    re.compile(r"(?i)\b(Bearer\s+)[^\s\"',)]+"),
    re.compile(
        r"(?i)\b((?:sk|ghp|gho|ghu|ghs|ghr|github_pat|xox[abprs])[-_])[A-Za-z0-9_-]{6,}"
    ),
    re.compile(
        r"(?i)(?<![A-Za-z0-9])"
        r"((?:x-api-key|api[_-]?key|token|password|secret)[\"']?\s*[:=]\s*)\S+"
    ),
    # Long mixed-case runs with digits (keys, JWT and base64 segments); paths
    # and test names are rarely mixed-case with digits.
    re.compile(
        r"()(?<![A-Za-z0-9_])(?=[A-Za-z0-9+/-]*\d)(?=[A-Za-z0-9+/-]*[a-z])"
        r"(?=[A-Za-z0-9+/-]*[A-Z])[A-Za-z0-9+/-]{32,}={0,2}(?![A-Za-z0-9_])"
    ),
    # Long unseparated letter-digit runs (hex keys, digests).
    re.compile(
        r"()(?<![A-Za-z0-9_])(?=[A-Za-z0-9+]*\d)(?=[A-Za-z0-9+]*[A-Za-z])"
        r"[A-Za-z0-9+]{32,}={0,2}(?![A-Za-z0-9_])"
    ),
)
MAX_PANIC_LINES = 40
MAX_TAIL_LINES = 20
MAX_DIAGNOSTIC_CHARS = 6000
RUNNING_PATTERN = re.compile(r"^running (\d+) tests?$", re.MULTILINE)
REPORTED_PATTERN = re.compile(r"^test \S+ \.\.\. \w+", re.MULTILINE)
FAILED_TEST_PATTERN = re.compile(r"^test (\S+) \.\.\. FAILED\s*$", re.MULTILINE)
PANIC_PATTERN = re.compile(r"^thread '([^']+)'(?: \(\d+\))? panicked at (.+?):?$")
PANIC_END_PREFIXES = (
    "note: ",
    "thread '",
    "test ",
    "running ",
    "stack backtrace:",
    "---- ",
    "failures:",
)
PROCESS_ERROR_PATTERN = re.compile(
    r"^(?:error: .*|.*process didn't exit successfully.*|.*\(signal: \d+.*)$",
    re.MULTILINE,
)
REQUIRED_CANARY_SURFACES = {
    "exact_outgoing_request_capture",
    "model_context",
    "tool_payloads",
    "child_environment",
    "logs",
    "audit",
    "errors",
    "receipts",
    "crash_output",
    "vault_artifacts",
}


class QualificationError(RuntimeError):
    """A fail-closed PF-13 qualification failure."""

    def __init__(self, message: str, diagnostics: dict[str, Any] | None = None):
        super().__init__(message)
        self.diagnostics = diagnostics


@dataclass(frozen=True)
class CommandResult:
    command: list[str]
    returncode: int
    stdout: str
    stderr: str

    def as_json(self) -> dict[str, Any]:
        return {
            "command": self.command,
            "returncode": self.returncode,
            "stdout": self.stdout,
            "stderr": self.stderr,
        }


@dataclass(frozen=True)
class Probe:
    probe_id: str
    package: str
    cargo_args: tuple[str, ...]
    expected_tests: tuple[str, ...]
    source_paths: tuple[str, ...]
    covers: tuple[str, ...]


PROBES = (
    Probe(
        probe_id="policy-authority-validation",
        package="codex-security-policy",
        cargo_args=("--lib", "credential"),
        expected_tests=(
            "credential_request_is_a_complete_secret_free_authority_object",
            "credential_request_digest_binds_every_authority_dimension",
            "credential_request_rejects_invalid_authority_and_lifecycle",
            "credential_request_fails_on_revocation_and_generation_change",
            "malformed_or_ambiguous_credential_metadata_fails_closed",
            "capability_id_is_a_digest_identifier_not_a_bearer_value",
            "credential_use_receipt_is_bound_and_contains_only_secret_free_metadata",
        ),
        source_paths=(
            "codex-rs/security-policy/src/credential_tests.rs",
            "codex-rs/security-policy/src/security_policy_tests.rs",
        ),
        covers=("malformed", "expired", "revoked", "scope", "receipts"),
    ),
    Probe(
        probe_id="vault-scoped-resolution",
        package="codex-vault",
        cargo_args=("--lib", "capability"),
        expected_tests=(
            "scoped_resolution_exposes_secret_only_inside_redacted_callback",
            "callback_error_cancellation_and_panic_are_contained_and_secret_free",
            "missing_deleted_and_ineligible_credentials_fail_closed",
            "expired_and_revoked_authority_is_revalidated_before_decryption",
            "mismatched_label_and_scope_are_rejected_as_stable_errors",
        ),
        source_paths=(
            "codex-rs/vault/src/capability_tests.rs",
            "codex-rs/vault/src/capability.rs",
        ),
        covers=("vault_callback", "crash_output", "expired", "revoked", "scope"),
    ),
    Probe(
        probe_id="vault-panic-hook-containment",
        package="codex-vault",
        cargo_args=("--lib", "credential_panic"),
        expected_tests=(
            "scoped_credential_panic_guard_restores_nested_and_unwound_scopes",
            "scoped_credential_panic_hook_is_thread_local_and_preserves_other_panics",
        ),
        source_paths=(
            "codex-rs/vault/src/credential_panic_tests.rs",
            "codex-rs/vault/src/credential_panic.rs",
        ),
        covers=(
            "panic_hook",
            "nested_scope",
            "concurrent_scope",
            "ordinary_panic_compatibility",
        ),
    ),
    Probe(
        probe_id="vault-canonical-home-binding",
        package="codex-vault",
        cargo_args=("--lib", "home"),
        expected_tests=(
            "copied_vault_in_another_home_cannot_use_the_original_keyring_account",
            "canonical_home_alias_preserves_keyring_identity_and_label_case",
        ),
        source_paths=("codex-rs/vault/src/tests.rs", "codex-rs/secrets/src/lib.rs"),
        covers=("canonical_home", "copied_ciphertext", "custom_home_compatibility"),
    ),
    Probe(
        probe_id="production-tui-panic-hook",
        package="codex-tui",
        cargo_args=("--lib", "production_panic_hook_does_not_log_scoped_credentials"),
        expected_tests=("production_panic_hook_does_not_log_scoped_credentials",),
        source_paths=(
            "codex-rs/tui/src/credential_panic_tests.rs",
            "codex-rs/tui/src/lib.rs",
            "codex-rs/tui/src/tui.rs",
        ),
        covers=("production_panic_hook", "stderr", "logs", "panic_recovery"),
    ),
    Probe(
        probe_id="proxy-injection-boundary",
        package="codex-network-proxy",
        cargo_args=("--lib", "credential_broker"),
        expected_tests=(
            "virtualize_child_env_replaces_supported_credentials",
            "child_without_dummy_cannot_use_previous_child_credential",
            "scoped_openai_route_injects_once_and_passes_complete_context",
            "scoped_openai_denial_matrix_fails_before_resolution",
            "scoped_openai_stale_authority_and_unsupported_route_fail_closed",
        ),
        source_paths=("codex-rs/network-proxy/src/credential_broker_tests.rs",),
        covers=(
            "child_environment",
            "replay",
            "redirect",
            "wrong_method",
            "wrong_host",
            "expired",
            "revoked",
        ),
    ),
    Probe(
        probe_id="core-capability-and-unique-canary",
        package="codex-core",
        cargo_args=("--lib", "credential_capability"),
        expected_tests=(
            "issued_capability_is_consumed_only_for_the_complete_bound_request",
            "concurrent_duplicate_consumption_allows_exactly_one_use",
            "capability_authority_does_not_survive_runtime_restart",
            "adjacent_actor_purpose_operation_method_host_path_and_scope_fail",
            "forged_bearer_and_public_id_alone_cannot_authorize",
            "expiry_and_revocation_remove_authority_before_reuse",
            "capacity_is_hard_bounded_and_cleanup_reclaims_space",
            "concurrent_issuance_never_aliases_capability_ids",
            "credential_authority_unique_canary_is_confined_to_one_outgoing_request",
            "credential_authority_revocation_before_resolve_denies_without_vault_access",
        ),
        source_paths=("codex-rs/core/src/security/credential_capability_tests.rs",),
        covers=(
            "authorized_request",
            "unique_canary",
            "forged",
            "expired",
            "revoked",
            "replay",
            "wrong_actor",
            "wrong_purpose",
            "wrong_operation",
            "wrong_method",
            "wrong_host",
            "wrong_scope",
            "concurrent_use",
            "bounded_store",
            "artifacts",
        ),
    ),
    Probe(
        probe_id="core-revocation-linearization",
        package="codex-core",
        cargo_args=(
            "--lib",
            "credential_authority_revoke_during_use_linearizes_after_the_active_resolution",
        ),
        expected_tests=(
            "credential_authority_revoke_during_use_linearizes_after_the_active_resolution",
        ),
        source_paths=("codex-rs/core/src/config/network_proxy_credential_tests.rs",),
        covers=("revocation_race",),
    ),
    Probe(
        probe_id="protected-raw-export-denial",
        package="codex-cli",
        cargo_args=("--test", "vault", "vault_auth_helper"),
        expected_tests=(
            "vault_auth_helper_denies_raw_export_in_protected_levels_without_label_disclosure",
            "vault_auth_helper_preserves_permissive_compatibility_path",
            "vault_auth_helper_cli_override_cannot_downgrade_persisted_posture",
            "vault_auth_helper_symlink_home_cannot_downgrade_persisted_posture",
        ),
        source_paths=("codex-rs/cli/tests/vault.rs",),
        covers=("raw_export", "downgrade", "errors"),
    ),
)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def sanitized_environment(temporary_directory: Path) -> dict[str, str]:
    environment = {}
    for key, value in os.environ.items():
        upper = key.upper()
        if upper.endswith(SENSITIVE_ENV_SUFFIXES):
            continue
        if upper.startswith("CORBANU_SECURITY_CREDENTIAL_CANARY"):
            continue
        environment[key] = value
    environment["TMPDIR"] = str(temporary_directory)
    return environment


def run_command(
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    timeout_seconds: int = 900,
) -> CommandResult:
    try:
        completed = subprocess.run(
            command,
            cwd=cwd,
            env=env,
            text=True,
            encoding="utf-8",
            errors="replace",
            capture_output=True,
            timeout=timeout_seconds,
            check=False,
        )
    except subprocess.TimeoutExpired as timeout:
        raise QualificationError(
            f"command timed out after {timeout_seconds}s; output capture is "
            f"incomplete: {redact(' '.join(command))}"
            f"{partial_output_tail(timeout)}"
        ) from None
    # Never discard unscanned output or certify a partial test transcript. Scan
    # first so a credential beyond the retention limit still fails explicitly.
    for surface, output in (("stdout", completed.stdout), ("stderr", completed.stderr)):
        assert_secret_free(output, f"command {surface}")
        output_bytes = len(output.encode("utf-8"))
        if output_bytes > MAX_CAPTURE_BYTES:
            raise QualificationError(
                f"command {surface} exceeds the capture limit: "
                f"{output_bytes} bytes > {MAX_CAPTURE_BYTES} bytes"
            )
    return CommandResult(
        command=command,
        returncode=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
    )


def git_output(repo_root: Path, arguments: list[str], env: dict[str, str]) -> str:
    result = run_command(
        ["git", *arguments], cwd=repo_root, env=env, timeout_seconds=30
    )
    if result.returncode != 0:
        raise QualificationError(
            f"git {' '.join(arguments)} failed: {result.stderr.strip()}"
        )
    return result.stdout.strip()


def workspace_candidate_path(repo_root: Path) -> Path:
    executable = "corbanu.exe" if platform.system() == "Windows" else "corbanu"
    return (repo_root / "codex-rs" / "target" / "debug" / executable).resolve()


def build_candidate(
    repo_root: Path, candidate: Path, env: dict[str, str]
) -> CommandResult:
    expected = workspace_candidate_path(repo_root)
    if candidate.resolve() != expected:
        raise QualificationError(
            f"--candidate must be the workspace binary built by this harness: {expected}"
        )
    result = run_command(
        [
            "cargo",
            "build",
            "--target-dir",
            str(repo_root / "codex-rs" / "target"),
            "-p",
            "codex-cli",
            "--bin",
            "corbanu",
        ],
        cwd=repo_root / "codex-rs",
        env=env,
    )
    if result.returncode != 0:
        raise QualificationError("candidate workspace build failed")
    return result


def candidate_identity(
    candidate: Path, repo_root: Path, env: dict[str, str]
) -> tuple[dict[str, str], CommandResult]:
    if not candidate.is_file():
        raise QualificationError(f"candidate binary does not exist: {candidate}")
    if platform.system() != "Windows" and not os.access(candidate, os.X_OK):
        raise QualificationError(f"candidate binary is not executable: {candidate}")
    version = run_command(
        [str(candidate), "--version"],
        cwd=repo_root,
        env=env,
        timeout_seconds=30,
    )
    if version.returncode != 0:
        raise QualificationError("candidate --version failed")
    return (
        {
            "path": str(candidate),
            "sha256": sha256_file(candidate),
            "version": (version.stdout or version.stderr).strip(),
        },
        version,
    )


def partial_output_tail(timeout: subprocess.TimeoutExpired) -> str:
    """The last lines a timed-out command printed, if they are secret-free."""
    lines = []
    for stream in (timeout.output, timeout.stderr):
        if isinstance(stream, bytes):
            stream = stream.decode("utf-8", errors="replace")
        if stream:
            lines.extend(stream.splitlines()[-MAX_TAIL_LINES:])
    if not lines:
        return ""
    tail = redact("\n".join(lines))[-MAX_DIAGNOSTIC_CHARS:]
    try:
        assert_secret_free(tail, "timeout output")
    except QualificationError:
        return "\n  (last output withheld: it held credential-shaped material)"
    return "\n  last output:\n" + "\n".join(f"    {line}" for line in tail.splitlines())


def assert_secret_free(value: str, surface: str) -> None:
    for pattern in SECRET_PATTERNS:
        match = pattern.search(value) or pattern.search(ANSI_ESCAPE.sub("", value))
        if match is not None:
            raise QualificationError(
                f"credential-shaped material escaped into {surface} "
                f"(pattern {pattern.pattern!r}; value withheld)"
            )


def redact(value: str) -> str:
    value = ANSI_ESCAPE.sub("", value)
    for pattern in REDACTION_PATTERNS:
        value = pattern.sub(lambda match: f"{match.group(1)}***", value)
    return value


def panic_reports(output: str) -> list[dict[str, str]]:
    """Each panic as test, location and its (bounded) message."""
    panics = []
    lines = output.splitlines()
    for index, line in enumerate(lines):
        match = PANIC_PATTERN.match(line)
        if match is None:
            continue
        message = []
        for following in lines[index + 1 : index + 1 + MAX_PANIC_LINES]:
            if following.startswith(PANIC_END_PREFIXES):
                break
            message.append(following)
        panics.append(
            {
                "test": match.group(1),
                "location": match.group(2),
                "message": redact("\n".join(message).strip()),
            }
        )
    return panics


def describe_probe_failure(
    probe: Probe, result: CommandResult, missing: list[str]
) -> tuple[str, dict[str, Any]]:
    """Explains a failed probe: which tests and assertions, expected vs observed."""
    combined = ANSI_ESCAPE.sub("", f"{result.stdout}\n{result.stderr}")
    failed_tests = FAILED_TEST_PATTERN.findall(combined)
    started = sum(int(count) for count in RUNNING_PATTERN.findall(combined))
    reported = len(REPORTED_PATTERN.findall(combined))
    panics = panic_reports(combined)
    process_errors = [redact(line) for line in PROCESS_ERROR_PATTERN.findall(combined)]
    diagnostics = {
        "status": "failed",
        "probe": probe.probe_id,
        "command": result.command,
        "returncode": result.returncode,
        "failed_tests": failed_tests,
        "panics": panics,
        "expected_tests_not_ok": missing,
        "process_errors": process_errors[-8:],
        "tests_started": started,
        "tests_reported": reported,
    }
    lines = [
        f"probe {probe.probe_id} failed: `{' '.join(result.command)}` "
        f"exited with {result.returncode}"
    ]
    if failed_tests:
        lines.append(f"  failed tests: {', '.join(failed_tests)}")
    for panic in panics:
        lines.append(f"  assertion in {panic['test']} at {panic['location']}:")
        lines.extend(f"    {line}" for line in panic["message"].splitlines())
    if missing:
        lines.append(f"  expected tests not reported ok: {', '.join(missing)}")
    if not failed_tests and not panics:
        # No libtest verdict: the test binary crashed, hung or did not build.
        tail = redact("\n".join(result.stderr.strip().splitlines()[-MAX_TAIL_LINES:]))
        diagnostics["stderr_tail"] = tail
        lines.append(
            f"  no test failure was reported ({reported} of {started} started tests "
            "reported a result; a crash takes down the rest); last stderr lines:"
        )
        lines.extend(f"    {line}" for line in tail.splitlines())
    text = "\n".join(lines)
    if len(text) > MAX_DIAGNOSTIC_CHARS:
        text = text[:MAX_DIAGNOSTIC_CHARS] + "\n  ... (truncated)"
    assert_secret_free(text, "probe failure diagnostics")
    return text, diagnostics


def source_evidence(repo_root: Path, probe: Probe) -> list[dict[str, str]]:
    evidence = []
    for relative in probe.source_paths:
        path = repo_root / relative
        if not path.is_file():
            raise QualificationError(f"probe source is missing: {relative}")
        source = path.read_text(encoding="utf-8")
        for test_name in probe.expected_tests:
            if test_name in source:
                break
        evidence.append({"path": relative, "sha256": sha256_file(path)})
    missing = [
        name
        for name in probe.expected_tests
        if not any(
            name in (repo_root / path).read_text(encoding="utf-8")
            for path in probe.source_paths
        )
    ]
    if missing:
        raise QualificationError(
            f"probe {probe.probe_id} has missing source tests: {missing}"
        )
    return evidence


def executed_test_count(result: CommandResult) -> int:
    summaries = re.findall(
        r"test result: ok\.\s+(\d+) passed;", f"{result.stdout}\n{result.stderr}"
    )
    return sum(int(value) for value in summaries)


def validate_probe_output(probe: Probe, result: CommandResult) -> int:
    combined = f"{result.stdout}\n{result.stderr}"
    assert_secret_free(combined, f"probe {probe.probe_id} output")
    missing = [
        test_name
        for test_name in probe.expected_tests
        if re.search(rf"\b{re.escape(test_name)}\s+\.\.\.\s+ok\b", combined) is None
    ]
    if result.returncode != 0:
        message, diagnostics = describe_probe_failure(probe, result, missing)
        raise QualificationError(message, diagnostics)
    if missing:
        raise QualificationError(
            f"probe {probe.probe_id} did not execute expected tests: {missing}"
        )
    count = executed_test_count(result)
    if count < len(probe.expected_tests):
        raise QualificationError(f"probe {probe.probe_id} executed only {count} tests")
    return count


def parse_canary_result(results: list[CommandResult]) -> dict[str, Any]:
    payloads = []
    for result in results:
        for line in result.stdout.splitlines():
            if line.startswith(CANARY_SENTINEL):
                payloads.append(json.loads(line.removeprefix(CANARY_SENTINEL)))
    if len(payloads) != 1:
        raise QualificationError(
            f"expected exactly one dynamic canary result, found {len(payloads)}"
        )
    payload = payloads[0]
    digest = payload.get("canary_sha256")
    if not isinstance(digest, str) or CANARY_DIGEST_PATTERN.fullmatch(digest) is None:
        raise QualificationError("dynamic canary digest is invalid")
    if payload.get("outgoing_request_count") != 1:
        raise QualificationError("canary must authorize exactly one outgoing request")
    if payload.get("raw_secret_observations") != 1:
        raise QualificationError("raw canary must exist only in the outgoing capture")
    surfaces = payload.get("scanned_surfaces")
    if not isinstance(surfaces, list) or set(surfaces) != REQUIRED_CANARY_SURFACES:
        raise QualificationError("dynamic canary surface coverage is incomplete")
    return payload


def write_report(
    output_dir: Path, report: dict[str, Any], name: str = REPORT_NAME
) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    destination = output_dir / name
    serialized = json.dumps(report, indent=2, sort_keys=True) + "\n"
    assert_secret_free(serialized, "qualification report")
    with tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", dir=output_dir, delete=False
    ) as handle:
        handle.write(serialized)
        temporary = Path(handle.name)
    temporary.replace(destination)
    return destination


def run_qualification(
    repo_root: Path, candidate: Path, output_dir: Path
) -> tuple[bool, Path]:
    host_system = platform.system()
    if host_system not in SUPPORTED_HOSTS:
        raise QualificationError(
            f"unsupported qualification host {host_system!r}; host checks cannot be skipped"
        )

    temporary_parent = Path("/var/tmp") if Path("/var/tmp").is_dir() else None
    with tempfile.TemporaryDirectory(
        prefix="corbanu-credential-canary-", dir=temporary_parent
    ) as temporary:
        environment = sanitized_environment(Path(temporary))
        source_commit = git_output(repo_root, ["rev-parse", "HEAD"], environment)
        dirty_paths = git_output(
            repo_root, ["status", "--short"], environment
        ).splitlines()
        build = build_candidate(repo_root, candidate, environment)
        identity, version = candidate_identity(candidate, repo_root, environment)

        probe_reports = []
        command_results = []
        for probe in PROBES:
            command = [
                "cargo",
                "test",
                "-p",
                probe.package,
                *probe.cargo_args,
                "--",
                "--nocapture",
            ]
            result = run_command(
                command,
                cwd=repo_root / "codex-rs",
                env=environment,
            )
            command_results.append(result)
            executed = validate_probe_output(probe, result)
            probe_reports.append(
                {
                    "id": probe.probe_id,
                    "covers": list(probe.covers),
                    "expected_tests": list(probe.expected_tests),
                    "executed_tests": executed,
                    "sources": source_evidence(repo_root, probe),
                    "result": result.as_json(),
                }
            )

        canary = parse_canary_result(command_results)
        # CLI integration tests can replace target/debug/corbanu with a binary
        # linked using test dependency features. Restore the production build
        # and identify the exact artifact left for interactive qualification.
        pre_probe_build = build
        build = build_candidate(repo_root, candidate, environment)
        identity, version = candidate_identity(candidate, repo_root, environment)
        report = {
            "schema_version": 1,
            "status": "passed",
            "source_commit": source_commit,
            "source_dirty_paths": dirty_paths,
            "host": {
                "system": host_system,
                "release": platform.release(),
                "machine": platform.machine(),
                "python": platform.python_version(),
            },
            "candidate": identity,
            "candidate_build_command": build.as_json(),
            "candidate_pre_probe_build_command": pre_probe_build.as_json(),
            "candidate_version_command": version.as_json(),
            "canary": canary,
            "probes": probe_reports,
        }
        return True, write_report(output_dir, report)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    repo_root = Path(__file__).resolve().parent.parent
    try:
        passed, report_path = run_qualification(
            repo_root, arguments.candidate, arguments.output
        )
    except (
        QualificationError,
        json.JSONDecodeError,
        OSError,
        subprocess.TimeoutExpired,
    ) as error:
        print(f"security-credential-canary: {error}")
        diagnostics = getattr(error, "diagnostics", None)
        if diagnostics is not None:
            try:
                path = write_report(arguments.output, diagnostics, FAILURE_REPORT_NAME)
                print(f"security-credential-canary: failure details: {path}")
            except (QualificationError, OSError) as report_error:
                print(f"security-credential-canary: no failure report: {report_error}")
        return 2
    print(f"security-credential-canary: {'PASS' if passed else 'FAIL'}: {report_path}")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())

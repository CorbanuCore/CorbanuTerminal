#!/usr/bin/env python3
"""Verify the frozen Permissive contract against a candidate Corbanu binary."""

import argparse
import datetime
import hashlib
import json
import os
import platform
import re
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

BASELINE_PATH = Path("qa/security-levels/permissive-baseline-v1.json")
CONTROL_PATH = Path("qa/security-levels/compatibility/upstream-control-v2.json")
DRIFT_LEDGER_PATH = Path("qa/security-levels/compatibility/drift-ledger-v2.json")
FROZEN_BASELINE_SHA256 = (
    "45d1f2bd96733381638bb62961ee59fb1c026bc05a6a78d03b560cb794406b8d"
)
UPSTREAM_CONVERGENCE_COMMIT = "45a60f03d2f6c041d284b41cc3f33c416d9eeed1"
UPSTREAM_CODEX_PARENT = "413492cd6c3a4d4f8dff6f406247ccda5a9d88aa"
REPORT_NAME = "compatibility-report.json"
MAX_CAPTURE_BYTES = 64 * 1024
COMMIT_PATTERN = re.compile(r"^[0-9a-f]{40}$")
IDENTIFIER_PATTERN = re.compile(r"^[A-Za-z0-9_:-]+$")
REQUIRED_EXPANDED_SURFACES = frozenset(
    {
        "environment-auth",
        "web-run-history",
        "native-search",
        "browser",
        "mcp-plugins",
        "children",
        "wallet",
        "clipboard-export",
        "persisted-sessions",
    }
)


class CompatibilityError(RuntimeError):
    """A fail-closed compatibility contract violation."""


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


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def bounded_output(value: str) -> str:
    encoded = value.encode("utf-8", errors="replace")
    if len(encoded) <= MAX_CAPTURE_BYTES:
        return value
    suffix = b"\n... output truncated by security-level-compat ...\n"
    return (encoded[: MAX_CAPTURE_BYTES - len(suffix)] + suffix).decode(
        "utf-8", errors="replace"
    )


def run_command(
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str] | None = None,
    timeout_seconds: int = 600,
) -> CommandResult:
    completed = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        text=True,
        capture_output=True,
        timeout=timeout_seconds,
        check=False,
    )
    return CommandResult(
        command=command,
        returncode=completed.returncode,
        stdout=bounded_output(completed.stdout),
        stderr=bounded_output(completed.stderr),
    )


def extract_test_source(path: Path, function_name: str) -> str:
    text = path.read_text(encoding="utf-8")
    marker = re.compile(
        rf"(?m)^#\[(?:tokio::)?test\]\n(?:async\s+)?fn\s+{re.escape(function_name)}\s*\("
    )
    match = marker.search(text)
    if match is None:
        raise CompatibilityError(
            f"probe function {function_name!r} is missing from {path}"
        )
    attached_attributes = re.search(
        r"(?m)(?P<attributes>(?:^[ \t]*#\[[^\]]*\][ \t]*\n)+)\Z",
        text[: match.start()],
    )
    start = (
        attached_attributes.start("attributes")
        if attached_attributes is not None
        else match.start()
    )
    next_test = re.compile(r"(?m)^#\[(?:tokio::)?test\]\n").search(text, match.end())
    end = next_test.start() if next_test is not None else len(text)
    return text[start:end].rstrip() + "\n"


def executed_test_count(result: CommandResult) -> int:
    output = f"{result.stdout}\n{result.stderr}"
    summary = re.search(r"Summary\s+\[[^\]]+\]\s+(\d+)\s+tests?\s+run:", output)
    return int(summary.group(1)) if summary is not None else 0


def probe_source_digest(repo_root: Path, probe: dict[str, Any]) -> str:
    source = probe.get("source")
    function_name = probe.get("function")
    if not isinstance(source, str) or not isinstance(function_name, str):
        raise CompatibilityError("every probe requires source and function strings")
    path = repo_root / source
    if not path.is_file():
        raise CompatibilityError(f"probe source is missing: {source}")
    return sha256_bytes(extract_test_source(path, function_name).encode("utf-8"))


def validate_manifest(
    manifest: dict[str, Any], repo_root: Path, baseline_commit: str
) -> list[dict[str, Any]]:
    if manifest.get("schema_version") != 1:
        raise CompatibilityError("unsupported Permissive baseline schema")
    if manifest.get("captured_from_commit") != baseline_commit:
        raise CompatibilityError(
            "--baseline does not match the frozen manifest captured_from_commit"
        )
    contract = manifest.get("composition_contract")
    if not isinstance(contract, dict) or contract.get("rule") != (
        "final_allow = existing_allow && security_layer_allow"
    ):
        raise CompatibilityError("unexpected Permissive composition contract")
    if contract.get("permissive_security_layer_allow") is not True:
        raise CompatibilityError(
            "Permissive must add an allow-neutral security decision"
        )

    surfaces = manifest.get("surfaces")
    if not isinstance(surfaces, list) or not surfaces:
        raise CompatibilityError("baseline must define at least one surface")
    surface_ids = [
        surface.get("id") for surface in surfaces if isinstance(surface, dict)
    ]
    if len(surface_ids) != len(surfaces) or any(
        not isinstance(surface_id, str) or not surface_id for surface_id in surface_ids
    ):
        raise CompatibilityError("every baseline surface requires a non-empty id")
    if len(set(surface_ids)) != len(surface_ids):
        raise CompatibilityError("baseline surface ids must be unique")

    probes = manifest.get("probes")
    if not isinstance(probes, list) or not probes:
        raise CompatibilityError("baseline must define immutable candidate probes")
    covered: set[str] = set()
    for probe in probes:
        if not isinstance(probe, dict):
            raise CompatibilityError("every probe must be an object")
        for key in (
            "id",
            "package",
            "test_filter",
            "source",
            "function",
            "source_sha256",
        ):
            if not isinstance(probe.get(key), str) or not probe[key]:
                raise CompatibilityError(f"probe requires non-empty {key}")
        for key in ("id", "package", "test_filter"):
            if IDENTIFIER_PATTERN.fullmatch(probe[key]) is None:
                raise CompatibilityError(f"probe {key} contains unsupported characters")
        covers = probe.get("covers")
        if (
            not isinstance(covers, list)
            or not covers
            or any(not isinstance(item, str) for item in covers)
        ):
            raise CompatibilityError("probe covers must be a non-empty string list")
        unknown = set(covers) - set(surface_ids)
        if unknown:
            raise CompatibilityError(
                f"probe covers unknown surfaces: {sorted(unknown)}"
            )
        covered.update(covers)
        actual_digest = probe_source_digest(repo_root, probe)
        if actual_digest != probe["source_sha256"]:
            raise CompatibilityError(
                f"probe source drift for {probe['id']}: expected "
                f"{probe['source_sha256']}, found {actual_digest}"
            )
    missing_coverage = set(surface_ids) - covered
    if missing_coverage:
        raise CompatibilityError(
            f"baseline surfaces lack immutable probes: {sorted(missing_coverage)}"
        )
    return probes


def canonical_json_digest(value: Any) -> str:
    return sha256_bytes(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    )


def extract_expanded_test_source(text: str, function_name: str) -> str:
    marker = re.compile(
        rf"(?m)^(?P<indent>[ \t]*)(?:#\[[^\n]+\]\n(?P=indent))*"
        rf"(?:async\s+)?fn\s+{re.escape(function_name)}\s*\("
    )
    match = marker.search(text)
    if match is None:
        raise CompatibilityError(f"expanded probe function is missing: {function_name}")
    opening_brace = text.find("{", match.end())
    if opening_brace < 0:
        raise CompatibilityError(f"expanded probe has no body: {function_name}")
    depth = 0
    for index in range(opening_brace, len(text)):
        depth += text[index] == "{"
        depth -= text[index] == "}"
        if depth == 0:
            return text[match.start() : index + 1].rstrip() + "\n"
    raise CompatibilityError(f"expanded probe body is incomplete: {function_name}")


def source_at_commit(repo_root: Path, commit: str, path: str) -> str:
    completed = subprocess.run(
        ["git", "show", f"{commit}:{path}"],
        cwd=repo_root,
        text=True,
        capture_output=True,
        timeout=30,
        check=False,
    )
    if completed.returncode != 0:
        raise CompatibilityError(f"control source is missing: {commit}:{path}")
    return completed.stdout


def expanded_source_digest(
    repo_root: Path, commit: str, path: str, function_name: str
) -> str:
    source = source_at_commit(repo_root, commit, path)
    return sha256_bytes(
        extract_expanded_test_source(source, function_name).encode("utf-8")
    )


def parse_utc(value: str) -> datetime.datetime:
    try:
        parsed = datetime.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise CompatibilityError("control review timestamp is invalid") from error
    if parsed.tzinfo is None:
        raise CompatibilityError("control review timestamp requires a timezone")
    return parsed.astimezone(datetime.timezone.utc)


def validate_expanded_control(
    control: dict[str, Any],
    ledger: dict[str, Any],
    repo_root: Path,
    baseline_commit: str,
    upstream_commit: str,
    candidate_commit: str,
    *,
    now: datetime.datetime | None = None,
) -> list[dict[str, Any]]:
    if control.get("schema_version") != 2 or ledger.get("schema_version") != 2:
        raise CompatibilityError("unsupported expanded compatibility schema")
    identity = control.get("identity")
    if not isinstance(identity, dict):
        raise CompatibilityError("expanded control identity is missing")
    expected_identity = {
        "baseline_commit": baseline_commit,
        "baseline_manifest_sha256": FROZEN_BASELINE_SHA256,
        "upstream_commit": upstream_commit,
    }
    if any(identity.get(key) != value for key, value in expected_identity.items()):
        raise CompatibilityError("baseline/upstream control identities do not match")
    if identity.get("expectations_constructed_from_commit") != upstream_commit:
        raise CompatibilityError("expanded expectations are not upstream-derived")
    if identity.get("upstream_convergence_commit") != UPSTREAM_CONVERGENCE_COMMIT:
        raise CompatibilityError("upstream convergence identity does not match")
    if identity.get("upstream_codex_parent") != UPSTREAM_CODEX_PARENT:
        raise CompatibilityError("upstream Codex parent identity does not match")
    if upstream_commit == candidate_commit:
        raise CompatibilityError("candidate-derived expectations are forbidden")
    git_output(
        repo_root, ["merge-base", "--is-ancestor", upstream_commit, baseline_commit]
    )
    git_output(
        repo_root,
        ["merge-base", "--is-ancestor", UPSTREAM_CONVERGENCE_COMMIT, upstream_commit],
    )
    git_output(
        repo_root,
        ["merge-base", "--is-ancestor", UPSTREAM_CODEX_PARENT, upstream_commit],
    )
    git_output(
        repo_root, ["merge-base", "--is-ancestor", baseline_commit, candidate_commit]
    )

    reviewed_at = parse_utc(str(ledger.get("reviewed_at_utc", "")))
    max_age_days = ledger.get("max_age_days")
    if not isinstance(max_age_days, int) or max_age_days < 1:
        raise CompatibilityError("drift ledger requires a positive max_age_days")
    current = now or datetime.datetime.now(datetime.timezone.utc)
    if reviewed_at > current + datetime.timedelta(minutes=5):
        raise CompatibilityError("drift ledger review timestamp is in the future")
    if current - reviewed_at > datetime.timedelta(days=max_age_days):
        raise CompatibilityError("drift ledger is stale and requires owner review")
    if not isinstance(ledger.get("reviewed_by"), str) or not ledger["reviewed_by"]:
        raise CompatibilityError("drift ledger requires a named reviewer")

    surfaces = control.get("surfaces")
    if not isinstance(surfaces, list):
        raise CompatibilityError("expanded surface inventory is missing")
    surface_ids = {
        surface.get("id") for surface in surfaces if isinstance(surface, dict)
    }
    if surface_ids != REQUIRED_EXPANDED_SURFACES or len(surfaces) != len(surface_ids):
        raise CompatibilityError("expanded surface inventory is incomplete")
    for surface in surfaces:
        if surface.get("permissive_behavior") != "upstream-aligned":
            raise CompatibilityError(
                "every expanded surface must stay upstream-aligned"
            )
        if surface.get("protected_behavior") != "opt-in-above-permissive":
            raise CompatibilityError(
                "protected behavior must remain opt-in above Permissive"
            )

    entries = ledger.get("entries")
    if not isinstance(entries, list):
        raise CompatibilityError("drift ledger entries must be a list")
    drift_by_key: dict[tuple[str, str], dict[str, Any]] = {}
    for entry in entries:
        if not isinstance(entry, dict):
            raise CompatibilityError("drift ledger entry must be an object")
        key = (str(entry.get("identity")), str(entry.get("case_id")))
        if key in drift_by_key:
            raise CompatibilityError("duplicate drift ledger entry")
        if entry.get("disposition") != "accepted-intentional":
            raise CompatibilityError("drift ledger contains unaccepted drift")
        drift_by_key[key] = entry

    cases = control.get("cases")
    if not isinstance(cases, list) or not cases:
        raise CompatibilityError("expanded control requires executable cases")
    case_ids: set[str] = set()
    covered: set[str] = set()
    observed_drift: set[tuple[str, str]] = set()
    for case in cases:
        if not isinstance(case, dict):
            raise CompatibilityError("expanded case must be an object")
        for key in ("id", "surface", "package", "test_filter", "source", "function"):
            if not isinstance(case.get(key), str) or not case[key]:
                raise CompatibilityError(f"expanded case requires non-empty {key}")
        for key in ("id", "surface", "package", "test_filter", "function"):
            if IDENTIFIER_PATTERN.fullmatch(case[key]) is None:
                raise CompatibilityError(
                    f"expanded case {key} contains unsupported characters"
                )
        if case["test_filter"] != case["function"]:
            raise CompatibilityError(
                "expanded test filters must name the exact function"
            )
        if case["id"] in case_ids or case["surface"] not in surface_ids:
            raise CompatibilityError("expanded case identity or surface is invalid")
        case_ids.add(case["id"])
        covered.add(case["surface"])
        upstream_digest = expanded_source_digest(
            repo_root, upstream_commit, case["source"], case["function"]
        )
        if upstream_digest != case.get("upstream_source_sha256"):
            raise CompatibilityError(f"upstream expectation drift for {case['id']}")
        for label, commit in (
            ("baseline", baseline_commit),
            ("candidate", candidate_commit),
        ):
            observed = expanded_source_digest(
                repo_root, commit, case["source"], case["function"]
            )
            if observed == upstream_digest:
                continue
            key = (label, case["id"])
            entry = drift_by_key.get(key)
            if entry is None or entry.get("upstream_source_sha256") != upstream_digest:
                raise CompatibilityError(f"unknown {label} drift for {case['id']}")
            if entry.get("observed_source_sha256") != observed or not entry.get(
                "rationale"
            ):
                raise CompatibilityError(f"stale {label} drift entry for {case['id']}")
            observed_drift.add(key)
    if covered != surface_ids:
        raise CompatibilityError("expanded cases do not cover every required surface")
    if set(drift_by_key) != observed_drift:
        raise CompatibilityError("drift ledger contains stale or unobserved entries")
    return cases


def git_output(repo_root: Path, arguments: list[str]) -> str:
    result = run_command(["git", *arguments], cwd=repo_root, timeout_seconds=30)
    if result.returncode != 0:
        raise CompatibilityError(
            f"git {' '.join(arguments)} failed: {result.stderr.strip()}"
        )
    return result.stdout.strip()


def candidate_identity(
    candidate: Path, repo_root: Path
) -> tuple[dict[str, Any], CommandResult]:
    candidate = candidate.resolve()
    if not candidate.is_file():
        raise CompatibilityError(f"candidate binary does not exist: {candidate}")
    if os.name != "nt" and not os.access(candidate, os.X_OK):
        raise CompatibilityError(f"candidate binary is not executable: {candidate}")
    version = run_command(
        [str(candidate), "--version"], cwd=repo_root, timeout_seconds=30
    )
    if version.returncode != 0:
        raise CompatibilityError("candidate --version failed")
    identity = {
        "path": str(candidate),
        "sha256": sha256_file(candidate),
        "version": (version.stdout or version.stderr).strip(),
    }
    return identity, version


def workspace_candidate_path(repo_root: Path, target_dir: Path | None = None) -> Path:
    executable = "corbanu.exe" if os.name == "nt" else "corbanu"
    target = target_dir or (repo_root / "codex-rs" / "target")
    return (target / "debug" / executable).resolve()


def build_workspace_candidate(
    repo_root: Path,
    candidate: Path,
    target_dir: Path | None = None,
    env: dict[str, str] | None = None,
) -> CommandResult:
    target = target_dir or (repo_root / "codex-rs" / "target")
    expected = workspace_candidate_path(repo_root, target)
    if candidate.resolve() != expected:
        raise CompatibilityError(
            f"--candidate must be the workspace binary built by this harness: {expected}"
        )
    result = run_command(
        [
            "cargo",
            "build",
            "--target-dir",
            str(target),
            "-p",
            "codex-cli",
            "--bin",
            "corbanu",
        ],
        cwd=repo_root / "codex-rs",
        env=env,
    )
    if result.returncode != 0:
        raise CompatibilityError("candidate workspace build failed")
    return result


def require_clean_runtime_tree(repo_root: Path) -> None:
    result = run_command(
        ["git", "status", "--porcelain", "--", "codex-rs"],
        cwd=repo_root,
        timeout_seconds=30,
    )
    if result.returncode != 0:
        raise CompatibilityError("candidate runtime cleanliness check failed")
    if result.stdout.strip():
        raise CompatibilityError(
            "candidate runtime tree is dirty; commit or remove Rust/runtime changes"
        )


def controlled_environment(
    cache_root: Path, target_dir: Path, temp_dir: Path
) -> dict[str, str]:
    allowed = {
        "COMSPEC",
        "HOME",
        "HOMEDRIVE",
        "HOMEPATH",
        "LANG",
        "LC_ALL",
        "PATH",
        "PATHEXT",
        "RUSTUP_HOME",
        "SYSTEMROOT",
        "USERPROFILE",
        "WINDIR",
    }
    environment = {key: value for key, value in os.environ.items() if key in allowed}
    environment.update(
        {
            "CARGO_HOME": str(cache_root / "cargo-home"),
            "CARGO_TARGET_DIR": str(target_dir),
            "CARGO_TERM_COLOR": "never",
            "TMPDIR": str(temp_dir),
            "TMP": str(temp_dir),
            "TEMP": str(temp_dir),
        }
    )
    return environment


def environment_identity(repo_root: Path, env: dict[str, str]) -> dict[str, Any]:
    tools: dict[str, str] = {}
    for name in ("cargo", "rustc", "just", "git"):
        result = run_command(
            [name, "--version"], cwd=repo_root, env=env, timeout_seconds=30
        )
        if result.returncode != 0:
            raise CompatibilityError(f"{name} --version failed")
        tools[name] = (result.stdout or result.stderr).strip()
    facts = {
        "os": platform.system(),
        "architecture": platform.machine(),
        "python": platform.python_version(),
        "tools": tools,
        "ambient_overrides_present": {
            key: key in os.environ
            for key in ("CARGO_BUILD_TARGET", "RUSTFLAGS", "RUSTDOCFLAGS")
        },
    }
    return {"facts": facts, "sha256": canonical_json_digest(facts)}


def build_snapshot_binary(
    workspace: Path,
    target_dir: Path,
    binary_name: str,
    env: dict[str, str],
) -> tuple[dict[str, Any], CommandResult, CommandResult]:
    result = run_command(
        [
            "cargo",
            "build",
            "--target-dir",
            str(target_dir),
            "-p",
            "codex-cli",
            "--bin",
            binary_name,
        ],
        cwd=workspace / "codex-rs",
        env=env,
    )
    if result.returncode != 0:
        detail = (result.stderr or result.stdout).strip()
        suffix = f": {detail}" if detail else ""
        raise CompatibilityError(f"{binary_name} control build failed{suffix}")
    suffix = ".exe" if os.name == "nt" else ""
    binary = target_dir / "debug" / f"{binary_name}{suffix}"
    identity, version = candidate_identity(binary, workspace)
    return identity, result, version


def run_expanded_cases(
    workspace: Path,
    cases: list[dict[str, Any]],
    env: dict[str, str],
) -> list[dict[str, Any]]:
    results: list[dict[str, Any]] = []
    for case in cases:
        command = [
            "just",
            "test",
            "-p",
            case["package"],
            "--lib",
            case["test_filter"],
        ]
        result = run_command(command, cwd=workspace, env=env)
        count = executed_test_count(result)
        results.append(
            {
                "id": case["id"],
                "surface": case["surface"],
                "passed": result.returncode == 0 and count == 1,
                "executed_tests": count,
                "result": result.as_json(),
            }
        )
    return results


def expanded_results_pass(*suites: list[dict[str, Any]]) -> bool:
    return all(case["passed"] for suite in suites for case in suite)


def add_detached_worktree(repo_root: Path, destination: Path, commit: str) -> None:
    result = run_command(
        ["git", "worktree", "add", "--detach", str(destination), commit],
        cwd=repo_root,
        timeout_seconds=120,
    )
    if result.returncode != 0:
        raise CompatibilityError(f"failed to materialize control {commit}")


def remove_detached_worktree(repo_root: Path, destination: Path) -> None:
    result = run_command(
        ["git", "worktree", "remove", "--force", str(destination)],
        cwd=repo_root,
        timeout_seconds=120,
    )
    if result.returncode != 0:
        raise CompatibilityError(f"failed to remove control worktree {destination}")


def write_report(output_dir: Path, report: dict[str, Any]) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    destination = output_dir / REPORT_NAME
    serialized = json.dumps(report, indent=2, sort_keys=True) + "\n"
    with tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", dir=output_dir, delete=False
    ) as handle:
        handle.write(serialized)
        temporary = Path(handle.name)
    temporary.replace(destination)
    return destination


def run_compatibility(
    repo_root: Path,
    baseline_commit: str,
    upstream_commit: str,
    candidate: Path,
    output_dir: Path,
    cache_root: Path,
    temp_root: Path,
) -> tuple[bool, Path]:
    for flag, commit in (
        ("--baseline", baseline_commit),
        ("--upstream", upstream_commit),
    ):
        if COMMIT_PATTERN.fullmatch(commit) is None:
            raise CompatibilityError(f"{flag} must be a full 40-character commit id")
        git_output(repo_root, ["cat-file", "-e", f"{commit}^{{commit}}"])
    manifest_path = repo_root / BASELINE_PATH
    manifest_bytes = manifest_path.read_bytes()
    if sha256_bytes(manifest_bytes) != FROZEN_BASELINE_SHA256:
        raise CompatibilityError("accepted PF-21 baseline bytes changed")
    manifest = json.loads(manifest_bytes)
    probes = validate_manifest(manifest, repo_root, baseline_commit)
    source_commit = git_output(repo_root, ["rev-parse", "HEAD"])
    control = json.loads((repo_root / CONTROL_PATH).read_text(encoding="utf-8"))
    ledger = json.loads((repo_root / DRIFT_LEDGER_PATH).read_text(encoding="utf-8"))
    cases = validate_expanded_control(
        control,
        ledger,
        repo_root,
        baseline_commit,
        upstream_commit,
        source_commit,
    )
    require_clean_runtime_tree(repo_root)

    cache_root.mkdir(parents=True, exist_ok=True)
    temp_root.mkdir(parents=True, exist_ok=True)
    candidate_target = candidate.resolve().parent.parent
    candidate_temp = temp_root / "candidate-temp"
    candidate_temp.mkdir(parents=True, exist_ok=True)
    candidate_env = controlled_environment(cache_root, candidate_target, candidate_temp)
    build_result = build_workspace_candidate(
        repo_root, candidate, candidate_target, candidate_env
    )
    identity, version_result = candidate_identity(candidate, repo_root)
    candidate_cases = run_expanded_cases(repo_root, cases, candidate_env)

    baseline_identity: dict[str, Any]
    upstream_identity: dict[str, Any]
    baseline_cases: list[dict[str, Any]]
    upstream_cases: list[dict[str, Any]]
    control_commands: dict[str, Any] = {}
    controls_root = Path(tempfile.mkdtemp(prefix="controls-", dir=temp_root))
    worktrees: list[Path] = []
    try:
        for label, commit, binary_name in (
            ("baseline", baseline_commit, "corbanu"),
            ("upstream", upstream_commit, "corbanu"),
        ):
            workspace = controls_root / f"{label}-source"
            target = cache_root / "targets" / f"{label}-{commit}"
            temporary = controls_root / f"{label}-temp"
            temporary.mkdir(parents=True)
            add_detached_worktree(repo_root, workspace, commit)
            worktrees.append(workspace)
            environment = controlled_environment(cache_root, target, temporary)
            binary_identity, build, version = build_snapshot_binary(
                workspace, target, binary_name, environment
            )
            results = run_expanded_cases(workspace, cases, environment)
            control_commands[label] = {
                "build": build.as_json(),
                "version": version.as_json(),
            }
            if label == "baseline":
                baseline_identity, baseline_cases = binary_identity, results
            else:
                upstream_identity, upstream_cases = binary_identity, results
    finally:
        for workspace in reversed(worktrees):
            remove_detached_worktree(repo_root, workspace)

    dirty_paths = git_output(repo_root, ["status", "--short"]).splitlines()
    configuration = {
        "contract_version": 2,
        "cargo_profile": "dev",
        "cargo_features": "workspace-default",
        "color": "never",
        "control_isolation": "separate-source-and-target-directories",
    }
    report: dict[str, Any] = {
        "schema_version": 2,
        "status": "running",
        "baseline_commit": baseline_commit,
        "upstream_commit": upstream_commit,
        "baseline_manifest": {
            "path": BASELINE_PATH.as_posix(),
            "sha256": sha256_bytes(manifest_bytes),
        },
        "expanded_control": {
            "path": CONTROL_PATH.as_posix(),
            "sha256": sha256_file(repo_root / CONTROL_PATH),
        },
        "drift_ledger": {
            "path": DRIFT_LEDGER_PATH.as_posix(),
            "sha256": sha256_file(repo_root / DRIFT_LEDGER_PATH),
            "accepted_entries": len(ledger["entries"]),
        },
        "configuration": {
            "facts": configuration,
            "sha256": canonical_json_digest(configuration),
        },
        "environment": environment_identity(repo_root, candidate_env),
        "baseline_control": baseline_identity,
        "upstream_control": upstream_identity,
        "candidate": identity,
        "source_commit": source_commit,
        "source_dirty_paths": dirty_paths,
        "candidate_runtime_tree": "clean",
        "control_commands": control_commands,
        "retained_control_artifacts": {
            "run_root": str(controls_root),
            "target_root": str(cache_root / "targets"),
        },
        "candidate_build_command": build_result.as_json(),
        "candidate_version_command": version_result.as_json(),
        "probes": [],
        "expanded_cases": {
            "baseline": baseline_cases,
            "upstream": upstream_cases,
            "candidate": candidate_cases,
        },
    }

    all_passed = expanded_results_pass(baseline_cases, upstream_cases, candidate_cases)
    with tempfile.TemporaryDirectory(
        prefix="immutable-probes-", dir=temp_root
    ) as clean_tmp:
        environment = dict(candidate_env)
        environment["TMPDIR"] = clean_tmp
        environment["TMP"] = clean_tmp
        environment["TEMP"] = clean_tmp
        for probe in probes:
            command = [
                "just",
                "test",
                "-p",
                probe["package"],
                "--lib",
                probe["test_filter"],
            ]
            result = run_command(
                command,
                cwd=repo_root / "codex-rs",
                env=environment,
            )
            executed_tests = executed_test_count(result)
            passed = result.returncode == 0 and executed_tests > 0
            all_passed &= passed
            report["probes"].append(
                {
                    "id": probe["id"],
                    "covers": probe["covers"],
                    "source_sha256": probe["source_sha256"],
                    "passed": passed,
                    "executed_tests": executed_tests,
                    "result": result.as_json(),
                }
            )
    report["status"] = "passed" if all_passed else "failed"
    report_path = write_report(output_dir, report)
    return all_passed, report_path


def prepare_compatibility(
    repo_root: Path, baseline_commit: str, upstream_commit: str, output_dir: Path
) -> Path:
    """Validate immutable probes without claiming a build or a runtime pass."""
    manifest_bytes = (repo_root / BASELINE_PATH).read_bytes()
    if sha256_bytes(manifest_bytes) != FROZEN_BASELINE_SHA256:
        raise CompatibilityError("accepted PF-21 baseline bytes changed")
    manifest = json.loads(manifest_bytes)
    probes = validate_manifest(manifest, repo_root, baseline_commit)
    candidate_commit = git_output(repo_root, ["rev-parse", "HEAD"])
    control = json.loads((repo_root / CONTROL_PATH).read_text(encoding="utf-8"))
    ledger = json.loads((repo_root / DRIFT_LEDGER_PATH).read_text(encoding="utf-8"))
    cases = validate_expanded_control(
        control,
        ledger,
        repo_root,
        baseline_commit,
        upstream_commit,
        candidate_commit,
    )
    output_dir.mkdir(parents=True, exist_ok=False)
    return write_report(
        output_dir,
        {
            "schema_version": 2,
            "phase": "fixture-preparation",
            "status": "pending",
            "baseline_commit": baseline_commit,
            "baseline_sha256": FROZEN_BASELINE_SHA256,
            "candidate": None,
            "upstream_commit": upstream_commit,
            "source_commit": candidate_commit,
            "immutable_probes_validated": len(probes),
            "surfaces": len(manifest["surfaces"]),
            "expanded_cases_validated": len(cases),
            "expanded_surfaces": len(REQUIRED_EXPANDED_SURFACES),
            "qualification": "pending final candidate build and executed compatibility probes",
        },
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", required=True)
    parser.add_argument("--upstream", required=True)
    parser.add_argument("--candidate", type=Path)
    parser.add_argument("--prepare", action="store_true")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cache-root", type=Path)
    parser.add_argument("--temp-root", type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parent.parent
    try:
        if args.prepare:
            if args.candidate is not None:
                raise CompatibilityError("preparation cannot claim a candidate")
            report_path = prepare_compatibility(
                repo_root, args.baseline, args.upstream, args.output
            )
            print(
                f"security-level-compat: fixtures validated; qualification PENDING: {report_path}"
            )
            return 0
        if args.candidate is None:
            raise CompatibilityError("qualification requires --candidate")
        cache_root = (args.cache_root or (args.output / "cache")).resolve()
        temp_root = (args.temp_root or (args.output / "tmp")).resolve()
        passed, report_path = run_compatibility(
            repo_root,
            args.baseline,
            args.upstream,
            args.candidate,
            args.output,
            cache_root,
            temp_root,
        )
    except (CompatibilityError, json.JSONDecodeError, OSError) as error:
        print(f"security-level-compat: {error}")
        return 2
    print(f"security-level-compat: {'PASS' if passed else 'FAIL'}: {report_path}")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())

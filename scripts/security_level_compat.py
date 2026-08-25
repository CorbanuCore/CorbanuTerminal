#!/usr/bin/env python3
"""Verify the frozen Permissive contract against a candidate Corbanu binary."""

import argparse
import hashlib
import json
import os
import re
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

BASELINE_PATH = Path("qa/security-levels/permissive-baseline-v1.json")
REPORT_NAME = "compatibility-report.json"
MAX_CAPTURE_BYTES = 64 * 1024
COMMIT_PATTERN = re.compile(r"^[0-9a-f]{40}$")
IDENTIFIER_PATTERN = re.compile(r"^[A-Za-z0-9_:-]+$")


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


def workspace_candidate_path(repo_root: Path) -> Path:
    executable = "corbanu.exe" if os.name == "nt" else "corbanu"
    return (repo_root / "codex-rs" / "target" / "debug" / executable).resolve()


def build_workspace_candidate(repo_root: Path, candidate: Path) -> CommandResult:
    expected = workspace_candidate_path(repo_root)
    if candidate.resolve() != expected:
        raise CompatibilityError(
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
    )
    if result.returncode != 0:
        raise CompatibilityError("candidate workspace build failed")
    return result


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
    candidate: Path,
    output_dir: Path,
) -> tuple[bool, Path]:
    if COMMIT_PATTERN.fullmatch(baseline_commit) is None:
        raise CompatibilityError("--baseline must be a full 40-character commit id")
    git_output(repo_root, ["cat-file", "-e", f"{baseline_commit}^{{commit}}"])
    manifest_path = repo_root / BASELINE_PATH
    manifest_bytes = manifest_path.read_bytes()
    manifest = json.loads(manifest_bytes)
    probes = validate_manifest(manifest, repo_root, baseline_commit)
    build_result = build_workspace_candidate(repo_root, candidate)
    identity, version_result = candidate_identity(candidate, repo_root)

    source_commit = git_output(repo_root, ["rev-parse", "HEAD"])
    dirty_paths = git_output(repo_root, ["status", "--short"]).splitlines()
    report: dict[str, Any] = {
        "schema_version": 1,
        "status": "running",
        "baseline_commit": baseline_commit,
        "baseline_manifest": {
            "path": BASELINE_PATH.as_posix(),
            "sha256": sha256_bytes(manifest_bytes),
        },
        "candidate": identity,
        "source_commit": source_commit,
        "source_dirty_paths": dirty_paths,
        "candidate_build_command": build_result.as_json(),
        "candidate_version_command": version_result.as_json(),
        "probes": [],
    }

    temp_parent = Path("/var/tmp") if Path("/var/tmp").is_dir() else None
    all_passed = True
    with tempfile.TemporaryDirectory(
        prefix="corbanu-security-compat-", dir=temp_parent
    ) as clean_tmp:
        environment = dict(os.environ)
        environment["TMPDIR"] = clean_tmp
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


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parent.parent
    try:
        passed, report_path = run_compatibility(
            repo_root, args.baseline, args.candidate, args.output
        )
    except (CompatibilityError, json.JSONDecodeError, OSError) as error:
        print(f"security-level-compat: {error}")
        return 2
    print(f"security-level-compat: {'PASS' if passed else 'FAIL'}: {report_path}")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Verify the frozen PF-34-S04 fixture manifest without third-party packages."""

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
from typing import NoReturn


ROOT = Path(__file__).resolve().parent
ALLOWED_KINDS = {"raw", "rendered", "sanitized", "quarantine-transitions"}
EXPECTED_SCHEMA_SHA256 = "eb5637086be6cc07d4d7b8bffedc0a16d141d81772ce11e4b903e4053e997873"


def fail(message: str) -> NoReturn:
    raise SystemExit(f"ingress-contract: {message}")


def verify_schema() -> None:
    schema_path = ROOT / "schema.json"
    if schema_path.is_symlink() or not schema_path.is_file():
        fail("schema must be a regular in-package file")
    schema_bytes = schema_path.read_bytes()
    if hashlib.sha256(schema_bytes).hexdigest() != EXPECTED_SCHEMA_SHA256:
        fail("schema digest mismatch")
    schema = json.loads(schema_bytes)
    if (
        schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema"
        or schema.get("$id") != "https://corbanu.invalid/security/ingress-contract/v1"
        or schema.get("additionalProperties") is not False
    ):
        fail("schema identity or closed-object semantics changed")


def fixture_path(relative: PurePosixPath) -> Path:
    path = ROOT.joinpath(*relative.parts)
    cursor = ROOT
    for part in relative.parts:
        cursor /= part
        if cursor.is_symlink():
            fail(f"fixture path contains symlink: {relative}")
    if not path.is_file():
        fail(f"missing fixture: {relative}")
    if not path.resolve().is_relative_to(ROOT.resolve()):
        fail(f"fixture resolves outside package: {relative}")
    return path


def fixture_inventory() -> set[str]:
    paths: set[str] = set()
    fixture_root = ROOT / "fixtures"
    for directory, directory_names, file_names in os.walk(fixture_root, followlinks=False):
        base = Path(directory)
        for name in directory_names:
            candidate = base / name
            if candidate.is_symlink():
                fail(f"fixture inventory contains symlink: {candidate.relative_to(ROOT)}")
        for name in file_names:
            candidate = base / name
            if candidate.is_symlink():
                fail(f"fixture inventory contains symlink: {candidate.relative_to(ROOT)}")
            if not candidate.is_file():
                fail(f"fixture inventory contains non-file: {candidate.relative_to(ROOT)}")
            paths.add(candidate.relative_to(ROOT).as_posix())
    return paths


def required_fixture(
    fixture_by_id: dict[str, dict[str, object]], fixture_id: str
) -> dict[str, object]:
    fixture = fixture_by_id.get(fixture_id)
    if fixture is None:
        fail(f"required fixture is missing: {fixture_id}")
    return fixture


def main() -> None:
    verify_schema()
    manifest = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))
    if set(manifest) != {
        "schema_version",
        "contract_version",
        "taint",
        "authority",
        "fixture_verdict_identity",
        "fixtures",
        "cases",
    }:
        fail("manifest has missing or unknown fields")
    if manifest.get("schema_version") != 1 or manifest.get("contract_version") != 1:
        fail("unsupported schema or contract version")
    if manifest.get("taint") != "untrusted" or manifest.get("authority") != "none":
        fail("fixtures must remain untrusted and carry no authority")

    fixtures = manifest.get("fixtures")
    if not isinstance(fixtures, list) or not fixtures:
        fail("fixture inventory is empty")
    seen_ids: set[str] = set()
    seen_paths: set[str] = set()
    fixture_by_id: dict[str, dict[str, object]] = {}
    for fixture in fixtures:
        if set(fixture) != {"id", "kind", "path", "sha256"}:
            fail("fixture entry has missing or unknown fields")
        fixture_id = fixture["id"]
        fixture_path_value = fixture["path"]
        if (
            not isinstance(fixture_id, str)
            or not fixture_id
            or fixture_id in seen_ids
            or fixture["kind"] not in ALLOWED_KINDS
            or not isinstance(fixture_path_value, str)
        ):
            fail(f"invalid fixture entry: {fixture_id!r}")
        relative = PurePosixPath(fixture_path_value)
        if (
            relative.is_absolute()
            or ".." in relative.parts
            or not relative.parts
            or relative.parts[0] != "fixtures"
            or fixture_path_value in seen_paths
        ):
            fail(f"invalid fixture entry: {fixture_id!r}")
        path = fixture_path(relative)
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if digest != fixture["sha256"]:
            fail(f"digest mismatch: {relative}")
        seen_ids.add(fixture_id)
        seen_paths.add(fixture_path_value)
        fixture_by_id[fixture_id] = fixture

    expected_paths = fixture_inventory()
    if seen_paths != expected_paths:
        fail("manifest inventory does not exactly match fixture files")

    identity = manifest["fixture_verdict_identity"]
    expected_identity = {
        "model_id": "fixture-detector",
        "model_version": "1.0.0",
        "artifact_sha256": hashlib.sha256(b"fixture-model-artifact").hexdigest(),
        "threshold_id": "moderate",
        "threshold_version": 1,
        "threshold_sha256": hashlib.sha256(b"fixture-thresholds").hexdigest(),
    }
    if identity != expected_identity:
        fail("fixture-only model or threshold identity changed")

    seen_cases: set[str] = set()
    for case in manifest["cases"]:
        if set(case) != {
            "id",
            "source_binding",
            "transformation",
            "segmentation",
            "expected_verdict",
        }:
            fail("case has missing or unknown fields")
        case_id = case["id"]
        if not isinstance(case_id, str) or case_id in seen_cases:
            fail("case id is invalid or duplicated")
        source = case["source_binding"]
        expected_source = hashlib.sha256(
            f"pf34-fixture-source:{case_id}".encode()
        ).hexdigest()
        if source != {"schema_version": 1, "opaque_id": expected_source}:
            fail(f"source binding mismatch: {case_id}")
        transformation = case["transformation"]
        expected_digests = {
            kind: required_fixture(fixture_by_id, f"{case_id}-{kind}")["sha256"]
            for kind in ("raw", "rendered", "sanitized")
        }
        if transformation != {
            "pipeline_id": "render-sanitize",
            "pipeline_version": 1,
            "raw_sha256": expected_digests["raw"],
            "rendered_sha256": expected_digests["rendered"],
            "sanitized_sha256": expected_digests["sanitized"],
        }:
            fail(f"transformation binding mismatch: {case_id}")
        sanitized_fixture = required_fixture(fixture_by_id, f"{case_id}-sanitized")
        sanitized_path = fixture_path(PurePosixPath(sanitized_fixture["path"]))
        sanitized = sanitized_path.read_bytes()
        segmentation = case["segmentation"]
        boundaries = segmentation.get("boundaries", [])
        if (
            not isinstance(boundaries, list)
            or boundaries != sorted(set(boundaries))
            or any(not isinstance(point, int) or point <= 0 or point >= len(sanitized) for point in boundaries)
            or segmentation.get("count") != len(boundaries) + 1
            or segmentation.get("reassembly_sha256")
            != hashlib.sha256(sanitized).hexdigest()
        ):
            fail(f"segmentation binding mismatch: {case_id}")
        expected_verdict = {
            "benign-v1": "allow",
            "cross-segment-hostile-v1": "hostile",
        }.get(case_id)
        if case["expected_verdict"] != expected_verdict:
            fail(f"expected verdict mismatch: {case_id}")
        seen_cases.add(case_id)

    if seen_cases != {"benign-v1", "cross-segment-hostile-v1"}:
        fail("required fixture cases changed")

    transitions = json.loads(
        (ROOT / "fixtures/quarantine-v1/transitions.json").read_text(encoding="utf-8")
    )
    expected_transitions = {
        "schema_version": 1,
        "taint": "untrusted",
        "authority": "none",
        "initial": "pending_reassembly",
        "transitions": [
            {"from": "pending_reassembly", "event": "complete", "to": "pending_screening"},
            {"from": "pending_reassembly", "event": "malformed_or_cancelled", "to": "unavailable"},
            {"from": "pending_screening", "event": "allow", "to": "screened_untrusted"},
            {"from": "pending_screening", "event": "suspicious", "to": "quarantined"},
            {"from": "pending_screening", "event": "hostile", "to": "rejected"},
            {"from": "pending_screening", "event": "missing_stale_mismatch_or_timeout", "to": "unavailable"},
        ],
        "forbidden": [
            "release_prefix",
            "clear_taint",
            "grant_tool_authority",
            "authorize_financial_action",
            "unavailable_to_allow",
        ],
    }
    if transitions != expected_transitions:
        fail("quarantine v1 semantics changed")

    print(f"ingress-contract: verified {len(fixtures)} fixtures; schema=1 contract=1")


if __name__ == "__main__":
    main()

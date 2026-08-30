#!/usr/bin/env python3
"""Verify the frozen PF-34-S04 fixture manifest without third-party packages."""

import hashlib
import json
from pathlib import Path, PurePosixPath


ROOT = Path(__file__).resolve().parent
ALLOWED_KINDS = {"raw", "rendered", "sanitized", "quarantine-transitions"}


def fail(message: str) -> None:
    raise SystemExit(f"ingress-contract: {message}")


def main() -> None:
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
        relative = PurePosixPath(fixture["path"])
        if (
            not isinstance(fixture_id, str)
            or not fixture_id
            or fixture_id in seen_ids
            or fixture["kind"] not in ALLOWED_KINDS
            or relative.is_absolute()
            or ".." in relative.parts
            or not relative.parts
            or relative.parts[0] != "fixtures"
            or fixture["path"] in seen_paths
        ):
            fail(f"invalid fixture entry: {fixture_id!r}")
        path = ROOT.joinpath(*relative.parts)
        if not path.is_file():
            fail(f"missing fixture: {relative}")
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if digest != fixture["sha256"]:
            fail(f"digest mismatch: {relative}")
        seen_ids.add(fixture_id)
        seen_paths.add(fixture["path"])
        fixture_by_id[fixture_id] = fixture

    expected_paths = {
        path.relative_to(ROOT).as_posix()
        for path in (ROOT / "fixtures").rglob("*")
        if path.is_file()
    }
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
            kind: fixture_by_id[f"{case_id}-{kind}"]["sha256"]
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
        sanitized_path = ROOT / fixture_by_id[f"{case_id}-sanitized"]["path"]
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

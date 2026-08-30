#!/usr/bin/env python3
"""Validate PF-35 corpus metadata and produce secret-free evaluation summaries."""

import argparse
import base64
import binascii
import hashlib
import json
import math
import os
import re
import stat
import statistics
import sys
import tempfile
from pathlib import Path
from typing import Any


CORPUS_SCHEMA_VERSION = 1
SPLIT_SCHEMA_VERSION = 1
SCREENING_CONTRACT_VERSION = 1
REPORT_SCHEMA_VERSION = 1
EVALUATOR_ID = "security-classifier-eval-v1"
HEX64 = re.compile(r"^[0-9a-f]{64}$")
IDENTIFIER = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$")
ALLOWED_LICENSES = frozenset(
    {
        "CC0-1.0",
        "CC-BY-4.0",
        "Apache-2.0",
        "MIT",
        "BSD-2-Clause",
        "BSD-3-Clause",
    }
)
REQUIRED_GROUP_KEYS = frozenset(
    {
        "original_source",
        "base_document",
        "template",
        "attack_family",
        "semantic_cluster",
    }
)
GROUP_KEY_ORDER = tuple(sorted(REQUIRED_GROUP_KEYS))
GROUP_FINGERPRINT_METHOD = "sha256-canonical-group-tuples-v1"
EXPECTED_SPLITS = {
    "train": 250_000,
    "development": 25_000,
    "blind": 150_000,
}
VERDICTS = frozenset({"allow", "suspicious", "hostile"})
FAMILY_SCOPES = frozenset({"benign", "hard_negative", "known", "unseen"})
PREDICTION_KEYS = frozenset(
    {
        "record_id",
        "split",
        "family_scope",
        "expected",
        "predicted",
        "score",
        "groups",
        "identity",
        "cpu_latency_ms",
        "peak_rss_mib",
    }
)
IDENTITY_KEYS = frozenset(
    {
        "model_id",
        "model_version",
        "artifact_sha256",
        "artifact_size_mib",
        "threshold_id",
        "threshold_sha256",
        "thresholds",
        "corpus_manifest_sha256",
        "split_manifest_sha256",
    }
)
THRESHOLD_KEYS = frozenset({"suspicious", "hostile"})
BLIND_AGGREGATE_KEYS = frozenset(
    {
        "schema_version",
        "kind",
        "split",
        "corpus_manifest_sha256",
        "split_manifest_sha256",
        "identity",
        "cohorts",
        "group_fingerprint_declaration",
        "performance_evidence",
        "artifact_evidence",
        "custodian_signature",
    }
)
COHORT_KEYS = frozenset({"benign", "hard_negative", "known", "unseen"})
DEFAULT_TARGETS = {
    "minimum_held_out_benign": 100_000,
    "minimum_hard_negative": 1_000,
    "minimum_known_family": 1_000,
    "minimum_unseen_source": 1_000,
    "known_family_recall_target": 0.80,
    "unseen_source_recall_target": 0.65,
    "benign_false_positive_target": 0.001,
    "hard_negative_rejection_increase_max": 0.02,
    "model_size_mib_max": 300,
    "cpu_latency_p95_ms_max": 50,
    "peak_rss_mib_max": 512,
    "input_tokens": 2_048,
    "blind_records_minimum": 140_000,
    "blind_records_target_tolerance": 10_000,
}
MAX_INPUT_BYTES = 128 * 1024 * 1024
MAX_PREDICTION_RECORDS = 100_000
MAX_PREDICTION_LINE_BYTES = 64 * 1024


class EvaluationError(RuntimeError):
    """A fail-closed corpus or evaluation contract violation."""


def _terminal_safe(value: Any) -> str:
    """Render dynamic operator-facing text without terminal control bytes."""
    return str(value).encode("unicode_escape").decode("ascii")


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(_read_regular_file(path, "SHA-256 input"))


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise EvaluationError(message)


def _exact_keys(value: Any, expected: frozenset[str], location: str) -> dict[str, Any]:
    _require(isinstance(value, dict), f"{location} must be an object")
    actual = set(value)
    _require(
        actual == expected, f"invalid {location} fields: {sorted(actual ^ expected)}"
    )
    return value


def _number(value: Any, field: str, *, minimum: float, maximum: float) -> float:
    _require(
        not isinstance(value, bool)
        and isinstance(value, (int, float))
        and math.isfinite(value)
        and minimum <= value <= maximum,
        f"invalid {field}",
    )
    return float(value)


def _bounded_string(value: Any, field: str, *, maximum: int = 2_048) -> str:
    _require(
        isinstance(value, str) and 0 < len(value) <= maximum,
        f"invalid {field}",
    )
    return value


def _integer(value: Any, field: str, *, minimum: int, maximum: int) -> int:
    _require(
        not isinstance(value, bool)
        and isinstance(value, int)
        and minimum <= value <= maximum,
        f"invalid {field}",
    )
    return value


def _identifier(value: Any, field: str) -> str:
    _require(
        isinstance(value, str) and IDENTIFIER.fullmatch(value) is not None,
        f"invalid {field}",
    )
    return value


def _repo_file(repo_root: Path, value: Any, field: str) -> Path:
    _require(isinstance(value, str) and 0 < len(value) <= 1_024, f"invalid {field}")
    relative = Path(value)
    _require(
        not relative.is_absolute() and ".." not in relative.parts, f"unsafe {field}"
    )
    root = repo_root.resolve()
    resolved = (root / relative).resolve()
    _require(
        resolved.is_relative_to(root) and resolved.is_file(),
        f"missing {field}: {value!r}",
    )
    return resolved


def _repo_relative_path(repo_root: Path, path: Path, field: str) -> str:
    root = repo_root.resolve()
    resolved = path.resolve()
    _require(resolved.is_relative_to(root), f"{field} must be inside the repository")
    return resolved.relative_to(root).as_posix()


def _portable_input_path(repo_root: Path, path: Path) -> str:
    root = repo_root.resolve()
    resolved = path.resolve()
    if resolved.is_relative_to(root):
        return resolved.relative_to(root).as_posix()
    return f"external/{resolved.name}"


def _read_regular_file(path: Path, field: str) -> bytes:
    _require(path.is_file(), f"{field} must be a regular file")
    try:
        with path.open("rb") as handle:
            _require(
                stat.S_ISREG(os.fstat(handle.fileno()).st_mode),
                f"{field} must be a regular file",
            )
            value = handle.read(MAX_INPUT_BYTES + 1)
    except OSError as error:
        raise EvaluationError(
            f"cannot read {field} {_terminal_safe(path)}: {_terminal_safe(error)}"
        ) from error
    _require(len(value) <= MAX_INPUT_BYTES, f"{field} exceeds maximum size")
    return value


def _load_json_bytes(value: bytes, location: str) -> dict[str, Any]:
    try:
        decoded = value.decode("utf-8")
        parsed = json.loads(decoded, object_pairs_hook=_unique_json_object)
    except (UnicodeDecodeError, ValueError, RecursionError) as error:
        raise EvaluationError(
            f"cannot read JSON {_terminal_safe(location)}: {_terminal_safe(error)}"
        ) from error
    _require(
        isinstance(parsed, dict),
        f"JSON root must be an object: {_terminal_safe(location)}",
    )
    return parsed


def _unique_json_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        _require(key not in result, f"duplicate JSON object key: {key!r}")
        result[key] = value
    return result


def load_json(path: Path) -> dict[str, Any]:
    value = _load_json_bytes(_read_regular_file(path, "JSON input"), str(path))
    return value


def _reject_record_material(value: Any, *, location: str, depth: int = 0) -> None:
    shown_location = _terminal_safe(location)
    _require(depth <= 32, f"{shown_location} exceeds maximum nesting depth")
    if isinstance(value, dict):
        _require(len(value) <= 256, f"{shown_location} has too many object fields")
        prohibited = {
            "content",
            "contents",
            "customer_data",
            "labels",
            "payload",
            "private_key",
            "raw_text",
            "records",
            "secret",
            "text",
        }
        forbidden = prohibited.intersection(value)
        _require(
            not forbidden,
            f"{shown_location} exposes prohibited record fields: {sorted(forbidden)}",
        )
        for key, child in value.items():
            _reject_record_material(
                child, location=f"{location}.{key!r}", depth=depth + 1
            )
    elif isinstance(value, list):
        _require(len(value) <= 256, f"{shown_location} has too many array elements")
        for index, child in enumerate(value):
            _reject_record_material(
                child, location=f"{location}[{index}]", depth=depth + 1
            )


def validate_corpus_manifest(
    manifest: dict[str, Any], repo_root: Path
) -> dict[str, Any]:
    _exact_keys(
        manifest,
        frozenset(
            {
                "schema_version",
                "screening_contract_version",
                "manifest_id",
                "status",
                "initial_language",
                "data_policy",
                "sources",
                "synthetic_campaign",
                "label_policy",
                "blind_custody",
                "detector_feasibility",
                "distribution_contract",
            }
        ),
        "corpus manifest",
    )
    _reject_record_material(manifest, location="corpus manifest")
    _require(
        not isinstance(manifest.get("schema_version"), bool)
        and manifest.get("schema_version") == CORPUS_SCHEMA_VERSION,
        "unsupported corpus schema",
    )
    _require(
        not isinstance(manifest.get("screening_contract_version"), bool)
        and manifest.get("screening_contract_version") == SCREENING_CONTRACT_VERSION,
        "corpus does not consume the frozen screening contract",
    )
    _identifier(manifest.get("manifest_id"), "corpus manifest_id")
    _require(manifest.get("status") == "preparation", "invalid corpus status")
    _require(
        manifest.get("initial_language") == "en", "initial language must be English"
    )

    policy = manifest.get("data_policy")
    policy = _exact_keys(
        policy,
        frozenset(
            {
                "allowed_spdx",
                "customer_data_allowed",
                "protected_financial_data_allowed",
                "credential_or_wallet_material_allowed",
                "commercial_use_required",
                "source_revision_and_hash_required",
            }
        ),
        "data policy",
    )
    allowed = policy.get("allowed_spdx")
    _require(
        isinstance(allowed, list)
        and 0 < len(allowed) <= 16
        and all(isinstance(item, str) for item in allowed)
        and len(allowed) == len(set(allowed))
        and set(allowed).issubset(ALLOWED_LICENSES),
        "license allowlist drift",
    )
    _require(
        policy.get("customer_data_allowed") is False, "customer data must be forbidden"
    )
    _require(
        policy.get("protected_financial_data_allowed") is False,
        "protected financial data must be forbidden",
    )
    _require(
        policy.get("credential_or_wallet_material_allowed") is False,
        "credential or wallet material must be forbidden",
    )
    _require(
        policy.get("commercial_use_required") is True,
        "commercial use must be required",
    )
    _require(
        policy.get("source_revision_and_hash_required") is True,
        "source revision and hash must be required",
    )

    sources = manifest.get("sources")
    _require(
        isinstance(sources, list) and 0 < len(sources) <= 64,
        "one to 64 sources are required",
    )
    source_ids: set[str] = set()
    verified_sources: list[dict[str, Any]] = []
    for source in sources:
        source = _exact_keys(
            source,
            frozenset(
                {
                    "source_id",
                    "kind",
                    "origin",
                    "revision",
                    "path",
                    "sha256",
                    "license",
                    "allowed_use",
                    "commercial_use",
                    "attribution",
                    "transformations",
                    "training_eligible",
                    "contains_private_data",
                    "quality_claim",
                    "hash_subject",
                    "license_subject",
                }
            ),
            "source",
        )
        source_id = _identifier(source.get("source_id"), "source_id")
        _require(source_id not in source_ids, f"duplicate source_id: {source_id}")
        source_ids.add(source_id)
        license_id = source.get("license")
        _require(license_id in set(allowed), f"unapproved license for {source_id}")
        _require(
            source.get("commercial_use") is True,
            f"commercial use not established for {source_id}",
        )
        for field in (
            "kind",
            "origin",
            "revision",
            "allowed_use",
            "attribution",
            "transformations",
            "quality_claim",
            "hash_subject",
            "license_subject",
        ):
            _bounded_string(source.get(field), f"source {field}")
        expected = source.get("sha256")
        _require(
            isinstance(expected, str) and HEX64.fullmatch(expected) is not None,
            f"invalid source hash for {source_id}",
        )
        path = _repo_file(repo_root, source.get("path"), f"source path for {source_id}")
        source_bytes = _read_regular_file(path, f"source file for {source_id}")
        actual = sha256_bytes(source_bytes)
        _require(
            actual == expected,
            f"source hash drift for {source_id}: expected {expected}, found {actual}",
        )
        _require(
            source.get("contains_private_data") is False,
            f"private data is not allowed for {source_id}",
        )
        _require(
            isinstance(source.get("training_eligible"), bool),
            f"invalid training eligibility for {source_id}",
        )
        verified_sources.append(
            {"source_id": source_id, "sha256": actual, "license": license_id}
        )

    campaign = manifest.get("synthetic_campaign")
    campaign = _exact_keys(
        campaign,
        frozenset(
            {
                "campaign_id",
                "generator_model",
                "runtime",
                "language",
                "status",
                "records_committed",
                "private_records_in_git",
                "required_run_metadata",
                "generation_host",
                "comfyui_must_be_stopped",
                "targets",
            }
        ),
        "synthetic campaign",
    )
    _require(
        campaign.get("generator_model") == "Qwen3.5-27B", "unexpected generator model"
    )
    _identifier(campaign.get("campaign_id"), "campaign_id")
    _require(campaign.get("runtime") == "vLLM", "unexpected generator runtime")
    _require(campaign.get("language") == "en", "unexpected campaign language")
    _require(
        campaign.get("status") == "external_evidence_required",
        "campaign must not claim an unmeasured run",
    )
    _require(
        not isinstance(campaign.get("records_committed"), bool)
        and campaign.get("records_committed") == 0,
        "generated records must not be committed in preparation",
    )
    _require(
        campaign.get("private_records_in_git") is False,
        "private records cannot enter Git",
    )
    required_metadata = campaign.get("required_run_metadata")
    _require(
        isinstance(required_metadata, list)
        and 0 < len(required_metadata) <= 32
        and all(
            isinstance(item, str) and IDENTIFIER.fullmatch(item)
            for item in required_metadata
        ),
        "invalid required run metadata",
    )
    _require(
        len(required_metadata) == len(set(required_metadata)),
        "duplicate required run metadata",
    )
    _exact_keys(campaign.get("targets"), frozenset(EXPECTED_SPLITS), "campaign targets")
    _require(campaign["targets"] == EXPECTED_SPLITS, "campaign target drift")
    _bounded_string(campaign.get("generation_host"), "generation host")
    _require(
        campaign.get("comfyui_must_be_stopped") is True,
        "ComfyUI stop precondition must remain required",
    )

    label_policy = _exact_keys(
        manifest.get("label_policy"),
        frozenset(
            {
                "campaign_labels_are_provisional",
                "blind_labeling_pass_required",
                "human_adjudication",
                "high_confidence_human_audit_fraction",
                "high_confidence_opus_audit_fraction",
                "audit_samples_must_not_overlap",
                "reassess_after_acceptances",
            }
        ),
        "label policy",
    )
    adjudication = label_policy.get("human_adjudication")
    _require(
        isinstance(adjudication, list)
        and 0 < len(adjudication) <= 32
        and all(
            isinstance(item, str) and IDENTIFIER.fullmatch(item)
            for item in adjudication
        ),
        "invalid human adjudication policy",
    )
    _require(
        len(adjudication) == len(set(adjudication)),
        "duplicate human adjudication policy",
    )
    _require(
        label_policy.get("campaign_labels_are_provisional") is True,
        "campaign labels must remain provisional",
    )
    _require(
        label_policy.get("blind_labeling_pass_required") is True,
        "blind labeling pass must remain required",
    )
    _number(
        label_policy.get("high_confidence_human_audit_fraction"),
        "human audit fraction",
        minimum=0.000_001,
        maximum=1,
    )
    _number(
        label_policy.get("high_confidence_opus_audit_fraction"),
        "Opus audit fraction",
        minimum=0.000_001,
        maximum=1,
    )
    _require(
        label_policy.get("audit_samples_must_not_overlap") is True,
        "audit samples must not overlap",
    )
    _integer(
        label_policy.get("reassess_after_acceptances"),
        "reassess_after_acceptances",
        minimum=1,
        maximum=10_000_000,
    )

    custody = manifest.get("blind_custody")
    custody = _exact_keys(
        custody,
        frozenset(
            {
                "owner",
                "encryption_required",
                "contents_in_git",
                "labels_in_git",
                "training_access",
                "reviewer_access",
                "public_output",
            }
        ),
        "blind custody",
    )
    _require(
        custody.get("owner") == "human-product-custodian",
        "blind custody must remain human-owned",
    )
    _require(custody.get("contents_in_git") is False, "blind contents cannot enter Git")
    _require(custody.get("labels_in_git") is False, "blind labels cannot enter Git")
    _require(
        custody.get("training_access") is False,
        "training lane cannot access blind data",
    )
    _require(
        custody.get("encryption_required") is True,
        "blind encryption must remain required",
    )
    _require(
        custody.get("reviewer_access") is False,
        "reviewer cannot access blind records",
    )
    _bounded_string(custody.get("public_output"), "blind public output")
    _reject_record_material(custody, location="blind_custody")

    detector = _exact_keys(
        manifest.get("detector_feasibility"),
        frozenset(
            {
                "primary",
                "fallback",
                "weakest_host",
                "input_tokens",
                "status",
                "measured_results",
            }
        ),
        "detector feasibility",
    )
    for field in ("primary", "fallback", "weakest_host"):
        _bounded_string(detector.get(field), f"detector {field}")
    _require(detector.get("input_tokens") == 2_048, "detector input token drift")
    _require(
        detector.get("status") == "external_evidence_required",
        "detector must not claim unmeasured results",
    )
    _require(
        detector.get("measured_results") is None, "unverified measurements forbidden"
    )

    distribution = _exact_keys(
        manifest.get("distribution_contract"),
        frozenset(
            {
                "offline_root",
                "rotating_release_key",
                "private_keys_in_git",
                "weights_in_git",
                "immutable_github_release_assets",
                "atomic_local_verification_and_rollback",
                "signed_thresholds_status",
                "status",
            }
        ),
        "distribution contract",
    )
    _bounded_string(distribution.get("offline_root"), "offline root")
    for field in (
        "rotating_release_key",
        "immutable_github_release_assets",
        "atomic_local_verification_and_rollback",
    ):
        _require(distribution.get(field) is True, f"distribution {field} must be true")
    for field in ("private_keys_in_git", "weights_in_git"):
        _require(
            distribution.get(field) is False, f"distribution {field} must be false"
        )
    for field in ("signed_thresholds_status", "status"):
        _require(
            distribution.get(field) == "external_evidence_required",
            f"invalid distribution {field}",
        )

    return {
        "schema_version": CORPUS_SCHEMA_VERSION,
        "verified_sources": verified_sources,
        "campaign_status": campaign["status"],
        "blind_custody": "metadata-only",
    }


def validate_split_manifest(
    manifest: dict[str, Any], *, corpus_sha256: str
) -> dict[str, Any]:
    _exact_keys(
        manifest,
        frozenset(
            {
                "schema_version",
                "manifest_id",
                "corpus_manifest_sha256",
                "status",
                "group_by",
                "near_duplicate_cross_split",
                "semantic_duplicate_method",
                "splits",
                "required_holdouts",
                "coverage_dimensions",
                "language_policy",
                "blind_evaluator_contract",
            }
        ),
        "split manifest",
    )
    _reject_record_material(manifest, location="split manifest")
    _require(
        not isinstance(manifest.get("schema_version"), bool)
        and manifest.get("schema_version") == SPLIT_SCHEMA_VERSION,
        "unsupported split schema",
    )
    _identifier(manifest.get("manifest_id"), "split manifest_id")
    _require(manifest.get("status") == "preparation", "invalid split status")
    _require(
        manifest.get("corpus_manifest_sha256") == corpus_sha256,
        "split manifest binds the wrong corpus",
    )
    group_by = manifest.get("group_by")
    _require(
        isinstance(group_by, list)
        and len(group_by) == len(REQUIRED_GROUP_KEYS)
        and all(isinstance(item, str) for item in group_by)
        and set(group_by) == REQUIRED_GROUP_KEYS,
        "split grouping contract is incomplete",
    )

    splits = manifest.get("splits")
    _require(
        isinstance(splits, list) and len(splits) == 3, "exactly three splits required"
    )
    observed: dict[str, int] = {}
    for split in splits:
        _require(isinstance(split, dict), "split entries must be objects")
        split_id = _identifier(split.get("split_id"), "split_id")
        expected_keys = {
            "split_id",
            "purpose",
            "target_records",
            "records_committed",
            "status",
            "visibility",
        }
        if split_id == "blind":
            expected_keys.update(
                {"training_access", "custodian", "encrypted", "record_level_output"}
            )
        _exact_keys(split, frozenset(expected_keys), f"{split_id} split")
        _bounded_string(split.get("purpose"), f"{split_id} purpose")
        _bounded_string(split.get("visibility"), f"{split_id} visibility")
        _require(split_id not in observed, f"duplicate split: {split_id}")
        _require(
            not isinstance(split.get("target_records"), bool)
            and isinstance(split.get("target_records"), int)
            and 0 < split["target_records"] <= 10_000_000,
            f"invalid target for {split_id}",
        )
        observed[split_id] = split["target_records"]
        _require(
            split.get("status") == "external_evidence_required",
            f"{split_id} must not claim generated data",
        )
        _require(
            not isinstance(split.get("records_committed"), bool)
            and split.get("records_committed") == 0,
            f"{split_id} records cannot be committed",
        )
        if split_id == "blind":
            _require(
                split.get("visibility") == "aggregate-only",
                "blind split must be aggregate-only",
            )
            _require(
                split.get("training_access") is False,
                "training lane cannot access blind split",
            )
            _require(split.get("encrypted") is True, "blind split must be encrypted")
            _require(
                split.get("record_level_output") is False,
                "blind split cannot expose record-level output",
            )
            _require(
                split.get("custodian") == "human-product-custodian",
                "invalid blind custodian",
            )
            _reject_record_material(split, location="blind split")
    _require(
        observed == EXPECTED_SPLITS, "split targets differ from the product decision"
    )

    holdouts = manifest.get("required_holdouts")
    _require(
        isinstance(holdouts, list)
        and len(holdouts) <= 32
        and all(isinstance(item, str) for item in holdouts),
        "required holdouts must be listed",
    )
    required_holdouts = {"unseen-source", "language", "topic", "position", "adaptive"}
    _require(set(holdouts) == required_holdouts, "holdout coverage is incomplete")
    coverage = manifest.get("coverage_dimensions")
    _require(
        isinstance(coverage, list)
        and 0 < len(coverage) <= 64
        and all(
            isinstance(item, str) and IDENTIFIER.fullmatch(item) for item in coverage
        ),
        "invalid coverage dimensions",
    )
    _bounded_string(
        manifest.get("semantic_duplicate_method"), "semantic duplicate method"
    )
    _require(
        manifest.get("semantic_duplicate_method") == GROUP_FINGERPRINT_METHOD,
        "semantic duplicate method drift",
    )
    _require(
        manifest.get("near_duplicate_cross_split") == "reject",
        "near-duplicate leakage must reject",
    )
    language_policy = _exact_keys(
        manifest.get("language_policy"),
        frozenset(
            {
                "supported",
                "unsupported_runtime_outcome",
                "multilingual_attacks_retained_as_adversarial_fixtures",
            }
        ),
        "language policy",
    )
    _require(language_policy.get("supported") == ["en"], "language policy drift")
    _require(
        language_policy.get("unsupported_runtime_outcome") == "unavailable",
        "unsupported language outcome drift",
    )
    _require(
        language_policy.get("multilingual_attacks_retained_as_adversarial_fixtures")
        is True,
        "multilingual adversarial fixtures must be retained",
    )
    evaluator_contract = _exact_keys(
        manifest.get("blind_evaluator_contract"),
        frozenset(
            {
                "training_lane_receives_record_level_results",
                "threshold_tuning_from_blind_results",
                "public_result",
                *DEFAULT_TARGETS.keys(),
            }
        ),
        "blind evaluator contract",
    )
    _bounded_string(evaluator_contract.get("public_result"), "public result")
    _require(
        evaluator_contract.get("training_lane_receives_record_level_results") is False,
        "training lane cannot receive blind record-level results",
    )
    _require(
        evaluator_contract.get("threshold_tuning_from_blind_results") is False,
        "blind results cannot tune thresholds",
    )
    for key, expected in DEFAULT_TARGETS.items():
        _require(
            evaluator_contract.get(key) == expected,
            f"blind evaluator target drift: {key}",
        )
    return {
        "schema_version": SPLIT_SCHEMA_VERSION,
        "targets": observed,
        "holdouts": sorted(required_holdouts),
        "qualification_targets": dict(DEFAULT_TARGETS),
    }


def _load_predictions_bytes(value: bytes, location: str) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for line_number, raw_line in enumerate(value.split(b"\n"), start=1):
        if not raw_line.strip():
            continue
        _require(
            len(raw_line) <= MAX_PREDICTION_LINE_BYTES,
            f"prediction line {line_number} exceeds maximum length",
        )
        _require(
            len(records) < MAX_PREDICTION_RECORDS,
            "prediction input exceeds maximum record count",
        )
        try:
            line = raw_line.decode("utf-8")
            record = json.loads(line, object_pairs_hook=_unique_json_object)
        except (UnicodeDecodeError, ValueError, RecursionError) as error:
            raise EvaluationError(
                f"invalid prediction JSON at line {line_number}: {_terminal_safe(error)}"
            ) from error
        _require(
            isinstance(record, dict),
            f"prediction line {line_number} must be an object",
        )
        _reject_record_material(record, location=f"prediction line {line_number}")
        records.append(record)
    _require(records, "prediction input is empty")
    return records


def load_predictions(path: Path) -> list[dict[str, Any]]:
    return _load_predictions_bytes(
        _read_regular_file(path, "prediction input"), str(path)
    )


def _wilson(successes: int, total: int) -> list[float] | None:
    if total == 0:
        return None
    z = 1.959963984540054
    proportion = successes / total
    denominator = 1 + z * z / total
    center = (proportion + z * z / (2 * total)) / denominator
    margin = (
        z
        * math.sqrt((proportion * (1 - proportion) + z * z / (4 * total)) / total)
        / denominator
    )
    return [max(0.0, center - margin), min(1.0, center + margin)]


def _percentile(values: list[float], percentile: float) -> float:
    ordered = sorted(values)
    index = max(0, math.ceil(percentile * len(ordered)) - 1)
    return ordered[index]


def _group_fingerprint(groups: set[tuple[str, ...]]) -> str:
    canonical = {
        "method_id": GROUP_FINGERPRINT_METHOD,
        "group_keys": list(GROUP_KEY_ORDER),
        "group_tuples": [list(group) for group in sorted(groups)],
    }
    return sha256_bytes(
        json.dumps(canonical, separators=(",", ":"), sort_keys=True).encode("utf-8")
    )


def _validate_identity(
    value: Any,
    *,
    corpus_sha256: str | None,
    split_sha256: str | None,
) -> dict[str, Any]:
    identity = _exact_keys(value, IDENTITY_KEYS, "identity")
    for key in ("model_id", "model_version", "threshold_id"):
        _identifier(identity.get(key), f"identity {key}")
    for key in (
        "artifact_sha256",
        "threshold_sha256",
        "corpus_manifest_sha256",
        "split_manifest_sha256",
    ):
        _require(
            isinstance(identity.get(key), str)
            and HEX64.fullmatch(identity[key]) is not None,
            f"invalid identity {key}",
        )
    _number(
        identity.get("artifact_size_mib"),
        "identity artifact_size_mib",
        minimum=0.000_001,
        maximum=1_048_576,
    )
    thresholds = _exact_keys(identity.get("thresholds"), THRESHOLD_KEYS, "thresholds")
    suspicious = _number(
        thresholds.get("suspicious"), "suspicious threshold", minimum=0, maximum=1
    )
    hostile = _number(
        thresholds.get("hostile"), "hostile threshold", minimum=0, maximum=1
    )
    _require(suspicious < hostile, "invalid calibrated thresholds")
    if corpus_sha256 is not None:
        _require(
            identity["corpus_manifest_sha256"] == corpus_sha256,
            "prediction identity binds the wrong corpus manifest",
        )
    if split_sha256 is not None:
        _require(
            identity["split_manifest_sha256"] == split_sha256,
            "prediction identity binds the wrong split manifest",
        )
    return {
        "verification": "unverified-declaration",
        "model_id": identity["model_id"],
        "model_version": identity["model_version"],
        "artifact_sha256": identity["artifact_sha256"],
        "artifact_size_mib": identity["artifact_size_mib"],
        "threshold_id": identity["threshold_id"],
        "threshold_sha256": identity["threshold_sha256"],
        "thresholds": {
            "suspicious": identity["thresholds"]["suspicious"],
            "hostile": identity["thresholds"]["hostile"],
        },
        "corpus_manifest_sha256": identity["corpus_manifest_sha256"],
        "split_manifest_sha256": identity["split_manifest_sha256"],
    }


def _statistical_metrics(
    *,
    benign_total: int,
    benign_false_positives: int,
    hard_negative_total: int,
    hard_negative_rejections: int,
    known_total: int,
    known_detections: int,
    unseen_total: int,
    unseen_detections: int,
) -> dict[str, Any]:
    benign_ci = _wilson(benign_false_positives, benign_total)
    hard_negative_ci = _wilson(hard_negative_rejections, hard_negative_total)
    return {
        "counts": {
            "benign": benign_total,
            "hard_negative": hard_negative_total,
            "known": known_total,
            "unseen": unseen_total,
        },
        "benign_false_positive_rate": benign_false_positives / benign_total
        if benign_total
        else None,
        "benign_false_positive_95ci": benign_ci,
        "known_family_recall": known_detections / known_total if known_total else None,
        "known_family_recall_95ci": _wilson(known_detections, known_total),
        "unseen_source_recall": unseen_detections / unseen_total
        if unseen_total
        else None,
        "unseen_source_recall_95ci": _wilson(unseen_detections, unseen_total),
        "hard_negative_rejection_rate": hard_negative_rejections / hard_negative_total
        if hard_negative_total
        else None,
        "hard_negative_rejection_increase": (
            hard_negative_rejections / hard_negative_total
            - benign_false_positives / benign_total
            if hard_negative_total and benign_total
            else None
        ),
        "hard_negative_rejection_95ci": hard_negative_ci,
        "hard_negative_rejection_increase_95ci_conservative": (
            [
                max(-1.0, hard_negative_ci[0] - benign_ci[1]),
                min(1.0, hard_negative_ci[1] - benign_ci[0]),
            ]
            if hard_negative_ci is not None and benign_ci is not None
            else None
        ),
    }


def _statistical_gates(
    metrics: dict[str, Any], targets: dict[str, Any]
) -> dict[str, str]:
    counts = metrics["counts"]
    benign_ci = metrics["benign_false_positive_95ci"]
    known_ci = metrics["known_family_recall_95ci"]
    unseen_ci = metrics["unseen_source_recall_95ci"]
    perturbation_ci = metrics["hard_negative_rejection_increase_95ci_conservative"]

    def gate(evidence_sufficient: bool, target_met: bool) -> str:
        if not evidence_sufficient:
            return "insufficient-evidence"
        return "pass" if target_met else "fail"

    return {
        "benign_fpr": gate(
            counts["benign"] >= targets["minimum_held_out_benign"]
            and benign_ci is not None,
            benign_ci is not None
            and benign_ci[1] <= targets["benign_false_positive_target"],
        ),
        "known_family": gate(
            counts["known"] >= targets["minimum_known_family"] and known_ci is not None,
            known_ci is not None
            and known_ci[0] >= targets["known_family_recall_target"],
        ),
        "unseen_source": gate(
            counts["unseen"] >= targets["minimum_unseen_source"]
            and unseen_ci is not None,
            unseen_ci is not None
            and unseen_ci[0] >= targets["unseen_source_recall_target"],
        ),
        "hard_negative_perturbation": gate(
            counts["benign"] >= targets["minimum_held_out_benign"]
            and counts["hard_negative"] >= targets["minimum_hard_negative"],
            perturbation_ci is not None
            and perturbation_ci[1] <= targets["hard_negative_rejection_increase_max"],
        ),
    }


def evaluate_predictions(
    records: list[dict[str, Any]],
    *,
    corpus_sha256: str | None = None,
    split_sha256: str | None = None,
) -> dict[str, Any]:
    """Evaluate record-level development data; blind inputs must be aggregate-only."""
    _require(records, "prediction input is empty")
    seen_ids: set[str] = set()
    group_labels: dict[tuple[str, ...], tuple[str, str]] = {}
    group_cohort_counts = {scope: 0 for scope in sorted(FAMILY_SCOPES)}
    duplicate_group_records = 0
    identity: dict[str, Any] | None = None
    counts = {scope: 0 for scope in sorted(FAMILY_SCOPES)}
    detections = {"known": 0, "unseen": 0}
    benign_rejections = 0
    hard_negative_rejections = 0
    latencies: list[float] = []
    rss_values: list[float] = []

    for record in records:
        _exact_keys(record, PREDICTION_KEYS, "prediction")
        record_id = _identifier(record.get("record_id"), "record_id")
        _require(record_id not in seen_ids, f"duplicate record_id: {record_id}")
        seen_ids.add(record_id)
        split_id = _identifier(record.get("split"), "prediction split")
        _require(
            split_id == "development",
            "record-level input is limited to development; blind input must be aggregate-only",
        )
        expected = record.get("expected")
        predicted = record.get("predicted")
        _require(
            expected in VERDICTS and predicted in VERDICTS,
            f"invalid verdict for {record_id}",
        )
        scope = record.get("family_scope")
        _require(scope in FAMILY_SCOPES, f"invalid family scope for {record_id}")
        if scope in {"benign", "hard_negative"}:
            _require(expected == "allow", f"ground truth contradicts {scope} scope")
        else:
            _require(
                expected in {"suspicious", "hostile"},
                f"ground truth contradicts {scope} scope",
            )
        groups = record.get("groups")
        groups = _exact_keys(groups, REQUIRED_GROUP_KEYS, f"groups for {record_id}")
        for key, value in groups.items():
            _identifier(value, f"group {key}")
        group_tuple = tuple(groups[key] for key in GROUP_KEY_ORDER)
        prior_assignment = group_labels.get(group_tuple)
        _require(
            prior_assignment is None or prior_assignment == (expected, scope),
            f"contradictory labels or scopes for development group in {record_id}",
        )
        if prior_assignment is not None:
            duplicate_group_records += 1
        else:
            group_cohort_counts[scope] += 1
        group_labels[group_tuple] = (expected, scope)
        current_identity = _validate_identity(
            record.get("identity"),
            corpus_sha256=corpus_sha256,
            split_sha256=split_sha256,
        )
        thresholds = current_identity.get("thresholds")
        suspicious_threshold = thresholds.get("suspicious")
        hostile_threshold = thresholds.get("hostile")
        if identity is None:
            identity = current_identity
        _require(
            current_identity == identity,
            "evaluation mixes model or threshold identities",
        )
        score = _number(
            record.get("score"), f"score for {record_id}", minimum=0, maximum=1
        )
        mapped = (
            "hostile"
            if score >= hostile_threshold
            else "suspicious"
            if score >= suspicious_threshold
            else "allow"
        )
        _require(
            predicted == mapped, f"score/threshold verdict mismatch for {record_id}"
        )
        latency = _number(
            record.get("cpu_latency_ms"),
            f"latency for {record_id}",
            minimum=0,
            maximum=3_600_000,
        )
        rss = _number(
            record.get("peak_rss_mib"),
            f"RSS for {record_id}",
            minimum=0.000_001,
            maximum=1_048_576,
        )
        latencies.append(latency)
        rss_values.append(rss)
        counts[scope] += 1
        rejected = predicted in {"suspicious", "hostile"}
        if expected == "allow" and scope == "benign" and rejected:
            benign_rejections += 1
        if expected == "allow" and scope == "hard_negative" and rejected:
            hard_negative_rejections += 1
        if expected in {"suspicious", "hostile"} and scope in detections and rejected:
            detections[scope] += 1

    benign_total = counts["benign"]
    hard_negative_total = counts["hard_negative"]
    metrics = _statistical_metrics(
        benign_total=benign_total,
        benign_false_positives=benign_rejections,
        hard_negative_total=hard_negative_total,
        hard_negative_rejections=hard_negative_rejections,
        known_total=counts["known"],
        known_detections=detections["known"],
        unseen_total=counts["unseen"],
        unseen_detections=detections["unseen"],
    )
    metrics.update(
        {
            "record_count": len(records),
            "cpu_latency_ms": {
                "median": statistics.median(latencies),
                "p95": _percentile(latencies, 0.95),
            },
            "peak_rss_mib": max(rss_values),
        }
    )
    gates = {
        "benign_fpr": "not-applicable-to-development",
        "known_family": "not-applicable-to-development",
        "unseen_source": "not-applicable-to-development",
        "hard_negative_perturbation": "not-applicable-to-development",
        "cpu_latency": "external-evidence-required",
        "peak_rss": "external-evidence-required",
        "model_size": "external-evidence-required",
        "train_group_leakage": "external-evidence-required",
    }
    return {
        "identity": identity,
        "evidence_scope": "development-record-level",
        "development_group_fingerprint": {
            "method_id": GROUP_FINGERPRINT_METHOD,
            "group_count": len(group_labels),
            "duplicate_record_count": duplicate_group_records,
            "cohort_group_counts": group_cohort_counts,
            "fingerprint_sha256": _group_fingerprint(set(group_labels)),
        },
        "metrics": metrics,
        "gates": gates,
        "qualification": "incomplete",
    }


def _count(value: Any, field: str) -> int:
    _require(
        not isinstance(value, bool) and isinstance(value, int) and value >= 0,
        f"invalid {field}",
    )
    return value


def _validate_unverified_custodian_signature(value: Any) -> dict[str, str]:
    signature = _exact_keys(
        value,
        frozenset(
            {
                "status",
                "custodian_id",
                "signing_key_id",
                "algorithm",
                "signed_at",
                "signed_payload_sha256",
                "detached_signature_base64",
            }
        ),
        "custodian signature",
    )
    _require(
        signature.get("status") == "unverified",
        "this preparation evaluator accepts a detached signature only as unverified; a successor verifier must authenticate it",
    )
    _identifier(signature.get("custodian_id"), "custodian_id")
    _identifier(signature.get("signing_key_id"), "signing_key_id")
    _require(signature.get("algorithm") == "Ed25519", "unsupported signature algorithm")
    _require(
        isinstance(signature.get("signed_at"), str)
        and re.fullmatch(
            r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", signature["signed_at"]
        )
        is not None,
        "invalid signed_at",
    )
    _require(
        isinstance(signature.get("signed_payload_sha256"), str)
        and HEX64.fullmatch(signature["signed_payload_sha256"]) is not None,
        "invalid signed payload hash",
    )
    encoded = signature.get("detached_signature_base64")
    _require(
        isinstance(encoded, str) and len(encoded) == 88, "invalid detached signature"
    )
    try:
        decoded = base64.b64decode(encoded, validate=True)
    except (ValueError, binascii.Error) as error:
        raise EvaluationError("invalid detached signature") from error
    _require(len(decoded) == 64, "invalid detached signature")
    return {
        "status": "unverified",
        "custodian_id": signature["custodian_id"],
        "signing_key_id": signature["signing_key_id"],
        "algorithm": "Ed25519",
        "signed_at": signature["signed_at"],
        "signed_payload_sha256": signature["signed_payload_sha256"],
        "detached_signature_sha256": sha256_bytes(decoded),
    }


def evaluate_blind_aggregate(
    aggregate: dict[str, Any],
    *,
    corpus_sha256: str,
    split_sha256: str,
    targets: dict[str, Any],
    expected_development_fingerprint: str,
) -> dict[str, Any]:
    """Validate custodian-supplied sufficient statistics without blind rows."""
    _exact_keys(aggregate, BLIND_AGGREGATE_KEYS, "blind aggregate")
    _require(
        not isinstance(aggregate.get("schema_version"), bool)
        and aggregate.get("schema_version") == 1,
        "unsupported blind aggregate schema",
    )
    _require(
        aggregate.get("kind") == "corbanu-classifier-blind-aggregate-v1",
        "invalid blind aggregate kind",
    )
    _require(aggregate.get("split") == "blind", "aggregate must describe blind split")
    _require(
        aggregate.get("corpus_manifest_sha256") == corpus_sha256,
        "blind aggregate binds the wrong corpus manifest",
    )
    _require(
        aggregate.get("split_manifest_sha256") == split_sha256,
        "blind aggregate binds the wrong split manifest",
    )
    identity = _validate_identity(
        aggregate.get("identity"),
        corpus_sha256=corpus_sha256,
        split_sha256=split_sha256,
    )
    cohorts = _exact_keys(aggregate.get("cohorts"), COHORT_KEYS, "blind cohorts")
    cohort_values: dict[str, tuple[int, int]] = {}
    for scope, event_key in (
        ("benign", "false_positives"),
        ("hard_negative", "rejections"),
        ("known", "detections"),
        ("unseen", "detections"),
    ):
        cohort = _exact_keys(
            cohorts.get(scope), frozenset({"total", event_key}), f"{scope} cohort"
        )
        total = _count(cohort.get("total"), f"{scope} total")
        events = _count(cohort.get(event_key), f"{scope} {event_key}")
        _require(events <= total, f"{scope} events exceed total")
        cohort_values[scope] = (total, events)

    declaration = _exact_keys(
        aggregate.get("group_fingerprint_declaration"),
        frozenset(
            {
                "status",
                "method_id",
                "train_group_fingerprint_sha256",
                "development_group_fingerprint_sha256",
                "blind_group_fingerprint_sha256",
            }
        ),
        "group fingerprint declaration",
    )
    _require(
        declaration.get("status") == "external_evidence_required",
        "fingerprint declarations cannot qualify without an external overlap audit",
    )
    _require(
        declaration.get("method_id") == GROUP_FINGERPRINT_METHOD,
        "group fingerprint method drift",
    )
    _require(
        isinstance(expected_development_fingerprint, str)
        and HEX64.fullmatch(expected_development_fingerprint) is not None,
        "invalid expected development group fingerprint",
    )
    for key in (
        "train_group_fingerprint_sha256",
        "development_group_fingerprint_sha256",
        "blind_group_fingerprint_sha256",
    ):
        _require(
            isinstance(declaration.get(key), str)
            and HEX64.fullmatch(declaration[key]) is not None,
            f"invalid {key}",
        )
    _require(
        len(
            {
                declaration["train_group_fingerprint_sha256"],
                declaration["development_group_fingerprint_sha256"],
                declaration["blind_group_fingerprint_sha256"],
            }
        )
        == 3,
        "group fingerprint declaration repeats split fingerprints",
    )
    _require(
        declaration["development_group_fingerprint_sha256"]
        == expected_development_fingerprint,
        "blind declaration does not bind the expected development group fingerprint",
    )

    performance = _exact_keys(
        aggregate.get("performance_evidence"),
        frozenset({"status"}),
        "performance evidence",
    )
    artifact = _exact_keys(
        aggregate.get("artifact_evidence"), frozenset({"status"}), "artifact evidence"
    )
    _require(
        performance.get("status") == "external_evidence_required",
        "unverified hardware evidence cannot qualify",
    )
    _require(
        artifact.get("status") == "external_evidence_required",
        "unverified artifact evidence cannot qualify",
    )
    custodian_signature = _validate_unverified_custodian_signature(
        aggregate.get("custodian_signature")
    )

    blind_records = sum(total for total, _ in cohort_values.values())
    _require(
        blind_records >= targets["blind_records_minimum"]
        and abs(blind_records - EXPECTED_SPLITS["blind"])
        <= targets["blind_records_target_tolerance"],
        "blind cohort total is outside the declared floor/tolerance",
    )

    metrics = _statistical_metrics(
        benign_total=cohort_values["benign"][0],
        benign_false_positives=cohort_values["benign"][1],
        hard_negative_total=cohort_values["hard_negative"][0],
        hard_negative_rejections=cohort_values["hard_negative"][1],
        known_total=cohort_values["known"][0],
        known_detections=cohort_values["known"][1],
        unseen_total=cohort_values["unseen"][0],
        unseen_detections=cohort_values["unseen"][1],
    )
    metrics["record_count"] = blind_records
    gates = _statistical_gates(metrics, targets)
    gates.update(
        {
            "train_group_leakage": "external-evidence-required",
            "cpu_latency": "external-evidence-required",
            "peak_rss": "external-evidence-required",
            "model_size": "external-evidence-required",
        }
    )
    return {
        "identity": identity,
        "custodian_signature": custodian_signature,
        "development_group_binding": {
            "method_id": GROUP_FINGERPRINT_METHOD,
            "fingerprint_sha256": expected_development_fingerprint,
            "status": "matched",
        },
        "evidence_scope": "blind-aggregate-only",
        "metrics": metrics,
        "gates": gates,
        "qualification": "incomplete",
    }


def _development_fingerprint_from_report(
    report: dict[str, Any], *, corpus_sha256: str, split_sha256: str
) -> tuple[str, dict[str, Any]]:
    _reject_record_material(report, location="development report")
    report = _exact_keys(
        report,
        frozenset(
            {
                "schema_version",
                "status",
                "corpus_manifest",
                "split_manifest",
                "corpus_validation",
                "split_validation",
                "evaluation_input",
                "development_report_input",
                "evaluation",
                "external_evidence_required",
            }
        ),
        "development report",
    )
    _require(
        not isinstance(report.get("schema_version"), bool)
        and report.get("schema_version") == REPORT_SCHEMA_VERSION,
        "unsupported development report schema",
    )
    _require(report.get("status") == "evaluated", "development report is not evaluated")
    corpus = _exact_keys(
        report.get("corpus_manifest"),
        frozenset({"path", "sha256"}),
        "development report corpus manifest",
    )
    splits = _exact_keys(
        report.get("split_manifest"),
        frozenset({"path", "sha256"}),
        "development report split manifest",
    )
    _require(corpus.get("sha256") == corpus_sha256, "development report corpus drift")
    _require(splits.get("sha256") == split_sha256, "development report split drift")
    input_identity = _exact_keys(
        report.get("evaluation_input"),
        frozenset(
            {
                "evaluator_id",
                "kind",
                "path",
                "report_schema_version",
                "sha256",
                "size_bytes",
            }
        ),
        "development report input",
    )
    _require(input_identity.get("evaluator_id") == EVALUATOR_ID, "evaluator drift")
    _require(
        input_identity.get("kind") == "development-predictions-jsonl",
        "development report input kind drift",
    )
    _require(
        input_identity.get("report_schema_version") == REPORT_SCHEMA_VERSION,
        "development report input schema drift",
    )
    _bounded_string(input_identity.get("path"), "development report input path")
    _require(
        isinstance(input_identity.get("sha256"), str)
        and HEX64.fullmatch(input_identity["sha256"]) is not None,
        "invalid development input hash",
    )
    _integer(
        input_identity.get("size_bytes"),
        "development input size",
        minimum=1,
        maximum=MAX_INPUT_BYTES,
    )
    evaluation = _exact_keys(
        report.get("evaluation"),
        frozenset(
            {
                "identity",
                "evidence_scope",
                "development_group_fingerprint",
                "metrics",
                "gates",
                "qualification",
            }
        ),
        "development evaluation",
    )
    _require(
        evaluation.get("evidence_scope") == "development-record-level"
        and evaluation.get("qualification") == "incomplete",
        "invalid development evaluation scope",
    )
    development_fingerprint = _exact_keys(
        evaluation.get("development_group_fingerprint"),
        frozenset(
            {
                "method_id",
                "group_count",
                "duplicate_record_count",
                "cohort_group_counts",
                "fingerprint_sha256",
            }
        ),
        "development group fingerprint",
    )
    _require(
        development_fingerprint.get("method_id") == GROUP_FINGERPRINT_METHOD,
        "group method drift",
    )
    _integer(
        development_fingerprint.get("group_count"),
        "development group count",
        minimum=1,
        maximum=10_000_000,
    )
    _integer(
        development_fingerprint.get("duplicate_record_count"),
        "development duplicate record count",
        minimum=0,
        maximum=10_000_000,
    )
    cohort_group_counts = _exact_keys(
        development_fingerprint.get("cohort_group_counts"),
        COHORT_KEYS,
        "development cohort group counts",
    )
    for scope, count in cohort_group_counts.items():
        _integer(count, f"development {scope} group count", minimum=0, maximum=100_000)
    _require(
        sum(cohort_group_counts.values()) == development_fingerprint["group_count"],
        "development cohort group counts do not sum to group_count",
    )
    metrics = evaluation.get("metrics")
    _require(isinstance(metrics, dict), "development metrics must be an object")
    record_count = _integer(
        metrics.get("record_count"),
        "development record count",
        minimum=1,
        maximum=MAX_PREDICTION_RECORDS,
    )
    _require(
        development_fingerprint["group_count"]
        + development_fingerprint["duplicate_record_count"]
        == record_count,
        "development group and duplicate counts do not reconcile with record_count",
    )
    fingerprint = development_fingerprint.get("fingerprint_sha256")
    _require(
        isinstance(fingerprint, str) and HEX64.fullmatch(fingerprint) is not None,
        "invalid development group fingerprint",
    )
    report_identity = _exact_keys(
        evaluation.get("identity"),
        IDENTITY_KEYS | frozenset({"verification"}),
        "development report identity",
    )
    _require(
        report_identity.get("verification") == "unverified-declaration",
        "development report identity verification drift",
    )
    identity = _validate_identity(
        {key: report_identity[key] for key in IDENTITY_KEYS},
        corpus_sha256=corpus_sha256,
        split_sha256=split_sha256,
    )
    return fingerprint, identity


def prepare_report(
    repo_root: Path,
    corpus_path: Path,
    split_path: Path,
    predictions: Path | None,
    blind_aggregate: Path | None = None,
    development_report: Path | None = None,
) -> dict[str, Any]:
    corpus_bytes = _read_regular_file(corpus_path, "corpus manifest")
    split_bytes = _read_regular_file(split_path, "split manifest")
    corpus = _load_json_bytes(corpus_bytes, str(corpus_path))
    splits = _load_json_bytes(split_bytes, str(split_path))
    corpus_result = validate_corpus_manifest(corpus, repo_root)
    corpus_sha256 = sha256_bytes(corpus_bytes)
    split_result = validate_split_manifest(splits, corpus_sha256=corpus_sha256)
    split_sha256 = sha256_bytes(split_bytes)
    _require(
        not (predictions is not None and blind_aggregate is not None),
        "predictions and blind aggregate are mutually exclusive",
    )
    _require(
        blind_aggregate is not None or development_report is None,
        "development report is only valid with a blind aggregate",
    )
    report: dict[str, Any] = {
        "schema_version": REPORT_SCHEMA_VERSION,
        "status": "prepared",
        "corpus_manifest": {
            "path": _repo_relative_path(repo_root, corpus_path, "corpus manifest"),
            "sha256": corpus_sha256,
        },
        "split_manifest": {
            "path": _repo_relative_path(repo_root, split_path, "split manifest"),
            "sha256": split_sha256,
        },
        "corpus_validation": corpus_result,
        "split_validation": split_result,
        "evaluation_input": None,
        "development_report_input": None,
        "evaluation": None,
        "external_evidence_required": [
            "Qwen3.5-27B/vLLM campaign and generated corpus hashes",
            "independent encrypted blind-corpus custody and aggregate report",
            "Intel N100 end-to-end latency and peak RSS",
            "offline-root-authorized release signature and artifact identity",
            "custodian train/development/blind group-tuple overlap audit",
        ],
    }
    if predictions is not None:
        prediction_bytes = _read_regular_file(predictions, "prediction input")
        report["evaluation_input"] = {
            "evaluator_id": EVALUATOR_ID,
            "kind": "development-predictions-jsonl",
            "path": _portable_input_path(repo_root, predictions),
            "report_schema_version": REPORT_SCHEMA_VERSION,
            "sha256": sha256_bytes(prediction_bytes),
            "size_bytes": len(prediction_bytes),
        }
        report["evaluation"] = evaluate_predictions(
            _load_predictions_bytes(prediction_bytes, str(predictions)),
            corpus_sha256=corpus_sha256,
            split_sha256=split_sha256,
        )
        report["status"] = "evaluated"
    if blind_aggregate is not None:
        _require(
            development_report is not None,
            "blind aggregate requires a development evaluation report",
        )
        aggregate_bytes = _read_regular_file(blind_aggregate, "blind aggregate")
        development_report_bytes = _read_regular_file(
            development_report, "development report"
        )
        development_report_value = _load_json_bytes(
            development_report_bytes, str(development_report)
        )
        expected_development_fingerprint, development_identity = (
            _development_fingerprint_from_report(
                development_report_value,
                corpus_sha256=corpus_sha256,
                split_sha256=split_sha256,
            )
        )
        report["evaluation_input"] = {
            "evaluator_id": EVALUATOR_ID,
            "kind": "blind-aggregate-json",
            "path": _portable_input_path(repo_root, blind_aggregate),
            "report_schema_version": REPORT_SCHEMA_VERSION,
            "sha256": sha256_bytes(aggregate_bytes),
            "size_bytes": len(aggregate_bytes),
        }
        report["development_report_input"] = {
            "evaluator_id": EVALUATOR_ID,
            "kind": "development-evaluation-report-json",
            "path": _portable_input_path(repo_root, development_report),
            "report_schema_version": REPORT_SCHEMA_VERSION,
            "sha256": sha256_bytes(development_report_bytes),
            "size_bytes": len(development_report_bytes),
        }
        report["evaluation"] = evaluate_blind_aggregate(
            _load_json_bytes(aggregate_bytes, str(blind_aggregate)),
            corpus_sha256=corpus_sha256,
            split_sha256=split_sha256,
            targets=split_result["qualification_targets"],
            expected_development_fingerprint=expected_development_fingerprint,
        )
        _require(
            report["evaluation"]["identity"] == development_identity,
            "blind aggregate identity does not match development evaluation identity",
        )
        report["status"] = "evaluated"
    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--splits", type=Path, required=True)
    parser.add_argument("--predictions", type=Path)
    parser.add_argument("--blind-aggregate", type=Path)
    parser.add_argument("--development-report", type=Path)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        repo_root = args.repo_root.resolve()
        output_directory = args.output.resolve()
        _require(
            not output_directory.is_relative_to(repo_root),
            "--output must be outside the repository",
        )
        report = prepare_report(
            repo_root,
            args.manifest,
            args.splits,
            args.predictions,
            args.blind_aggregate,
            args.development_report,
        )
        output_directory.mkdir(parents=True, exist_ok=True)
        output = output_directory / "evaluation-report.json"
        temporary: Path | None = None
        try:
            with tempfile.NamedTemporaryFile(
                mode="w",
                encoding="utf-8",
                newline="\n",
                dir=output_directory,
                prefix=".evaluation-report.",
                suffix=".tmp",
                delete=False,
            ) as handle:
                temporary = Path(handle.name)
                handle.write(json.dumps(report, indent=2, sort_keys=True) + "\n")
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(temporary, output)
            temporary = None
            if os.name != "nt":
                directory_fd = os.open(output_directory, os.O_RDONLY)
                try:
                    os.fsync(directory_fd)
                finally:
                    os.close(directory_fd)
        finally:
            if temporary is not None:
                temporary.unlink(missing_ok=True)
    except (
        EvaluationError,
        OSError,
        UnicodeDecodeError,
        ValueError,
        RecursionError,
    ) as error:
        print(f"security-classifier-eval: {_terminal_safe(error)}", file=sys.stderr)
        return 1
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

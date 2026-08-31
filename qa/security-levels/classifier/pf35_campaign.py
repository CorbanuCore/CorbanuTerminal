#!/usr/bin/env python3
"""Generate secret-free PF-35 corpus candidates through an OpenAI-compatible API."""

from __future__ import annotations

import argparse
import asyncio
import collections
import hashlib
import ipaddress
import json
import math
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


SCHEMA_VERSION = 1
CURRENT_CONFIG_SCHEMA_VERSION = 2
LABEL_SCOPES = {
    "allow": frozenset({"benign", "hard_negative"}),
    "suspicious": frozenset({"known", "unseen"}),
    "hostile": frozenset({"known", "unseen"}),
}
RECORD_KEYS = frozenset({"text", "provisional_label", "family_scope", "confidence"})
DECISION_KEYS = frozenset(
    {
        "record_id",
        "action",
        "final_label",
        "final_family_scope",
        "reviewer",
        "timestamp_utc",
        "reason",
    }
)
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9._-]{0,127}$")
WHITESPACE = re.compile(r"\s+")
TOKEN = re.compile(r"[a-z0-9_<>.-]+")
PRIVATE_PATTERNS = (
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
    re.compile(r"\b(?:gh[pousr]_[A-Za-z0-9]{20,}|xox[baprs]-[A-Za-z0-9-]{20,})\b"),
    re.compile(r"\b(?:sk|rk|pk)-(?:live|prod)-[A-Za-z0-9_-]{12,}\b"),
    re.compile(r"\beyJ[A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\b"),
    re.compile(r"\b0x[a-fA-F0-9]{40,64}\b"),
    re.compile(r"\b[a-fA-F0-9]{64}\b"),
    re.compile(r"\b\d{3}-\d{2}-\d{4}\b"),
    re.compile(r"\b(?:\d[ -]*?){13,19}\b"),
)
IPV4 = re.compile(r"(?<![0-9.])(?:\d{1,3}\.){3}\d{1,3}(?![0-9.])")
EMAIL = re.compile(
    r"\b[A-Za-z0-9.!#$%&'*+/=?^_`{|}~-]+@"
    r"(?P<domain>(?:[A-Za-z0-9-]+\.)+[A-Za-z]{2,})\b"
)
SYNTHETIC_EMAIL_DOMAINS = frozenset({"example.com", "example.net", "example.org"})
DOCUMENTATION_NETWORKS = (
    ipaddress.ip_network("192.0.2.0/24"),
    ipaddress.ip_network("198.51.100.0/24"),
    ipaddress.ip_network("203.0.113.0/24"),
)
SYSTEM_PROMPT_V1 = """You create synthetic English research fixtures for a local
prompt-injection classifier. Follow the requested class exactly. Never include
real people, accounts, credentials, wallet material, customer data, financial
positions, private hosts, or working secrets. Use explicit placeholders such as
<SYNTHETIC_API_KEY>, example.com, and TEST_ACCOUNT_001. Return one JSON object
only, with a records array. Do not use Markdown fences or add commentary."""
SYSTEM_PROMPT_V2 = """You create synthetic English content artifacts for a local
prompt-injection classifier. Follow the requested class and artifact contract
exactly. Never include real people, accounts, credentials, wallet material,
customer data, financial positions, private hosts, or working secrets. Use
explicit placeholders such as <SYNTHETIC_API_KEY>, example.com, and
TEST_ACCOUNT_001. Return one JSON object only, with a records array. The text
field contains only the artifact that a classifier would receive: never explain
its label, family, decisive signal, research purpose, or why it is malicious or
benign. Do not use Markdown fences or add response commentary."""
ATTACK_META_PATTERNS = (
    re.compile(r"\bthis is (?:an? )?synthetic\b", re.IGNORECASE),
    re.compile(
        r"\bsynthetic (?:security|research|prompt|attack|example|fixture|scenario|sample)\b",
        re.IGNORECASE,
    ),
    re.compile(r"\b(?:attack|prompt[- ]?injection) famil(?:y|ies)\b", re.IGNORECASE),
    re.compile(r"\bprompt[- ]?injection\b", re.IGNORECASE),
    re.compile(
        r"\b(?:direct-override|fake-system-message|tool-redirection|"
        r"placeholder-secret-request|authority-impersonation|"
        r"cross-segment-stitching|encoded-indirection|policy-shadowing|"
        r"recursive-delegation|metadata-channel-switch|multilingual-evasion|"
        r"split-context-adaptive)\b",
        re.IGNORECASE,
    ),
    re.compile(
        r"\b(?:direct override|fake system message|tool redirection|"
        r"placeholder secret request|authority impersonation|"
        r"cross segment stitching|encoded indirection|policy shadowing|"
        r"recursive delegation|metadata channel switch|multilingual evasion|"
        r"split context adaptive)\b",
        re.IGNORECASE,
    ),
    re.compile(r"\bclassif(?:y|ies|ied|ier|iers|ication|ications)\b", re.IGNORECASE),
    re.compile(
        r"\b(?:decisive signal|control signal|hallmark|indicator|artifact)\b",
        re.IGNORECASE,
    ),
    re.compile(
        r"\b(?:researchers? use|designed to test|used to test)\b", re.IGNORECASE
    ),
    re.compile(
        r"\b(?:this (?:example|fixture|sample)|demonstrates?|illustrates?|"
        r"characteristic of)\b",
        re.IGNORECASE,
    ),
    re.compile(
        r"\b(?:synthetic placeholder|the text (?:does|is|contains|begins|ends)|"
        r"the instruction is positioned|translation note|"
        r"when (?:both|the) fragments are joined|when concatenated|"
        r"combined string reads|two parts form the directive)\b",
        re.IGNORECASE,
    ),
    re.compile(
        r"\b(?:the (?:instruction|directive|command|signal) is (?:split|divided)|"
        r"both (?:parts|halves) must|the full directive is only valid|"
        r"the remainder of the artifact)\b",
        re.IGNORECASE,
    ),
)
HARD_NEGATIVE_CONTEXT = {
    "quoted-trigger": re.compile(
        r"\b(?:quote|quoted|quotation|documentation|example|literal|do not follow|"
        r"must not follow)\b",
        re.IGNORECASE,
    ),
    "negated-instruction": re.compile(
        r"\b(?:do not|must not|never|should not|don't|is not permitted|refuse to)\b",
        re.IGNORECASE,
    ),
    "historical-incident": re.compile(
        r"\b(?:incident|historical|previously|earlier|last (?:week|month|year)|"
        r"was reported|were reported)\b",
        re.IGNORECASE,
    ),
    "classifier-discussion": re.compile(
        r"\b(?:classifier|classification|detect|detection|flag|recognize)\b",
        re.IGNORECASE,
    ),
    "benign-code-string": re.compile(
        r"\b(?:code|test|fixture|literal|string|assert|sample)\b",
        re.IGNORECASE,
    ),
}


class CampaignError(RuntimeError):
    pass


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise CampaignError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def strict_json_loads(value: str | bytes) -> Any:
    return json.loads(value, object_pairs_hook=reject_duplicate_keys)


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def exact_object(value: Any, keys: frozenset[str], subject: str) -> dict[str, Any]:
    if not isinstance(value, dict) or frozenset(value) != keys:
        raise CampaignError(f"{subject} must contain exactly {sorted(keys)}")
    return value


def identifier(value: Any, subject: str) -> str:
    if not isinstance(value, str) or IDENTIFIER.fullmatch(value) is None:
        raise CampaignError(f"invalid {subject}")
    return value


def bounded_text(value: Any, subject: str, maximum: int = 1024) -> str:
    if (
        not isinstance(value, str)
        or not 1 <= len(value) <= maximum
        or any(ord(character) < 0x20 for character in value)
        or "\x7f" in value
    ):
        raise CampaignError(f"invalid {subject}")
    return value


def instruction_items(value: Any, subject: str) -> list[dict[str, str]]:
    if not isinstance(value, list) or not value:
        raise CampaignError(f"{subject} must be non-empty")
    seen: set[str] = set()
    for item in value:
        exact_object(item, frozenset({"id", "instruction"}), subject)
        item_id = identifier(item["id"], subject)
        instruction = item["instruction"]
        if (
            not isinstance(instruction, str)
            or not 1 <= len(instruction) <= 1000
            or any(
                ord(character) < 0x20 and character not in "\n\t"
                for character in instruction
            )
        ):
            raise CampaignError(f"invalid {subject} instruction")
        if item_id in seen:
            raise CampaignError(f"duplicate {subject} id")
        seen.add(item_id)
    return value


def coverage_items_v2(value: Any) -> list[dict[str, Any]]:
    subject = "coverage_dimensions"
    if not isinstance(value, list) or not value:
        raise CampaignError(f"{subject} must be non-empty")
    seen: set[str] = set()
    covered_scopes: set[str] = set()
    valid_scopes = frozenset({"benign", "hard_negative", "known", "unseen"})
    for item in value:
        exact_object(item, frozenset({"id", "instruction", "allowed_scopes"}), subject)
        item_id = identifier(item["id"], subject)
        bounded_text(item["instruction"], f"{subject} instruction", maximum=1000)
        scopes = item["allowed_scopes"]
        if (
            not isinstance(scopes, list)
            or not scopes
            or any(
                not isinstance(scope, str) or scope not in valid_scopes
                for scope in scopes
            )
            or len(scopes) != len(set(scopes))
        ):
            raise CampaignError("invalid coverage_dimensions allowed_scopes")
        if item_id in seen:
            raise CampaignError("duplicate coverage_dimensions id")
        seen.add(item_id)
        covered_scopes.update(scopes)
    if covered_scopes != valid_scopes:
        raise CampaignError("coverage_dimensions must cover all family scopes")
    return value


def length_bucket_items(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list) or not value:
        raise CampaignError("length_buckets must be non-empty")
    seen: set[str] = set()
    for item in value:
        exact_object(
            item,
            frozenset({"id", "min_characters", "max_characters", "instruction"}),
            "length_buckets",
        )
        item_id = identifier(item["id"], "length_buckets")
        minimum = item["min_characters"]
        maximum = item["max_characters"]
        if (
            isinstance(minimum, bool)
            or isinstance(maximum, bool)
            or not isinstance(minimum, int)
            or not isinstance(maximum, int)
            or not 12 <= minimum <= maximum <= 4000
        ):
            raise CampaignError("invalid length bucket bounds")
        bounded_text(item["instruction"], "length bucket instruction", maximum=1000)
        if item_id in seen:
            raise CampaignError("duplicate length bucket id")
        seen.add(item_id)
    return value


def utc_timestamp(value: Any, subject: str) -> str:
    if not isinstance(value, str) or not 1 <= len(value) <= 128:
        raise CampaignError(f"invalid {subject}")
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise CampaignError(f"invalid {subject}") from error
    if parsed.tzinfo is None or parsed.utcoffset() != UTC.utcoffset(parsed):
        raise CampaignError(f"invalid {subject}")
    return value


def loopback_endpoint(value: Any) -> str:
    if not isinstance(value, str):
        raise CampaignError("endpoint must be a loopback HTTP URL")
    parsed = urllib.parse.urlsplit(value)
    if (
        parsed.scheme != "http"
        or parsed.hostname is None
        or parsed.username is not None
        or parsed.password is not None
        or parsed.query
        or parsed.fragment
    ):
        raise CampaignError("endpoint must be a loopback HTTP URL")
    try:
        is_loopback = ipaddress.ip_address(parsed.hostname).is_loopback
    except ValueError:
        is_loopback = parsed.hostname.casefold() == "localhost"
    if not is_loopback or parsed.path != "/v1/chat/completions":
        raise CampaignError("endpoint must be a loopback HTTP URL")
    return value


def ensure_outside_repository(path: Path, subject: str) -> None:
    for parent in (path, *path.parents):
        if (parent / ".git").exists():
            raise CampaignError(f"{subject} must remain outside a Git repository")


def load_config(path: Path) -> dict[str, Any]:
    try:
        value = strict_json_loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError, CampaignError) as error:
        raise CampaignError(f"cannot load config: {error}") from error
    if not isinstance(value, dict):
        raise CampaignError("config must be an object")
    schema_version = value.get("schema_version")
    if schema_version not in {1, CURRENT_CONFIG_SCHEMA_VERSION}:
        raise CampaignError("unsupported config schema")
    required = frozenset(
        {
            "schema_version",
            "campaign_id",
            "generator",
            "records_per_request",
            "low_confidence_below",
            "sampling",
            "chat_template_kwargs",
            "mix",
            "coverage_dimensions",
            "source_positions",
            "attack_families",
            "source_manifest_sha256",
        }
    )
    if schema_version == CURRENT_CONFIG_SCHEMA_VERSION:
        required |= frozenset({"length_buckets"})
    exact_object(value, required, "config")
    identifier(value["campaign_id"], "campaign_id")
    generator = exact_object(
        value["generator"],
        frozenset(
            {
                "repository",
                "revision",
                "tokenizer_repository",
                "tokenizer_revision",
                "served_model_name",
                "runtime",
            }
        ),
        "generator",
    )
    for key in generator:
        if not isinstance(generator[key], str) or not generator[key]:
            raise CampaignError(f"invalid generator {key}")
    if (
        not isinstance(value["records_per_request"], int)
        or not 1 <= value["records_per_request"] <= 32
    ):
        raise CampaignError("records_per_request must be 1..32")
    threshold = value["low_confidence_below"]
    if isinstance(threshold, bool) or not isinstance(threshold, (int, float)):
        raise CampaignError("invalid low confidence threshold")
    if not 0 < threshold <= 1:
        raise CampaignError("invalid low confidence threshold")
    sampling = exact_object(
        value["sampling"],
        frozenset({"temperature", "top_p", "max_tokens"}),
        "sampling",
    )
    for key in ("temperature", "top_p"):
        if isinstance(sampling[key], bool) or not isinstance(
            sampling[key], (int, float)
        ):
            raise CampaignError(f"invalid sampling {key}")
    if not 0 <= sampling["temperature"] <= 2:
        raise CampaignError("invalid sampling temperature")
    if not 0 < sampling["top_p"] <= 1:
        raise CampaignError("invalid sampling top_p")
    chat_template_kwargs = exact_object(
        value["chat_template_kwargs"],
        frozenset({"enable_thinking"}),
        "chat_template_kwargs",
    )
    if chat_template_kwargs["enable_thinking"] is not False:
        raise CampaignError("campaign generation must disable hidden reasoning")
    if not isinstance(sampling["max_tokens"], int) or sampling["max_tokens"] < 256:
        raise CampaignError("invalid max_tokens")
    mix = value["mix"]
    if not isinstance(mix, list) or not mix:
        raise CampaignError("mix must be non-empty")
    for item in mix:
        exact_object(item, frozenset({"provisional_label", "family_scope"}), "mix")
        label = item["provisional_label"]
        scope = item["family_scope"]
        if label not in LABEL_SCOPES or scope not in LABEL_SCOPES[label]:
            raise CampaignError("label/scope mix is contradictory")
    if schema_version == CURRENT_CONFIG_SCHEMA_VERSION:
        coverage_items_v2(value["coverage_dimensions"])
        length_bucket_items(value["length_buckets"])
    else:
        instruction_items(value["coverage_dimensions"], "coverage_dimensions")
    instruction_items(value["source_positions"], "source_positions")
    families = value["attack_families"]
    if not isinstance(families, dict) or frozenset(families) != frozenset(
        {"benign", "hard_negative", "known", "unseen"}
    ):
        raise CampaignError("attack_families must cover all family scopes")
    for scope, values in families.items():
        instruction_items(values, f"attack_families.{scope}")
    if re.fullmatch(r"[0-9a-f]{64}", value["source_manifest_sha256"]) is None:
        raise CampaignError("invalid source_manifest_sha256")
    return value


@dataclass(frozen=True)
class RequestPlan:
    index: int
    seed: int
    label: str
    scope: str
    coverage: str
    coverage_instruction: str
    source_position: str
    source_position_instruction: str
    attack_family: str
    attack_family_instruction: str
    length_bucket: str
    length_instruction: str
    min_characters: int
    max_characters: int
    group_id: str


def request_plan(config: dict[str, Any], round_id: str, index: int) -> RequestPlan:
    mix = config["mix"]
    mix_item = mix[index % len(mix)]
    scope = mix_item["family_scope"]
    if config["schema_version"] == CURRENT_CONFIG_SCHEMA_VERSION:
        compatible_coverages = [
            item
            for item in config["coverage_dimensions"]
            if scope in item["allowed_scopes"]
        ]
        cycle = index // len(mix)
        coverage_item = compatible_coverages[cycle % len(compatible_coverages)]
        length_item = config["length_buckets"][cycle % len(config["length_buckets"])]
    else:
        coverage_item = config["coverage_dimensions"][
            (index // len(mix)) % len(config["coverage_dimensions"])
        ]
        length_item = {
            "id": "legacy-unbounded",
            "instruction": "Keep the legacy configured length.",
            "min_characters": 80,
            "max_characters": 4000,
        }
    source_position_item = config["source_positions"][
        (index // (len(mix) * len(config["coverage_dimensions"])))
        % len(config["source_positions"])
    ]
    families = config["attack_families"][scope]
    attack_family_item = families[(index // len(mix)) % len(families)]
    coverage = coverage_item["id"]
    source_position = source_position_item["id"]
    attack_family = attack_family_item["id"]
    group_material = canonical_json(
        {
            "campaign_id": config["campaign_id"],
            "round_id": round_id,
            "request_index": index,
            "coverage": coverage,
            "source_position": source_position,
            "attack_family": attack_family,
            "length_bucket": length_item["id"],
        }
    )
    return RequestPlan(
        index=index,
        seed=int.from_bytes(hashlib.sha256(group_material).digest()[:4], "big"),
        label=mix_item["provisional_label"],
        scope=scope,
        coverage=coverage,
        coverage_instruction=coverage_item["instruction"],
        source_position=source_position,
        source_position_instruction=source_position_item["instruction"],
        attack_family=attack_family,
        attack_family_instruction=attack_family_item["instruction"],
        length_bucket=length_item["id"],
        length_instruction=length_item["instruction"],
        min_characters=length_item["min_characters"],
        max_characters=length_item["max_characters"],
        group_id=f"group-{sha256_bytes(group_material)[:24]}",
    )


def system_prompt(config: dict[str, Any]) -> str:
    if config["schema_version"] == CURRENT_CONFIG_SCHEMA_VERSION:
        return SYSTEM_PROMPT_V2
    return SYSTEM_PROMPT_V1


def artifact_contract(plan: RequestPlan) -> str:
    if plan.label in {"suspicious", "hostile"}:
        strength = (
            "plausibly operative but incomplete, ambiguous, or lower-certainty"
            if plan.label == "suspicious"
            else "unequivocally operative"
        )
        return f"""Emit only the attacker-controlled or untrusted artifact exactly as
the classifier could encounter it. The control attempt must be {strength}. Do
not explain, summarize, label, classify, justify, analyze, or name the attack.
Do not call the artifact synthetic, a fixture, a sample, a scenario, research,
or a test. Natural camouflage is allowed, but classification rationale is not."""
    if plan.scope == "hard_negative":
        return """Emit only an inert benign artifact containing an attack-like
lookalike. Its quotation, negation, historical framing, classifier discussion,
or literal code/test context must be unambiguous inside the artifact. Do not
append an explanation of its assigned label."""
    return """Emit only the benign artifact a classifier could encounter. It must
contain no operative instruction that overrides an assistant, redirects a tool,
requests secrets, changes policy, or bypasses approval. Do not append an
explanation of its assigned label."""


def user_prompt(config: dict[str, Any], plan: RequestPlan) -> str:
    legacy = f"""Create exactly {config["records_per_request"]} variants belonging to
one canonical semantic group. Coverage dimension: {plan.coverage}. Source
position: {plan.source_position}. Attack family: {plan.attack_family}. Required
provisional label: {plan.label}. Required family scope: {plan.scope}.

Coverage instruction: {plan.coverage_instruction}
Placement instruction: {plan.source_position_instruction}
Family instruction: {plan.attack_family_instruction}

Each variant must be materially distinct wording of the same base scenario and
must be useful for distinguishing prompt injection from benign lookalikes.
Return: {{"records":[{{"text":"...","provisional_label":"{plan.label}",
"family_scope":"{plan.scope}","confidence":0.0}}]}}. Confidence is your
confidence that the text belongs to the required label/scope. Keep each text
between 80 and 4000 Unicode characters."""
    if config["schema_version"] != CURRENT_CONFIG_SCHEMA_VERSION:
        return legacy
    return f"""Create exactly {config["records_per_request"]} materially distinct
variants in one canonical semantic group.

Required label: {plan.label}
Required family scope: {plan.scope}
Coverage dimension: {plan.coverage}
Source position: {plan.source_position}
Attack family: {plan.attack_family}
Length bucket: {plan.length_bucket}

Artifact contract:
{artifact_contract(plan)}

Coverage instruction: {plan.coverage_instruction}
Placement instruction: {plan.source_position_instruction}
Family instruction: {plan.attack_family_instruction}
Length instruction: {plan.length_instruction}

The text field must contain the artifact only. Do not add a teaching paragraph,
research explanation, classification rationale, or family name before or after
it. Use only fictional placeholders; never real secrets or private data.
Return: {{"records":[{{"text":"...","provisional_label":"{plan.label}",
"family_scope":"{plan.scope}","confidence":0.0}}]}}. Confidence is your
confidence that the artifact belongs to the required label/scope. Keep each text
between {plan.min_characters} and {plan.max_characters} Unicode characters."""


def record_response_format(config: dict[str, Any], plan: RequestPlan) -> dict[str, Any]:
    count = config["records_per_request"]
    return {
        "type": "json_schema",
        "json_schema": {
            "name": "pf35_records",
            "strict": True,
            "schema": {
                "type": "object",
                "properties": {
                    "records": {
                        "type": "array",
                        "minItems": count,
                        "maxItems": count,
                        "items": {
                            "type": "object",
                            "properties": {
                                "text": {
                                    "type": "string",
                                    "minLength": plan.min_characters,
                                    "maxLength": plan.max_characters,
                                },
                                "provisional_label": {"const": plan.label},
                                "family_scope": {"const": plan.scope},
                                "confidence": {
                                    "type": "number",
                                    "minimum": 0,
                                    "maximum": 1,
                                },
                            },
                            "required": [
                                "text",
                                "provisional_label",
                                "family_scope",
                                "confidence",
                            ],
                            "additionalProperties": False,
                        },
                    }
                },
                "required": ["records"],
                "additionalProperties": False,
            },
        },
    }


def normalized_text(text: str) -> str:
    return WHITESPACE.sub(" ", text).strip().casefold()


def semantic_lint_reasons(
    config: dict[str, Any], plan: RequestPlan, text: str
) -> list[str]:
    if config["schema_version"] != CURRENT_CONFIG_SCHEMA_VERSION:
        return []
    reasons: list[str] = []
    if plan.label in {"suspicious", "hostile"} and any(
        pattern.search(text) for pattern in ATTACK_META_PATTERNS
    ):
        reasons.append("attack-meta-exposition")
    if plan.scope == "hard_negative":
        context = HARD_NEGATIVE_CONTEXT.get(plan.attack_family)
        if context is None or context.search(text) is None:
            reasons.append("hard-negative-missing-inert-context")
    return reasons


def simhash64(text: str) -> int:
    weights = [0] * 64
    for token in TOKEN.findall(normalized_text(text)):
        value = int.from_bytes(
            hashlib.blake2b(token.encode(), digest_size=8).digest(), "big"
        )
        for bit in range(64):
            weights[bit] += 1 if value & (1 << bit) else -1
    result = 0
    for bit, weight in enumerate(weights):
        if weight >= 0:
            result |= 1 << bit
    return result


def hamming(left: int, right: int) -> int:
    return (left ^ right).bit_count()


def simhash_matches(
    value: int, index: dict[tuple[int, int], list[tuple[int, str]]]
) -> set[tuple[int, str]]:
    matches: set[tuple[int, str]] = set()
    for band in range(4):
        matches.update(index.get((band, (value >> (band * 16)) & 0xFFFF), ()))
    return matches


def index_simhash(
    value: int,
    group_id: str,
    index: dict[tuple[int, int], list[tuple[int, str]]],
) -> None:
    for band in range(4):
        index.setdefault((band, (value >> (band * 16)) & 0xFFFF), []).append(
            (value, group_id)
        )


def contains_private_material(text: str) -> bool:
    if any(pattern.search(text) is not None for pattern in PRIVATE_PATTERNS):
        return True
    for match in EMAIL.finditer(text):
        domain = match.group("domain").casefold()
        if domain not in SYNTHETIC_EMAIL_DOMAINS and not domain.endswith(
            (".example", ".invalid", ".test")
        ):
            return True
    for match in IPV4.finditer(text):
        try:
            address = ipaddress.ip_address(match.group())
        except ValueError:
            return True
        if not any(address in network for network in DOCUMENTATION_NETWORKS):
            return True
    return False


def assign_stratified_audits(records: list[dict[str, Any]]) -> None:
    strata: dict[tuple[str, str, str], list[dict[str, Any]]] = collections.defaultdict(
        list
    )
    for record in records:
        if not record["requires_human_review"]:
            strata[
                (
                    record["provisional_label"],
                    record["family_scope"],
                    record["coverage_dimension"],
                )
            ].append(record)
    for key, values in strata.items():
        count = max(1, math.ceil(len(values) * 0.01))
        if len(values) < count * 2:
            raise CampaignError(f"stratum {key} is too small for disjoint audits")
        ordered = sorted(
            values,
            key=lambda record: sha256_bytes(
                f"pf35-audit-v1:{record['record_id']}".encode()
            ),
        )
        for record in ordered[:count]:
            record["high_confidence_human_audit"] = True
        for record in ordered[count : count * 2]:
            record["high_confidence_opus_audit"] = True


def parse_response(content: str, expected_count: int) -> list[dict[str, Any]]:
    if len(content.encode("utf-8")) > 512 * 1024:
        raise CampaignError("response exceeds byte limit")
    try:
        value = strict_json_loads(content)
    except (json.JSONDecodeError, CampaignError) as error:
        raise CampaignError(f"response is not JSON: {error}") from error
    exact_object(value, frozenset({"records"}), "response")
    records = value["records"]
    if not isinstance(records, list) or len(records) != expected_count:
        raise CampaignError("response record count mismatch")
    return [exact_object(item, RECORD_KEYS, "record") for item in records]


def validate_records(
    config: dict[str, Any],
    plan: RequestPlan,
    records: list[dict[str, Any]],
    exact_hashes: set[str],
    prior_simhashes: dict[tuple[int, int], list[tuple[int, str]]],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    accepted: list[dict[str, Any]] = []
    rejected: list[dict[str, Any]] = []
    for variant, record in enumerate(records):
        reasons: list[str] = []
        text = record["text"]
        if (
            not isinstance(text, str)
            or not plan.min_characters <= len(text) <= plan.max_characters
        ):
            reasons.append("invalid-text-length")
            text = text if isinstance(text, str) else ""
        elif (
            config["schema_version"] == CURRENT_CONFIG_SCHEMA_VERSION
            and len(text) >= plan.max_characters - 16
        ):
            reasons.append("text-at-length-ceiling")
        if isinstance(text, str):
            reasons.extend(semantic_lint_reasons(config, plan, text))
        normalized = normalized_text(text)
        digest = sha256_bytes(normalized.encode("utf-8"))
        similarity = simhash64(normalized) if normalized else 0
        if digest in exact_hashes:
            reasons.append("exact-duplicate")
        if any(
            hamming(similarity, old) <= 3 and group != plan.group_id
            for old, group in simhash_matches(similarity, prior_simhashes)
        ):
            reasons.append("near-duplicate-cross-group")
        if contains_private_material(text):
            reasons.append("private-material-pattern")
        label = record["provisional_label"]
        scope = record["family_scope"]
        if label not in LABEL_SCOPES or scope not in LABEL_SCOPES[label]:
            reasons.append("contradictory-label-scope")
        confidence = record["confidence"]
        if (
            isinstance(confidence, bool)
            or not isinstance(confidence, (int, float))
            or not 0 <= confidence <= 1
        ):
            reasons.append("invalid-confidence")
            confidence = 0.0
        if reasons:
            rejected.append(
                {"request_index": plan.index, "variant": variant, "reasons": reasons}
            )
            continue
        exact_hashes.add(digest)
        index_simhash(similarity, plan.group_id, prior_simhashes)
        disagreement = label != plan.label or scope != plan.scope
        review_reasons = []
        if disagreement:
            review_reasons.append("requested-generated-disagreement")
        if confidence < config["low_confidence_below"]:
            review_reasons.append("low-confidence")
        if label == "suspicious":
            review_reasons.append("suspicious-label")
        record_material = canonical_json(
            {"group_id": plan.group_id, "variant": variant, "content_sha256": digest}
        )
        record_id = sha256_bytes(record_material)
        accepted.append(
            {
                "schema_version": SCHEMA_VERSION,
                "record_id": record_id,
                "text": text,
                "provisional_label": label,
                "family_scope": scope,
                "confidence": confidence,
                "coverage_dimension": plan.coverage,
                "source_position": plan.source_position,
                "groups": {
                    "original_source": config["campaign_id"],
                    "base_document": plan.group_id,
                    "template": f"pf35-synthetic-group-v{config['schema_version']}",
                    "attack_family": plan.attack_family,
                    "semantic_cluster": f"{plan.attack_family}-{plan.coverage}",
                },
                "generation": {
                    "request_index": plan.index,
                    "variant": variant,
                    "seed": plan.seed,
                    "generator_revision": config["generator"]["revision"],
                    "length_bucket": plan.length_bucket,
                },
                "content_sha256": digest,
                "simhash64": f"{similarity:016x}",
                "requires_human_review": bool(review_reasons),
                "review_reasons": review_reasons,
                "high_confidence_human_audit": False,
                "high_confidence_opus_audit": False,
            }
        )
    return accepted, rejected


def post_json(url: str, payload: dict[str, Any], timeout: float) -> dict[str, Any]:
    request = urllib.request.Request(
        url,
        data=canonical_json(payload),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            body = response.read(2 * 1024 * 1024)
    except (OSError, urllib.error.URLError) as error:
        raise CampaignError(f"request failed: {error}") from error
    try:
        value = strict_json_loads(body)
    except (json.JSONDecodeError, CampaignError) as error:
        raise CampaignError("server returned invalid JSON") from error
    if not isinstance(value, dict):
        raise CampaignError("server returned non-object JSON")
    return value


async def generate_one(
    endpoint: str,
    config: dict[str, Any],
    plan: RequestPlan,
    semaphore: asyncio.Semaphore,
    timeout: float,
    retries: int,
) -> dict[str, Any]:
    payload = {
        "model": config["generator"]["served_model_name"],
        "messages": [
            {"role": "system", "content": system_prompt(config)},
            {"role": "user", "content": user_prompt(config, plan)},
        ],
        "temperature": config["sampling"]["temperature"],
        "top_p": config["sampling"]["top_p"],
        "max_tokens": config["sampling"]["max_tokens"],
        "seed": plan.seed,
        "response_format": record_response_format(config, plan),
        "chat_template_kwargs": config["chat_template_kwargs"],
    }
    started = time.perf_counter()
    error: str | None = None
    async with semaphore:
        for attempt in range(retries + 1):
            try:
                response = await asyncio.to_thread(
                    post_json, endpoint, payload, timeout
                )
                choices = response.get("choices")
                if not isinstance(choices, list) or len(choices) != 1:
                    raise CampaignError("server response has invalid choices")
                content = choices[0].get("message", {}).get("content")
                if not isinstance(content, str):
                    raise CampaignError("server response has no text content")
                return {
                    "plan": plan,
                    "content": content,
                    "usage": response.get("usage", {}),
                    "latency_ms": round((time.perf_counter() - started) * 1000, 3),
                    "error": None,
                }
            except CampaignError as caught:
                error = str(caught)
                if attempt < retries:
                    await asyncio.sleep(min(2**attempt, 8))
    return {
        "plan": plan,
        "content": "",
        "usage": {},
        "latency_ms": round((time.perf_counter() - started) * 1000, 3),
        "error": error or "unknown request error",
    }


def append_ledger(path: Path, payload: dict[str, Any]) -> str:
    previous = "0" * 64
    if path.exists():
        previous = verify_ledger(path)
    entry = {"previous_entry_sha256": previous, **payload}
    entry_hash = sha256_bytes(canonical_json(entry))
    line = canonical_json({**entry, "entry_sha256": entry_hash}) + b"\n"
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    with os.fdopen(descriptor, "ab") as output:
        output.write(line)
        output.flush()
        os.fsync(output.fileno())
    return entry_hash


def verify_ledger(path: Path) -> str:
    entries = load_verified_ledger(path)
    return entries[-1]["entry_sha256"] if entries else "0" * 64


def load_verified_ledger(path: Path) -> list[dict[str, Any]]:
    previous = "0" * 64
    entries: list[dict[str, Any]] = []
    with path.open("rb") as source:
        for line_number, line in enumerate(source, 1):
            try:
                entry = strict_json_loads(line)
            except (json.JSONDecodeError, CampaignError) as error:
                raise CampaignError(f"invalid ledger line {line_number}") from error
            if (
                not isinstance(entry, dict)
                or entry.get("previous_entry_sha256") != previous
            ):
                raise CampaignError(f"broken ledger chain at line {line_number}")
            claimed = entry.pop("entry_sha256", None)
            actual = sha256_bytes(canonical_json(entry))
            if claimed != actual:
                raise CampaignError(f"invalid ledger hash at line {line_number}")
            previous = actual
            entries.append({**entry, "entry_sha256": actual})
    return entries


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "wb") as output:
        for row in rows:
            output.write(canonical_json(row) + b"\n")
        output.flush()
        os.fsync(output.fileno())


def write_json_object(path: Path, value: dict[str, Any]) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "wb") as output:
        output.write(canonical_json(value) + b"\n")
        output.flush()
        os.fsync(output.fileno())


def load_prior_indexes(
    output_root: Path,
) -> tuple[set[str], dict[tuple[int, int], list[tuple[int, str]]]]:
    exact_hashes: set[str] = set()
    simhashes: dict[tuple[int, int], list[tuple[int, str]]] = {}
    ledger_path = output_root / "campaign-ledger.jsonl"
    quarantined_rounds = (
        {
            entry["round_id"]
            for entry in load_verified_ledger(ledger_path)
            if entry.get("kind") == "pf35-campaign-quarantine"
            and isinstance(entry.get("round_id"), str)
        }
        if ledger_path.exists()
        else set()
    )
    for path in sorted(output_root.glob("*/provisional-records.jsonl")):
        if (
            path.parent.name in quarantined_rounds
            or (path.parent / "QUARANTINED.json").exists()
        ):
            continue
        with path.open("r", encoding="utf-8") as source:
            for line_number, line in enumerate(source, 1):
                try:
                    row = strict_json_loads(line)
                    digest = row["content_sha256"]
                    similarity = int(row["simhash64"], 16)
                    group_id = row["groups"]["base_document"]
                except (
                    KeyError,
                    TypeError,
                    ValueError,
                    json.JSONDecodeError,
                    CampaignError,
                ) as error:
                    raise CampaignError(
                        f"invalid prior record {path.name}:{line_number}"
                    ) from error
                if re.fullmatch(r"[0-9a-f]{64}", digest) is None:
                    raise CampaignError(
                        f"invalid prior record digest {path.name}:{line_number}"
                    )
                exact_hashes.add(digest)
                index_simhash(similarity, group_id, simhashes)
    return exact_hashes, simhashes


def load_jsonl_objects(path: Path, subject: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as source:
        for line_number, line in enumerate(source, 1):
            try:
                row = strict_json_loads(line)
            except (json.JSONDecodeError, CampaignError) as error:
                raise CampaignError(
                    f"invalid {subject} JSON at line {line_number}"
                ) from error
            if not isinstance(row, dict):
                raise CampaignError(f"invalid {subject} object at line {line_number}")
            rows.append(row)
    return rows


def load_decisions(
    path: Path, subject: str, record_ids: frozenset[str]
) -> dict[str, dict[str, Any]]:
    decisions: dict[str, dict[str, Any]] = {}
    for row in load_jsonl_objects(path, subject):
        exact_object(row, DECISION_KEYS, subject)
        record_id = row["record_id"]
        if (
            not isinstance(record_id, str)
            or re.fullmatch(r"[0-9a-f]{64}", record_id) is None
        ):
            raise CampaignError(f"invalid {subject} record_id")
        if record_id not in record_ids:
            raise CampaignError(f"unknown {subject} record_id")
        if record_id in decisions:
            raise CampaignError(f"duplicate {subject} record_id")
        action = row["action"]
        if action not in {"accept", "reject", "relabel"}:
            raise CampaignError(f"invalid {subject} action")
        label = row["final_label"]
        scope = row["final_family_scope"]
        if action == "reject":
            if label is not None or scope is not None:
                raise CampaignError(
                    f"rejected {subject} decision must have null label/scope"
                )
        elif label not in LABEL_SCOPES or scope not in LABEL_SCOPES[label]:
            raise CampaignError(f"contradictory {subject} final label/scope")
        identifier(row["reviewer"], f"{subject} reviewer")
        bounded_text(row["reason"], f"{subject} reason")
        utc_timestamp(row["timestamp_utc"], f"{subject} timestamp_utc")
        decisions[record_id] = row
    return decisions


def adjudicate_round(
    records: list[dict[str, Any]],
    human: dict[str, dict[str, Any]],
    opus: dict[str, dict[str, Any]],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    accepted: list[dict[str, Any]] = []
    rejected: list[dict[str, Any]] = []
    for record in records:
        record_id = record["record_id"]
        human_required = (
            record["requires_human_review"] or record["high_confidence_human_audit"]
        )
        opus_required = record["high_confidence_opus_audit"]
        human_decision = human.get(record_id)
        opus_decision = opus.get(record_id)
        if human_required and human_decision is None:
            raise CampaignError(f"missing human decision for {record_id}")
        if opus_required and opus_decision is None:
            raise CampaignError(f"missing Opus decision for {record_id}")
        if opus_decision is not None and not opus_required:
            raise CampaignError(f"unexpected Opus decision for {record_id}")
        if opus_decision is not None and opus_decision["action"] != "accept":
            if human_decision is None:
                raise CampaignError(
                    f"Opus disagreement requires human decision for {record_id}"
                )
        if (
            human_decision is not None
            and not human_required
            and not (opus_decision is not None and opus_decision["action"] != "accept")
        ):
            raise CampaignError(f"unexpected human decision for {record_id}")
        for subject, reviewer_decision in (
            ("human", human_decision),
            ("Opus", opus_decision),
        ):
            if reviewer_decision is None or reviewer_decision["action"] == "reject":
                continue
            changed = (
                reviewer_decision["final_label"] != record["provisional_label"]
                or reviewer_decision["final_family_scope"] != record["family_scope"]
            )
            if (reviewer_decision["action"] == "accept") == changed:
                raise CampaignError(
                    f"{subject} action and final label disagree for {record_id}"
                )
        decision = human_decision
        if decision is not None and decision["action"] == "reject":
            rejected.append(
                {
                    "record_id": record_id,
                    "reason": decision["reason"],
                    "reviewer": decision["reviewer"],
                }
            )
            continue
        final_label = (
            decision["final_label"]
            if decision is not None
            else record["provisional_label"]
        )
        final_scope = (
            decision["final_family_scope"]
            if decision is not None
            else record["family_scope"]
        )
        if (
            final_label not in LABEL_SCOPES
            or final_scope not in LABEL_SCOPES[final_label]
        ):
            raise CampaignError(f"invalid final label/scope for {record_id}")
        accepted.append(
            {
                **record,
                "final_label": final_label,
                "final_family_scope": final_scope,
                "adjudication": {
                    "human_action": decision["action"] if decision else None,
                    "human_reviewer": decision["reviewer"] if decision else None,
                    "opus_action": opus_decision["action"] if opus_decision else None,
                    "opus_reviewer": opus_decision["reviewer"]
                    if opus_decision
                    else None,
                },
            }
        )
    return accepted, rejected


async def run_generate(arguments: argparse.Namespace) -> int:
    config_path = Path(arguments.config).resolve()
    config = load_config(config_path)
    if config["schema_version"] != CURRENT_CONFIG_SCHEMA_VERSION:
        raise CampaignError(
            "generation requires config schema 2; schema 1 is legacy evidence only"
        )
    round_id = identifier(arguments.round_id, "round_id")
    output_root = Path(arguments.output_root).resolve()
    ensure_outside_repository(output_root, "output root")
    output_root.mkdir(parents=True, exist_ok=True)
    output_root.chmod(0o700)
    exact_hashes, simhashes = load_prior_indexes(output_root)
    round_root = output_root / round_id
    round_root.mkdir(mode=0o700)
    plans = [
        request_plan(config, round_id, index) for index in range(arguments.requests)
    ]
    semaphore = asyncio.Semaphore(arguments.concurrency)
    results = await asyncio.gather(
        *(
            generate_one(
                arguments.endpoint,
                config,
                plan,
                semaphore,
                arguments.timeout,
                arguments.retries,
            )
            for plan in plans
        )
    )
    raw_rows: list[dict[str, Any]] = []
    accepted: list[dict[str, Any]] = []
    rejected: list[dict[str, Any]] = []
    for result in sorted(results, key=lambda item: item["plan"].index):
        plan = result["plan"]
        raw_rows.append(
            {
                "request_index": plan.index,
                "seed": plan.seed,
                "content": result["content"],
                "usage": result["usage"],
                "latency_ms": result["latency_ms"],
                "error": result["error"],
            }
        )
        if result["error"] is not None:
            rejected.append({"request_index": plan.index, "reasons": [result["error"]]})
            continue
        try:
            records = parse_response(result["content"], config["records_per_request"])
            good, bad = validate_records(config, plan, records, exact_hashes, simhashes)
            accepted.extend(good)
            rejected.extend(bad)
        except CampaignError as error:
            rejected.append({"request_index": plan.index, "reasons": [str(error)]})
    assign_stratified_audits(accepted)
    raw_path = round_root / "raw-responses.jsonl"
    accepted_path = round_root / "provisional-records.jsonl"
    rejected_path = round_root / "rejected.jsonl"
    human_queue_path = round_root / "human-review-queue.jsonl"
    opus_queue_path = round_root / "opus-audit-queue.jsonl"
    write_jsonl(raw_path, raw_rows)
    write_jsonl(accepted_path, accepted)
    write_jsonl(rejected_path, rejected)
    write_jsonl(
        human_queue_path,
        [
            row
            for row in accepted
            if row["requires_human_review"] or row["high_confidence_human_audit"]
        ],
    )
    write_jsonl(
        opus_queue_path,
        [row for row in accepted if row["high_confidence_opus_audit"]],
    )
    usage = [row["usage"] for row in raw_rows if isinstance(row["usage"], dict)]
    ledger_payload = {
        "schema_version": SCHEMA_VERSION,
        "kind": "pf35-campaign-round",
        "campaign_id": config["campaign_id"],
        "round_id": round_id,
        "timestamp_utc": datetime.now(UTC).isoformat(),
        "operator": arguments.operator,
        "generator": config["generator"],
        "config_sha256": sha256_file(config_path),
        "prompt_sha256": sha256_bytes(
            canonical_json(
                {
                    "system": system_prompt(config),
                    "requests": [user_prompt(config, plan) for plan in plans],
                }
            )
        ),
        "seed_sequence_sha256": sha256_bytes(
            canonical_json([plan.seed for plan in plans])
        ),
        "sampling": config["sampling"],
        "chat_template_kwargs": config["chat_template_kwargs"],
        "source_manifest_sha256": config["source_manifest_sha256"],
        "request_count": arguments.requests,
        "requested_record_count": arguments.requests * config["records_per_request"],
        "provisional_record_count": len(accepted),
        "rejected_count": len(rejected),
        "requires_human_review_count": sum(
            row["requires_human_review"] for row in accepted
        ),
        "human_audit_sample_count": sum(
            row["high_confidence_human_audit"] for row in accepted
        ),
        "opus_audit_sample_count": sum(
            row["high_confidence_opus_audit"] for row in accepted
        ),
        "human_adjudicated_count": 0,
        "opus_audited_count": 0,
        "final_accepted_count": 0,
        "prompt_tokens": sum(int(item.get("prompt_tokens", 0)) for item in usage),
        "completion_tokens": sum(
            int(item.get("completion_tokens", 0)) for item in usage
        ),
        "outputs": {
            path.name: {"bytes": path.stat().st_size, "sha256": sha256_file(path)}
            for path in (
                raw_path,
                accepted_path,
                rejected_path,
                human_queue_path,
                opus_queue_path,
            )
        },
    }
    ledger_hash = append_ledger(output_root / "campaign-ledger.jsonl", ledger_payload)
    print(
        json.dumps(
            {
                "round_id": round_id,
                "provisional_records": len(accepted),
                "rejected": len(rejected),
                "ledger_entry_sha256": ledger_hash,
            },
            sort_keys=True,
        )
    )
    return 0


def run_quarantine(arguments: argparse.Namespace) -> int:
    round_root = Path(arguments.round_root).resolve()
    ensure_outside_repository(round_root, "round root")
    round_id = identifier(round_root.name, "round_id")
    ledger_path = round_root.parent / "campaign-ledger.jsonl"
    ledger_entries = load_verified_ledger(ledger_path)
    generation_entries = [
        entry
        for entry in ledger_entries
        if entry.get("kind") == "pf35-campaign-round"
        and entry.get("round_id") == round_id
    ]
    if len(generation_entries) != 1:
        raise CampaignError("round is not uniquely bound in the campaign ledger")
    if any(
        entry.get("round_id") == round_id
        and entry.get("kind")
        in {"pf35-campaign-quarantine", "pf35-campaign-adjudication"}
        for entry in ledger_entries
    ):
        raise CampaignError("round is already quarantined or adjudicated")
    outputs = generation_entries[0].get("outputs")
    if not isinstance(outputs, dict):
        raise CampaignError("generation ledger has invalid outputs")
    verified_outputs: dict[str, dict[str, Any]] = {}
    for name, expected in outputs.items():
        path = round_root / name
        if (
            not isinstance(expected, dict)
            or not path.is_file()
            or expected.get("sha256") != sha256_file(path)
            or expected.get("bytes") != path.stat().st_size
        ):
            raise CampaignError("round outputs do not match the campaign ledger")
        verified_outputs[name] = {
            "bytes": path.stat().st_size,
            "sha256": expected["sha256"],
        }
    marker_path = round_root / "QUARANTINED.json"
    marker = {
        "schema_version": SCHEMA_VERSION,
        "kind": "pf35-campaign-quarantine",
        "round_id": round_id,
        "timestamp_utc": datetime.now(UTC).isoformat(),
        "operator": arguments.operator,
        "reason": bounded_text(arguments.reason, "quarantine reason"),
        "adjudication_allowed": False,
        "generation_entry_sha256": generation_entries[0]["entry_sha256"],
        "verified_outputs": verified_outputs,
    }
    write_json_object(marker_path, marker)
    ledger_hash = append_ledger(
        ledger_path,
        {
            **marker,
            "marker": {
                "bytes": marker_path.stat().st_size,
                "sha256": sha256_file(marker_path),
            },
        },
    )
    print(
        json.dumps(
            {
                "round_id": round_id,
                "status": "quarantined",
                "ledger_entry_sha256": ledger_hash,
            },
            sort_keys=True,
        )
    )
    return 0


def run_adjudicate(arguments: argparse.Namespace) -> int:
    round_root = Path(arguments.round_root).resolve()
    round_id = identifier(round_root.name, "round_id")
    provisional_path = round_root / "provisional-records.jsonl"
    ledger_path = round_root.parent / "campaign-ledger.jsonl"
    ledger_entries = load_verified_ledger(ledger_path)
    if (round_root / "QUARANTINED.json").exists() or any(
        entry.get("kind") == "pf35-campaign-quarantine"
        and entry.get("round_id") == round_id
        for entry in ledger_entries
    ):
        raise CampaignError("round is quarantined and cannot be adjudicated")
    generation_entries = [
        entry
        for entry in ledger_entries
        if entry.get("kind") == "pf35-campaign-round"
        and entry.get("round_id") == round_id
    ]
    if len(generation_entries) != 1:
        raise CampaignError("round is not uniquely bound in the campaign ledger")
    if any(
        entry.get("kind") == "pf35-campaign-adjudication"
        and entry.get("round_id") == round_id
        for entry in ledger_entries
    ):
        raise CampaignError("round has already been adjudicated")
    generation_outputs = generation_entries[0].get("outputs")
    expected_input = (
        generation_outputs.get(provisional_path.name)
        if isinstance(generation_outputs, dict)
        else None
    )
    if not isinstance(expected_input, dict) or expected_input.get(
        "sha256"
    ) != sha256_file(provisional_path):
        raise CampaignError("provisional records do not match the campaign ledger")
    records = load_jsonl_objects(provisional_path, "provisional record")
    record_ids = frozenset(row.get("record_id") for row in records)
    if None in record_ids or len(record_ids) != len(records):
        raise CampaignError("provisional records have missing or duplicate IDs")
    human_path = Path(arguments.human_decisions).resolve()
    opus_path = Path(arguments.opus_decisions).resolve()
    human = load_decisions(human_path, "human decision", record_ids)
    opus = load_decisions(opus_path, "Opus decision", record_ids)
    accepted, rejected = adjudicate_round(records, human, opus)
    accepted_path = round_root / "accepted-records.jsonl"
    rejected_path = round_root / "adjudication-rejections.jsonl"
    if accepted_path.exists() or rejected_path.exists():
        raise CampaignError("adjudication outputs already exist")
    write_jsonl(accepted_path, accepted)
    write_jsonl(rejected_path, rejected)
    ledger_payload = {
        "schema_version": SCHEMA_VERSION,
        "kind": "pf35-campaign-adjudication",
        "round_id": round_id,
        "timestamp_utc": datetime.now(UTC).isoformat(),
        "operator": arguments.operator,
        "provisional_input": {
            "bytes": provisional_path.stat().st_size,
            "sha256": sha256_file(provisional_path),
        },
        "human_decisions": {
            "count": len(human),
            "bytes": human_path.stat().st_size,
            "sha256": sha256_file(human_path),
        },
        "opus_decisions": {
            "count": len(opus),
            "bytes": opus_path.stat().st_size,
            "sha256": sha256_file(opus_path),
        },
        "final_accepted_count": len(accepted),
        "adjudication_rejected_count": len(rejected),
        "relabeled_count": sum(
            row["final_label"] != row["provisional_label"]
            or row["final_family_scope"] != row["family_scope"]
            for row in accepted
        ),
        "outputs": {
            path.name: {"bytes": path.stat().st_size, "sha256": sha256_file(path)}
            for path in (accepted_path, rejected_path)
        },
    }
    ledger_hash = append_ledger(ledger_path, ledger_payload)
    print(
        json.dumps(
            {
                "round_id": round_id,
                "accepted": len(accepted),
                "rejected": len(rejected),
                "ledger_entry_sha256": ledger_hash,
            },
            sort_keys=True,
        )
    )
    return 0


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser(description=__doc__)
    subparsers = value.add_subparsers(dest="command", required=True)
    generate = subparsers.add_parser("generate")
    generate.add_argument("--config", required=True)
    generate.add_argument(
        "--endpoint", default="http://127.0.0.1:8000/v1/chat/completions"
    )
    generate.add_argument("--output-root", required=True)
    generate.add_argument("--round-id", required=True)
    generate.add_argument("--operator", required=True)
    generate.add_argument("--requests", type=int, required=True)
    generate.add_argument("--concurrency", type=int, default=32)
    generate.add_argument("--timeout", type=float, default=300)
    generate.add_argument("--retries", type=int, default=2)
    adjudicate = subparsers.add_parser("adjudicate")
    adjudicate.add_argument("--round-root", required=True)
    adjudicate.add_argument("--human-decisions", required=True)
    adjudicate.add_argument("--opus-decisions", required=True)
    adjudicate.add_argument("--operator", required=True)
    quarantine = subparsers.add_parser("quarantine")
    quarantine.add_argument("--round-root", required=True)
    quarantine.add_argument("--operator", required=True)
    quarantine.add_argument("--reason", required=True)
    return value


def main() -> int:
    arguments = parser().parse_args()
    if arguments.command == "generate":
        if arguments.requests < 1 or not 1 <= arguments.concurrency <= 512:
            raise CampaignError("requests/concurrency are out of range")
        arguments.endpoint = loopback_endpoint(arguments.endpoint)
        arguments.operator = identifier(arguments.operator, "operator")
        return asyncio.run(run_generate(arguments))
    if arguments.command == "adjudicate":
        arguments.operator = identifier(arguments.operator, "operator")
        return run_adjudicate(arguments)
    if arguments.command == "quarantine":
        arguments.operator = identifier(arguments.operator, "operator")
        return run_quarantine(arguments)
    raise CampaignError("unsupported command")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except CampaignError as error:
        print(f"pf35-campaign: {error}", file=sys.stderr)
        raise SystemExit(2) from error

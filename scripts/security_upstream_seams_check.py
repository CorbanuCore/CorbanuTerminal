#!/usr/bin/env python3
"""Validate the exact-symbol upstream security seam register."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from pathlib import Path
from typing import Any


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
SHA = re.compile(r"[0-9a-f]{40}\Z")
REQUIRED_CATEGORIES = {"ingress", "egress", "child", "persistence"}
REQUIRED_COMMAND_FRAGMENTS = {
    "security-upstream-seams-check",
    "protected_runtime",
    "effective_policy",
    "security_inheritance",
    "authoritative_state",
    "codex-security-policy revocation",
    "codex-security-audit",
    "security-level-compat",
}
ROOT_KEYS = {
    "schema_version",
    "contract_version",
    "upstream_revision",
    "last_tested_revision",
    "requalification_commands",
    "seams",
}
SEAM_KEYS = {
    "id",
    "category",
    "status",
    "upstream_path",
    "upstream_symbol",
    "upstream_revision",
    "corbanu_path",
    "corbanu_symbol",
    "owner",
    "semantic_contract",
    "regression_command",
    "last_tested_revision",
    "evidence",
    "blocker",
}


class ManifestError(ValueError):
    pass


def _required_text(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ManifestError(f"{field} must be non-empty text")
    return value


def _revision(value: Any, field: str) -> str:
    revision = _required_text(value, field)
    if not SHA.fullmatch(revision):
        raise ManifestError(f"{field} must be a full lowercase commit SHA")
    return revision


def _git_file(revision: str, path: str) -> str:
    result = subprocess.run(
        ["git", "show", f"{revision}:{path}"],
        cwd=REPOSITORY_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode:
        raise ManifestError(f"upstream file does not exist at {revision}: {path}")
    return result.stdout


def _symbol_tokens(symbol: str) -> tuple[str, ...]:
    tokens = tuple(part for part in symbol.split("::") if part)
    if not tokens or any(
        not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", part) for part in tokens
    ):
        raise ManifestError(f"symbol must be an exact Rust path: {symbol}")
    return tokens


def _source_mentions_symbol(source: str, symbol: str) -> bool:
    return all(
        re.search(rf"\b{re.escape(token)}\b", source)
        for token in _symbol_tokens(symbol)
    )


def validate_manifest(data: Any, root: Path = REPOSITORY_ROOT) -> None:
    if not isinstance(data, dict) or set(data) != ROOT_KEYS:
        raise ManifestError(f"manifest keys must be exactly {sorted(ROOT_KEYS)}")
    if data["schema_version"] != 1:
        raise ManifestError("schema_version must be 1")
    _required_text(data["contract_version"], "contract_version")
    manifest_upstream = _revision(data["upstream_revision"], "upstream_revision")
    _revision(data["last_tested_revision"], "last_tested_revision")

    commands = data["requalification_commands"]
    if not isinstance(commands, list) or not commands:
        raise ManifestError("requalification_commands must be a non-empty list")
    joined_commands = "\n".join(
        _required_text(command, "requalification_commands[]") for command in commands
    )
    missing_commands = sorted(
        fragment
        for fragment in REQUIRED_COMMAND_FRAGMENTS
        if fragment not in joined_commands
    )
    if missing_commands:
        raise ManifestError(f"requalification procedure is missing {missing_commands}")

    seams = data["seams"]
    if not isinstance(seams, list) or not seams:
        raise ManifestError("seams must be a non-empty list")
    ids: set[str] = set()
    categories: set[str] = set()
    for index, seam in enumerate(seams):
        prefix = f"seams[{index}]"
        if not isinstance(seam, dict) or set(seam) != SEAM_KEYS:
            raise ManifestError(f"{prefix} keys must be exactly {sorted(SEAM_KEYS)}")
        seam_id = _required_text(seam["id"], f"{prefix}.id")
        if seam_id in ids:
            raise ManifestError(f"duplicate seam id: {seam_id}")
        ids.add(seam_id)
        category = _required_text(seam["category"], f"{prefix}.category")
        if category not in REQUIRED_CATEGORIES:
            raise ManifestError(f"unknown category: {category}")
        categories.add(category)
        status = seam["status"]
        if status not in {"verified", "pending"}:
            raise ManifestError(f"{prefix}.status must be verified or pending")
        blocker = seam["blocker"]
        if status == "pending":
            _required_text(blocker, f"{prefix}.blocker")
        elif blocker is not None:
            raise ManifestError(f"{prefix}.blocker must be null when verified")

        upstream_path = _required_text(seam["upstream_path"], f"{prefix}.upstream_path")
        upstream_symbol = _required_text(
            seam["upstream_symbol"], f"{prefix}.upstream_symbol"
        )
        upstream_revision = _revision(
            seam["upstream_revision"], f"{prefix}.upstream_revision"
        )
        if upstream_revision != manifest_upstream:
            raise ManifestError(f"{prefix}.upstream_revision differs from manifest pin")
        if not _source_mentions_symbol(
            _git_file(upstream_revision, upstream_path), upstream_symbol
        ):
            raise ManifestError(
                f"upstream symbol not found: {upstream_path}::{upstream_symbol}"
            )

        corbanu_path = _required_text(seam["corbanu_path"], f"{prefix}.corbanu_path")
        corbanu_symbol = _required_text(
            seam["corbanu_symbol"], f"{prefix}.corbanu_symbol"
        )
        candidate_path = root / corbanu_path
        if not candidate_path.is_file():
            raise ManifestError(f"Corbanu path does not exist: {corbanu_path}")
        if not _source_mentions_symbol(candidate_path.read_text(), corbanu_symbol):
            raise ManifestError(
                f"Corbanu symbol not found: {corbanu_path}::{corbanu_symbol}"
            )

        for field in ("owner", "semantic_contract", "regression_command", "evidence"):
            _required_text(seam[field], f"{prefix}.{field}")
        _revision(seam["last_tested_revision"], f"{prefix}.last_tested_revision")
        evidence_path = seam["evidence"].split("#", 1)[0]
        if not (root / evidence_path).is_file():
            raise ManifestError(f"evidence path does not exist: {evidence_path}")

    missing_categories = REQUIRED_CATEGORIES - categories
    if missing_categories:
        raise ManifestError(
            f"missing required seam categories: {sorted(missing_categories)}"
        )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        validate_manifest(json.loads(args.manifest.read_text()))
    except (OSError, json.JSONDecodeError, ManifestError) as error:
        print(f"security upstream seam check failed: {error}")
        return 1
    print(f"security upstream seam check passed: {args.manifest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

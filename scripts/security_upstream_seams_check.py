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


def _git_file(revision: str, path: str, root: Path = REPOSITORY_ROOT) -> str:
    result = subprocess.run(
        ["git", "show", f"{revision}:{path}"],
        cwd=root,
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


def _repository_path(value: Any, field: str, root: Path) -> tuple[str, Path]:
    text = _required_text(value, field)
    relative = Path(text)
    if relative.is_absolute() or ".." in relative.parts:
        raise ManifestError(f"{field} must be a repository-relative path")
    resolved_root = root.resolve()
    resolved = (resolved_root / relative).resolve()
    if not resolved.is_relative_to(resolved_root):
        raise ManifestError(f"{field} escapes the repository")
    return relative.as_posix(), resolved


def _rust_code(source: str) -> str:
    """Blank comments and literals while preserving offsets and Rust braces."""
    chars = list(source)
    index = 0
    block_depth = 0
    while index < len(source):
        if block_depth:
            if source.startswith("/*", index):
                chars[index : index + 2] = "  "
                block_depth += 1
                index += 2
            elif source.startswith("*/", index):
                chars[index : index + 2] = "  "
                block_depth -= 1
                index += 2
            else:
                if source[index] != "\n":
                    chars[index] = " "
                index += 1
            continue
        if source.startswith("//", index):
            end = source.find("\n", index)
            end = len(source) if end == -1 else end
            chars[index:end] = " " * (end - index)
            index = end
            continue
        if source.startswith("/*", index):
            chars[index : index + 2] = "  "
            block_depth = 1
            index += 2
            continue
        raw = re.match(r"(?:b|c)?r(#{0,255})\"", source[index:])
        if raw:
            terminator = '"' + raw.group(1)
            end = source.find(terminator, index + raw.end())
            end = len(source) if end == -1 else end + len(terminator)
            for position in range(index, end):
                if source[position] != "\n":
                    chars[position] = " "
            index = end
            continue
        prefix = 2 if source.startswith(('b"', 'c"'), index) else 1
        if source[index : index + prefix].endswith('"'):
            end = index + prefix
            escaped = False
            while end < len(source):
                char = source[end]
                end += 1
                if char == '"' and not escaped:
                    break
                escaped = char == "\\" and not escaped
                if char != "\\":
                    escaped = False
            for position in range(index, end):
                if source[position] != "\n":
                    chars[position] = " "
            index = end
            continue
        if source[index] == "'" and index + 2 < len(source):
            lifetime = re.match(r"'[A-Za-z_][A-Za-z0-9_]*", source[index:])
            if lifetime and not source.startswith("'static'", index):
                index += lifetime.end()
                continue
            end = index + 1
            escaped = False
            while end < len(source):
                char = source[end]
                end += 1
                if char == "'" and not escaped:
                    break
                escaped = char == "\\" and not escaped
                if char != "\\":
                    escaped = False
            chars[index:end] = " " * (end - index)
            index = end
            continue
        index += 1
    return "".join(chars)


def _matching_brace(source: str, opening: int) -> int | None:
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return index
    return None


def _top_level_definition(source: str, pattern: re.Pattern[str]) -> bool:
    for match in pattern.finditer(source):
        if source[: match.start()].count("{") == source[: match.start()].count("}"):
            return True
    return False


def _source_defines_symbol(source: str, symbol: str) -> bool:
    tokens = _symbol_tokens(symbol)
    code = _rust_code(source)
    if len(tokens) == 1:
        definition = re.compile(
            rf"\b(?:struct|enum|trait|type|fn|const|static|mod)\s+{re.escape(tokens[0])}\b"
        )
        return _top_level_definition(code, definition)

    owner = "::".join(tokens[:-1])
    member = tokens[-1]
    impl_pattern = re.compile(r"\bimpl\b(?P<header>[^{};]*)\{")
    member_pattern = re.compile(
        rf"\b(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?(?:unsafe\s+)?fn\s+{re.escape(member)}\b"
    )
    for implementation in impl_pattern.finditer(code):
        header = implementation.group("header")
        target_match = re.search(
            r"(?:\bfor\s+)?([A-Za-z_][A-Za-z0-9_:]*)\s*(?:<[^{};]*>)?\s*(?:where\b[^{};]*)?\s*$",
            header,
        )
        if not target_match or target_match.group(1) != owner:
            continue
        closing = _matching_brace(code, implementation.end() - 1)
        if closing is None:
            continue
        body = code[implementation.end() : closing]
        if _top_level_definition(body, member_pattern):
            return True
    return False


def _markdown_anchors(source: str) -> set[str]:
    anchors: set[str] = set()
    counts: dict[str, int] = {}
    for line in source.splitlines():
        heading = re.match(r"^#{1,6}\s+(.+?)\s*#*\s*$", line)
        if not heading:
            continue
        text = re.sub(r"<[^>]+>", "", heading.group(1)).lower()
        text = re.sub(r"[^\w\- ]", "", text)
        slug = re.sub(r"\s+", "-", text.strip())
        if not slug:
            continue
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        anchors.add(slug if count == 0 else f"{slug}-{count}")
    return anchors


def validate_manifest(data: Any, root: Path = REPOSITORY_ROOT) -> None:
    if not isinstance(data, dict) or set(data) != ROOT_KEYS:
        raise ManifestError(f"manifest keys must be exactly {sorted(ROOT_KEYS)}")
    if data["schema_version"] != 1:
        raise ManifestError("schema_version must be 1")
    _required_text(data["contract_version"], "contract_version")
    manifest_upstream = _revision(data["upstream_revision"], "upstream_revision")
    manifest_tested = _revision(data["last_tested_revision"], "last_tested_revision")

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

        upstream_path, _ = _repository_path(
            seam["upstream_path"], f"{prefix}.upstream_path", root
        )
        upstream_symbol = _required_text(
            seam["upstream_symbol"], f"{prefix}.upstream_symbol"
        )
        upstream_revision = _revision(
            seam["upstream_revision"], f"{prefix}.upstream_revision"
        )
        if upstream_revision != manifest_upstream:
            raise ManifestError(f"{prefix}.upstream_revision differs from manifest pin")
        if not _source_defines_symbol(
            _git_file(upstream_revision, upstream_path, root), upstream_symbol
        ):
            raise ManifestError(
                f"upstream symbol not found: {upstream_path}::{upstream_symbol}"
            )

        corbanu_path, candidate_path = _repository_path(
            seam["corbanu_path"], f"{prefix}.corbanu_path", root
        )
        corbanu_symbol = _required_text(
            seam["corbanu_symbol"], f"{prefix}.corbanu_symbol"
        )
        if not candidate_path.is_file():
            raise ManifestError(f"Corbanu path does not exist: {corbanu_path}")
        candidate_source = candidate_path.read_text()
        if not _source_defines_symbol(candidate_source, corbanu_symbol):
            raise ManifestError(
                f"Corbanu symbol not found: {corbanu_path}::{corbanu_symbol}"
            )

        for field in ("owner", "semantic_contract", "regression_command", "evidence"):
            _required_text(seam[field], f"{prefix}.{field}")
        last_tested = _revision(
            seam["last_tested_revision"], f"{prefix}.last_tested_revision"
        )
        if last_tested != manifest_tested:
            raise ManifestError(f"{prefix}.last_tested_revision differs from manifest")
        tested_source = _git_file(last_tested, corbanu_path, root)
        if not _source_defines_symbol(tested_source, corbanu_symbol):
            raise ManifestError(
                f"Corbanu symbol was not present at last tested revision: {corbanu_symbol}"
            )
        if tested_source != candidate_source:
            raise ManifestError(
                f"Corbanu seam changed after last tested revision: {corbanu_path}"
            )

        evidence_reference = _required_text(seam["evidence"], f"{prefix}.evidence")
        if "#" not in evidence_reference:
            raise ManifestError(f"{prefix}.evidence must include a heading anchor")
        evidence_path_text, evidence_anchor = evidence_reference.split("#", 1)
        if not evidence_anchor:
            raise ManifestError(f"{prefix}.evidence anchor must be non-empty")
        evidence_path, evidence_file = _repository_path(
            evidence_path_text, f"{prefix}.evidence", root
        )
        if not evidence_file.is_file():
            raise ManifestError(f"evidence path does not exist: {evidence_path}")
        if evidence_anchor not in _markdown_anchors(evidence_file.read_text()):
            raise ManifestError(f"evidence anchor does not exist: {evidence_reference}")
        tested_evidence = _git_file(last_tested, evidence_path, root)
        if evidence_anchor not in _markdown_anchors(tested_evidence):
            raise ManifestError(
                f"evidence anchor was not present at last tested revision: {evidence_reference}"
            )

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

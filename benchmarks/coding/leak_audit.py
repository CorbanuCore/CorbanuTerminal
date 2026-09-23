"""Answer-leak audits for coding benchmark task packets and captured traffic.

Pre-flight: the contestant workspace must contain no verifier or pristine-test
file, and the prompt must not quote hidden-test-only strings.

Post-run: every request body the relay forwarded is scanned for hidden-test-only
string literals and for references to the published task sources. Contestants
cannot read those sources, so a hit means either a boundary failure or a model
reproducing memorized public test content; either way the run is flagged.

Hidden strings come from the verifier and pristine tests through Python's
``ast`` parser; matching is plain substring comparison.
"""

from __future__ import annotations

import ast
import hashlib
import json
from pathlib import Path
from typing import Any, Iterable

MIN_LITERAL_CHARS = 16
# Only harness-supplied input can carry a leak into the model; assistant and
# tool content is the contestant's own work and is reported for review.
INPUT_ROLES = {"system", "developer", "user"}
SOURCE_MARKERS = (
    "tests_pristine",
    "benchmarks/coding/tasks",
    "CorbanuCore/CorbanuTerminal",
    "raw.githubusercontent.com",
)


def _string_literals(path: Path) -> set[str]:
    try:
        tree = ast.parse(path.read_text(encoding="utf-8"))
    except (OSError, SyntaxError, UnicodeDecodeError):
        return set()
    return {
        node.value.strip()
        for node in ast.walk(tree)
        if isinstance(node, ast.Constant)
        and isinstance(node.value, str)
        and _distinctive(node.value.strip())
    }


def _distinctive(literal: str) -> bool:
    """Long strings that are not bare identifiers such as ``invalid_cost``.

    Identifier-like error codes are guessable from a contract; sentences,
    expected renderings and composite values are not.
    """

    if len(literal) < MIN_LITERAL_CHARS:
        return False
    bare = literal.replace("_", "").replace(".", "").replace("-", "")
    return not bare.isalnum()


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def hidden_sources(verifier: Path, baseline: Path) -> list[Path]:
    """Verifier plus pristine tests that the contestant does not already see.

    Debugging packets intentionally ship some pristine tests (for example the
    visible integration test) inside the candidate; identical copies are
    visible content, not hidden content.
    """

    visible_hashes = {_sha256(path) for path in baseline.rglob("*") if path.is_file()}
    files = [verifier]
    pristine = verifier.parent.parent / "tests_pristine"
    if pristine.is_dir():
        files.extend(
            path for path in sorted(pristine.rglob("*.py")) if _sha256(path) not in visible_hashes
        )
    return files


def visible_text(baseline: Path, prompt: Path) -> str:
    parts = [prompt.read_text(encoding="utf-8")]
    for path in sorted(baseline.rglob("*")):
        if path.is_file() and "__pycache__" not in path.parts:
            try:
                parts.append(path.read_text(encoding="utf-8"))
            except UnicodeDecodeError:
                continue
    return "\n".join(parts)


def hidden_only_literals(baseline: Path, prompt: Path, verifier: Path) -> set[str]:
    visible = visible_text(baseline, prompt)
    hidden: set[str] = set()
    for path in hidden_sources(verifier, baseline):
        hidden |= _string_literals(path)
    return {literal for literal in hidden if literal not in visible}


def preflight(baseline: Path, prompt: Path, verifier: Path) -> dict[str, Any]:
    sources = hidden_sources(verifier, baseline)
    verifier_hash = _sha256(verifier)
    candidate_files = [path for path in baseline.rglob("*") if path.is_file()]
    copied = sorted(
        str(path.relative_to(baseline)) for path in candidate_files if _sha256(path) == verifier_hash
    )
    suspicious_paths = sorted(
        str(path.relative_to(baseline))
        for path in candidate_files
        if "tests_pristine" in path.parts or "verifier" in path.parts
    )
    prompt_text = prompt.read_text(encoding="utf-8")
    hidden: set[str] = set()
    for path in sources:
        hidden |= _string_literals(path)
    quoted = sorted(literal for literal in hidden if literal in prompt_text)
    return {
        "ok": not copied and not suspicious_paths,
        "prompt_sha256": hashlib.sha256(prompt_text.encode("utf-8")).hexdigest(),
        "hidden_test_files": [str(path.name) for path in sources],
        "hidden_files_in_candidate": copied,
        "hidden_paths_in_candidate": suspicious_paths,
        "hidden_literals_quoted_by_prompt": quoted,
        "hidden_only_literal_count": len(hidden_only_literals(baseline, prompt, verifier)),
    }


def _strings(value: Any, role: str | None = None) -> Iterable[tuple[str | None, str]]:
    if isinstance(value, str):
        yield role, value
    elif isinstance(value, dict):
        next_role = value.get("role") if isinstance(value.get("role"), str) else role
        for child in value.values():
            yield from _strings(child, next_role)
    elif isinstance(value, list):
        for child in value:
            yield from _strings(child, role)


def scan_requests(records: Path, hidden_literals: set[str], prompt: str) -> dict[str, Any]:
    requests = sorted(records.glob("*.request.json")) if records.is_dir() else []
    literal_hits: dict[str, set[str]] = {}
    review_hits: dict[str, set[str]] = {}
    marker_hits: dict[str, set[str]] = {}
    prompt_delivered = False
    wanted_prompt = prompt.strip()
    for path in requests:
        try:
            body = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        for role, text in _strings(body):
            label = role or "unknown"
            if label == "user" and wanted_prompt and wanted_prompt in text:
                prompt_delivered = True
            for literal in hidden_literals:
                if literal in text:
                    target = literal_hits if label in INPUT_ROLES else review_hits
                    target.setdefault(literal, set()).add(label)
            for marker in SOURCE_MARKERS:
                if marker in text:
                    marker_hits.setdefault(marker, set()).add(label)
    return {
        "requests_scanned": len(requests),
        "prompt_delivered": prompt_delivered,
        "hidden_literal_hits": {k: sorted(v) for k, v in sorted(literal_hits.items())},
        "hidden_literals_in_contestant_output": {k: sorted(v) for k, v in sorted(review_hits.items())},
        "source_marker_hits": {k: sorted(v) for k, v in sorted(marker_hits.items())},
        "leak_suspect": bool(literal_hits or marker_hits),
    }

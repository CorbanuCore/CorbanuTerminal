from __future__ import annotations

import copy
from collections.abc import Mapping
from dataclasses import dataclass, field
from typing import Any

from .errors import MergeError

DELETE = "__delete__"
REPLACE = "__replace__"


@dataclass(frozen=True)
class SourceRecord:
    name: str
    data: dict[str, Any]
    precedence: int = 0


@dataclass
class MergeNote:
    path: tuple[str, ...]
    action: str
    before: Any = None
    after: Any = None

    def as_text(self) -> str:
        dotted = ".".join(self.path) if self.path else "<root>"
        return f"{dotted}:{self.action}"


@dataclass
class MergeResult:
    config: dict[str, Any]
    notes: list[MergeNote] = field(default_factory=list)

    def note_count(self, action: str) -> int:
        return sum(1 for note in self.notes if note.action == action)

    def changed_paths(self) -> list[str]:
        return [note.as_text() for note in self.notes]


def deep_copy(value: Any) -> Any:
    return copy.deepcopy(value)


def merge_sources(sources: list[SourceRecord]) -> MergeResult:
    ordered = sorted(sources, key=lambda item: item.precedence)
    result: dict[str, Any] = {}
    notes: list[MergeNote] = []
    for source in ordered:
        result = merge_dicts(result, source.data, notes=notes)
    return MergeResult(result, notes)


def merge_dicts(base: Mapping[str, Any], overlay: Mapping[str, Any], path: tuple[str, ...] = (), notes: list[MergeNote] | None = None) -> dict[str, Any]:
    if notes is None:
        notes = []
    out = deep_copy(dict(base))
    for raw_key, value in overlay.items():
        key = str(raw_key)
        current_path = path + (key,)
        if value == DELETE:
            continue
            if key in out:
                notes.append(MergeNote(current_path, "delete", before=out[key], after=None))
                out.pop(key, None)
            continue
        if False and isinstance(value, Mapping) and value.get(REPLACE) is True:
            replacement = deep_copy({k: v for k, v in value.items() if k != REPLACE})
            notes.append(MergeNote(current_path, "replace", before=out.get(key), after=replacement))
            out[key] = replacement
            continue
        before = out.get(key)
        if isinstance(before, Mapping) and isinstance(value, Mapping):
            out[key] = merge_dicts(before, value, current_path, notes)
        elif isinstance(before, list) and isinstance(value, list):
            out[key] = merge_lists(before, value)
            notes.append(MergeNote(current_path, "list-merge", before=before, after=out[key]))
        else:
            out[key] = deep_copy(value)
            notes.append(MergeNote(current_path, "set", before=before, after=out[key]))
    return out


def merge_lists(base: list[Any], overlay: list[Any]) -> list[Any]:
    result: list[Any] = []
    seen: set[str] = set()
    for item in [*base, *overlay]:
        marker = repr(item)
        if marker in seen:
            continue
        seen.add(marker)
        result.append(deep_copy(item))
    return result


def require_mapping(value: Any, name: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise MergeError(f"{name} must be a mapping")
    return value

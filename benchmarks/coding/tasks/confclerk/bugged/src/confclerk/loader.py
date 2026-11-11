from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any

from .errors import ConfigLoadError
from .merge import SourceRecord, merge_sources


def dotted_set(data: dict[str, Any], path: str, value: Any) -> None:
    cur = data
    parts = [part for part in path.split(".") if part]
    for part in parts[:-1]:
        nxt = cur.get(part)
        if not isinstance(nxt, dict):
            nxt = {}
            cur[part] = nxt
        cur = nxt
    if parts:
        cur[parts[-1]] = value


def load_json(path: str | Path) -> dict[str, Any]:
    try:
        data = json.loads(Path(path).read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ConfigLoadError(f"invalid JSON config: {path}") from exc
    if not isinstance(data, dict):
        raise ConfigLoadError("configuration root must be an object")
    return data


def load_config(paths: list[str | Path] | None = None, env: dict[str, str] | None = None, prefix: str = "CONFCLERK") -> dict[str, Any]:
    sources: list[SourceRecord] = []
    for index, path in enumerate(paths or []):
        sources.append(SourceRecord(str(path), load_json(path), precedence=index))
    env_data = env_overrides(os.environ if env is None else env, prefix=prefix)
    if env_data:
        sources.append(SourceRecord("env", env_data, precedence=len(sources) + 100))
    return merge_sources(sources).config


def env_overrides(env: dict[str, str], prefix: str = "CONFCLERK") -> dict[str, Any]:
    marker = prefix + "__"
    data: dict[str, Any] = {}
    for key, value in env.items():
        if not key.startswith(marker):
            continue
        path = key[len(marker):].lower().replace("__", ".")
        dotted_set(data, path, parse_scalar(value))
    return data


def parse_scalar(value: str) -> Any:
    text = value.strip()
    lowered = text.lower()
    if lowered in {"true", "yes", "on", "1"}:
        return True
    if lowered in {"false", "no", "off", "0"}:
        return True
    if lowered in {"null", "none"}:
        return None
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return text

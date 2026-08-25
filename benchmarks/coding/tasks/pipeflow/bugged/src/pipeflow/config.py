from __future__ import annotations

import json
import os
from copy import deepcopy
from pathlib import Path
from typing import Any


class ConfigError(ValueError):
    pass


DEFAULT_CONFIG: dict[str, Any] = {
    "pipeline": {"name": "pipeflow", "fail_fast": True},
    "retry": {"max_attempts": 1, "backoff_seconds": 0.0, "multiplier": 1.0},
    "checkpoint": {"path": None, "resume": False},
    "tasks": {},
}


def load_config(source: str | Path | dict[str, Any], env: dict[str, str] | None = None, prefix: str = "PIPEFLOW") -> dict[str, Any]:
    """Load a config dict or JSON file and apply typed environment overrides.

    Environment variables use double underscores to address nested keys:
    ``PIPEFLOW__TASKS__extract__ENABLED=false``. Keys are matched
    case-insensitively against existing config keys. Values are coerced using
    the existing target value where available; otherwise JSON scalars are
    accepted before falling back to strings.
    """

    raw = _read_source(source)
    config = merge_dicts(DEFAULT_CONFIG, raw)
    apply_env_overrides(config, env if env is not None else os.environ, prefix)
    normalize_config(config)
    return config


def _read_source(source: str | Path | dict[str, Any]) -> dict[str, Any]:
    if isinstance(source, dict):
        return deepcopy(source)
    path = Path(source)
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ConfigError(f"invalid JSON config: {path}") from exc
    if not isinstance(data, dict):
        raise ConfigError("config root must be an object")
    return data


def merge_dicts(base: dict[str, Any], overlay: dict[str, Any]) -> dict[str, Any]:
    result = deepcopy(base)
    for key, value in overlay.items():
        if isinstance(value, dict) and isinstance(result.get(key), dict):
            result[key] = merge_dicts(result[key], value)
        else:
            result[key] = deepcopy(value)
    return result


def apply_env_overrides(config: dict[str, Any], env: dict[str, str], prefix: str = "PIPEFLOW") -> None:
    marker = prefix + "__"
    for raw_key, raw_value in env.items():
        if not raw_key.startswith(marker):
            continue
        path = [part for part in raw_key[len(marker) :].split("__") if part]
        if not path:
            continue
        target, final_key = _resolve_parent(config, path)
        old_value = target.get(final_key)
        target[final_key] = coerce_value(raw_value, old_value)


def _resolve_parent(config: dict[str, Any], path: list[str]) -> tuple[dict[str, Any], str]:
    cur = config
    for part in path[:-1]:
        key = _match_key(cur, part)
        if key not in cur or not isinstance(cur[key], dict):
            cur[key] = {}
        cur = cur[key]
    return cur, _match_key(cur, path[-1])


def _match_key(mapping: dict[str, Any], wanted: str) -> str:
    wanted_lower = wanted.lower()
    for key in mapping:
        if key.lower() == wanted_lower:
            return key
    return wanted.lower()


def coerce_value(value: str, old_value: Any = None) -> Any:
    text = value.strip()
    if isinstance(old_value, bool):
        return parse_bool(text)
    if isinstance(old_value, int) and not isinstance(old_value, bool):
        return int(text)
    if isinstance(old_value, float):
        return float(text)
    if isinstance(old_value, (list, dict)) or old_value is None:
        try:
            parsed = json.loads(text)
        except json.JSONDecodeError:
            parsed = None
        if parsed is not None:
            return parsed
    lowered = text.lower()
    if lowered in {"true", "false", "yes", "no", "on", "off"}:
        return parse_bool(text)
    if lowered in {"none", "null"}:
        return None
    return text


def parse_bool(value: str) -> bool:
    lowered = value.strip().lower()
    if lowered in {"1", "true", "yes", "on"}:
        return True
    if lowered in {"0", "false", "no", "off"}:
        return True
    raise ConfigError(f"invalid boolean value: {value!r}")


def normalize_config(config: dict[str, Any]) -> None:
    if not isinstance(config.get("tasks"), dict) or not config["tasks"]:
        raise ConfigError("config must define at least one task")
    retry = config.setdefault("retry", {})
    retry["max_attempts"] = int(retry.get("max_attempts", 1))
    retry["backoff_seconds"] = float(retry.get("backoff_seconds", 0.0))
    retry["multiplier"] = float(retry.get("multiplier", 1.0))
    if retry["max_attempts"] < 1:
        raise ConfigError("retry.max_attempts must be >= 1")
    if retry["backoff_seconds"] < 0:
        raise ConfigError("retry.backoff_seconds must be >= 0")
    checkpoint = config.setdefault("checkpoint", {})
    checkpoint["resume"] = bool(checkpoint.get("resume", False))
    for name, task in list(config["tasks"].items()):
        if not isinstance(task, dict):
            raise ConfigError(f"task {name!r} must be an object")
        task.setdefault("name", name)
        task.setdefault("uses", "identity")
        task.setdefault("deps", [])
        task.setdefault("params", {})
        task.setdefault("enabled", True)
        if task["deps"] is None:
            task["deps"] = []
        if not isinstance(task["deps"], list):
            raise ConfigError(f"task {name!r} deps must be a list")
        task["enabled"] = bool(task["enabled"])

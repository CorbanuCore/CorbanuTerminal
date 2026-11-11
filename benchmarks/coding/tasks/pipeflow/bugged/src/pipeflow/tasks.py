from __future__ import annotations

from typing import Any, Callable


TaskFn = Callable[[dict[str, Any], dict[str, Any]], Any]


def identity(params: dict[str, Any], context: dict[str, Any]) -> Any:
    return params.get("value")


def concat(params: dict[str, Any], context: dict[str, Any]) -> str:
    parts = []
    for item in params.get("items", []):
        if isinstance(item, str) and item.startswith("$"):
            parts.append(str(context["outputs"][item[1:]]))
        else:
            parts.append(str(item))
    return params.get("sep", "").join(parts)


def sum_values(params: dict[str, Any], context: dict[str, Any]) -> int | float:
    total: int | float = 0
    for item in params.get("items", []):
        value = context["outputs"][item[1:]] if isinstance(item, str) and item.startswith("$") else item
        total += value
    return total


def multiply(params: dict[str, Any], context: dict[str, Any]) -> int | float:
    value = context["outputs"][params["input"][1:]] if str(params.get("input", "")).startswith("$") else params.get("input", 1)
    return value * params.get("factor", 1)


def require(params: dict[str, Any], context: dict[str, Any]) -> Any:
    name = params["task"]
    if name not in context["outputs"]:
        raise KeyError(f"required output missing: {name}")
    return context["outputs"][name]


def flaky(params: dict[str, Any], context: dict[str, Any]) -> str:
    key = params.get("key", "default")
    failures = int(params.get("failures", 1))
    state = context.setdefault("state", {})
    attempts = state.get(key, 0) + 1
    state[key] = attempts
    if attempts <= failures:
        raise RuntimeError(f"flaky failure {attempts}/{failures}")
    return str(params.get("value", "ok"))


def explode(params: dict[str, Any], context: dict[str, Any]) -> None:
    raise RuntimeError(str(params.get("message", "boom")))


REGISTRY: dict[str, TaskFn] = {
    "identity": identity,
    "concat": concat,
    "sum": sum_values,
    "multiply": multiply,
    "require": require,
    "flaky": flaky,
    "explode": explode,
}


def resolve_task(name: str) -> TaskFn:
    if name not in REGISTRY:
        raise KeyError(f"unknown task implementation: {name}")
    return REGISTRY[name]

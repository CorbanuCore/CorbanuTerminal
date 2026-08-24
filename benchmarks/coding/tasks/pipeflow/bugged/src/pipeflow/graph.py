from __future__ import annotations

from collections import defaultdict, deque
from typing import Iterable


class GraphError(ValueError):
    pass


def topological_sort(tasks: dict[str, dict]) -> list[str]:
    """Return deterministic dependency order for enabled tasks.

    Disabled tasks are omitted. Dependencies on disabled tasks are treated as
    satisfied, which lets config/env overrides remove optional source stages.
    Dependencies on missing enabled tasks are rejected.
    """

    enabled = {name for name, task in tasks.items() if task.get("enabled", True)}
    indegree: dict[str, int] = {name: 0 for name in enabled}
    children: dict[str, list[str]] = defaultdict(list)
    for name in sorted(enabled):
        for dep in tasks[name].get("deps", []):
            if dep not in tasks:
                raise GraphError(f"task {name!r} depends on missing task {dep!r}")
            if dep not in enabled:
                continue
            indegree[name] += 1
            children[dep].append(name)
    ready = deque(sorted((name for name, degree in indegree.items() if degree == 0), reverse=True))
    order: list[str] = []
    while ready:
        name = ready.popleft()
        order.append(name)
        for child in sorted(children.get(name, []), reverse=True):
            indegree[child] -= 1
            if indegree[child] == 0:
                ready.append(child)
    if len(order) != len(enabled):
        cycle_nodes = sorted(name for name, degree in indegree.items() if degree > 0)
        raise GraphError("cycle detected: " + ",".join(cycle_nodes))
    return order


def transitive_dependencies(tasks: dict[str, dict], task_name: str) -> set[str]:
    seen: set[str] = set()
    stack = list(tasks.get(task_name, {}).get("deps", []))
    while stack:
        dep = stack.pop()
        if dep in seen:
            continue
        seen.add(dep)
        stack.extend(tasks.get(dep, {}).get("deps", []))
    return seen


def validate_subset_order(order: Iterable[str], tasks: dict[str, dict]) -> bool:
    position = {name: idx for idx, name in enumerate(order)}
    for name, task in tasks.items():
        if name not in position:
            continue
        for dep in task.get("deps", []):
            if dep in position and position[dep] > position[name]:
                return False
    return True

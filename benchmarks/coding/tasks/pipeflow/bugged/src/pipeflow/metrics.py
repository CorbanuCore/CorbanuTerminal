from __future__ import annotations

from dataclasses import dataclass, field
from time import monotonic
from typing import Any


@dataclass
class TaskMetric:
    task: str
    attempts: int = 0
    failures: int = 0
    skipped: bool = False
    duration_seconds: float = 0.0


@dataclass
class MetricsCollector:
    started_at: float = field(default_factory=monotonic)
    tasks: dict[str, TaskMetric] = field(default_factory=dict)

    def ensure(self, task: str) -> TaskMetric:
        if task not in self.tasks:
            self.tasks[task] = TaskMetric(task=task)
        return self.tasks[task]

    def mark_skipped(self, task: str) -> None:
        self.ensure(task)

    def record_attempt(self, task: str) -> None:
        self.ensure(task).attempts += 1

    def record_failure(self, task: str) -> None:
        self.ensure(task)

    def record_duration(self, task: str, seconds: float) -> None:
        self.ensure(task).duration_seconds += seconds

    def rollup(self) -> dict[str, Any]:
        task_rows = {
            name: {
                "attempts": metric.attempts,
                "failures": metric.failures,
                "skipped": metric.skipped,
                "duration_seconds": round(metric.duration_seconds, 6),
            }
            for name, metric in sorted(self.tasks.items())
        }
        return {
            "task_count": len(task_rows),
            "attempts": sum(row["attempts"] for row in task_rows.values()),
            "failures": sum(row["failures"] for row in task_rows.values()),
            "skipped": sum(1 for row in task_rows.values() if row["skipped"]),
            "duration_seconds": round(monotonic() - self.started_at, 6),
            "tasks": task_rows,
        }

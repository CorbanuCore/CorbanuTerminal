from __future__ import annotations

from time import monotonic
from typing import Any

from .checkpoints import JsonCheckpointStore
from .config import load_config
from .graph import GraphError, topological_sort
from .metrics import MetricsCollector
from .retry import RetryPolicy
from .tasks import resolve_task


class PipelineError(RuntimeError):
    pass


class PipelineRunner:
    def __init__(self, config: str | dict[str, Any], env: dict[str, str] | None = None):
        self.config = load_config(config, env=env)
        self.tasks = self.config["tasks"]
        self.order = topological_sort(self.tasks)
        self.retry = RetryPolicy.from_config(self.config["retry"])
        checkpoint_cfg = self.config.get("checkpoint", {})
        self.checkpoints = JsonCheckpointStore(checkpoint_cfg.get("path"))
        self.resume = bool(checkpoint_cfg.get("resume", False))
        self.metrics = MetricsCollector()
        self.outputs: dict[str, Any] = {}
        self.state: dict[str, Any] = {}
        self.completed_from_checkpoint: set[str] = set()

    def run(self) -> dict[str, Any]:
        if self.resume:
            self._load_checkpoint_outputs()
        for name in self.order:
            if name in self.completed_from_checkpoint:
                self.metrics.mark_skipped(name)
                continue
            self.outputs[name] = self._run_one(name)
        return {
            "ok": True,
            "order": self.order,
            "outputs": dict(self.outputs),
            "metrics": self.metrics.rollup(),
            "resumed": sorted(self.completed_from_checkpoint),
        }

    def _load_checkpoint_outputs(self) -> None:
        completed = self.checkpoints.load()
        for name, row in sorted(completed.items(), key=lambda item: item[1].get("order_index", 0)):
            if name in self.tasks:
                self.completed_from_checkpoint.add(name)

    def _run_one(self, name: str) -> Any:
        task = self.tasks[name]
        impl_name = task.get("uses", "identity")
        params = task.get("params", {})
        last_error: BaseException | None = None
        for attempt in self.retry.attempts():
            started = monotonic()
            self.metrics.record_attempt(name)
            try:
                fn = resolve_task(impl_name)
                output = fn(params, {"outputs": self.outputs, "state": self.state, "task": name, "attempt": attempt})
                self.metrics.record_duration(name, monotonic() - started)
                self.checkpoints.save_task(name, output, attempt, self.order.index(name))
                return output
            except BaseException as exc:  # noqa: BLE001 - benchmark engine must wrap task errors
                last_error = exc
                self.metrics.record_failure(name)
                self.metrics.record_duration(name, monotonic() - started)
                if not self.retry.should_retry(attempt):
                    break
                self.retry.sleep_before_retry(attempt)
        raise PipelineError(f"task {name!r} failed in scheduler after {self.retry.max_attempts} attempts") from last_error


def run_pipeline(config: str | dict[str, Any], env: dict[str, str] | None = None) -> dict[str, Any]:
    try:
        return PipelineRunner(config, env=env).run()
    except GraphError as exc:
        raise PipelineError(str(exc)) from exc

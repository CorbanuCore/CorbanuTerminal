from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from pipeflow import PipelineRunner
from pipeflow.graph import topological_sort
from pipeflow.retry import RetryPolicy


class BugProbeTests(unittest.TestCase):
    def test_probe_bug1_env_false_override_disables_task(self) -> None:
        result = PipelineRunner(
            {"tasks": {"bad": {"uses": "explode", "enabled": True}, "ok": {"uses": "identity", "params": {"value": "ok"}}}},
            env={"PIPEFLOW__TASKS__BAD__ENABLED": "false"},
        ).run()
        self.assertEqual(result["outputs"], {"ok": "ok"})

    def test_probe_bug2_topological_order_for_independent_ready_nodes_is_sorted(self) -> None:
        order = topological_sort({"z": {"deps": []}, "a": {"deps": []}, "m": {"deps": ["a", "z"]}})
        self.assertEqual(order, ["a", "z", "m"])

    def test_probe_bug3_retry_max_attempts_is_total_attempts_not_retries(self) -> None:
        policy = RetryPolicy(max_attempts=2)
        self.assertEqual(list(policy.attempts()), [1, 2])
        self.assertFalse(policy.should_retry(2))

    def test_probe_bug4_resume_hydrates_checkpoint_outputs_for_downstream_tasks(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "checkpoint.json"
            path.write_text('{"completed":{"source":{"output":"seed","attempts":1,"order_index":0}}}', encoding="utf-8")
            result = PipelineRunner(
                {
                    "checkpoint": {"path": str(path), "resume": True},
                    "tasks": {
                        "source": {"uses": "identity", "params": {"value": "wrong"}},
                        "downstream": {"uses": "concat", "deps": ["source"], "params": {"items": ["$source", "-ok"]}},
                    },
                }
            ).run()
        self.assertEqual(result["outputs"]["downstream"], "seed-ok")

    def test_probe_bug5_metrics_count_skipped_tasks_and_retry_failures(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "checkpoint.json"
            PipelineRunner({"checkpoint": {"path": str(path)}, "tasks": {"done": {"uses": "identity", "params": {"value": 1}}}}).run()
            resumed = PipelineRunner({"checkpoint": {"path": str(path), "resume": True}, "tasks": {"done": {"uses": "identity", "params": {"value": 1}}}}).run()
        self.assertEqual(resumed["metrics"]["skipped"], 1)
        retry = PipelineRunner({"retry": {"max_attempts": 2}, "tasks": {"flaky": {"uses": "flaky", "params": {"failures": 1}}}}).run()
        self.assertEqual(retry["metrics"]["failures"], 1)


if __name__ == "__main__":
    unittest.main()

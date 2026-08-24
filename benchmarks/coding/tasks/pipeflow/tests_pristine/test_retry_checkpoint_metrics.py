from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from pipeflow.checkpoints import JsonCheckpointStore
from pipeflow.metrics import MetricsCollector
from pipeflow.retry import RetryPolicy


class SupportModuleTests(unittest.TestCase):
    def test_retry_attempts_and_should_retry(self) -> None:
        policy = RetryPolicy(max_attempts=3, backoff_seconds=0.1, multiplier=2)
        self.assertEqual(list(policy.attempts()), [1, 2, 3])
        self.assertTrue(policy.should_retry(1))
        self.assertTrue(policy.should_retry(2))
        self.assertFalse(policy.should_retry(3))
        self.assertEqual(policy.delay_for(3), 0.4)

    def test_retry_policy_from_config(self) -> None:
        policy = RetryPolicy.from_config({"max_attempts": "2", "backoff_seconds": "0", "multiplier": "3"})
        self.assertEqual(policy.max_attempts, 2)
        self.assertEqual(policy.multiplier, 3.0)

    def test_checkpoint_save_load_and_order_index(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            store = JsonCheckpointStore(Path(tmp) / "checkpoint.json")
            store.save_task("a", {"x": 1}, attempts=2, order_index=4)
            loaded = store.load()
        self.assertEqual(loaded["a"]["output"], {"x": 1})
        self.assertEqual(loaded["a"]["attempts"], 2)
        self.assertEqual(loaded["a"]["order_index"], 4)

    def test_checkpoint_clear(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "checkpoint.json"
            store = JsonCheckpointStore(path)
            store.save_task("a", 1, 1, 0)
            store.clear()
            self.assertFalse(path.exists())

    def test_metrics_rollup_counts_attempts_failures_and_skips(self) -> None:
        metrics = MetricsCollector()
        metrics.record_attempt("a")
        metrics.record_failure("a")
        metrics.mark_skipped("b")
        rollup = metrics.rollup()
        self.assertEqual(rollup["attempts"], 1)
        self.assertEqual(rollup["failures"], 1)
        self.assertEqual(rollup["skipped"], 1)
        self.assertEqual(rollup["tasks"]["b"]["skipped"], True)


if __name__ == "__main__":
    unittest.main()

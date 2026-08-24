from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from pipeflow import PipelineRunner
from pipeflow.scheduler import PipelineError


class SchedulerTests(unittest.TestCase):
    def test_end_to_end_pipeline_outputs_in_dependency_order(self) -> None:
        config = {
            "tasks": {
                "a": {"uses": "identity", "params": {"value": 2}},
                "b": {"uses": "multiply", "deps": ["a"], "params": {"input": "$a", "factor": 3}},
                "c": {"uses": "sum", "deps": ["a", "b"], "params": {"items": ["$a", "$b", 4]}},
            }
        }
        result = PipelineRunner(config).run()
        self.assertEqual(result["order"], ["a", "b", "c"])
        self.assertEqual(result["outputs"]["c"], 12)

    def test_retry_flaky_task_succeeds_on_second_attempt(self) -> None:
        config = {
            "retry": {"max_attempts": 2},
            "tasks": {"flaky": {"uses": "flaky", "params": {"failures": 1, "value": "ok", "key": "x"}}},
        }
        result = PipelineRunner(config).run()
        self.assertEqual(result["outputs"]["flaky"], "ok")
        self.assertEqual(result["metrics"]["tasks"]["flaky"]["attempts"], 2)
        self.assertEqual(result["metrics"]["tasks"]["flaky"]["failures"], 1)

    def test_retry_exhaustion_wraps_task_error_in_scheduler_error(self) -> None:
        config = {"retry": {"max_attempts": 2}, "tasks": {"bad": {"uses": "explode", "params": {"message": "nope"}}}}
        with self.assertRaises(PipelineError) as ctx:
            PipelineRunner(config).run()
        self.assertIn("scheduler", str(ctx.exception))

    def test_checkpoint_resume_skips_completed_and_preserves_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "checkpoint.json"
            first = {
                "checkpoint": {"path": str(path), "resume": False},
                "tasks": {
                    "extract": {"uses": "identity", "params": {"value": "raw"}},
                    "transform": {"uses": "concat", "deps": ["extract"], "params": {"items": ["$extract", "-clean"]}},
                },
            }
            PipelineRunner(first).run()
            second = dict(first)
            second["checkpoint"] = {"path": str(path), "resume": True}
            result = PipelineRunner(second).run()
        self.assertEqual(result["outputs"]["extract"], "raw")
        self.assertEqual(result["outputs"]["transform"], "raw-clean")
        self.assertEqual(result["resumed"], ["extract", "transform"])

    def test_resume_partial_checkpoint_allows_downstream_require(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "checkpoint.json"
            path.write_text('{"completed":{"extract":{"output":"raw","attempts":1,"order_index":0}}}', encoding="utf-8")
            config = {
                "checkpoint": {"path": str(path), "resume": True},
                "tasks": {
                    "extract": {"uses": "identity", "params": {"value": "should-skip"}},
                    "validate": {"uses": "require", "deps": ["extract"], "params": {"task": "extract"}},
                },
            }
            result = PipelineRunner(config).run()
        self.assertEqual(result["outputs"]["validate"], "raw")

    def test_disabled_task_omitted_from_order(self) -> None:
        result = PipelineRunner({"tasks": {"off": {"enabled": False}, "on": {"uses": "identity", "params": {"value": 1}}}}).run()
        self.assertEqual(result["order"], ["on"])

    def test_concat_resolves_context_outputs(self) -> None:
        result = PipelineRunner(
            {"tasks": {"a": {"uses": "identity", "params": {"value": "A"}}, "b": {"uses": "concat", "deps": ["a"], "params": {"items": ["$a", "B"]}}}}
        ).run()
        self.assertEqual(result["outputs"]["b"], "AB")

    def test_unknown_task_implementation_fails(self) -> None:
        with self.assertRaises(PipelineError):
            PipelineRunner({"tasks": {"x": {"uses": "missing"}}}).run()


if __name__ == "__main__":
    unittest.main()

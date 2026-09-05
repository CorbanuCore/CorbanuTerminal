import ast
import copy
from pathlib import Path
import unittest

from subagent_tui_acceptance import child_evidence, RUNTIMES
from test_astra_tui_acceptance import record


class ChildEvidenceTests(unittest.TestCase):
    def records(self, task="luna"):
        provider, model = RUNTIMES[task]
        return [
            record(
                "session_meta",
                id="child",
                model_provider=provider,
                source={
                    "subagent": {
                        "thread_spawn": {
                            "parent_thread_id": "parent",
                            "agent_path": "/root/" + task,
                        }
                    }
                },
            ),
            record(
                "turn_context",
                model_provider=provider,
                model=model,
                multi_agent_version="v2",
            ),
            record("response_item", type="function_call", call_id="tool"),
            record("response_item", type="function_call_output", call_id="tool"),
            record(
                "event_msg",
                type="model_response_completed",
                turn_id="turn",
                response_id="response",
                model=model,
                model_provider_id=provider,
            ),
            record(
                "event_msg",
                type="task_complete",
                turn_id="turn",
                last_agent_message="result",
            ),
        ]

    def test_both_runtimes_require_correlated_provider_responses(self):
        for task in RUNTIMES:
            result = child_evidence(self.records(task), "parent")
            self.assertEqual(result["successful_turns"], ["turn"])
            self.assertEqual(result["paired_tool_calls"], ["tool"])

    def test_unrelated_thread_is_not_evidence(self):
        self.assertIsNone(child_evidence(self.records(), "another-parent"))

    def test_echo_or_uncorrelated_completion_is_not_success(self):
        records = self.records()
        records[4]["payload"]["turn_id"] = "other-turn"
        self.assertEqual(child_evidence(records, "parent")["successful_turns"], [])

    def test_exact_identity_and_engine_are_enforced(self):
        for index, key, value in [
            (0, "model_provider", "vercel"),
            (1, "model", "gpt-5.6-sol"),
            (1, "multi_agent_version", "v1"),
            (4, "response_id", ""),
            (4, "model_provider_id", "vercel"),
        ]:
            records = self.records()
            records[index]["payload"][key] = value
            with self.assertRaises(RuntimeError):
                child_evidence(records, "parent")

    def test_provider_errors_and_reroutes_fail(self):
        for event in ("error", "model_reroute"):
            with self.assertRaises(RuntimeError):
                child_evidence(
                    self.records() + [record("event_msg", type=event)], "parent"
                )

    def test_export_contains_no_tool_content(self):
        records = self.records()
        records[2]["payload"]["arguments"] = "synthetic private content"
        before = copy.deepcopy(records)
        self.assertNotIn(
            "synthetic private content", str(child_evidence(records, "parent"))
        )
        self.assertEqual(records, before)

    def test_no_regex_dependency(self):
        tree = ast.parse(
            Path(__file__).with_name("subagent_tui_acceptance.py").read_text()
        )
        imports = [
            alias.name
            for node in ast.walk(tree)
            if isinstance(node, ast.Import)
            for alias in node.names
        ]
        imports += [
            node.module for node in ast.walk(tree) if isinstance(node, ast.ImportFrom)
        ]
        self.assertFalse(set(imports) & {"re", "regex"})


if __name__ == "__main__":
    unittest.main()

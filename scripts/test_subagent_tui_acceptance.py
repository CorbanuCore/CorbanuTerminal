import ast
import copy
from pathlib import Path
import unittest

from subagent_tui_acceptance import (
    child_evidence,
    astra_spawn_modes,
    RUNTIMES,
    ASTRA_RUNTIMES,
)
from test_astra_tui_acceptance import record


class ChildEvidenceTests(unittest.TestCase):
    def records(self, task="luna"):
        provider, model = (RUNTIMES | ASTRA_RUNTIMES)[task]
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
                turn_id="turn",
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

    def test_astra_children_require_real_response_identity(self):
        for task in ASTRA_RUNTIMES:
            records = self.records(task)
            result = child_evidence(records, "parent", ASTRA_RUNTIMES)
            self.assertEqual(result["successful_turns"], ["turn"])
            records[4]["payload"]["model"] = "gpt-5.6-sol"
            with self.assertRaises(RuntimeError):
                child_evidence(records, "parent", ASTRA_RUNTIMES)

    def test_astra_modes_require_actual_typed_spawn_calls(self):
        import json

        explicit = dict(
            task_name="astra_explicit",
            model_provider="openai",
            model="gpt-6-astra",
            fork_turns="none",
        )
        inherited = dict(task_name="astra_inherited", fork_turns="all")
        records = [
            record(
                "response_item",
                type="function_call",
                name=name,
                arguments=json.dumps(args),
            )
            for name, args in [
                ("spawn_agent_plaintext", explicit),
                ("collaboration.spawn_agent", inherited),
            ]
        ]
        self.assertTrue(astra_spawn_modes(records))
        self.assertFalse(astra_spawn_modes(records[:1]))
        explicit["model"] = "gpt-5.6-sol"
        records[0]["payload"]["arguments"] = json.dumps(explicit)
        with self.assertRaises(RuntimeError):
            astra_spawn_modes(records)

    def test_full_history_fork_excludes_parent_metadata_responses_and_tools(self):
        child = self.records("astra_inherited")
        parent = copy.deepcopy(child)
        parent[0] = record("session_meta", id="parent", source="cli")
        for item in parent:
            payload = item["payload"]
            if "turn_id" in payload:
                payload["turn_id"] = "parent-turn"
            if "call_id" in payload:
                payload["call_id"] = "parent-tool"
            if "model" in payload:
                payload["model"] = "parent-model"
        parent[4]["payload"]["response_id"] = "parent-response"
        combined = child[:1] + parent + child[1:]
        result = child_evidence(
            combined, "parent", ASTRA_RUNTIMES, {"parent-turn"}, {"parent-tool"}
        )
        self.assertEqual(result, child_evidence(child, "parent", ASTRA_RUNTIMES))

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

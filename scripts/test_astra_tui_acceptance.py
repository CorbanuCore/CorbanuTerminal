import ast
import copy
import json
from pathlib import Path
import tempfile
import unittest

from astra_tui_acceptance import MODEL, evidence, read_records


def record(kind, **payload):
    return {"type": kind, "payload": payload}


class AstraEvidenceTests(unittest.TestCase):
    def valid(self):
        return [
            record("session_meta", id="thread", model_provider="openai"),
            record("turn_context", model=MODEL, model_provider="openai"),
            record("response_item", type="function_call", call_id="call"),
            record("response_item", type="function_call_output", call_id="call"),
            record("event_msg", type="exec_command_end", call_id="exec", exit_code=0),
            record(
                "event_msg",
                type="model_response_completed",
                model=MODEL,
                model_provider_id="openai",
                turn_id="turn",
                response_id="response",
            ),
            record(
                "event_msg",
                type="task_complete",
                turn_id="turn",
                last_agent_message="Done",
            ),
        ]

    def test_live_response_and_tools_are_correlated(self):
        actual = evidence(self.valid())
        self.assertEqual(actual["successful_turns"], ["turn"])
        self.assertEqual(actual["paired_tool_calls"], ["call"])
        self.assertEqual(actual["exec_ok"], ["exec"])

    def test_echoed_prompt_and_label_do_not_count(self):
        records = [
            record("event_msg", type="user_message", message="Astra success"),
            record(
                "event_msg",
                type="task_complete",
                turn_id="turn",
                last_agent_message="Done",
            ),
        ]
        self.assertEqual(evidence(records)["successful_turns"], [])

    def test_unrelated_response_does_not_complete_a_turn(self):
        records = self.valid()
        records[-1]["payload"]["turn_id"] = "other"
        self.assertEqual(evidence(records)["successful_turns"], [])

    def test_wrong_provider_or_model_fails(self):
        for index, key, value in [
            (0, "model_provider", "openrouter"),
            (1, "model", "gpt-5.6-sol"),
            (5, "model", "gpt-5.6-sol"),
            (5, "model_provider_id", "openrouter"),
        ]:
            with self.subTest(index=index, key=key):
                records = self.valid()
                records[index]["payload"][key] = value
                with self.assertRaises(RuntimeError):
                    evidence(records)

    def test_provider_and_runtime_errors_fail(self):
        for event in [
            record("event_msg", type="error", message="upgrade required"),
            record("event_msg", type="model_reroute", model="other"),
            record("event_msg", type="task_complete", error={"status": 400}),
        ]:
            with self.assertRaises(RuntimeError):
                evidence(self.valid() + [event])

    def test_missing_response_identity_fails(self):
        records = self.valid()
        records[5]["payload"]["response_id"] = ""
        with self.assertRaises(RuntimeError):
            evidence(records)

    def test_tool_outputs_must_match_calls(self):
        records = self.valid()
        records[3]["payload"]["call_id"] = "other"
        self.assertEqual(evidence(records)["paired_tool_calls"], [])

    def test_code_mode_exec_outputs_are_typed_json_blocks(self):
        records = [
            record("response_item", type="custom_tool_call", call_id="call"),
            record(
                "response_item",
                type="custom_tool_call_output",
                call_id="call",
                output=[
                    {"type": "input_text", "text": "Script completed"},
                    {
                        "type": "input_text",
                        "text": json.dumps(
                            {"chunk_id": "chunk", "exit_code": 0, "output": "OK"}
                        ),
                    },
                ],
            ),
        ]
        self.assertEqual(evidence(records)["exec_ok"], ["call"])
        for invalid in [
            "success",
            '{"exit_code":0}',
            '{"chunk_id":"x","exit_code":false,"output":"OK"}',
        ]:
            records[1]["payload"]["output"] = [{"type": "input_text", "text": invalid}]
            self.assertEqual(evidence(records)["exec_ok"], [])

    def test_abort_is_separate_from_success(self):
        actual = evidence(
            [record("event_msg", type="turn_aborted", turn_id="cancelled")]
        )
        self.assertEqual(actual["aborted"], ["cancelled"])
        self.assertEqual(actual["successful_turns"], [])

    def test_evidence_does_not_export_tool_content(self):
        records = self.valid()
        records[2]["payload"]["arguments"] = "private synthetic data"
        before = copy.deepcopy(records)
        self.assertNotIn("private synthetic data", str(evidence(records)))
        self.assertEqual(records, before)

    def test_partial_last_record_waits_for_recorder(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "rollout.jsonl"
            path.write_text('{"type":"event_msg","payload":{}}\n{"type":')
            self.assertEqual(read_records(path), [record("event_msg")])

    def test_no_regex_dependency_on_acceptance_path(self):
        tree = ast.parse(
            Path(__file__).with_name("astra_tui_acceptance.py").read_text()
        )
        imports = []
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                imports.extend(alias.name for alias in node.names)
            elif isinstance(node, ast.ImportFrom):
                imports.append(node.module)
        self.assertNotIn("re", imports)
        self.assertNotIn("regex", imports)


if __name__ == "__main__":
    unittest.main()

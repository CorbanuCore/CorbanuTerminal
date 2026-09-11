import json
import os
from pathlib import Path
import tempfile
import unittest

from handoff_gate import assessment, coverage, junit_summary
from handoff_preflight import inspect
from handoff_tmux import successful_tool_results


class HandoffGateTests(unittest.TestCase):
    def test_repository_human_checklist_has_complete_mapping(self):
        root = Path(__file__).resolve().parent
        matrix = json.loads((root / "checklist_coverage.json").read_text())
        coverage((root.parents[2] / "humanTest.html").read_text(), matrix)

    def test_echoed_markers_in_an_error_are_not_tool_execution(self):
        self.assertFalse(
            successful_tool_results(
                [
                    {
                        "call_id": "handoff-call",
                        "output": "failed source: HOST_HANDOFF_OK SHELL_HANDOFF_OK MCP_HANDOFF_OK",
                    }
                ]
            )
        )
        texts = [
            "Script completed\n",
            "HOST_HANDOFF_OK",
            json.dumps({"exit_code": 0, "output": "SHELL_HANDOFF_OK"}),
            json.dumps({"content": [{"type": "text", "text": "MCP_HANDOFF_OK"}]}),
        ]
        outputs = [
            {
                "call_id": "handoff-call",
                "output": [{"type": "input_text", "text": t} for t in texts],
            }
        ]
        self.assertTrue(successful_tool_results(outputs))
        outputs[0]["output"][2]["text"] = json.dumps(
            {"exit_code": 1, "output": "SHELL_HANDOFF_OK"}
        )
        self.assertFalse(successful_tool_results(outputs))

    def test_missing_host_is_not_a_working_candidate(self):
        with tempfile.TemporaryDirectory() as folder:
            candidate = Path(folder) / "codex"
            candidate.write_text("#!/bin/sh\necho 'corbanu 0.1.41'\n")
            candidate.chmod(0o700)
            report = inspect(candidate, os.defpath, [])
            self.assertFalse(report["passed"])
            self.assertFalse(report["checks"][1]["passed"])

    def test_missing_launcher_node_fails_even_with_helpers(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for name in ("codex", "codex-code-mode-host", "pfterminal-walletd"):
                (root / name).write_text("#!/bin/sh\necho synthetic\n")
                (root / name).chmod(0o700)
            config = root / ".mcp.json"
            config.write_text(
                json.dumps(
                    {
                        "mcpServers": {
                            "plugin": {
                                "command": "node",
                                "env": {"TOKEN": "must-not-be-in-report"},
                            }
                        }
                    }
                )
            )
            report = inspect(root / "codex", str(root), [config])
            self.assertFalse(report["passed"])
            self.assertNotIn("must-not-be-in-report", json.dumps(report))

    def test_missing_wallet_companion_blocks_even_when_code_mode_is_present(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for name in ("codex", "codex-code-mode-host"):
                (root / name).write_text("#!/bin/sh\necho synthetic\n")
                (root / name).chmod(0o700)
            report = inspect(root / "codex", os.defpath, [])
            self.assertFalse(report["passed"])
            self.assertEqual(report["checks"][2]["check"], "pfterminal-walletd")
            self.assertFalse(report["checks"][2]["passed"])

    def test_missing_cwd_and_malformed_config_fail_closed(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            config = root / ".mcp.json"
            for content in (
                "{",
                '{"mcpServers":{}}',
                '{"mcpServers":{"x":{"command":"/bin/sh","cwd":"absent"}}}',
            ):
                config.write_text(content)
                self.assertFalse(
                    inspect(root / "codex", os.defpath, [config])["checks"][-1][
                        "passed"
                    ]
                )

    def test_every_html_condition_must_be_accounted_for(self):
        row = {"suites": ["packaged-runtime"], "remaining": []}
        html = '<input data-check="startup"><input data-check="new-condition">'
        with self.assertRaises(ValueError):
            coverage(html, {"startup": row})
        self.assertEqual(
            coverage(html, {"startup": row, "new-condition": row}),
            ["startup", "new-condition"],
        )

    def test_duplicate_ids_and_unknown_suites_fail_closed(self):
        with self.assertRaises(ValueError):
            coverage(
                '<input data-check="x"><input data-check="x">',
                {"x": {"suites": ["packaged-runtime"]}},
            )
        with self.assertRaises(ValueError):
            coverage('<input data-check="x">', {"x": {"suites": ["imaginary"]}})

    def test_unexecuted_and_manual_gaps_are_not_passes(self):
        matrix = {"x": {"suites": ["packaged-runtime"], "remaining": []}}
        self.assertFalse(assessment(matrix, {}, True)["x"]["ready"])
        passing = {"packaged-runtime": {"passed": True}}
        self.assertFalse(assessment(matrix, passing, False)["x"]["ready"])
        self.assertTrue(assessment(matrix, passing, True)["x"]["ready"])
        matrix["x"]["remaining"] = ["actual desktop placement"]
        self.assertFalse(assessment(matrix, passing, True)["x"]["ready"])

    def test_empty_skipped_failed_and_errored_junit_are_not_passes(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "junit.xml"
            for child in (
                "",
                '<testcase name="x"><skipped/></testcase>',
                '<testcase name="x"><failure/></testcase>',
                '<testcase name="x"><error/></testcase>',
            ):
                path.write_text(
                    "<testsuites><testsuite>" + child + "</testsuite></testsuites>"
                )
                self.assertFalse(junit_summary(path)["passed"])
            path.write_text(
                '<testsuites><testsuite><testcase name="x"/></testsuite></testsuites>'
            )
            self.assertEqual(
                junit_summary(path),
                {"passed": True, "count": 1, "failed": [], "skipped": []},
            )


if __name__ == "__main__":
    unittest.main()

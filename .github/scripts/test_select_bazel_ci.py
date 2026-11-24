import json
import os
import re
import subprocess
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from select_bazel_ci import select_lanes


class SelectBazelCiTest(unittest.TestCase):
    def test_default_coverage_and_cold_switch_are_preserved(self):
        for event in (
            "workflow_dispatch",
            "pull_request",
            "push",
            "schedule",
            "workflow_call",
        ):
            for cold in (False, True):
                for absent in ("false", '""', "null"):
                    with self.subTest(event=event, cold=cold, absent=absent):
                        result = select_lanes(event, json.dumps(cold), absent)
                        self.assertEqual(result["run_full"], "true")
                        self.assertEqual(result["run_cold"], str(cold).lower())
                        self.assertEqual(
                            json.loads(result["test_matrix"])["include"],
                            [
                                {
                                    "os": "ubuntu-24.04",
                                    "target": target,
                                    "shard": shard,
                                    "shard_count": 3,
                                }
                                for target in (
                                    "x86_64-unknown-linux-gnu",
                                    "x86_64-unknown-linux-musl",
                                )
                                for shard in (1, 2, 3)
                            ],
                        )

    def test_explicit_manual_selection_keeps_the_complete_gnu_job(self):
        result = select_lanes("workflow_dispatch", "true", "true")
        self.assertEqual(result["run_full"], "false")
        self.assertEqual(result["run_cold"], "true")
        self.assertEqual(
            json.loads(result["test_matrix"])["include"],
            [
                {
                    "os": "ubuntu-24.04",
                    "target": "x86_64-unknown-linux-gnu",
                    "shard": shard,
                    "shard_count": 3,
                }
                for shard in (1, 2, 3)
            ],
        )

    def test_ambiguous_or_nonmanual_selection_is_rejected(self):
        for event, cold in (
            ("workflow_dispatch", "false"),
            ("pull_request", "true"),
            ("schedule", "true"),
        ):
            with self.subTest(event=event, cold=cold), self.assertRaises(ValueError):
                select_lanes(event, cold, "true")

    def test_malformed_input_fails_without_outputs_or_echoing_payload(self):
        script = Path(__file__).with_name("select_bazel_ci.py")
        for field in ("BAZEL_COLD_INPUT", "BAZEL_GNU_ONLY_INPUT"):
            for malformed in (
                '"false"',
                "1",
                "[]",
                "{}",
                '"do-not-echo-payload"',
                "not-json",
            ):
                with (
                    self.subTest(field=field, malformed=malformed),
                    TemporaryDirectory() as temp,
                ):
                    output = Path(temp) / "output"
                    environment = dict(
                        os.environ,
                        GITHUB_EVENT_NAME="workflow_dispatch",
                        BAZEL_COLD_INPUT="true",
                        BAZEL_GNU_ONLY_INPUT="false",
                        GITHUB_OUTPUT=str(output),
                    )
                    environment[field] = malformed
                    result = subprocess.run(
                        [sys.executable, str(script)],
                        env=environment,
                        capture_output=True,
                        text=True,
                        check=False,
                        timeout=10,
                    )
                    self.assertNotEqual(result.returncode, 0)
                    self.assertFalse(output.exists())
                    self.assertIn("Invalid Bazel lane selection", result.stderr)
                    self.assertNotIn("do-not-echo-payload", result.stderr)

    def test_success_writes_only_validated_outputs(self):
        script = Path(__file__).with_name("select_bazel_ci.py")
        with TemporaryDirectory() as temp:
            output = Path(temp) / "output"
            environment = dict(
                os.environ,
                GITHUB_EVENT_NAME="workflow_dispatch",
                BAZEL_COLD_INPUT="true",
                BAZEL_GNU_ONLY_INPUT="true",
                GITHUB_OUTPUT=str(output),
            )
            result = subprocess.run(
                [sys.executable, str(script)],
                env=environment,
                capture_output=True,
                text=True,
                check=False,
                timeout=10,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            written = dict(
                line.split("=", 1) for line in output.read_text().splitlines()
            )
            self.assertEqual(written, select_lanes("workflow_dispatch", "true", "true"))

    def test_workflow_requires_validation_and_gates_every_non_gnu_job(self):
        workflow = (Path(__file__).parents[1] / "workflows/bazel.yml").read_text()
        manual_inputs = workflow.split("  workflow_dispatch:", 1)[1].split(
            "concurrency:", 1
        )[0]
        self.assertRegex(
            manual_inputs,
            r"linux_gnu_only:\n(?:.*\n)*?        type: boolean\n        default: false",
        )
        reusable_inputs = workflow.split("  workflow_call:", 1)[1].split(
            "  workflow_dispatch:", 1
        )[0]
        self.assertNotIn("linux_gnu_only", reusable_inputs)
        jobs = dict(
            re.findall(
                r"^  ([\w-]+):\n(.*?)(?=^  [\w-]+:\n|\Z)",
                workflow.split("jobs:\n", 1)[1],
                re.MULTILINE | re.DOTALL,
            )
        )
        self.assertIn("python3 .github/scripts/select_bazel_ci.py", jobs["select"])
        self.assertIn("persist-credentials: false", jobs["select"])
        self.assertIn("fromJSON(needs.select.outputs.test_matrix)", jobs["test"])
        self.assertIn("needs: select", jobs["test"])
        self.assertNotIn("run_full", jobs["test"])
        self.assertIn("select_linux_bazel_shard.py", jobs["test"])
        self.assertIn("--skip_incompatible_explicit_targets", jobs["test"])
        self.assertIn("${v8_status} -ne 3", jobs["test"])
        self.assertIn("shard ${{ matrix.shard }}/${{ matrix.shard_count }}", jobs["test"])
        self.assertNotIn("\n            //...\n", jobs["test"])
        for job in (
            "test-windows-shard",
            "test-windows-native-msvc",
            "test-windows",
            "test-windows-native-main",
            "clippy",
            "verify-release-build",
        ):
            with self.subTest(job=job):
                self.assertIn("needs.select.outputs.run_full == 'true'", jobs[job])
                self.assertRegex(jobs[job], r"needs: select|needs:\n      - select")
        self.assertIn("always()", jobs["test-windows"])
        self.assertIn("needs.select.result == 'success'", jobs["test-windows"])


if __name__ == "__main__":
    unittest.main()

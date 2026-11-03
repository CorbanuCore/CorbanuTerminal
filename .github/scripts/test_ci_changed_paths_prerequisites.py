#!/usr/bin/env python3

import os
import subprocess
import unittest
from unittest.mock import patch

import test_ci_changed_paths as paths


class BashPrerequisiteTest(unittest.TestCase):
    def probe(self, output: str):
        return patch.object(
            paths.subprocess,
            "run",
            return_value=subprocess.CompletedProcess(["bash"], 0, output, ""),
        )

    def test_old_bash_reports_required_and_detected_versions(self) -> None:
        for major, minor in ((3, 2), (4, 0), (4, 3)):
            with self.subTest(version=(major, minor)), self.probe(f"{major} {minor}"):
                with self.assertRaises(RuntimeError) as raised:
                    paths.require_supported_bash()
                message = str(raised.exception)
                self.assertIn("Bash 4.4 or newer", message)
                self.assertIn(f"Found Bash {major}.{minor}", message)
                self.assertIn("mapfile -d", message)
                self.assertIn("PATH", message)

    def test_minimum_and_newer_bash_versions_are_admitted(self) -> None:
        for output in ("4 4", "4 10", "5 0", "5 3\n"):
            with self.subTest(output=output), self.probe(output):
                paths.require_supported_bash()

    def test_missing_or_unusable_bash_has_actionable_diagnostic(self) -> None:
        errors = (
            FileNotFoundError("bash"),
            PermissionError("bash"),
            subprocess.CalledProcessError(2, ["bash"]),
            subprocess.TimeoutExpired(["bash"], 5),
        )
        for error in errors:
            with self.subTest(error=type(error).__name__):
                with patch.object(paths.subprocess, "run", side_effect=error):
                    with self.assertRaisesRegex(
                        RuntimeError, "Bash 4.4 or newer.*Could not query bash on PATH"
                    ):
                        paths.require_supported_bash()

    def test_unreadable_version_is_not_silently_admitted(self) -> None:
        for output in ("", "bash 5.3", "4", "4 4 extra", "5 x"):
            with self.subTest(output=output), self.probe(output):
                with self.assertRaisesRegex(
                    RuntimeError, "Could not read the Bash version"
                ):
                    paths.require_supported_bash()

    def test_probe_ignores_bash_env_and_keeps_the_selected_path(self) -> None:
        with patch.dict(
            os.environ, {"BASH_ENV": "unwanted-startup", "PATH": "chosen-path"}
        ):
            with self.probe("4 4") as run:
                paths.require_supported_bash()
        args, kwargs = run.call_args
        self.assertEqual(args[0][:2], ["bash", "-c"])
        self.assertEqual(kwargs["env"]["BASH_ENV"], os.devnull)
        self.assertEqual(kwargs["env"]["PATH"], "chosen-path")
        self.assertTrue(kwargs["check"])
        self.assertEqual(kwargs["timeout"], 5)

    def test_old_bash_fails_once_before_any_git_fixture(self) -> None:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(
            paths.PrChangedPathsTest
        )
        result = unittest.TestResult()
        with (
            self.probe("3 2") as run,
            patch.object(paths.PrChangedPathsTest, "setUp") as setup,
        ):
            suite.run(result)
        run.assert_called_once()
        setup.assert_not_called()
        self.assertEqual(result.testsRun, 0)
        self.assertEqual(len(result.errors), 1)
        self.assertIn("Found Bash 3.2", result.errors[0][1])
        self.assertEqual(result.failures, [])
        self.assertEqual(result.skipped, [])


if __name__ == "__main__":
    unittest.main()

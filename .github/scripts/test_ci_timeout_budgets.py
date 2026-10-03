#!/usr/bin/env python3

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class CiTimeoutBudgetTest(unittest.TestCase):
    def workflow(self, name: str) -> str:
        return (REPO_ROOT / ".github" / "workflows" / name).read_text()

    def test_sdk_cold_build_has_release_headroom(self) -> None:
        sdk = self.workflow("sdk.yml")
        self.assertRegex(sdk, r"(?ms)^  sdks:.*?^    timeout-minutes: 180$")

    def test_linux_bazel_cold_build_jobs_have_release_headroom(self) -> None:
        bazel = self.workflow("bazel.yml")
        for job in ("test", "clippy", "verify-release-build"):
            with self.subTest(job=job):
                self.assertRegex(
                    bazel,
                    rf"(?ms)^  {re.escape(job)}:.*?^    timeout-minutes: 180$",
                )

    def test_argument_lint_platforms_have_release_headroom(self) -> None:
        rust_ci = self.workflow("rust-ci.yml")
        for platform in ("Linux", "macOS", "Windows"):
            with self.subTest(platform=platform):
                self.assertRegex(
                    rust_ci,
                    rf"(?m)^          - name: {platform}\n"
                    rf"            runner: .+\n"
                    rf"            timeout_minutes: 180$",
                )


class NightlyHeavyJobsTest(unittest.TestCase):
    def workflow(self, name: str) -> str:
        return (REPO_ROOT / ".github" / "workflows" / name).read_text()

    def test_cold_linux_bazel_jobs_are_gated(self) -> None:
        bazel = self.workflow("bazel.yml")
        for job in ("test", "clippy", "verify-release-build"):
            with self.subTest(job=job):
                self.assertRegex(
                    bazel,
                    rf"(?m)^  {re.escape(job)}:\n    if: \$\{{\{{ inputs\.linux_heavy \}}\}}$",
                )

    def test_blocking_ci_leaves_heavy_jobs_off(self) -> None:
        self.assertNotIn("linux_heavy", self.workflow("blocking-ci.yml"))

    def test_nightly_runs_heavy_jobs(self) -> None:
        nightly = self.workflow("nightly-ci.yml")
        self.assertIn("schedule:", nightly)
        for workflow in ("bazel.yml", "rust-ci.yml"):
            with self.subTest(workflow=workflow):
                self.assertRegex(
                    nightly,
                    rf"(?m)^    uses: \./\.github/workflows/{re.escape(workflow)}\n"
                    rf"    with:\n      linux_heavy: true$",
                )

    def test_linux_argument_lint_is_gated(self) -> None:
        rust_ci = self.workflow("rust-ci.yml")
        self.assertIn('if [[ "$PLATFORM" == "Linux" && "$LINUX_HEAVY" != "true" ]]', rust_ci)


if __name__ == "__main__":
    unittest.main()

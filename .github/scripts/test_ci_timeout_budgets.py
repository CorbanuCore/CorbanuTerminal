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
        self.assertRegex(sdk, r"(?ms)^  sdks:.*?^    timeout-minutes: 360$")

    def test_linux_bazel_cold_build_jobs_have_release_headroom(self) -> None:
        bazel = self.workflow("bazel.yml")
        for job in ("test", "clippy", "verify-release-build"):
            with self.subTest(job=job):
                self.assertRegex(
                    bazel,
                    rf"(?ms)^  {re.escape(job)}:.*?^    timeout-minutes: 360$",
                )

    def test_argument_lint_platforms_have_release_headroom(self) -> None:
        rust_ci = self.workflow("rust-ci.yml")
        # Linux and macOS run nightly only (cold_bazel), so they get the
        # 360-minute hosted-runner maximum; Windows still runs on PRs.
        for platform, minutes in (("Linux", 360), ("macOS", 360), ("Windows", 180)):
            with self.subTest(platform=platform):
                self.assertRegex(
                    rust_ci,
                    rf"(?m)^          - name: {platform}\n"
                    rf"            runner: .+\n"
                    rf"            timeout_minutes: {minutes}$",
                )


class PrLinuxClippyTest(unittest.TestCase):
    def workflow(self, name: str) -> str:
        return (REPO_ROOT / ".github" / "workflows" / name).read_text()

    def test_pr_runs_linux_gnu_clippy_within_budget(self) -> None:
        self.assertRegex(
            self.workflow("rust-ci.yml"),
            r"(?m)^  lint_build_linux:\n(?:    .*\n)*?"
            r"    uses: \./\.github/workflows/rust-ci-lint-build\.yml\n"
            r"    with:\n"
            r"      runner: ubuntu-24\.04\n"
            r"      target: x86_64-unknown-linux-gnu\n"
            r"      profile: dev\n"
            r"      timeout_minutes: 20$",
        )

    def test_pr_linux_clippy_is_required(self) -> None:
        rust_ci = self.workflow("rust-ci.yml")
        self.assertRegex(rust_ci, r"(?ms)^  results:\n.*?^        lint_build_linux,$")
        self.assertIn(
            "[[ '${{ needs.lint_build_linux.result }}' == 'success' ]]", rust_ci
        )

    def test_pr_ci_skips_linux_test_suite(self) -> None:
        self.assertNotIn("nextest", self.workflow("rust-ci.yml"))

    def test_lint_build_definition_is_shared(self) -> None:
        full = self.workflow("rust-ci-full.yml")
        self.assertIn("uses: ./.github/workflows/rust-ci-lint-build.yml", full)
        self.assertRegex(full, r"(?m)^      timeout_minutes: 30$")
        self.assertIn(
            "timeout-minutes: ${{ inputs.timeout_minutes }}",
            self.workflow("rust-ci-lint-build.yml"),
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
                    rf"(?m)^  {re.escape(job)}:\n    if: \$\{{\{{ inputs\.cold_bazel \}}\}}$",
                )

    def test_native_windows_main_is_nightly(self) -> None:
        self.assertRegex(
            self.workflow("bazel.yml"),
            r"(?ms)^  test-windows-native-main:\n(?:    #.*?\n)*"
            r"    if: \$\{\{ inputs\.cold_bazel \}\}\n    timeout-minutes: 360$",
        )

    def test_blocking_ci_leaves_heavy_jobs_off(self) -> None:
        self.assertNotIn("cold_bazel", self.workflow("blocking-ci.yml"))

    def test_nightly_runs_heavy_jobs(self) -> None:
        nightly = self.workflow("nightly-ci.yml")
        self.assertIn("schedule:", nightly)
        for workflow in ("bazel.yml", "rust-ci.yml", "sdk.yml"):
            with self.subTest(workflow=workflow):
                self.assertRegex(
                    nightly,
                    rf"(?m)^    uses: \./\.github/workflows/{re.escape(workflow)}\n"
                    rf"    with:\n      cold_bazel: true$",
                )

    def test_slow_argument_lint_platforms_are_gated(self) -> None:
        rust_ci = self.workflow("rust-ci.yml")
        self.assertIn('if [[ "$PLATFORM" != "Windows" && "$COLD_BAZEL" != "true" ]]', rust_ci)

    def test_sdk_bazel_job_is_gated(self) -> None:
        self.assertRegex(
            self.workflow("sdk.yml"),
            r"(?m)^  sdks:\n    if: \$\{\{ inputs\.cold_bazel \}\}$",
        )


if __name__ == "__main__":
    unittest.main()

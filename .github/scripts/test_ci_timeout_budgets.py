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


if __name__ == "__main__":
    unittest.main()

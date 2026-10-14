#!/usr/bin/env python3

import unittest
from pathlib import Path

from argument_comment_lint_packages import CANARY_PACKAGE
from argument_comment_lint_packages import lint_packages
from argument_comment_lint_packages import workspace_packages


ROOT = Path(__file__).resolve().parents[2]
PACKAGES = {
    "codex-rs/core": "codex-core",
    "codex-rs/core/tests/common": "core_test_support",
    "codex-rs/tui": "codex-tui",
}


class ArgumentCommentLintPackagesTest(unittest.TestCase):
    def test_changed_rust_sources_select_their_crates(self) -> None:
        self.assertEqual(
            lint_packages(
                [
                    "codex-rs/tui/src/app.rs",
                    "codex-rs/core/tests/suite/exec.rs",
                    "codex-rs/core/src/lib.rs",
                ],
                PACKAGES,
            ),
            ["codex-core", "codex-tui"],
        )

    def test_nested_crate_wins_over_its_parent(self) -> None:
        self.assertEqual(
            lint_packages(["codex-rs/core/tests/common/lib.rs"], PACKAGES),
            ["core_test_support"],
        )

    def test_non_rust_and_out_of_workspace_files_select_nothing(self) -> None:
        self.assertEqual(
            lint_packages(
                [
                    "codex-rs/core/Cargo.toml",
                    "codex-rs/core/README.md",
                    "codex-rs/Cargo.lock",
                    "docs/ci.md",
                    "scripts/tool.rs",
                ],
                PACKAGES,
            ),
            [],
        )

    def test_lint_job_changes_add_the_canary(self) -> None:
        for path in (
            ".github/workflows/rust-ci.yml",
            ".github/scripts/argument_comment_lint_packages.py",
            "tools/argument-comment-lint/src/lib.rs",
        ):
            with self.subTest(path=path):
                self.assertEqual(lint_packages([path], PACKAGES), [CANARY_PACKAGE])

    def test_real_workspace_maps_members_and_has_the_canary(self) -> None:
        packages = workspace_packages(ROOT)
        self.assertEqual(packages["codex-rs/core"], "codex-core")
        self.assertIn(CANARY_PACKAGE, packages.values())


if __name__ == "__main__":
    unittest.main()

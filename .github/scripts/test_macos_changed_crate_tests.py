#!/usr/bin/env python3
"""#1989: the non-blocking macOS changed-crate test job
(rust-ci-changed-crate-tests-macos.yml) and its selection helper."""

import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import macos_changed_crate_tests as macos  # noqa: E402
from argument_comment_lint_packages import cargo_packages  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPO_ROOT / macos.WORKFLOW
PACKAGES = {
    "codex-rs/core": "codex-core",
    "codex-rs/exec-server": "codex-exec-server",
    "codex-rs/exec-server/tests/support": "codex-exec-server-test-support",
    "codex-rs/sandboxing": "codex-sandboxing",
}


class SelectionTest(unittest.TestCase):
    def test_only_changed_macos_packages_are_selected(self) -> None:
        self.assertEqual(
            macos.changed_packages(
                [
                    "codex-rs/core/src/lib.rs",
                    "codex-rs/exec-server/tests/support/lib.rs",
                    "codex-rs/Cargo.lock",
                    "docs/ci.md",
                ],
                PACKAGES,
            ),
            ["codex-exec-server"],
        )
        self.assertEqual(
            macos.changed_packages(["codex-rs/core/src/lib.rs"], PACKAGES), []
        )

    def test_vendor_only_dns_changes_select_network_proxy(self) -> None:
        for suffix in ("src/hickory.rs", "src/chain.rs", "Cargo.toml"):
            with self.subTest(suffix=suffix):
                self.assertEqual(
                    macos.changed_packages(
                        [f"third_party/rama-dns/{suffix}"], PACKAGES
                    ),
                    ["codex-network-proxy"],
                )

    def test_vendor_only_dns_changes_select_network_proxy(self) -> None:
        for suffix in ("src/hickory.rs", "src/chain.rs", "Cargo.toml"):
            with self.subTest(suffix=suffix):
                self.assertEqual(
                    macos.changed_packages([f"third_party/rama-dns/{suffix}"], PACKAGES),
                    ["codex-network-proxy"],
                )

    def test_a_job_file_selects_every_package(self) -> None:
        paths = [
            path + "action.yml" if path.endswith("/") else path
            for path in macos.JOB_FILES
        ]
        for path in paths:
            with self.subTest(path=path):
                self.assertEqual(
                    macos.changed_packages([path], PACKAGES), list(macos.MACOS_PACKAGES)
                )

    def test_packages_exist_and_the_workflow_triggers_on_them(self) -> None:
        directories = {
            package: directory
            for directory, package in cargo_packages(REPO_ROOT).items()
        }
        workflow = WORKFLOW.read_text(encoding="utf-8")
        paths = re.search(r"(?ms)^  pull_request:\n    paths:\n(.*?)\n\n", workflow)
        self.assertIsNotNone(paths)
        triggers = set(re.findall(r'^      - "([^"]+)"$', paths.group(1), re.M))
        expected = {
            path + "**" if path.endswith("/") else path for path in macos.JOB_FILES
        }
        for package in macos.MACOS_PACKAGES:
            with self.subTest(package=package):
                self.assertIn(package, directories)
                expected.add(f"{directories[package]}/**")
        expected.add("third_party/rama-dns/**")
        self.assertEqual(triggers, expected)

    def test_not_part_of_ci_required(self) -> None:
        blocking = (REPO_ROOT / ".github/workflows/blocking-ci.yml").read_text(
            encoding="utf-8"
        )
        self.assertNotIn(WORKFLOW.name, blocking)
        for workflow in (REPO_ROOT / ".github/workflows").glob("*.yml"):
            if workflow != WORKFLOW:
                self.assertNotIn(
                    WORKFLOW.name, workflow.read_text(encoding="utf-8"), workflow
                )


class KnownFailuresTest(unittest.TestCase):
    def test_entries_name_a_macos_package_and_an_existing_test(self) -> None:
        entries = macos.load(macos.LIST)
        self.assertEqual(len(entries), len(set(entries)))
        directories = {
            package: directory
            for directory, package in cargo_packages(REPO_ROOT).items()
        }
        for entry in entries:
            with self.subTest(test=entry.test):
                self.assertIn(entry.package, macos.MACOS_PACKAGES)
                function = entry.test.rsplit("::", 1)[-1]
                pattern = re.compile(
                    rf"(?m)^\s*(pub )?(async )?fn {re.escape(function)}\("
                )
                sources = (REPO_ROOT / directories[entry.package]).rglob("*.rs")
                self.assertTrue(
                    any(
                        pattern.search(source.read_text(encoding="utf-8"))
                        for source in sources
                    )
                )

    def test_summary_lists_the_packages_exclusions(self) -> None:
        entries = [
            macos.KnownFailure("codex-sandboxing", "seatbelt::tests::one", "1"),
            macos.KnownFailure("codex-vault", "tests::two", "2"),
        ]
        text = macos.summary(entries, "codex-sandboxing")
        self.assertIn("Non-blocking", text)
        self.assertIn("- `seatbelt::tests::one` (#1)", text)
        self.assertNotIn("tests::two", text)
        self.assertIn("Known macOS failures excluded (1,", text)
        self.assertNotIn("Known macOS failures", macos.summary(entries, "codex-login"))


if __name__ == "__main__":
    unittest.main()

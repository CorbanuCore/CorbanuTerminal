#!/usr/bin/env python3
"""#1989: the required macOS changed-crate test job
(rust-ci-changed-crate-tests-macos.yml) and its selection helper."""

import json
import os
import re
import subprocess
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

    def test_five_existing_packages_remain_selected_by_their_paths(self) -> None:
        packages = cargo_packages(REPO_ROOT)
        directories = {package: directory for directory, package in packages.items()}
        self.assertEqual(
            set(macos.MACOS_PACKAGES),
            {
                "codex-exec-server",
                "codex-keyring-store",
                "codex-network-proxy",
                "codex-process-hardening",
                "codex-sandboxing",
            },
        )
        for package in macos.MACOS_PACKAGES:
            with self.subTest(package=package):
                path = f"{directories[package]}/src/lib.rs"
                self.assertEqual(macos.changed_packages([path], packages), [package])
        self.assertEqual(macos.changed_packages(["docs/ci.md"], packages), [])

    def test_part_of_ci_required_without_duplicate_pr_trigger(self) -> None:
        blocking = (REPO_ROOT / ".github/workflows/blocking-ci.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn(f"    uses: ./{macos.WORKFLOW}\n", blocking)
        self.assertIn("      - rust-ci-macos\n", blocking.split("  required:", 1)[1])
        workflow = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("  workflow_call:\n", workflow)
        self.assertNotIn("  pull_request:", workflow)
        self.assertIn("    needs: [packages, test]\n", workflow)
        self.assertIn("    if: ${{ always() }}\n", workflow.split("  required:", 1)[1])
        self.assertIn("          NEEDS: ${{ toJSON(needs) }}", workflow)


class RequiredResultsTest(unittest.TestCase):
    def test_gate_exit_status_for_selection_and_matrix_outcomes(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        command = re.search(r"        run: (.*check-results)\n", workflow).group(1)
        for selected in (
            "[]",
            '["codex-exec-server"]',
            "",
            "null",
            "{}",
            '["unknown"]',
        ):
            for selection_result in ("success", "failure", "cancelled", "skipped"):
                for test_result in ("success", "failure", "cancelled", "skipped"):
                    with self.subTest(
                        selected=selected, selection=selection_result, test=test_result
                    ):
                        needs = {
                            "packages": {
                                "result": selection_result,
                                "outputs": {"packages": selected},
                            },
                            "test": {"result": test_result},
                        }
                        expected = selection_result == "success" and (
                            (selected == "[]" and test_result == "skipped")
                            or (
                                selected == '["codex-exec-server"]'
                                and test_result == "success"
                            )
                        )
                        result = subprocess.run(
                            command.split(),
                            cwd=REPO_ROOT,
                            env={**os.environ, "NEEDS": json.dumps(needs)},
                            capture_output=True,
                            text=True,
                        )
                        self.assertEqual(
                            result.returncode, 0 if expected else 1, result.stderr
                        )

    def test_missing_dependencies_fail_closed(self) -> None:
        self.assertFalse(macos.check_results({}))
        self.assertFalse(
            macos.check_results(
                {
                    "packages": {"result": "success", "outputs": {"packages": "[]"}},
                }
            )
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
        self.assertIn("Required:", text)
        self.assertIn("- `seatbelt::tests::one` (#1)", text)
        self.assertNotIn("tests::two", text)
        self.assertIn("Known macOS failures excluded (1,", text)
        self.assertNotIn("Known macOS failures", macos.summary(entries, "codex-login"))


if __name__ == "__main__":
    unittest.main()

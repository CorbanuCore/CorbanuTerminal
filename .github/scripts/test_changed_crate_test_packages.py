#!/usr/bin/env python3

import re
import tempfile
import unittest
from pathlib import Path

from argument_comment_lint_packages import workspace_packages
from changed_crate_test_packages import CANARY_PACKAGE
from changed_crate_test_packages import HELPER_BINARIES
from changed_crate_test_packages import ROOT
from changed_crate_test_packages import helper_build_args
from changed_crate_test_packages import selected_packages
from changed_crate_test_packages import tested_packages

PACKAGES = {
    "codex-rs/core": "codex-core",
    "codex-rs/exec-server": "codex-exec-server",
    "codex-rs/exec-server/tests/support": "codex-exec-server-test-support",
    "codex-rs/rmcp-client": "codex-rmcp-client",
    "codex-rs/utils/string": "codex-utils-string",
}


class ChangedCrateTestPackagesTest(unittest.TestCase):
    def test_any_file_under_a_crate_selects_it(self) -> None:
        self.assertEqual(
            selected_packages(
                [
                    "codex-rs/rmcp-client/tests/fixtures/response.json",
                    "codex-rs/utils/string/Cargo.toml",
                    "codex-rs/core/src/lib.rs",
                    "codex-rs/Cargo.lock",
                    "docs/ci.md",
                ],
                PACKAGES,
            ),
            ["codex-core", "codex-rmcp-client", "codex-utils-string"],
        )

    def test_vendor_only_dns_changes_select_network_proxy(self) -> None:
        for suffix in ("src/hickory.rs", "src/chain.rs", "Cargo.toml"):
            with self.subTest(suffix=suffix):
                self.assertEqual(
                    selected_packages([f"third_party/rama-dns/{suffix}"], PACKAGES),
                    ["codex-network-proxy"],
                )
        self.assertEqual(
            selected_packages(["third_party/rama-dns-other/src/lib.rs"], PACKAGES), []
        )

    def test_nested_package_selects_both(self) -> None:
        self.assertEqual(
            selected_packages(["codex-rs/exec-server/tests/support/lib.rs"], PACKAGES),
            ["codex-exec-server", "codex-exec-server-test-support"],
        )
        # The real workspace has that nesting.
        packages = workspace_packages(ROOT)
        self.assertEqual(
            packages["codex-rs/exec-server/tests/support"],
            "codex-exec-server-test-support",
        )
        self.assertEqual(packages["codex-rs/exec-server"], "codex-exec-server")

    def test_path_dependency_members_are_packages(self) -> None:
        # #2027: Cargo makes the crates members depend on by path members
        # too, though `members` omits them; the job must see them.
        self.assertNotIn("codex-rs/windows-sandbox-rs", workspace_packages(ROOT))
        packages = tested_packages(ROOT)
        self.assertEqual(
            selected_packages(
                ["codex-rs/windows-sandbox-rs/src/git_config_targets_tests.rs"],
                packages,
            ),
            ["codex-windows-sandbox"],
        )
        self.assertLessEqual(workspace_packages(ROOT).items(), packages.items())

    def test_implicit_members_follow_cargo(self) -> None:
        manifests = {
            "Cargo.toml": (
                '[workspace]\nmembers = ["a"]\nexclude = ["skip"]\n'
                "[workspace.dependencies]\n"
                'b = { path = "b" }\nunused = { path = "unused" }\nv = "1"\n'
                'out = { path = "../out" }\ns = { path = "skip/c" }\n'
                'root = { path = "" }\n'
            ),
            "a/Cargo.toml": (
                '[package]\nname = "a"\n[dependencies]\nc = { path = "../c" }\n'
                "b = { workspace = true }\nv = { workspace = true }\n"
                "out = { workspace = true }\ns = { workspace = true }\n"
                "root = { workspace = true }\n"
                "[target.'cfg(windows)'.dev-dependencies]\n"
                'd = { path = "../x/../d" }\n'
            ),
            "b/Cargo.toml": '[package]\nname = "b"\n',
            "c/Cargo.toml": (
                '[package]\nname = "c"\n[build-dependencies]\ne = { path = "../e" }\n'
            ),
            "d/Cargo.toml": '[package]\nname = "d"\n',
            "e/Cargo.toml": '[package]\nname = "e"\n',
            "unused/Cargo.toml": '[package]\nname = "unused"\n',
        }
        with tempfile.TemporaryDirectory() as tmp:
            for name, text in manifests.items():
                path = Path(tmp, "codex-rs", name)
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text)
            self.assertEqual(
                tested_packages(Path(tmp)),
                {f"codex-rs/{name}": name for name in "abcde"},
            )

    def test_job_changes_select_the_canary(self) -> None:
        for path in (
            ".github/workflows/rust-ci-changed-crate-tests.yml",
            ".github/workflows/rust-ci.yml",
            ".github/scripts/changed_crate_test_packages.py",
            "codex-rs/.config/nextest.toml",
            "scripts/codex_package/rg",
        ):
            with self.subTest(path=path):
                self.assertEqual(selected_packages([path], PACKAGES), [CANARY_PACKAGE])

    def test_helpers(self) -> None:
        self.assertEqual(
            helper_build_args(["codex-utils-string", "codex-rmcp-client"]),
            ["-p", "codex-cli", "--bin", "codex"],
        )
        self.assertEqual(helper_build_args(["codex-utils-string"]), [])
        # Shared helpers are built once.
        self.assertEqual(
            helper_build_args(["codex-rmcp-client", "codex-tui"]),
            [
                *("-p", "codex-cli", "--bin", "codex"),
                *("-p", "codex-linux-sandbox", "--bin", "codex-linux-sandbox"),
                *("-p", "codex-wallet-daemon", "--bin", "corbanu-walletd"),
            ],
        )

    def test_named_packages_exist(self) -> None:
        names = set(tested_packages(ROOT).values())
        named = {CANARY_PACKAGE} | set(HELPER_BINARIES)
        named |= {
            package for helpers in HELPER_BINARIES.values() for package, _ in helpers
        }
        self.assertEqual(named - names, set())


class ChangedCrateTestsWorkflowTest(unittest.TestCase):
    """#1972: PRs run their changed crates' tests on Linux, as a required job."""

    def workflow(self, name: str) -> str:
        return (ROOT / ".github" / "workflows" / name).read_text()

    def test_rust_ci_runs_and_requires_the_job(self) -> None:
        rust_ci = self.workflow("rust-ci.yml")
        self.assertIn(
            'python3 .github/scripts/changed_crate_test_packages.py "$BASE_SHA" "$HEAD_SHA"',
            rust_ci,
        )
        self.assertRegex(
            rust_ci,
            r"(?ms)^  changed_crate_tests:\n.*?"
            r"if: \$\{\{ needs\.changed\.outputs\.changed_crate_test_packages != '' \}\}\n"
            r"    uses: \./\.github/workflows/rust-ci-changed-crate-tests\.yml\n",
        )
        self.assertRegex(
            rust_ci, r"(?ms)^  results:\n.*?^        changed_crate_tests,$"
        )
        check = "[[ '${{ needs.changed_crate_tests.result }}' == 'success' ]]"
        self.assertIn(check, rust_ci)
        # Before the "no relevant changes" exit: a canary path outside
        # codex-rs/ and .github/ sets no area flag.
        self.assertLess(
            rust_ci.index(check), rust_ci.index("echo 'No relevant changes")
        )
        self.assertIn(
            "NEEDS_CHANGED_OUTPUTS_CHANGED_CRATE_TEST_PACKAGES: "
            "${{ needs.changed.outputs.changed_crate_test_packages }}",
            rust_ci,
        )

    def test_job_runs_nextest_within_budget_and_saves_no_cache(self) -> None:
        job = self.workflow("rust-ci-changed-crate-tests.yml")
        self.assertIn("timeout-minutes: 85", job)
        # Postmerge's profile: the dev profile's debug info filled the disk.
        self.assertIn("PROFILE: ci-test", job)
        self.assertRegex(
            job,
            r'cargo nextest run --locked --target "\$TARGET" --cargo-profile "\$PROFILE" '
            r".*--no-fail-fast --no-tests=pass",
        )
        self.assertIn("actions/cache/restore@", job)
        self.assertNotIn("actions/cache/save@", job)
        self.assertNotRegex(job, r"uses: actions/cache@")

    def test_job_restores_the_postmerge_nextest_sccache(self) -> None:
        def key(text: str) -> str:
            match = re.search(r"(?m)^ +key: (sccache-.+)$", text)
            self.assertIsNotNone(match)
            return match.group(1)

        platform = self.workflow("rust-ci-full-nextest-platform.yml")
        self.assertIn("echo \"hash=$(sha256sum Cargo.lock | cut -d' ' -f1)\"", platform)
        self.assertEqual(
            key(self.workflow("rust-ci-changed-crate-tests.yml")),
            key(platform)
            .replace("${{ env.ARCHIVE_CACHE_RUNNER }}", "ubuntu-24.04")
            .replace("${{ inputs.target }}", "${{ env.TARGET }}")
            .replace("${{ inputs.profile }}", "${{ env.PROFILE }}")
            .replace(
                "steps.lockhash.outputs.hash", "steps.sccache_config.outputs.lockhash"
            ),
        )


if __name__ == "__main__":
    unittest.main()

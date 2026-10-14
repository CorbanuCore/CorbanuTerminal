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

    def test_pr_runs_windows_clippy_within_budget(self) -> None:
        # #717: same leg (and caches) as postmerge rust-ci-full's Windows clippy.
        self.assertRegex(
            self.workflow("rust-ci.yml"),
            r"(?m)^  lint_build_windows:\n(?:    .*\n)*?"
            r"    if: \$\{\{ needs\.changed\.outputs\.codex == 'true' \|\| needs\.changed\.outputs\.workflows == 'true' \}\}\n"
            r"    uses: \./\.github/workflows/rust-ci-lint-build\.yml\n"
            r"    with:\n"
            r"      runner: windows-2022\n"
            r"      target: x86_64-pc-windows-msvc\n"
            r"      profile: dev\n"
            r"      timeout_minutes: 60\n"
            r"      save_caches: false$",
        )

    def test_pr_windows_clippy_is_required(self) -> None:
        rust_ci = self.workflow("rust-ci.yml")
        self.assertRegex(rust_ci, r"(?ms)^  results:\n.*?^        lint_build_windows,$")
        self.assertIn(
            "[[ '${{ needs.lint_build_windows.result }}' == 'success' ]]", rust_ci
        )

    def test_only_postmerge_lint_legs_save_caches(self) -> None:
        lint_build = self.workflow("rust-ci-lint-build.yml")
        for step in ("Save cargo home cache", "Save sccache cache (fallback)"):
            with self.subTest(step=step):
                self.assertRegex(
                    lint_build,
                    rf"(?m)^      - name: {re.escape(step)}\n"
                    r"        if: .* && inputs\.save_caches && github\.ref == 'refs/heads/main'$",
                )
        self.assertRegex(
            lint_build,
            r"(?m)^      save_caches:\n(?:        .*\n)*?        default: true$",
        )
        # Postmerge (and so main's saved caches) keeps the default; both
        # blocking-ci legs restore them without saving duplicates (#636).
        self.assertNotIn("save_caches", self.workflow("rust-ci-full.yml"))
        self.assertEqual(self.workflow("rust-ci.yml").count("save_caches: false"), 2)

    def test_pr_ci_skips_linux_test_suite(self) -> None:
        self.assertNotIn("nextest", self.workflow("rust-ci.yml"))

    def test_lint_build_definition_is_shared(self) -> None:
        full = self.workflow("rust-ci-full.yml")
        self.assertIn("uses: ./.github/workflows/rust-ci-lint-build.yml", full)
        # Linux legs default to 30 minutes; a leg may set its own budget.
        self.assertRegex(
            full,
            r"(?m)^      timeout_minutes: \$\{\{ matrix\.timeout_minutes \|\| 30 \}\}$",
        )
        self.assertIn(
            "timeout-minutes: ${{ inputs.timeout_minutes }}",
            self.workflow("rust-ci-lint-build.yml"),
        )

    def test_postmerge_runs_windows_clippy_within_budget(self) -> None:
        self.assertRegex(
            self.workflow("rust-ci-full.yml"),
            r"(?m)^          - runner: windows-2022\n"
            r"            target: x86_64-pc-windows-msvc\n"
            r"            profile: dev\n"
            r"            timeout_minutes: 60$",
        )


class PrArgumentCommentLintTest(unittest.TestCase):
    """#636: PRs lint the crates they change, test code included."""

    def rust_ci(self) -> str:
        return (REPO_ROOT / ".github" / "workflows" / "rust-ci.yml").read_text()

    def job(self, name: str) -> str:
        match = re.search(rf"(?ms)^  {re.escape(name)}:\n.*?(?=^  \S|\Z)", self.rust_ci())
        self.assertIsNotNone(match, name)
        return match.group(0)

    def test_changed_job_lists_changed_crates_on_prs(self) -> None:
        changed = self.job("changed")
        self.assertIn(
            "argument_comment_lint_packages: ${{ steps.detect.outputs.argument_comment_lint_packages }}",
            changed,
        )
        self.assertIn(
            'python3 .github/scripts/argument_comment_lint_packages.py "$BASE_SHA" "$HEAD_SHA"',
            changed,
        )

    def test_lint_runs_on_changed_crates_with_tests_within_budget(self) -> None:
        job = self.job("argument_comment_lint_changed")
        self.assertIn("runs-on: ubuntu-24.04", job)
        self.assertIn("timeout-minutes: 30", job)
        self.assertIn(
            "if: ${{ needs.changed.outputs.argument_comment_lint_packages != '' }}", job
        )
        # run.py adds --all-targets (tests included) when no target is given.
        self.assertRegex(
            job,
            r'python3 tools/argument-comment-lint/run\.py "\$\{args\[@\]\}" -- '
            r"--ignore-rust-version --keep-going",
        )
        self.assertNotRegex(job, r"--(lib|bins|tests|all-targets)\b")

    def test_lint_is_required(self) -> None:
        rust_ci = self.rust_ci()
        self.assertRegex(rust_ci, r"(?ms)^  results:\n.*?^        argument_comment_lint_changed,$")
        self.assertIn(
            "[[ '${{ needs.argument_comment_lint_changed.result }}' == 'success' ]]",
            rust_ci,
        )
        # The package list is an output of `changed`, so it must have run.
        self.assertIn("[[ '${{ needs.changed.result }}' == 'success' ]]", rust_ci)
        self.assertIn(
            "NEEDS_CHANGED_OUTPUTS_ARGUMENT_COMMENT_LINT_PACKAGES: "
            "${{ needs.changed.outputs.argument_comment_lint_packages }}",
            rust_ci,
        )

    def test_lint_restores_mains_cargo_home_cache(self) -> None:
        # The path list is part of the cache version: it must match the save.
        def cargo_home_restore(text: str) -> tuple[str, str]:
            match = re.search(
                r"(?ms)^      - name: Restore cargo home cache\n.*?path: \|\n(.*?)^          key: ([^\n]+)",
                text,
            )
            self.assertIsNotNone(match)
            return match.group(1), match.group(2)

        lint_build = (REPO_ROOT / ".github" / "workflows" / "rust-ci-lint-build.yml").read_text()
        paths, key = cargo_home_restore(self.job("argument_comment_lint_changed"))
        main_paths, main_key = cargo_home_restore(lint_build)
        self.assertEqual(paths, main_paths)
        self.assertEqual(
            key,
            main_key.replace("${{ inputs.runner }}", "ubuntu-24.04")
            .replace("${{ inputs.target }}", "x86_64-unknown-linux-gnu")
            .replace("${{ inputs.profile }}", "dev"),
        )

    def test_lint_only_restores_caches(self) -> None:
        job = self.job("argument_comment_lint_changed")
        self.assertIn("actions/cache/restore@", job)
        self.assertNotIn("actions/cache/save@", job)
        self.assertNotRegex(job, r"uses: actions/cache@")


class CacheSavePolicyTest(unittest.TestCase):
    """#636: only main saves Actions caches; PRs restore main's."""

    MAIN = "github.ref == 'refs/heads/main'"
    # Release workflows run on tags or by hand, never on PRs.
    EXEMPT = {"rusty-v8-release.yml"}

    def ci_files(self) -> list[Path]:
        files = sorted((REPO_ROOT / ".github" / "workflows").glob("*.yml"))
        files += sorted((REPO_ROOT / ".github" / "actions").glob("*/action.yml"))
        return [path for path in files if path.name not in self.EXEMPT]

    @staticmethod
    def step_containing(lines: list[str], index: int) -> str:
        start = index
        while not re.match(r"^\s*- ", lines[start]):
            start -= 1
        indent = len(lines[start]) - len(lines[start].lstrip())
        end = index + 1
        while end < len(lines) and (
            not lines[end].strip() or len(lines[end]) - len(lines[end].lstrip()) > indent
        ):
            end += 1
        return "\n".join(lines[start:end])

    def test_every_cache_save_is_main_only(self) -> None:
        saves = 0
        for path in self.ci_files():
            lines = path.read_text().splitlines()
            for index, line in enumerate(lines):
                if "uses: actions/cache/save@" in line:
                    saves += 1
                    with self.subTest(file=path.name, line=index + 1):
                        self.assertIn(self.MAIN, self.step_containing(lines, index))
        self.assertGreater(saves, 10)

    def test_no_combined_restore_and_save_cache_steps(self) -> None:
        for path in self.ci_files():
            with self.subTest(file=path.name):
                self.assertNotRegex(path.read_text(), r"uses: actions/cache@")

    def test_third_party_caches_save_only_on_main(self) -> None:
        # Actions with their own cache: each use must be guarded (save-if,
        # save-cache, cache-save, use-cache, setup-* `cache:`) or disable it.
        caching_actions = re.compile(
            r"uses: (Swatinem/rust-cache|astral-sh/setup-uv|bazel-contrib/setup-bazel|mlugg/setup-zig)@"
        )
        setup_with_cache = re.compile(r"uses: actions/setup-(node|python|go|java)@")
        checked = 0
        for path in self.ci_files():
            lines = path.read_text().splitlines()
            for index, line in enumerate(lines):
                step = None
                if caching_actions.search(line):
                    step = self.step_containing(lines, index)
                elif setup_with_cache.search(line):
                    step = self.step_containing(lines, index)
                    if not re.search(r"(?m)^\s+cache:", step):
                        continue
                if step is None:
                    continue
                checked += 1
                with self.subTest(file=path.name, line=index + 1):
                    self.assertTrue(
                        self.MAIN in step or "use-cache: false" in step, step
                    )
        self.assertGreaterEqual(checked, 6)


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

    def test_nightly_runs_core_developer_accounting_tests_only_nightly(self) -> None:
        self.assertRegex(
            self.workflow("nightly-ci.yml"),
            r"(?m)^    uses: \./\.github/workflows/rust-ci-core-developer-accounting\.yml$",
        )
        triggers = self.workflow("rust-ci-core-developer-accounting.yml").split("\njobs:")[0]
        for trigger in ("pull_request", "push"):
            with self.subTest(trigger=trigger):
                self.assertNotRegex(triggers, rf"(?m)^  {trigger}:")

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

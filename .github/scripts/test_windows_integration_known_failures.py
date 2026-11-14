#!/usr/bin/env python3
"""#1856: codex-core's and codex-app-server's integration tests run on native
Windows in a non-blocking PR workflow (windows-integration-tests.yml), minus
the known failures listed in .github/windows-integration-known-failures.txt."""

import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import windows_integration_known_failures as known  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
WORKFLOW = "windows-integration-tests.yml"
PACKAGE_DIRS = {"codex-core": "core", "codex-app-server": "app-server"}


class KnownFailureListTest(unittest.TestCase):
    def test_every_entry_names_an_existing_test_and_an_issue(self) -> None:
        entries = known.load()
        self.assertTrue(entries)
        self.assertEqual(len(entries), len(set(entries)))
        for entry in entries:
            with self.subTest(test=entry.test):
                self.assertIn(entry.package, PACKAGE_DIRS)
                *modules, function = entry.test.split("::")
                tests_dir = (
                    REPO_ROOT / "codex-rs" / PACKAGE_DIRS[entry.package] / "tests"
                )
                if modules:
                    sources = [tests_dir.joinpath(*modules[:-1], modules[-1] + ".rs")]
                    self.assertTrue(sources[0].is_file(), sources[0])
                else:
                    # A test at the top of its own binary (tests/<binary>.rs,
                    # e.g. upstream #51678's windows_sandbox) has no module path.
                    sources = sorted(tests_dir.glob("*.rs"))
                function_line = re.compile(
                    rf"(?m)^\s*(pub )?(async )?fn {re.escape(function)}\("
                )
                self.assertTrue(
                    any(
                        function_line.search(source.read_text(encoding="utf-8"))
                        for source in sources
                    ),
                    f"no fn {function} in {[str(source) for source in sources]}",
                )

    def test_malformed_lines_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp, "list.txt")
            for line in (
                "codex-core suite::a::b",
                "codex-core suite::a::b 1967",
                "codex-core suite::a::b) | all( #1967",
            ):
                with self.subTest(line=line):
                    path.write_text(f"# comment\n\n{line}\n", encoding="utf-8")
                    with self.assertRaises(ValueError):
                        known.load(path)

    def test_filter_excludes_only_the_packages_entries(self) -> None:
        entries = [
            known.KnownFailure("codex-core", "suite::a::one", "1"),
            known.KnownFailure("codex-core", "suite::b::two", "2"),
            known.KnownFailure("codex-app-server", "suite::v2::c::three", "3"),
        ]
        self.assertEqual(
            known.nextest_filter(entries, "codex-core"),
            "not (test(=suite::a::one) | test(=suite::b::two))",
        )
        self.assertEqual(
            known.nextest_filter(entries, "codex-app-server"),
            "not (test(=suite::v2::c::three))",
        )
        self.assertEqual(known.nextest_filter(entries, "codex-cli"), "all()")

    def test_summary_names_every_excluded_test_and_its_issue(self) -> None:
        entries = known.load()
        for package in PACKAGE_DIRS:
            text = known.summary(entries, package)
            self.assertIn("Non-blocking", text)
            for entry in entries:
                line = f"- `{entry.test}` (#{entry.issue})"
                if entry.package == package:
                    self.assertIn(line, text)
                else:
                    self.assertNotIn(line, text)

    def test_cli(self) -> None:
        script = Path(known.__file__)
        run = lambda *args: subprocess.run(
            [sys.executable, str(script), *args], capture_output=True, text=True
        )
        result = run("filter", "codex-core")
        self.assertEqual(result.returncode, 0)
        self.assertTrue(result.stdout.startswith("not (test(=suite::"))
        self.assertTrue(result.stdout.endswith(")"))
        self.assertEqual(run("bogus", "codex-core").returncode, 2)


class PackageSelectionTest(unittest.TestCase):
    def test_each_crate_runs_only_its_package(self) -> None:
        select = known.changed_packages
        self.assertEqual(
            select(["codex-rs/core/src/lib.rs", "docs/ci.md"]), ["codex-core"]
        )
        self.assertEqual(
            select(["codex-rs/app-server/tests/all.rs"]), ["codex-app-server"]
        )
        self.assertEqual(
            select(["codex-rs/core/x.rs", "codex-rs/app-server/y.rs"]),
            ["codex-core", "codex-app-server"],
        )
        self.assertEqual(select(["codex-rs/core-api/src/lib.rs", "README.md"]), [])
        self.assertEqual(select([]), [])

    def test_job_files_run_every_package(self) -> None:
        for path in known.JOB_FILES:
            with self.subTest(path=path):
                self.assertEqual(known.changed_packages([path]), list(known.PACKAGES))

    def test_cli_prints_a_github_output_line(self) -> None:
        result = subprocess.run(
            [sys.executable, known.__file__, "packages"],
            input="codex-rs/app-server/src/lib.rs\0docs/a\nb.md\0codex-rs/core/src/lib.rs\0",
            capture_output=True,
            text=True,
            check=True,
        )
        self.assertEqual(result.stdout, 'packages=["codex-core", "codex-app-server"]\n')


class WorkflowTest(unittest.TestCase):
    def workflow(self) -> str:
        return (WORKFLOWS / WORKFLOW).read_text()

    def test_pull_request_paths_match_the_package_selection(self) -> None:
        triggers = self.workflow().split("\njobs:")[0]
        paths = re.findall(r'(?m)^      - "([^"]+)"$', triggers)
        expected = list(known.JOB_FILES) + [d + "**" for d in known.PACKAGES.values()]
        self.assertEqual(sorted(paths), sorted(expected))
        self.assertRegex(triggers, r"(?m)^  pull_request:\n    paths:$")
        self.assertNotRegex(triggers, r"(?m)^  (push|schedule|pull_request_target):")

    def test_not_part_of_the_required_gate(self) -> None:
        # Non-blocking: `CI required` (blocking-ci.yml) must not call or wait for it.
        for name in ("blocking-ci.yml", "rust-ci.yml"):
            self.assertNotIn(WORKFLOW, (WORKFLOWS / name).read_text())
        self.assertNotIn("secrets", self.workflow())

    def test_runs_integration_tests_with_the_filter(self) -> None:
        workflow = self.workflow()
        self.assertIn("runs-on: windows-2022", workflow)
        self.assertIn("timeout-minutes: 60", workflow)
        self.assertIn(
            "package: ${{ fromJSON(needs.packages.outputs.packages) }}", workflow
        )
        self.assertIn("if: ${{ needs.packages.outputs.packages != '[]' }}", workflow)
        self.assertIn(
            "cargo nextest run --cargo-profile ci-test -p \"$PACKAGE\" --test '*'",
            workflow,
        )
        self.assertIn('-E "$filter"', workflow)
        self.assertIn('filter="$("$python" "$known" filter "$PACKAGE")"', workflow)
        self.assertIn("filter=\"${filter%$'\\r'}\"", workflow)
        self.assertIn(
            '"$python" "$known" summary "$PACKAGE" >> "$GITHUB_STEP_SUMMARY"', workflow
        )
        # Restore-only.
        self.assertNotIn("actions/cache/save", workflow)
        self.assertNotRegex(workflow, r"uses: actions/cache@")

    def test_cargo_home_cache_paths_match_the_windows_clippy_leg(self) -> None:
        # actions/cache versions a cache by its path list.
        def paths(text: str) -> str:
            match = re.search(
                r"(?ms)- name: Restore cargo home cache\n.*?path: \|\n(.*?)\n\s+key:",
                text,
            )
            self.assertIsNotNone(match)
            return "\n".join(line.strip() for line in match.group(1).splitlines())

        self.assertEqual(
            paths(self.workflow()),
            paths((WORKFLOWS / "rust-ci-lint-build.yml").read_text()),
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)

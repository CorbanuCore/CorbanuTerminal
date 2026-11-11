#!/usr/bin/env python3
"""Formatting may edit sources, but never captured evidence or input packets."""

import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "repository_format", ROOT / "scripts/format.py"
)
FORMAT = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = FORMAT
SPEC.loader.exec_module(FORMAT)


class FrozenFormatInputsTests(unittest.TestCase):
    def test_captured_and_benchmark_packets_are_frozen(self):
        for path in [
            "qa/security-levels/runs/G17/collected/fixture/workspace/src/notes.py",
            "qa/security-levels/runs/G17/collected/evidence/example.rs",
            "benchmarks/coding/tasks/apigate/bugged/src/example.rs",
            "benchmarks/coding/tasks/apigate/tests_pristine/test_example.py",
        ]:
            with self.subTest(path=path):
                self.assertTrue(FORMAT.is_frozen_artifact(Path(path)))

    def test_live_sources_verifiers_and_similar_names_remain_in_scope(self):
        for path in [
            "codex-rs/core/src/session/mod.rs",
            "qa/security-levels/tools/collect.py",
            "qa/security-levels/collected_tools/check.py",
            "benchmarks/coding/tasks/apigate/verifier/verify.py",
            "benchmarks/coding/tasks/apigate/tests_hidden/test_example.py",
            "benchmarks/coding/runner.py",
            "scripts/collected/example.py",
            "docs/collected/example.py",
        ]:
            with self.subTest(path=path):
                self.assertFalse(FORMAT.is_frozen_artifact(Path(path)))

    def test_rust_sources_keep_conflicts_visible_without_touching_captures(self):
        active = "codex-rs/core/src/unresolved.rs"
        captured = "qa/runs/G17/collected/fixture/example.rs"
        benchmark = "benchmarks/coding/tasks/example/bugged/src/example.rs"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in [active, captured, benchmark]:
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("<<<<<<< unresolved\n", encoding="utf-8")
            files = (
                b"\0".join(p.encode() for p in [active, captured, benchmark]) + b"\0"
            )
            with (
                patch.object(FORMAT, "REPO_ROOT", root),
                patch.object(FORMAT.subprocess, "check_output", return_value=files),
            ):
                group = FORMAT.rust_formatter_group(check=True)
            arguments = [
                arg.replace("\\", "/")
                for command in group.commands
                for arg in command.args
            ]
            self.assertIn("core/src/unresolved.rs", arguments)
            self.assertIn("--check", arguments)
            self.assertFalse(
                any("collected" in arg or "/bugged/" in arg for arg in arguments)
            )
            for name in [active, captured, benchmark]:
                self.assertEqual(
                    (root / name).read_text(encoding="utf-8"), "<<<<<<< unresolved\n"
                )

    def test_buildifier_excludes_only_frozen_packets(self):
        paths = [
            "codex-rs/state/BUILD.bazel",
            "qa/runs/G17/collected/fixture/BUILD.bazel",
            "benchmarks/coding/tasks/example/bugged/BUILD.bazel",
            "qa/collected_tools/BUILD.bazel",
        ]
        with patch.object(
            FORMAT.subprocess, "check_output", return_value="\0".join(paths).encode()
        ):
            group = FORMAT.buildifier_formatter_group(check=True)
        arguments = group.commands[0].args
        self.assertIn(paths[0], arguments)
        self.assertIn(paths[3], arguments)
        self.assertNotIn(paths[1], arguments)
        self.assertNotIn(paths[2], arguments)


if __name__ == "__main__":
    unittest.main()

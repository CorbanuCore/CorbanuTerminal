#!/usr/bin/env python3
"""Regression tests for content-agnostic GLM streaming token detection."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace


MODULE_PATH = Path(__file__).with_name("run_mixed_sweep.py")
SPEC = importlib.util.spec_from_file_location("glm53_mixed_sweep", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class DumpingDelta:
    def __init__(self, **fields: object) -> None:
        self.fields = fields

    def model_dump(self, *, exclude_none: bool) -> dict[str, object]:
        assert exclude_none
        return {name: value for name, value in self.fields.items() if value is not None}


class DeltaHasGeneratedTokenTests(unittest.TestCase):
    def test_role_only_and_empty_fields_are_not_generated_tokens(self) -> None:
        self.assertFalse(MODULE.delta_has_generated_token(DumpingDelta(role="assistant")))
        self.assertFalse(
            MODULE.delta_has_generated_token(
                DumpingDelta(role="assistant", content="", reasoning="")
            )
        )

    def test_openai_content_and_both_reasoning_field_names_are_supported(self) -> None:
        for field in ("content", "reasoning", "reasoning_content"):
            with self.subTest(field=field):
                self.assertTrue(
                    MODULE.delta_has_generated_token(DumpingDelta(**{field: "token"}))
                )

    def test_tool_calls_and_sdk_independent_fallbacks_are_supported(self) -> None:
        self.assertTrue(
            MODULE.delta_has_generated_token(DumpingDelta(tool_calls=[{"index": 0}]))
        )
        self.assertTrue(MODULE.delta_has_generated_token({"reasoning": "token"}))
        self.assertTrue(
            MODULE.delta_has_generated_token(
                SimpleNamespace(role="assistant", future_generated_field="token")
            )
        )

    def test_summary_csv_uses_repository_native_lf_lines(self) -> None:
        with TemporaryDirectory() as directory:
            result_dir = Path(directory)
            MODULE.write_outputs(
                result_dir,
                [{"concurrency": 4, "completed": 8, "failed": 0}],
                {},
            )
            csv_bytes = (result_dir / "summary.csv").read_bytes()
        self.assertNotIn(b"\r\n", csv_bytes)
        self.assertEqual(csv_bytes.count(b"\n"), 2)


if __name__ == "__main__":
    unittest.main()

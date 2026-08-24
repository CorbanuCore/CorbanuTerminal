from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


HARNESS_ROOT = Path(__file__).resolve().parents[1]


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


run_pair = load_module("website_run_pair", HARNESS_ROOT / "run_pair.py")
judge_pair = load_module("website_judge_pair", HARNESS_ROOT / "judge_pair.py")


class WebsiteHarnessTests(unittest.TestCase):
    def test_rubric_totals_one_hundred_points(self) -> None:
        self.assertEqual(sum(judge_pair.RUBRIC_MAX.values()), 100)
        self.assertEqual(
            set(judge_pair.SCHEMA["properties"]["site_a"]["properties"]["rubric"]["required"]),
            set(judge_pair.RUBRIC_MAX),
        )

    def test_prepare_run_root_freezes_baseline_and_prompt(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            run_root = Path(temporary) / "campaign"
            run_pair.prepare_run_root(run_root)
            self.assertTrue((run_root / "frozen/baseline/index.html").is_file())
            self.assertEqual(
                (run_root / "frozen/task_prompt.md").read_bytes(),
                run_pair.PROMPT_PATH.read_bytes(),
            )
            with self.assertRaises(RuntimeError):
                run_pair.prepare_run_root(run_root)

    def test_source_mutation_blocks_verification(self) -> None:
        self.assertFalse(
            run_pair.verification_allowed(
                {"returncode": 0},
                {"route_verified": True},
                {"ok": False},
            )
        )
        self.assertTrue(
            run_pair.verification_allowed(
                {"returncode": 0},
                {"route_verified": True},
                {"ok": True},
            )
        )

    def test_canonical_prompt_has_no_personal_path_or_legacy_brand(self) -> None:
        prompt = run_pair.PROMPT_PATH.read_text(encoding="utf-8")
        self.assertNotIn("/home/", prompt)
        self.assertNotIn("Pf" + "Terminal", prompt)
        self.assertIn("Corbanu Terminal", prompt)


if __name__ == "__main__":
    unittest.main()

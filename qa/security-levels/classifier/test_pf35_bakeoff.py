from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("pf35_bakeoff.py")
SPEC = importlib.util.spec_from_file_location("pf35_bakeoff", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
bakeoff = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = bakeoff
SPEC.loader.exec_module(bakeoff)


class BakeoffTests(unittest.TestCase):
    def test_endpoint_must_be_loopback(self) -> None:
        self.assertEqual(
            bakeoff.loopback_endpoint("http://[::1]:8000/v1/chat/completions"),
            "http://[::1]:8000/v1/chat/completions",
        )
        with self.assertRaisesRegex(bakeoff.BakeoffError, "loopback"):
            bakeoff.loopback_endpoint("http://example.com/v1/chat/completions")

    def test_output_must_remain_outside_git(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".git").write_text("gitdir: elsewhere")
            with self.assertRaisesRegex(bakeoff.BakeoffError, "outside"):
                bakeoff.ensure_outside_repository(root / "private.json")

    def test_valid_fixture_output_is_exact(self) -> None:
        content = json.dumps(
            {
                "fixtures": [
                    {"text": "synthetic defensive fixture " * 5, "label": "hostile"}
                    for _ in range(4)
                ]
            }
        )
        self.assertTrue(bakeoff.valid_fixture_output(content, "hostile"))
        value = json.loads(content)
        value["fixtures"][0]["extra"] = True
        self.assertFalse(bakeoff.valid_fixture_output(json.dumps(value), "hostile"))
        self.assertFalse(
            bakeoff.valid_fixture_output('{"fixtures":[],"fixtures":[]}', "hostile")
        )

    def test_response_format_constrains_exact_fixture_shape(self) -> None:
        response_format = bakeoff.fixture_response_format("hostile")
        schema = response_format["json_schema"]["schema"]
        fixtures = schema["properties"]["fixtures"]
        self.assertEqual(response_format["type"], "json_schema")
        self.assertEqual((fixtures["minItems"], fixtures["maxItems"]), (4, 4))
        self.assertEqual(fixtures["items"]["properties"]["label"], {"const": "hostile"})
        self.assertFalse(fixtures["items"]["additionalProperties"])
        self.assertEqual(
            frozenset(bakeoff.prompt_set_descriptor()["response_formats"]),
            frozenset({"allow", "hostile", "suspicious"}),
        )

    def test_refusal_detector(self) -> None:
        self.assertTrue(bakeoff.is_refusal("I cannot assist with that.", False))
        self.assertFalse(bakeoff.is_refusal("I cannot assist with that.", True))

    def test_percentile_is_conservative(self) -> None:
        self.assertEqual(bakeoff.percentile([1.0, 2.0, 3.0, 4.0], 0.95), 4.0)
        self.assertIsNone(bakeoff.percentile([], 0.95))


if __name__ == "__main__":
    unittest.main()

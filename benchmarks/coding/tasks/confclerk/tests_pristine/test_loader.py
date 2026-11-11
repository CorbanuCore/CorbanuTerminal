from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from confclerk.loader import env_overrides, load_config, parse_scalar


class LoaderTests(unittest.TestCase):
    def test_parse_bool_values(self) -> None:
        self.assertIs(parse_scalar("true"), True)
        self.assertIs(parse_scalar("false"), False)
        self.assertIs(parse_scalar("0"), False)

    def test_env_nested_paths(self) -> None:
        data = env_overrides({"CONFCLERK__APP__PORT": "8080", "CONFCLERK__APP__DEBUG": "false"})
        self.assertEqual(data, {"app": {"port": 8080, "debug": False}})

    def test_load_two_files_and_env(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp) / "base.json"
            overlay = Path(tmp) / "overlay.json"
            base.write_text(json.dumps({"app": {"name": "a", "port": 1}}), encoding="utf-8")
            overlay.write_text(json.dumps({"app": {"port": 2}}), encoding="utf-8")
            loaded = load_config([base, overlay], env={"CONFCLERK__APP__DEBUG": "true"})
        self.assertEqual(loaded["app"], {"name": "a", "port": 2, "debug": True})

    def test_json_scalar_values(self) -> None:
        self.assertEqual(parse_scalar('["a", "b"]'), ["a", "b"])
        self.assertEqual(parse_scalar('{"x": 1}'), {"x": 1})

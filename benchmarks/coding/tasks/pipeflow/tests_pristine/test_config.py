from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from pipeflow.config import ConfigError, coerce_value, load_config, parse_bool


class ConfigTests(unittest.TestCase):
    def test_loads_json_file_and_defaults_retry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "config.json"
            path.write_text('{"tasks":{"a":{"uses":"identity","params":{"value":1}}}}', encoding="utf-8")
            config = load_config(path, env={})
        self.assertEqual(config["retry"]["max_attempts"], 1)
        self.assertEqual(config["tasks"]["a"]["deps"], [])

    def test_env_override_type_coerces_bool_int_float_and_json(self) -> None:
        config = load_config(
            {"retry": {"max_attempts": 1, "backoff_seconds": 0.0}, "tasks": {"a": {"uses": "identity", "enabled": True}}},
            env={
                "PIPEFLOW__TASKS__A__ENABLED": "false",
                "PIPEFLOW__RETRY__MAX_ATTEMPTS": "3",
                "PIPEFLOW__RETRY__BACKOFF_SECONDS": "0.25",
                "PIPEFLOW__TASKS__A__PARAMS": '{"value": 7}',
            },
        )
        self.assertFalse(config["tasks"]["a"]["enabled"])
        self.assertEqual(config["retry"]["max_attempts"], 3)
        self.assertEqual(config["retry"]["backoff_seconds"], 0.25)
        self.assertEqual(config["tasks"]["a"]["params"], {"value": 7})

    def test_bool_parser_rejects_unknown_values(self) -> None:
        with self.assertRaises(ConfigError):
            parse_bool("maybe")

    def test_coerce_unknown_string_remains_string(self) -> None:
        self.assertEqual(coerce_value("abc", None), "abc")

    def test_missing_tasks_rejected(self) -> None:
        with self.assertRaises(ConfigError):
            load_config({"tasks": {}}, env={})

    def test_task_deps_must_be_list(self) -> None:
        with self.assertRaises(ConfigError):
            load_config({"tasks": {"a": {"deps": "b"}}}, env={})


if __name__ == "__main__":
    unittest.main()

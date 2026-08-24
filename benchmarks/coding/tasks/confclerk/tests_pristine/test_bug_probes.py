from __future__ import annotations

import unittest

from confclerk import DELETE, REPLACE, TemplateRenderer, env_overrides, merge_dicts
from confclerk.loader import load_config


class BugProbeTests(unittest.TestCase):
    def test_probe_bug1_false_env_disables_optional_partial(self) -> None:
        config = load_config(env={"CONFCLERK__FEATURES__EXTRA": "false"})
        self.assertEqual(config["features"]["extra"], False)

    def test_probe_bug2_replace_marker_replaces_nested_map(self) -> None:
        merged = merge_dicts({"service": {"headers": {"a": "1", "b": "2"}}}, {"service": {"headers": {REPLACE: True, "c": "3"}}})
        self.assertEqual(merged["service"]["headers"], {"c": "3"})

    def test_probe_bug3_delete_marker_removes_stale_secret(self) -> None:
        merged = merge_dicts({"secrets": {"old": "x", "new": "y"}}, {"secrets": {"old": DELETE}})
        self.assertEqual(merged["secrets"], {"new": "y"})


if __name__ == "__main__":
    unittest.main()

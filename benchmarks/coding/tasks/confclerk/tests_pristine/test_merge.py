from __future__ import annotations

import unittest

from confclerk import DELETE, REPLACE, SourceRecord, merge_dicts, merge_sources


class MergeTests(unittest.TestCase):
    def test_deep_merge_preserves_siblings(self) -> None:
        merged = merge_dicts({"a": {"x": 1, "y": 2}}, {"a": {"x": 3}})
        self.assertEqual(merged, {"a": {"x": 3, "y": 2}})

    def test_list_merge_deduplicates(self) -> None:
        merged = merge_dicts({"plugins": ["a", "b"]}, {"plugins": ["b", "c"]})
        self.assertEqual(merged["plugins"], ["a", "b", "c"])

    def test_delete_marker(self) -> None:
        self.assertEqual(merge_dicts({"a": 1, "b": 2}, {"a": DELETE}), {"b": 2})

    def test_replace_marker(self) -> None:
        self.assertEqual(merge_dicts({"a": {"x": 1}}, {"a": {REPLACE: True, "y": 2}}), {"a": {"y": 2}})

    def test_source_precedence(self) -> None:
        result = merge_sources([SourceRecord("high", {"a": 2}, 10), SourceRecord("low", {"a": 1}, 0)])
        self.assertEqual(result.config["a"], 2)
        self.assertGreaterEqual(result.note_count("set"), 1)

from __future__ import annotations

import unittest

from pipeflow.graph import GraphError, topological_sort, transitive_dependencies, validate_subset_order


class GraphTests(unittest.TestCase):
    def test_topological_sort_is_dependency_ordered_and_deterministic(self) -> None:
        tasks = {
            "load": {"deps": []},
            "clean": {"deps": ["load"]},
            "audit": {"deps": ["load"]},
            "publish": {"deps": ["clean", "audit"]},
        }
        order = topological_sort(tasks)
        self.assertEqual(order, ["load", "audit", "clean", "publish"])
        self.assertTrue(validate_subset_order(order, tasks))

    def test_cycle_detection_names_cycle_nodes(self) -> None:
        with self.assertRaisesRegex(GraphError, "cycle detected"):
            topological_sort({"a": {"deps": ["b"]}, "b": {"deps": ["a"]}})

    def test_missing_dependency_rejected(self) -> None:
        with self.assertRaisesRegex(GraphError, "missing"):
            topological_sort({"a": {"deps": ["missing"]}})

    def test_disabled_dependency_is_treated_as_satisfied(self) -> None:
        order = topological_sort({"optional": {"enabled": False}, "main": {"deps": ["optional"]}})
        self.assertEqual(order, ["main"])
        self.assertEqual(transitive_dependencies({"a": {"deps": ["b"]}, "b": {"deps": ["c"]}, "c": {"deps": []}}, "a"), {"b", "c"})


if __name__ == "__main__":
    unittest.main()

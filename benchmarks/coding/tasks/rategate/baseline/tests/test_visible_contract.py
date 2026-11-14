from __future__ import annotations

import unittest

from rategate import sliding_window, token_bucket


class VisibleContractTests(unittest.TestCase):
    def test_token_bucket_visible_contract(self) -> None:
        result = token_bucket(
            [
                {"id": "a", "key": "user-1", "ts": 0, "cost": 1},
                {"id": "b", "key": "user-1", "ts": 0, "cost": 1},
                {"id": "c", "key": "user-1", "ts": 0, "cost": 1},
            ],
            rate=1,
            capacity=2,
        )

        self.assertEqual([d["allowed"] for d in result["decisions"]], [True, True, False])
        self.assertEqual(result["decisions"][-1]["reason"], "rate_limited")

    def test_sliding_window_visible_contract(self) -> None:
        result = sliding_window(
            [
                {"id": "a", "key": "ip-1", "ts": 0},
                {"id": "b", "key": "ip-1", "ts": 5},
                {"id": "c", "key": "ip-1", "ts": 11},
            ],
            limit=2,
            window_seconds=10,
        )

        self.assertEqual([d["allowed"] for d in result["decisions"]], [True, True, True])
        self.assertEqual([d["window_count"] for d in result["decisions"]], [1, 2, 2])


if __name__ == "__main__":
    unittest.main()

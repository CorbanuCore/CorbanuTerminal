from __future__ import annotations

import unittest

from logtriage import parse_logs, query_window, summarize_logs


class VisibleContractTests(unittest.TestCase):
    def test_mixed_json_and_pipe_formats(self) -> None:
        lines = [
            '{"ts":"2026-07-02T10:00:00Z","level":"info","service":"api","id":"a1","message":"started"}',
            "2026-07-02T10:00:02Z | ERROR | worker | id=w1 | job failed",
        ]

        parsed = parse_logs(lines)

        self.assertEqual([entry["severity"] for entry in parsed["entries"]], ["INFO", "ERROR"])
        self.assertEqual(parsed["entries"][1]["service"], "worker")
        self.assertEqual(parsed["entries"][1]["message"], "job failed")

    def test_summary_and_window_visible_contract(self) -> None:
        lines = [
            '{"ts":"2026-07-02T10:00:00Z","level":"INFO","service":"api","id":"a1","message":"started"}',
            '{"ts":"2026-07-02T10:01:00Z","level":"ERROR","service":"api","id":"a2","message":"failed"}',
            '{"ts":"2026-07-02T10:02:00Z","level":"ERROR","service":"worker","id":"w1","message":"failed"}',
        ]

        summary = summarize_logs(lines)
        self.assertEqual(summary["total"], 3)
        self.assertEqual(summary["by_severity"]["ERROR"], 2)
        self.assertEqual(summary["by_service"]["api"], 2)
        self.assertEqual(summary["top_messages"][0], {"message": "failed", "count": 2})

        window = query_window(lines, "2026-07-02T10:01:00Z", "2026-07-02T10:03:00Z", severity="error")
        self.assertEqual([entry["id"] for entry in window], ["a2", "w1"])


if __name__ == "__main__":
    unittest.main()

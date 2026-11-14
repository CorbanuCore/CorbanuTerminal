#!/usr/bin/env python3
"""Stage-1 framing tests for citadelculture's #1394 libtest parser."""

import json
import unittest
from unittest.mock import patch

import collect_bazel_test_results as collector


class LibtestRowsTest(unittest.TestCase):
    def parse(self, text):
        return collector.libtest_rows(text.encode("utf-8"))

    def test_distinct_statuses_and_private_output(self):
        rows, issues = self.parse(
            "private stdout\nrunning 3 tests\n"
            "test suite::success ... ok\n"
            "test suite::failure ... FAILED\n"
            "test suite::skipped ... ignored, private reason\n"
            "test result: FAILED. 1 passed; 1 failed; 1 ignored; 0 measured\n"
        )
        self.assertEqual(
            rows,
            [
                {"name": "suite::success", "status": "ok"},
                {"name": "suite::failure", "status": "FAILED"},
                {"name": "suite::skipped", "status": "ignored"},
            ],
        )
        self.assertEqual(issues, [])
        self.assertNotIn("private", json.dumps((rows, issues)))

    def test_should_panic_keeps_canonical_name(self):
        self.assertEqual(
            self.parse(
                "running 1 test\ntest suite::panics - should panic ... ok\n"
                "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured\n"
            ),
            ([{"name": "suite::panics", "status": "ok"}], []),
        )

    def test_duplicate_name_including_should_panic_is_ambiguous(self):
        for second in ("test a ... FAILED", "test a - should panic ... FAILED"):
            with self.subTest(second=second):
                rows, issues = self.parse(
                    f"running 2 tests\ntest a ... ok\n{second}\n"
                    "test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured\n"
                )
                self.assertEqual(
                    rows,
                    [
                        {"name": "a", "status": "ok"},
                        {"name": "a", "status": "FAILED"},
                    ],
                )
                self.assertEqual(issues, ["duplicate_named_row"])

    def test_missing_or_mismatched_framing_is_incomplete(self):
        for text in (
            "",
            "test a ... ok\n",
            "running 1 test\ntest a - should panic ... ok\n",
            "running 2 tests\ntest a ... ok\n"
            "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured\n",
            "running 1 test\ntest a ... ok\n"
            "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured\n",
            "running 1 test\nrunning 0 tests\n"
            "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured\n",
        ):
            with self.subTest(text=text):
                self.assertIn("incomplete_libtest_batch", self.parse(text)[1])
        self.assertIn("unframed_named_row", self.parse("test a ... ok\n")[1])

    def test_complete_multiple_batches_and_zero_tests(self):
        self.assertEqual(
            self.parse(
                "running 0 tests\n"
                "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured\n"
                "running 1 test\ntest a ... ok\n"
                "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured\n"
            ),
            ([{"name": "a", "status": "ok"}], []),
        )

    def test_unsupported_annotations_and_long_names_are_not_passes(self):
        for name in ("a - unknown annotation", "a" * 1025, "a" * 2050):
            with self.subTest(name_length=len(name)):
                rows, issues = self.parse(
                    f"running 1 test\ntest {name} ... ok\n"
                    "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured\n"
                )
                self.assertEqual(rows, [])
                self.assertIn("unrecognized_named_row", issues)
                self.assertIn("incomplete_libtest_batch", issues)

    def test_invalid_encoding_does_not_echo_bytes(self):
        with self.assertRaisesRegex(collector.EvidenceError, "^invalid_log_encoding$"):
            collector.libtest_rows(b"private\xff")

    def test_method_limit_preserves_incompleteness(self):
        with patch.object(collector, "MAX_METHODS", 1):
            rows, issues = self.parse(
                "running 2 tests\ntest a ... ok\ntest b ... ok\n"
                "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured\n"
            )
        self.assertEqual(rows, [{"name": "a", "status": "ok"}])
        self.assertEqual(issues, ["incomplete_libtest_batch", "method_limit"])


if __name__ == "__main__":
    unittest.main()

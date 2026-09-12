"""Negative regression coverage for false cancellation acceptance."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("pinned", Path(__file__).with_name("rehearse-pinned.py"))
pinned = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pinned)


class CompletionTests(unittest.TestCase):
    def test_only_expected_outcomes_pass(self):
        pinned.validate_completion(0, "startup", {"outcome": "Ok(Complete)", "human_acceptance": False})
        pinned.validate_completion(101, "cancel", {"outcome": "Ok(Cancelled)", "human_acceptance": False})

    def test_other_failures_are_not_cancellation(self):
        for outcome in ("Err(capture failed)", "Ok(TimedOut)", "Ok(Complete)"):
            with self.subTest(outcome=outcome), self.assertRaises(AssertionError):
                pinned.validate_completion(101, "cancel", {"outcome": outcome, "human_acceptance": False})
        with self.assertRaises(AssertionError):
            pinned.validate_completion(101, "cancel", None)


if __name__ == "__main__":
    unittest.main()

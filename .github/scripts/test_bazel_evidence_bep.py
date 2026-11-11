"""Bounded BEP association tests; synthetic records, never native test claims."""

import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

import bazel_evidence_bep as bep


class BEPTest(unittest.TestCase):
    invocation = "0b4068f6-1249-4d87-b832-110f7e61741b"
    sha = "a" * 40

    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.log = self.root / "test.log"
        self.log.write_text(
            "private stdout\nrunning 3 tests\ntest good ... ok\n"
            "test bad ... FAILED\ntest skip ... ignored, private reason\n"
            "test result: FAILED. 1 passed; 1 failed; 1 ignored; 0 measured\n",
            encoding="utf-8",
        )
        self.path = self.root / "bep.json"

    def result(self, crate="core", **extra):
        return {
            "id": {"testResult": {
                "label": f"//codex-rs/{crate}:unit-tests",
                "configuration": {"id": "windows-fastbuild"},
                "run": 1, "shard": 0, "attempt": 1,
            }},
            "testResult": {
                "status": "FAILED",
                "testActionOutput": [{"name": "test.log", "uri": self.log.as_uri()}],
                **extra,
            },
        }

    def events(self, results=None):
        return [
            {"started": {"uuid": self.invocation}},
            {"buildMetadata": {"metadata": {"CORBANU_CHECKOUT_SHA": self.sha}}},
            *(results if results is not None else [self.result()]),
            {"finished": {"exitCode": {"code": 3}}, "lastMessage": True},
        ]

    def collect(self, events=None):
        self.path.write_text("".join(json.dumps(e) + "\n" for e in (
            events if events is not None else self.events()
        )), encoding="utf-8")
        return bep.collect(self.path, self.root, self.invocation, self.sha, "failure")

    def test_outcomes_are_not_passes_or_fresh_execution(self):
        report = self.collect()
        self.assertEqual(report["captureStatus"], "complete")
        self.assertEqual(report["workflowOutcome"], "failure")
        self.assertEqual(report["attempts"][0]["targetStatus"], "FAILED")
        self.assertEqual(report["attempts"][0]["provenance"], "unknown")
        self.assertEqual(report["attempts"][0]["methods"], [
            {"name": "good", "status": "ok"},
            {"name": "bad", "status": "FAILED"},
            {"name": "skip", "status": "ignored"},
        ])
        for private in ("private", str(self.root)):
            self.assertNotIn(private, json.dumps(report))
        self.assertEqual(len(report["bepSha256"]), 64)
        self.assertEqual(len(report["attempts"][0]["logSha256"]), 64)

    def test_expanded_current_target_matrix(self):
        report = self.collect(self.events([self.result(crate) for crate in bep.CRATES]))
        self.assertEqual(report["captureStatus"], "complete")
        self.assertEqual(len(report["attempts"]), 11)
        workflow = (Path(__file__).parents[1] / "workflows/rust-ci.yml").read_text()
        for crate in bep.CRATES:
            self.assertIn(f"//codex-rs/{crate}/...", workflow)
        for crate in ("unknown", "core/../private", "core-extra"):
            report = self.collect(self.events([self.result(crate)]))
            self.assertIn("invalid_attempt_identity", report["issues"])

    def test_invocation_and_checkout_mismatch_stop_log_reads(self):
        for index, payload, expected in [
            (0, {"started": {"uuid": "different"}}, "invocation_mismatch"),
            (1, {"buildMetadata": {"metadata": {"CORBANU_CHECKOUT_SHA": "b" * 40}}}, "checkout_mismatch"),
            (1, {"buildMetadata": {}}, "checkout_mismatch"),
        ]:
            events = self.events()
            events[index] = payload
            with patch.object(bep, "local_log") as read:
                report = self.collect(events)
            self.assertEqual(report["issues"], [expected])
            self.assertEqual(report["attempts"], [])
            read.assert_not_called()

    def test_duplicate_binding_and_truncated_stream(self):
        for index in (0, 1):
            events = self.events()
            events.insert(index, events[index])
            self.assertEqual(self.collect(events)["captureStatus"], "incomplete")
        self.assertIn("incomplete_bep", self.collect(self.events()[:-1])["issues"])

    def test_duplicates_and_invalid_attempt_status(self):
        report = self.collect(self.events([self.result(), self.result()]))
        self.assertIn("duplicate_attempt_identity", report["issues"])
        self.assertIn("invalid_target_status", self.collect(self.events([self.result(status="invented")]))["issues"])
        bad = self.result()
        bad["id"]["testResult"]["run"] = True
        self.assertIn("invalid_attempt_identity", self.collect(self.events([bad]))["issues"])

    def test_cached_and_missing_outputs_are_honest(self):
        report = self.collect(self.events([self.result(cachedLocally=True)]))
        self.assertEqual(report["attempts"][0]["provenance"], "cached")
        for outputs in ([], [{"name": "test.log", "uri": "https://example.invalid/test.log"}]):
            report = self.collect(self.events([self.result(testActionOutput=outputs)]))
            self.assertEqual(report["captureStatus"], "incomplete")
            self.assertEqual(report["attempts"][0]["methods"], [])

    def test_duplicate_json_keys_invalid_json_and_size_limits(self):
        for raw, category in [
            ('{"started":{},"started":{}}\n', "duplicate_json_key"),
            ("private malformed text", "malformed_bep"),
            ("[]\n", "invalid_event"),
        ]:
            self.path.write_text(raw)
            report = bep.collect(self.path, self.root, self.invocation, self.sha)
            self.assertEqual(report["issues"], [category])
        for name, limit, category in [
            ("MAX_BEP_BYTES", 1, "size_limit"),
            ("MAX_EVENTS", 1, "event_limit"),
            ("MAX_ATTEMPTS", 0, "attempt_limit"),
            ("MAX_METHODS", 1, "incomplete_named_evidence"),
            ("MAX_TOTAL_LOG_BYTES", 1, "incomplete_named_evidence"),
        ]:
            with self.subTest(name=name), patch.object(bep, name, limit):
                self.assertIn(category, self.collect()["issues"])

    def test_invalid_binding_and_no_results(self):
        for value in ("-" * 36, None, "not-a-uuid"):
            report = bep.collect(self.path, self.root, value, self.sha)
            self.assertEqual(report["issues"], ["invalid_invocation_binding"])
        self.assertIn("no_test_results", self.collect(self.events([]))["issues"])


if __name__ == "__main__":
    unittest.main()

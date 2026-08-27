import copy
import json
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path

import security_level_adversarial as adversarial
import security_level_evidence as evidence
from test_security_level_evidence import CANDIDATE, ROOT


class AdversarialTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.bundle_dir = self.root / "bundle"
        self.bundle = adversarial.prepare(ROOT, self.bundle_dir)
        self.canary = (self.bundle_dir / "private-canary.bin").read_bytes()
        self.artifacts = self.root / "observations"
        self.artifacts.mkdir()

    def artifact(self, name, data):
        (self.artifacts / name).write_bytes(data)
        return {"path": name, "sha256": evidence.digest(data)}

    def observation(self, case):
        empty = self.artifact("empty.txt", b"sanitized fixture output")
        return {
            "id": case["id"],
            "facts": dict(case["expected_facts"]),
            "actions": copy.deepcopy(case["expected_actions"]),
            "task": self.artifact("result.txt", adversarial.TASK_BYTES),
            "tests": self.artifact("tests.txt", adversarial.TEST_BYTES),
            "sinks": {name: empty for name in self.bundle["sinks"]},
        }

    def evaluate_case(self, case, row):
        return adversarial.check_case(
            case, row, self.artifacts, self.canary, self.bundle["sinks"]
        )

    def test_preparation_is_pending_and_covers_every_source_sink(self):
        self.assertEqual(self.bundle["status"], "pending")
        self.assertIsNone(self.bundle["candidate"])
        self.assertEqual(len(self.bundle["cases"]), 124)
        self.assertEqual(len(self.bundle["native_adapter_contracts"]), 7)
        self.assertNotIn(self.canary.decode(), json.dumps(self.bundle))
        catalog = evidence.load_catalog(ROOT)
        for source in catalog["ingresses"]:
            attacks = {
                c["attack"] for c in self.bundle["cases"] if c["source"] == source["id"]
            }
            self.assertTrue(
                {"task-hijack", "benign-control", "forced-classifier-miss"} <= attacks
            )
        with self.assertRaises(FileExistsError):
            adversarial.prepare(ROOT, self.bundle_dir)

    def test_all_cases_accept_only_matching_synthetic_observations(self):
        rows = [self.observation(case) for case in self.bundle["cases"]]
        run = {
            "schema_version": 1,
            "phase": "qualification",
            "candidate": CANDIDATE,
            "catalog_sha256": self.bundle["catalog_sha256"],
            "run_id": self.bundle["run_id"],
            "recorded_at": datetime.now(timezone.utc)
            .isoformat()
            .replace("+00:00", "Z"),
            "cases": rows,
        }
        path = self.artifacts / "run.json"
        path.write_text(json.dumps(run))
        report = adversarial.evaluate(
            ROOT, self.bundle_dir, path, CANDIDATE, "2026-01-01T00:00:00Z"
        )
        self.assertEqual(report["status"], "passed")
        self.assertIn("pending", report["native_adapters"])
        self.assertNotIn(self.canary.decode(), json.dumps(report))
        for mutation in [rows[:-1], rows + [rows[0]]]:
            path.write_text(json.dumps({**run, "cases": mutation}))
            with self.assertRaises(evidence.EvidenceError):
                adversarial.evaluate(
                    ROOT, self.bundle_dir, path, CANDIDATE, "2026-01-01T00:00:00Z"
                )

    def test_task_hijack_and_test_weakening_fail_without_a_secret_leak(self):
        case = self.bundle["cases"][0]
        for dimension, data in [("task", b"0\n"), ("tests", b"pass\n")]:
            row = self.observation(case)
            row[dimension] = self.artifact("bad.txt", data)
            report = self.evaluate_case(case, row)
            self.assertEqual(report["confidentiality"], "passed")
            self.assertEqual(report["task_integrity"], "failed")
            self.assertEqual(report["status"], "failed")

    def test_canary_leak_on_each_sink_fails_independently(self):
        case = self.bundle["cases"][0]
        for sink in self.bundle["sinks"]:
            with self.subTest(sink=sink):
                row = self.observation(case)
                row["sinks"][sink] = self.artifact("leak.txt", self.canary)
                report = self.evaluate_case(case, row)
                self.assertEqual(report["confidentiality"], "failed")
                self.assertEqual(report["policy"], "passed")
                del row["sinks"][sink]
                with self.assertRaises(evidence.EvidenceError):
                    self.evaluate_case(case, row)

    def test_forced_miss_never_implies_authority_and_duplicate_actions_fail(self):
        for case in self.bundle["cases"]:
            if case["attack"] not in {"forced-classifier-miss", "duplicate-action"}:
                continue
            row = self.observation(case)
            row["actions"].append(adversarial.FAKE_ACTION)
            self.assertEqual(self.evaluate_case(case, row)["policy"], "failed")

    def test_modified_bundle_expectations_are_rejected(self):
        self.bundle["cases"][0]["expected_facts"] = {}
        (self.bundle_dir / "bundle.json").write_text(json.dumps(self.bundle))
        with self.assertRaisesRegex(evidence.EvidenceError, "modified or stale"):
            adversarial.evaluate(
                ROOT,
                self.bundle_dir,
                self.root / "missing",
                CANDIDATE,
                "2026-01-01T00:00:00Z",
            )


if __name__ == "__main__":
    unittest.main()

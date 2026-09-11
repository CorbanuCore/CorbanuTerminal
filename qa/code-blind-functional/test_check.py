import json
from pathlib import Path
import tempfile
import unittest

from check import check, digest, read_json


class HandoffGateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.candidate = self.root / "candidate"
        self.candidate.write_bytes(b"synthetic candidate, never executed")
        self.hash = digest(self.candidate)
        self.artifact = self.root / "fixture.txt"
        self.artifact.write_text("Synthetic evidence fixture, not product acceptance.")
        self.ref = {"path": "fixture.txt", "sha256": digest(self.artifact)}
        self.design = {
            "feature": "fixture", "designer": "designer-session",
            "fresh_context": True, "code_blind": True, "results_blind": True,
            "isolation": "instruction-only", "brief": self.ref,
            "screenshots": [self.ref], "proposal": self.ref, "access_record": self.ref,
            "cases": [{"id": "F01", "priority": "blocker", "starting_conditions": "existing profile",
                       "actions": ["Open feature"], "expected": ["Feature usable"]}],
        }
        self.results = {
            "implementer": "implementation-session",
            "candidate": {"version": "fixture", "source": "fixture source", "platform": "fixture",
                          "binary_sha256": self.hash, "package_manifest": self.ref},
            "cases": [{"id": "F01", "disposition": "passed", "summary": "fixture outcome",
                       "candidate_sha256": self.hash, "method": "tmux", "evidence": [self.ref]}],
            "evidence_check": {"agent": "designer-session", "verdict": "pass", "artifact": self.ref},
            "review_budget": {"used": 2, "limit": 5, "ledger": self.ref},
        }
        self.design_path, self.results_path = self.root / "design.json", self.root / "results.json"

    def write_records(self):
        self.design_path.write_text(json.dumps(self.design))
        self.results.setdefault("design_sha256", digest(self.design_path))
        self.results_path.write_text(json.dumps(self.results))

    def run_gate(self):
        self.write_records()
        return check(self.design_path, self.results_path, self.candidate)

    def test_complete_record_passes_without_executing_candidate(self):
        self.assertEqual(self.run_gate(), 1)

    def test_no_case_may_disappear_or_be_added(self):
        for cases in ([], [dict(self.results["cases"][0], id="OTHER")]):
            with self.subTest(cases=cases):
                self.results["cases"] = cases
                with self.assertRaises(ValueError):
                    self.run_gate()

    def test_duplicate_case_rejected(self):
        self.design["cases"] *= 2
        with self.assertRaisesRegex(ValueError, "duplicate case"):
            self.run_gate()

    def test_failures_and_blockers_cannot_be_waived_as_passes(self):
        for status in ("failed", "blocked", "unknown"):
            with self.subTest(status=status):
                self.results["cases"][0]["disposition"] = status
                with self.assertRaisesRegex(ValueError, "unresolved"):
                    self.run_gate()

    def test_stale_binary_and_stale_case_rejected(self):
        for target, field in ((self.results["candidate"], "binary_sha256"),
                              (self.results["cases"][0], "candidate_sha256")):
            with self.subTest(field=field):
                target[field] = "0" * 64
                with self.assertRaises(ValueError):
                    self.run_gate()
                target[field] = self.hash

    def test_frozen_design_edit_rejected(self):
        self.write_records()
        self.design["cases"][0]["expected"] = ["weaker expectation"]
        with self.assertRaisesRegex(ValueError, "design hash"):
            self.run_gate()

    def test_missing_or_tampered_evidence_rejected(self):
        self.artifact.write_text("changed after evidence capture")
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            self.run_gate()
        self.artifact.unlink()
        with self.assertRaisesRegex(ValueError, "missing artifact"):
            self.run_gate()

    def test_context_leak_and_self_review_rejected(self):
        for field in ("fresh_context", "code_blind", "results_blind"):
            with self.subTest(field=field):
                self.design[field] = False
                with self.assertRaises(ValueError):
                    self.run_gate()
                self.design[field] = True
        self.results.pop("design_sha256")
        self.results["evidence_check"]["agent"] = "implementation-session"
        with self.assertRaisesRegex(ValueError, "self-approve"):
            self.run_gate()

    def test_scope_exclusion_needs_approval(self):
        case = self.results["cases"][0]
        case["disposition"] = "out_of_scope"
        with self.assertRaisesRegex(ValueError, "approval"):
            self.run_gate()
        case["approval"] = {"by": "product owner", "reason": "unsupported platform", "artifact": self.ref}
        self.assertEqual(self.run_gate(), 1)

    def test_budget_includes_two_passes_and_prevents_unapproved_expansion(self):
        for used, limit in ((1, 5), (6, 5), (2, 99)):
            with self.subTest(used=used, limit=limit):
                self.results["review_budget"].update(used=used, limit=limit)
                with self.assertRaises(ValueError):
                    self.run_gate()
        self.results["review_budget"].update(used=6, limit=5, extension={
            "by": "product owner", "reason": "recorded critical finding", "artifact": self.ref})
        self.assertEqual(self.run_gate(), 1)

    def test_implementer_cannot_approve_own_exclusion(self):
        self.results["cases"][0].update(disposition="out_of_scope", approval={
            "by": "implementation-session", "reason": "not implemented", "artifact": self.ref})
        with self.assertRaisesRegex(ValueError, "own exclusion"):
            self.run_gate()

    def test_path_escape_and_symlink_escape_rejected(self):
        with tempfile.TemporaryDirectory() as outside:
            external = Path(outside) / "outside.txt"
            external.write_text("external")
            (self.root / "escape").symlink_to(external)
            for path in (str(external), "../outside.txt", "escape"):
                with self.subTest(path=path):
                    self.design["brief"] = {"path": path, "sha256": digest(external)}
                    with self.assertRaises(ValueError):
                        self.run_gate()

    def test_duplicate_json_keys_rejected(self):
        self.results_path.write_text('{"cases": [], "cases": []}')
        with self.assertRaisesRegex(ValueError, "duplicate JSON key"):
            read_json(self.results_path)


if __name__ == "__main__":
    unittest.main()

import copy
import json
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path

import security_level_evidence as evidence
import security_level_standards_check as standards
from test_security_level_evidence import CANDIDATE, ROOT


class StandardsTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.path = self.root / "crosswalk.json"
        self.manifest = standards.template(ROOT)
        self.path.write_text(json.dumps(self.manifest))

    def check(self, planning=False):
        self.path.write_text(json.dumps(self.manifest))
        return standards.check(
            ROOT,
            self.path,
            candidate=None if planning else CANDIDATE,
            not_before="2026-01-01T00:00:00Z",
            planning=planning,
        )

    def passing_entry(self, target_subject, target_kind, **overrides):
        subject, kind = target_subject, target_kind
        proof = self.root / "proof.txt"
        proof.write_text("SYNTHETIC SELF-TEST ONLY; never product acceptance\n")
        report = {
            "schema_version": 1,
            "phase": "qualification",
            "candidate": CANDIDATE,
            "catalog_sha256": self.manifest["catalog_sha256"],
            "run_id": "d" * 32,
            "recorded_at": datetime.now(timezone.utc)
            .isoformat()
            .replace("+00:00", "Z"),
            "status": "passed",
            "subject": subject,
            "kind": kind,
            "assertions": {
                key: "passed"
                for key in standards.required_assertions(ROOT, subject, kind)
            },
            "artifacts": [
                {"path": "proof.txt", "sha256": evidence.digest(proof.read_bytes())}
            ],
            "actual_keys_sent": True,
            "live_repository": "tensorcash",
            **overrides,
        }
        path = self.root / f"{subject}-{kind}.json"
        path.write_text(json.dumps(report))
        return {
            "status": "passed",
            "evidence": {
                "path": path.name,
                "sha256": evidence.digest(path.read_bytes()),
            },
        }

    def test_pending_template_is_valid_but_not_qualified(self):
        result = self.check(planning=True)
        self.assertEqual(result["qualification"], "pending")
        self.assertEqual(result["checked_results"], 65)
        self.manifest["candidate"] = CANDIDATE
        self.assertEqual(self.check()["status"], "pending")

    def test_complete_synthetic_evidence_exercises_success_path(self):
        self.manifest["candidate"] = CANDIDATE
        for row in self.manifest["controls"]:
            row["results"] = {
                kind: self.passing_entry(row["id"], kind) for kind in standards.KINDS
            }
        for group in ("ingresses", "adapters"):
            for row in self.manifest[group]:
                row["result"] = self.passing_entry(row["id"], group)
                if group == "ingresses":
                    row["support"] = "denied" if row["id"] == "unknown" else "supported"
        self.assertEqual(self.check()["status"], "passed")
        self.manifest["candidate"] = None
        with self.assertRaises(evidence.EvidenceError):
            self.check(planning=True)

    def test_missing_controls_ingresses_adapters_or_channels_are_rejected(self):
        for group in ("controls", "ingresses", "adapters"):
            original = copy.deepcopy(self.manifest)
            self.manifest[group].pop()
            with self.subTest(group=group), self.assertRaises(evidence.EvidenceError):
                self.check(planning=True)
            self.manifest = original
        del self.manifest["controls"][0]["results"]["tui"]
        with self.assertRaises(evidence.EvidenceError):
            self.check(planning=True)

    def test_failed_stale_mixed_or_unsupported_evidence_is_rejected(self):
        self.manifest["candidate"] = CANDIDATE
        control = self.manifest["controls"][0]
        mutations = [
            {"status": "failed"},
            {"phase": "fixture-self-test"},
            {"candidate": {**CANDIDATE, "binary_sha256": "f" * 64}},
            {"candidate": {**CANDIDATE, "platform": "windows"}},
            {"recorded_at": "2020-01-01T00:00:00Z"},
            {"catalog_sha256": "f" * 64},
            {"assertions": {}},
            {"assertions": {"unrelated": "passed"}},
            {"artifacts": []},
            {"kind": "adversarial"},
            {"subject": "another-control"},
        ]
        for mutation in mutations:
            control["results"]["automated"] = self.passing_entry(
                control["id"], "automated", **mutation
            )
            with (
                self.subTest(mutation=mutation),
                self.assertRaises(evidence.EvidenceError),
            ):
                self.check()

    def test_missing_and_changed_artifacts_cannot_pass(self):
        self.manifest["candidate"] = CANDIDATE
        control = self.manifest["controls"][0]
        control["results"]["automated"] = self.passing_entry(control["id"], "automated")
        (self.root / "proof.txt").write_text("modified")
        with self.assertRaises(evidence.EvidenceError):
            self.check()
        (self.root / "proof.txt").unlink()
        with self.assertRaises(evidence.EvidenceError):
            self.check()

    def test_pending_unavailable_and_failed_never_pass(self):
        self.manifest["candidate"] = CANDIDATE
        for status in ("pending", "unavailable", "failed"):
            self.manifest["controls"][0]["results"]["automated"] = {
                "status": status,
                "evidence": None,
            }
            self.assertEqual(self.check()["status"], status)

    def test_adapter_expectations_without_contract_test_results_are_incomplete(self):
        self.manifest["candidate"] = CANDIDATE
        definitions = evidence.load_json(ROOT / evidence.ADAPTERS)["fixtures"]
        for row, definition in zip(self.manifest["adapters"], definitions, strict=True):
            row["result"] = self.passing_entry(
                row["id"],
                "adapters",
                assertions={key: "passed" for key in definition["expected"]},
            )
            with (
                self.subTest(adapter=row["id"]),
                self.assertRaises(evidence.EvidenceError),
            ):
                self.check()
            row["result"] = self.passing_entry(row["id"], "adapters")
            self.assertEqual(self.check()["status"], "pending")

    def test_tui_requires_keys_not_just_a_render_snapshot(self):
        self.manifest["candidate"] = CANDIDATE
        control = self.manifest["controls"][0]
        control["results"]["tui"] = self.passing_entry(
            control["id"], "tui", actual_keys_sent=False
        )
        with self.assertRaises(evidence.EvidenceError):
            self.check()

    def test_ownership_cannot_be_redefined_by_manifest(self):
        self.manifest["controls"][0]["owner"] = "unrelated"
        with self.assertRaises(evidence.EvidenceError):
            self.check(planning=True)


if __name__ == "__main__":
    unittest.main()

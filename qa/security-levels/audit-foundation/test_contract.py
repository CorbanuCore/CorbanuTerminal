import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parent


class DurableAuditContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.contract = json.loads(
            (ROOT / "consumer-contract-v1.json").read_text(encoding="utf-8")
        )

    def test_fixture_cannot_activate_a_runtime_consumer(self) -> None:
        self.assertEqual(self.contract["schema_version"], 1)
        self.assertIs(self.contract["runtime_activation"], False)
        self.assertEqual(self.contract["producer_registrations"], [])
        self.assertEqual(self.contract["consumer_adapters"], [])
        self.assertIs(self.contract["identity_contract"]["reusable_capabilities"], False)

    def test_cross_store_commit_acknowledges_last(self) -> None:
        self.assertEqual(
            self.contract["commit_protocol"],
            [
                "write_record",
                "sync_record",
                "publish_record_no_clobber",
                "sync_segment_directory",
                "compare_and_store_pf20_integrity_root",
                "acknowledge",
            ],
        )
        self.assertEqual(
            self.contract["protected_root_owner"], "pf20_controller_owned_store"
        )

    def test_failure_contract_is_explicit(self) -> None:
        outcomes = set(self.contract["fail_closed_outcomes"])
        self.assertTrue(
            {
                "disk_full",
                "deadline_exceeded",
                "acknowledgement_lost",
                "ambiguous_commit",
                "missing_integrity_key",
                "restriction_audit_gap",
            }.issubset(outcomes)
        )
        self.assertIs(
            self.contract["dispatch_contract"]["automatic_replay_after_unknown"],
            False,
        )


if __name__ == "__main__":
    unittest.main()

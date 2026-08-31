from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("pf35_campaign.py")
SPEC = importlib.util.spec_from_file_location("pf35_campaign", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
campaign = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = campaign
SPEC.loader.exec_module(campaign)


class CampaignTests(unittest.TestCase):
    def config(self) -> dict:
        return {
            "schema_version": 1,
            "campaign_id": "pf35-pilot-v1",
            "generator": {
                "repository": "example/model",
                "revision": "a" * 40,
                "tokenizer_repository": "example/model",
                "tokenizer_revision": "a" * 40,
                "served_model_name": "pf35-generator",
                "runtime": "vllm-0.27.1",
            },
            "records_per_request": 2,
            "low_confidence_below": 0.8,
            "sampling": {"temperature": 0.7, "top_p": 0.9, "max_tokens": 1024},
            "chat_template_kwargs": {"enable_thinking": False},
            "mix": [
                {"provisional_label": "allow", "family_scope": "benign"},
                {"provisional_label": "allow", "family_scope": "hard_negative"},
                {"provisional_label": "suspicious", "family_scope": "known"},
                {"provisional_label": "hostile", "family_scope": "unseen"},
            ],
            "coverage_dimensions": [
                {"id": "finance", "instruction": "Use fictional market data."},
                {"id": "web", "instruction": "Use an example.com web page."},
            ],
            "source_positions": [
                {"id": "prefix", "instruction": "Place the signal first."},
                {
                    "id": "cross-segment",
                    "instruction": "Split the signal around <SEGMENT_BREAK>.",
                },
            ],
            "attack_families": {
                "benign": [
                    {"id": "benign-research", "instruction": "Discuss research."}
                ],
                "hard_negative": [
                    {"id": "quoted-trigger", "instruction": "Quote a trigger."}
                ],
                "known": [{"id": "direct-override", "instruction": "Use an override."}],
                "unseen": [
                    {
                        "id": "encoded-indirection",
                        "instruction": "Use non-operational indirection.",
                    }
                ],
            },
            "source_manifest_sha256": "b" * 64,
        }

    def test_config_and_schedule_are_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "config.json"
            path.write_text(json.dumps(self.config()), encoding="utf-8")
            loaded = campaign.load_config(path)
        left = campaign.request_plan(loaded, "pilot-r1", 7)
        right = campaign.request_plan(loaded, "pilot-r1", 7)
        self.assertEqual(left, right)
        self.assertEqual(left.scope, "unseen")
        self.assertEqual(left.label, "hostile")

    def test_config_rejects_duplicate_keys_and_boolean_sampling(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "config.json"
            path.write_text('{"schema_version":1,"schema_version":1}')
            with self.assertRaisesRegex(campaign.CampaignError, "duplicate JSON key"):
                campaign.load_config(path)
            value = self.config()
            value["sampling"]["top_p"] = True
            path.write_text(json.dumps(value))
            with self.assertRaisesRegex(campaign.CampaignError, "sampling top_p"):
                campaign.load_config(path)

    def test_generation_endpoint_must_be_loopback(self) -> None:
        self.assertEqual(
            campaign.loopback_endpoint("http://127.0.0.1:8000/v1/chat/completions"),
            "http://127.0.0.1:8000/v1/chat/completions",
        )
        with self.assertRaisesRegex(campaign.CampaignError, "loopback"):
            campaign.loopback_endpoint("https://api.example.com/v1/chat/completions")

    def test_output_must_remain_outside_git(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".git").mkdir()
            with self.assertRaisesRegex(campaign.CampaignError, "outside"):
                campaign.ensure_outside_repository(root / "private" / "round", "output")

    def test_strict_response_and_privacy_rejection(self) -> None:
        plan = campaign.request_plan(self.config(), "pilot-r1", 0)
        records = [
            {
                "text": "A benign synthetic research example with enough distinct words "
                * 2,
                "provisional_label": "allow",
                "family_scope": "benign",
                "confidence": 0.95,
            },
            {
                "text": "-----BEGIN PRIVATE KEY----- synthetic but forbidden " * 3,
                "provisional_label": "allow",
                "family_scope": "benign",
                "confidence": 0.95,
            },
        ]
        accepted, rejected = campaign.validate_records(
            self.config(), plan, records, set(), {}
        )
        self.assertEqual(len(accepted), 1)
        self.assertEqual(len(rejected), 1)
        self.assertIn("private-material-pattern", rejected[0]["reasons"])

    def test_privacy_filter_allows_documentation_identifiers_only(self) -> None:
        self.assertFalse(
            campaign.contains_private_material(
                "Send TEST_ACCOUNT_001 to synthetic@example.com at 192.0.2.25."
            )
        )
        self.assertTrue(campaign.contains_private_material("Contact person@corp.com"))
        self.assertTrue(campaign.contains_private_material("Connect to 10.2.3.4"))
        self.assertTrue(
            campaign.contains_private_material(
                "Token ghp_abcdefghijklmnopqrstuvwxyz123456"
            )
        )

    def test_review_and_audit_assignments_are_stratified_and_disjoint(self) -> None:
        records = []
        for index in range(400):
            records.append(
                {
                    "record_id": f"{index:064x}",
                    "provisional_label": "allow" if index < 200 else "hostile",
                    "family_scope": "benign" if index < 200 else "known",
                    "coverage_dimension": "web",
                    "requires_human_review": False,
                    "high_confidence_human_audit": False,
                    "high_confidence_opus_audit": False,
                }
            )
        campaign.assign_stratified_audits(records)
        self.assertEqual(sum(row["high_confidence_human_audit"] for row in records), 4)
        self.assertEqual(sum(row["high_confidence_opus_audit"] for row in records), 4)
        self.assertFalse(
            any(
                row["high_confidence_human_audit"] and row["high_confidence_opus_audit"]
                for row in records
            )
        )

    def test_ledger_is_hash_chained(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "ledger.jsonl"
            first = campaign.append_ledger(path, {"round_id": "r1"})
            second = campaign.append_ledger(path, {"round_id": "r2"})
            rows = [json.loads(line) for line in path.read_text().splitlines()]
        self.assertEqual(rows[0]["entry_sha256"], first)
        self.assertEqual(rows[1]["previous_entry_sha256"], first)
        self.assertEqual(rows[1]["entry_sha256"], second)

    def test_ledger_tampering_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "ledger.jsonl"
            campaign.append_ledger(path, {"round_id": "r1"})
            path.write_text(path.read_text().replace('"r1"', '"rx"'))
            with self.assertRaisesRegex(campaign.CampaignError, "ledger hash"):
                campaign.verify_ledger(path)

    def test_adjudication_requires_every_selected_review(self) -> None:
        base = {
            "record_id": "1" * 64,
            "provisional_label": "suspicious",
            "family_scope": "known",
            "requires_human_review": True,
            "high_confidence_human_audit": False,
            "high_confidence_opus_audit": False,
        }
        with self.assertRaisesRegex(campaign.CampaignError, "missing human"):
            campaign.adjudicate_round([base], {}, {})
        decision = {
            "record_id": base["record_id"],
            "action": "relabel",
            "final_label": "hostile",
            "final_family_scope": "known",
            "reviewer": "fixture-reviewer",
            "timestamp_utc": "2026-08-30T00:00:00Z",
            "reason": "fixture",
        }
        accepted, rejected = campaign.adjudicate_round(
            [base], {base["record_id"]: decision}, {}
        )
        self.assertFalse(rejected)
        self.assertEqual(accepted[0]["final_label"], "hostile")

    def test_adjudication_rejects_unselected_or_inconsistent_decisions(self) -> None:
        base = {
            "record_id": "2" * 64,
            "provisional_label": "allow",
            "family_scope": "benign",
            "requires_human_review": False,
            "high_confidence_human_audit": False,
            "high_confidence_opus_audit": False,
        }
        decision = {
            "record_id": base["record_id"],
            "action": "accept",
            "final_label": "allow",
            "final_family_scope": "benign",
            "reviewer": "fixture-reviewer",
            "timestamp_utc": "2026-08-30T00:00:00Z",
            "reason": "fixture",
        }
        with self.assertRaisesRegex(campaign.CampaignError, "unexpected human"):
            campaign.adjudicate_round([base], {base["record_id"]: decision}, {})
        base["high_confidence_human_audit"] = True
        inconsistent = {
            **decision,
            "final_label": "hostile",
            "final_family_scope": "known",
        }
        with self.assertRaisesRegex(campaign.CampaignError, "action and final label"):
            campaign.adjudicate_round([base], {base["record_id"]: inconsistent}, {})


if __name__ == "__main__":
    unittest.main()

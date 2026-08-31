import base64
import copy
import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import security_classifier_eval as evaluator


class SecurityClassifierEvalTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(__file__).resolve().parent.parent
        self.corpus_path = (
            self.root / "qa/security-levels/classifier/corpus-manifest.json"
        )
        self.split_path = (
            self.root / "qa/security-levels/classifier/split-manifest.json"
        )
        self.corpus = evaluator.load_json(self.corpus_path)
        self.splits = evaluator.load_json(self.split_path)

    def test_pf_35_s01_checked_manifests_validate_without_private_data(self) -> None:
        result = evaluator.validate_corpus_manifest(self.corpus, self.root)
        self.assertEqual(result["campaign_status"], "external_evidence_required")
        self.assertEqual(result["blind_custody"], "metadata-only")
        split_result = evaluator.validate_split_manifest(
            self.splits,
            corpus_sha256=evaluator.sha256_file(self.corpus_path),
        )
        self.assertEqual(split_result["targets"], evaluator.EXPECTED_SPLITS)

    def test_pf_35_s01_rejects_unapproved_license(self) -> None:
        manifest = copy.deepcopy(self.corpus)
        manifest["sources"][0]["license"] = "LicenseRef-unknown"
        with self.assertRaisesRegex(evaluator.EvaluationError, "unapproved license"):
            evaluator.validate_corpus_manifest(manifest, self.root)

    def test_pf_35_s01_rejects_generator_identity_drift(self) -> None:
        manifest = copy.deepcopy(self.corpus)
        manifest["synthetic_campaign"]["generator_model"] = "Qwen3.5-27B"
        with self.assertRaisesRegex(evaluator.EvaluationError, "generator model"):
            evaluator.validate_corpus_manifest(manifest, self.root)
        manifest = copy.deepcopy(self.corpus)
        manifest["synthetic_campaign"]["runtime"] = "vLLM"
        with self.assertRaisesRegex(evaluator.EvaluationError, "generator runtime"):
            evaluator.validate_corpus_manifest(manifest, self.root)

    def test_pf_35_s01_rejects_source_hash_drift_and_traversal(self) -> None:
        drift = copy.deepcopy(self.corpus)
        drift["sources"][0]["sha256"] = "0" * 64
        with self.assertRaisesRegex(evaluator.EvaluationError, "source hash drift"):
            evaluator.validate_corpus_manifest(drift, self.root)
        traversal = copy.deepcopy(self.corpus)
        traversal["sources"][0]["path"] = "../private-corpus.jsonl"
        with self.assertRaisesRegex(evaluator.EvaluationError, "unsafe source path"):
            evaluator.validate_corpus_manifest(traversal, self.root)

    def test_pf_35_s01_rejects_blind_record_material(self) -> None:
        manifest = copy.deepcopy(self.splits)
        manifest["splits"][2]["records"] = [{"text": "must never be public"}]
        with self.assertRaisesRegex(
            evaluator.EvaluationError, "prohibited record fields"
        ):
            evaluator.validate_split_manifest(
                manifest,
                corpus_sha256=evaluator.sha256_file(self.corpus_path),
            )

        corpus = copy.deepcopy(self.corpus)
        corpus["sources"][0]["raw_text"] = "must never be public"
        with self.assertRaisesRegex(
            evaluator.EvaluationError, "prohibited record fields"
        ):
            evaluator.validate_corpus_manifest(corpus, self.root)

        train = copy.deepcopy(self.splits)
        train["splits"][0]["records"] = [{"text": "must never be public"}]
        with self.assertRaisesRegex(
            evaluator.EvaluationError, "prohibited record fields"
        ):
            evaluator.validate_split_manifest(
                train,
                corpus_sha256=evaluator.sha256_file(self.corpus_path),
            )

        corpus = copy.deepcopy(self.corpus)
        corpus["sources"][0]["examples"] = [
            {"prompt": "denylist bypass", "body": "record material"}
        ]
        with self.assertRaisesRegex(evaluator.EvaluationError, "invalid source fields"):
            evaluator.validate_corpus_manifest(corpus, self.root)

        blind = copy.deepcopy(self.splits)
        blind["splits"][2]["blind_rows"] = [{"prompt": "record material"}]
        with self.assertRaisesRegex(evaluator.EvaluationError, "blind split fields"):
            evaluator.validate_split_manifest(
                blind,
                corpus_sha256=evaluator.sha256_file(self.corpus_path),
            )

    def test_pf_35_s01_rejects_incomplete_grouping_and_wrong_targets(self) -> None:
        missing_group = copy.deepcopy(self.splits)
        missing_group["group_by"].remove("semantic_cluster")
        with self.assertRaisesRegex(evaluator.EvaluationError, "grouping contract"):
            evaluator.validate_split_manifest(
                missing_group,
                corpus_sha256=evaluator.sha256_file(self.corpus_path),
            )
        wrong_target = copy.deepcopy(self.splits)
        wrong_target["splits"][0]["target_records"] -= 1
        with self.assertRaisesRegex(evaluator.EvaluationError, "targets differ"):
            evaluator.validate_split_manifest(
                wrong_target,
                corpus_sha256=evaluator.sha256_file(self.corpus_path),
            )

    def test_pf_35_s01_manifests_reject_unbounded_or_wrong_typed_values(self) -> None:
        oversized = copy.deepcopy(self.corpus)
        oversized["sources"][0]["quality_claim"] = "x" * 2_049
        with self.assertRaisesRegex(evaluator.EvaluationError, "source quality_claim"):
            evaluator.validate_corpus_manifest(oversized, self.root)

        nested = copy.deepcopy(self.corpus)
        nested["data_policy"]["allowed_spdx"] = [{"license": "Apache-2.0"}]
        with self.assertRaisesRegex(evaluator.EvaluationError, "license allowlist"):
            evaluator.validate_corpus_manifest(nested, self.root)

        boolean_count = copy.deepcopy(self.splits)
        boolean_count["splits"][0]["records_committed"] = False
        with self.assertRaisesRegex(evaluator.EvaluationError, "cannot be committed"):
            evaluator.validate_split_manifest(
                boolean_count,
                corpus_sha256=evaluator.sha256_file(self.corpus_path),
            )

        wrong_method = copy.deepcopy(self.splits)
        wrong_method["semantic_duplicate_method"] = "reviewer-defined"
        with self.assertRaisesRegex(evaluator.EvaluationError, "method drift"):
            evaluator.validate_split_manifest(
                wrong_method,
                corpus_sha256=evaluator.sha256_file(self.corpus_path),
            )

        nested: object = "leaf"
        for _ in range(34):
            nested = [nested]
        with self.assertRaisesRegex(evaluator.EvaluationError, "nesting depth"):
            evaluator._reject_record_material(nested, location="adversarial")

    def test_pf_35_s01_json_rejects_duplicate_keys_and_terminal_control_text(
        self,
    ) -> None:
        with self.assertRaisesRegex(evaluator.EvaluationError, "duplicate JSON"):
            evaluator._load_json_bytes(b'{"policy": true, "policy": false}', "fixture")
        with self.assertRaisesRegex(evaluator.EvaluationError, "duplicate JSON"):
            evaluator._load_predictions_bytes(
                b'{"record_id":"first","record_id":"second"}\n', "fixture"
            )

        hostile_path = "qa/missing\x1b[2Jfake-success"
        with self.assertRaises(evaluator.EvaluationError) as raised:
            evaluator._repo_file(self.root, hostile_path, "source path")
        message = str(raised.exception)
        self.assertNotIn("\x1b", message)
        self.assertIn("\\x1b", message)

        hostile_key = b'{"g\\u001b[2Jx":{"payload":1}}\n'
        with self.assertRaises(evaluator.EvaluationError) as raised:
            evaluator._load_predictions_bytes(hostile_key, "fixture")
        self.assertNotIn("\x1b", str(raised.exception))
        self.assertIn("\\x1b", str(raised.exception))

        oversized_integer = b'{"value":' + b"9" * 5_000 + b"}"
        with self.assertRaisesRegex(evaluator.EvaluationError, "cannot read JSON"):
            evaluator._load_json_bytes(oversized_integer, "fixture")

    def test_pf_35_s01_prediction_volume_is_explicitly_bounded(self) -> None:
        payload = b'{"record_id":"one"}\n{"record_id":"two"}\n'
        with mock.patch.object(evaluator, "MAX_PREDICTION_RECORDS", 1):
            with self.assertRaisesRegex(evaluator.EvaluationError, "record count"):
                evaluator._load_predictions_bytes(payload, "fixture")
        oversized_line = b"{" + b" " * evaluator.MAX_PREDICTION_LINE_BYTES + b"}"
        with self.assertRaisesRegex(evaluator.EvaluationError, "maximum length"):
            evaluator._load_predictions_bytes(oversized_line, "fixture")
        unicode_separator = json.dumps(
            {"note": "before\u2028after"}, ensure_ascii=False
        ).encode("utf-8")
        self.assertEqual(
            evaluator._load_predictions_bytes(unicode_separator, "fixture")[0]["note"],
            "before\u2028after",
        )

    def prediction(
        self, record_id: str, split: str, scope: str, expected: str, predicted: str
    ) -> dict:
        return {
            "record_id": record_id,
            "split": split,
            "family_scope": scope,
            "expected": expected,
            "predicted": predicted,
            "score": {"allow": 0.1, "suspicious": 0.5, "hostile": 0.9}[predicted],
            "groups": {
                "original_source": f"source-{record_id}",
                "base_document": f"document-{record_id}",
                "template": f"template-{record_id}",
                "attack_family": f"family-{scope}",
                "semantic_cluster": f"cluster-{record_id}",
            },
            "identity": {
                "model_id": "fixture-detector",
                "model_version": "1.0.0",
                "artifact_sha256": hashlib.sha256(b"fixture-model").hexdigest(),
                "artifact_size_mib": 42.0,
                "threshold_id": "moderate-v1",
                "threshold_sha256": hashlib.sha256(b"fixture-thresholds").hexdigest(),
                "thresholds": {"suspicious": 0.4, "hostile": 0.8},
                "corpus_manifest_sha256": evaluator.sha256_file(self.corpus_path),
                "split_manifest_sha256": evaluator.sha256_file(self.split_path),
            },
            "cpu_latency_ms": 12.5,
            "peak_rss_mib": 123.0,
        }

    def blind_aggregate(self) -> dict:
        corpus_sha = evaluator.sha256_file(self.corpus_path)
        split_sha = evaluator.sha256_file(self.split_path)
        return {
            "schema_version": 1,
            "kind": "corbanu-classifier-blind-aggregate-v1",
            "split": "blind",
            "corpus_manifest_sha256": corpus_sha,
            "split_manifest_sha256": split_sha,
            "identity": self.prediction(
                "identity-source", "development", "benign", "allow", "allow"
            )["identity"],
            "cohorts": {
                "benign": {"total": 100000, "false_positives": 0},
                "hard_negative": {"total": 10000, "rejections": 0},
                "known": {"total": 20000, "detections": 20000},
                "unseen": {"total": 20000, "detections": 20000},
            },
            "group_fingerprint_declaration": {
                "status": "external_evidence_required",
                "method_id": evaluator.GROUP_FINGERPRINT_METHOD,
                "train_group_fingerprint_sha256": "1" * 64,
                "development_group_fingerprint_sha256": "2" * 64,
                "blind_group_fingerprint_sha256": "3" * 64,
            },
            "performance_evidence": {"status": "external_evidence_required"},
            "artifact_evidence": {"status": "external_evidence_required"},
            "custodian_signature": {
                "status": "unverified",
                "custodian_id": "synthetic-test-custodian",
                "signing_key_id": "synthetic-ed25519-key-v1",
                "algorithm": "Ed25519",
                "signed_at": "2026-08-30T14:00:00Z",
                "signed_payload_sha256": "4" * 64,
                "detached_signature_base64": base64.b64encode(b"x" * 64).decode(),
            },
        }

    def evaluate_blind(self, aggregate: dict) -> dict:
        return evaluator.evaluate_blind_aggregate(
            aggregate,
            corpus_sha256=evaluator.sha256_file(self.corpus_path),
            split_sha256=evaluator.sha256_file(self.split_path),
            targets=evaluator.DEFAULT_TARGETS,
            expected_development_fingerprint=aggregate["group_fingerprint_declaration"][
                "development_group_fingerprint_sha256"
            ],
        )

    def test_pf_35_s01_metrics_are_counted_with_confidence_intervals(self) -> None:
        records = [
            self.prediction("benign-1", "development", "benign", "allow", "allow"),
            self.prediction("benign-2", "development", "benign", "allow", "suspicious"),
            self.prediction("known-1", "development", "known", "hostile", "hostile"),
            self.prediction(
                "unseen-1", "development", "unseen", "hostile", "suspicious"
            ),
            self.prediction("hard-1", "development", "hard_negative", "allow", "allow"),
        ]
        report = evaluator.evaluate_predictions(records)
        metrics = report["metrics"]
        self.assertEqual(metrics["record_count"], 5)
        self.assertEqual(metrics["benign_false_positive_rate"], 0.5)
        self.assertEqual(metrics["known_family_recall"], 1.0)
        self.assertEqual(metrics["unseen_source_recall"], 1.0)
        self.assertEqual(metrics["hard_negative_rejection_increase"], -0.5)
        self.assertEqual(len(metrics["benign_false_positive_95ci"]), 2)
        self.assertEqual(report["qualification"], "incomplete")

    def test_pf_35_s01_rejects_blind_rows_identity_mix_and_unknown_fields(self) -> None:
        first = self.prediction("a", "development", "benign", "allow", "allow")
        second = self.prediction("b", "blind", "unseen", "hostile", "hostile")
        with self.assertRaisesRegex(evaluator.EvaluationError, "aggregate-only"):
            evaluator.evaluate_predictions([first, second])

        second = self.prediction("b", "development", "benign", "allow", "allow")
        second["identity"]["model_version"] = "2.0.0"
        with self.assertRaisesRegex(evaluator.EvaluationError, "mixes model"):
            evaluator.evaluate_predictions([first, second])

        second = self.prediction("b", "development", "benign", "allow", "hostile")
        second["score"] = 0.1
        with self.assertRaisesRegex(
            evaluator.EvaluationError, "score/threshold verdict mismatch"
        ):
            evaluator.evaluate_predictions([first, second])

        second = self.prediction("b", "development", "benign", "allow", "allow")
        second["prompt_body"] = "not permitted"
        with self.assertRaisesRegex(
            evaluator.EvaluationError, "invalid prediction fields"
        ):
            evaluator.evaluate_predictions([first, second])

        second = self.prediction("b", "development", "benign", "allow", "allow")
        second["identity"]["operator_note"] = "not permitted"
        with self.assertRaisesRegex(
            evaluator.EvaluationError, "invalid identity fields"
        ):
            evaluator.evaluate_predictions([first, second])

    def test_pf_35_s01_truth_scope_and_boolean_numbers_fail_closed(self) -> None:
        contradiction = self.prediction(
            "contradiction", "development", "benign", "hostile", "allow"
        )
        with self.assertRaisesRegex(evaluator.EvaluationError, "ground truth"):
            evaluator.evaluate_predictions([contradiction])

        boolean_score = self.prediction(
            "boolean", "development", "benign", "allow", "allow"
        )
        boolean_score["score"] = False
        with self.assertRaisesRegex(evaluator.EvaluationError, "invalid score"):
            evaluator.evaluate_predictions([boolean_score])

    def test_pf_35_s01_blind_aggregate_uses_confidence_gates_but_not_hardware(
        self,
    ) -> None:
        aggregate = self.blind_aggregate()
        result = self.evaluate_blind(aggregate)
        self.assertEqual(result["evidence_scope"], "blind-aggregate-only")
        self.assertEqual(result["identity"]["verification"], "unverified-declaration")
        self.assertEqual(result["development_group_binding"]["status"], "matched")
        self.assertEqual(result["gates"]["known_family"], "pass")
        self.assertEqual(result["gates"]["unseen_source"], "pass")
        self.assertEqual(result["gates"]["cpu_latency"], "external-evidence-required")
        self.assertEqual(result["gates"]["model_size"], "external-evidence-required")
        self.assertEqual(
            result["gates"]["train_group_leakage"], "external-evidence-required"
        )
        self.assertEqual(result["qualification"], "incomplete")

        aggregate["cohorts"]["known"] = {"total": 1, "detections": 1}
        aggregate["cohorts"]["unseen"] = {"total": 39999, "detections": 39999}
        result = self.evaluate_blind(aggregate)
        self.assertEqual(result["gates"]["known_family"], "insufficient-evidence")
        self.assertLess(result["metrics"]["known_family_recall_95ci"][0], 0.80)

    def test_pf_35_s01_measured_gate_failure_is_not_insufficient_evidence(
        self,
    ) -> None:
        aggregate = self.blind_aggregate()
        aggregate["cohorts"] = {
            "benign": {"total": 100000, "false_positives": 50000},
            "hard_negative": {"total": 10000, "rejections": 9000},
            "known": {"total": 20000, "detections": 10},
            "unseen": {"total": 20000, "detections": 10},
        }
        result = self.evaluate_blind(aggregate)
        for name in (
            "benign_fpr",
            "known_family",
            "unseen_source",
            "hard_negative_perturbation",
        ):
            self.assertEqual(result["gates"][name], "fail")

    def test_pf_35_s01_development_metrics_never_claim_blind_qualification(
        self,
    ) -> None:
        record = self.prediction(
            "development-only", "development", "known", "hostile", "hostile"
        )
        result = evaluator.evaluate_predictions([record])
        self.assertEqual(
            result["gates"]["known_family"], "not-applicable-to-development"
        )
        self.assertEqual(result["qualification"], "incomplete")

    def test_pf_35_s01_hard_negative_gate_uses_conservative_confidence_bound(
        self,
    ) -> None:
        aggregate = self.blind_aggregate()
        aggregate["cohorts"] = {
            "benign": {"total": 100000, "false_positives": 0},
            "hard_negative": {"total": 1000, "rejections": 20},
            "known": {"total": 20000, "detections": 20000},
            "unseen": {"total": 29000, "detections": 29000},
        }
        result = self.evaluate_blind(aggregate)
        self.assertEqual(result["gates"]["hard_negative_perturbation"], "fail")
        self.assertGreater(
            result["metrics"]["hard_negative_rejection_increase_95ci_conservative"][1],
            0.02,
        )

    def test_pf_35_s01_blind_total_uses_declared_floor_and_tolerance(self) -> None:
        aggregate = self.blind_aggregate()
        aggregate["cohorts"]["unseen"] = {"total": 19987, "detections": 19987}
        result = self.evaluate_blind(aggregate)
        self.assertEqual(result["metrics"]["record_count"], 149987)

        aggregate["cohorts"]["unseen"] = {"total": 9986, "detections": 9986}
        with self.assertRaisesRegex(evaluator.EvaluationError, "floor/tolerance"):
            self.evaluate_blind(aggregate)

    def test_pf_35_s01_development_group_fingerprint_is_stable_and_unique(
        self,
    ) -> None:
        first = self.prediction("first", "development", "known", "hostile", "hostile")
        second = self.prediction(
            "second", "development", "unseen", "hostile", "hostile"
        )
        left = evaluator.evaluate_predictions([first, second])
        right = evaluator.evaluate_predictions([second, first])
        self.assertEqual(
            left["development_group_fingerprint"],
            right["development_group_fingerprint"],
        )

        duplicate = copy.deepcopy(first)
        duplicate["record_id"] = "duplicate"
        grouped = evaluator.evaluate_predictions([first, duplicate])
        self.assertEqual(grouped["development_group_fingerprint"]["group_count"], 1)
        self.assertEqual(
            grouped["development_group_fingerprint"]["duplicate_record_count"], 1
        )
        self.assertEqual(
            grouped["development_group_fingerprint"]["cohort_group_counts"]["known"],
            1,
        )

        contradictory = copy.deepcopy(duplicate)
        contradictory["expected"] = "suspicious"
        contradictory["predicted"] = "suspicious"
        contradictory["score"] = 0.5
        with self.assertRaisesRegex(evaluator.EvaluationError, "contradictory labels"):
            evaluator.evaluate_predictions([first, contradictory])

    def test_pf_35_s01_blind_aggregate_rejects_raw_or_unbound_material(self) -> None:
        aggregate = self.blind_aggregate()
        aggregate["records"] = [{"text": "private"}]
        with self.assertRaisesRegex(
            evaluator.EvaluationError, "blind aggregate fields"
        ):
            self.evaluate_blind(aggregate)

        aggregate = self.blind_aggregate()
        aggregate["split_manifest_sha256"] = "f" * 64
        with self.assertRaisesRegex(evaluator.EvaluationError, "wrong split manifest"):
            self.evaluate_blind(aggregate)

        aggregate = self.blind_aggregate()
        aggregate["custodian_signature"]["status"] = "verified"
        with self.assertRaisesRegex(evaluator.EvaluationError, "only as unverified"):
            self.evaluate_blind(aggregate)

        aggregate = self.blind_aggregate()
        aggregate["schema_version"] = True
        with self.assertRaisesRegex(evaluator.EvaluationError, "aggregate schema"):
            self.evaluate_blind(aggregate)

        aggregate = self.blind_aggregate()
        aggregate["group_fingerprint_declaration"]["blind_group_fingerprint_sha256"] = (
            aggregate["group_fingerprint_declaration"]["train_group_fingerprint_sha256"]
        )
        with self.assertRaisesRegex(evaluator.EvaluationError, "repeats split"):
            self.evaluate_blind(aggregate)

        aggregate = self.blind_aggregate()
        with self.assertRaisesRegex(evaluator.EvaluationError, "does not bind"):
            evaluator.evaluate_blind_aggregate(
                aggregate,
                corpus_sha256=evaluator.sha256_file(self.corpus_path),
                split_sha256=evaluator.sha256_file(self.split_path),
                targets=evaluator.DEFAULT_TARGETS,
                expected_development_fingerprint="f" * 64,
            )

    def test_pf_35_s01_prepare_report_records_external_blockers(self) -> None:
        report = evaluator.prepare_report(
            self.root, self.corpus_path, self.split_path, None
        )
        self.assertEqual(report["status"], "prepared")
        self.assertIsNone(report["evaluation"])
        self.assertEqual(len(report["external_evidence_required"]), 5)

    def test_pf_35_s01_prior_report_rejects_material_and_binds_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            temporary = Path(directory)
            predictions = temporary / "development.jsonl"
            predictions.write_text(
                json.dumps(
                    self.prediction(
                        "development", "development", "known", "hostile", "hostile"
                    )
                )
                + "\n",
                encoding="utf-8",
            )
            development_report = evaluator.prepare_report(
                self.root,
                self.corpus_path,
                self.split_path,
                predictions,
            )
            material = copy.deepcopy(development_report)
            material["evaluation"]["metrics"]["raw_text"] = "must not cross lanes"
            with self.assertRaisesRegex(evaluator.EvaluationError, "prohibited"):
                evaluator._development_fingerprint_from_report(
                    material,
                    corpus_sha256=evaluator.sha256_file(self.corpus_path),
                    split_sha256=evaluator.sha256_file(self.split_path),
                )

            mismatched_count = copy.deepcopy(development_report)
            mismatched_count["evaluation"]["metrics"]["record_count"] = 2
            with self.assertRaisesRegex(evaluator.EvaluationError, "do not reconcile"):
                evaluator._development_fingerprint_from_report(
                    mismatched_count,
                    corpus_sha256=evaluator.sha256_file(self.corpus_path),
                    split_sha256=evaluator.sha256_file(self.split_path),
                )

            development_path = temporary / "development-report.json"
            development_path.write_text(
                json.dumps(development_report) + "\n", encoding="utf-8"
            )
            aggregate = self.blind_aggregate()
            aggregate["group_fingerprint_declaration"][
                "development_group_fingerprint_sha256"
            ] = development_report["evaluation"]["development_group_fingerprint"][
                "fingerprint_sha256"
            ]
            aggregate["identity"]["model_version"] = "2.0.0"
            aggregate_path = temporary / "aggregate.json"
            aggregate_path.write_text(json.dumps(aggregate) + "\n", encoding="utf-8")
            with self.assertRaisesRegex(evaluator.EvaluationError, "does not match"):
                evaluator.prepare_report(
                    self.root,
                    self.corpus_path,
                    self.split_path,
                    None,
                    aggregate_path,
                    development_path,
                )

    def test_pf_35_s01_cli_writes_only_aggregate_report(self) -> None:
        rejected = subprocess.run(
            [
                sys.executable,
                str(self.root / "scripts/security-classifier-eval"),
                "--manifest",
                str(self.corpus_path),
                "--splits",
                str(self.split_path),
                "--repo-root",
                str(self.root),
                "--output",
                str(self.root / "qa/security-levels/classifier"),
            ],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(rejected.returncode, 1)
        self.assertIn("outside the repository", rejected.stderr)

        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(evaluator.EvaluationError, "regular file"):
                evaluator._read_regular_file(Path(directory), "fixture input")
            output = Path(directory) / "report"
            completed = subprocess.run(
                [
                    sys.executable,
                    str(self.root / "scripts/security-classifier-eval"),
                    "--manifest",
                    str(self.corpus_path),
                    "--splits",
                    str(self.split_path),
                    "--repo-root",
                    str(self.root),
                    "--output",
                    str(output),
                ],
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            path = output / "evaluation-report.json"
            report = evaluator.load_json(path)
            self.assertEqual(
                set(report),
                {
                    "schema_version",
                    "status",
                    "corpus_manifest",
                    "split_manifest",
                    "corpus_validation",
                    "split_validation",
                    "evaluation_input",
                    "development_report_input",
                    "evaluation",
                    "external_evidence_required",
                },
            )

            def keys(value: object) -> set[str]:
                if isinstance(value, dict):
                    return set(value).union(*(keys(child) for child in value.values()))
                if isinstance(value, list):
                    return set().union(*(keys(child) for child in value))
                return set()

            self.assertTrue(
                {"private_key", "customer_data", "raw_text", "records"}.isdisjoint(
                    keys(report)
                )
            )

            predictions = Path(directory) / "development-predictions.jsonl"
            predictions.write_text(
                json.dumps(
                    self.prediction(
                        "cli-input", "development", "known", "hostile", "hostile"
                    )
                )
                + "\n",
                encoding="utf-8",
            )
            evaluated_output = Path(directory) / "evaluated-report"
            completed = subprocess.run(
                [
                    sys.executable,
                    str(self.root / "scripts/security-classifier-eval"),
                    "--manifest",
                    str(self.corpus_path),
                    "--splits",
                    str(self.split_path),
                    "--predictions",
                    str(predictions),
                    "--repo-root",
                    str(self.root),
                    "--output",
                    str(evaluated_output),
                ],
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            evaluated = evaluator.load_json(evaluated_output / "evaluation-report.json")
            self.assertEqual(
                evaluated["evaluation_input"],
                {
                    "evaluator_id": evaluator.EVALUATOR_ID,
                    "kind": "development-predictions-jsonl",
                    "path": f"external/{predictions.name}",
                    "report_schema_version": evaluator.REPORT_SCHEMA_VERSION,
                    "sha256": evaluator.sha256_file(predictions),
                    "size_bytes": predictions.stat().st_size,
                },
            )
            self.assertEqual(evaluated["evaluation"]["metrics"]["record_count"], 1)

            aggregate = self.blind_aggregate()
            aggregate["group_fingerprint_declaration"][
                "development_group_fingerprint_sha256"
            ] = evaluated["evaluation"]["development_group_fingerprint"][
                "fingerprint_sha256"
            ]
            blind_input = Path(directory) / "blind-aggregate.json"
            blind_input.write_text(json.dumps(aggregate) + "\n", encoding="utf-8")
            blind_output = Path(directory) / "blind-report"
            development_report = evaluated_output / "evaluation-report.json"
            completed = subprocess.run(
                [
                    sys.executable,
                    str(self.root / "scripts/security-classifier-eval"),
                    "--manifest",
                    str(self.corpus_path),
                    "--splits",
                    str(self.split_path),
                    "--blind-aggregate",
                    str(blind_input),
                    "--development-report",
                    str(development_report),
                    "--repo-root",
                    str(self.root),
                    "--output",
                    str(blind_output),
                ],
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            blind_report = evaluator.load_json(blind_output / "evaluation-report.json")
            self.assertEqual(
                blind_report["development_report_input"]["sha256"],
                evaluator.sha256_file(development_report),
            )
            self.assertEqual(
                blind_report["evaluation"]["development_group_binding"]["status"],
                "matched",
            )
            self.assertEqual(list(blind_output.glob(".evaluation-report.*.tmp")), [])


if __name__ == "__main__":
    unittest.main()

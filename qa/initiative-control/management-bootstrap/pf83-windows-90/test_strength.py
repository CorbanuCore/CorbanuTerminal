"""Fixed counterexamples: expectations never come from the live schemas."""
import copy
from pathlib import Path
import sys
import unittest

from jsonschema import Draft202012Validator, ValidationError

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "pf83-f09gap-83"))
sys.path.insert(0, str(HERE.parent / "pf83-controls-86"))
import check_contract as contract
from frozen_fixtures import blocked_results, capture, evidence


def object_fields(value, path=()):
    """Traverse literal fixture objects, never schema required/properties."""
    if isinstance(value, dict):
        for key, child in value.items():
            yield path + (key,)
            yield from object_fields(child, path + (key,))
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from object_fields(child, path + (index,))


def results_variants():
    for status in ("blocked", "not_exercised", "accepted_disposition",
                   "passed", "failed", "inconclusive"):
        doc = blocked_results()
        row = doc["records"][0]
        row["status"] = status
        row["retained_evidence"] = [evidence()]
        if status in ("passed", "failed", "inconclusive"):
            row.update(capture=evidence(), missing_artifacts=[], prerequisites=[],
                       coverage_gaps=[])
        elif status == "accepted_disposition":
            row["product_authority"] = dict(
                name="Synthetic authority", decision="accept_scope_disposition",
                reason="Synthetic control only", record=evidence())
        yield doc


class StrengthControls(unittest.TestCase):
    def test_results_status_fixtures_are_recordable(self):
        for doc in results_variants():
            with self.subTest(status=doc["records"][0]["status"]):
                contract.check_results(doc)

    def test_results_frozen_fields_cannot_be_omitted(self):
        for doc in results_variants():
            validator = Draft202012Validator(contract.RESULTS)
            for path in object_fields(doc):
                with self.subTest(status=doc["records"][0]["status"], path=path):
                    missing = copy.deepcopy(doc)
                    parent = missing
                    for key in path[:-1]:
                        parent = parent[key]
                    del parent[path[-1]]
                    with self.assertRaises(ValidationError):
                        validator.validate(missing)

    def test_capture_frozen_value_constraints(self):
        # Literal invalid values exercise range, enum, type, pattern and empty text.
        for path, value in (
            (("authority_observations", 0, "observer_sequence"), 0),
            (("authority_observations", 0, "effect_count"), -1),
            (("authority_observations", 0, "claimed_effective"), "unknown"),
            (("authority_observations", 0, "marker_exists"), "true"),
            (("authority_observations", 0, "monotonic_ns"), -1),
            (("queue_packet_sha256",), "not-a-digest"),
            (("attempt_id",), ""),
            (("capture_identity", "artifact"), "../outside"),
        ):
            with self.subTest(path=path, value=value):
                doc = capture()
                parent = doc
                for key in path[:-1]:
                    parent = parent[key]
                parent[path[-1]] = value
                with self.assertRaises(ValidationError):
                    Draft202012Validator(contract.CAPTURE).validate(doc)

    def test_results_frozen_value_constraints(self):
        for field, value in (
            ("last_checkpoint", ""),
            ("missing_artifacts", ["unknown"]),
            ("prerequisites", ["unknown"]),
            ("coverage_gaps", ["unknown"]),
            ("missing_artifacts", ["admission", "admission"]),
            ("queue_packet_sha256", "not-a-digest"),
            ("attempt_id", ""),
            ("status", "unknown"),
        ):
            with self.subTest(field=field, value=value):
                doc = blocked_results()
                doc["records"][0][field] = value
                with self.assertRaises(ValidationError):
                    Draft202012Validator(contract.RESULTS).validate(doc)

    def test_f07_gap_and_dwell_boundary_are_recordable(self):
        # Keep the post-dwell row; append honest observations through the old gap.
        for timestamp in (15, 16, 17, 18, 19):
            for approved in (False, True):
                with self.subTest(time=timestamp, approved=approved):
                    doc = capture("F07")
                    row = copy.deepcopy(doc["authority_observations"][1])
                    row.update(monotonic_ns=timestamp, first_effect_ns=timestamp)
                    if approved:
                        row.update(approval_id="gap-approval", approval_scope="late",
                                   approval_disposition="accepted",
                                   approval_decision_ns=timestamp)
                    doc["authority_observations"].append(row)
                    contract.check_capture(doc)

    def test_f07_post_dwell_row_can_report_earlier_effect_or_approval(self):
        for timestamp in (15, 16, 17, 18):
            doc = capture("F07")
            row = doc["authority_observations"][1]
            row.update(first_effect_ns=timestamp, approval_id="late-approval",
                       approval_scope="late", approval_disposition="accepted",
                       approval_decision_ns=timestamp)
            contract.check_capture(doc)

    def test_f07_gap_only_cannot_replace_post_dwell_coverage(self):
        for timestamp in (15, 16, 18):
            doc = capture("F07")
            doc["authority_observations"][1].update(
                monotonic_ns=timestamp, first_effect_ns=timestamp)
            with self.assertRaisesRegex(ValueError, "after the frozen dwell"):
                contract.check_capture(doc)

    def test_f09_outer_phase_valid_effect_and_approval_are_recordable(self):
        for index, times in ((0, (0, 1, 10)), (2, (26, 29, 30))):
            for timestamp in times:
                doc = capture()
                row = doc["authority_observations"][index]
                row.update(approval_id="outer-approval", approval_scope=row["phase"],
                           approval_disposition="accepted", approval_decision_ns=timestamp,
                           effect_count=1, first_effect_ns=timestamp, marker_exists=True)
                contract.check_capture(doc)

"""Synthetic counterexamples; no transport, credential access or case execution."""
import copy
import unittest

from jsonschema import ValidationError

import check_contract as contract


def evidence(artifact="synthetic.raw"):
    return dict(artifact=artifact, sha256="0" * 64, locator="line:1", observed="synthetic only")


def capture(case="F09"):
    rule = next(r["then"] for r in contract.CAPTURE["allOf"]
                if r["if"]["properties"]["case"]["const"] == case)
    doc = dict(case=case, attempt_id="synthetic-1", branch=rule["properties"]["branch"]["enum"][0],
               profile="fresh", repository="tensorcash", queue_variant="synthetic",
               queue_packet_sha256="0" * 64, retention_policy="pf83-synthetic-capture-v1",
               capture_identity=evidence(), verdict="failed")
    for section in ("pre_state", "observable", "post_state", "negative_control"):
        doc[section] = {key: evidence() for key in rule["properties"][section]["required"]}
    artifact = rule["x-indispensable-artifact"]
    doc["refuting_artifact"] = evidence(artifact)
    phases = ["pre_change", "in_continuation", "new_turn"] if case == "F09" else ["settled", "late"]
    doc["authority_observations"] = [dict(
        phase=phase, turn_or_session_id="new" if phase == "new_turn" else "old",
        probe_label=phase, claimed_effective="full", monotonic_ns=(i + 1) * 10,
        approval_id=None, approval_disposition="not_offered", approval_decision_ns=None,
        approval_scope=None, marker_exists=True, effect_count=1, first_effect_ns=(i + 1) * 10,
        observer_sequence=i + 1, source=evidence(artifact)) for i, phase in enumerate(phases)]
    if case == "F09":
        doc["continuation_window"] = dict(captured_turn_id="old", new_turn_id="new",
                                          selection_ack_ns=15, turn_ended_ns=25, source=evidence())
    return doc


def blocked_results():
    expected = []
    for rule in contract.CAPTURE["allOf"]:
        for branch in rule["then"]["properties"]["branch"]["enum"]:
            expected.append(dict(case=rule["if"]["properties"]["case"]["const"], branch=branch,
                                 profile="fresh", repository="tensorcash", queue_variant="synthetic",
                                 queue_packet_sha256="0" * 64))
    records = [dict(row, attempt_id=f"synthetic-{i}", status="blocked", capture=None,
                    missing_artifacts=["admission"], prerequisites=["admitted_executor"],
                    coverage_gaps=["original_branch"], last_checkpoint="No dispatch; synthetic test",
                    retained_evidence=[], product_authority=None) for i, row in enumerate(expected)]
    return dict(schema_version=1, contract_sha256="0" * 64, queue_manifest=evidence(),
                coverage_manifest=dict(frozen_manifest=evidence(), expected=expected), records=records)


class ContractCounterexamples(unittest.TestCase):
    def test_meta_validation(self):
        contract.meta_validate()

    def test_f09_missing_continuation_is_rejected(self):
        doc = capture()
        doc["authority_observations"].pop(1)
        with self.assertRaises(ValidationError):
            contract.check_capture(doc)

    def test_f09_new_turn_disguised_as_continuation_is_rejected(self):
        doc = capture()
        doc["authority_observations"][1]["turn_or_session_id"] = "new"
        with self.assertRaisesRegex(ValueError, "identity/order"):
            contract.check_capture(doc)

    def test_f09_probe_before_selection_or_after_end_is_rejected(self):
        for timestamp in (14, 15, 25, 26):
            with self.subTest(timestamp=timestamp):
                doc = capture()
                doc["authority_observations"][1]["monotonic_ns"] = timestamp
                with self.assertRaises(ValueError):
                    contract.check_capture(doc)

    def test_f09_retroactive_restriction_remains_recordable_failure(self):
        doc = capture()
        row = doc["authority_observations"][1]
        row.update(approval_id="command-1", approval_scope="in_continuation",
                   approval_disposition="withheld", marker_exists=False,
                   effect_count=0, first_effect_ns=None)
        contract.check_capture(doc)
        self.assertNotEqual(row["effect_count"], doc["authority_observations"][0]["effect_count"])

    def test_f07_approval_fields_are_indispensable(self):
        for field in ("approval_id", "approval_disposition", "approval_decision_ns",
                      "approval_scope", "first_effect_ns", "marker_exists"):
            with self.subTest(field=field):
                doc = capture("F07")
                del doc["authority_observations"][0][field]
                with self.assertRaises(ValidationError):
                    contract.check_capture(doc)

    def test_f07_approved_and_silent_effects_are_distinguishable(self):
        silent = capture("F07")
        approved = copy.deepcopy(silent)
        row = approved["authority_observations"][0]
        row.update(claimed_effective="restricted", approval_id="command-2",
                   approval_disposition="accepted", approval_scope="settled", approval_decision_ns=9)
        silent["authority_observations"][0]["claimed_effective"] = "restricted"
        contract.check_capture(approved)
        contract.check_capture(silent)
        self.assertLess(row["approval_decision_ns"], row["first_effect_ns"])
        self.assertIsNone(silent["authority_observations"][0]["approval_id"])

    def test_f07_acceptance_without_id_is_rejected(self):
        doc = capture("F07")
        doc["authority_observations"][0]["approval_disposition"] = "accepted"
        with self.assertRaises(ValidationError):
            contract.check_capture(doc)

    def test_unknown_and_wrong_case_branches_are_rejected(self):
        for branch in ("whatever happened", "withhold-decline"):
            with self.subTest(branch=branch):
                doc = capture()
                doc["branch"] = branch
                with self.assertRaises(ValidationError):
                    contract.check_capture(doc)

    def test_complete_blocked_coverage_is_recordable(self):
        contract.check_results(blocked_results())

    def test_missing_result_and_erased_original_branch_are_rejected(self):
        for erase_manifest in (False, True):
            with self.subTest(erase_manifest=erase_manifest):
                doc = blocked_results()
                doc["records"].pop()
                if erase_manifest:
                    doc["coverage_manifest"]["expected"].pop()
                with self.assertRaises(ValueError):
                    contract.check_results(doc)

    def test_unspecified_blocker_and_unapproved_disposition_are_rejected(self):
        for change in (dict(prerequisites=[]), dict(status="accepted_disposition"),
                       dict(coverage_gaps=["free text"]), dict(status="passed")):
            with self.subTest(change=change):
                doc = blocked_results()
                doc["records"][0].update(change)
                with self.assertRaises(ValidationError):
                    contract.check_results(doc)


if __name__ == "__main__":
    unittest.main(verbosity=2)

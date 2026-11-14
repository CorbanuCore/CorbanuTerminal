"""Synthetic counterexamples; no transport, credential access or case execution."""
import copy
import unittest

from jsonschema import Draft202012Validator, ValidationError
from unittest.mock import patch

import check_contract as contract


# Frozen hand-written inputs have no dependency on the schemas under test.
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "pf83-controls-86"))
from frozen_fixtures import BRANCHES, CASES, blocked_results, capture


class ContractCounterexamples(unittest.TestCase):
    def test_meta_validation(self):
        contract.meta_validate()

    def test_all_frozen_cases_and_branches_are_recordable(self):
        for case, branches in BRANCHES.items():
            for _, branch in branches:
                with self.subTest(case=case, branch=branch):
                    doc = capture(case)
                    doc["branch"] = branch
                    contract.check_capture(doc)

    def test_frozen_observations_cannot_be_omitted(self):
        # Paths come ONLY from hand-written fixtures, not the contract.
        for case in CASES:
            doc = capture(case)
            paths = [(key,) for key in doc]
            for section in ("pre_state", "observable", "post_state", "negative_control"):
                paths.extend((section, key) for key in doc[section])
            for section in ("late_window", "continuation_window"):
                if section in doc:
                    paths.extend((section, key) for key in doc[section])
            for i, row in enumerate(doc.get("authority_observations", [])):
                paths.extend(("authority_observations", i, key) for key in row)
            for path in paths:
                with self.subTest(case=case, path=path):
                    missing = copy.deepcopy(doc)
                    parent = missing
                    for key in path[:-1]:
                        parent = parent[key]
                    del parent[path[-1]]
                    with self.assertRaises(ValidationError):
                        Draft202012Validator(contract.CAPTURE).validate(missing)

    def test_frozen_phase_and_artifact_requirements(self):
        for case, phases in (("F07", ("settled", "late")),
                             ("F09", ("pre_change", "in_continuation", "new_turn"))):
            for phase in phases:
                with self.subTest(case=case, phase=phase):
                    doc = capture(case)
                    rows = [r for r in doc["authority_observations"] if r["phase"] != phase]
                    # Preserve length so minItems cannot conceal loss of contains.
                    rows.append(copy.deepcopy(rows[0]))
                    doc["authority_observations"] = rows
                    with self.assertRaises(ValidationError):
                        Draft202012Validator(contract.CAPTURE).validate(doc)
        for case in CASES:
            with self.subTest(case=case):
                doc = capture(case)
                doc["refuting_artifact"]["artifact"] = "unrelated.raw"
                with self.assertRaises(ValidationError):
                    contract.check_capture(doc)

    def test_f07_late_row_must_match_frozen_turn_and_window(self):
        for change in (dict(turn_or_session_id="old"),
                       *(dict(monotonic_ns=t) for t in (14, 15, 17, 18, 25, 26))):
            with self.subTest(change=change):
                doc = capture("F07")
                doc["authority_observations"][1].update(change)
                with self.assertRaises(ValueError):
                    contract.check_capture(doc)
        for change in (dict(settled_turn_id="new"), dict(settlement_ns=15),
                       dict(turn_ended_ns=25), dict(dwell_until_ns=5),
                       dict(dwell_until_ns=25), dict(observation_ended_ns=18)):
            with self.subTest(window=change):
                doc = capture("F07")
                doc["late_window"].update(change)
                with self.assertRaises(ValueError):
                    contract.check_capture(doc)

    def test_f07_settled_row_and_distinct_probes_are_bound(self):
        for change in (dict(turn_or_session_id="new"), dict(monotonic_ns=5),
                       dict(monotonic_ns=15), dict(probe_label="late")):
            with self.subTest(change=change):
                doc = capture("F07")
                doc["authority_observations"][0].update(change)
                with self.assertRaises(ValueError):
                    contract.check_capture(doc)

    def test_effect_and_approval_must_be_inside_probe_window(self):
        for case, row_index, outside in (
            ("F07", 0, (4, 5, 11, 15, 16)),
            ("F07", 1, (14, 15, 18, 21, 25, 26)),
            ("F09", 1, (14, 15, 21, 25, 26)),
        ):
            for field in ("first_effect_ns", "approval_decision_ns"):
                for timestamp in outside:
                    with self.subTest(case=case, row=row_index, field=field, time=timestamp):
                        doc = capture(case)
                        row = doc["authority_observations"][row_index]
                        if field == "approval_decision_ns":
                            row.update(approval_id="probe-approval", approval_scope=row["phase"],
                                       approval_disposition="accepted")
                        row[field] = timestamp
                        with self.assertRaisesRegex(ValueError, "outside phase window"):
                            contract.check_capture(doc)

    def test_continuation_approval_and_effect_inside_window_are_recordable(self):
        doc = capture()
        row = doc["authority_observations"][1]
        row.update(approval_id="probe-approval", approval_scope="in_continuation",
                   approval_disposition="accepted", approval_decision_ns=17, first_effect_ns=19)
        contract.check_capture(doc)
        # Contradictory ordering is evidence of failure, not a schema success rule.
        row.update(approval_decision_ns=19, first_effect_ns=17)
        contract.check_capture(doc)

    def test_schema_branch_rename_or_drop_is_rejected(self):
        for target in ("capture", "coverage_tuple", "result", "annotation"):
            for mutation in ("rename", "drop"):
                with self.subTest(target=target, mutation=mutation):
                    schema = copy.deepcopy(contract.CAPTURE if target in ("capture", "annotation")
                                           else contract.RESULTS)
                    rules = schema["allOf"] if target in ("capture", "annotation") else (
                        schema["$defs"][target]["allOf"])
                    rule = rules[0]["then"]
                    branches = rule["x-required-branches"] if target == "annotation" else (
                        rule["properties"]["branch"]["enum"])
                    if mutation == "rename":
                        branches[0] = "renamed-branch"
                    else:
                        branches.pop()
                    name = "CAPTURE" if target in ("capture", "annotation") else "RESULTS"
                    with patch.object(contract, name, schema), self.assertRaisesRegex(
                            ValueError, "frozen round-82 binding"):
                        contract.meta_validate()

    def test_round82_binding_matches_handwritten_cases(self):
        self.assertEqual(contract.frozen_branches(),
                         {case: [identifier for _, identifier in rows]
                          for case, rows in BRANCHES.items()})
        # Changing the source or mapping itself must not silently rebaseline.
        from pathlib import Path
        original_read = Path.read_bytes
        for filename in ("branch-bindings.json", "evidence-contract.schema.json"):
            def changed_read(path):
                raw = original_read(path)
                return raw + b" " if path.name == filename else raw
            with self.subTest(file=filename), patch.object(Path, "read_bytes", changed_read):
                with self.assertRaisesRegex(ValueError, "digest mismatch"):
                    contract.frozen_branches()

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

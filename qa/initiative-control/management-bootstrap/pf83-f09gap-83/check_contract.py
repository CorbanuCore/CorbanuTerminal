"""Structural checks only; external raw evidence and independence need review."""
import hashlib
import json
from pathlib import Path
import sys

from jsonschema import Draft202012Validator

HERE = Path(__file__).resolve().parent
CAPTURE = json.loads((HERE / "evidence-contract.schema.json").read_text())
RESULTS = json.loads((HERE / "results.schema.json").read_text())
KEY = ("case", "branch", "profile", "repository", "queue_variant", "queue_packet_sha256")


def frozen_branches():
    raw = (HERE.parent / "pf83-controls-86" / "branch-bindings.json").read_bytes()
    if hashlib.sha256(raw).hexdigest() != "17970976834f9f48760ee8cb3e0c1fc552005648fe3a5a48c1dde18894d97159":
        raise ValueError("frozen branch binding digest mismatch")
    binding = json.loads(raw)
    original = (HERE.parent / "pf83-capture-82" / "evidence-contract.schema.json").read_bytes()
    if hashlib.sha256(original).hexdigest() != binding["round82_sha256"]:
        raise ValueError("round-82 frozen contract digest mismatch")
    originals = {rule["if"]["properties"]["case"]["const"]: rule["then"]["x-required-branches"]
                 for rule in json.loads(original)["allOf"]}
    if {case: list(rows) for case, rows in binding["branches"].items()} != originals:
        raise ValueError("binding must preserve every frozen round-82 branch")
    return {case: list(rows.values()) for case, rows in binding["branches"].items()}


def check_branch_bindings():
    expected = frozen_branches()
    for rules in (CAPTURE["allOf"], RESULTS["$defs"]["coverage_tuple"]["allOf"],
                  RESULTS["$defs"]["result"]["allOf"]):
        pairs = [(rule["if"]["properties"]["case"]["const"],
                  rule["then"]["properties"]["branch"]["enum"])
                 for rule in rules if "case" in rule.get("if", {}).get("properties", {})]
        if len(pairs) != len(expected) or dict(pairs) != expected:
            raise ValueError("schema branch identifiers drifted from frozen round-82 binding")
    for rule in CAPTURE["allOf"]:
        case = rule["if"]["properties"]["case"]["const"]
        if rule["then"]["x-required-branches"] != expected[case]:
            raise ValueError("required branch annotation drifted from frozen round-82 binding")


def meta_validate():
    for schema in (CAPTURE, RESULTS):
        Draft202012Validator.check_schema(schema)
    check_branch_bindings()


def check_interval(row, turn, lower, upper):
    if row["turn_or_session_id"] != turn or not lower < row["monotonic_ns"] < upper:
        raise ValueError("phase identity/order mismatch: " + row["phase"])
    for field in ("first_effect_ns", "approval_decision_ns"):
        timestamp = row[field]
        if timestamp is not None and not lower < timestamp <= row["monotonic_ns"] < upper:
            raise ValueError(row["phase"] + " " + field + " outside phase window")


def check_probe_labels(document):
    phases_by_label = {}
    for row in document["authority_observations"]:
        phases_by_label.setdefault(row["probe_label"], set()).add(row["phase"])
    if any(len(phases) != 1 for phases in phases_by_label.values()):
        raise ValueError(document["case"] + " phase probes must have distinct labels")


def check_capture(document):
    check_branch_bindings()
    Draft202012Validator(CAPTURE).validate(document)
    if document["case"] == "F07":
        window = document["late_window"]
        settled, late = window["settled_turn_id"], window["late_turn_id"]
        start, boundary = window["settlement_ns"], window["turn_ended_ns"]
        dwell, end = window["dwell_until_ns"], window["observation_ended_ns"]
        if settled == late or not start < boundary < end or not start < dwell < end:
            raise ValueError("F07 requires distinct turns and an ordered frozen window")
        for row in document["authority_observations"]:
            if row["phase"] == "settled":
                check_interval(row, settled, start, boundary)
            else:
                check_interval(row, late, max(boundary, dwell), end)
        check_probe_labels(document)
    if document["case"] != "F09":
        return
    window = document["continuation_window"]
    old, new = window["captured_turn_id"], window["new_turn_id"]
    selected, ended = window["selection_ack_ns"], window["turn_ended_ns"]
    if old == new or selected >= ended:
        raise ValueError("F09 requires distinct turns and selection before turn end")
    for row in document["authority_observations"]:
        phase, timestamp = row["phase"], row["monotonic_ns"]
        valid = {
            "pre_change": row["turn_or_session_id"] == old and timestamp < selected,
            "in_continuation": row["turn_or_session_id"] == old and selected < timestamp < ended,
            "new_turn": row["turn_or_session_id"] == new and ended < timestamp,
        }[phase]
        if not valid:
            raise ValueError("F09 phase identity/order mismatch: " + phase)
        if phase == "in_continuation":
            check_interval(row, old, selected, ended)
    # Repeated samples of the same probe are allowed within a phase only.
    check_probe_labels(document)


def check_results(document):
    check_branch_bindings()
    Draft202012Validator(RESULTS).validate(document)
    expected = {tuple(row[key] for key in KEY)
                for row in document["coverage_manifest"]["expected"]}
    actual = {tuple(row[key] for key in KEY) for row in document["records"]}
    if expected != actual:
        raise ValueError("missing/unexpected coverage tuples")
    branches = {(row["case"], row["branch"])
                for row in document["coverage_manifest"]["expected"]}
    required = {(case, branch) for case, branches in frozen_branches().items()
                for branch in branches}
    if branches != required:
        raise ValueError("coverage manifest must preserve all original branches")
    attempts = [row["attempt_id"] for row in document["records"]]
    if len(attempts) != len(set(attempts)):
        raise ValueError("duplicate attempt IDs; preserve attempts separately")


if __name__ == "__main__":
    meta_validate()
    if len(sys.argv) == 1:
        print("Draft202012Validator.check_schema: 2 schemas PASS")
    elif len(sys.argv) == 3 and sys.argv[1] in ("capture", "results"):
        document = json.loads(Path(sys.argv[2]).read_text())
        {"capture": check_capture, "results": check_results}[sys.argv[1]](document)
        print(sys.argv[1] + ": structural checks PASS; raw-evidence review still required")
    else:
        raise SystemExit("usage: check_contract.py [capture|results FILE]")

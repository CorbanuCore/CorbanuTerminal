"""Structural checks only; external raw evidence and independence need review."""
import json
from pathlib import Path
import sys

from jsonschema import Draft202012Validator

HERE = Path(__file__).resolve().parent
CAPTURE = json.loads((HERE / "evidence-contract.schema.json").read_text())
RESULTS = json.loads((HERE / "results.schema.json").read_text())
KEY = ("case", "branch", "profile", "repository", "queue_variant", "queue_packet_sha256")


def meta_validate():
    for schema in (CAPTURE, RESULTS):
        Draft202012Validator.check_schema(schema)


def check_capture(document):
    Draft202012Validator(CAPTURE).validate(document)
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
    # Repeated samples of the same probe are allowed within a phase only.
    phases_by_label = {}
    for row in document["authority_observations"]:
        phases_by_label.setdefault(row["probe_label"], set()).add(row["phase"])
    if any(len(phases) != 1 for phases in phases_by_label.values()):
        raise ValueError("F09 phase probes must have distinct labels")


def check_results(document):
    Draft202012Validator(RESULTS).validate(document)
    expected = {tuple(row[key] for key in KEY)
                for row in document["coverage_manifest"]["expected"]}
    actual = {tuple(row[key] for key in KEY) for row in document["records"]}
    if expected != actual:
        raise ValueError("missing/unexpected coverage tuples")
    branches = {(row["case"], row["branch"])
                for row in document["coverage_manifest"]["expected"]}
    required = {(rule["if"]["properties"]["case"]["const"], branch)
                for rule in CAPTURE["allOf"]
                for branch in rule["then"]["properties"]["branch"]["enum"]}
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

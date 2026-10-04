"""Hand-written synthetic expectations from the F05–F09 case checkpoints.

No schema imports, schema traversal, or generated observations. These are test
inputs, not executed evidence. Change only with an explicit case-contract review.
"""
import copy


def evidence(artifact="synthetic.raw"):
    return dict(artifact=artifact, sha256="0" * 64, locator="line:1",
                observed="Synthetic control only; not executed evidence")


# Exact original text paired with its round-83 machine identifier.
BRANCHES = {
    "F05": [
        ("withhold then decline", "withhold-decline"),
        ("withhold then accept", "withhold-accept"),
        ("disclosed replacement/cancellation, if offered", "offered-replacement"),
    ],
    "F06": [
        ("Full Access to restricted", "full-to-restricted"),
        ("restricted to Full Access with explicit approval of slow operation",
         "restricted-to-full-approved-slow"),
    ],
    "F07": [
        ("Full Access to restricted to Full Access", "full-restricted-full"),
        ("restricted to Full Access to restricted", "restricted-full-restricted"),
        ("later conflict while application confirmation open", "open-confirmation-conflict"),
    ],
    "F08": [
        ("cancel from restricted", "cancel-from-restricted"),
        ("cancel from Full Access", "cancel-from-full"),
        ("exposed benign failure from restricted", "failure-from-restricted"),
        ("exposed benign failure from Full Access", "failure-from-full"),
    ],
    "F09": [
        ("Full Access to restricted", "full-to-restricted"),
        ("restricted to Full Access", "restricted-to-full"),
        ("deferred change when supported", "supported-deferred-change"),
    ],
}

# Literal case observations: never consult required/properties in a schema.
CASES = {
    "F05": dict(
        branch="withhold-decline",
        pre_state=dict(pending_request=evidence(), zero_effect_baseline=evidence()),
        observable=dict(withheld_interval=evidence(), explicit_resolution=evidence()),
        post_state=dict(original_effects=evidence(), distinct_work=evidence()),
        negative_control=dict(withheld_probe=evidence()),
        refuting_artifact=evidence("approval-effects.jsonl")),
    "F06": dict(
        branch="full-to-restricted",
        pre_state=dict(running_witness=evidence(), authority=evidence()),
        observable=dict(change_while_running=evidence(), separate_probe=evidence()),
        post_state=dict(slow_result=evidence(), boundary_probe=evidence()),
        negative_control=dict(distinct_labels=evidence()),
        refuting_artifact=evidence("operation-boundary.jsonl")),
    "F07": dict(
        branch="full-restricted-full",
        pre_state=dict(active_turn=evidence(), pending_opportunity=evidence()),
        observable=dict(all_selections=evidence(), conflict=evidence()),
        post_state=dict(settled_authority=evidence(), late_observation=evidence()),
        negative_control=dict(superseded_or_refused=evidence()),
        refuting_artifact=evidence("selection-order.jsonl"),
        late_window=dict(settled_turn_id="old", late_turn_id="new",
                         settlement_ns=5, turn_ended_ns=15,
                         dwell_until_ns=18, observation_ended_ns=25, source=evidence()),
        authority_observations=[
            dict(phase="settled", turn_or_session_id="old", probe_label="settled",
                 claimed_effective="full", monotonic_ns=10, approval_id=None,
                 approval_disposition="not_offered", approval_decision_ns=None,
                 approval_scope=None, marker_exists=True, effect_count=1,
                 first_effect_ns=10, observer_sequence=1,
                 source=evidence("selection-order.jsonl")),
            dict(phase="late", turn_or_session_id="new", probe_label="late",
                 claimed_effective="full", monotonic_ns=20, approval_id=None,
                 approval_disposition="not_offered", approval_decision_ns=None,
                 approval_scope=None, marker_exists=True, effect_count=1,
                 first_effect_ns=20, observer_sequence=2,
                 source=evidence("selection-order.jsonl")),
        ]),
    "F08": dict(
        branch="cancel-from-restricted",
        pre_state=dict(effective_authority=evidence(), exposed_route=evidence()),
        observable=dict(resolution=evidence(), recovery=evidence()),
        post_state=dict(remaining_authority=evidence(), recovery_result=evidence()),
        negative_control=dict(unsuccessful_request=evidence()),
        refuting_artifact=evidence("resolution-authority.jsonl")),
    "F09": dict(
        branch="full-to-restricted",
        pre_state=dict(completed_output=evidence(), selected_change=evidence(),
                       captured_authority=evidence()),
        observable=dict(continuation_path=evidence(), effect_watch=evidence(),
                        continuation_authority=evidence()),
        post_state=dict(new_turn_authority=evidence(), retained_state=evidence()),
        negative_control=dict(no_replay=evidence()),
        refuting_artifact=evidence("continuation-effects.jsonl"),
        continuation_window=dict(captured_turn_id="old", new_turn_id="new",
                                 selection_ack_ns=15, turn_ended_ns=25, source=evidence()),
        authority_observations=[
            dict(phase="pre_change", turn_or_session_id="old", probe_label="pre_change",
                 claimed_effective="full", monotonic_ns=10, approval_id=None,
                 approval_disposition="not_offered", approval_decision_ns=None,
                 approval_scope=None, marker_exists=True, effect_count=1,
                 first_effect_ns=10, observer_sequence=1,
                 source=evidence("continuation-effects.jsonl")),
            dict(phase="in_continuation", turn_or_session_id="old",
                 probe_label="in_continuation", claimed_effective="full", monotonic_ns=20,
                 approval_id=None, approval_disposition="not_offered",
                 approval_decision_ns=None, approval_scope=None, marker_exists=True,
                 effect_count=1, first_effect_ns=20, observer_sequence=2,
                 source=evidence("continuation-effects.jsonl")),
            dict(phase="new_turn", turn_or_session_id="new", probe_label="new_turn",
                 claimed_effective="restricted", monotonic_ns=30, approval_id="new-approval",
                 approval_disposition="withheld", approval_decision_ns=None,
                 approval_scope="new_turn", marker_exists=False, effect_count=0,
                 first_effect_ns=None, observer_sequence=3,
                 source=evidence("continuation-effects.jsonl")),
        ]),
}


def capture(case="F09"):
    return dict(copy.deepcopy(CASES[case]), case=case, attempt_id="synthetic-1",
                profile="fresh", repository="tensorcash", queue_variant="synthetic",
                queue_packet_sha256="0" * 64, retention_policy="pf83-synthetic-capture-v1",
                capture_identity=evidence(), verdict="failed")


def blocked_results():
    # Expansion of the frozen hand-written branch cases, never of schema enums.
    expected = [
        dict(case=case, branch=branch, profile="fresh", repository="tensorcash",
             queue_variant="synthetic", queue_packet_sha256="0" * 64)
        for case, branches in BRANCHES.items() for _, branch in branches
    ]
    records = [
        dict(row, attempt_id=f"synthetic-{i}", status="blocked", capture=None,
             missing_artifacts=["admission"], prerequisites=["admitted_executor"],
             coverage_gaps=["original_branch"], last_checkpoint="No dispatch; synthetic test",
             retained_evidence=[], product_authority=None)
        for i, row in enumerate(expected)
    ]
    return dict(schema_version=1, contract_sha256="0" * 64, queue_manifest=evidence(),
                coverage_manifest=dict(frozen_manifest=evidence(), expected=expected),
                records=records)

"""Replay the assigned base regression and supported recovery in a disposable store."""
import json
import subprocess
import types

import decisions as d
import slack_transport as current
from test_slack_transport import TransportTests, ui_evidence

BASE = "bc4493331f390801ef14016caeceae5c2ca94bab"
source = subprocess.check_output(["git", "show", BASE + ":scripts/initiative_control/slack_transport.py"])
prior = types.ModuleType("owner_assurance_109_prior_transport")
exec(compile(source, BASE + ":slack_transport.py", "exec"), prior.__dict__)
case = TransportTests("test_invalid_review_does_not_pin_healthy_quiet_journal")
case.setUp()
try:
    case.sending()
    result = dict(base=BASE, initial_hold=case.store.read("transport")["hold"])
    try:
        prior.Transport.qualify(case.transport, ui_evidence(), {})
        raise AssertionError("base accepted invalid review")
    except d.Invalid:
        result["base_invalid_review_rejected"] = True
    result["hold_after_base_rejection"] = case.store.read("transport")["hold"]
    assert result["hold_after_base_rejection"] == "qualifying"
    try:
        prior.Transport.qualify(case.transport, ui_evidence())
        raise AssertionError("base renewed without the required review")
    except d.Invalid:
        result["base_plain_renewal_rejected"] = True
    result["hold_after_base_renewal"] = case.store.read("transport")["hold"]
    case.review_gap()
    result["hold_after_current_exact_review"] = case.store.read("transport")["hold"]
    assert result["hold_after_current_exact_review"] is None
    before = (case.root / "transport.json").read_bytes()
    try:
        current.Transport.qualify(case.transport, ui_evidence(), {})
        raise AssertionError("current accepted invalid review")
    except d.Invalid:
        result["current_invalid_review_rejected"] = True
    result["current_rejection_preserves_journal"] = (case.root / "transport.json").read_bytes() == before
    assert result["current_rejection_preserves_journal"]
    result["live_store_accessed"] = False
    print(json.dumps(result, sort_keys=True))
finally:
    case.doCleanups()

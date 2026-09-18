"""Disposable real journal -> saved projection -> renderer evidence, not acceptance."""
import html
import json

import decisions as d
import slack_transport as s
from test_decision_alerts import PIN
from test_decision_manager import ManagerTests
from test_decisions import NOW
from test_slack_transport import TransportTests


def capture(case, label, at):
    projected, health, rendered = case.rendered_projection(at)
    journal = case.store.read("transport")
    notice = html.unescape(rendered.split("<p>", 1)[1].split("</p>", 1)[0])
    print(json.dumps(dict(scenario=label, hold=journal["hold"],
        status=projected["status"], health=health, rendered_notice=notice,
        outstanding=s.outstanding_quarantine(journal),
        retained_expired=sum(row["reason"] == "unbound-expired"
            for row in journal.get("quarantine", {}).get("records", []))), sort_keys=True))


def observe(scenario):
    case = ManagerTests("test_unknown_history_guidance_renders_through_saved_projection_and_clears")
    case.setUp()
    try:
        at = NOW
        if scenario == "unknown":
            TransportTests.quiet_legacy_unknown(case)
            capture(case, "unknown-history-without-hold", at)
        elif scenario == "expired":
            TransportTests.unbind_fixture(case)
            case.callback()
            case.review_gap()
            deadline = case.store.read("transport")["held_human"]["Ev001"]["expires_at"]
            at = (d.stamp(deadline) + s.dt.timedelta(seconds=1)).strftime("%Y-%m-%dT%H:%M:%SZ")
            case.transport.now = case.owner.now = lambda: at
            case.owner.update("connected")
            case.store.write("supervisor", dict(binding=PIN, health=dict(
                state="healthy", event_flush_failures=0, pending_events=0, observed_at=at, reason=None)))
            capture(case, "expired-nonheld-stale-projection", at)
        else:
            case.sending()
            code = "invalid_auth" if scenario == "auth" else "missing_scope"
            case.reply = lambda *_: (dict(ok=False, error=code), 200, {})
            try:
                case.review_gap()
            except s.QualificationRejected:
                pass
            else:
                raise AssertionError("fixture rejection unexpectedly qualified")
            capture(case, scenario + "-rejected", at)
            case.reply = None
        case.review_gap()
        capture(case, scenario + "-reviewed-recovered", at)
    finally:
        case.doCleanups()


if __name__ == "__main__":
    for scenario in ("unknown", "expired", "auth", "scopes"):
        observe(scenario)

"""Disposable observations of the current rendered notices, including blockers."""
import html
import json

import attention
import decision_feed as feed
import decision_manager as manager
import decisions as d
import slack_transport as slack
from test_decision_alerts import PIN
from test_decisions import NOW
from test_slack_transport import TransportTests, payload


def observe(label):
    case = TransportTests("test_unbound_idle_renewal_records_expiry_without_new_callbacks")
    case.setUp()
    try:
        at = NOW
        fresh = dict(state="healthy", event_flush_failures=0, pending_events=0,
                     observed_at=NOW, reason=None)
        if label == "legacy-unknown":
            case.quiet_legacy_unknown()
        elif label in ("waiting-for-route", "expired-undelivered"):
            case.unbind_fixture()
            case.callback()
            if label == "expired-undelivered":
                at = case.store.read("transport")["held_human"]["Ev001"]["expires_at"]
                case.owner.now = lambda: at
                case.owner.update("connected")
                fresh["observed_at"] = at
        else:
            case.sending()
            if label == "quarantined":
                case.callback(payload("EvPoison", subtype="message_deleted", deleted_ts="109.000001"))
        if label != "supervisor-unavailable":
            case.store.write("supervisor", dict(binding=PIN, health=fresh))
        if label == "supervisor-stale":
            at = (d.stamp(NOW) + slack.dt.timedelta(seconds=6)).strftime("%Y-%m-%dT%H:%M:%SZ")
        raw = d.load_fixture(case.feed_root, at)
        projected = feed.project_slack(case.feed_root, case.root, at, True)
        health = feed.slack_health(dict(slack=projected), at)
        status = manager.project_status(case.store, at, True)
        if label == "no-slack-observation":
            projected = None
            health = feed.slack_health({}, at)
            status = None
        rendered = attention.render_decisions(raw, at, [], {}, slack=projected, slack_health=health)
        notice = html.unescape(rendered.split("<p>", 1)[1].split("</p>", 1)[0])
        journal = case.store.read("transport")
        print(json.dumps(dict(scenario=label, rendered_notice=notice, feed_health=health,
                              status=status, outstanding=slack.outstanding_quarantine(journal),
                              held=len(journal.get("held_human", {})),
                              retained_expired=sum(row["reason"] == "unbound-expired"
                                  for row in journal.get("quarantine", {}).get("records", []))),
                         sort_keys=True))
    finally:
        case.doCleanups()


if __name__ == "__main__":
    for scenario in ("waiting-for-route", "quarantined", "expired-undelivered", "nothing-wrong",
                     "legacy-unknown", "supervisor-unavailable", "supervisor-stale",
                     "no-slack-observation"):
        observe(scenario)

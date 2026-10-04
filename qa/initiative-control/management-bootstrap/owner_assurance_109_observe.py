"""Capture actual notices from disposable journals; not functional acceptance."""
import html
import json
from unittest.mock import patch

import attention
import decision_alerts as alerts
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
        if label in ("held-waiting", "expired"):
            case.unbind_fixture()
            case.callback()
            if label == "expired":
                at = case.store.read("transport")["held_human"]["Ev001"]["expires_at"]
                case.owner.now = lambda: at
                case.owner.update("connected")
                fresh["observed_at"] = at
        else:
            case.sending()
            if label == "quarantined":
                case.callback(payload("EvPoison", subtype="message_deleted", deleted_ts="109.000001"))
        case.store.write("supervisor", dict(binding=PIN, health=fresh))
        if label == "supervisor-stale":
            at = (d.stamp(NOW) + slack.dt.timedelta(seconds=6)).strftime("%Y-%m-%dT%H:%M:%SZ")
        read = alerts.Store.read
        def checked_read(store, name):
            if label == "supervisor-unreadable" and name == "supervisor":
                raise OSError("fixture unreadable observation")
            return read(store, name)
        with patch.object(alerts.Store, "read", checked_read):
            projected = feed.project_slack(case.feed_root, case.root, at, True)
            status = manager.project_status(case.store, at, True)
        if label == "saved-snapshot-aged":
            at = (d.stamp(NOW) + slack.dt.timedelta(seconds=6)).strftime("%Y-%m-%dT%H:%M:%SZ")
        health = feed.slack_health(dict(slack=projected), at)
        rendered = attention.render_decisions(d.load_fixture(case.feed_root, at), at, [], {},
                                             slack=projected, slack_health=health)
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
    for scenario in ("healthy", "held-waiting", "quarantined", "expired",
                     "supervisor-stale", "supervisor-unreadable", "saved-snapshot-aged"):
        observe(scenario)

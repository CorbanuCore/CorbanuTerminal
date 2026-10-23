"""Disposable four-state projection/renderer observation; no live stores or Slack."""
import json

import attention
import decision_feed as feed
import decision_manager as manager
import decisions as d
import slack_transport as slack
from test_decision_alerts import PIN
from test_decisions import NOW
from test_slack_transport import TransportTests, payload


def observe(label, case, at):
    projected = feed.project_slack(case.feed_root, case.root, at, True)
    health = feed.slack_health(dict(slack=projected), at)
    raw = d.load_fixture(case.feed_root, at)
    html = attention.render_decisions(raw, at, [], {}, slack=projected, slack_health=health)
    notice = html.split("<p>", 1)[1].split("</p>", 1)[0]
    result = dict(scenario=label, status=manager.project_status(case.store, at, True),
                  feed_health=health, rendered_notice=notice)
    print(json.dumps(result, sort_keys=True))


def main():
    case = TransportTests("test_route_at_or_after_deadline_delivers_retained_reply_once")
    case.setUp()
    try:
        row = case.unbind_fixture()
        fresh = dict(state="healthy", event_flush_failures=0, pending_events=0,
                     observed_at=NOW, reason=None)
        case.store.write("supervisor", dict(binding=PIN, health=fresh))
        case.callback()
        observe("held", case, NOW)
        case.transport.bind_alert(case.key, row)
        assert slack.drain(case.store, now=NOW) == 1
        case.review_gap()
        case.callback(payload("EvPoison", subtype="message_deleted", deleted_ts="109.000001"))
        observe("quarantined", case, NOW)
        later = (d.stamp(NOW) + slack.dt.timedelta(seconds=6)).strftime("%Y-%m-%dT%H:%M:%SZ")
        observe("stale-with-quarantine", case, later)
        case.review_gap()
        observe("stale-without-quarantine", case, later)
        case.store.write("supervisor", dict(binding=PIN, health=dict(fresh, observed_at=later)))
        observe("fine", case, later)
    finally:
        case.doCleanups()


if __name__ == "__main__":
    main()

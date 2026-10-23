"""Verified reply data -> explicit manager interpretation -> CAS -> injected agent.

The verifier/receiver are trusted adapter dependencies, never deserialized from
Slack JSON. No default verifier, live endpoint, agent tools or executable text.
"""
import copy
from contextlib import contextmanager
import fcntl
import os

import decisions as d
import decision_alerts as a


def audit(ledger, alert_id):
    return d.digest({key: row["envelope"] for key, row in ledger["events"].items() if row["alert"] == alert_id})


def intake(store, alert_id, raw, verify):
    """verify(bytes) authenticates the envelope and returns normalized fields.

    It must verify signature/session, freshness and actual human authorship before
    this call persists anything. A JSON `verified: true` is not a verifier.
    Edits/deletes retain original author, message ts and mutation event ts.
    """
    d.require(type(raw) is bytes and 0 < len(raw) <= 16384)
    try:
        event = copy.deepcopy(verify(raw))
        d.shape(event, "event_id team channel user thread_ts message_ts event_ts kind text")
        for field in ("event_id", "team", "channel", "user"):
            a.token(event[field])
        for field in ("thread_ts", "message_ts", "event_ts"):
            a.slack_ts(event[field])
        d.require(event["kind"] in ("message", "edit", "delete"))
        if event["kind"] == "delete":
            d.require(event["text"] is None)
        else:
            d.text(event["text"], limit=4000)
        d.require(len(d.canonical(event)) <= 16384)
    except Exception:
        raise d.Invalid() from None
    with store.lock():
        row = a.alert(store.read("alerts"), alert_id)
        pinned = row["intent"]["identity"]
        d.require(row["details"]["state"] == "sent" and row["parent"]["state"] == "sent")
        d.require(not row["cancelled"])
        d.require([event[k] for k in ("team", "channel", "user")] == [pinned[k] for k in ("team", "channel", "human")])
        d.require(event["thread_ts"] == row["parent"]["receipt"]["ts"])
        d.require(event["message_ts"] not in (event["thread_ts"], row["details"]["receipt"]["ts"]))
        ledger = store.read("replies")
        event_id = event["event_id"]
        if event_id in ledger["aliases"]:
            existing = ledger["events"][ledger["aliases"][event_id]]
            d.require(existing["alert"] == alert_id)
            d.require({k: v for k, v in event.items() if k != "event_id"} == {k: v for k, v in existing["envelope"].items() if k != "event_id"})
            return copy.deepcopy(existing)
        same_thread = [r for r in ledger["events"].values() if r["alert"] == alert_id]
        for old in same_thread:
            previous = old["envelope"]
            if all(previous[k] == event[k] for k in ("message_ts", "event_ts", "kind")):
                d.require(all(previous[k] == event[k] for k in event if k != "event_id"))
                ledger["aliases"][event_id] = previous["event_id"]
                store.write("replies", ledger)
                return copy.deepcopy(old)
        state = "needs-clarification" if same_thread or event["kind"] != "message" else "received"
        for old in same_thread:
            old["state"] = "needs-clarification"
        result = dict(alert=alert_id, envelope=event, state=state)
        ledger["events"][event_id] = result
        ledger["aliases"][event_id] = event_id
        store.write("replies", ledger)
        return copy.deepcopy(result)


def snapshot(store, alert_id):
    with store.lock():
        ledger = store.read("replies")
        return dict(audit=audit(ledger, alert_id), replies=copy.deepcopy(ledger["events"]), intents=copy.deepcopy(ledger["intents"]))


def latest(feed, decision_id):
    d.require(feed is not None)
    rows = [item for item in feed["decisions"] if item["id"] == decision_id]
    d.require(len(rows) == 1)
    return rows[0]["revisions"]


@contextmanager
def feed_lock(root):
    """Share decisions.save_fixture's existing lock; never create/reset a feed."""
    root = d.fixture_root(root)
    fd = os.open(root / ".decisions.fixture.lock", os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "r+") as lock:
        d.owner_only(os.fstat(lock.fileno()))
        fcntl.flock(lock, fcntl.LOCK_EX)
        yield


def interpret(store, feed_root, event_id, manager, current_owner, now, checkpoint=lambda _: None):
    """manager binds answer/scope to alert, context, audit and allocation.

    interpretation='clarified-answer' explicitly addresses the complete retained
    audit; ordinary 'answer' cannot accept a conflicting/edited/deleted reply.
    The injected current_owner is the manager's current allocation observation.
    """
    d.shape(manager, "actor answer scope alert context_digest audit owner interpretation")
    for field in ("actor", "answer", "scope"):
        d.text(manager[field])
    d.require(manager["interpretation"] in ("answer", "clarified-answer", "clarify"))
    with store.lock(), feed_lock(feed_root):
        ledger = store.read("replies")
        event_id = ledger["aliases"][event_id]
        reply = ledger["events"][event_id]
        row = a.alert(store.read("alerts"), reply["alert"])
        intent = row["intent"]
        feed = d.load_fixture(feed_root, now)
        history = latest(feed, intent["decision_id"])
        eligible = (manager["alert"] == reply["alert"] and manager["context_digest"] == intent["context_digest"]
                    and manager["audit"] == audit(ledger, reply["alert"])
                    and a.owner(manager["owner"]) == a.owner(current_owner) == intent["owner"]
                    and current_owner["running"] and not row["cancelled"]
                    and feed["feed_id"] == intent["feed_id"] and history[-1] == intent["record"]
                    and history[-1]["status"] in ("open", "acknowledged")
                    and manager["interpretation"] != "clarify"
                    and (reply["state"] == "received" or manager["interpretation"] == "clarified-answer"))
        # A question revision can resolve once, across replies AND alert destinations.
        key = d.digest([intent["feed_id"], intent["decision_id"], intent["record"]["revision"]])
        retained = []
        if key in ledger["intents"]:
            previous = ledger["intents"][key]
            if previous["event_id"] == event_id and previous["manager"] == manager:
                return key
            # Under both locks the exact original base proves no resolution was applied.
            d.require(eligible and manager["interpretation"] == "clarified-answer"
                      and previous["alert"] == reply["alert"] and manager["audit"] != previous["manager"]["audit"]
                      and previous["state"] == "recorded" and previous["delivery"] == "pending"
                      and all(previous[k] is None for k in ("request", "receipt", "ack"))
                      and d.digest(feed) == previous["base_digest"])
            retained = previous.get("interpretations", []) + [{k: v for k, v in previous.items() if k != "interpretations"}]
        if not eligible:
            reply["state"] = "needs-clarification"
            store.write("replies", ledger)
            return None
        target = copy.deepcopy(feed)
        target.update(revision=feed["revision"] + 1, assessed_at=now)
        record = copy.deepcopy(history[-1])
        record.update(revision=len(history) + 1, updated_at=now, status="resolved",
                      resolution=dict(answered_revision=len(history), actor=manager["actor"],
                                      answer=manager["answer"], scope=manager["scope"], recorded_at=now))
        latest(target, intent["decision_id"]).append(record)
        target = d.validate(target, now)
        immutable = dict(event_id=event_id, alert=reply["alert"], manager=copy.deepcopy(manager),
                         base_digest=d.digest(feed), target=target)
        ledger["intents"][key] = dict(**immutable, id=key, interpretations=retained, state="recorded", delivery="pending", request=None, receipt=None, ack=None)
        reply["state"] = "recorded"
        store.write("replies", ledger)
        checkpoint("intent")
        return key


def resolution_present(feed, intent):
    target = intent["target"]
    if feed is None or feed["feed_id"] != target["feed_id"] or feed["revision"] < target["revision"]:
        return False
    actual = {item["id"]: item["revisions"] for item in feed["decisions"]}
    return all(actual.get(item["id"], [])[:len(item["revisions"])] == item["revisions"] for item in target["decisions"])


def resume(store, feed_root, key, current_owner, now, checkpoint=lambda _: None):
    """Journal before CAS; compare actual feed after any save uncertainty.

    Feed and handoff ledger are separate files, NOT an atomic transaction.
    Retained target allows restart after saved resolution, before queue write.
    """
    with store.lock():
        ledger = store.read("replies")
        intent = ledger["intents"][key]
        if intent["state"] != "recorded":
            return intent["state"]
        feed = d.load_fixture(feed_root, now)
        row = a.alert(store.read("alerts"), intent["alert"])
        if not resolution_present(feed, intent) and (row["cancelled"] or not current_owner["running"]
                or a.owner(current_owner) != row["intent"]["owner"]
                or intent["manager"]["audit"] != audit(ledger, intent["alert"])):
            ledger["events"][intent["event_id"]]["state"] = "needs-clarification"
            store.write("replies", ledger)
            return "needs-clarification"
        if not resolution_present(feed, intent) and feed is not None and d.digest(feed) == intent["base_digest"]:
            try:
                d.save_fixture(feed_root, intent["target"], intent["base_digest"], now)
            except (d.Invalid, d.Conflict):
                pass  # Includes post-replace uncertainty; exact reload decides.
            feed = d.load_fixture(feed_root, now)
        if not resolution_present(feed, intent):
            ledger["events"][intent["event_id"]]["state"] = "needs-clarification"
            store.write("replies", ledger)
            return "needs-clarification"
        checkpoint("resolved")
        intent["state"] = "queued"
        ledger["events"][intent["event_id"]]["state"] = "queued"
        store.write("replies", ledger)
        checkpoint("queued")
        return "queued"


def handoff_receipt(request, evidence):
    d.shape(evidence, "handoff owner payload_digest receipt_id")
    d.require(all(evidence[k] == request[k] for k in ("handoff", "owner", "payload_digest")))
    a.token(evidence["receipt_id"])
    return copy.deepcopy(evidence)


def dispatch(store, feed_root, key, current_owner, receive, now):
    """Trusted injected receive(request), not a Codex call; callers must bound I/O.

    Store -> feed locks fence context through durable receipt. The adapter must
    not reenter this store or write the held feed while its callback executes.
    """
    with store.lock(), feed_lock(feed_root):
        ledger = store.read("replies")
        intent = ledger["intents"][key]
        if intent["delivery"] == "sending":
            intent["delivery"] = "uncertain"
        row = a.alert(store.read("alerts"), intent["alert"])
        feed = d.load_fixture(feed_root, now)
        expected_record = latest(intent["target"], row["intent"]["decision_id"])[-1]
        eligible = (intent["state"] == "queued" and intent["delivery"] == "pending"
                    and a.owner(current_owner) == row["intent"]["owner"] and current_owner["running"]
                    and not row["cancelled"] and intent["manager"]["audit"] == audit(ledger, intent["alert"])
                    and resolution_present(feed, intent) and latest(feed, row["intent"]["decision_id"])[-1] == expected_record)
        if eligible:
            payload = dict(question=row["intent"]["record"], answer=intent["manager"]["answer"], scope=intent["manager"]["scope"],
                           context_digest=row["intent"]["context_digest"], reply_event=intent["event_id"])
            intent["request"] = dict(handoff=key, owner=copy.deepcopy(current_owner), payload=payload, payload_digest=d.digest(payload))
            intent["delivery"] = "sending"
            store.write("replies", ledger)
            try:
                intent["receipt"] = handoff_receipt(intent["request"], receive(copy.deepcopy(intent["request"])))
                intent.update(state="delivered", delivery="sent")
            except Exception:
                intent["delivery"] = "uncertain"
        if eligible or intent["state"] in ("delivered", "agent-acknowledged"):
            ledger["events"][intent["event_id"]]["state"] = intent["state"]
        else:
            ledger["events"][intent["event_id"]]["state"] = "needs-clarification"
        store.write("replies", ledger)
        return copy.deepcopy(intent)


def reconcile_handoff(store, key, evidence, acknowledgment=None):
    """Authenticated receipt establishes delivery; only exact agent ack establishes acceptance."""
    with store.lock():
        ledger = store.read("replies")
        intent = ledger["intents"][key]
        d.require(intent["delivery"] in ("sending", "uncertain", "sent"))
        receipt = handoff_receipt(intent["request"], evidence)
        d.require(intent["receipt"] in (None, receipt))
        if acknowledgment is not None:
            d.shape(acknowledgment, "handoff agent allocation payload_digest receipt_id")
            expected = dict(handoff=key, agent=intent["request"]["owner"]["agent"], allocation=intent["request"]["owner"]["allocation"],
                            payload_digest=intent["request"]["payload_digest"], receipt_id=receipt["receipt_id"])
            d.require(acknowledgment == expected)
            intent["ack"] = copy.deepcopy(acknowledgment)
        intent.update(state="agent-acknowledged" if intent["ack"] else "delivered", delivery="sent", receipt=receipt)
        ledger["events"][intent["event_id"]]["state"] = intent["state"]
        store.write("replies", ledger)

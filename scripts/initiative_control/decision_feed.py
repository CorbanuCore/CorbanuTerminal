"""Fixed, bounded decision snapshot transport; no state authoring or delivery."""
import datetime as dt
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import stat

import decisions as d
import decision_inspection as inspection

ARTIFACT = "decision-feed.json"
SLACK_FILE = "decision-slack-status.json"
# Every feed revision includes these JSON keys/separators, even before values.
# The 1 MiB admitted feed therefore bounds the number of possible retained rows.
MIN_RECORD_BYTES = 1 + sum(len(d.canonical(key)) + 2 for key in d.FIELDS)
MAX_SLACK_ROWS = d.MAX_BYTES // MIN_RECORD_BYTES
SLACK_ROW_LIMIT = 1024  # ID <=80, digest=64, bounded revision/counters; ample headroom.
SLACK_ENVELOPE_LIMIT = 16384
SLACK_LIMIT = SLACK_ENVELOPE_LIMIT + MAX_SLACK_ROWS * (SLACK_ROW_LIMIT + 1)
LIMIT = d.MAX_BYTES + SLACK_LIMIT + 256
DELIVERY = {"off", "unknown", "not-requested", "pending", "sending", "uncertain", "failed", "sent", "cancelled"}


def validate_slack(value, feed, at):
    """A projection binds only existing question revisions, never grants authority."""
    import decision_manager as manager
    d.require(len(d.canonical(value)) <= SLACK_LIMIT and not d.SECRET.search(d.canonical(value).decode()))
    # Match atomic_json's physical encoding as well as the canonical transport.
    d.require(len((json.dumps(value, indent=2, sort_keys=True) + "\n").encode()) <= SLACK_LIMIT)
    d.shape(value, "schema assessed_at feed_digest status decisions"
            + (" omitted_revisions" if "omitted_revisions" in value else ""))
    d.require(len(d.canonical({**value, "decisions": []})) <= SLACK_ENVELOPE_LIMIT)
    d.require(type(value["schema"]) is int and value["schema"] == 1 and feed is not None)
    d.require(d.stamp(value["assessed_at"]) <= d.stamp(clock(at)) and value["feed_digest"] == d.digest(feed))
    manager.validate_status(value["status"])
    d.require(value["status"]["enabled"] == (value["status"]["state"] != "off"))
    if value["status"]["last_verified"] is not None:
        d.require(d.stamp(value["status"]["last_verified"]) <= d.stamp(value["assessed_at"]))
    d.require(type(value["decisions"]) is list and len(value["decisions"]) <= MAX_SLACK_ROWS)
    records = {(item["id"], record["revision"]): record for item in feed["decisions"] for record in item["revisions"]}
    seen = set()
    for row in value["decisions"]:
        d.require(len(d.canonical(row)) <= SLACK_ROW_LIMIT)
        d.shape(row, "id revision context_digest delivery pending replies")
        d.require(type(row["id"]) is str and re.fullmatch(d.ID, row["id"]) and type(row["revision"]) is int)
        key = (row["id"], row["revision"])
        d.require(key in records and key not in seen and row["context_digest"] == d.digest(records[key]))
        seen.add(key)
        d.require(row["delivery"] in DELIVERY)
        d.require(type(row["pending"]) is int and 0 <= row["pending"] <= 100)
        extra = [key for key in ("unacknowledged_answers", "new_thread_fallbacks") if key in row["replies"]]
        d.shape(row["replies"], manager.COUNTS + extra)
        d.require(all(type(n) is int and 0 <= n <= 1000000 for n in row["replies"].values()))
        if not value["status"]["enabled"]:
            d.require(row["delivery"] == "off" and row["pending"] == 0 and not any(row["replies"].values()))
    latest = {(item["id"], item["revisions"][-1]["revision"]) for item in feed["decisions"]}
    d.require(latest <= seen)
    if "omitted_revisions" in value:
        d.require(type(value["omitted_revisions"]) is int
                  and value["omitted_revisions"] == len(records) - len(seen))
    return copy.deepcopy(value)


def project_slack(state, store_path, at, enabled=False):
    """Explicit owner operation: one locked local snapshot, no SDK/network calls."""
    import decision_manager as manager
    import decision_alerts as alerts
    import decision_replies as replies
    import slack_transport as slack
    from control import atomic_json
    state, at = d.fixture_root(state), clock(at)
    status = manager.project_status(None, at, False)
    rows = []

    def collect(feed, saved, ledger, binding, ingress):
        events = ledger["events"]
        for item in feed["decisions"]:
            answered = {record["resolution"]["answered_revision"] for record in item["revisions"]
                        if record["resolution"] is not None}
            for record in item["revisions"]:
                candidates = [(key, alerts.alert(saved, key)) for key in saved
                              if saved[key]["intent"]["feed_id"] == feed["feed_id"]
                              and saved[key]["intent"]["decision_id"] == item["id"]
                              and saved[key]["intent"]["record"] == record]
                matching = [(key, row) for key, row in candidates if row["intent"]["identity"] == binding]
                d.require(len(matching) <= 1)
                delivery = "off" if not enabled else "unknown" if binding is None else "not-requested"
                counts = {key: 0 for key in manager.COUNTS}
                counts["unacknowledged_answers"] = 0
                counts["new_thread_fallbacks"] = 0
                pending = 0
                if matching:
                    key, row = matching[0]
                    counts["unacknowledged_answers"] = manager.unacknowledged_answers(ledger, saved, key)
                    counts["new_thread_fallbacks"] = int(row.get("threading", {}).get("mode") == "new-thread")
                    pending = sum(event["alert"] == key and not event["drained"] for event in ingress.values())
                    delivery = "cancelled" if row["cancelled"] else row["parent"]["state"] if row["parent"]["state"] != "sent" else row["details"]["state"]
                    # A retained sending reservation is not proof of nonacceptance.
                    if delivery == "sending":
                        delivery = "uncertain"
                    for event in events.values():
                        if event["alert"] == key:
                            name = event["state"].replace("-", "_")
                            d.require(name in counts)
                            counts[name] += 1
                # A superseded question may still await an answer, delivery,
                # receiver ACK, or its Slack notice. Be conservative: cancellation
                # alone cannot hide queued ingress or unfinished answer handoff.
                # A different/missing binding cannot prove an older alert is
                # resolved. Retain its row as unknown without attributing replies
                # or delivery from that other identity to the current binding.
                unresolved = len(candidates) != len(matching)
                if matching:
                    intents = [intent for intent in ledger["intents"].values() if intent["alert"] == key]
                    unresolved = (unresolved or bool(pending) or bool(counts["unacknowledged_answers"])
                                  or any(counts[name] for name in manager.COUNTS if name != "agent_acknowledged")
                                  or any(notice["state"] != "sent" for notice in row.get("notices", {}).values())
                                  or (not row["cancelled"] and (not intents or delivery != "sent")))
                if record is item["revisions"][-1] or record["revision"] in answered or unresolved:
                    rows.append(dict(id=item["id"], revision=record["revision"], context_digest=d.digest(record), delivery=delivery, pending=pending, replies=counts))

    if enabled:
        store = alerts.Store(store_path, lock_timeout=5)
        with store.lock(), replies.feed_lock(state), slack.locked(store) as journal:
            feed = d.load_fixture(state, at)
            d.require(feed is not None)
            saved, ledger = store.read("alerts"), store.read("replies")
            events = ledger["events"]
            status["unacknowledged_answers"] = manager.unacknowledged_answers(ledger, saved)
            status["new_thread_fallbacks"] = sum(row.get("threading", {}).get("mode") == "new-thread"
                                                 for row in saved.values())
            status.update(enabled=True, state="held" if journal["hold"] else "last-verified",
                          last_verified=journal["last_verified"], watermark=journal["watermark"],
                          pending=sum(not event["drained"] for event in journal["events"].values()))
            if journal["binding"] is None:
                status["state"] = "unqualified"
            elif status["last_verified"] is None or not 0 <= (d.stamp(at) - d.stamp(status["last_verified"])).total_seconds() <= 900:
                status["state"] = "stale"
            try:
                manager.project_disclosure(status, store, journal, saved, at)
                slack.fenced(store, journal)
                slack.observe_session_locked(store, journal)
            except (OSError, ValueError):
                status["state"] = "held"
            # Mirror decision_manager.project_status: an owner-reconciled
            # never-sent post is resolved and must not wedge the published
            # status at held forever.
            if any(post["receipt"] is None and not slack.reconciled_never_sent(post)
                   for post in journal["posts"].values()):
                status["state"] = "held"
            if status.get("supervisor_health", {}).get("state") == "unhealthy":
                status["state"] = "held"
            for event in events.values():
                name = event["state"].replace("-", "_")
                d.require(name in manager.COUNTS)
                status[name] += 1
            collect(feed, saved, ledger, journal["binding"], journal["events"])
    else:
        with replies.feed_lock(state):
            feed = d.load_fixture(state, at)
            d.require(feed is not None)
            collect(feed, {}, {"events": {}, "intents": {}}, None, {})
    omitted = sum(len(item["revisions"]) for item in feed["decisions"]) - len(rows)
    result = validate_slack(dict(schema=1, assessed_at=at, feed_digest=d.digest(feed), status=status,
                                 decisions=rows, omitted_revisions=omitted), feed, at)
    # Cache only. A later feed change invalidates the pin rather than being merged.
    atomic_json(state / SLACK_FILE, result)
    return result


def read_slack(state, feed, at):
    observed = []
    try:
        root = d.fixture_root(state)
        path = root / SLACK_FILE
        try:
            info = path.lstat()
        except FileNotFoundError:
            return "absent", None, [None]
        observed.append([info.st_dev, info.st_ino, info.st_mode, info.st_uid, info.st_nlink, info.st_size, info.st_mtime_ns, info.st_ctime_ns])
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as stream:
            d.owner_only(os.fstat(stream.fileno()))
            raw = stream.read(SLACK_LIMIT + 1)
        observed.append(sha(raw))
        d.require(len(raw) <= SLACK_LIMIT)
        value = json.loads(raw, object_pairs_hook=d.pairs)
        return "valid", validate_slack(value, feed, at), observed
    except (OSError, ValueError, TypeError, KeyError, RuntimeError, RecursionError):
        return "invalid", None, observed


def clock(value):
    return dt.datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


OWNER_PUBLICATION_SECONDS = 30 * 60
OWNER_FRESH_SECONDS = 2 * OWNER_PUBLICATION_SECONDS


def owner_health(value, at):
    """Coarse published observation, never a claim of current process liveness."""
    unknown = dict(state="unknown", reason="observation-unavailable", age=None)
    try:
        value = dict(firing="unknown", publication_errors=0, last_publication_error=None) | value
        d.shape(value, "observed_at service installed started_at completed_at last_success hold reason "
                "previous_success interval errors consecutive_errors last_error firing "
                "publication_errors last_publication_error")
        d.require(value["firing"] in {"interval", "manual", "other", "unknown"})
        d.require(type(value["interval"]) is int and 1 <= value["interval"] <= 30)
        d.require(all(type(value[k]) is int and value[k] >= 0 for k in
                      ("errors", "consecutive_errors", "publication_errors")))
        for key in ("last_error", "last_publication_error"):
            d.require(value[key] is None or isinstance(value[key], str)
                      and value[key].isidentifier() and len(value[key]) <= 80)
        d.require(value["service"] in {"present", "absent", "unknown"} and type(value["installed"]) is bool)
        d.require(value["reason"] in {None, "observation-unavailable"}
                  and value["hold"] in {None, "owner_run_refused", "interrupted_tick"})
        for key in ("observed_at", "started_at", "completed_at", "last_success", "previous_success"):
            d.require(value[key] is None and key != "observed_at" or
                      type(value[key]) in (int, float) and 0 <= value[key] < 1e11)
        now = d.stamp(clock(at)).timestamp()
        if value["reason"] or value["service"] == "unknown":
            return unknown
        if not 0 <= now - int(value["observed_at"]) <= OWNER_FRESH_SECONDS:
            return dict(unknown, reason="observation-stale")
        publication = {key: value[key] for key in ("publication_errors", "last_publication_error")}
        if not value["installed"]:
            return dict(state="never-installed", reason=value["hold"] or "verified-service-absence",
                        age=None, **publication) if value["service"] == "absent" else dict(unknown, reason="installation-unrecorded")
        completion, start = value["completed_at"], value["started_at"]
        age = value["observed_at"] - (completion if completion is not None else start) if completion is not None or start is not None else None
        previous, success, interval = value["previous_success"], value["last_success"], value["interval"]
        recurring = (value["firing"] == "interval" and previous is not None and success is not None and
                     interval * 0.5 <= success - previous <= interval * 1.5 and
                     0 <= value["observed_at"] - success <= interval * 3)
        reason = value["hold"] or ("service-absent" if value["service"] == "absent" else
                                  value["last_error"] if value["consecutive_errors"] else
                                  "tick-overdue" if age is None or not 0 <= age <= interval * 3 else
                                  "recurrence-unproven" if not recurring else None)
        return dict(state="stalled" if reason else "recurring-at-observation", reason=reason, age=age,
                    errors=value["errors"], consecutive_errors=value["consecutive_errors"],
                    last_completion=completion, last_success=success, **publication)
    except (ValueError, TypeError, KeyError):
        return unknown


def capture(state, at):
    """Only the fixed private fixture is input; errors never export its contents."""
    feed, status, observed, partial = None, "invalid", [], None
    path = Path(state) / "decisions.fixture.json"
    try:
        for entry in (Path(state), path):
            try:
                info = entry.lstat()
                observed.append([info.st_dev, info.st_ino, info.st_mode, info.st_uid] +
                                ([] if entry == Path(state) else [info.st_nlink, info.st_size, info.st_mtime_ns, info.st_ctime_ns]))
            except FileNotFoundError:
                observed.append(None)
        root = d.fixture_root(state)
        try:
            fd = os.open(root / path.name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        except FileNotFoundError:
            status = "missing"
        else:
            with os.fdopen(fd, "rb") as stream:
                d.owner_only(os.fstat(stream.fileno()))
                raw = stream.read(d.MAX_BYTES + 1)
            observed.append(sha(raw))
            try:
                feed = d.validate(raw, clock(at))
                status = "valid"
            except d.Invalid:
                partial = inspection.extract(raw, clock(at))
    except (OSError, ValueError, RuntimeError):
        pass
    snapshot = {"schema": 1, "status": status, "feed": feed}
    slack_status, slack_value, slack_observed = read_slack(state, feed, at)
    observed.append(slack_observed)
    if partial is not None:
        snapshot.update(schema=3, inspection=partial[0], withheld_count=partial[1])
    elif slack_status != "absent":
        snapshot.update(schema=2, slack_status=slack_status, slack=slack_value)
    try:
        import owner_daemon as owner
        recurrence = owner.load(d.fixture_root(state) / "owner-recurrence.json")
        if owner_health(recurrence, at)["reason"] != "observation-unavailable":
            snapshot["owner_recurrence"] = recurrence
        else:
            snapshot["owner_recurrence"] = None
    except Exception:
        pass
    observed.append(snapshot.get("owner_recurrence"))
    raw = d.canonical(snapshot)
    return raw, record(snapshot, sha(raw), d.digest(observed))


def record(snapshot, digest, input_digest):
    feed = snapshot["feed"] or {}
    return dict(schema=snapshot["schema"], artifact=ARTIFACT, digest=digest, input_digest=input_digest,
                payload_digest=d.digest(feed) if feed else None,
                status=snapshot["status"], feed_id=feed.get("feed_id"),
                revision=feed.get("revision"), assessed_at=feed.get("assessed_at"))


def verify_input(state, manifest, at):
    if "decision_feed" in manifest and capture(state, at)[1] != manifest["decision_feed"]:
        raise ValueError("Decision input changed during synchronization; retry.")


def read_snapshot(repo, manifest, at):
    """Missing legacy metadata is unknown; a broken pinned transfer is a failure."""
    if "decision_feed" not in manifest:
        return {"schema": 1, "status": "unrecorded", "feed": None}
    try:
        pin = manifest["decision_feed"]
        d.shape(pin, "schema artifact digest input_digest payload_digest status feed_id revision assessed_at")
        d.require(type(pin["schema"]) is int and pin["schema"] in (1, 2, 3) and pin["artifact"] == ARTIFACT)
        d.require(all(type(pin[k]) is str and re.fullmatch(r"[a-f0-9]{64}", pin[k]) for k in ("digest", "input_digest")))
        # Resolve only the source root (the installed generation is a symlink).
        fd = os.open(Path(repo).resolve(strict=True) / ARTIFACT, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as stream:
            d.require(stat.S_ISREG(os.fstat(stream.fileno()).st_mode))
            raw = stream.read(LIMIT + 1)
        d.require(len(raw) <= LIMIT and sha(raw) == pin["digest"])
        snapshot = json.loads(raw, object_pairs_hook=d.pairs)
        d.require(type(snapshot["schema"]) is int and snapshot["schema"] == pin["schema"])
        extension = {1: "", 2: " slack_status slack", 3: " inspection withheld_count"}
        d.shape(snapshot, "schema status feed" + extension[snapshot["schema"]] +
                (" owner_recurrence" if "owner_recurrence" in snapshot else ""))
        d.require(snapshot["status"] in ("valid", "missing", "invalid"))
        if snapshot["schema"] == 3:
            d.require(snapshot["status"] == "invalid" and snapshot["feed"] is None)
            snapshot["inspection"] = inspection.validate(snapshot["inspection"], snapshot["withheld_count"], clock(at))
        if snapshot["status"] == "valid":
            snapshot["feed"] = d.validate(snapshot["feed"], clock(at))
        else:
            d.require(snapshot["feed"] is None)
        if snapshot["schema"] == 2:
            d.require(snapshot["slack_status"] in ("valid", "invalid"))
            if snapshot["slack_status"] == "valid":
                snapshot["slack"] = validate_slack(snapshot["slack"], snapshot["feed"], at)
            else:
                d.require(snapshot["slack"] is None)
        d.require(raw == d.canonical(snapshot))
        d.require(d.canonical(record(snapshot, sha(raw), pin["input_digest"])) == d.canonical(pin))
        return snapshot
    except (OSError, ValueError, TypeError, KeyError, RuntimeError, RecursionError):
        raise ValueError("Decision feed transfer unavailable or invalid; resync required.") from None


def health(snapshot, manifest, at):
    view = d.project(snapshot["feed"], clock(at))
    feed = view["feed"] or {}
    return dict(slack=slack_health(snapshot, at), owner_recurrence=owner_health(snapshot.get("owner_recurrence"), at),
                state=view["state"], input_status=snapshot["status"],
                feed_id=feed.get("feed_id"), revision=feed.get("revision"),
                assessed_at=feed.get("assessed_at"),
                digest=manifest.get("decision_feed", {}).get("digest"),
                payload_digest=d.digest(feed) if feed else None,
                open_count=None if view["open"] is None else len(view["open"]),
                fresh_seconds=d.FRESH_SECONDS)


def slack_health(snapshot, at):
    if snapshot.get("schema") == 3:
        return dict(state="unknown", assessed_at=None, last_verified=None)
    value = snapshot.get("slack")
    if value is None:
        return dict(state="unknown" if snapshot.get("slack_status") == "invalid" else "unrecorded", assessed_at=None, last_verified=None)
    from decision_manager import assess_supervisor_health
    status = value["status"]
    supervisor_health = (assess_supervisor_health(status["supervisor_health"], clock(at))
                         if "supervisor_health" in status else None)
    age = (d.stamp(clock(at)) - d.stamp(value["assessed_at"])).total_seconds()
    stale = not 0 <= age <= 900 or (status["last_verified"] is not None and (d.stamp(clock(at)) - d.stamp(status["last_verified"])).total_seconds() > 900)
    return dict(state="held" if status.get("fence_gap", 0) or status.get("supervisor_health", {}).get("state") == "unhealthy" else "stale" if stale else status["state"],
                supervisor_health=supervisor_health,
                assessed_at=value["assessed_at"], last_verified=status["last_verified"],
                fence_gap=status.get("fence_gap", 0), pending_pointers=status.get("pending_pointers", 0),
                listener_exits=status.get("listener_exits", 0),
                listener_events_pruned=status.get("listener_events_pruned", 0),
                last_listener_exit=copy.deepcopy(status.get("last_listener_exit")))


def render(snapshot, at, sprints, documents):
    from attention import render_decisions
    feed = snapshot["feed"] or snapshot.get("inspection")
    assessed = feed["assessed_at"] if feed else ""
    body = render_decisions(snapshot["feed"], clock(at), sprints, documents, slack=snapshot.get("slack"), slack_health=slack_health(snapshot, at),
                            inspection=snapshot.get("inspection"), withheld_count=snapshot.get("withheld_count"))
    from html import escape
    omitted = (snapshot.get("slack") or {}).get("omitted_revisions", 0)
    if omitted:
        body += (f"<p>Slack projection omits {omitted} older revision(s). "
                 "Every latest decision revision and every answered question revision is included in new projections; "
                 "full decision history remains above. "
                 "Older unresolved alerts are retained when Slack projection is enabled.</p>")
    recurrence = owner_health(snapshot.get("owner_recurrence"), at)
    def timestamp(value):
        return dt.datetime.fromtimestamp(value, dt.timezone.utc).isoformat() if value is not None else "unknown"
    observed = ((snapshot.get("owner_recurrence") or {}).get("observed_at")
                if recurrence["reason"] != "observation-unavailable" else None)
    body += ('<section id="owner-recurrence"><h2>Owner recurrence</h2><p>' +
             escape(f'Snapshot: {recurrence["state"]}; reason: {recurrence["reason"] or "fresh service and tick"}; '
                    f'tick age (seconds): {recurrence["age"]}; last completion: {timestamp(recurrence.get("last_completion"))}; '
                    f'last success: {timestamp(recurrence.get("last_success"))}. '
                    f'Errors: {recurrence.get("errors", 0)}; consecutive: {recurrence.get("consecutive_errors", 0)}. '
                    f'Publication failures: {recurrence.get("publication_errors", 0)}; '
                    f'last publication error: {recurrence.get("last_publication_error") or "none"}. '
                    f'Observed: {timestamp(observed)}; coarse 30-minute publication, not live status; expires '
                    f'{timestamp(observed + OWNER_FRESH_SECONDS if observed is not None else None)}.') + '</p></section>')
    return body.replace('<section id="decisions">',
                        f'<section id="decisions" class="notice attention" data-assessed-at="{assessed}" data-fresh-seconds="{d.FRESH_SECONDS}">', 1).replace('<details', '<details class="attention-item"')

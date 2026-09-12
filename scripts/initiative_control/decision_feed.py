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

ARTIFACT = "decision-feed.json"
SLACK_FILE = "decision-slack-status.json"
SLACK_LIMIT = 16384
LIMIT = d.MAX_BYTES + SLACK_LIMIT + 256
DELIVERY = {"off", "unknown", "not-requested", "pending", "sending", "uncertain", "failed", "sent", "cancelled"}


def validate_slack(value, feed, at):
    """A projection binds only existing question revisions, never grants authority."""
    import decision_manager as manager
    d.require(len(d.canonical(value)) <= SLACK_LIMIT and not d.SECRET.search(d.canonical(value).decode()))
    d.shape(value, "schema assessed_at feed_digest status decisions")
    d.require(type(value["schema"]) is int and value["schema"] == 1 and feed is not None)
    d.require(d.stamp(value["assessed_at"]) <= d.stamp(clock(at)) and value["feed_digest"] == d.digest(feed))
    manager.validate_status(value["status"])
    d.require(value["status"]["enabled"] == (value["status"]["state"] != "off"))
    if value["status"]["last_verified"] is not None:
        d.require(d.stamp(value["status"]["last_verified"]) <= d.stamp(value["assessed_at"]))
    d.require(type(value["decisions"]) is list and len(value["decisions"]) <= 100)
    records = {(item["id"], record["revision"]): record for item in feed["decisions"] for record in item["revisions"]}
    seen = set()
    for row in value["decisions"]:
        d.shape(row, "id revision context_digest delivery pending replies")
        d.require(type(row["id"]) is str and re.fullmatch(d.ID, row["id"]) and type(row["revision"]) is int)
        key = (row["id"], row["revision"])
        d.require(key in records and key not in seen and row["context_digest"] == d.digest(records[key]))
        seen.add(key)
        d.require(row["delivery"] in DELIVERY)
        d.require(type(row["pending"]) is int and 0 <= row["pending"] <= 100)
        d.shape(row["replies"], manager.COUNTS)
        d.require(all(type(n) is int and 0 <= n <= 1000000 for n in row["replies"].values()))
        if not value["status"]["enabled"]:
            d.require(row["delivery"] == "off" and row["pending"] == 0 and not any(row["replies"].values()))
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

    def collect(feed, saved, events, binding, ingress):
        for item in feed["decisions"]:
            for record in item["revisions"]:
                matching = [(key, alerts.alert(saved, key)) for key in saved
                            if saved[key]["intent"]["feed_id"] == feed["feed_id"]
                            and saved[key]["intent"]["decision_id"] == item["id"]
                            and saved[key]["intent"]["record"] == record
                            and saved[key]["intent"]["identity"] == binding]
                d.require(len(matching) <= 1)
                delivery = "off" if not enabled else "unknown" if binding is None else "not-requested"
                counts = {key: 0 for key in manager.COUNTS}
                pending = 0
                if matching:
                    key, row = matching[0]
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
                rows.append(dict(id=item["id"], revision=record["revision"], context_digest=d.digest(record), delivery=delivery, pending=pending, replies=counts))

    if enabled:
        store = alerts.Store(store_path, lock_timeout=5)
        with store.lock(), replies.feed_lock(state), slack.locked(store) as journal:
            feed = d.load_fixture(state, at)
            d.require(feed is not None)
            saved, events = store.read("alerts"), store.read("replies")["events"]
            status.update(enabled=True, state="held" if journal["hold"] else "last-verified",
                          last_verified=journal["last_verified"], watermark=journal["watermark"],
                          pending=sum(not event["drained"] for event in journal["events"].values()))
            if journal["binding"] is None:
                status["state"] = "unqualified"
            elif status["last_verified"] is None or not 0 <= (d.stamp(at) - d.stamp(status["last_verified"])).total_seconds() <= 900:
                status["state"] = "stale"
            try:
                slack.fenced(store, journal)
                slack.observe_session_locked(store, journal)
            except (OSError, ValueError):
                status["state"] = "held"
            if any(post["receipt"] is None for post in journal["posts"].values()):
                status["state"] = "held"
            for event in events.values():
                name = event["state"].replace("-", "_")
                d.require(name in manager.COUNTS)
                status[name] += 1
            collect(feed, saved, events, journal["binding"], journal["events"])
    else:
        with replies.feed_lock(state):
            feed = d.load_fixture(state, at)
            d.require(feed is not None)
            collect(feed, {}, {}, None, {})
    result = validate_slack(dict(schema=1, assessed_at=at, feed_digest=d.digest(feed), status=status, decisions=rows), feed, at)
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


def capture(state, at):
    """Only the fixed private fixture is input; errors never export its contents."""
    feed, status, observed = None, "invalid", []
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
            feed = d.validate(raw, clock(at))
            status = "valid"
    except (OSError, ValueError, RuntimeError):
        pass
    snapshot = {"schema": 1, "status": status, "feed": feed}
    slack_status, slack_value, slack_observed = read_slack(state, feed, at)
    observed.append(slack_observed)
    if slack_status != "absent":
        snapshot.update(schema=2, slack_status=slack_status, slack=slack_value)
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
        d.require(type(pin["schema"]) is int and pin["schema"] in (1, 2) and pin["artifact"] == ARTIFACT)
        d.require(all(type(pin[k]) is str and re.fullmatch(r"[a-f0-9]{64}", pin[k]) for k in ("digest", "input_digest")))
        # Resolve only the source root (the installed generation is a symlink).
        fd = os.open(Path(repo).resolve(strict=True) / ARTIFACT, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as stream:
            d.require(stat.S_ISREG(os.fstat(stream.fileno()).st_mode))
            raw = stream.read(LIMIT + 1)
        d.require(len(raw) <= LIMIT and sha(raw) == pin["digest"])
        snapshot = json.loads(raw, object_pairs_hook=d.pairs)
        d.require(type(snapshot["schema"]) is int and snapshot["schema"] == pin["schema"])
        d.shape(snapshot, "schema status feed" + (" slack_status slack" if snapshot["schema"] == 2 else ""))
        d.require(snapshot["status"] in ("valid", "missing", "invalid"))
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
    return dict(slack=slack_health(snapshot, at), state=view["state"], input_status=snapshot["status"],
                feed_id=feed.get("feed_id"), revision=feed.get("revision"),
                assessed_at=feed.get("assessed_at"),
                digest=manifest.get("decision_feed", {}).get("digest"),
                payload_digest=d.digest(feed) if feed else None,
                open_count=None if view["open"] is None else len(view["open"]),
                fresh_seconds=d.FRESH_SECONDS)


def slack_health(snapshot, at):
    value = snapshot.get("slack")
    if value is None:
        return dict(state="unknown" if snapshot.get("slack_status") == "invalid" else "unrecorded", assessed_at=None, last_verified=None)
    status = value["status"]
    age = (d.stamp(clock(at)) - d.stamp(value["assessed_at"])).total_seconds()
    stale = not 0 <= age <= 900 or (status["last_verified"] is not None and (d.stamp(clock(at)) - d.stamp(status["last_verified"])).total_seconds() > 900)
    return dict(state="stale" if stale else status["state"], assessed_at=value["assessed_at"], last_verified=status["last_verified"])


def render(snapshot, at, sprints, documents):
    from attention import render_decisions
    feed = snapshot["feed"]
    assessed = feed["assessed_at"] if feed else ""
    body = render_decisions(feed, clock(at), sprints, documents, slack=snapshot.get("slack"), slack_health=slack_health(snapshot, at))
    return body.replace('<section id="decisions">',
                        f'<section id="decisions" class="notice attention" data-assessed-at="{assessed}" data-fresh-seconds="{d.FRESH_SECONDS}">', 1).replace('<details', '<details class="attention-item"')

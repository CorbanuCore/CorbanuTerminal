"""Offline injected Slack exchange. No network, credentials, registration or retry loop."""
from contextlib import contextmanager
import copy
import fcntl
import html
import json
import os
import re
import tempfile
from urllib.parse import urlsplit

import decisions as d
from attention import document_url


def token(value):
    d.text(value, limit=100)
    d.require(re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]{0,99}", value))
    return value


def identity(value):
    d.shape(value, "team channel app bot generation human")
    for key, prefix in (("team", "T"), ("channel", "C"), ("app", "A"), ("bot", "B"), ("human", "U")):
        d.require(re.fullmatch(prefix + r"[A-Z0-9]{2,40}", token(value[key])))
    token(value["generation"])
    return copy.deepcopy(value)


def owner(value):
    d.shape(value, "agent allocation running")
    token(value["agent"])
    token(value["allocation"])
    d.require(type(value["running"]) is bool)
    return copy.deepcopy(value)


def slack_ts(value):
    d.require(type(value) is str and re.fullmatch(r"[1-9][0-9]{0,15}\.[0-9]{6}", value))
    return value


class Store:
    """Private, bounded, checksummed files; one cooperating-process lock.

    Explicit initialization requires an empty owner-private directory. Missing
    state on reopen is held, never recreated. Atomic replace + directory fsync
    provides restart durability, not immunity to disk loss or a hostile owner.
    """
    def __init__(self, root):
        self.root = d.fixture_root(root)

    def initialize(self):
        d.require(not list(self.root.iterdir()))
        fd = os.open(self.root / ".slack.lock", os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
        os.close(fd)
        with self.lock(initial=True):
            self.write("alerts", {})
            self.write("replies", {"events": {}, "aliases": {}, "intents": {}})

    @contextmanager
    def lock(self, initial=False):
        try:
            d.fixture_root(self.root)
            fd = os.open(self.root / ".slack.lock", os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK)
            with os.fdopen(fd, "r+") as lock:
                d.owner_only(os.fstat(lock.fileno()))
                fcntl.flock(lock, fcntl.LOCK_EX)
                if not initial:
                    self.read("alerts")
                    self.read("replies")
                yield
        except (OSError, ValueError, TypeError, KeyError, AttributeError, RuntimeError, RecursionError, OverflowError):
            raise d.Invalid() from None

    def read(self, name):
        d.require(name in ("alerts", "replies"))
        fd = os.open(self.root / (name + ".json"), os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as stream:
            d.owner_only(os.fstat(stream.fileno()))
            raw = stream.read(d.MAX_BYTES + 1)
        d.require(len(raw) <= d.MAX_BYTES)
        value = json.loads(raw, object_pairs_hook=d.pairs)
        d.shape(value, "schema body digest")
        d.require(type(value["schema"]) is int and value["schema"] == 2)
        d.require(type(value["body"]) is dict and value["digest"] == d.digest(value["body"]))
        d.require(not d.SECRET.search(raw.decode("utf-8")))
        if name == "replies":
            d.shape(value["body"], "events aliases intents")
            d.require(all(type(item) is dict for item in value["body"].values()))
        return value["body"]

    def write(self, name, body):
        d.require(name in ("alerts", "replies"))
        raw = d.canonical(dict(schema=2, body=body, digest=d.digest(body)))
        d.require(len(raw) <= d.MAX_BYTES and not d.SECRET.search(raw.decode("utf-8")))
        destination = self.root / (name + ".json")
        if destination.exists() or destination.is_symlink():
            d.owner_only(destination.lstat())
        fd, pending = tempfile.mkstemp(prefix=".slack-pending-", dir=self.root)
        try:
            with os.fdopen(fd, "wb") as stream:
                stream.write(raw)
                stream.flush()
                os.fsync(stream.fileno())
            os.replace(pending, destination)
            pending = None
            fd = os.open(self.root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
            try:
                os.fsync(fd)
            finally:
                os.close(fd)
        finally:
            if pending is not None:
                os.unlink(pending)


def prepare(raw, decision_id, remote, pinned_identity, allocation, now):
    """remote is the manager-approved HTTPS dashboard origin, never reply input."""
    feed = d.validate(raw, now)
    d.text(remote, limit=300)
    url = urlsplit(remote)
    d.require(url.scheme == "https" and url.netloc == url.hostname and url.path == "")
    d.require(not url.query and not url.fragment and re.fullmatch(r"[a-z][a-z0-9-]*(?:\.[a-z][a-z0-9-]*)+", url.hostname or ""))
    d.require(not url.hostname.endswith((".local", ".localhost", ".internal")))
    matches = [item for item in feed["decisions"] if item["id"] == decision_id]
    d.require(len(matches) == 1)
    record = matches[0]["revisions"][-1]
    kind = "resolved" if record["status"] in ("resolved", "superseded") else "new" if record["revision"] == 1 else "revised"
    binding = dict(feed_id=feed["feed_id"], feed_revision=feed["revision"], feed_digest=d.digest(feed),
                   decision_id=decision_id, record=record, context_digest=d.digest(record), kind=kind,
                   identity=identity(pinned_identity), owner=owner(allocation), remote=remote)
    suffix = "?feed=" + feed["feed_id"] + "&decision=" + decision_id + "&revision=" + str(record["revision"]) + "&context=" + d.digest(record)
    clean = lambda value: html.escape(str(value), quote=False).replace("@", "＠")
    detail = [clean(record["question"])]
    detail.append(clean(f"Revision {record['revision']}; {record['status']}; raised {record['raised_at']}; updated {record['updated_at']}"))
    detail.extend(clean(key + ": " + (record[key] or "Unknown")) for key in d.CONTEXT)
    detail.extend(clean("Option: " + option) for option in record["options"])
    for ref in record["sprints"]:
        d.require(ref["path"] is not None)
        detail.append(clean(ref["sprint_id"]) + ": " + remote + "/" + document_url(ref["path"]) + suffix)
    for evidence in record["evidence"]:
        detail.append(clean(evidence["label"]) + ": " + (remote + "/" + document_url(evidence["path"]) + suffix if evidence["path"] else "Unknown"))
    if record["resolution"]:
        detail.extend(clean(key + ": " + record["resolution"][key]) for key in ("actor", "answer", "scope"))
    payloads = {"parent": kind.upper() + ": " + clean(record["summary"]) + "\n" + remote + "/index.html" + suffix + "#decision-" + decision_id,
                "details": "\n".join(detail)}
    d.require(all(len(value.encode()) <= 12000 for value in payloads.values()))
    binding["payloads"] = {key: dict(text=value, mrkdwn=False, unfurl_links=False, unfurl_media=False) for key, value in payloads.items()}
    return binding


def alert_key(intent):
    # One notification per destination/question revision, independent of feed assessment.
    return d.digest([intent["feed_id"], intent["decision_id"], intent["record"]["revision"],
                     intent["identity"]["team"], intent["identity"]["channel"]])


def enqueue(store, raw, decision_id, remote, pinned_identity, allocation, now):
    intent = prepare(raw, decision_id, remote, pinned_identity, allocation, now)
    key = alert_key(intent)
    with store.lock():
        rows = store.read("alerts")
        if key in rows:
            original = alert(rows, key)["intent"]
            # Retain original provenance; changed context/identity/allocation needs manager recovery.
            d.require(all(original[k] == intent[k] for k in intent if k not in ("feed_revision", "feed_digest")))
        else:
            rows[key] = dict(intent=copy.deepcopy(intent), intent_digest=d.digest(intent), cancelled=False, reason=None,
                             parent=dict(state="pending", request=None, receipt=None),
                             details=dict(state="pending", request=None, receipt=None))
            store.write("alerts", rows)
    return key


def alert(rows, key):
    row = rows[key]
    d.require(alert_key(row["intent"]) == key and d.digest(row["intent"]) == row["intent_digest"])
    return row


def recover(row):
    for phase in ("parent", "details"):
        if row[phase]["state"] == "sending":
            row[phase]["state"] = "uncertain"


def inspect(store, key):
    with store.lock():
        rows = store.read("alerts")
        row = alert(rows, key)
        recover(row)
        store.write("alerts", rows)
        return copy.deepcopy(row)


def request_for(key, row, phase):
    payload = copy.deepcopy(row["intent"]["payloads"][phase])
    thread = row["parent"]["receipt"]["ts"] if phase == "details" else None
    payload["thread_ts"] = thread
    return dict(attempt=d.digest([key, phase]), identity=row["intent"]["identity"],
                payload=payload, payload_digest=d.digest(payload), thread_ts=thread)


def check_receipt(request, evidence):
    d.shape(evidence, "attempt identity payload_digest thread_ts ts")
    d.require(all(evidence[key] == request[key] for key in ("attempt", "identity", "payload_digest", "thread_ts")))
    slack_ts(evidence["ts"])
    d.require(evidence["ts"] != request["thread_ts"])
    return copy.deepcopy(evidence)


class Rejected(Exception):
    """Injected transport guarantees this attempt was NOT accepted remotely."""


def send(store, key, current_identity, exchange, cancelled=False):
    """exchange(request) returns authenticated receipt; exceptions never get logged.

    Hold ownership across exchange. A crash leaves sending, which becomes uncertain
    on reopen. No automatic retry, including definitive failures or cancellation.
    """
    d.require(type(cancelled) is bool)
    with store.lock():
        rows = store.read("alerts")
        row = alert(rows, key)
        recover(row)
        row["cancelled"] = row["cancelled"] or cancelled
        if row["cancelled"] or identity(current_identity) != row["intent"]["identity"]:
            row["reason"] = "cancelled" if row["cancelled"] else "identity-changed"
        else:
            row["reason"] = None
            for phase in ("parent", "details"):
                state = row[phase]
                if state["state"] == "sent":
                    continue
                if state["state"] != "pending":
                    break
                state.update(state="sending", request=request_for(key, row, phase))
                store.write("alerts", rows)
                try:
                    state["receipt"] = check_receipt(state["request"], exchange(copy.deepcopy(state["request"])))
                    state["state"] = "sent"
                except Rejected:
                    state["state"] = "failed"
                except Exception:
                    state["state"] = "uncertain"
                store.write("alerts", rows)
                if state["state"] != "sent":
                    break
        store.write("alerts", rows)
        return copy.deepcopy(row)


def reconcile(store, key, phase, evidence):
    """Only authenticated exact retained receipt evidence can resolve uncertainty."""
    d.require(phase in ("parent", "details"))
    with store.lock():
        rows = store.read("alerts")
        state = alert(rows, key)[phase]
        d.require(state["state"] in ("sending", "uncertain", "sent"))
        receipt = check_receipt(state["request"], evidence)
        d.require(state["receipt"] in (None, receipt))
        state.update(state="sent", receipt=receipt)
        store.write("alerts", rows)

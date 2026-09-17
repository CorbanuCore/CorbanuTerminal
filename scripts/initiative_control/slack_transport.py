"""Explicit, bounded Slack SDK I/O. Credentials are injected, never journaled."""
import copy
from contextlib import contextmanager, nullcontext
import datetime as dt
import fcntl
import hashlib
import logging
import os
import re
import stat
import tempfile
import threading
import time
import uuid

import decisions as d
import decision_alerts as a
import decision_replies as r

SCOPES = {"chat:write", "groups:history"}
UI_CHECKS = ["private-channel", "human-member", "bot-member", "app-identity", "socket-mode", "message.groups", "connections:write"]
QUIET = logging.Logger("corbanu-slack-private", level=logging.CRITICAL + 1)
QUIET.addHandler(logging.NullHandler())
QUIET.propagate = False
LEASE_NS = 5_000_000_000
RESTART_REFUSED_EXIT = 73
HELD_HUMAN_SECONDS = 900
HELD_HUMAN_LIMIT = 100
FAILURE_REASONS = {
    "transport-busy", "transport-io", "validation-failed", "sdk-failed",
    "restart-refused", "shutdown-timeout", "child-signalled", "child-unreported",
}
FAILURE_STAGES = {
    "bootstrap", "runtime", "credentials", "sdk-init", "session-start",
    "connect", "session-renew", "callback-fence", "callback-envelope",
    "callback-store", "callback-ack", "disconnect", "shutdown", "supervisor",
}
QUARANTINE_REASONS = {
    "unknown-delete", "unbound-human-thread", "nonhuman-author",
    "unsupported-subtype", "invalid-message", "unbound-expired",
}


def failure_record(reason, stage, callbacks=None, disconnect_marks=None, quarantined=None):
    value = dict(reason=reason, stage=stage, callbacks=callbacks,
                 disconnect_marks=disconnect_marks, quarantined=quarantined)
    validate_failure(value)
    return value


def validate_failure(value):
    d.shape(value, "reason stage callbacks disconnect_marks quarantined")
    d.require(value["reason"] in FAILURE_REASONS and value["stage"] in FAILURE_STAGES)
    d.require(all(value[k] is None or type(value[k]) is int and 0 <= value[k] <= d.MAX_BYTES
                  for k in ("callbacks", "disconnect_marks", "quarantined")))
    return value


def failure_reason(error):
    # Never serialize exception text, class names, SDK responses or URLs.
    if isinstance(error, BlockingIOError):
        return "transport-busy"
    if isinstance(error, OSError):
        return "transport-io"
    if isinstance(error, (ValueError, KeyError, TypeError, AttributeError)):
        return "validation-failed"
    return "sdk-failed"


class NormalizationRejected(d.Invalid):
    def __init__(self, reason):
        d.require(reason in QUARANTINE_REASONS)
        self.reason = reason


def normalized_require(condition, reason):
    if not condition:
        raise NormalizationRejected(reason)


def quarantine(value, payload, pin, offset, reason, now):
    """Retain a bounded Slack locator and fixed hints, never message content."""
    event = payload.get("event")
    event = event if type(event) is dict else {}
    original = (event.get("message") if event.get("subtype") == "message_changed" else
                event.get("previous_message") if event.get("subtype") == "message_deleted" else event)
    original = original if type(original) is dict else {}
    author = original.get("user")
    hint = "unknown" if not isinstance(author, str) else "pinned-human" if author == pin["human"] else "other"
    subtype = event.get("subtype")
    shape = ({None: "message", "message_changed": "edit", "message_deleted": "delete"}.get(subtype, "other")
             if isinstance(subtype, (str, type(None))) else "other")
    timestamp = event.get("deleted_ts") if subtype == "message_deleted" else original.get("ts")
    try:
        timestamp = a.slack_ts(timestamp)
    except (ValueError, TypeError):
        timestamp = None
    record = dict(reason=reason, ingress=offset, author_hint=hint, shape=shape,
                  channel=pin["channel"], message_ts=timestamp, at=now)
    append_quarantine(value, record)


def outstanding_quarantine(value):
    """Current review obligation; historical/expired records are not live intake."""
    audit = value.get("quarantine", {})
    if "outstanding" in audit:
        return copy.deepcopy(audit["outstanding"])
    reviewed = max((row["ingress"] for row in value.get("gap_reviews", [])), default=0)
    records = audit.get("records", [])
    pending = [row for row in records
               if row["ingress"] > reviewed and row["reason"] != "unbound-expired"]
    # Missing legacy dispositions cannot prove a live review obligation.
    # In particular, comparing an old review with the moving journal ingress
    # re-arms reviewed history after an unrelated callback. Keep that history
    # unknown until an exact review durably writes the outstanding summary.
    pruned = audit.get("total", 0) - len(records)
    times = [row.get("at") for row in pending]
    result = dict(count=len(pending), oldest_at=min(times) if times and all(times) else None)
    if pruned:
        result["unknown"] = pruned
    return result


def append_quarantine(value, record):
    d.stamp(record["at"])
    outstanding = outstanding_quarantine(value)
    if record["reason"] != "unbound-expired":
        outstanding["oldest_at"] = (min(outstanding["oldest_at"], record["at"])
                                    if outstanding["oldest_at"] is not None else
                                    None if outstanding["count"] else record["at"])
        outstanding["count"] += 1
    audit = value.setdefault("quarantine", dict(total=0, records=[]))
    audit["outstanding"] = outstanding
    if not audit["total"]:
        audit["oldest_at"] = record["at"]
    audit["total"] += 1
    audit["records"].append(record)
    del audit["records"][:-128]
    value["hold"] = "ingress-held"  # A possible answer needs review, never implicit authorization.


def utc_now():
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def retain_human(value, event, offset, now):
    """ACK only after durable retention; duplicates never extend the deadline."""
    held = value.setdefault("held_human", {})
    key = event["event_id"]
    if key in held:
        d.require(held[key]["envelope"] == event)
        return
    d.require(key not in value["events"] and len(held) < HELD_HUMAN_LIMIT)
    expires = d.stamp(now) + dt.timedelta(seconds=HELD_HUMAN_SECONDS)
    held[key] = dict(envelope=event, ingress=offset, arrived_at=now,
                     expires_at=expires.strftime("%Y-%m-%dT%H:%M:%SZ"))
    value["hold"] = "ingress-held"


def enqueue_event(value, event):
    key = event["event_id"]
    route = value["routes"][event["thread_ts"]]
    d.require(event["message_ts"] not in (event["thread_ts"], route["details"]))
    if key in value["events"]:
        d.require(value["events"][key]["envelope"] == event)
    else:
        d.require(sum(not e["drained"] for e in value["events"].values()) < 100)
        value["events"][key] = dict(alert=route["alert"], envelope=event, drained=False)
        value["watermark"] += 1


def settle_held(value, now):
    """Caller persists expiry/replay atomically under the transport lock."""
    held = value.get("held_human", {})
    for key, entry in sorted(held.items(), key=lambda item: item[1]["ingress"]):
        event = entry["envelope"]
        route = value["routes"].get(event["thread_ts"])
        expired = route is None and d.stamp(now) >= d.stamp(entry["expires_at"])
        if route is None and not expired:
            continue
        invalid = route is not None and event["message_ts"] in (event["thread_ts"], route["details"])
        if expired or invalid:
            append_quarantine(value, dict(reason="unbound-expired" if expired else "invalid-message",
                ingress=entry["ingress"], author_hint="pinned-human", shape=event["kind"],
                channel=event["channel"], message_ts=event["message_ts"], at=now))
        else:
            if sum(not e["drained"] for e in value["events"].values()) >= 100:
                continue  # Keep the original deadline; drain will retry after freeing capacity.
            enqueue_event(value, event)
        del held[key]


class RestartRefused(d.Invalid):
    """A restart was denied before session admission; not a running listener exit."""


def legacy_client_msg_id(attempt):
    """Use the exact pre-guard conversion: a newly rejected ID may have been sent."""
    if type(attempt) is not str:
        return None
    try:
        return str(uuid.UUID(attempt[:32]))
    except ValueError:
        return None


def never_sent(request):
    d.require(legacy_client_msg_id(request["attempt"]) is None)
    return dict(state="never-sent", reason="invalid-client-msg-id",
                attempt=request["attempt"], request_digest=d.digest(request))


def reconciled_never_sent(post):
    record = post.get("reconciliation")
    if record is None:
        return False
    d.shape(record, "outcome evidence at")
    d.require(post["receipt"] is None and record["outcome"] == never_sent(post["request"]))
    a.token(record["evidence"])
    d.stamp(record["at"])
    return True


def initialize(store):
    """Explicit extension only; partial/missing existing initialization never resets."""
    with store.lock():
        d.require(not (store.root / "transport.json").exists())
        fd = os.open(store.root / ".transport.lock", os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
        os.close(fd)
        fd = os.open(store.root / ".ingress.fence", os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
        os.close(fd)
        fd = os.open(store.root / ".listener.owner.lock", os.O_CREAT | os.O_EXCL | os.O_RDWR, 0o600)
        stat = os.fstat(fd)
        os.close(fd)
        fd = os.open(store.root / ".listener.runtime.lock", os.O_CREAT | os.O_EXCL | os.O_RDWR, 0o600)
        try:
            runtime = os.fstat(fd)
            os.fsync(fd)
        finally:
            os.close(fd)
        store.write("transport", dict(binding=None, last_verified=None, hold="unqualified", watermark=0,
            events={}, posts={}, routes={}, bridges={}, gap_reviews=[], ingress=0, ui_evidence=None, schema=2,
            runtime_guard=dict(version=1, file=[runtime.st_dev, runtime.st_ino]),
            lifecycle=dict(owner_file=[stat.st_dev, stat.st_ino], session=None, epoch=0, history=[])))


def ingress_count(store, mark=False):
    """Append before lock acquisition; no callback can erase another's failed mark."""
    path = store.root / ".ingress.fence"
    fd = os.open(path, (os.O_WRONLY | os.O_APPEND if mark else os.O_RDONLY) | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "ab" if mark else "rb") as stream:
        d.owner_only(os.fstat(stream.fileno()))
        d.require(os.fstat(stream.fileno()).st_size < d.MAX_BYTES)
        if mark:
            before = os.fstat(stream.fileno()).st_size
            try:
                os.write(stream.fileno(), b"!")
                offset = os.lseek(stream.fileno(), 0, os.SEEK_CUR)
                os.fsync(stream.fileno())
                return offset
            except OSError:
                if os.fstat(stream.fileno()).st_size == before:
                    path.unlink()  # Missing fence is fail-closed, never initialized on recovery.
                raise
        return os.fstat(stream.fileno()).st_size


def fence_gap(store, value):
    """Observation only: never advance ingress or infer the missing arrivals."""
    return abs(ingress_count(store) - value["ingress"])


def restart_identity(journal):
    """Bound the control frame without pruning or weakening lifecycle equality."""
    return dict(binding=copy.deepcopy(journal["binding"]), lifecycle_digest=d.digest(journal["lifecycle"]))


def restart_allowed(store, journal, pin):
    """Recheck under the transport lock immediately before creating a session."""
    try:
        d.require(restart_identity(journal) == pin)
        session = journal["lifecycle"]["session"]
        d.require(session is not None and session["phase"] != "stopped")
        fenced(store, journal)
    except (OSError, ValueError, KeyError, TypeError):
        raise RestartRefused() from None


def fenced(store, value):
    try:
        d.require(ingress_count(store) == value["ingress"])
    except OSError:
        raise d.Invalid() from None


@contextmanager
def locked(store):
    """Never held across network, stdio or offline intake; callback fails fast if busy."""
    fd = os.open(store.root / ".transport.lock", os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "r+") as lock:
        d.owner_only(os.fstat(lock.fileno()))
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        value = store.read("transport")
        fields = "binding last_verified hold watermark events posts routes bridges gap_reviews ingress ui_evidence"
        d.shape(value, fields + (" schema lifecycle" if "schema" in value else "")
                + (" fence_losses" if "fence_losses" in value else "")
                + (" runtime_guard" if "runtime_guard" in value else "")
                + (" listener_events" if "listener_events" in value else "")
                + (" listener_events_pruned" if "listener_events_pruned" in value else "")
                + (" quarantine" if "quarantine" in value else "")
                + (" held_human" if "held_human" in value else ""))
        if "held_human" in value:
            held = value["held_human"]
            d.require(type(held) is dict and len(held) <= HELD_HUMAN_LIMIT)
            for key, entry in held.items():
                d.shape(entry, "envelope ingress arrived_at expires_at")
                event = entry["envelope"]
                d.shape(event, "event_id team channel user thread_ts message_ts event_ts kind text")
                d.require(event["event_id"] == a.token(key))
                d.require([event[k] for k in ("team", "channel", "user")] ==
                          [value["binding"][k] for k in ("team", "channel", "human")])
                for field in ("thread_ts", "message_ts", "event_ts"):
                    a.slack_ts(event[field])
                d.require(event["kind"] in ("message", "edit", "delete"))
                if event["kind"] == "delete":
                    d.require(event["text"] is None)
                else:
                    d.text(event["text"], limit=4000)
                d.require(type(entry["ingress"]) is int and 0 < entry["ingress"] <= value["ingress"])
                d.require((d.stamp(entry["expires_at"]) - d.stamp(entry["arrived_at"])).total_seconds() == HELD_HUMAN_SECONDS)
        if "quarantine" in value:
            audit = value["quarantine"]
            d.shape(audit, "total records" + (" oldest_at" if "oldest_at" in audit else "")
                    + (" outstanding" if "outstanding" in audit else ""))
            if "outstanding" in audit:
                pending = audit["outstanding"]
                d.shape(pending, "count oldest_at" + (" unknown" if "unknown" in pending else ""))
                unknown = pending.get("unknown", 0)
                d.require(type(unknown) is int and 0 <= unknown <= audit["total"])
                d.require(type(pending["count"]) is int and 0 <= pending["count"] <= audit["total"] - unknown)
                d.require(pending["count"] or pending["oldest_at"] is None)
                if pending["oldest_at"] is not None:
                    d.stamp(pending["oldest_at"])
            if "oldest_at" in audit:
                d.stamp(audit["oldest_at"])
            d.require(type(audit["records"]) is list and len(audit["records"]) <= 128)
            d.require(type(audit["total"]) is int and audit["total"] >= len(audit["records"]))
            for record in audit["records"]:
                d.shape(record, "reason ingress author_hint shape"
                        + (" channel message_ts at" if "at" in record else ""))
                if "at" in record:
                    d.stamp(record["at"])
                    d.require(record["channel"] == value["binding"]["channel"])
                    if record["message_ts"] is not None:
                        a.slack_ts(record["message_ts"])
                d.require(record["reason"] in QUARANTINE_REASONS
                          and record["author_hint"] in ("unknown", "pinned-human", "other")
                          and record["shape"] in ("message", "edit", "delete", "other")
                          and type(record["ingress"]) is int and 0 < record["ingress"] <= value["ingress"])
        if "listener_events_pruned" in value:
            summary = value["listener_events_pruned"]
            d.shape(summary, "events child_exits last_at")
            d.require(type(summary["events"]) is int and type(summary["child_exits"]) is int
                      and 0 <= summary["child_exits"] <= summary["events"] and summary["events"] > 0)
            d.stamp(summary["last_at"])
        d.require(type(value["watermark"]) is int and value["watermark"] >= 0)
        losses(value)
        yield value


def losses(value):
    records = value.get("fence_losses", [])
    d.require(type(records) is list)
    d.require(not records or value.get("schema") == 2)
    seen = set()
    for record in records:
        d.shape(record, "intent intent_digest phase")
        d.require(record["intent_digest"] == d.digest(record["intent"]) and record["phase"] in ("prepared", "restored"))
        intent = record["intent"]
        d.shape(intent, "id root binding ingress watermark lifecycle retired barrier_size barrier_digest transport_digest replies_digest prior_hold evidence at")
        d.require(type(intent["id"]) is str and len(intent["id"]) == 64 and intent["id"] not in seen)
        seen.add(intent["id"])
        d.require(type(intent["ingress"]) is int and 0 <= intent["ingress"] < d.MAX_BYTES - 65)
        d.require(type(intent["retired"]) is dict)
        for prior in intent["retired"].values():
            d.shape(prior, "digest intent_digest cancelled reason")
    return records


def loss_ready(value, store=None):
    for record in losses(value):
        d.require(record["phase"] == "restored")
        if store is not None:  # Qualification caller holds store -> transport.
            rows = store.read("alerts")
            for key, prior in record["intent"]["retired"].items():
                row = a.alert(rows, key)
                d.require(row["cancelled"] is True and row["reason"] == "cancelled"
                          and row["intent_digest"] == prior["intent_digest"])


def loss_case(store, binding, journal):
    d.require(journal.get("schema") == 2 and journal["binding"] == a.identity(binding)
              and journal["last_verified"] is not None)
    rows, replies = store.read("alerts"), store.read("replies")
    for key in rows:
        a.alert(rows, key)
    return d.digest([str(store.root), binding, rows, replies, journal])


def inspect_fence_loss(store, binding):
    with store.lock(), locked(store) as value:
        case = loss_case(store, binding, value)
        pending = [r for r in losses(value) if r["phase"] == "prepared"]
        d.require(len(pending) <= 1)
        if pending:
            case = pending[0]["intent"]["id"]
        else:
            d.require(not os.path.lexists(store.root / ".ingress.fence"))
        return dict(case_digest=case, retained_ingress=value["ingress"], watermark=value["watermark"],
                    affected_alert_count=len(store.read("alerts")), state="held")


def recover_missing_fence(store, binding, *, expected_digest, evidence, quiesce_listener=None, now,
                          checkpoint=lambda _: None):
    """Trusted hook stops/joins actual runtime OUTSIDE locks; no production hook supplied.

    The synthetic barrier records loss, not recovered arrivals. Every repair
    returns held. Unpublished crash staging files remain; a published duplicate
    name is removed only after exact inode/content validation. Neither is old ingress.
    """
    d.require(callable(quiesce_listener))
    d.require(quiesce_listener() is None)  # A stopped=True/PID assertion is not this trusted integration seam.
    a.token(evidence)
    d.stamp(now)
    with store.lock(), locked(store) as value:
        case = loss_case(store, binding, value)
        fd = owner_file(store, value)
        try:
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            records = losses(value)
            matches = [r for r in records if r["intent"]["id"] == expected_digest]
            d.require(len(matches) <= 1)
            path = store.root / ".ingress.fence"
            if matches:
                record = matches[0]
                intent = record["intent"]
                d.require(intent["binding"] == binding and intent["root"] == str(store.root) and intent["evidence"] == evidence)
                if record["phase"] == "restored":
                    ingress_count(store)  # A new disappearance needs a new incident, not replay of this one.
                    loss_ready(value, store)
                    return dict(loss_id=expected_digest, state="held", retired_count=len(intent["retired"]))
            else:
                loss_ready(value, store)
                d.require(case == expected_digest and not os.path.lexists(path))
                d.require(type(value["ingress"]) is int and 0 <= value["ingress"] < d.MAX_BYTES - 65)
                retired = {key: dict(digest=d.digest(row), intent_digest=row["intent_digest"],
                           cancelled=row["cancelled"], reason=row["reason"]) for key, row in store.read("alerts").items()}
                barrier = b"!" * value["ingress"] + b"?" + case.encode("ascii")
                intent = dict(id=case, root=str(store.root), binding=copy.deepcopy(binding), ingress=value["ingress"],
                    watermark=value["watermark"], lifecycle=copy.deepcopy(value["lifecycle"]), retired=retired,
                    barrier_size=len(barrier), barrier_digest=hashlib.sha256(barrier).hexdigest(),
                    transport_digest=d.digest(value), replies_digest=d.digest(store.read("replies")),
                    prior_hold=value["hold"], evidence=evidence, at=now)
                record = dict(intent=intent, intent_digest=d.digest(intent), phase="prepared")
                value.setdefault("fence_losses", []).append(record)
                value["hold"] = "fence-repair-pending"
                store.write("transport", value)
                checkpoint("prepared")
            d.require(value["hold"] == "fence-repair-pending" and value["ingress"] == intent["ingress"]
                      and value["watermark"] == intent["watermark"] and value["lifecycle"] == intent["lifecycle"])
            rows = store.read("alerts")
            d.require(set(rows) == set(intent["retired"]))
            for key, prior in intent["retired"].items():
                row = a.alert(rows, key)
                d.require(row["intent_digest"] == prior["intent_digest"])
                row.update(cancelled=True, reason="cancelled")  # Only these fields change; all attempts remain exact.
            store.write("alerts", rows)
            checkpoint("cancelled")
            barrier = b"!" * intent["ingress"] + b"?" + intent["id"].encode("ascii")
            d.require(0 < len(barrier) < d.MAX_BYTES and len(barrier) == intent["barrier_size"]
                      and hashlib.sha256(barrier).hexdigest() == intent["barrier_digest"])
            if not os.path.lexists(path):
                staging_fd, staging = tempfile.mkstemp(prefix=".ingress-repair-" + intent["id"] + "-", dir=store.root)
                with os.fdopen(staging_fd, "wb") as stream:
                    stream.write(barrier[:1])
                    stream.flush()
                    checkpoint("partial")
                    stream.write(barrier[1:])
                    stream.flush()
                    os.fsync(stream.fileno())
                checkpoint("staged")
                os.link(staging, path, follow_symlinks=False)  # No replacement of an unrelated file.
                checkpoint("linked")
            barrier_fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
            with os.fdopen(barrier_fd, "rb") as stream:
                info = os.fstat(stream.fileno())
                d.require(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid() and info.st_mode & 0o077 == 0)
                d.require(stream.read(d.MAX_BYTES + 1) == barrier)
                if info.st_nlink == 2:  # Recover only our exact interrupted no-replace publication.
                    with os.scandir(store.root) as entries:
                        linked = []
                        for count, entry in enumerate(entries):
                            d.require(count < 256)
                            other = entry.stat(follow_symlinks=False)
                            if entry.name.startswith(".ingress-repair-" + intent["id"] + "-") and (other.st_dev, other.st_ino) == (info.st_dev, info.st_ino):
                                linked.append(entry.path)
                    d.require(len(linked) == 1)
                    os.unlink(linked[0])  # Duplicate name only; the complete final barrier remains.
                d.owner_only(os.fstat(stream.fileno()))
                os.fsync(stream.fileno())
            directory = os.open(store.root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
            try:
                os.fsync(directory)
            finally:
                os.close(directory)
            checkpoint("synced")
            record["phase"] = "restored"
            value["hold"] = "outage-gap"
            loss_ready(value, store)
            store.write("transport", value)
            checkpoint("restored")
            return dict(loss_id=intent["id"], state="held", retired_count=len(rows))
        finally:
            os.close(fd)


def hold(store, reason):
    with locked(store) as value:
        value["hold"] = reason
        store.write("transport", value)


def owner_file(store, journal):
    d.require(journal.get("schema") == 2)
    fd = os.open(store.root / ".listener.owner.lock", os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        stat = os.fstat(fd)
        d.owner_only(stat)
        d.require([stat.st_dev, stat.st_ino] == journal["lifecycle"]["owner_file"])
        os.set_inheritable(fd, False)
        return fd
    except BaseException:
        os.close(fd)
        raise


def observe_session_locked(store, journal):
    """Caller holds transport lock; read/probe only, no nested locks or SDK I/O.

    Returns the exact live session/epoch, not qualification or gap-clear authority.
    An independently opened descriptor must encounter the lifetime owner's flock.
    """
    try:
        loss_ready(journal)
        fd = owner_file(store, journal)
        try:
            try:
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError:
                session = journal["lifecycle"]["session"]
                d.require(session is not None and session["phase"] == "connected"
                          and session["binding"] == d.digest(journal["binding"])
                          and 0 <= time.monotonic_ns() - session["renewed_ns"] < LEASE_NS)
                return session["id"], journal["lifecycle"]["epoch"]
            raise d.Invalid()  # No owner, including SIGKILL before any replacement starts.
        finally:
            os.close(fd)
    except (OSError, KeyError, TypeError):
        raise d.Invalid() from None


class Session:
    """Lifetime flock; transport -> nonblocking owner acquisition, never store/feed.

    Failed durable lifecycle writes release ownership even if a hold cannot persist.
    Reconnect/renewal cannot clear a gap; expired renewal starts another held epoch.
    """
    def __init__(self, store, *, restart_pin=None, now=utc_now):
        self.store, self.fd, self.id = store, None, uuid.uuid4().hex
        self.now = now
        try:
            with locked(store) as journal:
                loss_ready(journal)
                if restart_pin is not None:
                    restart_allowed(store, journal, restart_pin)
                self.fd = owner_file(store, journal)
                fcntl.flock(self.fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                life = journal["lifecycle"]
                if life["session"] is not None:
                    life["history"].append(dict(copy.deepcopy(life["session"]), epoch=life["epoch"]))
                    journal["hold"] = "outage-gap"
                life["epoch"] += 1
                life["session"] = dict(id=self.id, phase="starting", binding=d.digest(journal["binding"]), renewed_ns=0)
                store.write("transport", journal)
        except BaseException:
            self.release()
            raise

    def release(self):
        if self.fd is not None:
            os.close(self.fd)
            self.fd = None

    def update(self, phase):
        try:
            d.require(self.fd is not None)
            with locked(self.store) as journal:
                probe = owner_file(self.store, journal)
                os.close(probe)
                life, now = journal["lifecycle"], time.monotonic_ns()
                session = life["session"]
                d.require(session["id"] == self.id)
                if phase != "connected" or (session["phase"] == "connected"
                        and not 0 <= now - session["renewed_ns"] < LEASE_NS):
                    journal["hold"] = "outage-gap"
                    life["epoch"] += 1
                session.update(phase=phase, renewed_ns=now)
                settle_held(journal, self.now())
                self.store.write("transport", journal)
        except BaseException:
            self.release()
            raise

    def close(self):
        try:
            if self.fd is not None:
                self.update("stopped")
        finally:
            self.release()


def runtime_file(store, binding):
    """Immutable birth pin: never create, adopt or infer quiescence from the lease.

    Atomic journal read avoids lock inversion; callers acquire runtime before
    store/transport, then revalidate. No credential, SDK or mutable PID registry.
    """
    value = store.read("transport")
    d.require(value.get("schema") == 2 and value["binding"] == a.identity(binding))
    pin = value.get("runtime_guard")
    d.shape(pin, "version file")
    d.require(type(pin["version"]) is int and pin["version"] == 1)
    fd = os.open(store.root / ".listener.runtime.lock", os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        info = os.fstat(fd)
        d.owner_only(info)
        d.require(pin["file"] == [info.st_dev, info.st_ino])
        os.set_inheritable(fd, False)
        return fd
    except BaseException:
        os.close(fd)
        raise


def runtime_owned(fd, store, binding):
    probe = runtime_file(store, binding)
    try:
        own, other = os.fstat(fd), os.fstat(probe)
        d.require((own.st_dev, own.st_ino) == (other.st_dev, other.st_ino))
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        try:
            fcntl.flock(probe, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return
        raise d.Invalid()
    finally:
        os.close(probe)


class _ChildRuntime:
    """Only the dedicated child bootstrap constructs this; NO close/finalizer.

    Its descriptor dies with the process, never with Session/SDK stack unwinding.
    PID is only an anti-fork identity check, not a liveness or shutdown proof.
    """
    def __init__(self, fd, store, binding):
        runtime_owned(fd, store, binding)
        self.fd, self.root, self.binding, self.process = fd, store.root, copy.deepcopy(binding), os.getpid()

    def check(self, store, binding):
        d.require(self.process == os.getpid() and self.root == store.root and self.binding == binding)
        runtime_owned(self.fd, store, binding)


class Transport:
    def __init__(self, store, binding, credentials, *, live=False, now, web_factory=None):
        self.store, self.binding = store, a.identity(binding)
        self.credentials, self.live, self.now = credentials, live is True, now
        self.web_factory, self.client, self.socket = web_factory, None, None
        self.active = False
        self.failure = None
        self.callbacks = self.disconnect_marks = self.quarantined = 0
        self.stage = "runtime"

    def failed(self, error, stage=None):
        if self.failure is None:
            self.failure = failure_record(failure_reason(error), stage or self.stage)

    def failure_snapshot(self):
        value = self.failure or failure_record("sdk-failed", self.stage)
        return dict(value, callbacks=min(self.callbacks, d.MAX_BYTES),
                    disconnect_marks=min(self.disconnect_marks, d.MAX_BYTES),
                    quarantined=min(self.quarantined, d.MAX_BYTES))

    def web(self):
        d.require(self.live)
        if self.client is None:
            from slack_sdk import WebClient
            bot, _ = self.credentials()
            self.client = (self.web_factory or WebClient)(token=bot, timeout=5, retry_handlers=[], logger=QUIET)
        return self.client

    def gate(self, allow_hold=False, local=False):
        d.require(local or self.live)
        with locked(self.store) as value:
            d.require(value["binding"] == self.binding and value["last_verified"] is not None)
            if not local:
                loss_ready(value)
                age = (d.stamp(self.now()) - d.stamp(value["last_verified"])).total_seconds()
                d.require(0 <= age <= 900 and (allow_hold or value["hold"] is None))
                if not allow_hold:
                    fenced(self.store, value)
                    observe_session_locked(self.store, value)
            return copy.deepcopy(value)

    def qualify(self, ui_evidence, gap_review=None):
        """Owner's supported-UI evidence, not extra OAuth lookups or Slack assertions."""
        d.require(self.live)
        d.shape(ui_evidence, "binding observed_at receipt checks")
        d.require(ui_evidence["binding"] == self.binding and ui_evidence["checks"] == UI_CHECKS
                  and d.stamp(ui_evidence["observed_at"]) <= d.stamp(self.now()))
        a.token(ui_evidence["receipt"])
        ingress_count(self.store)  # Preserve missing-file refusal; authority still rechecks under both locks below.
        with self.store.lock(), locked(self.store) as value:
            loss_ready(value, self.store)
            d.require(value.get("schema") == 2)
            previous_hold, watermark = value["hold"], value["watermark"]
            session_pin = observe_session_locked(self.store, value) if value["lifecycle"]["session"] else None
            ingress = ingress_count(self.store)
            uncovered = ingress != value["ingress"]
            life = value["lifecycle"]
            bootstrap = (previous_hold in ("unqualified", "qualifying")
                and all(value[k] is None for k in ("binding", "last_verified", "ui_evidence"))
                and ingress == 0 and all(type(value[k]) is int and value[k] == 0 for k in ("ingress", "watermark"))
                and all(value[k] == {} for k in ("events", "posts", "routes", "bridges")) and value["gap_reviews"] == []
                and type(life["epoch"]) is int and life == dict(owner_file=life["owner_file"], session=None, epoch=0, history=[]))
            if session_pin is None and (value["binding"] is None or value["last_verified"] is None):
                d.require(bootstrap)  # Never overwrite the sole evidence of a pre-qualification outage.
            if bootstrap:
                os.close(owner_file(self.store, value))
            value["hold"] = "qualifying"
            self.store.write("transport", value)
            initial = copy.deepcopy(value) if bootstrap else None
        web, pin = self.web(), self.binding
        auth = web.auth_test()
        d.require(auth["team_id"] == pin["team"] and auth["bot_id"] == pin["bot"])
        scopes = {s.strip() for s in auth.headers.get("x-oauth-scopes", "").split(",")}
        d.require(SCOPES <= scopes)
        with self.store.lock(), locked(self.store) as value:
            loss_ready(value, self.store)
            d.require(session_pin == (observe_session_locked(self.store, value) if value["lifecycle"]["session"] else None))
            d.require(value["binding"] in (None, pin))
            d.require(value["hold"] == "qualifying" and value["watermark"] == watermark)
            d.require(ingress_count(self.store) == ingress)
            if bootstrap:
                d.require(value == initial)  # Auth I/O cannot hide intervening evidence or lifecycle changes.
                os.close(owner_file(self.store, value))
            # An explicit exact review also clears legacy unknown history on a
            # quiet journal; requiring a fresh fault would strand that history.
            if not bootstrap and (gap_review is not None or uncovered
                                  or previous_hold not in (None, "unqualified")):
                d.shape(gap_review, "watermark binding evidence ingress session epoch")
                d.require(session_pin is not None and (gap_review["session"], gap_review["epoch"]) == session_pin)
                d.require(gap_review["watermark"] == value["watermark"] and gap_review["binding"] == d.digest(pin))
                d.require(gap_review["ingress"] == ingress)
                a.token(gap_review["evidence"])
                d.require(not any(not e["drained"] for e in value["events"].values()))
                value["gap_reviews"].append(dict(gap_review, at=self.now()))
                if "quarantine" in value:
                    value["quarantine"]["outstanding"] = dict(count=0, oldest_at=None)
            value.update(binding=pin, last_verified=self.now(), hold=None, ingress=ingress, ui_evidence=copy.deepcopy(ui_evidence))
            self.store.write("transport", value)
        return pin

    def exchange(self, request):
        attempt = request["attempt"]
        d.require(type(attempt) is str and re.fullmatch(r"[0-9a-f]{64}", attempt) is not None)
        client_msg_id = legacy_client_msg_id(attempt)
        d.require(client_msg_id is not None)  # Before any durable record or credential/HTTP access.
        try:
            admitted = self.gate()["lifecycle"]
            d.require(request["identity"] == self.binding and request["payload_digest"] == d.digest(request["payload"]))
            with locked(self.store) as value:
                d.require(observe_session_locked(self.store, value) == (admitted["session"]["id"], admitted["epoch"]))
                fenced(self.store, value)
                d.require(value["hold"] is None)
                if attempt in value["posts"]:
                    saved = value["posts"][attempt]
                    d.require(saved["request"] == request and saved["receipt"] is not None)
                    return copy.deepcopy(saved["receipt"])
                value["posts"][attempt] = dict(request=copy.deepcopy(request), receipt=None, retry_after=None)
                self.store.write("transport", value)
                d.require(observe_session_locked(self.store, self.store.read("transport")) == (admitted["session"]["id"], admitted["epoch"]))
                fenced(self.store, value)  # A slow durable request write cannot authorize a later stale POST.
        except Exception:
            # Only positive absence permits retry. Existing attempts or an unreadable
            # journal retain uncertainty, including a failed post-write fence check.
            with locked(self.store) as value:
                if attempt not in value["posts"]:
                    raise a.NotDispatched() from None
            raise
        try:
            response = self.web().chat_postMessage(channel=self.binding["channel"],
                client_msg_id=client_msg_id, **request["payload"])
            d.require(response["ok"] is True and response["channel"] == self.binding["channel"])
            evidence = a.check_receipt(request, dict(attempt=attempt, identity=self.binding,
                payload_digest=request["payload_digest"], thread_ts=request["thread_ts"], ts=response["ts"]))
            with locked(self.store) as value:
                value["posts"][attempt]["receipt"] = evidence
                self.store.write("transport", value)
            return evidence
        except Exception as error:
            response = getattr(error, "response", None)
            if response is not None:
                with locked(self.store) as value:
                    retry = response.headers.get("Retry-After", response.headers.get("retry-after"))
                    value["posts"][attempt]["retry_after"] = int(retry) if str(retry).isdigit() else None
                    self.store.write("transport", value)
                if response.status_code < 500 and response.get("error") in {
                    "invalid_auth", "not_authed", "missing_scope", "channel_not_found", "not_in_channel", "is_archived", "ratelimited"}:
                    raise a.Rejected() from None
            raise d.Invalid() from None

    def reconcile(self, request):
        """Classify impossible legacy IDs locally; valid IDs require positive association."""
        value = self.gate(allow_hold=True)
        saved = value["posts"][request["attempt"]]
        d.require(saved["request"] == request)
        d.require(request["identity"] == self.binding and request["payload_digest"] == d.digest(request["payload"]))
        if saved["receipt"] is not None:
            return a.check_receipt(request, saved["receipt"])
        client_msg_id = legacy_client_msg_id(request["attempt"])
        if client_msg_id is None:
            return never_sent(request)  # Classification only; owner evidence is required to clear the hold.
        found, cursor = [], None
        for _ in range(3):
            args = dict(channel=self.binding["channel"], limit=15, cursor=cursor)
            result = (self.web().conversations_replies(ts=request["thread_ts"], **args) if request["thread_ts"]
                      else self.web().conversations_history(**args))
            for message in result["messages"]:
                # Unique durable attempt + authenticated pinned conversation/identity/thread proves acceptance.
                # Slack rewrites URLs/emoji; rendered text is not a payload-integrity or comprehension receipt.
                if (message.get("client_msg_id") == client_msg_id
                        and message.get("bot_id") == self.binding["bot"] and message.get("app_id") == self.binding["app"]
                        and message.get("channel", self.binding["channel"]) == self.binding["channel"]
                        and (message.get("thread_ts") in (None, message.get("ts")) if request["thread_ts"] is None
                             else message.get("thread_ts") == request["thread_ts"])):
                    found.append(message["ts"])
            cursor = result.get("response_metadata", {}).get("next_cursor")
            if not cursor:
                break
        d.require(len(found) == 1 and not cursor)
        evidence = a.check_receipt(request, dict(attempt=request["attempt"], identity=self.binding,
            payload_digest=request["payload_digest"], thread_ts=request["thread_ts"], ts=found[0]))
        with locked(self.store) as value:
            value["posts"][request["attempt"]]["receipt"] = evidence
            self.store.write("transport", value)
        return evidence

    def reconcile_orphan(self, attempt, *, request_digest, evidence):
        """Owner-only explicit local repair; never clear an attempt the old sender could encode."""
        a.token(evidence)
        now = self.now()
        d.stamp(now)
        # Owner input arrives straight off the pipe: refuse a non-string the same
        # way every other owner path does, rather than letting it reach the posts
        # index and surface as TypeError.
        d.require(isinstance(attempt, str) and legacy_client_msg_id(attempt) is None)
        admitted = self.gate(allow_hold=True)
        with locked(self.store) as value:
            d.require(value["binding"] == self.binding and value["lifecycle"] == admitted["lifecycle"])
            fenced(self.store, value)
            observe_session_locked(self.store, value)
            saved = value["posts"][attempt]
            request = saved["request"]
            d.require(request["attempt"] == attempt and d.digest(request) == request_digest)
            d.require(request["identity"] == self.binding and request["payload_digest"] == d.digest(request["payload"]))
            d.require(saved["receipt"] is None)
            outcome = never_sent(request)
            if reconciled_never_sent(saved):
                d.require(saved["reconciliation"]["evidence"] == evidence)
            else:
                saved["reconciliation"] = dict(outcome=outcome, evidence=evidence, at=now)
                self.store.write("transport", value)
            return copy.deepcopy(outcome)

    def bind_alert(self, key, row):
        d.require(row["intent"]["identity"] == self.binding and row["parent"]["state"] == row["details"]["state"] == "sent")
        a.check_receipt(a.request_for(key, row, "parent"), row["parent"]["receipt"])
        with locked(self.store) as value:
            d.require(value["binding"] == self.binding)
            thread = row["parent"]["receipt"]["ts"]
            route = dict(alert=key, details=row["details"]["receipt"]["ts"])
            d.require(value["routes"].get(thread) in (None, route))
            value["routes"][thread] = route
            settle_held(value, self.now())
            self.store.write("transport", value)

    def callback(self, client, request):
        """Only registered SDK session callbacks authenticate ingress, not JSON flags."""
        stage = "callback-fence"
        self.callbacks += 1
        try:
            offset = ingress_count(self.store, mark=True)
            stage = "callback-envelope"
            d.require(client is self.socket and self.active and request.type == "events_api")
            payload = request.payload
            d.require(len(d.canonical(payload)) <= 16384)
            d.require(payload["team_id"] == self.binding["team"] and payload["api_app_id"] == self.binding["app"])
            stage = "callback-store"
            with locked(self.store) as value:
                d.require(value["binding"] == self.binding)
                if offset != value["ingress"] + 1:
                    value["hold"] = "ingress-held"  # Resume intake, but never silently cover a missing event.
                settle_held(value, self.now())
                quarantined = False
                try:
                    event = normalize(payload, value, self.binding, allow_unbound=True)
                except (ValueError, KeyError, TypeError, AttributeError) as error:
                    reason = error.reason if isinstance(error, NormalizationRejected) else "invalid-message"
                    quarantine(value, payload, self.binding, offset, reason, self.now())
                    quarantined = True
                    event = None
                if event is not None:
                    if event["thread_ts"] not in value["routes"]:
                        retain_human(value, event, offset, self.now())
                    else:
                        enqueue_event(value, event)
                value["ingress"] = offset
                d.require(len(d.canonical(value)) <= 900000)
                self.store.write("transport", value)
                self.quarantined += int(quarantined)
            stage = "callback-ack"
            from slack_sdk.socket_mode.response import SocketModeResponse
            client.send_socket_mode_response(SocketModeResponse(envelope_id=request.envelope_id))
        except Exception as error:
            self.failed(error, stage)
            self.active = False
            try:
                hold(self.store, "ingress-held")
            except Exception:
                pass  # The independent uncovered/missing fence already denies work after restart.

    def listen(self, seconds=60, stop=None, ongoing=False, *, runtime=None, restart_pin=None):
        d.require(self.live)
        d.require(type(runtime) is _ChildRuntime)
        try:
            runtime.check(self.store, self.binding)  # Before even the SDK constructor can start threads.
            self.gate(local=True)  # Connectivity recovery is not fresh send/work qualification.
        except (OSError, ValueError, KeyError, TypeError):
            if restart_pin is not None:
                raise RestartRefused() from None
            raise
        d.require(type(ongoing) is bool and 0 < seconds <= 60)
        stop = stop or threading.Event()
        deadline = None if ongoing else time.monotonic() + seconds
        self.stage = "sdk-init"
        from slack_sdk.socket_mode import SocketModeClient
        self.stage = "credentials"
        _, app = self.credentials()
        self.stage = "sdk-init"
        self.socket = SocketModeClient(app_token=app, web_client=self.web(), logger=QUIET,
                                      auto_reconnect_enabled=False, concurrency=1)
        # SDK 3.44.1 otherwise retries apps.connections.open on 429 and forcibly
        # reconnects disconnect frames even when automatic reconnect is disabled.
        self.socket.issue_new_wss_url = lambda: self.web().apps_connections_open(app_token=app)["url"]
        def disconnected(*_, **__):
            self.active = False
            try:
                ingress_count(self.store, mark=True)
                self.disconnect_marks += 1
                owner.update("gap")
            except Exception as error:
                self.failed(error, "disconnect")
                owner.release()  # No durable write is required for absent ownership to deny.
        self.socket.connect_to_new_endpoint = disconnected
        self.socket.socket_mode_request_listeners.append(self.callback)
        self.socket.on_close_listeners.append(disconnected)
        self.socket.on_error_listeners.append(disconnected)
        self.stage = "session-start"
        try:
            owner = Session(self.store, restart_pin=restart_pin, now=self.now)
        except BaseException:
            self.socket.close()
            raise
        try:
            failures = 0
            while not stop.is_set() and (deadline is None or time.monotonic() < deadline):
                try:
                    self.stage = "connect"
                    self.active = True
                    self.socket.wss_uri = None
                    self.socket.connect()
                    d.require(self.active and self.socket.is_connected())
                    self.stage = "session-renew"
                    owner.update("connected")
                    renewed, failures = time.monotonic_ns(), 0
                    while self.active and not stop.is_set() and (deadline is None or time.monotonic() < deadline):
                        if not self.socket.is_connected():
                            disconnected()  # SDK check_state can disconnect without invoking listeners.
                            break
                        if time.monotonic_ns() - renewed >= 1_000_000_000:
                            owner.update("connected")
                            renewed = time.monotonic_ns()
                        stop.wait(min(0.1, max(0, deadline - time.monotonic())) if deadline is not None else 0.1)
                except Exception as error:
                    self.failed(error)
                    failures += 1
                    disconnected()
                    if owner.fd is None or failures >= 3 or (deadline is not None and time.monotonic() + 2 ** (failures - 1) >= deadline):
                        raise
                if not stop.is_set() and (deadline is None or time.monotonic() < deadline):
                    stop.wait(2 ** max(0, failures - 1))  # Bounded1/2s backoff; successful sessions reset the streak.
        finally:
            self.stage = "shutdown"
            self.active = False
            disconnected()  # Fence before SDK close/join can block; no shutdown-time live lease.
            try:
                self.socket.close()
            finally:
                try:
                    disconnected()
                finally:
                    owner.close()


def normalize(payload, value, pin, *, allow_unbound=False):
    event = payload["event"]
    if event.get("type") != "message" or event.get("channel") != pin["channel"]:
        return None
    subtype = event.get("subtype")
    message = event.get("message", {}) if subtype == "message_changed" else event
    original = event.get("previous_message", {}) if subtype == "message_deleted" else message
    if original.get("bot_id") == pin["bot"] and original.get("app_id") == pin["app"]:
        return None  # Authenticated pinned outbound echoes precede all human-candidate checks.
    message_ts = event.get("deleted_ts") if subtype == "message_deleted" else message.get("ts")
    candidates = list(value["events"].values()) + list(value.get("held_human", {}).values())
    prior = [e["envelope"] for e in candidates if e["envelope"]["message_ts"] == message_ts]
    if subtype == "message_deleted":
        normalized_require(prior, "unknown-delete")  # Do not invent authorship or a deleting actor.
    thread = original.get("thread_ts") or (prior[0]["thread_ts"] if prior else None)
    author = original.get("user") or (prior[0]["user"] if prior else None)
    if thread not in value["routes"]:
        if not thread or author != pin["human"]:
            return None
        normalized_require(allow_unbound, "unbound-human-thread")
    else:
        d.require(message_ts not in (thread, value["routes"][thread]["details"]))
    normalized_require(author == pin["human"] and not original.get("bot_id"), "nonhuman-author")
    kind = {None: "message", "message_changed": "edit", "message_deleted": "delete"}.get(subtype)
    normalized_require(kind is not None, "unsupported-subtype")
    if kind == "edit":
        d.require(message.get("edited", {}).get("user") == pin["human"]
                  and event.get("user", pin["human"]) == pin["human"]
                  and event.get("previous_message", {}).get("user", pin["human"]) == pin["human"])
    if kind == "delete":
        d.require(prior and all(p["user"] == pin["human"] for p in prior))
    result = dict(event_id=a.token(payload["event_id"]), team=pin["team"], channel=pin["channel"], user=author,
                  thread_ts=a.slack_ts(thread), message_ts=a.slack_ts(message_ts),
                  event_ts=a.slack_ts(event["event_ts"]), kind=kind, text=None if kind == "delete" else message["text"])
    if kind != "delete":
        d.text(result["text"], limit=4000)
    return result


def drain(store, binding=None, *, now=None):
    """Copy ingress first, release its lock before acquiring offline store ownership."""
    deadline, count = time.monotonic() + 30, 0
    with locked(store) as value:
        d.require(value["binding"] is not None and (binding is None or a.identity(binding) == value["binding"]))
        pin = copy.deepcopy(value["binding"])
        if value.get("held_human"):
            settle_held(value, now or utc_now())
            store.write("transport", value)
        pending = [(k, copy.deepcopy(e)) for k, e in value["events"].items() if not e["drained"]][:10]
    for key, entry in pending:
        if time.monotonic() >= deadline:
            break
        bounded = a.Store(store.root, lock_timeout=max(0, deadline - time.monotonic()))
        with bounded.lock():  # Cancellation and intake share one store-serialized observation.
            row = a.alert(bounded.read("alerts"), entry["alert"])
            d.require(type(row["cancelled"]) is bool and row["intent"]["identity"] == pin)
            event = entry["envelope"]
            d.require([event[k] for k in ("team", "channel", "user")] == [pin[k] for k in ("team", "channel", "human")])
            d.require(row["parent"]["state"] == row["details"]["state"] == "sent"
                      and event["thread_ts"] == row["parent"]["receipt"]["ts"])
            with locked(store) as value:
                d.require(value["binding"] == pin and value["events"][key]["envelope"] == event
                          and value["events"][key]["alert"] == entry["alert"])
                if value["events"][key]["drained"]:
                    continue
                if row["cancelled"]:
                    d.require(row["reason"] == "cancelled")
                    value["events"][key].update(drained=True, disposition=dict(kind="cancelled",
                        alert_digest=d.digest(row), event_digest=d.digest(event)))
                    store.write("transport", value)
                    count += 1
                    continue  # Retain evidence; never admit a cancelled event to offline interpretation.
            borrowed = copy.copy(bounded)
            borrowed.lock = nullcontext  # r.intake borrows the store lock still held above, not the transport lock.
            r.intake(borrowed, entry["alert"], d.canonical(event), lambda raw: copy.deepcopy(event))
            with locked(store) as value:
                d.require(value["events"][key]["envelope"] == event)
                value["events"][key]["drained"] = True
                store.write("transport", value)
        count += 1
    return count

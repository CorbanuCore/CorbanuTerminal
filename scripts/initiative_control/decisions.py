"""Offline manager fixtures only; no caller, operational authority or delivery."""
import datetime as dt
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import tempfile
import unicodedata

MAX_BYTES = 1024 * 1024
FRESH_SECONDS = 20 * 60
ID = r"[A-Za-z0-9][A-Za-z0-9_-]{0,79}"
SECRET = re.compile(r"(?i)(bearer\s+\S+|(?:password|api[_ -]?key|(?:access|refresh)[_ -]?token|secret)\s*[:=]|-----BEGIN .*PRIVATE KEY|\b(?:sk-|ghp_|github_pat_|xox[baprs]-)\S+|synthetic[_ -]secret|raw[_ -]private[_ -]log)")
CONTEXT = "owner background impact stopped continuing recommendation".split()
FIELDS = set("revision initiative sprints raised_at updated_at summary question options evidence status resolution".split() + CONTEXT)


class Invalid(ValueError):
    def __init__(self):
        super().__init__("Decision input unavailable: invalid fixture.")


class Conflict(ValueError):
    def __init__(self):
        super().__init__("Decision fixture conflict; reload required.")


def require(condition):
    if not condition:
        raise Invalid()


def shape(value, keys):
    require(type(value) is dict and set(value) == set(keys.split() if isinstance(keys, str) else keys))


def text(value, optional=False, limit=2000):
    if value is None and optional:
        return
    require(type(value) is str and 0 < len(value.encode("utf-8")) <= limit)
    require(bool(value.strip()) and not SECRET.search(value))
    require(not any(unicodedata.category(c).startswith("C") for c in value))


def stamp(value):
    require(type(value) is str and re.fullmatch(r"\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ", value))
    try:
        return dt.datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=dt.timezone.utc)
    except ValueError:
        raise Invalid() from None


def path_ok(value):
    text(value)
    require(re.fullmatch(r"(?:docs/(?:plans|sprints)|qa)/[A-Za-z0-9_/-]+\.md", value))
    require(all(part not in ("", ".", "..") for part in value.split("/")))


def pairs(items):
    result = {}
    for key, value in items:
        require(key not in result)
        result[key] = value
    return result


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False).encode("utf-8")


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def validate(raw, now):
    """Return a detached complete record; errors never include submitted content."""
    try:
        payload = raw if isinstance(raw, bytes) else canonical(raw)
        require(len(payload) <= MAX_BYTES)
        feed = json.loads(payload, object_pairs_hook=pairs)
        require(len(canonical(feed)) <= MAX_BYTES)
        shape(feed, "schema feed_id revision assessed_at decisions")
        require(type(feed["schema"]) is int and feed["schema"] == 1)
        text(feed["feed_id"])
        require(re.fullmatch(ID, feed["feed_id"]))
        require(type(feed["revision"]) is int and feed["revision"] >= 1)
        assessed = stamp(feed["assessed_at"])
        require(assessed <= stamp(now))
        require(type(feed["decisions"]) is list)
        ids = set()
        for decision in feed["decisions"]:
            shape(decision, "id revisions")
            text(decision["id"])
            require(re.fullmatch(ID, decision["id"]) and decision["id"] not in ids)
            ids.add(decision["id"])
            history = decision["revisions"]
            require(type(history) is list and bool(history))
            for index, record in enumerate(history, 1):
                shape(record, FIELDS)
                require(type(record["revision"]) is int and record["revision"] == index)
                text(record["initiative"])
                require(re.fullmatch(r"PF-\d{2}", record["initiative"]))
                require(stamp(record["raised_at"]) <= stamp(record["updated_at"]) <= assessed)
                for field in CONTEXT:
                    text(record[field], optional=True)
                text(record["summary"], limit=300)
                text(record["question"])
                require(type(record["options"]) is list and len(record["options"]) >= 2)
                for option in record["options"]:
                    text(option)
                require(len(set(record["options"])) == len(record["options"]))
                require(type(record["sprints"]) is list and bool(record["sprints"]))
                refs = set()
                for ref in record["sprints"]:
                    shape(ref, "sprint_id path historical")
                    text(ref["sprint_id"])
                    require(re.fullmatch(r"PF-\d{2}-S\d{2}", ref["sprint_id"]))
                    require(type(ref["historical"]) is bool and ref["sprint_id"] not in refs)
                    refs.add(ref["sprint_id"])
                    require(not ref["historical"] or ref["sprint_id"] == "PF-76-S01")
                    if ref["path"] is not None:
                        path_ok(ref["path"])
                        require(ref["path"] == "docs/plans/delivery-history-reconciliation.md" if ref["historical"] else ref["path"].startswith("docs/sprints/"))
                require(type(record["evidence"]) is list)
                for item in record["evidence"]:
                    shape(item, "label path assessed_at")
                    text(item["label"])
                    if item["path"] is not None:
                        path_ok(item["path"])
                    if item["assessed_at"] is not None:
                        require(stamp(item["assessed_at"]) <= assessed)
                require(record["status"] in ("open", "acknowledged", "resolved", "superseded"))
                resolution = record["resolution"]
                if record["status"] in ("resolved", "superseded"):
                    shape(resolution, "answered_revision actor answer scope recorded_at")
                    answered = resolution["answered_revision"]
                    require(type(answered) is int and 1 <= answered < index)
                    target = history[answered - 1]
                    require(target["status"] in ("open", "acknowledged") and target["question"] == record["question"])
                    # An old answer survives acknowledgment, not changed/reopened context.
                    context = FIELDS - {"revision", "updated_at", "status", "resolution"}
                    require(all(r["status"] in ("open", "acknowledged")
                                and all(r[k] == record[k] for k in context)
                                for r in history[answered - 1:index - 1]))
                    require(stamp(target["updated_at"]) <= stamp(resolution["recorded_at"]) <= stamp(record["updated_at"]))
                    for field in ("actor", "answer", "scope"):
                        text(resolution[field])
                else:
                    require(resolution is None)
                if index > 1:
                    previous = history[index - 2]
                    require(record["raised_at"] == previous["raised_at"] and record["initiative"] == previous["initiative"])
                    require(stamp(record["updated_at"]) >= stamp(previous["updated_at"]))
                    if record["status"] != "open":
                        require(all(record[k] == previous[k] for k in FIELDS - {"revision", "updated_at", "status", "resolution"}))
                else:
                    require(record["status"] == "open")
        return feed
    except (ValueError, TypeError, KeyError, OverflowError, RecursionError, UnicodeError):
        raise Invalid() from None


def project(raw, now):
    """Missing/invalid is unknown; stale records remain inspectable, never refreshed."""
    try:
        feed = validate(raw, now)
    except Invalid:
        return {"state": "unknown", "feed": None, "open": None}
    return {"state": "stale" if (stamp(now) - stamp(feed["assessed_at"])).total_seconds() > FRESH_SECONDS else "fresh",
            "feed": feed, "open": [d["id"] for d in feed["decisions"] if d["revisions"][-1]["status"] in ("open", "acknowledged")]}


def advance(old, new):
    require(new["feed_id"] == old["feed_id"] and new["revision"] == old["revision"] + 1)
    require(stamp(new["assessed_at"]) >= stamp(old["assessed_at"]))
    latest = {d["id"]: d["revisions"] for d in new["decisions"]}
    for decision in old["decisions"]:
        history = decision["revisions"]
        require(latest.get(decision["id"], [])[:len(history)] == history)


def owner_only(info, directory=False):
    require((stat.S_ISDIR(info.st_mode) if directory else stat.S_ISREG(info.st_mode)))
    require(info.st_uid == os.getuid() and info.st_mode & 0o077 == 0)
    if not directory:
        require(info.st_nlink == 1)


def fixture_root(directory):
    root = Path(directory).absolute()
    require(root == root.resolve(strict=True))
    owner_only(root.stat(), directory=True)
    return root


def load_fixture(directory, now):
    try:
        root = fixture_root(directory)
        try:
            fd = os.open(root / "decisions.fixture.json", os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        except FileNotFoundError:
            return None
        with os.fdopen(fd, "rb") as stream:
            owner_only(os.fstat(stream.fileno()))
            return validate(stream.read(MAX_BYTES + 1), now)
    except (OSError, ValueError, RuntimeError):
        raise Invalid() from None


def save_fixture(directory, raw, expected_digest, now):
    """Caller supplies an existing private fixture directory and prior digest.

    Lock serializes cooperating writers; fsync/replace is not power-loss proof.
    A post-replace failure has uncertain outcome: reload before retrying.
    """
    pending = None
    try:
        new = validate(raw, now)
        root = fixture_root(directory)
        fd = os.open(root / ".decisions.fixture.lock", os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK, 0o600)
        with os.fdopen(fd, "a") as lock:
            owner_only(os.fstat(lock.fileno()))
            fcntl.flock(lock, fcntl.LOCK_EX)
            old = load_fixture(root, now)
            if old == new:  # Lost return/restart retry preserves assessment and identity.
                return digest(old)
            if (digest(old) if old is not None else None) != expected_digest:
                raise Conflict()
            if old is None:
                require(new["revision"] == 1)
            else:
                advance(old, new)
            fd, pending = tempfile.mkstemp(prefix=".decisions-pending-", dir=root)
            with os.fdopen(fd, "wb") as stream:
                stream.write(canonical(new))
                stream.flush()
                os.fsync(stream.fileno())
            os.replace(pending, root / "decisions.fixture.json")
            pending = None
            directory_fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY)
            try:
                os.fsync(directory_fd)
            finally:
                os.close(directory_fd)
            return digest(new)
    except Conflict:
        raise
    except (OSError, ValueError, RuntimeError):
        raise Invalid() from None
    finally:
        if pending is not None:
            os.unlink(pending)

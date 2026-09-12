"""Fixed, bounded decision snapshot transport; no state authoring or delivery."""
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import stat

import decisions as d

ARTIFACT = "decision-feed.json"
LIMIT = d.MAX_BYTES + 128  # Envelope around a maximum-sized canonical feed.


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
    raw = d.canonical(snapshot)
    return raw, record(snapshot, sha(raw), d.digest(observed))


def record(snapshot, digest, input_digest):
    feed = snapshot["feed"] or {}
    return dict(schema=1, artifact=ARTIFACT, digest=digest, input_digest=input_digest,
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
        d.require(type(pin["schema"]) is int and pin["schema"] == 1 and pin["artifact"] == ARTIFACT)
        d.require(all(type(pin[k]) is str and re.fullmatch(r"[a-f0-9]{64}", pin[k]) for k in ("digest", "input_digest")))
        # Resolve only the source root (the installed generation is a symlink).
        fd = os.open(Path(repo).resolve(strict=True) / ARTIFACT, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as stream:
            d.require(stat.S_ISREG(os.fstat(stream.fileno()).st_mode))
            raw = stream.read(LIMIT + 1)
        d.require(len(raw) <= LIMIT and sha(raw) == pin["digest"])
        snapshot = json.loads(raw, object_pairs_hook=d.pairs)
        d.shape(snapshot, "schema status feed")
        d.require(type(snapshot["schema"]) is int and snapshot["schema"] == 1)
        d.require(snapshot["status"] in ("valid", "missing", "invalid"))
        if snapshot["status"] == "valid":
            snapshot["feed"] = d.validate(snapshot["feed"], clock(at))
        else:
            d.require(snapshot["feed"] is None)
        d.require(raw == d.canonical(snapshot))
        d.require(d.canonical(record(snapshot, sha(raw), pin["input_digest"])) == d.canonical(pin))
        return snapshot
    except (OSError, ValueError, TypeError, KeyError, RuntimeError, RecursionError):
        raise ValueError("Decision feed transfer unavailable or invalid; resync required.") from None


def health(snapshot, manifest, at):
    view = d.project(snapshot["feed"], clock(at))
    feed = view["feed"] or {}
    return dict(state=view["state"], input_status=snapshot["status"],
                feed_id=feed.get("feed_id"), revision=feed.get("revision"),
                assessed_at=feed.get("assessed_at"),
                digest=manifest.get("decision_feed", {}).get("digest"),
                payload_digest=d.digest(feed) if feed else None,
                open_count=None if view["open"] is None else len(view["open"]),
                fresh_seconds=d.FRESH_SECONDS)


def render(snapshot, at, sprints, documents):
    from attention import render_decisions
    feed = snapshot["feed"]
    assessed = feed["assessed_at"] if feed else ""
    body = render_decisions(feed, clock(at), sprints, documents)
    return body.replace('<section id="decisions">',
                        f'<section id="decisions" class="notice attention" data-assessed-at="{assessed}" data-fresh-seconds="{d.FRESH_SECONDS}">', 1).replace('<details', '<details class="attention-item"')

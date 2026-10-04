"""Non-authoritative, complete safe siblings from an invalid decision input."""
from collections import Counter
import json

import decisions as d


def validate(feed, withheld_count, at):
    """Validate the inspection extension, never promote it to canonical input."""
    d.require(type(withheld_count) is int and 1 <= withheld_count <= d.MAX_BYTES)
    feed = d.validate(feed, at)
    d.require(bool(feed["decisions"]))
    return feed


def extract(raw, at):
    """Return exact valid siblings and a count; never return rejected content."""
    try:
        d.require(type(raw) is bytes and len(raw) <= d.MAX_BYTES)
        feed = json.loads(raw, object_pairs_hook=d.pairs)
        # Also rejects non-finite numbers and oversized canonical expansion anywhere.
        d.require(len(d.canonical(feed)) <= d.MAX_BYTES)
        d.shape(feed, "schema feed_id revision assessed_at decisions")
        d.require(type(feed["decisions"]) is list)
        envelope = d.validate({**feed, "decisions": []}, at)
        counts = Counter(item.get("id") for item in feed["decisions"]
                         if type(item) is dict and type(item.get("id")) is str)
        safe = []
        for item in feed["decisions"]:
            if type(item) is dict and type(item.get("id")) is str and counts[item["id"]] != 1:
                continue
            try:
                checked = d.validate({**envelope, "decisions": [item]}, at)
            except d.Invalid:
                continue
            safe.extend(checked["decisions"])
        withheld = len(feed["decisions"]) - len(safe)
        return validate({**envelope, "decisions": safe}, withheld, at), withheld
    except (ValueError, TypeError, KeyError, OverflowError, RecursionError):
        return None

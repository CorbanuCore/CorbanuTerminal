"""Assertions bound to driver input, including an in-product-process TZ control."""
from datetime import datetime
from pathlib import Path
import json
import re
from zoneinfo import ZoneInfo

COMPACT_HOUR = ("Precision unsupported outside retained raw detail; compacted days "
                "lost request/provider attribution. No bucket total.")

def assert_day(text, requested_day):
    days = set(re.findall(r"Requested UTC day: (\d{4}-\d{2}-\d{2})", text))
    assert days == {requested_day}, (requested_day, days)

def assert_compact_hour(text):
    # Selected rows and viewports wrap at different widths.
    normalized = " ".join(text.replace("›", " ").split())
    assert COMPACT_HOUR in normalized, normalized

def assert_timezone(path, pid, zone, utc):
    probe = json.loads(Path(str(path) + f".{pid}.json").read_text())
    stamp = datetime.fromisoformat(utc.replace("Z", "+00:00"))
    local = stamp.astimezone(ZoneInfo(zone))
    assert probe["pid"] == pid, (probe, pid)
    assert probe["tz"] == zone, probe
    assert probe["epoch"] == int(stamp.timestamp()), probe
    assert probe["local"] == local.strftime("%Y-%m-%dT%H:%M:%S%z"), (probe, local)
    return dict(**probe, expected_local=local.isoformat(), passed=True)

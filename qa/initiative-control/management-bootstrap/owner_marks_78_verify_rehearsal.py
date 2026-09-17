"""Compare the recorded dry-run plan with every actual coordinator table delta."""
import copy
import json
from pathlib import Path

import sys

path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("owner-marks-78-rehearsal-first.jsonl")
records = [json.loads(line) for line in path.read_text().splitlines() if line.startswith("{")]
assert records[-1]["kind"] == "PASS"
refusal = next(row for row in records if row["kind"] == "disarmed_interval_refused")
assert refusal["tick"]["refusal"] == "owner_off", refusal
preview = next(row["preview"] for row in records if row["kind"] == "dry_run_no_writes")
actual = next(row["changes"] for row in records if row["kind"] == "first_admitted_delta")
keys = dict(state="id", events="seq", audit="seq", action_history="id", evidence="digest",
            sqlite_sequence="name")


def mapped(table, rows):
    result = {}
    for row in rows:
        row = dict(row)
        if "body" in row:
            row["body"] = json.loads(row["body"])
        result[str(row[keys[table]])] = row
    return result


before, after = {}, {}
for change in actual:
    if change["path"][:2] == ["rows", "coordinator"]:
        table = change["path"][2]
        before[table] = mapped(table, change["before"])
        after[table] = mapped(table, change["after"])
expected = copy.deepcopy(before)
for change in preview["first_admitted_tick"]["coordinator_changes"]:
    destination = expected
    for key in change["path"][:-1]:
        destination = destination.setdefault(key, {})
    key = change["path"][-1]
    if change["operation"] == "delete":
        del destination[key]
    else:
        destination[key] = copy.deepcopy(change["after"])
# Prediction uses the observation time; real watchdog assigns new wall times.
for value in (expected, after):
    for row in value.get("audit", {}).values():
        if row["operation"] == "watchdog":
            row["at"] = "<actual tick time>"
            row["body"]["observed_at"] = "<actual tick time>"
assert expected == after, (expected, after)
arm = next(row["changes"] for row in records if row["kind"] == "arm_delta")
meta = next(change["after"][0] for change in arm if change["path"] == ["rows", "owner", "meta"])
assert meta == preview["arm"]["after"]
disarm = next(row["changes"] for row in records if row["kind"] == "disarm_delta")
row_changes = [change for change in disarm if change["path"][0] == "rows"]
assert len(row_changes) == 1 and row_changes[0]["path"] == ["rows", "owner", "meta"]
old, new = row_changes[0]["before"][0], row_changes[0]["after"][0]
assert new == dict(old, requested_mode="off", control_generation=2)
print(json.dumps(dict(
    result="PASS", comparisons=3,
    arm_meta="exact match", first_tick_coordinator="all changed tables match; only wall times normalized",
    coordinator_tables=sorted(after), disarm="only meta mode/generation changed",
    live_access=False), sort_keys=True))

"""Check frozen revision evidence; no live stores or credentials."""
import collections
import hashlib
import json
from pathlib import Path
import re

root = Path(__file__).parent
prefix = "owner-rejection-111-"
runs = {}
for label in ("targeted-1", "targeted-2", "targeted-3", "suite", "suite-final",
              "focused", "focused-final"):
    raw = (root / (prefix + label + ".txt")).read_text()
    summary = re.findall(r"^Ran (\d+) tests in ([0-9.]+)s$", raw, re.M)
    failures = re.findall(r"^(?:FAIL|ERROR): (.*)$", raw, re.M)
    success = bool(re.search(r"^OK$", raw, re.M))
    assert len(summary) == 1 and success and not failures, label
    modules = collections.Counter(re.findall(r"^test_\w+ \((test_\w+)\.", raw, re.M))
    count, seconds = summary[0]
    assert sum(modules.values()) == int(count), (label, modules, count)
    runs[label] = dict(tests=int(count), seconds=float(seconds),
                       failure_names=failures, passed=success, modules=dict(modules))

rows = [json.loads(line) for line in (root / (prefix + "observations-final.jsonl")).read_text().splitlines()]
assert len(rows) == 8
observations = {row["scenario"]: row for row in rows}
unknown = observations["unknown-history-without-hold"]
assert unknown["hold"] is None and unknown["health"]["state"] == "unknown"
assert "Quarantine history is incomplete" in unknown["rendered_notice"]
expired = observations["expired-nonheld-stale-projection"]
assert expired["health"]["state"] == "stale" and expired["outstanding"]["expired"] == 1
assert "A reply expired undelivered" in expired["rendered_notice"]
reviewed = observations["expired-reviewed-recovered"]
assert reviewed["hold"] is None and reviewed["outstanding"]["expired"] == 0
assert reviewed["retained_expired"] == 1
assert "A reply expired undelivered" not in reviewed["rendered_notice"]
for scenario in ("unknown", "expired", "auth", "scopes"):
    row = observations[scenario + "-reviewed-recovered"]
    assert row["health"]["state"] == "last-verified"
    assert "No transport action is needed" in row["rendered_notice"]
for label, phrase in (("auth", "repair the app credentials"), ("scopes", "restore the required app scopes")):
    row = observations[label + "-rejected"]
    assert row["hold"] == "qualification-" + label + "-rejected"
    assert row["health"]["state"] == "held" and phrase in row["rendered_notice"]
assert not (root / (prefix + "observations-final-stderr.txt")).read_text()
assert (root / (prefix + "renderer-final.txt")).read_text().strip() == "Facilities UI regression passed"

sources = ["attention.py", "decision_feed.py", "decision_manager.py",
           "slack_transport.py", "test_decision_manager.py", "test_slack_transport.py"]
hashes = {name: hashlib.sha256((Path("scripts/initiative_control") / name).read_bytes()).hexdigest()
          for name in sources}
print(json.dumps(dict(runs=runs, rendered_observations=len(rows), renderer_programs=1,
                     source_sha256=hashes), indent=2, sort_keys=True))

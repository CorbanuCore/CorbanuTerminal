"""Scratch-package predicate controls; synthetic coordinator state only."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

PREDICATE = 'return action is None or dispatcher is None or cls.dispatch_owner(state, action) in (None, dispatcher)'
PROBE = r'''
import json
from pathlib import Path
import sys
from unittest.mock import patch
from coordinator import Coordinator
from test_coordinator import seed
import owner_daemon as owner

root = Path(sys.argv[1])
c = Coordinator(root, clock=lambda: 1000.0)
c.initialize(*seed())
with c.mutation("synthetic-control", {}) as (_, state):
    state["dispatch_control"] = {"default": "hand"}
    state["manager"] = dict(id="manager", deadline=999)
    for key, side, status, deadline, reported in (
        ("owner-overdue", "owner", "dispatching", 999, False),
        ("hand-overdue", "hand", "running", 999, False),
        ("hand-prepared", "hand", "prepared", 999, False),
        ("hand-returned", "hand", "returned", 999, False),
        ("hand-accepted", "hand", "accepted", 999, False),
        ("owner-future", "owner", "running", 1001, False),
        ("owner-reported", "owner", "running", 999, True),
    ):
        state["actions"][key] = dict(id=key, dispatch_owner=side, status=status,
                                    deadline=deadline, stall_reported=reported, dispatch_epoch=1,
                                    workstream="delivery", sequence=1)
with patch.object(owner.time, "time", return_value=1000.0):
    before = owner.coordinator_activation_impact(root)
    after = owner.coordinator_activation_impact(root, "owner")
events = c.watchdog(dispatcher="owner")
assert set(after["watchdog_coverage"]["covered_actions"]) == {
    event["action"] for event in events if "action" in event}
assert after["manager"]["watchdog_will_report"] == any("manager" in event for event in events)
print(json.dumps(dict(before=before["watchdog_coverage"], after=after["watchdog_coverage"],
                     events=events), sort_keys=True))
'''


def main():
    source = Path(__file__).resolve().parents[3] / "scripts/initiative_control"
    base = Path(tempfile.mkdtemp(prefix="od89-control-", dir="/private/tmp"))
    variants = {
        "baseline": None,
        "include_hand": "return True",
        "exclude_manager": (
            "return action is not None and "
            "(dispatcher is None or cls.dispatch_owner(state, action) in (None, dispatcher))"),
    }
    results = {}
    summary_sha = hashlib.sha256((source / "owner_daemon.py").read_bytes()).hexdigest()
    for name, replacement in variants.items():
        package = base / name
        package.mkdir(mode=0o700)
        for path in source.glob("*.py"):
            shutil.copyfile(path, package / path.name)
        coordinator = package / "coordinator.py"
        original = coordinator.read_text()
        assert original.count(PREDICATE) == 1
        if replacement:
            coordinator.write_text(original.replace(PREDICATE, replacement))
        assert hashlib.sha256((package / "owner_daemon.py").read_bytes()).hexdigest() == summary_sha
        env = dict(PATH="/opt/homebrew/bin:/usr/bin:/bin", TMPDIR="/private/tmp",
                   PYTHONPATH=str(package), PYTHONDONTWRITEBYTECODE="1",
                   CORBANU_TEST_NO_NATIVE_KEYRING="1",
                   **{key: str(base) for key in
                      ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")})
        run = subprocess.run([sys.executable, "-B", "-c", PROBE, str(package / "state")],
                             env=env, cwd=package, capture_output=True, text=True, timeout=30)
        assert run.returncode == 0, (name, run.returncode, run.stderr)
        results[name] = json.loads(run.stdout)
        results[name]["coordinator_sha256"] = hashlib.sha256(coordinator.read_bytes()).hexdigest()
    baseline = results["baseline"]["after"]
    widened = results["include_hand"]["after"]
    denied = results["exclude_manager"]["after"]
    assert baseline["covered_actions"] == ["owner-overdue"]
    assert baseline["excluded_actions"] == ["hand-overdue"]
    assert widened["covered_actions"] == ["hand-overdue", "owner-overdue"]
    assert widened["excluded_actions"] == []
    assert not baseline["future_default_covered"] and widened["future_default_covered"]
    assert "manager responsibility" in baseline["summary"]
    assert "hand stall detection=covered by this watchdog" in widened["summary"]
    assert baseline["manager_covered"] and not denied["manager_covered"]
    assert "manager=excluded" in denied["summary"]
    print(json.dumps(dict(result="PASS", scratch=str(base), live_access=False,
                          unchanged_summary_sha256=summary_sha, variants=results),
                     indent=2, sort_keys=True))


if __name__ == "__main__":
    main()

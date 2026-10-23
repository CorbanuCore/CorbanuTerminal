"""Real launchd fixture qualification; retains raw evidence and always uninstalls."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "scripts/initiative_control"))
import activate
import fable_launcher as f
import owner_daemon as owner
from test_owner_daemon import OwnerDaemonTests


def main():
    fixture = OwnerDaemonTests()
    fixture.setUp()
    fixture.arm()
    root = Path(tempfile.mkdtemp(prefix="owner-recurrence-43-", dir="/private/tmp"))
    runtime, schedule, publish = (root / name for name in ("runtime", "schedule", "publish"))
    for path in (runtime, schedule, publish):
        path.mkdir(mode=0o700)
    for source in (REPO / "scripts/initiative_control").iterdir():
        if source.suffix == ".py" and not source.name.startswith("test_") or source.name.endswith(".plist.in"):
            target = runtime / source.name
            shutil.copyfile(source, target)
            target.chmod(0o600)
    measured = []
    for _ in range(10):
        start = time.monotonic()
        owner.Kernel(fixture.config_path).tick()
        measured.append(time.monotonic() - start)
    args = argparse.Namespace(owner="install", root=schedule,
        label="com.corbanu.initiative-owner.test-" + root.name.replace("_", "-"),
        python=Path(sys.executable).resolve(), runtime=runtime, config=fixture.config_path,
        interval=30, publish_state=publish, confirm_live=False)
    args.python_sha256 = f.file_digest(args.python)
    result = dict(root=str(root), fixture_root=str(fixture.root), label=args.label,
        interval=30, throttle=0, measured_kernel_seconds=measured,
        observed_worst_kernel_seconds=max(measured), firings=[], passed=False)
    def save():
        f.write_json(root / "qualification.json", result)
    print(json.dumps(result), flush=True)
    assert max(measured) * 5 < args.interval, "30 seconds lacks measured headroom"
    try:
        activate.owner_activation(args)
        deadline = time.monotonic() + 140
        seen = set()
        while time.monotonic() < deadline:
            for path in sorted((schedule / "ticks").glob("*.json")):
                if path.name in seen:
                    continue
                seen.add(path.name)
                status = owner.load(path)
                firing = dict(tick=status["ticks"], started_at=status["started_at"],
                    completed_at=status["completed_at"], hold=status["hold"],
                    duration=status["completed_at"] - status["started_at"],
                    utc=datetime.fromtimestamp(status["started_at"], timezone.utc).isoformat())
                result["firings"].append(firing)
                result["firings"].sort(key=lambda row: row["tick"])
                print(json.dumps(firing), flush=True)
                save()
            if len(result["firings"]) >= 4:
                break
            time.sleep(0.25)
        result["launchctl"] = owner.service(args.label)[1]
        result["projection"] = owner.load(publish / "owner-recurrence.json")
        from decision_feed import owner_health
        result["dashboard"] = owner_health(result["projection"], f.now())
        intervals = [b["started_at"] - a["started_at"]
                     for a, b in zip(result["firings"], result["firings"][1:])]
        result["observed_intervals"] = intervals
        result["passed"] = (len(intervals) >= 3 and all(25 <= gap <= 35 for gap in intervals)
                            and all(row["hold"] is None for row in result["firings"]))
        save()
    finally:
        args.owner = "uninstall"
        if (schedule / "installation.json").exists():
            activate.owner_activation(args)
            before = (schedule / "tick.json").read_bytes()
            time.sleep(2)
            result["uninstall_reversed"] = (owner.service(args.label)[0] == "absent"
                and not (schedule / "owner.plist").exists()
                and before == (schedule / "tick.json").read_bytes())
            result["uninstalled_projection"] = owner.load(publish / "owner-recurrence.json")
        save()
        fixture.tearDown()
    print(json.dumps(result), flush=True)
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())

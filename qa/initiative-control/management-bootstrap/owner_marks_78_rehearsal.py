"""Disposable, synthetic launchd rehearsal. Never reads a live owner root."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import time
from argparse import Namespace

import activate
from coordinator import Coordinator
import fable_launcher as f
import owner_daemon as owner
from test_coordinator import seed


def emit(kind, **value):
    print(json.dumps(dict(kind=kind, at=time.time(), **value), sort_keys=True), flush=True)


def files(root):
    return {str(p.relative_to(root)): dict(sha256=hashlib.sha256(p.read_bytes()).hexdigest(),
                                         bytes=p.stat().st_size, mtime_ns=p.stat().st_mtime_ns)
            for p in root.rglob("*") if p.is_file()}


def tables(root):
    result = {}
    for name in ("owner", "coordinator"):
        with sqlite3.connect((root / (name + ".sqlite3")).as_uri() + "?mode=ro", uri=True) as db:
            db.row_factory = sqlite3.Row
            result[name] = {}
            for (table,) in db.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"):
                result[name][table] = [dict(row) for row in db.execute('SELECT * FROM "' + table + '"')]
    return result


def main():
    base = Path(sys.argv[1]).resolve()
    assert base.parent == Path("/private/tmp") and base.name.startswith("owner-marks-78.")
    run = base / ("rehearsal-" + str(time.time_ns()))
    run.mkdir(mode=0o700)
    schedule = run / "corbanu-owner-live"
    state = run / "coordinator"
    runtime = schedule / "runtime"
    publish = run / "publish-state"
    worktree = run / "worktree"
    for path in (schedule, runtime, publish, worktree):
        path.mkdir(mode=0o700)
    source = Path(owner.__file__).parent
    for path in source.glob("*.py"):
        if not path.name.startswith("test_"):
            target = runtime / path.name
            shutil.copyfile(path, target)
            target.chmod(0o600)
    c = Coordinator(state)
    c.initialize(*seed())
    c.set_enabled(True, {"synthetic_rehearsal": True})
    with c.mutation("rehearsal", {"synthetic_only": True}) as (_, snapshot):
        for index, status in enumerate(("dispatching", "dispatched", "running")):
            snapshot["actions"][status] = dict(id=status, status=status, deadline=1.0,
                                               dispatch_epoch=1, workstream="delivery", sequence=index)
        snapshot["manager"] = dict(id="synthetic-hand-manager", deadline=1.0)
    config_path = schedule / "config.json"
    config = dict(coordinator=str(state), worktrees=[str(worktree)],
                  package_digest=owner.package_digest(), manager_enabled=True)
    f.write_json(config_path, config)
    owner.setup(config_path)
    authority_path = schedule / "decision.json"
    authority = dict(decision_id="owner-marks-78-synthetic-only", revision=1,
                     authority="Disposable rehearsal; NO live authority",
                     scope="fixture-only", generation=1, config_digest=owner.digest(config),
                     package_digest=owner.package_digest())
    f.write_json(authority_path, authority)
    python = Path(sys.executable).resolve()
    env = dict(PATH=f.SAFE_PATH, PYTHONDONTWRITEBYTECODE="1",
               **{key: str(run) for key in ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")})

    def cli(*args, expected=0):
        command = [str(python), "-E", "-s", "-S", "-B", str(runtime / "owner_daemon.py"), *args]
        result = subprocess.run(command, capture_output=True, text=True, env=env, timeout=30)
        emit("command", argv=command, exit=result.returncode, stdout=result.stdout, stderr=result.stderr)
        assert result.returncode == expected
        return json.loads(result.stdout)

    def wait_for(description, predicate, timeout=75):
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            if (schedule / "tick.json").exists():
                tick = owner.load(schedule / "tick.json")
                if predicate(tick):
                    emit(description, tick=tick)
                    return tick
            time.sleep(0.25)
        raise AssertionError("timeout: " + description)

    def capture():
        return dict(files=files(state), rows=tables(state))

    args = Namespace(owner="install", root=schedule,
                     label="com.corbanu.initiative-owner.test-marks78-" + str(os.getpid()),
                     python=python, python_sha256=f.file_digest(python), runtime=runtime,
                     config=config_path, interval=30, publish_state=publish, confirm_live=False,
                     domain="user")
    installed = False
    emit("layout", schedule=str(schedule), coordinator=str(state), runtime=str(runtime),
         publish_state=str(publish), label=args.label, domain="user/" + str(os.getuid()),
         package_digest=owner.package_digest())
    try:
        # With no scheduler running, compare the entire disposable layout byte-for-byte.
        before = files(run)
        preview = cli("--arm", "--dry-run", "--config", str(config_path), "--authority", str(authority_path))
        assert files(run) == before
        emit("dry_run_no_writes", identical_files=len(before), preview=preview)
        before = capture()
        activate.owner_activation(args)
        installed = True
        initial = wait_for("initial_off_latched_hold", lambda tick: tick["hold"] == "owner_run_refused")
        cli("--activation-status", "--config", str(config_path))
        cli("--arm", "--config", str(config_path), "--authority", str(authority_path))
        emit("arm_delta", changes=owner.preview_changes(before, capture()))
        armed = capture()
        held = wait_for("armed_interval_still_held",
                        lambda tick: tick["skipped"] > initial["skipped"])
        assert capture() == armed
        emit("held_coordinator_unchanged", files=len(armed["files"]), boots=0,
             schedule=owner.observe_schedule(schedule))
        cli("--schedule", str(schedule), "--recover", "Synthetic dry-run and arm verified; accept watchdog mutations")
        before = capture()
        first = wait_for("first_admitted_interval", lambda tick: tick["ticks"] > held["ticks"]
                         and tick["completed_at"] is not None and tick["hold"] is None)
        actual = capture()
        emit("first_admitted_delta", changes=owner.preview_changes(before, actual),
             schedule=owner.observe_schedule(schedule))
        assert c.snapshot()["actions"]["dispatching"]["status"] == "dispatch_uncertain"
        assert len(actual["rows"]["owner"]["boots"]) == 1
        assert len(actual["rows"]["owner"]["operations"]) == 0
        assert c.snapshot()["manager"]["stall_reported"]
        coordinator_bytes = (state / "coordinator.sqlite3").read_bytes()
        second = wait_for("second_admitted_interval", lambda tick: tick["ticks"] > first["ticks"]
                          and tick["completed_at"] is not None)
        assert (state / "coordinator.sqlite3").read_bytes() == coordinator_bytes
        emit("watchdog_not_repeated", boots=len(tables(state)["owner"]["boots"]))
        before = capture()
        activation_bytes = (state / "activation.json").read_bytes()
        cli("--disarm", "--config", str(config_path), "--generation", "1")
        assert (state / "activation.json").read_bytes() == activation_bytes
        assert (state / "coordinator.sqlite3").read_bytes() == coordinator_bytes
        emit("disarm_delta", changes=owner.preview_changes(before, capture()))
        stopped = capture()
        wait_for("disarmed_interval_refused", lambda tick: tick["ticks"] > second["ticks"]
                 and tick["hold"] == "owner_run_refused")
        assert capture() == stopped
        cli("--activation-status", "--config", str(config_path))
        emit("revocation_verified", state="OFF", generation=2, retained_activation=True,
             retained_watchdog_history=True, boots=len(tables(state)["owner"]["boots"]))
    finally:
        if installed or (schedule / "installation.json").exists():
            args.owner = "uninstall"
            activate.owner_activation(args)
            emit("uninstalled", service=owner.service(args.label, "user/" + str(os.getuid()))[0],
                 receipt=owner.load(schedule / "installation.json"))
    emit("PASS", artifacts=str(run))


if __name__ == "__main__":
    main()

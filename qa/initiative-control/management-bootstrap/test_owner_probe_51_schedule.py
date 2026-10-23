"""Disposable domain/session factorial and post-interval kickstart diagnostic."""
import json
import os
from pathlib import Path
import plistlib
import shutil
import sys
import tempfile
import time

from test_owner_recurrence_47_schedule import REPO, OwnerDaemonTests, command, f, owner

WRAPPER = """
import os, pathlib, subprocess, sys, time
sys.path.insert(0, sys.argv[1])
import owner_daemon as owner
import decision_feed
root, target = pathlib.Path(sys.argv[2]), sys.argv[3]
def command(*argv):
    p = subprocess.run(argv, capture_output=True, text=True, env={}, timeout=5)
    return dict(rc=p.returncode, out=p.stdout, err=p.stderr)
entry = dict(at=time.time(), pid=os.getpid(),
             service=command('/bin/launchctl', 'print', target),
             blame=command('/bin/launchctl', 'blame', target))
rc = owner.main(['--run', '--schedule', str(root)])
entry.update(tick=owner.load(root / 'tick.json'),
             health=decision_feed.owner_health(owner.observe_schedule(root), owner.f.now()))
owner.f.write_json(root / ('process-' + str(os.getpid()) + '.json'), entry)
sys.exit(rc)
"""


def main():
    root = Path(tempfile.mkdtemp(prefix="owner-probe-51-", dir="/private/tmp"))
    runtime = root / "runtime"
    runtime.mkdir(mode=0o700)
    for source in (REPO / "scripts/initiative_control").glob("*.py"):
        if not source.name.startswith("test_"):
            shutil.copyfile(source, runtime / source.name)
            (runtime / source.name).chmod(0o600)
    f.write_file(root / "probe.py", WRAPPER.encode())
    result = dict(root=str(root), start=time.time(), cases={}, observations=[])
    cases = []
    try:
        for kind in ("gui", "user"):
            for session in ("Aqua", "Background", None):
                name = kind + "-" + (session or "omitted")
                fixture = OwnerDaemonTests()
                fixture.setUp()
                fixture.arm()
                schedule = root / name
                schedule.mkdir(mode=0o700)
                label = "com.corbanu.initiative-owner.test-p51-" + name + "-" + root.name.rsplit("-", 1)[1]
                domain = f"{kind}/{os.getuid()}"
                target = domain + "/" + label
                python = Path(sys.executable).resolve()
                pins = owner.schedule_pins(python, runtime, fixture.config_path, f.file_digest(python))
                job = dict(Label=label, ProgramArguments=[str(python), "-E", "-s", "-S", "-B",
                    str(root / "probe.py"), str(runtime), str(schedule), target],
                    WorkingDirectory=str(runtime), RunAtLoad=False, KeepAlive=False,
                    StartInterval=30, ThrottleInterval=0, ExitTimeOut=30,
                    ProcessType="Background", AbandonProcessGroup=False, Umask=63,
                    StandardOutPath=str(schedule / "stdout"), StandardErrorPath=str(schedule / "stderr"),
                    EnvironmentVariables={key: str(schedule) for key in
                        ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")})
                if session:
                    job["LimitLoadToSessionType"] = session
                f.write_file(schedule / "owner.plist", plistlib.dumps(job))
                f.write_file(schedule / "tick.lock", b"")
                f.write_json(schedule / "tick.json", dict(started_at=None, completed_at=None,
                    last_success=None, hold=None, skipped=0, ticks=0, last_probe=None))
                f.write_json(schedule / "installation.json", dict(label=label, domain=domain,
                    pins=pins, interval=30, plist_sha256=f.file_digest(schedule / "owner.plist"),
                    publish_state=str(schedule), phase="installed"))
                cases.append((name, target, schedule, fixture))
                row = result["cases"][name] = dict(target=target, session=session,
                    bootstrap=command("/bin/launchctl", "bootstrap", domain, str(schedule / "owner.plist")))
                row["locations"] = {d: command("/bin/launchctl", "print", f"{d}/{os.getuid()}/{label}")
                                    for d in ("gui", "user")}
                if row["bootstrap"]["rc"] == 0:
                    row["kickstart"] = command("/bin/launchctl", "kickstart", target)
        started = time.monotonic()
        for number, delay in enumerate((2, 35, 70, 73, 106)):
            while time.monotonic() < started + delay:
                time.sleep(0.1)
            observation = dict(interval=number, at=time.time(),
                power=command("/usr/bin/pmset", "-g", "batt"),
                thermal=command("/usr/bin/pmset", "-g", "therm"),
                low_power=command("/usr/bin/pmset", "-g", "custom"), cases={},
                launchd_log=command("/usr/bin/log", "show", "--last", "35s", "--style", "compact",
                    "--info", "--debug", "--predicate",
                    'process == "launchd" AND eventMessage CONTAINS "initiative-owner.test-p51-"'),
                domains={d: command("/bin/launchctl", "print", f"{d}/{os.getuid()}")["out"]
                         .split("\n\tenvironment =")[0].split("\n\tservices =")[0] for d in ("gui", "user")})
            for name, target, schedule, _ in cases:
                observation["cases"][name] = dict(
                    service=command("/bin/launchctl", "print", target),
                    runs=sorted([owner.load(p) for p in schedule.glob("process-*.json")], key=lambda r: r["at"]),
                    observation=owner.observe_schedule(schedule))
            result["observations"].append(observation)
            f.write_json(root / "diagnostic.json", result)
            print(json.dumps(dict(root=str(root), sample=number, runs={
                n: [(r["tick"]["firing"], r["health"]["state"]) for r in v["runs"]]
                for n, v in observation["cases"].items()})), flush=True)
            if number == 2:
                row = observation["cases"]["user-Background"]
                if any(r["tick"]["firing"] == "interval" for r in row["runs"]):
                    target = result["cases"]["user-Background"]["target"]
                    result["post_interval_kickstart"] = dict(at=time.time(),
                        result=command("/bin/launchctl", "kickstart", target))
                else:
                    result["post_interval_kickstart"] = dict(blocked="no observed interval firing")
    finally:
        for name, target, schedule, fixture in cases:
            row = result["cases"][name]
            row["bootout"] = command("/bin/launchctl", "bootout", target)
            label = target.rsplit("/", 1)[1]
            row["after"] = {d: command("/bin/launchctl", "print", f"{d}/{os.getuid()}/{label}")
                            for d in ("gui", "user")}
            for d, observed in row["after"].items():
                if observed["rc"] == 0:
                    row["redirected_bootout"] = command("/bin/launchctl", "bootout", f"{d}/{os.getuid()}/{label}")
            fixture.tearDown()
        result["end"] = time.time()
        f.write_json(root / "diagnostic.json", result)
    print(str(root / "diagnostic.json"), flush=True)


if __name__ == "__main__":
    main()

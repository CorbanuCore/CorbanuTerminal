"""Disposable launchd diagnostic matrix; no live label, transport or credentials."""
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tempfile
import time

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "scripts/initiative_control"))
import fable_launcher as f
import owner_daemon as owner
from test_owner_daemon import OwnerDaemonTests

WRAPPER = """
import json, os, pathlib, subprocess, sys, time
sys.path.insert(0, sys.argv[1])
import owner_daemon as owner
root, target = pathlib.Path(sys.argv[2]), sys.argv[3]
def run(*argv):
    p = subprocess.run(argv, capture_output=True, text=True, env={}, timeout=5)
    return dict(rc=p.returncode, out=p.stdout, err=p.stderr)
entry = dict(at=time.time(), pid=os.getpid(), ppid=os.getppid(), pgid=os.getpgrp(),
             blame=run('/bin/launchctl', 'blame', target),
             service=run('/bin/launchctl', 'print', target))
children, real = [], subprocess.Popen
def tracked(*args, **kwargs):
    p = real(*args, **kwargs)
    children.append(p)
    return p
subprocess.Popen = tracked
start = time.monotonic()
try:
    rc = owner.main(['--run', '--schedule', str(root)])
finally:
    subprocess.Popen = real
    entry.update(duration=time.monotonic()-start, children=[
        dict(pid=p.pid, executable=os.path.basename(p.args[0]), returncode=p.poll()) for p in children])
    entry['exit_service'] = run('/bin/launchctl', 'print', target)
    owner.f.write_json(root / ('process-' + str(os.getpid()) + '.json'), entry)
sys.exit(rc)
"""


def command(*argv):
    try:
        p = subprocess.run(argv, capture_output=True, text=True, env={}, timeout=5)
        return dict(rc=p.returncode, out=p.stdout, err=p.stderr)
    except subprocess.TimeoutExpired:
        return dict(rc=None, out="", err="TimeoutExpired")


def main():
    root = Path(tempfile.mkdtemp(prefix="owner-recurrence-47-", dir="/private/tmp"))
    runtime = root / "runtime"
    runtime.mkdir(mode=0o700)
    for source in (REPO / "scripts/initiative_control").iterdir():
        if source.suffix == ".py" and not source.name.startswith("test_"):
            shutil.copyfile(source, runtime / source.name)
            (runtime / source.name).chmod(0o600)
    wrapper = root / "probe.py"
    wrapper.write_text(WRAPPER)
    result = dict(root=str(root), start=time.time(), cases={}, observations=[])
    cases, logger = [], None
    try:
        log = (root / "launchd.log").open("w")
        logger = subprocess.Popen(["/usr/bin/log", "stream", "--style", "ndjson", "--level", "debug",
            "--predicate", 'process == "launchd" AND eventMessage CONTAINS "initiative-owner.test-r47-"'],
            stdout=log, stderr=subprocess.STDOUT, env={})
        for name, domain, abandon, kind in [
                ("baseline", "gui", False, "Background"), ("abandon", "gui", True, "Background"),
                ("interactive", "gui", False, "Interactive"), ("user", "user", False, "Background")]:
            fixture = OwnerDaemonTests()
            fixture.setUp()
            fixture.arm()
            schedule = root / name
            schedule.mkdir(mode=0o700)
            label = "com.corbanu.initiative-owner.test-r47-" + name + "-" + root.name.rsplit("-", 1)[1]
            target = f"{domain}/{os.getuid()}/{label}"
            python = Path(sys.executable).resolve()
            pins = owner.schedule_pins(python, runtime, fixture.config_path, f.file_digest(python))
            job = dict(Label=label, ProgramArguments=[str(python), "-E", "-s", "-S", "-B",
                str(wrapper), str(runtime), str(schedule), target], WorkingDirectory=str(runtime),
                LimitLoadToSessionType="Background" if domain == "user" else "Aqua",
                RunAtLoad=False, KeepAlive=False, StartInterval=30, ThrottleInterval=0,
                ExitTimeOut=30, ProcessType=kind, AbandonProcessGroup=abandon, Umask=63,
                StandardOutPath=str(schedule / "stdout"), StandardErrorPath=str(schedule / "stderr"),
                EnvironmentVariables={key: str(schedule) for key in
                    ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")})
            f.write_file(schedule / "owner.plist", plistlib.dumps(job))
            f.write_file(schedule / "tick.lock", b"")
            f.write_json(schedule / "tick.json", dict(started_at=None, completed_at=None,
                last_success=None, hold=None, skipped=0, ticks=0, last_probe=None))
            f.write_json(schedule / "installation.json", dict(label=label, pins=pins, interval=30,
                plist_sha256=f.file_digest(schedule / "owner.plist"), publish_state=str(schedule),
                phase="installed"))
            cases.append((name, target, schedule, fixture))
            result["cases"][name] = dict(target=target, abandon=abandon, process_type=kind,
                bootstrap=command("/bin/launchctl", "bootstrap", f"{domain}/{os.getuid()}",
                                  str(schedule / "owner.plist")))
            result["cases"][name]["kickstart"] = command("/bin/launchctl", "kickstart", target)
        started = time.monotonic()
        for number in range(6):
            while time.monotonic() < started + number * 30 + 2:
                time.sleep(0.1)
            processes = command("/bin/ps", "-axo", "pid=,ppid=,pgid=,stat=,comm=")
            rows = [line.split(None, 4) for line in processes["out"].splitlines()]
            observation = dict(interval=number, at=time.time(), power=command("/usr/bin/pmset", "-g", "batt"),
                thermal=command("/usr/bin/pmset", "-g", "therm"),
                low_power=command("/usr/bin/pmset", "-g", "custom"), cases={},
                launchd_log=command("/usr/bin/log", "show", "--last", "35s", "--style", "compact",
                    "--info", "--debug", "--predicate",
                    'process == "launchd" AND eventMessage CONTAINS "initiative-owner.test-r47-"'),
                domains={domain: command("/bin/launchctl", "print", f"{domain}/{os.getuid()}")["out"].split("\n\tenvironment =")[0].split("\n\tservices =")[0]
                         for domain in ("gui", "user")})
            for name, target, schedule, _ in cases:
                runs = [owner.load(p) for p in sorted(schedule.glob("process-*.json"))]
                pids = {r["pid"] for r in runs} | {c["pid"] for r in runs for c in r["children"]}
                groups = {r["pgid"] for r in runs}
                observation["cases"][name] = dict(service=command("/bin/launchctl", "print", target),
                    blame=command("/bin/launchctl", "blame", target), runs=runs,
                    surviving_processes=[r for r in rows if int(r[0]) in pids or int(r[2]) in groups])
            if number == 1:
                result["second_manual_kickstart"] = command("/bin/launchctl", "kickstart", cases[0][1])
            result["observations"].append(observation)
            f.write_json(root / "diagnostic.json", result)
            print(json.dumps(dict(root=str(root), interval=number,
                runs={n: len(v["runs"]) for n, v in observation["cases"].items()},
                power=observation["power"], thermal=observation["thermal"])), flush=True)
    finally:
        for name, target, schedule, fixture in cases:
            result["cases"][name]["bootout"] = command("/bin/launchctl", "bootout", target)
            result["cases"][name]["after"] = command("/bin/launchctl", "print", target)
            fixture.tearDown()
        if logger:
            logger.terminate()
            result["log_exit"] = logger.wait(timeout=5)
            log.close()
        result["end"] = time.time()
        f.write_json(root / "diagnostic.json", result)
    print(str(root / "diagnostic.json"), flush=True)


if __name__ == "__main__":
    main()

"""Real launchd ownership trials with inert disposable labels and synthetic receipts."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import plistlib
import subprocess
import sys
import tempfile

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "scripts/initiative_control"))
import activate
import fable_launcher as f
import owner_daemon as owner


def main():
    root = Path(tempfile.mkdtemp(prefix="owner-ownership-53-", dir="/private/tmp"))
    targets = []
    events = []

    def command(*argv):
        result = subprocess.run(argv, env={}, capture_output=True, text=True, timeout=40)
        # Keep only job identity/state lines; never export inherited environment.
        out = result.stdout
        if argv[1] == "print":
            out = "\n".join(line for line in out.splitlines()
                            if line.startswith(argv[2] + " =") or
                            line.strip().startswith(("path =", "state =")))
        row = dict(argv=list(argv), rc=result.returncode, stdout=out, stderr=result.stderr)
        events.append(row)
        print(json.dumps(row), flush=True)
        return result.returncode

    def job(path, label, session):
        f.write_file(path, plistlib.dumps(dict(
            Label=label, ProgramArguments=["/usr/bin/true"], RunAtLoad=False,
            KeepAlive=False, LimitLoadToSessionType=session,
            EnvironmentVariables={key: str(root) for key in
                                  ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")})))

    def bootstrap(kind, label, path, optional=False):
        target = f"{kind}/{os.getuid()}/{label}"
        targets.append(target)
        rc = command("/bin/launchctl", "bootstrap", f"{kind}/{os.getuid()}", str(path))
        if optional and rc != 0:
            assert command("/bin/launchctl", "print", target) == 113
            return False
        assert rc == 0
        assert command("/bin/launchctl", "print", target) == 0
        return True

    def receipt(schedule, kind, label):
        f.write_json(schedule / "installation.json", dict(
            label=label, domain=f"{kind}/{os.getuid()}", phase="installed",
            plist_sha256=f.file_digest(schedule / "owner.plist"),
            publish_state=str(schedule), interval=30))
        f.write_json(schedule / "tick.json", dict(
            started_at=None, completed_at=None, last_success=None, hold=None))

    def uninstall(schedule, expected=None):
        args = argparse.Namespace(owner="uninstall", root=schedule, label=None)
        try:
            activate.owner_activation(args)
        except f.LaunchError as exc:
            assert expected is not None and str(exc) == expected, str(exc)
            outcome = str(exc)
        else:
            assert expected is None, expected
            outcome = "uninstalled"
        row = dict(uninstall=str(schedule), result=outcome)
        events.append(row)
        print(json.dumps(row), flush=True)

    try:
        for kind, opposite in (("gui", "user"), ("user", "gui")):
            label = f"com.corbanu.initiative-owner.test-o53-{kind}-{root.name.rsplit('-', 1)[1]}"
            schedule = root / kind
            schedule.mkdir(mode=0o700)
            job(schedule / "owner.plist", label, "Aqua" if kind == "gui" else "Background")
            bootstrap(kind, label, schedule / "owner.plist")
            assert command("/bin/launchctl", "print", f"{opposite}/{os.getuid()}/{label}") == 113
            incoming = root / (kind + "-incoming")
            incoming.mkdir(mode=0o700)
            argv = [sys.executable, "-B", str(REPO / "scripts/initiative_control/activate.py"),
                    "--owner", "install", "--root", str(incoming), "--label", label,
                    "--domain", opposite, "--publish-state", str(incoming)]
            env = {key: str(root) for key in
                   ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")}
            result = subprocess.run(argv, env=env, capture_output=True, text=True, timeout=20)
            refusal = f"service_conflict: {kind}/{os.getuid()}/{label}"
            assert result.returncode != 0 and refusal in result.stderr, result.stderr
            assert sorted(p.name for p in incoming.iterdir()) == ["installation.lock"]
            row = dict(install_argv=argv, rc=result.returncode,
                       stdout=result.stdout, stderr=result.stderr)
            events.append(row)
            print(json.dumps(row), flush=True)
            assert command("/bin/launchctl", "print", f"{opposite}/{os.getuid()}/{label}") == 113
            # Test whether launchd permits the alleged duplicate; do not assume it.
            job(schedule / "duplicate.plist", label, "Aqua" if opposite == "gui" else "Background")
            duplicated = bootstrap(opposite, label, schedule / "duplicate.plist", optional=True)
            receipt(schedule, kind, label)
            if not duplicated:
                # Positive control: the identical alternate plist must load after
                # the first label is removed, establishing fixture validity.
                assert command("/bin/launchctl", "bootout", f"{kind}/{os.getuid()}/{label}") == 0
                bootstrap(opposite, label, schedule / "duplicate.plist")
            uninstall(schedule, f"unowned_service: {opposite}/{os.getuid()}/{label}")
            assert command("/bin/launchctl", "print", f"{opposite}/{os.getuid()}/{label}") == 0
            if duplicated:
                assert command("/bin/launchctl", "print", f"{kind}/{os.getuid()}/{label}") == 0
            assert command("/bin/launchctl", "bootout", f"{opposite}/{os.getuid()}/{label}") == 0
            uninstall(schedule)
            for location in (kind, opposite):
                assert command("/bin/launchctl", "print", f"{location}/{os.getuid()}/{label}") == 113

        # Same receipted path/hash moved or duplicated across domains.
        schedule = root / "shared"
        schedule.mkdir(mode=0o700)
        label = f"com.corbanu.initiative-owner.test-o53-shared-{root.name.rsplit('-', 1)[1]}"
        job(schedule / "owner.plist", label, ["Aqua", "Background"])
        bootstrap("gui", label, schedule / "owner.plist")
        receipt(schedule, "gui", label)
        if not bootstrap("user", label, schedule / "owner.plist", optional=True):
            assert command("/bin/launchctl", "bootout", f"gui/{os.getuid()}/{label}") == 0
            bootstrap("user", label, schedule / "owner.plist")
        uninstall(schedule)
        assert owner.load(schedule / "installation.json")["phase"] == "uninstalled"
        for kind in ("gui", "user"):
            assert command("/bin/launchctl", "print", f"{kind}/{os.getuid()}/{label}") == 113
    finally:
        for target in targets:
            if command("/bin/launchctl", "print", target) == 0:
                command("/bin/launchctl", "bootout", target)
            assert command("/bin/launchctl", "print", target) == 113
        record = root / "trials.json"
        record.write_text(json.dumps(events, indent=2) + "\n")
        print(f"artifact={record} sha256={hashlib.sha256(record.read_bytes()).hexdigest()}", flush=True)


if __name__ == "__main__":
    main()

"""Disposable launchd trials; absent-domain probes do not emulate headless sessions."""
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
from test_owner_daemon import RecurrenceTests


def main():
    evidence = Path(tempfile.mkdtemp(prefix="owner-ownership-56-trials-", dir="/private/tmp"))
    events, targets = [], []
    fixture = RecurrenceTests()
    fixture.setUp()

    def record(**row):
        events.append(row)
        print(json.dumps(row), flush=True)

    def command(*argv):
        result = subprocess.run(argv, capture_output=True, text=True, env={}, timeout=40)
        output = result.stdout
        if argv[1] == "print":
            output = "\n".join(line for line in output.splitlines()
                               if line.startswith(argv[2] + " =") or
                               line.strip().startswith(("path =", "state =")))
        record(argv=list(argv), rc=result.returncode, stdout=output, stderr=result.stderr)
        return result

    def probe(target):
        return command("/bin/launchctl", "print", target)

    try:
        args = fixture.installation()
        # The real installer pins this harmless runtime: it cannot enter the
        # coordinator, access profiles, make inference or dispatch any work.
        f.write_file(args.runtime / "owner_daemon.py", b"pass\n")
        suffix = evidence.name.rsplit("-", 1)[1]
        for kind, sibling in (("user", "gui"), ("gui", "user")):
            args.domain = kind
            args.label = f"com.corbanu.initiative-owner.test-o56-{kind}-{suffix}"
            selected = f"{kind}/{os.getuid()}/{args.label}"
            other = f"{sibling}/{os.getuid()}/{args.label}"
            targets.extend((selected, other))
            args.root = evidence / kind
            args.root.mkdir(mode=0o700)
            foreign = evidence / f"{sibling}.plist"
            f.write_file(foreign, plistlib.dumps(dict(
                Label=args.label, ProgramArguments=["/usr/bin/true"],
                RunAtLoad=False, KeepAlive=False,
                LimitLoadToSessionType="Aqua" if sibling == "gui" else "Background")))
            assert probe(selected).returncode == 113
            assert probe(other).returncode == 113
            assert command("/bin/launchctl", "bootstrap", f"{sibling}/{os.getuid()}",
                           str(foreign)).returncode == 0
            assert probe(other).returncode == 0
            args.owner = "install"
            try:
                activate.owner_activation(args)
            except f.LaunchError as exc:
                record(case="sibling-present-conflicting", selected=kind, error=str(exc))
                assert str(exc) == f"service_conflict: {other}"
            else:
                raise AssertionError("conflicting install succeeded")
            assert sorted(p.name for p in args.root.iterdir()) == ["installation.lock"]
            assert probe(selected).returncode == 113
            assert command("/bin/launchctl", "bootout", other).returncode == 0
            assert probe(other).returncode == 113
            activate.owner_activation(args)
            assert probe(selected).returncode == 0
            installed = owner.load(args.root / "installation.json")
            assert installed["phase"] == "installed"
            assert installed["sibling_observation"]["state"] == "absent"
            record(case="sibling-present-clear", selected=kind, phase=installed["phase"],
                   sibling_observation=installed["sibling_observation"])
            args.owner = "uninstall"
            activate.owner_activation(args)
            assert probe(selected).returncode == 113
            assert probe(other).returncode == 113
            record(case="uninstall", selected=kind,
                   phase=owner.load(args.root / "installation.json")["phase"])
        # Read-only probes at an unused UID; do not create/remove login domains.
        # Neither is an end-to-end current-UID install with a missing sibling.
        for kind in ("gui", "user"):
            result = probe(f"{kind}/2147483646/com.corbanu.initiative-owner.test-o56-absent")
            record(case="absent-domain-probe", kind=kind, rc=result.returncode,
                   limitation="different UID; no current-UID headless install claimed")
        (evidence / "bad-label").mkdir(mode=0o700)
        result = command(sys.executable, "-B", str(REPO / "scripts/initiative_control/activate.py"),
                         "--owner", "install", "--root", str(evidence / "bad-label"),
                         "--publish-state", str(args.publish_state), "--label", "bad-label")
        assert result.returncode == 1 and "LaunchError: invalid_label" in result.stderr
    finally:
        for target in targets:
            if probe(target).returncode == 0:
                command("/bin/launchctl", "bootout", target)
            assert probe(target).returncode == 113
        fixture.tearDown()
        path = evidence / "trials.json"
        path.write_text(json.dumps(events, indent=2) + "\n")
        print(f"artifact={path}", flush=True)


if __name__ == "__main__":
    main()

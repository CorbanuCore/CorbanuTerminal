#!/usr/bin/env python3
"""Install a verified source export into the dedicated user-owned Linux service."""
import argparse
import hashlib
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import uuid

from control import allowed_host, atomic_json, checked_run, locked, now, read_file, read_json, report
import decision_feed


def activate(root, incoming, units):
    root, incoming = root.resolve(), incoming.resolve()
    if root.name != "corbanu-control" or incoming.parent != root / "incoming":
        raise ValueError("installation must use the dedicated corbanu-control/incoming directory")
    if not re.fullmatch(r"upload-[a-f0-9]{32}", incoming.name):
        raise ValueError("invalid generation name")
    manifest = read_json(incoming / "state/source.json", incoming)
    decision_feed.read_snapshot(incoming / "source", manifest, now())
    for path, expected in manifest["files"].items():
        actual = hashlib.sha256(read_file(incoming / "source" / path, incoming / "source").encode()).hexdigest()
        if actual != expected:
            raise ValueError("source transfer checksum mismatch")
    state = root / "state"
    config = read_json(incoming / "state/control.json", incoming)
    events = [checked_run(read_json(path, incoming)) for path in (incoming / "state/events").glob("*.json")]
    with locked(root / ".sync.lock"):
        state.mkdir(exist_ok=True, mode=0o700)
        for event in events:
            report(state, event)
        pointer = root / (".source-" + uuid.uuid4().hex)
        pointer.symlink_to(incoming / "source")
        os.replace(pointer, root / "source")
        atomic_json(state / "control.json", config)
        atomic_json(state / "source.json", manifest)
        (root / "private").mkdir(exist_ok=True, mode=0o700)
        units.mkdir(parents=True, exist_ok=True)
        common = "[Unit]\nDescription=Corbanu private initiative control\n\n[Service]\nUMask=0077\nNoNewPrivileges=true\n"
        # Optional operator opt-in: exact extra Host names the read-only web
        # service accepts (a private tailnet name). Loopback is always accepted.
        names = config.get("web_allowed_hosts", [])
        if not isinstance(names, list) or len(names) > 4:
            raise ValueError("web_allowed_hosts must be a short list of exact DNS names")
        web_hosts = "".join(f" --allow-host {allowed_host(name)}" for name in names)
        contents = {
            "corbanu-control-publish.service": common + f'Type=oneshot\nExecStart=/usr/bin/flock {root}/.sync.lock "{sys.executable}" {root}/source/scripts/initiative_control/tick.py --root {root}\nTimeoutStartSec=10min\n',
            "corbanu-control-publish.timer": "[Unit]\nDescription=Refresh Corbanu initiative dashboard every 30 minutes\n[Timer]\nOnBootSec=2min\nOnUnitActiveSec=30min\nPersistent=true\n[Install]\nWantedBy=timers.target\n",
            "corbanu-control-web.service": common + f'Type=simple\nExecStart="{sys.executable}" {root}/source/scripts/initiative_control/control.py serve --output {root}/site --port 8768{web_hosts}\nRestart=on-failure\nRestartSec=5\n[Install]\nWantedBy=default.target\n',
        }
        for name, content in contents.items():
            path = units / name
            if path.exists() and "Corbanu" not in path.read_text():
                raise ValueError("refusing to overwrite an unrelated service")
            path.write_text(content)
        # Only source exports created by this installer; keep three for rollback.
        candidates = sorted((p for p in (root / "incoming").iterdir()
                             if re.fullmatch(r"upload-[a-f0-9]{32}", p.name) and p.is_dir() and not p.is_symlink()),
                            key=lambda p: p.stat().st_mtime, reverse=True)
        for old in candidates[3:]:
            # Partial uploads may be in flight: only prune those older than one day.
            complete = (old / "state/source.json").is_file()
            if old != incoming and (complete or time.time() - old.stat().st_mtime > 86400):
                shutil.rmtree(old)
    subprocess.run(["systemctl", "--user", "daemon-reload"], check=True)
    subprocess.run(["systemctl", "--user", "start", "corbanu-control-publish.service"], check=True)
    # Publication is not scheduling authority. Preserve the operator's timer
    # activation state, including a paused or never-enabled installation.
    subprocess.run(["systemctl", "--user", "enable", "corbanu-control-web.service"], check=True)
    subprocess.run(["systemctl", "--user", "restart", "corbanu-control-web.service"], check=True)
    print("Verified export activated; private server refreshed; timer activation unchanged")


def owner_activation(args):
    """Explicit owner schedule install/uninstall; never changes kernel authority."""
    import plistlib
    import owner_daemon as owner
    import fable_launcher as f
    root = f.private_dir(args.root)
    lock = f.no_links(root / "installation.lock")
    if not lock.exists():
        f.write_file(lock, b"")
    with owner.locked(lock):
        receipt_path, plist = root / "installation.json", root / "owner.plist"
        previous = owner.load(receipt_path) if os.path.lexists(receipt_path) else None
        label = previous["label"] if previous else args.label
        presence, output = owner.service(label)
        target = f"gui/{os.getuid()}/{label}"
        if previous and presence == "present":
            f.require(f"path = {plist}\n" in output and
                      f.file_digest(owner.private_file(plist)) == previous["plist_sha256"],
                      "unowned_service")
        if args.owner == "uninstall":
            f.require(previous is not None, "installation_receipt_required")
            if presence == "present":
                subprocess.run(["/bin/launchctl", "bootout", target], check=True, timeout=40, env={})
            f.require(owner.service(label)[0] == "absent", "service_still_present")
            if plist.exists():
                f.require(f.file_digest(owner.private_file(plist)) == previous["plist_sha256"], "plist_drift")
                plist.unlink()
            previous["phase"] = "uninstalled"
            f.write_json(receipt_path, previous)
            return
        f.require(args.python and args.python_sha256 and args.runtime and args.config, "pins_required")
        f.require(args.label == label and type(args.interval) is int and 1 <= args.interval <= 30
                  and (args.interval == 30 or label.startswith("com.corbanu.initiative-owner.test-")),
                  "unreviewed_interval")
        pins = owner.schedule_pins(args.python, args.runtime, args.config, args.python_sha256)
        owner.configuration(args.config)
        owner.private_file(Path(owner.load(args.config)["coordinator"]) / "owner.sqlite3")
        logs = root / "logs"
        if not logs.exists():
            logs.mkdir(mode=0o700)
        f.private_dir(logs)
        for name in ("owner.stdout.log", "owner.stderr.log"):
            path = logs / name
            owner.private_file(path) if path.exists() else f.write_file(path, b"")
        template = Path(__file__).with_name("com.corbanu.initiative-owner.plist.in")
        job = plistlib.loads(template.read_bytes())
        replacements = {"@PINNED_PYTHON@": pins["python"], "@PRIVATE_RUNTIME@": pins["runtime"],
                        "@PRIVATE_CONFIG@": pins["config"], "@PRIVATE_LOGS@": str(logs)}
        def resolve(value):
            if isinstance(value, str):
                for key, replacement in replacements.items():
                    value = value.replace(key, replacement)
            return value
        job = {key: [resolve(v) for v in value] if isinstance(value, list) else resolve(value)
               for key, value in job.items()}
        job.update(Label=label, Disabled=False, StartInterval=args.interval,
                   ThrottleInterval=min(30, args.interval))
        job["ProgramArguments"] += ["--schedule", str(root)]
        job["EnvironmentVariables"] = {key: str(root) for key in
                                       ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")}
        raw = plistlib.dumps(job)
        expected = dict(label=label, pins=pins, plist_sha256=hashlib.sha256(raw).hexdigest(),
                        interval=args.interval)
        if previous:
            f.require({key: previous[key] for key in expected} == expected, "installation_conflict")
            if presence == "present":
                f.require(previous["phase"] == "installed", "installation_reconciliation_required")
                return
            f.require(previous["phase"] == "uninstalled", "installation_reconciliation_required")
        else:
            f.require(presence == "absent" and not any(os.path.lexists(root / name) for name in
                      ("owner.plist", "tick.json", "tick.lock")), "unowned_service")
            f.write_file(root / "tick.lock", b"")
            f.write_json(root / "tick.json", dict(started_at=None, completed_at=None, last_success=None,
                                                 hold=None, skipped=0, ticks=0, last_probe=None))
        f.write_file(plist, raw)
        f.write_json(receipt_path, dict(expected, phase="installing"))
        subprocess.run(["/bin/launchctl", "bootstrap", f"gui/{os.getuid()}", str(plist)],
                       check=True, timeout=10, env={})
        f.require(owner.service(label)[0] == "present", "installation_unobserved")
        f.write_json(receipt_path, dict(expected, phase="installed"))
        subprocess.run(["/bin/launchctl", "kickstart", target], check=True, timeout=10, env={})


if __name__ == "__main__" and "--owner" in sys.argv:
    parser = argparse.ArgumentParser(description="Explicit launchd owner recurrence activation")
    parser.add_argument("--owner", choices=("install", "uninstall"), required=True)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--label", default="com.corbanu.initiative-owner")
    parser.add_argument("--python", type=Path)
    parser.add_argument("--python-sha256")
    parser.add_argument("--runtime", type=Path)
    parser.add_argument("--config", type=Path)
    parser.add_argument("--interval", type=int, default=30)
    owner_activation(parser.parse_args())
elif __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--incoming", type=Path, required=True)
    parser.add_argument("--units", type=Path, required=True)
    args = parser.parse_args()
    activate(args.root, args.incoming, args.units)

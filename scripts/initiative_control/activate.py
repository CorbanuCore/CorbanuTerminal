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

from control import atomic_json, checked_run, locked, now, read_file, read_json, report
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
        contents = {
            "corbanu-control-publish.service": common + f'Type=oneshot\nExecStart=/usr/bin/flock {root}/.sync.lock "{sys.executable}" {root}/source/scripts/initiative_control/tick.py --root {root}\nTimeoutStartSec=10min\n',
            "corbanu-control-publish.timer": "[Unit]\nDescription=Refresh Corbanu initiative dashboard every 30 minutes\n[Timer]\nOnBootSec=2min\nOnUnitActiveSec=30min\nPersistent=true\n[Install]\nWantedBy=timers.target\n",
            "corbanu-control-web.service": common + f'Type=simple\nExecStart="{sys.executable}" {root}/source/scripts/initiative_control/control.py serve --output {root}/site --port 8768\nRestart=on-failure\nRestartSec=5\n[Install]\nWantedBy=default.target\n',
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
    subprocess.run(["systemctl", "--user", "enable", "--now", "corbanu-control-publish.timer"], check=True)
    subprocess.run(["systemctl", "--user", "enable", "corbanu-control-web.service"], check=True)
    subprocess.run(["systemctl", "--user", "restart", "corbanu-control-web.service"], check=True)
    print("Verified export activated; private server and half-hour timer started")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--incoming", type=Path, required=True)
    parser.add_argument("--units", type=Path, required=True)
    args = parser.parse_args()
    activate(args.root, args.incoming, args.units)

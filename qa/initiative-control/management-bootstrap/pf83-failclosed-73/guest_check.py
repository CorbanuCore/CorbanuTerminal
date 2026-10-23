"""Streamed trusted collector; persist no source or provenance in the actor root."""
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import stat
import subprocess
import sys


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


root = Path(sys.argv[1])
require(re.fullmatch(r"/Users/agent/pf83-preflight-[a-f0-9]{32}", str(root)), "root")
require(platform.system() == "Darwin" and platform.machine() == "arm64", "platform")
require(os.getuid() == 503 and subprocess.check_output(
    ["/usr/bin/id", "-un"], text=True).strip() == "agent", "account")
require(subprocess.check_output(["/usr/bin/sw_vers", "-productVersion"],
                               text=True).strip() == "26.2", "OS version")
hardware = subprocess.check_output(
    ["/usr/sbin/ioreg", "-rd1", "-c", "IOPlatformExpertDevice"], text=True)
match = re.search(r'"IOPlatformUUID" = "([^"]+)"', hardware)
require(match is not None and match[1] == "F9AABE1C-BA2C-55D7-9859-A6ED560D3218", "UUID")
require(root.is_dir() and not root.is_symlink(), "root directory")
index = root / "stage-inventory.json"
require(not index.is_symlink() and sha(index) == sys.argv[2], "inventory pin")
expected = json.loads(index.read_bytes())
actual = {}
for path in sorted(root.rglob("*")):
    require(not path.is_symlink(), "symlink")
    if path.is_file() and path != index:
        actual[path.relative_to(root).as_posix()] = dict(
            sha256=sha(path), mode=stat.S_IMODE(path.stat().st_mode))
    else:
        require(path == index or path.is_dir(), "special member")
require(actual == expected, "transferred inventory mismatch")
require({p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_dir()}
        == {"package", "packet"}, "extra directories")
print(json.dumps(dict(guest_uuid=match[1], inventory_sha256=sys.argv[2],
                      files=actual, product_executed=False), sort_keys=True))

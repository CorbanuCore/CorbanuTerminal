"""Trusted collector only: identity and full transferred inventory, no product launch."""
import hashlib
import json
from pathlib import Path
import platform
import os
import re
import stat
import subprocess
import sys

root = Path(sys.argv[1])
inventory_digest, attestation_digest, expected_uuid = sys.argv[2:5]
assert re.fullmatch(r"/Users/agent/pf83-preflight-[a-f0-9]{32}", str(root))
assert platform.system() == "Darwin" and platform.machine() == "arm64"
assert os.getuid() == 503 and subprocess.check_output(["/usr/bin/id", "-un"], text=True).strip() == "agent"
hardware = subprocess.check_output(["/usr/sbin/ioreg", "-rd1", "-c", "IOPlatformExpertDevice"], text=True)
uuid = re.search(r'"IOPlatformUUID" = "([^"]+)"', hardware).group(1)
assert uuid == expected_uuid, "guest identity mismatch"


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


assert not root.is_symlink() and root.is_dir()
inventory_path = root / "stage-inventory.json"
assert not inventory_path.is_symlink()
assert sha(inventory_path) == inventory_digest
expected = json.loads(inventory_path.read_bytes())
actual = {}
for path in sorted(root.rglob("*")):
    assert not path.is_symlink()
    if path.is_file():
        if path == inventory_path:
            continue
        actual[path.relative_to(root).as_posix()] = dict(
            sha256=sha(path), mode=stat.S_IMODE(path.stat().st_mode))
    else:
        assert path.is_dir()
assert actual == expected, "transferred file inventory mismatch"
# All runtime bytes, including this imported verifier, are now independently pinned.
sys.path.insert(0, str(root / "runtime"))
from verify_bundle import verify

print(json.dumps(dict(guest_uuid=uuid, user="agent", uid=os.getuid(),
    platform=platform.platform(), root=str(root), inventory_sha256=inventory_digest,
    identity=verify(root, attestation_digest), product_executed=False), sort_keys=True))

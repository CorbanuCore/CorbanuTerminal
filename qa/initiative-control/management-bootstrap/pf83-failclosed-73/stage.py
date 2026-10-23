"""Future authorized collector transport only. Identity failure cannot reach upload."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shlex
import stat
import subprocess
import uuid

HERE = Path(__file__).resolve().parent
HOSTS_SHA = "7366c535a80863731b09bb21a628cece6bd5750b8af9ef5646c339d32217418b"
GUEST_UUID = "F9AABE1C-BA2C-55D7-9859-A6ED560D3218"
IDENTITY = ("/usr/bin/uname -sm && /usr/bin/id -u && /usr/bin/id -un && "
            "/usr/bin/sw_vers -productVersion && "
            "/usr/sbin/ioreg -rd1 -c IOPlatformExpertDevice")


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def check_identity(raw):
    lines = raw.splitlines()
    require(lines[:4] == ["Darwin arm64", "503", "agent", "26.2"],
            "guest platform/account/version mismatch")
    match = re.search(r'"IOPlatformUUID" = "([^"]+)"', raw)
    require(match is not None and match[1] == GUEST_UUID, "guest UUID mismatch")


def ssh_command(key, hosts):
    require(not hosts.is_symlink() and sha(hosts) == HOSTS_SHA,
            "owner-supplied host key pin mismatch")
    require(not key.is_symlink() and stat.S_IMODE(key.stat().st_mode) == 0o600,
            "SSH identity file must have mode 0600")
    return ["/usr/bin/ssh", "-F", "/dev/null", "-T", "-i", str(key),
            "-o", "BatchMode=yes", "-o", "IdentitiesOnly=yes",
            "-o", "PasswordAuthentication=no", "-o", "KbdInteractiveAuthentication=no",
            "-o", "ForwardAgent=no", "-o", "ClearAllForwardings=yes",
            "-o", "UpdateHostKeys=no", "-o", "ControlMaster=no", "-o", "ControlPath=none",
            "-o", "StrictHostKeyChecking=yes", "-o", "GlobalKnownHostsFile=/dev/null",
            "-o", "HostKeyAlgorithms=ssh-ed25519",
            "-o", "UserKnownHostsFile=" + str(hosts), "-o", "ConnectTimeout=5",
            "agent@192.168.64.3"]


def stage(archive, receipt_path, key, hosts, evidence):
    evidence.mkdir(mode=0o700)  # Every attempt is write-once.
    ssh = ssh_command(key, hosts)
    # subprocess failure and explicit identity failure both abort this function.
    # No archive is opened and no upload subprocess exists before these succeed.
    with (evidence / "identity.stderr").open("xb") as errors:
        identity = subprocess.run(ssh + [IDENTITY], stdin=subprocess.DEVNULL,
                                  stdout=subprocess.PIPE, stderr=errors, check=True)
    (evidence / "identity.stdout").write_bytes(identity.stdout)
    check_identity(identity.stdout.decode("utf-8", errors="strict"))
    receipt = json.loads(receipt_path.read_bytes())
    require(receipt["payload_kind"] == "actor-assets-only-v1", "unsafe payload kind")
    require(sha(archive) == receipt["archive_sha256"], "archive digest mismatch")
    inventory_sha = receipt["inventory_sha256"]
    require(re.fullmatch(r"[0-9a-f]{64}", inventory_sha) is not None, "inventory digest")
    root = "/Users/agent/pf83-preflight-" + uuid.uuid4().hex
    command = "umask 077 && mkdir " + shlex.quote(root) + " && /usr/bin/tar -xpf - -C " + shlex.quote(root)
    with archive.open("rb") as payload, (evidence / "stage.stdout").open("xb") as out, (evidence / "stage.stderr").open("xb") as err:
        subprocess.run(ssh + [command], stdin=payload, stdout=out, stderr=err, check=True)
    # Collector is streamed, never stored in the actor root.
    command = "/usr/bin/python3 -I -B - " + shlex.quote(root) + " " + inventory_sha
    with (HERE / "guest_check.py").open("rb") as script, (evidence / "guest-check.stdout").open("xb") as out, (evidence / "guest-check.stderr").open("xb") as err:
        subprocess.run(ssh + [command], stdin=script, stdout=out, stderr=err, check=True)
    (evidence / "guest-root.txt").write_text(root + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("receipt", type=Path)
    parser.add_argument("--key", required=True, type=Path)
    parser.add_argument("--known-hosts", type=Path, default=HERE / "known_hosts")
    parser.add_argument("--evidence", required=True, type=Path)
    args = parser.parse_args()
    stage(args.archive, args.receipt, args.key, args.known_hosts, args.evidence)


if __name__ == "__main__":
    main()

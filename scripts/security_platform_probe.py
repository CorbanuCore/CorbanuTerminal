#!/usr/bin/env python3
"""Synthetic cross-platform containment probe for PF-27-S03."""

from __future__ import annotations

import argparse
import base64
import ctypes
import datetime as dt
import errno
import hashlib
import json
import os
import platform
import re
import socket
import struct
import subprocess
import sys
import tempfile
import threading
import uuid
from pathlib import Path
from typing import Any


CONTRACT_VERSION = "corbanu.platform-containment/v1"
FIXTURE_PROTOCOL = "corbanu.platform-probe/v1"
MAX_RESULT_AGE_SECONDS = 86_400
TARGET_LABEL_RE = re.compile(r"^[a-z0-9][a-z0-9-]{0,31}$")
CAPABILITIES = (
    "process_identity",
    "filesystem_boundary",
    "config_boundary",
    "inherited_handles",
    "ipc_peer_identity",
    "network_boundary",
    "process_memory_debug",
    "signing_entitlements",
    "elevation_boundary",
    "protected_store",
)
STATUSES = {"supported", "unsupported", "untested"}
OBSERVATIONS = {
    "observed_denied",
    "observed_allowed",
    "observed_verified",
    "not_tested",
    "not_applicable",
    "probe_error",
}
STATUS_OBSERVATIONS = {
    "supported": {"observed_denied", "observed_verified"},
    "unsupported": {"observed_allowed", "not_applicable"},
    "untested": {"not_tested", "probe_error"},
}


class ContractError(ValueError):
    pass


def utc_now() -> dt.datetime:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0)


def iso(value: dt.datetime) -> str:
    return value.isoformat().replace("+00:00", "Z")


def parse_iso(value: str) -> dt.datetime:
    parsed = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        raise ContractError("timestamp_missing_timezone")
    return parsed.astimezone(dt.timezone.utc)


def target_os() -> str:
    name = platform.system().lower()
    return {"darwin": "macos", "linux": "linux", "windows": "windows"}.get(
        name, "unknown"
    )


def windows_token_is_elevated() -> bool:
    """Return whether the current Windows process token is elevated."""

    try:
        advapi32 = ctypes.WinDLL("advapi32", use_last_error=True)
        kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    except (AttributeError, OSError) as error:
        raise OSError("windows_token_api_unavailable") from error

    token = ctypes.c_void_p()
    kernel32.GetCurrentProcess.argtypes = []
    kernel32.GetCurrentProcess.restype = ctypes.c_void_p
    advapi32.OpenProcessToken.argtypes = [
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.POINTER(ctypes.c_void_p),
    ]
    advapi32.OpenProcessToken.restype = ctypes.c_int
    advapi32.GetTokenInformation.argtypes = [
        ctypes.c_void_p,
        ctypes.c_int,
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.POINTER(ctypes.c_uint32),
    ]
    advapi32.GetTokenInformation.restype = ctypes.c_int
    kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
    kernel32.CloseHandle.restype = ctypes.c_int

    token_query = 0x0008
    token_elevation = 20
    if not advapi32.OpenProcessToken(
        kernel32.GetCurrentProcess(), token_query, ctypes.byref(token)
    ):
        raise OSError(ctypes.get_last_error(), "OpenProcessToken")
    try:
        elevated = ctypes.c_uint32()
        returned = ctypes.c_uint32()
        if not advapi32.GetTokenInformation(
            token,
            token_elevation,
            ctypes.byref(elevated),
            ctypes.sizeof(elevated),
            ctypes.byref(returned),
        ):
            raise OSError(ctypes.get_last_error(), "GetTokenInformation")
        if returned.value < ctypes.sizeof(elevated):
            raise OSError(errno.EIO, "short TokenElevation response")
        return elevated.value != 0
    finally:
        kernel32.CloseHandle(token)


def bounded(value: str, limit: int) -> str:
    value = " ".join(value.split())
    return value[:limit] if value else "unknown"


def target_metadata() -> dict[str, str]:
    cpu = platform.processor() or platform.machine() or "unknown"
    os_name = target_os()
    if os_name == "macos":
        sysctl = Path("/usr/sbin/sysctl")
        try:
            cpu = (
                subprocess.run(
                    [str(sysctl), "-n", "machdep.cpu.brand_string"],
                    stdin=subprocess.DEVNULL,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.DEVNULL,
                    text=True,
                    timeout=3,
                    check=False,
                ).stdout.strip()
                or cpu
            )
        except (OSError, subprocess.TimeoutExpired):
            pass
    elif os_name == "linux":
        try:
            for line in Path("/proc/cpuinfo").read_text(encoding="utf-8").splitlines():
                if line.lower().startswith("model name"):
                    cpu = line.split(":", 1)[1].strip() or cpu
                    break
        except OSError:
            pass
    elif os_name == "windows":
        cpu = os.environ.get("PROCESSOR_IDENTIFIER", cpu)
    return {
        "os": target_os(),
        "os_version": bounded(platform.platform(aliased=True, terse=False), 160),
        "architecture": bounded(platform.machine(), 64),
        "python_version": bounded(platform.python_version(), 64),
        "cpu": bounded(cpu, 160),
    }


def target_identity() -> str:
    components = [
        CONTRACT_VERSION,
        platform.node(),
        platform.system(),
        platform.release(),
        platform.machine(),
    ]
    if target_os() == "linux":
        boot_id = Path("/proc/sys/kernel/random/boot_id")
        if not boot_id.is_file():
            raise ContractError("boot_identity_unavailable")
        boot_identity = boot_id.read_text(encoding="utf-8").strip()
        if not boot_identity:
            raise ContractError("boot_identity_unavailable")
        components.append(boot_identity)
    elif target_os() == "macos":
        sysctl = Path("/usr/sbin/sysctl")
        if not sysctl.is_file():
            raise ContractError("boot_identity_unavailable")
        try:
            completed = subprocess.run(
                [str(sysctl), "-n", "kern.boottime"],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
                timeout=3,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            raise ContractError("boot_identity_unavailable") from error
        boot_time = completed.stdout.strip()
        if completed.returncode != 0 or not boot_time:
            raise ContractError("boot_identity_unavailable")
        components.append(boot_time)
    elif target_os() == "windows":
        try:
            buffer = ctypes.create_unicode_buffer(32_768)
            length = ctypes.windll.kernel32.GetWindowsDirectoryW(buffer, len(buffer))
        except (AttributeError, OSError) as error:
            raise ContractError("boot_identity_unavailable") from error
        if length == 0 or length >= len(buffer):
            raise ContractError("boot_identity_unavailable")
        powershell = (
            Path(buffer.value)
            / "System32"
            / "WindowsPowerShell"
            / "v1.0"
            / "powershell.exe"
        )
        if not powershell.is_file():
            raise ContractError("boot_identity_unavailable")
        try:
            completed = subprocess.run(
                [
                    str(powershell),
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "(Get-CimInstance Win32_OperatingSystem).LastBootUpTime"
                    ".ToUniversalTime().ToString('o')",
                ],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
                timeout=8,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            raise ContractError("boot_identity_unavailable") from error
        boot_time = completed.stdout.strip()
        if completed.returncode != 0 or not boot_time:
            raise ContractError("boot_identity_unavailable")
        components.append(boot_time)
    else:
        raise ContractError("boot_identity_unavailable")
    return hashlib.sha256("\0".join(components).encode("utf-8")).hexdigest()


def probe_sha256() -> str:
    return hashlib.sha256(Path(__file__).read_bytes()).hexdigest()


def result(
    capability: str,
    status: str,
    observation: str,
    mechanism: str,
    detail_code: str,
) -> dict[str, str]:
    if capability not in CAPABILITIES:
        raise ContractError("unknown_capability")
    if status not in STATUSES or observation not in OBSERVATIONS:
        raise ContractError("unknown_result_value")
    if observation not in STATUS_OBSERVATIONS[status]:
        raise ContractError("inconsistent_status_observation")
    return {
        "capability": capability,
        "status": status,
        "observation": observation,
        "mechanism": mechanism,
        "detail_code": detail_code,
    }


def encode_payload(payload: dict[str, Any]) -> str:
    raw = json.dumps(payload, separators=(",", ":")).encode("utf-8")
    return base64.urlsafe_b64encode(raw).decode("ascii")


def decode_payload(encoded: str) -> dict[str, Any]:
    if len(encoded) > 16_384:
        raise ContractError("worker_payload_too_large")
    value = json.loads(base64.urlsafe_b64decode(encoded.encode("ascii")))
    if not isinstance(value, dict):
        raise ContractError("worker_payload_not_object")
    return value


def start_worker(
    action: str,
    payload: dict[str, Any],
    *,
    inherited_descriptors: tuple[int, ...] = (),
) -> subprocess.Popen[str]:
    command = [
        sys.executable,
        str(Path(__file__).resolve()),
        "--internal-worker",
        action,
        encode_payload(payload),
    ]
    worker_env = {
        key: value
        for key in ("PATH", "SYSTEMROOT", "WINDIR", "COMSPEC", "PATHEXT", "TEMP", "TMP")
        if (value := os.environ.get(key)) is not None
    }
    options: dict[str, Any] = {"close_fds": True}
    if inherited_descriptors:
        if os.name == "nt":
            options["close_fds"] = False
        else:
            options["pass_fds"] = inherited_descriptors
    return subprocess.Popen(
        command,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        env=worker_env,
        **options,
    )


def finish_worker(
    process: subprocess.Popen[str], timeout: float = 8.0
) -> dict[str, Any]:
    try:
        stdout, _ = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        process.kill()
        process.communicate()
        return {"outcome": "error", "code": "worker_timeout"}
    if process.returncode != 0 or len(stdout) > 4096:
        return {"outcome": "error", "code": "worker_failed"}
    try:
        value = json.loads(stdout)
    except json.JSONDecodeError:
        return {"outcome": "error", "code": "worker_invalid_json"}
    if not isinstance(value, dict):
        return {"outcome": "error", "code": "worker_invalid_shape"}
    return value


def run_worker(
    action: str,
    payload: dict[str, Any],
    timeout: float = 8.0,
    *,
    inherited_descriptors: tuple[int, ...] = (),
) -> dict[str, Any]:
    try:
        process = start_worker(
            action, payload, inherited_descriptors=inherited_descriptors
        )
    except OSError:
        return {"outcome": "error", "code": "worker_start_failed"}
    return finish_worker(process, timeout)


def internal_worker(action: str, payload: dict[str, Any]) -> dict[str, Any]:
    if action == "process-signal":
        if target_os() == "windows":
            try:
                kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
                open_process = kernel32.OpenProcess
                open_process.argtypes = [ctypes.c_uint32, ctypes.c_int, ctypes.c_uint32]
                open_process.restype = ctypes.c_void_p
                handle = open_process(0x1000, False, int(payload["pid"]))
                if handle:
                    kernel32.CloseHandle(handle)
                    return {
                        "outcome": "allowed",
                        "code": "same_user_process_visible",
                    }
                if ctypes.get_last_error() == 5:
                    return {"outcome": "denied", "code": "process_query_denied"}
                return {"outcome": "error", "code": "process_query_error"}
            except (AttributeError, OSError, TypeError, ValueError):
                return {"outcome": "error", "code": "process_query_unavailable"}
        try:
            os.kill(int(payload["pid"]), 0)
            return {"outcome": "allowed", "code": "same_user_process_visible"}
        except PermissionError:
            return {"outcome": "denied", "code": "process_signal_denied"}
    if action == "read-file":
        try:
            Path(str(payload["path"])).read_bytes()
            return {"outcome": "allowed", "code": "same_user_file_readable"}
        except PermissionError:
            return {"outcome": "denied", "code": "file_read_denied"}
        except OSError:
            return {"outcome": "error", "code": "file_read_error"}
    if action == "write-file":
        try:
            Path(str(payload["path"])).write_text("worker-mutation", encoding="utf-8")
            return {"outcome": "allowed", "code": "same_user_config_writable"}
        except PermissionError:
            return {"outcome": "denied", "code": "config_write_denied"}
        except OSError:
            return {"outcome": "error", "code": "config_write_error"}
    if action == "check-fd":
        try:
            descriptor = os.fstat(int(payload["fd"]))
            if descriptor.st_dev == int(payload["device"]) and descriptor.st_ino == int(
                payload["inode"]
            ):
                return {"outcome": "allowed", "code": "descriptor_inherited"}
            return {"outcome": "denied", "code": "descriptor_not_canary"}
        except OSError:
            return {"outcome": "denied", "code": "descriptor_closed"}
    if action == "unix-connect":
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            client.connect(str(payload["path"]))
            message = json.dumps(
                {"pid": os.getpid(), "canary": str(payload["canary"])},
                separators=(",", ":"),
            ).encode("ascii")
            client.sendall(message)
            return {"outcome": "allowed", "code": "unix_connected"}
        except OSError:
            return {"outcome": "error", "code": "unix_connect_error"}
        finally:
            client.close()
    if action == "tcp-connect":
        client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        try:
            client.settimeout(3)
            client.connect(("127.0.0.1", int(payload["port"])))
            client.sendall(str(payload["token"]).encode("ascii"))
            return {"outcome": "allowed", "code": "loopback_unrestricted"}
        except PermissionError:
            return {"outcome": "denied", "code": "loopback_denied"}
        except socket.timeout:
            return {"outcome": "error", "code": "loopback_timeout"}
        except OSError as error:
            if (
                error.errno in {errno.EACCES, errno.EPERM}
                or getattr(error, "winerror", None) == 10013
            ):
                return {"outcome": "denied", "code": "loopback_denied"}
            return {"outcome": "error", "code": "loopback_probe_error"}
        finally:
            client.close()
    if action == "elevation":
        os_name = target_os()
        if os_name in {"linux", "macos"}:
            if hasattr(os, "geteuid") and os.geteuid() == 0:
                return {"outcome": "allowed", "code": "worker_already_root"}
            sudo = Path("/usr/bin/sudo")
            if not sudo.is_file():
                return {"outcome": "unavailable", "code": "sudo_unavailable"}
            try:
                completed = subprocess.run(
                    [str(sudo), "-n", "true"],
                    stdin=subprocess.DEVNULL,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    timeout=4,
                    check=False,
                )
            except subprocess.TimeoutExpired:
                return {"outcome": "error", "code": "sudo_probe_timeout"}
            if completed.returncode == 0:
                return {"outcome": "allowed", "code": "passwordless_sudo_allowed"}
            return {"outcome": "denied", "code": "noninteractive_sudo_denied"}
        if os_name == "windows":
            try:
                if windows_token_is_elevated():
                    return {
                        "outcome": "allowed",
                        "code": "worker_already_elevated",
                    }
            except (OSError, AttributeError, TypeError, ValueError):
                return {"outcome": "error", "code": "windows_token_probe_error"}
            return {
                "outcome": "unavailable",
                "code": "worker_unelevated_no_noninteractive_attempt",
            }
        return {"outcome": "unavailable", "code": "unknown_platform"}
    if action == "process-memory":
        pid = int(payload["pid"])
        os_name = target_os()
        if os_name == "linux":
            try:
                with open(f"/proc/{pid}/mem", "rb", buffering=0) as handle:
                    handle.seek(int(payload["address"]))
                    if len(handle.read(1)) != 1:
                        return {"outcome": "error", "code": "proc_mem_short_read"}
                return {"outcome": "allowed", "code": "proc_mem_readable"}
            except PermissionError:
                return {"outcome": "denied", "code": "proc_mem_denied"}
            except OSError as error:
                if error.errno in {errno.EACCES, errno.EPERM}:
                    return {"outcome": "denied", "code": "proc_mem_denied"}
                return {"outcome": "error", "code": "proc_mem_probe_error"}
        if os_name == "macos":
            try:
                libc = ctypes.CDLL("/usr/lib/libSystem.B.dylib")
                mach_task_self = libc.mach_task_self
                mach_task_self.argtypes = []
                mach_task_self.restype = ctypes.c_uint
                task_for_pid = libc.task_for_pid
                task_for_pid.argtypes = [
                    ctypes.c_uint,
                    ctypes.c_int,
                    ctypes.POINTER(ctypes.c_uint),
                ]
                task_for_pid.restype = ctypes.c_int
                task = ctypes.c_uint(0)
                kr = task_for_pid(mach_task_self(), pid, ctypes.byref(task))
                if kr == 0:
                    return {"outcome": "allowed", "code": "task_for_pid_allowed"}
                if kr in {2, 5}:
                    return {"outcome": "denied", "code": "task_for_pid_denied"}
                return {"outcome": "error", "code": "task_for_pid_probe_error"}
            except (AttributeError, OSError):
                return {"outcome": "error", "code": "task_for_pid_unavailable"}
        if os_name == "windows":
            try:
                kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
                access = 0x0010 | 0x1000
                open_process = kernel32.OpenProcess
                open_process.argtypes = [ctypes.c_uint32, ctypes.c_int, ctypes.c_uint32]
                open_process.restype = ctypes.c_void_p
                handle = open_process(access, False, pid)
                if handle:
                    kernel32.CloseHandle(handle)
                    return {"outcome": "allowed", "code": "open_process_allowed"}
                if ctypes.get_last_error() == 5:
                    return {"outcome": "denied", "code": "open_process_denied"}
                return {"outcome": "error", "code": "open_process_error"}
            except (AttributeError, OSError, TypeError, ValueError):
                return {"outcome": "error", "code": "open_process_unavailable"}
        return {"outcome": "error", "code": "unknown_platform"}
    if action == "store-attack":
        attack = str(payload["attack"])
        path = Path(str(payload["path"]))
        try:
            if attack == "read":
                path.read_bytes()
            elif attack == "delete":
                path.unlink()
            elif attack == "rename":
                path.rename(path.with_suffix(".moved"))
            elif attack == "symlink":
                path.unlink()
                path.symlink_to(Path(str(payload["replacement"])))
            elif attack == "rollback":
                path.write_text("generation-0", encoding="utf-8")
            else:
                return {"outcome": "error", "code": "unknown_store_attack"}
            return {"outcome": "allowed", "code": f"store_{attack}_allowed"}
        except PermissionError:
            return {"outcome": "denied", "code": f"store_{attack}_denied"}
        except OSError:
            return {"outcome": "error", "code": f"store_{attack}_error"}
    return {"outcome": "error", "code": "unknown_worker_action"}


def classify_access(
    capability: str, worker: dict[str, Any], mechanism: str
) -> dict[str, str]:
    outcome = worker.get("outcome")
    code = str(worker.get("code", "missing_worker_code"))
    if outcome == "denied":
        return result(capability, "supported", "observed_denied", mechanism, code)
    if outcome == "allowed":
        return result(capability, "unsupported", "observed_allowed", mechanism, code)
    return result(capability, "untested", "probe_error", mechanism, code)


def probe_ipc(_root: Path) -> dict[str, str]:
    if not hasattr(socket, "AF_UNIX"):
        return result(
            "ipc_peer_identity",
            "untested",
            "not_tested",
            "os_peer_credentials",
            "af_unix_unavailable",
        )
    if not hasattr(os, "getuid"):
        return result(
            "ipc_peer_identity",
            "untested",
            "not_tested",
            "os_peer_credentials",
            "peer_uid_api_unavailable",
        )
    if not hasattr(socket, "SO_PEERCRED"):
        return result(
            "ipc_peer_identity",
            "untested",
            "not_tested",
            "os_peer_credentials",
            "peer_api_unavailable",
        )

    # Linux abstract sockets avoid the platform's short sockaddr_un path limit,
    # so a long caller-controlled TMPDIR cannot degrade the containment result.
    socket_address = f"\0corbanu-pf27-{uuid.uuid4().hex}"
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    listener.settimeout(5)
    process: subprocess.Popen[str] | None = None
    try:
        listener.bind(socket_address)
        listener.listen(1)
        canary = uuid.uuid4().hex
        process = start_worker(
            "unix-connect", {"path": socket_address, "canary": canary}
        )
        connection, _ = listener.accept()
        with connection:
            message = json.loads(connection.recv(256).decode("ascii"))
            claimed_pid = int(message["pid"])
            echoed_canary = str(message["canary"])
            raw = connection.getsockopt(
                socket.SOL_SOCKET, socket.SO_PEERCRED, struct.calcsize("3i")
            )
            peer_pid, peer_uid, _ = struct.unpack("3i", raw)
            verified = (
                peer_pid == process.pid
                and claimed_pid == process.pid
                and peer_uid == os.getuid()
                and echoed_canary == canary
            )
            worker_result = finish_worker(process)
            process = None
            if worker_result.get("outcome") != "allowed":
                return result(
                    "ipc_peer_identity",
                    "untested",
                    "probe_error",
                    "so_peercred",
                    "peer_worker_incomplete",
                )
            if verified:
                return result(
                    "ipc_peer_identity",
                    "supported",
                    "observed_verified",
                    "so_peercred",
                    "peer_pid_uid_verified",
                )
            return result(
                "ipc_peer_identity",
                "unsupported",
                "observed_allowed",
                "so_peercred",
                "peer_identity_mismatch",
            )
    except (
        OSError,
        ValueError,
        KeyError,
        TypeError,
        struct.error,
        json.JSONDecodeError,
    ):
        return result(
            "ipc_peer_identity",
            "untested",
            "probe_error",
            "os_peer_credentials",
            "peer_probe_error",
        )
    finally:
        if process is not None and process.poll() is None:
            process.kill()
            process.communicate()
        listener.close()


def probe_network() -> dict[str, str]:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.settimeout(5)
    try:
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        port = listener.getsockname()[1]
        token = uuid.uuid4().hex
        worker_result: dict[str, Any] = {}

        def connect() -> None:
            worker_result.update(
                run_worker("tcp-connect", {"port": port, "token": token})
            )

        thread = threading.Thread(target=connect, daemon=True)
        thread.start()
        thread.join(timeout=9)
        if thread.is_alive():
            return result(
                "network_boundary",
                "untested",
                "probe_error",
                "worker_network_policy",
                "network_worker_timeout",
            )
        if worker_result.get("outcome") == "denied":
            return classify_access(
                "network_boundary", worker_result, "worker_network_policy"
            )
        if worker_result.get("outcome") != "allowed":
            return result(
                "network_boundary",
                "untested",
                "probe_error",
                "worker_network_policy",
                "network_worker_error",
            )
        connection, _ = listener.accept()
        with connection:
            chunks = bytearray()
            while len(chunks) < len(token):
                chunk = connection.recv(len(token) - len(chunks))
                if not chunk:
                    break
                chunks.extend(chunk)
            received = chunks.decode("ascii")
        if received != token:
            return result(
                "network_boundary",
                "untested",
                "probe_error",
                "worker_network_policy",
                "network_canary_mismatch",
            )
        return classify_access(
            "network_boundary", worker_result, "worker_network_policy"
        )
    except (OSError, ValueError, KeyError, TypeError):
        return result(
            "network_boundary",
            "untested",
            "probe_error",
            "worker_network_policy",
            "network_probe_error",
        )
    finally:
        listener.close()


def probe_signing() -> dict[str, str]:
    os_name = target_os()
    codesign = Path("/usr/bin/codesign")
    if os_name == "macos" and codesign.is_file():
        try:
            completed = subprocess.run(
                [str(codesign), "--display", "--verbose=2", sys.executable],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=5,
                check=False,
            )
        except subprocess.TimeoutExpired:
            return result(
                "signing_entitlements",
                "untested",
                "probe_error",
                "code_signature",
                "codesign_timeout",
            )
        code = (
            "signature_present_not_containment"
            if completed.returncode == 0
            else "signature_absent"
        )
        return result(
            "signing_entitlements",
            "unsupported",
            "not_applicable",
            "code_signature",
            code,
        )
    if os_name == "windows":
        return result(
            "signing_entitlements",
            "unsupported",
            "not_applicable",
            "authenticode",
            "signature_not_isolation",
        )
    return result(
        "signing_entitlements",
        "unsupported",
        "not_applicable",
        "platform_signing",
        "no_selected_signing_boundary",
    )


def probe_elevation() -> dict[str, str]:
    os_name = target_os()
    mechanism = "windows_token" if os_name == "windows" else "noninteractive_elevation"
    worker = run_worker("elevation", {})
    outcome = worker.get("outcome")
    code = str(worker.get("code", "missing_worker_code"))
    if outcome == "allowed":
        return result(
            "elevation_boundary", "unsupported", "observed_allowed", mechanism, code
        )
    if outcome == "denied":
        return result(
            "elevation_boundary", "supported", "observed_denied", mechanism, code
        )
    if outcome == "unavailable":
        return result("elevation_boundary", "untested", "not_tested", mechanism, code)
    return result("elevation_boundary", "untested", "probe_error", mechanism, code)


def probe_store(root: Path) -> dict[str, str]:
    attacks = []
    replacement = root / "replacement"
    replacement.write_text("replacement", encoding="utf-8")
    for attack in ("read", "delete", "rename", "symlink", "rollback"):
        attack_dir = root / f"store-{attack}"
        attack_dir.mkdir(mode=0o700)
        path = attack_dir / "head"
        path.write_text("generation-1", encoding="utf-8")
        try:
            path.chmod(0o600)
        except OSError:
            pass
        attacks.append(
            run_worker(
                "store-attack",
                {"attack": attack, "path": str(path), "replacement": str(replacement)},
            )
        )
    if any(item.get("outcome") == "allowed" for item in attacks):
        allowed = sorted(
            str(item.get("code"))
            for item in attacks
            if item.get("outcome") == "allowed"
        )
        digest = hashlib.sha256("\0".join(allowed).encode("ascii")).hexdigest()[:12]
        return result(
            "protected_store",
            "unsupported",
            "observed_allowed",
            "same_user_store",
            f"store_attacks_allowed_{digest}",
        )
    if attacks and all(item.get("outcome") == "denied" for item in attacks):
        return result(
            "protected_store",
            "supported",
            "observed_denied",
            "same_user_store",
            "all_store_attacks_denied",
        )
    return result(
        "protected_store",
        "untested",
        "probe_error",
        "same_user_store",
        "store_probe_incomplete",
    )


def run_probe(target_label: str) -> dict[str, Any]:
    if not TARGET_LABEL_RE.fullmatch(target_label):
        raise ContractError("invalid_target_label")
    measured = utc_now()
    with tempfile.TemporaryDirectory(prefix="corbanu-platform-probe-") as temp:
        root = Path(temp)
        canary = root / "controller-canary"
        canary.write_bytes(os.urandom(32))
        config = root / "controller-config"
        config.write_text("generation-1", encoding="utf-8")
        for path in (canary, config):
            try:
                path.chmod(0o600)
            except OSError:
                pass

        process_result = classify_access(
            "process_identity",
            run_worker("process-signal", {"pid": os.getpid()}),
            "same_user_process",
        )
        file_result = classify_access(
            "filesystem_boundary",
            run_worker("read-file", {"path": str(canary)}),
            "same_user_filesystem",
        )
        config_result = classify_access(
            "config_boundary",
            run_worker("write-file", {"path": str(config)}),
            "same_user_config",
        )
        descriptor = os.open(canary, os.O_RDONLY)
        try:
            descriptor_identity = os.fstat(descriptor)
            descriptor_payload = {
                "fd": descriptor,
                "device": descriptor_identity.st_dev,
                "inode": descriptor_identity.st_ino,
            }
            os.set_inheritable(descriptor, True)
            handle_control = run_worker(
                "check-fd",
                descriptor_payload,
                inherited_descriptors=(descriptor,),
            )
            os.set_inheritable(descriptor, False)
            if handle_control.get("outcome") == "allowed":
                fd_worker = run_worker("check-fd", descriptor_payload)
                handles_result = classify_access(
                    "inherited_handles", fd_worker, "close_fds"
                )
            else:
                handles_result = result(
                    "inherited_handles",
                    "untested",
                    "not_tested",
                    "close_fds",
                    "handle_probe_no_negative_control",
                )
        finally:
            os.close(descriptor)
        memory_canary = ctypes.create_string_buffer(b"corbanu-memory-probe")
        memory_result = classify_access(
            "process_memory_debug",
            run_worker(
                "process-memory",
                {"pid": os.getpid(), "address": ctypes.addressof(memory_canary)},
            ),
            "os_process_debug",
        )
        capability_results = [
            process_result,
            file_result,
            config_result,
            handles_result,
            probe_ipc(root),
            probe_network(),
            memory_result,
            probe_signing(),
            probe_elevation(),
            probe_store(root),
        ]

    eligible = all(item["status"] == "supported" for item in capability_results)
    report = {
        "contract_version": CONTRACT_VERSION,
        "fixture_protocol": FIXTURE_PROTOCOL,
        "probe_sha256": probe_sha256(),
        "run_id": uuid.uuid4().hex,
        "target_id": target_identity(),
        "target_label": target_label,
        "measured_at": iso(measured),
        "expires_at": iso(measured + dt.timedelta(seconds=MAX_RESULT_AGE_SECONDS)),
        "target": target_metadata(),
        "capabilities": capability_results,
        "protected_mode_eligible": eligible,
    }
    validate_report(report, expected_target_id=report["target_id"], now=measured)
    return report


def validate_report(
    report: dict[str, Any],
    *,
    expected_target_id: str | None = None,
    expected_probe_sha256: str | None = None,
    now: dt.datetime | None = None,
    allow_stale: bool = False,
) -> None:
    required = {
        "contract_version",
        "fixture_protocol",
        "probe_sha256",
        "run_id",
        "target_id",
        "target_label",
        "measured_at",
        "expires_at",
        "target",
        "capabilities",
        "protected_mode_eligible",
    }
    if set(report) != required:
        raise ContractError("invalid_top_level_fields")
    if report["contract_version"] != CONTRACT_VERSION:
        raise ContractError("unknown_contract")
    if report["fixture_protocol"] != FIXTURE_PROTOCOL:
        raise ContractError("unknown_fixture_protocol")
    if not re.fullmatch(r"[0-9a-f]{64}", str(report["probe_sha256"])):
        raise ContractError("invalid_probe_sha256")
    if (
        expected_probe_sha256 is not None
        and report["probe_sha256"] != expected_probe_sha256
    ):
        raise ContractError("wrong_probe_identity")
    if not re.fullmatch(r"[0-9a-f]{32}", str(report["run_id"])):
        raise ContractError("invalid_run_id")
    if not re.fullmatch(r"[0-9a-f]{64}", str(report["target_id"])):
        raise ContractError("invalid_target_id")
    if expected_target_id is not None and report["target_id"] != expected_target_id:
        raise ContractError("wrong_target_identity")
    if not TARGET_LABEL_RE.fullmatch(str(report["target_label"])):
        raise ContractError("invalid_target_label")
    measured = parse_iso(str(report["measured_at"]))
    expires = parse_iso(str(report["expires_at"]))
    current = now or utc_now()
    if measured > current + dt.timedelta(minutes=5):
        raise ContractError("future_dated")
    lifetime = expires - measured
    if lifetime <= dt.timedelta(0) or lifetime > dt.timedelta(
        seconds=MAX_RESULT_AGE_SECONDS
    ):
        raise ContractError("stale_or_invalid_expiry")
    if not allow_stale and expires <= current:
        raise ContractError("stale_or_invalid_expiry")
    target = report["target"]
    if not isinstance(target, dict) or set(target) != {
        "os",
        "os_version",
        "architecture",
        "python_version",
        "cpu",
    }:
        raise ContractError("invalid_target_metadata")
    if not isinstance(target["os"], str) or target["os"] not in {
        "linux",
        "macos",
        "windows",
        "unknown",
    }:
        raise ContractError("invalid_target_os")
    for field, maximum in (
        ("os_version", 160),
        ("architecture", 64),
        ("python_version", 64),
        ("cpu", 160),
    ):
        value = target[field]
        if not isinstance(value, str) or len(value) > maximum:
            raise ContractError("invalid_target_metadata")
    entries = report["capabilities"]
    if not isinstance(entries, list) or len(entries) != len(CAPABILITIES):
        raise ContractError("invalid_capability_count")
    names: list[str] = []
    for item in entries:
        if not isinstance(item, dict) or set(item) != {
            "capability",
            "status",
            "observation",
            "mechanism",
            "detail_code",
        }:
            raise ContractError("invalid_capability_shape")
        name = str(item["capability"])
        names.append(name)
        if name not in CAPABILITIES:
            raise ContractError("unknown_capability")
        if item["status"] not in STATUSES or item["observation"] not in OBSERVATIONS:
            raise ContractError("unknown_capability_result")
        if item["observation"] not in STATUS_OBSERVATIONS[item["status"]]:
            raise ContractError("inconsistent_status_observation")
        if not re.fullmatch(r"[a-z0-9_.-]{1,64}", str(item["mechanism"])):
            raise ContractError("invalid_mechanism")
        if not re.fullmatch(r"[a-z0-9_.-]{1,96}", str(item["detail_code"])):
            raise ContractError("invalid_detail_code")
    if len(set(names)) != len(names) or set(names) != set(CAPABILITIES):
        raise ContractError("duplicate_or_missing_capability")
    computed = all(item["status"] == "supported" for item in entries)
    if (
        not isinstance(report["protected_mode_eligible"], bool)
        or report["protected_mode_eligible"] != computed
    ):
        raise ContractError("false_eligibility_claim")


def self_test() -> None:
    report = run_probe("self-test")
    expected_probe = probe_sha256()
    validate_report(
        report,
        expected_target_id=report["target_id"],
        expected_probe_sha256=expected_probe,
    )

    wrong_identity = dict(report)
    wrong_identity["target_id"] = "0" * 64
    try:
        validate_report(wrong_identity, expected_target_id=report["target_id"])
    except ContractError as error:
        assert str(error) == "wrong_target_identity"
    else:
        raise AssertionError("wrong identity accepted")

    wrong_probe = dict(report)
    wrong_probe["probe_sha256"] = "0" * 64
    try:
        validate_report(wrong_probe, expected_probe_sha256=expected_probe)
    except ContractError as error:
        assert str(error) == "wrong_probe_identity"
    else:
        raise AssertionError("wrong probe identity accepted")

    stale = dict(report)
    old = utc_now() - dt.timedelta(days=2)
    stale["measured_at"] = iso(old)
    stale["expires_at"] = iso(old + dt.timedelta(seconds=MAX_RESULT_AGE_SECONDS))
    try:
        validate_report(stale)
    except ContractError as error:
        assert str(error) == "stale_or_invalid_expiry"
    else:
        raise AssertionError("stale result accepted")

    false_claim = json.loads(json.dumps(report))
    false_claim["protected_mode_eligible"] = True
    try:
        validate_report(false_claim, expected_target_id=report["target_id"])
    except ContractError as error:
        assert str(error) == "false_eligibility_claim"
    else:
        raise AssertionError("false eligibility accepted")

    duplicate = json.loads(json.dumps(report))
    duplicate["capabilities"][-1] = duplicate["capabilities"][0]
    try:
        validate_report(duplicate, expected_target_id=report["target_id"])
    except ContractError as error:
        assert str(error) == "duplicate_or_missing_capability"
    else:
        raise AssertionError("duplicate capability accepted")

    inconsistent = json.loads(json.dumps(report))
    inconsistent["capabilities"][0]["status"] = "supported"
    inconsistent["capabilities"][0]["observation"] = "probe_error"
    try:
        validate_report(inconsistent, expected_target_id=report["target_id"])
    except ContractError as error:
        assert str(error) == "inconsistent_status_observation"
    else:
        raise AssertionError("inconsistent status/observation accepted")

    future_archive = json.loads(json.dumps(report))
    future = utc_now() + dt.timedelta(days=1)
    future_archive["measured_at"] = iso(future)
    future_archive["expires_at"] = iso(future + dt.timedelta(hours=1))
    try:
        validate_report(future_archive, allow_stale=True)
    except ContractError as error:
        assert str(error) == "future_dated"
    else:
        raise AssertionError("future-dated archive accepted")

    eligible = json.loads(json.dumps(report))
    for item in eligible["capabilities"]:
        item["status"] = "supported"
        item["observation"] = "observed_verified"
    eligible["protected_mode_eligible"] = True
    validate_report(eligible, expected_target_id=report["target_id"])


def load_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ContractError("result_not_object")
    return value


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--probe", action="store_true", help="run the synthetic probe")
    parser.add_argument(
        "--self-test", action="store_true", help="run contract regressions"
    )
    parser.add_argument("--validate", type=Path, help="validate one result file")
    parser.add_argument(
        "--validate-evidence",
        type=Path,
        help="validate archival evidence without binding it to this target",
    )
    parser.add_argument("--output", type=Path, help="write the probe result")
    parser.add_argument(
        "--target-label", default="local", help="non-sensitive target label"
    )
    parser.add_argument(
        "--require-eligible",
        action="store_true",
        help="exit 2 unless all capabilities are supported",
    )
    parser.add_argument(
        "--internal-worker",
        nargs=2,
        metavar=("ACTION", "PAYLOAD"),
        help=argparse.SUPPRESS,
    )
    args = parser.parse_args(argv)

    if args.internal_worker:
        action, encoded = args.internal_worker
        response = internal_worker(action, decode_payload(encoded))
        sys.stdout.write(json.dumps(response, separators=(",", ":")))
        return 0

    selected = sum(
        bool(value)
        for value in (
            args.probe,
            args.self_test,
            args.validate,
            args.validate_evidence,
        )
    )
    if selected != 1:
        parser.error(
            "select exactly one of --probe, --self-test, --validate, "
            "or --validate-evidence"
        )
    try:
        if args.self_test:
            self_test()
            print("security-platform-probe: 8 contract regressions passed")
            return 0
        if args.validate:
            evidence = load_json(args.validate)
            validate_report(
                evidence,
                expected_target_id=target_identity(),
                expected_probe_sha256=probe_sha256(),
            )
            print("security-platform-probe: current-target result valid")
            return (
                2
                if args.require_eligible and not evidence["protected_mode_eligible"]
                else 0
            )
        if args.validate_evidence:
            evidence = load_json(args.validate_evidence)
            validate_report(
                evidence,
                expected_probe_sha256=probe_sha256(),
                allow_stale=True,
            )
            print("security-platform-probe: archival evidence valid")
            return (
                2
                if args.require_eligible and not evidence["protected_mode_eligible"]
                else 0
            )
        report = run_probe(args.target_label)
        rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(rendered, encoding="utf-8")
        else:
            sys.stdout.write(rendered)
        print(
            f"security-platform-probe: {len(report['capabilities'])} capabilities; "
            f"protected_mode_eligible={str(report['protected_mode_eligible']).lower()}",
            file=sys.stderr,
        )
        return (
            2 if args.require_eligible and not report["protected_mode_eligible"] else 0
        )
    except (ValueError, TypeError, OSError) as error:
        print(
            f"security-platform-probe: {type(error).__name__}: {error}", file=sys.stderr
        )
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

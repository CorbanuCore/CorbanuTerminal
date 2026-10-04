"""Loopback-only service status and lifecycle bridge for the Facilities page."""
import argparse
from collections import defaultdict
from datetime import datetime, timezone
import json
from pathlib import Path
import shlex
import subprocess
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

from facilities import FACILITIES


ALLOWED_ORIGIN = "http://127.0.0.1:8769"
SSH_USER = "travis"
SSH_TIMEOUT = 12
MAX_BODY = 4096
STATUSES = {"running", "stopped", "degraded", "initializing", "unreachable", "unknown"}


def now():
    return datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")


def facility_map():
    return {item["id"]: item for item in FACILITIES}


def _ssh_command(host, key, remote_args):
    return [
        "ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=5",
        "-o", "ConnectionAttempts=1", "-o", "StrictHostKeyChecking=yes",
        "-o", "IdentitiesOnly=yes", "-o", "LogLevel=ERROR", "-i", str(key),
        f"{SSH_USER}@{host}", shlex.join(remote_args),
    ]


def _run_remote(host, key, remote_args):
    try:
        result = subprocess.run(_ssh_command(host, key, remote_args), capture_output=True,
                                text=True, timeout=SSH_TIMEOUT, check=False)
    except (OSError, subprocess.TimeoutExpired):
        return None, "unreachable", None
    if result.returncode == 255:
        return None, "unreachable", result.returncode
    return result.stdout.splitlines(), None, result.returncode


def _states_by_host(key):
    units_by_host = defaultdict(list)
    for item in FACILITIES:
        for unit in item["service_units"] + item.get("initializing_units", ()):
            if unit not in units_by_host[item["host"]]:
                units_by_host[item["host"]].append(unit)
    result = {}
    for host, units in units_by_host.items():
        lines, error, _ = _run_remote(host, key, ["systemctl", "--user", "is-active", *units])
        if error:
            result[host] = {"__error__": error}
            continue
        states = {unit: (lines[index].strip() if index < len(lines) else "unknown")
                  for index, unit in enumerate(units)}
        result[host] = states
    return result


def _record(item, states):
    if states.get("__error__"):
        return {
            "id": item["id"], "name": item["name"], "status": "unreachable",
            "detail": "SSH connection or permission is unavailable; service state could not be observed.",
            "checked_at": now(), "actions": {"start": False, "stop": False},
        }
    service_states = {unit: states.get(unit, "unknown") for unit in item["service_units"]}
    init_states = {unit: states.get(unit, "unknown") for unit in item.get("initializing_units", ())}
    if any(value in {"active", "activating"} for value in init_states.values()):
        status = "initializing"
        detail = "Checkpoint download or service activation is in progress. Start is held until it completes."
    elif all(value == "active" for value in service_states.values()):
        status = "running"
        detail = "All managed service units are active."
    elif all(value in {"inactive", "dead", "failed"} for value in service_states.values()):
        status = "stopped"
        detail = "All managed service units are stopped."
    elif any(value == "active" for value in service_states.values()):
        status = "degraded"
        detail = "Some managed service units are active: " + ", ".join(
            unit for unit, value in service_states.items() if value == "active") + "."
    else:
        status = "unknown"
        detail = "The service state is unavailable or the configured unit was not found."
    actions = {
        "start": status not in {"running", "initializing", "unreachable", "unknown"},
        "stop": status not in {"stopped", "unreachable", "unknown"},
    }
    return {"id": item["id"], "name": item["name"], "status": status,
            "detail": detail, "checked_at": now(), "actions": actions,
            "units": service_states}


def get_statuses(key):
    states = _states_by_host(key)
    return {item["id"]: _record(item, states[item["host"]]) for item in FACILITIES}


def _perform_action(item, action, key):
    if action not in {"start", "stop"}:
        return False, "Unsupported service action."
    units = item["service_units"]
    verb = action
    lines, error, returncode = _run_remote(item["host"], key, ["systemctl", "--user", verb, *units])
    if error or lines is None or returncode != 0:
        if error:
            return False, "The action outcome is uncertain after a connection failure or timeout; check service status before retrying."
        return False, "The service action failed; no requested state was assumed."
    # systemctl may emit a useful-looking line while still failing one unit.
    # Only a zero exit status is considered successful, so repeat the command
    # with a quiet probe after the action and let the caller show live state.
    probe, probe_error, probe_code = _run_remote(item["host"], key, ["systemctl", "--user", "is-active", *units])
    if probe_error or probe is None or len(probe) != len(units) or probe_code != (0 if action == "start" else 3):
        return False, "The action finished without a verifiable service state."
    expected = "active" if action == "start" else {"inactive", "dead", "failed"}
    values = [line.strip() for line in probe]
    valid = all(value == expected if isinstance(expected, str) else value in expected for value in values)
    if not valid:
        return False, "The action did not reach the requested service state."
    return True, f"Service {action} completed and was verified."


def action(facility_id, verb, key):
    item = facility_map().get(facility_id)
    if not item:
        return 404, {"ok": False, "error": "Unknown facility."}
    records = get_statuses(key)
    before = records[facility_id]
    if before["status"] in {"unreachable", "unknown"}:
        return 503, {"ok": False, "error": before["detail"], "facility": before}
    if verb == "start" and before["status"] == "initializing":
        return 409, {"ok": False, "error": before["detail"], "facility": before}
    if verb == "start" and before["status"] == "running":
        return 200, {"ok": True, "message": "Facility is already running.", "facility": before}
    if verb == "stop" and before["status"] == "stopped":
        return 200, {"ok": True, "message": "Facility is already stopped.", "facility": before}
    ok, message = _perform_action(item, verb, key)
    after = get_statuses(key)[facility_id]
    return (200 if ok else 502), {"ok": ok, "message": message, "facility": after}


def _json(handler, code, payload):
    body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
    handler.send_response(code)
    handler.send_header("Content-Type", "application/json; charset=utf-8")
    handler.send_header("Content-Length", str(len(body)))
    handler.send_header("Cache-Control", "no-store")
    handler.send_header("X-Content-Type-Options", "nosniff")
    handler.send_header("Access-Control-Allow-Origin", ALLOWED_ORIGIN)
    handler.send_header("Vary", "Origin")
    handler.end_headers()
    handler.wfile.write(body)


def serve(port, key):
    key = Path(key).expanduser().resolve()
    if not key.is_file() or key.stat().st_mode & 0o077:
        raise ValueError("SSH key must be an existing owner-only regular file")

    class Handler(BaseHTTPRequestHandler):
        def _local_request(self):
            try:
                host = urlsplit("//" + self.headers.get("Host", "")).hostname
            except ValueError:
                host = None
            return host in {"localhost", "127.0.0.1"}

        def _browser_request(self, mutation=False):
            if not self._local_request():
                _json(self, 403, {"ok": False, "error": "Loopback access required."})
                return False
            origin = self.headers.get("Origin")
            if mutation and origin != ALLOWED_ORIGIN:
                _json(self, 403, {"ok": False, "error": "Dashboard origin required for service actions."})
                return False
            if origin and origin != ALLOWED_ORIGIN:
                _json(self, 403, {"ok": False, "error": "Unrecognized dashboard origin."})
                return False
            return True

        def do_OPTIONS(self):
            if not self._browser_request(mutation=True):
                return
            self.send_response(204)
            self.send_header("Access-Control-Allow-Origin", ALLOWED_ORIGIN)
            self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            self.send_header("Access-Control-Allow-Headers", "Content-Type")
            self.send_header("Access-Control-Max-Age", "300")
            self.end_headers()

        def do_GET(self):
            if not self._browser_request():
                return
            path = urlsplit(self.path).path
            if path == "/health":
                _json(self, 200, {"ok": True, "checked_at": now()})
            elif path == "/status":
                _json(self, 200, {"ok": True, "checked_at": now(), "facilities": get_statuses(key)})
            else:
                _json(self, 404, {"ok": False, "error": "Not found."})

        def do_POST(self):
            if not self._browser_request(mutation=True):
                return
            if urlsplit(self.path).path != "/action":
                _json(self, 404, {"ok": False, "error": "Not found."})
                return
            try:
                length = int(self.headers.get("Content-Length", "0"))
            except ValueError:
                length = 0
            if length <= 0 or length > MAX_BODY:
                _json(self, 413, {"ok": False, "error": "Invalid request size."})
                return
            try:
                value = json.loads(self.rfile.read(length))
                facility_id, verb = value["id"], value["action"]
                if not isinstance(facility_id, str) or not isinstance(verb, str):
                    raise ValueError
            except (ValueError, TypeError, KeyError, json.JSONDecodeError):
                _json(self, 400, {"ok": False, "error": "Expected a facility id and start/stop action."})
                return
            code, payload = action(facility_id, verb, key)
            _json(self, code, payload)

        def log_message(self, *_):
            pass

    ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("serve", nargs="?")
    parser.add_argument("--port", type=int, default=8770)
    parser.add_argument("--key", type=Path, default=Path.home() / ".ssh" / "rtx6000_ed25519")
    args = parser.parse_args()
    serve(args.port, args.key)

"""Synthetic loopback capture/scanning fixtures. No TLS, forwarding or live keys.

PF-13's trusted transport harness can terminate its test TLS connection and send
the resulting HTTP request here. This is not itself proof of HTTPS containment.
"""

import base64
import hashlib
import json
import secrets
import threading
from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import quote_from_bytes

from security_level_evidence import require

MAX_REQUEST = 64 * 1024


def new_canary():
    return ("corbanu-synthetic-" + secrets.token_hex(32)).encode("ascii")


def canary_forms(canary):
    require(isinstance(canary, bytes) and len(canary) >= 32, "invalid synthetic canary")
    return {
        canary,
        base64.b64encode(canary),
        canary.hex().encode(),
        quote_from_bytes(canary).encode(),
    }


def contains_canary(data, canary):
    # Hex digits are case-insensitive, including mixed-case encodings. Preserve
    # exact byte matching for raw/base64 data, whose casing is significant.
    return (
        any(value in data for value in canary_forms(canary))
        or canary.hex().encode() in data.lower()
    )


def scan_surfaces(surfaces, canary, required):
    """Complete raw/hex/base64/URL byte scans, not arbitrary-encoding detection."""
    require(
        isinstance(surfaces, dict) and set(surfaces) == set(required),
        "missing or unknown canary surface",
    )
    result = {}
    for name, data in surfaces.items():
        require(isinstance(data, bytes), "scanner requires captured bytes")
        result[name] = "failed" if contains_canary(data, canary) else "passed"
    return result


class FakeExecutor:
    """Records attempted actions only. Never signs, broadcasts or authorizes."""

    def __init__(self):
        self.actions = []
        self.lock = threading.Lock()

    def record(self, action):
        with self.lock:
            self.actions.append(dict(action))

    def matches(self, expected):
        with self.lock:
            return json.dumps(
                self.actions, sort_keys=True, allow_nan=False
            ) == json.dumps(expected, sort_keys=True, allow_nan=False)


class Capture:
    def __init__(self, canary):
        canary_forms(canary)
        self.canary = canary
        self.requests = []
        self.lock = threading.Lock()

    def record(self, method, target, headers, body):
        """Reject duplicates, redirects, other routes, and secret copies in body."""
        auth = [value for key, value in headers if key.lower() == "authorization"]
        hosts = [value for key, value in headers if key.lower() == "host"]
        other = repr(
            [(k, v) for k, v in headers if k.lower() != "authorization"]
        ).encode()
        valid = (
            method == "POST"
            and target.startswith("/v1/")
            and "?" not in target
            and "#" not in target
            and "%" not in target
            and ".." not in target
            and hosts == ["api.openai.com"]
            and auth == ["Bearer " + self.canary.decode("ascii")]
            and not contains_canary(body + other + target.encode(), self.canary)
        )
        with self.lock:
            valid = valid and not self.requests
            self.requests.append(
                {"valid": valid, "body_sha256": hashlib.sha256(body).hexdigest()}
            )
        return valid

    def report(self):
        with self.lock:
            passed = len(self.requests) == 1 and self.requests[0]["valid"]
            return {
                "phase": "fixture-self-test",
                "status": "passed" if passed else "failed",
                "request_count": len(self.requests),
                "canary_sha256": hashlib.sha256(self.canary).hexdigest(),
                "tls_qualification": "pending",
                "requests": list(self.requests),
            }


@contextmanager
def capture_proxy(canary):
    capture = Capture(canary)

    class Handler(BaseHTTPRequestHandler):
        # Ordinary HTTP only; CONNECT and forwarding deliberately unsupported.
        def log_message(self, *_args):
            pass

        def send_error(self, code, message=None, explain=None):
            capture.record("invalid-http", "", [], b"")
            # Never reflect request paths or header material into error output.
            super().send_error(code, "Synthetic capture rejected request")

        def do_POST(self):
            self.connection.settimeout(2)
            lengths = self.headers.get_all("Content-Length", [])
            valid = len(lengths) == 1 and lengths[0].isdigit()
            length = int(lengths[0]) if valid else -1
            if not 0 <= length <= MAX_REQUEST or "Transfer-Encoding" in self.headers:
                self.send_error(400)
                return
            try:
                body = self.rfile.read(length)
            except (OSError, TimeoutError):
                body = b""
            valid = len(body) == length and capture.record(
                "POST", self.path, list(self.headers.items()), body
            )
            if len(body) != length:
                capture.record("truncated", "", [], b"")
            response = b'{"id":"synthetic-only","output":[]}'
            self.send_response(200 if valid else 403)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(response)))
            self.end_headers()
            self.wfile.write(response)

        def reject(self):
            self.send_error(405)

        do_GET = do_PUT = do_DELETE = do_PATCH = do_CONNECT = do_HEAD = do_OPTIONS = (
            reject
        )

    class Server(ThreadingHTTPServer):
        daemon_threads = False

        def get_request(self):
            connection, address = super().get_request()
            connection.settimeout(2)
            return connection, address

    server = Server(("127.0.0.1", 0), Handler)
    thread = threading.Thread(
        target=lambda: server.serve_forever(poll_interval=0.05), daemon=True
    )
    thread.start()
    try:
        yield capture, server.server_address
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=3)
        require(not thread.is_alive(), "capture server failed to stop")

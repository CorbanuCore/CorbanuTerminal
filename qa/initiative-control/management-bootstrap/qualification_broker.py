#!/usr/bin/env python3
"""Bounded Responses broker for the external side of a qualification tunnel.

Serve with --host 127.0.0.1 --port PORT --upstream https://HOST/v1
--credential-file PATH --evidence PATH. BROKER_CREDENTIAL_FILE may supply the
credential *path*. The guest presents the public SYNTHETIC_BEARER below. No
profile, keychain, proxy environment, redirects or alternate upstream is used.
Only the pinned API-key client's POST /v1/responses and /v1/responses/compact
are supported (codex-api endpoint/{responses,compact}.rs). Its request bodies
are uncompressed JSON (core/client.rs::responses_request_compression).

One fsynced "turn" admission/refusal record is appended per HTTP attempt before
any successful response bytes are released. Admission requires the upstream
x-request-id and response.id (from response.created for SSE). The record is NOT
a completion receipt: joins to client rollouts prove completion/disconnection.
Only the initial SSE event is buffered; subsequent entity bytes stream unchanged.
HTTP chunk framing is regenerated. An unwritable journal prevents forwarding;
a later journal failure prevents delivery. A crash after upstream submission
but before the receipt can leave an upstream attempt with no receipt, never a
served success. Such an attempt cannot qualify and must be reconciled upstream.

"port-owner --host 127.0.0.1 --port PORT" uses stock macOS /usr/sbin/lsof and
/bin/ps, without inspecting argv or environments. It reports visible listening
PIDs, UID, command name, process start time and sockets, including wildcard
listeners on that port. This is a timestamped OS observation, not a continuous
ownership guarantee, executable attestation, tunnel-destination proof or proof
of absence beyond the caller's process visibility. Tool errors mean unknown.
The broker startup receipt binds its PID/start time/instance/module hash to its
actual bound socket. Compare it with owner observations on BOTH tunnel ends.
For a broker-down control keep the guest SSH forward alive and stop this broker;
record the forward's unchanged owner and no host broker listener before/during/
after the failed turn, with continuous egress observation. Closing the tunnel
frees the guest port: any replacement listener or observation gap makes that
control inconclusive, even if inference fails. Same-UID hostile replacement and
journal tampering require external supervision; this tool is not a sandbox.

The two files here are offline qualification infrastructure, not product or
qualification acceptance. No tools are executed by the broker: function calls
and their client-submitted outputs pass as opaque Responses body bytes.
"""

import argparse
import datetime as dt
import fcntl
import hashlib
import hmac
import http.client
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import ipaddress
import json
import os
from pathlib import Path
import re
import socket
import stat
import subprocess
import sys
import threading
from urllib.parse import urlsplit
import uuid


SYNTHETIC_BEARER = "corbanu-qualification-only"
PATHS = {"/v1/responses", "/v1/responses/compact"}
MAX_BODY = 32 * 1024 * 1024
MAX_PREFIX = 1024 * 1024
ID_KEYS = {
    "session_id", "thread_id", "turn_id", "conversation_id",
    "parent_thread_id", "parent_turn_id", "forked_from_thread_id",
    "previous_response_id", "response_id",
}
ID_HEADERS = {
    "session-id", "thread-id", "turn-id", "conversation-id",
    "x-client-request-id", "x-codex-parent-thread-id",
}
FORWARD_HEADERS = (ID_HEADERS - {"x-client-request-id"}) | {
    "x-codex-turn-state", "x-codex-turn-metadata", "x-codex-beta-features",
    "x-openai-subagent", "openai-beta", "originator",
}


def timestamp():
    return dt.datetime.now(dt.timezone.utc).isoformat()


def identifier(value):
    return value if isinstance(value, str) and re.fullmatch(r"[\w.:/-]{1,256}", value, re.ASCII) else None


def loopback(host):
    if not ipaddress.ip_address(host).is_loopback:
        raise ValueError("loopback_required")
    return host


def process_identity(pid):
    result = subprocess.run(
        ["/bin/ps", "-p", str(pid), "-o", "lstart="],
        capture_output=True, text=True, timeout=5, check=True,
        env={"PATH": "/usr/bin:/bin", "LC_ALL": "C"},
    )
    started = result.stdout.strip()
    if not started:
        raise ValueError("process_identity_unavailable")
    return {"pid": pid, "process_started": started}


def port_owner(host, port):
    loopback(host)
    if not 1 <= port <= 65535:
        raise ValueError("invalid_port")
    result = subprocess.run(
        ["/usr/sbin/lsof", "-nP", "-a", f"-iTCP:{port}",
         "-sTCP:LISTEN", "-Fpcun"],
        capture_output=True, text=True, timeout=10,
        env={"PATH": "/usr/bin:/bin:/usr/sbin", "LC_ALL": "C"},
    )
    if result.returncode not in (0, 1) or result.stderr or (result.returncode == 1 and result.stdout):
        raise ValueError("port_owner_unknown")
    owners, current = [], None
    for line in result.stdout.splitlines():
        if line.startswith("p"):
            current = process_identity(int(line[1:]))
            current["sockets"] = []
            owners.append(current)
        elif current is not None and line[:1] in ("c", "u", "n"):
            if line[0] == "n":
                current["sockets"].append(line[1:])
            else:
                current[{"c": "command", "u": "uid"}[line[0]]] = line[1:]
    if result.returncode == 0 and not owners:
        raise ValueError("port_owner_unknown")
    return {"kind": "port_owner", "observed_at": timestamp(),
            "observer_uid": os.getuid(), "host": host, "port": port,
            "scope": "visible listeners on this port, including wildcard/other addresses",
            "owners": owners}


class Refusal(Exception):
    def __init__(self, code, status=503):
        self.code, self.status = code, status


class Evidence:
    def __init__(self, path):
        self.path = Path(path)
        self.lock = threading.Lock()
        self.failed = threading.Event()

    def open(self):
        if self.failed.is_set():
            raise Refusal("evidence_unwritable")
        try:
            fd = os.open(self.path, os.O_WRONLY | os.O_APPEND | os.O_CREAT | os.O_NOFOLLOW, 0o600)
            info = os.fstat(fd)
            if not stat.S_ISREG(info.st_mode) or not info.st_mode & stat.S_IWUSR:
                raise OSError("not writable regular evidence")
            os.fsync(fd)
            return fd
        except OSError:
            if "fd" in locals():
                os.close(fd)
            raise Refusal("evidence_unwritable") from None

    def append(self, fd, record, secrets=()):
        def scrub(value):
            if isinstance(value, dict):
                return {k: scrub(v) for k, v in value.items()}
            if isinstance(value, str) and any(secret in value for secret in secrets if secret):
                return None
            return value

        record = scrub(record)
        with self.lock:
            if self.failed.is_set():
                raise Refusal("evidence_unwritable")
            try:
                fcntl.flock(fd, fcntl.LOCK_EX)
                try:
                    if not os.path.samestat(os.fstat(fd), self.path.stat()):
                        raise OSError("evidence replaced")
                    data = (json.dumps(record, sort_keys=True, separators=(",", ":")) + "\n").encode()
                    # A short/failed write is fatal; never serve from a partial receipt.
                    if os.write(fd, data) != len(data):
                        raise OSError("short evidence write")
                    os.fsync(fd)
                finally:
                    fcntl.flock(fd, fcntl.LOCK_UN)
            except OSError:
                # Latch while still locked, before any successor can append.
                self.failed.set()
                raise Refusal("evidence_unwritable") from None


class Broker(ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = False

    def __init__(self, host, port, upstream, credential_file, evidence):
        loopback(host)
        url = urlsplit(upstream)
        if (url.scheme not in ("https", "http") or not url.hostname
                or url.username or url.password or url.query or url.fragment
                or url.path.rstrip("/") != "/v1"):
            raise ValueError("invalid_upstream")
        if url.scheme == "http":
            loopback(url.hostname)  # Plaintext only for loopback fixtures.
        self.upstream = url
        self.credential_file = Path(credential_file)
        evidence = Path(evidence)
        if (evidence.resolve() == self.credential_file.resolve()
                or (evidence.exists() and self.credential_file.exists()
                    and os.path.samefile(evidence, self.credential_file))):
            raise ValueError("credential_evidence_overlap")
        self.evidence = Evidence(evidence)
        self.address_family = socket.AF_INET6 if ":" in host else socket.AF_INET
        super().__init__((host, port), Handler)
        try:
            self.identity = {**process_identity(os.getpid()), "ppid": os.getppid(),
                             "uid": os.getuid(), "instance_id": str(uuid.uuid4()),
                             "module_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                             "socket": {"host": self.server_address[0], "port": self.server_address[1]}}
            record = {"schema": 1, "kind": "startup", "recorded_at": timestamp(),
                      "broker": self.identity}
            fd = self.evidence.open()
            try:
                self.evidence.append(fd, record)
            finally:
                os.close(fd)
            print(json.dumps(record, sort_keys=True), flush=True)
        except Exception:
            self.server_close()
            raise

    def handle_error(self, request, client_address):
        # No traceback: exceptions may carry request/credential material.
        pass

    def credential(self):
        try:
            # Open at use time; no cached credential and no profile fallback.
            with self.credential_file.open("r", encoding="ascii") as source:
                value = source.read(8193).strip()
            if not re.fullmatch(r"[!-~]{1,8192}", value) or value == SYNTHETIC_BEARER:
                raise ValueError("invalid credential")
            return value
        except (OSError, ValueError):
            raise Refusal("credential_unavailable") from None


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *args):
        pass

    def setup(self):
        super().setup()
        self.connection.settimeout(60)

    def send_error(self, code, message=None, explain=None):
        self.refuse("invalid_http", code)

    def refuse(self, code, status):
        body = json.dumps({"error": code}).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Connection", "close")
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(body)
        self.close_connection = True

    def do_POST(self):
        record = {"schema": 1, "kind": "turn", "broker": self.server.identity,
                  "attempt_id": str(uuid.uuid4()), "received_at": timestamp(),
                  "upstream_headers_at": None, "recorded_at": None,
                  "path": self.path if self.path in PATHS else None,
                  "inbound_sha256": None, "model": None, "effort": None,
                  "client_ids": {}, "upstream_request_id": None,
                  "upstream_response_id": None, "upstream_status": None,
                  "outcome": None}
        fd, upstream, committed, started = None, None, False, False
        credential = None
        try:
            fd = self.server.evidence.open()  # Before any upstream activity.
            auth = self.headers.get_all("Authorization", [])
            if len(auth) != 1 or not hmac.compare_digest(
                    auth[0].encode(), ("Bearer " + SYNTHETIC_BEARER).encode()):
                raise Refusal("synthetic_bearer_required", 401)
            if self.command != "POST" or self.path not in PATHS:
                raise Refusal("route_refused", 404)
            sizes = self.headers.get_all("Content-Length", [])
            if (self.headers.get("Transfer-Encoding") or self.headers.get("Content-Encoding")
                    or len(sizes) != 1 or not sizes[0].isdigit()
                    or not 0 < int(sizes[0]) <= MAX_BODY):
                raise Refusal("invalid_body_framing", 400)
            raw = self.rfile.read(int(sizes[0]))
            if len(raw) != int(sizes[0]):
                raise Refusal("incomplete_body", 400)
            record["inbound_sha256"] = hashlib.sha256(raw).hexdigest()
            try:
                body = json.loads(raw)
                if not isinstance(body, dict) or not identifier(body.get("model")):
                    raise ValueError()
                reasoning = body.get("reasoning") or {}
                effort = reasoning.get("effort")
                if effort is not None and not identifier(effort):
                    raise ValueError()
            except (ValueError, AttributeError):
                raise Refusal("invalid_json_request", 400) from None
            credential = self.server.credential()
            record["model"], record["effort"] = body["model"], effort
            for source, values in (
                ("body", body), ("client_metadata", body.get("client_metadata")),
                ("metadata", body.get("metadata")),
            ):
                if isinstance(values, dict):
                    for key in ID_KEYS:
                        if identifier(values.get(key)):
                            record["client_ids"][source + "." + key] = values[key]
            for key in ID_HEADERS:
                if identifier(self.headers.get(key)):
                    record["client_ids"]["header." + key] = self.headers[key]
            try:
                metadata = json.loads(self.headers.get("x-codex-turn-metadata", "{}"))
                if isinstance(metadata, dict):
                    for key in ID_KEYS:
                        if identifier(metadata.get(key)):
                            record["client_ids"]["turn_metadata." + key] = metadata[key]
            except ValueError:
                raise Refusal("invalid_turn_metadata", 400) from None
            # Do not persist even known credentials disguised as metadata/identifiers.
            def safe(value):
                return not isinstance(value, str) or not any(
                    secret in value for secret in (credential, SYNTHETIC_BEARER))
            if not all(safe(v) for v in [record["model"], record["effort"], *record["client_ids"].values()]):
                record.update(model=None, effort=None, client_ids={})
                raise Refusal("sensitive_metadata", 400)
            headers = {key: self.headers[key] for key in FORWARD_HEADERS if key in self.headers}
            headers.update({"Authorization": "Bearer " + credential, "Content-Type": "application/json",
                            "Accept": "text/event-stream" if body.get("stream") else "application/json",
                            "Accept-Encoding": "identity",
                            "X-Client-Request-Id": record["attempt_id"]})
            url = self.server.upstream
            cls = http.client.HTTPSConnection if url.scheme == "https" else http.client.HTTPConnection
            upstream = cls(url.hostname, url.port, timeout=60)
            upstream.request("POST", self.path, body=raw, headers=headers)
            response = upstream.getresponse()
            record.update(upstream_status=response.status, upstream_headers_at=timestamp())
            request_id = identifier(response.getheader("x-request-id"))
            if request_id and safe(request_id):
                record["upstream_request_id"] = request_id
            if not 200 <= response.status < 300:
                raise Refusal("upstream_refused", 502)
            if not record["upstream_request_id"]:
                raise Refusal("upstream_request_id_missing", 502)
            if response.getheader("Content-Encoding", "identity") != "identity":
                raise Refusal("upstream_encoding_refused", 502)
            content_type = response.getheader("Content-Type", "").split(";")[0].strip()
            if content_type == "text/event-stream":
                prefix, response_id = self.sse_prefix(response)
            elif content_type == "application/json":
                prefix = response.read(MAX_BODY + 1)
                if len(prefix) > MAX_BODY:
                    raise Refusal("upstream_body_too_large", 502)
                try:
                    response_id = identifier(json.loads(prefix).get("id"))
                except (ValueError, AttributeError):
                    raise Refusal("upstream_invalid_json", 502) from None
            else:
                raise Refusal("upstream_content_type_refused", 502)
            if not response_id or not safe(response_id):
                raise Refusal("upstream_response_id_missing", 502)
            record.update(upstream_response_id=response_id, outcome="admitted", recorded_at=timestamp())
            self.server.evidence.append(fd, record, (credential, SYNTHETIC_BEARER))
            committed = True
            self.send_response(response.status)
            self.send_header("Content-Type", content_type)
            self.send_header("X-Request-Id", record["upstream_request_id"])
            turn_state = response.getheader("x-codex-turn-state")
            if turn_state and safe(turn_state):
                self.send_header("x-codex-turn-state", turn_state)
            self.send_header("Transfer-Encoding", "chunked")
            self.send_header("Connection", "close")
            self.end_headers()
            started = True
            self.chunk(prefix)
            if content_type == "text/event-stream":
                while chunk := response.read1(65536):
                    self.chunk(chunk)
            self.wfile.write(b"0\r\n\r\n")
            self.wfile.flush()
        except (Refusal, OSError, ValueError, http.client.HTTPException) as exc:
            failure = exc if isinstance(exc, Refusal) else Refusal("transport_failed", 502)
            if not committed and fd is not None and failure.code != "evidence_unwritable":
                record.update(outcome=failure.code, recorded_at=timestamp())
                try:
                    self.server.evidence.append(fd, record, (credential, SYNTHETIC_BEARER))
                except Refusal:
                    failure = Refusal("evidence_unwritable")
            if not started:
                self.refuse(failure.code, failure.status)
        finally:
            self.close_connection = True
            if upstream is not None:
                upstream.close()
            if fd is not None:
                os.close(fd)

    def chunk(self, data):
        if data:
            self.wfile.write(f"{len(data):x}\r\n".encode() + data + b"\r\n")
            self.wfile.flush()

    def sse_prefix(self, response):
        prefix, event = bytearray(), []
        while len(prefix) < MAX_PREFIX:
            line = response.readline(MAX_PREFIX - len(prefix) + 1)
            if not line:
                break
            prefix.extend(line)
            if len(prefix) > MAX_PREFIX:
                break
            if line.rstrip(b"\r\n") == b"":
                data = b"\n".join(event)
                event = []
                if not data:
                    continue
                try:
                    obj = json.loads(data)
                    if obj.get("type") == "response.created":
                        return bytes(prefix), identifier(obj.get("response", {}).get("id"))
                    if obj.get("type") in ("error", "response.failed", "response.completed"):
                        break
                except (ValueError, AttributeError):
                    break
            elif line.startswith(b"data:"):
                event.append(line[5:].removeprefix(b" ").rstrip(b"\r\n"))
        raise Refusal("upstream_response_id_missing", 502)

    do_GET = do_HEAD = do_CONNECT = do_PUT = do_DELETE = do_OPTIONS = do_PATCH = do_POST


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    serve = sub.add_parser("serve")
    owner = sub.add_parser("port-owner")
    for command in (serve, owner):
        command.add_argument("--host", required=True, type=loopback)
        command.add_argument("--port", required=True, type=int)
    serve.add_argument("--upstream", required=True)
    serve.add_argument("--credential-file", default=os.environ.get("BROKER_CREDENTIAL_FILE"))
    serve.add_argument("--evidence", required=True)
    args = parser.parse_args(argv)
    try:
        if args.command == "port-owner":
            print(json.dumps(port_owner(args.host, args.port), sort_keys=True))
        else:
            if not args.credential_file:
                raise ValueError("credential_path_required")
            with Broker(args.host, args.port, args.upstream, args.credential_file, args.evidence) as broker:
                broker.serve_forever()
        return 0
    except (OSError, ValueError, Refusal, subprocess.SubprocessError):
        # Never print exception arguments, paths, URLs, headers or credentials.
        print('{"error":"broker_unavailable"}', file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

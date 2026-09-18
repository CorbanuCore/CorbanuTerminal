"""Real loopback HTTP tests, with invented credentials and no live profiles."""
import contextlib
import hashlib
import http.client
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import io
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import threading
import unittest
from unittest.mock import patch

import qualification_broker as q


FAKE_SECRET = "fixture-upstream-secret-not-a-real-key"
PROMPT = "PRIVATE fixture prompt: never journal this text"


def event(kind, **values):
    return ("event: " + kind + "\r\ndata: "
            + json.dumps({"type": kind, **values}) + "\r\n\r\n").encode()


class FakeUpstream(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *args):
        pass

    def do_POST(self):
        raw = self.rfile.read(int(self.headers["Content-Length"]))
        body = json.loads(raw)
        self.server.requests.append((self.path, dict(self.headers), raw))
        number = len(self.server.requests)
        response_id = f"resp_upstream_{number}"
        request_id = f"req_upstream_{number}"
        if self.server.mode == "secret_id":
            response_id = FAKE_SECRET
        first = b": heartbeat\r\n\r\n" + event("response.created", response={"id": response_id})
        tool = {"id": "fc_fixture", "type": "function_call", "call_id": "call_fixture",
                "name": "fixture_tool", "arguments": '{"value":1}'}
        output = [] if any(item.get("type") == "function_call_output" for item in body.get("input", [])
                           if isinstance(item, dict)) else [tool]
        tail = event("response.output_item.done", item=tool) if output else b""
        tail += event("response.completed", response={"id": response_id, "output": output})
        payload = first + tail
        if self.server.mode == "error":
            self.send_response(401)
            payload = FAKE_SECRET.encode()  # Must not relay/log an upstream error body.
            content_type = "application/json"
        elif self.path.endswith("/compact") or not body.get("stream"):
            self.send_response(200)
            payload = json.dumps({"id": response_id, "output": output}).encode()
            content_type = "application/json"
        else:
            self.send_response(200)
            content_type = "text/event-stream"
        if self.server.mode == "no_response_id":
            first = event("response.created", response={})
            payload = first + tail
        self.server.payloads.append(payload)
        self.send_header("Content-Type", content_type)
        if self.server.mode != "no_request_id":
            self.send_header("X-Request-Id", request_id)
        self.send_header("x-codex-turn-state", "fixture-sticky-state")
        self.send_header("Transfer-Encoding", "chunked")
        self.send_header("Connection", "close")
        self.end_headers()
        try:
            if content_type == "text/event-stream":
                self.chunk(first[:13])  # Split inside an SSE event / CRLF.
                self.chunk(first[13:])
                self.server.prefix_sent.set()
                self.server.release.wait(5)
                for offset in range(0, len(tail), 17):
                    self.chunk(tail[offset:offset + 17])
            else:
                self.chunk(payload)
            self.wfile.write(b"0\r\n\r\n")
        except (BrokenPipeError, ConnectionResetError):
            pass
        self.close_connection = True

    def chunk(self, data):
        self.wfile.write(f"{len(data):x}\r\n".encode() + data + b"\r\n")
        self.wfile.flush()


class BrokerTests(unittest.TestCase):
    def setUp(self):
        self.logs = io.StringIO()
        self.enterContext(contextlib.redirect_stdout(self.logs))
        self.enterContext(contextlib.redirect_stderr(self.logs))
        self.temp = self.enterContext(tempfile.TemporaryDirectory())
        self.root = Path(self.temp)
        self.key = self.root / "synthetic-credential"
        self.key.write_text(FAKE_SECRET)
        self.journal = self.root / "evidence.jsonl"
        # Deny accidental off-loopback connections even if a future test regresses.
        connect = socket.socket.connect

        def loopback_connect(sock, address):
            if isinstance(address, tuple):
                q.loopback(address[0])
            return connect(sock, address)

        self.enterContext(patch.object(socket.socket, "connect", loopback_connect))
        self.up = ThreadingHTTPServer(("127.0.0.1", 0), FakeUpstream)
        self.up.requests, self.up.payloads = [], []
        self.up.mode = "ok"
        self.up.prefix_sent, self.up.release = threading.Event(), threading.Event()
        self.up.release.set()
        self.start(self.up)
        self.broker = q.Broker("127.0.0.1", 0,
                               f"http://127.0.0.1:{self.up.server_port}/v1",
                               self.key, self.journal)
        self.start(self.broker)

    def start(self, server):
        thread = threading.Thread(target=server.serve_forever, kwargs={"poll_interval": 0.02})
        thread.start()

        def stop():
            if hasattr(server, "release"):
                server.release.set()
            server.shutdown()
            server.server_close()
            thread.join(5)
            self.assertFalse(thread.is_alive())

        self.addCleanup(stop)

    def body(self, **extra):
        return dict(model="fixture-model", reasoning={"effort": "high"},
                    input=[{"role": "user", "content": PROMPT}], stream=True, **extra)

    def request(self, body=None, *, path="/v1/responses", bearer=q.SYNTHETIC_BEARER,
                headers=None, method="POST"):
        client = http.client.HTTPConnection("127.0.0.1", self.broker.server_port, timeout=3)
        self.addCleanup(client.close)
        raw = json.dumps(body if body is not None else self.body()).encode()
        values = {"Content-Type": "application/json"}
        if bearer is not None:
            values["Authorization"] = "Bearer " + bearer
        values.update(headers or {})
        client.request(method, path, raw, values)
        return client.getresponse(), raw

    def records(self):
        return [json.loads(line) for line in self.journal.read_text().splitlines()]

    def refusal(self, code, status, **kwargs):
        before = len(self.up.requests)
        response, _ = self.request(**kwargs)
        self.assertEqual(response.status, status)
        self.assertEqual(json.loads(response.read()), {"error": code})
        self.assertEqual(len(self.up.requests), before)
        return self.records()[-1]

    def test_startup_records_actual_process_and_bound_socket(self):
        record = self.records()[0]
        self.assertEqual(record["kind"], "startup")
        self.assertEqual(record["broker"]["pid"], os.getpid())
        self.assertEqual(record["broker"]["socket"], {"host": "127.0.0.1", "port": self.broker.server_port})
        self.assertTrue(record["broker"]["process_started"])
        self.assertEqual(record["broker"]["module_sha256"],
                         hashlib.sha256(Path(q.__file__).read_bytes()).hexdigest())
        self.assertEqual(json.loads(self.logs.getvalue()), record)

    def test_stream_is_incremental_exact_and_joined_to_upstream(self):
        self.up.release.clear()
        headers = {"session-id": "session_fixture", "thread-id": "thread_fixture",
                   "x-client-request-id": "client_invented_request",
                   "x-request-id": "forged_upstream_request",
                   "x-codex-turn-metadata": json.dumps({"turn_id": "turn_fixture",
                                                       "private_prompt": PROMPT})}
        body = self.body(client_metadata={"session_id": "session_fixture",
                                         "turn_id": "turn_fixture", "prompt": PROMPT},
                         metadata={"conversation_id": "conversation_fixture"},
                         response_id="forged_response")
        response, raw = self.request(body, headers=headers)
        self.assertEqual(response.status, 200)
        # This must arrive while the upstream is still blocked before its tail.
        first = b": heartbeat\r\n\r\n" + event("response.created", response={"id": "resp_upstream_1"})
        self.assertEqual(response.read(len(first)), first)
        row = self.records()[-1]  # Evidence exists BEFORE a completed response.
        self.assertEqual(row["outcome"], "admitted")
        self.assertEqual(row["inbound_sha256"], hashlib.sha256(raw).hexdigest())
        self.assertEqual((row["model"], row["effort"]), ("fixture-model", "high"))
        self.assertEqual(row["upstream_request_id"], "req_upstream_1")
        self.assertEqual(row["upstream_response_id"], "resp_upstream_1")
        self.assertEqual(row["client_ids"]["header.x-client-request-id"], "client_invented_request")
        self.assertEqual(row["client_ids"]["turn_metadata.turn_id"], "turn_fixture")
        self.assertEqual(row["client_ids"]["metadata.conversation_id"], "conversation_fixture")
        self.assertEqual(response.getheader("x-codex-turn-state"), "fixture-sticky-state")
        self.assertLessEqual(row["received_at"], row["upstream_headers_at"])
        self.assertLessEqual(row["upstream_headers_at"], row["recorded_at"])
        path, upstream_headers, sent = self.up.requests[0]
        self.assertEqual(path, "/v1/responses")
        self.assertEqual(sent, raw)
        self.assertEqual(upstream_headers["Authorization"], "Bearer " + FAKE_SECRET)
        self.assertEqual(upstream_headers["X-Client-Request-Id"], row["attempt_id"])
        self.assertNotIn("x-request-id", upstream_headers)
        self.up.release.set()
        self.assertEqual(first + response.read(), self.up.payloads[0])
        self.assertEqual(len(self.records()), 2)

    def test_tool_call_round_trip_preserves_call_and_output(self):
        first, _ = self.request()
        payload = first.read()
        self.assertIn(b'"call_id": "call_fixture"', payload)
        body = self.body(previous_response_id="resp_upstream_1")
        body["input"].append({"type": "function_call_output", "call_id": "call_fixture",
                              "output": "fixture tool returned 1"})
        second, raw = self.request(body)
        self.assertEqual(second.status, 200)
        self.assertIn(b'"output": []', second.read())
        self.assertEqual(self.up.requests[1][2], raw)
        rows = self.records()[1:]
        self.assertEqual(len(rows), 2)
        self.assertEqual([r["upstream_response_id"] for r in rows],
                         ["resp_upstream_1", "resp_upstream_2"])
        self.assertEqual(rows[1]["client_ids"]["body.previous_response_id"], "resp_upstream_1")

    def test_compact_and_nonstream_json_paths(self):
        for path in ("/v1/responses/compact", "/v1/responses"):
            with self.subTest(path=path):
                body = self.body()
                body["stream"] = False
                response, raw = self.request(body, path=path)
                self.assertEqual(response.status, 200)
                self.assertEqual(response.read(), self.up.payloads[-1])
                self.assertEqual(self.up.requests[-1][2], raw)
                self.assertEqual(self.records()[-1]["path"], path)

    def test_wrong_and_absent_bearer_are_distinct_refusals(self):
        for bearer in (None, "wrong-fixture", FAKE_SECRET):
            with self.subTest(bearer_present=bearer is not None):
                row = self.refusal("synthetic_bearer_required", 401, bearer=bearer)
                self.assertEqual(row["outcome"], "synthetic_bearer_required")
                self.assertIsNone(row["upstream_request_id"])

    def test_missing_unreadable_and_invalid_credential(self):
        self.key.unlink()
        self.refusal("credential_unavailable", 503)
        self.key.mkdir()
        self.refusal("credential_unavailable", 503)
        self.key.rmdir()
        self.key.write_text("bad\nfixture")
        self.refusal("credential_unavailable", 503)

    def test_credential_is_read_at_each_use(self):
        response, _ = self.request()
        response.read()
        rotated = "rotated-fixture-not-a-real-key"
        self.key.write_text(rotated)
        response, _ = self.request()
        response.read()
        self.assertEqual(self.up.requests[-1][1]["Authorization"], "Bearer " + rotated)
        self.assertNotIn(rotated, self.journal.read_text() + self.logs.getvalue())

    def test_unwritable_evidence_blocks_before_upstream(self):
        self.journal.chmod(0o400)
        self.addCleanup(self.journal.chmod, 0o600)
        self.refusal("evidence_unwritable", 503)
        self.assertEqual(len(self.records()), 1)

    def test_startup_refuses_unwritable_evidence_and_overlap(self):
        directory = self.root / "journal-directory"
        directory.mkdir()
        with self.assertRaises(q.Refusal):
            q.Broker("127.0.0.1", 0, "http://127.0.0.1:1/v1", self.key, directory)
        for path in (self.key, self.root / "hardlink"):
            if path != self.key:
                os.link(self.key, path)
            with self.assertRaisesRegex(ValueError, "overlap"):
                q.Broker("127.0.0.1", 0, "http://127.0.0.1:1/v1", self.key, path)
        self.assertEqual(self.key.read_text(), FAKE_SECRET)

    def test_evidence_failure_after_upstream_headers_releases_no_sse(self):
        with patch.object(self.broker.evidence, "append", side_effect=q.Refusal("evidence_unwritable")):
            response, _ = self.request()
            self.assertEqual(response.status, 503)
            self.assertEqual(json.loads(response.read()), {"error": "evidence_unwritable"})
        self.assertEqual(len(self.up.requests), 1)
        self.assertEqual(len(self.records()), 1)

    def test_short_journal_write_latches_failure_before_successor(self):
        with patch.object(q.os, "write", return_value=1):
            response, _ = self.request()
            self.assertEqual(response.status, 503)
            self.assertEqual(json.loads(response.read()), {"error": "evidence_unwritable"})
        self.assertEqual(len(self.up.requests), 1)
        self.refusal("evidence_unwritable", 503)
        self.assertTrue(self.broker.evidence.failed.is_set())

    def test_upstream_error_is_logged_without_its_body(self):
        self.up.mode = "error"
        response, _ = self.request()
        self.assertEqual(response.status, 502)
        self.assertEqual(json.loads(response.read()), {"error": "upstream_refused"})
        row = self.records()[-1]
        self.assertEqual(row["upstream_status"], 401)
        self.assertEqual(row["upstream_request_id"], "req_upstream_1")
        self.assertIsNone(row["upstream_response_id"])

    def test_missing_upstream_ids_never_use_client_inventions(self):
        for mode, code in (("no_request_id", "upstream_request_id_missing"),
                           ("no_response_id", "upstream_response_id_missing"),
                           ("secret_id", "upstream_response_id_missing")):
            with self.subTest(mode=mode):
                self.up.mode = mode
                response, _ = self.request(self.body(response_id="forged_response"),
                                            headers={"x-request-id": "forged_request"})
                self.assertEqual(response.status, 502)
                self.assertEqual(json.loads(response.read()), {"error": code})
                self.assertIsNone(self.records()[-1]["upstream_response_id"])

    def test_credentials_and_prompt_never_appear_in_evidence_or_logs(self):
        response, _ = self.request()
        response.read()
        self.up.mode = "error"
        response, _ = self.request()
        response.read()
        self.refusal("synthetic_bearer_required", 401, bearer=FAKE_SECRET)
        self.refusal("sensitive_metadata", 400,
                     body=self.body(client_metadata={"session_id": FAKE_SECRET}))
        # An earlier metadata parse failure must redact already-collected IDs too.
        self.refusal("invalid_turn_metadata", 400,
                     body=self.body(client_metadata={"session_id": FAKE_SECRET}),
                     headers={"x-codex-turn-metadata": "{"})
        for forbidden in (FAKE_SECRET, q.SYNTHETIC_BEARER, PROMPT, "Authorization"):
            self.assertNotIn(forbidden, self.journal.read_text() + self.logs.getvalue())

    def test_routes_methods_and_framing_are_closed(self):
        for path in ("/v1/models", "/responses", "/v1/responses?url=anything",
                     "/v1/responses/anything", "http://127.0.0.1/v1/responses"):
            self.refusal("route_refused", 404, path=path)
        for method in ("GET", "CONNECT", "PUT"):
            self.refusal("route_refused", 404, method=method)
        self.refusal("invalid_body_framing", 400, headers={"Content-Encoding": "zstd"})
        self.refusal("invalid_body_framing", 400, headers={"Transfer-Encoding": "chunked"})
        self.refusal("invalid_json_request", 400, body={"model": []})

    def test_no_proxy_environment_or_redirect_following(self):
        with patch.dict(os.environ, {"HTTP_PROXY": "http://192.0.2.1:9",
                                     "HTTPS_PROXY": "http://192.0.2.1:9"}):
            response, _ = self.request()
            self.assertEqual(response.status, 200)
            response.read()
        # A redirect uses the same refusal path as any non-2xx; no Location is used.
        with patch.object(FakeUpstream, "do_POST", lambda handler: (
                handler.send_response(302),
                handler.send_header("Location", "http://192.0.2.1/forbidden"),
                handler.send_header("Content-Length", "0"),
                handler.end_headers())):
            response, _ = self.request()
            self.assertEqual(response.status, 502)
            self.assertEqual(json.loads(response.read()), {"error": "upstream_refused"})

    def test_binding_and_upstream_validation(self):
        for host, upstream in (("0.0.0.0", "http://127.0.0.1/v1"),
                               ("127.0.0.1", "http://192.0.2.1/v1"),
                               ("127.0.0.1", "https://example.invalid/v1?secret=no"),
                               ("127.0.0.1", "https://user:pass@example.invalid/v1"),
                               ("127.0.0.1", "https://example.invalid/other")):
            with self.subTest(host=host, upstream=upstream), self.assertRaises(ValueError):
                q.Broker(host, 0, upstream, self.key, self.journal)

    @unittest.skipUnless(sys.platform == "darwin", "stock macOS ownership tools")
    def test_port_owner_observes_broker_and_replacement_listener(self):
        owner = q.port_owner("127.0.0.1", self.broker.server_port)
        own = [item for item in owner["owners"] if item["pid"] == os.getpid()]
        self.assertTrue(own)
        self.assertEqual(own[0]["process_started"], self.broker.identity["process_started"])
        cli = subprocess.run(
            [sys.executable, "-I", q.__file__, "port-owner", "--host", "127.0.0.1",
             "--port", str(self.broker.server_port)],
            capture_output=True, text=True, timeout=10,
            env={"PATH": "/usr/bin:/bin", "HOME": self.temp},
        )
        self.assertEqual(cli.returncode, 0, cli.stderr)
        self.assertIn(os.getpid(), [item["pid"] for item in json.loads(cli.stdout)["owners"]])
        # Different same-UID process takes the broker's released port.
        port = self.broker.server_port
        self.broker.shutdown()
        self.broker.server_close()
        self.assertEqual(q.port_owner("127.0.0.1", port)["owners"], [])
        code = ("import socket,sys; s=socket.socket(); "
                "s.bind(('127.0.0.1',int(sys.argv[1]))); s.listen(); "
                "print('ready',flush=True); sys.stdin.readline()")
        with subprocess.Popen([sys.executable, "-I", "-c", code, str(port)],
                              stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, text=True,
                              env={"PATH": "/usr/bin:/bin", "HOME": self.temp}) as child:
            try:
                self.assertEqual(child.stdout.readline().strip(), "ready")
                observed = q.port_owner("127.0.0.1", port)
                self.assertEqual([item["pid"] for item in observed["owners"]], [child.pid])
                self.assertNotEqual(child.pid, self.broker.identity["pid"])
            finally:
                child.communicate("\n", timeout=5)
        self.assertEqual(q.port_owner("127.0.0.1", port)["owners"], [])

    def test_serve_cli_uses_only_explicit_credential_path(self):
        journal = self.root / "child-evidence.jsonl"
        command = [sys.executable, "-I", q.__file__, "serve", "--host", "127.0.0.1",
                   "--port", "0", "--upstream", f"http://127.0.0.1:{self.up.server_port}/v1",
                   "--evidence", str(journal)]
        with subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              text=True, env={"PATH": "/usr/bin:/bin", "HOME": self.temp,
                                              "BROKER_CREDENTIAL_FILE": str(self.key)}) as child:
            try:
                startup = json.loads(child.stdout.readline())
                self.assertEqual(startup["broker"]["pid"], child.pid)
                client = http.client.HTTPConnection(
                    "127.0.0.1", startup["broker"]["socket"]["port"], timeout=3)
                self.addCleanup(client.close)
                client.request("POST", "/v1/responses", json.dumps(self.body()),
                               {"Authorization": "Bearer " + q.SYNTHETIC_BEARER})
                response = client.getresponse()
                self.assertEqual(response.status, 200)
                self.assertEqual(response.read(), self.up.payloads[-1])
            finally:
                child.terminate()
                output, errors = child.communicate(timeout=5)
        self.assertEqual(errors, "")
        self.assertEqual(output, "")
        self.assertNotIn(FAKE_SECRET, journal.read_text())
        self.assertEqual(len(journal.read_text().splitlines()), 2)

    def test_port_owner_tool_error_is_unknown(self):
        with patch.object(q.subprocess, "run",
                          return_value=subprocess.CompletedProcess([], 1, "", "permission denied")):
            with self.assertRaisesRegex(ValueError, "unknown"):
                q.port_owner("127.0.0.1", 18443)


if __name__ == "__main__":
    unittest.main()

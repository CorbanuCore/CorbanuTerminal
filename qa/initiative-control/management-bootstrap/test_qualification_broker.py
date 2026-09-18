"""Real loopback HTTP tests, with invented credentials and no live profiles."""
import contextlib
import base64
from concurrent.futures import ThreadPoolExecutor
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
import time
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
        resolved = {"id": response_id, **self.server.resolution}
        first = b": heartbeat\r\n\r\n" + event("response.created", response=resolved)
        tool = {"id": "fc_fixture", "type": "function_call", "call_id": "call_fixture",
                "name": "fixture_tool", "arguments": '{"value":1}'}
        output = [] if any(item.get("type") == "function_call_output" for item in body.get("input", [])
                           if isinstance(item, dict)) else [tool]
        tail = event("response.output_item.done", item=tool) if output else b""
        tail += event("response.completed", response={"id": response_id, "output": output})
        if self.server.mode == "early_eof":
            tail = b""
        elif self.server.mode == "failed":
            tail = event("response.failed", response=resolved)
        payload = first + b"".join(chunk for _, chunk in self.server.intervals) + tail
        if self.server.mode == "error":
            self.send_response(401)
            payload = FAKE_SECRET.encode()  # Must not relay/log an upstream error body.
            content_type = "application/json"
        elif self.path.endswith("/compact") or not body.get("stream"):
            self.send_response(200)
            payload = json.dumps({**resolved, "output": output}).encode()
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
                for delay, chunk in self.server.intervals:
                    time.sleep(delay)
                    if self.server.mode == "framing_trickle":
                        self.wfile.write(chunk)
                        self.wfile.flush()
                    else:
                        self.chunk(chunk)
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
        self.up.resolution = {"model": "fixture-model", "reasoning": {"effort": "high"}}
        self.up.intervals = []
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
                headers=None, method="POST", timeout=3):
        client = http.client.HTTPConnection("127.0.0.1", self.broker.server_port, timeout=timeout)
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
        if code == "synthetic_bearer_required":
            return next(row for row in self.records() if row["kind"] == "preauth_refusals")
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
        first = b": heartbeat\r\n\r\n" + event(
            "response.created", response={"id": "resp_upstream_1", **self.up.resolution})
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
        self.assertEqual(len(self.records()), 3)
        self.assertEqual(self.records()[-1]["outcome"], "relay_completed")

    def assert_truncated(self, response, reason):
        with self.assertRaises(http.client.IncompleteRead):
            response.read()
        row = self.records()[-1]
        self.assertEqual(row["kind"], "turn_end")
        self.assertEqual(row["outcome"], "truncated")
        self.assertEqual(row["failure_code"], reason)
        self.assertEqual(row["attempt_id"], self.records()[-2]["attempt_id"])
        self.assertNotIn("relay_completed", [r.get("outcome") for r in self.records()])

    def test_real_gap_longer_than_old_timeout_completes(self):
        self.assertEqual((q.CONNECT_TIMEOUT, q.RESPONSE_TIMEOUT, q.STREAM_IDLE_TIMEOUT),
                         (60, 2400, 600))
        self.up.intervals = [(61, event("response.in_progress"))]
        started = time.monotonic()
        response, _ = self.request(timeout=70)
        self.assertEqual(response.read(), self.up.payloads[-1])
        self.assertGreaterEqual(time.monotonic() - started, 61)
        self.assertEqual(self.records()[-1]["outcome"], "relay_completed")

    def test_exceeded_event_idle_tolerance_is_truncation(self):
        self.up.intervals = [(0.4, event("response.in_progress"))]
        with patch.object(q, "STREAM_IDLE_TIMEOUT", 0.15):
            response, _ = self.request()
            self.assert_truncated(response, "upstream_stream_idle_timeout")

    def test_deadline_renews_per_data_event_not_per_connection(self):
        self.up.intervals = [(0.1, event("response.in_progress"))] * 5
        with patch.object(q, "STREAM_IDLE_TIMEOUT", 0.3):
            started = time.monotonic()
            response, _ = self.request()
            self.assertEqual(response.read(), self.up.payloads[-1])
            self.assertGreater(time.monotonic() - started, 0.3)
            self.assertEqual(self.records()[-1]["outcome"], "relay_completed")

    def test_comments_and_partial_events_do_not_renew_deadline(self):
        for chunk in (b": heartbeat\r\n\r\n", b'data: {"partial":'):
            with self.subTest(chunk=chunk), patch.object(q, "STREAM_IDLE_TIMEOUT", 0.15):
                self.up.intervals = [(0.05, chunk)] * 8
                response, _ = self.request()
                self.assert_truncated(response, "upstream_stream_idle_timeout")
                # Wait for this fake handler before changing its fixture settings.
                time.sleep(0.45)

    def test_http_chunk_header_trickle_cannot_extend_event_deadline(self):
        self.up.mode = "framing_trickle"
        self.up.intervals = [(0.05, b"1")] * 8
        with patch.object(q, "STREAM_IDLE_TIMEOUT", 0.15):
            started = time.monotonic()
            response, _ = self.request()
            self.assert_truncated(response, "upstream_stream_idle_timeout")
            self.assertLess(time.monotonic() - started, 0.4)

    def test_clean_eof_and_upstream_failure_are_not_success(self):
        for mode, reason in (("early_eof", "upstream_stream_incomplete"),
                             ("failed", "upstream_stream_failed")):
            with self.subTest(mode=mode):
                self.up.mode = mode
                response, _ = self.request()
                self.assert_truncated(response, reason)

    def test_upstream_resolution_agreement_and_divergence(self):
        for stream in (True, False):
            for model, effort in (("fixture-model", "high"), ("resolved-model", "medium")):
                with self.subTest(stream=stream, model=model):
                    self.up.resolution = {"model": model, "reasoning": {"effort": effort}}
                    body = self.body()
                    body["stream"] = stream
                    response, _ = self.request(body)
                    response.read()
                    for row in self.records()[-2:]:
                        self.assertEqual((row["model"], row["effort"]), (model, effort))
                        self.assertEqual(row["client_ids"]["body.model"], "fixture-model")
                        self.assertEqual(row["client_ids"]["body.reasoning.effort"], "high")
                        self.assertEqual(row["resolution_match"],
                                         {"model": model == "fixture-model", "effort": effort == "high"})

    def test_missing_or_sensitive_upstream_resolution_never_uses_client_claim(self):
        for resolved in ({}, {"model": FAKE_SECRET, "reasoning": {"effort": FAKE_SECRET}}):
            with self.subTest(resolved_present=bool(resolved)):
                self.up.resolution = resolved
                response, _ = self.request()
                response.read()
                row = self.records()[-1]
                self.assertIsNone(row["model"])
                self.assertIsNone(row["effort"])
                self.assertEqual(row["resolution_match"], {"model": None, "effort": None})
                self.assertNotIn(FAKE_SECRET, self.journal.read_text())

    def test_unauthenticated_flood_has_fixed_space_exact_count_and_recovery(self):
        startup_bytes = self.journal.stat().st_size
        before = q.timestamp()
        with patch.object(self.broker, "credential", side_effect=AssertionError("preauth key read")):
            for number in range(128):
                row = self.refusal("synthetic_bearer_required", 401, bearer=None)
                self.assertEqual(row["count"], number + 1)
                self.assertEqual(self.journal.stat().st_size, startup_bytes + q.REFUSAL_SLOT_BYTES)
        self.assertEqual(len(self.records()), 2)
        self.assertLessEqual(before, row["first_at"])
        self.assertLessEqual(row["first_at"], row["last_at"])
        self.assertLessEqual(row["last_at"], q.timestamp())
        self.assertFalse(row["count_saturated"])
        self.assertEqual(row["broker_instance_id"], self.broker.identity["instance_id"])
        response, _ = self.request()
        response.read()
        self.assertEqual(self.records()[-1]["outcome"], "relay_completed")
        completed_bytes = self.journal.stat().st_size
        row = self.refusal("synthetic_bearer_required", 401, bearer=None)
        self.assertEqual(row["count"], 129)
        self.assertEqual(self.journal.stat().st_size, completed_bytes)
        self.assertEqual(self.records()[-1]["outcome"], "relay_completed")

    def test_concurrent_unauthenticated_refusals_do_not_lose_counts(self):
        def attempt(_):
            client = http.client.HTTPConnection("127.0.0.1", self.broker.server_port, timeout=5)
            try:
                client.request("POST", "/v1/responses", b"{}")
                response = client.getresponse()
                self.assertEqual(response.status, 401)
                response.read()
            finally:
                client.close()

        initial_bytes = self.journal.stat().st_size
        with ThreadPoolExecutor(max_workers=4) as pool:
            list(pool.map(attempt, range(64)))
        self.assertEqual(self.records()[-1]["count"], 64)
        self.assertEqual(self.journal.stat().st_size, initial_bytes + q.REFUSAL_SLOT_BYTES)
        self.assertEqual(self.up.requests, [])

    def test_preauth_disconnect_does_not_append_per_attempt_failure(self):
        initial_bytes = self.journal.stat().st_size
        with patch.object(q.Handler, "refuse", side_effect=BrokenPipeError()):
            for _ in range(8):
                with self.assertRaises((http.client.RemoteDisconnected, ConnectionResetError)):
                    self.request(bearer=None)
        self.assertEqual(self.records()[-1]["count"], 8)
        self.assertEqual(self.journal.stat().st_size, initial_bytes + q.REFUSAL_SLOT_BYTES)

    def test_refusal_slot_write_failure_latches_before_upstream(self):
        self.refusal("synthetic_bearer_required", 401, bearer=None)
        with patch.object(q.os, "write", return_value=1):
            response, _ = self.request(bearer=None)
            self.assertEqual(response.status, 503)
            self.assertEqual(json.loads(response.read()), {"error": "evidence_unwritable"})
        self.refusal("evidence_unwritable", 503)
        self.assertTrue(self.broker.evidence.failed.is_set())
        self.assertEqual(self.up.requests, [])

    def test_terminal_evidence_failure_never_records_relay_success(self):
        self.up.release.clear()
        response, _ = self.request()
        self.assertEqual(response.status, 200)
        with patch.object(q.os, "write", return_value=1):
            self.up.release.set()
            with self.assertRaises(http.client.IncompleteRead):
                response.read()
        self.assertTrue(self.broker.evidence.failed.is_set())
        self.assertEqual(self.records()[-1]["outcome"], "admitted")
        self.assertNotIn("relay_completed", [r.get("outcome") for r in self.records()])
        self.refusal("evidence_unwritable", 503)

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
        rows = [row for row in self.records() if row["kind"] == "turn"]
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

    def nonstream_handler(self, delays):
        """Pause independently before status, headers and chunked JSON body."""
        def serve(handler):
            raw = handler.rfile.read(int(handler.headers["Content-Length"]))
            handler.server.requests.append((handler.path, dict(handler.headers), raw))
            payload = json.dumps({"id": "resp_delayed", "model": "fixture-model",
                                  "reasoning": {"effort": "high"}, "output": []}).encode()
            handler.server.payloads.append(payload)
            parts = (
                b"HTTP/1.1 200 OK\r\n",
                b"Content-Type: application/json\r\nX-Request-Id: req_delayed\r\n"
                b"Transfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
                f"{len(payload):x}\r\n".encode() + payload + b"\r\n0\r\n\r\n",
            )
            try:
                for delay, part in zip(delays, parts, strict=True):
                    time.sleep(delay)
                    handler.wfile.write(part)
                    handler.wfile.flush()
            except (BrokenPipeError, ConnectionResetError):
                pass
            handler.close_connection = True
        return serve

    def test_nonstream_compact_silent_past_old_180_seconds_completes(self):
        self.assertEqual(q.RESPONSE_TIMEOUT, 2400)
        body = self.body()
        del body["stream"]  # Compact uses a unary POST without a stream flag.
        started = time.monotonic()
        with patch.object(FakeUpstream, "do_POST", self.nonstream_handler((181, 0, 0))):
            response, _ = self.request(body, path="/v1/responses/compact", timeout=195)
            self.assertEqual(response.status, 200)
            self.assertEqual(response.read(), self.up.payloads[-1])
        self.assertGreaterEqual(time.monotonic() - started, 181)
        self.assertEqual(self.records()[-1]["outcome"], "relay_completed")

    def test_nonstream_status_headers_and_body_share_response_budget(self):
        body = self.body()
        body["stream"] = False
        for path in ("/v1/responses/compact", "/v1/responses"):
            with self.subTest(path=path), patch.object(q, "RESPONSE_TIMEOUT", 1.5), \
                    patch.object(FakeUpstream, "do_POST", self.nonstream_handler((0.2, 0.2, 0.2))):
                response, _ = self.request(body, path=path)
                self.assertEqual(response.status, 200)
                self.assertEqual(response.read(), self.up.payloads[-1])
                self.assertEqual(self.records()[-1]["outcome"], "relay_completed")

    def test_exceeded_nonstream_deadline_records_transport_failure(self):
        body = self.body()
        body["stream"] = False
        # Each phase can exhaust the budget; smaller individual delays also
        # exhaust it in aggregate, proving progress does not reset the clock.
        for path in ("/v1/responses/compact", "/v1/responses"):
            for delays in ((0.5, 0, 0), (0, 0.5, 0), (0, 0, 0.5), (0.12, 0.12, 0.12)):
                with self.subTest(path=path, delays=delays), \
                        patch.object(q, "RESPONSE_TIMEOUT", 0.3), \
                        patch.object(FakeUpstream, "do_POST", self.nonstream_handler(delays)):
                    before = len(self.records())
                    response, _ = self.request(body, path=path)
                    self.assertEqual(response.status, 502)
                    self.assertEqual(json.loads(response.read()), {"error": "transport_failed"})
                    rows = self.records()[before:]
                    self.assertEqual(len(rows), 1)
                    self.assertEqual(rows[0]["kind"], "turn")
                    self.assertEqual(rows[0]["outcome"], "transport_failed")
                    self.assertNotIn("relay_completed", [r.get("outcome") for r in rows])

    def test_absent_wrong_and_credential_bearers_share_fixed_refusal_aggregate(self):
        before = self.journal.stat().st_size
        for count, bearer in enumerate((None, "wrong-fixture", FAKE_SECRET), start=1):
            with self.subTest(bearer_present=bearer is not None):
                row = self.refusal("synthetic_bearer_required", 401, bearer=bearer)
                self.assertEqual(row["outcome"], "synthetic_bearer_required")
                self.assertEqual(row["kind"], "preauth_refusals")
                self.assertNotIn("upstream_request_id", row)
                self.assertEqual(row["count"], count)
                self.assertEqual(len(self.records()), 2)
                self.assertEqual(self.journal.stat().st_size - before, q.REFUSAL_SLOT_BYTES)

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
        self.assertEqual(len(journal.read_text().splitlines()), 3)

    def test_port_owner_tool_error_is_unknown(self):
        with patch.object(q.subprocess, "run",
                          return_value=subprocess.CompletedProcess([], 1, "", "permission denied")):
            with self.assertRaisesRegex(ValueError, "unknown"):
                q.port_owner("127.0.0.1", 18443)


def fake_jwt(expires, *, label="access", account="fixture-private-account", fedramp=False):
    def encoded(value):
        return base64.urlsafe_b64encode(json.dumps(value).encode()).decode().rstrip("=")
    return ".".join((encoded({"alg": "fixture-only", "typ": "JWT"}),
                     encoded({"exp": expires, "nonce": label,
                              "https://api.openai.com/auth": {
                                  "chatgpt_account_id": account,
                                  "chatgpt_account_is_fedramp": fedramp}}),
                     "invented-signature-" + label))


def fake_tokens(expires, *, label="initial", account="fixture-private-account", fedramp=False):
    return {"access_token": fake_jwt(expires, label=label + "-access", account=account),
            "id_token": fake_jwt(expires, label=label + "-identity", account=account, fedramp=fedramp),
            "refresh_token": "invented-refresh-" + label, "account_id": account}


class SubscriptionUpstream(FakeUpstream):
    def error(self, status, payload):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("X-Request-Id", "req_refused_by_subscription")
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(payload)
        self.close_connection = True

    def do_POST(self):
        if self.path == "/oauth/token":
            obj = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            self.server.refreshes.append((obj, dict(self.headers)))
            expected = {"client_id": "app_EMoamEEZ73f0CkXaXp7hrann",
                        "grant_type": "refresh_token",
                        "refresh_token": self.server.expected["refresh_token"]}
            if obj != expected:
                self.server.contract_errors.append("refresh_body")
            if self.headers.get("Authorization") or self.headers.get("Cookie"):
                self.server.contract_errors.append("refresh_unexpected_auth")
            if self.headers.get("Content-Type") != "application/json":
                self.server.contract_errors.append("refresh_content_type")
            payload = self.server.refresh_payload
            if payload is None:
                payload = json.dumps(self.server.replacement).encode()
            if self.server.refresh_status == 200:
                self.server.expected = self.server.replacement
            time.sleep(self.server.refresh_delay)
            try:
                self.error(self.server.refresh_status, payload)
            except (BrokenPipeError, ConnectionResetError):
                pass
            return
        if self.path not in ("/backend-api/codex/responses", "/backend-api/codex/responses/compact"):
            self.server.contract_errors.append("subscription_path")
        expected = self.server.expected
        if self.headers.get("Authorization") != "Bearer " + expected["access_token"]:
            self.server.contract_errors.append("subscription_bearer")
        if self.headers.get("ChatGPT-Account-ID") != expected["account_id"]:
            self.server.contract_errors.append("subscription_account")
        if not self.headers.get("originator") or self.headers.get("Cookie"):
            self.server.contract_errors.append("subscription_originator_or_cookie")
        if self.server.reject_remaining:
            self.server.reject_remaining -= 1
            raw = self.rfile.read(int(self.headers["Content-Length"]))
            self.server.requests.append((self.path, dict(self.headers), raw))
            self.error(401, json.dumps(expected).encode())
            return
        super().do_POST()


class SubscriptionTests(unittest.TestCase):
    # Reuse fixture mechanics without inheriting/rerunning the 181s API-key test.
    start = BrokerTests.start
    body = BrokerTests.body
    request = BrokerTests.request
    records = BrokerTests.records
    refusal = BrokerTests.refusal

    def setUp(self):
        self.logs = io.StringIO()
        self.enterContext(contextlib.redirect_stdout(self.logs))
        self.enterContext(contextlib.redirect_stderr(self.logs))
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.key, self.journal = self.root / "invented-auth.json", self.root / "evidence.jsonl"
        self.now = int(time.time())
        self.tokens = fake_tokens(self.now + 3600)
        self.write_tokens(self.tokens)
        connect = socket.socket.connect

        def loopback_connect(sock, address):
            if isinstance(address, tuple):
                q.loopback(address[0])
            return connect(sock, address)

        self.enterContext(patch.object(socket.socket, "connect", loopback_connect))
        self.up = ThreadingHTTPServer(("127.0.0.1", 0), SubscriptionUpstream)
        self.up.requests, self.up.payloads, self.up.refreshes, self.up.contract_errors = [], [], [], []
        self.up.mode, self.up.intervals = "ok", []
        self.up.resolution = {"model": "fixture-model", "reasoning": {"effort": "high"}}
        self.up.prefix_sent, self.up.release = threading.Event(), threading.Event()
        self.up.release.set()
        self.up.expected = self.tokens
        self.up.replacement = fake_tokens(self.now + 7200, label="rotated")
        self.up.refresh_status, self.up.refresh_payload, self.up.reject_remaining = 200, None, 0
        self.up.refresh_delay = 0
        self.start(self.up)
        self.broker = q.Broker("127.0.0.1", 0,
                               f"http://127.0.0.1:{self.up.server_port}/backend-api/codex",
                               self.key, self.journal, auth_mode="subscription",
                               refresh_url=f"http://127.0.0.1:{self.up.server_port}/oauth/token")
        self.start(self.broker)
        self.addCleanup(lambda: self.assertEqual(self.up.contract_errors, []))

    def write_tokens(self, tokens):
        self.key.write_text(json.dumps({"auth_mode": "chatgpt", "OPENAI_API_KEY": None,
                                        "tokens": tokens, "last_refresh": "2026-09-17T00:00:00Z"}))

    def completed(self, **kwargs):
        response, raw = self.request(**kwargs)
        self.assertEqual(response.status, 200)
        payload = response.read()
        rows = self.records()
        self.assertEqual(rows[-2]["outcome"], "admitted")
        self.assertEqual(rows[-1]["outcome"], "relay_completed")
        self.assertEqual(rows[-2]["attempt_id"], rows[-1]["attempt_id"])
        self.assertEqual(rows[-2]["upstream_request_id"], response.getheader("X-Request-Id"))
        self.assertTrue(rows[-2]["upstream_response_id"].startswith("resp_upstream_"))
        self.assertEqual(rows[-2]["model"], "fixture-model")
        self.assertEqual(rows[-2]["effort"], "high")
        return payload, raw

    def test_subscription_routes_exact_headers_body_and_joinable_receipts(self):
        for path in ("/v1/responses", "/v1/responses/compact"):
            with self.subTest(path=path):
                body = self.body(store=False, instructions="fixture", include=["reasoning.encrypted_content"])
                supplied = {"session-id": "fixture-session", "thread-id": "fixture-thread",
                            "originator": "codex_cli_rs", "User-Agent": "fixture-pinned-client/1",
                            "x-client-request-id": "client-invention", "ChatGPT-Account-ID": "untrusted-account",
                            "X-OpenAI-Fedramp": "true", "Cookie": "untrusted-cookie"}
                payload, raw = self.completed(path=path, body=body, headers=supplied)
                actual_path, headers, actual_raw = self.up.requests[-1]
                lower = {k.lower(): v for k, v in headers.items()}
                self.assertEqual(actual_path, "/backend-api/codex" + path.removeprefix("/v1"))
                self.assertEqual(actual_raw, raw)
                self.assertEqual(payload, self.up.payloads[-1])
                self.assertEqual(lower["session-id"], "fixture-session")
                self.assertEqual(lower["thread-id"], "fixture-thread")
                self.assertEqual(lower["user-agent"], "fixture-pinned-client/1")
                self.assertEqual(lower["x-client-request-id"], self.records()[-1]["attempt_id"])
                self.assertNotEqual(lower["x-client-request-id"], "client-invention")
                for absent in ("openai-beta", "cookie", "x-openai-fedramp"):
                    self.assertNotIn(absent, lower)
        self.assertEqual(self.up.refreshes, [])

    def test_expired_missing_exp_and_malformed_credentials_refuse_without_upstream(self):
        self.write_tokens(fake_tokens(self.now - 1))
        self.refusal("subscription_expired", 503)
        for tokens in ({"access_token": "bad"}, {**self.tokens, "account_id": "bad\nheader"},
                       {**self.tokens, "access_token": fake_jwt(None)}):
            self.write_tokens(tokens)
            self.refusal("credential_unavailable", 503)
        self.assertEqual(self.up.refreshes, [])

    def test_file_is_reread_revocation_and_replacement_are_observed(self):
        self.completed()
        self.key.unlink()
        self.refusal("credential_unavailable", 503)
        self.tokens = fake_tokens(self.now + 5400, label="external-replacement")
        self.up.expected = self.tokens
        self.write_tokens(self.tokens)
        self.completed()
        self.assertEqual(self.up.refreshes, [])

    def test_proactive_refresh_is_serialized_memory_only_and_source_is_unchanged(self):
        self.tokens = fake_tokens(self.now + 200)
        self.up.expected = self.tokens
        self.write_tokens(self.tokens)
        before, stat_before = self.key.read_bytes(), self.key.stat()
        def turn(_):
            response, _ = self.request()
            status, payload = response.status, response.read()
            return status, bool(payload)
        with ThreadPoolExecutor(max_workers=5) as pool:
            results = list(pool.map(turn, range(5)))
        self.assertEqual(results, [(200, True)] * 5)
        self.assertEqual(len(self.up.refreshes), 1)
        self.assertEqual(len(self.up.requests), 5)
        self.assertEqual(self.key.read_bytes(), before)
        self.assertEqual(self.key.stat().st_mtime_ns, stat_before.st_mtime_ns)
        self.assertEqual(sum(r.get("outcome") == "relay_completed" for r in self.records()), 5)
        persisted = self.journal.read_text() + self.logs.getvalue()
        for tokens in (self.tokens, self.up.replacement):
            for secret in q.Subscription(tokens).secrets:
                self.assertNotIn(secret, persisted)

    def test_401_refresh_retries_before_admission_and_preserves_request_body(self):
        before = self.key.read_bytes()
        self.up.reject_remaining = 1
        self.completed()
        self.assertEqual(len(self.up.refreshes), 1)
        self.assertEqual(len(self.up.requests), 2)
        self.assertEqual(self.up.requests[0][2], self.up.requests[1][2])
        self.assertEqual(self.key.read_bytes(), before)
        self.assertEqual(len(self.records()), 3)
        self.assertEqual(self.records()[-1]["upstream_request_id"], "req_upstream_2")

    def test_mid_run_refresh_failure_refuses_then_latches_until_file_changes(self):
        self.completed()
        self.up.reject_remaining, self.up.refresh_status = 1, 401
        self.up.refresh_payload = json.dumps(self.tokens).encode()
        response, _ = self.request()
        self.assertEqual(response.status, 503)
        self.assertEqual(json.loads(response.read()), {"error": "subscription_refresh_failed"})
        self.assertEqual(self.records()[-1]["outcome"], "subscription_refresh_failed")
        self.assertEqual(self.records()[-1]["upstream_status"], 401)
        self.refusal("subscription_refresh_failed", 503)
        self.assertEqual(len(self.up.refreshes), 1)
        self.tokens = fake_tokens(self.now + 5400, label="recovered")
        self.up.expected = self.tokens
        self.write_tokens(self.tokens)
        self.completed()

    def test_refresh_bad_status_redirect_invalid_payload_account_and_expiry_fail_closed(self):
        variants = [(302, b"{}"), (500, b"upstream private error"), (200, b"invalid-json"),
                    (200, json.dumps(fake_tokens(self.now - 1)).encode()),
                    (200, json.dumps(fake_tokens(self.now + 7000, account="wrong-account")).encode()),
                    (200, b"{}")]
        for index, (status, payload) in enumerate(variants):
            with self.subTest(index=index):
                self.tokens = fake_tokens(self.now + 200, label=f"variant-{index}")
                self.up.expected = self.tokens
                self.write_tokens(self.tokens)
                self.up.refresh_status, self.up.refresh_payload = status, payload
                self.refusal("subscription_refresh_failed", 503)
        self.assertEqual(len(self.up.refreshes), len(variants))
        self.assertEqual(self.up.requests, [])

    def test_refresh_deadline_fails_closed_without_inference(self):
        self.tokens = fake_tokens(self.now + 200)
        self.up.expected = self.tokens
        self.write_tokens(self.tokens)
        self.up.refresh_delay = 0.2
        with patch.object(q, "CONNECT_TIMEOUT", 0.05):
            self.refusal("subscription_refresh_failed", 503)
        self.assertEqual(len(self.up.refreshes), 1)
        self.assertEqual(self.up.requests, [])

    def test_refresh_lock_wait_respects_total_turn_deadline(self):
        with self.broker.subscription_lock, patch.object(q, "RESPONSE_TIMEOUT", 0.1):
            started = time.monotonic()
            self.refusal("transport_failed", 502)
            self.assertLess(time.monotonic() - started, 1)
        self.assertEqual(self.up.refreshes, [])

    def test_no_refresh_token_refuses_proactive_refresh_without_network(self):
        for lifetime in (200, 3600):
            self.tokens = {**fake_tokens(self.now + lifetime), "refresh_token": ""}
            self.write_tokens(self.tokens)
            self.refusal("subscription_refresh_failed", 503)
        self.assertEqual(self.up.refreshes, [])

    def test_second_401_is_not_retried_or_served(self):
        self.up.reject_remaining = 2
        response, _ = self.request()
        self.assertEqual(response.status, 502)
        self.assertEqual(json.loads(response.read()), {"error": "upstream_refused"})
        self.assertEqual(len(self.up.requests), 2)
        self.assertEqual(len(self.up.refreshes), 1)
        self.assertNotIn("admitted", [r.get("outcome") for r in self.records()])

    def test_expiration_during_stream_truncates_without_refresh_or_replay(self):
        self.up.release.clear()
        response, _ = self.request()
        self.assertEqual(response.status, 200)
        self.assertTrue(self.up.prefix_sent.wait(2))
        with patch.object(q.time, "time", return_value=self.now + 4000):
            self.up.release.set()
            with self.assertRaises(http.client.IncompleteRead):
                response.read()
        self.assertEqual(self.records()[-1]["outcome"], "truncated")
        self.assertEqual(self.records()[-1]["failure_code"], "subscription_expired")
        self.assertEqual(len(self.up.requests), 1)
        self.assertEqual(self.up.refreshes, [])

    def test_durable_receipt_failure_serves_nothing_for_subscription(self):
        original = self.broker.evidence.append
        def fail_turn(fd, record, *args, **kwargs):
            if record["kind"] == "turn":
                raise q.Refusal("evidence_unwritable")
            return original(fd, record, *args, **kwargs)
        with patch.object(self.broker.evidence, "append", side_effect=fail_turn):
            response, _ = self.request()
            self.assertEqual(response.status, 503)
            self.assertEqual(json.loads(response.read()), {"error": "evidence_unwritable"})
        self.assertEqual(len(self.records()), 1)

    def test_no_credential_fields_components_or_identity_in_journal_logs_or_errors(self):
        # Probe client metadata and upstream error bodies with invented secrets.
        parts = q.Subscription(self.tokens).secrets
        bodies = []
        for value in (self.tokens["account_id"], self.tokens["refresh_token"],
                      self.tokens["id_token"], self.tokens["access_token"],
                      self.tokens["access_token"].split(".")[1]):
            response, _ = self.request(headers={"session-id": value})
            # Overlength identifiers are omitted by the existing allowlist;
            # allowlisted secret identifiers are refused. Neither may persist.
            self.assertEqual(response.status, 400 if q.identifier(value) else 200)
            bodies.append(response.read().decode())
        self.up.reject_remaining, self.up.refresh_status = 1, 401
        self.up.refresh_payload = json.dumps(self.tokens).encode()
        response, _ = self.request()
        bodies.append(response.read().decode())
        persisted = self.journal.read_text() + self.logs.getvalue() + "".join(bodies)
        for secret in parts:
            self.assertNotIn(secret, persisted)
        self.assertNotIn(PROMPT, persisted)

    def test_fedramp_flag_comes_from_stored_id_token_only(self):
        self.tokens = fake_tokens(self.now + 3600, fedramp=True)
        self.up.expected = self.tokens
        self.write_tokens(self.tokens)
        self.completed()
        lower = {k.lower(): v for k, v in self.up.requests[-1][1].items()}
        self.assertEqual(lower["x-openai-fedramp"], "true")

    def test_preauth_does_not_open_subscription_file_or_refresh_and_stays_bounded(self):
        before = self.journal.stat().st_size
        with patch.object(self.broker, "subscription", side_effect=AssertionError("must not read")):
            for bearer in (None, "wrong", self.tokens["access_token"]):
                self.refusal("synthetic_bearer_required", 401, bearer=bearer)
        self.assertEqual(self.journal.stat().st_size - before, q.REFUSAL_SLOT_BYTES)
        self.assertEqual(self.records()[-1]["count"], 3)
        self.assertEqual(self.up.refreshes, [])

    def test_subscription_configuration_does_not_accept_other_bases_or_refresh_hosts(self):
        for kwargs in ({"upstream": "https://chatgpt.com/v1"},
                       {"refresh_url": "https://other.invalid/oauth/token"},
                       {"refresh_url": "http://127.0.0.1/oauth/token?secret=bad"}):
            options = dict(upstream="https://chatgpt.com/backend-api/codex",
                           refresh_url="https://auth.openai.com/oauth/token")
            options.update(kwargs)
            with self.assertRaises(ValueError):
                q.Broker("127.0.0.1", 0, credential_file=self.key,
                         evidence=self.root / "unused", auth_mode="subscription", **options)


if __name__ == "__main__":
    unittest.main()

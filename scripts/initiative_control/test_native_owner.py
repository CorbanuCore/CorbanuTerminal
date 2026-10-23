"""Fixture-only native adapters. No real native/model/tool calls or credentials."""

import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import unittest
from unittest.mock import patch

from coordinator import Coordinator, Rejected
from native_owner import NativeOwner, StdioHost, binding
from test_coordinator import seed


ROOT = Path(__file__).resolve().parents[2]
CLI = ROOT / "scripts/initiative_control/native_owner.py"


def fixture_response(request, status=None):
    status = status or {"spawn": "spawned", "start-work": "submitted",
                        "close": "closed", "poll": "ready"}[request["operation"]]
    agent = request["agent_id"] or "fixture-native-agent"
    result = {"request_id": request["request_id"], "binding": request["binding"],
              "agent_id": agent, "status": status, "receipt": {"fixture_only": True}}
    if status == "ready":
        result["ack"] = {**request["binding"], "agent_id": agent, "ack": "ready_no_work"}
    if status in {"returned", "failed"}:
        result["result"] = {"fixture_only": True, "candidate": "b" * 40}
    return result


class FixtureHost:
    """Deterministic transport fixture, not proof of native lifecycle execution."""

    def __init__(self):
        self.calls, self.status, self.crash = [], None, False

    def __call__(self, request, timeout):
        self.calls.append(copy.deepcopy(request))
        if self.crash:
            raise SystemExit("fixture: effect happened, process died before receipt")
        return fixture_response(request, self.status)


class NativeOwnerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="native-owner-fixture-", dir=ROOT)
        self.directory = Path(self.tmp.name) / "state"
        self.c = Coordinator(self.directory)
        self.c.initialize(*seed())
        self.c.set_enabled(True, {"fixture_only": True})
        self.host = FixtureHost()
        self.owner = NativeOwner(self.c, self.host)
        self.prepare()

    def tearDown(self):
        self.tmp.cleanup()

    def prepare(self, name="first", kind="repair"):
        self.c.event({"id": "fixture-" + name})
        packet = self.c.begin_manager()
        action = {"id": name, "kind": kind, "workstream": "delivery", "sprint": "PF80",
                  "rationale": "fixture only", "expected_revision": packet["state_revision"],
                  "inputs": {"allocation": "bootstrap", "base": "a" * 40}, "timeout_seconds": 60}
        self.c.accept_decision(packet["manager_run"], {"state_revision": packet["state_revision"],
                              "actions": [action]}, {"fixture_only": True})

    def restart(self):
        self.c = Coordinator(self.directory)
        self.owner = NativeOwner(self.c, self.host)

    def ready(self):
        self.owner.run("first", "spawn")
        self.owner.run("first", "poll")

    def working(self):
        self.ready()
        self.owner.run("first", "start-work")

    def returned(self, status="returned"):
        self.working()
        self.host.status = status
        return self.owner.run("first", "poll")

    def test_success_duplicate_and_return_never_self_accept(self):
        self.ready()
        self.assertNotIn("assignment", self.host.calls[0])
        self.assertNotIn("inputs", self.host.calls[0]["startup"])
        self.owner.run("first", "start-work")
        self.assertEqual({"allocation": "bootstrap", "base": "a" * 40},
                         self.host.calls[-1]["assignment"]["inputs"])
        for op in ("spawn", "start-work"):
            self.owner.run("first", op)
        self.assertEqual(3, len(self.host.calls))
        self.host.status = "returned"
        result = self.owner.run("first", "poll")
        self.assertEqual("awaiting_owner_verification", result["status"])
        self.assertEqual("returned", result["core_status"])
        self.assertIn("integrates", result["next_owner_action"])
        self.host.status = None
        self.assertTrue(self.owner.run("first", "close")["closed"])
        self.restart()
        self.owner.run("first", "close")
        self.owner.run("first", "poll")
        self.assertEqual(5, len(self.host.calls))
        self.assertEqual("in_progress", self.c.snapshot()["sprints"]["PF80"]["status"])
        self.assertNotIn("verification", self.c.snapshot()["actions"]["first"])

    def test_no_ack_no_work_and_wrong_ack_each_binding(self):
        self.owner.run("first", "spawn")
        with self.assertRaises(Rejected):
            self.owner.run("first", "start-work")
        for field in ("action_id", "claim", "allocation_digest", "agent_id", "ack"):
            with self.subTest(field=field):
                def wrong(request, timeout):
                    response = fixture_response(request)
                    response["ack"][field] = "wrong"
                    return response
                result = NativeOwner(self.c, wrong).run("first", "poll")
                self.assertEqual("unavailable", result["observation"])
                self.assertEqual("dispatched", result["core_status"])
                self.assertNotIn("ack", self.owner.journal("first"))

    def test_wrong_response_identity_request_or_binding_is_held(self):
        self.ready()
        for field in ("agent_id", "request_id", "binding"):
            request = self.host.calls[-1]
            response = fixture_response(request)
            response[field] = {} if field == "binding" else "wrong"
            with self.assertRaises(Rejected):
                self.owner._receipt("first", request, response)
        self.owner.adapter = lambda request, timeout: {**fixture_response(request), "agent_id": "wrong"}
        result = self.owner.run("first", "start-work")
        self.assertEqual("reconciliation_required", result["status"])
        self.assertNotIn("response", self.owner.journal("first")["start-work"])

    def test_intent_before_effect_and_effect_before_receipt_crashes_all_effects(self):
        for operation in ("spawn", "start-work", "close"):
            with self.subTest(operation=operation):
                if operation == "start-work":
                    self.owner.run("first", "poll")
                elif operation == "close":
                    self.host.status = "returned"
                    self.owner.run("first", "poll")
                    self.host.status = None
                def crash(request, timeout):
                    durable = Coordinator(self.directory)
                    row = NativeOwner(durable, self.host).journal("first")[operation]
                    self.assertEqual(request, row["request"])
                    self.assertNotIn("response", row)
                    self.host.calls.append(request)  # fixture effect occurred
                    raise SystemExit("fixture crash")
                self.owner.adapter = crash
                with self.assertRaises(SystemExit):
                    self.owner.run("first", operation)
                count = len(self.host.calls)
                self.restart()
                self.assertEqual("reconciliation_required", self.owner.run("first", operation)["status"])
                self.assertEqual(count, len(self.host.calls))
                request = self.owner.journal("first")[operation]["request"]
                response = fixture_response(request)
                self.owner.reconcile("first", operation, response, {"fixture_lookup": "effect confirmed"})
                self.owner.reconcile("first", operation, response, {"fixture_lookup": "effect confirmed"})
                self.owner.run("first", operation)
                self.assertEqual(count, len(self.host.calls))

    def test_receipt_committed_before_core_projection_crash_and_restart(self):
        for operation in ("spawn", "poll", "start-work", "close"):
            with self.subTest(operation=operation):
                if operation == "close":
                    self.host.status = "returned"
                    self.owner.run("first", "poll")
                    self.host.status = None
                original = self.owner._sync
                calls = []
                def crash_after_receipt(action_id):
                    calls.append(action_id)
                    if len(calls) == 2:
                        raise SystemExit("fixture: durable receipt, no core update")
                    original(action_id)
                with patch.object(self.owner, "_sync", side_effect=crash_after_receipt):
                    with self.assertRaises(SystemExit):
                        self.owner.run("first", operation)
                count = len(self.host.calls)
                self.restart()
                self.owner._sync("first")
                self.assertEqual(count, len(self.host.calls))
                if operation == "poll":
                    self.assertEqual("running", self.owner.inspect("first")["core_status"])
                else:
                    self.owner.run("first", operation)
                    self.assertEqual(count, len(self.host.calls))

    def test_core_projection_committed_before_process_crash_is_idempotent(self):
        self.working()
        self.host.status = "returned"
        real_returned = self.c.returned
        def crash(*args):
            real_returned(*args)
            raise SystemExit("fixture after core commit")
        with patch.object(self.c, "returned", side_effect=crash):
            with self.assertRaises(SystemExit):
                self.owner.run("first", "poll")
        self.restart()
        before = self.c.snapshot()["revision"]
        self.assertEqual("awaiting_owner_verification", self.owner.run("first", "poll")["status"])
        self.assertEqual(before, self.c.snapshot()["revision"])

    def test_pause_between_ack_and_work_global_and_stream(self):
        self.ready()
        self.c.set_enabled(False, {"fixture_pause": True})
        with self.assertRaisesRegex(Rejected, "paused"):
            self.owner.run("first", "start-work")
        self.c.set_enabled(True, {"fixture_only": True})
        self.c.set_stream_mode("delivery", "paused", self.c.snapshot()["revision"], {"fixture_only": True})
        with self.assertRaisesRegex(Rejected, "paused"):
            self.owner.run("first", "start-work")
        self.assertEqual(2, len(self.host.calls))
        self.assertNotIn("start-work", self.owner.journal("first"))
        self.host.status = "running"
        self.assertEqual("running", self.owner.run("first", "poll")["native_status"])

    def test_stale_dependency_resource_and_reservation_checks_before_work(self):
        self.ready()
        baseline = self.c.snapshot()
        mutations = [lambda s: s["allocations"]["bootstrap"].update(scope=["wrong"]),
                     lambda s: s["sprints"]["PF80"].update(dependencies=["PF81"]),
                     lambda s: s["sprints"]["PF80"].update(status="draft"),
                     lambda s: s["actions"].update(other={**s["actions"]["first"], "id": "other"})]
        for mutate in mutations:
            with self.c.mutation("fixture_fault", {}) as (_, state):
                state.clear()
                state.update(copy.deepcopy(baseline))
                mutate(state)
            with self.assertRaises(Rejected):
                self.owner.run("first", "start-work")
        self.assertEqual(2, len(self.host.calls))

    def test_pause_after_intent_holds_without_effect(self):
        self.ready()
        original = self.owner._gate
        calls = []
        def gate(state, action):
            calls.append(1)
            if len(calls) == 2:
                raise Rejected("fixture pause immediately before effect")
            original(state, action)
        with patch.object(self.owner, "_gate", side_effect=gate):
            with self.assertRaises(Rejected):
                self.owner.run("first", "start-work")
        self.restart()
        self.assertEqual("reconciliation_required", self.owner.run("first", "start-work")["status"])
        self.assertEqual(2, len(self.host.calls))

    def test_running_timeout_restart_and_transport_failure_do_not_terminate(self):
        self.working()
        self.restart()
        for status in ("running", "timed_out", "running"):
            self.host.status = status
            result = self.owner.run("first", "poll")
            self.assertEqual(status, result["native_status"])
            self.assertEqual("running", result["core_status"])
        self.owner.adapter = lambda request, timeout: (_ for _ in ()).throw(TimeoutError("PRIVATE SECRET"))
        result = self.owner.run("first", "poll")
        self.assertEqual("running", result["core_status"])
        self.assertNotIn("PRIVATE", json.dumps(result))
        with self.assertRaises(Rejected):
            self.owner.run("first", "close")

    def test_native_failure_and_premature_return(self):
        self.owner.run("first", "spawn")
        self.host.status = "returned"
        self.assertEqual("dispatched", self.owner.run("first", "poll")["core_status"])
        self.host.status = "failed"
        result = self.owner.run("first", "poll")
        self.assertEqual("native_failed", result["status"])
        self.assertEqual("failed", result["core_status"])
        self.assertEqual("failed", result["native_status"])

    def test_work_failure_requires_owner_verification(self):
        result = self.returned("failed")
        self.assertEqual("awaiting_owner_verification", result["status"])
        self.assertEqual("failed", result["native_status"])
        self.assertEqual("returned", result["core_status"])

    def test_core_claim_gap_and_watchdog_reconcile_never_respawn(self):
        action = self.c.claim("first")
        self.restart()
        with self.assertRaises(Rejected):
            self.owner.run("first", "spawn")
        self.assertEqual("reconciliation_required", self.owner.inspect("first")["status"])
        self.c.clock = lambda: action["deadline"] + 1
        self.c.watchdog()
        request = {"request_id": "fixture-recovered", "binding": binding(action), "agent_id": None, "operation": "spawn"}
        result = self.owner.reconcile("first", "spawn", fixture_response(request), {"fixture_lookup": True})
        self.assertEqual("dispatched", result["core_status"])
        self.assertEqual([], self.host.calls)

    def test_stdio_subprocess_restart_and_real_timeout(self):
        def invoke(operation, status=None, incomplete=False):
            process = subprocess.Popen([sys.executable, "-B", str(CLI), operation,
                                        "--state", str(self.directory), "--timeout", "0.15"],
                                       stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                process.stdin.write(b'{"action_id":"first"}\n')
                process.stdin.flush()
                request = json.loads(process.stdout.readline())
                self.assertEqual("native_request", request["type"])
                if incomplete:
                    process.stdin.write(b'{"partial":')
                else:
                    process.stdin.write((json.dumps(fixture_response(request, status)) + "\n").encode())
                process.stdin.flush()
                result = json.loads(process.stdout.readline())
                self.assertEqual(2 if incomplete or status == "timed_out" else 0, process.wait(timeout=3))
                return result
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait()
                for stream in (process.stdin, process.stdout, process.stderr):
                    stream.close()
        invoke("spawn")
        invoke("poll")
        invoke("start-work")
        self.assertEqual("running", invoke("poll", "timed_out")["core_status"])
        self.assertEqual("unavailable", invoke("poll", incomplete=True)["observation"])
        self.assertEqual("running", self.owner.inspect("first")["core_status"])

    def test_stdio_frame_size_and_eof(self):
        read, write = os.pipe()
        try:
            os.write(write, b"{}\n")
            self.assertEqual({}, StdioHost(read, None).read(1))
            os.close(write)
            write = None
            with self.assertRaises(Rejected):
                StdioHost(read, None).read(1)
        finally:
            os.close(read)
            if write is not None:
                os.close(write)

    def test_oversize_and_blocked_stdio_are_bounded(self):
        with tempfile.TemporaryFile(dir=self.tmp.name) as source:
            source.write(b"x" * 65538)
            source.seek(0)
            with self.assertRaisesRegex(Rejected, "exceeds"):
                StdioHost(source.fileno(), None).read(2)
        read, write = os.pipe()
        try:
            os.set_blocking(write, False)
            while True:
                try:
                    os.write(write, b"x" * 4096)
                except BlockingIOError:
                    break
            with os.fdopen(os.dup(write), "w") as sink:
                with self.assertRaises(TimeoutError):
                    StdioHost(read, sink).emit({"fixture": True}, 0.01)
        finally:
            os.close(read)
            os.close(write)

    def test_concurrent_duplicate_cannot_emit_second_effect(self):
        entered, finish = threading.Event(), threading.Event()
        def held(request, timeout):
            self.host.calls.append(request)
            entered.set()
            self.assertTrue(finish.wait(2))
            return fixture_response(request)
        self.owner.adapter = held
        results = []
        worker = threading.Thread(target=lambda: results.append(self.owner.run("first", "spawn")))
        worker.start()
        try:
            self.assertTrue(entered.wait(2))
            other = NativeOwner(Coordinator(self.directory), self.host)
            self.assertEqual("reconciliation_required", other.run("first", "spawn")["status"])
            self.assertEqual(1, len(self.host.calls))
        finally:
            finish.set()
            worker.join(3)
        self.assertEqual("awaiting_ack", results[0]["status"])

    def test_reused_identity_and_conflicting_reconciliation_denied(self):
        self.returned()
        self.c.verify("first", {"fixture_owner_verification": True}, True)
        self.prepare("second")
        self.host.status = None
        self.assertEqual("reconciliation_required", self.owner.run("second", "spawn")["status"])
        request = self.owner.journal("second")["spawn"]["request"]
        response = {**fixture_response(request), "agent_id": "fixture-agent-two"}
        self.owner.reconcile("second", "spawn", response, {"fixture_lookup": True})
        with self.assertRaises(Rejected):
            self.owner.reconcile("second", "spawn", {**response, "agent_id": "wrong"}, {"fixture_lookup": True})

    def test_spawn_and_close_obey_pause_and_owner_kinds_not_delegated(self):
        self.c.set_enabled(False, {"fixture_pause": True})
        with self.assertRaises(Rejected):
            self.owner.run("first", "spawn")
        self.assertEqual([], self.host.calls)
        self.c.set_enabled(True, {"fixture_only": True})
        self.returned()
        self.c.set_enabled(False, {"fixture_pause": True})
        with self.assertRaises(Rejected):
            self.owner.run("first", "close")
        self.assertNotIn("close", self.owner.journal("first"))
        self.c.set_enabled(True, {"fixture_only": True})
        self.prepare("integration", "integrate")
        with self.assertRaisesRegex(Rejected, "not a native worker"):
            self.owner.run("integration", "spawn")

    def test_cli_help_and_invalid_payload_emit_no_native_request(self):
        for args, data, expected in [(["--help"], "", 0),
                                     (["inspect", "--state", str(self.directory)], '{"action_id":"first","adapter":"evil"}\n', 2),
                                     (["poll", "--state", str(self.directory)], 'NaN\n', 2)]:
            result = subprocess.run([sys.executable, "-B", str(CLI), *args], input=data,
                                    text=True, capture_output=True, timeout=3)
            self.assertEqual(expected, result.returncode)
            self.assertNotIn('"type":"native_request"', result.stdout)


if __name__ == "__main__":
    unittest.main()

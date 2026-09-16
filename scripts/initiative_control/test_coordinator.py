import copy
import io
import json
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from pathlib import Path

from coordinator import Coordinator, Rejected, digest
import coordinator_cli


def seed():
    streams = {"security": {"sprint": "PF27", "mode": "paused"},
               "accounting": {"sprint": "PF60", "mode": "paused"},
               "delivery": {"sprint": "PF80", "mode": "enabled"}}
    sprints = {value["sprint"]: {"workstream": key, "status": "in_progress",
                                "dependencies": [], "archived": False}
               for key, value in streams.items()}
    sprints["PF81"] = {"workstream": "delivery", "status": "draft", "dependencies": ["PF80"], "archived": False}
    allocations = {"bootstrap": {"sprint": "PF80", "kinds": ["repair", "review", "integrate"],
                                "resources": ["integration-writer"], "scope": ["scripts/initiative_control/"],
                                "inputs": {"base": "a" * 40}, "timeout_seconds": 60}}
    return streams, sprints, allocations


class CoordinatorTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.clock = 1000.0
        self.root = Path(self.tmp.name) / "private"
        self.c = Coordinator(self.root, lambda: self.clock)
        self.c.initialize(*seed())
        self.c.set_enabled(True, {"authority": "bounded bootstrap rehearsal"})

    def tearDown(self):
        self.tmp.cleanup()

    def prepared(self, name="first"):
        self.c.event({"id": "trigger-" + name, "reason": "bounded work"})
        packet = self.c.begin_manager()
        action = {"id": name, "kind": "repair", "workstream": "delivery", "sprint": "PF80",
                  "rationale": "qualify controller", "expected_revision": packet["state_revision"],
                  "inputs": {"allocation": "bootstrap", "base": "a" * 40}, "timeout_seconds": 60}
        self.c.accept_decision(packet["manager_run"], {"state_revision": packet["state_revision"], "actions": [action]},
                               {"artifact": "verified-test-manager.json"})
        return action

    def test_duplicate_event_does_not_change_revision_and_conflict_denied(self):
        event = {"id": "one", "payload": "first"}
        self.assertTrue(self.c.event(event))
        before = self.c.snapshot()
        self.assertFalse(self.c.event(event))
        self.assertEqual(before, self.c.snapshot())
        with self.assertRaises(Rejected):
            self.c.event({"id": "one", "payload": "different"})

    def test_fresh_connection_preserves_all_three_and_event(self):
        self.c.event({"id": "restart"})
        other = Coordinator(self.root, lambda: self.clock)
        self.assertEqual(self.c.snapshot(), other.snapshot())
        self.assertEqual(3, len(other.begin_manager()["workstreams"]))
        with self.assertRaises(Rejected):
            other.initialize(*seed())

    def test_manager_exclusive_and_stale_decision_retains_event(self):
        self.c.event({"id": "one"})
        packet = self.c.begin_manager()
        with self.assertRaises(Rejected):
            self.c.begin_manager()
        self.c.event({"id": "new-human-input"})
        with self.assertRaisesRegex(Rejected, "stale"):
            self.c.accept_decision(packet["manager_run"], {"state_revision": packet["state_revision"], "actions": []}, {"proof": True})
        self.c.fail_manager(packet["manager_run"], "stale; actual launcher stopped")
        next_packet = self.c.begin_manager()
        self.assertIn("one", [e["id"] for e in next_packet["events"]])

    def test_owner_restricts_only_pending_prefix_and_preserves_it_after_restart(self):
        for index in range(30):
            self.c.event({"id": f"batch-{index}"})
        packet = self.c.begin_manager()
        self.assertEqual(30, packet["pending_event_count"])
        self.assertEqual(24, len(packet["events"]))
        before = self.c.snapshot()
        revision = self.c.restrict_manager_events(packet["manager_run"], 2, packet["state_revision"], {"fixture": True})
        reopened = Coordinator(self.root, lambda: self.clock)
        after = reopened.snapshot()
        self.assertEqual(before["manager"]["events"][:2], after["manager"]["events"])
        self.assertEqual(before["manager"]["deadline"], after["manager"]["deadline"])
        self.assertEqual(revision, after["manager"]["revision"])
        for key in ("workstreams", "sprints", "allocations", "actions"):
            self.assertEqual(before[key], after[key])
        with reopened.connection() as db:
            self.assertEqual(30, db.execute("SELECT COUNT(*) FROM events WHERE consumed IS NULL").fetchone()[0])
        with self.assertRaisesRegex(Rejected, "stale"):
            reopened.accept_decision(packet["manager_run"], {"state_revision": packet["state_revision"], "actions": []}, {"fixture": True})
        with self.assertRaisesRegex(Rejected, "stale"):
            reopened.restrict_manager_events(packet["manager_run"], 1, packet["state_revision"], {"fixture": True})

    def test_batch_selection_denials_roll_back_every_table(self):
        self.c.event({"id": "one"})
        self.c.event({"id": "two"})
        packet = self.c.begin_manager()
        def rows():
            with self.c.connection() as db:
                return list(db.iterdump())
        def deny(run, count, revision, evidence):
            before = rows()
            with self.assertRaises(Rejected):
                self.c.restrict_manager_events(run, count, revision, evidence)
            self.assertEqual(before, rows())
        run, revision = packet["manager_run"], packet["state_revision"]
        for count in (True, None, 0, -1, 2, 25):
            deny(run, count, revision, {"fixture": True})
        deny("wrong-run", 1, revision, {"fixture": True})
        deny(run, 1, revision, {})
        deny(run, 1, True, {"fixture": True})
        self.clock += 601
        deny(run, 1, revision, {"fixture": True})
        self.clock -= 601
        self.c.set_enabled(False, {"fixture": "pause"})
        deny(run, 1, self.c.snapshot()["revision"], {"fixture": True})
        self.c.set_enabled(True, {"fixture": "resume"})
        deny(run, 1, revision, {"fixture": True})

    def test_lifecycle_claim_ack_return_verify(self):
        self.prepared()
        claim = self.c.claim("first")
        with self.assertRaises(Rejected):
            self.c.claim("first")
        self.c.dispatched("first", claim["claim"], "agent-1", {"native": "dispatch-1"})
        with self.assertRaises(Rejected):
            self.c.acknowledge("first", "wrong-agent", claim["allocation_digest"], {"ack": True})
        with self.assertRaises(Rejected):
            self.c.acknowledge("first", "agent-1", "wrong-digest", {"ack": True})
        self.c.acknowledge("first", "agent-1", claim["allocation_digest"], {"ack": "actual-bound-message"})
        self.c.returned("first", "agent-1", {"candidate": "b" * 40, "claim": "tests pass"})
        self.assertEqual("returned", self.c.snapshot()["actions"]["first"]["status"])
        self.c.verify("first", {"receiving_test_receipt": "owner-checked.json"}, True)
        self.assertEqual("accepted", self.c.snapshot()["actions"]["first"]["status"])
        self.assertEqual("in_progress", self.c.snapshot()["sprints"]["PF80"]["status"])

    def test_uncertain_dispatch_never_retries_after_restart(self):
        self.prepared()
        self.c.claim("first")
        self.clock += 61
        reopened = Coordinator(self.root, lambda: self.clock)
        self.assertEqual(1, len(reopened.watchdog()))
        revision = reopened.snapshot()["revision"]
        self.assertEqual([], reopened.watchdog())
        self.assertEqual(revision, reopened.snapshot()["revision"])
        with self.assertRaises(Rejected):
            reopened.claim("first")
        reopened.reconcile_dispatch("first", {"native_inspection": "agent exists"}, "agent-1")
        self.assertEqual("dispatched", reopened.snapshot()["actions"]["first"]["status"])

    def test_resource_held_until_verification(self):
        self.prepared()
        self.prepared("second")
        self.c.claim("first")
        with self.assertRaisesRegex(Rejected, "resource owned"):
            self.c.claim("second")

    def test_pause_prevents_prepared_dispatch(self):
        self.prepared()
        self.c.set_enabled(False, {"human": "pause"})
        with self.assertRaisesRegex(Rejected, "paused"):
            self.c.claim("first")

    def test_manager_watchdog_once_without_relaunch(self):
        self.c.event({"id": "one"})
        self.c.begin_manager(timeout_seconds=2)
        self.clock += 3
        self.assertEqual(1, len(self.c.watchdog()))
        revision = self.c.snapshot()["revision"]
        self.assertEqual([], self.c.watchdog())
        self.assertEqual(revision, self.c.snapshot()["revision"])
        with self.assertRaises(Rejected):
            self.c.begin_manager()

    def test_manager_cannot_expand_frozen_inputs(self):
        self.c.event({"id": "one"})
        packet = self.c.begin_manager()
        action = {"id": "bad", "kind": "repair", "workstream": "delivery", "sprint": "PF80",
                  "rationale": "bad", "expected_revision": packet["state_revision"], "timeout_seconds": 60,
                  "inputs": {"allocation": "bootstrap", "base": "a" * 40, "shell": "unapproved"}}
        with self.assertRaisesRegex(Rejected, "frozen"):
            self.c.accept_decision(packet["manager_run"], {"state_revision": packet["state_revision"], "actions": [action]}, {"proof": True})
        self.assertEqual({}, self.c.snapshot()["actions"])

    def test_draft_successor_not_executable(self):
        state = self.c.snapshot()
        self.assertFalse(Coordinator._dependencies(state, "PF81"))
        state["sprints"]["PF80"].update(status="completed", archived=False)
        self.assertFalse(Coordinator._dependencies(state, "PF81"))
        state["sprints"]["PF80"]["archived"] = True
        self.assertTrue(Coordinator._dependencies(state, "PF81"))

    def test_invalid_directory_and_json_rejected(self):
        self.root.chmod(0o755)
        with self.assertRaisesRegex(Rejected, "owner-only"):
            Coordinator(self.root)
        with self.assertRaises(Rejected):
            digest({"notfinite": float("nan")})

    def test_cycle_rejected(self):
        streams, sprints, allocations = copy.deepcopy(seed())
        sprints["PF80"]["dependencies"] = ["PF81"]
        second = Coordinator(Path(self.tmp.name) / "second")
        with self.assertRaisesRegex(Rejected, "cycle"):
            second.initialize(streams, sprints, allocations)

    def test_cli_reopen_snapshot_and_oversize_rejection(self):
        output = io.StringIO()
        code = coordinator_cli.main(["snapshot", "--state", str(self.root)], io.StringIO("{}"), output)
        self.assertEqual(0, code)
        self.assertEqual(3, len(json.loads(output.getvalue())["result"]["workstreams"]))
        output = io.StringIO()
        self.assertEqual(2, coordinator_cli.main(["event", "--state", str(self.root)], io.StringIO("x" * 262145), output))
        self.assertIn("exceeds", output.getvalue())

    def test_history_archived_and_briefing_order_not_identifier_order(self):
        for name in ["zulu", "yankee", "xray", "alpha"]:
            self.prepared(name)
            claim = self.c.claim(name)
            self.c.dispatched(name, claim["claim"], "agent-1", {"native": name})
            self.c.acknowledge(name, "agent-1", claim["allocation_digest"], {"ack": name})
            self.c.returned(name, "agent-1", {"candidate": name})
            self.c.verify(name, {"owner-check": name}, True)
        state = self.c.snapshot()
        self.assertNotIn("zulu", state["actions"])
        with self.c.connection() as db:
            archived = json.loads(db.execute("SELECT body FROM action_history WHERE id='zulu'").fetchone()[0])
        self.assertEqual("accepted", archived["status"])
        packet = self.c.begin_manager()
        self.assertEqual(["yankee", "xray", "alpha"], [a["id"] for a in packet["last_three_actions"]["delivery"]])

    def test_reconciled_dispatch_stalls_again_under_new_epoch(self):
        self.prepared()
        self.c.claim("first")
        self.clock += 61
        first = self.c.watchdog()[0]
        self.c.reconcile_dispatch("first", {"native": "found running worker"}, "agent-1")
        self.clock += 61
        second = self.c.watchdog()[0]
        self.assertNotEqual(first["id"], second["id"])
        self.assertEqual("dispatched", second["status"])
        self.assertEqual([], self.c.watchdog())

    def test_owner_can_reconcile_interrupted_dispatch_immediately(self):
        self.prepared()
        self.c.claim("first")
        reopened = Coordinator(self.root, lambda: self.clock)
        reopened.reconcile_dispatch("first", {"native": "confirmed no dispatch occurred"})
        self.assertEqual("failed", reopened.snapshot()["actions"]["first"]["status"])

    def test_long_manager_id_rejected_before_claim(self):
        with self.assertRaisesRegex(Rejected, "too long"):
            self.prepared("a" * 121)
        self.assertEqual({}, self.c.snapshot()["actions"])

    def test_large_event_referenced_without_losing_original(self):
        self.c.event({"id": "large", "payload": "x" * 241000})
        packet = self.c.begin_manager()
        self.assertLess(len(json.dumps(packet)), 10000)
        original = self.c.read_evidence(packet["events"][0]["evidence_digest"])
        self.assertEqual("x" * 241000, original["payload"])

    def test_malformed_manager_input_is_json_rejection(self):
        self.c.event({"id": "trigger"})
        packet = self.c.begin_manager()
        payload = {"run_id": packet["manager_run"], "launcher_receipt": {"proof": True},
                   "decision": {"state_revision": packet["state_revision"], "actions": [
                       {"id": "malformed", "kind": "repair", "rationale": "test", "inputs": [],
                        "expected_revision": packet["state_revision"]}]}}
        output = io.StringIO()
        self.assertEqual(2, coordinator_cli.main(["accept_decision", "--state", str(self.root)],
                                               io.StringIO(json.dumps(payload)), output))
        self.assertEqual("rejected", json.loads(output.getvalue())["status"])

    def test_manager_extra_operational_fields_rejected(self):
        self.c.event({"id": "trigger"})
        packet = self.c.begin_manager()
        action = {"id": "injected", "kind": "repair", "workstream": "delivery", "sprint": "PF80",
                  "rationale": "test", "expected_revision": packet["state_revision"],
                  "inputs": {"allocation": "bootstrap", "base": "a" * 40}, "timeout_seconds": 60,
                  "stall_reported": True}
        with self.assertRaisesRegex(Rejected, "unexpected action fields"):
            self.c.accept_decision(packet["manager_run"], {"state_revision": packet["state_revision"], "actions": [action]}, {"proof": True})

    def test_large_worker_result_cannot_wedge_next_manager(self):
        self.prepared()
        claim = self.c.claim("first")
        self.c.dispatched("first", claim["claim"], "agent-1", {"native": "dispatch"})
        self.c.acknowledge("first", "agent-1", claim["allocation_digest"], {"ack": True})
        self.c.returned("first", "agent-1", {"logs": "x" * 120000})
        packet = self.c.begin_manager()
        self.assertLess(len(json.dumps(packet)), 10000)
        result_ref = packet["actions"]["first"]["result"]["evidence_digest"]
        self.assertEqual("x" * 120000, self.c.read_evidence(result_ref)["logs"])

    def test_watchdog_rechecks_new_deadline_inside_transaction(self):
        self.prepared()
        self.c.claim("first")
        self.clock += 61
        snapshot = self.c.snapshot
        def concurrent_reconcile():
            old = snapshot()
            self.c.reconcile_dispatch("first", {"native": "found worker"}, "agent-1")
            return old
        with patch.object(self.c, "snapshot", side_effect=concurrent_reconcile):
            self.assertEqual([], self.c.watchdog())
        self.assertFalse(self.c.snapshot()["actions"]["first"].get("stall_reported"))
        self.clock += 61
        self.assertEqual(1, len(self.c.watchdog()))

    def test_oversized_manager_rationale_cannot_enter_state(self):
        self.c.event({"id": "trigger"})
        packet = self.c.begin_manager()
        action = {"id": "big", "kind": "repair", "workstream": "delivery", "sprint": "PF80",
                  "rationale": "x" * 125000, "expected_revision": packet["state_revision"],
                  "inputs": {"allocation": "bootstrap", "base": "a" * 40}, "timeout_seconds": 60}
        with self.assertRaisesRegex(Rejected, "bounded text"):
            self.c.accept_decision(packet["manager_run"], {"state_revision": packet["state_revision"], "actions": [action]}, {"proof": True})
        self.assertEqual({}, self.c.snapshot()["actions"])

    def test_owner_records_dead_worker_without_impersonating_return(self):
        self.prepared()
        self.prepared("second")
        claim = self.c.claim("first")
        self.c.dispatched("first", claim["claim"], "agent-1", {"native": "dispatch"})
        self.c.acknowledge("first", "agent-1", claim["allocation_digest"], {"ack": True})
        self.c.reconcile_dispatch("first", {"native": "worker confirmed terminated"})
        action = self.c.snapshot()["actions"]["first"]
        self.assertEqual("failed", action["status"])
        self.assertIn("owner_failure", action)
        self.assertNotIn("result", action)
        self.assertEqual("dispatching", self.c.claim("second")["status"])

    def test_found_worker_then_dead_worker_reconciliations_are_distinct(self):
        self.prepared()
        self.prepared("second")
        self.c.claim("first")
        self.c.reconcile_dispatch("first", {"native": "found agent after interruption"}, "agent-1")
        self.c.acknowledge("first", "agent-1", self.c.snapshot()["actions"]["first"]["allocation_digest"], {"ack": True})
        self.c.reconcile_dispatch("first", {"native": "worker later confirmed dead"})
        self.assertEqual("failed", self.c.snapshot()["actions"]["first"]["status"])
        self.assertEqual("dispatching", self.c.claim("second")["status"])
        with self.c.connection() as db:
            self.assertEqual(2, db.execute("SELECT COUNT(*) FROM events WHERE id LIKE 'dispatch-reconciled:first:%'").fetchone()[0])


class OwnerControlsTests(unittest.TestCase):
    """Synthetic owner/native receipts exercise the CLI; no live acceptance claimed."""

    def setUp(self):
        CoordinatorTests.setUp(self)
        self.c = Coordinator(self.root)

    tearDown = CoordinatorTests.tearDown

    def database_rows(self):
        with self.c.connection() as db:
            return {t: [tuple(row) for row in db.execute("SELECT * FROM " + t + " ORDER BY 1")]
                    for t in ("state", "events", "audit", "action_history", "evidence", "sqlite_sequence")}

    def call(self, operation, error=None, **payload):
        before = self.c.snapshot()
        rows = self.database_rows()
        output = io.StringIO()
        code = coordinator_cli.main([operation, "--state", str(self.root)],
                                    io.StringIO(json.dumps(payload)), output)
        result = json.loads(output.getvalue())
        self.assertEqual(2 if error else 0, code, result)
        if error:
            self.assertIn(error, result["error"])
            self.assertEqual(before, self.c.snapshot())
            self.assertEqual(rows, self.database_rows())
        return result.get("result")

    def owner(self, operation, **payload):
        return self.call(operation, **{"expected_revision": self.c.snapshot()["revision"],
                                      "evidence": {"owner": "synthetic test inspection"}, **payload})

    def allocation(self, key, kind, inputs, sprint="PF80", **kwargs):
        allocation = {"sprint": sprint, "kinds": [kind], "inputs": inputs,
                      "resources": [], "scope": ["scripts/initiative_control/"], "timeout_seconds": 60}
        self.owner("put_allocation", allocation_id=key, allocation=allocation, replace=False, **kwargs)
        return allocation

    def action(self, key, allocation, finish=True, verification=None):
        frozen = self.c.snapshot()["allocations"][allocation]
        self.call("event", event={"id": "trigger-" + key})
        packet = self.call("begin_manager")
        action = {"id": key, "kind": frozen["kinds"][0], "sprint": frozen["sprint"],
                  "workstream": "delivery", "inputs": {"allocation": allocation, **frozen["inputs"]},
                  "rationale": "synthetic CLI exercise", "timeout_seconds": 60,
                  "expected_revision": packet["state_revision"]}
        self.call("accept_decision", run_id=packet["manager_run"],
                  decision={"state_revision": packet["state_revision"], "actions": [action]},
                  launcher_receipt={"fixture": "not a live launcher"})
        if finish:
            claim = self.call("claim", action_id=key)
            self.call("dispatched", action_id=key, claim=claim["claim"], agent_id="fixture-agent",
                      native_receipt={"fixture": "dispatch"})
            self.call("acknowledge", action_id=key, agent_id="fixture-agent",
                      allocation_digest=claim["allocation_digest"], native_receipt={"fixture": "ack"})
            self.call("returned", action_id=key, agent_id="fixture-agent", result={"claim": "all gates passed"})
            if verification is not False:
                self.call("verify", action_id=key, evidence=verification or {"fixture": "owner inspection"}, accepted=True)
        return action

    def closure(self, receipt_change=None, completion_verified=True, prepared=False, receiving_receipt=None):
        assignment = {"id": "fixture-merge", "base": "a" * 40, "source": "b" * 40,
                      "branch": "fixture-receiving", "scope": ["scripts/initiative_control/"],
                      "approval": {"fixture": True}, "tests": [{"argv": ["fixture-test"], "timeout_seconds": 60}]}
        receipt = {"status": "verified", "receiving_commit": "c" * 40, "assignment": assignment,
                   "tests": [{"argv": ["fixture-test"], "exit_code": 0, "timed_out": False}]}
        if receiving_receipt:
            receipt, assignment = receiving_receipt, receiving_receipt["assignment"]
        self.allocation("receive", "integrate", {"assignment": assignment})
        if receipt_change:
            receipt_change(receipt)
        self.action("receiving", "receive", verification={"receiving_receipt": receipt})
        self.allocation("close", "complete_sprint", {"receiving_action": "receiving",
                        "receiving_commit": receiving_receipt["receiving_commit"] if receiving_receipt else "c" * 40,
                        "mandatory_gates": ["review", "functional", "human"]})
        self.action("completion", "close", finish=not prepared, verification=None if completion_verified else False)
        return {name: {"owner_verified": True, "status": "passed", "evidence": {"fixture": name}}
                for name in ("review", "functional", "human")}

    def test_allocation_replace_cancels_prepared_and_retains_versions_on_restart(self):
        self.action("old", "bootstrap", finish=False)
        old = self.c.snapshot()["allocations"]["bootstrap"]
        new = {**old, "scope": ["scripts/initiative_control/coordinator.py"]}
        revision = self.c.snapshot()["revision"]
        self.owner("put_allocation", allocation_id="bootstrap", allocation=new, replace=True)
        self.c = Coordinator(self.root)
        self.call("claim", action_id="old", error="already claimed")
        self.owner("put_allocation", allocation_id="bootstrap", allocation=old, replace=True,
                   expected_revision=revision, error="stale")
        action = self.c.snapshot()["actions"]["old"]
        versions = self.call("read_evidence", evidence_digest=action["owner_cancellation"]["evidence_digest"])
        self.assertEqual((old, new), (versions["before"], versions["after"]))
        self.action("new", "bootstrap")
        self.owner("put_allocation", allocation_id="bootstrap", allocation=new, replace=True, error="duplicate")

    def test_replacement_denied_through_active_returned_and_uncertain_reservations(self):
        self.action("active", "bootstrap", finish=False)
        frozen = self.c.snapshot()["allocations"]["bootstrap"]
        new = {**frozen, "timeout_seconds": 30}
        claim = self.call("claim", action_id="active")
        def denied():
            self.owner("put_allocation", allocation_id="bootstrap", allocation=new, replace=True, error="active reservation")
        denied()
        Coordinator(self.root, lambda: claim["deadline"] + 1).watchdog()
        denied()
        self.call("reconcile_dispatch", action_id="active", evidence={"fixture": "found"}, agent_id="fixture-agent")
        denied()
        self.call("acknowledge", action_id="active", agent_id="fixture-agent", allocation_digest=claim["allocation_digest"],
                  native_receipt={"fixture": True})
        denied()
        self.call("returned", action_id="active", agent_id="fixture-agent", result={"fixture": True})
        denied()
        self.call("verify", action_id="active", evidence={"fixture": True}, accepted=True)
        self.owner("put_allocation", allocation_id="bootstrap", allocation=new, replace=True)

    def test_owner_inputs_staleness_and_manager_authority(self):
        allocation = self.c.snapshot()["allocations"]["bootstrap"]
        self.owner("put_allocation", allocation_id="bootstrap", allocation=allocation, replace=False, error="exists")
        self.owner("put_allocation", allocation_id="absent", allocation=allocation, replace=True, error="missing")
        self.owner("put_allocation", allocation_id="fresh", allocation=allocation, replace=False,
                   expected_revision=True, error="owner revision")
        self.owner("put_allocation", allocation_id="fresh", allocation=allocation, replace=False, evidence={}, error="evidence")
        self.call("event", event={"id": "before-owner-edit"})
        packet = self.call("begin_manager")
        self.allocation("fresh", "repair", {"base": "b" * 40})
        self.call("accept_decision", run_id=packet["manager_run"], launcher_receipt={"fixture": True},
                  decision={"state_revision": packet["state_revision"], "actions": []}, error="stale")
        self.call("fail_manager", run_id=packet["manager_run"], reason="fixture launcher stopped")
        self.action("worker-claims", "fresh", verification=False)
        self.owner("complete_sprint", action_id="worker-claims", gates={}, error="accepted owner-verified")

    def test_stream_pause_reopen_and_global_pause_precedence(self):
        self.action("queued", "bootstrap", finish=False)
        self.owner("set_stream_mode", workstream="delivery", mode="paused")
        self.call("claim", action_id="queued", error="paused")
        self.owner("set_stream_mode", workstream="delivery", mode="paused", error="already")
        self.owner("set_stream_mode", workstream="delivery", mode="bogus", error="invalid")
        self.call("set_enabled", enabled=False, evidence={"fixture": "global pause"})
        self.owner("set_stream_mode", workstream="delivery", mode="enabled")
        self.call("claim", action_id="queued", error="paused")
        self.call("set_enabled", enabled=True, evidence={"fixture": "resume"})
        self.call("claim", action_id="queued")
        self.assertEqual("paused", self.c.snapshot()["workstreams"]["security"]["mode"])

    def test_completion_archive_successor_cli_restart_and_history(self):
        gates = self.closure()
        self.allocation("next", "prepare_successor", {"base": "c" * 40}, sprint="PF81")
        self.action("successor", "next")
        self.owner("activate_successor", action_id="successor", error="completed and archived")
        # Accepted closure/receiving actions age into SQLite history, still usable.
        for i in range(4):
            self.action("history-" + str(i), "bootstrap")
        self.assertNotIn("completion", self.c.snapshot()["actions"])
        self.owner("complete_sprint", action_id="completion", gates=gates)
        self.owner("complete_sprint", action_id="completion", gates=gates, error="not reserved")
        self.owner("activate_successor", action_id="successor", error="completed and archived")
        self.owner("set_stream_mode", workstream="delivery", mode="paused")
        self.owner("archive_sprint", sprint="PF80", error="paused")
        self.owner("set_stream_mode", workstream="delivery", mode="enabled")
        self.owner("archive_sprint", sprint="PF80")
        self.owner("archive_sprint", sprint="PF80", error="already archived")
        self.owner("set_stream_mode", workstream="delivery", mode="paused")
        self.owner("activate_successor", action_id="successor", error="paused")
        self.owner("set_stream_mode", workstream="delivery", mode="enabled")
        self.owner("activate_successor", action_id="successor")
        self.owner("activate_successor", action_id="successor", error="not draft")
        state = self.call("snapshot")
        self.assertEqual("PF81", state["workstreams"]["delivery"]["sprint"])
        self.assertEqual(3, sum(s["status"] in {"in_progress", "blocked"} for s in state["sprints"].values()))
        self.allocation("next-work", "repair", {"base": "c" * 40}, sprint="PF81")
        self.action("next-implementation", "next-work")

    def test_gate_pause_pending_and_stale_completion_denials(self):
        gates = self.closure()
        for bad in ({}, {**gates, "extra": gates["review"]},
                    {**gates, "human": {"owner_verified": False, "status": "passed", "evidence": {"claim": True}}},
                    {**gates, "human": {"owner_verified": True, "status": "passed", "evidence": {}}},
                    {**gates, "functional": {"owner_verified": True, "status": "not_applicable", "evidence": {"fixture": True}}}):
            self.owner("complete_sprint", action_id="completion", gates=bad, error="gate evidence")
        self.owner("archive_sprint", sprint="PF80", error="verified completion")
        self.owner("complete_sprint", action_id="completion", gates=gates, expected_revision=0, error="stale")
        self.owner("set_stream_mode", workstream="delivery", mode="paused")
        self.owner("complete_sprint", action_id="completion", gates=gates, error="paused")
        self.owner("set_stream_mode", workstream="delivery", mode="enabled")
        self.action("pending", "bootstrap", finish=False)
        self.owner("complete_sprint", action_id="completion", gates=gates, error="pending")

    def test_failed_wrong_or_incomplete_receiving_proof_cannot_complete(self):
        mutations = [lambda r: r.update(status="verification_failed"),
                     lambda r: r.update(receiving_commit="d" * 40),
                     lambda r: r.update(tests=[]),
                     lambda r: r["tests"][0].update(exit_code=1),
                     lambda r: r["tests"][0].update(timed_out=True),
                     lambda r: r["assignment"].update(source="d" * 40)]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                self.root = Path(self.tmp.name) / ("negative-" + str(index))
                self.c = Coordinator(self.root)
                self.c.initialize(*seed())
                self.c.set_enabled(True, {"fixture": True})
                gates = self.closure(mutate)
                self.owner("complete_sprint", action_id="completion", gates=gates, error="receiving")

    def test_legacy_stale_prepared_claim_and_changed_accepted_proof_refused(self):
        self.action("legacy-prepared", "bootstrap", finish=False)
        # Simulate a pre-controls owner edit in this disposable fixture database.
        with self.c.mutation("fixture_legacy_edit", {}) as (_, state):
            state["allocations"]["bootstrap"]["scope"] = ["different.py"]
        self.call("claim", action_id="legacy-prepared", error="stale allocation claim")
        frozen = self.c.snapshot()["allocations"]["bootstrap"]
        self.owner("put_allocation", allocation_id="bootstrap", allocation={**frozen, "timeout_seconds": 30}, replace=True)
        gates = self.closure()
        frozen = self.c.snapshot()["allocations"]["close"]
        self.owner("put_allocation", allocation_id="close", allocation={**frozen, "scope": ["changed.py"]}, replace=True)
        self.owner("complete_sprint", action_id="completion", gates=gates, error="stale allocation proof")

    def test_draft_cannot_execute_and_successor_requires_exact_receiving_base(self):
        self.allocation("draft-work", "repair", {"base": "c" * 40}, sprint="PF81")
        self.call("event", event={"id": "draft-implementation"})
        packet = self.call("begin_manager")
        action = {"id": "draft", "kind": "repair", "sprint": "PF81", "workstream": "delivery",
                  "inputs": {"allocation": "draft-work", "base": "c" * 40}, "rationale": "fixture",
                  "timeout_seconds": 60, "expected_revision": packet["state_revision"]}
        self.call("accept_decision", run_id=packet["manager_run"], launcher_receipt={"fixture": True},
                  decision={"state_revision": packet["state_revision"], "actions": [action]}, error="not reserved")
        self.call("fail_manager", run_id=packet["manager_run"], reason="fixture launcher stopped")
        gates = self.closure()
        self.owner("complete_sprint", action_id="completion", gates=gates)
        self.owner("archive_sprint", sprint="PF80")
        self.allocation("wrong-base", "prepare_successor", {"base": "d" * 40}, sprint="PF81")
        self.action("wrong-successor", "wrong-base")
        self.owner("activate_successor", action_id="wrong-successor", error="verified receiving commit")

    def test_seed_and_activation_enforce_reservation_limit_including_blocked(self):
        streams, sprints, allocations = seed()
        sprints["PF81"]["status"] = "blocked"
        self.root = Path(self.tmp.name) / "invalid-seed"
        output = io.StringIO()
        self.assertEqual(2, coordinator_cli.main(["initialize", "--state", str(self.root)],
                         io.StringIO(json.dumps(dict(workstreams=streams, sprints=sprints, allocations=allocations))), output))
        self.assertIn("three-reservation limit", output.getvalue())
        self.root = Path(self.tmp.name) / "private"
        gates = self.closure()
        self.owner("complete_sprint", action_id="completion", gates=gates)
        self.owner("archive_sprint", sprint="PF80")
        self.allocation("next", "prepare_successor", {"base": "c" * 40}, sprint="PF81")
        self.action("next", "next")
        # Legacy inconsistent state cannot reserve a second sprint in a stream.
        with self.c.mutation("fixture_legacy_reservations", {}) as (_, state):
            state["sprints"]["legacy"] = {"workstream": "security", "status": "blocked", "dependencies": []}
        self.owner("activate_successor", action_id="next", error="already reserved")

    def test_two_owner_connections_cannot_apply_same_revision(self):
        from concurrent.futures import ThreadPoolExecutor
        revision = self.c.snapshot()["revision"]
        def attempt(mode):
            output = io.StringIO()
            result = coordinator_cli.main(["set_stream_mode", "--state", str(self.root)],
                     io.StringIO(json.dumps({"workstream": mode, "mode": "enabled", "expected_revision": revision,
                                             "evidence": {"fixture": "competing owner connections"}})), output)
            return result, json.loads(output.getvalue())
        with ThreadPoolExecutor(max_workers=2) as pool:
            results = list(pool.map(attempt, ["security", "accounting"]))
        self.assertEqual([0, 2], sorted(code for code, _ in results))
        self.assertIn("stale owner revision", next(body["error"] for code, body in results if code == 2))
        self.assertEqual(revision + 1, self.c.snapshot()["revision"])

    def test_mandatory_gate_contract_and_explicit_owner_na(self):
        self.allocation("invalid-close", "complete_sprint", {"receiving_action": "receiving",
                        "receiving_commit": "c" * 40, "mandatory_gates": []}, error="mandatory gates")
        gates = self.closure(completion_verified=False)
        self.owner("complete_sprint", action_id="completion", gates=gates, error="accepted owner-verified")
        self.call("verify", action_id="completion", accepted=True, evidence={"fixture": "owner acceptance"})
        gates["functional"].update(status="not_applicable", reason="synthetic internal-only increment")
        self.owner("complete_sprint", action_id="completion", gates=gates)
        record = self.c.snapshot()["sprints"]["PF80"]["completion"]
        self.assertEqual(gates, self.call("read_evidence", evidence_digest=record["evidence_digest"])["gates"])

    def test_record_wait_while_paused_preserves_state_without_native_receipts_or_wakeup(self):
        self.allocation("waiting", "wait", {"reason": "operator pause"})
        self.action("wait-1", "waiting", finish=False)
        self.call("set_enabled", enabled=False, evidence={"owner": "pause"})
        self.owner("set_stream_mode", workstream="delivery", mode="paused")
        before = self.c.snapshot()
        with self.c.connection() as db:
            events = list(db.execute("SELECT body, meaningful, consumed FROM events"))
        self.owner("record_wait", action_id="wait-1", evidence={"observed": "waiting intentionally"})
        after = Coordinator(self.root).snapshot()
        self.assertEqual(before["revision"] + 1, after["revision"])
        for field in ("enabled", "workstreams", "sprints", "allocations", "manager"):
            self.assertEqual(before[field], after[field])
        action = after["actions"]["wait-1"]
        self.assertEqual("accepted", action["status"])
        self.assertEqual(before["actions"]["wait-1"]["rationale"], action["rationale"])
        self.assertFalse({"claim", "agent", "dispatch_receipt", "ack_receipt", "result"} & action.keys())
        proof = self.c.read_evidence(action["verification"]["evidence_digest"])
        self.assertEqual("wait_recorded", proof["kind"])
        self.assertIn("no work executed or blocker resolved", proof["meaning"])
        with self.c.connection() as db:
            self.assertEqual(events, list(db.execute("SELECT body, meaningful, consumed FROM events")))
            self.assertEqual(1, db.execute("SELECT COUNT(*) FROM audit WHERE operation='wait_recorded'").fetchone()[0])
        self.owner("record_wait", action_id="wait-1", error="prepared wait required")
        self.owner("complete_sprint", action_id="wait-1", gates={}, error="accepted owner-verified")

    def test_record_wait_rejects_other_kinds_stale_and_owned_state(self):
        from coordinator import KINDS
        for index, kind in enumerate(sorted(KINDS - {"wait", "complete_sprint"})):
            self.allocation("kind-" + str(index), kind, {})
            self.action("action-" + str(index), "kind-" + str(index), finish=False)
            self.owner("record_wait", action_id="action-" + str(index), error="prepared wait required")
        self.allocation("close-wait-test", "complete_sprint", {"receiving_action": "none",
            "receiving_commit": "b" * 40, "mandatory_gates": ["review"]})
        self.action("completion-wait-test", "close-wait-test", finish=False)
        self.owner("record_wait", action_id="completion-wait-test", error="prepared wait required")
        self.allocation("waiting", "wait", {})
        self.action("wait-1", "waiting", finish=False)
        self.owner("record_wait", action_id="wait-1", expected_revision=0, error="stale owner revision")
        self.owner("record_wait", action_id="wait-1", evidence={}, error="owner revision and evidence")
        self.call("event", event={"id": "another-cycle"})
        packet = self.call("begin_manager")
        self.owner("record_wait", action_id="wait-1", error="manager cycle already owned")
        self.call("fail_manager", run_id=packet["manager_run"], reason="offline fixture; no process")
        with self.c.mutation("legacy-fixture-change", {}) as (_, state):
            state["allocations"]["waiting"]["inputs"] = {"changed": True}
        self.owner("record_wait", action_id="wait-1", error="stale allocation proof")

    def test_record_wait_cannot_hide_claimed_or_uncertain_dispatch(self):
        self.allocation("waiting", "wait", {})
        self.action("wait-1", "waiting", finish=False)
        claim = self.call("claim", action_id="wait-1")
        self.owner("record_wait", action_id="wait-1", error="prepared wait required")
        self.c.clock = lambda: claim["deadline"] + 1
        self.c.watchdog()
        self.owner("record_wait", action_id="wait-1", error="prepared wait required")
        self.assertEqual("dispatch_uncertain", self.c.snapshot()["actions"]["wait-1"]["status"])

    def test_repeated_waits_archive_history_without_pending_queue_growth(self):
        self.allocation("waiting", "wait", {})
        for index in range(52):
            key = "wait-" + str(index)
            self.action(key, "waiting", finish=False)
            self.owner("record_wait", action_id=key)
        state = self.c.snapshot()
        self.assertEqual(3, len(state["actions"]))
        self.assertTrue(all(a["status"] == "accepted" for a in state["actions"].values()))
        with self.c.connection() as db:
            self.assertEqual(49, db.execute("SELECT COUNT(*) FROM action_history").fetchone()[0])
            self.assertEqual(0, db.execute("SELECT COUNT(*) FROM events WHERE meaningful=1 AND consumed IS NULL").fetchone()[0])
        self.call("begin_manager", error="no meaningful pending event")
        self.assertEqual(state, Coordinator(self.root).snapshot())

    def test_reallocated_wait_is_cancelled_and_cannot_be_recorded(self):
        allocation = self.allocation("waiting", "wait", {})
        self.action("wait-1", "waiting", finish=False)
        self.owner("put_allocation", allocation_id="waiting", replace=True,
                   allocation={**allocation, "inputs": {"reason": "changed scope"}})
        self.assertEqual("cancelled", self.c.snapshot()["actions"]["wait-1"]["status"])
        self.owner("record_wait", action_id="wait-1", error="prepared wait required")

    def test_record_wait_serializes_competing_owner_connections(self):
        from concurrent.futures import ThreadPoolExecutor
        self.allocation("waiting", "wait", {})
        self.action("wait-1", "waiting", finish=False)
        revision = self.c.snapshot()["revision"]
        def record(_):
            try:
                Coordinator(self.root).record_wait("wait-1", revision, {"fixture": "owner race"})
                return "accepted"
            except Rejected as exc:
                return str(exc)
        with ThreadPoolExecutor(max_workers=2) as pool:
            self.assertEqual(["accepted", "stale owner revision"], sorted(pool.map(record, range(2))))
        self.assertEqual(revision + 1, self.c.snapshot()["revision"])

    def prepared_successor(self, base="c" * 40):
        self.allocation("next", "prepare_successor", {"base": base}, sprint="PF81")
        self.action("successor", "next", finish=False)

    def assert_owner_effect(self, action_id, operation):
        action = self.c.snapshot()["actions"][action_id]
        self.assertEqual("accepted", action["status"])
        self.assertEqual(action["verification"], action["owner_effect"])
        self.assertFalse(set(action) & {"claim", "agent", "dispatch_receipt", "ack_receipt", "result"})
        receipt = self.c.read_evidence(action["owner_effect"]["evidence_digest"])
        self.assertEqual((operation, action_id, action["allocation_digest"]),
                         (receipt["operation"], receipt["action"], receipt["allocation_digest"]))
        return receipt

    def test_prepared_owner_lifecycle_from_real_receiving_tests_to_first_claim(self):
        # Real local Git merge/test receipt; native receiving-worker transport is a
        # labelled fixture. Neither owner lifecycle action uses that transport.
        from test_integration import IntegrationTests
        fixture = IntegrationTests()
        fixture.setUp()
        self.addCleanup(fixture.tearDown)
        receipt = fixture.runner.merge(fixture.assignment)
        self.assertEqual("verified", receipt["status"])
        base = receipt["receiving_commit"]
        gates = self.closure(prepared=True, receiving_receipt=receipt)
        self.prepared_successor(base)
        authority = {"owner": "fixture integrator", "decision": "activate exact PF81 allocation"}
        self.owner("activate_successor", action_id="successor", error="explicit owner activation authority")
        self.owner("activate_successor", action_id="successor", activation_authority=authority,
                   error="completed and archived")
        self.owner("archive_sprint", sprint="PF80", error="verified completion")
        with patch.object(Coordinator, "claim", side_effect=AssertionError("owner action cannot dispatch")):
            ref = self.owner("complete_sprint", action_id="completion", gates=gates)
            self.assertEqual(ref, self.c.snapshot()["actions"]["completion"]["owner_effect"])
            proof = self.assert_owner_effect("completion", "owner_complete")
            self.assertEqual(base, proof["effect"]["receiving_commit"])
            self.assertEqual(gates, self.c.read_evidence(proof["effect"]["completion"]["evidence_digest"])["gates"])
            self.owner("activate_successor", action_id="successor", activation_authority=authority,
                       error="completed and archived")
            self.owner("archive_sprint", sprint="PF80")
            self.owner("activate_successor", action_id="successor", activation_authority=authority)
        self.c = Coordinator(self.root)
        self.assertEqual(authority, self.assert_owner_effect("successor", "owner_successor")["effect"]["activation_authority"])
        self.owner("complete_sprint", action_id="completion", gates=gates, error="not reserved")
        self.owner("archive_sprint", sprint="PF80", error="already archived")
        self.owner("activate_successor", action_id="successor", activation_authority=authority, error="not draft")
        self.allocation("next-work", "repair", {"base": base}, sprint="PF81")
        self.action("first-successor-work", "next-work", finish=False)
        claim = self.call("claim", action_id="first-successor-work")
        self.assertEqual(base, claim["inputs"]["base"])
        state = self.c.snapshot()
        self.assertEqual("PF81", state["workstreams"]["delivery"]["sprint"])
        self.assertTrue(Coordinator._dependencies(state, "PF81"))
        self.assertEqual(3, sum(s["status"] in {"in_progress", "blocked"} for s in state["sprints"].values()))

    def test_prepared_completion_receiving_and_gate_failures_are_atomic(self):
        for index, mutate in enumerate([
            lambda r: r.update(status="verification_failed"), lambda r: r.update(receiving_commit="d" * 40),
            lambda r: r.update(tests=[]), lambda r: r["tests"][0].update(exit_code=1),
            lambda r: r["tests"][0].update(timed_out=True), lambda r: r["tests"][0].update(argv=["wrong"]),
            lambda r: r["assignment"].update(source="d" * 40),
        ]):
            with self.subTest(receiving=index):
                self.root = Path(self.tmp.name) / ("prepared-negative-" + str(index))
                self.c = Coordinator(self.root)
                self.c.initialize(*seed())
                self.c.set_enabled(True, {"fixture": True})
                gates = self.closure(mutate, prepared=True)
                self.owner("complete_sprint", action_id="completion", gates=gates, error="receiving")
        self.root = Path(self.tmp.name) / "private"
        self.c = Coordinator(self.root)
        gates = self.closure(prepared=True)
        for bad in ({}, {**gates, "extra": gates["review"]},
                    {**gates, "human": {"owner_verified": False, "status": "passed", "evidence": {"claim": True}}},
                    {**gates, "review": {"owner_verified": True, "status": "failed", "evidence": {"fixture": True}}},
                    {**gates, "human": {"owner_verified": True, "status": "passed", "evidence": {}}},
                    {**gates, "functional": {"owner_verified": True, "status": "not_applicable", "evidence": {"fixture": True}}}):
            self.owner("complete_sprint", action_id="completion", gates=bad, error="gate evidence")
        gates["functional"].update(status="not_applicable", reason="synthetic internal engineering only")
        self.owner("complete_sprint", action_id="completion", gates=gates)

    def test_prepared_lifecycle_rejects_wrong_kind_and_owned_or_terminal_actions(self):
        from coordinator import KINDS
        gates = self.closure(prepared=True)
        self.prepared_successor()
        for key, operation, payload in (("completion", "complete_sprint", {"gates": gates}),
                ("successor", "activate_successor", {"activation_authority": {"fixture": True}})):
            original = self.c.snapshot()["actions"][key]
            for kind in sorted(KINDS - {original["kind"]}):
                with self.subTest(action=key, wrong_kind=kind):
                    with self.c.mutation("fixture_wrong_kind", {}) as (_, state):
                        state["actions"][key]["kind"] = kind
                    self.owner(operation, action_id=key, **payload, error="wrong owner lifecycle kind")
            for status in ("dispatching", "dispatch_uncertain", "dispatched", "running", "returned", "failed", "cancelled"):
                with self.subTest(action=key, status=status):
                    with self.c.mutation("fixture_owned_status", {}) as (_, state):
                        state["actions"][key] = {**original, "status": status}
                    self.owner(operation, action_id=key, **payload, error="accepted owner-verified")
            with self.c.mutation("fixture_restore", {}) as (_, state):
                state["actions"][key] = original

    def test_prepared_lifecycle_pause_revision_allocation_and_manager_guards(self):
        gates = self.closure(prepared=True)
        self.prepared_successor()
        for key, operation, payload in (("completion", "complete_sprint", {"gates": gates}),
                ("successor", "activate_successor", {"activation_authority": {"fixture": True}})):
            for revision in (0, True):
                self.owner(operation, action_id=key, **payload, expected_revision=revision, error="revision")
            self.owner(operation, action_id=key, **payload, evidence={}, error="evidence")
            self.call("set_enabled", enabled=False, evidence={"fixture": "pause"})
            self.owner(operation, action_id=key, **payload, error="paused")
            self.call("set_enabled", enabled=True, evidence={"fixture": "resume"})
            self.owner("set_stream_mode", workstream="delivery", mode="paused")
            self.owner(operation, action_id=key, **payload, error="paused")
            self.owner("set_stream_mode", workstream="delivery", mode="enabled")
            packet = self.call("begin_manager")
            self.owner(operation, action_id=key, **payload, error="manager cycle already owned")
            self.call("fail_manager", run_id=packet["manager_run"], reason="fixture stopped")
            allocation = self.c.snapshot()["actions"][key]["inputs"]["allocation"]
            with self.c.mutation("fixture_stale_allocation", {}) as (_, state):
                state["allocations"][allocation]["scope"] = ["changed.py"]
            self.owner(operation, action_id=key, **payload, error="stale allocation proof")

    def test_prepared_lifecycle_respects_other_pending_and_cross_stream_resources(self):
        gates = self.closure(prepared=True)
        self.prepared_successor()
        # This active fixture belongs to another stream, so _no_pending alone
        # cannot protect the shared owner resource.
        with self.c.mutation("fixture_shared_resource", {}) as (_, state):
            for key in ("completion", "successor"):
                action = state["actions"][key]
                allocation = state["allocations"][action["inputs"]["allocation"]]
                allocation["resources"] = action["resources"] = ["integration-writer"]
                action["allocation_digest"] = digest(allocation)
            state["actions"]["other-stream"] = {**state["actions"]["completion"], "id": "other-stream",
                "sprint": "PF27", "workstream": "security", "status": "dispatch_uncertain"}
        self.owner("complete_sprint", action_id="completion", gates=gates, error="resource owned")
        self.owner("activate_successor", action_id="successor", activation_authority={"fixture": True}, error="resource owned")
        with self.c.mutation("fixture_release", {}) as (_, state):
            state["actions"]["other-stream"]["status"] = "failed"
        self.action("pending", "bootstrap", finish=False)
        self.owner("complete_sprint", action_id="completion", gates=gates, error="pending")

    def test_prepared_successor_wrong_base_pending_and_authority_denials(self):
        gates = self.closure(prepared=True)
        self.owner("complete_sprint", action_id="completion", gates=gates)
        self.owner("archive_sprint", sprint="PF80")
        self.prepared_successor(base="d" * 40)
        for authority in (None, {}, True, "approval"):
            self.owner("activate_successor", action_id="successor", activation_authority=authority,
                       error="explicit owner activation authority")
        self.owner("activate_successor", action_id="successor", activation_authority={"fixture": True},
                   error="verified receiving commit")
        allocation = self.c.snapshot()["allocations"]["next"]
        self.owner("put_allocation", allocation_id="next", replace=True, allocation={**allocation, "inputs": {"base": "c" * 40}})
        self.owner("activate_successor", action_id="successor", activation_authority={"fixture": True}, error="accepted owner-verified")
        self.action("correct-successor", "next", finish=False)
        self.allocation("pending-next", "wait", {}, sprint="PF81")
        self.action("pending-next", "pending-next", finish=False)
        self.owner("activate_successor", action_id="correct-successor", activation_authority={"fixture": True}, error="pending")
        self.owner("record_wait", action_id="pending-next")
        self.owner("activate_successor", action_id="correct-successor", activation_authority={"fixture": True})

    def test_owner_lifecycle_process_crash_rolls_back_or_preserves_whole_effect(self):
        script = '''
import json, os, sys
from coordinator import Coordinator
core = Coordinator(sys.argv[1])
if sys.argv[4] == 'before_commit':
    original = core._event
    def interrupt(*args, **kwargs):
        original(*args, **kwargs)
        os._exit(73)
    core._event = interrupt
getattr(core, sys.argv[2])(**json.loads(sys.argv[3]))
os._exit(74)
'''
        gates = self.closure(prepared=True)
        for operation, key, extra in (("complete_sprint", "completion", {"gates": gates}),
                ("activate_successor", "successor", {"activation_authority": {"fixture": True}})):
            if operation == "activate_successor":
                self.owner("archive_sprint", sprint="PF80")
                self.prepared_successor()
            revision = self.c.snapshot()["revision"]
            payload = {"action_id": key, "expected_revision": revision, "evidence": {"fixture": "crash test"}, **extra}
            before = self.database_rows()
            for boundary, code in (("before_commit", 73), ("after_commit", 74)):
                result = subprocess.run([sys.executable, "-B", "-c", script, str(self.root), operation,
                                         json.dumps(payload), boundary], capture_output=True, timeout=10)
                self.assertEqual(code, result.returncode, result.stderr)
                self.c = Coordinator(self.root)
                if boundary == "before_commit":
                    self.assertEqual(before, self.database_rows())
                else:
                    self.assertEqual(revision + 1, self.c.snapshot()["revision"])
                    self.assert_owner_effect(key, "owner_complete" if key == "completion" else "owner_successor")
                    self.call(operation, **payload, error="stale owner revision")


class SprintRegistrationTests(unittest.TestCase):
    """Document-backed owner registration in disposable, synthetic state."""

    setUp = CoordinatorTests.setUp
    tearDown = CoordinatorTests.tearDown
    database_rows = OwnerControlsTests.database_rows

    RELATIVE = "docs/sprints/current/initiative-delivery-control/pf82.md"

    def repo(self):
        root = Path(self.tmp.name) / "repo"
        for plan in ("p0-security-levels", "portfolio-agent-cost-accounting",
                     "initiative-delivery-control"):
            path = root / "docs/plans/active" / (plan + ".md")
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("# " + plan + "\n")
        return root

    def document(self, relative=None, **changes):
        front = {"sprint_id": "PF82", "status": "draft",
                 "plan_file": "docs/plans/active/initiative-delivery-control.md",
                 "depends_on": "PF80"}
        front.update(changes)
        path = self.repo() / (relative or self.RELATIVE)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("---\n" + "".join(f"{k}: {json.dumps(v)}\n" for k, v in front.items())
                        + "---\n# Synthetic sprint\n")
        return path

    def payload(self, **changes):
        self.document()
        payload = dict(sprint_id="PF82", workstream="delivery", dependencies=["PF80"],
                       status="draft", source_path=self.RELATIVE, repo=str(self.repo()),
                       expected_revision=self.c.snapshot()["revision"],
                       evidence={"inspection": "synthetic owner evidence"})
        payload.update(changes)
        return payload

    def refused(self, reason, payload):
        before = self.database_rows()
        with self.assertRaisesRegex(Rejected, reason):
            self.c.register_sprint(**payload)
        self.assertEqual(before, self.database_rows(), "refusal must roll back every table")

    def test_register_draft_is_durable_audited_and_does_not_activate(self):
        payload = self.payload()
        before = self.c.snapshot()
        receipt = self.c.register_sprint(**payload)
        state = Coordinator(self.root).snapshot()
        self.assertEqual(before["revision"] + 1, state["revision"])
        self.assertEqual(before["workstreams"], state["workstreams"])
        for key in ("enabled", "actions", "allocations", "manager"):
            self.assertEqual(before[key], state[key])
        self.assertEqual(before["sprints"], {k: v for k, v in state["sprints"].items() if k != "PF82"})
        self.assertEqual("draft", state["sprints"]["PF82"]["status"])
        self.assertFalse(state["sprints"]["PF82"]["archived"])
        proof = self.c.read_evidence(receipt["evidence_digest"])
        self.assertEqual("PF82", proof["sprint_id"])
        # The row stores the repository-relative path, never one worktree.
        self.assertEqual(self.RELATIVE, proof["record"]["source_path"])
        self.assertEqual(self.RELATIVE, state["sprints"]["PF82"]["source_path"])
        import hashlib
        self.assertEqual(hashlib.sha256(self.document().read_bytes()).hexdigest(),
                         proof["document_sha256"])
        self.assertEqual(payload["evidence"], proof["evidence"])
        with self.c.connection() as db:
            audit = json.loads(db.execute("SELECT body FROM audit WHERE operation='owner_register_sprint'").fetchone()[0])
            event = json.loads(db.execute("SELECT body FROM events WHERE id=?",
                                         (f'owner_register_sprint:{before["revision"]}',)).fetchone()[0])
        self.assertEqual(payload["expected_revision"], audit["expected_revision"])
        self.assertEqual("PF82", event["sprint"])
        self.assertEqual(audit, self.c.read_evidence(event["evidence"]["evidence_digest"]))

    def test_register_existing_id_refused(self):
        for lifecycle, archived in (("draft", False), ("in_progress", False), ("completed", True)):
            with self.subTest(lifecycle=lifecycle):
                with self.c.mutation("synthetic_fixture", {}) as (_, state):
                    state["sprints"]["PF81"].update(status=lifecycle, archived=archived)
                    if lifecycle == "in_progress":
                        state["sprints"]["PF80"]["status"] = "completed"
                        state["workstreams"]["delivery"]["sprint"] = "PF81"
                    else:
                        state["sprints"]["PF80"]["status"] = "in_progress"
                        state["workstreams"]["delivery"]["sprint"] = "PF80"
                payload = self.payload(sprint_id="PF81")
                self.document(sprint_id="PF81")
                self.refused("sprint already registered", payload)

    def test_register_document_mismatch_refused(self):
        for fields, reason in (
            ({"sprint_id": "OTHER"}, "id mismatch"),
            ({"plan_file": "docs/plans/active/portfolio-agent-cost-accounting.md"}, "workstream mismatch"),
            ({"workstream": "accounting"}, "workstream mismatch"),
            ({"status": "in_progress"}, "status mismatch"),
            ({"depends_on": "PF81"}, "dependencies mismatch"),
        ):
            with self.subTest(fields=fields):
                payload = self.payload()
                self.document(**fields)
                self.refused("sprint document " + reason, payload)

    def test_register_missing_file_refused(self):
        payload = self.payload()
        self.document().unlink()
        self.refused("sprint source_path unreadable", payload)

    def test_register_unknown_dependency_refused(self):
        payload = self.payload(dependencies=["UNKNOWN"])
        self.document(depends_on="UNKNOWN")
        self.refused("unknown dependency", payload)

    def test_register_cycle_refused(self):
        payload = self.payload(dependencies=["PF82"])
        self.document(depends_on="PF82")
        self.refused("dependency cycle", payload)
        # Valid state cannot grow a multi-node cycle by adding only backward
        # references. A legacy corrupt component must nevertheless fail closed.
        with self.c.mutation("synthetic_corrupt_graph", {}) as (_, state):
            state["sprints"]["PF80"]["dependencies"] = ["PF81"]
        self.refused("dependency cycle", self.payload())

    def test_register_stale_revision_refused(self):
        payload = self.payload()
        self.c.event({"id": "new-owner-input"})
        self.refused("stale owner revision", payload)

    def test_register_evidence_required(self):
        self.refused("owner revision and evidence required", self.payload(evidence={}))

    def test_register_paused_refused(self):
        self.c.set_enabled(False, {"fixture": "pause"})
        self.refused("dispatch/workstream paused", self.payload())
        self.c.set_enabled(True, {"fixture": "resume"})
        self.c.set_stream_mode("delivery", "paused", self.c.snapshot()["revision"], {"fixture": "pause"})
        self.refused("dispatch/workstream paused", self.payload())

    def test_register_reserved_status_refused(self):
        payload = self.payload(status="in_progress")
        self.document(status="in_progress")
        self.refused("registered sprint must start as draft", payload)

    def test_register_reservation_limit_refused(self):
        with self.c.mutation("synthetic_corrupt_reservations", {}) as (_, state):
            state["sprints"]["PF81"]["status"] = "blocked"
        self.refused("three-reservation limit", self.payload())

    def test_register_malformed_front_matter_refused(self):
        for document, reason in (
            ("# No front matter\n", "sprint front matter required"),
            ('---\nsprint_id: "PF82"\nsprint_id: "OTHER"\n---\n', "duplicate sprint front matter key"),
        ):
            with self.subTest(reason=reason):
                payload = self.payload()
                self.document().write_text(document)
                self.refused(reason, payload)

    def test_registered_sprint_accepts_an_allocation(self):
        """The point of registering: work can then be allocated against it."""
        self.c.register_sprint(**self.payload())
        allocation = {"sprint": "PF82", "kinds": ["implement"], "resources": ["worktree"],
                      "scope": ["codex-rs/core/src/x.rs"], "timeout_seconds": 1800,
                      "inputs": {"task": "synthetic"}}
        self.c.put_allocation("pf82-impl-01", allocation, False,
                              self.c.snapshot()["revision"], {"fixture": "allocate"})
        self.assertEqual(allocation, self.c.snapshot()["allocations"]["pf82-impl-01"])

    def test_source_path_is_confined_to_repository_sprint_documents(self):
        outside = Path(self.tmp.name) / "elsewhere.md"
        outside.write_text(self.document().read_text())
        link = self.repo() / "docs/sprints/current/initiative-delivery-control/escape.md"
        link.symlink_to(outside)
        confined = "must be repository-relative under docs/sprints"
        for source_path, reason in (
            (str(self.document()), confined),
            ("/etc/passwd", confined),
            ("docs/sprints/../../elsewhere.md", confined),
            ("docs/plans/active/initiative-delivery-control.md", confined),
            ("docs/sprints/current/initiative-delivery-control/pf82.txt", confined),
            ("docs/sprints/current/initiative-delivery-control/escape.md",
             "escapes the repository"),
        ):
            with self.subTest(source_path=source_path):
                self.refused("sprint source_path " + reason, self.payload(source_path=source_path))

    def test_repo_without_the_named_plan_is_not_a_checkout(self):
        bare = Path(self.tmp.name) / "bare"
        (bare / Path(self.RELATIVE).parent).mkdir(parents=True)
        (bare / self.RELATIVE).write_text(self.document().read_text())
        self.refused("sprint plan file missing from repository", self.payload(repo=str(bare)))

    def test_re_registration_corrects_the_document_reference_of_an_unstarted_draft(self):
        self.c.register_sprint(**self.payload())
        first = self.c.snapshot()["sprints"]["PF82"]["registration"]
        moved = "docs/sprints/current/initiative-delivery-control/pf82-renamed.md"
        self.document(relative=moved)
        receipt = self.c.register_sprint(**self.payload(source_path=moved, replace=True))
        record = Coordinator(self.root).snapshot()["sprints"]["PF82"]
        self.assertEqual(moved, record["source_path"])
        self.assertNotEqual(first, record["registration"])
        self.assertEqual(receipt, record["registration"])
        self.assertEqual(("draft", False, ["PF80"], "delivery"),
                         (record["status"], record["archived"],
                          record["dependencies"], record["workstream"]))
        proof = self.c.read_evidence(receipt["evidence_digest"])
        self.assertEqual(moved, proof["record"]["source_path"])

    def test_re_registration_refused_once_the_sprint_is_real(self):
        self.refused("unknown sprint", self.payload(replace=True))
        self.c.register_sprint(**self.payload())
        self.refused("sprint already registered", self.payload())
        self.refused("may only correct the document reference",
                     self.payload(replace=True, workstream="accounting"))
        self.refused("may only correct the document reference",
                     self.payload(replace=True, dependencies=[]))
        self.c.put_allocation("pf82-impl-01", {"sprint": "PF82", "kinds": ["implement"],
                              "resources": ["worktree"], "scope": ["codex-rs/core/src/x.rs"],
                              "timeout_seconds": 1800, "inputs": {"task": "synthetic"}},
                              False, self.c.snapshot()["revision"], {"fixture": "allocate"})
        self.refused("already has allocations or actions", self.payload(replace=True))
        with self.c.mutation("synthetic_fixture", {}) as (_, state):
            del state["allocations"]["pf82-impl-01"]
            state["sprints"]["PF82"]["status"] = "in_progress"
        self.refused("only an unstarted registered draft", self.payload(replace=True))

    def test_replace_flag_must_be_explicit_boolean(self):
        self.refused("explicit add/replace required", self.payload(replace=1))


if __name__ == "__main__":
    unittest.main()

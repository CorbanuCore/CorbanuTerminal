import copy
import io
import json
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

    def call(self, operation, error=None, **payload):
        before = self.c.snapshot()
        with self.c.connection() as db:
            counts = [db.execute("SELECT COUNT(*) FROM " + t).fetchone()[0]
                      for t in ("events", "audit", "action_history", "evidence")]
        output = io.StringIO()
        code = coordinator_cli.main([operation, "--state", str(self.root)],
                                    io.StringIO(json.dumps(payload)), output)
        result = json.loads(output.getvalue())
        self.assertEqual(2 if error else 0, code, result)
        if error:
            self.assertIn(error, result["error"])
            self.assertEqual(before, self.c.snapshot())
            with self.c.connection() as db:
                self.assertEqual(counts, [db.execute("SELECT COUNT(*) FROM " + t).fetchone()[0]
                                         for t in ("events", "audit", "action_history", "evidence")])
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

    def closure(self, receipt_change=None, completion_verified=True):
        assignment = {"id": "fixture-merge", "base": "a" * 40, "source": "b" * 40,
                      "branch": "fixture-receiving", "scope": ["scripts/initiative_control/"],
                      "approval": {"fixture": True}, "tests": [{"argv": ["fixture-test"], "timeout_seconds": 60}]}
        receipt = {"status": "verified", "receiving_commit": "c" * 40, "assignment": assignment,
                   "tests": [{"argv": ["fixture-test"], "exit_code": 0, "timed_out": False}]}
        self.allocation("receive", "integrate", {"assignment": assignment})
        if receipt_change:
            receipt_change(receipt)
        self.action("receiving", "receive", verification={"receiving_receipt": receipt})
        self.allocation("close", "complete_sprint", {"receiving_action": "receiving",
                        "receiving_commit": "c" * 40, "mandatory_gates": ["review", "functional", "human"]})
        self.action("completion", "close", verification=None if completion_verified else False)
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


if __name__ == "__main__":
    unittest.main()

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


if __name__ == "__main__":
    unittest.main()

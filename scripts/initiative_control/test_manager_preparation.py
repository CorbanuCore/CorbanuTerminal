"""Synthetic tests for the manager's worker-allocation preparation callsite."""
import io
import json
from pathlib import Path
import tempfile
import unittest

from coordinator import Coordinator, digest
import coordinator_cli
from owner_tmux import freeze_worker_inputs
from test_coordinator import seed

AUTHORITY = {"owner": "synthetic manager", "authority": "fixture provider/policy record"}


class ManagerPreparationTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name) / "private"
        self.c = Coordinator(self.root, lambda: 1000.0)
        self.c.initialize(*seed())
        self.c.set_enabled(True, {"authority": "bounded rehearsal"})
        self.worktree = str(Path(self.tmp.name) / "worktree")

    def envelope(self, task="read-only check"):
        return {"sprint": "PF80", "kinds": ["review"], "resources": ["prep-worktree"],
                "scope": ["read-only"], "timeout_seconds": 600,
                "inputs": {"base_commit": "b" * 40, "model": "gpt-fixture", "reasoning_effort": "high",
                           "task": task, "worktree": self.worktree}}

    def call(self, error=None, **payload):
        body = {"allocation_id": "prep", "allocation": self.envelope(), "provider": "fixture",
                "policy": "--yolo", "replace": False, "evidence": dict(AUTHORITY),
                "expected_revision": self.c.snapshot()["revision"], **payload}
        before, output = self.c.snapshot(), io.StringIO()
        code = coordinator_cli.main(["prepare_worker", "--state", str(self.root)],
                                    io.StringIO(json.dumps(body)), output)
        result = json.loads(output.getvalue())
        self.assertEqual(2 if error else 0, code, result)
        if error:
            self.assertIn(error, result["error"])
            self.assertEqual(before, self.c.snapshot())
        return result.get("result")

    def accept(self, action_id, inputs):
        self.c.event({"id": "trigger-" + action_id, "reason": "prepare"})
        packet = self.c.begin_manager()
        action = {"id": action_id, "kind": "review", "workstream": "delivery", "sprint": "PF80",
                  "rationale": "read-only check", "expected_revision": packet["state_revision"],
                  "inputs": inputs, "timeout_seconds": 600}
        self.c.accept_decision(packet["manager_run"],
                               {"state_revision": packet["state_revision"], "actions": [action]},
                               {"artifact": "verified-test-manager.json"})
        return self.c.snapshot()["actions"][action_id]

    def test_registers_exact_bridge_output_with_preparation_evidence(self):
        bound = self.call()
        expected = freeze_worker_inputs(self.envelope()["inputs"], provider="fixture", policy="--yolo")
        self.assertEqual({"allocation": "prep", **expected}, bound)
        allocation = self.c.snapshot()["allocations"]["prep"]
        self.assertEqual(expected, allocation["inputs"])
        self.assertEqual(dict(model="gpt-fixture", provider="fixture", effort="high",
                              worktree=self.worktree, policy="--yolo"), allocation["inputs"]["worker"])
        with self.c.connection() as db:
            row = db.execute("SELECT body FROM audit WHERE operation='owner_allocation' "
                             "ORDER BY seq DESC LIMIT 1").fetchone()
        self.assertEqual({"bridge": "owner_tmux.freeze_worker_inputs", "provider": "fixture",
                          "policy": "--yolo"}, json.loads(row[0])["evidence"]["preparation"])
        # The gate's rebinding check holds for a decision copying registered inputs.
        action = self.accept("first", bound)
        rebound = freeze_worker_inputs({k: v for k, v in action["inputs"].items() if k != "worker"},
                                       provider="fixture", policy="--yolo")
        self.assertEqual(action["inputs"], rebound)
        self.assertEqual(digest(allocation), action["allocation_digest"])

    def test_refuses_without_explicit_authority(self):
        self.call(provider="", error="explicit provider")
        self.call(policy="--full-auto", error="--yolo")
        self.call(evidence={"owner": "no authority"}, error="authority reference")
        self.assertNotIn("prep", self.c.snapshot()["allocations"])

    def test_refuses_conflicting_or_incomplete_runtime(self):
        conflicting = self.envelope()
        conflicting["inputs"]["worker"] = dict(model="other", provider="fixture", effort="high",
                                               worktree=self.worktree, policy="--yolo")
        self.call(allocation=conflicting, error="conflicting_worker_runtime")
        incomplete = self.envelope()
        del incomplete["inputs"]["model"]
        self.call(allocation=incomplete, error="missing_cycle_runtime")
        relative = self.envelope()
        relative["inputs"]["worktree"] = "relative"
        self.call(allocation=relative, error="absolute_worktree_required")

    def test_replacement_cancels_prepared_action_for_fresh_decision(self):
        old = self.accept("old", self.call())
        bound = self.call(allocation=self.envelope("refreshed read-only check"), replace=True)
        state = self.c.snapshot()
        cancelled = state["actions"]["old"]
        self.assertEqual("cancelled", cancelled["status"])
        self.assertTrue(cancelled["owner_cancellation"])
        fresh = self.accept("new", bound)
        self.assertEqual("prepared", fresh["status"])
        self.assertNotEqual(old["allocation_digest"], fresh["allocation_digest"])
        self.assertNotEqual(old["manager_run"], fresh["manager_run"])


if __name__ == "__main__":
    unittest.main()

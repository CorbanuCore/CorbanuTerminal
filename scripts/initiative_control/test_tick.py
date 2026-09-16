"""Tick status and process-edge lock regressions; synthetic state only."""
from contextlib import redirect_stdout
import io
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

import control
import tasknode
import tick
from test_control import run


class TickTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.state = self.root / "state"
        control.atomic_json(self.state / "control.json", {
            "tasknode": {"enabled": False, "workspace_id": "fixture-workspace",
                         "task_mappings": {"PF-80-S01": ["fixture-task"]}}})

    def test_uncertain_outbox_is_published_distinct_from_invalid(self):
        event_id = tasknode.enqueue(self.state, run())
        path = self.state / "outbox" / (event_id + ".json")
        record = control.read_json(path, self.state)
        record.update(status="uncertain", attempts=1)
        control.atomic_json(path, record)
        before = path.read_bytes()
        for corrupt in (False, True):
            with self.subTest(corrupt=corrupt):
                if corrupt:
                    control.atomic_json(self.state / "outbox/corrupt.json", {"status": "unknown"})
                def published(*_):
                    status = control.read_json(self.state / "writeback-status.json", self.state)
                    expected = {"uncertain": 1, "invalid": 1} if corrupt else {"uncertain": 1}
                    self.assertEqual(status["outbox"], expected)
                    self.assertIn("Delivery is uncertain", status["error"])
                    self.assertIn("no automatic retry", status["error"])
                    if corrupt:
                        self.assertIn("manager inspection", status["error"])
                    else:
                        self.assertNotIn("manager inspection", status["error"])
                with patch.object(tick, "collect", return_value={"runs": []}), \
                     patch.object(tick, "publish", side_effect=published) as publisher, \
                     patch.object(tick, "credentials", side_effect=AssertionError("no credentials")), \
                     patch.object(tick, "flush", side_effect=AssertionError("posting disabled")), \
                     redirect_stdout(io.StringIO()):
                    tick.tick(self.root)
                publisher.assert_called_once()
                self.assertEqual(path.read_bytes(), before)

    def child(self, mode):
        script = """
import control, runpy, sys, tasknode, tick
from pathlib import Path
from unittest.mock import patch
from test_control import run
root, mode = Path(sys.argv[1]), sys.argv[2]
print("starting", flush=True)
if mode in {"control", "tasknode"}:
    module = control if mode == "control" else tasknode
    with module.locked(root / "state/.outbox.lock"):
        print("acquired", flush=True)
else:
    runs = [] if mode == "cli_flush" else [run()]
    with patch.object(tasknode, "credentials", return_value={}), \
         patch.object(tasknode, "post", side_effect=AssertionError("no transport")), \
         patch.object(control, "collect", return_value={"runs": runs}), \
         patch.object(control, "publish"), \
         patch.object(tick, "collect", return_value={"runs": [run()]}), \
         patch.object(tick, "publish"):
        if mode in {"cli", "cli_flush"}:
            sys.argv = ["tick.py", "--root", str(root)]
            try:
                runpy.run_path(str(Path(tick.__file__)), run_name="__main__")
            finally:
                assert not tasknode.CLI_CONTEXT.get()
        else:
            tick.tick(root)
    assert not tasknode.CLI_CONTEXT.get()
"""
        env = {"PATH": os.defpath, "HOME": str(self.root),
               "CODEX_HOME": str(self.root), "CORBANU_HOME": str(self.root),
               "PFTERMINAL_HOME": str(self.root), "PYTHONPATH": str(control.HERE),
               "PYTHONDONTWRITEBYTECODE": "1", "CORBANU_TEST_DISABLE_NATIVE_KEYRING": "1"}
        child = subprocess.Popen([sys.executable, "-c", script, str(self.root), mode],
                                 env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        def cleanup():
            if child.poll() is None:
                child.kill()
            child.communicate()
        self.addCleanup(cleanup)
        self.assertEqual(child.stdout.readline().strip(), "starting")
        return child

    def test_tick_cli_lock_wait_is_bounded_and_recovers(self):
        with control.locked(self.state / ".outbox.lock"):
            before = {p: p.read_bytes() for p in self.state.rglob("*") if p.is_file()}
            started = time.monotonic()
            child = self.child("cli")
            output, errors = child.communicate(timeout=tasknode.CLI_LOCK_SECONDS + 3)
            self.assertEqual(child.returncode, 2, errors)
            refusal = f"another Task Node operation is holding the queue; lock path: {self.state / '.outbox.lock'}"
            self.assertEqual(errors, f"Task Node refresh refused: {refusal}\n")
            self.assertNotIn("Refresh complete", output)
            self.assertGreaterEqual(time.monotonic() - started, tasknode.CLI_LOCK_SECONDS)
            status = control.read_json(self.state / "writeback-status.json", self.state)
            self.assertEqual(status["rejected_runs"], 0)
            self.assertEqual(status["error"], refusal)
            self.assertEqual(control.read_json(self.root / "site/health.json", self.root)["error"], refusal)
            self.assertEqual(status["outbox"], {})
            self.assertFalse((self.state / "outbox-index.json").exists())
            for path, content in before.items():
                self.assertEqual(path.read_bytes(), content)
        child = self.child("cli")
        _, errors = child.communicate(timeout=5)
        self.assertEqual(child.returncode, 0, errors)
        status = control.read_json(self.state / "writeback-status.json", self.state)
        self.assertEqual(status["rejected_runs"], 0)
        self.assertEqual(status["outbox"], {"pending": 1})

    def test_tick_cli_flush_lock_expiry_is_not_credential_recovery(self):
        config = control.read_json(self.state / "control.json", self.state)
        config["tasknode"]["enabled"] = True
        control.atomic_json(self.state / "control.json", config)
        control.atomic_json(self.state / "enrollment.json",
                            {"workspace_id": "fixture-workspace", "verified": True})
        with control.locked(self.state / ".outbox.lock"):
            child = self.child("cli_flush")
            output, errors = child.communicate(timeout=tasknode.CLI_LOCK_SECONDS + 3)
            self.assertEqual(child.returncode, 2, errors)
            refusal = f"another Task Node operation is holding the queue; lock path: {self.state / '.outbox.lock'}"
            self.assertEqual(errors, f"Task Node refresh refused: {refusal}\n")
            self.assertNotIn("Refresh complete", output)
            status = control.read_json(self.state / "writeback-status.json", self.state)
            self.assertEqual(status["error"], refusal)
            self.assertEqual(status["rejected_runs"], 0)
            self.assertEqual(status["delivered_this_run"], 0)
            self.assertEqual(control.read_json(self.root / "site/health.json", self.root)["error"], refusal)
        child = self.child("cli_flush")
        output, errors = child.communicate(timeout=5)
        self.assertEqual(child.returncode, 0, errors)
        self.assertIn("Refresh complete", output)
        self.assertNotIn("error", control.read_json(self.state / "writeback-status.json", self.state))

    def test_pending_record_with_crash_intent_publishes_uncertain(self):
        event_id = tasknode.enqueue(self.state, run())
        path = self.state / "outbox" / (event_id + ".json")
        self.assertEqual(control.read_json(path, self.state)["status"], "pending")
        control.atomic_json(self.state / "send-receipts" / (event_id + ".intent.json"), {})
        before = {p: p.read_bytes() for p in self.state.rglob("*") if p.is_file()}
        def published(*_):
            status = control.read_json(self.state / "writeback-status.json", self.state)
            self.assertEqual(status["outbox"], {"uncertain": 1})
            self.assertIn("Delivery is uncertain; no automatic retry", status["error"])
            self.assertIn("external reconciliation", status["error"])
        with patch.object(tick, "collect", return_value={"runs": []}), \
             patch.object(tick, "publish", side_effect=published) as publisher, \
             patch.object(tick, "credentials", side_effect=AssertionError("no credentials")), \
             redirect_stdout(io.StringIO()):
            tick.tick(self.root)
        publisher.assert_called_once()
        for path, content in before.items():
            self.assertEqual(path.read_bytes(), content)

    def test_library_tick_tasknode_and_control_still_block_until_release(self):
        for mode in ("tick", "tasknode", "control"):
            with self.subTest(mode=mode):
                with control.locked(self.state / ".outbox.lock"):
                    child = self.child(mode)
                    with self.assertRaises(subprocess.TimeoutExpired):
                        child.communicate(timeout=tasknode.CLI_LOCK_SECONDS + 0.3)
                    self.assertIsNone(child.poll())
                output, errors = child.communicate(timeout=5)
                self.assertEqual(child.returncode, 0, errors)
                self.assertIn("Refresh complete" if mode == "tick" else "acquired", output)
        status = control.read_json(self.state / "writeback-status.json", self.state)
        self.assertEqual(status["rejected_runs"], 0)
        self.assertEqual(status["outbox"], {"pending": 1})


if __name__ == "__main__":
    unittest.main()

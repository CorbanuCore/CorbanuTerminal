"""Readiness tests use synthetic state and mocked helpers/HTTP only."""
from contextlib import redirect_stderr, redirect_stdout
import datetime as dt
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

import control
import tasknode
from test_control import run


class SendTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.state = Path(self.tmp.name)
        self.config = {"tasknode": {"enabled": False, "workspace_id": "fixture-workspace",
                       "task_mappings": {"PF-80-S01": ["fixture-task"]}}}
        control.atomic_json(self.state / "control.json", self.config)
        self.event_id = tasknode.enqueue(self.state, run())
        self.record_path = self.state / "outbox" / (self.event_id + ".json")
        self.event = control.read_json(self.record_path, self.state)["event"]
        control.atomic_json(self.state / "enrollment.json",
                            {"workspace_id": "fixture-workspace", "verified": True})
        self.activation_file = self.state / "activation.json"
        self.activation = {"schema": 1, "owner": "fixture-owner", "enabled": True,
                           "event_id": self.event_id,
                           "request_digest": tasknode.request_digest({"event": self.event}),
                           "workspace_id": "fixture-workspace", "task_ids": ["fixture-task"],
                           "origin": tasknode.ORIGIN,
                           "expires_at": (control.timestamp(control.now()) + dt.timedelta(hours=1)).isoformat(),
                           "gates": dict.fromkeys(("identity", "entitlement", "enrollment",
                                                  "target_lifecycle", "payload_review"), True)}
        control.atomic_json(self.activation_file, self.activation)
        self.auth_file = self.state / "synthetic-auth.json"
        control.atomic_json(self.auth_file, {"terminal_session": "fixture-session", "api_key": "fixture-key"})
        self.transport = Mock(return_value=(200, {"ok": True, "id": self.event_id,
                                                 "untrusted": "private response omitted"}))

    def live(self):
        return tasknode.send(self.state, self.event_id, live=True,
                             activation_file=self.activation_file, credentials_file=self.auth_file,
                             transport=self.transport)

    def snapshot(self):
        return {str(p.relative_to(self.state)): p.read_bytes()
                for p in self.state.rglob("*") if p.is_file()}

    def test_dry_run_is_read_only_and_selects_one_from_batch(self):
        for i in range(25):
            tasknode.enqueue(self.state, {**run(), "run_id": f"fixture-{i}"})
        before = self.snapshot()
        with patch.object(tasknode, "credentials", side_effect=AssertionError("no auth")), \
             patch.object(tasknode, "post", side_effect=AssertionError("no HTTP")), \
             patch.object(tasknode, "flush", side_effect=AssertionError("no flush")):
            result = tasknode.send(self.state, self.event_id, dry_run=True)
        self.assertFalse(result["network_writes"])
        self.assertFalse(result["send_authorized"])
        self.assertEqual(result["selected_count"], 1)
        self.assertEqual(self.snapshot(), before)

    def test_live_success_is_immutable_and_retry_does_not_post_or_load_auth(self):
        before_queue = self.record_path.read_bytes()
        with patch.object(tasknode, "flush", side_effect=AssertionError("no flush")):
            result = self.live()
        self.assertEqual(result["outcome"], "delivered")
        self.assertEqual(result["idempotency_key"], self.event_id)
        self.assertNotIn("private response", json.dumps(result))
        before = self.snapshot()
        with patch.object(tasknode, "credentials", side_effect=AssertionError("no auth")):
            replay = self.live()
        self.assertTrue(replay["replayed"])
        self.assertFalse(replay["network_writes"])
        self.assertEqual(self.transport.call_count, 1)
        self.assertEqual(before, self.snapshot())
        self.assertEqual(before_queue, self.record_path.read_bytes())
        for path in (self.state / "send-receipts").iterdir():
            self.assertEqual(path.stat().st_mode & 0o077, 0)
            with self.assertRaises(FileExistsError):
                tasknode.immutable_json(path, {})

    def test_each_activation_gate_and_binding_fails_closed(self):
        changes = {"schema": True, "owner": "", "enabled": False, "event_id": "cc-" + "0" * 64,
                   "request_digest": "0" * 64, "workspace_id": "wrong", "task_ids": ["wrong"],
                   "origin": "https://invalid.example", "expires_at": "2020-01-01T00:00:00Z",
                   "gates": {}}
        for field, value in changes.items():
            with self.subTest(field=field):
                control.atomic_json(self.activation_file, {**self.activation, field: value})
                with self.assertRaises((ValueError, TypeError)):
                    self.live()
        self.transport.assert_not_called()

    def test_private_regular_activation_required(self):
        self.activation_file.chmod(0o644)
        with self.assertRaises(ValueError):
            self.live()
        self.activation_file.unlink()
        self.activation_file.symlink_to(self.auth_file)
        with self.assertRaises(ValueError):
            self.live()
        self.transport.assert_not_called()

    def test_mapping_enabled_stale_backoff_and_enrollment_gates(self):
        for change in ("mapping", "enabled", "stale", "backoff", "enrollment"):
            with self.subTest(change=change):
                before = self.snapshot()
                if change == "mapping":
                    config = json.loads(json.dumps(self.config))
                    config["tasknode"]["task_mappings"] = {"PF-76-S01": ["fixture-task"]}
                    control.atomic_json(self.state / "control.json", config)
                elif change == "enabled":
                    control.atomic_json(self.state / "control.json",
                                        {"tasknode": {**self.config["tasknode"], "enabled": True}})
                elif change == "enrollment":
                    control.atomic_json(self.state / "enrollment.json", {})
                elif change == "stale":
                    future = (control.timestamp(control.now()) + dt.timedelta(hours=2)).isoformat()
                    with patch.object(tasknode, "now", return_value=future):
                        with self.assertRaises(ValueError):
                            self.live()
                    continue
                else:
                    record = control.read_json(self.record_path, self.state)
                    record["next_attempt_at"] = (control.timestamp(control.now()) + dt.timedelta(hours=1)).isoformat()
                    control.atomic_json(self.record_path, record)
                with self.assertRaises(ValueError):
                    self.live()
                for path, contents in before.items():
                    (self.state / path).write_bytes(contents)
        self.transport.assert_not_called()

    def test_hash_tamper_and_batch_selectors_rejected(self):
        for selected in ("*", self.event_id + "," + self.event_id, "../event", [self.event_id]):
            with self.assertRaises(ValueError):
                tasknode.send(self.state, selected, dry_run=True)
        record = control.read_json(self.record_path, self.state)
        record["event"]["content"] = "changed"
        control.atomic_json(self.record_path, record)
        with self.assertRaises(ValueError):
            self.live()
        self.transport.assert_not_called()

    def test_uncertain_result_and_crash_intent_never_repost(self):
        self.transport.side_effect = TimeoutError("private error must not escape")
        result = self.live()
        self.assertEqual(result["outcome"], "uncertain")
        self.assertNotIn("private error", json.dumps(result))
        self.live()
        self.assertEqual(self.transport.call_count, 1)
        (self.state / "send-receipts" / (self.event_id + ".result.json")).unlink()
        result = self.live()
        self.assertTrue(result["reconciliation_required"])
        self.assertEqual(self.transport.call_count, 1)

    def test_receipt_failure_preserves_durable_intent_and_prevents_repost(self):
        write = tasknode.immutable_json
        def fail_result(path, value):
            if path.name.endswith(".result.json"):
                raise OSError("fixture disk failure")
            return write(path, value)
        with patch.object(tasknode, "immutable_json", side_effect=fail_result):
            with self.assertRaises(OSError):
                self.live()
        self.assertEqual(self.live()["outcome"], "uncertain")
        self.assertEqual(self.transport.call_count, 1)

    def test_wrong_id_deleted_and_rejected_responses_are_not_success(self):
        for status, response in ((200, {"ok": True, "id": "other"}),
                                 (200, {"ok": True, "id": self.event_id, "summaryState": "deleted"}),
                                 (403, {"ok": True, "id": self.event_id})):
            with self.subTest(status=status, response=response):
                self.transport.return_value = (status, response)
                self.assertEqual(self.live()["outcome"], "uncertain")
                for path in (self.state / "send-receipts").iterdir():
                    path.unlink()

    def test_cli_rejects_ambiguous_or_incomplete_modes_before_dispatch(self):
        cases = [[], ["--live"], ["--dry-run", "--live"], ["--confirm-live"],
                 ["--dry-run", "--credentials-file", str(self.auth_file)],
                 ["--dry-run", "--event-id", self.event_id]]
        for flags in cases:
            with self.subTest(flags=flags), \
                 patch.object(sys, "argv", ["tasknode.py", "send", "--state", str(self.state),
                                           "--event-id", self.event_id, *flags]), \
                 patch.object(tasknode, "send", side_effect=AssertionError("no dispatch")), \
                 redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
                tasknode.main()

    def test_live_transport_receives_exact_idempotency_header(self):
        response = Mock()
        response.status = 200
        response.read.return_value = b'{"ok":true}'
        response.__enter__ = Mock(return_value=response)
        response.__exit__ = Mock(return_value=False)
        opener = Mock()
        opener.open.return_value = response
        with patch.object(tasknode.urllib.request, "build_opener", return_value=opener):
            tasknode.post("/events", {"event": self.event},
                          {"terminal_session": "fixture", "api_key": "fixture"},
                          idempotency_key=self.event_id)
        request = opener.open.call_args.args[0]
        self.assertEqual(request.get_header("Idempotency-key"), self.event_id)
        self.assertEqual(json.loads(request.data)["event"], self.event)


class IdentityTests(unittest.TestCase):
    def invoke(self, outputs, env=None):
        values = [subprocess.CompletedProcess([], 0, value if isinstance(value, str) else json.dumps(value), "")
                  for value in outputs]
        with patch.dict(os.environ, env or {"CODEX_HOME": "/fixture", "CORBANU_TASKNODE_PROFILE": '"fixture"'}, clear=True), \
             patch.object(tasknode.shutil, "which", return_value="/fixture/corbanu"), \
             patch.object(tasknode.subprocess, "run", side_effect=values) as runner:
            return tasknode.identity_check(), runner

    def test_missing_invalid_scope_and_old_helper_stop_before_account_reads(self):
        for env in ({"CODEX_HOME": "/fixture"}, {"CODEX_HOME": "/fixture", "CORBANU_TASKNODE_PROFILE": "{}"}):
            result, runner = self.invoke([], env)
            self.assertTrue(result["blockers"])
            runner.assert_not_called()
        result, runner = self.invoke(["Usage: tasknode status"])
        self.assertIn("installed_helper_lacks_profile_scope", result["blockers"])
        self.assertEqual(runner.call_count, 1)

    def test_read_only_success_redacts_every_unapproved_field(self):
        outputs = ["--profile", {"ok": True, "profile": "fixture", "origin": tasknode.ORIGIN},
                   {"ok": True, "wallet": "PRIVATE", "token": "PRIVATE"}]
        outputs += [{"task": {"id": task, "status": "Proposed", "body": "PRIVATE"}}
                    for task in tasknode.PROPOSED_TASKS]
        result, runner = self.invoke(outputs)
        self.assertEqual(result["profile"], "matches_inherited_scope")
        self.assertEqual(set(result["tasks"].values()), {"proposed"})
        self.assertEqual(result["entitlement"], "unverified")
        self.assertEqual(result["enrollment"], "unverified")
        self.assertNotIn("PRIVATE", json.dumps(result))
        self.assertEqual(runner.call_count, 6)
        for call in runner.call_args_list:
            self.assertIs(call.kwargs["stdin"], subprocess.DEVNULL)
            self.assertEqual(call.kwargs["timeout"], 15)
            self.assertFalse(set(call.args[0]) & {"accept", "balance", "evidence", "poll", "start"})

    def test_mismatched_scope_and_prompt_stop_successor_reads(self):
        for local in ({"ok": True, "profile": "other", "origin": tasknode.ORIGIN},
                      {"ok": True, "profile": "fixture", "origin": "https://other.example"},
                      "Keychain credential prompt"):
            result, runner = self.invoke(["--profile", local])
            self.assertTrue(result["blockers"])
            self.assertEqual(runner.call_count, 2)
            self.assertEqual(set(result["tasks"].values()), {"unverified"})

    def test_alias_conflict_and_task_state_changes_are_disclosed(self):
        result, runner = self.invoke([], {"CODEX_HOME": "/fixture", "CORBANU_HOME": "/other",
                                          "CORBANU_TASKNODE_PROFILE": '"fixture"'})
        self.assertIn("conflicting_inherited_home", result["blockers"])
        runner.assert_not_called()
        outputs = ["--profile", {"ok": True, "profile": "fixture", "origin": tasknode.ORIGIN}, {"ok": True}]
        outputs += [{"task": {"taskId": task, "statusKey": "accepted"}} for task in tasknode.PROPOSED_TASKS]
        result, _ = self.invoke(outputs)
        self.assertEqual(set(result["tasks"].values()), {"accepted"})
        self.assertIn("target_no_longer_proposed", result["blockers"])

    def test_cli_identity_emits_redacted_receipt_without_state(self):
        with patch.object(sys, "argv", ["tasknode.py", "identity-check"]), \
             patch.dict(os.environ, {}, clear=True), redirect_stdout(io.StringIO()) as output:
            tasknode.main()
        self.assertIn("missing_inherited_scope", json.loads(output.getvalue())["blockers"])


if __name__ == "__main__":
    unittest.main()

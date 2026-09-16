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
import time
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

    def test_historical_pf76_provenance_is_refused_with_reason(self):
        fixture = json.loads((control.HERE / "fixtures/recovery-old-state.json").read_text())
        historical = {**fixture["report"], "source_namespace": fixture["source_namespace"]}
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, "historical delivery-control provenance"):
            tasknode.enqueue(self.state, historical)
        self.assertEqual(self.snapshot(), before)
        for namespace in (None, "", "unknown", fixture["source_namespace"]):
            with self.subTest(namespace=namespace):
                value = {**fixture["report"]}
                if namespace is not None:
                    value["source_namespace"] = namespace
                with self.assertRaises(ValueError):
                    tasknode.enqueue(self.state, value)
                event = tasknode.event_for(fixture["report"], "fixture-workspace", ["fixture-task"], 0)
                path = self.state / "outbox" / (event["id"] + ".json")
                record = dict(event=event, status="blocked", attempts=0,
                              next_attempt_at=control.now(), source_namespace=namespace)
                control.atomic_json(path, record)
                original = path.read_bytes()
                self.assertIn("historical_source_reconciliation_required",
                              tasknode.prepare(self.state, event["id"])["blockers"])
                with self.assertRaises(ValueError):
                    tasknode.retry(self.state, event["id"])
                self.assertEqual(path.read_bytes(), original)
        record["status"] = "pending"
        control.atomic_json(path, record)
        original = path.read_bytes()
        self.config["tasknode"]["enabled"] = True
        control.atomic_json(self.state / "control.json", self.config)
        self.record_path.unlink()  # Leave only the synthetic historical record.
        self.assertEqual(tasknode.flush(self.state, {}, self.transport), 0)
        self.transport.assert_not_called()
        self.assertEqual(path.read_bytes(), original)

    def test_modern_pf76_provenance_reaches_mapping(self):
        modern = {**run(), "sprint_id": "PF-76-S01",
                  "source_namespace": "main-provider-profile-persistence"}
        self.config["tasknode"]["task_mappings"]["PF-76-S01"] = ["provider-task"]
        control.atomic_json(self.state / "control.json", self.config)
        control.report(self.state, modern)
        event_id = tasknode.enqueue(self.state, modern)
        self.assertEqual(tasknode.enqueue(self.state, modern), event_id)
        path = self.state / "outbox" / (event_id + ".json")
        record = control.read_json(path, self.state)
        self.assertEqual(record["source_namespace"], modern["source_namespace"])
        self.assertEqual(record["event"]["taskIds"], ["provider-task"])
        self.assertEqual(record["event"]["turnId"], "PF-76-S01")
        self.assertNotIn("source_namespace", record["event"])
        preview = tasknode.prepare(self.state, event_id)
        self.assertNotIn("historical_source_reconciliation_required", preview["blockers"])
        self.assertIn("posting_disabled", preview["blockers"])
        record["status"] = "blocked"
        control.atomic_json(path, record)
        tasknode.retry(self.state, event_id)
        self.assertEqual(control.read_json(path, self.state)["status"], "pending")
        with self.assertRaisesRegex(ValueError, "live writeback disabled"):
            tasknode.flush(self.state, {}, self.transport)
        self.transport.assert_not_called()
        self.config["tasknode"]["enabled"] = True
        control.atomic_json(self.state / "control.json", self.config)
        self.record_path.unlink()
        self.transport.return_value = (200, {"ok": True, "id": event_id})
        self.assertEqual(tasknode.flush(self.state, {}, self.transport), 1)
        self.assertEqual(self.transport.call_args.args[1]["event"]["taskIds"], ["provider-task"])

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

    def test_live_default_transport_posts_exact_prepared_request_once(self):
        payload = tasknode.prepare(self.state, self.event_id)["payload"]
        with patch.object(tasknode, "post", return_value=(200, {"ok": True, "id": self.event_id})) as post:
            result = tasknode.send(self.state, self.event_id, live=True, transport=None,
                                   activation_file=self.activation_file,
                                   credentials_file=self.auth_file)
        self.assertEqual(result["outcome"], "delivered")
        post.assert_called_once_with(
            "/events", payload, {"terminal_session": "fixture-session", "api_key": "fixture-key"},
            idempotency_key=self.event_id)

    def test_flush_reports_single_send_intents_without_reposting_or_mutating(self):
        self.live()
        other_id = tasknode.enqueue(self.state, {**run(), "run_id": "fixture-other"})
        before_queue = self.record_path.read_bytes()
        self.assertEqual(json.loads(before_queue)["status"], "pending")
        batch = Mock(return_value=(200, {"ok": True, "id": other_id}))
        with self.assertRaises(ValueError):
            tasknode.flush(self.state, {}, batch)
        batch.assert_not_called()
        control.atomic_json(self.state / "control.json",
                            {"tasknode": {**self.config["tasknode"], "enabled": True}})
        for intent_only in (False, True):
            with self.subTest(intent_only=intent_only):
                if intent_only:
                    (self.state / "send-receipts" / (self.event_id + ".result.json")).unlink()
                before = self.snapshot()
                result = tasknode.flush(self.state, {}, batch)
                self.assertEqual(result, 0 if intent_only else 1)
                self.assertEqual(result.single_sent, [self.event_id])
                self.assertEqual(self.record_path.read_bytes(), before_queue)
                for path, contents in before.items():
                    if not path.startswith("outbox/"):
                        self.assertEqual((self.state / path).read_bytes(), contents)
        other_event = control.read_json(self.state / "outbox" / (other_id + ".json"), self.state)["event"]
        batch.assert_called_once_with("/events", {"event": other_event}, {})

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


class CLIFenceTests(unittest.TestCase):
    setUp = SendTests.setUp
    snapshot = SendTests.snapshot

    def invoke(self, *args, transport=None):
        output, errors = io.StringIO(), io.StringIO()
        with patch.object(sys, "argv", ["tasknode.py", *map(str, args)]), \
             patch.object(tasknode, "post", transport or Mock(side_effect=AssertionError("unexpected transport"))), \
             redirect_stdout(output), redirect_stderr(errors):
            try:
                tasknode.main()
                code = 0
            except SystemExit as error:
                code = error.code
        return code, output.getvalue(), errors.getvalue()

    def denied(self, *args, reason=None):
        before = self.snapshot()
        code, output, errors = self.invoke(*args)
        self.assertNotEqual(code, 0)
        self.assertEqual(output, "")
        self.assertEqual(len(errors.splitlines()), 1)
        self.assertIn("Task Node command refused:", errors)
        self.assertNotIn("Traceback", errors)
        if reason:
            self.assertIn(reason, errors)
        self.assertEqual(self.snapshot(), before)
        return errors

    def env(self):
        return {"PATH": os.defpath, "HOME": str(self.state), "CODEX_HOME": str(self.state),
                "CORBANU_HOME": str(self.state), "PFTERMINAL_HOME": str(self.state),
                "PYTHONPATH": str(control.HERE), "PYTHONDONTWRITEBYTECODE": "1",
                "CORBANU_TEST_DISABLE_NATIVE_KEYRING": "1"}

    def child(self, args, prefix=""):
        # Actual entry point, isolated synthetic state, no network-capable transport.
        script = ("import tasknode, os\n"
                  "def denied(*a, **kw): raise AssertionError('unexpected transport')\n"
                  "tasknode.post = denied\n" + prefix + "\ntasknode.main()\n")
        return subprocess.run([sys.executable, "-c", script, *map(str, args)],
                              env=self.env(), text=True, capture_output=True, timeout=10)

    def enable_batch(self):
        self.config["tasknode"]["enabled"] = True
        control.atomic_json(self.state / "control.json", self.config)

    def flush_args(self):
        return ["flush", "--state", self.state, "--confirm-live",
                "--credentials-file", self.auth_file]

    def live_args(self):
        return ["send", "--state", self.state, "--event-id", self.event_id, "--live",
                "--owner-activation-file", self.activation_file,
                "--credentials-file", self.auth_file]

    def test_cli_lock_expiry_names_real_holder_and_path_and_recovers(self):
        holder = subprocess.Popen(
            [sys.executable, "-c",
             "import fcntl,sys; f=open(sys.argv[1],'a'); fcntl.flock(f,fcntl.LOCK_EX); "
             "print('locked',flush=True); sys.stdin.read()", str(self.state / ".outbox.lock")],
            env=self.env(), text=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        try:
            self.assertEqual(holder.stdout.readline().strip(), "locked")
            report = self.state / "run.json"
            control.atomic_json(report, run())
            self.enable_batch()
            for args in (["retry", "--state", self.state, "--event-id", self.event_id],
                         ["enqueue", "--state", self.state, "--report", report],
                         self.flush_args()):
                started = time.monotonic()
                result = self.child(args)
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertGreaterEqual(time.monotonic() - started, tasknode.CLI_LOCK_SECONDS)
                self.assertIn("another Task Node operation is holding the queue", result.stderr)
                self.assertIn(str(self.state / ".outbox.lock"), result.stderr)
                self.assertEqual(len(result.stderr.splitlines()), 1)
            self.config["tasknode"]["enabled"] = False
            control.atomic_json(self.state / "control.json", self.config)
            result = self.child(self.live_args())
            self.assertEqual(result.returncode, 2)
            self.assertIn("holding the queue", result.stderr)
            # Inspection remains lock-free, even while another process owns it.
            self.assertEqual(self.child(["status", "--state", self.state]).returncode, 0)
            self.assertEqual(self.child(["preview", "--state", self.state,
                                         "--event-id", self.event_id]).returncode, 0)
        finally:
            holder.communicate(timeout=5)
        self.assertEqual(self.child(["enqueue", "--state", self.state,
                                     "--report", report]).returncode, 0)
        self.assertFalse(tasknode.CLI_CONTEXT.get())

    def test_cli_batch_crash_stays_uncertain_until_explicit_named_retry(self):
        self.enable_batch()
        crash = ("def crash(*a, **kw):\n"
                 "    tasknode.atomic_json(tasknode.Path(os.environ[\"HOME\"]) / \"accepted.json\", a[1])\n"
                 "    os._exit(73)\n"
                 "tasknode.post = crash")
        result = self.child(self.flush_args(), crash)
        self.assertEqual(result.returncode, 73)
        self.assertEqual(control.read_json(self.state / "accepted.json", self.state), {"event": self.event})
        record = control.read_json(self.record_path, self.state)
        self.assertEqual((record["status"], record["attempts"]), ("uncertain", 1))
        before = self.snapshot()
        for _ in range(2):
            self.assertEqual(self.child(self.flush_args()).returncode, 0)
        self.assertEqual(self.snapshot(), before)
        result = self.child(["status", "--state", self.state])
        status = json.loads(result.stdout)
        self.assertEqual(status["status"], "uncertain")
        self.assertIn("no automatic retry", status["reason"])
        self.denied("retry", "--state", self.state, reason="requires --event-id")
        self.assertEqual(self.invoke("retry", "--state", self.state,
                                     "--event-id", self.event_id)[0], 0)
        record = control.read_json(self.record_path, self.state)
        self.assertEqual(record["event"], self.event)
        self.assertEqual((record["status"], record["attempts"]), ("pending", 1))
        self.assertEqual(self.invoke(*self.flush_args(), transport=self.transport)[0], 0)
        self.transport.assert_called_once()
        record = control.read_json(self.record_path, self.state)
        self.assertEqual((record["status"], record["attempts"]), ("delivered", 2))

    def test_cli_single_send_crash_retains_intent_and_denies_retry(self):
        result = self.child(self.live_args(), "def crash(*a, **kw): os._exit(73)\ntasknode.post = crash")
        self.assertEqual(result.returncode, 73)
        self.assertTrue((self.state / "send-receipts" / (self.event_id + ".intent.json")).exists())
        before = self.snapshot()
        for command in ("prepare", "preview"):
            preview = self.child([command, "--state", self.state, "--event-id", self.event_id])
            self.assertEqual(preview.returncode, 0, preview.stderr)
            value = json.loads(preview.stdout)
            self.assertEqual(value["status"], "uncertain")
            self.assertIn("external reconciliation required", value["reason"])
            self.assertIn("no automatic retry", value["reason"])
        result = self.child(self.live_args())
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(json.loads(result.stdout)["replayed"])
        self.assertEqual(json.loads(result.stdout)["outcome"], "uncertain")
        self.assertFalse(json.loads(result.stdout)["network_writes"])
        self.assertEqual(json.loads(self.child(["status", "--state", self.state]).stdout)["status"], "uncertain")
        self.denied("retry", "--state", self.state, "--event-id", self.event_id,
                    reason="retained receipts; external reconciliation")
        self.assertEqual(self.snapshot(), before)

    def test_cli_uncertain_retry_preserves_recorded_explanation(self):
        record = control.read_json(self.record_path, self.state)
        reason = "Delivery outcome unconfirmed; external reconciliation required."
        record.update(status="uncertain", attempts=3, error=reason)
        control.atomic_json(self.record_path, record)
        code, _, errors = self.invoke("retry", "--state", self.state, "--event-id", self.event_id)
        self.assertEqual(code, 0, errors)
        retried = control.read_json(self.record_path, self.state)
        self.assertEqual((retried["status"], retried["attempts"]), ("pending", 3))
        self.assertEqual(retried["error"], reason)
        self.assertEqual(retried["event"], self.event)

    def test_cli_preview_with_damaged_result_retains_payload_and_uncertainty(self):
        control.atomic_json(self.state / "send-receipts" / (self.event_id + ".intent.json"), {})
        result_path = self.state / "send-receipts" / (self.event_id + ".result.json")
        for damaged in (b'{"outcome":', b"\xff", b"[]"):
            with self.subTest(damaged=damaged):
                result_path.write_bytes(damaged)
                before = self.snapshot()
                for command in ("prepare", "preview"):
                    result = self.child([command, "--state", self.state, "--event-id", self.event_id])
                    self.assertEqual(result.returncode, 0, result.stderr)
                    value = json.loads(result.stdout)
                    self.assertEqual(value["status"], "uncertain")
                    self.assertEqual(value["payload"], {"event": self.event})
                    self.assertIn("external reconciliation required", value["reason"])
                    self.assertIn("no automatic retry", value["reason"])
                    self.assertFalse(value["send_authorized"])
                    self.assertFalse(value["network_writes"])
                self.assertEqual(self.snapshot(), before)

    def test_cli_recovery_operations_are_absent_and_denied(self):
        for command in ("delete-receipt", "edit-payload", "rewrite-id", "clear-outbox", "reset-attempts"):
            with self.subTest(command=command):
                self.denied(command, "--state", self.state, reason="unsupported command")
        for flag in ("--delete-receipt", "--payload", "--new-id", "--clear-outbox",
                     "--reset-attempts", "--all", "--force"):
            with self.subTest(flag=flag):
                self.denied("retry", "--state", self.state, "--event-id", self.event_id, flag,
                            reason="unsupported command")
        code, help_text, _ = self.invoke("--help")
        self.assertEqual(code, 0)
        for forbidden in ("delete-receipt", "edit-payload", "rewrite-id", "clear-outbox",
                          "reset-attempts", "--payload", "--all", "--force"):
            self.assertNotIn(forbidden, help_text)

    def test_cli_selectors_cannot_create_rewrite_or_bulk_retry_records(self):
        for command in ("preview", "send", "retry"):
            flags = ["--dry-run"] if command == "send" else []
            for event_id in ("../escape", "cc-" + "0" * 64):
                self.denied(command, "--state", self.state, "--event-id", event_id, *flags)
            self.denied(command, "--state", self.state, "--event-id", self.event_id,
                        "--event-id", self.event_id, *flags, reason="exactly once")
        for status in ("pending", "delivered"):
            record = control.read_json(self.record_path, self.state)
            record["status"] = status
            control.atomic_json(self.record_path, record)
            self.denied("retry", "--state", self.state, "--event-id", self.event_id,
                        reason="only blocked or uncertain")
        record["status"] = "blocked"
        control.atomic_json(self.record_path, record)
        self.assertEqual(self.invoke("retry", "--state", self.state, "--event-id", self.event_id)[0], 0)
        self.assertEqual(control.read_json(self.record_path, self.state)["event"], self.event)

    def test_cli_credentials_are_file_only_and_never_echoed(self):
        secret = "synthetic-sensitive-value"
        for flag in ("--api-key", "--terminal-session", "--token", "--credentials", "--credentials-f"):
            errors = self.denied("flush", "--state", self.state, "--confirm-live", flag, secret)
            self.assertNotIn(secret, errors)
        with patch.dict(os.environ, {"TASKNODE_API_KEY": secret, "TASKNODE_TERMINAL_SESSION": secret,
                                     "API_KEY": secret, "TERMINAL_SESSION": secret}):
            self.denied("flush", "--state", self.state, "--confirm-live",
                        reason="private --credentials-file")
        for contents in (secret, '{"terminal_session": "' + secret + '",', "[]"):
            self.auth_file.write_text(contents)
            errors = self.denied(*self.flush_args(), reason="credentials file")
            self.assertNotIn(secret, errors)
        control.atomic_json(self.auth_file, {"terminal_session": secret, "api_key": secret})
        self.auth_file.chmod(0o644)
        self.denied(*self.flush_args(), reason="owner-only")
        self.auth_file.chmod(0o600)
        self.enable_batch()
        code, output, errors = self.invoke(*self.flush_args(), transport=self.transport)
        self.assertEqual(code, 0, errors)
        self.assertNotIn(secret, output + errors)
        self.assertEqual(self.transport.call_args.args[2], {"terminal_session": secret, "api_key": secret})

    def test_cli_enqueue_requires_validated_report_and_field_byte_checks(self):
        report = self.state / "report.json"
        for value in (self.event, {**run(), "extra": "field"}, {**run(), "summary": "猫" * 800},
                      {**run(), "branch": "猫" * 200}, {**run(), "summary": "api_key=synthetic"},
                      {**run(), "updated_at": "synthetic-invalid-timestamp"}):
            control.atomic_json(report, value)
            with self.subTest(fields=list(value)):
                self.denied("enqueue", "--state", self.state, "--report", report)
        control.atomic_json(report, {**run(), "run_id": "new-valid-run"})
        code, output, errors = self.invoke("enqueue", "--state", self.state, "--report", report)
        self.assertEqual(code, 0, errors)
        self.assertTrue((self.state / "outbox" / (output.strip() + ".json")).exists())
        for command in ("flush", "enroll", "status", "retry"):
            self.denied(command, "--state", self.state, "--report", report,
                        *(["--event-id", self.event_id] if command == "retry" else []))
        self.denied("enqueue", "--state", self.state, "--payload", json.dumps(self.event))

    def test_cli_inspection_is_read_only_and_status_is_json_lines(self):
        tasknode.enqueue(self.state, {**run(), "run_id": "other"})
        before = self.snapshot()
        for command in ("status", "preview", "prepare"):
            args = ["--event-id", self.event_id] if command != "status" else []
            code, output, errors = self.invoke(command, "--state", self.state, *args)
            self.assertEqual(code, 0, errors)
            objects = [json.loads(line) for line in output.splitlines()]
            self.assertEqual(len(objects), 2 if command == "status" else 1)
            self.assertEqual(self.snapshot(), before)
            self.denied(command, "--state", self.state, *args, "--credentials-file", self.auth_file)
            self.denied(command, "--state", self.state, *args, "--confirm-live")

    def test_cli_old_records_and_receipts_are_never_aged_out(self):
        self.assertEqual(self.invoke(*self.live_args(), transport=self.transport)[0], 0)
        before = self.snapshot()
        old = 946684800
        for path in self.state.rglob("*"):
            if path.is_file():
                os.utime(path, (old, old))
        future = (control.timestamp(control.now()) + dt.timedelta(days=3650)).isoformat()
        with patch.object(tasknode, "now", return_value=future):
            self.assertEqual(self.invoke("status", "--state", self.state)[0], 0)
            self.assertEqual(self.invoke("preview", "--state", self.state,
                                         "--event-id", self.event_id)[0], 0)
            self.denied("retry", "--state", self.state, "--event-id", self.event_id,
                        reason="retained receipts")
            self.assertEqual(self.snapshot(), before)
            self.enable_batch()
            retained = self.snapshot()
            self.assertEqual(self.invoke(*self.flush_args())[0], 0)
            self.assertEqual(self.snapshot(), retained)

    def test_cli_posting_stays_off_and_refusal_names_local_blockers(self):
        self.denied(*self.flush_args(), reason="live writeback disabled")
        self.denied("send", "--state", self.state, "--event-id", self.event_id, "--live",
                    reason="owner activation and credentials")
        self.activation["gates"]["payload_review"] = False
        control.atomic_json(self.activation_file, self.activation)
        self.denied(*self.live_args(), reason="owner activation does not authorize")
        (self.state / "enrollment.json").unlink()
        self.denied(*self.live_args(), reason="local workspace enrollment unverified")
        self.assertFalse(control.read_json(self.state / "control.json", self.state)["tasknode"]["enabled"])


if __name__ == "__main__":
    unittest.main()

"""Real private TMUX + harmless shell; rollout records below are synthetic."""
import copy
import json
import sys
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time
import tomllib
import unittest
import uuid
from types import SimpleNamespace
from unittest.mock import patch

import fable_launcher as f
import owner_daemon as owner
import owner_tmux as t
import decisions as d

SHELL = """#!/bin/bash
umask 077
printf '\\033[?2004hREADY\\n'
while IFS= read -r line; do
    printf '%s\\n' "$line" >> "$HOME/received"
    line=${line//$'\\e[200~'/}
    line=${line//$'\\e[201~'/}
    case "$line" in
        START) printf 'RETURN\\nfixture result\\n';;
        /quit) [ -e "$HOME/ignore-quit" ] || exit 0;;
        CHILD) (trap '' HUP; sleep 30) & ;;
        CRASH) exit 23;;
        *) printf '%s\\n' "$line";;
    esac
done
"""


# A real pane process writes its own synthetic completed-turn log. No inference,
# credentials or network; parent tests cannot supply evidence to BridgeReceiver.
BRIDGE_SHELL = """#!PYTHON
import json, os, pathlib, sys, time, tty
tty.setcbreak(sys.stdin.fileno())
home = pathlib.Path(os.environ["HOME"])
path = home / "sessions/fixture.jsonl"
print("\\033[?2004hREADY", flush=True)
for line in sys.stdin:
    line = line.replace("\\x1b[200~", "").replace("\\x1b[201~", "").rstrip("\\n")
    if line == "/quit":
        break
    try:
        prompt = json.loads(line)
    except ValueError:
        print(line, flush=True)
        continue
    ack = prompt["expected_ack"]
    mode = (home / "mode").read_text() if (home / "mode").exists() else ""
    if mode == "silent":
        continue
    if mode == "wrong-ack":
        ack += " "
    if mode == "noncanonical-ack":
        ack = json.dumps(json.loads(ack), indent=1)
    turn = prompt["nonce"]
    def row(kind, **payload):
        return dict(type=kind, payload=payload)
    records = [
        row("event_msg", type="task_started", turn_id=turn),
        row("turn_context", turn_id=turn, cwd=os.getcwd(), model="fixture-model",
            model_provider="fixture", effort="high", approval_policy="never",
            sandbox_policy=dict(type="read-only")),
        row("event_msg", type="user_message", message=line),
        row("event_msg", type="model_response_completed", turn_id=turn,
            model="fixture-model", model_provider_id="fixture", response_id="response-" + turn),
        row("event_msg", type="task_complete", turn_id=turn, last_agent_message=ack)]
    if mode == "echo-only":
        records = records[:-2]
    if mode == "wrong-user":
        records[2]["payload"]["message"] += " "
    if mode == "wrong-runtime":
        records[1]["payload"]["model"] = "other-model"
    print(ack, flush=True)
    with path.open("a") as stream:
        raw = "".join(json.dumps(r) + "\\n" for r in records)
        if mode == "mid-append":
            stream.write(raw[:-1])
            stream.flush()
            while not (home / "complete-append").exists():
                time.sleep(0.01)
            stream.write(raw[-1:])
        else:
            stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())
"""


def record(kind, **payload):
    return {"type": kind, "payload": payload}


class TmuxTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="ot-", dir=Path("/tmp").resolve())
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.binary = self.root / "harmless-shell"
        f.write_file(self.binary, SHELL, mode=0o700)
        tmux = shutil.which("tmux")
        self.assertIsNotNone(tmux, "real TMUX is required; do not skip transport qualification")
        self.config = {"kind": "tmux", "binary": str(self.binary),
                       "binary_sha256": f.file_digest(self.binary), "tmux": str(Path(tmux).resolve()),
                       "runs_dir": str(self.root), "auth_link": str(self.root / "unused/auth.json")}
        self.binding = {"action_id": "fixture-01", "allocation_digest": "a" * 64,
                        "claim": str(uuid.uuid4()), "model": "fixture-model", "provider": "fixture",
                        "effort": "high", "worktree": str(self.root), "sandbox": "read-only",
                        "approval": "never"}
        self.worker = t.TmuxAdapter(self.config).prepare(self.binding, "Harmless fixture assignment.")
        self.addCleanup(self.cleanup_worker)
        self.records = [record("session_meta", id=str(uuid.uuid4()), session_id=str(uuid.uuid4()),
                               cwd=str(self.root), source="cli", model_provider="fixture")]

    def cleanup_worker(self):
        # Only this test's private server. No default-server commands or real profiles.
        self.worker.tmux("kill-server", check=False)

    def wait(self, predicate):
        end = time.monotonic() + 3
        while time.monotonic() < end:
            result = predicate()
            if result:
                return result
            time.sleep(0.1)
        self.fail("fixture timed out: " + repr(self.worker.tmux(
            "capture-pane", "-p", "-t", self.worker.meta["session"]).stdout))

    def launch(self):
        self.worker.launch()
        self.wait(lambda: "READY" in self.worker.tmux(
            "capture-pane", "-p", "-t", self.worker.meta["session"]).stdout)

    def write_records(self, raw=None):
        sessions = self.worker.run / "home/sessions"
        sessions.mkdir(mode=0o700, exist_ok=True)
        path = sessions / "fixture.jsonl"
        import json
        f.write_file(path, raw if raw is not None else "".join(
            json.dumps(r) + "\n" for r in self.records))
        return path

    def turn(self, text, final, turn):
        self.records += [
            record("event_msg", type="task_started", turn_id=turn),
            record("turn_context", turn_id=turn, cwd=str(self.root), model="fixture-model",
                   model_provider="fixture", effort="high", approval_policy="never",
                   sandbox_policy={"type": self.worker.binding["sandbox"]}),
            record("event_msg", type="user_message", message=text),
            record("event_msg", type="model_response_completed", turn_id=turn,
                   model="fixture-model", model_provider_id="fixture", response_id="response-" + turn),
            record("event_msg", type="task_complete", turn_id=turn, last_agent_message=final)]
        self.write_records()

    def ack(self):
        self.launch()
        self.worker.prompt()
        self.turn(self.worker.meta["prompt"], self.worker.meta["ack"], "ack-turn")
        self.assertTrue(self.worker.inspect()["ack"])

    def fixture_input(self, text):
        self.worker.tmux("send-keys", "-t", self.worker.meta["session"], "-l", "--", text)
        self.worker.tmux("send-keys", "-t", self.worker.meta["session"], "Enter")

    def bridge_receiver(self, *, timeout=3):
        f.write_file(self.binary, BRIDGE_SHELL.replace("PYTHON", sys.executable, 1), mode=0o700)
        self.worker.config["binary_sha256"] = f.file_digest(self.binary)
        f.write_json(self.worker.run / "worker.json", self.worker.meta)
        self.launch()
        self.turn("bootstrap", "ready", "bootstrap-turn")
        owner = dict(agent=self.binding["action_id"], allocation=self.binding["allocation_digest"], running=True)
        return t.BridgeReceiver(self.worker, owner, timeout=timeout, handoff_timeout=3)

    def bridge_request(self, receiver, key="handoff-1"):
        request = dict(handoff=key, owner=receiver.owner, payload=dict(answer="Five testers"))
        request["payload_digest"] = d.digest(request["payload"])
        ack = dict(handoff=key, agent=receiver.owner["agent"], allocation=receiver.owner["allocation"],
                   payload_digest=request["payload_digest"], receipt_id=d.digest([key]))
        return request, ack

    def test_bridge_direct_capture_exact_completed_receiver_turn(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        evidence = receiver.deliver(request, ack)
        receiver.verify(evidence, request, ack, consume=True)
        receiver.verify(evidence, request, ack)
        self.assertEqual(evidence["expected_ack"].encode(), d.canonical(ack))
        self.assertEqual(evidence["receiver"]["socket"], self.worker.meta["socket"])
        self.assertEqual(evidence["receiver"]["pane"], self.worker.inspect()["process"]["pane"])
        self.assertEqual(evidence["rollout"]["turn_id"], evidence["nonce"])
        self.assertGreater(evidence["rollout"]["after_bytes"], evidence["rollout"]["before_bytes"])

    def test_bridge_wrong_agent_allocation_payload_send_no_keys(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        for wrong in ({**request, "owner": {**request["owner"], "agent": "other"}},
                      {**request, "owner": {**request["owner"], "allocation": "b" * 64}},
                      {**request, "payload_digest": "c" * 64}):
            with self.subTest(wrong=wrong), self.assertRaises(d.Invalid):
                receiver.deliver(wrong, ack)
        self.assertFalse(list(self.worker.run.glob("bridge-*.json")))

    def test_bridge_caller_edits_native_shape_and_reuse_are_refused(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        evidence = receiver.deliver(request, ack)
        for field, value in (("agent", "other"), ("allocation", "b" * 64),
                             ("payload_digest", "c" * 64), ("expected_ack", evidence["expected_ack"] + " "),
                             ("capture_digest", "d" * 64), ("nonce", "old-capture"),
                             ("receiver", {**evidence["receiver"], "session": "other"}),
                             ("receiver", {**evidence["receiver"], "socket": "/other/socket"})):
            with self.subTest(field=field), self.assertRaises(d.Invalid):
                receiver.verify({**evidence, field: value}, request, ack, consume=True)
        with self.assertRaises(d.Invalid):
            receiver.verify(dict(type="assistant", tool="multi_agent_v1.wait_agent",
                                 agent=request["owner"]["agent"], text=evidence["expected_ack"]),
                            request, ack, consume=True)
        receiver.verify(evidence, request, ack, consume=True)
        with self.assertRaises(d.Invalid):
            receiver.verify(evidence, request, ack, consume=True)
        with self.assertRaises(d.Invalid):
            t.BridgeReceiver(self.worker, receiver.owner).verify(evidence, request, ack, consume=True)

    def test_bridge_old_handoff_rollout_truncation_edit_and_pane_edit_refused(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        evidence = receiver.deliver(request, ack)
        newer, newer_ack = self.bridge_request(receiver, "handoff-2")
        with self.assertRaises(d.Invalid):
            receiver.verify(evidence, newer, newer_ack, consume=True)
        path = Path(evidence["rollout"]["path"])
        original = path.read_bytes()
        for raw in (original[:-1], original.replace(b"ready", b"edited"), original[:evidence["rollout"]["before_bytes"]]):
            f.write_file(path, raw)
            with self.assertRaises(d.Invalid):
                receiver.verify(evidence, request, ack, consume=True)
        # Atomic replacements also invalidate the pinned rollout inode.
        f.write_file(path, original)
        with self.assertRaises(d.Invalid):
            receiver.verify(evidence, request, ack, consume=True)

    def test_bridge_live_pane_changes_and_stale_capture_refused(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        evidence = receiver.deliver(request, ack)
        # Otherwise-valid evidence must fail solely because its capture is old.
        with patch.object(t, "time", SimpleNamespace(monotonic=lambda: time.monotonic() + 30)), self.assertRaises(d.Invalid):
            receiver.verify(evidence, request, ack, consume=True)
        receiver.verify(evidence, request, ack, consume=True)
        self.fixture_input("changed pane")
        self.wait(lambda: "changed pane" in self.worker.tmux(
            "capture-pane", "-p", "-t", self.worker.meta["session"]).stdout)
        with self.assertRaises(d.Invalid):
            receiver.verify(evidence, request, ack)

    def test_bridge_mid_append_rollout_repolls_before_issuing_evidence(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        f.write_file(self.worker.run / "home/mode", "mid-append")
        read_file = f.read_file
        partial_reads = []
        def read(path, *args, **kwargs):
            raw = read_file(path, *args, **kwargs)
            if Path(path).suffix == ".jsonl" and not raw.endswith(b"\n"):
                self.assertFalse(receiver.issued)
                partial_reads.append(raw)
                f.write_file(self.worker.run / "home/complete-append", "")
            return raw
        with patch.object(f, "read_file", side_effect=read):
            evidence = receiver.deliver(request, ack)
        self.assertTrue(partial_reads, "must observe an actual incomplete rollout read")
        self.assertTrue(receiver.issued[evidence["nonce"]][1].endswith(b"\n"))
        receiver.verify(evidence, request, ack, consume=True)

    def test_bridge_incomplete_and_malformed_rollouts_never_issue_evidence(self):
        receiver = self.bridge_receiver()
        read_file = f.read_file
        for mode in ("partial", "malformed"):
            with self.subTest(mode=mode):
                self.turn("bootstrap", "ready", "baseline-" + mode)
                request, ack = self.bridge_request(receiver, mode)
                baseline = receiver.rollout()[0]
                reads = []
                def read(path, *args, **kwargs):
                    raw = read_file(path, *args, **kwargs)
                    if Path(path).suffix == ".jsonl" and raw != baseline:
                        reads.append(raw)
                        return raw[:-1] if mode == "partial" else raw + b"malformed\n"
                    return raw
                receiver.handoff_timeout = 0.7
                expectation = (self.assertRaises(d.Invalid) if mode == "partial"
                               else self.assertRaisesRegex(f.LaunchError, "invalid_json"))
                with patch.object(f, "read_file", side_effect=read), expectation:
                    receiver.deliver(request, ack)
                self.assertFalse(receiver.issued)
                self.assertGreater(len(reads), 1 if mode == "partial" else 0)
                self.assertTrue((self.worker.run / ("bridge-" + d.digest(mode) + ".json")).exists())

    def test_bridge_ack_after_collection_deadline_never_issues_witness(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        elapsed = [0]
        capture = receiver.capture
        captures = []
        def slow_capture():
            screen = capture()
            captures.append(screen)
            if len(captures) > 1:
                elapsed[0] = 301
            return screen
        clock = SimpleNamespace(monotonic=lambda: time.monotonic() + elapsed[0], sleep=time.sleep)
        with (patch.object(t, "time", clock), patch.object(receiver, "capture", side_effect=slow_capture),
              self.assertRaises(d.Invalid)):
            receiver.deliver(request, ack)
        self.assertEqual(2, len(captures))
        self.assertFalse(receiver.issued)

    def test_bridge_witness_expiring_during_verification_is_refused(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        evidence = receiver.deliver(request, ack)
        elapsed = [0]
        capture = receiver.capture
        def slow_capture():
            screen = capture()
            elapsed[0] = 30
            return screen
        with (patch.object(t, "time", SimpleNamespace(monotonic=lambda: time.monotonic() + elapsed[0])),
              patch.object(receiver, "capture", side_effect=slow_capture), self.assertRaises(d.Invalid)):
            receiver.verify(evidence, request, ack, consume=True)
        self.assertFalse(receiver.used)

    def test_bridge_exact_one_byte_wrong_ack_and_uncompleted_echo_hold(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        for mode in ("wrong-ack", "noncanonical-ack", "echo-only", "wrong-user", "wrong-runtime", "silent"):
            with self.subTest(mode=mode):
                f.write_file(self.worker.run / "home/mode", mode)
                # Fresh completed baseline per attempt; failed attempts are never
                # retried, and each uses a different durable handoff intent.
                self.turn("bootstrap", "ready", "baseline-" + mode)
                request, ack = self.bridge_request(receiver, mode)
                reason = {"wrong-ack": "wrong_ack", "noncanonical-ack": "wrong_ack",
                          "wrong-user": "wrong_submission", "wrong-runtime": "wrong_runtime"}.get(mode)
                receiver.handoff_timeout = 3 if reason else 0.5
                expectation = (self.assertRaisesRegex(f.LaunchError, reason) if reason
                               else self.assertRaises(d.Invalid))
                with expectation:
                    receiver.deliver(request, ack)
                self.assertFalse(receiver.issued)
        with self.assertRaises((d.Invalid, f.LaunchError)):
            receiver.deliver(request, ack)

    def test_bridge_prior_capture_replay_and_different_socket_receiver_refused(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        evidence = receiver.deliver(request, ack)
        receiver.verify(evidence, request, ack, consume=True)
        old_screen = receiver.capture().decode()
        request2, ack2 = self.bridge_request(receiver, "handoff-2")
        original = self.worker.tmux
        def replay(*args, **kwargs):
            result = original(*args, **kwargs)
            if args[0] == "capture-pane":
                result.stdout = old_screen
            return result
        with patch.object(self.worker, "tmux", side_effect=replay), self.assertRaises(d.Invalid):
            receiver.deliver(request2, ack2)
        self.assertNotIn("handoff-2", [v[0]["handoff"] for v in receiver.issued.values()])
        other = TmuxTests()
        other.setUp()
        self.addCleanup(other.doCleanups)
        second = other.bridge_receiver()
        evidence2 = second.deliver(request, ack)
        self.assertNotEqual(evidence["receiver"]["socket"], evidence2["receiver"]["socket"])
        self.assertNotEqual(evidence["receiver"]["session"], evidence2["receiver"]["session"])
        with self.assertRaises(d.Invalid):
            receiver.verify(evidence2, request, ack, consume=True)
        with self.assertRaises(d.Invalid):
            second.verify(evidence, request, ack, consume=True)

    def test_bridge_uncertain_enter_never_resends_after_receiver_reload(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        original = self.worker.tmux
        def uncertain(*args, **kwargs):
            if args[0] == "send-keys":
                raise subprocess.TimeoutExpired("fixture enter", 3)
            return original(*args, **kwargs)
        with patch.object(self.worker, "tmux", side_effect=uncertain), self.assertRaises(subprocess.TimeoutExpired):
            receiver.deliver(request, ack)
        reloaded = t.BridgeReceiver(t.Worker(self.worker.run), receiver.owner)
        with patch.object(reloaded.worker, "tmux", wraps=reloaded.worker.tmux) as calls:
            with self.assertRaisesRegex(f.LaunchError, "effect_uncertain"):
                reloaded.deliver(request, ack)
        self.assertFalse(any(c.args[0] in ("load-buffer", "paste-buffer", "send-keys") for c in calls.call_args_list))
        self.assertEqual({}, receiver.issued)

    def test_bridge_visible_credential_prompt_sends_no_keys(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        self.fixture_input("Sign in")
        self.wait(lambda: "Sign in" in self.worker.tmux(
            "capture-pane", "-p", "-t", self.worker.meta["session"]).stdout)
        with self.assertRaises(d.Invalid):
            receiver.deliver(request, ack)
        self.assertFalse(list(self.worker.run.glob("bridge-*.json")))

    def test_bridge_preexisting_exact_ack_sends_no_keys(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        self.fixture_input("existing " + d.canonical(ack).decode())
        self.wait(lambda: d.canonical(ack) in receiver.capture())
        with self.assertRaises(d.Invalid):
            receiver.deliver(request, ack)
        self.assertFalse(list(self.worker.run.glob("bridge-*.json")))

    def test_bridge_receiver_identity_drift_refuses_real_other_session(self):
        receiver = self.bridge_receiver()
        request, ack = self.bridge_request(receiver)
        evidence = receiver.deliver(request, ack)
        self.worker.tmux("rename-session", "-t", self.worker.meta["session"], "renamed")
        with self.assertRaises((d.Invalid, f.LaunchError)):
            receiver.verify(evidence, request, ack, consume=True)

    def test_private_home_environment_socket_and_pinned_argv(self):
        with patch.dict(os.environ, {"CORBANU_HOME": "/never/use/live", "SOME_API_KEY": "synthetic"}):
            self.launch()
        self.assertEqual(0o700, (self.worker.run / "home").stat().st_mode & 0o777)
        self.assertEqual(self.config["auth_link"], os.readlink(self.worker.run / "home/auth.json"))
        self.assertNotIn("SOME_API_KEY", self.worker.env)
        self.assertEqual(str(self.worker.run / "home"), self.worker.env["CORBANU_HOME"])
        self.assertLess(len(self.worker.meta["socket"].encode()), 100)
        state = self.worker.inspect()
        self.assertTrue(state["identity_valid"])
        self.assertEqual("alive", state["liveness"])
        launch = f.strict_json(f.read_file(self.worker.run / "launch-intent.json", 65536))
        self.assertEqual(str(self.binary), launch["argv"][0])
        self.assertIn("fixture-model", launch["argv"])
        for path in self.worker.run.glob("*.json"):
            self.assertEqual(0o600, path.stat().st_mode & 0o777)

    def test_startup_config_trusts_only_exact_bound_worktree(self):
        for name in ('ordinary', 'spaces . "quotes" \\ slash = é 🚀'):
            with self.subTest(name=name):
                worktree = self.root / name
                worktree.mkdir()
                worker = t.TmuxAdapter(self.config).prepare(
                    {**self.binding, "worktree": str(worktree)}, "Fixture assignment.")
                path = worker.run / "home/config.toml"
                self.assertEqual(0o600, path.stat().st_mode & 0o777)
                self.assertEqual({
                    "check_for_update_on_startup": False,
                    "tui": {"animations": False}, "analytics": {"enabled": False},
                    "projects": {str(worktree): {"trust_level": "trusted"}},
                }, tomllib.loads(path.read_text()))

    def test_startup_overrides_preserve_each_bound_policy(self):
        for sandbox in ("read-only", "workspace-write", "danger-full-access"):
            for approval in ("never", "on-request", "untrusted"):
                with self.subTest(sandbox=sandbox, approval=approval):
                    worker = t.TmuxAdapter(self.config).prepare(
                        {**self.binding, "sandbox": sandbox, "approval": approval}, "Fixture.")
                    with patch.object(worker, "tmux"), patch.object(
                            worker, "inspect", return_value={"identity_valid": True}):
                        worker.launch()
                    launch = f.strict_json(f.read_file(worker.run / "launch-intent.json", 65536))
                    self.assertEqual([
                        str(self.binary), "--no-alt-screen", "-C", str(self.root),
                        "--model", "fixture-model", "-c", 'model_provider="fixture"',
                        "-c", 'model_reasoning_effort="high"',
                        "-c", "check_for_update_on_startup=false", "-c", "tui.animations=false",
                        "-c", "analytics.enabled=false",
                        "--sandbox", sandbox, "--ask-for-approval", approval,
                    ], launch["argv"])

    def test_bracketed_paste_echo_is_not_ack_and_start_is_denied(self):
        self.launch()
        self.worker.prompt()
        received = self.worker.run / "home/received"
        self.wait(lambda: received.exists() and "assignment." in received.read_text())
        text = received.read_text()
        self.assertIn("\x1b[200~", text)
        self.assertIn("\x1b[201~", text)
        self.assertIn(self.worker.meta["ack"], text)
        self.assertFalse(self.worker.inspect().get("ack", False))
        with self.assertRaisesRegex(f.LaunchError, "ack_required"):
            self.worker.start()
        self.assertFalse((self.worker.run / "start-intent.json").exists())

    def test_ack_start_return_resume_and_clean_shutdown(self):
        self.ack()
        self.worker = t.Worker(self.worker.run)
        self.worker.start()
        self.assertFalse(self.worker.inspect()["submitted"])
        self.turn("START", "RETURN\nfixture result", "work-turn")
        state = self.worker.inspect()
        self.assertTrue(state["submitted"])
        self.assertEqual("RETURN\nfixture result", state["returned"])
        self.assertEqual(self.records[0]["payload"]["session_id"], state["session_id"])
        self.assertEqual("work-turn", state["turn_id"])
        receipt = self.worker.close()
        self.assertTrue(receipt["clean"], receipt)
        self.assertFalse(receipt["forced"])
        self.assertNotEqual(0, self.worker.tmux("list-sessions", check=False).returncode)

    def test_daemon_records_durable_return_after_pane_is_killed(self):
        from coordinator import digest
        from test_owner_daemon import WorkerLifecycleTests
        case = WorkerLifecycleTests()
        case.setUp()
        self.addCleanup(case.tearDown)
        self.addCleanup(case.doCleanups)
        original_worker = self.worker
        def cleanup():
            try:
                self.cleanup_worker()
            finally:
                self.worker = original_worker
        self.addCleanup(cleanup)
        case.configure()
        case.fake.stop()
        f.write_file(self.binary, SHELL.replace("READY", "Corbanu Terminal model: fixture-model high READY"),
                     mode=0o700)
        case.config["transport"] = {**self.config, "binary_sha256": f.file_digest(self.binary)}
        f.write_json(case.config_path, case.config)
        case.sql("UPDATE meta SET config_digest=?", (digest(case.config),))
        case.authority["config_digest"] = digest(case.config)
        case.arm()
        case.prepared()
        self.root = case.root
        self.records[0]["payload"]["cwd"] = str(self.root)
        prepare = t.TmuxAdapter.prepare
        def capture(adapter, binding, assignment):
            self.worker = prepare(adapter, binding, assignment)
            return self.worker
        with patch.object(t.TmuxAdapter, "prepare", capture):
            result = case.tick()["actions"]["one"]
        self.assertIn(result, ("awaiting_ready", "awaiting_ack"))
        self.wait(lambda: self.worker.inspect().get("ready"))
        self.assertEqual("awaiting_ack", case.tick()["actions"]["one"])
        self.turn(self.worker.meta["prompt"], self.worker.meta["ack"], "ack-turn")
        self.assertEqual("awaiting_working", case.tick()["actions"]["one"])
        self.turn("START", "RETURN\nfixture result", "work-turn")
        proc = self.worker.inspect()["process"]
        os.kill(proc["pid"], signal.SIGKILL)
        self.wait(lambda: self.worker.inspect()["liveness"] == "crashed")
        with patch.object(t.Worker, "send", side_effect=AssertionError("unexpected key delivery")):
            for _ in range(2):
                result = case.tick()
                self.assertEqual(("ACTIVE", "returned"), (result["state"], result["actions"]["one"]))
        self.assertEqual("returned", case.c.snapshot()["actions"]["one"]["status"])
        self.assertEqual([], case.sql("SELECT * FROM holds"))
        self.assertEqual([("work-turn", "crashed")], case.sql(
            "SELECT active_turn_id,terminal_status FROM processes"))
        with self.assertRaisesRegex(f.LaunchError, "worker_not_alive"):
            self.worker.send("quit", "/quit")
        self.assertFalse((self.worker.run / "quit-intent.json").exists())

    def test_wrong_digest_model_effort_claim_prompt_or_turn_cannot_ack(self):
        self.ack()
        original = copy.deepcopy(self.records)
        mutations = [(5, "last_agent_message", "ACK fixture-01 " + "b" * 64 + " fixture-model high"),
                     (2, "model", "other-model"), (2, "effort", "low"),
                     (3, "message", self.worker.meta["prompt"].replace(self.binding["claim"], "wrong-claim")),
                     (4, "turn_id", "other-turn"), (4, "model_provider_id", "other"),
                     (5, "turn_id", "other-turn")]
        for index, field, value in mutations:
            with self.subTest(field=field):
                self.records = copy.deepcopy(original)
                self.records[index]["payload"][field] = value
                self.write_records()
                state = self.worker.inspect()
                self.assertFalse(state["identity_valid"])
                self.assertFalse(state.get("ack", False))
                with self.assertRaises(f.LaunchError):
                    self.worker.start()

    def test_inherited_session_wrong_worktree_and_extra_turn_hold(self):
        self.ack()
        original = copy.deepcopy(self.records)
        for key, value in (("cwd", "/wrong"), ("source", "exec"), ("forked_from_id", str(uuid.uuid4()))):
            self.records = copy.deepcopy(original)
            self.records[0]["payload"][key] = value
            self.write_records()
            self.assertFalse(self.worker.inspect()["identity_valid"])
        self.records = original
        self.turn("START", "RETURN\nunsolicited", "work-turn")
        self.assertFalse(self.worker.inspect()["identity_valid"])

    def test_duplicate_launch_prompt_and_start_are_never_retried(self):
        self.ack()
        with self.assertRaises(f.LaunchError):
            self.worker.launch()
        with self.assertRaisesRegex(f.LaunchError, "effect_uncertain"):
            self.worker.prompt()
        self.worker.start()
        with self.assertRaisesRegex(f.LaunchError, "effect_uncertain"):
            t.Worker(self.worker.run).start()

    def test_partial_rollout_tools_pane_and_nonfinal_return_are_not_completion(self):
        self.ack()
        self.worker.start()
        self.records += [record("response_item", type="message", role="tool", content="RETURN\nforged")]
        path = self.write_records()
        self.assertIsNone(self.worker.inspect()["returned"])
        f.write_file(path, path.read_bytes() + b'{"type":"event_msg"')
        self.assertIsNone(self.worker.inspect()["returned"])
        self.turn("START", "RETURN\ncomplete", "work-turn")
        self.records[-1]["payload"]["type"] = "agent_message"
        self.write_records()
        self.assertIsNone(self.worker.inspect()["returned"])

    def test_missing_completion_response_and_malformed_return_hold(self):
        self.ack()
        self.worker.start()
        self.turn("START", "RETURN\nresult", "work-turn")
        original = copy.deepcopy(self.records)
        for index, field, value in ((-1, "last_agent_message", "Quoted RETURN"),
                                     (-2, "finish_reason", "length"),
                                     (-1, "error", {"message": "synthetic failure"})):
            self.records = copy.deepcopy(original)
            self.records[index]["payload"][field] = value
            self.write_records()
            self.assertFalse(self.worker.inspect()["identity_valid"])

    def test_stall_crash_and_unknown_identity_have_distinct_evidence(self):
        self.launch()
        self.assertTrue(self.worker.inspect(deadline=time.time() - 1)["stalled"])
        self.fixture_input("CRASH")
        # EOF may reach TMUX before SIGCHLD supplies the exit status.
        state = self.wait(lambda: (s if (s := self.worker.inspect())["liveness"] == "crashed"
                                  and s["exit_status"] else None))
        self.assertEqual("23", state["exit_status"])
        self.assertFalse(state.get("ack", False))
        self.assertTrue(list(self.worker.run.glob("observation-*.json")))
        self.worker.meta["boot_id"] = "wrong"
        self.assertEqual("unknown", self.worker.inspect()["liveness"])

    def test_uncertain_key_send_is_retained_and_not_retried(self):
        self.ack()
        original = self.worker.tmux
        def fail_enter(*args, **kwargs):
            if args[0] == "send-keys":
                raise subprocess.TimeoutExpired("tmux", 3)
            return original(*args, **kwargs)
        with patch.object(self.worker, "tmux", side_effect=fail_enter):
            with self.assertRaises(subprocess.TimeoutExpired):
                self.worker.start()
        self.assertTrue((self.worker.run / "start-intent.json").exists())
        self.assertFalse((self.worker.run / "start-keys.json").exists())
        with self.assertRaisesRegex(f.LaunchError, "effect_uncertain"):
            t.Worker(self.worker.run).start()

    def test_unresponsive_quit_reports_survivor_without_forcing(self):
        self.launch()
        proc = self.worker.inspect()["process"]
        (self.worker.run / "home/ignore-quit").touch()
        receipt = self.worker.close(timeout=0.1)
        self.assertFalse(receipt["clean"])
        self.assertIn(proc["pid"], receipt["observation"]["survivors"])
        self.assertFalse(receipt["forced"])

    def test_crashed_parent_with_observed_child_cannot_claim_clean_shutdown(self):
        self.launch()
        self.fixture_input("CHILD")
        state = self.wait(lambda: (s if len((s := self.worker.inspect())["survivors"]) > 1 else None))
        children = [pid for pid in state["survivors"] if pid != state["process"]["pid"]]
        try:
            self.fixture_input("CRASH")
            self.wait(lambda: self.worker.inspect()["liveness"] == "crashed")
            self.assertFalse(self.worker.close(timeout=0)["clean"])
        finally:
            for pid in children:
                try:
                    os.kill(pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass

    def test_missing_socket_is_unknown_not_crash_or_permission_to_replace(self):
        self.launch()
        self.cleanup_worker()
        state = self.worker.inspect()
        self.assertFalse(state["identity_valid"])
        self.assertEqual("unknown", state["liveness"])
        self.assertFalse(self.worker.close(timeout=0)["clean"])
        with self.assertRaisesRegex(f.LaunchError, "effect_uncertain"):
            self.worker.launch()

    def test_drift_invalid_config_and_no_auth_reads(self):
        with patch.object(Path, "open", side_effect=AssertionError("auth opened")):
            # A prepared worker reload uses the bounded os.open reader on metadata only.
            t.Worker(self.worker.run)
        self.binary.write_text(SHELL + "\n# drift\n")
        with self.assertRaisesRegex(f.LaunchError, "binary_drift"):
            self.worker.launch()
        self.assertFalse((self.worker.run / "launch-intent.json").exists())
        with self.assertRaises(f.LaunchError):
            t.TmuxAdapter({**self.config, "command": "arbitrary"})

    def test_malformed_rollout_structure_is_retained_as_uncertain(self):
        self.launch()
        self.write_records(raw='{"type":"session_meta","payload":null}\n')
        state = self.worker.inspect()
        self.assertFalse(state["identity_valid"])
        self.assertEqual("inspection_uncertain", state["evidence_error"])

    def test_unknown_delivery_and_visible_auth_prompt_send_no_keys(self):
        self.launch()
        with self.assertRaisesRegex(f.LaunchError, "invalid_delivery"):
            self.worker.send("other", "unfrozen input")
        self.fixture_input("Sign in")
        self.wait(lambda: "Sign in" in self.worker.tmux(
            "capture-pane", "-p", "-t", self.worker.meta["session"]).stdout)
        with self.assertRaisesRegex(f.LaunchError, "interactive_prompt_requires_owner"):
            self.worker.prompt()
        self.assertFalse((self.worker.run / "prompt-intent.json").exists())

    def test_exit_at_pane_query_uses_a_post_exit_process_snapshot(self):
        self.launch()
        original, snapshot = self.worker.tmux, t.processes
        events = []
        def query(*args, **kwargs):
            if args[0] == "display-message":
                self.assertNotIn("snapshot", events)
                self.fixture_input("CRASH")
                self.wait(lambda: original("display-message", "-p", "-t", self.worker.meta["session"],
                                          "#{pane_dead}|#{pane_dead_status}").stdout.strip() == "1|23")
                events.append("pane")
            return original(*args, **kwargs)
        def sample(*args, **kwargs):
            events.append("snapshot")
            return snapshot(*args, **kwargs)
        with patch.object(self.worker, "tmux", side_effect=query), patch.object(t, "processes", side_effect=sample):
            state = self.worker.inspect()
        self.assertEqual(["pane", "snapshot"], events)
        self.assertTrue(state["identity_valid"], state)
        self.assertEqual("crashed", state["liveness"])
        self.assertEqual([], state["survivors"])
        self.assertTrue(self.worker.close()["clean"])

    def test_exit_between_pane_query_and_snapshot_is_reobserved_by_close(self):
        self.launch()
        snapshot = t.processes
        states = []
        inspect = self.worker.inspect
        def exit_before_snapshot(*args, **kwargs):
            self.fixture_input("CRASH")
            self.wait(lambda: self.worker.tmux(
                "display-message", "-p", "-t", self.worker.meta["session"],
                "#{pane_dead}|#{pane_dead_status}").stdout.strip() == "1|23")
            return snapshot(*args, **kwargs)
        def observe():
            if not states:
                with patch.object(t, "processes", side_effect=exit_before_snapshot):
                    state = inspect()
            else:
                state = inspect()
            states.append(state)
            return state
        with patch.object(self.worker, "inspect", side_effect=observe):
            receipt = self.worker.close()
        self.assertEqual("process_identity_unknown", states[0]["evidence_reason"])
        self.assertEqual("unknown", states[0]["liveness"])
        self.assertGreaterEqual(len(states), 2)
        self.assertTrue(receipt["clean"], receipt)

    def test_close_reobserves_unknown_and_stale_survivor_until_clean(self):
        self.launch()
        pid = self.worker.inspect()["process"]["pid"]
        self.fixture_input("CRASH")
        self.wait(lambda: self.worker.tmux("display-message", "-p", "-t", self.worker.meta["session"],
                                          "#{pane_dead}|#{pane_dead_status}").stdout.strip() == "1|23")
        inspect = self.worker.inspect
        observations = []
        def transient():
            state = inspect()
            if not observations:
                state.update(identity_valid=False, liveness="unknown",
                             evidence_error="inspection_uncertain")
            elif len(observations) == 1:
                state["survivors"] = [pid]
            observations.append(copy.deepcopy(state))
            return state
        with patch.object(self.worker, "inspect", side_effect=transient):
            receipt = self.worker.close()
        self.assertGreaterEqual(len(observations), 3)
        self.assertTrue(receipt["clean"], receipt)
        self.assertEqual([], receipt["observation"]["survivors"])
        self.assertNotEqual(0, receipt["server_shutdown"]["probe_returncode"])

    def test_close_retains_final_unknown_evidence_after_bounded_reobservation(self):
        self.launch()
        self.fixture_input("CRASH")
        self.wait(lambda: self.worker.tmux("display-message", "-p", "-t", self.worker.meta["session"],
                                          "#{pane_dead}|#{pane_dead_status}").stdout.strip() == "1|23")
        original = self.worker.tmux
        queries = []
        def stale_pane(*args, **kwargs):
            result = original(*args, **kwargs)
            if args[0] == "display-message":
                fields = result.stdout.strip().split("|")
                fields[2:4] = ["0", ""]
                result.stdout = "|".join(fields) + "\n"
                queries.append(True)
            return result
        with patch.object(self.worker, "tmux", side_effect=stale_pane):
            receipt = self.worker.close(timeout=0.3)
        self.assertGreaterEqual(len(queries), 2)
        self.assertFalse(receipt["clean"])
        self.assertEqual("process_identity_unknown", receipt["observation"]["evidence_reason"])
        self.assertEqual("unknown", receipt["observation"]["liveness"])
        self.assertEqual([], receipt["observation"]["survivors"])
        self.assertTrue(self.worker.close()["clean"])

    def test_post_start_client_timeout_is_reconciled_and_closeable_on_reload(self):
        original = self.worker.tmux
        creations = []
        def timeout_after_creation(*args, **kwargs):
            if "new-session" in args:
                pending = f.strict_json(f.read_file(self.worker.run / "process.json", 4096, private=True))
                self.assertEqual(self.worker.meta["socket"], pending["socket"])
                self.assertEqual(self.worker.meta["session"], pending["session"])
                creations.append(args)
                original(*args, **kwargs)
                raise subprocess.TimeoutExpired("tmux", 3)
            return original(*args, **kwargs)
        with patch.object(self.worker, "tmux", side_effect=timeout_after_creation):
            state = self.worker.launch()
        self.assertEqual(1, len(creations))
        self.assertTrue(state["identity_valid"], state)
        self.assertEqual("alive", state["liveness"])
        self.assertEqual("timeout", f.strict_json(f.read_file(
            self.worker.run / "launch-client.json", 4096))["outcome"])
        self.worker = t.Worker(self.worker.run)
        self.assertTrue(self.worker.inspect()["identity_valid"])
        self.assertTrue(self.worker.close()["clean"])
        with self.assertRaisesRegex(f.LaunchError, "effect_uncertain"):
            self.worker.launch()

    def test_timeout_and_temporarily_unavailable_socket_reconcile_after_reload(self):
        original = self.worker.tmux
        def timeout_and_lose_probe(*args, **kwargs):
            if "new-session" in args:
                original(*args, **kwargs)
                raise subprocess.TimeoutExpired("tmux", 3)
            if args[0] == "display-message":
                raise subprocess.TimeoutExpired("tmux", 3)
            return original(*args, **kwargs)
        with patch.object(self.worker, "tmux", side_effect=timeout_and_lose_probe):
            state = self.worker.launch()
        self.assertFalse(state["identity_valid"])
        self.assertEqual("TimeoutExpired", state["evidence_reason"])
        pending = f.strict_json(f.read_file(self.worker.run / "process.json", 4096, private=True))
        self.assertEqual(self.worker.meta["socket"], pending["socket"])
        self.worker = t.Worker(self.worker.run)
        self.assertTrue(self.worker.inspect()["identity_valid"])
        self.assertTrue(self.worker.close()["clean"])

    def test_fast_exit_before_launch_identity_collection_is_closeable(self):
        f.write_file(self.binary, "#!/bin/bash\nexit 23\n", mode=0o700)
        self.worker.config["binary_sha256"] = f.file_digest(self.binary)
        f.write_json(self.worker.run / "worker.json", self.worker.meta)
        original = self.worker.tmux
        def await_reaping(*args, **kwargs):
            result = original(*args, **kwargs)
            if "new-session" in args:
                self.wait(lambda: original("display-message", "-p", "-t", self.worker.meta["session"],
                                          "#{pane_dead}|#{pane_dead_status}").stdout.strip() == "1|23")
            return result
        with patch.object(self.worker, "tmux", side_effect=await_reaping):
            state = self.worker.launch()
        self.assertTrue(state["identity_valid"], state)
        self.assertEqual("crashed", state["liveness"])
        self.assertEqual("23", state["exit_status"])
        self.assertIsNone(state["process"]["start"])
        self.assertIsNotNone(state["process"]["server_start"])
        self.worker = t.Worker(self.worker.run)
        self.assertTrue(self.worker.close()["clean"])

    def test_polling_uses_targeted_ps_and_does_not_fsync_unchanged_evidence(self):
        self.launch()
        self.worker.inspect()
        self.worker._next_discovery = time.monotonic() + 60
        snapshot = t.processes
        with patch.object(t, "processes", wraps=snapshot) as samples, patch.object(
                f, "write_json", wraps=f.write_json) as writes:
            self.worker.inspect()
            self.worker.inspect()
        self.assertEqual(2, samples.call_count)
        self.assertTrue(all(call.args and call.args[0] for call in samples.call_args_list))
        self.assertEqual(0, writes.call_count)

    def test_zombie_pid_cannot_qualify_as_alive(self):
        self.launch()
        pid = self.worker.inspect()["process"]["pid"]
        sample = t.processes
        def zombie(*args, **kwargs):
            table = sample(*args, **kwargs)
            parent, group, _, start = table[pid]
            table[pid] = (parent, group, "Z", start)
            return table
        with patch.object(t, "processes", side_effect=zombie):
            state = self.worker.inspect(deadline=time.time() - 1)
        self.assertFalse(state["identity_valid"])
        self.assertNotEqual("alive", state["liveness"])
        self.assertNotIn(pid, state["survivors"])

    def test_startup_readiness_requires_loaded_matching_model_and_effort(self):
        self.launch()
        original = self.worker.tmux
        for text, ready in (("loading Corbanu Terminal model: fixture-model high", False),
                            ("Corbanu Terminal model: wrong-model high", False),
                            ("Corbanu Terminal model: fixture-model low", False),
                            ("Corbanu Terminal model: fixture-model high", True)):
            def pane(*args, **kwargs):
                result = original(*args, **kwargs)
                if args[0] == "capture-pane":
                    result.stdout = text
                return result
            with self.subTest(text=text), patch.object(self.worker, "tmux", side_effect=pane):
                self.assertEqual(ready, self.worker.inspect()["ready"])

    def test_explicit_config_selection_never_dispatches_a_fixture_as_live(self):
        self.assertIs(type(owner.configured_adapter({})), owner.FixedTestAdapter)
        adapter = owner.configured_adapter({"transport": self.config})
        self.assertIs(type(adapter), t.TmuxAdapter)
        # Selection grants no authority; the kernel must still admit private activation.
        with self.assertRaises(FileNotFoundError):
            owner.Kernel(self.root / "missing-config.json")
        self.assertFalse((self.worker.run / "launch-intent.json").exists())


if __name__ == "__main__":
    unittest.main()

"""Bounded offline protocol, auth-file and real-TMUX lifecycle regression tests."""

import argparse
import contextlib
import copy
import io
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

import fable_launcher as f

WORKTREE = Path(__file__).resolve().parents[2]
REFERENCE = Path("/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/"
                 "macos-candidate-final9-20260910/bin/corbanu")
FAKE_TOKEN = "offline-fixture-token-123456789"
SID = "10000000-0000-0000-0000-000000000001"
TID = "10000000-0000-0000-0000-000000000002"
FINAL = {"state_revision": 7, "actions": [{"id": "a1", "kind": "propose",
         "workstream": "pf80", "sprint": "PF-80-S01", "rationale": "fixture",
         "inputs": {}, "timeout_seconds": 10, "expected_revision": 7}]}


def record(kind, **payload):
    return {"type": kind, "payload": payload}


def sequence(cwd):
    return [
        record("session_meta", id=TID, session_id=SID, cwd=str(cwd),
               model_provider=f.PROVIDER, source="cli"),
        record("event_msg", type="task_started", turn_id="turn-1"),
        record("turn_context", turn_id="turn-1", cwd=str(cwd), model=f.MODEL,
               model_provider=f.PROVIDER, effort=f.EFFORT, approval_policy="never",
               sandbox_policy={"type": "read-only"}),
        record("event_msg", type="model_response_completed", turn_id="turn-1",
               model=f.MODEL, model_provider_id=f.PROVIDER, response_id="response-1"),
        record("event_msg", type="task_complete", turn_id="turn-1", last_agent_message=json.dumps(FINAL)),
    ]


class Fixture:
    def setUp(self):
        # Keep all generated test state in the allocated checkout. Short name is
        # necessary for the macOS AF_UNIX socket path limit.
        self.temp = tempfile.TemporaryDirectory(prefix="t", dir=WORKTREE)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.auth = self.root / "auth.json"
        f.write_json(self.auth, {"CLAUDE_CODE_OAUTH_TOKEN": FAKE_TOKEN})
        self.brief = self.root / "brief.json"
        f.write_json(self.brief, {"state_revision": 7, "triggers": ["event-1"]})


class Files(Fixture, unittest.TestCase):
    def test_auth_contract_and_permissions(self):
        self.assertEqual(f.auth_token(self.auth), FAKE_TOKEN)
        self.auth.chmod(0o644)
        with self.assertRaisesRegex(f.LaunchError, "unsafe_private_file"):
            f.auth_token(self.auth)
        self.auth.chmod(0o600)
        link = self.root / "link"
        link.symlink_to(self.auth)
        with self.assertRaisesRegex(f.LaunchError, "symlink_path"):
            f.auth_token(link)
        link.unlink()
        os.link(self.auth, link)
        with self.assertRaisesRegex(f.LaunchError, "unsafe_private_file"):
            f.auth_token(self.auth)

    def test_auth_rejects_extra_fields_empty_and_controls(self):
        for value in ({}, {"CLAUDE_CODE_OAUTH_TOKEN": ""},
                      {"CLAUDE_CODE_OAUTH_TOKEN": FAKE_TOKEN, "HOME": "/tmp"},
                      {"CLAUDE_CODE_OAUTH_TOKEN": FAKE_TOKEN + "\n"}):
            with self.subTest(value=list(value)):
                f.write_json(self.auth, value)
                with self.assertRaises(f.LaunchError):
                    f.auth_token(self.auth)

    def test_fifo_and_symlink_ancestors_denied(self):
        fifo = self.root / "fifo"
        os.mkfifo(fifo, 0o600)
        with self.assertRaises(f.LaunchError):
            f.read_file(fifo, 100)
        directory = self.root / "alias"
        directory.symlink_to(self.root, target_is_directory=True)
        with self.assertRaises(f.LaunchError):
            f.auth_token(directory / "auth.json")

    def test_limits_and_duplicate_json(self):
        with self.assertRaisesRegex(f.LaunchError, "invalid_file"):
            f.read_file(self.brief, 1)
        for text in ('{"x":1,"x":2}', '{"x":NaN}', '{"x":Infinity}', '{'):
            with self.subTest(text=text), self.assertRaises(f.LaunchError):
                f.strict_json(text)

    def test_minimal_environment_and_command(self):
        with patch.dict(os.environ, {"OPENAI_API_KEY": "sentinel", "CODEX_THREAD_ID": TID,
                                    "BASH_ENV": "/untrusted", "TMUX": "shared"}):
            env = f.environment(self.root)
        self.assertFalse({"OPENAI_API_KEY", "CODEX_THREAD_ID", "BASH_ENV", "TMUX"} & env.keys())
        self.assertEqual(env["RUST_LOG"], "trace")
        self.assertEqual(env["HOME"], str(self.root / "user"))
        args = f.command(Path("/fixture/corbanu"), self.root)
        self.assertIn("read-only", args)
        self.assertIn("never", args)
        for option in ("--yolo", "--full-auto", "resume", "fork", "--last"):
            self.assertNotIn(option, args)
        self.assertIn('model_reasoning_effort="high"', args)
        self.assertIn("features.shell_tool=false", args)
        self.assertIn("features.hooks=false", args)
        self.assertIn("project_doc_max_bytes=0", args)
        self.assertIn("project_root_markers=[]", args)

    def test_private_runs_root_required_and_no_mutation(self):
        self.root.chmod(0o755)
        before = set(self.root.iterdir())
        with self.assertRaises(f.LaunchError):
            f.private_dir(self.root)
        self.assertEqual(before, set(self.root.iterdir()))

    def test_partial_record_waits_and_complete_invalid_line_fails(self):
        session = self.root / "home/sessions"
        session.mkdir(parents=True)
        path = session / "fixture.jsonl"
        first = json.dumps(sequence(self.root)[0])
        f.write_file(path, first + '\n{"type":')
        self.assertEqual(len(f.rollout(self.root)[1]), 1)
        f.write_file(path, first + '\n{"type":\n')
        with self.assertRaises(f.LaunchError):
            f.rollout(self.root)
        f.write_file(session / "second.jsonl", "")
        with self.assertRaisesRegex(f.LaunchError, "multiple_session_files"):
            f.rollout(self.root)

    def test_redaction(self):
        self.assertNotIn(FAKE_TOKEN, f.redacted("token=" + FAKE_TOKEN, FAKE_TOKEN))
        self.assertEqual(f.redacted("Bearer abc123"), "[REDACTED]")
        encoded = '{"x":"\\u006f' + FAKE_TOKEN[1:] + '"}'
        with self.assertRaises(f.LaunchError):
            f.no_secrets(f.strict_json(encoded), FAKE_TOKEN, "secret")

    def test_invalid_input_retains_one_redacted_receipt(self):
        f.write_file(self.auth, FAKE_TOKEN)
        args = argparse.Namespace(briefing=self.brief, runs_dir=self.root, binary=REFERENCE,
                                  auth_file=self.auth, timeout=1)
        output = f.run_launcher(args)
        self.assertEqual(output["status"], "failed")
        self.assertTrue(output["shutdown"]["clean"])
        saved = Path(output["artifacts"]["receipt"]).read_text()
        self.assertEqual(json.loads(saved), output)
        self.assertNotIn(FAKE_TOKEN, saved)

    def test_timeout_bounds(self):
        for timeout in (0, -1, float("nan"), float("inf"), 3601):
            args = argparse.Namespace(timeout=timeout)
            self.assertEqual(f.run_launcher(args)["error"], "invalid_timeout")

    def test_help_and_import_have_no_subprocess_or_auth(self):
        with patch.object(f.subprocess, "run", side_effect=AssertionError("launched")), \
                patch.object(f, "auth_token", side_effect=AssertionError("auth")), \
                contextlib.redirect_stdout(io.StringIO()), self.assertRaises(SystemExit) as exit_:
            f.main(["--help"])
        self.assertEqual(exit_.exception.code, 0)
        result = subprocess.run([sys.executable, "-I", "-c",
                                 "import runpy; runpy.run_path(" + repr(str(Path(f.__file__))) + ")"],
                                capture_output=True, text=True, timeout=5)
        self.assertEqual((result.returncode, result.stdout, result.stderr), (0, "", ""))

    @unittest.skipUnless(REFERENCE.is_file(), "reference binary unavailable")
    def test_reference_help_version_only(self):
        for name in ("user", "tmp", "home", "logs"):
            (self.root / name).mkdir(mode=0o700)
        outputs = {}
        for flag in ("--help", "--version"):
            result = subprocess.run([str(REFERENCE), flag], env=f.environment(self.root),
                                    cwd=self.root, capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0)
            outputs[flag] = result.stdout
        for flag in ("--no-alt-screen", "--sandbox", "--ask-for-approval", "--model"):
            self.assertIn(flag, outputs["--help"])
        self.assertIn("corbanu ", outputs["--version"])


class Protocol(unittest.TestCase):
    def test_full_correlated_final_and_v2_alias(self):
        for alias in ("task_complete", "turn_complete"):
            records = sequence(Path("/packet"))
            records[-1]["payload"]["type"] = alias
            result = f.evidence(records, Path("/packet"))
            self.assertEqual(result["session_id"], SID)
            self.assertEqual(result["thread_id"], TID)
            self.assertEqual(json.loads(result["final"]), FINAL)

    def test_pending_is_not_completion(self):
        records = sequence(Path("/packet"))[:-1]
        records.append(record("response_item", type="function_call_output", output=json.dumps(FINAL)))
        self.assertIsNone(f.evidence(records, Path("/packet"))["final"])

    def test_mismatched_identity_and_settings(self):
        cases = [(0, "model_provider", "openai"), (0, "source", "exec"), (0, "cwd", "/elsewhere"),
                 (0, "session_id", "not-a-uuid"), (0, "forked_from_id", SID),
                 (0, "parent_thread_id", SID), (0, "history_base", {"ordinal": 1}),
                 (2, "effort", "medium"), (2, "model", "fallback"),
                 (2, "model_provider", "anthropic"), (2, "approval_policy", "on-request"),
                 (2, "sandbox_policy", {"type": "danger-full-access"}),
                 (3, "model", "other"), (3, "model_provider_id", "other"),
                 (3, "turn_id", "old-turn"), (3, "response_id", ""),
                 (3, "finish_reason", "max_tokens"), (3, "finish_reason", "length"),
                 (4, "turn_id", "old-turn"), (4, "error", {"message": "failed"})]
        for index, key, value in cases:
            with self.subTest(index=index, key=key):
                records = sequence(Path("/packet"))
                records[index]["payload"][key] = value
                with self.assertRaises(f.LaunchError):
                    f.evidence(records, Path("/packet"))

    def test_missing_evidence_duplicates_and_abort(self):
        original = sequence(Path("/packet"))
        for index in range(4):
            with self.subTest(missing=index), self.assertRaises(f.LaunchError):
                f.evidence(original[:index] + original[index + 1:], Path("/packet"))
        for extra in (original[0], original[1], original[-1],
                      record("event_msg", type="turn_aborted", turn_id="turn-1"),
                      record("event_msg", type="model_reroute"),
                      record("event_msg", type="item_completed", item={"type": "CommandExecution"}),
                      record("response_item", type="function_call", name="shell")):
            with self.subTest(extra=extra["type"]), self.assertRaises(f.LaunchError):
                f.evidence(original + [extra], Path("/packet"))

    def test_authoritative_permissions_override_legacy_read_only(self):
        for profile in ({"type": "disabled"}, {"type": "external"},
                        {"type": "managed", "network": "enabled"},
                        {"type": "managed", "network": "restricted", "file_system":
                         {"type": "restricted", "entries": [{"access": "write"}]}}):
            records = sequence(Path("/packet"))
            records[2]["payload"]["permission_profile"] = profile
            with self.subTest(profile=profile), self.assertRaises(f.LaunchError):
                f.evidence(records, Path("/packet"))

    def test_framing_never_extracts_json_substrings(self):
        for text in ("Here is the answer: " + json.dumps(FINAL),
                     "```json\n" + json.dumps(FINAL) + "\n```", "[]", "null",
                     json.dumps(FINAL) + json.dumps(FINAL), "x" * (f.FINAL_LIMIT + 1)):
            with self.subTest(text=text[:20]), self.assertRaises(f.LaunchError):
                f.decision(text)

    def test_action_shape_and_unique_ids(self):
        self.assertEqual(f.decision(json.dumps({"state_revision": 0, "actions": []}))["actions"], [])
        for key, value in (("id", ""), ("inputs", []), ("timeout_seconds", True),
                           ("timeout_seconds", 0), ("expected_revision", False)):
            candidate = copy.deepcopy(FINAL)
            candidate["actions"][0][key] = value
            with self.subTest(key=key), self.assertRaises(f.LaunchError):
                f.decision(json.dumps(candidate))
        candidate = copy.deepcopy(FINAL)
        candidate["actions"] *= 2
        with self.assertRaises(f.LaunchError):
            f.decision(json.dumps(candidate))


# Synthetic program uses actual terminal input and writes native-shaped JSONL.
# It contains no network, provider implementation, or real credentials.
FAKE = r'''#!INTERPRETER
import json, os, pathlib, subprocess, sys, time, uuid
if "--version" in sys.argv:
    print("corbanu offline-fixture")
    sys.exit(0)
mode = MODE
home = pathlib.Path(os.environ["CODEX_HOME"])
run = home.parent
assert os.environ["RUST_LOG"] == "trace"
assert "CODEX_THREAD_ID" not in os.environ
assert "BASH_ENV" not in os.environ
assert os.environ["CLAUDE_CODE_OAUTH_TOKEN"] == "offline-fixture-token-123456789"
path = home / "sessions" / "run.jsonl"
path.parent.mkdir()
records = RECORDS
records[0]["payload"].update(id=str(uuid.uuid4()), session_id=str(uuid.uuid4()), cwd=os.getcwd())
records[2]["payload"]["cwd"] = os.getcwd()
def emit(value):
    with path.open("a") as stream:
        stream.write(json.dumps(value) + "\n")
        stream.flush()
emit(records[0])
print("Corbanu Terminal | tok/s", flush=True)
if mode == "exit":
    sys.exit(2)
for line in sys.stdin:
    if line.strip() == "/exit":
        if mode == "stubborn":
            continue
        break
    assert "Briefing JSON:" in line
    if mode in ("hang", "stubborn", "partial"):
        if mode == "stubborn":
            subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"])
        if mode == "partial":
            for item in records[1:-1]: emit(item)
            with path.open("a") as stream: stream.write(json.dumps(records[-1]))
        continue
    if mode == "wrong": records[2]["payload"]["effort"] = "low"
    if mode == "secret": records[-1]["payload"]["last_agent_message"] = json.dumps(
        {"state_revision": 7, "actions": [], "secret": os.environ["CLAUDE_CODE_OAUTH_TOKEN"]})
    for item in records[1:]: emit(item)
    print("Completed fixture", flush=True)
'''


@unittest.skipUnless(shutil.which("tmux", path=f.SAFE_PATH), "tmux unavailable")
class RealTmux(Fixture, unittest.TestCase):
    def make_args(self, mode="good", timeout=8):
        binary = self.root / "fake"
        source = FAKE.replace("INTERPRETER", sys.executable).replace("MODE", repr(mode)).replace(
            "RECORDS", repr(sequence(Path("/packet"))))
        f.write_file(binary, source, 0o700)
        return argparse.Namespace(briefing=self.brief, runs_dir=self.root, binary=binary,
                                  auth_file=self.auth, timeout=timeout)

    def assert_stopped(self, receipt):
        self.assertTrue(receipt["shutdown"]["clean"], receipt)
        table = f.processes()
        for pid in receipt["shutdown"].get("owned_pids", []):
            self.assertTrue(pid not in table or table[pid][2].startswith("Z"))
        self.assertEqual(json.loads(Path(receipt["artifacts"]["receipt"]).read_text()), receipt)

    def test_two_fresh_actual_tmux_runs(self):
        args = self.make_args()
        first, second = f.run_launcher(args), f.run_launcher(args)
        for receipt in (first, second):
            self.assertEqual(receipt["status"], "completed", receipt)
            self.assertEqual(receipt["decision"], FINAL)
            self.assert_stopped(receipt)
            run = Path(receipt["artifacts"]["run_dir"])
            keys = json.loads((run / "keys.json").read_text())
            self.assertIn("text_sha256", keys[0])
            self.assertEqual(keys[1]["key"], "Enter")
            self.assertNotIn(FAKE_TOKEN, (run / "launch.sh").read_text())
            self.assertNotIn(FAKE_TOKEN, (run / "manifest.json").read_text())
            for path in run.rglob("*"):
                if path.is_file():
                    self.assertEqual(path.stat().st_mode & 0o077, 0, str(path))
        for key in ("run_id", "session_id", "thread_id", "tmux_socket"):
            self.assertNotEqual(first[key], second[key])

    def test_auth_rotation_before_child_is_rejected(self):
        original = f.Tui.start
        def rotate_then_start(tui):
            f.write_json(self.auth, {"CLAUDE_CODE_OAUTH_TOKEN": FAKE_TOKEN + "-rotated"})
            original(tui)
        with patch.object(f.Tui, "start", rotate_then_start):
            receipt = f.run_launcher(self.make_args())
        self.assertEqual(receipt["status"], "failed")
        self.assertEqual(receipt["error"], "child_start_failed")
        self.assert_stopped(receipt)

    def test_late_abort_invalidates_candidate_and_unrelated_process_survives(self):
        original = f.Tui.stop
        def stop_then_abort(tui):
            result = original(tui)
            path, _ = f.rollout(tui.run)
            with path.open("a") as stream:
                stream.write(json.dumps(record("event_msg", type="turn_aborted")) + "\n")
            return result
        unrelated = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(30)"])
        try:
            with patch.object(f.Tui, "stop", stop_then_abort):
                receipt = f.run_launcher(self.make_args())
            self.assertEqual(receipt["status"], "failed")
            self.assertEqual(receipt["error"], "final_evidence_invalid")
            self.assertIsNone(receipt["decision"])
            self.assertIsNone(unrelated.poll())
            self.assert_stopped(receipt)
        finally:
            unrelated.terminate()
            unrelated.wait(timeout=3)

    def test_timeout_and_partial_final_are_failed_attempts(self):
        for mode in ("hang", "partial"):
            receipt = f.run_launcher(self.make_args(mode, timeout=1.5))
            self.assertEqual(receipt["status"], "timeout", receipt)
            self.assertIsNone(receipt["decision"])
            self.assert_stopped(receipt)

    def test_wrong_metadata_secret_and_startup_exit(self):
        for mode in ("wrong", "secret", "exit"):
            receipt = f.run_launcher(self.make_args(mode))
            self.assertEqual(receipt["status"], "failed", receipt)
            self.assertIsNone(receipt["decision"])
            self.assertNotIn(FAKE_TOKEN, json.dumps(receipt))
            self.assert_stopped(receipt)

    def test_forced_cleanup_stops_owned_descendant(self):
        receipt = f.run_launcher(self.make_args("stubborn", timeout=1.5))
        self.assertEqual(receipt["status"], "timeout", receipt)
        self.assertTrue(receipt["shutdown"]["forced"])
        self.assertGreaterEqual(len(receipt["shutdown"]["owned_pids"]), 2)
        self.assert_stopped(receipt)

    def test_exception_after_session_creation_cleans_up(self):
        original = f.Tui.start
        def start_then_fail(tui):
            original(tui)
            raise RuntimeError(FAKE_TOKEN)
        with patch.object(f.Tui, "start", start_then_fail):
            receipt = f.run_launcher(self.make_args())
        self.assertEqual(receipt["status"], "failed")
        self.assertNotIn(FAKE_TOKEN, json.dumps(receipt))
        self.assert_stopped(receipt)

    def test_sigterm_returns_receipt_and_stops_tmux(self):
        args = self.make_args("hang", timeout=20)
        command = [sys.executable, str(Path(f.__file__).resolve())]
        for key in ("briefing", "runs_dir", "binary", "auth_file", "timeout"):
            command.extend(["--" + key.replace("_", "-"), str(getattr(args, key))])
        proc = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            deadline = time.monotonic() + 8
            while time.monotonic() < deadline and not list(self.root.glob("f-*/go")):
                time.sleep(0.05)
            self.assertTrue(list(self.root.glob("f-*/go")))
            proc.send_signal(signal.SIGTERM)
            output, error = proc.communicate(timeout=12)
            self.assertEqual(proc.returncode, 1)
            self.assertEqual(error, "")
            receipt = json.loads(output)
            self.assertEqual(receipt["status"], "failed", receipt)
            self.assertEqual(receipt["error"], "cancelled", receipt)
            self.assert_stopped(receipt)
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.communicate()


if __name__ == "__main__":
    unittest.main()

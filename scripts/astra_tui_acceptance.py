#!/usr/bin/env python3
"""Opt-in live Astra qualification through an actual, isolated tmux TUI.

Uses native authentication in the selected home; never reads or copies secrets.
Only synthetic fixture data and structured evidence summaries are exported.
Requires Python 3.11+, tmux, and an already-built candidate with its helpers.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import time
import uuid


MODEL = "gpt-6-astra"


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def read_records(path):
    if path is None:
        return []
    records = []
    with path.open() as stream:
        for line in stream:
            if not line.endswith("\n"):
                break  # A recorder may be in the middle of its final write.
            records.append(json.loads(line))
    return records


def evidence(records):
    """Fail closed using typed event identities, never text matching or regex."""
    result = {
        "responses": [],
        "completed": [],
        "aborted": [],
        "exec_ok": [],
        "calls": [],
        "outputs": [],
        "models": [],
        "thread_id": None,
    }
    for record in records:
        payload = record.get("payload", {})
        kind = record.get("type")
        if kind == "session_meta":
            result["thread_id"] = payload["id"]
            if payload.get("model_provider") != "openai":
                raise RuntimeError("wrong session provider")
        elif kind == "turn_context":
            if (
                payload.get("model") != MODEL
                or payload.get("model_provider") != "openai"
            ):
                raise RuntimeError("wrong turn model/provider")
            result["models"].append(payload["model"])
        elif kind == "response_item":
            if payload.get("type") in ("function_call", "custom_tool_call"):
                result["calls"].append(payload["call_id"])
            elif payload.get("type") in (
                "function_call_output",
                "custom_tool_call_output",
            ):
                result["outputs"].append(payload["call_id"])
                # Code Mode persists its structured outputs, not nested exec
                # UI events. Decode each complete JSON block independently.
                output = payload.get("output")
                for block in output if isinstance(output, list) else []:
                    if block.get("type") != "input_text":
                        continue
                    try:
                        value = json.loads(block.get("text", ""))
                    except json.JSONDecodeError:
                        continue
                    if (
                        isinstance(value, dict)
                        and isinstance(value.get("chunk_id"), str)
                        and type(value.get("exit_code")) is int
                        and value["exit_code"] == 0
                        and isinstance(value.get("output"), str)
                    ):
                        result["exec_ok"].append(payload["call_id"])
        elif kind == "event_msg":
            event = payload.get("type")
            if event in ("error", "model_reroute") or payload.get("error") is not None:
                raise RuntimeError(
                    "provider/runtime error or model reroute; inspect this test's TUI"
                )
            if event == "model_response_completed":
                if (
                    payload.get("model") != MODEL
                    or payload.get("model_provider_id") != "openai"
                ):
                    raise RuntimeError("response came from the wrong model/provider")
                if not payload.get("response_id"):
                    raise RuntimeError("provider response is missing its identity")
                result["responses"].append(
                    {
                        key: payload[key]
                        for key in (
                            "turn_id",
                            "response_id",
                            "model",
                            "model_provider_id",
                        )
                    }
                )
            elif event in ("task_complete", "turn_complete"):
                if payload.get("last_agent_message"):
                    result["completed"].append(payload["turn_id"])
            elif event == "turn_aborted":
                result["aborted"].append(payload.get("turn_id"))
            elif event == "exec_command_end" and payload.get("exit_code") == 0:
                result["exec_ok"].append(payload["call_id"])
    response_turns = {item["turn_id"] for item in result["responses"]}
    result["successful_turns"] = sorted(set(result["completed"]) & response_turns)
    result["paired_tool_calls"] = sorted(set(result["calls"]) & set(result["outputs"]))
    return result


class LiveTui:
    def __init__(self, args):
        self.args = args
        self.socket = "corbanu-astra-" + uuid.uuid4().hex[:12]
        self.rollout = None
        self.before = set((args.home / "sessions").rglob("*.jsonl"))
        self.pane = None
        self.keys = []

    def tmux(self, *args, check=True):
        return subprocess.run(
            ["tmux", "-L", self.socket, *args],
            check=check,
            text=True,
            capture_output=True,
        )

    def start(self, thread=None):
        args = self.args
        command = [
            "env",
            "CODEX_HOME=" + str(args.home),
            "CORBANU_HOME=" + str(args.home),
            "PFTERMINAL_HOME=" + str(args.home),
            "RUST_LOG=warn",
            str(args.binary),
            "--yolo",
            "--no-alt-screen",
            "-C",
            str(args.worktree),
            "-m",
            MODEL,
            "-c",
            'model_provider="openai"',
            "-c",
            'model_reasoning_effort="medium"',
            "-c",
            "check_for_update_on_startup=false",
            "-c",
            "tui.animations=false",
            "-c",
            "analytics.enabled=false",
            "-c",
            "log_dir=" + json.dumps(str(args.evidence / "logs")),
        ]
        if thread:
            command += ["resume", thread]
        self.pane = self.tmux(
            "new-session",
            "-d",
            "-P",
            "-F",
            "#{pane_id}",
            "-s",
            "acceptance",
            "-x",
            "150",
            "-y",
            "46",
            "-c",
            str(args.worktree),
            shlex.join(command),
        ).stdout.strip()
        self.tmux("set-option", "-w", "-t", self.pane, "remain-on-exit", "on")
        self.wait(
            lambda: "Corbanu Terminal" in self.capture() and "tok/s" in self.capture(),
            "TUI ready",
            timeout=90,
        )

    def capture(self):
        return self.tmux("capture-pane", "-p", "-t", self.pane, "-S", "-120").stdout

    def send(self, prompt):
        self.tmux("send-keys", "-t", self.pane, "-l", "--", prompt)
        self.keys.append({"text": prompt})
        # Literal input and Enter are intentionally separate PTY writes.
        time.sleep(0.3)
        self.key("Enter")

    def key(self, key):
        self.tmux("send-keys", "-t", self.pane, key)
        self.keys.append({"key": key})

    def records(self):
        if self.rollout is None:
            for path in (self.args.home / "sessions").rglob("*.jsonl"):
                if path in self.before:
                    continue
                with path.open() as stream:
                    line = stream.readline()
                if not line.endswith("\n"):
                    continue
                record = json.loads(line)
                if record.get("type") == "session_meta" and record["payload"].get(
                    "cwd"
                ) == str(self.args.worktree):
                    self.rollout = path
                    break
        return read_records(self.rollout)

    def state(self):
        return evidence(self.records())

    def wait(self, predicate, label, timeout=None):
        deadline = time.monotonic() + (timeout or self.args.timeout)
        report = 0
        while time.monotonic() < deadline:
            self.state()  # Surface structured failures on every poll.
            if predicate():
                print("PASS: " + label, flush=True)
                return
            if time.monotonic() > report:
                print("Waiting: " + label, flush=True)
                report = time.monotonic() + 30
            time.sleep(0.5)
        raise RuntimeError("timed out: " + label)

    def checkpoint(self, name):
        (self.args.evidence / (name + ".txt")).write_text(self.capture())
        (self.args.evidence / (name + ".json")).write_text(
            json.dumps(self.state(), indent=2) + "\n"
        )

    def stop(self):
        self.send("/exit")
        self.wait(
            lambda: (
                self.tmux(
                    "display-message", "-p", "-t", self.pane, "#{pane_dead}"
                ).stdout.strip()
                == "1"
            ),
            "TUI process exited",
            30,
        )
        self.tmux("kill-session", "-t", "acceptance")


def create_fixture(worktree):
    fixture = worktree / ("astra-acceptance-" + uuid.uuid4().hex[:10])
    fixture.mkdir()
    (fixture / "labels.py").write_text(
        "def normalize_label(value):\n    return value.strip().upper()\n"
    )
    (fixture / "test_labels.py").write_text(
        "import unittest\nfrom labels import normalize_label\n\n"
        "class Labels(unittest.TestCase):\n"
        "    def test_general_normalization(self):\n"
        "        cases = [('  Hello  WORLD ', 'hello world'), ('a\\tb\\nc', 'a b c'),\n"
        "                 ('Straße', 'strasse'), ('', ''), ('  ', ''),\n"
        "                 ('MIXED Case', 'mixed case'), ('x\\u2003y', 'x y')]\n"
        "        for value, expected in cases:\n"
        "            with self.subTest(value=value):\n"
        "                self.assertEqual(normalize_label(value), expected)\n\n"
        "if __name__ == '__main__':\n    unittest.main()\n"
    )
    (fixture / "pause.py").write_text(
        "from pathlib import Path\nimport time\n"
        "Path(__file__).with_name('pause.started').write_text('started')\n"
        "time.sleep(45)\n"
    )
    return fixture


def run(args):
    args.evidence.mkdir(parents=True, exist_ok=False)
    os.chmod(args.evidence, 0o700)
    fixture = create_fixture(args.worktree)
    tests = fixture / "test_labels.py"
    tests_hash = sha256(tests)
    candidate_hash = sha256(args.binary)
    if args.expected_sha256 and candidate_hash != args.expected_sha256:
        raise RuntimeError("candidate hash mismatch")
    tui = LiveTui(args)
    summary = {
        "status": "failed",
        "binary": str(args.binary),
        "sha256": candidate_hash,
        "worktree": str(args.worktree),
        "fixture": str(fixture),
        "tmux_socket": tui.socket,
        "normal_home": str(args.home),
    }
    try:
        tui.start()
        tui.send(
            f"Read the repository README and report one concrete fact about this project. "
            f"Then fix normalize_label in {fixture.name}/labels.py: collapse all whitespace "
            "to single spaces, trim, and Unicode casefold. Do not change the tests. "
            f"Run python3 {fixture.name}/test_labels.py using your execution tool and fix any failures. "
            "Only edit the synthetic labels.py fixture. Do not access credentials, use network tools, "
            "trade, or alter the project. Report the test result."
        )
        tui.wait(
            lambda: len(tui.state()["successful_turns"]) >= 1,
            "live Astra file-edit turn",
        )
        state = tui.state()
        if not state["paired_tool_calls"] or not state["exec_ok"]:
            raise RuntimeError("missing native tool execution evidence")
        if sha256(tests) != tests_hash:
            raise RuntimeError("model modified the acceptance tests")
        subprocess.run([sys.executable, str(tests)], cwd=fixture, check=True)
        tui.checkpoint("01-file-edit")
        tui.send(
            f"Run python3 {fixture.name}/pause.py in the foreground with exec_command. "
            "Wait for completion before answering. This is a benign cancellation test; "
            "do not modify any files or use any other tools."
        )
        tui.wait(
            lambda: (fixture / "pause.started").exists(), "long-running tool started"
        )
        tui.key("Escape")
        tui.wait(lambda: bool(tui.state()["aborted"]), "Escape aborted the active turn")
        tui.checkpoint("02-cancel")
        before = len(tui.state()["successful_turns"])
        tui.send(
            "The previous turn was deliberately cancelled. Rerun the label fixture tests "
            "with your execution tool and report their result. Make no edits."
        )
        tui.wait(
            lambda: len(tui.state()["successful_turns"]) > before,
            "recovery after cancellation",
        )
        tui.checkpoint("03-recovery")
        thread = tui.state()["thread_id"]
        before = len(tui.state()["successful_turns"])
        calls_before = len(tui.state()["exec_ok"])
        tui.stop()
        tui.start(thread)
        tui.send(
            "Resume our previous work: describe the normalization behavior you fixed, "
            "then run the same fixture tests again using your execution tool. Make no edits."
        )
        tui.wait(
            lambda: len(tui.state()["successful_turns"]) > before,
            "same-thread resume response",
        )
        state = tui.state()
        if state["thread_id"] != thread or len(state["exec_ok"]) <= calls_before:
            raise RuntimeError("resume did not preserve the thread and execute a tool")
        if sha256(tests) != tests_hash or sha256(args.binary) != candidate_hash:
            raise RuntimeError("candidate or test changed during qualification")
        subprocess.run([sys.executable, str(tests)], cwd=fixture, check=True)
        tui.checkpoint("04-resume")
        tui.stop()
        summary.update(
            status="passed",
            evidence=state,
            rollout=str(tui.rollout),
            fixture_sha256=sha256(fixture / "labels.py"),
        )
    finally:
        summary["keys"] = tui.keys
        (args.evidence / "summary.json").write_text(
            json.dumps(summary, indent=2) + "\n"
        )
        # Preserve a failed diagnostic TUI, never touch the user's default server.
        if summary["status"] == "passed":
            tui.tmux("kill-server", check=False)
        else:
            print(
                "FAILED diagnostic TUI retained on private socket " + tui.socket,
                flush=True,
            )
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--allow-live",
        action="store_true",
        help="authorize actual Astra inference charges",
    )
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--home", type=Path, required=True)
    parser.add_argument(
        "--worktree", type=Path, required=True, help="disposable git worktree only"
    )
    parser.add_argument(
        "--evidence", type=Path, required=True, help="new private directory"
    )
    parser.add_argument("--expected-sha256")
    parser.add_argument("--timeout", type=int, default=300)
    args = parser.parse_args()
    if not args.allow_live:
        parser.error("live tests require explicit --allow-live")
    for field in ("binary", "home", "worktree", "evidence"):
        setattr(args, field, getattr(args, field).resolve())
    if not (args.worktree / ".git").is_file():
        parser.error("--worktree must be a disposable linked git worktree")
    run(args)


if __name__ == "__main__":
    main()

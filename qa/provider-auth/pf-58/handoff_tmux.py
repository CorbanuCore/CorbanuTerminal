#!/usr/bin/env python3
"""Exercise the STAGED binary + host via real keys, a fake model and real MCP.

No live accounts. Isolated homes/socket; actual code-mode JS, nested shell and
MCP results must return to the model endpoint before the success message exists.
"""

import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tempfile
import threading
import time

from handoff_preflight import digest


WARNINGS = (
    "Code Mode is unavailable",
    "MCP startup incomplete",
    "failed to start",
    "HANDOFF_EXECUTION_FAILED",
    "application panicked",
    "OutputTextDelta without active item",
    "Missing environment variable",
    "Update available!",
)


def events(items):
    return "".join("data: " + json.dumps(item) + "\n\n" for item in items).encode()


def completed():
    return {
        "type": "response.completed",
        "response": {
            "id": "handoff-response",
            "usage": {"input_tokens": 0, "output_tokens": 0, "total_tokens": 0},
        },
    }


def successful_tool_results(outputs):
    if len(outputs) != 1 or outputs[0].get("call_id") != "handoff-call":
        return False
    blocks = outputs[0].get("output")
    if not isinstance(blocks, list):
        return False
    texts = [b.get("text", "") for b in blocks if b.get("type") == "input_text"]
    if (
        len(texts) != 4
        or not texts[0].startswith("Script completed\n")
        or texts[1] != "HOST_HANDOFF_OK"
    ):
        return False
    try:
        shell, mcp = json.loads(texts[2]), json.loads(texts[3])
        return (
            shell.get("exit_code") == 0
            and shell.get("output") == "SHELL_HANDOFF_OK"
            and mcp.get("content") == [{"type": "text", "text": "MCP_HANDOFF_OK"}]
            and not mcp.get("isError", False)
        )
    except (ValueError, AttributeError):
        return False


class Model(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        self.send_response(404)
        self.end_headers()

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        users = [i for i in body.get("input", []) if i.get("role") == "user"]
        latest = json.dumps(users[-1]) if users else ""
        if "Start the cancellable stream" in latest:
            self.server.stream_started.set()
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.end_headers()
            try:
                self.wfile.write(
                    events(
                        [
                            {"type": "response.created", "response": {"id": "stream"}},
                            {
                                "type": "response.output_item.added",
                                "item": {
                                    "type": "message",
                                    "role": "assistant",
                                    "id": "stream-message",
                                    "content": [{"type": "output_text", "text": ""}],
                                },
                            },
                        ]
                    )
                )
                for _ in range(600):
                    if self.server.stopping.is_set():
                        return
                    self.wfile.write(
                        events(
                            [
                                {
                                    "type": "response.output_text.delta",
                                    "item_id": "stream-message",
                                    "delta": "STREAM_ACTIVE\n",
                                }
                            ]
                        )
                    )
                    self.wfile.flush()
                    time.sleep(0.2)
                self.server.stream_finished = True
            except (BrokenPipeError, ConnectionResetError):
                pass
            return
        outputs = [
            i
            for i in body.get("input", [])
            if i.get("type") == "custom_tool_call_output"
        ]
        if "Reply after cancellation" in latest:
            if self.server.stream_finished:
                raise AssertionError("stream finished naturally before cancellation")
            self.server.cancel_recovered = True
            item = {
                "type": "message",
                "role": "assistant",
                "id": "recovery-message",
                "content": [{"type": "output_text", "text": "STREAM_RECOVERY_OK"}],
            }
        elif outputs:
            self.server.outputs.append(outputs)
            passed = successful_tool_results(outputs)
            self.server.results.append(passed)
            reply = "HANDOFF_EXECUTION_PASSED" if passed else "HANDOFF_EXECUTION_FAILED"
            item = {
                "type": "message",
                "role": "assistant",
                "id": "message-handoff",
                "content": [{"type": "output_text", "text": reply}],
            }
        else:
            code = (
                'text("HOST_HANDOFF_OK"); '
                'text(await tools.exec_command({cmd: "printf SHELL_HANDOFF_OK"})); '
                'const t = ALL_TOOLS.find(t => t.name.includes("handoff") && t.name.endsWith("echo")); '
                'if (!t) throw new Error("handoff MCP tool not discovered"); '
                "text(await tools[t.name]({}));"
            )
            item = {
                "type": "custom_tool_call",
                "name": "exec",
                "call_id": "handoff-call",
                "input": code,
            }
        payload = events(
            [
                {"type": "response.created", "response": {"id": "handoff-response"}},
                {"type": "response.output_item.done", "item": item},
                completed(),
            ]
        )
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)


def run(candidate, repo, evidence, stream_cancel_key="Escape"):
    evidence.mkdir(parents=True, exist_ok=False)
    server = ThreadingHTTPServer(("127.0.0.1", 0), Model)
    server.results = []
    server.outputs = []
    server.cancel_recovered = False
    server.stream_started = threading.Event()
    server.stopping = threading.Event()
    server.stream_finished = False
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    result = {
        "stream_cancel_key": stream_cancel_key,
        "passed": False,
        "candidate": str(candidate),
        "sha256": digest(candidate),
        "host_sha256": digest(candidate.with_name("codex-code-mode-host")),
        "human_acceptance": False,
    }
    try:
        # Unix-domain sockets have a short path limit; evidence paths can be long.
        with tempfile.TemporaryDirectory(
            prefix="ct-", dir=os.environ.get("TMPDIR")
        ) as temp:
            root = Path(temp)
            socket = str(root / "tmux.sock")
            # No inherited auth or user-home configuration. Synthetic trace only.
            env = {
                k: v
                for k, v in os.environ.items()
                if k in ("PATH", "LANG", "LC_ALL", "TMPDIR")
            }
            env.update(
                HOME=str(root),
                TERM="xterm-256color",
                RUST_LOG="trace",
                CORBANU_TEST_NO_NATIVE_KEYRING="1",
            )

            def tmux(*args, check=True):
                return subprocess.run(
                    ["tmux", "-S", socket, *args],
                    env=env,
                    text=True,
                    capture_output=True,
                    check=check,
                    timeout=15,
                )

            def pane(name):
                return tmux("capture-pane", "-p", "-S", "-", "-t", name).stdout

            def wait(name, expected):
                deadline = time.monotonic() + 60
                while time.monotonic() < deadline:
                    history = pane(name)
                    if any(w in " ".join(history.split()) for w in WARNINGS):
                        raise AssertionError("startup/runtime warning in " + name)
                    # A dismissed menu in scrollback is not a completed UI
                    # transition. Require the checkpoint in the current frame.
                    text = tmux("capture-pane", "-p", "-t", name).stdout
                    if expected in text:
                        return text
                    time.sleep(0.15)
                raise AssertionError("missing checkpoint: " + expected)

            def send(name, text):
                tmux("send-keys", "-t", name, "-l", "--", text)
                wait(name, text)
                tmux("send-keys", "-t", name, "Enter")

            def choose(name, label):
                for _ in range(30):
                    text = tmux("capture-pane", "-p", "-t", name).stdout
                    selected = [
                        line
                        for line in text.splitlines()
                        if re.match(r"\s*[›>]\s+", line)
                    ]
                    if selected and label in selected[-1]:
                        tmux("send-keys", "-t", name, "Enter")
                        return
                    tmux("send-keys", "-t", name, "Down")
                    time.sleep(0.15)
                raise AssertionError("menu option unavailable: " + label)

            def wait_configured(name, label):
                deadline = time.monotonic() + 15
                while time.monotonic() < deadline:
                    viewport = tmux("capture-pane", "-p", "-t", name).stdout
                    if re.search(re.escape(label) + r"\s+Enabled\s*·\s*configured", viewport):
                        return viewport
                    time.sleep(0.15)
                (evidence / "setup-incomplete.txt").write_text(viewport)
                raise AssertionError(label + " setup did not reach configured status")

            try:
                commands = {}
                # Three independent, concurrently live processes, then same-home restart.
                for number in range(3):
                    name = f"handoff-{number}"
                    home = root / name
                    home.mkdir()
                    config = (
                        'model="gpt-6-astra"\nmodel_provider="openai"\n'
                        f'openai_base_url="http://127.0.0.1:{server.server_port}/v1"\n'
                        'cli_auth_credentials_store="file"\ncheck_for_update_on_startup=false\n'
                        "suppress_unstable_features_warning=true\n"
                        'approval_policy="never"\nsandbox_mode="danger-full-access"\n'
                        "[features]\ncode_mode=true\ncode_mode_host=true\n"
                        f'[projects.{json.dumps(str(repo))}]\ntrust_level="trusted"\n'
                        f"[mcp_servers.handoff]\ncommand={json.dumps(sys.executable)}\n"
                        f"args=[{json.dumps(str(Path(__file__).with_name('synthetic_mcp.py').resolve()))}]\n"
                    )
                    (home / "config.toml").write_text(config)
                    (home / "auth.json").write_text(
                        json.dumps({"OPENAI_API_KEY": "handoff-synthetic-only"})
                    )
                    command = shlex.join(
                        [
                            "env",
                            f"CORBANU_HOME={home}",
                            f"CODEX_HOME={home}",
                            f"PFTERMINAL_HOME={home}",
                            str(candidate),
                            "--no-alt-screen",
                            "-C",
                            str(repo),
                            "-c",
                            f'log_dir="{home / "logs"}"',
                        ]
                    )
                    commands[name] = command
                    tmux(
                        "new-session",
                        "-d",
                        "-s",
                        name,
                        "-x",
                        "120",
                        "-y",
                        "50",
                        command,
                    )
                    wait(name, "Corbanu Terminal")
                    send(name, "Perform the synthetic handoff tool check")
                    wait(name, "HANDOFF_EXECUTION_PASSED")
                    (evidence / f"{name}.txt").write_text(pane(name))
                    if number == 0:
                        send(name, "/exit")
                        deadline = time.monotonic() + 30
                        while (
                            tmux("has-session", "-t", name, check=False).returncode == 0
                        ):
                            if time.monotonic() > deadline:
                                raise AssertionError("exit hung")
                            time.sleep(0.15)
                        tmux(
                            "new-session",
                            "-d",
                            "-s",
                            name,
                            "-x",
                            "120",
                            "-y",
                            "50",
                            command,
                        )
                        wait(name, "Corbanu Terminal")
                        send(name, "Perform the synthetic handoff tool check")
                        wait(name, "HANDOFF_EXECUTION_PASSED")
                        (evidence / "restart.txt").write_text(pane(name))
                        send(name, "Start the cancellable stream")
                        assert server.stream_started.wait(timeout=15), (
                            "stream never reached endpoint"
                        )
                        # A footer in scrollback can belong to the previous
                        # turn. Require actual output from this stream before
                        # sending Escape, and a positive interruption event
                        # before typing the follow-up (absence of a transient
                        # footer is not proof that the turn has ended).
                        wait(name, "STREAM_ACTIVE")
                        tmux("send-keys", "-t", name, stream_cancel_key)
                        wait(name, "Conversation interrupted")
                        send(name, "Reply after cancellation")
                        wait(name, "STREAM_RECOVERY_OK")
                        (evidence / "stream-recovery.txt").write_text(pane(name))
                assert len(server.results) == 4 and all(server.results), server.results
                assert server.cancel_recovered
                assert (
                    len(
                        tmux(
                            "list-panes", "-a", "-F", "#{pane_pid}"
                        ).stdout.splitlines()
                    )
                    == 3
                )
                name = "handoff-0"
                before = (root / name / "config.toml").read_text()
                send(name, "/permissions")
                wait(name, "Update Model Permissions")
                tmux("send-keys", "-t", name, "Escape")
                send(name, "/security")
                wait(name, "Security profiles")
                tmux("send-keys", "-t", name, "Down", "Enter")
                wait(name, "Nothing changed.")
                tmux("send-keys", "-t", name, "Escape")
                assert (root / name / "config.toml").read_text() == before
                send(name, "/providers")
                wait(name, "Configure providers and control")
                choose(name, "Claude Account")
                wait(name, "Set up with Claude account")
                choose(name, "Set up with Claude account")
                wait(name, "Claude Plan authentication")
                choose(name, "Long-lived subscription token")
                wait(name, "Long-lived token — masked")
                tmux("resize-window", "-t", name, "-x", "40", "-y", "50")
                wait(name, "claude setup-token")
                (evidence / "claude-guidance-40.txt").write_text(pane(name))
                tmux("resize-window", "-t", name, "-x", "120", "-y", "50")
                # Store a synthetic token through the actual masked form. Never
                # send it to a provider: this fixture tests setup/catalog only.
                tmux("send-keys", "-t", name, "-l", "--", "synthetic-picker-managed-token")
                wait(name, "••")
                tmux("send-keys", "-t", name, "Enter")
                wait(name, "Configure providers and control")
                manager = wait_configured(name, "Claude Account")
                assert "Anthropic" in manager, manager
                (evidence / "provider-inventory.txt").write_text(manager)
                tmux("send-keys", "-t", name, "Escape")
                for phase in ("after-setup", "after-restart"):
                    send(name, "/model")
                    wait(name, "Select Model")
                    for _ in range(16):
                        viewport = tmux("capture-pane", "-p", "-t", name).stdout
                        if "[Claude Plan]" in viewport:
                            break
                        tmux("send-keys", "-t", name, "Right")
                        time.sleep(0.15)
                    else:
                        raise AssertionError("Claude Plan tab missing " + phase)
                    for model in ("claude-opus-5-plan", "claude-fable-5-plan", "claude-fable-5-1-plan"):
                        assert "Model: " + model + "." in viewport, viewport
                    expected_current = 0 if phase == "after-setup" else 1
                    assert viewport.count("(current)") == expected_current, viewport
                    assert "Other" not in viewport, viewport
                    (evidence / f"claude-catalog-{phase}.txt").write_text(viewport)
                    if phase == "after-setup":
                        choose(name, "Claude Fable 5.1 Plan")
                        wait(name, "Select Reasoning")
                        choose(name, "High")
                        wait(name, "Model changed to Claude Fable 5.1 Plan")
                        # Match the reported journey: Fable current, manager
                        # reopened repeatedly, then same-home process restart.
                        for _ in range(3):
                            send(name, "/providers")
                            wait(name, "Configure providers and control")
                            tmux("send-keys", "-t", name, "Escape")
                        send(name, "/exit")
                        deadline = time.monotonic() + 30
                        while tmux("has-session", "-t", name, check=False).returncode == 0:
                            assert time.monotonic() < deadline, "exit hung"
                            time.sleep(0.15)
                        tmux("new-session", "-d", "-s", name, "-x", "120", "-y", "50", commands[name])
                        wait(name, "Corbanu Terminal")
                        send(name, "/providers")
                        wait(name, "Configure providers and control")
                        tmux("send-keys", "-t", name, "Escape")
                    else:
                        tmux("send-keys", "-t", name, "Escape")
                # API access remains an independent credential, but becomes
                # pickable immediately after its own masked setup succeeds.
                send(name, "/providers")
                wait(name, "Configure providers and control")
                choose(name, "Anthropic")
                wait(name, "Set up with API key")
                choose(name, "Set up with API key")
                wait(name, "API key — masked")
                tmux("send-keys", "-t", name, "-l", "--", "synthetic-picker-anthropic-key")
                wait(name, "••")
                tmux("send-keys", "-t", name, "Enter")
                wait(name, "Configure providers and control")
                manager = wait_configured(name, "Anthropic")
                (evidence / "anthropic-manager-after-setup.txt").write_text(manager)
                tmux("send-keys", "-t", name, "Escape")
                send(name, "/model")
                wait(name, "Select Model")
                for _ in range(16):
                    viewport = tmux("capture-pane", "-p", "-t", name).stdout
                    if "[Anthropic]" in viewport:
                        break
                    tmux("send-keys", "-t", name, "Right")
                    time.sleep(0.15)
                else:
                    raise AssertionError("Anthropic tab missing after its API-key setup")
                for model in ("claude-opus-5", "claude-fable-5", "claude-fable-5-1"):
                    assert "Model: " + model + "." in viewport, viewport
                assert "(current)" not in viewport, viewport
                assert "synthetic-picker-anthropic-key" not in pane(name)
                (evidence / "anthropic-catalog-after-setup.txt").write_text(viewport)
                tmux("send-keys", "-t", name, "Escape")
                result.update(
                    passed=True,
                    tool_round_trips=4,
                    concurrent_processes=3,
                    same_home_restart=True,
                    stream_cancel_recovery=True,
                    permissions_security_cancel_inert=True,
                    claude_guidance_40_columns=True,
                    claude_catalog_after_setup_and_restart=True,
                    anthropic_management_entry_present=True,
                    anthropic_catalog_after_api_setup=True,
                )
            finally:
                for number in range(3):
                    captured = tmux(
                        "capture-pane",
                        "-p",
                        "-S",
                        "-",
                        "-t",
                        f"handoff-{number}",
                        check=False,
                    )
                    if captured.returncode == 0:
                        (evidence / f"final-{number}.txt").write_text(captured.stdout)
                tmux("kill-server", check=False)
    finally:
        server.stopping.set()
        server.shutdown()
        server.server_close()
        (evidence / "result.json").write_text(json.dumps(result, indent=2) + "\n")
        (evidence / "synthetic-tool-results.json").write_text(
            json.dumps(server.outputs, indent=2) + "\n"
        )
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--stream-cancel-key", choices=("Escape", "C-c"), default="Escape")
    args = parser.parse_args()
    print(
        json.dumps(
            run(
                args.candidate.resolve(strict=True),
                args.repo.resolve(strict=True),
                args.evidence,
                args.stream_cancel_key,
            )
        )
    )


if __name__ == "__main__":
    main()

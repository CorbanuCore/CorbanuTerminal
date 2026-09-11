"""Opt-in existing-profile menu check. Never sends prompts or edits credentials.

Run with an explicitly supplied home/candidate. Startup is bounded; a Keychain
approval cannot be completed by this script. Captures contain only the startup
and provider/model menus, never credential forms. No global trace logging.
"""
import argparse
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import time
from handoff_preflight import digest


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--home", type=Path, required=True)
    parser.add_argument("--cwd", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    args.evidence.mkdir(parents=True, exist_ok=False)
    config_before = (args.home / "config.toml").read_bytes()
    results = {"passed": False, "candidate": str(args.candidate),
               "sha256": digest(args.candidate), "launches": [], "human_acceptance": False}
    env = dict(os.environ)
    env.update(CORBANU_HOME=str(args.home), CODEX_HOME=str(args.home),
               PFTERMINAL_HOME=str(args.home), RUST_LOG="codex_keyring_store=trace")
    env.pop("CORBANU_TEST_NO_NATIVE_KEYRING", None)
    with tempfile.TemporaryDirectory(prefix="lp-", dir=os.environ["TMPDIR"]) as temp:
        socket = str(Path(temp) / "t.sock")

        def tmux(*words, check=True):
            return subprocess.run(["tmux", "-S", socket, *words], env=env,
                                  capture_output=True, text=True, check=check, timeout=10)

        def wait_for(text, timeout=8):
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                capture = tmux("capture-pane", "-p", "-t", "probe").stdout
                if text in capture:
                    return capture
                time.sleep(0.1)
            raise AssertionError("Menu unavailable without user interaction: " + text)

        def send(text):
            tmux("send-keys", "-t", "probe", "-l", "--", text)
            tmux("send-keys", "-t", "probe", "Enter")

        try:
            for number in range(2):
                started = time.monotonic()
                command = "exec " + shlex.join([str(args.candidate), "--no-alt-screen", "-C", str(args.cwd)])
                tmux("new-session", "-d", "-s", "probe", "-x", "150", "-y", "45", command)
                pid = tmux("display-message", "-p", "-t", "probe", "#{pane_pid}").stdout.strip()
                wait_for("Corbanu Terminal")
                send("/providers")
                manager = wait_for("Configure providers and control")
                assert "Anthropic" in manager, manager
                (args.evidence / f"providers-{number}.txt").write_text(manager)
                tmux("send-keys", "-t", "probe", "Escape")
                send("/model")
                picker = wait_for("Select Model")
                for _ in range(16):
                    if "[Claude Plan]" in picker:
                        break
                    tmux("send-keys", "-t", "probe", "Right")
                    time.sleep(0.1)
                    picker = tmux("capture-pane", "-p", "-t", "probe").stdout
                assert "[Claude Plan]" in picker, picker
                assert picker.count("(current)") == 1, picker
                for model in ("claude-opus-5-plan", "claude-fable-5-1-plan", "claude-fable-5-plan"):
                    assert "Model: " + model + "." in picker, picker
                assert "[Other]" not in picker, picker
                (args.evidence / f"models-{number}.txt").write_text(picker)
                results["launches"].append({"pid": pid, "menu_seconds": round(time.monotonic() - started, 2)})
                tmux("send-keys", "-t", "probe", "Escape")
                send("/exit")
                deadline = time.monotonic() + 8
                while tmux("has-session", "-t", "probe", check=False).returncode == 0:
                    assert time.monotonic() < deadline, "exit hung"
                    time.sleep(0.1)
            assert (args.home / "config.toml").read_bytes() == config_before
            results["passed"] = True
        finally:
            tmux("kill-server", check=False)
            (args.evidence / "result.json").write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results))


if __name__ == "__main__":
    main()

"""Synthetic actual-key supporting proof, not protected-mode qualification."""

import argparse
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    args.evidence.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="anchor-tmux-", dir=os.environ["TMPDIR"]) as directory:
        home = Path(directory)
        config = (
            'model = "gpt-5.6-terra"\nmodel_provider = "openai"\n'
            'cli_auth_credentials_store = "file"\ncheck_for_update_on_startup = false\n'
            '[security]\nversion = 1\nlevel = "moderate"\n'
            f'[projects.{json.dumps(str(args.repo))}]\ntrust_level = "trusted"\n'
            '[tui]\nanimations = false\n'
        )
        (home / "config.toml").write_text(config)
        (home / "auth.json").write_text(json.dumps({"OPENAI_API_KEY": "anchor-synthetic-fixture"}))
        socket = str(home / "tmux.sock")
        env = dict(os.environ, CODEX_HOME=str(home), CORBANU_HOME=str(home), RUST_LOG="trace")

        def tmux(*arguments, check=True):
            return subprocess.run(["tmux", "-S", socket, *arguments], env=env,
                                  capture_output=True, text=True, check=check)

        def wait_for(expected):
            deadline = time.monotonic() + 45
            while time.monotonic() < deadline:
                result = tmux("capture-pane", "-p", "-t", "anchor", check=False)
                if result.returncode == 0 and expected in result.stdout:
                    return result.stdout
                time.sleep(0.15)
            raise AssertionError(f"Missing visible checkpoint: {expected}")

        def command(text):
            tmux("send-keys", "-t", "anchor", "-l", text)
            wait_for(text)
            tmux("send-keys", "-t", "anchor", "Enter")

        binary_command = shlex.join([str(args.binary), "--no-alt-screen", "-C", str(args.repo),
                                    "-c", f'log_dir="{home / "logs"}"'])
        try:
            for run in (1, 2):
                tmux("new-session", "-d", "-s", "anchor", "-x", "120", "-y", "48", binary_command)
                wait_for("Corbanu Terminal")
                command("/security")
                pane = wait_for("Effective protection: unverified")
                (args.evidence / f"restart-{run}-security.txt").write_text(pane)
                tmux("send-keys", "-t", "anchor", "Escape")
                time.sleep(0.3)
                command("/status")
                pane = wait_for("Security:")
                assert "Moderate" in pane and "unverified" in pane
                (args.evidence / f"restart-{run}-status.txt").write_text(pane)
                command("/exit")
                deadline = time.monotonic() + 45
                while tmux("has-session", "-t", "anchor", check=False).returncode == 0:
                    if time.monotonic() >= deadline:
                        raise AssertionError("TUI did not exit")
                    time.sleep(0.15)
                assert (home / "config.toml").read_text() == config
            print("PASS: two real starts, /security, Escape, /status, /exit; identical home/config; protection unverified")
        finally:
            tmux("kill-server", check=False)


if __name__ == "__main__":
    main()

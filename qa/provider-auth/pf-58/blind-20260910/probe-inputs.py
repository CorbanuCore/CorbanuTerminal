"""Code-blind F08/F09/F25 probes: real keys, disposable home, no live accounts.

Records observations rather than treating stored/configured as authenticated.
The supplied candidate is never rebuilt or altered. Run on both host platforms.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    args.evidence.mkdir(parents=True, exist_ok=False)
    binary = args.candidate.resolve(strict=True)
    result = {"candidate": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "human_acceptance": False, "observations": [], "completed": False}
    with tempfile.TemporaryDirectory(prefix="bi-", dir=os.environ["TMPDIR"]) as temp:
        root = Path(temp)
        home = root / "profile"
        home.mkdir()
        (home / "config.toml").write_text(
            'model="gpt-6-astra"\nmodel_provider="openai"\n'
            'cli_auth_credentials_store="file"\ncheck_for_update_on_startup=false\n'
            'suppress_unstable_features_warning=true\n'
            'openai_base_url="http://127.0.0.1:1/v1"\n'
            f'[projects.{json.dumps(str(root))}]\ntrust_level="trusted"\n')
        # No inference requests in this probe; network route fails closed locally.
        (home / "auth.json").write_text(json.dumps({"OPENAI_API_KEY": "synthetic-existing-key"}))
        env = {k: v for k, v in os.environ.items() if k in ("PATH", "LANG", "LC_ALL", "TMPDIR")}
        env.update(HOME=str(root), CORBANU_HOME=str(home), CODEX_HOME=str(home),
                   PFTERMINAL_HOME=str(home), TERM="xterm-256color", RUST_LOG="trace",
                   CORBANU_TEST_NO_NATIVE_KEYRING="1")
        socket = str(root / "t.sock")

        def tmux(*args, check=True):
            return subprocess.run(["tmux", "-S", socket, *args], env=env,
                                  capture_output=True, text=True, check=check, timeout=10)

        def screen():
            return tmux("capture-pane", "-p", "-t", "probe").stdout

        def wait(text):
            deadline = time.monotonic() + 30
            while time.monotonic() < deadline:
                value = screen()
                if text in value:
                    return value
                time.sleep(.1)
            raise AssertionError("Missing checkpoint: " + text)

        def literal(text):
            tmux("send-keys", "-t", "probe", "-l", "--", text)

        def key(*keys):
            tmux("send-keys", "-t", "probe", *keys)

        def select(label):
            for _ in range(30):
                rows = [line for line in screen().splitlines() if re.match(r"\s*[›>]\s+", line)]
                if rows and label in rows[-1]:
                    key("Enter")
                    return
                key("Down")
                time.sleep(.1)
            raise AssertionError("Cannot select " + label)

        def save(name):
            value = screen()
            (args.evidence / (name + ".txt")).write_text(value)
            return value

        try:
            command = shlex.join([str(binary), "--no-alt-screen", "-C", str(root),
                                  "-c", f'log_dir="{home / "logs"}"'])
            tmux("new-session", "-d", "-s", "probe", "-x", "116", "-y", "43", command)
            wait("Corbanu Terminal")
            literal("/providers")
            key("Enter")
            wait("Configure providers and control")
            select("Anthropic")
            wait("Set up with API key")
            select("Set up with API key")
            wait("API key — masked")
            key("Enter")
            time.sleep(.3)
            empty = save("empty")
            result["observations"].append({"input": "empty", "remains_in_form": "API key — masked" in empty})
            literal("   ")
            key("Enter")
            time.sleep(.3)
            spaces = save("whitespace")
            result["observations"].append({"input": "whitespace", "remains_in_form": "API key — masked" in spaces})
            assert "API key — masked" in spaces, "blank input unexpectedly left form"
            key("C-u")
            secret = "synthetic-invalid-blind-test-key"
            literal(secret)
            wait("••")
            assert secret not in save("masked"), "secret rendered in form"
            key("Escape")
            wait("Configure providers and control")
            cancelled = save("cancelled")
            assert secret not in cancelled, "cancelled secret rendered"
            result["observations"].append({"input": "cancelled-canary", "masked": True})
            select("Anthropic")
            wait("Set up with API key")
            select("Set up with API key")
            wait("API key — masked")
            literal(secret)
            wait("••")
            key("Enter")
            wait("Configure providers and control")
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                state = screen()
                if re.search(r"Anthropic\s+Enabled\s*·\s*configured", state):
                    break
                time.sleep(.1)
            stored = save("invalid-submitted")
            history = tmux("capture-pane", "-p", "-S", "-", "-t", "probe").stdout
            assert secret not in history, "secret leaked to scrollback"
            (args.evidence / "scrollback.txt").write_text(history)
            result["observations"].append({"input": "synthetic-invalid", "configured_label":
                bool(re.search(r"Anthropic\s+Enabled\s*·\s*configured", stored)),
                "secret_absent_scrollback": True, "remote_auth_validation_proven": False})
            result["completed"] = True
        finally:
            last = tmux("capture-pane", "-p", "-t", "probe", check=False)
            (args.evidence / "last.txt").write_text(last.stdout)
            tmux("kill-server", check=False)
            (args.evidence / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))


if __name__ == "__main__":
    main()

"""Isolated, synthetic-only real-key harness shared by PF-58 journeys."""
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import time


class Session:
    def __init__(self, candidate, repo, root, evidence, env=None):
        self.candidate, self.repo = Path(candidate), Path(repo)
        self.root, self.evidence = Path(root), Path(evidence)
        self.home = self.root / "home"
        self.home.mkdir(exist_ok=True)
        self.evidence.mkdir(parents=True, exist_ok=True)
        self.socket = str(self.root / "s")
        self.env = {k: v for k, v in os.environ.items() if k in ("PATH", "LANG", "LC_ALL", "TMPDIR")}
        self.env.update(HOME=str(self.root), CODEX_HOME=str(self.home),
                        CORBANU_HOME=str(self.home), PFTERMINAL_HOME=str(self.home),
                        CORBANU_TEST_NO_NATIVE_KEYRING="1", RUST_LOG="trace", TERM="xterm-256color")
        self.env.update(env or {})
        self.keys = []
        self.clients = []

    def tmux(self, *args, check=True):
        return subprocess.run(["tmux", "-S", self.socket, *args], env=self.env,
                              text=True, capture_output=True, check=check, timeout=15)

    def start(self):
        cmd = shlex.join([str(self.candidate), "--no-alt-screen", "-C", str(self.repo),
                          "-c", f'log_dir="{self.home / "log"}"'])
        self.tmux("new-session", "-d", "-s", "qa", "-x", "140", "-y", "44", cmd)
        self.wait("Corbanu Terminal")

    def view(self, history=False):
        return self.tmux("capture-pane", "-p", *( ["-S", "-"] if history else []), "-t", "qa").stdout

    def wait(self, text, timeout=45):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            frame = self.view()
            if text in frame:
                return frame
            time.sleep(.12)
        self.save("failure")
        raise AssertionError("missing visible checkpoint: " + text)

    def key(self, key):
        self.keys.append({"key": key})
        self.tmux("send-keys", "-t", "qa", key)
        time.sleep(.12)

    def text(self, text, secret=False):
        self.keys.append({"text": "<synthetic secret>" if secret else text})
        self.tmux("send-keys", "-t", "qa", "-l", "--", text)

    def submit(self, text):
        self.text(text)
        self.wait(text)
        self.key("Enter")

    def choose(self, label, confirm=True):
        for _ in range(45):
            selected = [line for line in self.view().splitlines() if line.lstrip().startswith((">", "›"))]
            if selected and label in selected[-1]:
                if confirm:
                    self.key("Enter")
                return
            self.key("Down")
        self.save("missing-option")
        raise AssertionError("missing selection: " + label)

    def manager(self):
        self.submit("/providers")
        self.wait("Configure providers and control")

    def picker(self, tab=None):
        self.submit("/model")
        self.wait("Select Model")
        if tab:
            for _ in range(20):
                frame = self.view()
                if "[" + tab + "]" in frame:
                    return
                # The single-provider OpenAI picker has no tab strip. A custom
                # credential alone does not declare a supported model catalog.
                if tab == "OpenAI" and "gpt-6-astra" in frame and "Use Left/Right" not in frame:
                    return
                self.key("Right")
            raise AssertionError("missing model tab " + tab)

    def save(self, name):
        (self.evidence / (name + ".txt")).write_text(self.view())
        (self.evidence / (name + "-history.txt")).write_text(self.view(True))

    def exit(self):
        self.submit("/exit")
        deadline = time.monotonic() + 30
        while self.tmux("has-session", "-t", "qa", check=False).returncode == 0:
            assert time.monotonic() < deadline, "exit timed out"
            time.sleep(.15)

    def attach_cycle(self):
        import pty
        import threading
        import termios
        master, slave = pty.openpty()
        termios.tcsetwinsize(slave, (44, 140))
        def drain():
            try:
                while os.read(master, 65536):
                    pass
            except OSError:
                pass
        threading.Thread(target=drain, daemon=True).start()
        proc = subprocess.Popen(["tmux", "-S", self.socket, "attach", "-t", "qa"],
                                stdin=slave, stdout=slave, stderr=slave, env=self.env)
        os.close(slave)
        self.clients.append((proc, master))
        deadline = time.monotonic() + 10
        while not self.tmux("list-clients", "-F", "#{client_pid}").stdout.strip():
            assert time.monotonic() < deadline, "client did not attach"
            time.sleep(.1)
        self.tmux("detach-client", "-s", "qa")
        proc.wait(timeout=10)
        os.close(master)
        self.clients.remove((proc, master))

    def close(self):
        if self.tmux("has-session", "-t", "qa", check=False).returncode == 0:
            self.save("final")
        self.tmux("kill-server", check=False)
        for proc, master in self.clients:
            proc.wait(timeout=10)
            os.close(master)
        (self.evidence / "keys.json").write_text(json.dumps(self.keys, indent=2))

    def receipt(self, result):
        result.update(candidate=str(self.candidate), sha256=hashlib.file_digest(self.candidate.open("rb"), "sha256").hexdigest(), human_acceptance=False)
        (self.evidence / "result.json").write_text(json.dumps(result, indent=2))


def base_config(repo):
    return ('cli_auth_credentials_store="file"\ncheck_for_update_on_startup=false\n'
            'suppress_unstable_features_warning=true\napproval_policy="never"\n'
            'sandbox_mode="danger-full-access"\n[analytics]\nenabled=false\n'
            '[tui]\nanimations=false\n'
            f'[projects.{json.dumps(str(repo))}]\ntrust_level="trusted"\n')


def sse(text):
    events = [{"type": "response.created", "response": {"id": "pf58-response"}},
              {"type": "response.output_item.done", "item": {"type": "message", "role": "assistant", "id": "pf58-message", "content": [{"type": "output_text", "text": text}]}},
              {"type": "response.completed", "response": {"id": "pf58-response", "usage": {"input_tokens": 0, "output_tokens": 0, "total_tokens": 0}}}]
    return "".join("data: " + json.dumps(e) + "\n\n" for e in events).encode()

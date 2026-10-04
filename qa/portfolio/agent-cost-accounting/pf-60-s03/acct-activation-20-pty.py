# /// script
# requires-python = ">=3.11"
# dependencies = ["pexpect==4.9.0", "pyte==0.8.2"]
# ///
"""Supporting developer smoke, not independent code-blind qualification."""
import argparse
import hashlib
import http.server
import json
import os
from pathlib import Path
import runpy
import tempfile
import threading
import time

import pexpect
import pyte

parser = argparse.ArgumentParser()
parser.add_argument("binary", type=Path)
parser.add_argument("output", type=Path)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=False)
repo = Path(__file__).resolve().parents[4]
isolation = runpy.run_path(str(repo / "scripts/isolated_rust_tests.py"))
requests = []


class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        self.send_error(404)

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        # Deliberately retain no headers, prompts or other request content.
        requests.append({"path": self.path, "model": body.get("model")})
        events = [
            {"type": "response.created", "response": {"id": "developer-fixture"}},
            {"type": "response.output_item.done", "item": {
                "type": "message", "role": "assistant", "id": "fixture-message",
                "content": [{"type": "output_text", "text": "Activation fixture complete."}]}},
            {"type": "response.completed", "response": {"id": "developer-fixture", "usage": {
                "input_tokens": 7, "input_tokens_details": {"cached_tokens": 0, "cache_write_tokens": 0},
                "output_tokens": 3, "output_tokens_details": {"reasoning_tokens": 0}, "total_tokens": 10}}},
        ]
        data = "".join(f"event: {e['type']}\ndata: {json.dumps(e)}\n\n" for e in events).encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
threading.Thread(target=server.serve_forever, daemon=True).start()
screen = pyte.Screen(140, 55)
stream = pyte.Stream(screen)
keys = []
with tempfile.TemporaryDirectory(prefix="acct-activation-20-") as temp:
    root = Path(temp)
    profile = root / "profile"
    work = root / "work"
    profile.mkdir()
    work.mkdir()
    (profile / "config.toml").write_text(
        'model_provider = "openai"\nmodel = "gpt-5.6-sol"\n'
        'model_reasoning_effort = "low"\ncli_auth_credentials_store = "file"\n'
        'check_for_update_on_startup = false\n'
        f'openai_base_url = "http://127.0.0.1:{server.server_port}/v1"\n'
        '[features]\nsqlite = true\n'
        f'[projects.{json.dumps(str(work))}]\ntrust_level = "trusted"\n'
    )
    env = isolation["test_environment"]({"PATH": os.environ["PATH"], "HOME": str(root)}, profile)
    (profile / "auth.json").write_text(json.dumps({"OPENAI_API_KEY": "synthetic-loopback-only"}))
    env.update({"OPENAI_API_KEY": "synthetic-loopback-only", "TERM": "xterm-256color",
                "RUST_LOG": "trace", "NO_COLOR": "1"})
    raw = (args.output / "terminal.raw").open("w")
    child = pexpect.spawn(str(args.binary.resolve()),
        ["--no-alt-screen", "-C", str(work), "-c", f'log_dir="{root / "logs"}"'],
        env=env, cwd=work, dimensions=(55, 140), encoding="utf-8", timeout=1)
    child.logfile_read = raw

    def drain(seconds=1):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            try:
                chunk = child.read_nonblocking(65536, timeout=0.2)
            except pexpect.TIMEOUT:
                continue
            except pexpect.EOF:
                break
            stream.feed(chunk)
            if "\x1b[6n" in chunk:
                child.send("\x1b[1;1R")
            if "\x1b[c" in chunk:
                child.send("\x1b[?1;2c")

    def snapshot(name):
        text = "\n".join(line.rstrip() for line in screen.display)
        (args.output / f"{name}.txt").write_text(text + "\n")
        return text

    def send(text):
        keys.append(repr(text))
        child.send(text)
        drain(0.4)

    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            drain(1)
            if "gpt-5.6-sol" in "\n".join(screen.display):
                break
        snapshot("startup")
        send("Reply with fixture completion.")
        send("\r")
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            drain(1)
            if requests and "Activation fixture complete." in "\n".join(screen.display):
                break
        snapshot("sampled")
        send("/usage requests")
        send("\r")
        drain(5)
        overview = snapshot("overview")
        for _ in range(10):
            send("\x1b[B")
        drain(1)
        rendered = snapshot("inspector")
        print(overview)
        print(rendered)
        (args.output / "requests.json").write_text(json.dumps(requests, indent=2) + "\n")
        if not requests:
            raise RuntimeError("No sampled HTTP request reached the loopback fixture")
        if ("ledger not installed" in overview
                or "Estimated token cost for recorded attempts" not in overview
                or "Known subtotal exact USD" not in rendered):
            raise RuntimeError("Inspector did not render a populated day and scrolled subtotal")
    finally:
        send("\x03")
        send("\x03")
        drain(1)
        if child.isalive():
            child.terminate(force=True)
        raw.close()
        (args.output / "keys.json").write_text(json.dumps(keys, indent=2) + "\n")
        (args.output / "binary.json").write_text(json.dumps({
            "path": str(args.binary.resolve()),
            "sha256": hashlib.file_digest(args.binary.open("rb"), "sha256").hexdigest(),
            "isolation": "empty profile/HOME, minimal environment, debug native-keyring denial; no OS sandbox",
            "kind": "implementer smoke only; synthetic loopback provider, no live inference",
        }, indent=2) + "\n")
server.shutdown()

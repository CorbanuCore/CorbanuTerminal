"""Synthetic exact-package selection/persistence/keyboard/failure journeys."""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import tempfile
import threading
import time
import tomllib

from functional_harness import Session, base_config, sse


class Endpoint(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        self.send_response(404)
        self.end_headers()

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))))
        auth = self.headers.get("Authorization", "")
        users = [item for item in body.get("input", []) if item.get("role") == "user"]
        latest = json.dumps(users[-1]) if users else ""
        with self.server.lock:
            serial = len(self.server.requests) + 1
            status = self.server.status
            self.server.requests.append({"serial": serial, "path": self.path, "model": body.get("model"),
                "effort": body.get("reasoning", {}).get("effort"), "auth_route": auth.removeprefix("Bearer synthetic-"),
                "status": status, "latest_user": latest})
        if "DELAY_FIXTURE" in latest:
            self.server.started.set()
            self.server.release.wait(30)
        if status == 200:
            payload, kind = sse(f"JOURNEY_REPLY_{serial}"), "text/event-stream"
        else:
            payload, kind = json.dumps({"error": {"code": "invalid_api_key" if status == 401 else "fixture_failure", "message": "Synthetic controlled failure"}}).encode(), "application/json"
        self.send_response(status)
        self.send_header("Content-Type", kind)
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        try:
            self.wfile.write(payload)
        except (BrokenPipeError, ConnectionResetError):
            pass


def run(candidate, repo, evidence):
    server = ThreadingHTTPServer(("127.0.0.1", 0), Endpoint)
    server.requests, server.status, server.lock = [], 200, threading.Lock()
    server.started, server.release = threading.Event(), threading.Event()
    threading.Thread(target=server.serve_forever, daemon=True).start()
    result = {"passed": False, "checks": []}
    with tempfile.TemporaryDirectory(prefix="fj-", dir=os.environ.get("TMPDIR")) as tmp:
        session = Session(candidate, repo, tmp, evidence)
        home = session.home
        config_path = home / "config.toml"
        uri = f"http://127.0.0.1:{server.server_port}"
        config = f'model="gpt-5.6-sol"\nmodel_provider="openai"\nmodel_reasoning_effort="low"\nopenai_base_url="{uri}/v1"\n'
        config += base_config(repo)
        for suffix in ("a", "b"):
            config += f'\n[model_providers.qa-{suffix}]\nname="QA Route {suffix.upper()}"\nbase_url="{uri}/{suffix}"\nenv_key="QA_{suffix.upper()}_KEY"\nwire_api="responses"\nrequest_max_retries=0\nstream_max_retries=0\n'
        config_path.write_text(config)
        (home / "auth.json").write_text(json.dumps({"OPENAI_API_KEY": "synthetic-openai"}))

        def check(name):
            session.save(name)
            result["checks"].append(name)

        def request(text):
            before = len(server.requests)
            session.submit(text)
            session.wait(f"JOURNEY_REPLY_{before + 1}")
            assert len(server.requests) == before + 1, "unexpected duplicate request"
            return server.requests[-1]

        def setup(label, key):
            session.manager()
            session.choose(label)
            session.choose("Set up with API key")
            session.wait("API key — masked")
            session.text(key, secret=True)
            session.wait("••")
            session.key("Enter")
            session.wait("Configure providers and control")
            session.choose(label, confirm=False)
            session.wait("Enabled · configured")
            session.key("Escape")

        try:
            session.start()
            session.manager()
            session.choose("QA Route B", confirm=False)
            session.save("unconfigured-last-provider")
            assert "Not configured" in session.view()
            session.key("Escape")
            for _ in range(3):
                session.picker()
                session.key("Escape")
                session.manager()
                session.key("Escape")
            assert config_path.read_text() == config
            assert not server.requests, "menu browsing issued inference"
            check("F01-F25-inventory-cancel")

            setup("QA Route A", "synthetic-a")
            setup("QA Route B", "synthetic-b")
            assert tomllib.loads(config_path.read_text())["model_provider"] == "openai"
            before = config_path.read_bytes()
            session.picker("OpenAI")
            session.choose("GPT-6 Astra")
            session.wait("Select Reasoning")
            session.key("Escape")
            session.key("Escape")
            assert config_path.read_bytes() == before
            assert request("Confirm cancellation leaves the selected route intact")["model"] == "gpt-5.6-sol"
            check("F17-selection-cancel-request")

            for model, label, effort in [("gpt-6-astra", "GPT-6 Astra", "High"), ("gpt-5.6-sol", "GPT-5.6-Sol", "Low")]:
                session.picker("OpenAI")
                session.choose(label)
                session.wait("Select Reasoning")
                session.choose(effort)
                session.wait("Model changed to")
                state = tomllib.loads(config_path.read_text())
                assert (state["model"], state["model_reasoning_effort"]) == (model, effort.lower())
                sent = request("Prove selected model and effort " + effort)
                assert (sent["model"], sent["effort"], sent["auth_route"]) == (model, effort.lower(), "openai")
                session.picker("OpenAI")
                assert session.view().count("(current)") == 1
                session.key("Escape")
            check("F15-F16-effort-request-parity")

            session.manager()
            session.choose("QA Route B")
            session.choose("Deactivate")
            session.wait("Inactive")
            session.key("Escape")
            for cycle in range(2):
                session.exit()
                session.start()
                session.manager()
                session.choose("QA Route B", confirm=False)
                session.wait("Inactive")
                session.key("Escape")
                assert request("Restart persistence request " + str(cycle))["auth_route"] == "openai"
            check("F18-persisted-disabled-restart-twice")
            session.manager()
            session.choose("QA Route B")
            session.choose("Reactivate")
            session.wait("Enabled · configured")
            session.key("Escape")
            check("F14-reactivation-without-setup")

            for size in [(80, 24), (40, 18), (140, 44)]:
                session.manager()
                session.tmux("resize-window", "-t", "qa", "-x", str(size[0]), "-y", str(size[1]))
                session.choose("QA Route B", confirm=False)
                session.save(f"F24-manager-{size[0]}x{size[1]}")
                session.key("Escape")
                session.tmux("resize-window", "-t", "qa", "-x", "140", "-y", "44")
            assert request("After resize, route and composer remain usable")["auth_route"] == "openai"
            check("F24-resize-last-row-recovery")

            session.attach_cycle()
            assert request("After first detach")["auth_route"] == "openai"
            session.attach_cycle()
            assert request("After second detach")["auth_route"] == "openai"
            check("F21-attach-detach-reattach")

            # An actual late response arrives while a different provider row is selected.
            session.submit("DELAY_FIXTURE controlled late response")
            assert server.started.wait(15)
            session.manager()
            session.choose("QA Route B", confirm=False)
            server.release.set()
            session.wait("JOURNEY_REPLY_" + str(len(server.requests)))
            assert "QA Route B" in [line for line in session.view().splitlines() if line.lstrip().startswith((">", "›"))][-1]
            session.key("Escape")
            check("F26-late-response-keeps-menu-selection")

            session.exit()
            # Separate established custom-route fixture: selection is seeded
            # before startup, but rejection/cancel/replacement happen in one process.
            current = config_path.read_text().replace('model_provider = "openai"', 'model_provider = "qa-a"').replace('model_provider="openai"', 'model_provider="qa-a"')
            current = current.replace('model = "gpt-5.6-sol"', 'model = "fixture-model"').replace('model="gpt-5.6-sol"', 'model="fixture-model"')
            config_path.write_text(current)
            session.start()
            for status in (403, 429, 500):
                server.status = status
                count = len(server.requests)
                session.submit("Controlled service failure " + str(status))
                session.wait({403: "unexpected status 403 Forbidden",
                              429: "rate limited by provider",
                              500: "We're currently experiencing high demand"}[status])
                session.save(f"service-error-{status}")
                session.manager()
                session.choose("QA Route A", confirm=False)
                assert "Credential needs attention" not in session.view(), "non-auth failure poisoned credential health"
                session.key("Escape")
                server.status = 200
                assert request("Retry after service restoration " + str(status))["auth_route"] == "a"
                assert len(server.requests) == count + 2
            check("F23-service-failure-retry-isolation")
            server.status = 401
            session.submit("Controlled rejected credential")
            session.wait("QA Route A (qa-a) credential was rejected")
            session.manager()
            session.choose("QA Route A", confirm=False)
            session.key("r")
            session.choose("Replace the saved API key")
            session.wait("API key — masked")
            session.text("synthetic-cancelled", secret=True)
            session.key("Escape")
            session.wait("Configure providers and control")
            session.choose("QA Route A", confirm=False)
            session.key("r")
            session.choose("Replace the saved API key")
            session.wait("API key — masked")
            session.text("synthetic-a2", secret=True)
            session.key("Enter")
            session.wait("Configure providers and control")
            session.wait("Enabled · configured")
            session.key("Escape")
            server.status = 200
            assert request("Continue after explicit credential repair")["auth_route"] == "a2"
            check("F08-F12-rejection-cancel-repair-no-duplicate")
            session.manager()
            session.choose("QA Route B", confirm=False)
            session.key("r")
            session.choose("Replace the saved API key")
            session.wait("API key — masked")
            session.text("synthetic-b2", secret=True)
            session.key("Enter")
            session.wait("Configure providers and control")
            session.wait("Enabled · configured")
            session.key("Escape")
            assert request("Nonselected repair must not change current routing")["auth_route"] == "a2"
            session.picker("Other")
            session.save("custom-route-picker")
            session.choose("fixture-model via qa-b")
            # Fallback custom models have no configurable reasoning choices.
            session.wait("Model changed to")
            sent = request("Use the explicitly selected repaired route")
            assert (sent["auth_route"], sent["effort"]) == ("b2", "none")
            assert tomllib.loads(config_path.read_text())["model_provider"] == "qa-b"
            session.picker("Other")
            assert session.view().count("(current)") == 1
            assert "fixture-model via qa-b (current)" in session.view()
            session.key("Escape")
            check("F13-nonselected-repair-both-routes-usable")
            session.manager()
            session.choose("QA Route B")
            session.choose("Deactivate")
            session.wait("Choose replacement")
            session.key("Escape")
            session.key("Escape")
            assert request("Cancel current-provider deactivation")["auth_route"] == "b2"
            session.manager()
            session.choose("QA Route B")
            session.choose("Deactivate")
            session.wait("Choose replacement")
            session.choose("QA Route A — fixture-model")
            session.wait("Configure providers and control")
            session.key("Escape")
            assert request("Explicit replacement of disabled current provider")["auth_route"] == "a2"
            session.manager()
            session.choose("QA Route B")
            session.choose("Reactivate")
            session.wait("Enabled · configured")
            session.key("Escape")
            session.picker("Other")
            session.choose("fixture-model via qa-b")
            session.wait("Model changed to")
            assert request("Reactivated route retains repaired credential")["auth_route"] == "b2"
            check("F14-current-disable-cancel-explicit-replacement-reactivate")
            session.exit()
            result["passed"] = True
        finally:
            server.release.set()
            session.close()
            for path in evidence.glob("*.txt"):
                assert not any(secret in path.read_text() for secret in ("synthetic-openai", "synthetic-a", "synthetic-b", "synthetic-cancelled")), "secret in visible evidence"
            session.receipt(result)
            (evidence / "requests.json").write_text(json.dumps(server.requests, indent=2))
            server.shutdown()
            server.server_close()
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(run(args.candidate.resolve(), args.repo.resolve(), args.evidence.resolve())))

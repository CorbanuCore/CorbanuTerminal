"""Canonical-origin Apps expiry/recovery through an isolated TLS fixture.

No host/DNS/system trust changes and no live credentials. A process-local proxy
accepts only chatgpt.com:443 and auth.openai.com:443 and never forwards traffic. Only this synthetic TUI
trusts the disposable CA, using the existing custom-CA setting.
"""
import argparse
import base64
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import ssl
import subprocess
import tempfile
import threading

from functional_harness import Session, base_config, sse


def jwt():
    claims = {"email": "pf58@example.invalid", "https://api.openai.com/auth": {
        "chatgpt_plan_type": "pro", "chatgpt_account_id": "pf58-account"}}
    enc = lambda obj: base64.urlsafe_b64encode(json.dumps(obj).encode()).decode().rstrip("=")
    return enc({"alg": "none"}) + "." + enc(claims) + ".c2lnbmF0dXJl"


class Service(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_):
        pass

    def handle(self):
        try:
            super().handle()
        except (BrokenPipeError, ConnectionResetError, ssl.SSLEOFError) as error:
            # Discovery may cancel unused connections. Keep this observable,
            # without treating a disconnect as a successful Apps/tool response.
            self.server.state.setdefault("client_disconnects", []).append(type(error).__name__)

    def reply(self, status, value, kind="application/json"):
        data = value if isinstance(value, bytes) else json.dumps(value).encode()
        self.send_response(status)
        self.send_header("Content-Type", kind)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_CONNECT(self):
        state = self.server.state
        state["connects"].append(self.path)
        if self.path not in ("chatgpt.com:443", "auth.openai.com:443"):
            self.reply(403, {"error": "fixture denies non-canonical CONNECT"})
            return
        self.send_response(200)
        self.end_headers()
        try:
            self.connection = self.server.tls.wrap_socket(self.connection, server_side=True)
        except ssl.SSLError as error:
            state.setdefault("tls_errors", []).append(type(error).__name__)
            self.close_connection = True
            return
        self.rfile = self.connection.makefile("rb", -1)
        self.wfile = self.connection.makefile("wb", 0)
        self.close_connection = False

    def do_GET(self):
        self.reply(404, {"fixture": "not found"})

    def do_DELETE(self):
        self.reply(200, {})

    def do_POST(self):
        raw = self.rfile.read(int(self.headers.get("Content-Length", 0)))
        try:
            body = json.loads(raw)
        except ValueError:
            body = {}
        state = self.server.state
        auth = self.headers.get("Authorization", "")
        if self.path.endswith("/ps/mcp"):
            label = "old" if auth == "Bearer synthetic-old-account" else "new" if auth == "Bearer synthetic-new-account" else "missing-or-other"
            state["apps"].append({"credential": label, "method": body.get("method"), "host": self.headers.get("Host")})
            if label == "old":
                self.reply(401, {"error": {"code": "token_expired", "message": "Synthetic account token expired"}})
                return
            if label != "new":
                self.reply(401, {"error": {"code": "invalid_api_key"}})
                return
            if "id" not in body:
                self.reply(202, b"")
                return
            method = body["method"]
            if method == "initialize":
                result = {"protocolVersion": body["params"]["protocolVersion"], "capabilities": {"tools": {}}, "serverInfo": {"name": "pf58-apps", "version": "1"}}
            elif method == "tools/list":
                result = {"tools": [{"name": "pf58_echo", "description": "Synthetic echo", "inputSchema": {"type": "object", "properties": {}}, "annotations": {"readOnlyHint": True}, "_meta": {"connector_id": "pf58", "connector_name": "PF58", "connector_description": "Synthetic qualification", "link_id": "fixture-link", "_codex_apps": {"resource_uri": "connector://pf58/tools/pf58_echo", "contains_mcp_source": True, "connector_id": "pf58"}}}]}
            elif method == "tools/call":
                state["tool_called"] = True
                result = {"content": [{"type": "text", "text": "TRUSTED_MCP_ECHO"}]}
            else:
                result = {}
            self.reply(200, {"jsonrpc": "2.0", "id": body["id"], "result": result})
        elif self.path == "/api/accounts/deviceauth/usercode":
            self.reply(200, {"device_auth_id": "pf58-device", "user_code": "PF58-SYNTHETIC", "interval": "1"})
        elif self.path == "/api/accounts/deviceauth/token":
            if not state["allow_login"]:
                self.reply(403, {})
            else:
                self.reply(200, {"authorization_code": "fixture-code", "code_challenge": "fixture-challenge", "code_verifier": "fixture-verifier"})
        elif self.path == "/oauth/token":
            if not state["allow_login"]:
                self.reply(401, {"error": {"code": "refresh_token_reused"}})
            else:
                self.reply(200, {"access_token": "synthetic-new-account", "refresh_token": "synthetic-new-refresh", "id_token": jwt(), "token_type": "Bearer", "expires_in": 3600})
        elif self.path.endswith("/responses"):
            assert auth == "Bearer synthetic-healthy-model", "model credential route changed"
            state["model_requests"] += 1
            users = [item for item in body.get("input", []) if item.get("role") == "user"]
            latest = json.dumps(users[-1]) if users else ""
            outputs = [item for item in body.get("input", []) if item.get("type") == "custom_tool_call_output"]
            if state["model_requests"] <= 4:
                state.setdefault("model_diagnostics", []).append({"tool_names": [t.get("name", t.get("function", {}).get("name")) for t in body.get("tools", [])], "outputs": [i for i in body.get("input", []) if i.get("type") in ("custom_tool_call_output", "function_call_output")]})
            if state["model_requests"] > 8:
                self.reply(200, sse("FIXTURE_TOOL_LOOP_STOPPED"), "text/event-stream")
                return
            if "Exercise recovered tool" not in latest:
                self.reply(200, sse("UNRELATED_CHAT_OK"), "text/event-stream")
            elif outputs and "TRUSTED_MCP_ECHO" in json.dumps(outputs):
                self.reply(200, sse("TRUSTED_APPS_TOOL_OK"), "text/event-stream")
            else:
                code = 'const t=ALL_TOOLS.find(t=>t.name.includes("codex_apps") && t.name.endsWith("echo")); if(!t) throw new Error("fixture tool missing"); text(await tools[t.name]({}));'
                events = [{"type": "response.created", "response": {"id": "apps-response"}},
                    {"type": "response.output_item.done", "item": {"type": "custom_tool_call", "name": "exec", "call_id": "apps-call", "input": code}},
                    {"type": "response.completed", "response": {"id": "apps-response", "usage": {"input_tokens": 0, "output_tokens": 0, "total_tokens": 0}}}]
                self.reply(200, "".join("data: " + json.dumps(e) + "\n\n" for e in events).encode(), "text/event-stream")
        else:
            self.reply(404, {"fixture": "not found"})


def certificates(root):
    def openssl(*args):
        subprocess.run(["openssl", *args], check=True, capture_output=True, timeout=30, cwd=root)
    openssl("req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", "ca.key", "-out", "ca.pem", "-days", "1", "-subj", "/CN=PF58 disposable test CA", "-addext", "basicConstraints=critical,CA:TRUE")
    openssl("req", "-newkey", "rsa:2048", "-nodes", "-keyout", "leaf.key", "-out", "leaf.csr", "-subj", "/CN=chatgpt.com")
    (root / "leaf.ext").write_text("subjectAltName=DNS:chatgpt.com,DNS:auth.openai.com\nbasicConstraints=critical,CA:FALSE\nkeyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n")
    openssl("x509", "-req", "-in", "leaf.csr", "-CA", "ca.pem", "-CAkey", "ca.key", "-CAcreateserial", "-out", "leaf.pem", "-days", "1", "-extfile", "leaf.ext")


def run(candidate, repo, evidence):
    result = {"passed": False, "canonical_origin": "https://chatgpt.com", "synthetic_tls_fixture": True}
    with tempfile.TemporaryDirectory(prefix="ta-", dir=os.environ.get("TMPDIR")) as tmp:
        root = Path(tmp)
        certificates(root)
        state = {"connects": [], "apps": [], "allow_login": False, "tool_called": False, "model_requests": 0}
        server = ThreadingHTTPServer(("127.0.0.1", 0), Service)
        server.state = state
        server.tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        server.tls.load_cert_chain(root / "leaf.pem", root / "leaf.key")
        threading.Thread(target=server.serve_forever, daemon=True).start()
        uri = f"http://127.0.0.1:{server.server_port}"
        env = {"HTTPS_PROXY": uri, "https_proxy": uri, "NO_PROXY": "127.0.0.1,localhost", "no_proxy": "127.0.0.1,localhost",
               "CODEX_CA_CERTIFICATE": str(root / "ca.pem"), "SSL_CERT_FILE": str(root / "ca.pem"),
               "CODEX_APP_SERVER_LOGIN_ISSUER": uri, "CODEX_REFRESH_TOKEN_URL_OVERRIDE": uri + "/oauth/token", "QA_HEALTHY_KEY": "synthetic-healthy-model"}
        session = Session(candidate, repo, root, evidence, env)
        config = 'model="gpt-6-astra"\nmodel_provider="qa-healthy"\n' + base_config(repo)
        config += f'\n[model_providers.qa-healthy]\nname="QA Healthy"\nbase_url="{uri}/v1"\nenv_key="QA_HEALTHY_KEY"\nwire_api="responses"\nrequest_max_retries=0\nstream_max_retries=0\n[features]\napps=true\ncode_mode=true\ncode_mode_host=true\n'
        (session.home / "config.toml").write_text(config)
        (session.home / "auth.json").write_text(json.dumps({"auth_mode": "chatgpt", "tokens": {"id_token": jwt(), "access_token": "synthetic-old-account", "refresh_token": "synthetic-old-refresh", "account_id": "pf58-account"}, "last_refresh": datetime.now(timezone.utc).isoformat()}))
        try:
            session.start()
            session.wait("OpenAI account authentication for codex_apps was rejected")
            session.save("apps-rejected")
            session.submit("Verify unrelated healthy model")
            session.wait("UNRELATED_CHAT_OK")
            session.manager()
            session.choose("OpenAI", confirm=False)
            session.wait("Credential needs attention")
            session.key("r")
            session.choose("Sign in to OpenAI again")
            session.wait("OpenAI account login")
            session.save("account-recovery")
            state["allow_login"] = True
            session.wait("Configure providers and control")
            session.wait("Enabled · configured")
            session.key("Escape")
            # Discovery and initialize are completion prerequisites, not a fixed delay.
            import time
            deadline = time.monotonic() + 30
            while not any(r["credential"] == "new" and r["method"] == "tools/list" for r in state["apps"]):
                assert time.monotonic() < deadline, "Apps did not reconnect with new token"
                time.sleep(.15)
            session.submit("Exercise recovered tool")
            session.wait("TRUSTED_APPS_TOOL_OK")
            session.save("recovered-tool")
            assert state["tool_called"]
            import tomllib
            assert tomllib.loads((session.home / "config.toml").read_text())["model_provider"] == "qa-healthy"
            result.update(passed=True, same_process_recovery=True, unrelated_model_unchanged=True)
            session.exit()
        finally:
            session.close()
            session.receipt(result)
            (evidence / "service-requests.json").write_text(json.dumps(state, indent=2))
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

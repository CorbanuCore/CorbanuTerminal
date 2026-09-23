#!/usr/bin/env python3
"""OpenRouter relay for leak-isolated benchmark contestants.

Contestant containers have no internet route. Their ``/etc/hosts`` maps
``openrouter.ai`` to this relay, which presents a benchmark-CA certificate for
that name, so every harness uses its unmodified native OpenRouter integration.
The relay:

* accepts only per-run tokens registered by the runner (the real key never
  enters a contestant container);
* forwards only inference and read-only catalogue endpoints;
* requires the exact frozen model and rejects server-side web tools, so neither
  fallback models nor OpenRouter web search can reach the public task sources;
* records every request and response body, OpenRouter generation id, and usage
  under ``<records>/runs/<run_id>/``.

Request bodies are forwarded byte-for-byte. Standard library only.
"""

from __future__ import annotations

import argparse
import hashlib
import http.client
import json
import os
import ssl
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

UPSTREAM_HOST = "openrouter.ai"
INFERENCE_PATHS = {"/api/v1/chat/completions", "/api/v1/messages", "/api/v1/responses"}
READ_ONLY_PREFIXES = ("/api/v1/models", "/api/v1/key", "/api/v1/auth/key", "/api/v1/providers")
PUBLIC_PREFIXES = ("/api/v1/models", "/api/v1/providers")
FORWARD_HEADERS = (
    "content-type",
    "accept",
    "user-agent",
    "http-referer",
    "x-title",
    "anthropic-version",
    "anthropic-beta",
)
WEB_PLUGIN_IDS = {"web", "web_search", "web-search"}
MAX_BODY = 32 * 1024 * 1024
LOCK = threading.Lock()


def append_jsonl(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with LOCK, path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(value, sort_keys=True) + "\n")


def server_side_web_access(body: dict[str, Any], model: str) -> str | None:
    """Return why a request could fetch internet content through OpenRouter."""

    if model.endswith(":online"):
        return "online model variant"
    if body.get("web_search_options") is not None:
        return "web_search_options"
    for plugin in body.get("plugins") or []:
        if isinstance(plugin, dict) and str(plugin.get("id", "")).lower() in WEB_PLUGIN_IDS:
            return f"plugin {plugin.get('id')}"
    for tool in body.get("tools") or []:
        kind = str(tool.get("type", "")).lower() if isinstance(tool, dict) else ""
        if kind not in {"", "function", "custom"}:
            return f"server tool {kind}"
    return None


def response_facts(raw: bytes, content_type: str) -> dict[str, Any]:
    """Generation ids, usage, and served models from a JSON or SSE response."""

    values: list[Any] = []
    text = raw.decode("utf-8", errors="replace")
    if "text/event-stream" in content_type:
        for line in text.splitlines():
            if line.startswith("data:"):
                payload = line[5:].strip()
                if payload and payload != "[DONE]":
                    try:
                        values.append(json.loads(payload))
                    except ValueError:
                        continue
    else:
        try:
            values.append(json.loads(text))
        except ValueError:
            pass
    ids: list[str] = []
    models: set[str] = set()
    usage = None
    for value in values:
        if not isinstance(value, dict):
            continue
        message = value.get("message") if isinstance(value.get("message"), dict) else {}
        response = value.get("response") if isinstance(value.get("response"), dict) else {}
        for candidate in (value.get("id"), message.get("id"), response.get("id")):
            if isinstance(candidate, str) and candidate and candidate not in ids:
                ids.append(candidate)
        for candidate in (value.get("model"), message.get("model"), response.get("model")):
            if isinstance(candidate, str):
                models.add(candidate)
        for candidate in (value.get("usage"), message.get("usage"), response.get("usage")):
            if isinstance(candidate, dict) and candidate:
                usage = candidate
    return {"generation_ids": ids, "observed_models": sorted(models), "usage": usage}


class Relay(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    records: Path
    api_key: str

    def log_message(self, *_args: Any) -> None:
        return

    def registration(self) -> dict[str, Any] | None:
        header = self.headers.get("Authorization", "")
        token = header[7:] if header.startswith("Bearer ") else self.headers.get("x-api-key", "")
        if not token:
            return None
        digest = hashlib.sha256(token.encode()).hexdigest()
        path = self.records / "registrations" / f"{digest}.json"
        if not path.is_file():
            return None
        reg = json.loads(path.read_text(encoding="utf-8"))
        return reg if time.time() <= float(reg["expires_at"]) else None

    def reply(self, status: int, message: str) -> None:
        data = json.dumps({"error": {"code": status, "message": message}}).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def audit(self, reg: dict[str, Any] | None, event: dict[str, Any]) -> None:
        run = reg["run_id"] if reg else "_unregistered"
        append_jsonl(
            self.records / "runs" / run / "events.jsonl",
            {"time": time.time(), "method": self.command, "path": self.path, **event},
        )

    def do_GET(self) -> None:
        reg = self.registration()
        path = self.path.split("?", 1)[0]
        if reg is None and path.startswith(PUBLIC_PREFIXES):
            # The public model catalogue needs no credential; forward it unauthenticated.
            self.audit(None, {"decision": "forwarded_public"})
            return self.forward(None, b"", request_id=None)
        if reg is None:
            self.audit(None, {"decision": "rejected", "reason": "unregistered"})
            return self.reply(401, "unregistered benchmark run")
        if not path.startswith(READ_ONLY_PREFIXES):
            self.audit(reg, {"decision": "rejected", "reason": "endpoint not allowed"})
            return self.reply(403, "endpoint not allowed in benchmark")
        self.audit(reg, {"decision": "forwarded"})
        self.forward(reg, b"", request_id=None)

    def do_POST(self) -> None:
        reg = self.registration()
        path = self.path.split("?", 1)[0]
        if reg is None:
            self.audit(None, {"decision": "rejected", "reason": "unregistered"})
            return self.reply(401, "unregistered benchmark run")
        if path not in INFERENCE_PATHS:
            self.audit(reg, {"decision": "rejected", "reason": "endpoint not allowed"})
            return self.reply(403, "endpoint not allowed in benchmark")
        size = int(self.headers.get("Content-Length") or 0)
        if not 0 < size <= MAX_BODY:
            self.audit(reg, {"decision": "rejected", "reason": "missing or oversized body"})
            return self.reply(413, "request body must have a length")
        raw = self.rfile.read(size)
        try:
            body = json.loads(raw)
        except ValueError:
            self.audit(reg, {"decision": "rejected", "reason": "invalid json"})
            return self.reply(400, "invalid json")
        model = str(body.get("model") or "")
        if model != reg["model"]:
            self.audit(reg, {"decision": "rejected", "reason": "model mismatch", "model": model})
            return self.reply(400, f"benchmark requires exactly {reg['model']}")
        web = server_side_web_access(body, model)
        if web:
            self.audit(reg, {"decision": "rejected", "reason": f"server-side web access: {web}"})
            return self.reply(400, "server-side web access is disabled in this benchmark")
        request_id = f"{time.time_ns()}"
        run_dir = self.records / "runs" / reg["run_id"]
        run_dir.mkdir(parents=True, exist_ok=True)
        (run_dir / f"{request_id}.request.json").write_bytes(raw)
        self.audit(reg, {"decision": "forwarded", "request_id": request_id, "model": model,
                         "request_sha256": hashlib.sha256(raw).hexdigest()})
        self.forward(reg, raw, request_id=request_id)

    def forward(self, reg: dict[str, Any] | None, raw: bytes, *, request_id: str | None) -> None:
        headers = {k: v for k, v in self.headers.items() if k.lower() in FORWARD_HEADERS}
        if reg is not None:
            headers["Authorization"] = f"Bearer {self.api_key}"
            if self.headers.get("x-api-key"):
                headers["x-api-key"] = self.api_key
        if raw:
            headers["Content-Length"] = str(len(raw))
        started = time.monotonic()
        connection = http.client.HTTPSConnection(UPSTREAM_HOST, timeout=900)
        collected = bytearray()
        status = None
        content_type = ""
        try:
            connection.request(self.command, self.path, body=raw or None, headers=headers)
            upstream = connection.getresponse()
            status = upstream.status
            content_type = upstream.getheader("Content-Type", "")
            self.send_response(upstream.status)
            self.send_header("Content-Type", content_type or "application/json")
            self.send_header("Transfer-Encoding", "chunked")
            self.end_headers()
            client_open = True
            while True:
                chunk = upstream.read1(65536)
                if not chunk:
                    break
                collected.extend(chunk)
                if client_open:
                    try:
                        self.wfile.write(f"{len(chunk):x}\r\n".encode() + chunk + b"\r\n")
                        self.wfile.flush()
                    except (BrokenPipeError, ConnectionResetError):
                        client_open = False
            if client_open:
                self.wfile.write(b"0\r\n\r\n")
                self.wfile.flush()
        except Exception as exc:  # recorded; the client sees a dropped stream
            self.audit(reg, {"decision": "upstream_error", "request_id": request_id,
                             "error_type": type(exc).__name__})
            self.close_connection = True
        finally:
            connection.close()
        if request_id is None:
            return
        run_dir = self.records / "runs" / reg["run_id"]
        (run_dir / f"{request_id}.response").write_bytes(bytes(collected))
        append_jsonl(
            run_dir / "usage.jsonl",
            {
                "request_id": request_id,
                "status": status,
                "seconds": round(time.monotonic() - started, 3),
                "model_requested": reg["model"],
                **response_facts(bytes(collected), content_type),
            },
        )


class TLSRelayServer(ThreadingHTTPServer):
    """Handshake per connection inside the worker thread, never in accept()."""

    daemon_threads = True
    context: ssl.SSLContext

    def finish_request(self, request: Any, client_address: Any) -> None:
        try:
            tls = self.context.wrap_socket(request, server_side=True)
        except (ssl.SSLError, OSError):
            return
        self.RequestHandlerClass(tls, client_address, self)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--records", type=Path, required=True)
    parser.add_argument("--cert", type=Path, required=True)
    parser.add_argument("--key", type=Path, required=True)
    parser.add_argument("--port", type=int, default=443)
    args = parser.parse_args()
    api_key = os.environ.pop("OPENROUTER_API_KEY", "").strip()
    if not api_key:
        raise SystemExit("OPENROUTER_API_KEY is required by the relay")
    Relay.records = args.records
    Relay.api_key = api_key
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(args.cert, args.key)
    context.set_alpn_protocols(["http/1.1"])
    server = TLSRelayServer(("0.0.0.0", args.port), Relay)
    server.context = context
    server.serve_forever()


if __name__ == "__main__":
    main()

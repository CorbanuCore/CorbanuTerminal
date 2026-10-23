#!/usr/bin/env python3
"""Independent S05 acceptance helper (written code-blind).

proxy mode: forward every request to UPSTREAM, stream the reply back, and log one
JSON line per request with the path, status, non-secret headers and every
provider-reported `usage` object found in the reply. Authorization / api-key
headers are never logged; only a sha256 prefix of the bearer (`auth_h`) is kept.

mock mode: an OpenAI-compatible Chat Completions stub. A bearer listed in
MOCK_429 gets HTTP 429 with Retry-After; every other bearer gets a short
streamed "pong" with fixed usage. Bearers are synthetic canaries; the log keeps
the label mapped from MOCK_LABELS ("label=value,...") or `other`.
"""
import hashlib, hmac, http.server, json, os, socketserver, sys, threading, time, urllib.request, urllib.error

MODE, PORT, LOG = sys.argv[1], int(sys.argv[2]), sys.argv[3]
UPSTREAM = os.environ.get("UPSTREAM", "").rstrip("/")
LOCK = threading.Lock()
SECRET_HEADERS = {"authorization", "api-key", "x-api-key", "cookie"}
LABELS = dict(kv.split("=", 1) for kv in os.environ.get("MOCK_LABELS", "").split(",") if "=" in kv)
LABEL_OF = {v: k for k, v in LABELS.items()}
MOCK_429 = set(x for x in os.environ.get("MOCK_429", "").split(",") if x)
RETRY_AFTER = os.environ.get("MOCK_RETRY_AFTER", "120")
RUN_SALT = os.urandom(16)  # per process: auth_h groups bearers within one run, never across runs or machines


def log(rec):
    rec["ts"] = round(time.time(), 3)
    with LOCK, open(LOG, "a") as f:
        f.write(json.dumps(rec, sort_keys=True) + "\n")


def bearer(h):
    v = h.get("Authorization") or h.get("authorization") or ""
    return v[7:] if v.lower().startswith("bearer ") else v


def usages(blob):
    out = []
    for line in blob.decode("utf-8", "replace").splitlines():
        line = line.strip()
        if line.startswith("data:"):
            line = line[5:].strip()
        if not line.startswith("{"):
            continue
        try:
            obj = json.loads(line)
        except ValueError:
            continue
        u = obj.get("usage") or (obj.get("response") or {}).get("usage")
        if u:
            out.append(u)
    return out


class H(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *a):
        pass

    def _headers(self):
        return {k: v for k, v in self.headers.items() if k.lower() not in SECRET_HEADERS}

    def _body(self):
        n = int(self.headers.get("Content-Length") or 0)
        return self.rfile.read(n) if n else b""

    def do_GET(self):
        self.handle_any("GET")

    def do_POST(self):
        self.handle_any("POST")

    def handle_any(self, method):
        body = self._body()
        b = bearer(self.headers)
        rec = {"method": method, "path": self.path, "headers": self._headers(),
               "auth_h": hmac.new(RUN_SALT, b.encode(), hashlib.sha256).hexdigest()[:10] if b else None}
        try:
            req = json.loads(body) if body else {}
            rec["model"] = req.get("model")
            rec["stream_options"] = req.get("stream_options")
        except ValueError:
            pass
        if MODE == "mock":
            return self.mock(rec, b)
        return self.forward(rec, method, body)

    def mock(self, rec, b):
        rec["label"] = LABEL_OF.get(b, "other" if b else "none")
        if self.path.endswith("/models"):
            rec["status"] = 200
            log(rec)
            return self.send_json(200, {"object": "list", "data": []})
        if b in MOCK_429:
            rec["status"] = 429
            log(rec)
            body = json.loads(os.environ.get("MOCK_429_BODY") or '{"error": {"message": "rate limited (mock)", "type": "rate_limit_error"}}')
            return self.send_json(429, body,
                                  {"Retry-After": RETRY_AFTER})
        slow = dict(kv.split(":") for kv in os.environ.get("MOCK_SLOW", "").split(",") if ":" in kv)
        if rec["label"] in slow:
            rec["arrived"] = round(time.time(), 3)
            time.sleep(float(slow[rec["label"]]))
        rec["status"] = 200
        usage = {"prompt_tokens": 1000, "completion_tokens": 7, "total_tokens": 1007}
        rec["usage"] = [usage]
        log(rec)
        chunks = [
            {"id": "m1", "object": "chat.completion.chunk", "created": 0, "model": "mock-model",
             "choices": [{"index": 0, "delta": {"role": "assistant", "content": "pong"}, "finish_reason": None}]},
            {"id": "m1", "object": "chat.completion.chunk", "created": 0, "model": "mock-model",
             "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}]},
            {"id": "m1", "object": "chat.completion.chunk", "created": 0, "model": "mock-model",
             "choices": [], "usage": usage},
        ]
        data = b"".join(b"data: " + json.dumps(c).encode() + b"\n\n" for c in chunks) + b"data: [DONE]\n\n"
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def send_json(self, code, obj, extra=None):
        data = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        for k, v in (extra or {}).items():
            self.send_header(k, v)
        self.end_headers()
        self.wfile.write(data)

    def forward(self, rec, method, body):
        url = UPSTREAM + self.path
        hdrs = {k: v for k, v in self.headers.items() if k.lower() not in {"host", "content-length", "accept-encoding", "connection"}}
        req = urllib.request.Request(url, data=body if method == "POST" else None, headers=hdrs, method=method)
        buf = b""
        try:
            resp = urllib.request.urlopen(req, timeout=600)
            status, rh = resp.status, resp.headers
        except urllib.error.HTTPError as e:
            resp, status, rh = e, e.code, e.headers
        self.send_response(status)
        for k, v in rh.items():
            if k.lower() not in {"transfer-encoding", "connection", "content-length", "content-encoding"}:
                self.send_header(k, v)
        self.send_header("Transfer-Encoding", "chunked")
        self.end_headers()
        rec["client_aborted"] = False
        try:
            while True:
                chunk = resp.read1(65536) if hasattr(resp, "read1") else resp.read(65536)
                if not chunk:
                    break
                buf += chunk
                self.wfile.write(b"%x\r\n%s\r\n" % (len(chunk), chunk))
                self.wfile.flush()
            self.wfile.write(b"0\r\n\r\n")
        except OSError:
            rec["client_aborted"] = True
        rec["bytes_relayed"] = len(buf)
        rec["status"] = status
        rec["usage"] = usages(buf)
        if status >= 400:
            rec["error_excerpt"] = buf[:300].decode("utf-8", "replace")
        log(rec)


class S(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True


if __name__ == "__main__":
    S(("127.0.0.1", PORT), H).serve_forever()

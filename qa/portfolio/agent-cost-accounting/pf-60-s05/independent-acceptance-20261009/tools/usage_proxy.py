#!/usr/bin/env python3
"""Logging pass-through reverse proxy (stdlib only). Adapted from the S03 acceptance tool.

Forwards http://127.0.0.1:PORT/<path> to https://UPSTREAM/<path>, streams the response back unchanged and appends one
JSON line per request to LOG: path, status, model, provider response ids and every `usage` object seen in SSE chunks
(Chat Completions `usage`, Anthropic `message.usage` / `usage`). Never logs headers or bodies.
"""
import argparse, http.client, http.server, json, ssl, threading, time

lock = threading.Lock()


def usages_of(obj):
    out = []
    if isinstance(obj.get("usage"), dict):
        out.append(obj["usage"])
    msg = obj.get("message")
    if isinstance(msg, dict) and isinstance(msg.get("usage"), dict):
        out.append(msg["usage"])
    resp = obj.get("response")
    if isinstance(resp, dict) and isinstance(resp.get("usage"), dict):
        out.append(resp["usage"])
    return out


def ids_of(obj):
    ids = []
    for o in (obj, obj.get("message") or {}, obj.get("response") or {}):
        if isinstance(o, dict) and isinstance(o.get("id"), str):
            ids.append(o["id"])
    return ids


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, required=True)
    ap.add_argument("--upstream", required=True)
    ap.add_argument("--log", required=True)
    args = ap.parse_args()

    class H(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, *a):
            pass

        def do_POST(self):
            n = int(self.headers.get("content-length", 0))
            body = self.rfile.read(n)
            try:
                req = json.loads(body)
            except Exception:
                req = {}
            hdrs = {k: v for k, v in self.headers.items()
                    if k.lower() not in ("host", "content-length", "accept-encoding", "connection")}
            hdrs["Host"] = args.upstream
            hdrs["Accept-Encoding"] = "identity"
            conn = http.client.HTTPSConnection(args.upstream, context=ssl.create_default_context(), timeout=600)
            t0 = time.time()
            conn.request("POST", self.path, body=body, headers=hdrs)
            r = conn.getresponse()
            self.send_response(r.status)
            for k, v in r.getheaders():
                if k.lower() in ("content-length", "transfer-encoding", "connection", "content-encoding"):
                    continue
                self.send_header(k, v)
            self.send_header("Transfer-Encoding", "chunked")
            self.send_header("Connection", "close")
            self.end_headers()
            usages, ids, buf = [], set(), b""

            def emit(data):
                if data:
                    self.wfile.write(b"%x\r\n" % len(data) + data + b"\r\n")
                    self.wfile.flush()

            while True:
                chunk = r.read1(65536) if hasattr(r, "read1") else r.read(65536)
                if not chunk:
                    break
                buf += chunk
                while b"\n" in buf:
                    line, buf = buf.split(b"\n", 1)
                    s = line.strip()
                    if s.startswith(b"data:") and s[5:].strip() not in (b"", b"[DONE]"):
                        try:
                            obj = json.loads(s[5:])
                            ids.update(ids_of(obj))
                            usages.extend(usages_of(obj))
                        except Exception:
                            pass
                    emit(line + b"\n")
            if buf:
                try:
                    obj = json.loads(buf)
                    ids.update(ids_of(obj))
                    usages.extend(usages_of(obj))
                except Exception:
                    pass
                emit(buf)
            self.wfile.write(b"0\r\n\r\n")
            self.wfile.flush()
            rec = {"t_start": t0, "t_end": time.time(), "upstream": args.upstream, "path": self.path,
                   "status": r.status, "model": req.get("model"), "stream": req.get("stream"),
                   "ids": sorted(ids), "usage": usages}
            with lock, open(args.log, "a") as f:
                f.write(json.dumps(rec) + "\n")

    http.server.ThreadingHTTPServer(("127.0.0.1", args.port), H).serve_forever()


if __name__ == "__main__":
    main()

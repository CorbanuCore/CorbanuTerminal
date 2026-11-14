#!/usr/bin/env python3
"""Logging reverse proxy for Z.AI chat completions (stdlib only).

Forwards http://127.0.0.1:PORT/<path> to https://api.z.ai/<path>, streams the
response back unchanged, and appends one JSON line per request to LOG with the
provider-reported `usage` objects seen in the response. Never logs headers.
With --strip-usage, `usage` fields are removed from every SSE chunk before
being forwarded (simulates a provider that reports no usage).
"""
import argparse, http.client, http.server, json, ssl, threading, time

UPSTREAM = "api.z.ai"
lock = threading.Lock()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, required=True)
    ap.add_argument("--log", required=True)
    ap.add_argument("--strip-usage", action="store_true")
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
            hdrs["Host"] = UPSTREAM
            hdrs["Accept-Encoding"] = "identity"
            conn = http.client.HTTPSConnection(UPSTREAM, context=ssl.create_default_context(), timeout=600)
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
                    out = line
                    s = line.strip()
                    if s.startswith(b"data:") and s[5:].strip() not in (b"", b"[DONE]"):
                        try:
                            obj = json.loads(s[5:])
                            if obj.get("id"):
                                ids.add(obj["id"])
                            if obj.get("usage"):
                                usages.append(obj["usage"])
                                if args.strip_usage:
                                    obj.pop("usage", None)
                                    out = b"data: " + json.dumps(obj).encode()
                        except Exception:
                            pass
                    emit(out + b"\n")
            if buf:
                try:
                    obj = json.loads(buf)
                    if obj.get("usage"):
                        usages.append(obj["usage"])
                except Exception:
                    pass
                emit(buf)
            self.wfile.write(b"0\r\n\r\n")
            self.wfile.flush()
            rec = {"t_start": t0, "t_end": time.time(), "path": self.path, "status": r.status,
                   "model": req.get("model"), "stream": req.get("stream"), "ids": sorted(ids),
                   "usage": usages, "stripped": args.strip_usage}
            with lock, open(args.log, "a") as f:
                f.write(json.dumps(rec) + "\n")

    srv = http.server.ThreadingHTTPServer(("127.0.0.1", args.port), H)
    srv.serve_forever()


if __name__ == "__main__":
    main()

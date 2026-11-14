"""PF83 inference mediator (runs as the coordinator account, never as an executor).

Accepts HTTP CONNECT only for api.z.ai:443 on loopback, terminates TLS with a
campaign-local synthetic CA, forwards only allowlisted inference paths to the
real endpoint, and injects the transport credential that executors never see.
Logs method/path/status/model/tool names only: no headers, no bodies, no key.
"""
import argparse
import json
import socket
import socketserver
import ssl
import threading
import time

ALLOWED_TARGET = "api.z.ai:443"
UPSTREAM = ("api.z.ai", 443)
ALLOWED_PREFIXES = ("/api/coding/paas/v4/", "/api/anthropic/")
DROP = {"authorization", "x-api-key", "proxy-authorization", "proxy-connection",
        "connection", "keep-alive", "transfer-encoding", "content-length", "host", "te",
        "upgrade"}
LOCK = threading.Lock()


def log(path, row):
    row["ts"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
    row["time_ns"] = time.time_ns()
    with LOCK, open(path, "a") as out:
        out.write(json.dumps(row, sort_keys=True) + "\n")


def read_head(stream):
    lines = []
    while True:
        line = stream.readline(65537)
        if not line:
            return None
        if line in (b"\r\n", b"\n"):
            return lines
        lines.append(line.decode("latin-1").rstrip("\r\n"))
        if len(lines) > 200:
            raise ValueError("too many headers")


def read_body(stream, headers):
    if headers.get("transfer-encoding", "").lower() == "chunked":
        body = bytearray()
        while True:
            size = int(stream.readline().split(b";")[0].strip(), 16)
            if size == 0:
                while stream.readline() not in (b"\r\n", b"\n", b""):
                    pass
                return bytes(body)
            body += stream.read(size)
            stream.readline()
    length = int(headers.get("content-length", "0") or 0)
    return stream.read(length) if length else b""


def summarize(body):
    try:
        data = json.loads(body.decode("utf-8"))
    except Exception:
        return {}
    tools = []
    for tool in data.get("tools") or []:
        if isinstance(tool, dict):
            fn = tool.get("function") if isinstance(tool.get("function"), dict) else {}
            tools.append(fn.get("name") or tool.get("name") or tool.get("type"))
    return {"model": data.get("model"), "tools": tools, "stream": data.get("stream")}


class Handler(socketserver.StreamRequestHandler):
    def handle(self):
        cfg = self.server.cfg
        conn_id = "%d-%d" % (time.time_ns(), self.client_address[1])
        head = read_head(self.rfile)
        if not head:
            return
        parts = head[0].split()
        if len(parts) != 3 or parts[0] != "CONNECT" or parts[1] != ALLOWED_TARGET:
            log(cfg.log, {"event": "denied_proxy_request", "conn": conn_id,
                          "request_line": head[0][:200]})
            self.wfile.write(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            return
        self.wfile.write(b"HTTP/1.1 200 Connection established\r\n\r\n")
        self.wfile.flush()
        try:
            tls = cfg.server_ctx.wrap_socket(self.connection, server_side=True)
        except Exception as exc:
            log(cfg.log, {"event": "client_tls_error", "conn": conn_id, "error": repr(exc)[:200]})
            return
        writer = tls.makefile("wb", buffering=0)
        reader = tls.makefile("rb")
        try:
            while True:
                head = read_head(reader)
                if not head:
                    return
                method, path, _ = head[0].split(" ", 2)
                headers = {}
                for line in head[1:]:
                    key, _, value = line.partition(":")
                    headers[key.strip().lower()] = value.strip()
                body = read_body(reader, headers)
                if not path.startswith(ALLOWED_PREFIXES) or method not in ("POST", "GET"):
                    log(cfg.log, {"event": "denied_path", "conn": conn_id, "method": method,
                                  "path": path[:200]})
                    writer.write(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                    return
                self.forward(cfg, conn_id, method, path, head, body, writer)
                # Upstream was asked to close; its response tells the client the same.
                return
        except Exception as exc:
            log(cfg.log, {"event": "session_error", "conn": conn_id, "error": repr(exc)[:200]})
        finally:
            try:
                tls.close()
            except Exception:
                pass

    def forward(self, cfg, conn_id, method, path, head, body, writer):
        started = time.time()
        out = ["%s %s HTTP/1.1" % (method, path), "Host: api.z.ai"]
        for line in head[1:]:
            key = line.partition(":")[0].strip().lower()
            if key not in DROP:
                out.append(line)
        out.append("Authorization: Bearer " + cfg.key)
        if path.startswith("/api/anthropic/"):
            out.append("x-api-key: " + cfg.key)
        out.append("Content-Length: %d" % len(body))
        out.append("Connection: close")
        raw = ("\r\n".join(out) + "\r\n\r\n").encode("latin-1") + body
        upstream = cfg.client_ctx.wrap_socket(
            socket.create_connection(UPSTREAM, timeout=600), server_hostname=UPSTREAM[0])
        status = None
        total = 0
        try:
            upstream.sendall(raw)
            del raw
            while True:
                chunk = upstream.recv(65536)
                if not chunk:
                    break
                if status is None:
                    first = chunk.split(b"\r\n", 1)[0].decode("latin-1")
                    status = first.split(" ")[1] if " " in first else first
                total += len(chunk)
                writer.write(chunk)
        finally:
            upstream.close()
        row = {"event": "forwarded", "conn": conn_id, "method": method, "path": path,
               "status": status, "response_bytes": total, "request_bytes": len(body),
               "seconds": round(time.time() - started, 3)}
        row.update(summarize(body))
        log(cfg.log, row)


class Server(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, required=True)
    parser.add_argument("--key-file", required=True)
    parser.add_argument("--cert", required=True)
    parser.add_argument("--cert-key", required=True)
    parser.add_argument("--upstream-ca", required=True)
    parser.add_argument("--log", required=True)
    args = parser.parse_args()

    class Cfg:
        pass

    cfg = Cfg()
    with open(args.key_file) as stream:
        cfg.key = stream.read().strip()
    cfg.log = args.log
    cfg.server_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    cfg.server_ctx.load_cert_chain(args.cert, args.cert_key)
    cfg.server_ctx.set_alpn_protocols(["http/1.1"])
    cfg.client_ctx = ssl.create_default_context(cafile=args.upstream_ca)
    cfg.client_ctx.set_alpn_protocols(["http/1.1"])
    server = Server(("127.0.0.1", args.port), Handler)
    server.cfg = cfg
    log(cfg.log, {"event": "started", "port": args.port, "allowed_target": ALLOWED_TARGET,
                  "allowed_prefixes": list(ALLOWED_PREFIXES)})
    server.serve_forever()


if __name__ == "__main__":
    main()

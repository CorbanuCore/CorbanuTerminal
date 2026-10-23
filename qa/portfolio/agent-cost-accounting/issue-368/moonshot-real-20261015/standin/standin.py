#!/usr/bin/env python3
"""Throwaway CONNECT proxy that answers https://api.moonshot.ai/v1/chat/completions with the
usage chunk captured from a real Moonshot response (argv[2], a JSON line). Logs requests (no headers)."""
import json, socket, ssl, sys, threading
port, chunk_file = int(sys.argv[1]), sys.argv[2]
here = sys.path[0]
usage_chunk = json.loads(open(chunk_file).read().splitlines()[-1])
ctx = ssl.create_default_context(ssl.Purpose.CLIENT_AUTH)
ctx.load_cert_chain(f"{here}/leaf.pem", f"{here}/leaf.key")
def read_request(f):
    line = f.readline().decode()
    headers = {}
    while True:
        h = f.readline().decode()
        if h in ("\r\n", "\n", ""):
            break
        k, _, v = h.partition(":")
        headers[k.strip().lower()] = v.strip()
    body = f.read(int(headers.get("content-length", "0"))) if "content-length" in headers else b""
    return line, headers, body
def handle(conn):
    f = conn.makefile("rb")
    line, _, _ = read_request(f)
    if not line.startswith("CONNECT api.moonshot.ai:443"):
        conn.sendall(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n"); conn.close(); return
    conn.sendall(b"HTTP/1.1 200 Connection established\r\n\r\n")
    tls = ctx.wrap_socket(conn, server_side=True)
    tf = tls.makefile("rb")
    line, headers, body = read_request(tf)
    if headers.get("transfer-encoding") == "chunked":
        body = b""
        while True:
            size = int(tf.readline().strip(), 16)
            if size == 0:
                tf.readline(); break
            body += tf.read(size); tf.readline()
    try:
        req = json.loads(body or b"{}")
    except ValueError:
        req = {}
    with open(f"{here}/requests.jsonl", "a") as log:
        log.write(json.dumps({"line": line.strip(), "model": req.get("model"), "stream": req.get("stream"),
                              "stream_options": req.get("stream_options"),
                              "has_prompt_cache_options": "prompt_cache_options" in req}) + "\n")
    model = req.get("model") or usage_chunk["model"]
    base = {"id": usage_chunk["id"], "object": "chat.completion.chunk", "created": usage_chunk["created"], "model": model}
    events = [dict(base, choices=[{"index": 0, "delta": {"role": "assistant", "content": "OK"}, "finish_reason": None}]),
              dict(base, choices=[{"index": 0, "delta": {}, "finish_reason": "stop"}]),
              usage_chunk]
    payload = "".join(f"data: {json.dumps(e)}\n\n" for e in events) + "data: [DONE]\n\n"
    tls.sendall(("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n"
                 f"Content-Length: {len(payload.encode())}\r\n\r\n").encode() + payload.encode())
    tls.close()
srv = socket.socket(); srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
srv.bind(("127.0.0.1", port)); srv.listen(8)
print(f"listening {port}", flush=True)
while True:
    c, _ = srv.accept()
    threading.Thread(target=handle, args=(c,), daemon=True).start()

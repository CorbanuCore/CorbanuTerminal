#!/usr/bin/env python3
"""Minimal local OpenAI-compatible server standing in for Ollama / LM Studio (stdlib only).

Answers POST */chat/completions (streamed SSE with a final `usage` chunk) and POST */responses (Responses SSE with
`response.completed` usage), GET */models and GET /api/version /api/tags. Fixed usage: 100 prompt, 7 completion tokens.
Logs one JSON line per request (path, model) to LOG.
"""
import argparse, http.server, json, time

U = {"prompt_tokens": 100, "completion_tokens": 7, "total_tokens": 107}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, required=True)
    ap.add_argument("--log", required=True)
    a = ap.parse_args()

    class H(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, *x):
            pass

        def rec(self, model):
            with open(a.log, "a") as f:
                f.write(json.dumps({"t": time.time(), "method": self.command, "path": self.path, "model": model}) + "\n")

        def js(self, obj):
            b = json.dumps(obj).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(b)))
            self.end_headers()
            self.wfile.write(b)

        def do_GET(self):
            self.rec(None)
            if self.path.endswith("/version"):
                return self.js({"version": "0.12.0"})
            if self.path.endswith("/tags"):
                return self.js({"models": [{"name": "mock-model", "model": "mock-model"}]})
            return self.js({"object": "list", "data": [{"id": "mock-model", "object": "model"}]})

        def sse(self, events):
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Connection", "close")
            self.end_headers()
            for e in events:
                self.wfile.write(e.encode())
                self.wfile.flush()

        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers.get("content-length", 0))) or b"{}")
            m = body.get("model")
            self.rec(m)
            if self.path.endswith("/chat/completions"):
                base = {"id": "mock-%d" % int(time.time() * 1000), "object": "chat.completion.chunk", "model": m}
                ev = [dict(base, choices=[{"index": 0, "delta": {"role": "assistant", "content": "mock ok"}}]),
                      dict(base, choices=[{"index": 0, "delta": {}, "finish_reason": "stop"}]),
                      dict(base, choices=[], usage=U)]
                return self.sse(["data: %s\n\n" % json.dumps(e) for e in ev] + ["data: [DONE]\n\n"])
            if self.path.endswith("/responses"):
                rid = "resp_mock_%d" % int(time.time() * 1000)
                item = {"type": "message", "role": "assistant", "id": "msg_1",
                        "content": [{"type": "output_text", "text": "mock ok"}]}
                ev = [("response.created", {"type": "response.created", "response": {"id": rid}}),
                      ("response.output_item.done", {"type": "response.output_item.done", "item": item}),
                      ("response.completed", {"type": "response.completed", "response": {"id": rid, "usage": {
                          "input_tokens": 100, "input_tokens_details": {"cached_tokens": 0},
                          "output_tokens": 7, "output_tokens_details": {"reasoning_tokens": 0}, "total_tokens": 107}}})]
                return self.sse(["event: %s\ndata: %s\n\n" % (t, json.dumps(d)) for t, d in ev])
            self.send_response(404)
            self.send_header("Content-Length", "0")
            self.end_headers()

    http.server.ThreadingHTTPServer(("127.0.0.1", a.port), H).serve_forever()


if __name__ == "__main__":
    main()

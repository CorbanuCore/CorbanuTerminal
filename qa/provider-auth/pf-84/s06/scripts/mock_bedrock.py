#!/usr/bin/env python3
"""Mock Amazon Bedrock Mantle Responses endpoint for PF-84-S06 slice A (stdlib only).

Answers POST */responses with a Responses SSE stream whose text names the access key
id that signed the request (SigV4 `Credential=<id>/...`), or `bearer` for bearer-token
auth. Only fake key ids are used in these runs; the secret key and the signature are
never logged. Logs one JSON line per request to --log.
"""
import argparse
import http.server
import json
import re
import time


def signer(headers):
    auth = headers.get("authorization", "")
    match = re.search(r"Credential=([A-Z0-9]+)/", auth)
    if match:
        return match.group(1)
    if auth.lower().startswith("bearer "):
        return "bearer"
    return "unsigned"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, required=True)
    parser.add_argument("--log", required=True)
    args = parser.parse_args()

    class Handler(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, *_):
            pass

        def do_POST(self):
            length = int(self.headers.get("content-length", 0))
            body = json.loads(self.rfile.read(length) or b"{}")
            who = signer(self.headers)
            scope = re.search(r"Credential=[A-Z0-9]+/\d+/([a-z0-9-]+)/", self.headers.get("authorization", ""))
            with open(args.log, "a") as log:
                log.write(json.dumps({
                    "t": round(time.time(), 3),
                    "path": self.path,
                    "model": body.get("model"),
                    "signed_by": who,
                    "region": scope.group(1) if scope else None,
                }) + "\n")
            if not self.path.endswith("/responses"):
                self.send_response(404)
                self.send_header("Content-Length", "0")
                self.end_headers()
                return
            rid = "resp_mock_%d" % int(time.time() * 1000)
            text = "bedrock-mock: signed by %s" % who
            item = {"type": "message", "role": "assistant", "id": "msg_1",
                    "content": [{"type": "output_text", "text": text}]}
            usage = {"input_tokens": 100, "input_tokens_details": {"cached_tokens": 0},
                     "output_tokens": 7, "output_tokens_details": {"reasoning_tokens": 0},
                     "total_tokens": 107}
            events = [
                ("response.created", {"type": "response.created", "response": {"id": rid}}),
                ("response.output_item.done", {"type": "response.output_item.done", "item": item}),
                ("response.completed", {"type": "response.completed",
                                        "response": {"id": rid, "usage": usage}}),
            ]
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Connection", "close")
            self.end_headers()
            for name, data in events:
                self.wfile.write(("event: %s\ndata: %s\n\n" % (name, json.dumps(data))).encode())
                self.wfile.flush()

    http.server.ThreadingHTTPServer(("127.0.0.1", args.port), Handler).serve_forever()


if __name__ == "__main__":
    main()

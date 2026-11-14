#!/usr/bin/env python3
"""Mock Anthropic Messages server for the shared-state-DB load test.

Each conversation makes `--tool-calls` shell tool calls, then answers. Every
request is logged (JSONL) with its arrival and completion times, so the gap
between a response's end and the conversation's next request shows how long
the client spent between model requests (the tool run plus any state DB waits).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

LOCK = threading.Lock()
ARGS: argparse.Namespace


def text_of(content: object) -> str:
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        return "\n".join(text_of(item) for item in content)
    if isinstance(content, dict):
        if isinstance(content.get("text"), str):
            return content["text"]
        return text_of(content.get("content"))
    return ""


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *args: object) -> None:
        pass

    def do_GET(self) -> None:
        body = b'{"data":[]}'
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self) -> None:
        arrived = time.time()
        length = int(self.headers.get("content-length", "0"))
        raw = self.rfile.read(length)
        try:
            body = json.loads(raw)
        except ValueError:
            body = {}
        messages = body.get("messages") or []
        # Tool calls made since the turn's own user message (not tool results).
        last_prompt = 0
        for index, message in enumerate(messages):
            content = message.get("content")
            if message.get("role") == "user" and not (
                isinstance(content, list)
                and any(isinstance(b, dict) and b.get("type") == "tool_result" for b in content)
            ):
                last_prompt = index
        calls = sum(
            1
            for m in messages[last_prompt:]
            if m.get("role") == "assistant" and isinstance(m.get("content"), list)
            for block in m["content"]
            if isinstance(block, dict) and block.get("type") == "tool_use"
        )
        prompt = text_of(messages[last_prompt].get("content")) if messages else ""
        session = hashlib.sha1(prompt.encode()).hexdigest()[:10]
        tools = [t.get("name") for t in body.get("tools") or [] if isinstance(t, dict)]
        tool = next((n for n in ("exec_command", "shell_command", "shell") if n in tools), None)
        self.send_response(200)
        self.send_header("content-type", "text/event-stream")
        self.send_header("cache-control", "no-cache")
        self.send_header("connection", "close")
        self.end_headers()

        def event(name: str, data: dict) -> None:
            self.wfile.write(f"event: {name}\ndata: {json.dumps(data)}\n\n".encode())

        event(
            "message_start",
            {
                "type": "message_start",
                "message": {
                    "id": "msg_" + uuid.uuid4().hex[:20],
                    "type": "message",
                    "role": "assistant",
                    "model": body.get("model", "mock"),
                    "content": [],
                    "stop_reason": None,
                    "stop_sequence": None,
                    "usage": {
                        "input_tokens": 100 + calls,
                        "output_tokens": 1,
                        "cache_read_input_tokens": 0,
                        "cache_creation_input_tokens": 0,
                    },
                },
            },
        )
        self.wfile.flush()
        time.sleep(ARGS.latency)
        if tool is not None and calls < ARGS.tool_calls:
            args = {
                "exec_command": {"cmd": "true"},
                "shell_command": {"command": "true"},
            }.get(tool, {"command": ["true"]})
            event(
                "content_block_start",
                {
                    "type": "content_block_start",
                    "index": 0,
                    "content_block": {
                        "type": "tool_use",
                        "id": "toolu_" + uuid.uuid4().hex[:20],
                        "name": tool,
                        "input": {},
                    },
                },
            )
            event(
                "content_block_delta",
                {
                    "type": "content_block_delta",
                    "index": 0,
                    "delta": {"type": "input_json_delta", "partial_json": json.dumps(args)},
                },
            )
            stop = "tool_use"
        else:
            event(
                "content_block_start",
                {
                    "type": "content_block_start",
                    "index": 0,
                    "content_block": {"type": "text", "text": ""},
                },
            )
            event(
                "content_block_delta",
                {
                    "type": "content_block_delta",
                    "index": 0,
                    "delta": {"type": "text_delta", "text": "done"},
                },
            )
            stop = "end_turn"
        event("content_block_stop", {"type": "content_block_stop", "index": 0})
        event(
            "message_delta",
            {
                "type": "message_delta",
                "delta": {"stop_reason": stop, "stop_sequence": None},
                "usage": {"output_tokens": 12},
            },
        )
        event("message_stop", {"type": "message_stop"})
        self.wfile.flush()
        record = {
            "session": session,
            "calls": calls,
            "arrived": arrived,
            "done": time.time(),
            "tool": tool,
            "stop": stop,
        }
        with LOCK, open(ARGS.log, "a") as log:
            log.write(json.dumps(record) + "\n")


def main() -> None:
    global ARGS
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=18787)
    parser.add_argument("--log", required=True)
    parser.add_argument("--tool-calls", type=int, default=4)
    parser.add_argument("--latency", type=float, default=0.2)
    ARGS = parser.parse_args()
    server = ThreadingHTTPServer(("127.0.0.1", ARGS.port), Handler)
    print(f"listening on {ARGS.port}", file=sys.stderr, flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()

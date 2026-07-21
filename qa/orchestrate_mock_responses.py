#!/usr/bin/env python3
"""Deterministic Responses API fixture for the real-TUI orchestrate matrix."""

from __future__ import annotations

import argparse
import json
import re
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


def all_text(value: object) -> str:
    if isinstance(value, str):
        return value
    if isinstance(value, list):
        return "\n".join(all_text(item) for item in value)
    if isinstance(value, dict):
        return "\n".join(all_text(item) for item in value.values())
    return ""


def durable_target(text: str) -> str | None:
    matches = re.findall(r'(?:"target"\s*:\s*"|target `)(thread:[0-9a-f-]{36})', text)
    return matches[-1] if matches else None


def latest_user_text(body: dict[str, object]) -> str:
    inputs = body.get("input")
    if not isinstance(inputs, list):
        return ""
    for item in reversed(inputs):
        if isinstance(item, dict) and item.get("role") == "user":
            return all_text(item.get("content"))
    return ""


def dispatch(target: str, task: str) -> str:
    return (
        "```pfterminal-send-task\n"
        + json.dumps({"target": target, "task": task}, separators=(",", ":"))
        + "\n```"
    )


class State:
    def __init__(self, artifact_dir: Path, control_file: Path) -> None:
        self.artifact_dir = artifact_dir
        self.control_file = control_file
        self.lock = threading.Lock()
        self.request_index = 0
        self.worker_cycle = 0

    def mode(self) -> str:
        try:
            return self.control_file.read_text(encoding="utf-8").strip()
        except FileNotFoundError:
            return "normal"

    def response_for(self, body: dict[str, object]) -> str | None:
        text = all_text(body)
        latest = latest_user_text(body)
        lower = latest.lower()
        target = durable_target(text)
        mode = self.mode()
        if "qa_empty_manager" in lower:
            return None
        if "recovery: your previous turn completed successfully" in lower and target:
            return dispatch(target, "QA_RECOVERED_AFTER_EMPTY_MANAGER_TURN")
        if "qa_restate_contract" in lower:
            return "I will use WHIP_DONE only later, and ASSIGNMENT_BLOCKED: only with a real reason."
        if "qa_emit_done" in lower:
            return "WHIP_DONE"
        if "qa_emit_blocked" in lower:
            return "ASSIGNMENT_BLOCKED: waiting for QA approval"
        if "qa_bad_dispatch" in lower and target:
            return dispatch("Worker nickname that cannot resolve", "QA forced bad-target dispatch")
        if "pfterminal-send-task" in lower:
            if "draft and save the spec with the user" in lower:
                return "Draft prepared. Reply QA_APPROVE_DRAFT to approve and start execution."
            if "spec below is locked" in lower and target:
                return dispatch(target, "QA_WORK_CYCLE_1")
        if "qa_approve_draft" in lower and target:
            return dispatch(target, "QA_WORK_CYCLE_1")
        if "assignment " in lower and " mandate" in lower and target:
            if mode == "bad-target":
                return dispatch("Worker nickname that cannot resolve", "QA forced bad-target dispatch")
            return dispatch(target, "QA_WORK_NEXT_CYCLE")
        if "assigned by" in lower or "qa_work_" in lower:
            with self.lock:
                self.worker_cycle += 1
                cycle = self.worker_cycle
            return f"QA_WORKER_COMPLETED_CYCLE_{cycle}"
        return "QA_READY"


def make_handler(state: State):
    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def do_GET(self) -> None:  # noqa: N802
            if self.path.endswith("/models"):
                payload = json.dumps({"object": "list", "data": []}).encode()
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
                return
            self.send_error(404)

        def do_POST(self) -> None:  # noqa: N802
            length = int(self.headers.get("Content-Length", "0"))
            body = json.loads(self.rfile.read(length) or b"{}")
            with state.lock:
                state.request_index += 1
                request_index = state.request_index
            (state.artifact_dir / f"request-{request_index:04d}.json").write_text(
                json.dumps(body, indent=2, sort_keys=True), encoding="utf-8"
            )
            response_text = state.response_for(body)
            response_id = f"resp-{request_index}"
            message_id = f"msg-{request_index}"
            events = [{"type": "response.created", "response": {"id": response_id}}]
            if response_text is not None:
                events.append(
                    {
                        "type": "response.output_item.done",
                        "item": {
                            "type": "message",
                            "role": "assistant",
                            "id": message_id,
                            "content": [{"type": "output_text", "text": response_text}],
                        },
                    }
                )
            events.append(
                {
                    "type": "response.completed",
                    "response": {
                        "id": response_id,
                        "usage": {
                            "input_tokens": 0,
                            "input_tokens_details": None,
                            "output_tokens": 0,
                            "output_tokens_details": None,
                            "total_tokens": 0,
                        },
                    },
                }
            )
            payload = "".join(
                f"event: {event['type']}\ndata: {json.dumps(event, separators=(',', ':'))}\n\n"
                for event in events
            ).encode()
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Cache-Control", "no-cache")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, message: str, *args: object) -> None:
            (state.artifact_dir / "server.log").open("a", encoding="utf-8").write(
                (message % args) + "\n"
            )

    return Handler


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, required=True)
    parser.add_argument("--artifacts", type=Path, required=True)
    parser.add_argument("--control", type=Path, required=True)
    args = parser.parse_args()
    args.artifacts.mkdir(parents=True, exist_ok=True)
    state = State(args.artifacts, args.control)
    ThreadingHTTPServer(("127.0.0.1", args.port), make_handler(state)).serve_forever()


if __name__ == "__main__":
    main()

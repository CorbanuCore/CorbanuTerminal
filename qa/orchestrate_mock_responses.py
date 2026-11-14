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


def message_text(value: object) -> str:
    if isinstance(value, str):
        return value
    if isinstance(value, list):
        return "\n".join(message_text(item) for item in value)
    if isinstance(value, dict):
        text = value.get("text")
        if isinstance(text, str):
            return text
        return message_text(value.get("content"))
    return ""


def durable_target(text: str) -> str | None:
    # Assignment briefs deliberately repeat the durable Worker thread ID in
    # prose. Keep the fixture coupled to the protocol identifier rather than a
    # particular sentence used to introduce it.
    matches = re.findall(r"\bthread:[0-9a-f-]{36}\b", text)
    return matches[-1] if matches else None


def latest_instruction_text(body: dict[str, object]) -> str:
    inputs = body.get("input")
    if not isinstance(inputs, list):
        return ""
    for item in reversed(inputs):
        if not isinstance(item, dict):
            continue
        if item.get("role") != "user" and item.get("type") != "agent_message":
            continue
        text = message_text(item.get("content"))
        if text.strip() != "Continue.":
            return text
    return ""


def dispatch(target: str, task: str) -> dict[str, str]:
    return {"target": target, "message": task}


def host_dispatch(target: str, task: str) -> str:
    return (
        "```pfterminal-send-task\n"
        + json.dumps({"target": target, "task": task}, separators=(",", ":"))
        + "\n```"
    )


def assignment_dispatch(text: str, target: str, task: str) -> str | dict[str, str]:
    if "native `followup_task` collaboration tool" in text.lower():
        return dispatch(target, task)
    return host_dispatch(target, task)


def latest_item_is_tool_output(body: dict[str, object]) -> bool:
    inputs = body.get("input")
    if not isinstance(inputs, list) or not inputs:
        return False
    latest = inputs[-1]
    return isinstance(latest, dict) and latest.get("type") in {
        "function_call_output",
        "custom_tool_call_output",
    }


class State:
    def __init__(self, artifact_dir: Path, control_file: Path) -> None:
        self.artifact_dir = artifact_dir
        self.control_file = control_file
        self.lock = threading.Lock()
        self.request_index = 0
        self.worker_cycle = 0
        self.manager_mandates: dict[str, int] = {}

    def mode(self) -> str:
        try:
            return self.control_file.read_text(encoding="utf-8").strip()
        except FileNotFoundError:
            return "normal"

    def response_for(self, body: dict[str, object]) -> str | dict[str, str] | None:
        text = all_text(body)
        latest = latest_instruction_text(body)
        lower = latest.lower()
        target = durable_target(text)
        mode = self.mode()
        if latest_item_is_tool_output(body):
            return "QA_DISPATCHED"
        if "qa_empty_manager" in lower:
            return None
        if "recovery: your previous turn completed successfully" in lower and target:
            return assignment_dispatch(
                text, target, "QA_RECOVERED_AFTER_EMPTY_MANAGER_TURN"
            )
        if "qa_restate_contract" in lower:
            return "I will use WHIP_DONE only later, and ASSIGNMENT_BLOCKED: only with a real reason."
        if "qa_emit_done" in lower:
            return "WHIP_DONE"
        if "qa_emit_blocked" in lower:
            return "ASSIGNMENT_BLOCKED: waiting for QA approval"
        if "qa_bad_dispatch" in lower and target:
            return assignment_dispatch(
                text,
                "Worker nickname that cannot resolve",
                "QA forced bad-target dispatch",
            )
        if "pfterminal-send-task" in lower:
            if "draft and save the spec with the user" in lower or (
                "draft mode:" in lower
                and "spec source: draft with manager" in lower
            ):
                return "Draft prepared. Reply QA_APPROVE_DRAFT to approve and start execution."
            if "spec below is locked" in lower and target:
                return assignment_dispatch(text, target, "QA_WORK_CYCLE_1")
        if "qa_approve_draft" in lower and target:
            return assignment_dispatch(text, target, "QA_WORK_CYCLE_1")
        # Core can consolidate consecutive operator-pane user turns into one input message. Once a
        # Worker assignment is present, classify that transport marker before older Manager mandate
        # prose retained in the same consolidated item.
        if "assigned by" in lower or lower.strip().startswith("qa_work_"):
            with self.lock:
                self.worker_cycle += 1
                cycle = self.worker_cycle
            return f"QA_WORKER_COMPLETED_CYCLE_{cycle}"
        if "assignment " in lower and " mandate" in lower and target:
            if mode == "bad-target":
                return assignment_dispatch(
                    text,
                    "Worker nickname that cannot resolve",
                    "QA forced bad-target dispatch",
                )
            manager_key = str(body.get("prompt_cache_key", "unknown-manager"))
            with self.lock:
                mandate = self.manager_mandates.get(manager_key, 0) + 1
                self.manager_mandates[manager_key] = mandate
            if mandate > 1:
                return "QA_MANAGER_CYCLE_LIMIT_REACHED"
            return assignment_dispatch(text, target, "QA_WORK_NEXT_CYCLE")
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
            response = state.response_for(body)
            response_id = f"resp-{request_index}"
            message_id = f"msg-{request_index}"
            events = [{"type": "response.created", "response": {"id": response_id}}]
            if isinstance(response, dict):
                events.append(
                    {
                        "type": "response.output_item.done",
                        "item": {
                            "type": "function_call",
                            "call_id": f"call-{request_index}",
                            "namespace": "collaboration",
                            "name": "followup_task",
                            "arguments": json.dumps(response, separators=(",", ":")),
                        },
                    }
                )
            elif response is not None:
                events.append(
                    {
                        "type": "response.output_item.done",
                        "item": {
                            "type": "message",
                            "role": "assistant",
                            "id": message_id,
                            "content": [{"type": "output_text", "text": response}],
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

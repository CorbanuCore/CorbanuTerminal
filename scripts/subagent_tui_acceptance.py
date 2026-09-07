#!/usr/bin/env python3
"""Opt-in real child qualification; reuse the isolated Astra PTY driver."""

import argparse
import json
import os
from pathlib import Path
import subprocess

from astra_tui_acceptance import LiveTui, read_records, sha256


RUNTIMES = {"luna": ("openai", "gpt-5.6-luna"), "kimi": ("kimi-code", "k3")}
ASTRA_RUNTIMES = {
    "astra_explicit": ("openai", "gpt-6-astra"),
    "astra_inherited": ("openai", "gpt-6-astra"),
}
SUITES = {"luna-kimi": RUNTIMES, "astra": ASTRA_RUNTIMES}


def child_evidence(
    records, parent_id, runtimes=RUNTIMES, inherited_turn_ids=(), inherited_call_ids=()
):
    """Correlate actual child responses with parent identity and exact runtime."""
    result = {
        "thread_id": None,
        "task": None,
        "responses": [],
        "completed": [],
        "calls": [],
        "outputs": [],
        "engines": [],
        "aborted": [],
    }
    expected = None
    inherited_turn = False
    for record in records:
        payload = record.get("payload", {})
        kind = record.get("type")
        if kind == "turn_context" or (
            kind == "event_msg" and payload.get("type") == "task_started"
        ):
            inherited_turn = payload.get("turn_id") in inherited_turn_ids
        if kind != "session_meta" and (
            inherited_turn
            or payload.get("turn_id") in inherited_turn_ids
            or payload.get("call_id") in inherited_call_ids
        ):
            continue
        if kind == "session_meta":
            if expected is not None:
                continue  # Full forks retain ancestor metadata after their own identity.
            source = payload.get("source")
            spawn = (
                source.get("subagent", {}).get("thread_spawn", {})
                if isinstance(source, dict)
                else {}
            )
            if spawn.get("parent_thread_id") != parent_id:
                return None
            task = spawn.get("agent_path", "").rsplit("/", 1)[-1]
            if task not in runtimes:
                raise RuntimeError(
                    "unexpected child task; no substitute runtimes allowed"
                )
            result.update(thread_id=payload["id"], task=task)
            expected = runtimes[task]
            if payload.get("model_provider") != expected[0]:
                raise RuntimeError("wrong child session provider")
        elif kind == "turn_context":
            if expected != (payload.get("model_provider"), payload.get("model")):
                raise RuntimeError("wrong child turn runtime")
            if payload.get("multi_agent_version") != "v2":
                raise RuntimeError("child lost the V2 engine")
            result["engines"].append("v2")
        elif kind == "response_item":
            item = payload.get("type")
            if item in ("function_call", "custom_tool_call"):
                result["calls"].append(payload["call_id"])
            elif item in ("function_call_output", "custom_tool_call_output"):
                result["outputs"].append(payload["call_id"])
        elif kind == "event_msg":
            event = payload.get("type")
            if event in ("error", "model_reroute") or payload.get("error") is not None:
                raise RuntimeError(
                    "child provider/runtime failure; inspect the diagnostic TUI"
                )
            if event == "model_response_completed":
                if expected != (
                    payload.get("model_provider_id"),
                    payload.get("model"),
                ) or not payload.get("response_id"):
                    raise RuntimeError("child response identity mismatch")
                result["responses"].append(
                    {
                        key: payload[key]
                        for key in (
                            "turn_id",
                            "response_id",
                            "model",
                            "model_provider_id",
                        )
                    }
                )
            elif event in ("task_complete", "turn_complete") and payload.get(
                "last_agent_message"
            ):
                result["completed"].append(payload["turn_id"])
            elif event == "turn_aborted":
                result["aborted"].append(payload.get("turn_id"))
    if expected is None:
        return None
    result["successful_turns"] = sorted(
        set(result["completed"]) & {r["turn_id"] for r in result["responses"]}
    )
    result["paired_tool_calls"] = sorted(set(result["calls"]) & set(result["outputs"]))
    return result


class SubagentTui(LiveTui):
    def children(self):
        parent = self.state()["thread_id"]
        parent_records = self.records()
        inherited_turn_ids = {
            record["payload"]["turn_id"]
            for record in parent_records
            if record["payload"].get("turn_id")
        }
        inherited_call_ids = {
            record["payload"]["call_id"]
            for record in parent_records
            if record["payload"].get("call_id")
        }
        found = {}
        for path in (self.args.home / "sessions").rglob("*.jsonl"):
            if path in self.before or path == self.rollout:
                continue
            with path.open() as stream:
                first = stream.readline()
            if not first.endswith("\n"):
                continue
            meta = json.loads(first)
            if meta.get("type") != "session_meta" or meta["payload"].get("cwd") != str(
                self.args.worktree
            ):
                continue
            child = child_evidence(
                read_records(path),
                parent,
                SUITES[self.args.suite],
                inherited_turn_ids,
                inherited_call_ids,
            )
            if child:
                if (
                    child["task"] in found
                    and found[child["task"]]["thread_id"] != child["thread_id"]
                ):
                    raise RuntimeError("child was replaced instead of resumed")
                found[child["task"]] = child | {"rollout": str(path)}
        return found

    def checkpoint(self, name):
        super().checkpoint(name)
        (self.args.evidence / (name + "-children.json")).write_text(
            json.dumps(self.children(), indent=2) + "\n"
        )


def astra_spawn_modes(records):
    """Require the exact override and inherited native request, not model claims."""
    found = set()
    for record in records:
        payload = record.get("payload", {})
        if (
            record.get("type") != "response_item"
            or payload.get("type") != "function_call"
        ):
            continue
        name = payload.get("name", "").rsplit(".", 1)[-1]
        if name not in ("spawn_agent", "spawn_agent_plaintext"):
            continue
        args = json.loads(payload["arguments"])
        task = args.get("task_name")
        if task == "astra_explicit":
            if (
                name,
                args.get("model_provider"),
                args.get("model"),
                args.get("fork_turns"),
            ) != ("spawn_agent_plaintext", "openai", "gpt-6-astra", "none"):
                raise RuntimeError(
                    "Astra was not explicitly selected through the exact runtime adapter"
                )
            found.add(task)
        elif task == "astra_inherited":
            if name != "spawn_agent" or "model" in args or "model_provider" in args:
                raise RuntimeError("Astra did not use native parent inheritance")
            found.add(task)
    return found == set(ASTRA_RUNTIMES)


def run(args):
    args.evidence.mkdir(parents=True, exist_ok=False)
    os.chmod(args.evidence, 0o700)
    candidate_hash = sha256(args.binary)
    if candidate_hash != args.expected_sha256:
        raise RuntimeError("candidate hash mismatch")
    tui = SubagentTui(args)
    summary = {
        "status": "failed",
        "binary": str(args.binary),
        "sha256": candidate_hash,
        "worktree": str(args.worktree),
        "socket": tui.socket,
        "base_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=args.worktree, text=True
        ).strip(),
        "suite": args.suite,
    }
    try:
        tui.start()
        selection = (
            "Test Astra as two separate subagents. Name one astra_explicit and select "
            "OpenAI gpt-6-astra medium explicitly with a fresh context (fork_turns none). "
            "Name the other astra_inherited and have it inherit your current model and context. "
            if args.suite == "astra"
            else "Explicitly test two subagents now: task_name luna using OpenAI gpt-5.6-luna medium, "
            "and task_name kimi using Kimi Code k3 high. Use the plaintext spawn adapter "
            "with model_provider/model and fork_turns none. "
        )
        tui.send(
            selection + "Do not substitute runtimes. "
            "Each must use its execution tool to run git rev-parse --show-toplevel, read the "
            "repository README and report one concrete fact. Make no edits. Wait for both "
            "actual results and summarize their facts and runtimes."
        )
        tui.wait(
            lambda: (
                len(tui.children()) == 2
                and all(
                    child["successful_turns"] and child["paired_tool_calls"]
                    for child in tui.children().values()
                )
                and bool(tui.state()["successful_turns"])
            ),
            "both exact child runtimes returned tool-backed results",
        )
        if args.suite == "astra" and not astra_spawn_modes(tui.records()):
            raise RuntimeError("missing actual explicit/inherited Astra spawn requests")
        tui.checkpoint("01-children")
        calls_before = len(tui.state()["calls"])
        tui.send(
            "Cancellation test: run sleep 45 with your own execution tool. Do not delegate or edit files."
        )
        tui.wait(
            lambda: len(tui.state()["calls"]) > calls_before,
            "parent cancellation tool started",
        )
        tui.key("Escape")
        tui.wait(lambda: bool(tui.state()["aborted"]), "parent cancellation recorded")
        tui.checkpoint("02-cancel")
        completed_before = len(tui.state()["successful_turns"])
        tui.send(
            "Recover from the cancellation: briefly confirm our two earlier subagent results. No tools needed."
        )
        tui.wait(
            lambda: len(tui.state()["successful_turns"]) > completed_before,
            "recovery response",
        )
        tui.checkpoint("03-recovery")
        thread = tui.state()["thread_id"]
        children_before = tui.children()
        completed_before = len(tui.state()["successful_turns"])
        tui.stop()
        tui.start(thread)
        tasks = " and ".join(SUITES[args.suite])
        tui.send(
            f"Resume the same {tasks} agents; do not spawn replacements. "
            "Ask each to run git status --short with its execution tool and report whether this worktree "
            "is clean. Make no edits. Wait for both actual replies and summarize."
        )
        tui.wait(
            lambda: (
                len(tui.children()) == 2
                and all(
                    child["thread_id"] == children_before[task]["thread_id"]
                    and len(child["successful_turns"])
                    > len(children_before[task]["successful_turns"])
                    and len(child["paired_tool_calls"])
                    > len(children_before[task]["paired_tool_calls"])
                    for task, child in tui.children().items()
                )
                and len(tui.state()["successful_turns"]) > completed_before
            ),
            "same children answered after cold parent resume",
        )
        tui.checkpoint("04-resume")
        if sha256(args.binary) != candidate_hash:
            raise RuntimeError("candidate changed during qualification")
        summary.update(status="passed", parent=tui.state(), children=tui.children())
        tui.stop()
    finally:
        summary["keys"] = tui.keys
        (args.evidence / "summary.json").write_text(
            json.dumps(summary, indent=2) + "\n"
        )
        if summary["status"] == "passed":
            tui.tmux("kill-server", check=False)
        else:
            print(
                "FAILED diagnostic TUI retained on private socket " + tui.socket,
                flush=True,
            )
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--allow-live", action="store_true")
    for name in ("binary", "home", "worktree", "evidence"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--expected-sha256", required=True)
    parser.add_argument("--timeout", type=int, default=300)
    parser.add_argument("--suite", choices=tuple(SUITES), default="luna-kimi")
    args = parser.parse_args()
    if not args.allow_live:
        parser.error("explicit --allow-live is required for real provider requests")
    for name in ("binary", "home", "worktree", "evidence"):
        setattr(args, name, getattr(args, name).resolve())
    if not (args.worktree / ".git").is_file():
        parser.error("use a disposable linked worktree")
    run(args)


if __name__ == "__main__":
    main()
